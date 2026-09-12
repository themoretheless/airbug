//! Demonstration service for testing Airbug ergonomics, not a production payment integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderRequest {
    pub id: u64,
    pub customer: String,
    pub amount_cents: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub order_id: u64,
    pub charge_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedOrder {
    pub request: OrderRequest,
    pub charge_id: String,
}
impl SavedOrder {
    pub fn receipt(&self) -> Receipt {
        Receipt {
            order_id: self.request.id,
            charge_id: self.charge_id.clone(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderPlaced {
    pub order_id: u64,
    pub amount_cents: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Unavailable,
    Rejected,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckoutError {
    InvalidRequest,
    IdempotencyConflict,
    Lookup(Failure),
    Charge(Failure),
    Save {
        cause: Failure,
        refund_failure: Option<Failure>,
    },
    CommittedButEventFailed {
        receipt: Receipt,
        cause: Failure,
    },
}
#[cfg_attr(feature = "macros", airbug::mock)]
pub trait Payments: Send + Sync {
    fn charge(&self, order_id: u64, amount_cents: u64) -> Result<String, Failure>;
    fn refund(&self, charge_id: &str) -> Result<(), Failure>;
}
#[cfg_attr(feature = "macros", airbug::mock)]
pub trait Orders: Send + Sync {
    fn find(&self, id: u64) -> Result<Option<SavedOrder>, Failure>;
    fn save(&self, order: &SavedOrder) -> Result<(), Failure>;
}
#[cfg_attr(feature = "macros", airbug::mock)]
pub trait Events: Send + Sync {
    fn publish(&self, event: &OrderPlaced) -> Result<(), Failure>;
}
pub struct Checkout<P, O, E> {
    payments: P,
    orders: O,
    events: E,
}
impl<P: Payments, O: Orders, E: Events> Checkout<P, O, E> {
    pub fn new(payments: P, orders: O, events: E) -> Self {
        Self {
            payments,
            orders,
            events,
        }
    }
    pub fn submit(&self, request: &OrderRequest) -> Result<Receipt, CheckoutError> {
        if request.id == 0 || request.customer.trim().is_empty() || request.amount_cents == 0 {
            return Err(CheckoutError::InvalidRequest);
        }
        if let Some(existing) = self
            .orders
            .find(request.id)
            .map_err(CheckoutError::Lookup)?
        {
            return if existing.request == *request {
                Ok(existing.receipt())
            } else {
                Err(CheckoutError::IdempotencyConflict)
            };
        }
        let charge_id = self
            .payments
            .charge(request.id, request.amount_cents)
            .map_err(CheckoutError::Charge)?;
        let order = SavedOrder {
            request: request.clone(),
            charge_id,
        };
        if let Err(cause) = self.orders.save(&order) {
            let refund_failure = self.payments.refund(&order.charge_id).err();
            return Err(CheckoutError::Save {
                cause,
                refund_failure,
            });
        }
        let receipt = order.receipt();
        self.events
            .publish(&OrderPlaced {
                order_id: request.id,
                amount_cents: request.amount_cents,
            })
            .map_err(|cause| CheckoutError::CommittedButEventFailed {
                receipt: receipt.clone(),
                cause,
            })?;
        Ok(receipt)
    }
}
