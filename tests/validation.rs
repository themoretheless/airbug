use runit::Validator;
struct Address {
    city: String,
}
struct User {
    name: String,
    age: u8,
    address: Address,
    token: Option<String>,
}
fn validator() -> Validator<User> {
    Validator::<User>::new()
        .rule_for("name", |u| &u.name)
        .not_empty()
        .with_message("name is required")
        .length(2, 20)
        .done()
        .rule_for("age", |u| &u.age)
        .inclusive_between(18, 120)
        .done()
        .rule_for("token", |u| &u.token)
        .not_none()
        .done()
        .child(
            "address",
            |u| &u.address,
            Validator::<Address>::new()
                .rule_for("city", |a| a.city.as_str())
                .not_empty()
                .done(),
        )
}
#[test]
fn accumulates_errors_in_order_with_nested_paths() {
    let user = User {
        name: String::new(),
        age: 17,
        address: Address { city: " ".into() },
        token: None,
    };
    let errors = validator().validate(&user).unwrap_err();
    assert_eq!(
        errors
            .0
            .iter()
            .map(|e| e.field.as_str())
            .collect::<Vec<_>>(),
        ["name", "name", "age", "token", "address.city"]
    );
    assert_eq!(errors.0[0].message, "name is required");
    assert!(
        errors
            .to_string()
            .contains("address.city: must not be empty")
    );
}
#[test]
fn validator_is_reusable_and_accepts_inclusive_boundaries() {
    let validator = validator();
    for age in [18, 120] {
        let user = User {
            name: "Юя".into(),
            age,
            address: Address {
                city: "Tbilisi".into(),
            },
            token: Some("x".into()),
        };
        assert_eq!(validator.validate(&user), Ok(()));
    }
}
#[test]
fn custom_rules_and_comparisons() {
    let validator = Validator::<i32>::new()
        .rule_for("value", |v| v)
        .greater_than(0)
        .must(|v| v % 2 == 0, "must be even")
        .done();
    assert!(validator.validate(&2).is_ok());
    assert_eq!(validator.validate(&0).unwrap_err().0.len(), 1);
    assert_eq!(validator.validate(&-1).unwrap_err().0.len(), 2);
}
#[test]
fn empty_validator_accepts_values() {
    assert!(Validator::<()>::new().validate(&()).is_ok());
}
