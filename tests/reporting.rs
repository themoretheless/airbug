use airbug::{assert_that, report};

#[test]
fn nested_checkout_steps_include_payload_and_comparison() {
    report::step("Checkout", || {
        let total = report::step("Prepare order", || {
            report::attach_bytes(
                "order.json",
                "application/json",
                br#"{"id":42,"quantity":2,"price":1250}"#,
            )
            .unwrap();
            2 * 1250
        });
        report::step("Verify payment", || {
            report::assert_equal("Charged amount", &2500, &total);
            report::attach_text("payment.log", "POST /payments\nstatus: 201\namount: 2500")
                .unwrap();
            report::attach_bytes("receipt.bin", "application/octet-stream", &[0, 1, 2, 255])
                .unwrap();
        });
    });
}

#[test]
#[should_panic(expected = "response body")]
fn expected_panic_keeps_text_diff() {
    report::step("Request order", || {
        report::step("Check response", || {
            report::assert_text_equal(
                "response body",
                "status: paid\namount: 2500\n",
                "status: pending\namount: 2400\n",
            );
        });
    });
}

#[test]
#[should_panic(expected = "to equal")]
fn fluent_assertion_keeps_native_panic_and_records_diff() {
    report::step("Check total", || {
        assert_that(&2400).is_equal_to(&2500);
    });
}

#[test]
fn fallible_step_preserves_error_value_without_debug_bound() {
    struct Error;
    let result = report::try_step("Expected decline", || Err::<(), _>(Error));
    assert!(result.is_err());
}

#[test]
fn caught_panic_restores_step_parent_and_preserves_payload() {
    report::step("Recovery", || {
        let panic = std::panic::catch_unwind(|| {
            report::step("Expected failure", || std::panic::panic_any(42u32))
        });
        assert_eq!(*panic.unwrap_err().downcast::<u32>().unwrap(), 42);
        report::step("Continue after recovery", || {
            report::assert_equal("Recovered", &true, &true)
        });
    });
}

#[test]
fn check_report_records_each_field_failure() {
    let mut checks = airbug::checks::CheckReport::default();
    checks
        .equal("order.quantity", &1, &2)
        .equal("order.total", &100, &200);
    assert!(std::panic::catch_unwind(|| checks.assert()).is_err());
}
