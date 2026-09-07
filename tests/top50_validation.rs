use runit::Validator;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[test]
fn stable_codes_are_independent_of_messages() {
    let validator = Validator::<String>::new()
        .rule_for("name", |v| v)
        .not_empty()
        .with_message("Required")
        .done();
    let errors = validator.validate(&String::new()).unwrap_err();
    assert_eq!(errors.0[0].code, "not_empty");
    assert_eq!(errors.0[0].message, "Required");
}
#[test]
fn cross_field_predicate_compares_fields() {
    let validator =
        Validator::<(u8, u8)>::new().check("end", "order", "end precedes start", |(a, b)| a <= b);
    assert!(validator.validate(&(1, 2)).is_ok());
    assert_eq!(validator.validate(&(2, 1)).unwrap_err().0[0].code, "order");
}
#[test]
fn conditional_group_is_skipped_and_preserves_order() {
    let group = Validator::<u8>::new()
        .rule_for("value", |v| v)
        .greater_than(5)
        .done();
    let validator = Validator::new().group_when(|v: &u8| *v != 0, group);
    assert!(validator.validate(&0).is_ok());
    assert!(validator.validate(&1).is_err());
    assert!(validator.validate(&6).is_ok());
}
#[test]
fn optional_child_checks_only_present_values() {
    let child = Validator::<String>::new()
        .rule_for("name", |v| v)
        .not_empty()
        .done();
    let validator = Validator::<Option<String>>::new().optional("customer", |v| v, child);
    assert!(validator.validate(&None).is_ok());
    let error = validator.validate(&Some(String::new())).unwrap_err();
    assert_eq!(error.0[0].field, "customer.name");
    assert_eq!(error.0[0].code, "not_empty");
}
#[test]
fn collection_uniqueness_uses_selected_key() {
    let validator = Validator::<Vec<(u8, String)>>::new().unique_by("items", |v| v, |v| v.0);
    let errors = validator
        .validate(&vec![(1, "a".into()), (1, "b".into()), (2, "a".into())])
        .unwrap_err();
    assert_eq!(errors.0.len(), 1);
    assert_eq!(errors.0[0].field, "items[1]");
    assert_eq!(errors.0[0].code, "unique");
}
#[test]
fn error_budget_skips_nested_rules_after_limit() {
    let visits = Arc::new(AtomicUsize::new(0));
    let count = visits.clone();
    let child = Validator::<u8>::new()
        .rule_for("v", |v| v)
        .must(
            move |_| {
                count.fetch_add(1, Ordering::SeqCst);
                false
            },
            "bad",
        )
        .done();
    let validator = Validator::<Vec<u8>>::new()
        .for_each("items", |v| v, child)
        .max_errors(2);
    assert_eq!(
        validator.validate(&vec![1, 2, 3, 4]).unwrap_err().0.len(),
        2
    );
    assert_eq!(visits.load(Ordering::SeqCst), 2);
}
#[test]
fn global_stop_skips_later_checks_even_on_same_field() {
    let validator = Validator::<String>::new()
        .rule_for("name", |v| v)
        .not_empty()
        .length(2, 4)
        .done()
        .stop_on_first_failure();
    assert_eq!(validator.validate(&String::new()).unwrap_err().0.len(), 1);
}
#[test]
fn errors_convert_to_application_format() {
    let validator = Validator::<u8>::new()
        .rule_for("age", |v| v)
        .greater_than(18)
        .with_code("adult_required")
        .done();
    let errors = validator
        .validate(&10)
        .unwrap_err()
        .map(|e| (e.field, e.code));
    assert_eq!(errors, [("age".into(), "adult_required".into())]);
}
