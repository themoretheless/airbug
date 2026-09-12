//! Baseline: same happy path with standard Rust assertions and handwritten doubles.
use checkout_domain::*;
use std::sync::{Arc, Mutex};
#[derive(Debug, PartialEq)]
enum Call {
    Find(u64),
    Charge(u64, u64),
    Refund(String),
    Save(SavedOrder),
    Publish(OrderPlaced),
}
#[derive(Clone)]
struct Doubles(Arc<Mutex<Vec<Call>>>, Option<Failure>);
impl Payments for Doubles {
    fn charge(&self, id: u64, amount: u64) -> Result<String, Failure> {
        self.0.lock().unwrap().push(Call::Charge(id, amount));
        Ok("charge-7".into())
    }
    fn refund(&self, id: &str) -> Result<(), Failure> {
        self.0.lock().unwrap().push(Call::Refund(id.into()));
        Ok(())
    }
}
impl Orders for Doubles {
    fn find(&self, id: u64) -> Result<Option<SavedOrder>, Failure> {
        self.0.lock().unwrap().push(Call::Find(id));
        match &self.1 {
            Some(error) => Err(error.clone()),
            None => Ok(None),
        }
    }
    fn save(&self, order: &SavedOrder) -> Result<(), Failure> {
        self.0.lock().unwrap().push(Call::Save(order.clone()));
        Ok(())
    }
}
impl Events for Doubles {
    fn publish(&self, event: &OrderPlaced) -> Result<(), Failure> {
        self.0.lock().unwrap().push(Call::Publish(event.clone()));
        Ok(())
    }
}
#[test]
fn native_success_uses_shared_call_log() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let doubles = Doubles(calls.clone(), None);
    let service = Checkout::new(doubles.clone(), doubles.clone(), doubles);
    let request = OrderRequest {
        id: 7,
        customer: "Alice".into(),
        amount_cents: 2500,
    };
    let order = SavedOrder {
        request: request.clone(),
        charge_id: "charge-7".into(),
    };
    assert_eq!(service.submit(&request), Ok(order.receipt()));
    assert_eq!(
        *calls.lock().unwrap(),
        [
            Call::Find(7),
            Call::Charge(7, 2500),
            Call::Save(order),
            Call::Publish(OrderPlaced {
                order_id: 7,
                amount_cents: 2500
            })
        ]
    );
}
#[test]
fn native_lookup_failure_shortcircuits() {
    for failure in [Failure::Unavailable, Failure::Rejected] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let doubles = Doubles(calls.clone(), Some(failure.clone()));
        let service = Checkout::new(doubles.clone(), doubles.clone(), doubles);
        let request = OrderRequest {
            id: 7,
            customer: "Alice".into(),
            amount_cents: 2500,
        };
        assert_eq!(
            service.submit(&request),
            Err(CheckoutError::Lookup(failure))
        );
        assert_eq!(*calls.lock().unwrap(), [Call::Find(7)]);
    }
}
