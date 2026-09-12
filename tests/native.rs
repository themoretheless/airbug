use airbug::{Mock, assert_contains, assert_same_items, check_all, with_mocks};
#[test]
fn collection_macros_preserve_duplicates() {
    assert_contains!(vec![1, 2], 2);
    assert_contains!("hello", "ell");
    let values = vec![1, 2, 1];
    assert_same_items!(values, [2, 1, 1]);
    assert!(std::panic::catch_unwind(|| assert_same_items!([1, 1, 2], [1, 2, 2])).is_err());
}
#[test]
fn collection_macro_evaluates_expressions_once() {
    let mut calls = 0;
    assert_contains!(
        {
            calls += 1;
            vec![1, 2]
        },
        {
            calls += 1;
            2
        }
    );
    assert_eq!(calls, 2);
}
#[test]
fn group_collects_false_checks_without_short_circuit() {
    let mut evaluations = 0;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        check_all! { { evaluations += 1; false }; { evaluations += 1; false }; }
    }));
    assert_eq!(evaluations, 2);
    let message = result.unwrap_err().downcast::<String>().unwrap();
    assert!(message.contains("failed checks"));
    check_all! { 1 < 2; "ok".len() == 2; }
}
#[test]
fn verification_wrapper_detects_missing_calls() {
    let mock = Mock::<(), ()>::new("required");
    mock.expect("once", |_| true).returns(());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| with_mocks(&[&mock], || {})))
            .is_err()
    );
}
#[test]
fn verification_wrapper_preserves_original_panic() {
    let mock = Mock::<(), ()>::new("required");
    mock.expect("once", |_| true).returns(());
    let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        with_mocks(&[&mock], || panic!("original failure"))
    }))
    .unwrap_err();
    assert_eq!(*error.downcast::<&str>().unwrap(), "original failure");
}
