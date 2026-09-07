#![cfg(feature = "macros")]
use runit::{FixtureContext, Generate, GenerationError, VerifyMocks, with_mocks};

#[derive(Debug, Generate)]
struct Order {
    customer: String,
    note: String,
    quantity: u32,
    #[fixture(default)]
    marker: std::marker::PhantomData<()>,
    #[fixture(with = limited)]
    limit: u32,
}
fn limited(_: &mut FixtureContext) -> Result<u32, GenerationError> {
    Ok(10)
}
#[test]
fn generated_builder_overrides_one_field_only() {
    let mut ctx = FixtureContext::new();
    ctx.reuse(String::from("default"));
    let order = ctx
        .builder::<Order>()
        .with_customer("Alice".into())
        .with_quantity(2)
        .build();
    assert_eq!(order.customer, "Alice");
    assert_eq!(order.note, "default");
    assert_eq!(order.quantity, 2);
    assert_eq!(order.limit, 10);
    assert_eq!(order.marker, std::marker::PhantomData);
    assert_eq!(ctx.build::<Order>().customer, "default");
}
#[derive(Generate)]
struct Pair<T>
where
    T: Clone,
{
    left: T,
    right: T,
}
#[derive(Generate)]
struct Tuple(u64, String);
#[derive(Generate)]
struct Unit;
#[derive(Generate)]
struct Const<const N: usize> {
    #[fixture(default)]
    marker: std::marker::PhantomData<[u8; N]>,
}
#[test]
fn derive_supports_generics_tuple_unit_and_const_generics() {
    let mut ctx = FixtureContext::new();
    let pair = ctx
        .builder::<Pair<u8>>()
        .with_left(9)
        .with_right(10)
        .build();
    assert_eq!((pair.left, pair.right), (9, 10));
    let tuple = ctx
        .builder::<Tuple>()
        .with_0(12)
        .with_1("tuple".into())
        .build();
    assert_eq!((tuple.0, tuple.1), (12, "tuple".into()));
    let _ = ctx.build::<Unit>();
    let _ = ctx.builder::<Unit>().build();
    let _ = ctx.build::<Const<5>>().marker;
}
#[derive(Generate)]
struct Fallible {
    #[fixture(with = fails)]
    value: u8,
}
fn fails(_: &mut FixtureContext) -> Result<u8, GenerationError> {
    Err(GenerationError::custom("field failed"))
}
#[test]
fn override_skips_failing_generator_and_tracks_root_depth() {
    let mut ctx = FixtureContext::new();
    assert_eq!(ctx.try_build::<Fallible>().err().unwrap().path.len(), 1);
    assert_eq!(ctx.builder::<Fallible>().with_value(7).build().value, 7);
    ctx.max_depth(0);
    assert!(ctx.builder::<Fallible>().with_value(7).try_build().is_err());
}
#[runit::mock]
trait Repository: Send + Sync {
    fn lookup(&self, name: &str) -> Option<u64>;
    fn save(&mut self, id: u64, labels: &[String]) -> Result<(), String>;
    fn ping(&self);
}
#[test]
fn generated_trait_mock_handles_borrowed_inputs_and_owned_responses() {
    let mut repository = MockRepository::default();
    repository
        .lookup
        .expect("Alice", |(name,)| name == "Alice")
        .returns(Some(7));
    repository
        .save
        .expect("labels", |(id, labels)| *id == 7 && labels == &["a"])
        .returns(Ok(()));
    repository.ping.expect("ping", |()| true).returns(());
    assert_eq!(Repository::lookup(&repository, "Alice"), Some(7));
    assert!(repository.save(7, &["a".into()]).is_ok());
    repository.ping();
    repository.verify_mocks().unwrap();
}
#[test]
fn generated_mock_aggregates_missing_method_calls() {
    let repository = MockRepository::default();
    repository.lookup.expect("lookup", |_| true).returns(None);
    repository.ping.expect("ping", |_| true).returns(());
    assert_eq!(repository.verify_mocks().unwrap_err().0.len(), 2);
}
#[test]
fn optional_verification_wrapper_preserves_native_assertions() {
    let repository = MockRepository::default();
    repository.lookup.expect("Alice", |_| true).returns(Some(1));
    with_mocks(&[&repository], || {
        assert_eq!(repository.lookup("Alice"), Some(1));
    });
}
#[runit::cases(zero(0, 0), positive(2, 4), negative(-3, -6))]
fn doubles(input: i32, expected: i32) {
    assert_eq!(input * 2, expected);
}
#[runit::cases(valid("42"))]
fn parsing(value: &str) -> Result<(), std::num::ParseIntError> {
    assert_eq!(value.parse::<u8>()?, 42);
    Ok(())
}
#[runit::cases(wrong(0))]
#[should_panic(expected = "positive")]
fn requires_positive(value: i32) {
    assert!(value > 0, "positive");
}

#[runit::mock]
trait AsyncStore {
    async fn get(&self, id: u64) -> String;
}
#[test]
fn async_mock_and_verification_use_callers_executor() {
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    let store = MockAsyncStore::default();
    store
        .get
        .expect("id", |(id,)| *id == 1)
        .returns("found".into());
    let mocks: [&dyn VerifyMocks; 1] = [&store];
    let future = runit::with_mocks_async(&mocks, async {
        assert_eq!(store.get(1).await, "found");
    });
    let mut future = std::pin::pin!(future);
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(())
    ));
}
#[derive(Generate)]
struct Keywords {
    r#type: u64,
}
#[test]
fn raw_identifiers_generate_valid_setters() {
    assert_eq!(
        FixtureContext::new()
            .builder::<Keywords>()
            .with_type(3)
            .build()
            .r#type,
        3
    );
}
#[test]
fn builder_overrides_root_factory_without_mutating_it() {
    let mut ctx = FixtureContext::new();
    ctx.register(|_| Keywords { r#type: 4 });
    assert_eq!(ctx.builder::<Keywords>().with_type(3).build().r#type, 3);
    assert_eq!(ctx.build::<Keywords>().r#type, 4);
}
