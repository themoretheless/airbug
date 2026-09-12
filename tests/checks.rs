use airbug::checks::*;
#[test]
fn field_report_preserves_path_actual_and_expected() {
    struct User {
        age: u8,
    }
    let user = User { age: 17 };
    let mut report = CheckReport::default();
    report.field_equal(&user, "user.age", |u| &u.age, &18);
    assert_eq!(
        report.failures[0],
        CheckFailure {
            path: "user.age".into(),
            actual: "17".into(),
            expected: "18".into()
        }
    );
    assert!(std::panic::catch_unwind(|| report.assert()).is_err());
    let mut good = CheckReport::default();
    good.equal("age", &18, &18);
    good.assert();
}
#[test]
fn unique_rejects_duplicate_indexes() {
    assert_unique(&[1, 2]);
    assert_panics("indexes 0 and 2", || assert_unique(&[1, 2, 1]));
}
#[test]
fn sorted_accepts_equal_values_and_rejects_nan() {
    assert_sorted(&[1, 1, 2]);
    assert_panics("out of order", || assert_sorted(&[2, 1]));
    assert_panics("unordered", || assert_sorted(&[0.0, f64::NAN]));
}
#[test]
fn all_and_count_have_actionable_failures() {
    assert_all(&[2, 4], |x| x % 2 == 0);
    assert_count(&[1, 2, 3, 4], 2, |x| x % 2 == 0);
    assert_panics("[1]", || assert_all(&[2, 3], |x| x % 2 == 0));
    assert_panics("matching count", || {
        assert_count(&[1, 2], 2, |x| x % 2 == 0)
    });
}
#[test]
fn subset_and_superset_preserve_multiplicity() {
    assert_subset(&[1, 1], &[1, 2, 1]);
    assert_superset(&[1, 2, 1], &[1, 1]);
    assert_panics("missing", || assert_subset(&[1, 1], &[1, 2]));
}
#[test]
fn relative_comparison_avoids_overflow() {
    assert_relative_eq(f64::MAX, f64::MAX / 2.0, 0.5);
    assert_relative_eq(0.0, -0.0, 0.0);
    assert_panics("relative tolerance", || {
        assert_relative_eq(f64::MAX, -f64::MAX, 1.0)
    });
    assert_panics("finite", || assert_relative_eq(f64::NAN, 1.0, 0.1));
}
#[test]
fn newline_normalization_preserves_other_whitespace() {
    assert_text_eq("a\r\nb\r", "a\nb\n");
    assert_panics("text differs", || assert_text_eq("a ", "a"));
}
#[derive(Debug)]
struct Outer(std::io::Error);
impl std::fmt::Display for Outer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "outer")
    }
}
impl std::error::Error for Outer {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
#[test]
fn error_chain_finds_nested_message() {
    let e = Outer(std::io::Error::other("inner"));
    assert_error_chain_contains(&e, "inner");
    assert_panics("inspected", || {
        let e = Outer(std::io::Error::other("inner"));
        assert_error_chain_contains(&e, "absent");
    });
}
#[test]
fn panic_check_rejects_missing_wrong_and_non_string_payloads() {
    assert_panics("needle", || panic!("needle in message"));
    assert_panics("body returned", || assert_panics("needle", || {}));
    assert_panics("actual", || assert_panics("needle", || panic!("different")));
    assert_panics("non-string", || {
        assert_panics("needle", || std::panic::panic_any(7u8))
    });
}
#[test]
fn report_accumulates_structured_errors() {
    let mut report = CheckReport::default();
    report.equal("a", &1, &2).equal("b", &3, &4);
    assert_eq!(report.failures.len(), 2);
    assert!(report.to_string().contains("b: expected 4, actual 3"));
}
