use airbug_bench::{
    BatchPolicy, Config, DropPolicy, Sampling, Suite, alloc::TrackingAllocator,
    workloads::LocalExecutor,
};
use std::{alloc::System, num::NonZeroU64, rc::Rc, time::Duration};

#[global_allocator]
static ALLOCATOR: TrackingAllocator<System> = TrackingAllocator::new(System);

fn sampling() -> Sampling {
    Sampling {
        iterations: Some(5),
        ..Default::default()
    }
}

#[test]
fn policies_size_all_worker_executors_and_keep_remainders() {
    for (batch, expected) in [
        (BatchPolicy::PerIteration, vec![1, 1, 1, 1, 1]),
        (BatchPolicy::SmallInput, vec![1, 1, 1, 1, 1]),
        (BatchPolicy::LargeInput, vec![1, 1, 1, 1, 1]),
        (
            BatchPolicy::Iterations(NonZeroU64::new(2).unwrap()),
            vec![2, 2, 1],
        ),
        (
            BatchPolicy::Batches(NonZeroU64::new(2).unwrap()),
            vec![3, 2],
        ),
        (BatchPolicy::Batches(NonZeroU64::new(1).unwrap()), vec![5]),
    ] {
        let mut suite = Suite::new("batch");
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            ..Default::default()
        });
        suite.with_batch_defaults(batch, |suite| {
            suite
                .bench_threads_with_local_input(
                    "local",
                    2,
                    || Rc::new(1),
                    |v| v.clone(),
                    DropPolicy::OutsideTiming,
                )
                .sampling(sampling());
            suite
                .bench_threads_with_input(
                    "coordinator",
                    2,
                    || vec![1],
                    |v| v.clone(),
                    DropPolicy::OutsideTiming,
                )
                .sampling(sampling());
            suite
                .bench_async_threads_with_input(
                    "async",
                    2,
                    || LocalExecutor,
                    || Rc::new(1),
                    async |v| v.clone(),
                    DropPolicy::OutsideTiming,
                )
                .sampling(sampling());
            suite
                .bench_threads_allocated_with_local_input(
                    "allocated-local",
                    &ALLOCATOR,
                    2,
                    || Rc::new(1),
                    |_| vec![0u8; 8],
                    DropPolicy::OutsideTiming,
                )
                .sampling(sampling());
            suite
                .bench_threads_allocated_with_input(
                    "allocated-coordinator",
                    &ALLOCATOR,
                    2,
                    || vec![1],
                    |_| vec![0u8; 8],
                    DropPolicy::OutsideTiming,
                )
                .sampling(sampling());
            suite
                .bench_async_threads_allocated_with_input(
                    "allocated-async",
                    &ALLOCATOR,
                    2,
                    || LocalExecutor,
                    || Rc::new(1),
                    async |_| vec![0u8; 8],
                    DropPolicy::OutsideTiming,
                )
                .sampling(sampling());
        });
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        assert_eq!(run.cases.len(), 6);
        for case in &run.cases {
            assert_eq!(case.contract["threads.batch_policy"], format!("{batch:?}"));
            let operations: Vec<_> = if case.id.contains("allocated") {
                let bytes = run
                    .observations
                    .iter()
                    .find(|row| row.case == case.id && row.metric == "alloc.bytes")
                    .unwrap();
                let bytes: u64 = bytes.value.as_deref().unwrap().parse().unwrap();
                if case.id.ends_with("allocated-async") {
                    // Async execution may also allocate the polled future.
                    assert!(bytes >= 80);
                } else {
                    assert_eq!(bytes, 80, "{}", case.id);
                }
                run.worker_allocations
                    .iter()
                    .filter(|row| row.case == case.id && row.worker == 0)
                    .map(|row| row.operations)
                    .collect()
            } else {
                run.worker_timings
                    .iter()
                    .filter(|row| row.case == case.id && row.worker == 0)
                    .map(|row| row.operations)
                    .collect()
            };
            assert_eq!(operations, expected, "{}", case.id);
        }
    }
}

#[airbug_bench::group(samples = 1, iterations = 5, warmup_ms = 0)]
mod attributed {
    #[bench(threads = 2, batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()), drop_output = "inside")]
    fn plain() -> Vec<u8> {
        vec![0; 8]
    }
    #[bench(threads = 2, batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()), setup = || vec![1u8])]
    fn coordinator(v: &mut [u8]) -> Vec<u8> {
        v.to_vec()
    }
    #[bench(threads = 2, batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()), setup_thread = "worker", setup = || std::rc::Rc::new(1))]
    fn local(v: &mut std::rc::Rc<i32>) -> std::rc::Rc<i32> {
        v.clone()
    }
    #[bench(threads = 2, batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()))]
    async fn asynchronous() -> Vec<u8> {
        vec![0; 8]
    }
    #[bench(threads = 2, batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()), allocator = &super::ALLOCATOR)]
    fn allocated() -> Vec<u8> {
        vec![0; 8]
    }
}

#[test]
fn explicit_attributes_apply_batches_to_threads() {
    let mut suite = Suite::new("attributes");
    attributed::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    assert_eq!(run.cases.len(), 5);
    for case in &run.cases {
        assert_eq!(case.contract["threads.batch_policy"], "Iterations(2)");
        let ops: Vec<_> = if case.id.contains("allocated/") {
            run.worker_allocations
                .iter()
                .filter(|r| r.case == case.id && r.worker == 0)
                .map(|r| r.operations)
                .collect()
        } else {
            run.worker_timings
                .iter()
                .filter(|r| r.case == case.id && r.worker == 0)
                .map(|r| r.operations)
                .collect()
        };
        assert_eq!(ops, [2, 2, 1], "{}", case.id);
    }
}

#[test]
fn oversized_worker_batch_fails_before_input_setup() {
    let mut suite = Suite::new("oversized");
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    suite.with_batch_defaults(BatchPolicy::Batches(NonZeroU64::new(1).unwrap()), |suite| {
        suite.bench_threads_with_local_input(
            "capacity",
            2,
            || -> [u64; 8] { panic!("must reject capacity before setup") },
            |_| (),
            DropPolicy::OutsideTiming,
        );
    });
    suite.sampling(Sampling {
        iterations: Some(1 << 60),
        ..Default::default()
    });
    let error = suite.run("").unwrap_err();
    assert!(
        error.to_string().contains("batch input capacity"),
        "{error}"
    );
}

#[airbug_bench::group]
mod inferred {
    #[bench(threads = 2)]
    fn shared() {}
}

#[test]
fn imported_batch_reaches_inferred_thread_dispatch() {
    let mut suite = Suite::new("imported");
    suite.with_batch_defaults(BatchPolicy::PerIteration, |suite| {
        inferred::__airbug_register_group(suite);
    });
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    suite.sampling(sampling());
    let run = suite.run("").unwrap();
    assert_eq!(run.worker_timings.len(), 10);
    assert!(run.worker_timings.iter().all(|r| r.operations == 1));
    assert_eq!(
        run.cases[0].contract["threads.batch_policy"],
        "PerIteration"
    );
    assert!(!suite.has_registration_batch_policy());
}

struct Retained(std::rc::Rc<std::cell::Cell<usize>>);
impl Retained {
    fn new(state: &std::rc::Rc<std::cell::Cell<usize>>, limit: usize) -> Self {
        let count = state.get() + 1;
        assert!(
            count <= limit,
            "batch retained too many outputs: {count} > {limit}"
        );
        state.set(count);
        Self(state.clone())
    }
}
impl Drop for Retained {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

#[airbug_bench::group(samples = 1, iterations = 5, warmup_ms = 0, drop_output = "outside")]
mod local_imported {
    use std::{cell::Cell, rc::Rc};
    #[bench(args = [Rc::new(Cell::new(0))])]
    fn plain(state: &Rc<Cell<usize>>) -> super::Retained {
        super::Retained::new(state, 2)
    }
    #[bench(args = [Rc::new(Cell::new(0))])]
    async fn asynchronous(state: &Rc<Cell<usize>>) -> super::Retained {
        super::Retained::new(state, 2)
    }
    #[bench(args = [Rc::new(Cell::new(0))], allocator = &super::ALLOCATOR)]
    fn allocated(state: &Rc<Cell<usize>>) -> super::Retained {
        super::Retained::new(state, 2)
    }
    #[bench(args = [Rc::new(Cell::new(0))], allocator = &super::ALLOCATOR)]
    async fn async_allocated(state: &Rc<Cell<usize>>) -> super::Retained {
        super::Retained::new(state, 2)
    }
    #[bench(args = [Rc::new(Cell::new(0))], batch = airbug_bench::BatchPolicy::PerIteration)]
    fn overridden(state: &Rc<Cell<usize>>) -> super::Retained {
        super::Retained::new(state, 1)
    }
}
#[airbug_bench::group(groups = [crate::local_imported], threads = 1,
    batch = airbug_bench::BatchPolicy::Iterations(2.try_into().unwrap()))]
mod local_parent {}

#[test]
fn imported_batch_controls_non_send_local_output_lifetimes() {
    let mut suite = Suite::new("local");
    local_parent::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    assert_eq!(run.cases.len(), 5);
    for case in &run.cases {
        let expected = if case.id.contains("/overridden/") {
            "PerIteration"
        } else {
            "Iterations(2)"
        };
        assert_eq!(
            case.contract["threads.batch_policy"], expected,
            "{}",
            case.id
        );
    }
}
