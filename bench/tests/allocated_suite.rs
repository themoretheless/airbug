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
    fn borrowed(input: &mut [u8]) -> Vec<u8> {
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
    let timing = summary.wall_per_operation.as_ref().unwrap();
    assert_eq!(
        (
            timing.count,
            timing.minimum,
            timing.maximum,
            timing.mean,
            timing.median
        ),
        (4, 10.0, 40.0, 25.0, 25.0)
    );
    assert!(
        airbug_bench::report::descriptive_markdown(&rows)
            .contains("Worker wall time per operation")
    );
    let mut legacy = serde_json::to_value(summary).unwrap();
    legacy.as_object_mut().unwrap().remove("wall_per_operation");
    let decoded: airbug_bench::report::WorkerAllocationSummary =
        serde_json::from_value(legacy).unwrap();
    assert!(decoded.wall_per_operation.is_none());
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
    let timing = uneven_summary.wall_per_operation.as_ref().unwrap();
    assert_eq!(timing.minimum, 0.1);
    assert_eq!(timing.maximum, 30.0);
    assert!((timing.mean - 12.625).abs() < 1e-12);
    assert!((timing.median - 10.2).abs() < 1e-12);
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

#[cfg(feature = "macros")]
#[test]
fn runtime_workers_update_sync_async_allocations_and_dynamic_input_totals() {
    for asynchronous in [false, true] {
        let mut suite = Suite::new("override-workers");
        if asynchronous {
            attributed_async_worker_allocations::__airbug_register_group(&mut suite);
        } else {
            attributed_worker_allocations::__airbug_register_group(&mut suite);
        }
        suite.thread_count(2).unwrap();
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        assert_eq!(run.cases.len(), 6);
        // 65 iterations require two input waves for each of two workers.
        assert_eq!(run.worker_allocations.len(), 6 * 2 * 2);
        for case in &run.cases {
            assert_eq!(case.contract["threads"], "2");
            let observations: Vec<_> = run
                .observations
                .iter()
                .filter(|o| o.case == case.id)
                .collect();
            assert!(observations.iter().all(|o| o.operations == 130));
            let bytes = observations
                .iter()
                .find(|o| o.metric == "alloc.bytes")
                .unwrap();
            assert_eq!(bytes.value.as_deref(), Some("8320"));
            if case.contract.contains_key("work.input.bytes") {
                let wall = observations.iter().find(|o| o.metric == "wall").unwrap();
                assert_eq!(wall.work_totals["bytes"], (130 * 128).to_string());
                assert_eq!(wall.worker_work_totals.len(), 2);
                assert!(
                    wall.worker_work_totals
                        .values()
                        .all(|totals| totals["bytes"] == (65 * 128).to_string())
                );
                let slots = run.worker_slot_samples().unwrap();
                assert!(
                    slots
                        .iter()
                        .filter(|slot| slot.case == case.id)
                        .all(|slot| slot.work_totals["bytes"] == (65 * 128).to_string())
                );
            }
        }
    }
}

#[test]
fn worker_identity_json_preserves_full_u64_and_legacy_indices() {
    for worker in [0, u32::MAX as u64, u32::MAX as u64 + 1, u64::MAX] {
        let record = airbug_bench::WorkerAllocation {
            case: "identity".into(),
            variant: "candidate".into(),
            process: 0,
            sequence: 0,
            wave: 0,
            worker,
            operations: 1,
            wall_ns: "1".into(),
            adjusted_wall_ns: None,
            metrics: Default::default(),
        };
        let bytes = serde_json::to_vec(&record).unwrap();
        let decoded: airbug_bench::WorkerAllocation = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.worker, worker);
        let reference = airbug_bench::report::WorkerSampleRef {
            sequence: 0,
            wave: 0,
            worker,
        };
        let bytes = serde_json::to_vec(&reference).unwrap();
        let decoded: airbug_bench::report::WorkerSampleRef =
            serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, reference);
    }
    let legacy: airbug_bench::report::WorkerSampleRef =
        serde_json::from_str(r#"{"sequence":0,"wave":1,"worker":2}"#).unwrap();
    assert_eq!(legacy.worker, 2);
}

#[test]
fn ordinary_worker_timing_records_validate_waves_and_roundtrip_without_allocations() {
    let mut suite = Suite::new("timing-records");
    suite.bench_threads_allocated_with_local_input(
        "case",
        &ALLOCATOR,
        2,
        || (),
        |_| (),
        airbug_bench::DropPolicy::InsideTiming,
    );
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
    run.worker_timings = run
        .worker_allocations
        .iter()
        .map(|w| airbug_bench::WorkerTiming {
            case: w.case.clone(),
            variant: w.variant.clone(),
            process: w.process,
            sequence: w.sequence,
            wave: w.wave,
            worker: w.worker,
            operations: w.operations,
            wall_ns: w.wall_ns.clone(),
            adjusted_wall_ns: w.adjusted_wall_ns.clone(),
        })
        .collect();
    assert_eq!(run.worker_timings.len(), 8);
    let both_charts = airbug_bench::report::worker_timing_charts(&run).unwrap();
    assert_eq!(both_charts.matches("<svg").count(), 1);
    assert!(both_charts.contains("4 samples") || both_charts.contains("4 identical samples"));
    let mut allocation_only = run.clone();
    allocation_only.worker_timings.clear();
    assert_eq!(
        airbug_bench::report::worker_timing_charts(&allocation_only).unwrap(),
        both_charts
    );
    run.worker_allocations.clear();
    for case in &mut run.cases {
        case.contract.remove("alloc.worker_records");
        case.contract
            .insert("threads.timing_records".into(), "wave-v1".into());
        case.metrics.retain(|m| !m.id.starts_with("alloc."));
    }
    run.observations.retain(|o| !o.metric.starts_with("alloc."));
    run.validate().unwrap();
    let saved = serde_json::to_vec(&run).unwrap();
    let decoded: airbug_bench::Run = serde_json::from_slice(&saved).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded.worker_timings.len(), 8);
    for mutation in 0..6 {
        let mut invalid = run.clone();
        match mutation {
            0 => {
                invalid.worker_timings.pop();
            }
            1 => invalid
                .worker_timings
                .push(invalid.worker_timings[0].clone()),
            2 => invalid.worker_timings[0].sequence = u64::MAX,
            3 => invalid.worker_timings[0].worker = 2,
            4 => invalid.worker_timings[0].operations += 1,
            _ => invalid.worker_timings[0].wall_ns = u128::MAX.to_string(),
        }
        assert!(invalid.validate().is_err(), "mutation {mutation}");
    }
    let mut legacy = serde_json::to_value(&run).unwrap();
    legacy.as_object_mut().unwrap().remove("worker_timings");
    let mut decoded: airbug_bench::Run = serde_json::from_value(legacy).unwrap();
    assert!(decoded.worker_timings.is_empty());
    assert!(
        decoded.validate().is_err(),
        "declared capture must not silently disappear"
    );
    decoded.cases[0].contract.remove("threads.timing_records");
    decoded.validate().unwrap();
}

#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 1, iterations = 3, warmup_ms = 0)]
mod inherited_plain_allocated {
    #[bench]
    fn vector() -> Vec<u8> {
        vec![std::hint::black_box(7); std::hint::black_box(64)]
    }
    #[bench]
    async fn local_output() -> std::rc::Rc<u8> {
        std::rc::Rc::new(std::hint::black_box(7))
    }
    #[bench(threads = false)]
    fn sequential() -> Vec<u8> {
        vec![std::hint::black_box(1); std::hint::black_box(32)]
    }
}

#[airbug_bench::group(groups = [crate::inherited_plain_allocated], threads = [1, 2])]
mod allocated_parent {}

#[test]
fn imported_threads_enable_plain_allocated_sync_and_async_cases() {
    for runtime in [false, true] {
        let mut suite = Suite::new("inherited-alloc");
        if runtime {
            suite.registration_threads(&[3]).unwrap();
        }
        allocated_parent::__airbug_register_group(&mut suite);
        assert_eq!(suite.list("").len(), if runtime { 3 } else { 5 });
        let run = suite.run("").unwrap();
        for case in &run.cases {
            let workers = case
                .contract
                .get("threads")
                .map(|s| s.parse::<u64>().unwrap())
                .unwrap_or(1);
            let row = |metric| {
                run.observations
                    .iter()
                    .find(|o| o.case == case.id && o.metric == metric)
                    .unwrap()
            };
            assert_eq!(row("wall").operations, workers * 3);
            // LocalExecutor allocates one Arc-backed waker per block_on, in
            // addition to the workload's Rc. Both are inside the allocator phase.
            let allocations_per_call = if case.id.contains("local_output") {
                2
            } else {
                1
            };
            assert_eq!(
                row("alloc.count").value.as_deref(),
                Some((workers * 3 * allocations_per_call).to_string().as_str())
            );
            assert_eq!(row("alloc.dealloc_count").value, row("alloc.count").value);
            if case.id.contains("vector") {
                assert_eq!(
                    row("alloc.bytes").value.as_deref(),
                    Some((workers * 3 * 64).to_string().as_str())
                );
            }
            if case.id.contains("sequential") {
                assert!(!case.contract.contains_key("threads"));
            } else {
                assert!(case.id.ends_with(&format!("threads={workers}")));
                assert_eq!(
                    run.worker_allocations
                        .iter()
                        .filter(|r| r.case == case.id)
                        .count() as u64,
                    workers
                );
            }
        }
    }
}

#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 1, iterations = 2, warmup_ms = 0)]
mod inherited_allocated_values {
    #[bench(args = [16usize, 32])]
    fn shared(size: usize) -> Vec<u8> {
        vec![std::hint::black_box(1); std::hint::black_box(size)]
    }
    #[bench(args = [std::rc::Rc::new(24usize)])]
    fn local(size: &std::rc::Rc<usize>) -> Vec<u8> {
        vec![std::hint::black_box(1); std::hint::black_box(**size)]
    }
}

#[test]
fn allocated_parameter_threads_preserve_local_fallback_and_selected_execution() {
    let mut inherited = Suite::new("parent-allocated-args");
    inherited.with_thread_defaults(Some(vec![1]), |suite| {
        inherited_allocated_values::__airbug_register_group(suite);
    });
    let inherited_run = inherited.run("").unwrap();
    assert_eq!(inherited_run.cases.len(), 3);
    assert!(
        inherited_run
            .cases
            .iter()
            .all(|case| case.id.ends_with("threads=1"))
    );
    let mut suite = Suite::new("allocated-args");
    suite.registration_threads(&[1, 2]).unwrap();
    inherited_allocated_values::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 6);
    let shared = suite.run("shared").unwrap();
    assert_eq!(shared.cases.len(), 4);
    for case in &shared.cases {
        let workers: u64 = case.contract["threads"].parse().unwrap();
        let size = if case.id.contains("/16/") { 16 } else { 32 };
        let bytes = shared
            .observations
            .iter()
            .find(|o| o.case == case.id && o.metric == "alloc.bytes")
            .unwrap();
        assert_eq!(bytes.operations, workers * 2);
        assert_eq!(
            bytes.value.as_deref(),
            Some((workers * 2 * size).to_string().as_str())
        );
    }
    let local_id = suite
        .list("local")
        .into_iter()
        .find(|id| id.ends_with("threads=1"))
        .unwrap()
        .to_owned();
    let local = suite.run(&local_id).unwrap();
    assert_eq!(local.cases[0].contract["threads.execution"], "caller");
    let bytes = local
        .observations
        .iter()
        .find(|o| o.metric == "alloc.bytes")
        .unwrap();
    assert_eq!(bytes.value.as_deref(), Some("48"));
    assert_eq!(bytes.operations, 2);
    assert!(
        suite
            .run("local")
            .unwrap_err()
            .to_string()
            .contains("cannot share")
    );
}

#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 1, iterations = 2, warmup_ms = 0)]
mod inherited_async_allocated_values {
    #[bench(args = [String::from("hello")])]
    async fn shared(input: &String) -> Vec<u8> {
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
        input.as_bytes().to_vec()
    }
    #[bench(args = [std::rc::Rc::new(24usize)])]
    async fn local(size: &std::rc::Rc<usize>) -> Vec<u8> {
        std::future::ready(()).await;
        vec![std::hint::black_box(1); std::hint::black_box(**size)]
    }
}

#[test]
fn async_allocated_arguments_borrow_across_await_and_retain_local_worker() {
    let mut suite = Suite::new("async-allocated-args");
    suite.registration_threads(&[1, 2]).unwrap();
    inherited_async_allocated_values::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 4);
    let run = suite.run("shared").unwrap();
    let mut bytes_per_worker = None;
    for case in &run.cases {
        let workers: u64 = case.contract["threads"].parse().unwrap();
        let row = |metric| {
            run.observations
                .iter()
                .find(|o| o.case == case.id && o.metric == metric)
                .unwrap()
        };
        assert_eq!(row("wall").operations, workers * 2);
        assert_eq!(
            row("alloc.count").value.as_deref(),
            Some((workers * 4).to_string().as_str())
        );
        assert_eq!(row("alloc.dealloc_count").value, row("alloc.count").value);
        let bytes: u64 = row("alloc.bytes").value.as_ref().unwrap().parse().unwrap();
        let per_worker = bytes / workers;
        assert_eq!(bytes, per_worker * workers);
        assert_eq!(*bytes_per_worker.get_or_insert(per_worker), per_worker);
        assert_eq!(
            run.worker_allocations
                .iter()
                .filter(|r| r.case == case.id)
                .count() as u64,
            workers
        );
    }
    let id = suite
        .list("local")
        .into_iter()
        .find(|id| id.ends_with("threads=1"))
        .unwrap()
        .to_owned();
    let run = suite.run(&id).unwrap();
    assert_eq!(run.cases[0].contract["threads.execution"], "caller");
    assert_eq!(
        run.observations
            .iter()
            .find(|o| o.metric == "alloc.count")
            .unwrap()
            .value
            .as_deref(),
        Some("4")
    );
    assert!(
        suite
            .run("local")
            .unwrap_err()
            .to_string()
            .contains("cannot share")
    );
}

#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 1, iterations = 3, warmup_ms = 0)]
mod inherited_allocated_setup {
    #[bench(setup = || vec![9u8; 128])]
    fn borrowed(input: &mut [u8]) -> Vec<u8> {
        assert_eq!(input[0], 9);
        input[0] = 1;
        vec![std::hint::black_box(input[0]); 16]
    }
    #[bench(setup = || vec![9u8; 128])]
    fn owned(input: Vec<u8>) -> Vec<u8> {
        assert_eq!(input.len(), 128);
        vec![std::hint::black_box(input[0]); 16]
    }
    #[bench(setup = || std::rc::Rc::new(9u8))]
    fn local(input: &mut std::rc::Rc<u8>) -> Vec<u8> {
        vec![std::hint::black_box(**input); 16]
    }
}

#[test]
fn allocated_setup_inherits_threads_without_counting_preparation() {
    for runtime in [false, true] {
        let mut suite = Suite::new("allocated-setup");
        if runtime {
            suite.registration_threads(&[1, 2]).unwrap();
            inherited_allocated_setup::__airbug_register_group(&mut suite);
        } else {
            suite.with_thread_defaults(Some(vec![1]), |suite| {
                inherited_allocated_setup::__airbug_register_group(suite);
            });
        }
        // Listing must not prepare inputs or execute the benchmark.
        assert_eq!(suite.list("").len(), if runtime { 6 } else { 3 });
        for filter in ["borrowed", "owned"] {
            let run = suite.run(filter).unwrap();
            for case in &run.cases {
                let workers: u64 = case.contract["threads"].parse().unwrap();
                let row = |metric| {
                    run.observations
                        .iter()
                        .find(|o| o.case == case.id && o.metric == metric)
                        .unwrap()
                };
                assert_eq!(row("wall").operations, workers * 3);
                assert_eq!(
                    row("alloc.count").value.as_deref(),
                    Some((workers * 3).to_string().as_str())
                );
                assert_eq!(
                    row("alloc.bytes").value.as_deref(),
                    Some((workers * 3 * 16).to_string().as_str())
                );
            }
        }
        let id = suite
            .list("local")
            .into_iter()
            .find(|id| id.ends_with("threads=1"))
            .unwrap()
            .to_owned();
        let run = suite.run(&id).unwrap();
        assert_eq!(run.cases[0].contract["threads.execution"], "caller");
        assert_eq!(
            run.observations
                .iter()
                .find(|o| o.metric == "alloc.bytes")
                .unwrap()
                .value
                .as_deref(),
            Some("48")
        );
        if runtime {
            assert!(
                suite
                    .run("local")
                    .unwrap_err()
                    .to_string()
                    .contains("cannot transfer")
            );
        }
    }
}

#[airbug_bench::group(allocator = &crate::ALLOCATOR, samples = 1, iterations = 3, warmup_ms = 0)]
mod inherited_async_allocated_setup {
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
    #[bench(setup = || vec![9u8; 128], input_bytes = |input: &Vec<u8>| input.len() as u64)]
    async fn borrowed(input: &mut [u8]) -> Vec<u8> {
        assert_eq!(input[0], 9);
        suspend().await;
        input[0] = 1;
        vec![std::hint::black_box(input[0]); 16]
    }
    #[bench(setup = || std::rc::Rc::new(vec![9u8; 128]))]
    async fn owned(input: std::rc::Rc<Vec<u8>>) -> Vec<u8> {
        suspend().await;
        assert_eq!(input.len(), 128);
        vec![std::hint::black_box(input[0]); 16]
    }
}

#[test]
#[allow(clippy::needless_borrow)] // Exercise the same autoref fallback emitted by the macro.
fn async_allocated_setup_inherits_workers_and_borrows_across_suspension() {
    for runtime in [false, true] {
        let mut suite = Suite::new("async-allocated-setup");
        if runtime {
            suite.registration_threads(&[1, 2]).unwrap();
            inherited_async_allocated_setup::__airbug_register_group(&mut suite);
        } else {
            suite.with_thread_defaults(Some(vec![1]), |suite| {
                inherited_async_allocated_setup::__airbug_register_group(suite);
            });
        }
        assert_eq!(suite.list("").len(), if runtime { 4 } else { 2 });
        for filter in ["borrowed", "owned"] {
            let run = suite.run(filter).unwrap();
            for case in &run.cases {
                let workers: u64 = case.contract["threads"].parse().unwrap();
                let row = |metric| {
                    run.observations
                        .iter()
                        .find(|o| o.case == case.id && o.metric == metric)
                        .unwrap()
                };
                assert_eq!(row("wall").operations, workers * 3);
                if filter == "borrowed" {
                    assert_eq!(
                        row("wall").work_totals["bytes"],
                        (workers * 3 * 128).to_string()
                    );
                }
                // One Vec plus LocalExecutor's waker per operation; preparation
                // allocates a large Vec (and an Rc for owned), both excluded.
                assert_eq!(
                    row("alloc.count").value.as_deref(),
                    Some((workers * 6).to_string().as_str())
                );
                assert_eq!(
                    run.worker_allocations
                        .iter()
                        .filter(|r| r.case == case.id)
                        .count(),
                    workers as usize
                );
            }
        }
    }
    use airbug_bench::threads::RegisterInheritedAsyncAllocatedSetup as _;
    let mut suite = Suite::new("local-setup");
    for workers in [1, 2] {
        let state = std::rc::Rc::new(128usize);
        (&&airbug_bench::threads::inherited_async_setup(
            std::marker::PhantomData::<std::rc::Rc<usize>>,
            || airbug_bench::workloads::LocalExecutor,
            move || vec![9u8; *state],
            async |input: &mut Vec<u8>| input.len(),
        ))
            .register_inherited_async_allocated_setup(
                &mut suite,
                &format!("local/threads={workers}"),
                &ALLOCATOR,
                workers,
                airbug_bench::DropPolicy::InsideTiming,
            );
    }
    let run = suite.run("local/threads=1").unwrap();
    assert_eq!(run.cases[0].contract["threads.execution"], "caller");
    assert!(
        suite
            .run("local/threads=2")
            .unwrap_err()
            .to_string()
            .contains("cannot share")
    );
}

#[airbug_bench::group(samples = 1, iterations = 3, warmup_ms = 0)]
mod inherited_worker_setup {
    pub struct Input {
        bytes: std::rc::Rc<Vec<u8>>,
        origin: std::thread::ThreadId,
    }
    impl Drop for Input {
        fn drop(&mut self) {
            assert_eq!(self.origin, std::thread::current().id());
        }
    }
    fn input() -> Input {
        Input {
            bytes: std::rc::Rc::new(vec![9; 128]),
            origin: std::thread::current().id(),
        }
    }
    fn work(input: &Input) -> Vec<u8> {
        assert_eq!(input.origin, std::thread::current().id());
        assert_eq!(input.bytes.len(), 128);
        vec![std::hint::black_box(input.bytes[0]); 16]
    }
    #[bench(setup = input, setup_thread = "worker", input_bytes = |i: &Input| i.bytes.len() as u64)]
    fn borrowed(input: &mut Input) -> Vec<u8> {
        work(input)
    }
    #[bench(setup = input, setup_thread = "worker")]
    fn owned(input: Input) -> Vec<u8> {
        work(&input)
    }
    #[bench(allocator = &crate::ALLOCATOR, setup = input, setup_thread = "worker", input_bytes = |i: &Input| i.bytes.len() as u64)]
    fn allocated_borrowed(input: &mut Input) -> Vec<u8> {
        work(input)
    }
    #[bench(allocator = &crate::ALLOCATOR, setup = input, setup_thread = "worker")]
    fn allocated_owned(input: Input) -> Vec<u8> {
        work(&input)
    }
}

#[airbug_bench::group(groups = [crate::inherited_worker_setup], threads = [1, 2])]
mod inherited_worker_setup_parent {}

#[test]
fn worker_local_setup_uses_inherited_or_runtime_threads_without_send_inputs() {
    for mode in 0..3 {
        let mut suite = Suite::new("worker-setup");
        match mode {
            0 => inherited_worker_setup::__airbug_register_group(&mut suite),
            1 => inherited_worker_setup_parent::__airbug_register_group(&mut suite),
            _ => {
                suite.registration_threads(&[1, 3]).unwrap();
                inherited_worker_setup_parent::__airbug_register_group(&mut suite);
            }
        }
        assert_eq!(suite.list("").len(), if mode == 0 { 4 } else { 8 });
        let run = suite.run("").unwrap();
        for case in &run.cases {
            let workers: u64 = case
                .contract
                .get("threads")
                .map(|s| s.parse().unwrap())
                .unwrap_or(1);
            assert!(mode != 2 || workers == 1 || workers == 3);
            let row = |metric| {
                run.observations
                    .iter()
                    .find(|o| o.case == case.id && o.metric == metric)
                    .unwrap()
            };
            assert_eq!(row("wall").operations, workers * 3);
            if case.id.contains("borrowed") {
                assert_eq!(
                    row("wall").work_totals["bytes"],
                    (workers * 3 * 128).to_string()
                );
            }
            if case.id.contains("allocated") {
                assert_eq!(
                    row("alloc.count").value.as_deref(),
                    Some((workers * 3).to_string().as_str())
                );
                assert_eq!(
                    row("alloc.bytes").value.as_deref(),
                    Some((workers * 3 * 16).to_string().as_str())
                );
            }
        }
    }
}
