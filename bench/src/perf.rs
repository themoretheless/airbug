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
}

impl Event {
    pub const ALL: [Event; 6] = [
        Event::Instructions,
        Event::Cycles,
        Event::CacheReferences,
        Event::CacheMisses,
        Event::Branches,
        Event::BranchMisses,
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
        }
    }

    /// `PERF_COUNT_HW_*` of `linux/perf_event.h`.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn config(self) -> u64 {
        match self {
            Event::Cycles => 0,
            Event::Instructions => 1,
            Event::CacheReferences => 2,
            Event::CacheMisses => 3,
            Event::Branches => 4,
            Event::BranchMisses => 5,
        }
    }
}

/// One counter for the calling thread, user space only. Counts from `open` until drop; a
/// measurement is the difference between two reads, so nesting and reuse are safe.
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

#[cfg(not(target_os = "linux"))]
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

    const PERF_TYPE_HARDWARE: u32 = 0;
    const EXCLUDE_KERNEL: u64 = 1 << 5;
    const EXCLUDE_HV: u64 = 1 << 6;
    const PERF_FLAG_FD_CLOEXEC: libc::c_ulong = 1 << 3;

    impl PerfCounter {
        pub fn open(event: Event) -> Result<Self> {
            let attr = Attr {
                kind: PERF_TYPE_HARDWARE,
                size: std::mem::size_of::<Attr>() as u32,
                config: event.config(),
                // Enabled from the start: a measurement is the difference of two reads.
                flags: EXCLUDE_KERNEL | EXCLUDE_HV,
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

    /// Where the machine has a PMU, more work must count more instructions.
    #[test]
    fn instructions_grow_with_work_where_counters_exist() {
        let Ok(mut counter) = PerfCounter::open(Event::Instructions) else {
            return;
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
        assert!(small > 0 && large > small * 10, "{small} vs {large}");
    }
}
