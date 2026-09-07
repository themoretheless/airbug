use runit::{FixtureContext, Generate, GenerationError, assert_that};

#[derive(Debug)]
struct Order {
    id: u64,
    names: Vec<String>,
}
impl Generate for Order {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(Self {
            id: ctx.try_build()?,
            names: ctx.try_build()?,
        })
    }
}
#[test]
fn graph_is_reproducible_and_factories_override_nested_edges() {
    let mut a = FixtureContext::with_seed(42);
    let mut b = FixtureContext::with_seed(42);
    let first = a.build::<Order>();
    let second = b.build::<Order>();
    assert_eq!(first.id, second.id);
    assert_eq!(first.names, second.names);
    assert_ne!(first.id, a.build::<Order>().id);
    a.reuse(String::from("client")).collection_len(2);
    assert_eq!(a.build::<Order>().names, vec!["client", "client"]);
}
struct Cycle;
impl Generate for Cycle {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        ctx.try_build::<Cycle>()
    }
}
#[test]
fn cycles_report_path_and_restore_context() {
    let mut ctx = FixtureContext::new();
    let error = ctx.try_build::<Cycle>().err().unwrap();
    assert_eq!(error.path.len(), 2);
    ctx.max_depth(1);
    let _ = ctx.build::<u64>();
}
#[test]
fn unrelated_panics_propagate_and_restore_context() {
    let mut ctx = FixtureContext::new();
    ctx.register::<u64>(|_| panic!("factory failed"));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ctx.try_build::<u64>()));
    assert!(result.is_err());
    ctx.reuse(7u64).max_depth(1);
    assert_eq!(ctx.build::<u64>(), 7);
}
#[test]
fn factory_can_build_a_type_without_generate() {
    struct Service(u64);
    let mut ctx = FixtureContext::new();
    ctx.reuse(12u64).register(|ctx| Service(ctx.build()));
    assert_eq!(ctx.build_registered::<Service>().0, 12);
}
#[test]
fn fluent_assertions_borrow_and_chain() {
    let text = String::from("hello world");
    assert_that(&text).contains_text("world").is_equal_to(&text);
    assert_that(&5).is_greater_than(&3).is_not_equal_to(&4);
    assert_that(&true).is_true();
    assert_that(&false).is_false();
    assert_that(&Some(1)).is_some();
    assert_that(&None::<u8>).is_none();
    assert_that(&Ok::<_, ()>(1)).is_ok();
    assert_that(&Err::<(), _>("error")).is_err();
    assert_that([1, 2, 3].as_slice()).has_length(3).contains(&2);
    assert_eq!(text, "hello world");
}
#[test]
fn failure_reports_actual_expected_and_reason() {
    let failure = std::panic::catch_unwind(|| {
        assert_that(&2)
            .because("two items were requested")
            .is_equal_to(&3);
    })
    .unwrap_err();
    let message = failure.downcast_ref::<String>().unwrap();
    assert!(message.contains("expected 2 to equal 3"));
    assert!(message.contains("because two items were requested"));
}
