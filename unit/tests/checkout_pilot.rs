#![cfg(feature = "macros")]
use airbug::{CallSequence, with_mocks};
use checkout_domain::*;

struct Harness {
    payments: MockPayments,
    orders: MockOrders,
    events: MockEvents,
    sequence: CallSequence,
}
impl Harness {
    fn new() -> Self {
        Self {
            payments: MockPayments::default(),
            orders: MockOrders::default(),
            events: MockEvents::default(),
            sequence: CallSequence::new("checkout"),
        }
    }
    fn run(&self, request: &OrderRequest) -> Result<Receipt, CheckoutError> {
        let service = Checkout::new(
            self.payments.clone(),
            self.orders.clone(),
            self.events.clone(),
        );
        with_mocks(
            &[&self.payments, &self.orders, &self.events, &self.sequence],
            || service.submit(request),
        )
    }
    fn missing_order(&self) {
        self.orders
            .find
            .expect("lookup", |(id,)| *id == 7)
            .in_sequence(&self.sequence)
            .returns(Ok(None));
    }
    fn charge(&self) {
        self.payments
            .charge
            .expect("charge exact amount", |(id, amount)| {
                *id == 7 && *amount == 2500
            })
            .in_sequence(&self.sequence)
            .returns(Ok("charge-7".into()));
    }
    fn save(&self, result: Result<(), Failure>) {
        self.orders
            .save
            .expect("save paid order", |(order,)| order == &saved())
            .in_sequence(&self.sequence)
            .returns(result);
    }
}
fn request() -> OrderRequest {
    OrderRequest {
        id: 7,
        customer: "Alice".into(),
        amount_cents: 2500,
    }
}
fn saved() -> SavedOrder {
    SavedOrder {
        request: request(),
        charge_id: "charge-7".into(),
    }
}
#[test]
fn success_charges_saves_then_publishes_exact_payload() {
    let test = Harness::new();
    test.missing_order();
    test.charge();
    test.save(Ok(()));
    test.events
        .publish
        .expect("publish exact event", |(event,)| {
            event
                == &OrderPlaced {
                    order_id: 7,
                    amount_cents: 2500,
                }
        })
        .in_sequence(&test.sequence)
        .returns(Ok(()));
    assert_eq!(test.run(&request()), Ok(saved().receipt()));
}
#[airbug::cases(
    no_id(0, "Alice", 2500),
    no_customer(7, " ", 2500),
    no_amount(7, "Alice", 0)
)]
fn invalid_request_makes_no_external_calls(id: u64, customer: &str, amount_cents: u64) {
    let test = Harness::new();
    let request = OrderRequest {
        id,
        customer: customer.into(),
        amount_cents,
    };
    assert_eq!(test.run(&request), Err(CheckoutError::InvalidRequest));
}
#[test]
fn duplicate_request_does_not_charge_or_publish_again() {
    let test = Harness::new();
    test.orders
        .find
        .expect("existing", |(id,)| *id == 7)
        .in_sequence(&test.sequence)
        .returns(Ok(Some(saved())));
    assert_eq!(test.run(&request()), Ok(saved().receipt()));
}
#[test]
fn reused_id_with_changed_payload_is_rejected() {
    let test = Harness::new();
    test.orders
        .find
        .expect("existing", |(id,)| *id == 7)
        .in_sequence(&test.sequence)
        .returns(Ok(Some(saved())));
    let mut changed = request();
    changed.amount_cents = 2600;
    assert_eq!(test.run(&changed), Err(CheckoutError::IdempotencyConflict));
}
#[test]
fn lookup_failure_does_not_charge() {
    let test = Harness::new();
    test.orders
        .find
        .expect("unavailable", |_| true)
        .in_sequence(&test.sequence)
        .returns(Err(Failure::Unavailable));
    assert_eq!(
        test.run(&request()),
        Err(CheckoutError::Lookup(Failure::Unavailable))
    );
}
#[test]
fn rejected_charge_does_not_save_publish_or_refund() {
    let test = Harness::new();
    test.missing_order();
    test.payments
        .charge
        .expect("declined", |_| true)
        .in_sequence(&test.sequence)
        .returns(Err(Failure::Rejected));
    assert_eq!(
        test.run(&request()),
        Err(CheckoutError::Charge(Failure::Rejected))
    );
}
#[test]
fn save_failure_refunds_the_exact_charge() {
    let test = Harness::new();
    test.missing_order();
    test.charge();
    test.save(Err(Failure::Unavailable));
    test.payments
        .refund
        .expect("compensate", |(charge,)| charge == "charge-7")
        .in_sequence(&test.sequence)
        .returns(Ok(()));
    assert_eq!(
        test.run(&request()),
        Err(CheckoutError::Save {
            cause: Failure::Unavailable,
            refund_failure: None
        })
    );
}
#[test]
fn failed_refund_retains_both_errors() {
    let test = Harness::new();
    test.missing_order();
    test.charge();
    test.save(Err(Failure::Unavailable));
    test.payments
        .refund
        .expect("failed compensation", |_| true)
        .in_sequence(&test.sequence)
        .returns(Err(Failure::Rejected));
    assert_eq!(
        test.run(&request()),
        Err(CheckoutError::Save {
            cause: Failure::Unavailable,
            refund_failure: Some(Failure::Rejected)
        })
    );
}
#[test]
fn event_failure_reports_committed_receipt_without_refund() {
    let test = Harness::new();
    test.missing_order();
    test.charge();
    test.save(Ok(()));
    test.events
        .publish
        .expect("event unavailable", |_| true)
        .in_sequence(&test.sequence)
        .returns(Err(Failure::Unavailable));
    assert_eq!(
        test.run(&request()),
        Err(CheckoutError::CommittedButEventFailed {
            receipt: saved().receipt(),
            cause: Failure::Unavailable
        })
    );
}
#[test]
fn deliberately_wrong_order_is_detected_before_side_effects() {
    let test = Harness::new();
    test.missing_order();
    test.charge();
    test.save(Ok(()));
    let error = test
        .orders
        .save
        .try_call((saved(),))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("expected step 1 Orders::find")
            && error.contains("received step 3 Orders::save")
    );
    assert!(test.sequence.verify().is_err());
}
