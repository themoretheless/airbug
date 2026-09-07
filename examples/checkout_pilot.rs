#[path = "checkout/domain.rs"]
mod domain;
use domain::*;
use runit::{CallSequence, with_mocks};
fn main() {
    let scenario = std::env::args().nth(1).unwrap_or_else(|| "success".into());
    assert!(
        ["success", "decline", "store-failure"].contains(&scenario.as_str()),
        "expected success, decline or store-failure"
    );
    let payments = MockPayments::default();
    let orders = MockOrders::default();
    let events = MockEvents::default();
    let sequence = CallSequence::new("checkout");
    orders
        .find
        .expect("lookup request", |(id,)| *id == 7)
        .in_sequence(&sequence)
        .returns(Ok(None));
    payments
        .charge
        .expect("charge 2500", |(id, amount)| *id == 7 && *amount == 2500)
        .in_sequence(&sequence)
        .returns(if scenario == "decline" {
            Err(Failure::Rejected)
        } else {
            Ok("charge-7".into())
        });
    if scenario != "decline" {
        orders
            .save
            .expect("persist paid order", |(order,)| {
                order.charge_id == "charge-7" && order.request.id == 7
            })
            .in_sequence(&sequence)
            .returns(if scenario == "store-failure" {
                Err(Failure::Unavailable)
            } else {
                Ok(())
            });
        if scenario == "store-failure" {
            payments
                .refund
                .expect("compensate", |(charge,)| charge == "charge-7")
                .in_sequence(&sequence)
                .returns(Ok(()));
        } else {
            events
                .publish
                .expect("publish order", |(event,)| {
                    event.order_id == 7 && event.amount_cents == 2500
                })
                .in_sequence(&sequence)
                .returns(Ok(()));
        }
    }
    let service = Checkout::new(payments.clone(), orders.clone(), events.clone());
    let result = with_mocks(&[&payments, &orders, &events, &sequence], || {
        service.submit(&OrderRequest {
            id: 7,
            customer: "Alice".into(),
            amount_cents: 2500,
        })
    });
    let expected = match scenario.as_str() {
        "decline" => Err(CheckoutError::Charge(Failure::Rejected)),
        "store-failure" => Err(CheckoutError::Save {
            cause: Failure::Unavailable,
            refund_failure: None,
        }),
        _ => Ok(Receipt {
            order_id: 7,
            charge_id: "charge-7".into(),
        }),
    };
    assert_eq!(result, expected);
    println!("{scenario}: {result:?}");
}
