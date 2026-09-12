use airbug::{FixtureContext, Generate, Validator, with_mocks};

#[derive(Generate)]
struct Order {
    customer: String,
    quantity: u32,
}
#[airbug::mock]
trait Repository: Send + Sync {
    fn save(&self, customer: &str, quantity: u32) -> Result<u64, String>;
}
struct Service<R> {
    repository: R,
    validator: Validator<Order>,
}
impl<R: Repository> Service<R> {
    fn place(&self, order: &Order) -> Result<u64, String> {
        self.validator.validate(order).map_err(|e| e.to_string())?;
        self.repository.save(&order.customer, order.quantity)
    }
}
fn main() {
    let repository = MockRepository::default();
    repository
        .save
        .expect("valid order", |(name, quantity)| {
            name == "Alice" && *quantity == 2
        })
        .returns(Ok(42));
    let service = Service {
        repository: repository.clone(),
        validator: Validator::<Order>::new()
            .rule_for("customer", |o| &o.customer)
            .not_empty()
            .done()
            .rule_for("quantity", |o| &o.quantity)
            .inclusive_between(1, 100)
            .done(),
    };
    let mut fixture = FixtureContext::with_seed(42);
    with_mocks(&[&repository], || {
        let order = fixture
            .builder::<Order>()
            .with_customer("Alice".into())
            .with_quantity(2)
            .build();
        assert_eq!(service.place(&order), Ok(42));
        let invalid = fixture.builder::<Order>().with_quantity(0).build();
        assert!(service.place(&invalid).is_err());
    });
}
