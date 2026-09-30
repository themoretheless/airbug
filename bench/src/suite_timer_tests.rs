use super::*;
use crate::{alloc::TrackingAllocator, timer::Timer, workloads::LocalExecutor};
use std::alloc::System;

#[test]
fn selected_clock_reaches_local_async_allocated_and_worker_boundaries() {
    let allocator = TrackingAllocator::new(System);
    for drop in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
        let mut suite = Suite::new("clock");
        let batch = BatchPolicy::Iterations(std::num::NonZeroU64::new(32).unwrap());
        suite.bench("one_plain", || ());
        suite.bench_fixture("one_fixture", crate::Fixture::new(|| ()), |_| ());
        suite.bench_allocated("one_allocated", &allocator, || ());
        suite.bench_async_allocated(
            "one_async_allocated",
            &allocator,
            || LocalExecutor,
            async || (),
        );
        suite.bench_async("one_async", LocalExecutor, || async {});
        suite.bench_with_input("two_ref", || (), |_| (), drop);
        suite.bench_with_owned_input("two_owned", || (), |_| (), drop);
        suite.bench_checked("two_checked", || (), |_| (), |_, _| Ok(()), drop);
        suite.bench_batched_ref("three_ref", || (), |_| (), drop, batch);
        suite.bench_batched("three_owned", || (), |_| (), drop, batch);
        suite.bench_async_factory("two_async_factory", || LocalExecutor, || async {}, drop);
        suite.bench_async_with_input("two_async_ref", || LocalExecutor, || (), async |_| (), drop);
        suite.bench_async_with_value(
            "two_async_value",
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
        );
        suite.bench_async_with_owned_input(
            "two_async_owned",
            || LocalExecutor,
            || (),
            |_| async {},
            drop,
        );
        suite.bench_async_batched_ref(
            "three_async_ref",
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
            batch,
        );
        suite.bench_async_batched(
            "three_async_owned",
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
            batch,
        );
        suite.bench_allocated_batched_ref(
            "three_allocated_ref",
            &allocator,
            || (),
            |_| (),
            drop,
            batch,
        );
        suite.bench_allocated_batched(
            "three_allocated_owned",
            &allocator,
            || (),
            |_| (),
            drop,
            batch,
        );
        suite.bench_async_allocated_batched_ref(
            "three_async_allocated_ref",
            &allocator,
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
            batch,
        );
        suite.bench_async_allocated_batched(
            "three_async_allocated_owned",
            &allocator,
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
            batch,
        );
        suite.bench_threads("one_workers", 2, || ());
        suite.bench_threads_with_input("two_workers_ref", 2, || (), |_| (), drop);
        suite.bench_threads_with_owned_input("two_workers_owned", 2, || (), |_| (), drop);
        suite.bench_threads_with_local_input("two_local_ref", 2, || (), |_| (), drop);
        suite.bench_threads_with_local_owned_input("two_local_owned", 2, || (), |_| (), drop);
        suite.bench_async_threads("two_async_workers", 2, || LocalExecutor, async || (), drop);
        suite.bench_async_threads_with_input(
            "two_async_workers_ref",
            2,
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
        );
        suite.bench_async_threads_with_owned_input(
            "two_async_workers_owned",
            2,
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
        );
        suite.bench_threads_allocated_with_input(
            "two_allocated_workers_ref",
            &allocator,
            2,
            || (),
            |_| (),
            drop,
        );
        suite.bench_threads_allocated_with_owned_input(
            "two_allocated_workers_owned",
            &allocator,
            2,
            || (),
            |_| (),
            drop,
        );
        suite.bench_threads_allocated_with_local_input(
            "two_allocated_local_ref",
            &allocator,
            2,
            || (),
            |_| (),
            drop,
        );
        suite.bench_threads_allocated_with_local_owned_input(
            "two_allocated_local_owned",
            &allocator,
            2,
            || (),
            |_| (),
            drop,
        );
        suite.bench_async_threads_allocated_with_input(
            "two_async_allocated_workers_ref",
            &allocator,
            2,
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
        );
        suite.bench_async_threads_allocated_with_owned_input(
            "two_async_allocated_workers_owned",
            &allocator,
            2,
            || LocalExecutor,
            || (),
            async |_| (),
            drop,
        );
        suite.bench_custom("custom", |_| Duration::from_nanos(17));
        suite.bench_async_custom(
            "custom_async",
            || LocalExecutor,
            async |_| Duration::from_nanos(17),
        );
        for entry in &mut suite.entries {
            entry.sampling.iterations = Some(65);
        }
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 65,
        });
        // Selecting after registration and changing between runs must affect
        // captured closures, all helpers and worker timestamps alike.
        for (tick, compensate) in [(37u128, false), (91, false), (37, true), (91, true)] {
            suite.compensate_overhead(compensate);
            suite.timer(if compensate {
                Timer::fixed_corrected(
                    tick,
                    crate::timing::Correction {
                        loop_batch_ns: 2,
                        loop_iterations: 1,
                        tally_batch_ns: [0; 4],
                        tally_iterations: 1,
                    },
                )
            } else {
                Timer::fixed(Some(tick))
            });
            let run = suite.run("").unwrap();
            run.validate().unwrap();
            for observation in run.observations.iter().filter(|o| o.metric == "wall") {
                let name = observation.case.strip_prefix("clock/").unwrap();
                let expected = if name.starts_with("one_") {
                    tick
                } else if name.starts_with("two_") {
                    2 * tick
                } else if name.starts_with("three_") {
                    3 * tick
                } else {
                    17
                };
                assert_eq!(
                    observation.value.as_deref(),
                    Some(expected.to_string().as_str()),
                    "{name}"
                );
                let case = run.cases.iter().find(|c| c.id == observation.case).unwrap();
                assert_eq!(
                    case.contract["timer.clock"],
                    if name.starts_with("custom") {
                        "caller"
                    } else {
                        "test"
                    }
                );
            }
            let adjusted: Vec<_> = run
                .observations
                .iter()
                .filter(|o| o.metric == "wall.adjusted")
                .collect();
            assert_eq!(adjusted.len(), if compensate { 68 } else { 0 });
            for observation in adjusted {
                let name = observation.case.strip_prefix("clock/").unwrap();
                let expected = if name.starts_with("one_") {
                    tick.saturating_sub(130)
                } else if name.starts_with("two_") {
                    tick.saturating_sub(128) + tick.saturating_sub(2)
                } else {
                    2 * tick.saturating_sub(64) + tick.saturating_sub(2)
                };
                assert_eq!(
                    observation.value.as_deref(),
                    Some(expected.to_string().as_str()),
                    "{name}"
                );
            }
            for record in &run.worker_allocations {
                assert_eq!(record.wall_ns, tick.to_string());
                assert_eq!(
                    record.adjusted_wall_ns,
                    compensate.then(|| tick
                        .saturating_sub(record.operations as u128 * 2)
                        .to_string())
                );
            }
        }
    }
}

#[test]
fn failed_timer_releases_workers_and_allocation_phases() {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let allocator = TrackingAllocator::new(System);
        for local in [false, true] {
            let mut suite = Suite::new("errors");
            if local {
                suite.bench_threads_allocated_with_local_input(
                    "worker",
                    &allocator,
                    4,
                    || (),
                    |_| (),
                    DropPolicy::OutsideTiming,
                );
            } else {
                suite.bench_threads_allocated_with_input(
                    "worker",
                    &allocator,
                    4,
                    || (),
                    |_| (),
                    DropPolicy::OutsideTiming,
                );
            }
            suite.timer(Timer::fixed(None));
            let error = suite
                .test_selected(&crate::Selection::default())
                .unwrap_err();
            assert!(error.to_string().contains("injected clock failure"));
            suite.timer(Timer::fixed(Some(7)));
            let run = suite.test_selected(&crate::Selection::default()).unwrap();
            run.validate().unwrap();
            assert_eq!(run.observations[0].value.as_deref(), Some("7"));
        }
        tx.send(()).unwrap();
    });
    rx.recv_timeout(Duration::from_secs(5))
        .expect("clock failure must release all workers");
    handle.join().unwrap();
}

#[test]
fn calibration_reports_fractional_loop_cost_without_changing_samples() {
    let mut suite = Suite::new("calibration");
    suite.timer(Timer::fixed(Some(37)));
    suite.bench("raw", || ());
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.provenance["timer.test.calibration.loop_batch_ns"], "37");
    assert_eq!(
        run.provenance["timer.test.calibration.loop_iterations"],
        "10000"
    );
    assert_eq!(
        run.provenance["timer.test.calibration.empty_interval_ns"],
        "37"
    );
    assert_eq!(run.observations[0].value.as_deref(), Some("37"));
    assert_eq!(
        run.provenance["timer.test.calibration.resolution_status"],
        "observed"
    );
    suite.timer(Timer::fixed(Some(0)));
    let zero = suite.run("").unwrap();
    assert_eq!(
        zero.provenance["timer.test.calibration.resolution_status"],
        "unresolved"
    );
    assert_eq!(
        zero.provenance["timer.test.calibration.resolution_trials"],
        "608"
    );
    assert!(
        !zero
            .provenance
            .contains_key("timer.test.calibration.observed_resolution_ns")
    );
    assert_eq!(zero.observations[0].value.as_deref(), Some("0"));
    let smoke = suite.test_selected(&crate::Selection::default()).unwrap();
    assert!(
        !smoke
            .provenance
            .keys()
            .any(|key| key.contains(".calibration."))
    );
    let mut cold = Suite::new("cold");
    cold.compensate_overhead(true);
    cold.timer(Timer::fixed(Some(37)))
        .bench("single", || ())
        .cold();
    let run = cold.run("").unwrap();
    assert_eq!(run.cases[0].contract["timer.overhead_policy"], "raw");
    assert!(!run.observations.iter().any(|o| o.metric == "wall.adjusted"));
    assert!(
        !run.provenance
            .keys()
            .any(|key| key.contains(".calibration."))
    );
    let mut custom = Suite::new("custom");
    // A failed diagnostic clock must not affect caller-provided durations.
    custom
        .timer(Timer::fixed(None))
        .bench_custom("caller", |_| Duration::from_nanos(123));
    custom.sampling(Sampling {
        samples: Some(1),
        iterations: Some(1),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let run = custom.run("").unwrap();
    assert_eq!(run.observations[0].value.as_deref(), Some("123"));
    assert!(
        !run.provenance
            .keys()
            .any(|key| key.contains(".calibration."))
    );
}
