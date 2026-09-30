//! Explicit monotonic clocks for benchmark measurement boundaries.
use crate::{Result, error};
use std::{
    num::NonZeroU64,
    sync::OnceLock,
    time::{Duration, Instant},
};

/// Lazy clock selection. Parsing does not access CPU counters or calibrate them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimerKind {
    #[default]
    Os,
    Cpu,
}
impl TimerKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "os" => Ok(Self::Os),
            "cpu" => Ok(Self::Cpu),
            _ => Err(error("timer must be os or cpu")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Os => "os",
            Self::Cpu => "cpu",
        }
    }
    /// Validate hardware and discover frequency only when execution begins.
    pub fn resolve(self) -> Result<Timer> {
        match self {
            Self::Os => Ok(Timer::os()),
            Self::Cpu => Timer::cpu(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Clock {
    Os,
    Cpu(NonZeroU64),
    #[cfg(test)]
    Test(Option<u128>),
}
/// A validated clock. CPU frequency discovery is lazy and cached process-wide.
#[derive(Clone, Copy, Debug)]
pub struct Timer {
    clock: Clock,
    correction: Option<crate::timing::Correction>,
}
impl Default for Timer {
    fn default() -> Self {
        Self::os()
    }
}
#[derive(Clone, Copy, Debug)]
enum Stamp {
    Os(Instant),
    Cpu(u64),
    #[cfg(test)]
    Test,
}
#[derive(Clone, Copy, Debug)]
pub struct TimerStart {
    timer: Timer,
    stamp: Stamp,
}
/// Diagnostic calibration; no overhead is subtracted from observations.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Calibration {
    pub clock: String,
    pub frequency_hz: Option<u64>,
    pub trials: u32,
    /// Smallest nonzero observed interval; None when bounded retries also return zero.
    pub observed_resolution_ns: Option<u128>,
    /// Includes immediate trials and any delayed retries used for resolution.
    pub resolution_trials: u32,
    /// Black-box iterations between timestamp reads in the final retry batch.
    pub resolution_delay_iterations: u32,
    /// Minimum observed cost of an empty start/end interval (may be zero).
    pub empty_interval_ns: u128,
    /// Minimum elapsed time for a black-box loop. Divide by loop_iterations
    /// for an estimate per iteration; retaining the ratio avoids rounding to zero.
    pub loop_batch_ns: u128,
    pub loop_iterations: u64,
    pub loop_trials: u32,
}
impl Timer {
    #[cfg(test)]
    pub(crate) fn fixed(nanos: Option<u128>) -> Self {
        Self {
            clock: Clock::Test(nanos),
            correction: None,
        }
    }
    pub const fn os() -> Self {
        Self {
            clock: Clock::Os,
            correction: None,
        }
    }
    pub fn cpu() -> Result<Self> {
        static FREQUENCY: OnceLock<std::result::Result<NonZeroU64, String>> = OnceLock::new();
        let frequency = FREQUENCY.get_or_init(|| {
            cpu::frequency().and_then(|n| {
                NonZeroU64::new(n).ok_or_else(|| "CPU counter frequency is zero".into())
            })
        });
        match frequency {
            Ok(hz) => Ok(Self {
                clock: Clock::Cpu(*hz),
                correction: None,
            }),
            Err(reason) => Err(error(reason.clone())),
        }
    }
    pub fn name(self) -> &'static str {
        match self.clock {
            Clock::Os => "os",
            Clock::Cpu(_) => "cpu",
            #[cfg(test)]
            Clock::Test(_) => "test",
        }
    }
    pub fn frequency_hz(self) -> Option<u64> {
        match self.clock {
            Clock::Os => None,
            Clock::Cpu(hz) => Some(hz.get()),
            #[cfg(test)]
            Clock::Test(_) => None,
        }
    }
    pub fn start(self) -> TimerStart {
        let stamp = match self.clock {
            Clock::Os => Stamp::Os(Instant::now()),
            Clock::Cpu(_) => Stamp::Cpu(cpu::start()),
            #[cfg(test)]
            Clock::Test(_) => Stamp::Test,
        };
        TimerStart { timer: self, stamp }
    }
    pub(crate) fn measured(
        self,
        raw_ns: u128,
        operations: u64,
        counts: Option<&crate::alloc::ThreadStats>,
    ) -> Result<crate::timing::Measured> {
        match self.correction {
            Some(correction) => correction.apply(raw_ns, operations, counts),
            None => Ok(crate::timing::Measured::raw(raw_ns)),
        }
    }
    pub(crate) fn correction(self) -> Option<crate::timing::Correction> {
        self.correction
    }
    #[cfg(test)]
    pub(crate) fn fixed_corrected(nanos: u128, correction: crate::timing::Correction) -> Self {
        Self {
            clock: Clock::Test(Some(nanos)),
            correction: Some(correction),
        }
    }
    pub(crate) fn with_correction(mut self) -> Result<Self> {
        if self.correction.is_some() {
            return Ok(self);
        }
        let calibration = self.cached_calibration()?;
        static OS: OnceLock<std::result::Result<crate::alloc::AllocationOverhead, String>> =
            OnceLock::new();
        static CPU: OnceLock<std::result::Result<crate::alloc::AllocationOverhead, String>> =
            OnceLock::new();
        let cache = match self.clock {
            Clock::Os => Some(&OS),
            Clock::Cpu(_) => Some(&CPU),
            #[cfg(test)]
            Clock::Test(_) => None,
        };
        let allocation = if let Some(cache) = cache {
            cache
                .get_or_init(|| crate::alloc::calibrate_overhead(self).map_err(|e| e.to_string()))
                .clone()
                .map_err(error)?
        } else {
            crate::alloc::calibrate_overhead(self)?
        };
        self.correction = Some(crate::timing::Correction {
            loop_batch_ns: calibration.loop_batch_ns,
            loop_iterations: calibration.loop_iterations,
            tally_batch_ns: [
                allocation.alloc_batch_ns,
                allocation.dealloc_batch_ns,
                allocation.grow_batch_ns,
                allocation.shrink_batch_ns,
            ]
            .map(|n| n.saturating_sub(allocation.loop_batch_ns)),
            tally_iterations: allocation.iterations,
        });
        Ok(self)
    }
    pub fn calibrate(self) -> Result<Calibration> {
        let mut minimum = u128::MAX;
        let mut resolution = None::<u128>;
        for _ in 0..256 {
            let elapsed = self.start().elapsed_ns()?;
            minimum = minimum.min(elapsed);
            if elapsed > 0 {
                resolution = Some(resolution.map_or(elapsed, |old| old.min(elapsed)));
            }
        }
        let (resolution, resolution_trials, resolution_delay_iterations) =
            probe_resolution(resolution, |delay| {
                let start = self.start();
                for index in 0..delay {
                    std::hint::black_box(index);
                }
                start.elapsed_ns()
            })?;
        let mut loop_batch_ns = u128::MAX;
        for _ in 0..100 {
            let start = self.start();
            for index in 0..10_000u64 {
                std::hint::black_box(index);
            }
            loop_batch_ns = loop_batch_ns.min(start.elapsed_ns()?);
        }
        Ok(Calibration {
            clock: self.name().into(),
            frequency_hz: self.frequency_hz(),
            trials: 256,
            observed_resolution_ns: resolution,
            resolution_trials,
            resolution_delay_iterations,
            empty_interval_ns: minimum,
            loop_batch_ns,
            loop_iterations: 10_000,
            loop_trials: 100,
        })
    }
    /// Process-wide diagnostics, collected once per clock outside workloads.
    pub(crate) fn cached_calibration(self) -> Result<Calibration> {
        static OS: OnceLock<std::result::Result<Calibration, String>> = OnceLock::new();
        static CPU: OnceLock<std::result::Result<Calibration, String>> = OnceLock::new();
        let cache = match self.clock {
            Clock::Os => &OS,
            Clock::Cpu(_) => &CPU,
            #[cfg(test)]
            Clock::Test(_) => return self.calibrate(),
        };
        cache
            .get_or_init(|| self.calibrate().map_err(|err| err.to_string()))
            .clone()
            .map_err(error)
    }
}

/// A coarse or stalled clock must not make calibration loop forever. Once
/// immediate reads fail, try 32 intervals at each power-of-two delay up to 1024.
/// Delayed intervals estimate observed resolution, not empty-read overhead.
fn probe_resolution(
    initial: Option<u128>,
    mut measure: impl FnMut(u32) -> Result<u128>,
) -> Result<(Option<u128>, u32, u32)> {
    if initial.is_some() {
        return Ok((initial, 256, 0));
    }
    let mut trials = 256;
    for power in 0..=10 {
        let delay = 1 << power;
        let mut minimum = None::<u128>;
        for _ in 0..32 {
            let elapsed = measure(delay)?;
            trials += 1;
            if elapsed > 0 {
                minimum = Some(minimum.map_or(elapsed, |old| old.min(elapsed)));
            }
        }
        if minimum.is_some() {
            return Ok((minimum, trials, delay));
        }
    }
    Ok((None, trials, 1024))
}
impl TimerStart {
    pub(crate) fn measure(self, operations: u64) -> Result<crate::timing::Measured> {
        self.timer.measured(self.elapsed_ns()?, operations, None)
    }
    pub fn elapsed_ns(self) -> Result<u128> {
        match (self.timer.clock, self.stamp) {
            (Clock::Os, Stamp::Os(start)) => Ok(start.elapsed().as_nanos()),
            (Clock::Cpu(hz), Stamp::Cpu(start)) => ticks_to_ns(start, cpu::end(), hz),
            #[cfg(test)]
            (Clock::Test(value), Stamp::Test) => {
                value.ok_or_else(|| error("injected clock failure"))
            }
            _ => unreachable!("timer stamp is constructed with its clock"),
        }
    }
    pub fn elapsed(self) -> Result<Duration> {
        let nanos = self.elapsed_ns()?;
        Ok(Duration::new(
            (nanos / 1_000_000_000)
                .try_into()
                .map_err(|_| error("timer duration exceeds Duration range"))?,
            (nanos % 1_000_000_000) as u32,
        ))
    }
}
fn ticks_to_ns(start: u64, end: u64, frequency: NonZeroU64) -> Result<u128> {
    let ticks = end
        .checked_sub(start)
        .ok_or_else(|| error("CPU counter moved backwards or wrapped"))?;
    Ok(u128::from(ticks) * 1_000_000_000 / u128::from(frequency.get()))
}

#[cfg(all(
    not(miri),
    target_arch = "aarch64",
    any(target_os = "macos", target_os = "linux")
))]
mod cpu {
    pub fn frequency() -> Result<u64, String> {
        let frequency: u64;
        // EL0 architectural virtual counter frequency, configured by the OS.
        unsafe {
            std::arch::asm!("mrs {value}, cntfrq_el0", value = out(reg) frequency, options(nomem, nostack, preserves_flags));
        }
        Ok(frequency)
    }
    pub fn start() -> u64 {
        read()
    }
    pub fn end() -> u64 {
        read()
    }
    fn read() -> u64 {
        let ticks: u64;
        // ISB orders the counter read; omitting nomem also orders compiler memory operations.
        unsafe {
            std::arch::asm!("isb", "mrs {value}, cntvct_el0", "isb", value = out(reg) ticks, options(nostack, preserves_flags));
        }
        ticks
    }
}
#[cfg(all(not(miri), any(target_arch = "x86", target_arch = "x86_64")))]
mod cpu {
    #[cfg(target_arch = "x86")]
    use std::arch::x86 as arch;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64 as arch;
    use std::{
        sync::atomic::{Ordering, compiler_fence},
        time::{Duration, Instant},
    };
    pub fn frequency() -> Result<u64, String> {
        // CPUID is unprivileged; no timestamp instructions run before validation.
        if !std::is_x86_feature_detected!("sse2")
            || arch::__cpuid(1).edx & (1 << 4) == 0
            || arch::__cpuid(0x8000_0000).eax < 0x8000_0007
            || arch::__cpuid(0x8000_0001).edx & (1 << 27) == 0
        {
            return Err("CPU timer requires SSE2, TSC and RDTSCP".into());
        }
        if arch::__cpuid(0x8000_0007).edx & (1 << 8) == 0 {
            return Err("CPU timer requires an invariant timestamp counter".into());
        }
        let mut frequencies = [0u64; 5];
        for value in &mut frequencies {
            let wall = Instant::now();
            let first = start();
            std::thread::sleep(Duration::from_millis(10));
            let last = end();
            let nanos = wall.elapsed().as_nanos();
            let ticks = last
                .checked_sub(first)
                .ok_or("CPU counter moved backwards during calibration")?;
            if ticks == 0 || nanos == 0 {
                return Err("CPU counter did not advance during calibration".into());
            }
            *value = (u128::from(ticks) * 1_000_000_000 / nanos)
                .try_into()
                .map_err(|_| "CPU frequency overflow")?;
        }
        frequencies.sort_unstable();
        Ok(frequencies[2])
    }
    pub fn start() -> u64 {
        compiler_fence(Ordering::SeqCst);
        let ticks = unsafe {
            arch::_mm_lfence();
            let t = arch::_rdtsc();
            arch::_mm_lfence();
            t
        };
        compiler_fence(Ordering::SeqCst);
        ticks
    }
    pub fn end() -> u64 {
        compiler_fence(Ordering::SeqCst);
        let ticks = unsafe {
            let t = arch::__rdtscp(&mut 0);
            arch::_mm_lfence();
            t
        };
        compiler_fence(Ordering::SeqCst);
        ticks
    }
}
#[cfg(any(
    miri,
    not(any(
        any(target_arch = "x86", target_arch = "x86_64"),
        all(target_arch = "aarch64", any(target_os = "macos", target_os = "linux"))
    ))
))]
mod cpu {
    pub fn frequency() -> Result<u64, String> {
        Err("CPU timer is unavailable on this platform".into())
    }
    pub fn start() -> u64 {
        unreachable!("unsupported CPU timer cannot be constructed")
    }
    pub fn end() -> u64 {
        unreachable!("unsupported CPU timer cannot be constructed")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolution_probe_handles_coarse_stalled_and_failed_clocks() {
        assert_eq!(
            probe_resolution(Some(7), |_| panic!("no retry needed")).unwrap(),
            (Some(7), 256, 0)
        );
        let mut calls = 0;
        let found = probe_resolution(None, |delay| {
            calls += 1;
            Ok(if delay < 8 {
                0
            } else if calls % 2 == 0 {
                100
            } else {
                200
            })
        })
        .unwrap();
        assert_eq!(found, (Some(100), 384, 8));
        assert_eq!(calls, 128);
        let mut max_delay = 0;
        let missing = probe_resolution(None, |delay| {
            max_delay = max_delay.max(delay);
            Ok(0)
        })
        .unwrap();
        assert_eq!(missing, (None, 608, 1024));
        assert_eq!(max_delay, 1024);
        let mut calls = 0;
        let failure = probe_resolution(None, |_| {
            calls += 1;
            if calls == 7 {
                Err(error("counter failed"))
            } else {
                Ok(0)
            }
        })
        .unwrap_err();
        assert!(failure.to_string().contains("counter failed"));
        assert_eq!(calls, 7);
    }

    #[test]
    fn zero_clock_keeps_zero_overhead_and_reports_unknown_resolution() {
        let calibration = Timer::fixed(Some(0)).calibrate().unwrap();
        assert_eq!(calibration.empty_interval_ns, 0);
        assert_eq!(calibration.observed_resolution_ns, None);
        assert_eq!(calibration.resolution_trials, 608);
        assert_eq!(calibration.resolution_delay_iterations, 1024);
        assert_eq!(calibration.loop_batch_ns, 0);
    }
    #[test]
    fn clock_selection_parses_without_hardware_access() {
        assert_eq!(TimerKind::default(), TimerKind::Os);
        assert_eq!(TimerKind::parse("os").unwrap(), TimerKind::Os);
        assert_eq!(TimerKind::parse("cpu").unwrap(), TimerKind::Cpu);
        assert_eq!(TimerKind::Cpu.name(), "cpu");
        for invalid in ["", "CPU", "tsc", "auto", " os"] {
            assert!(TimerKind::parse(invalid).is_err());
        }
        assert_eq!(TimerKind::Os.resolve().unwrap().name(), "os");
    }
    #[test]
    fn exact_tick_conversion_handles_large_values_zero_and_backwards_counter() {
        let hz = NonZeroU64::new(24_000_000).unwrap();
        assert_eq!(ticks_to_ns(100, 124, hz).unwrap(), 1000);
        assert_eq!(ticks_to_ns(100, 100, hz).unwrap(), 0);
        assert!(ticks_to_ns(101, 100, hz).is_err());
        assert_eq!(
            ticks_to_ns(0, u64::MAX, NonZeroU64::new(1).unwrap()).unwrap(),
            u128::from(u64::MAX) * 1_000_000_000
        );
    }
    #[test]
    fn os_calibration_and_elapsed_interval() {
        let timer = Timer::os();
        let c = timer.calibrate().unwrap();
        assert_eq!(c.clock, "os");
        assert_eq!(c.trials, 256);
        assert_eq!(c.frequency_hz, None);
        let start = timer.start();
        std::thread::sleep(Duration::from_millis(2));
        assert!(start.elapsed().unwrap() >= Duration::from_millis(1));
    }
    #[test]
    fn cpu_clock_matches_monotonic_elapsed_when_available() {
        let timer = match Timer::cpu() {
            Ok(timer) => timer,
            Err(reason) => {
                eprintln!("CPU timer unavailable: {reason}");
                return;
            }
        };
        assert!(timer.frequency_hz().unwrap() > 0);
        let c = timer.calibrate().unwrap();
        assert_eq!(c.clock, "cpu");
        let wall = Instant::now();
        let start = timer.start();
        std::thread::sleep(Duration::from_millis(5));
        let cpu = start.elapsed_ns().unwrap();
        let wall = wall.elapsed().as_nanos();
        assert!(
            cpu > wall / 2 && cpu < wall * 2,
            "CPU {cpu} ns vs OS {wall} ns"
        );
    }

    #[test]
    fn cpu_frequency_matches_os_bracketed_intervals() {
        let timer = match Timer::cpu() {
            Ok(timer) => timer,
            Err(error) => {
                eprintln!("CPU timer unavailable: {error}");
                return;
            }
        };
        for _ in 0..5 {
            let before_start = Instant::now();
            let start = timer.start();
            let after_start = Instant::now();
            std::thread::sleep(Duration::from_millis(5));
            let before_end = Instant::now();
            let measured = start.elapsed_ns().unwrap();
            let after_end = Instant::now();
            let lower = before_end.duration_since(after_start).as_nanos();
            let upper = after_end.duration_since(before_start).as_nanos();
            // Brackets account for scheduling around clock reads. The margin
            // allows 1% frequency calibration error plus 1us quantization.
            let margin = upper / 100 + 1000;
            assert!(
                measured >= lower.saturating_sub(margin) && measured <= upper + margin,
                "CPU {measured}ns outside OS bracket {lower}..{upper}ns (margin {margin})"
            );
        }
    }

    #[test]
    fn common_start_can_be_read_by_all_workers() {
        // Suite worker measurements share one start; check this property before
        // integrating the CPU clock into the synchronized worker gate.
        let mut timers = vec![Timer::os()];
        if let Ok(cpu) = Timer::cpu() {
            timers.push(cpu);
        }
        for timer in timers {
            let start = timer.start();
            std::thread::scope(|scope| {
                let handles: Vec<_> = (0..4)
                    .map(|_| scope.spawn(move || start.elapsed_ns()))
                    .collect();
                let observations: Vec<_> = handles
                    .into_iter()
                    .map(|handle| handle.join().unwrap().unwrap())
                    .collect();
                let end = start.elapsed_ns().unwrap();
                for elapsed in observations {
                    assert!(
                        elapsed <= end,
                        "{} counter is inconsistent across workers",
                        timer.name()
                    );
                }
            });
        }
    }
}
