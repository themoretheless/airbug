use super::TrackingAllocator;
use std::{cell::Cell, marker::PhantomData, rc::Rc};

/// Successful allocator events performed on this thread during one phase.
/// Net live bytes may be negative when freeing an allocation made elsewhere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThreadStats {
    pub allocations: u128,
    pub deallocations: u128,
    pub reallocations: u128,
    pub allocated_bytes: u128,
    pub deallocated_bytes: u128,
    pub grow_operations: u128,
    pub shrink_operations: u128,
    pub grown_bytes: u128,
    pub shrunk_bytes: u128,
    /// Net block balance; freeing pre-existing or foreign allocations can make it negative.
    pub net_live_count: i128,
    /// Largest positive net block balance since the phase began.
    pub peak_above_start_count: u128,
    pub net_live_bytes: i128,
    /// Largest positive net byte balance since the phase began.
    pub peak_above_start_bytes: u128,
    pub overflowed: bool,
}
#[derive(Clone, Copy)]
struct Active {
    owner: usize,
    stats: ThreadStats,
}
thread_local! { static ACTIVE: Cell<Option<Active>> = const { Cell::new(None) }; }

pub(super) enum Event {
    Allocate(usize),
    Free(usize),
    Resize(usize, usize),
}
fn add(target: &mut u128, value: u128, overflowed: &mut bool) {
    match target.checked_add(value) {
        Some(sum) => *target = sum,
        None => {
            *target = u128::MAX;
            *overflowed = true;
        }
    }
}
impl<A> TrackingAllocator<A> {
    /// Begin an isolated current-thread phase. Only this allocator instance is
    /// counted. Nested phases on the same thread are rejected. Other threads
    /// can independently start their own phases.
    pub fn begin_thread_phase(&self) -> crate::Result<ThreadPhase<'_, A>> {
        let started = ACTIVE
            .try_with(|cell| {
                if cell.get().is_some() {
                    return false;
                }
                cell.set(Some(Active {
                    owner: self as *const Self as usize,
                    stats: ThreadStats::default(),
                }));
                true
            })
            .unwrap_or(false);
        if !started {
            return Err(crate::error(
                "thread allocation phase already active or TLS unavailable",
            ));
        }
        Ok(ThreadPhase {
            allocator: self,
            finished: false,
            not_send: PhantomData,
        })
    }
    pub(super) fn record_thread(&self, event: Event) {
        let _ = ACTIVE.try_with(|cell| {
            let Some(mut active) = cell.get() else {
                return;
            };
            if active.owner != self as *const Self as usize {
                return;
            }
            let s = &mut active.stats;
            let count_delta = match event {
                Event::Allocate(_) => 1,
                Event::Free(_) => -1,
                Event::Resize(_, _) => 0,
            };
            s.net_live_count = s
                .net_live_count
                .checked_add(count_delta)
                .unwrap_or_else(|| {
                    s.overflowed = true;
                    s.net_live_count.saturating_add(count_delta)
                });
            s.peak_above_start_count = s
                .peak_above_start_count
                .max(s.net_live_count.max(0) as u128);
            let delta = match event {
                Event::Allocate(n) => {
                    add(&mut s.allocations, 1, &mut s.overflowed);
                    add(&mut s.allocated_bytes, n as u128, &mut s.overflowed);
                    n as i128
                }
                Event::Free(n) => {
                    add(&mut s.deallocations, 1, &mut s.overflowed);
                    add(&mut s.deallocated_bytes, n as u128, &mut s.overflowed);
                    -(n as i128)
                }
                Event::Resize(old, new) => {
                    add(&mut s.reallocations, 1, &mut s.overflowed);
                    if new >= old {
                        add(&mut s.grow_operations, 1, &mut s.overflowed);
                        add(&mut s.grown_bytes, (new - old) as u128, &mut s.overflowed);
                    } else if new < old {
                        add(&mut s.shrink_operations, 1, &mut s.overflowed);
                        add(&mut s.shrunk_bytes, (old - new) as u128, &mut s.overflowed);
                    }
                    new as i128 - old as i128
                }
            };
            s.net_live_bytes = s.net_live_bytes.checked_add(delta).unwrap_or_else(|| {
                s.overflowed = true;
                if delta < 0 { i128::MIN } else { i128::MAX }
            });
            s.peak_above_start_bytes = s
                .peak_above_start_bytes
                .max(s.net_live_bytes.max(0) as u128);
            cell.set(Some(active));
        });
    }
}
/// A thread-bound guard. Dropping it disables tracking, including on unwind.
pub struct ThreadPhase<'a, A> {
    allocator: &'a TrackingAllocator<A>,
    finished: bool,
    not_send: PhantomData<Rc<()>>,
}
impl<A> ThreadPhase<'_, A> {
    pub fn finish(mut self) -> ThreadStats {
        let active = ACTIVE
            .with(|cell| cell.take())
            .expect("active thread phase");
        debug_assert_eq!(active.owner, self.allocator as *const _ as usize);
        self.finished = true;
        active.stats
    }
}
impl<A> Drop for ThreadPhase<'_, A> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = ACTIVE.try_with(|cell| cell.set(None));
        }
    }
}

impl ThreadStats {
    pub(crate) const METRICS: [(&'static str, &'static str, &'static str); 13] = [
        ("alloc.count", "count", "batch_total"),
        ("alloc.dealloc_count", "count", "batch_total"),
        ("alloc.realloc_count", "count", "batch_total"),
        ("alloc.bytes", "bytes", "batch_total"),
        ("alloc.dealloc_bytes", "bytes", "batch_total"),
        ("alloc.grow_count", "count", "batch_total"),
        ("alloc.shrink_count", "count", "batch_total"),
        ("alloc.grown_bytes", "bytes", "batch_total"),
        ("alloc.shrunk_bytes", "bytes", "batch_total"),
        ("alloc.net_growth_bytes", "bytes", "batch_total"),
        ("alloc.net_release_bytes", "bytes", "batch_total"),
        ("alloc.peak_above_start_bytes", "bytes", "sample_peak"),
        ("alloc.peak_above_start_count", "count", "sample_peak"),
    ];
    pub(crate) fn metric_values(self) -> [u128; 13] {
        [
            self.allocations,
            self.deallocations,
            self.reallocations,
            self.allocated_bytes,
            self.deallocated_bytes,
            self.grow_operations,
            self.shrink_operations,
            self.grown_bytes,
            self.shrunk_bytes,
            self.net_live_bytes.max(0) as u128,
            self.net_live_bytes.min(0).unsigned_abs(),
            self.peak_above_start_bytes,
            self.peak_above_start_count,
        ]
    }
}

impl ThreadStats {
    pub(crate) fn append_batch(&mut self, other: Self) {
        macro_rules! sum { ($($field:ident),*) => { $(add(&mut self.$field, other.$field, &mut self.overflowed);)* }; }
        sum!(
            allocations,
            deallocations,
            reallocations,
            allocated_bytes,
            deallocated_bytes,
            grow_operations,
            shrink_operations,
            grown_bytes,
            shrunk_bytes
        );
        self.net_live_count = self
            .net_live_count
            .checked_add(other.net_live_count)
            .unwrap_or_else(|| {
                self.overflowed = true;
                self.net_live_count.saturating_add(other.net_live_count)
            });
        self.peak_above_start_count = self
            .peak_above_start_count
            .max(other.peak_above_start_count);
        self.net_live_bytes = self
            .net_live_bytes
            .checked_add(other.net_live_bytes)
            .unwrap_or_else(|| {
                self.overflowed = true;
                self.net_live_bytes.saturating_add(other.net_live_bytes)
            });
        self.peak_above_start_bytes = self
            .peak_above_start_bytes
            .max(other.peak_above_start_bytes);
        self.overflowed |= other.overflowed;
    }
}
impl<A> TrackingAllocator<A> {
    pub(crate) fn measure_input_batch<I, O>(
        &self,
        timer: crate::timer::Timer,
        n: u64,
        batch_size: u64,
        setup: &mut impl FnMut() -> I,
        operation: &mut impl FnMut(&mut I) -> O,
        policy: crate::DropPolicy,
    ) -> crate::Result<(crate::timing::Measured, ThreadStats)> {
        use std::hint::black_box;
        let mut remaining = n;
        let mut duration = crate::timing::Measured::default();
        let mut counts = ThreadStats::default();
        while remaining > 0 {
            let size = remaining.min(batch_size);
            let mut inputs: Vec<_> = (0..size).map(|_| setup()).collect();
            let mut outputs =
                Vec::with_capacity(if matches!(policy, crate::DropPolicy::OutsideTiming) {
                    size as usize
                } else {
                    0
                });
            let phase = self.begin_thread_phase()?;
            let start = timer.start();
            for input in &mut inputs {
                let output = black_box(operation(black_box(input)));
                match policy {
                    crate::DropPolicy::InsideTiming => drop(output),
                    crate::DropPolicy::OutsideTiming => outputs.push(output),
                }
            }
            let raw = start.elapsed_ns()?;
            let stats = phase.finish();
            duration = duration.add(timer.measured(raw, size, Some(&stats))?)?;
            counts.append_batch(stats);
            black_box(&outputs);
            remaining -= size;
        }
        if counts.overflowed {
            return Err(crate::error("allocation batch counters overflowed"));
        }
        Ok((duration, counts))
    }
}
