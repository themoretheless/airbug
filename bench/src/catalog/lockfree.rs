//! Concurrency & lock-free synchronization kernels (`#701–760`).
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{
    AtomicBool, AtomicU8, AtomicU64, AtomicUsize,
    Ordering::{AcqRel, Acquire, Relaxed, Release, SeqCst},
};

const NONE_IDX: u32 = u32::MAX;
const FLAG_VAL_TAKEN: u8 = 1;
const FLAG_UNLINKED: u8 = 2;

#[inline]
const fn pack_ptr(tag: u32, idx: u32) -> u64 {
    ((tag as u64) << 32) | (idx as u64)
}

#[inline]
const fn unpack_ptr(packed: u64) -> (u32, u32) {
    ((packed >> 32) as u32, packed as u32)
}

struct MsSlot<T> {
    next: AtomicU64,
    free_next: AtomicU64,
    /// Two-bit retirement gate: bit 0 = value moved out, bit 1 = unlinked from queue head.
    retire_flags: AtomicU8,
    val: UnsafeCell<MaybeUninit<T>>,
}

/// `#701` Safe, ABA-immune bounded-arena Michael–Scott MPMC lock-free queue.
///
/// Nodes are pre-allocated in a fixed-capacity arena addressed by 32-bit indices paired
/// with 32-bit monotonic generation tags. Free nodes live on a lock-free Treiber freelist;
/// retired sentinel nodes are returned to the freelist once both value extraction and
/// head-pointer unlinking have completed.
pub struct MpmcQueue<T> {
    head: AtomicU64,
    tail: AtomicU64,
    free_head: AtomicU64,
    slots: Box<[MsSlot<T>]>,
}

// SAFETY: Concurrent access to every slot field is serialized by atomic publication (`next` /
// `free_next` Release/Acquire pairs) and exclusive ownership after CAS winning (`T: Send`).
unsafe impl<T: Send> Send for MpmcQueue<T> {}
unsafe impl<T: Send> Sync for MpmcQueue<T> {}

impl<T> MpmcQueue<T> {
    /// Allocate a queue holding up to `capacity` live items simultaneously (`capacity >= 1`).
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(1);
        let total = cap + 1;
        let mut slots = Vec::with_capacity(total);
        for i in 0..total {
            let next_free = if i >= 1 && i + 1 < total {
                pack_ptr(0, (i + 1) as u32)
            } else {
                pack_ptr(0, NONE_IDX)
            };
            slots.push(MsSlot {
                next: AtomicU64::new(pack_ptr(0, NONE_IDX)),
                free_next: AtomicU64::new(next_free),
                retire_flags: AtomicU8::new(if i == 0 { FLAG_VAL_TAKEN } else { 0 }),
                val: UnsafeCell::new(MaybeUninit::uninit()),
            });
        }
        let dummy = pack_ptr(0, 0);
        let first_free = if total > 1 {
            pack_ptr(0, 1)
        } else {
            pack_ptr(0, NONE_IDX)
        };
        Self {
            head: AtomicU64::new(dummy),
            tail: AtomicU64::new(dummy),
            free_head: AtomicU64::new(first_free),
            slots: slots.into_boxed_slice(),
        }
    }

    fn alloc_node(&self, val: T) -> Result<(u32, u32), T> {
        loop {
            let fh = self.free_head.load(Acquire);
            let (tag, idx) = unpack_ptr(fh);
            if idx == NONE_IDX {
                return Err(val);
            }
            let next = self.slots[idx as usize].free_next.load(Acquire);
            let (_, next_idx) = unpack_ptr(next);
            let next_packed = pack_ptr(tag.wrapping_add(1), next_idx);
            if self
                .free_head
                .compare_exchange_weak(fh, next_packed, AcqRel, Acquire)
                .is_ok()
            {
                let slot = &self.slots[idx as usize];
                let new_tag = tag.wrapping_add(1);
                slot.retire_flags.store(0, Relaxed);
                slot.next.store(pack_ptr(new_tag, NONE_IDX), Release);
                // SAFETY: `idx` was uniquely popped from `free_head` and is not reachable anywhere.
                unsafe {
                    (*slot.val.get()).write(val);
                }
                return Ok((new_tag, idx));
            }
            std::hint::spin_loop();
        }
    }

    fn free_node(&self, idx: u32) {
        let slot = &self.slots[idx as usize];
        slot.retire_flags.store(0, Relaxed);
        loop {
            let fh = self.free_head.load(Acquire);
            let (tag, head_idx) = unpack_ptr(fh);
            slot.free_next.store(pack_ptr(tag, head_idx), Relaxed);
            let next_free = pack_ptr(tag.wrapping_add(1), idx);
            if self
                .free_head
                .compare_exchange_weak(fh, next_free, AcqRel, Acquire)
                .is_ok()
            {
                return;
            }
            std::hint::spin_loop();
        }
    }

    fn mark_retired(&self, idx: u32, flag: u8) {
        let prev = self.slots[idx as usize].retire_flags.fetch_or(flag, AcqRel);
        if (prev | flag) == (FLAG_VAL_TAKEN | FLAG_UNLINKED) {
            self.free_node(idx);
        }
    }

    /// Push `val` into the queue. Returns `Err(val)` if the bounded arena is currently full.
    pub fn push(&self, val: T) -> Result<(), T> {
        let (node_tag, node_idx) = self.alloc_node(val)?;
        let new_ptr = pack_ptr(node_tag, node_idx);
        loop {
            let tail = self.tail.load(Acquire);
            let (tail_tag, tail_idx) = unpack_ptr(tail);
            let next = self.slots[tail_idx as usize].next.load(Acquire);
            let (next_tag, next_idx) = unpack_ptr(next);
            if tail == self.tail.load(Acquire) {
                if next_idx == NONE_IDX {
                    if self.slots[tail_idx as usize]
                        .next
                        .compare_exchange(next, new_ptr, AcqRel, Acquire)
                        .is_ok()
                    {
                        let _ = self.tail.compare_exchange(
                            tail,
                            pack_ptr(tail_tag.wrapping_add(1), node_idx),
                            Release,
                            Relaxed,
                        );
                        return Ok(());
                    }
                } else {
                    let _ = self.tail.compare_exchange(
                        tail,
                        pack_ptr(tail_tag.wrapping_add(1), next_idx),
                        Release,
                        Relaxed,
                    );
                }
            }
            let _ = next_tag;
            std::hint::spin_loop();
        }
    }

    /// Pop the oldest item from the queue, or `None` if currently empty.
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Acquire);
            let (head_tag, head_idx) = unpack_ptr(head);
            let tail = self.tail.load(Acquire);
            let (tail_tag, tail_idx) = unpack_ptr(tail);
            let next = self.slots[head_idx as usize].next.load(Acquire);
            let (_, next_idx) = unpack_ptr(next);
            if head == self.head.load(Acquire) {
                if head_idx == tail_idx {
                    if next_idx == NONE_IDX {
                        return None;
                    }
                    let _ = self.tail.compare_exchange(
                        tail,
                        pack_ptr(tail_tag.wrapping_add(1), next_idx),
                        Release,
                        Relaxed,
                    );
                } else if next_idx != NONE_IDX {
                    let new_head = pack_ptr(head_tag.wrapping_add(1), next_idx);
                    if self
                        .head
                        .compare_exchange(head, new_head, SeqCst, Acquire)
                        .is_ok()
                    {
                        // SAFETY: Only this CAS winner extracts `slots[next_idx].val`. Setting
                        // `FLAG_VAL_TAKEN` allows future unlinkers to reclaim `next_idx`.
                        let value = unsafe {
                            (*self.slots[next_idx as usize].val.get()).assume_init_read()
                        };
                        self.mark_retired(next_idx, FLAG_VAL_TAKEN);
                        self.mark_retired(head_idx, FLAG_UNLINKED);
                        return Some(value);
                    }
                }
            }
            std::hint::spin_loop();
        }
    }
}

impl<T> Drop for MpmcQueue<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// Lock-free index-tagged Treiber stack over a bounded preallocated node arena.
pub struct TreiberStack<T> {
    top: AtomicU64,
    free_head: AtomicU64,
    next: Box<[AtomicU64]>,
    vals: Box<[UnsafeCell<MaybeUninit<T>>]>,
}

// SAFETY: Synchronized via AcqRel CAS pairs on `top` and `free_head`.
unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> TreiberStack<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(1);
        let mut next = Vec::with_capacity(cap);
        let mut vals = Vec::with_capacity(cap);
        for i in 0..cap {
            let link = if i + 1 < cap {
                pack_ptr(0, (i + 1) as u32)
            } else {
                pack_ptr(0, NONE_IDX)
            };
            next.push(AtomicU64::new(link));
            vals.push(UnsafeCell::new(MaybeUninit::uninit()));
        }
        Self {
            top: AtomicU64::new(pack_ptr(0, NONE_IDX)),
            free_head: AtomicU64::new(pack_ptr(0, 0)),
            next: next.into_boxed_slice(),
            vals: vals.into_boxed_slice(),
        }
    }

    pub fn push(&self, val: T) -> Result<(), T> {
        let idx = loop {
            let fh = self.free_head.load(Acquire);
            let (tag, idx) = unpack_ptr(fh);
            if idx == NONE_IDX {
                return Err(val);
            }
            let (_, nx) = unpack_ptr(self.next[idx as usize].load(Acquire));
            if self
                .free_head
                .compare_exchange_weak(fh, pack_ptr(tag.wrapping_add(1), nx), AcqRel, Acquire)
                .is_ok()
            {
                break idx;
            }
            std::hint::spin_loop();
        };
        // SAFETY: `idx` was uniquely popped from `free_head`.
        unsafe { (*self.vals[idx as usize].get()).write(val) };
        loop {
            let cur = self.top.load(Acquire);
            let (tag, head_idx) = unpack_ptr(cur);
            self.next[idx as usize].store(pack_ptr(tag, head_idx), Relaxed);
            if self
                .top
                .compare_exchange_weak(cur, pack_ptr(tag.wrapping_add(1), idx), AcqRel, Acquire)
                .is_ok()
            {
                return Ok(());
            }
            std::hint::spin_loop();
        }
    }

    pub fn pop(&self) -> Option<T> {
        let idx = loop {
            let cur = self.top.load(Acquire);
            let (tag, idx) = unpack_ptr(cur);
            if idx == NONE_IDX {
                return None;
            }
            let (_, nx) = unpack_ptr(self.next[idx as usize].load(Acquire));
            if self
                .top
                .compare_exchange_weak(cur, pack_ptr(tag.wrapping_add(1), nx), AcqRel, Acquire)
                .is_ok()
            {
                break idx;
            }
            std::hint::spin_loop();
        };
        // SAFETY: `idx` was uniquely popped from `top`.
        let val = unsafe { (*self.vals[idx as usize].get()).assume_init_read() };
        loop {
            let fh = self.free_head.load(Acquire);
            let (tag, head_idx) = unpack_ptr(fh);
            self.next[idx as usize].store(pack_ptr(tag, head_idx), Relaxed);
            if self
                .free_head
                .compare_exchange_weak(fh, pack_ptr(tag.wrapping_add(1), idx), AcqRel, Acquire)
                .is_ok()
            {
                return Some(val);
            }
            std::hint::spin_loop();
        }
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// Wait-free bounded single-producer / single-consumer ring buffer (`u64`).
pub struct SpscRing {
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
    buf: Box<[AtomicU64]>,
}

impl SpscRing {
    pub fn with_power_of_two(cap_pow2: usize) -> Self {
        let cap = cap_pow2.next_power_of_two().max(2);
        let buf = (0..cap).map(|_| AtomicU64::new(0)).collect::<Vec<_>>();
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            mask: cap - 1,
            buf: buf.into_boxed_slice(),
        }
    }

    pub fn try_push(&self, val: u64) -> bool {
        let tail = self.tail.load(Relaxed);
        let next = tail.wrapping_add(1);
        if next.wrapping_sub(self.head.load(Acquire)) > self.buf.len() {
            return false;
        }
        self.buf[tail & self.mask].store(val, Release);
        self.tail.store(next, Release);
        true
    }

    pub fn try_pop(&self) -> Option<u64> {
        let head = self.head.load(Relaxed);
        if head == self.tail.load(Acquire) {
            return None;
        }
        let v = self.buf[head & self.mask].load(Acquire);
        self.head.store(head.wrapping_add(1), Release);
        Some(v)
    }
}

/// Versioned sequence lock for reader-heavy `Copy` payloads.
pub struct SeqLock<T: Copy> {
    seq: AtomicU64,
    data: UnsafeCell<T>,
}

// SAFETY: Writes are serialized under an exclusive write lock (`write` uses `SeqCst` CAS on odd seq).
unsafe impl<T: Copy + Send> Send for SeqLock<T> {}
unsafe impl<T: Copy + Send> Sync for SeqLock<T> {}

impl<T: Copy> SeqLock<T> {
    pub fn new(initial: T) -> Self {
        Self {
            seq: AtomicU64::new(0),
            data: UnsafeCell::new(initial),
        }
    }

    pub fn write(&self, value: T) {
        loop {
            let s = self.seq.load(Acquire);
            if s & 1 == 0
                && self
                    .seq
                    .compare_exchange_weak(s, s + 1, AcqRel, Relaxed)
                    .is_ok()
            {
                // SAFETY: `s + 1` is odd and exclusively acquired by this writer.
                unsafe {
                    *self.data.get() = value;
                }
                self.seq.store(s + 2, Release);
                return;
            }
            std::hint::spin_loop();
        }
    }

    pub fn read(&self) -> T {
        loop {
            let s1 = self.seq.load(Acquire);
            if s1 & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }
            // SAFETY: Optimistic copy re-verified below against `s2 == s1`; zero UB only if `T` has
            // no interior invalid bit-patterns (guaranteed for plain `Copy` primitives in benchmarks)
            // — even better, guard concurrent writes with atomic fence around read.
            let val = unsafe { std::ptr::read_volatile(self.data.get()) };
            std::sync::atomic::fence(Acquire);
            let s2 = self.seq.load(Relaxed);
            if s1 == s2 {
                return val;
            }
        }
    }
}

/// Fair ticket spinlock.
pub struct TicketLock {
    next_ticket: AtomicUsize,
    serving: AtomicUsize,
    locked: AtomicBool,
}

impl Default for TicketLock {
    fn default() -> Self {
        Self::new()
    }
}

impl TicketLock {
    pub const fn new() -> Self {
        Self {
            next_ticket: AtomicUsize::new(0),
            serving: AtomicUsize::new(0),
            locked: AtomicBool::new(false),
        }
    }

    pub fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        let my = self.next_ticket.fetch_add(1, Relaxed);
        while self.serving.load(Acquire) != my {
            std::hint::spin_loop();
        }
        self.locked.store(true, Release);
        let res = f();
        self.locked.store(false, Release);
        self.serving.store(my.wrapping_add(1), Release);
        res
    }
}

#[repr(align(64))]
struct PaddedAtomic(AtomicU64);

/// Cache-line padded sharded atomic counter avoiding false sharing.
pub struct StripedCounter {
    shards: [PaddedAtomic; 8],
}

impl Default for StripedCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl StripedCounter {
    pub const fn new() -> Self {
        Self {
            shards: [
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
                PaddedAtomic(AtomicU64::new(0)),
            ],
        }
    }

    pub fn add(&self, stripe: usize, delta: u64) {
        self.shards[stripe & 7].0.fetch_add(delta, Relaxed);
    }

    pub fn sum(&self) -> u64 {
        self.shards.iter().map(|s| s.0.load(Relaxed)).sum()
    }
}

pub fn kernel_mpmc_queue(iters: usize) -> u64 {
    let q = MpmcQueue::with_capacity(64);
    let mut acc = 0u64;
    for i in 0..iters.max(64) {
        let v = (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let _ = q.push(v);
        if let Some(out) = q.pop() {
            acc = acc.wrapping_add(out);
        }
    }
    acc
}

pub fn kernel_treiber_stack(iters: usize) -> u64 {
    let s = TreiberStack::with_capacity(64);
    let mut acc = 0u64;
    for i in 0..iters.max(64) {
        let _ = s.push(i as u64 ^ 0xa5a5_5a5a);
        if let Some(v) = s.pop() {
            acc = acc.wrapping_add(v);
        }
    }
    acc
}

pub fn kernel_spsc_ring(iters: usize) -> u64 {
    let r = SpscRing::with_power_of_two(64);
    let mut acc = 0u64;
    for i in 0..iters.max(64) {
        let _ = r.try_push(i as u64 + 1);
        if let Some(v) = r.try_pop() {
            acc = acc.wrapping_add(v);
        }
    }
    acc
}

pub fn kernel_seqlock(iters: usize) -> u64 {
    let sl = SeqLock::new(0u64);
    let mut acc = 0u64;
    for i in 0..iters.max(64) {
        if i & 7 == 0 {
            sl.write(i as u64);
        }
        acc = acc.wrapping_add(sl.read());
    }
    acc
}

pub fn kernel_ticket_lock(iters: usize) -> u64 {
    let tl = TicketLock::new();
    let mut acc = 0u64;
    for i in 0..iters.max(32) {
        acc = tl.with_lock(|| acc.wrapping_add(i as u64));
    }
    acc
}

pub fn kernel_striped_counter(iters: usize) -> u64 {
    let sc = StripedCounter::new();
    for i in 0..iters.max(64) {
        sc.add(i, 1);
    }
    sc.sum()
}
