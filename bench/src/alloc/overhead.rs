//! Isolated calibration of the profiler's bookkeeping, without allocator calls.
use super::{ThreadStats, TrackingAllocator};
use crate::{Result, error, timer::Timer};
use std::hint::black_box;

/// Minimum batch costs for allocation instrumentation. All batches contain the
/// same number of events; keep ratios intact to preserve fractional nanoseconds.
/// Estimates are local to one uncontended thread, not a contention model.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AllocationOverhead {
    pub clock: String,
    pub trials: u32,
    pub iterations: u64,
    pub loop_batch_ns: u128,
    pub alloc_batch_ns: u128,
    pub dealloc_batch_ns: u128,
    pub grow_batch_ns: u128,
    pub shrink_batch_ns: u128,
}
impl AllocationOverhead {
    /// Estimated tally cost, excluding the calibration loop. This does not
    /// include sample-loop overhead or real allocator costs. Round once after
    /// adding event costs, and reject arithmetic overflow rather than wrap.
    pub fn tally_ns(&self, stats: &ThreadStats) -> Result<u128> {
        if self.iterations == 0 || stats.overflowed {
            return Err(error("invalid allocation overhead calibration or counters"));
        }
        if stats.grow_operations.checked_add(stats.shrink_operations) != Some(stats.reallocations) {
            return Err(error(
                "allocation overhead requires consistent realloc counters",
            ));
        }
        let mut total = 0u128;
        for (batch, count) in [
            (self.alloc_batch_ns, stats.allocations),
            (self.dealloc_batch_ns, stats.deallocations),
            (self.grow_batch_ns, stats.grow_operations),
            (self.shrink_batch_ns, stats.shrink_operations),
        ] {
            let term = batch
                .saturating_sub(self.loop_batch_ns)
                .checked_mul(count)
                .ok_or_else(|| error("allocation overhead estimate overflow"))?;
            total = total
                .checked_add(term)
                .ok_or_else(|| error("allocation overhead estimate overflow"))?;
        }
        Ok(total / u128::from(self.iterations))
    }
}

/// Measure the exact bookkeeping helpers used after successful allocator calls.
/// Timed loops use a private scratch tracker and never invoke an underlying
/// allocator. Synthetic events do not alter application allocation totals. Rejects nested TLS phases.
/// All temporary phase state is released on errors and unwinding.
pub fn calibrate_overhead(timer: Timer) -> Result<AllocationOverhead> {
    const TRIALS: u32 = 100;
    const ITERATIONS: u64 = 10_000;
    fn sample(timer: Timer, operation: impl Fn(&TrackingAllocator<()>, usize)) -> Result<u128> {
        let mut minimum = u128::MAX;
        for _ in 0..TRIALS {
            let scratch = TrackingAllocator::new(());
            // Enough synthetic live bytes for free/shrink bookkeeping; this
            // seeds counters only, outside both the TLS phase and timed loop.
            scratch.allocated(ITERATIONS as usize * 128);
            let phase = scratch.begin_thread_phase()?;
            let start = timer.start();
            for index in 0..ITERATIONS {
                black_box(index);
                operation(&scratch, black_box(64));
            }
            let elapsed = start.elapsed_ns()?;
            black_box(phase.finish());
            black_box(scratch.snapshot());
            minimum = minimum.min(elapsed);
        }
        Ok(minimum)
    }
    Ok(AllocationOverhead {
        clock: timer.name().into(),
        trials: TRIALS,
        iterations: ITERATIONS,
        loop_batch_ns: sample(timer, |_, _| ())?,
        alloc_batch_ns: sample(timer, |tracker, size| tracker.allocated(size))?,
        dealloc_batch_ns: sample(timer, |tracker, size| tracker.deallocated(size))?,
        grow_batch_ns: sample(timer, |tracker, size| tracker.resized(size, size * 2))?,
        shrink_batch_ns: sample(timer, |tracker, size| tracker.resized(size * 2, size))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn known() -> AllocationOverhead {
        AllocationOverhead {
            clock: "os".into(),
            trials: 100,
            iterations: 10,
            loop_batch_ns: 10,
            alloc_batch_ns: 11,
            dealloc_batch_ns: 12,
            grow_batch_ns: 13,
            shrink_batch_ns: 14,
        }
    }
    #[test]
    fn fractional_event_costs_round_once_and_realloc_is_not_counted_twice() {
        let stats = ThreadStats {
            allocations: 3,
            deallocations: 2,
            grow_operations: 1,
            shrink_operations: 2,
            reallocations: 3,
            ..Default::default()
        };
        // (3*1 + 2*2 + 1*3 + 2*4)/10 = 1.8ns, rounded only at the end.
        assert_eq!(known().tally_ns(&stats).unwrap(), 1);
        let mut noisy = known();
        noisy.alloc_batch_ns = 0;
        assert_eq!(
            noisy
                .tally_ns(&ThreadStats {
                    allocations: 100,
                    ..Default::default()
                })
                .unwrap(),
            0
        );
        let invalid = ThreadStats {
            reallocations: 1,
            ..Default::default()
        };
        assert!(known().tally_ns(&invalid).is_err());
        let overflow = ThreadStats {
            deallocations: u128::MAX,
            ..Default::default()
        };
        assert!(known().tally_ns(&overflow).is_err());
        let mut zero = known();
        zero.iterations = 0;
        assert!(zero.tally_ns(&ThreadStats::default()).is_err());
    }

    #[test]
    fn calibration_is_isolated_and_releases_phase_after_clock_error() {
        let app = TrackingAllocator::new(());
        app.allocated(128);
        let before = app.snapshot();
        let active = app.begin_thread_phase().unwrap();
        assert!(calibrate_overhead(Timer::fixed(Some(1))).is_err());
        assert_eq!(active.finish(), ThreadStats::default());
        assert_eq!(app.snapshot(), before);
        assert!(calibrate_overhead(Timer::fixed(None)).is_err());
        let phase = app.begin_thread_phase().unwrap();
        assert_eq!(phase.finish(), ThreadStats::default());
        let calibration = calibrate_overhead(Timer::fixed(Some(37))).unwrap();
        assert_eq!(calibration.loop_batch_ns, 37);
        assert_eq!(calibration.alloc_batch_ns, 37);
        assert_eq!(calibration.dealloc_batch_ns, 37);
        assert_eq!(calibration.grow_batch_ns, 37);
        assert_eq!(calibration.shrink_batch_ns, 37);
        assert_eq!(app.snapshot(), before);
        let measured = calibrate_overhead(Timer::os()).unwrap();
        assert_eq!(measured.clock, "os");
        assert_eq!(measured.iterations, 10_000);
        assert_eq!(measured.trials, 100);
        assert_eq!(app.snapshot(), before);
        assert_eq!(
            app.begin_thread_phase().unwrap().finish(),
            ThreadStats::default()
        );
    }
}
