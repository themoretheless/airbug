#![cfg(feature = "macros")]
//! Real cross-crate service pilot exercising `airbug-checkout-domain` end-to-end with
//! `#[airbug::mock]` (including generic + associated-type mocks), `CallSequence`,
//! `CompletionBarrier`, `FixtureContext` + `Generate`, `Prop::for_all_shrunk`,
//! `Snapshots` (`UpdateMode::InlineSource`), and `SoftAssert`.

use airbug::checks::SoftAssert;
use airbug::fixture::{FixtureContext, Generate as _, GenerationError};
use airbug::native::TempDir;
use airbug::order::{CallSequence, CompletionBarrier};
use airbug::prop::{Prop, PropError};
use airbug::snapshot::{Snapshots, UpdateMode};
use airbug::strategy::Strategy;
use airbug::verify::{VerifyMocks, with_mocks};
use airbug_checkout_domain::{
    Checkout, CheckoutError, Failure, MockEvents, MockOrders, MockPayments, OrderPlaced,
    OrderRequest, SavedOrder,
};
use std::{thread, time::Duration};

#[derive(Debug, Clone, airbug::Generate)]
struct PilotOrderSeed {
    #[fixture(with = gen_positive_id)]
    id: u64,
    #[fixture(with = gen_customer)]
    customer: String,
    #[fixture(with = gen_amount)]
    amount_cents: u64,
}

fn gen_positive_id(ctx: &mut FixtureContext) -> Result<u64, GenerationError> {
    Ok(u64::generate(ctx)? % 10_000 + 1)
}

fn gen_customer(ctx: &mut FixtureContext) -> Result<String, GenerationError> {
    Ok(format!("cust-{}", u64::generate(ctx)? % 999 + 1))
}

fn gen_amount(ctx: &mut FixtureContext) -> Result<u64, GenerationError> {
    Ok(u64::generate(ctx)? % 50_000 + 100)
}

#[airbug::mock]
trait AuditSink<ID>
where
    ID: Clone,
{
    type ReceiptTag: Clone;
    fn record(&self, order_id: ID, tag: &Self::ReceiptTag) -> bool;
    fn attach<T: 'static>(&self, order_id: ID, detail: T) -> usize;
}

#[test]
fn service_pilot_end_to_end_checkout_flow_and_audit() {
    let mut ctx = FixtureContext::with_seed(2026);
    let seed = ctx.build::<PilotOrderSeed>();
    let req = OrderRequest {
        id: seed.id,
        customer: seed.customer.clone(),
        amount_cents: seed.amount_cents,
    };

    let payments = MockPayments::default();
    let orders = MockOrders::default();
    let events = MockEvents::default();
    let audit = MockAuditSink::<u64, String>::default();
    let seq = CallSequence::new("pilot_checkout");

    orders
        .find
        .expect("find_absent", move |(id,)| *id == req.id)
        .times(1)
        .in_sequence(&seq)
        .returns(Ok(None));
    let expected_id = req.id;
    let expected_amt = req.amount_cents;
    payments
        .charge
        .expect("charge", move |(id, amt)| {
            *id == expected_id && *amt == expected_amt
        })
        .times(1)
        .in_sequence(&seq)
        .returns(Ok(format!("ch-{expected_id}")));
    orders
        .save
        .expect("save_order", move |(saved,)| {
            saved.request.id == expected_id && saved.charge_id == format!("ch-{expected_id}")
        })
        .times(1)
        .in_sequence(&seq)
        .returns(Ok(()));
    events
        .publish
        .expect("publish_event", move |(ev,)| {
            *ev == OrderPlaced {
                order_id: expected_id,
                amount_cents: expected_amt,
            }
        })
        .times(1)
        .in_sequence(&seq)
        .returns(Ok(()));
    audit
        .record
        .expect("audit_tag", move |(id, tag)| {
            *id == expected_id && tag == "settled"
        })
        .times(1)
        .returns(true);
    audit
        .attach
        .expect::<(u64, String), usize>("attach_note", move |(id, note)| {
            *id == expected_id && note.contains("receipt:")
        })
        .times(1)
        .returns(1);

    let mocks: [&dyn VerifyMocks; 5] = [&payments, &orders, &events, &audit, &seq];
    with_mocks(&mocks, || {
        let checkout = Checkout::new(payments.clone(), orders.clone(), events.clone());
        let receipt = checkout.submit(&req).expect("checkout succeeds");
        assert!(audit.record(receipt.order_id, &"settled".to_string()));
        assert_eq!(
            audit.attach(receipt.order_id, format!("receipt:{}", receipt.charge_id)),
            1
        );

        let mut soft = SoftAssert::default();
        soft.equal("order_id", &receipt.order_id, &req.id);
        soft.equal("charge_id", &receipt.charge_id, &format!("ch-{}", req.id));
        soft.assert();
    });
}

#[test]
fn service_pilot_save_failure_compensates_and_coordinates_workers() {
    let barrier = CompletionBarrier::named("worker_settle", 2);
    let payments = MockPayments::default();
    let orders = MockOrders::default();
    let events = MockEvents::default();

    orders
        .find
        .expect("miss", |_| true)
        .times(2)
        .returns(Ok(None));
    payments
        .charge
        .expect("charge", |_| true)
        .times(2)
        .returns_sequence([Ok("ch-1".into()), Ok("ch-2".into())]);
    orders
        .save
        .expect("fail_one_pass_one", |_| true)
        .times(2)
        .returns_sequence([Err(Failure::Unavailable), Ok(())]);
    payments
        .refund
        .expect("compensate", |(cid,)| cid == "ch-1")
        .times(1)
        .returns(Ok(()));
    events
        .publish
        .expect("pub_second", |(ev,)| ev.order_id == 2)
        .times(1)
        .returns(Ok(()));

    let checkout_a = Checkout::new(payments.clone(), orders.clone(), events.clone());
    let checkout_b = Checkout::new(payments.clone(), orders.clone(), events.clone());
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let h1 = thread::spawn(move || {
        let _tok = b1.enter("first");
        let res = checkout_a.submit(&OrderRequest {
            id: 1,
            customer: "Alice".into(),
            amount_cents: 500,
        });
        assert_eq!(
            res,
            Err(CheckoutError::Save {
                cause: Failure::Unavailable,
                refund_failure: None,
            })
        );
    });
    let h2 = thread::spawn(move || {
        let _tok = b2.enter("second");
        let res = checkout_b.submit(&OrderRequest {
            id: 2,
            customer: "Bob".into(),
            amount_cents: 900,
        });
        assert!(res.is_ok());
    });

    barrier.wait_timeout(Duration::from_secs(2)).unwrap();
    h1.join().unwrap();
    h2.join().unwrap();
    let mocks: [&dyn VerifyMocks; 4] = [&payments, &orders, &events, &barrier];
    with_mocks(&mocks, || {});
}

#[test]
fn service_pilot_property_shrinking_and_snapshots() {
    // Reject invalid requests via property check and verify counterexample shrinks cleanly.
    let strat = Strategy::from_fn(|ctx| {
        Ok((
            u64::generate(ctx)? % 3,
            String::generate(ctx)?,
            u64::generate(ctx)? % 50,
        ))
    });
    let prop_res = Prop::new(17)
        .trials(50)
        .max_shrinks(200)
        .for_all_shrunk_strategy(strat, |(id, cust, amt)| {
            let p = MockPayments::default();
            let o = MockOrders::default();
            let e = MockEvents::default();
            let c = Checkout::new(p, o, e);
            let res = c.submit(&OrderRequest {
                id: *id,
                customer: cust.clone(),
                amount_cents: *amt,
            });
            Ok(!matches!(res, Err(CheckoutError::InvalidRequest)))
        });
    assert!(matches!(prop_res, Err(PropError::Failed(_))));

    // Inline snapshot verification against rendered domain summary.
    let saved = SavedOrder {
        request: OrderRequest {
            id: 42,
            customer: "Ada".into(),
            amount_cents: 1200,
        },
        charge_id: "ch-42".into(),
    };
    let receipt = saved.receipt();
    let summary = format!("order={} charge={}", receipt.order_id, receipt.charge_id);
    airbug::assert_inline_snapshot!(summary, @"order=42 charge=ch-42");

    // InlineSource update flow in temporary file.
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("pilot_snap.rs");
    std::fs::write(&file, "let s = \"pending\";\n").unwrap();
    let updater = Snapshots::new(dir.path()).mode(UpdateMode::InlineSource);
    updater
        .inline_source(&file, 1, 9, "pending", &summary)
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "let s = \"order=42 charge=ch-42\";\n"
    );

    #[cfg(feature = "json")]
    {
        let json_snaps = Snapshots::new(dir.path()).mode(UpdateMode::Overwrite);
        json_snaps
            .check_json(
                "receipt",
                &serde_json::json!({"order_id": receipt.order_id, "charge_id": receipt.charge_id}),
            )
            .unwrap();
    }
}
