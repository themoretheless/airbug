use airbug_bench::{Config, Sampling, Selection, Suite};
use std::{cell::Cell, time::Duration};

#[test]
fn duplicate_case_ids_fail_before_any_selected_work_runs() {
    for smoke in [false, true] {
        let calls = Cell::new(0);
        let mut suite = Suite::new("names");
        for name in ["first", "duplicate", "duplicate"] {
            suite.bench(name, || calls.set(calls.get() + 1));
        }
        let error = suite.validate_registration().unwrap_err().to_string();
        assert!(error.contains("names/duplicate"), "{error}");
        assert_eq!(calls.get(), 0);
        let result = if smoke {
            suite.test_selected(&Selection::default())
        } else {
            suite.run("")
        };
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate benchmark ID")
        );
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn group_registration_unwind_restores_parent_and_retains_existing_cases() {
    let mut suite = Suite::new("parent");
    suite.bench("before", || ());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        suite.group("child", |suite| {
            suite.bench("registered", || ());
            panic!("registration failure");
        });
    }));
    assert!(result.is_err());
    suite.bench("after", || ());
    assert_eq!(
        suite.list(""),
        ["parent/before", "parent/child/registered", "parent/after"]
    );
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
    assert_eq!(suite.run("").unwrap().cases.len(), 3);
}
