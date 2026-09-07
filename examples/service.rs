use runit::{FixtureContext, Generate, GenerationError, Mock, Validator, assert_that};

#[derive(Debug)]
struct Order {
    customer: String,
    quantity: u32,
}
impl Generate for Order {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(Self {
            customer: ctx.try_build()?,
            quantity: ctx.try_build()?,
        })
    }
}
trait Repository: Send + Sync {
    fn save(&self, customer: String, quantity: u32) -> Result<u64, String>;
}
struct MockRepository {
    save: Mock<(String, u32), Result<u64, String>>,
}
impl Repository for MockRepository {
    fn save(&self, customer: String, quantity: u32) -> Result<u64, String> {
        self.save.call((customer, quantity))
    }
}
struct Service {
    repository: Box<dyn Repository>,
    validator: Validator<Order>,
}
impl Service {
    fn place(&self, order: &Order) -> Result<u64, String> {
        self.validator.validate(order).map_err(|e| e.to_string())?;
        self.repository.save(order.customer.clone(), order.quantity)
    }
}
fn main() {
    let save = Mock::new("Repository::save");
    save.expect("valid order", |(name, quantity): &(String, u32)| {
        name == "Alice" && *quantity == 2
    })
    .times(1)
    .returns(Ok(42));
    let mut fixture = FixtureContext::with_seed(42);
    fixture.reuse(String::from("Alice")).reuse(2u32);
    let repository = save.clone();
    fixture.register(move |_| Service {
        repository: Box::new(MockRepository {
            save: repository.clone(),
        }),
        validator: Validator::<Order>::new()
            .rule_for("customer", |o| &o.customer)
            .not_empty()
            .done()
            .rule_for("quantity", |o| &o.quantity)
            .inclusive_between(1, 100)
            .done(),
    });
    let service = fixture.build_registered::<Service>();
    let order = fixture.build::<Order>();
    assert_that(&service.place(&order)).value().is_equal_to(&42);
    let invalid = Order {
        customer: String::new(),
        quantity: 0,
    };
    assert_that(&service.place(&invalid)).is_err();
    save.assert_verified(); // invalid order must not call the repository
}
