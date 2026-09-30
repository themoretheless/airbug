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
