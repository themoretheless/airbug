use airbug_bench::{Config, Sampling, Suite, alloc::TrackingAllocator};
use std::{alloc::System, hint::black_box, time::Duration};
#[global_allocator]
static ALLOCATOR: TrackingAllocator<System> = TrackingAllocator::new(System);

#[test]
fn suite_records_allocation_metrics_for_each_measured_sample_only() {
    let mut suite = Suite::new("allocated");
    suite.bench_allocated("vector", &ALLOCATOR, || {
        let vector = vec![black_box(7u8); black_box(64)];
        black_box(vector)
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 3,
    });
    suite.sampling(Sampling {
        iterations: Some(3),
        ..Default::default()
    });
    assert_eq!(suite.list(""), ["allocated/vector"]);
    let run = suite.run("").unwrap();
    assert_eq!(run.cases[0].metrics.len(), 14);
    assert_eq!(run.observations.len(), 28);
    for sequence in 0..2 {
        let value = |metric| {
            run.observations
                .iter()
                .find(|o| o.metric == metric && o.sequence == sequence)
                .unwrap()
                .value
                .as_deref()
                .unwrap()
        };
        assert_eq!(value("alloc.count"), "3");
        assert_eq!(value("alloc.dealloc_count"), "3");
        assert_eq!(value("alloc.bytes"), "192");
        assert_eq!(value("alloc.dealloc_bytes"), "192");
        assert_eq!(value("alloc.peak_above_start_bytes"), "64");
        assert_eq!(value("alloc.peak_above_start_count"), "1");
        assert_eq!(value("alloc.net_growth_bytes"), "0");
        assert_eq!(value("alloc.net_release_bytes"), "0");
    }
    assert!(run.observations.iter().all(|o| o.operations == 3));
    let summary = airbug_bench::report::descriptive(&run).unwrap();
    assert_eq!(
        summary
            .iter()
            .find(|r| r.metric == "alloc.bytes")
            .unwrap()
            .summary
            .as_ref()
            .unwrap()
            .mean,
        64.0
    );
    let peak = summary
        .iter()
        .find(|r| r.metric == "alloc.peak_above_start_bytes")
        .unwrap();
    assert!(!peak.normalized_per_operation);
    assert_eq!(peak.summary.as_ref().unwrap().mean, 64.0);
    let restored: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
    restored.validate().unwrap();
    suite.sampling(Sampling::default());
    suite.config(Config {
        samples: 2,
        warmup: Duration::from_millis(1),
        sample_time: Duration::from_nanos(1),
        max_iterations: 3,
    });
    let warmed = suite.run("").unwrap();
    assert_eq!(
        warmed.observations.len(),
        28,
        "pilots and warmup must not become samples"
    );
    for o in warmed
        .observations
        .iter()
        .filter(|o| o.metric == "alloc.count")
    {
        assert_eq!(
            o.value.as_ref().unwrap().parse::<u64>().unwrap(),
            o.operations
        );
    }
}

#[test]
fn async_allocations_exclude_executor_setup_and_complete_non_send_futures() {
    use std::{
        cell::Cell,
        future::Future,
        rc::Rc,
        task::{Context, Poll, Waker},
    };
    struct Executor(Vec<u8>);
    impl airbug_bench::workloads::Executor for Executor {
        fn block_on<F: Future>(&mut self, future: F) -> F::Output {
            assert_eq!(self.0.len(), 1024);
            let mut future = std::pin::pin!(future);
            let mut context = Context::from_waker(Waker::noop());
            loop {
                if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
                    return value;
                }
            }
        }
    }
    let setups = Cell::new(0);
    let calls = Rc::new(Cell::new(0));
    let mut suite = Suite::new("allocated-async");
    suite.bench_async_allocated(
        "vector",
        &ALLOCATOR,
        || {
            setups.set(setups.get() + 1);
            Executor(vec![black_box(1); black_box(1024)])
        },
        async || {
            let mut pending = true;
            std::future::poll_fn(|cx| {
                if std::mem::take(&mut pending) {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
            calls.set(calls.get() + 1);
            black_box(vec![black_box(7u8); black_box(64)])
        },
    );
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 3,
    });
    suite.sampling(Sampling {
        iterations: Some(3),
        ..Default::default()
    });
    assert_eq!(suite.list("").len(), 1);
    assert_eq!(setups.get(), 0);
    assert_eq!(calls.get(), 0);
    let run = suite.run("").unwrap();
    assert_eq!(setups.get(), 1);
    assert_eq!(calls.get(), 6);
    for sample in run
        .observations
        .iter()
        .filter(|o| o.metric == "alloc.bytes" || o.metric == "alloc.dealloc_bytes")
    {
        assert_eq!(sample.value.as_deref(), Some("192"));
    }
    assert_eq!(run.observations.len(), 28);
}

#[cfg(feature = "macros")]
#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 2, iterations = 3, warmup_ms = 0)]
mod attributed_allocations {
    #[bench(args = [16usize, 32])]
    fn sync(size: usize) -> Vec<u8> {
        vec![std::hint::black_box(1); size]
    }

    #[bench(args = [String::from("borrowed")])]
    async fn asynchronous(value: &str) -> Vec<u8> {
        std::future::ready(()).await;
        vec![std::hint::black_box(1); value.len()]
    }
}
#[cfg(feature = "macros")]
#[test]
fn attributed_allocator_defaults_register_sync_and_async_parameter_cases() {
    let mut suite = Suite::new("attributes");
    attributed_allocations::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 3);
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 3);
    for case in &run.cases {
        assert_eq!(case.metrics.len(), 14);
        assert!(case.contract.contains_key("alloc.scope"));
        let allocations: Vec<_> = run
            .observations
            .iter()
            .filter(|o| o.case == case.id && o.metric == "alloc.bytes")
            .collect();
        assert_eq!(allocations.len(), 2);
        if case.id.ends_with("/16") {
            assert!(allocations.iter().all(|o| o.value.as_deref() == Some("48")));
        } else if case.id.ends_with("/32") {
            assert!(allocations.iter().all(|o| o.value.as_deref() == Some("96")));
        } else {
            assert!(
                allocations
                    .iter()
                    .all(|o| o.value.as_ref().unwrap().parse::<u128>().unwrap() >= 24)
            );
        }
    }
}

#[test]
fn allocation_summaries_pair_by_sequence_and_preserve_peak_units() {
    let mut sizes = [100usize, 3, 50, 10].into_iter();
    let mut suite = Suite::new("paired-allocations");
    suite.bench_allocated("vector", &ALLOCATOR, || {
        vec![black_box(1u8); black_box(sizes.next().unwrap())]
    });
    suite.config(Config {
        samples: 4,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    let mut run = suite.run("").unwrap();
    for observation in run.observations.iter_mut().filter(|o| o.metric == "wall") {
        observation.value = Some([10, 40, 20, 30][observation.sequence as usize].to_string());
    }
    // Input order must not determine the association.
    run.observations.reverse();
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    let wall = rows.iter().find(|r| r.metric == "wall").unwrap();
    let bytes = wall.associated_allocations["alloc.bytes"].as_ref().unwrap();
    assert_eq!(
        (bytes.fastest, bytes.slowest, bytes.median, bytes.mean),
        (100.0, 3.0, 30.0, 40.75)
    );
    assert!(bytes.normalized_per_operation);
    let peak = wall.associated_allocations["alloc.peak_above_start_bytes"]
        .as_ref()
        .unwrap();
    assert!(!peak.normalized_per_operation);
    assert_eq!(peak.median, 30.0);
    assert!(
        airbug_bench::report::descriptive_markdown(&rows)
            .contains("Allocations associated with timing")
    );
    let mut unequal = run.clone();
    for o in unequal.observations.iter_mut().filter(|o| o.sequence == 1) {
        o.operations = 100;
    }
    let unequal_rows = airbug_bench::report::descriptive(&unequal).unwrap();
    let associated = &unequal_rows
        .iter()
        .find(|r| r.metric == "wall")
        .unwrap()
        .associated_allocations;
    let bytes = associated["alloc.bytes"].as_ref().unwrap();
    assert_eq!(
        (bytes.fastest, bytes.slowest, bytes.median),
        (0.03, 10.0, 75.0)
    );
    assert!((bytes.mean - 163.0 / 103.0).abs() < 1e-12);
    assert_eq!(
        associated["alloc.peak_above_start_bytes"]
            .as_ref()
            .unwrap()
            .mean,
        40.75
    );
    let peak = associated["alloc.peak_above_start_bytes"].as_ref().unwrap();
    let normalized = peak.per_operation.as_ref().unwrap();
    assert_eq!(
        (normalized.fastest, normalized.slowest, normalized.median),
        (0.03, 10.0, 75.0)
    );
    assert!((normalized.operation_mean - 163.0 / 103.0).abs() < 1e-12);
    assert!(bytes.per_operation.is_none());
    let count = associated["alloc.peak_above_start_count"].as_ref().unwrap();
    assert_eq!(count.mean, 1.0);
    let normalized_count = count.per_operation.as_ref().unwrap();
    assert_eq!(
        (
            normalized_count.fastest,
            normalized_count.slowest,
            normalized_count.median
        ),
        (0.01, 1.0, 1.0)
    );
    assert!((normalized_count.operation_mean - 4.0 / 103.0).abs() < 1e-12);
    let markdown = airbug_bench::report::descriptive_markdown(&unequal_rows);
    assert!(markdown.contains("alloc.peak_above_start_bytes (per operation)"));
    assert!(markdown.contains("alloc.peak_above_start_count (per operation)"));
    assert!(markdown.contains("count/op"));
    let encoded = serde_json::to_value(peak).unwrap();
    let decoded: airbug_bench::report::AssociatedAllocation =
        serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded.per_operation.unwrap().fastest, 0.03);
    let mut old = encoded;
    old.as_object_mut().unwrap().remove("per_operation");
    let old: airbug_bench::report::AssociatedAllocation = serde_json::from_value(old).unwrap();
    assert!(old.per_operation.is_none());
    let mut mismatched = run.clone();
    mismatched
        .observations
        .iter_mut()
        .find(|o| o.metric == "alloc.bytes")
        .unwrap()
        .operations = 2;
    assert!(airbug_bench::report::descriptive(&mismatched).is_err());
    run.observations
        .retain(|o| !(o.metric == "alloc.bytes" && o.sequence == 0));
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    assert!(
        rows.iter()
            .find(|r| r.metric == "wall")
            .unwrap()
            .associated_allocations["alloc.bytes"]
            .is_none()
    );
}

#[test]
fn allocated_batches_exclude_setup_and_apply_input_and_output_drop_policies() {
    use airbug_bench::{BatchPolicy, DropPolicy};
    for owned in [false, true] {
        for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
            let mut suite = Suite::new("allocated-batch");
            let setup = || vec![black_box(3u8); black_box(128)];
            let batch = BatchPolicy::Iterations(2.try_into().unwrap());
            if owned {
                suite.bench_allocated_batched(
                    "owned",
                    &ALLOCATOR,
                    setup,
                    |input| {
                        black_box(&input);
                        black_box(vec![black_box(7u8); black_box(64)])
                    },
                    policy,
                    batch,
                );
            } else {
                suite.bench_allocated_batched_ref(
                    "borrowed",
                    &ALLOCATOR,
                    setup,
                    |input| {
                        black_box(input);
                        black_box(vec![black_box(7u8); black_box(64)])
                    },
                    policy,
                    batch,
                );
            }
            suite.config(Config {
                samples: 1,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 5,
            });
            suite.sampling(Sampling {
                iterations: Some(5),
                ..Default::default()
            });
            let run = suite.run("").unwrap();
            let metric = |name| {
                run.observations
                    .iter()
                    .find(|o| o.metric == name)
                    .unwrap()
                    .value
                    .as_ref()
                    .unwrap()
                    .parse::<u128>()
                    .unwrap()
            };
            let inside = matches!(policy, DropPolicy::InsideTiming);
            assert_eq!(metric("alloc.count"), 5);
            assert_eq!(metric("alloc.bytes"), 320);
            assert_eq!(
                metric("alloc.dealloc_count"),
                u128::from(owned) * 5 + u128::from(inside) * 5
            );
            assert_eq!(
                metric("alloc.dealloc_bytes"),
                u128::from(owned) * 640 + u128::from(inside) * 320
            );
            assert_eq!(
                metric("alloc.peak_above_start_bytes"),
                if !owned && !inside { 128 } else { 64 }
            );
            assert_eq!(
                metric("alloc.peak_above_start_count"),
                if !owned && !inside { 2 } else { 1 }
            );
            let net = 320i128 - (i128::from(owned) * 640 + i128::from(inside) * 320);
            assert_eq!(metric("alloc.net_growth_bytes"), net.max(0) as u128);
            assert_eq!(metric("alloc.net_release_bytes"), net.min(0).unsigned_abs());
        }
    }
}

#[test]
fn async_allocated_batches_exclude_setup_and_apply_input_and_output_drop_policies() {
    use airbug_bench::{BatchPolicy, DropPolicy};
    struct Executor(Vec<u8>);
    impl airbug_bench::workloads::Executor for Executor {
        fn block_on<F: std::future::Future>(&mut self, future: F) -> F::Output {
            assert_eq!(self.0.len(), 1024);
            let mut future = std::pin::pin!(future);
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            loop {
                if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut context) {
                    return value;
                }
            }
        }
    }
    async fn suspend() {
        let mut pending = true;
        std::future::poll_fn(|cx| {
            if std::mem::take(&mut pending) {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            } else {
                std::task::Poll::Ready(())
            }
        })
        .await;
    }
    for owned in [false, true] {
        for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
            let executor_setups = std::cell::Cell::new(0);
            let mut suite = Suite::new("allocated-batch");
            let setup = || vec![black_box(3u8); black_box(128)];
            let batch = BatchPolicy::Iterations(2.try_into().unwrap());
            if owned {
                suite.bench_async_allocated_batched(
                    "owned",
                    &ALLOCATOR,
                    || {
                        executor_setups.set(executor_setups.get() + 1);
                        Executor(vec![black_box(0); black_box(1024)])
                    },
                    setup,
                    async |input| {
                        suspend().await;
                        black_box(&input);
                        black_box(vec![black_box(7u8); black_box(64)])
                    },
                    policy,
                    batch,
                );
            } else {
                suite.bench_async_allocated_batched_ref(
                    "borrowed",
                    &ALLOCATOR,
                    || {
                        executor_setups.set(executor_setups.get() + 1);
                        Executor(vec![black_box(0); black_box(1024)])
                    },
                    setup,
                    async |input| {
                        suspend().await;
                        black_box(input);
                        black_box(vec![black_box(7u8); black_box(64)])
                    },
                    policy,
                    batch,
                );
            }
            suite.config(Config {
                samples: 1,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 5,
            });
            suite.sampling(Sampling {
                iterations: Some(5),
                ..Default::default()
            });
            assert_eq!(suite.list("").len(), 1);
            assert_eq!(executor_setups.get(), 0);
            let run = suite.run("").unwrap();
            assert_eq!(executor_setups.get(), 1);
            let metric = |name| {
                run.observations
                    .iter()
                    .find(|o| o.metric == name)
                    .unwrap()
                    .value
                    .as_ref()
                    .unwrap()
                    .parse::<u128>()
                    .unwrap()
            };
            let inside = matches!(policy, DropPolicy::InsideTiming);
            assert_eq!(metric("alloc.count"), 5);
            assert_eq!(metric("alloc.bytes"), 320);
            assert_eq!(
                metric("alloc.dealloc_count"),
                u128::from(owned) * 5 + u128::from(inside) * 5
            );
            assert_eq!(
                metric("alloc.dealloc_bytes"),
                u128::from(owned) * 640 + u128::from(inside) * 320
            );
            assert_eq!(
                metric("alloc.peak_above_start_bytes"),
                if !owned && !inside { 128 } else { 64 }
            );
            assert_eq!(
                metric("alloc.peak_above_start_count"),
                if !owned && !inside { 2 } else { 1 }
            );
            let net = 320i128 - (i128::from(owned) * 640 + i128::from(inside) * 320);
            assert_eq!(metric("alloc.net_growth_bytes"), net.max(0) as u128);
            assert_eq!(metric("alloc.net_release_bytes"), net.min(0).unsigned_abs());
        }
    }
}

#[cfg(feature = "macros")]
#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 1, iterations = 5, warmup_ms = 0,
    batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()))]
mod attributed_batches {
    pub struct Executor;
    impl airbug_bench::workloads::Executor for Executor {
        fn block_on<F: std::future::Future>(&mut self, future: F) -> F::Output {
            let mut future = std::pin::pin!(future);
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            loop {
                if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut context) {
                    return value;
                }
            }
        }
    }
    #[bench(drop_output = "inside")]
    fn sync_plain_inside() -> Vec<u8> {
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "outside")]
    fn sync_plain_outside() -> Vec<u8> {
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "inside", setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    fn sync_owned_inside(input: Vec<u8>) -> Vec<u8> {
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "outside", setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    fn sync_owned_outside(input: Vec<u8>) -> Vec<u8> {
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "inside", setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    fn sync_borrowed_inside(input: &mut Vec<u8>) -> Vec<u8> {
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "outside", setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    fn sync_borrowed_outside(input: &mut Vec<u8>) -> Vec<u8> {
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "inside", executor = Executor)]
    async fn async_plain_inside() -> Vec<u8> {
        std::future::ready(()).await;
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "outside", executor = Executor)]
    async fn async_plain_outside() -> Vec<u8> {
        std::future::ready(()).await;
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "inside", executor = Executor, setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    async fn async_owned_inside(input: Vec<u8>) -> Vec<u8> {
        std::future::ready(()).await;
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "outside", executor = Executor, setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    async fn async_owned_outside(input: Vec<u8>) -> Vec<u8> {
        std::future::ready(()).await;
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "inside", executor = Executor, setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    async fn async_borrowed_inside(input: &mut Vec<u8>) -> Vec<u8> {
        std::future::ready(()).await;
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(drop_output = "outside", executor = Executor, setup = || vec![std::hint::black_box(1u8); 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    async fn async_borrowed_outside(input: &mut Vec<u8>) -> Vec<u8> {
        std::future::ready(()).await;
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
}
#[cfg(feature = "macros")]
#[test]
fn attributed_allocated_batches_support_all_local_lifecycles() {
    let mut suite = Suite::new("attributes");
    attributed_batches::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 12);
    let run = suite.run("").unwrap();
    for case in &run.cases {
        let owned = case.id.contains("_owned_");
        let inside = case.id.ends_with("_inside");
        if !case.id.contains("_plain_") {
            let wall = run
                .observations
                .iter()
                .find(|o| o.case == case.id && o.metric == "wall")
                .unwrap();
            assert_eq!(wall.work_totals["bytes"], "640");
        }
        let value = |metric| {
            run.observations
                .iter()
                .find(|o| o.case == case.id && o.metric == metric)
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .parse::<u128>()
                .unwrap()
        };
        assert_eq!(value("alloc.bytes"), 320, "{}", case.id);
        assert_eq!(value("alloc.count"), 5, "{}", case.id);
        assert_eq!(
            value("alloc.dealloc_bytes"),
            u128::from(owned) * 640 + u128::from(inside) * 320,
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.peak_above_start_bytes"),
            if !owned && !inside { 128 } else { 64 },
            "{}",
            case.id
        );
    }
}

#[test]
fn allocated_workers_exclude_local_setup_and_sum_all_workers_across_waves() {
    use airbug_bench::DropPolicy;
    for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
        let mut suite = Suite::new("allocated-workers");
        suite.bench_threads_allocated_with_local_input(
            "local",
            &ALLOCATOR,
            3,
            || (vec![black_box(1u8); 128], std::rc::Rc::new(())),
            |input| {
                black_box(input);
                vec![black_box(7u8); 64]
            },
            policy,
        );
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 65,
        });
        suite.sampling(Sampling {
            iterations: Some(65),
            ..Default::default()
        });
        let run = suite.run("").unwrap();
        assert_eq!(run.worker_allocations.len(), 12);
        for sequence in 0..2 {
            for wave in 0..2 {
                let workers: Vec<_> = run
                    .worker_allocations
                    .iter()
                    .filter(|w| w.sequence == sequence && w.wave == wave)
                    .collect();
                assert_eq!(workers.len(), 3);
                let slots: std::collections::BTreeSet<_> =
                    workers.iter().map(|w| w.worker).collect();
                assert_eq!(slots, [0, 1, 2].into_iter().collect());
                let operations = if wave == 0 { 64 } else { 1 };
                for worker in workers {
                    assert_eq!(worker.operations, operations);
                    assert_eq!(worker.metrics["alloc.bytes"], (operations * 64).to_string());
                    assert_eq!(worker.metrics["alloc.count"], operations.to_string());
                }
            }
        }
        let encoded = serde_json::to_value(&run).unwrap();
        let decoded: airbug_bench::Run = serde_json::from_value(encoded.clone()).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded.worker_allocations.len(), 12);
        let mut old = encoded;
        old.as_object_mut().unwrap().remove("worker_allocations");
        for case in old["cases"].as_array_mut().unwrap() {
            case["contract"]
                .as_object_mut()
                .unwrap()
                .remove("alloc.worker_records");
        }
        let old: airbug_bench::Run = serde_json::from_value(old).unwrap();
        old.validate().unwrap();
        assert!(old.worker_allocations.is_empty());
        let mut missing = run.clone();
        missing.worker_allocations.pop();
        assert!(missing.validate().is_err());
        missing.worker_allocations.clear();
        assert!(missing.validate().is_err());
        let mut wrong_operations = run.clone();
        wrong_operations.worker_allocations[0].operations += 1;
        assert!(wrong_operations.validate().is_err());
        let mut wrong_time = run.clone();
        wrong_time.worker_allocations[0].wall_ns = u128::MAX.to_string();
        assert!(wrong_time.validate().is_err());
        let mut wrong_metric = run.clone();
        wrong_metric.worker_allocations[0]
            .metrics
            .insert("alloc.bytes".into(), "1".into());
        assert!(wrong_metric.validate().is_err());
        let mut wrong_peak = run.clone();
        wrong_peak.worker_allocations[0]
            .metrics
            .insert("alloc.peak_above_start_bytes".into(), u128::MAX.to_string());
        assert!(wrong_peak.validate().is_err());
        let mut duplicate = run.clone();
        duplicate
            .worker_allocations
            .push(duplicate.worker_allocations[0].clone());
        assert!(duplicate.validate().is_err());
        let mut malformed = run.clone();
        malformed.worker_allocations[0]
            .metrics
            .remove("alloc.bytes");
        assert!(malformed.validate().is_err());

        for sequence in 0..2 {
            let value = |metric| {
                run.observations
                    .iter()
                    .find(|o| o.metric == metric && o.sequence == sequence)
                    .unwrap()
                    .value
                    .as_ref()
                    .unwrap()
                    .parse::<u128>()
                    .unwrap()
            };
            let inside = matches!(policy, DropPolicy::InsideTiming);
            assert_eq!(value("alloc.count"), 195);
            assert_eq!(value("alloc.bytes"), 195 * 64);
            assert_eq!(value("alloc.dealloc_count"), if inside { 195 } else { 0 });
            assert_eq!(
                value("alloc.dealloc_bytes"),
                if inside { 195 * 64 } else { 0 }
            );
            assert_eq!(
                value("alloc.peak_above_start_bytes"),
                if inside { 3 * 64 } else { 3 * 64 * 64 }
            );
        }
        assert!(run.observations.iter().all(|o| o.operations == 195));
    }
}

#[test]
fn allocated_worker_panics_release_peers_and_allow_subsequent_measurement() {
    use airbug_bench::DropPolicy;
    let fail = std::sync::atomic::AtomicBool::new(true);
    let mut suite = Suite::new("worker-panic");
    suite.bench_threads_allocated_with_local_input(
        "case",
        &ALLOCATOR,
        3,
        || (),
        |_| {
            if fail.swap(false, std::sync::atomic::Ordering::SeqCst) {
                panic!("operation failed");
            }
            vec![black_box(1u8); 64]
        },
        DropPolicy::OutsideTiming,
    );
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    assert!(suite.run("").is_err());
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations
            .iter()
            .find(|o| o.metric == "alloc.bytes")
            .unwrap()
            .value
            .as_deref(),
        Some("192")
    );
}

#[cfg(feature = "macros")]
#[airbug_bench::group(allocator = &crate::ALLOCATOR, threads = [3],
    samples = 1, iterations = 65, warmup_ms = 0)]
mod attributed_worker_allocations {
    pub struct Local(Vec<u8>, std::marker::PhantomData<std::rc::Rc<()>>);
    fn input(size: usize) -> Local {
        Local(
            vec![std::hint::black_box(1u8); size],
            std::marker::PhantomData,
        )
    }
    #[bench(drop_output = "inside", args = [64usize])]
    fn plain_inside(size: usize) -> Local {
        input(size)
    }
    #[bench(drop_output = "outside", args = [64usize])]
    fn plain_outside(size: usize) -> Local {
        input(size)
    }
    #[bench(drop_output = "inside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    fn borrowed_inside(value: &mut Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        input(size)
    }
    #[bench(drop_output = "outside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    fn borrowed_outside(value: &mut Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        input(size)
    }
    #[bench(drop_output = "inside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    fn owned_inside(value: Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        input(size)
    }
    #[bench(drop_output = "outside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    fn owned_outside(value: Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        input(size)
    }
}
#[cfg(feature = "macros")]
#[test]
fn attributed_worker_allocations_cover_plain_borrowed_owned_and_drop_policies() {
    let mut suite = Suite::new("worker-attributes");
    attributed_worker_allocations::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 6);
    let run = suite.run("").unwrap();
    for case in &run.cases {
        let owned = case.id.contains("/owned_");
        let inside = case.id.contains("_inside/");
        let value = |metric| {
            run.observations
                .iter()
                .find(|o| o.case == case.id && o.metric == metric)
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .parse::<u128>()
                .unwrap()
        };
        assert_eq!(value("alloc.count"), 195, "{}", case.id);
        assert_eq!(value("alloc.bytes"), 195 * 64, "{}", case.id);
        assert_eq!(
            value("alloc.dealloc_count"),
            (u128::from(owned) + u128::from(inside)) * 195,
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.dealloc_bytes"),
            (u128::from(owned) * 128 + u128::from(inside) * 64) * 195,
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.peak_above_start_bytes"),
            if owned || inside { 3 * 64 } else { 3 * 64 * 64 },
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.peak_above_start_count"),
            if owned || inside { 3 } else { 3 * 64 }
        );
        let wall = run
            .observations
            .iter()
            .find(|o| o.case == case.id && o.metric == "wall")
            .unwrap();
        assert_eq!(wall.operations, 195);
        if !case.id.contains("/plain_") {
            assert_eq!(wall.work_totals["bytes"], (195 * 128).to_string());
        }
        assert_eq!(case.contract["threads.setup"], "worker");
    }
}

#[test]
fn allocated_coordinator_setup_accepts_local_state_and_preserves_drop_threads() {
    use airbug_bench::DropPolicy;
    struct Value {
        _bytes: Vec<u8>,
        coordinator: std::thread::ThreadId,
        drop_on_coordinator: bool,
    }
    impl Drop for Value {
        fn drop(&mut self) {
            assert_eq!(
                std::thread::current().id() == self.coordinator,
                self.drop_on_coordinator
            );
        }
    }
    let coordinator = std::thread::current().id();
    for owned in [false, true] {
        for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
            let inside = matches!(policy, DropPolicy::InsideTiming);
            let setups = std::cell::Cell::new(0);
            let setup = || {
                assert_eq!(std::thread::current().id(), coordinator);
                setups.set(setups.get() + 1);
                Value {
                    _bytes: vec![black_box(1u8); 128],
                    coordinator,
                    drop_on_coordinator: !owned,
                }
            };
            let operation = || {
                assert_ne!(std::thread::current().id(), coordinator);
                Value {
                    _bytes: vec![black_box(7u8); 64],
                    coordinator,
                    drop_on_coordinator: !inside,
                }
            };
            let mut suite = Suite::new("coordinator");
            if owned {
                suite.bench_threads_allocated_with_owned_input(
                    "owned",
                    &ALLOCATOR,
                    3,
                    setup,
                    |input| {
                        black_box(&input);
                        operation()
                    },
                    policy,
                );
            } else {
                suite.bench_threads_allocated_with_input(
                    "borrowed",
                    &ALLOCATOR,
                    3,
                    setup,
                    |input| {
                        black_box(input);
                        operation()
                    },
                    policy,
                );
            }
            suite.config(Config {
                samples: 1,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 65,
            });
            suite.sampling(Sampling {
                iterations: Some(65),
                ..Default::default()
            });
            assert_eq!(suite.list("").len(), 1);
            assert_eq!(setups.get(), 0);
            let run = suite.run("").unwrap();
            assert_eq!(run.worker_allocations.len(), 6);
            assert!(
                run.worker_allocations
                    .iter()
                    .all(|w| w.metrics["alloc.bytes"] == (w.operations * 64).to_string())
            );
            assert_eq!(setups.get(), 195);
            let value = |metric| {
                run.observations
                    .iter()
                    .find(|o| o.metric == metric)
                    .unwrap()
                    .value
                    .as_ref()
                    .unwrap()
                    .parse::<u128>()
                    .unwrap()
            };
            assert_eq!(value("alloc.bytes"), 195 * 64);
            assert_eq!(
                value("alloc.dealloc_bytes"),
                (u128::from(owned) * 128 + u128::from(inside) * 64) * 195
            );
            assert_eq!(run.cases[0].contract["threads.setup"], "coordinator");
        }
    }
}

#[cfg(feature = "macros")]
#[airbug_bench::group(allocator = &crate::ALLOCATOR, threads = [2], samples = 1,
    iterations = 3, warmup_ms = 0)]
mod attributed_coordinator_allocations {
    #[bench(setup = || vec![1u8; 128], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    fn borrowed(input: &mut Vec<u8>) -> Vec<u8> {
        std::hint::black_box(input);
        vec![std::hint::black_box(7u8); 64]
    }
    #[bench(setup = || vec![1u8; 128], setup_thread = "coordinator", drop_output = "outside")]
    fn owned(input: Vec<u8>) -> Vec<u8> {
        std::hint::black_box(&input);
        vec![std::hint::black_box(7u8); 64]
    }
}
#[cfg(feature = "macros")]
#[test]
fn allocator_attributes_preserve_default_and_explicit_coordinator_setup() {
    let mut suite = Suite::new("attributes");
    attributed_coordinator_allocations::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 2);
    for case in &run.cases {
        assert_eq!(case.contract["threads.setup"], "coordinator");
        let value = |metric| {
            run.observations
                .iter()
                .find(|o| o.case == case.id && o.metric == metric)
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .parse::<u128>()
                .unwrap()
        };
        assert_eq!(value("alloc.bytes"), 384);
        assert_eq!(
            value("alloc.dealloc_bytes"),
            if case.id.contains("/owned/") {
                768
            } else {
                384
            }
        );
        if case.id.contains("borrowed") {
            let wall = run
                .observations
                .iter()
                .find(|o| o.case == case.id && o.metric == "wall")
                .unwrap();
            assert_eq!(wall.work_totals["bytes"], "768");
        }
    }
}

#[cfg(feature = "macros")]
#[airbug_bench::group(allocator = &crate::ALLOCATOR, threads = [3],
    samples = 1, iterations = 65, warmup_ms = 0)]
mod attributed_async_worker_allocations {
    pub struct Executor(Vec<u8>, std::marker::PhantomData<std::rc::Rc<()>>);
    impl Default for Executor {
        fn default() -> Self {
            Self(
                vec![std::hint::black_box(0); 4096],
                std::marker::PhantomData,
            )
        }
    }
    impl airbug_bench::workloads::Executor for Executor {
        fn block_on<F: std::future::Future>(&mut self, future: F) -> F::Output {
            assert_eq!(self.0.len(), 4096);
            let mut future = std::pin::pin!(future);
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            loop {
                if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut context) {
                    return value;
                }
            }
        }
    }
    async fn suspend() {
        let mut pending = true;
        std::future::poll_fn(|cx| {
            if std::mem::take(&mut pending) {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            } else {
                std::task::Poll::Ready(())
            }
        })
        .await;
    }
    pub struct Local(Vec<u8>, std::marker::PhantomData<std::rc::Rc<()>>);
    fn input(size: usize) -> Local {
        Local(
            vec![std::hint::black_box(1u8); size],
            std::marker::PhantomData,
        )
    }
    #[bench(executor = Executor::default(), drop_output = "inside", args = [64usize])]
    async fn plain_inside(size: usize) -> Local {
        suspend().await;
        input(size)
    }
    #[bench(executor = Executor::default(), drop_output = "outside", args = [64usize])]
    async fn plain_outside(size: usize) -> Local {
        suspend().await;
        input(size)
    }
    #[bench(executor = Executor::default(), drop_output = "inside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    async fn borrowed_inside(value: &mut Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        suspend().await;
        input(size)
    }
    #[bench(executor = Executor::default(), drop_output = "outside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    async fn borrowed_outside(value: &mut Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        suspend().await;
        input(size)
    }
    #[bench(executor = Executor::default(), drop_output = "inside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    async fn owned_inside(value: Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        suspend().await;
        input(size)
    }
    #[bench(executor = Executor::default(), drop_output = "outside", setup = input, args = [128usize], setup_thread = "worker", input_bytes = |v: &Local| v.0.len() as u64)]
    async fn owned_outside(value: Local) -> Local {
        std::hint::black_box(&value);
        let size = 64;
        suspend().await;
        input(size)
    }
}
#[cfg(feature = "macros")]
#[test]
fn attributed_async_worker_allocations_cover_plain_borrowed_owned_and_drop_policies() {
    let mut suite = Suite::new("worker-attributes");
    attributed_async_worker_allocations::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 6);
    let run = suite.run("").unwrap();
    assert_eq!(run.worker_allocations.len(), 36);
    assert!(
        run.worker_allocations
            .iter()
            .all(|w| w.metrics["alloc.bytes"] == (w.operations * 64).to_string())
    );
    for case in &run.cases {
        let owned = case.id.contains("/owned_");
        let inside = case.id.contains("_inside/");
        let value = |metric| {
            run.observations
                .iter()
                .find(|o| o.case == case.id && o.metric == metric)
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .parse::<u128>()
                .unwrap()
        };
        assert_eq!(value("alloc.count"), 195, "{}", case.id);
        assert_eq!(value("alloc.bytes"), 195 * 64, "{}", case.id);
        assert_eq!(
            value("alloc.dealloc_count"),
            (u128::from(owned) + u128::from(inside)) * 195,
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.dealloc_bytes"),
            (u128::from(owned) * 128 + u128::from(inside) * 64) * 195,
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.peak_above_start_bytes"),
            if owned || inside { 3 * 64 } else { 3 * 64 * 64 },
            "{}",
            case.id
        );
        assert_eq!(
            value("alloc.peak_above_start_count"),
            if owned || inside { 3 } else { 3 * 64 }
        );
        let wall = run
            .observations
            .iter()
            .find(|o| o.case == case.id && o.metric == "wall")
            .unwrap();
        assert_eq!(wall.operations, 195);
        if !case.id.contains("/plain_") {
            assert_eq!(wall.work_totals["bytes"], (195 * 128).to_string());
        }
        assert_eq!(case.contract["threads.setup"], "worker");
    }
}

#[cfg(feature = "macros")]
#[test]
fn async_allocated_worker_panics_release_peers_and_allow_subsequent_measurement() {
    use airbug_bench::DropPolicy;
    let fail = std::sync::atomic::AtomicBool::new(true);
    let mut suite = Suite::new("worker-panic");
    suite.bench_async_threads_allocated_with_input(
        "case",
        &ALLOCATOR,
        3,
        attributed_async_worker_allocations::Executor::default,
        || (),
        async |_| {
            std::future::ready(()).await;
            if fail.swap(false, std::sync::atomic::Ordering::SeqCst) {
                panic!("operation failed");
            }
            vec![black_box(1u8); 64]
        },
        DropPolicy::OutsideTiming,
    );
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    assert!(suite.run("").is_err());
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations
            .iter()
            .find(|o| o.metric == "alloc.bytes")
            .unwrap()
            .value
            .as_deref(),
        Some("192")
    );
}

#[test]
fn worker_validation_cancels_opposite_signed_net_balances() {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut suite = Suite::new("mixed-balances");
    suite.bench_threads_allocated_with_local_owned_input(
        "owned",
        &ALLOCATOR,
        2,
        || {
            vec![
                black_box(0u8);
                if next.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                    16
                } else {
                    128
                }
            ]
        },
        |input| {
            black_box(&input);
            vec![black_box(1u8); 64]
        },
        airbug_bench::DropPolicy::OutsideTiming,
    );
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    let releases: u128 = run
        .worker_allocations
        .iter()
        .map(|w| {
            w.metrics["alloc.net_release_bytes"]
                .parse::<u128>()
                .unwrap()
        })
        .sum();
    let growth: u128 = run
        .worker_allocations
        .iter()
        .map(|w| w.metrics["alloc.net_growth_bytes"].parse::<u128>().unwrap())
        .sum();
    assert_eq!((growth, releases), (48, 64));
    assert_eq!(
        run.observations
            .iter()
            .find(|o| o.metric == "alloc.net_release_bytes")
            .unwrap()
            .value
            .as_deref(),
        Some("16")
    );
}

#[test]
fn worker_allocation_reports_pair_metrics_by_worker_time() {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut suite = Suite::new("worker-report");
    suite.bench_threads_allocated_with_local_input(
        "varied",
        &ALLOCATOR,
        2,
        || [100usize, 3, 50, 10][next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)],
        |size| vec![black_box(1u8); *size],
        airbug_bench::DropPolicy::InsideTiming,
    );
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    let mut run = suite.run("").unwrap();
    for w in &mut run.worker_allocations {
        w.wall_ns = match w.metrics["alloc.bytes"].as_str() {
            "100" => "10",
            "3" => "40",
            "50" => "20",
            "10" => "30",
            _ => unreachable!(),
        }
        .into();
    }
    for wall in run.observations.iter_mut().filter(|o| o.metric == "wall") {
        wall.value = Some(
            run.worker_allocations
                .iter()
                .filter(|w| w.sequence == wall.sequence)
                .map(|w| w.wall_ns.parse::<u128>().unwrap())
                .max()
                .unwrap()
                .to_string(),
        );
    }
    run.worker_allocations.reverse();
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    let summary = rows
        .iter()
        .find(|r| r.metric == "wall")
        .unwrap()
        .worker_allocations
        .as_ref()
        .unwrap();
    assert_eq!(summary.samples, 4);
    let bytes = summary.allocations["alloc.bytes"].as_ref().unwrap();
    assert_eq!(
        (bytes.fastest, bytes.slowest, bytes.median, bytes.mean),
        (100.0, 3.0, 30.0, 40.75)
    );
    let selected = |id: &airbug_bench::report::WorkerSampleRef| {
        run.worker_allocations
            .iter()
            .find(|w| (w.sequence, w.wave, w.worker) == (id.sequence, id.wave, id.worker))
            .unwrap()
    };
    assert_eq!(selected(&summary.fastest).metrics["alloc.bytes"], "100");
    assert_eq!(selected(&summary.slowest).metrics["alloc.bytes"], "3");
    assert_eq!(
        summary
            .median_samples
            .iter()
            .map(|id| selected(id).metrics["alloc.bytes"].clone())
            .collect::<Vec<_>>(),
        ["50", "10"]
    );
    let peak = summary.allocations["alloc.peak_above_start_bytes"]
        .as_ref()
        .unwrap();
    assert_eq!(peak.mean, 40.75);
    assert_eq!(peak.per_operation.as_ref().unwrap().operation_mean, 40.75);
    assert!(
        airbug_bench::report::descriptive_markdown(&rows)
            .contains("Worker allocations associated with timing")
    );
    let mut uneven = run.clone();
    for w in uneven
        .worker_allocations
        .iter_mut()
        .filter(|w| w.sequence == 0)
    {
        w.operations *= 100;
        for (id, value) in &mut w.metrics {
            if !id.starts_with("alloc.peak_") {
                *value = (value.parse::<u128>().unwrap() * 100).to_string();
            }
        }
    }
    for o in uneven.observations.iter_mut().filter(|o| o.sequence == 0) {
        o.operations *= 100;
        if o.metric.starts_with("alloc.") && !o.metric.starts_with("alloc.peak_") {
            o.value = Some((o.value.as_ref().unwrap().parse::<u128>().unwrap() * 100).to_string());
        }
    }
    let uneven_rows = airbug_bench::report::descriptive(&uneven).unwrap();
    let uneven_summary = uneven_rows
        .iter()
        .find(|r| r.metric == "wall")
        .unwrap()
        .worker_allocations
        .as_ref()
        .unwrap();
    let bytes = uneven_summary.allocations["alloc.bytes"].as_ref().unwrap();
    assert_eq!(
        (bytes.fastest, bytes.slowest, bytes.median),
        (100.0, 10.0, 26.5)
    );
    assert!((bytes.mean - 10360.0 / 202.0).abs() < 1e-12);
    let peak = uneven_summary.allocations["alloc.peak_above_start_bytes"]
        .as_ref()
        .unwrap()
        .per_operation
        .as_ref()
        .unwrap();
    assert_eq!(
        (peak.fastest, peak.slowest, peak.median),
        (1.0, 10.0, 25.015)
    );
    assert!((peak.operation_mean - 163.0 / 202.0).abs() < 1e-12);
    let encoded = serde_json::to_value(&rows).unwrap();
    let decoded: Vec<airbug_bench::report::DescriptiveRow> =
        serde_json::from_value(encoded).unwrap();
    assert_eq!(
        decoded
            .iter()
            .find(|r| r.metric == "wall")
            .unwrap()
            .worker_allocations
            .as_ref()
            .unwrap()
            .fastest,
        summary.fastest
    );
}

#[test]
fn adaptive_quick_preserves_worker_allocations_and_resets_between_runs() {
    let calls = std::sync::atomic::AtomicU64::new(0);
    let mut suite = Suite::new("quick-workers");
    suite.bench_threads_allocated_with_local_input(
        "vectors",
        &ALLOCATOR,
        2,
        || (),
        |_| {
            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            vec![black_box(1u8); 64]
        },
        airbug_bench::DropPolicy::InsideTiming,
    );
    suite.config(Config {
        max_iterations: 4,
        ..Default::default()
    });
    suite.quick(airbug_bench::QuickConfig {
        min_time: Duration::ZERO,
        max_time: Duration::from_millis(20),
        relative_deviation: 0.5,
    });
    for _ in 0..2 {
        let before = calls.load(std::sync::atomic::Ordering::Relaxed);
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        let walls: Vec<_> = run
            .observations
            .iter()
            .filter(|o| o.metric == "wall")
            .collect();
        assert_eq!(walls[0].operations, 2);
        assert!(walls.iter().all(|o| o.operations <= 8));
        assert_eq!(run.worker_allocations.len(), walls.len() * 2);
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed) - before,
            walls.iter().map(|o| o.operations).sum::<u64>()
        );
        for (sequence, wall) in walls.iter().enumerate() {
            assert_eq!(wall.sequence, sequence as u64);
            let bytes = run
                .observations
                .iter()
                .find(|o| o.sequence == wall.sequence && o.metric == "alloc.bytes")
                .unwrap();
            assert_eq!(
                bytes.value.as_ref().unwrap(),
                &(wall.operations * 64).to_string()
            );
        }
        let rows = airbug_bench::report::descriptive(&run).unwrap();
        let workers = rows
            .iter()
            .find(|r| r.metric == "wall")
            .unwrap()
            .worker_allocations
            .as_ref()
            .unwrap();
        assert!((workers.allocations["alloc.bytes"].as_ref().unwrap().mean - 64.0).abs() < 1e-12);
    }
}

#[test]
fn compensated_worker_records_roundtrip_and_reject_inconsistent_aggregates() {
    let mut suite = Suite::new("compensated");
    suite.bench_threads_allocated_with_input(
        "workers",
        &ALLOCATOR,
        2,
        || (),
        |_| vec![black_box(3u8); black_box(64)],
        airbug_bench::DropPolicy::InsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(65),
        ..Default::default()
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 65,
    });
    suite.compensate_overhead(true);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    assert_eq!(
        run.cases[0].contract["timer.overhead_policy"],
        "subtract-v1"
    );
    assert_eq!(run.worker_allocations.len(), 8);
    for worker in &run.worker_allocations {
        let raw = worker.wall_ns.parse::<u128>().unwrap();
        let adjusted = worker
            .adjusted_wall_ns
            .as_ref()
            .unwrap()
            .parse::<u128>()
            .unwrap();
        assert!(adjusted <= raw);
        assert_eq!(worker.metrics["alloc.count"], worker.operations.to_string());
    }
    let restored: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
    restored.validate().unwrap();
    assert!(
        airbug_bench::report::descriptive(&restored)
            .unwrap()
            .iter()
            .any(|r| r.metric == "wall.adjusted")
    );
    let mut wrong = run.clone();
    let adjusted = wrong
        .observations
        .iter_mut()
        .find(|o| o.metric == "wall.adjusted")
        .unwrap();
    adjusted.value =
        Some((adjusted.value.as_ref().unwrap().parse::<u128>().unwrap() + 1).to_string());
    assert!(
        wrong
            .validate()
            .unwrap_err()
            .to_string()
            .contains("adjusted worker durations")
    );
    let mut missing = run.clone();
    missing.worker_allocations[0].adjusted_wall_ns = None;
    assert!(
        missing
            .validate()
            .unwrap_err()
            .to_string()
            .contains("missing adjusted worker")
    );
    suite.compensate_overhead(false);
    let raw = suite.run("").unwrap();
    assert!(!raw.cases[0].metrics.iter().any(|m| m.id == "wall.adjusted"));
    assert!(
        raw.worker_allocations
            .iter()
            .all(|w| w.adjusted_wall_ns.is_none())
    );
}
