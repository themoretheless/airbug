#![cfg(feature = "macros")]
use airbug_bench::{Config, Suite};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static SETUPS: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct Input {
    values: Vec<usize>,
}
impl Drop for Input {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
fn input(n: usize) -> Input {
    SETUPS.fetch_add(1, Ordering::SeqCst);
    Input {
        values: (0..n).rev().collect(),
    }
}
#[airbug_bench::bench(args = [4usize, 8], setup = input, name = "sort", drop_output = "outside")]
fn sorted(input: &mut Input) -> Vec<usize> {
    assert!(input.values[0] > input.values[1]);
    CALLS.fetch_add(1, Ordering::SeqCst);
    input.values.sort_unstable();
    input.values.clone()
}

#[test]
fn attributes_preserve_lazy_registration_fresh_inputs_and_contracts() {
    let mut suite = Suite::new("attributes");
    register_sorted(&mut suite);
    assert_eq!(suite.list(""), ["attributes/sort/4", "attributes/sort/8"]);
    assert_eq!(SETUPS.load(Ordering::SeqCst), 0);
    assert_eq!(CALLS.load(Ordering::SeqCst), 0);
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    let result = suite.run("sort/4").unwrap();
    assert_eq!(result.cases.len(), 1);
    assert_eq!(result.cases[0].contract["param.arg"], "4");
    assert!(
        result.cases[0]
            .contract
            .values()
            .any(|value| value.contains("output drop excluded"))
    );
    let calls = CALLS.load(Ordering::SeqCst);
    assert!(calls >= 2);
    assert_eq!(SETUPS.load(Ordering::SeqCst), calls);
    assert_eq!(DROPS.load(Ordering::SeqCst), calls);
}

#[airbug_bench::bench(types = [u16, u64], consts = [2, 4], args = [1usize, 3], items = |n| n as u64)]
fn typed<T: Default + Copy, const N: usize>(n: usize) -> Vec<[T; N]> {
    vec![[T::default(); N]; n]
}

fn fast_config() -> Config {
    Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    }
}

#[test]
fn generic_dimensions_and_parameters_form_independent_cases() {
    let mut suite = Suite::new("generic");
    register_typed(&mut suite);
    assert_eq!(suite.list("").len(), 8);
    assert!(suite.list("").contains(&"generic/typed/type=u64/const=4/3"));
    suite.config(fast_config());
    let run = suite.run("type=u64/const=4/3").unwrap();
    assert_eq!(run.cases.len(), 1);
    assert_eq!(run.cases[0].contract["param.arg"], "3");
    assert_eq!(run.cases[0].contract["work.count"], "3");
    assert_eq!(run.observations.len(), 2);
}

static EXECUTORS: AtomicUsize = AtomicUsize::new(0);
static ASYNC_INPUTS: AtomicUsize = AtomicUsize::new(0);
static ASYNC_CALLS: AtomicUsize = AtomicUsize::new(0);
fn executor() -> airbug_bench::workloads::LocalExecutor {
    EXECUTORS.fetch_add(1, Ordering::SeqCst);
    airbug_bench::workloads::LocalExecutor
}
fn async_input(n: usize) -> Vec<usize> {
    ASYNC_INPUTS.fetch_add(1, Ordering::SeqCst);
    vec![n]
}
#[airbug_bench::bench(args = [3usize, 7], setup = async_input, executor = executor(), samples = 3, warmup_ms = 0)]
async fn pending(input: &mut [usize]) {
    let mut first = true;
    std::future::poll_fn(|cx| {
        if first {
            first = false;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(())
        }
    })
    .await;
    assert_eq!(input[0], 3);
    input[0] = 0;
    ASYNC_CALLS.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn async_borrows_fresh_input_and_constructs_executor_only_when_selected() {
    let mut suite = Suite::new("async");
    register_pending(&mut suite);
    assert_eq!(suite.list("").len(), 2);
    assert_eq!(EXECUTORS.load(Ordering::SeqCst), 0);
    assert_eq!(ASYNC_INPUTS.load(Ordering::SeqCst), 0);
    suite.config(fast_config());
    let run = suite.run("pending/3").unwrap();
    assert_eq!(EXECUTORS.load(Ordering::SeqCst), 1);
    assert_eq!(
        ASYNC_INPUTS.load(Ordering::SeqCst),
        ASYNC_CALLS.load(Ordering::SeqCst)
    );
    assert_eq!(run.observations.len(), 3); // Per-case samples override Suite's defaults.
    assert_eq!(run.cases[0].contract["samples"], "3");
}

static MANUAL_CALLS: AtomicUsize = AtomicUsize::new(0);
#[airbug_bench::bench(custom = true, args = [128u64], bytes = |bytes| bytes)]
fn exact_duration(iterations: u64, bytes: u64) -> Duration {
    MANUAL_CALLS.fetch_add(1, Ordering::SeqCst);
    assert_eq!(bytes, 128);
    Duration::from_nanos(iterations * 100)
}
#[test]
fn custom_timing_uses_reported_duration_and_derives_throughput() {
    let mut suite = Suite::new("manual");
    register_exact_duration(&mut suite);
    assert_eq!(MANUAL_CALLS.load(Ordering::SeqCst), 0);
    suite.config(Config {
        max_iterations: 16,
        sample_time: Duration::from_nanos(400),
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    assert!(
        run.observations
            .iter()
            .all(|o| o.operations == 4 && o.value.as_deref() == Some("400"))
    );
    let rates = airbug_bench::report::throughput(&run).unwrap();
    assert_eq!(rates.len(), 1);
    assert!(
        rates[0]
            .values
            .iter()
            .all(|value| (*value - (128.0 * 1e9 / 100.0 / 1_048_576.0)).abs() < 1e-9)
    );
    assert_eq!(run.cases[0].contract["work.count"], "128");
}

static THREAD_CALLS: AtomicUsize = AtomicUsize::new(0);
#[airbug_bench::bench(threads = [1, 2], items = 1)]
fn concurrent_count() {
    THREAD_CALLS.fetch_add(1, Ordering::SeqCst);
}
#[test]
fn concurrent_case_normalizes_by_all_workers() {
    let mut suite = Suite::new("threads");
    register_concurrent_count(&mut suite);
    assert_eq!(
        suite.list(""),
        [
            "threads/concurrent_count/threads=1",
            "threads/concurrent_count/threads=2"
        ]
    );
    assert_eq!(THREAD_CALLS.load(Ordering::SeqCst), 0);
    suite.config(fast_config());
    let run = suite.run("threads=2").unwrap();
    assert_eq!(THREAD_CALLS.load(Ordering::SeqCst), 6); // calibration + two samples, two workers
    assert!(run.observations.iter().all(|o| o.operations == 2));
}

#[test]
fn workers_join_on_failure_and_invalid_counts_do_not_execute() {
    let completed = AtomicUsize::new(0);
    let counter = AtomicUsize::new(0);
    let mut suite = Suite::new("panic");
    suite.config(fast_config());
    suite.bench_threads("workers", 2, || {
        if counter.fetch_add(1, Ordering::SeqCst) == 0 {
            panic!("worker failure");
        }
        completed.fetch_add(1, Ordering::SeqCst);
    });
    assert!(
        suite
            .run("")
            .unwrap_err()
            .to_string()
            .contains("worker panicked")
    );
    assert_eq!(completed.load(Ordering::SeqCst), 1);
    let mut invalid = Suite::new("invalid");
    invalid.bench_threads("zero", 0, || panic!("must not run"));
    assert!(invalid.run("").unwrap_err().to_string().contains("workers"));
}

#[test]
fn threaded_inputs_are_fresh_and_dropped_after_all_workers_finish() {
    use std::sync::{Arc, Barrier};
    struct Tracked {
        finished: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    }
    impl Drop for Tracked {
        fn drop(&mut self) {
            assert_eq!(self.finished.load(Ordering::SeqCst) % 2, 0);
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    let finished = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let gate = Barrier::new(2);
    let mut suite = Suite::new("lifecycle");
    suite.config(fast_config());
    suite.bench_threads_with_input(
        "fresh",
        2,
        || Tracked {
            finished: finished.clone(),
            drops: drops.clone(),
        },
        |input| {
            gate.wait();
            input.finished.fetch_add(1, Ordering::SeqCst);
        },
        airbug_bench::DropPolicy::OutsideTiming,
    );
    let run = suite.run("").unwrap();
    assert_eq!(finished.load(Ordering::SeqCst), 6);
    assert_eq!(drops.load(Ordering::SeqCst), 6);
    assert!(run.observations.iter().all(|o| o.operations == 2));
}

#[airbug_bench::bench(args = [String::from("hello")], bytes = |s: &str| s.len() as u64)]
fn owned_arg_by_reference(value: &str) -> usize {
    value.len()
}
#[test]
fn noncopy_arguments_can_be_borrowed_without_cloning() {
    let mut suite = Suite::new("borrowed");
    register_owned_arg_by_reference(&mut suite);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    assert_eq!(run.cases[0].contract["work.count"], "5");
}

#[allow(dead_code)]
mod grouped_cases {
    #[airbug_bench::suite(samples = 4, warmup_ms = 0, items = 2)]
    mod cases {
        #[group(name = "renamed", samples = 3, bytes = 16)]
        mod inner {
            #[bench]
            fn inherited() {}
            #[bench(samples = 2, items = 5)]
            fn overridden() {}
            #[group(ignore = true)]
            mod hidden {
                #[bench]
                fn ignored_by_group() {}
                #[bench(ignore = false)]
                fn enabled() {}
            }
        }
        #[bench]
        #[ignore = "expensive"]
        fn ignored_function() {}
        #[bench]
        fn sibling() {}
        #[cfg(any())]
        mod disabled {
            #[bench]
            fn unreachable() {
                panic!("disabled group ran");
            }
        }
    }
    pub(super) fn register(suite: &mut airbug_bench::Suite<'_>) {
        cases::__airbug_register_group(suite);
    }
}

#[test]
fn nested_groups_inherit_defaults_override_locally_and_select_ignored() {
    use airbug_bench::Selection;
    let mut suite = Suite::new("groups");
    grouped_cases::register(&mut suite);
    assert_eq!(
        suite.list(""),
        [
            "groups/renamed/inherited",
            "groups/renamed/overridden",
            "groups/renamed/hidden/enabled",
            "groups/sibling"
        ]
    );
    let all = Selection {
        include_ignored: true,
        ..Selection::default()
    };
    assert_eq!(suite.list_selected(&all).len(), 6);
    let ignored = Selection {
        only_ignored: true,
        ..Selection::default()
    };
    assert_eq!(
        suite.list_selected(&ignored),
        [
            "groups/renamed/hidden/ignored_by_group",
            "groups/ignored_function"
        ]
    );
    suite.config(fast_config());
    let run = suite.run_selected(&all).unwrap();
    for (suffix, samples, items) in [
        ("inherited", 3, "2"),
        ("overridden", 2, "5"),
        ("sibling", 4, "2"),
    ] {
        let case = run
            .cases
            .iter()
            .find(|case| case.id.ends_with(suffix))
            .unwrap();
        assert_eq!(
            run.observations
                .iter()
                .filter(|o| o.case == case.id)
                .count(),
            samples
        );
        assert_eq!(case.contract["work.counter.items"], items);
    }
}

#[test]
fn failed_group_registration_restores_parent_path() {
    let mut suite = Suite::new("root");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        suite.group("broken", |_| panic!("registration failed"));
    }));
    assert!(result.is_err());
    suite.bench("after", || ());
    assert_eq!(suite.list(""), ["root/after"]);
}

#[airbug_bench::bench(custom = true, bytes = 128, items = 2, chars = 4, cycles = 100)]
fn multiple_counters(n: u64) -> Duration {
    Duration::from_nanos(n * 100)
}
#[test]
fn simultaneous_counters_and_unit_replacement_preserve_rates() {
    let mut suite = Suite::new("counters");
    register_multiple_counters(&mut suite);
    suite.work_units("items", 3);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    let series = airbug_bench::report::throughput(&run).unwrap();
    assert_eq!(series.len(), 4);
    for (unit, expected) in [
        ("MiB", 128.0 / 1_048_576.0 * 1e7),
        ("items", 3e7),
        ("chars", 4e7),
        ("cycles", 1e9),
    ] {
        let values = &series.iter().find(|s| s.unit == unit).unwrap().values;
        assert!(values.iter().all(|v| (v - expected).abs() < 1e-7));
    }
    let mut invalid = Suite::new("invalid");
    invalid
        .bench("count", || panic!("validation must precede work"))
        .work_units("", 0)
        .work_units("items", 1);
    assert!(invalid.run("").is_err());
}

static OWNED_CREATED: AtomicUsize = AtomicUsize::new(0);
static OWNED_CONSUMED: AtomicUsize = AtomicUsize::new(0);
static OWNED_DROPPED: AtomicUsize = AtomicUsize::new(0);
struct OwnedInput(String);
impl Drop for OwnedInput {
    fn drop(&mut self) {
        OWNED_DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}
fn owned_input() -> OwnedInput {
    OWNED_CREATED.fetch_add(1, Ordering::SeqCst);
    OwnedInput("owned".to_owned())
}
#[airbug_bench::bench(setup = owned_input, drop_output = "outside")]
fn consume_owned(input: OwnedInput) -> OwnedInput {
    assert_eq!(input.0, "owned");
    OWNED_CONSUMED.fetch_add(1, Ordering::SeqCst);
    input
}
#[airbug_bench::bench(setup = owned_input, drop_output = "outside")]
async fn consume_async_owned(input: OwnedInput) -> OwnedInput {
    std::future::ready(()).await;
    consume_owned(input)
}
#[airbug_bench::bench(setup = owned_input, drop_output = "outside", threads = [2])]
fn consume_threaded_owned(input: OwnedInput) -> OwnedInput {
    consume_owned(input)
}
#[test]
fn nonclone_owned_inputs_are_consumed_once_in_sync_async_and_threads() {
    let mut suite = Suite::new("owned");
    register_consume_owned(&mut suite);
    register_consume_async_owned(&mut suite);
    register_consume_threaded_owned(&mut suite);
    assert_eq!(OWNED_CREATED.load(Ordering::SeqCst), 0);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 3);
    assert_eq!(OWNED_CREATED.load(Ordering::SeqCst), 12); // 3 batches each, 1+1+2 workers
    assert_eq!(OWNED_CONSUMED.load(Ordering::SeqCst), 12);
    assert_eq!(OWNED_DROPPED.load(Ordering::SeqCst), 12);
    assert!(
        run.cases
            .iter()
            .all(|case| case.contract["input.ownership"].starts_with("moved"))
    );
}

#[test]
fn smoke_mode_executes_each_case_once_and_preserves_correctness_checks() {
    let operations = std::cell::Cell::new(0);
    let validations = std::cell::Cell::new(0);
    let mut suite = Suite::new("smoke");
    suite.bench("plain", || operations.set(operations.get() + 1));
    suite.bench_checked(
        "checked",
        || 3,
        |input| {
            operations.set(operations.get() + 1);
            *input += 1;
            *input
        },
        |input, output| {
            validations.set(validations.get() + 1);
            assert_eq!((*input, *output), (4, 4));
            Ok(())
        },
        airbug_bench::DropPolicy::InsideTiming,
    );
    let run = suite
        .test_selected(&airbug_bench::Selection::default())
        .unwrap();
    assert_eq!(operations.get(), 2);
    assert_eq!(validations.get(), 1);
    assert_eq!(run.observations.len(), 2);
    assert!(run.observations.iter().all(|o| o.operations == 1));
    assert!(
        run.cases
            .iter()
            .all(|c| c.contract["execution.mode"] == "test_once")
    );
}

#[airbug_bench::suite(
    samples = 3,
    warmup_ms = 0,
    iterations = 7,
    exclude_external_time = true
)]
mod fixed_sampling {
    #[bench(custom = true, max_time_ms = 15)]
    fn bounded(n: u64) -> std::time::Duration {
        assert_eq!(n, 7);
        std::time::Duration::from_millis(10)
    }
    #[bench(custom = true, min_time_ms = 45)]
    fn extended(n: u64) -> std::time::Duration {
        assert_eq!(n, 7);
        std::time::Duration::from_millis(10)
    }
}

#[test]
fn fixed_sampling_inherits_options_and_obeys_measured_time_limits() {
    let mut suite = Suite::new("limits");
    fixed_sampling::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    let bounded: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case.ends_with("bounded"))
        .collect();
    let extended: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case.ends_with("extended"))
        .collect();
    assert_eq!(bounded.len(), 2);
    assert_eq!(extended.len(), 5);
    assert!(
        run.observations
            .iter()
            .all(|o| o.operations == 7 && o.value.as_deref() == Some("10000000"))
    );
    assert!(
        run.cases
            .iter()
            .all(|c| c.contract["sampling.time_accounting"] == "measured")
    );
}

#[test]
fn threaded_fixed_sample_spans_waves_without_losing_inputs() {
    use airbug_bench::{DropPolicy, Sampling};
    let setups = AtomicUsize::new(0);
    let calls = AtomicUsize::new(0);
    let mut suite = Suite::new("waves");
    suite.bench_threads_with_input(
        "fresh",
        2,
        || setups.fetch_add(1, Ordering::SeqCst),
        |_| {
            calls.fetch_add(1, Ordering::SeqCst);
        },
        DropPolicy::OutsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(130),
        samples: Some(2),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert_eq!(setups.load(Ordering::SeqCst), 520);
    assert_eq!(calls.load(Ordering::SeqCst), 520);
    assert_eq!(run.observations.len(), 2);
    assert!(run.observations.iter().all(|o| o.operations == 260));
}

#[airbug_bench::bench(
    custom = true,
    sampling = "linear",
    samples = 4,
    warmup_ms = 0,
    sample_ms = 1
)]
fn linear_cost(n: u64) -> Duration {
    Duration::from_nanos(n * 10_000)
}

#[test]
fn attributed_linear_sampling_preserves_per_operation_cost() {
    let mut suite = Suite::new("linear");
    register_linear_cost(&mut suite);
    let run = suite.run("").unwrap();
    assert_eq!(run.cases[0].contract["sampling.mode"], "linear");
    assert_eq!(
        run.observations
            .iter()
            .map(|o| o.operations)
            .collect::<Vec<_>>(),
        [40, 80, 120, 160]
    );
    for observation in run.observations {
        assert_eq!(
            observation.value.unwrap().parse::<u64>().unwrap(),
            observation.operations * 10_000
        );
    }
    assert!(!run.notes.iter().any(|note| note.contains("below half")));
}

#[airbug_bench::bench(args = [10u64, 2, 1])]
fn source_first(n: u64) -> u64 {
    n
}
#[airbug_bench::bench]
fn source_second() {}

#[test]
fn attributes_sort_by_declaration_with_stable_parameter_order() {
    use airbug_bench::{Selection, SortOrder};
    let mut suite = Suite::new("source");
    register_source_second(&mut suite);
    register_source_first(&mut suite);
    let selection = Selection {
        sort: SortOrder::Source,
        ..Default::default()
    };
    assert_eq!(
        suite.list_selected(&selection),
        [
            "source/source_first/10",
            "source/source_first/2",
            "source/source_first/1",
            "source/source_second"
        ]
    );
    let run = suite.test_selected(&selection).unwrap();
    assert_eq!(run.cases[0].id, "source/source_first/10");
    assert!(
        run.cases
            .iter()
            .all(|c| !c.contract.keys().any(|k| k.starts_with("source.")))
    );
}

#[path = "../benches/support/external.rs"]
mod imported_file;

#[airbug_bench::group(name = "bundle", groups = [crate::imported_file::cases])]
mod imported_bundle {
    #[bench]
    fn local() -> usize {
        1
    }
}

#[test]
fn external_groups_register_recursively_without_a_generated_main() {
    let mut suite = Suite::new("files");
    suite.group(
        imported_bundle::__AIRBUG_GROUP_NAME,
        imported_bundle::__airbug_register_group,
    );
    assert_eq!(
        suite.list(""),
        [
            "files/bundle/local",
            "files/bundle/external/from_file/3",
            "files/bundle/external/from_file/7",
            "files/bundle/external/nested/leaf"
        ]
    );
    let run = suite.run("external/").unwrap();
    assert_eq!(run.cases.len(), 3);
    for case in run.cases {
        assert_eq!(case.contract["samples"], "2");
        assert_eq!(
            run.observations
                .iter()
                .filter(|o| o.case == case.id)
                .count(),
            2
        );
    }
}

#[airbug_bench::bench(custom = true, args = [0usize, 4],
    bytes = |n| airbug_bench::counters::bytes_of_many::<u16>(n as u64), items = |n| n as u64)]
fn empty_work(iterations: u64, size: usize) -> Duration {
    std::hint::black_box(size);
    Duration::from_nanos(iterations * 10)
}

#[test]
fn zero_counters_are_reported_as_zero_throughput() {
    let mut suite = Suite::new("zero");
    register_empty_work(&mut suite);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    let series = airbug_bench::report::throughput(&run).unwrap();
    assert_eq!(series.len(), 4);
    let empty: Vec<_> = series.iter().filter(|s| s.case.ends_with("/0")).collect();
    assert_eq!(empty.len(), 2);
    assert!(
        empty
            .iter()
            .all(|s| !s.values.is_empty() && s.values.iter().all(|v| *v == 0.0))
    );
    assert!(
        series
            .iter()
            .filter(|s| s.case.ends_with("/4"))
            .all(|s| s.values.iter().all(|v| *v > 0.0))
    );
    let mut malformed = run.clone();
    malformed.cases[0]
        .contract
        .insert("work.counter.bytes".into(), "invalid".into());
    assert!(airbug_bench::report::throughput(&malformed).is_err());
    // Legacy artifacts also retain declared zero counters.
    let mut legacy = run;
    for case in &mut legacy.cases {
        case.contract.retain(|k, _| !k.starts_with("work.counter."));
    }
    assert!(
        airbug_bench::report::throughput(&legacy)
            .unwrap()
            .iter()
            .any(|s| s.case.ends_with("/0") && s.values.iter().all(|v| *v == 0.0))
    );
}

static CUSTOM_ASYNC_EXECUTORS: AtomicUsize = AtomicUsize::new(0);
static CUSTOM_ASYNC_CALLS: AtomicUsize = AtomicUsize::new(0);
fn custom_async_executor() -> airbug_bench::workloads::LocalExecutor {
    CUSTOM_ASYNC_EXECUTORS.fetch_add(1, Ordering::SeqCst);
    airbug_bench::workloads::LocalExecutor
}
#[airbug_bench::bench(custom = true, args = [String::from("borrowed")],
    executor = custom_async_executor(), iterations = 7)]
async fn async_exact_duration(iterations: u64, value: &str) -> Duration {
    assert_eq!(value, "borrowed");
    assert_eq!(iterations, 7);
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
    CUSTOM_ASYNC_CALLS.fetch_add(1, Ordering::SeqCst);
    Duration::from_nanos(iterations * 123)
}
#[test]
fn async_custom_timing_is_lazy_borrows_arguments_and_awaits_completion() {
    let mut suite = Suite::new("async-custom");
    register_async_exact_duration(&mut suite);
    assert_eq!(suite.list("").len(), 1);
    assert_eq!(CUSTOM_ASYNC_EXECUTORS.load(Ordering::SeqCst), 0);
    assert_eq!(CUSTOM_ASYNC_CALLS.load(Ordering::SeqCst), 0);
    suite.config(Config {
        max_iterations: 16,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    assert_eq!(CUSTOM_ASYNC_EXECUTORS.load(Ordering::SeqCst), 1);
    assert!(CUSTOM_ASYNC_CALLS.load(Ordering::SeqCst) >= run.observations.len());
    assert!(!run.observations.is_empty());
    assert!(
        run.observations
            .iter()
            .all(|o| o.operations == 7 && o.value.as_deref() == Some("861"))
    );
    assert!(run.cases[0].contract["async.scope"].contains("caller defines reported interval"));
}

#[airbug_bench::group]
mod batch_policies {
    #[bench(setup = || vec![1u8], batch = airbug_bench::BatchPolicy::PerIteration)]
    fn borrowed(input: &mut Vec<u8>) {
        input.push(2);
    }

    #[bench(setup = || vec![1u8], batch = airbug_bench::BatchPolicy::Batches(3.try_into().unwrap()))]
    fn owned(input: Vec<u8>) -> Vec<u8> {
        input
    }
}
#[test]
fn attributed_batch_policies_record_measurement_contracts() {
    let mut suite = Suite::new("batches");
    batch_policies::__airbug_register_group(&mut suite);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 2);
    assert_eq!(run.cases[0].contract["batch.policy"], "PerIteration");
    assert_eq!(run.cases[1].contract["batch.policy"], "Batches(3)");
}

#[airbug_bench::group]
mod async_batch_policies {
    #[bench(setup = || vec![1u8], input_bytes = |v: &Vec<u8>| v.len() as u64, batch = airbug_bench::BatchPolicy::PerIteration)]
    async fn borrowed(input: &mut Vec<u8>) {
        input.push(std::future::ready(2).await);
    }

    #[bench(setup = || vec![1u8], input_bytes = |v: &Vec<u8>| v.len() as u64, batch = airbug_bench::BatchPolicy::SmallInput)]
    async fn owned(input: Vec<u8>) -> Vec<u8> {
        std::future::ready(input).await
    }
}
#[test]
fn attributed_async_batch_policies_register_both_input_modes() {
    let mut suite = Suite::new("async-batches");
    async_batch_policies::__airbug_register_group(&mut suite);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 2);
    assert_eq!(run.cases[0].contract["batch.policy"], "PerIteration");
    assert_eq!(run.cases[1].contract["batch.policy"], "SmallInput");
    assert!(
        run.cases
            .iter()
            .all(|c| c.contract.contains_key("async.executor"))
    );
    assert!(
        run.observations
            .iter()
            .all(|o| o.work_totals["bytes"] == o.operations.to_string())
    );
}

#[test]
fn async_batches_wait_and_drop_before_preparing_next_batch() {
    use airbug_bench::{BatchPolicy, DropPolicy, Sampling, workloads::LocalExecutor};
    use std::{cell::RefCell, rc::Rc};
    struct Value(Rc<RefCell<Vec<&'static str>>>, &'static str);
    impl Drop for Value {
        fn drop(&mut self) {
            self.0.borrow_mut().push(self.1);
        }
    }
    async fn run(input: &mut Value) -> Value {
        let mut pending = true;
        std::future::poll_fn(|cx| {
            if std::mem::take(&mut pending) {
                input.0.borrow_mut().push("pending");
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            } else {
                input.0.borrow_mut().push("ready");
                std::task::Poll::Ready(())
            }
        })
        .await;
        Value(input.0.clone(), "output drop")
    }
    for owned in [false, true] {
        let events = Rc::new(RefCell::new(vec![]));
        let mut suite = Suite::new("lifetime");
        let executor = || {
            events.borrow_mut().push("executor");
            LocalExecutor
        };
        let setup = || {
            events.borrow_mut().push("setup");
            Value(events.clone(), "input drop")
        };
        let batch = BatchPolicy::Iterations(3.try_into().unwrap());
        if owned {
            suite.bench_async_batched(
                "owned",
                executor,
                setup,
                async |mut input| run(&mut input).await,
                DropPolicy::OutsideTiming,
                batch,
            );
        } else {
            suite.bench_async_batched_ref(
                "borrowed",
                executor,
                setup,
                async |input| run(input).await,
                DropPolicy::OutsideTiming,
                batch,
            );
        }
        suite.sampling(Sampling {
            iterations: Some(5),
            ..Sampling::default()
        });
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            max_iterations: 5,
            ..fast_config()
        });
        assert_eq!(suite.list("").len(), 1);
        assert!(events.borrow().is_empty());
        let result = suite.run("").unwrap();
        assert_eq!(result.observations[0].operations, 5);
        let mut expected = vec!["executor"];
        for count in [3, 2] {
            expected.extend(std::iter::repeat_n("setup", count));
            for _ in 0..count {
                expected.extend(["pending", "ready"]);
                if owned {
                    expected.push("input drop");
                }
            }
            expected.extend(std::iter::repeat_n("output drop", count));
            if !owned {
                expected.extend(std::iter::repeat_n("input drop", count));
            }
        }
        assert_eq!(*events.borrow(), expected);
    }
}

const TUPLE_INPUTS: [(usize, usize); 2] = [(2, 3), (4, 5)];
#[airbug_bench::bench(args = TUPLE_INPUTS, custom = true, items = |(a, b)| (a + b) as u64)]
fn tuple_input(iterations: u64, (a, b): (usize, usize)) -> Duration {
    Duration::from_nanos(iterations * (a + b) as u64)
}
fn generated_strings() -> impl Iterator<Item = (String, String)> {
    ["a", "bb"].into_iter().map(|s| (s.to_owned(), s.repeat(2)))
}
#[airbug_bench::bench(args = generated_strings(), custom = true)]
fn tuple_borrowed(iterations: u64, input: &(String, String)) -> Duration {
    Duration::from_nanos(iterations * (input.0.len() + input.1.len()) as u64)
}
#[airbug_bench::bench(types = [(u8, u16), (u32, u64)], consts = [2, 4],
    args = TUPLE_INPUTS, custom = true)]
fn multiple_types<const N: usize, T, U>(n: u64, input: (usize, usize)) -> Duration
where
    T: Copy + Default,
    U: Copy + Default,
{
    let _ = (T::default(), U::default());
    Duration::from_nanos(
        n * (std::mem::size_of::<T>() + std::mem::size_of::<U>() + N + input.0) as u64,
    )
}
#[test]
fn external_generated_tuple_args_and_multiple_types_keep_distinct_cases() {
    let mut suite = Suite::new("parameters");
    register_tuple_input(&mut suite);
    register_tuple_borrowed(&mut suite);
    register_multiple_types(&mut suite);
    assert_eq!(suite.list("").len(), 12);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 12);
    let values: Vec<_> = run
        .observations
        .iter()
        .map(|o| o.value.as_deref().unwrap())
        .collect();
    assert_eq!(&values[..8], ["5", "5", "9", "9", "3", "3", "6", "6"]);
    assert_eq!(
        &values[8..],
        [
            "7", "7", "9", "9", "9", "9", "11", "11", "16", "16", "18", "18", "18", "18", "20",
            "20"
        ]
    );
    assert!(
        run.cases
            .iter()
            .any(|c| c.id.contains("type=u8/type=u16/const=2"))
    );
    assert!(
        run.cases
            .iter()
            .any(|c| c.id.contains("type=u32/type=u64/const=4"))
    );
}

const DIMENSIONS: [usize; 2] = [2, 5];
const fn next_dimension(n: usize) -> usize {
    n + 1
}
#[airbug_bench::bench(types = [(u8, u16), (u32, u64)],
    consts = [(DIMENSIONS[0], false, 'a'), (next_dimension(DIMENSIONS[1]), true, 'b')],
    args = [1u64, 2], custom = true)]
fn multiple_consts<const N: usize, T, const FLAG: bool, U, const C: char>(
    n: u64,
    factor: u64,
) -> Duration {
    let value =
        N + std::mem::size_of::<T>() + std::mem::size_of::<U>() + usize::from(FLAG) + C as usize;
    Duration::from_nanos(n * factor * value as u64)
}
#[test]
fn const_tuple_rows_preserve_types_expressions_and_parameter_order() {
    let mut suite = Suite::new("const-rows");
    register_multiple_consts(&mut suite);
    suite.config(Config {
        samples: 1,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    let values: Vec<_> = run
        .observations
        .iter()
        .map(|o| o.value.as_deref().unwrap())
        .collect();
    assert_eq!(
        values,
        ["102", "204", "108", "216", "111", "222", "117", "234"]
    );
    assert_eq!(run.cases.len(), 8);
    assert!(
        run.cases[0]
            .id
            .contains("type=u8/type=u16/const=2/const=false/const='a'/1")
    );
    assert!(
        run.cases[7]
            .id
            .contains("type=u32/type=u64/const=6/const=true/const='b'/2")
    );
}

const EXTERNAL_CONST_ROWS: &[(usize, bool)] = &[(3, false), (7, true)];
#[airbug_bench::bench(consts = EXTERNAL_CONST_ROWS, custom = true)]
fn external_const_rows<const N: usize, const B: bool>(n: u64) -> Duration {
    Duration::from_nanos(n * (N + usize::from(B)) as u64)
}
const EXTERNAL_CONST_VALUES: [usize; 20] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];
#[airbug_bench::bench(consts = EXTERNAL_CONST_VALUES, custom = true)]
fn external_const_values<const N: usize>(n: u64) -> Duration {
    Duration::from_nanos(n * N as u64)
}
#[test]
fn external_const_arrays_and_slices_register_only_real_entries() {
    let mut suite = Suite::new("external-consts");
    register_external_const_rows(&mut suite);
    register_external_const_values(&mut suite);
    assert_eq!(suite.list("").len(), 22);
    suite.config(Config {
        samples: 1,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    let values: Vec<u64> = run
        .observations
        .iter()
        .map(|o| o.value.as_ref().unwrap().parse().unwrap())
        .collect();
    assert_eq!(values, [vec![3, 8], (1..=20).collect()].concat());
}

const ONE_CONST: &[usize] = &[7];
static EXTERNAL_ARG_GENERATIONS: AtomicUsize = AtomicUsize::new(0);
fn external_const_arguments() -> [u64; 2] {
    EXTERNAL_ARG_GENERATIONS.fetch_add(1, Ordering::SeqCst);
    [2, 3]
}
#[airbug_bench::bench(types = [u8, u16], consts = ONE_CONST,
    args = external_const_arguments(), custom = true)]
fn single_external_const<T, const N: usize>(n: u64, arg: u64) -> Duration {
    Duration::from_nanos(n * arg * (N + std::mem::size_of::<T>()) as u64)
}
#[test]
fn unused_external_const_slots_do_not_generate_arguments() {
    let mut suite = Suite::new("single-external");
    register_single_external_const(&mut suite);
    assert_eq!(EXTERNAL_ARG_GENERATIONS.load(Ordering::SeqCst), 2);
    assert_eq!(suite.list("").len(), 4);
    suite.config(Config {
        samples: 1,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    let values: Vec<_> = run
        .observations
        .iter()
        .map(|o| o.value.as_deref().unwrap())
        .collect();
    assert_eq!(values, ["16", "24", "18", "27"]);
    assert_eq!(EXTERNAL_ARG_GENERATIONS.load(Ordering::SeqCst), 2);
}

#[airbug_bench::group(threads = [2], setup_thread = "worker", iterations = 130, samples = 1, warmup_ms = 0)]
mod worker_inputs {
    use std::{cell::Cell, rc::Rc};
    #[bench(setup = || Rc::new(Cell::new(1u64)), drop_output = "outside", input_items = |input: &Rc<Cell<u64>>| input.get(), input_bytes = |_| 0)]
    fn borrowed(value: &mut Rc<Cell<u64>>) -> Rc<Cell<u64>> {
        assert_eq!(value.replace(2), 1);
        value.clone()
    }
    #[bench(setup = || Rc::new(Cell::new(1u64)), drop_output = "outside", input_items = |input: &Rc<Cell<u64>>| input.get(), input_bytes = |_| 0)]
    fn owned(value: Rc<Cell<u64>>) -> Rc<Cell<u64>> {
        assert_eq!(value.replace(2), 1);
        value
    }
}
#[test]
fn worker_local_attributes_accept_non_send_inputs_and_outputs_across_waves() {
    let mut suite = Suite::new("worker-inputs");
    worker_inputs::__airbug_register_group(&mut suite);
    suite.config(Config {
        max_iterations: 130,
        ..fast_config()
    });
    assert_eq!(suite.list("").len(), 2);
    let run = suite.run("").unwrap();
    assert_eq!(run.observations.len(), 2);
    assert!(run.observations.iter().all(|o| o.operations == 260));
    assert!(
        run.cases
            .iter()
            .all(|c| c.contract["threads.setup"] == "worker")
    );
    assert_eq!(
        run.cases[1].contract["input.ownership"],
        "moved; setup excluded; consumed input drop inside operation"
    );
    assert!(
        run.observations
            .iter()
            .all(|o| o.work_totals["items"] == "260" && o.work_totals["bytes"] == "0")
    );
}

static VARIABLE_INPUTS: AtomicUsize = AtomicUsize::new(0);
#[airbug_bench::bench(threads = [2], setup_thread = "worker", iterations = 130,
    samples = 2, warmup_ms = 0,
    setup = || vec![0u8; VARIABLE_INPUTS.fetch_add(1, Ordering::SeqCst) % 7],
    input_bytes = |input: &Vec<u8>| input.len() as u64, input_items = |_| 1)]
fn variable_worker_input(input: &mut Vec<u8>) {
    input.clear();
}
#[test]
fn dynamic_worker_counts_sum_actual_inputs_before_mutation() {
    let mut suite = Suite::new("variable-input");
    register_variable_worker_input(&mut suite);
    assert_eq!(suite.list("").len(), 1);
    assert_eq!(VARIABLE_INPUTS.load(Ordering::SeqCst), 0);
    suite.config(Config {
        max_iterations: 130,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.observations.len(), 2);
    for (index, observation) in run.observations.iter().enumerate() {
        let expected: usize = (index * 260..(index + 1) * 260).map(|n| n % 7).sum();
        assert_eq!(observation.work_totals["bytes"], expected.to_string());
        assert_eq!(observation.work_totals["items"], "260");
    }
    assert_eq!(VARIABLE_INPUTS.load(Ordering::SeqCst), 520);
}

#[airbug_bench::bench(setup = || vec![1u8], iterations = 3, samples = 2, warmup_ms = 0,
    input_items = { let mut calls = 0; move |_: &Vec<u8>| { calls += 1; calls } })]
fn stateful_input_counter(input: &mut Vec<u8>) {
    input.clear();
}
#[test]
fn input_counter_closure_is_constructed_once_per_case() {
    let mut suite = Suite::new("stateful-counter");
    register_stateful_input_counter(&mut suite);
    suite.config(Config {
        max_iterations: 3,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.observations[0].work_totals["items"], "6");
    assert_eq!(run.observations[1].work_totals["items"], "15");
}

static THREADED_ASYNC_EXECUTORS: AtomicUsize = AtomicUsize::new(0);
static THREADED_ASYNC_DROPS: AtomicUsize = AtomicUsize::new(0);
static THREADED_ASYNC_CALLS: AtomicUsize = AtomicUsize::new(0);
struct WorkerExecutor {
    origin: std::thread::ThreadId,
    _local: std::rc::Rc<()>,
}
impl airbug_bench::workloads::Executor for WorkerExecutor {
    fn block_on<F: std::future::Future>(&mut self, future: F) -> F::Output {
        assert_eq!(self.origin, std::thread::current().id());
        airbug_bench::workloads::LocalExecutor.block_on(future)
    }
}
impl Drop for WorkerExecutor {
    fn drop(&mut self) {
        assert_eq!(self.origin, std::thread::current().id());
        THREADED_ASYNC_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
fn worker_executor() -> WorkerExecutor {
    THREADED_ASYNC_EXECUTORS.fetch_add(1, Ordering::SeqCst);
    WorkerExecutor {
        origin: std::thread::current().id(),
        _local: std::rc::Rc::new(()),
    }
}
async fn worker_yield() {
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
    THREADED_ASYNC_CALLS.fetch_add(1, Ordering::SeqCst);
}
#[airbug_bench::group(threads = [2], iterations = 130, samples = 1, warmup_ms = 0)]
mod threaded_async {
    use super::{worker_executor, worker_yield};
    use std::rc::Rc;
    #[bench(executor = worker_executor())]
    async fn plain() {
        worker_yield().await;
    }
    #[bench(executor = worker_executor(), args = [String::from("hello")])]
    async fn argument(value: &str) {
        worker_yield().await;
        assert_eq!(value, "hello");
    }
    #[bench(executor = worker_executor(), setup = || Rc::new(3u64), input_items = |_| 1, drop_output = "outside")]
    async fn borrowed(value: &mut Rc<u64>) -> Rc<u64> {
        worker_yield().await;
        assert_eq!(**value, 3);
        value.clone()
    }
    #[bench(executor = worker_executor(), setup = || Rc::new(7u64), input_items = |_| 1, drop_output = "outside")]
    async fn owned(value: Rc<u64>) -> Rc<u64> {
        worker_yield().await;
        assert_eq!(*value, 7);
        value
    }
}
#[test]
fn threaded_async_keeps_executors_and_non_send_values_on_each_worker() {
    let mut suite = Suite::new("threaded-async");
    threaded_async::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 4);
    assert_eq!(THREADED_ASYNC_EXECUTORS.load(Ordering::SeqCst), 0);
    assert_eq!(THREADED_ASYNC_CALLS.load(Ordering::SeqCst), 0);
    suite.config(Config {
        max_iterations: 130,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.observations.len(), 4);
    assert!(run.observations.iter().all(|o| o.operations == 260));
    assert_eq!(THREADED_ASYNC_CALLS.load(Ordering::SeqCst), 1040);
    assert_eq!(THREADED_ASYNC_EXECUTORS.load(Ordering::SeqCst), 24);
    assert_eq!(THREADED_ASYNC_DROPS.load(Ordering::SeqCst), 24);
    for o in &run.observations[2..] {
        assert_eq!(o.work_totals["items"], "260");
    }
}

#[airbug_bench::group]
#[allow(clippy::needless_lifetimes)]
mod explicit_lifetimes {
    #[bench(types = [fn(&str) -> usize, Box<dyn Fn(&str) -> usize>], args = [String::from("hello")])]
    fn higher_ranked<'input, F>(input: &'input str) -> usize
    where
        F: for<'other> Fn(&'other str) -> usize,
    {
        input.len() + std::mem::size_of::<F>()
    }
    type T = u16;
    const N: usize = 3;
    #[bench(types = [T], consts = [N], args = [String::from("value")], custom = true)]
    fn collisions<'input, T, const N: usize>(n: u64, input: &'input str) -> std::time::Duration {
        std::time::Duration::from_nanos(n * (std::mem::size_of::<T>() + N + input.len()) as u64)
    }
    #[bench(types = [&str, fn(&str) -> usize], consts = [2, 4], setup = || String::from("x"))]
    fn shapes<'input, T, const N: usize>(input: &'input mut String) -> usize {
        assert_eq!(input, "x");
        std::mem::size_of::<T>() + N
    }
    #[bench(args = [("left", "right")])]
    fn related<'long: 'short, 'short>(input: &'short (&'long str, &'long str)) -> usize {
        input.0.len() + input.1.len()
    }

    #[bench(args = [String::from("hello")])]
    fn borrowed<'input>(input: &'input str) -> usize {
        input.len()
    }
    #[bench(setup = || vec![1u8])]
    fn fresh<'input>(input: &'input mut Vec<u8>) {
        input.push(2);
    }
    #[bench(setup = || vec![1u8])]
    async fn future<'input>(input: &'input mut Vec<u8>) {
        input.push(std::future::ready(2).await);
    }
    #[bench(types = [String], args = [String::from("value")])]
    fn constrained<'input, T>(input: &'input str) -> T
    where
        T: From<&'input str>,
    {
        T::from(input)
    }
}
#[test]
fn explicit_input_lifetimes_support_sync_async_and_type_bounds() {
    let mut suite = Suite::new("lifetimes");
    explicit_lifetimes::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 12);
    suite.config(fast_config());
    let run = suite.run("").unwrap();
    assert_eq!(run.observations.len(), 24);
}

static AUTO_WORK: AtomicUsize = AtomicUsize::new(0);
#[airbug_bench::bench(threads, samples = 1, warmup_ms = 0, iterations = 1)]
fn automatic_workers() {
    AUTO_WORK.fetch_add(1, Ordering::SeqCst);
}
#[airbug_bench::bench(threads = 2, samples = 1, warmup_ms = 0, iterations = 1)]
fn scalar_workers() {}
const SELECTED_WORKERS: &[usize] = &[1, 2, 2];
#[airbug_bench::bench(threads = SELECTED_WORKERS, samples = 1, warmup_ms = 0, iterations = 1)]
fn slice_workers() {}
#[airbug_bench::bench(threads = 1..=2, samples = 1, warmup_ms = 0, iterations = 1)]
fn range_workers() {}
#[airbug_bench::group(threads = true, setup_thread = "worker")]
mod local_override {
    #[bench(threads = false, args = [std::rc::Rc::new(7)])]
    fn local(value: &std::rc::Rc<usize>) -> usize {
        **value
    }
}
#[test]
fn automatic_scalar_slice_range_and_local_override_are_lazy() {
    let mut suite = Suite::new("thread-selection");
    register_automatic_workers(&mut suite);
    register_scalar_workers(&mut suite);
    register_slice_workers(&mut suite);
    register_range_workers(&mut suite);
    local_override::__airbug_register_group(&mut suite);
    let names = suite.list("");
    assert_eq!(names.len(), 7);
    assert_eq!(AUTO_WORK.load(Ordering::SeqCst), 0);
    assert!(names.iter().any(|n| n.ends_with(&format!(
        "automatic_workers/threads={}",
        airbug_bench::threads::available()
    ))));
    suite.config(Config {
        samples: 1,
        ..fast_config()
    });
    let run = suite.run("").unwrap();
    assert_eq!(
        AUTO_WORK.load(Ordering::SeqCst),
        airbug_bench::threads::available()
    );
    assert_eq!(
        run.observations[0].operations,
        airbug_bench::threads::available() as u64
    );
    assert!(!run.cases.last().unwrap().contract.contains_key("threads"));
}

mod output_batches {
    use std::sync::Mutex;
    static EVENTS: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
    pub struct Output(Vec<u8>);
    impl Drop for Output {
        fn drop(&mut self) {
            assert_eq!(self.0.len(), 4096);
            EVENTS.lock().unwrap().push("drop");
        }
    }
    fn output() -> Output {
        EVENTS.lock().unwrap().push("make");
        Output(vec![0; 4096])
    }
    #[airbug_bench::bench(drop_output = "outside", batch = airbug_bench::BatchPolicy::Iterations(3.try_into().unwrap()), iterations = 7, samples = 1, warmup_ms = 0)]
    fn sync_output() -> Output {
        output()
    }
    #[airbug_bench::bench(args = [String::from("borrowed")], drop_output = "outside", batch = airbug_bench::BatchPolicy::Iterations(3.try_into().unwrap()), iterations = 7, samples = 1, warmup_ms = 0)]
    async fn async_output(arg: &str) -> Output {
        assert_eq!(arg, "borrowed");
        output()
    }
    #[test]
    fn plain_outputs_follow_explicit_batch_lifetimes_in_sync_and_async() {
        for register in [register_sync_output, register_async_output] {
            EVENTS.lock().unwrap().clear();
            let mut suite = airbug_bench::Suite::new("outputs");
            register(&mut suite);
            assert!(EVENTS.lock().unwrap().is_empty(), "registration is lazy");
            let run = suite.run("").unwrap();
            assert_eq!(run.observations.len(), 1);
            assert_eq!(run.observations[0].operations, 7);
            assert_eq!(run.cases[0].contract["output.drop"], "outside");
            assert_eq!(run.cases[0].contract["batch.policy"], "Iterations(3)");
            assert_eq!(
                *EVENTS.lock().unwrap(),
                [
                    "make", "make", "make", "drop", "drop", "drop", "make", "make", "make", "drop",
                    "drop", "drop", "make", "drop",
                ]
            );
        }
    }
}

#[airbug_bench::group]
mod kind_families {
    #[bench(types = [u16, u8], consts = [10usize, 2])]
    fn aaa<T, const N: usize>() -> usize {
        std::mem::size_of::<T>() + N
    }
    #[bench(args = [10usize, 2])]
    fn zzz(n: usize) -> usize {
        n
    }
    #[airbug_bench::group]
    mod bbb {
        #[bench]
        fn child() {}
    }
}
#[test]
fn kind_sort_treats_generic_families_as_parents_and_value_args_as_leaves() {
    use airbug_bench::{Selection, SortOrder};
    let mut suite = Suite::new("root");
    kind_families::__airbug_register_group(&mut suite);
    let selection = Selection {
        sort: SortOrder::Kind,
        ..Default::default()
    };
    let names: Vec<_> = suite
        .list_selected(&selection)
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert!(names[0].ends_with("/zzz/2"), "{names:?}");
    assert!(names[1].ends_with("/zzz/10"), "{names:?}");
    assert!(
        names[2..6].iter().all(|n| n.contains("/aaa/type=")),
        "{names:?}"
    );
    assert!(names[6].ends_with("/bbb/child"), "{names:?}");
    let run = suite.test_selected(&selection).unwrap();
    assert_eq!(
        run.cases.into_iter().map(|c| c.id).collect::<Vec<_>>(),
        names
    );
}

static GROUP_EXECUTOR_DEFAULTS: AtomicUsize = AtomicUsize::new(0);
static GROUP_EXECUTOR_OVERRIDES: AtomicUsize = AtomicUsize::new(0);
fn group_default_executor() -> airbug_bench::workloads::LocalExecutor {
    GROUP_EXECUTOR_DEFAULTS.fetch_add(1, Ordering::SeqCst);
    airbug_bench::workloads::LocalExecutor
}
fn group_override_executor() -> airbug_bench::workloads::LocalExecutor {
    GROUP_EXECUTOR_OVERRIDES.fetch_add(1, Ordering::SeqCst);
    airbug_bench::workloads::LocalExecutor
}
#[airbug_bench::group(executor = crate::group_default_executor(), samples = 1, iterations = 1, warmup_ms = 0)]
mod executor_defaults {
    #[bench]
    fn synchronous() {}
    #[bench]
    async fn asynchronous() {}
    mod nested {
        #[bench]
        async fn inherited() {}
        #[bench(executor = crate::group_override_executor())]
        async fn overridden_case() {}
    }
    #[group(executor = crate::group_override_executor())]
    mod overridden_group {
        #[bench]
        async fn asynchronous() {}
        #[bench]
        fn synchronous() {}
    }
}
#[test]
fn group_executor_defaults_are_lazy_async_only_and_locally_overridable() {
    GROUP_EXECUTOR_DEFAULTS.store(0, Ordering::SeqCst);
    GROUP_EXECUTOR_OVERRIDES.store(0, Ordering::SeqCst);
    let mut suite = Suite::new("executor-defaults");
    executor_defaults::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 6);
    assert_eq!(GROUP_EXECUTOR_DEFAULTS.load(Ordering::SeqCst), 0);
    assert_eq!(GROUP_EXECUTOR_OVERRIDES.load(Ordering::SeqCst), 0);
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 6);
    assert_eq!(GROUP_EXECUTOR_DEFAULTS.load(Ordering::SeqCst), 2);
    assert_eq!(GROUP_EXECUTOR_OVERRIDES.load(Ordering::SeqCst), 2);
    for case in &run.cases {
        let synchronous = case.id.ends_with("/synchronous");
        assert_eq!(case.contract.contains_key("async.executor"), !synchronous);
    }
}

#[airbug_bench::group(samples = 2)]
mod imported_sampling_leaf {
    #[bench]
    fn inherited() {}
    #[bench(samples = 1, iterations = 5)]
    fn overridden() {}
}
#[airbug_bench::group(groups = [crate::imported_sampling_leaf], iterations = 3)]
mod imported_sampling_middle {}
#[airbug_bench::group(groups = [crate::imported_sampling_middle], samples = 4,
    iterations = 7, warmup_ms = 0, sample_ms = 1, sampling = "flat",
    min_time_ms = 0, max_time_ms = 1000, exclude_external_time = true)]
mod imported_sampling_parent {
    #[bench]
    fn sibling() {}
}
#[test]
fn imported_groups_inherit_sampling_per_field_with_nearest_override() {
    let mut suite = Suite::new("imported-defaults");
    imported_sampling_parent::__airbug_register_group(&mut suite);
    suite.bench("unrelated", || ());
    suite.sampling(airbug_bench::Sampling {
        samples: Some(1),
        iterations: Some(1),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    assert_eq!(suite.list("").len(), 4);
    let run = suite.run("").unwrap();
    for case in &run.cases {
        let observations: Vec<_> = run
            .observations
            .iter()
            .filter(|o| o.case == case.id)
            .collect();
        let (samples, operations) = if case.id.ends_with("/inherited") {
            (2, 3)
        } else if case.id.ends_with("/overridden") {
            (1, 5)
        } else if case.id.ends_with("/sibling") {
            (4, 7)
        } else {
            (1, 1)
        };
        assert_eq!(observations.len(), samples, "{}", case.id);
        assert!(
            observations.iter().all(|o| o.operations == operations),
            "{}",
            case.id
        );
        if !case.id.ends_with("/unrelated") {
            assert_eq!(case.contract["sampling.mode"], "flat");
            assert_eq!(case.contract["sampling.min_time_ns"], "0");
            assert_eq!(case.contract["sampling.max_time_ns"], "1000000000");
            assert_eq!(case.contract["sampling.time_accounting"], "measured");
        }
    }
}

#[airbug_bench::group(items = 2)]
mod imported_counter_leaf {
    #[bench]
    fn inherited() {}
    #[bench(ignore = false, bytes = 0)]
    fn enabled_zero() {}
    #[bench(setup = || vec![1u8; 7], input_bytes = |v: &Vec<u8>| v.len() as u64)]
    fn dynamic(input: &mut Vec<u8>) {
        std::hint::black_box(input);
    }
}
#[airbug_bench::group(groups = [crate::imported_counter_leaf], items = 3, chars = 4)]
mod imported_counter_middle {}
#[airbug_bench::group(groups = [crate::imported_counter_middle], ignore = true,
    bytes = 128, items = 9, chars = 8, cycles = 16, samples = 1, iterations = 2, warmup_ms = 0)]
mod imported_counter_parent {}
#[test]
fn imported_groups_preserve_false_zero_dynamic_counters_and_nearest_defaults() {
    let mut suite = Suite::new("import-counters");
    imported_counter_parent::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 1);
    assert!(suite.list("")[0].ends_with("/enabled_zero"));
    assert_eq!(
        suite
            .list_selected(&airbug_bench::Selection {
                only_ignored: true,
                ..Default::default()
            })
            .len(),
        2
    );
    let run = suite
        .run_selected(&airbug_bench::Selection {
            include_ignored: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(run.cases.len(), 3);
    for case in &run.cases {
        assert_eq!(case.contract["work.counter.items"], "2");
        assert_eq!(case.contract["work.counter.chars"], "4");
        assert_eq!(case.contract["work.counter.cycles"], "16");
        if case.id.ends_with("/dynamic") {
            assert!(!case.contract.contains_key("work.counter.bytes"));
            let o = run.observations.iter().find(|o| o.case == case.id).unwrap();
            assert_eq!(o.work_totals["bytes"], "14");
        } else {
            assert_eq!(
                case.contract["work.counter.bytes"],
                if case.id.ends_with("/enabled_zero") {
                    "0"
                } else {
                    "128"
                }
            );
        }
    }
}

#[airbug_bench::group]
mod plot_imported {
    #[bench]
    fn inherited() {
        panic!("registration must be lazy");
    }
    #[bench(summary_scale = "linear")]
    fn explicit() {
        panic!("registration must be lazy");
    }
}
#[airbug_bench::group(summary_scale = "logarithmic", groups = [crate::plot_imported])]
mod plot_outer {
    #[bench]
    fn inherited() {
        panic!("registration must be lazy");
    }
    #[group(summary_scale = "linear")]
    mod inner {
        #[bench]
        fn inherited() {
            panic!("registration must be lazy");
        }
    }
}
#[test]
fn summary_scale_attributes_inherit_inline_and_imported_groups() {
    use airbug_bench::viz::charts::AxisScale::{Linear, Logarithmic};
    let mut suite = Suite::new("plots");
    suite.group("plot_outer", plot_outer::__airbug_register_group);
    let scales = suite.summary_scales(&Default::default()).unwrap();
    assert_eq!(scales["plots/plot_outer/inherited"], Logarithmic);
    assert_eq!(scales["plots/plot_outer/inner/inherited"], Linear);
    assert_eq!(
        scales["plots/plot_outer/plot_imported/inherited"],
        Logarithmic
    );
    assert_eq!(scales["plots/plot_outer/plot_imported/explicit"], Linear);
}

mod plot_suite_test {
    use super::*;
    #[airbug_bench::suite(summary_scale = "logarithmic")]
    mod plot_suite {
        #[bench]
        fn inherited() {
            panic!("registration must be lazy");
        }
        #[bench(summary_scale = "linear")]
        fn explicit() {
            panic!("registration must be lazy");
        }
    }
    #[test]
    fn summary_scale_suite_attribute_preserves_case_override() {
        use airbug_bench::viz::charts::AxisScale::{Linear, Logarithmic};
        let mut suite = Suite::new("plots");
        plot_suite::__airbug_register_group(&mut suite);
        let scales = suite.summary_scales(&Default::default()).unwrap();
        assert_eq!(scales["plots/inherited"], Logarithmic);
        assert_eq!(scales["plots/explicit"], Linear);
    }
}

#[airbug_bench::group]
mod family_import {
    #[bench(args = [1usize, 10])]
    fn sort(n: usize) {
        std::hint::black_box(n);
    }
    #[bench(args = [1usize, 10], summary_family = "explicit")]
    fn other(n: usize) {
        std::hint::black_box(n);
    }
}
#[airbug_bench::group(summary_family = "default", groups = [crate::family_import])]
mod family_parent {
    #[bench(args = [1usize, 10])]
    fn local(n: usize) {
        std::hint::black_box(n);
    }
}
#[test]
fn summary_family_attributes_persist_inline_and_imported_identities() {
    let mut suite = Suite::new("family");
    suite.group("parent", family_parent::__airbug_register_group);
    let run = suite.test_selected(&Default::default()).unwrap();
    let families = airbug_bench::presentation::load_families(&run).unwrap();
    assert_eq!(families["family/parent/local/1"], "family/parent/default");
    assert_eq!(
        families["family/parent/family_import/sort/10"],
        "family/parent/family_import/default"
    );
    assert_eq!(
        families["family/parent/family_import/other/1"],
        "family/parent/family_import/explicit"
    );
    let chart = airbug_bench::report::parameter_charts(
        &run,
        &airbug_bench::report::SummaryPlot {
            estimator: Default::default(),
            parameter: "arg",
            scale: airbug_bench::viz::charts::AxisScale::Linear,
        },
    )
    .unwrap();
    assert!(chart.contains("Lines connect observed estimates"));
    assert!(!chart.contains("duplicate inputs"));
}

thread_local! { static MEASURED_TICKS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
struct AttributeMeasure;
impl airbug_bench::measurement::Measurement for AttributeMeasure {
    type Start = u64;
    type Value = u64;
    fn metric(&self) -> airbug_bench::Metric {
        airbug_bench::Metric::duration("ticks", "attribute-fixture", "batch_total")
    }
    fn start(&mut self) -> airbug_bench::Result<u64> {
        Ok(MEASURED_TICKS.get())
    }
    fn end(&mut self, start: u64) -> airbug_bench::Result<u64> {
        Ok(MEASURED_TICKS.get() - start)
    }
    fn zero(&self) -> u64 {
        0
    }
    fn add(&self, a: u64, b: u64) -> airbug_bench::Result<u64> {
        Ok(a + b)
    }
    fn to_f64(&self, v: &u64) -> airbug_bench::Result<f64> {
        Ok(*v as f64)
    }
}
struct AttributeFormatter;
impl airbug_bench::measurement::ValueFormatter for AttributeFormatter {
    fn scale_values(
        &self,
        _: f64,
        values: &[f64],
    ) -> airbug_bench::Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.iter().map(|v| v / 3.0).collect(),
            unit: "triples".into(),
        })
    }
    fn scale_for_machines(
        &self,
        values: &[f64],
    ) -> airbug_bench::Result<airbug_bench::measurement::FormattedValues> {
        self.scale_values(0.0, values)
    }
    fn scale_throughputs(
        &self,
        _: f64,
        work: f64,
        unit: &str,
        values: &[f64],
    ) -> airbug_bench::Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.iter().map(|v| work / v).collect(),
            unit: format!("{unit}/tick"),
        })
    }
}
fn measured_tick() {
    MEASURED_TICKS.set(MEASURED_TICKS.get() + 3);
}
fn measured_setup() -> Vec<u8> {
    MEASURED_TICKS.set(MEASURED_TICKS.get() + 100);
    vec![1; 6]
}

#[airbug_bench::group(measurement = super::AttributeMeasure, formatter = super::AttributeFormatter, samples = 2, iterations = 5, warmup_ms = 0)]
mod attributed_measurements {
    #[bench]
    fn plain() {
        super::measured_tick();
    }
    #[bench(setup = super::measured_setup, input_items = |v: &Vec<u8>| v.len() as u64)]
    fn borrowed(v: &mut [u8]) {
        v[0] = 2;
        super::measured_tick();
    }
    #[bench(setup = super::measured_setup, drop_output = "outside")]
    fn owned(v: Vec<u8>) -> Vec<u8> {
        super::measured_tick();
        v
    }
    #[bench]
    async fn future() {
        super::measured_tick();
    }
    #[bench(setup = super::measured_setup)]
    async fn future_borrowed(v: &mut [u8]) {
        v[0] = 2;
        super::measured_tick();
    }
    #[bench(setup = super::measured_setup, drop_output = "outside")]
    async fn future_owned(v: Vec<u8>) -> Vec<u8> {
        super::measured_tick();
        v
    }
    #[bench(args = [2usize, 4])]
    fn argument(n: usize) {
        for _ in 0..n {
            super::measured_tick();
        }
    }
}

#[test]
fn measurement_attributes_cover_sync_async_inputs_and_formatter() {
    let mut suite = Suite::new("attributes");
    suite.group("measured", attributed_measurements::__airbug_register_group);
    let before = MEASURED_TICKS.get();
    assert_eq!(suite.list("").len(), 8);
    assert_eq!(MEASURED_TICKS.get(), before);
    let run = suite.run("").unwrap();
    for o in run.observations.iter().filter(|o| o.metric == "ticks") {
        let expected = if o.case.ends_with("argument/2") {
            30.0
        } else if o.case.ends_with("argument/4") {
            60.0
        } else {
            15.0
        };
        assert_eq!(o.number().unwrap(), Some(expected), "{}", o.case);
        assert_eq!(o.operations, 5);
    }
    let formatted = suite.formatted_metrics(&run).unwrap();
    assert_eq!(formatted.len(), 8);
    let borrowed = formatted
        .iter()
        .find(|m| m.case.ends_with("/borrowed"))
        .unwrap();
    assert_eq!(borrowed.human[0].value, Some(1.0));
    assert_eq!(
        borrowed.throughput["items"].observations[0].value,
        Some(2.0)
    );
}

static ARG_GENERATIONS: AtomicUsize = AtomicUsize::new(0);
static ARG_DROPS: AtomicUsize = AtomicUsize::new(0);
static ARG_CALLS: [AtomicUsize; 2] = [AtomicUsize::new(0), AtomicUsize::new(0)];
static ARG_ADDRESSES: [AtomicUsize; 2] = [AtomicUsize::new(0), AtomicUsize::new(0)];
#[derive(Debug)]
struct PersistentArgument {
    id: usize,
}
impl Drop for PersistentArgument {
    fn drop(&mut self) {
        ARG_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
fn persistent_arguments() -> impl Iterator<Item = PersistentArgument> {
    ARG_GENERATIONS.fetch_add(1, Ordering::SeqCst);
    (0..2).map(|id| PersistentArgument { id })
}
fn inspect_persistent_argument(arg: &PersistentArgument) {
    let pointer = arg as *const PersistentArgument as usize;
    let previous = ARG_ADDRESSES[arg.id].swap(pointer, Ordering::SeqCst);
    assert!(
        previous == 0 || previous == pointer,
        "argument moved between operations"
    );
    assert_eq!(ARG_DROPS.load(Ordering::SeqCst), 0);
    ARG_CALLS[arg.id].fetch_add(1, Ordering::SeqCst);
}
#[airbug_bench::bench(args = persistent_arguments(), samples = 2, iterations = 3, warmup_ms = 0)]
fn persistent_sync(arg: &PersistentArgument) {
    inspect_persistent_argument(arg);
}
#[airbug_bench::bench(args = persistent_arguments(), samples = 2, iterations = 3, warmup_ms = 0)]
async fn persistent_async(arg: &PersistentArgument) {
    inspect_persistent_argument(arg);
}

#[test]
fn generated_nonclone_arguments_live_across_samples_and_drop_with_suite() {
    for asynchronous in [false, true] {
        ARG_GENERATIONS.store(0, Ordering::SeqCst);
        ARG_DROPS.store(0, Ordering::SeqCst);
        for counter in ARG_CALLS.iter().chain(&ARG_ADDRESSES) {
            counter.store(0, Ordering::SeqCst);
        }
        let mut suite = Suite::new("lifetime");
        if asynchronous {
            register_persistent_async(&mut suite);
        } else {
            register_persistent_sync(&mut suite);
        }
        assert_eq!(ARG_GENERATIONS.load(Ordering::SeqCst), 1);
        assert_eq!(suite.list("").len(), 2);
        assert_eq!(ARG_CALLS[0].load(Ordering::SeqCst), 0);
        assert_eq!(ARG_CALLS[1].load(Ordering::SeqCst), 0);
        assert_eq!(ARG_DROPS.load(Ordering::SeqCst), 0);
        let run = suite.run("").unwrap();
        assert_eq!(run.cases.len(), 2);
        assert_eq!(ARG_GENERATIONS.load(Ordering::SeqCst), 1);
        for calls in &ARG_CALLS {
            assert_eq!(calls.load(Ordering::SeqCst), 6);
        }
        assert_eq!(ARG_DROPS.load(Ordering::SeqCst), 0);
        drop(suite);
        assert_eq!(ARG_DROPS.load(Ordering::SeqCst), 2);
    }
}

const SHARED_ARGUMENTS: &[usize] = &[2, 7];
#[airbug_bench::bench(args = SHARED_ARGUMENTS, custom = true, iterations = 3, samples = 1, warmup_ms = 0)]
fn shared_slice_value(n: u64, value: usize) -> Duration {
    Duration::from_nanos(n * value as u64)
}
#[airbug_bench::bench(args = SHARED_ARGUMENTS, custom = true, iterations = 3, samples = 1, warmup_ms = 0)]
async fn shared_slice_async_value(n: u64, value: usize) -> Duration {
    Duration::from_nanos(n * value as u64)
}
#[test]
fn shared_slices_supply_copy_values_to_sync_and_async_cases() {
    let mut suite = Suite::new("shared");
    register_shared_slice_value(&mut suite);
    register_shared_slice_async_value(&mut suite);
    assert_eq!(suite.list("").len(), 4);
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations
            .iter()
            .map(|o| o.number().unwrap().unwrap())
            .collect::<Vec<_>>(),
        [6.0, 21.0, 6.0, 21.0]
    );
    assert_eq!(SHARED_ARGUMENTS, &[2, 7]);
}

#[derive(Clone, Copy)]
struct DisplayArgument(usize);
impl std::fmt::Display for DisplayArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "size-{}", self.0)
    }
}
#[airbug_bench::bench(args = [DisplayArgument(2), DisplayArgument(7)], custom = true, samples = 1, iterations = 3, warmup_ms = 0)]
fn display_only_value(n: u64, arg: DisplayArgument) -> Duration {
    Duration::from_nanos(n * arg.0 as u64)
}
struct BorrowedDisplayArgument(String);
impl std::fmt::Display for BorrowedDisplayArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[airbug_bench::bench(args = [BorrowedDisplayArgument("borrowed-label".into())], custom = true, samples = 1, iterations = 3, warmup_ms = 0)]
async fn display_only_borrowed(n: u64, arg: &BorrowedDisplayArgument) -> Duration {
    Duration::from_nanos(n * arg.0.len() as u64)
}
#[derive(Clone, Copy, Debug)]
struct BothArgument;
impl std::fmt::Display for BothArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("display-must-not-rename-existing-case")
    }
}
#[airbug_bench::bench(args = [BothArgument], custom = true, samples = 1, iterations = 3, warmup_ms = 0)]
fn both_argument(n: u64, _: BothArgument) -> Duration {
    Duration::from_nanos(n)
}
#[test]
fn display_only_argument_labels_work_without_renaming_debug_cases() {
    let mut suite = Suite::new("labels");
    register_display_only_value(&mut suite);
    register_display_only_borrowed(&mut suite);
    register_both_argument(&mut suite);
    assert_eq!(
        suite.list(""),
        [
            "labels/display_only_value/size-2",
            "labels/display_only_value/size-7",
            "labels/display_only_borrowed/borrowed-label",
            "labels/both_argument/BothArgument"
        ]
    );
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations
            .iter()
            .map(|o| o.number().unwrap().unwrap())
            .collect::<Vec<_>>(),
        [6.0, 21.0, 42.0, 3.0]
    );
    assert_eq!(run.cases[0].contract["param.arg"], "size-2");
}

static SHARED_SETUP_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
fn shared_setup(size: usize) -> Vec<u8> {
    SHARED_SETUP_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    vec![1; size]
}
#[airbug_bench::bench(args = SHARED_ARGUMENTS, setup = shared_setup, input_bytes = |v: &Vec<u8>| v.len() as u64, iterations = 3, samples = 1, warmup_ms = 0)]
fn shared_slice_setup(data: &mut [u8]) {
    assert!(matches!(data.len(), 2 | 7));
    assert_eq!(data[0], 1);
    data[0] = 9;
}
#[airbug_bench::bench(args = SHARED_ARGUMENTS, setup = |n: usize| shared_setup(n), input_bytes = |v: &Vec<u8>| v.len() as u64, iterations = 3, samples = 1, warmup_ms = 0)]
async fn shared_slice_async_setup(data: Vec<u8>) {
    assert!(matches!(data.len(), 2 | 7));
    assert_eq!(data[0], 1);
}
#[test]
fn shared_slices_setup_accepts_owned_parameters_without_iterator_adapters() {
    use std::sync::atomic::Ordering;
    SHARED_SETUP_CALLS.store(0, Ordering::SeqCst);
    let mut suite = Suite::new("shared-setup");
    register_shared_slice_setup(&mut suite);
    register_shared_slice_async_setup(&mut suite);
    assert_eq!(suite.list("").len(), 4);
    assert_eq!(SHARED_SETUP_CALLS.load(Ordering::SeqCst), 0);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    assert_eq!(SHARED_SETUP_CALLS.load(Ordering::SeqCst), 12);
    for observation in &run.observations {
        assert_eq!(observation.operations, 3);
        let size = if observation.case.ends_with("/2") {
            2
        } else {
            7
        };
        assert_eq!(observation.work_totals["bytes"], (3 * size).to_string());
    }
}

#[airbug_bench::bench(args = ["ab", "abcdefg"], setup = |s| vec![0u8; s.len()])]
fn inferred_string_setup(data: &mut [u8]) {
    std::hint::black_box(data);
}
#[test]
fn untyped_setup_infers_method_receiver_from_argument_values() {
    let mut suite = Suite::new("inference");
    register_inferred_string_setup(&mut suite);
    assert_eq!(suite.list("").len(), 2);
}

static MATRIX_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[airbug_bench::bench(threads = [2, 3], iterations = 3, samples = 1, warmup_ms = 0)]
fn replaceable_thread_matrix() {
    MATRIX_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}
#[test]
fn registration_thread_matrix_replaces_defaults_with_truthful_names() {
    use std::sync::atomic::Ordering;
    MATRIX_CALLS.store(0, Ordering::SeqCst);
    let mut suite = Suite::new("matrix");
    suite.registration_threads(&[1, 4, 1]).unwrap();
    register_replaceable_thread_matrix(&mut suite);
    assert_eq!(
        suite.list(""),
        [
            "matrix/replaceable_thread_matrix/threads=1",
            "matrix/replaceable_thread_matrix/threads=4"
        ]
    );
    assert_eq!(MATRIX_CALLS.load(Ordering::SeqCst), 0);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    assert_eq!(MATRIX_CALLS.load(Ordering::SeqCst), 15);
    assert_eq!(
        run.observations
            .iter()
            .map(|o| o.operations)
            .collect::<Vec<_>>(),
        [3, 12]
    );
    assert!(suite.registration_threads(&[]).is_err());
    assert!(suite.registration_threads(&[257]).is_ok());
}

#[airbug_bench::bench(args = [f64::INFINITY, -10.0, f64::NEG_INFINITY, 0.0, 10.0])]
fn special_float_arguments(value: f64) {
    std::hint::black_box(value);
}
#[test]
fn numeric_argument_sort_places_infinities_at_the_boundaries() {
    let mut suite = Suite::new("numeric");
    register_special_float_arguments(&mut suite);
    let expected = ["-inf", "-10.0", "0.0", "10.0", "inf"]
        .map(|label| format!("numeric/special_float_arguments/{label}"));
    for sort in [
        airbug_bench::SortOrder::Natural,
        airbug_bench::SortOrder::Kind,
    ] {
        let mut selection = airbug_bench::Selection {
            sort,
            ..Default::default()
        };
        assert_eq!(suite.list_selected(&selection), expected);
        selection.reverse = true;
        assert_eq!(
            suite.list_selected(&selection),
            expected
                .iter()
                .rev()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
    }
}

#[airbug_bench::group(bits = 8, samples = 1, iterations = 3, warmup_ms = 0)]
mod bit_counters {
    #[bench(custom = true, bits = 1_000_000_000)]
    fn gigabit(n: u64) -> std::time::Duration {
        std::time::Duration::from_secs(n)
    }
    #[bench(custom = true)]
    fn fixed(n: u64) -> std::time::Duration {
        std::time::Duration::from_secs(n)
    }
    #[bench(custom = true, bits = 0)]
    fn empty(n: u64) -> std::time::Duration {
        std::time::Duration::from_secs(n)
    }
    #[bench(args = [0usize, 2, 7], setup = |n| vec![0u8; n],
        input_bits = |v: &Vec<u8>| airbug_bench::counters::bits_from_bytes(v.len() as u64))]
    fn dynamic(data: &mut [u8]) {
        std::hint::black_box(data);
    }
}
#[test]
fn bit_counters_inherit_override_and_record_actual_input_work() {
    let mut suite = Suite::new("bits");
    bit_counters::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    let rates = airbug_bench::report::throughput(&run).unwrap();
    assert_eq!(
        rates
            .iter()
            .find(|r| r.case.ends_with("/fixed"))
            .unwrap()
            .values,
        [8.0]
    );
    assert_eq!(
        rates
            .iter()
            .find(|r| r.case.ends_with("/empty"))
            .unwrap()
            .values,
        [0.0]
    );
    assert!(rates.iter().all(|r| r.unit == "bits"));
    assert_eq!(
        rates
            .iter()
            .find(|r| r.case.ends_with("/gigabit"))
            .unwrap()
            .values,
        [1e9]
    );
    let displayed = airbug_bench::report::throughput_display(&run).unwrap();
    let gigabit = displayed
        .iter()
        .find(|r| r.case.ends_with("/gigabit"))
        .unwrap();
    assert_eq!(gigabit.unit, "Gbit");
    assert_eq!(gigabit.values, [1.0]);
    assert!(
        airbug_bench::report::markdown(&run)
            .unwrap()
            .contains("Gbit/s")
    );
    let process = airbug_bench::report::throughput_process_medians(&run).unwrap();
    let raw = process
        .iter()
        .find(|r| r.case.ends_with("/gigabit"))
        .unwrap();
    assert_eq!(raw.unit, "bits");
    assert_eq!(raw.median, 1e9);

    for observation in run
        .observations
        .iter()
        .filter(|o| o.case.contains("/dynamic/"))
    {
        let size = observation
            .case
            .rsplit('/')
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_eq!(observation.work_totals["bits"], (size * 8 * 3).to_string());
    }
    assert_eq!(airbug_bench::counters::bits_from_bytes(0), 0);
    assert_eq!(
        airbug_bench::counters::bits_from_bytes(u64::MAX / 8),
        u64::MAX - 7
    );
    assert!(
        std::panic::catch_unwind(|| airbug_bench::counters::bits_from_bytes(u64::MAX / 8 + 1))
            .is_err()
    );
}

#[airbug_bench::group]
mod source_first_group {
    #[bench]
    fn work() {}
}
#[airbug_bench::group]
mod source_last_group {
    #[bench]
    fn work() {}
}
#[airbug_bench::group(groups = [crate::source_last_group, crate::source_first_group])]
mod source_imports {}

#[test]
fn source_sort_imported_groups_uses_declaration_not_import_order() {
    let mut suite = Suite::new("root");
    source_imports::__airbug_register_group(&mut suite);
    let selection = airbug_bench::Selection {
        sort: airbug_bench::SortOrder::Source,
        ..Default::default()
    };
    assert_eq!(
        suite.list_selected(&selection),
        [
            "root/source_first_group/work",
            "root/source_last_group/work",
        ]
    );
}

#[airbug_bench::group]
mod upstream_type_examples {
    use std::collections::{BTreeSet, HashSet};

    #[bench(types = [&str, String])]
    fn from_str<'a, T>() -> T
    where
        T: From<&'a str>,
    {
        std::hint::black_box("hello world").into()
    }

    #[bench(types = [Vec<i32>, BTreeSet<i32>, HashSet<i32>], args = [0, 2, 16])]
    fn from_range<T>(n: i32) -> T
    where
        T: FromIterator<i32> + IntoIterator<Item = i32>,
    {
        let collection: T = (0..n).collect();
        let mut values: Vec<_> = collection.into_iter().collect();
        values.sort_unstable();
        assert_eq!(values, (0..n).collect::<Vec<_>>());
        values.into_iter().collect()
    }
}

#[test]
fn upstream_type_examples_cover_lifetime_outputs_and_type_argument_product() {
    let mut suite = Suite::new("types");
    upstream_type_examples::__airbug_register_group(&mut suite);
    let listed = suite.list("");
    assert_eq!(listed.len(), 11);
    assert_eq!(
        listed
            .iter()
            .filter(|name| name.contains("from_range"))
            .count(),
        9
    );
    let run = suite.test_selected(&Default::default()).unwrap();
    assert_eq!(run.cases.len(), 11);
    assert!(
        run.observations
            .iter()
            .all(|observation| observation.operations == 1)
    );
}

#[airbug_bench::group]
mod imported_drop_cases {
    use std::sync::atomic::{AtomicUsize, Ordering};
    pub static LIVE: AtomicUsize = AtomicUsize::new(0);
    pub static PEAK: AtomicUsize = AtomicUsize::new(0);
    struct Output;
    impl Drop for Output {
        fn drop(&mut self) {
            LIVE.fetch_sub(1, Ordering::SeqCst);
        }
    }
    #[bench]
    fn inherited() -> Output {
        let live = LIVE.fetch_add(1, Ordering::SeqCst) + 1;
        PEAK.fetch_max(live, Ordering::SeqCst);
        Output
    }
    #[bench(drop_output = "inside")]
    fn explicit_inside() -> String {
        String::from("output")
    }
    #[bench(setup = || vec![1u8])]
    async fn fresh(values: Vec<u8>) -> Vec<u8> {
        values
    }
}
#[airbug_bench::group(groups = [crate::imported_drop_cases], drop_output = "outside", iterations = 3)]
mod imported_drop_parent {}

#[test]
fn imported_drop_defaults_reach_plain_and_async_cases_and_restore_parent() {
    let mut suite = Suite::new("drop");
    imported_drop_parent::__airbug_register_group(&mut suite);
    assert!(matches!(
        suite.registration_drop_policy(),
        airbug_bench::DropPolicy::InsideTiming
    ));
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 3,
    });
    let selection = airbug_bench::Selection::default();
    // Fixed samples bypass calibration and retain three outputs until timing ends.
    let run = suite.run_selected(&selection).unwrap();
    assert_eq!(imported_drop_cases::LIVE.load(Ordering::SeqCst), 0);
    assert!(imported_drop_cases::PEAK.load(Ordering::SeqCst) > 1);
    for case in &run.cases {
        let json = serde_json::to_string(&case.contract).unwrap();
        if case.id.ends_with("explicit_inside") {
            assert!(json.contains("output drop included"), "{json}");
        } else if case.id.ends_with("fresh") {
            assert_eq!(
                case.contract.get("output.drop").map(String::as_str),
                Some("outside")
            );
        } else {
            assert!(json.contains("output drop excluded"), "{json}");
        }
    }
    suite.with_drop_defaults(airbug_bench::DropPolicy::OutsideTiming, |suite| {
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            suite.with_drop_defaults(airbug_bench::DropPolicy::InsideTiming, |suite| {
                assert!(matches!(
                    suite.registration_drop_policy(),
                    airbug_bench::DropPolicy::InsideTiming
                ));
                panic!("registration failed");
            });
        }));
        assert!(failed.is_err());
        assert!(matches!(
            suite.registration_drop_policy(),
            airbug_bench::DropPolicy::OutsideTiming
        ));
    });
    assert!(!suite.has_registration_drop_policy());
}

#[airbug_bench::bench(consts = [-2, -10, 0, 10, 2])]
fn signed_const_sort<const N: i32>() -> i32 {
    N
}

#[test]
fn natural_sort_orders_signed_const_values_numerically() {
    let mut suite = Suite::new("consts");
    register_signed_const_sort(&mut suite);
    for sort in [
        airbug_bench::SortOrder::Natural,
        airbug_bench::SortOrder::Kind,
    ] {
        let selection = airbug_bench::Selection {
            sort,
            ..Default::default()
        };
        assert_eq!(
            suite.list_selected(&selection),
            [-10, -2, 0, 2, 10].map(|n| format!("consts/signed_const_sort/const={n}"))
        );
    }
}

#[airbug_bench::group]
mod imported_batch_cases {
    use std::sync::atomic::{AtomicUsize, Ordering};
    pub static LIVE: AtomicUsize = AtomicUsize::new(0);
    pub static PEAK: AtomicUsize = AtomicUsize::new(0);
    pub struct Token;
    impl Drop for Token {
        fn drop(&mut self) {
            LIVE.fetch_sub(1, Ordering::SeqCst);
        }
    }
    fn token() -> Token {
        PEAK.fetch_max(LIVE.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
        Token
    }
    #[bench]
    fn plain() -> Token {
        token()
    }
    #[bench(setup = token)]
    fn owned(input: Token) -> Token {
        input
    }
    #[bench(args = [String::from("borrowed")])]
    async fn borrowed(value: &str) -> Token {
        assert_eq!(value, "borrowed");
        token()
    }
    #[bench(batch = airbug_bench::BatchPolicy::Iterations(std::num::NonZeroU64::new(3).unwrap()))]
    fn overridden() -> Token {
        token()
    }
    #[bench(threads = false)]
    fn sequential() -> Token {
        token()
    }
}
#[airbug_bench::group(groups = [crate::imported_batch_cases],
    batch = airbug_bench::BatchPolicy::PerIteration, drop_output = "outside",
    iterations = 5, samples = 1, warmup_ms = 0)]
mod imported_batch_parent {}

#[test]
fn imported_batch_defaults_bound_outputs_and_preserve_async_borrows_and_overrides() {
    let mut suite = Suite::new("batch");
    imported_batch_parent::__airbug_register_group(&mut suite);
    assert!(!suite.has_registration_batch_policy());
    assert_eq!(imported_batch_cases::PEAK.load(Ordering::SeqCst), 0);
    let ids: Vec<_> = suite.list("").into_iter().map(str::to_owned).collect();
    assert_eq!(ids.len(), 5);
    for id in ids {
        imported_batch_cases::PEAK.store(0, Ordering::SeqCst);
        let run = suite
            .run_selected(&airbug_bench::Selection {
                pattern: id.clone(),
                exact: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(imported_batch_cases::LIVE.load(Ordering::SeqCst), 0);
        assert_eq!(
            imported_batch_cases::PEAK.load(Ordering::SeqCst),
            if id.ends_with("overridden") { 3 } else { 1 }
        );
        assert!(run.observations.iter().all(|o| o.operations == 5));
    }
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        suite.with_batch_defaults(airbug_bench::BatchPolicy::PerIteration, |_| {
            panic!("registration")
        });
    }));
    assert!(failed.is_err());
    assert!(!suite.has_registration_batch_policy());
}

#[airbug_bench::group(confidence_level = 0.9, analysis_seed = 7)]
mod bootstrap_import {
    #[bench]
    fn inherited() {}
    #[bench(confidence_level = 0.99, analysis_seed = 0)]
    fn overridden() {}
}
#[airbug_bench::group(groups = [crate::bootstrap_import], resamples = 64, confidence_level = 0.8)]
mod bootstrap_parent {
    #[bench]
    fn local() {}
    #[group(resamples = 96)]
    mod nested {
        #[bench]
        fn child() {}
    }
}

#[test]
fn bootstrap_defaults_inherit_per_field_and_cli_overrides_only_explicit_fields() {
    use airbug_bench::bootstrap::{Config, Options};
    let mut suite = Suite::new("analysis");
    bootstrap_parent::__airbug_register_group(&mut suite);
    let fallback = Config::default();
    let settings = suite
        .bootstrap_settings(&Default::default(), &fallback, Options::default())
        .unwrap();
    assert_eq!(settings["analysis/local"].resamples, 64);
    assert_eq!(settings["analysis/local"].confidence_level, 0.8);
    assert_eq!(settings["analysis/nested/child"].resamples, 96);
    assert_eq!(settings["analysis/nested/child"].confidence_level, 0.8);
    assert_eq!(
        settings["analysis/bootstrap_import/inherited"].resamples,
        64
    );
    assert_eq!(
        settings["analysis/bootstrap_import/inherited"].confidence_level,
        0.9
    );
    assert_eq!(settings["analysis/bootstrap_import/inherited"].seed, 7);
    assert_eq!(
        settings["analysis/bootstrap_import/overridden"].confidence_level,
        0.99
    );
    assert_eq!(settings["analysis/bootstrap_import/overridden"].seed, 0);
    let cli = Options {
        resamples: Some(32),
        ..Default::default()
    };
    let overridden = suite
        .bootstrap_settings(&Default::default(), &fallback, cli)
        .unwrap();
    for (id, config) in &overridden {
        assert_eq!(config.resamples, 32);
        assert_eq!(config.confidence_level, settings[id].confidence_level);
        assert_eq!(config.seed, settings[id].seed);
    }
    assert!(
        suite
            .bootstrap_settings(
                &Default::default(),
                &fallback,
                Options {
                    confidence_level: Some(1.0),
                    ..Default::default()
                }
            )
            .is_err()
    );
}

#[airbug_bench::group(threads = [1, 2], iterations = 3, samples = 1, warmup_ms = 0)]
mod plain_worker_outputs {
    use std::{
        rc::Rc,
        sync::atomic::{AtomicUsize, Ordering},
        thread::ThreadId,
    };
    pub static DROPS: AtomicUsize = AtomicUsize::new(0);
    pub struct Output {
        owner: ThreadId,
        _local: Rc<()>,
    }
    impl Drop for Output {
        fn drop(&mut self) {
            assert_eq!(self.owner, std::thread::current().id());
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn output() -> Output {
        Output {
            owner: std::thread::current().id(),
            _local: Rc::new(()),
        }
    }
    #[bench]
    fn default_drop() -> Output {
        output()
    }
    #[bench(drop_output = "outside")]
    fn deferred_drop() -> Output {
        output()
    }
}

#[test]
fn plain_threaded_cases_keep_non_send_outputs_on_their_worker() {
    let mut suite = Suite::new("local-results");
    plain_worker_outputs::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 4);
    assert_eq!(plain_worker_outputs::DROPS.load(Ordering::SeqCst), 0);
    let run = suite.run("").unwrap();
    assert_eq!(plain_worker_outputs::DROPS.load(Ordering::SeqCst), 18);
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        18
    );
}

#[airbug_bench::group]
mod imported_worker_defaults {
    #[bench]
    fn local_output() -> std::rc::Rc<()> {
        std::rc::Rc::new(())
    }
    #[bench(threads = [3])]
    fn override_workers() {}
    #[bench(threads = false, args = [std::rc::Rc::new(())])]
    fn stay_local(value: &std::rc::Rc<()>) {
        assert_eq!(std::rc::Rc::strong_count(value), 1);
    }
}
#[airbug_bench::group(groups = [crate::imported_worker_defaults], threads = [1, 2], samples = 1, iterations = 1, warmup_ms = 0)]
mod importing_workers {}

#[airbug_bench::group(groups = [crate::imported_worker_defaults], threads = false)]
mod importing_sequential {}
#[airbug_bench::group(groups = [crate::importing_sequential], threads = [2])]
mod importing_nested_workers {}

#[test]
fn imported_worker_defaults_select_paths_and_restore_after_unwind() {
    let mut suite = Suite::new("imported-workers");
    importing_workers::__airbug_register_group(&mut suite);
    let names = suite.list("");
    assert_eq!(names.len(), 4, "{names:?}");
    assert!(names.iter().any(|n| n.ends_with("local_output/threads=1")));
    assert!(names.iter().any(|n| n.ends_with("local_output/threads=2")));
    assert!(
        names
            .iter()
            .any(|n| n.ends_with("override_workers/threads=3"))
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("stay_local") && !n.contains("threads="))
    );
    assert!(suite.inherited_thread_counts().is_none());
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        7
    );

    suite.with_thread_defaults(Some(vec![2]), |suite| {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            suite.with_thread_defaults(None, |suite| {
                assert!(suite.inherited_thread_counts().is_none());
                panic!("restore thread defaults");
            });
        }));
        assert!(result.is_err());
        assert_eq!(suite.inherited_thread_counts(), Some(vec![2]));
    });
    assert!(suite.inherited_thread_counts().is_none());

    let mut sequential = Suite::new("sequential-override");
    importing_nested_workers::__airbug_register_group(&mut sequential);
    let names = sequential.list("");
    assert_eq!(names.len(), 3);
    assert!(names.iter().any(|n| n.ends_with("local_output")));
    assert!(
        names
            .iter()
            .any(|n| n.ends_with("override_workers/threads=3"))
    );

    let mut overridden = Suite::new("runtime-workers");
    overridden.registration_threads(&[4]).unwrap();
    importing_workers::__airbug_register_group(&mut overridden);
    let names = overridden.list("");
    assert_eq!(names.len(), 3);
    assert_eq!(names.iter().filter(|n| n.ends_with("threads=4")).count(), 2);
}

#[airbug_bench::group(samples = 1, iterations = 1, warmup_ms = 0)]
mod imported_parameter_workers {
    use std::{
        rc::Rc,
        sync::atomic::{AtomicUsize, Ordering},
        thread::ThreadId,
    };
    pub static OUTPUTS: AtomicUsize = AtomicUsize::new(0);
    pub static EXECUTORS: AtomicUsize = AtomicUsize::new(0);
    pub struct LocalOutput(Rc<()>, ThreadId);
    impl Drop for LocalOutput {
        fn drop(&mut self) {
            assert_eq!(self.1, std::thread::current().id());
            assert_eq!(Rc::strong_count(&self.0), 1);
            OUTPUTS.fetch_add(1, Ordering::SeqCst);
        }
    }
    pub struct LocalExecutor(Rc<()>, ThreadId);
    impl airbug_bench::workloads::Executor for LocalExecutor {
        fn block_on<F: std::future::Future>(&mut self, future: F) -> F::Output {
            assert_eq!(self.1, std::thread::current().id());
            assert_eq!(Rc::strong_count(&self.0), 1);
            airbug_bench::workloads::LocalExecutor.block_on(future)
        }
    }
    fn executor() -> LocalExecutor {
        EXECUTORS.fetch_add(1, Ordering::SeqCst);
        LocalExecutor(Rc::new(()), std::thread::current().id())
    }
    #[bench(args = [2usize, 3], drop_output = "outside")]
    fn values(n: usize) -> LocalOutput {
        assert!([2, 3].contains(&n));
        LocalOutput(Rc::new(()), std::thread::current().id())
    }
    #[bench(args = [String::from("hello")])]
    fn borrowed(value: &str) -> usize {
        value.len()
    }
    #[bench(executor = executor(), args = [String::from("async")])]
    async fn async_borrowed(value: &str) -> usize {
        std::future::ready(value.len()).await
    }
    #[bench(executor = executor(), args = [1usize, 2])]
    async fn async_values(value: usize) -> usize {
        std::future::ready(value).await
    }
    #[bench(executor = executor(), drop_output = "outside")]
    async fn asynchronous() -> LocalOutput {
        let result = LocalOutput(Rc::new(()), std::thread::current().id());
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
        result
    }
}
#[airbug_bench::group(groups = [crate::imported_parameter_workers], threads = [1, 2])]
mod parameter_worker_parent {}
#[airbug_bench::group(samples = 1, iterations = 1, warmup_ms = 0)]
mod imported_unshareable_arguments {
    #[bench(args = [std::rc::Rc::new(5)])]
    fn local(value: &std::rc::Rc<i32>) -> i32 {
        **value
    }
    #[bench(args = [std::rc::Rc::new(7)])]
    async fn async_local(value: &std::rc::Rc<i32>) -> i32 {
        std::future::ready(**value).await
    }
}
#[airbug_bench::group]
mod imported_async_unshareable_arguments {
    #[bench(args = [std::rc::Rc::new(7)])]
    async fn local(value: &std::rc::Rc<i32>) -> i32 {
        std::future::ready(**value).await
    }
}
#[test]
fn imported_threads_cover_values_borrows_async_and_reject_unshareable_inputs_lazily() {
    let mut suite = Suite::new("parameter-workers");
    parameter_worker_parent::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 14);
    assert_eq!(
        imported_parameter_workers::EXECUTORS.load(Ordering::SeqCst),
        0
    );
    assert_eq!(
        imported_parameter_workers::OUTPUTS.load(Ordering::SeqCst),
        0
    );
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        21
    );
    assert_eq!(
        imported_parameter_workers::EXECUTORS.load(Ordering::SeqCst),
        12
    );
    assert_eq!(
        imported_parameter_workers::OUTPUTS.load(Ordering::SeqCst),
        9
    );

    let mut local = Suite::new("local");
    imported_unshareable_arguments::__airbug_register_group(&mut local);
    assert_eq!(local.run("").unwrap().observations.len(), 2);
    let mut rejected = Suite::new("rejected");
    let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rejected.with_thread_defaults(
            Some(vec![2]),
            imported_unshareable_arguments::__airbug_register_group,
        );
    }))
    .unwrap_err();
    let message = error.downcast_ref::<String>().unwrap();
    assert!(
        message.contains("cannot share its arguments between workers"),
        "{message}"
    );
    assert!(rejected.inherited_thread_counts().is_none());
    let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rejected.with_thread_defaults(
            Some(vec![2]),
            imported_async_unshareable_arguments::__airbug_register_group,
        );
    }))
    .unwrap_err();
    let message = error.downcast_ref::<String>().unwrap();
    assert!(
        message.contains("cannot share its arguments or executor factory"),
        "{message}"
    );
    assert!(rejected.inherited_thread_counts().is_none());
}

#[airbug_bench::group(samples = 1, iterations = 3, warmup_ms = 0)]
mod imported_setup_workers {
    use std::{
        rc::Rc,
        sync::{
            OnceLock,
            atomic::{AtomicUsize, Ordering},
        },
        thread::ThreadId,
    };
    pub static COORDINATOR: OnceLock<ThreadId> = OnceLock::new();
    pub static SYNC_SETUPS: AtomicUsize = AtomicUsize::new(0);
    pub static ASYNC_SETUPS: AtomicUsize = AtomicUsize::new(0);
    pub static ASYNC_DROPS: AtomicUsize = AtomicUsize::new(0);
    pub static SYNC_CALLER: AtomicUsize = AtomicUsize::new(0);
    pub static ASYNC_CALLER: AtomicUsize = AtomicUsize::new(0);
    pub struct Input {
        values: Vec<usize>,
        origin: ThreadId,
    }
    fn input(n: usize) -> Input {
        assert_eq!(COORDINATOR.get(), Some(&std::thread::current().id()));
        SYNC_SETUPS.fetch_add(1, Ordering::SeqCst);
        Input {
            values: vec![n; n],
            origin: std::thread::current().id(),
        }
    }
    pub struct LocalInput {
        value: Rc<usize>,
        origin: ThreadId,
    }
    impl Drop for LocalInput {
        fn drop(&mut self) {
            assert_eq!(self.origin, std::thread::current().id());
            ASYNC_DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn local_input(n: usize) -> LocalInput {
        if COORDINATOR.get() == Some(&std::thread::current().id()) {
            ASYNC_CALLER.fetch_add(1, Ordering::SeqCst);
        }
        ASYNC_SETUPS.fetch_add(1, Ordering::SeqCst);
        LocalInput {
            value: Rc::new(n),
            origin: std::thread::current().id(),
        }
    }
    #[bench(args = [2usize, 5], setup = input, input_items = |i: &Input| i.values.len() as u64)]
    fn borrowed(input: &mut Input) -> usize {
        if input.origin == std::thread::current().id() {
            SYNC_CALLER.fetch_add(1, Ordering::SeqCst);
        }
        input.values.len()
    }
    #[bench(args = [2usize, 5], setup = input, drop_output = "outside")]
    fn owned(input: Input) -> Vec<usize> {
        if input.origin == std::thread::current().id() {
            SYNC_CALLER.fetch_add(1, Ordering::SeqCst);
        }
        input.values
    }
    #[bench(args = [2usize, 5], setup = local_input, input_items = |i: &LocalInput| *i.value as u64, drop_output = "outside")]
    async fn async_borrowed(input: &mut LocalInput) -> Rc<usize> {
        assert_eq!(input.origin, std::thread::current().id());
        std::future::ready(input.value.clone()).await
    }
    #[bench(args = [2usize, 5], setup = local_input, drop_output = "outside")]
    async fn async_owned(input: LocalInput) -> Rc<usize> {
        assert_eq!(input.origin, std::thread::current().id());
        std::future::ready(input.value.clone()).await
    }
}
#[airbug_bench::group(groups = [crate::imported_setup_workers], threads = [1, 2])]
mod imported_setup_parent {}
#[airbug_bench::group(samples = 1, iterations = 1, warmup_ms = 0)]
mod imported_setup_non_send {
    #[bench(setup = || std::rc::Rc::new(1))]
    fn local(input: &mut std::rc::Rc<i32>) -> std::rc::Rc<i32> {
        input.clone()
    }
}
#[test]
fn inherited_setup_threads_preserve_preparation_location_and_non_send_local_cases() {
    use imported_setup_workers as cases;
    cases::COORDINATOR.set(std::thread::current().id()).unwrap();
    let mut suite = Suite::new("setup-workers");
    imported_setup_parent::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 16);
    assert_eq!(cases::SYNC_SETUPS.load(Ordering::SeqCst), 0);
    assert_eq!(cases::ASYNC_SETUPS.load(Ordering::SeqCst), 0);
    let run = suite.run("").unwrap();
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        72
    );
    assert_eq!(cases::SYNC_SETUPS.load(Ordering::SeqCst), 36);
    assert_eq!(cases::ASYNC_SETUPS.load(Ordering::SeqCst), 36);
    assert_eq!(cases::ASYNC_DROPS.load(Ordering::SeqCst), 36);
    assert_eq!(cases::SYNC_CALLER.load(Ordering::SeqCst), 12);
    assert_eq!(cases::ASYNC_CALLER.load(Ordering::SeqCst), 12);
    for case in &run.cases {
        assert_eq!(
            case.contract["threads.execution"],
            if case.contract["threads"] == "1" {
                "caller"
            } else {
                "spawned"
            }
        );
    }
    assert_eq!(
        run.observations
            .iter()
            .filter_map(|o| o.work_totals.get("items"))
            .map(|n| n.parse::<u64>().unwrap())
            .sum::<u64>(),
        126
    );

    let mut local = Suite::new("non-send-setup");
    imported_setup_non_send::__airbug_register_group(&mut local);
    assert_eq!(local.run("").unwrap().observations.len(), 1);
    let mut rejected = Suite::new("rejected-setup");
    let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rejected.with_thread_defaults(
            Some(vec![2]),
            imported_setup_non_send::__airbug_register_group,
        );
    }))
    .unwrap_err();
    assert!(
        error
            .downcast_ref::<String>()
            .unwrap()
            .contains("cannot transfer setup inputs or outputs")
    );
    assert!(rejected.inherited_thread_counts().is_none());
}

#[airbug_bench::bench(setup = || 1usize, input_items = {
    let count = std::rc::Rc::new(std::cell::Cell::new(0u64));
    move |_: &usize| { count.set(count.get() + 1); count.get() }
}, samples = 1, iterations = 3, warmup_ms = 0)]
async fn local_async_counter(input: &mut usize) -> usize {
    *input
}
#[test]
fn inherited_setup_support_keeps_local_async_counter_state_legal() {
    let mut suite = Suite::new("local-counter");
    register_local_async_counter(&mut suite);
    let run = suite.run("").unwrap();
    assert_eq!(run.observations[0].work_totals["items"], "6");
}

#[airbug_bench::group(samples = 1, iterations = 2, warmup_ms = 0)]
mod runtime_thread_opt_in {
    use std::sync::atomic::{AtomicUsize, Ordering};
    pub static CALLS: AtomicUsize = AtomicUsize::new(0);
    #[bench]
    fn regular() {
        CALLS.fetch_add(1, Ordering::SeqCst);
    }
    #[bench(setup = || std::rc::Rc::new(1))]
    fn unshareable(input: &mut std::rc::Rc<i32>) -> i32 {
        **input
    }
    #[bench(custom = true)]
    fn custom(n: u64) -> std::time::Duration {
        std::time::Duration::from_nanos(n)
    }
    #[bench(threads = false, args = [std::rc::Rc::new(1)])]
    fn sequential(input: &std::rc::Rc<i32>) -> i32 {
        **input
    }
}
#[test]
fn runtime_threads_opt_in_validate_only_selected_cases_and_respect_false() {
    let mut suite = Suite::new("runtime");
    suite.registration_threads(&[1, 3]).unwrap();
    runtime_thread_opt_in::__airbug_register_group(&mut suite);
    let names = suite.list("");
    assert_eq!(names.len(), 6, "{names:?}");
    assert_eq!(
        names
            .iter()
            .filter(|n| n.contains("regular/threads="))
            .count(),
        2
    );
    assert!(
        names
            .iter()
            .any(|n| n.contains("sequential") && !n.contains("threads="))
    );
    assert!(suite.run("").is_err());
    assert_eq!(runtime_thread_opt_in::CALLS.load(Ordering::SeqCst), 0);
    let run = suite.run("regular").unwrap();
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        8
    );
    assert_eq!(runtime_thread_opt_in::CALLS.load(Ordering::SeqCst), 8);
    assert!(
        suite
            .run("unshareable")
            .unwrap_err()
            .to_string()
            .contains("cannot transfer setup inputs")
    );
    assert!(
        suite
            .run("custom")
            .unwrap_err()
            .to_string()
            .contains("cannot enable threads at runtime")
    );
    assert!(suite.run("sequential").is_ok());

    let mut disabled = Suite::new("disabled");
    disabled.registration_threads(&[1, 3]).unwrap();
    disabled.with_thread_defaults(None, runtime_thread_opt_in::__airbug_register_group);
    assert_eq!(disabled.list("").len(), 4);
    assert!(disabled.list("").iter().all(|n| !n.contains("threads=")));
    assert!(disabled.run("unshareable").is_ok());
    assert_eq!(disabled.inherited_thread_counts(), Some(vec![1, 3]));
}

static CONDITIONAL_IGNORE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static CONDITIONAL_CALLS: AtomicUsize = AtomicUsize::new(0);

#[airbug_bench::group(ignore = crate::CONDITIONAL_IGNORE.load(Ordering::SeqCst))]
mod conditional_ignore_group {
    use super::*;
    #[bench]
    fn inherited() {
        CONDITIONAL_CALLS.fetch_add(1, Ordering::SeqCst);
    }
    #[bench(ignore = 1 > 2)]
    fn enabled() {}
}

#[test]
fn computed_ignore_defaults_are_evaluated_at_registration_and_can_be_overridden() {
    use airbug_bench::Selection;
    CONDITIONAL_IGNORE.store(true, Ordering::SeqCst);
    let mut skipped = Suite::new("conditional");
    conditional_ignore_group::__airbug_register_group(&mut skipped);
    assert_eq!(skipped.list(""), ["conditional/enabled"]);
    assert_eq!(
        skipped.list_selected(&Selection {
            only_ignored: true,
            ..Default::default()
        }),
        ["conditional/inherited"]
    );
    skipped.config(fast_config());
    skipped.run("").unwrap();
    assert_eq!(CONDITIONAL_CALLS.load(Ordering::SeqCst), 0);
    CONDITIONAL_IGNORE.store(false, Ordering::SeqCst);
    let mut enabled = Suite::new("conditional");
    conditional_ignore_group::__airbug_register_group(&mut enabled);
    assert_eq!(enabled.list("").len(), 2);
    enabled.config(fast_config());
    enabled.run("").unwrap();
    assert!(CONDITIONAL_CALLS.load(Ordering::SeqCst) > 0);
    // A registered suite retains its selection even if the condition changes later.
    assert_eq!(skipped.list(""), ["conditional/enabled"]);
}

#[test]
fn one_inherited_worker_accepts_local_arguments_and_rejects_later_parallel_override() {
    for runtime in [false, true] {
        let mut suite = Suite::new("one-local");
        if runtime {
            suite.registration_threads(&[1]).unwrap();
            imported_unshareable_arguments::__airbug_register_group(&mut suite);
        } else {
            suite.with_thread_defaults(
                Some(vec![1]),
                imported_unshareable_arguments::__airbug_register_group,
            );
        }
        assert_eq!(suite.list("").len(), 2);
        let run = suite.run("").unwrap();
        assert_eq!(run.observations.len(), 2);
        for case in &run.cases {
            assert!(case.id.ends_with("threads=1"));
            assert_eq!(case.contract["threads.execution"], "caller");
            assert_eq!(case.contract["threads.local_only"], "true");
        }
        assert!(run.observations.iter().all(|o| o.operations == 1));
        assert!(
            suite
                .thread_count(2)
                .err()
                .unwrap()
                .to_string()
                .contains("requires one worker")
        );
        suite.thread_count(1).unwrap();
        assert_eq!(suite.run("").unwrap().observations.len(), 2);
    }
}

#[airbug_bench::group(samples = 1, iterations = 2, warmup_ms = 0)]
mod local_setup_one {
    #[bench(setup = || std::rc::Rc::new(5), drop_output = "outside")]
    fn borrowed(input: &mut std::rc::Rc<i32>) -> std::rc::Rc<i32> {
        input.clone()
    }
    #[bench(setup = || std::rc::Rc::new(7), drop_output = "outside")]
    fn owned(input: std::rc::Rc<i32>) -> std::rc::Rc<i32> {
        input
    }
}

#[test]
fn one_worker_setup_accepts_local_inputs_outputs_and_async_captures() {
    use airbug_bench::{DropPolicy, Sampling, threads::*, workloads::LocalExecutor};
    use std::{marker::PhantomData, rc::Rc};
    let mut suite = Suite::new("local-setup");
    suite.with_thread_defaults(Some(vec![1]), local_setup_one::__airbug_register_group);
    let borrowed = inherited_async_setup(
        PhantomData::<Rc<()>>,
        || LocalExecutor,
        || Rc::new(11),
        async |input: &mut Rc<i32>| input.clone(),
    );
    (&borrowed).register_inherited_async_setup(
        &mut suite,
        "async-borrowed",
        1,
        DropPolicy::OutsideTiming,
    );
    suite.sampling(Sampling {
        iterations: Some(2),
        ..Default::default()
    });
    let owned = inherited_async_owned_setup(
        PhantomData::<Rc<()>>,
        || LocalExecutor,
        || Rc::new(13),
        async |input: Rc<i32>| input,
    );
    (&owned).register_inherited_async_owned_setup(
        &mut suite,
        "async-owned",
        1,
        DropPolicy::OutsideTiming,
    );
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        ..Default::default()
    });
    suite.sampling(Sampling {
        iterations: Some(2),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 4);
    assert!(
        run.cases
            .iter()
            .all(|c| c.contract["threads.local_only"] == "true")
    );
    assert_eq!(run.worker_timings.len(), 4);
    assert!(
        run.worker_timings
            .iter()
            .all(|w| w.worker == 0 && w.operations == 2)
    );
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        8
    );
    assert!(suite.thread_count(2).is_err());
    run.validate().unwrap();
}

#[airbug_bench::group(samples = 1, iterations = 2, warmup_ms = 0)]
mod explicit_local_worker {
    use std::rc::Rc;
    #[bench(threads = 1, args = [Rc::new(1)])]
    fn argument(v: &Rc<i32>) -> Rc<i32> {
        v.clone()
    }
    #[bench(threads = [1], args = [Rc::new(2)])]
    async fn async_argument(v: &Rc<i32>) -> Rc<i32> {
        v.clone()
    }
    #[bench(threads = 1, setup = || Rc::new(3))]
    fn borrowed(v: &mut Rc<i32>) -> Rc<i32> {
        v.clone()
    }
    #[bench(threads = 1, setup = || Rc::new(4))]
    fn owned(v: Rc<i32>) -> Rc<i32> {
        v
    }
}
#[test]
fn explicit_single_worker_attributes_accept_local_inputs_and_cli_override_rejects_sharing() {
    let mut suite = Suite::new("explicit-local");
    explicit_local_worker::__airbug_register_group(&mut suite);
    let run = suite.run("").unwrap();
    assert_eq!(run.cases.len(), 4);
    assert!(
        run.cases
            .iter()
            .all(|c| c.contract["threads.local_only"] == "true")
    );
    assert_eq!(run.worker_timings.len(), 4);
    assert!(
        run.worker_timings
            .iter()
            .all(|w| w.worker == 0 && w.operations == 2)
    );
    assert_eq!(
        run.observations.iter().map(|o| o.operations).sum::<u64>(),
        8
    );
    let mut parallel = Suite::new("reject-sharing");
    parallel.registration_threads(&[2]).unwrap();
    explicit_local_worker::__airbug_register_group(&mut parallel);
    assert!(
        parallel
            .run("")
            .unwrap_err()
            .to_string()
            .contains("cannot share")
    );
}

#[airbug_bench::bench(threads = 2, setup = || 1usize, input_items = {
    let counter = std::sync::atomic::AtomicU64::new(0);
    move |_: &usize| counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}, samples = 1, iterations = 3, warmup_ms = 0)]
async fn constructed_shared_async_counter(input: &mut usize) -> usize {
    *input
}

#[airbug_bench::bench(setup = || 1usize, input_items = {
    let mut count = 0u64;
    move |_: &usize| { count += 1; count }
}, samples = 1, iterations = 3, warmup_ms = 0)]
async fn constructed_mutable_async_counter(input: &mut usize) -> usize {
    *input
}

#[test]
fn constructed_async_counters_support_local_inheritance_and_explicit_workers() {
    let mut shared = Suite::new("constructed-shared");
    register_constructed_shared_async_counter(&mut shared);
    let run = shared.run("").unwrap();
    assert_eq!(run.observations[0].operations, 6);
    assert_eq!(run.observations[0].work_totals["items"], "21");
    assert_eq!(run.observations[0].worker_work_totals.len(), 2);
    assert_eq!(run.cases[0].contract["threads.execution"], "spawned");
    for register in [
        register_constructed_mutable_async_counter,
        register_local_async_counter,
    ] {
        let mut local = Suite::new("constructed-local");
        local.with_thread_defaults(Some(vec![1]), register);
        let run = local.run("").unwrap();
        assert_eq!(run.observations[0].operations, 3);
        assert_eq!(run.observations[0].work_totals["items"], "6");
        assert_eq!(run.observations[0].worker_work_totals[&0]["items"], "6");
        assert_eq!(run.cases[0].contract["threads.execution"], "caller");
    }
}

#[airbug_bench::group(samples = 0, iterations = 1, warmup_ms = 0)]
mod zero_sample_group {
    #[bench]
    fn disabled() {
        panic!("zero sample budget executed");
    }
    #[bench(samples = 1)]
    fn enabled() {}
    #[bench(samples = 1, iterations = 0)]
    fn zero_iterations() {
        panic!("zero iteration budget executed");
    }
}

#[test]
fn zero_sampling_disables_cases_and_child_values_can_enable_them() {
    let mut suite = Suite::new("zero-budget");
    zero_sample_group::__airbug_register_group(&mut suite);
    let selection = airbug_bench::Selection {
        include_ignored: true,
        ..Default::default()
    };
    assert_eq!(suite.list_selected(&selection), ["zero-budget/enabled"]);
    let run = suite.run_selected(&selection).unwrap();
    assert_eq!(run.cases.len(), 1);
    assert_eq!(run.observations.len(), 1);
    let mut disabled = Suite::new("zero-global");
    disabled.config(Config {
        samples: 0,
        ..Default::default()
    });
    disabled.bench("never", || panic!("disabled global budget executed"));
    assert!(disabled.run("").unwrap().observations.is_empty());
    disabled.sampling(airbug_bench::Sampling {
        samples: Some(1),
        iterations: Some(1),
        warmup: Some(std::time::Duration::ZERO),
        ..Default::default()
    });
    assert_eq!(disabled.list(""), ["zero-global/never"]);
}
