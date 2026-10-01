use airbug_bench::{Config, DropPolicy, Sampling, Suite};
use std::{rc::Rc, time::Duration};

#[test]
fn local_worker_timings_preserve_waves_runs_and_selected_baselines() {
    let mut suite = Suite::new("timings");
    for name in ["first", "second"] {
        suite.bench_threads_with_local_input(
            name,
            2,
            || Rc::new(1),
            |value| value.clone(),
            DropPolicy::OutsideTiming,
        );
        suite.sampling(Sampling {
            iterations: Some(65),
            worker_start: Some(airbug_bench::timer::WorkerStart::Local),
            ..Default::default()
        });
    }
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert!(run.worker_allocations.is_empty());
    assert_eq!(run.worker_timings.len(), 16);
    assert!(
        run.worker_timings
            .iter()
            .all(|w| w.operations == 64 || w.operations == 1)
    );
    assert_eq!(
        run.worker_timings.iter().map(|w| w.operations).sum::<u64>(),
        520
    );
    run.validate().unwrap();
    let decoded: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
    decoded.validate().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = airbug_bench::baseline::Store::new(root.path());
    store
        .save_cases("timings", &run, airbug_bench::baseline::SaveMode::Create)
        .unwrap();
    let selected = store
        .load_selected_runs("timings", &["timings/first"])
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].worker_timings.len(), 8);
    assert!(
        selected[0]
            .worker_timings
            .iter()
            .all(|w| w.case == "timings/first")
    );
    selected[0].validate().unwrap();
    suite.thread_count(3).unwrap();
    let next = suite.run("first").unwrap();
    assert_eq!(next.worker_timings.len(), 12);
    assert!(
        next.worker_timings
            .iter()
            .all(|w| w.worker < 3 && w.sequence < 2)
    );
    assert_eq!(
        next.worker_timings
            .iter()
            .map(|w| w.operations)
            .sum::<u64>(),
        390
    );
    next.validate().unwrap();
}

#[test]
fn descriptive_worker_times_are_normalized_and_separated_by_process() {
    let mut suite = Suite::new("distribution");
    suite.bench_threads_with_local_input("case", 2, || (), |_| (), DropPolicy::InsideTiming);
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    let mut run = suite.run("").unwrap();
    for worker in &mut run.worker_timings {
        worker.wall_ns = (worker.operations * (10 + 10 * worker.worker)).to_string();
    }
    run.observations[0].value = Some("1300".into());
    let mut second = run.clone();
    for worker in &mut second.worker_timings {
        worker.process = 1;
        worker.wall_ns = (worker.wall_ns.parse::<u64>().unwrap() * 2).to_string();
    }
    for observation in &mut second.observations {
        observation.process = 1;
        observation.value = Some("2600".into());
    }
    run.worker_timings.extend(second.worker_timings);
    run.observations.extend(second.observations);
    let before = serde_json::to_vec(&run).unwrap();
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    assert_eq!(rows.len(), 2);
    for row in &rows {
        let scale = f64::from(row.process + 1);
        let summary = row.worker_wall_per_operation.as_ref().unwrap();
        assert_eq!(summary.count, 2);
        assert_eq!(
            (
                summary.minimum,
                summary.maximum,
                summary.mean,
                summary.median
            ),
            (10.0 * scale, 20.0 * scale, 15.0 * scale, 15.0 * scale)
        );
        assert_eq!(row.worker_operation_weighted_mean, Some(15.0 * scale));
        assert!(row.worker_allocations.is_none());
    }
    let numerical = airbug_bench::report::html_run_without_plots(&run).unwrap();
    assert!(numerical.contains("id=\"worker-timings\""));
    assert!(numerical.contains("Case contracts"));
    assert!(!numerical.contains("<svg"));
    assert!(!numerical.contains("worker-distributions"));
    assert!(!numerical.contains("<!--CHARTS-->"));
    let html = airbug_bench::report::html_run(&run).unwrap();
    assert!(html.contains("id=\"worker-timings\""));
    assert!(html.contains("id=\"worker-distributions\""));
    assert!(html.contains("distribution/case / candidate / process 0"));
    assert!(html.contains("distribution/case / candidate / process 1"));
    let charts = airbug_bench::report::worker_timing_charts(&run).unwrap();
    assert_eq!(charts.matches("<svg").count(), 2);
    assert!(html.contains(
        "<td>0</td><td>2</td><td>10.0000</td><td>20.0000</td><td>15.0000</td><td>15.0000</td>"
    ));
    assert!(html.contains(
        "<td>1</td><td>2</td><td>20.0000</td><td>40.0000</td><td>30.0000</td><td>30.0000</td>"
    ));
    let markdown = airbug_bench::report::descriptive_markdown(&rows);
    assert!(markdown.contains("Worker wall time per operation"));
    assert!(markdown.contains("| 0 | 2 | 10.0000 | 20.0000 | 15.0000 | 15.0000 |"));
    assert!(markdown.contains("| 1 | 2 | 20.0000 | 40.0000 | 30.0000 | 30.0000 |"));
    assert_eq!(before, serde_json::to_vec(&run).unwrap());
    let mut legacy = serde_json::to_value(&rows[0]).unwrap();
    legacy
        .as_object_mut()
        .unwrap()
        .remove("worker_wall_per_operation");
    let decoded: airbug_bench::report::DescriptiveRow = serde_json::from_value(legacy).unwrap();
    assert!(decoded.worker_wall_per_operation.is_none());
}

#[test]
fn coordinator_inputs_record_every_wave_without_moving_setup_or_drop() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let coordinator = std::thread::current().id();
    let prepared = AtomicUsize::new(0);
    let dropped = AtomicUsize::new(0);
    struct Input<'a>(&'a AtomicUsize, std::thread::ThreadId);
    impl Drop for Input<'_> {
        fn drop(&mut self) {
            assert_eq!(std::thread::current().id(), self.1);
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let mut suite = Suite::new("coordinator");
    suite.bench_threads_with_input(
        "borrowed",
        2,
        || {
            assert_eq!(std::thread::current().id(), coordinator);
            prepared.fetch_add(1, Ordering::SeqCst);
            Input(&dropped, coordinator)
        },
        |_| (),
        DropPolicy::OutsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    // Return the owned input so destruction follows the outside-timing policy.
    suite.bench_threads_with_owned_input(
        "owned",
        2,
        || {
            assert_eq!(std::thread::current().id(), coordinator);
            prepared.fetch_add(1, Ordering::SeqCst);
            Input(&dropped, coordinator)
        },
        |input| input,
        DropPolicy::OutsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.worker_timings.len(), 16);
    assert!(run.worker_allocations.is_empty());
    assert_eq!(prepared.load(Ordering::SeqCst), 520);
    assert_eq!(dropped.load(Ordering::SeqCst), 520);
    run.validate().unwrap();
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    assert!(
        rows.iter()
            .all(|row| row.worker_wall_per_operation.as_ref().unwrap().count == 4)
    );
}

#[test]
fn async_worker_timings_include_pending_futures_and_keep_local_values_on_workers() {
    use airbug_bench::workloads::LocalExecutor;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let dropped = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    struct Local<'a>(Rc<()>, std::thread::ThreadId, &'a AtomicUsize);
    impl Drop for Local<'_> {
        fn drop(&mut self) {
            assert_eq!(self.1, std::thread::current().id());
            self.2.fetch_add(1, Ordering::SeqCst);
        }
    }
    async fn yield_once() {
        let mut pending = true;
        std::future::poll_fn(|cx| {
            if std::mem::take(&mut pending) {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            } else {
                std::task::Poll::Ready(())
            }
        })
        .await
    }
    let mut suite = Suite::new("async-times");
    suite.bench_async_threads_with_input(
        "borrowed",
        2,
        || LocalExecutor,
        || Local(Rc::new(()), std::thread::current().id(), &dropped),
        async |input| {
            yield_once().await;
            completed.fetch_add(1, Ordering::SeqCst);
            input.0.clone()
        },
        DropPolicy::OutsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    suite.bench_async_threads_with_owned_input(
        "owned",
        2,
        || LocalExecutor,
        || Local(Rc::new(()), std::thread::current().id(), &dropped),
        async |input| {
            yield_once().await;
            completed.fetch_add(1, Ordering::SeqCst);
            input
        },
        DropPolicy::OutsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.worker_timings.len(), 16);
    assert_eq!(completed.load(Ordering::SeqCst), 520);
    assert_eq!(dropped.load(Ordering::SeqCst), 520);
    assert!(run.worker_allocations.is_empty());
    run.validate().unwrap();
}

#[test]
fn plain_manual_threads_record_one_complete_wave_per_sample() {
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let mut suite = Suite::new("plain");
    suite.bench_threads("case", 3, || {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Rc::new(())
    });
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.worker_timings.len(), 6);
    assert!(
        run.worker_timings
            .iter()
            .all(|w| w.wave == 0 && w.operations == 65)
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 390);
    run.validate().unwrap();
}

#[test]
fn worker_slot_samples_sum_unequal_waves_before_normalization() {
    let mut suite = Suite::new("slots");
    suite.bench_threads_with_local_input("case", 2, || (), |_| (), DropPolicy::InsideTiming);
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    suite.sampling(Sampling {
        iterations: Some(65),
        worker_start: Some(airbug_bench::timer::WorkerStart::Local),
        ..Default::default()
    });
    let mut run = suite.run("").unwrap();
    for timing in &mut run.worker_timings {
        // A 64-operation wave at 10 ns/op and a one-operation tail at 100 ns/op.
        timing.wall_ns = if timing.wave == 0 { "640" } else { "100" }.into();
        timing.adjusted_wall_ns = Some(if timing.wave == 0 { "576" } else { "90" }.into());
    }
    for observation in &mut run.observations {
        if observation.metric == "wall" {
            observation.value = Some("740".into());
        }
    }
    let samples = run.worker_slot_samples().unwrap();
    assert_eq!(samples.len(), 4);
    for sample in &samples {
        assert_eq!(sample.waves, 2);
        assert_eq!(sample.operations, 65);
        assert_eq!(sample.wall_ns, "740");
        assert_eq!(sample.adjusted_wall_ns.as_deref(), Some("666"));
        // Equal weighting of waves would incorrectly give 55 ns/op.
        assert!(sample.wall_ns.parse::<f64>().unwrap() / (sample.operations as f64) < 12.0);
    }
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    let summary = rows[0].worker_wall_per_operation.as_ref().unwrap();
    assert_eq!(summary.count, 4);
    assert!((summary.mean - 740.0 / 65.0).abs() < 1e-10);
    let html = airbug_bench::report::html_run_without_plots(&run).unwrap();
    assert!(html.contains("11.3846"));
    let charts = airbug_bench::report::worker_timing_charts(&run).unwrap();
    assert!(charts.contains("4 identical samples"));
    let markdown = airbug_bench::report::descriptive_markdown(&rows);
    assert!(markdown.contains("11.3846"));
    // Eight wave records become four complete worker slots, but statistical
    // resampling still has only two aggregate observations from one process.
    let config = airbug_bench::bootstrap::Config {
        resamples: 32,
        ..Default::default()
    };
    let before = serde_json::to_vec(&run).unwrap();
    let estimates = airbug_bench::bootstrap::analyze(&run, &config).unwrap();
    let wall = estimates
        .rows
        .iter()
        .find(|row| row.metric == "wall")
        .unwrap();
    assert_eq!(wall.resampling_unit, "normalized_observation");
    assert_eq!(wall.units, 2);
    assert!((wall.estimates.as_ref().unwrap().mean.point - 740.0 / 130.0).abs() < 1e-10);
    assert_eq!(before, serde_json::to_vec(&run).unwrap());

    let mut repeated = run.clone();
    let mut second = run.clone();
    for timing in &mut second.worker_timings {
        timing.process = 1;
    }
    for observation in &mut second.observations {
        observation.process = 1;
    }
    repeated.worker_timings.extend(second.worker_timings);
    repeated.observations.extend(second.observations);
    repeated.validate().unwrap();
    assert_eq!(repeated.worker_slot_samples().unwrap().len(), 8);
    let estimates = airbug_bench::bootstrap::analyze(&repeated, &config).unwrap();
    let wall = estimates
        .rows
        .iter()
        .find(|row| row.metric == "wall")
        .unwrap();
    assert_eq!(wall.resampling_unit, "process_median");
    assert_eq!(wall.units, 2);
    assert!((wall.estimates.as_ref().unwrap().mean.point - 740.0 / 130.0).abs() < 1e-10);

    run.worker_timings.reverse();
    assert_eq!(samples, run.worker_slot_samples().unwrap());
    run.worker_timings[0].adjusted_wall_ns = None;
    assert_eq!(
        run.worker_slot_samples()
            .unwrap()
            .iter()
            .filter(|s| s.adjusted_wall_ns.is_none())
            .count(),
        1
    );
    run.worker_timings.pop();
    assert!(run.worker_slot_samples().is_err());
}

#[airbug_bench::group(
    samples = 5,
    sample_count_unit = "workers",
    worker_start = "local",
    threads = 2,
    iterations = 1,
    warmup_ms = 0
)]
mod worker_budget {
    #[bench]
    fn inherited() {}
    #[bench(sample_count_unit = "batches", worker_start = "shared")]
    fn batches() {}
}

#[test]
fn worker_sample_budget_rounds_up_and_tracks_runtime_worker_counts() {
    let mut suite = Suite::new("budget");
    worker_budget::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    for (suffix, batches, slots) in [("inherited/threads=2", 3, 6), ("batches/threads=2", 5, 10)] {
        let case = run.cases.iter().find(|c| c.id.ends_with(suffix)).unwrap();
        assert_eq!(case.contract["samples"], batches.to_string());
        let local = suffix.starts_with("inherited");
        assert_eq!(
            case.contract["threads.timer_start"],
            if local { "local" } else { "shared" }
        );
        assert_eq!(case.metrics[0].scope.contains("worker-local"), local);
        assert_eq!(case.contract["sampling.requested_samples"], "5");
        assert_eq!(
            run.worker_slot_samples()
                .unwrap()
                .iter()
                .filter(|s| s.case == case.id)
                .count(),
            slots
        );
    }
    // Runtime overrides change the budget denominator without changing the request.
    let mut manual = Suite::new("manual-budget");
    manual.bench_threads("case", 2, || ());
    manual.sampling(Sampling {
        samples: Some(5),
        sample_count_unit: Some(airbug_bench::SampleCountUnit::Workers),
        iterations: Some(1),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    manual.thread_count(3).unwrap();
    let run = manual.run("").unwrap();
    assert_eq!(run.cases[0].contract["samples"], "2");
    assert_eq!(run.cases[0].contract["sampling.count_unit"], "workers");
    assert_eq!(run.worker_slot_samples().unwrap().len(), 6);
    manual.thread_count(8).unwrap();
    assert_eq!(
        manual.run("").unwrap().worker_slot_samples().unwrap().len(),
        8
    );
}

#[test]
fn worker_counters_preserve_exact_fixed_and_captured_dynamic_totals() {
    let counters = airbug_bench::counters::InputCounters::new(&["items"]).unwrap();
    let captured = counters.clone();
    let mut suite = Suite::new("worker-counts");
    suite.bench_threads_with_local_input(
        "case",
        2,
        move || {
            captured.add("items", 2);
        },
        |_| (),
        DropPolicy::InsideTiming,
    );
    suite
        .work_units("bytes", u64::MAX)
        .work_units("chars", 0)
        .work_units("items", 7);
    suite.input_counters(counters);
    suite.sampling(Sampling {
        samples: Some(2),
        iterations: Some(65),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    let samples = run.worker_slot_samples().unwrap();
    assert_eq!(samples.len(), 4);
    for sample in &samples {
        assert_eq!(
            sample.work_totals["bytes"],
            (u128::from(u64::MAX) * 65).to_string()
        );
        assert_eq!(sample.work_totals["chars"], "0");
        assert_eq!(sample.work_totals["items"], "130");
    }
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    assert_eq!(rows[0].associated_counters["items"].fastest, 2.0);
    assert_eq!(rows[0].worker_associated_counters["items"].fastest, 2.0);
    assert_eq!(
        rows[0].worker_associated_counters["bytes"].fastest,
        u64::MAX as f64
    );
    assert_eq!(
        rows[0].worker_associated_counters["chars"].operation_mean,
        0.0
    );
    assert!(
        airbug_bench::report::descriptive_markdown(&rows)
            .contains("Worker work counts associated with timing")
    );
    assert!(
        airbug_bench::report::html_run_without_plots(&run)
            .unwrap()
            .contains("id=\"worker-counters\"")
    );
    let mut old = serde_json::to_value(&samples[0]).unwrap();
    old.as_object_mut().unwrap().remove("work_totals");
    let restored: airbug_bench::WorkerSlotSample = serde_json::from_value(old).unwrap();
    assert!(restored.work_totals.is_empty());
}

#[test]
fn dynamic_worker_counts_follow_coordinator_slots_across_waves_and_validate() {
    use std::sync::atomic::{AtomicU64, Ordering};
    let counters = airbug_bench::counters::InputCounters::new(&["items"]).unwrap();
    let index = AtomicU64::new(0);
    let captured = counters.clone();
    let mut suite = Suite::new("dynamic-slots");
    suite.bench_threads_with_input(
        "case",
        2,
        || {
            let value = if index.fetch_add(1, Ordering::SeqCst) % 128 < 64 {
                1
            } else {
                3
            };
            captured.add("items", value);
            value
        },
        |v| *v,
        DropPolicy::InsideTiming,
    );
    suite.input_counters(counters);
    suite.sampling(Sampling {
        samples: Some(1),
        iterations: Some(65),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let mut run = suite.run("").unwrap();
    for timing in &mut run.worker_timings {
        timing.wall_ns = (timing.operations * (10 + timing.worker * 10)).to_string();
    }
    run.observations[0].value = Some("1300".into());
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    let counts = &rows[0].worker_associated_counters["items"];
    assert_eq!(counts.fastest, 1.0);
    assert!((counts.slowest - 193.0 / 65.0).abs() < 1e-12);
    assert!((counts.operation_mean - 258.0 / 130.0).abs() < 1e-12);
    let samples = run.worker_slot_samples().unwrap();
    assert_eq!(samples[0].work_totals["items"], "65");
    assert_eq!(samples[1].work_totals["items"], "193");
    assert_eq!(run.observations[0].work_totals["items"], "258");
    let restored: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(samples, restored.worker_slot_samples().unwrap());
    let mut invalid = run.clone();
    invalid.observations[0]
        .worker_work_totals
        .get_mut(&0)
        .unwrap()
        .insert("items".into(), "66".into());
    assert!(invalid.validate().is_err());
    invalid = run.clone();
    invalid.observations[0].worker_work_totals.remove(&1);
    assert!(invalid.validate().is_err());
    let mut legacy = run;
    for observation in &mut legacy.observations {
        observation.worker_work_totals.clear();
    }
    legacy.validate().unwrap();
    assert!(
        legacy
            .worker_slot_samples()
            .unwrap()
            .iter()
            .all(|s| s.work_totals.is_empty())
    );
}

#[test]
fn worker_lifetime_spans_waves_and_single_worker_stays_on_caller() {
    use std::{collections::HashMap, sync::Mutex, thread};
    for workers in [1, 2] {
        let caller = thread::current().id();
        let calls = Mutex::new(HashMap::new());
        let mut suite = Suite::new("worker-lifetime");
        suite.bench_threads_with_local_input(
            "waves",
            workers,
            || thread::current().id(),
            |origin| {
                let current = thread::current().id();
                assert_eq!(*origin, current);
                *calls.lock().unwrap().entry(current).or_insert(0usize) += 1;
            },
            DropPolicy::OutsideTiming,
        );
        suite.sampling(Sampling {
            iterations: Some(65),
            ..Default::default()
        });
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            ..Default::default()
        });
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        let calls = calls.lock().unwrap();
        assert_eq!(calls.values().sum::<usize>(), 130 * workers);
        if workers == 1 {
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[&caller], 130);
        } else {
            assert!(!calls.contains_key(&caller));
            assert_eq!(calls.len(), 2);
            assert!(calls.values().all(|&n| n == 130));
        }
    }
}

#[test]
fn pooled_workers_keep_thread_local_state_through_warmup_and_repeated_runs() {
    use std::{cell::Cell, collections::HashMap, sync::Mutex, thread};
    thread_local! { static OPERATIONS: Cell<usize> = const { Cell::new(0) }; }
    let seen = Mutex::new(HashMap::new());
    let mut suite = Suite::new("persistent-workers");
    suite.bench_threads_with_local_input(
        "state",
        2,
        || Rc::new(()),
        |_| {
            let count = OPERATIONS.with(|value| {
                let next = value.get() + 1;
                value.set(next);
                next
            });
            let mut seen = seen.lock().unwrap();
            let previous = seen.entry(thread::current().id()).or_insert(0);
            assert_eq!(count, *previous + 1);
            *previous = count;
        },
        DropPolicy::OutsideTiming,
    );
    suite.config(Config {
        samples: 2,
        warmup: Duration::from_millis(5),
        sample_time: Duration::from_nanos(1),
        max_iterations: 65,
    });
    let first = suite.run("").unwrap();
    first.validate().unwrap();
    let after_first = seen.lock().unwrap().clone();
    assert_eq!(after_first.len(), 2);
    suite.sampling(Sampling {
        iterations: Some(65),
        ..Default::default()
    });
    let second = suite.run("").unwrap();
    second.validate().unwrap();
    let after_second = seen.lock().unwrap();
    assert_eq!(after_second.len(), 2);
    for (worker, count) in after_first {
        assert!(after_second[&worker] >= count + 130);
    }
}
