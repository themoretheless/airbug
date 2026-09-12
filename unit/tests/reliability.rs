use airbug::{
    FixtureContext, Generate, GenerationError, GenerationErrorKind, Mock, Validator, assert_that,
};
use std::sync::Arc;

#[test]
fn missing_factory_is_a_typed_error() {
    let error = FixtureContext::new()
        .try_build_registered::<u64>()
        .unwrap_err();
    assert_eq!(error.kind, GenerationErrorKind::MissingFactory);
    assert_eq!(error.path, ["u64"]);
}
#[test]
fn custom_factory_failure_has_graph_path() {
    let mut ctx = FixtureContext::new();
    ctx.register_fallible::<u64>(|_| Err(GenerationError::custom("database unavailable")));
    let error = ctx.try_build::<Vec<u64>>().unwrap_err();
    assert_eq!(error.path.len(), 2);
    assert_eq!(
        error.kind,
        GenerationErrorKind::Custom("database unavailable".into())
    );
    assert!(ctx.remove::<u64>());
    assert!(!ctx.remove::<u64>());
    assert_eq!(ctx.build::<Vec<u64>>().len(), 3);
}
#[test]
fn depth_limit_is_distinct_and_zero_collection_is_valid() {
    let mut ctx = FixtureContext::new();
    ctx.max_depth(1);
    assert_eq!(
        ctx.try_build::<Vec<u64>>().unwrap_err().kind,
        GenerationErrorKind::DepthLimit
    );
    ctx.collection_len(0);
    assert!(ctx.build::<Vec<u64>>().is_empty());
}
struct Node {
    next: Option<Box<Node>>,
}
impl Generate for Node {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(Self {
            next: ctx.try_build()?,
        })
    }
}
#[test]
fn explicit_optional_rule_terminates_recursion() {
    let mut ctx = FixtureContext::new();
    assert_eq!(
        ctx.try_build::<Node>().err().unwrap().kind,
        GenerationErrorKind::Cycle
    );
    ctx.register::<Option<Box<Node>>>(|_| None);
    assert!(ctx.build::<Node>().next.is_none());
}
#[test]
fn shared_identity_is_explicit() {
    let mut ctx = FixtureContext::new();
    let shared = Arc::new("same".to_string());
    ctx.reuse(shared.clone());
    assert!(Arc::ptr_eq(&shared, &ctx.build::<Arc<String>>()));
}
#[test]
fn generated_floats_are_finite_unit_interval() {
    let mut ctx = FixtureContext::new();
    for _ in 0..1000 {
        assert!((0.0..1.0).contains(&ctx.build::<f64>()));
        assert!((0.0..1.0).contains(&ctx.build::<f32>()));
    }
}
#[test]
fn mock_checks_arguments_and_exact_count() {
    let mock = Mock::new("lookup");
    mock.expect("known id", |id: &u64| *id == 7)
        .times(2)
        .returns("found");
    assert!(mock.verify().is_err());
    assert_eq!(mock.call(7), "found");
    assert_eq!(mock.call(7), "found");
    mock.assert_verified();
    assert!(mock.try_call(7).is_err());
    assert!(
        mock.verify()
            .unwrap_err()
            .to_string()
            .contains("exceeded 2")
    );
}
#[test]
fn mock_remembers_unexpected_calls() {
    let mock = Mock::<u64, ()>::new("send");
    assert!(mock.try_call(42).is_err());
    assert!(mock.verify().unwrap_err().to_string().contains("42"));
}
#[test]
fn mock_supports_never_and_disjoint_matchers() {
    let mock = Mock::new("get");
    mock.expect("forbidden", |v: &i32| *v < 0)
        .times(0)
        .returns(0);
    mock.expect("positive", |v: &i32| *v > 0)
        .returning(|v| v * 2);
    assert_eq!(mock.call(3), 6);
    mock.assert_verified();
    assert!(mock.try_call(-1).is_err());
}
#[test]
fn mock_counts_concurrent_calls() {
    let mock = Mock::new("concurrent");
    mock.expect("all", |_: &usize| true)
        .times(800)
        .returning(|v| v + 1);
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let mock = mock.clone();
            std::thread::spawn(move || {
                for i in 0..100 {
                    assert_eq!(mock.call(i), i + 1);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    mock.assert_verified();
}
#[test]
fn answer_can_reenter_mock_without_deadlock() {
    let mock = Mock::new("recursive");
    let child = mock.clone();
    mock.expect("outer", |v: &u8| *v == 1)
        .returning(move |_| child.call(2));
    mock.expect("inner", |v: &u8| *v == 2).returns(5);
    assert_eq!(mock.call(1), 5);
    mock.assert_verified();
}
#[test]
fn answer_panic_does_not_poison_mock() {
    let mock = Mock::<(), ()>::new("panic");
    mock.expect("panics", |_| true)
        .returning(|_| panic!("answer"));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mock.call(()))).is_err());
    mock.assert_verified(); // counts invocation, not successful completion
}
#[test]
fn validation_conditions_cascade_and_collection_paths() {
    struct Batch {
        enabled: bool,
        values: Vec<String>,
    }
    let item = Validator::<String>::new()
        .rule_for("text", |v| v)
        .not_empty()
        .length(2, 5)
        .stop_on_first_failure()
        .done();
    let validator = Validator::<Batch>::new()
        .rule_for("values", |b| &b.values)
        .must(|v| !v.is_empty(), "required")
        .when(|b| b.enabled)
        .done()
        .for_each("values", |b| &b.values, item);
    assert!(
        validator
            .validate(&Batch {
                enabled: false,
                values: vec![]
            })
            .is_ok()
    );
    assert!(
        validator
            .validate(&Batch {
                enabled: true,
                values: vec![]
            })
            .is_err()
    );
    let errors = validator
        .validate(&Batch {
            enabled: true,
            values: vec!["ok".into(), "".into()],
        })
        .unwrap_err();
    assert_eq!(errors.0.len(), 1);
    assert_eq!(errors.0[0].field, "values[1].text");
}
#[test]
fn validator_is_send_sync_and_shared() {
    let validator = Arc::new(
        Validator::<u8>::new()
            .rule_for("age", |v| v)
            .greater_than(18)
            .done(),
    );
    let other = validator.clone();
    assert!(
        std::thread::spawn(move || other.validate(&20))
            .join()
            .unwrap()
            .is_ok()
    );
    assert!(validator.validate(&10).is_err());
}
#[test]
#[should_panic(expected = "ordered bounds")]
fn invalid_validation_range_fails_at_construction() {
    let _ = Validator::<f64>::new()
        .rule_for("x", |v| v)
        .inclusive_between(f64::NAN, 1.0);
}
#[test]
fn assertions_support_projections_vectors_and_tolerance() {
    assert_that(&Some(5))
        .value()
        .is_between(&1, &6)
        .is_less_than(&10);
    assert_that(&Ok::<_, ()>(5)).value().is_equal_to(&5);
    assert_that(&vec![1, 2]).has_length(2).contains(&1);
    assert_that("hello").starts_with("he").ends_with("lo");
    assert_that(&0.1f64).is_close_to(0.10001, 0.001);
}
#[test]
fn float_assertion_rejects_nan_infinity_and_invalid_tolerance() {
    for (value, expected, tolerance) in [
        (f64::NAN, 0.0, 1.0),
        (f64::INFINITY, f64::INFINITY, 0.0),
        (1.0, 1.0, -1.0),
        (1.0, 1.0, f64::INFINITY),
    ] {
        assert!(
            std::panic::catch_unwind(|| assert_that(&value).is_close_to(expected, tolerance))
                .is_err()
        );
    }
}

#[test]
fn late_mock_configuration_is_rejected_without_poisoning() {
    let mock = Mock::<(), ()>::new("sealed");
    mock.expect("once", |_| true).returns(());
    mock.call(());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mock.expect("late", |_| true).returns(());
        }))
        .is_err()
    );
    mock.assert_verified();
}
#[test]
fn matcher_panic_does_not_poison_state_or_count_as_a_match() {
    let mock = Mock::<(), ()>::new("matcher");
    mock.expect("broken matcher", |_| panic!("matcher panic"))
        .returns(());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mock.call(()))).is_err());
    assert!(mock.verify().unwrap_err().to_string().contains("actual 0"));
}
#[test]
fn validation_failure_messages_and_invalid_configuration() {
    let validator = Validator::<String>::new()
        .rule_for("value", |v| v)
        .length(1, 2)
        .with_message("one or two characters")
        .done();
    assert_eq!(
        validator.validate(&"abc".into()).unwrap_err().0[0].message,
        "one or two characters"
    );
    assert!(
        std::panic::catch_unwind(|| {
            let _ = Validator::<String>::new()
                .rule_for("value", |v| v)
                .length(2, 1);
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            let _ = Validator::<String>::new()
                .rule_for("value", |v| v)
                .with_message("no rule");
        })
        .is_err()
    );
}
#[test]
fn assertion_failures_cover_wrong_variants_strings_and_collections() {
    let checks: Vec<Box<dyn Fn()>> = vec![
        Box::new(|| {
            assert_that(&None::<u8>).value();
        }),
        Box::new(|| {
            assert_that(&Err::<u8, _>("failure")).value();
        }),
        Box::new(|| {
            assert_that(&vec![1]).contains(&2);
        }),
        Box::new(|| {
            assert_that(&vec![1]).has_length(0);
        }),
        Box::new(|| {
            assert_that("Alice").starts_with("Bob");
        }),
        Box::new(|| {
            assert_that("Alice").ends_with("Bob");
        }),
        Box::new(|| {
            assert_that(&2).is_less_than(&2);
        }),
        Box::new(|| {
            assert_that(&2).is_between(&3, &5);
        }),
    ];
    for check in checks {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(check)).is_err());
    }
}
