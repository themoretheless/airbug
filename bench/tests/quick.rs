use airbug_bench::{Config, QuickConfig, Suite};
use std::{cell::RefCell, time::Duration};

#[test]
fn quick_grows_real_batches_without_warmup_and_stops_on_stability() {
    let calls = RefCell::new(Vec::new());
    let mut suite = Suite::new("quick");
    suite.bench_custom("linear", |n| {
        calls.borrow_mut().push(n);
        Duration::from_nanos(n * 100)
    });
    suite.quick(QuickConfig {
        min_time: Duration::ZERO,
        ..Default::default()
    });
    assert_eq!(suite.list("").len(), 1);
    assert!(calls.borrow().is_empty());
    let run = suite.run("").unwrap();
    assert_eq!(*calls.borrow(), [1, 2]);
    assert_eq!(
        run.observations
            .iter()
            .map(|o| o.operations)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(run.cases[0].contract["quick.stop"], "stable");
    assert_eq!(run.cases[0].contract["samples"], "2");
    assert_eq!(run.cases[0].contract["warmup_ns"], "0");
    assert_eq!(run.cases[0].contract["sampling.mode"], "quick_adaptive");
}
#[test]
fn quick_respects_iteration_cap_and_retains_single_slow_sample() {
    let calls = RefCell::new(Vec::new());
    let mut suite = Suite::new("cap");
    suite.config(Config {
        max_iterations: 1,
        ..Default::default()
    });
    suite.bench_custom("constant", |n| {
        calls.borrow_mut().push(n);
        Duration::from_nanos(n * 100)
    });
    suite.quick(QuickConfig {
        min_time: Duration::ZERO,
        ..Default::default()
    });
    suite.run("").unwrap();
    assert_eq!(*calls.borrow(), [1, 1]);
    let mut slow = Suite::new("slow");
    slow.bench_custom("one", |_| {
        std::thread::sleep(Duration::from_millis(2));
        Duration::ZERO
    });
    slow.sampling(airbug_bench::Sampling {
        min_time: Some(Duration::from_secs(1)),
        exclude_external_time: Some(true),
        ..Default::default()
    });
    slow.quick(QuickConfig {
        min_time: Duration::ZERO,
        max_time: Duration::from_millis(1),
        ..Default::default()
    });
    let run = slow.run("").unwrap();
    assert_eq!(run.observations.len(), 1);
    assert_eq!(run.observations[0].value.as_deref(), Some("0"));
    assert_eq!(run.cases[0].contract["quick.stop"], "max_time");
}
#[test]
fn quick_rejects_fixed_iterations_before_work() {
    let mut suite = Suite::new("conflict");
    suite.bench("never", || panic!("must not execute"));
    suite.sampling(airbug_bench::Sampling {
        iterations: Some(10),
        ..Default::default()
    });
    suite.quick(Default::default());
    assert!(
        suite
            .run("")
            .unwrap_err()
            .to_string()
            .contains("fixed iterations")
    );
}

#[cfg(feature = "macros")]
#[airbug_bench::group]
mod imported {
    #[bench(custom = true)]
    fn inherited(n: u64) -> std::time::Duration {
        std::time::Duration::from_nanos(n * 100)
    }
    #[bench(
        custom = true,
        quick = false,
        iterations = 3,
        samples = 1,
        warmup_ms = 0
    )]
    fn disabled(n: u64) -> std::time::Duration {
        std::time::Duration::from_nanos(n * 100)
    }
}
#[cfg(feature = "macros")]
#[airbug_bench::group(groups = [crate::imported], quick = true,
    quick_config = airbug_bench::QuickConfig { min_time: std::time::Duration::ZERO, ..Default::default() })]
mod adaptive {
    #[bench(custom = true)]
    fn inherited(n: u64) -> std::time::Duration {
        std::time::Duration::from_nanos(n * 100)
    }
    #[group(quick = false, samples = 1, iterations = 4, warmup_ms = 0)]
    mod disabled {
        #[bench(custom = true)]
        fn fixed(n: u64) -> std::time::Duration {
            std::time::Duration::from_nanos(n * 100)
        }
    }
}
#[cfg(feature = "macros")]
#[test]
fn quick_attributes_inherit_into_inline_and_imported_groups_and_allow_disable() {
    let mut suite = Suite::new("attributes");
    adaptive::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 4);
    let run = suite.run("").unwrap();
    for case in &run.cases {
        let operations: Vec<_> = run
            .observations
            .iter()
            .filter(|o| o.case == case.id)
            .map(|o| o.operations)
            .collect();
        if case.id.ends_with("/inherited") {
            assert_eq!(operations, [1, 2]);
            assert_eq!(case.contract["quick.stop"], "stable");
        } else {
            assert_eq!(
                operations,
                if case.id.ends_with("/fixed") {
                    vec![4]
                } else {
                    vec![3]
                }
            );
            assert!(!case.contract.contains_key("quick.stop"));
        }
    }
}
#[test]
fn quick_conflicts_in_later_cases_are_rejected_before_any_work() {
    let mut suite = Suite::new("preflight");
    suite.bench("first", || panic!("must not run"));
    suite.bench("invalid", || panic!("must not run"));
    suite.sampling(airbug_bench::Sampling {
        iterations: Some(2),
        ..Default::default()
    });
    suite.quick_case(Some(Default::default()));
    assert!(suite.run("").is_err());
}

#[test]
fn quick_time_budgets_gate_stability_and_maximum_takes_precedence() {
    for (min_ns, max_ns, expected, reason) in [
        (500, None, vec![1, 2, 4], "stable"),
        (0, Some(250), vec![1, 2], "time_budget"),
        (500, Some(250), vec![1, 2], "time_budget"),
    ] {
        let mut suite = Suite::new("budgets");
        suite.bench_custom("linear", |n| Duration::from_nanos(n * 100));
        suite.sampling(airbug_bench::Sampling {
            min_time: Some(Duration::from_nanos(min_ns)),
            max_time: max_ns.map(Duration::from_nanos),
            exclude_external_time: Some(true),
            ..Default::default()
        });
        suite.quick(QuickConfig {
            min_time: Duration::ZERO,
            ..Default::default()
        });
        let run = suite.run("").unwrap();
        assert_eq!(
            run.observations
                .iter()
                .map(|o| o.operations)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(run.cases[0].contract["quick.stop"], reason);
    }
}
#[test]
fn disabling_quick_after_execution_removes_stale_contract_and_resets_sampling() {
    let mut suite = Suite::new("repeat");
    suite.bench_custom("linear", |n| Duration::from_nanos(n * 100));
    suite.quick_case(Some(QuickConfig {
        min_time: Duration::ZERO,
        ..Default::default()
    }));
    for _ in 0..2 {
        let run = suite.run("").unwrap();
        assert_eq!(
            run.observations
                .iter()
                .map(|o| o.operations)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }
    suite.quick_case(None);
    suite.sampling(airbug_bench::Sampling {
        samples: Some(1),
        iterations: Some(3),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let normal = suite.run("").unwrap();
    assert_eq!(normal.observations.len(), 1);
    assert_eq!(normal.observations[0].operations, 3);
    assert!(
        normal.cases[0]
            .contract
            .keys()
            .all(|key| !key.starts_with("quick."))
    );
    assert_eq!(normal.cases[0].contract["sampling.mode"], "flat");
}
