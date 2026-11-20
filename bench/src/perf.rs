//! Hardware performance counters as a [`Measurement`]: instructions, cycles, cache and branch
//! events counted by the CPU while a batch runs.
//!
//! Linux only (`perf_event_open`), and only where the kernel lets an unprivileged process
//! count its own user-space work (`/proc/sys/kernel/perf_event_paranoid` ≤ 2, a PMU the
//! virtual machine exposes). [`PerfCounter::open`] fails with a reason everywhere else, so a
//! suite decides for itself whether a missing counter is an error or a skipped case.
//!
//! ```no_run
//! use airbug_bench::{BatchPolicy, Suite, perf::{Event, PerfCounter}};
//! # fn main() -> airbug_bench::Result<()> {
//! let mut suite = Suite::new("sort");
//! match PerfCounter::open(Event::CacheMisses) {
//!     Ok(counter) => {
//!         suite.bench_measured("sort/cache", counter, BatchPolicy::SmallInput, || {
//!             std::hint::black_box((0..512u32).rev().collect::<Vec<_>>()).sort();
//!         })?;
//!     }
//!     Err(why) => eprintln!("no cache-miss counter here: {why}"),
//! }
//! suite.main()
//! # }
//! ```
use crate::{Metric, Result, error, measurement::Measurement, model::Direction};

/// What the CPU counts. Generic events: the kernel maps each to this processor's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Instructions,
    Cycles,
    CacheReferences,
    CacheMisses,
    Branches,
    BranchMisses,
    ICacheMisses,
    DtlbMisses,
    ItlbMisses,
    PageFaults,
}

impl Event {
    pub const ALL: [Event; 10] = [
        Event::Instructions,
        Event::Cycles,
        Event::CacheReferences,
        Event::CacheMisses,
        Event::Branches,
        Event::BranchMisses,
        Event::ICacheMisses,
        Event::DtlbMisses,
        Event::ItlbMisses,
        Event::PageFaults,
    ];

    /// Metric id, stable across runs: `hw.cache_misses`.
    pub fn id(self) -> &'static str {
        match self {
            Event::Instructions => "hw.instructions",
            Event::Cycles => "hw.cycles",
            Event::CacheReferences => "hw.cache_references",
            Event::CacheMisses => "hw.cache_misses",
            Event::Branches => "hw.branches",
            Event::BranchMisses => "hw.branch_misses",
            Event::ICacheMisses => "hw.icache_misses",
            Event::DtlbMisses => "hw.dtlb_misses",
            Event::ItlbMisses => "hw.itlb_misses",
            Event::PageFaults => "sw.page_faults",
        }
    }

    /// `(type, config)` for `linux/perf_event.h`.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn attr_kind_and_config(self) -> (u32, u64) {
        const PERF_TYPE_HARDWARE: u32 = 0;
        const PERF_TYPE_SOFTWARE: u32 = 1;
        const PERF_TYPE_HW_CACHE: u32 = 3;
        const fn hw_cache(id: u64) -> u64 {
            // (id) | (OP_READ << 8) | (RESULT_MISS << 16)
            id | (1u64 << 16)
        }
        match self {
            Event::Cycles => (PERF_TYPE_HARDWARE, 0),
            Event::Instructions => (PERF_TYPE_HARDWARE, 1),
            Event::CacheReferences => (PERF_TYPE_HARDWARE, 2),
            Event::CacheMisses => (PERF_TYPE_HARDWARE, 3),
            Event::Branches => (PERF_TYPE_HARDWARE, 4),
            Event::BranchMisses => (PERF_TYPE_HARDWARE, 5),
            Event::ICacheMisses => (PERF_TYPE_HW_CACHE, hw_cache(1)),
            Event::DtlbMisses => (PERF_TYPE_HW_CACHE, hw_cache(3)),
            Event::ItlbMisses => (PERF_TYPE_HW_CACHE, hw_cache(4)),
            Event::PageFaults => (PERF_TYPE_SOFTWARE, 2),
        }
    }
}

/// One counter for the calling thread (or process page faults on Windows). Counts from `open`
/// until drop; a measurement is the difference between two reads, so nesting and reuse are safe.
#[derive(Debug)]
pub struct PerfCounter {
    event: Event,
    #[cfg(target_os = "linux")]
    file: std::fs::File,
}

impl PerfCounter {
    pub fn event(&self) -> Event {
        self.event
    }

    /// Whether this machine can count `event` for the calling thread.
    pub fn available(event: Event) -> bool {
        Self::open(event).is_ok()
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    #[repr(C)]
    #[derive(Default)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut core::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }

    fn read_page_faults() -> Result<u64> {
        let mut counters = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            ..ProcessMemoryCounters::default()
        };
        // SAFETY: `counters` is repr(C) matching Win32 `PROCESS_MEMORY_COUNTERS` with valid `cb`.
        let ok = unsafe {
            K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters as *mut ProcessMemoryCounters,
                counters.cb,
            )
        };
        if ok == 0 {
            return Err(error(format!(
                "software counter sw.page_faults unavailable: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(u64::from(counters.page_fault_count))
    }

    impl PerfCounter {
        pub fn open(event: Event) -> Result<Self> {
            match event {
                Event::PageFaults => {
                    let _ = read_page_faults()?;
                    Ok(Self { event })
                }
                _ => Err(error(format!(
                    "hardware counter {} needs Linux perf_event_open; unavailable on Windows",
                    event.id()
                ))),
            }
        }

        pub(super) fn read(&mut self) -> Result<u64> {
            match self.event {
                Event::PageFaults => read_page_faults(),
                _ => Err(error(format!(
                    "hardware counter {} unavailable on this platform",
                    self.event.id()
                ))),
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    unsafe extern "C" {
        fn thread_selfcounts(kind: libc::c_int, buf: *mut u64, nbytes: libc::size_t)
        -> libc::c_int;
    }

    /// `THCOUNTS_INSTRS_CYCLES` (`1`): returns `[instructions, cycles]` for the calling thread
    /// via XNU's `kpc` / PMC per-thread accumulators without root privileges.
    fn read_thread_instrs_cycles(event: Event) -> Result<[u64; 2]> {
        let mut counts = [0u64; 2];
        // SAFETY: `counts` is a 16-byte buffer for `[instructions, cycles]`.
        let rc = unsafe {
            thread_selfcounts(
                1,
                counts.as_mut_ptr(),
                std::mem::size_of::<[u64; 2]>() as libc::size_t,
            )
        };
        if rc != 0 {
            let os = std::io::Error::last_os_error();
            let hint = match os.raw_os_error() {
                Some(libc::ENOTSUP) | Some(libc::EOPNOTSUPP) | Some(libc::ENOENT) => {
                    " (this CPU or virtual machine exposes no per-thread PMC counters)"
                }
                _ => "",
            };
            return Err(error(format!(
                "hardware counter {} unavailable: macOS thread_selfcounts failed: {os}{hint}",
                event.id()
            )));
        }
        Ok(counts)
    }

    fn read_page_faults() -> Result<u64> {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: `getrusage(RUSAGE_SELF)` initializes `usage` on return 0.
        let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
        if rc != 0 {
            return Err(error(format!(
                "software counter sw.page_faults unavailable: {}",
                std::io::Error::last_os_error()
            )));
        }
        let usage = unsafe { usage.assume_init() };
        let minflt = u64::try_from(usage.ru_minflt).unwrap_or(0);
        let majflt = u64::try_from(usage.ru_majflt).unwrap_or(0);
        Ok(minflt.saturating_add(majflt))
    }

    impl PerfCounter {
        pub fn open(event: Event) -> Result<Self> {
            match event {
                Event::Instructions | Event::Cycles => {
                    let counts = read_thread_instrs_cycles(event)?;
                    let value = match event {
                        Event::Instructions => counts[0],
                        Event::Cycles => counts[1],
                        _ => unreachable!(),
                    };
                    if value == 0 {
                        return Err(error(format!(
                            "hardware counter {} unavailable: thread_selfcounts reported 0",
                            event.id()
                        )));
                    }
                    Ok(Self { event })
                }
                Event::PageFaults => {
                    let _ = read_page_faults()?;
                    Ok(Self { event })
                }
                _ => Err(error(format!(
                    "hardware counter {} needs Linux perf_event_open; unavailable on macOS",
                    event.id()
                ))),
            }
        }

        pub(super) fn read(&mut self) -> Result<u64> {
            match self.event {
                Event::Instructions => Ok(read_thread_instrs_cycles(self.event)?[0]),
                Event::Cycles => Ok(read_thread_instrs_cycles(self.event)?[1]),
                Event::PageFaults => read_page_faults(),
                _ => Err(error(format!(
                    "hardware counter {} unavailable on macOS",
                    self.event.id()
                ))),
            }
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl PerfCounter {
    pub fn open(event: Event) -> Result<Self> {
        Err(error(format!(
            "hardware counter {} needs Linux perf_event_open; unavailable on this platform",
            event.id()
        )))
    }

    fn read(&mut self) -> Result<u64> {
        Err(error("hardware counters are unavailable on this platform"))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{io::Read, os::fd::FromRawFd};

    /// `struct perf_event_attr`, `PERF_ATTR_SIZE_VER5` (112 bytes).
    #[repr(C)]
    #[derive(Default)]
    struct Attr {
        kind: u32,
        size: u32,
        config: u64,
        sample_period: u64,
        sample_type: u64,
        read_format: u64,
        flags: u64,
        wakeup_events: u32,
        bp_type: u32,
        bp_addr: u64,
        bp_len: u64,
        branch_sample_type: u64,
        sample_regs_user: u64,
        sample_stack_user: u32,
        clockid: i32,
        sample_regs_intr: u64,
        aux_watermark: u32,
        sample_max_stack: u16,
        reserved: u16,
    }

    const EXCLUDE_KERNEL: u64 = 1 << 5;
    const EXCLUDE_HV: u64 = 1 << 6;
    const PERF_FLAG_FD_CLOEXEC: libc::c_ulong = 1 << 3;

    impl PerfCounter {
        pub fn open(event: Event) -> Result<Self> {
            let (kind, config) = event.attr_kind_and_config();
            let flags = if matches!(event, Event::PageFaults) {
                0
            } else {
                EXCLUDE_KERNEL | EXCLUDE_HV
            };
            let attr = Attr {
                kind,
                size: std::mem::size_of::<Attr>() as u32,
                config,
                // Enabled from the start: a measurement is the difference of two reads.
                flags,
                ..Attr::default()
            };
            // SAFETY: `attr` is a properly sized, initialised `perf_event_attr`; the call
            // only reads it. pid 0 / cpu -1 = the calling thread on any CPU, no group.
            let fd = unsafe {
                libc::syscall(
                    libc::SYS_perf_event_open,
                    &attr as *const Attr,
                    0 as libc::pid_t,
                    -1 as libc::c_int,
                    -1 as libc::c_int,
                    PERF_FLAG_FD_CLOEXEC,
                )
            };
            if fd < 0 {
                let os = std::io::Error::last_os_error();
                let hint = match os.raw_os_error() {
                    Some(libc::EACCES) | Some(libc::EPERM) => {
                        " (lower /proc/sys/kernel/perf_event_paranoid to 2 or less, or grant CAP_PERFMON)"
                    }
                    Some(libc::ENOENT) | Some(libc::EOPNOTSUPP) | Some(libc::ENODEV) => {
                        " (this CPU or virtual machine exposes no such counter)"
                    }
                    Some(libc::ENOSYS) => " (the kernel was built without perf events)",
                    _ => "",
                };
                return Err(error(format!(
                    "hardware counter {} unavailable: {os}{hint}",
                    event.id()
                )));
            }
            // SAFETY: `fd` is a fresh descriptor owned by nobody else.
            let file = unsafe { std::fs::File::from_raw_fd(fd as libc::c_int) };
            Ok(Self { event, file })
        }

        pub(super) fn read(&mut self) -> Result<u64> {
            let mut bytes = [0u8; 8];
            self.file
                .read_exact(&mut bytes)
                .map_err(|e| error(format!("reading {}: {e}", self.event.id())))?;
            Ok(u64::from_ne_bytes(bytes))
        }
    }
}

impl Measurement for PerfCounter {
    type Start = u64;
    type Value = u64;

    fn metric(&self) -> Metric {
        Metric {
            id: self.event.id().into(),
            unit: "events".into(),
            scope: "perf_event v1; calling thread; user space only".into(),
            phase: "measurement".into(),
            statistic: "batch_total".into(),
            direction: Direction::Lower,
        }
    }

    fn start(&mut self) -> Result<u64> {
        self.read()
    }

    fn end(&mut self, start: u64) -> Result<u64> {
        Ok(self.read()?.saturating_sub(start))
    }

    fn zero(&self) -> u64 {
        0
    }

    fn add(&self, total: u64, value: u64) -> Result<u64> {
        total
            .checked_add(value)
            .ok_or_else(|| error("hardware counter total overflowed"))
    }

    fn to_f64(&self, value: &u64) -> Result<f64> {
        Ok(*value as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct_and_a_missing_counter_says_why() {
        let ids: std::collections::BTreeSet<_> = Event::ALL.iter().map(|e| e.id()).collect();
        assert_eq!(ids.len(), Event::ALL.len());
        let configs: std::collections::BTreeSet<_> = Event::ALL
            .iter()
            .map(|e| e.attr_kind_and_config())
            .collect();
        assert_eq!(configs.len(), Event::ALL.len());
        assert_eq!(Event::Cycles.attr_kind_and_config(), (0, 0));
        assert_eq!(Event::Instructions.attr_kind_and_config(), (0, 1));
        assert_eq!(Event::CacheReferences.attr_kind_and_config(), (0, 2));
        assert_eq!(Event::CacheMisses.attr_kind_and_config(), (0, 3));
        assert_eq!(Event::Branches.attr_kind_and_config(), (0, 4));
        assert_eq!(Event::BranchMisses.attr_kind_and_config(), (0, 5));
        assert_eq!(
            Event::ICacheMisses.attr_kind_and_config(),
            (3, 1 | (1 << 16))
        );
        assert_eq!(Event::DtlbMisses.attr_kind_and_config(), (3, 3 | (1 << 16)));
        assert_eq!(Event::ItlbMisses.attr_kind_and_config(), (3, 4 | (1 << 16)));
        assert_eq!(Event::PageFaults.attr_kind_and_config(), (1, 2));
        for event in Event::ALL {
            match PerfCounter::open(event) {
                Ok(counter) => assert_eq!(counter.metric().id, event.id()),
                Err(why) => {
                    let text = why.to_string();
                    assert!(text.contains(event.id()), "{text}");
                    assert!(text.contains("unavailable"), "{text}");
                }
            }
        }
    }

    /// Where the machine has a PMU, more work must count more instructions and cycles.
    #[test]
    fn instructions_grow_with_work_where_counters_exist() {
        for event in [Event::Instructions, Event::Cycles] {
            let Ok(mut counter) = PerfCounter::open(event) else {
                continue;
            };
            let mut spin = |n: u64| {
                let start = counter.start().unwrap();
                let mut x = 0u64;
                for i in 0..n {
                    x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(i));
                }
                std::hint::black_box(x);
                counter.end(start).unwrap()
            };
            let (small, large) = (spin(1_000), spin(1_000_000));
            assert!(
                small > 0 && large > small * 10,
                "{}: {small} vs {large}",
                event.id()
            );
        }
    }
}
