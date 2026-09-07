use runit::GenerationErrorKind;
use runit::prelude::*;
#[test]
fn prelude_supports_native_tests() {
    assert_contains!([1, 2], 2);
    assert_unique(&[1, 2]);
}
#[test]
fn scoped_overrides_restore_on_success_and_panic() {
    let mut ctx = FixtureContext::with_seed(8);
    ctx.reuse(10u64).collection_len(2);
    assert_eq!(
        ctx.scoped(|ctx| {
            ctx.reuse(20u64).collection_len(1);
            ctx.build::<Vec<u64>>()
        }),
        vec![20]
    );
    assert_eq!(ctx.build::<Vec<u64>>(), vec![10, 10]);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ctx.scoped(|ctx| {
            ctx.reuse(30u64);
            panic!("body");
        })))
        .is_err()
    );
    assert_eq!(ctx.build::<u64>(), 10);
}
#[test]
fn child_inherits_rules_without_mutating_parent() {
    let mut parent = FixtureContext::new();
    parent.reuse("parent".to_string());
    let mut child = parent.child(22);
    assert_eq!(child.seed(), 22);
    assert_eq!(child.build::<String>(), "parent");
    child.reuse("child".to_string());
    assert_eq!(parent.build::<String>(), "parent");
}
#[test]
fn checkpoint_replays_rng_and_rules() {
    let mut ctx = FixtureContext::with_seed(13);
    let checkpoint = ctx.checkpoint();
    let before = ctx.build::<Vec<u64>>();
    ctx.reuse(99u64);
    ctx.restore(checkpoint);
    assert_eq!(ctx.build::<Vec<u64>>(), before);
}
#[test]
fn node_budget_protects_wide_graph_and_errors_include_seed() {
    let mut ctx = FixtureContext::with_seed(77);
    ctx.max_nodes(3).collection_len(3);
    let error = ctx.try_build::<Vec<u8>>().unwrap_err();
    assert_eq!(error.kind, GenerationErrorKind::NodeLimit);
    assert_eq!(error.seed, Some(77));
    assert!(error.to_string().contains("77"));
    ctx.collection_len(2);
    assert_eq!(ctx.build::<Vec<u8>>().len(), 2);
}
#[test]
fn choice_is_repeatable_and_empty_is_error() {
    let mut a = FixtureContext::with_seed(1);
    let mut b = FixtureContext::with_seed(1);
    for _ in 0..20 {
        assert_eq!(a.choose(&[1, 2, 3]).unwrap(), b.choose(&[1, 2, 3]).unwrap());
    }
    assert!(a.choose::<u8>(&[]).is_err());
}
#[test]
fn sequential_ids_stop_at_overflow() {
    let mut ctx = FixtureContext::new();
    ctx.sequence_u64(u64::MAX - 1);
    assert_eq!(ctx.build::<u64>(), u64::MAX - 1);
    assert_eq!(ctx.build::<u64>(), u64::MAX);
    assert!(ctx.try_build::<u64>().is_err());
}
#[test]
fn boundary_sets_include_special_values() {
    use runit::fixture::boundaries::*;
    assert!(I64.contains(&i64::MIN) && I64.contains(&0) && I64.contains(&i64::MAX));
    assert_eq!(U64[0], 0);
    assert!(F64.iter().any(|x| x.is_nan()));
    assert!(F64.iter().any(|x| *x == 0.0 && x.is_sign_negative()));
}
#[test]
fn ordered_pairs_cover_full_range_without_overflow() {
    let mut ctx = FixtureContext::new();
    for _ in 0..100 {
        let (a, b) = ctx.ordered_pair(3, 9).unwrap();
        assert!(3 <= a && a <= b && b <= 9);
    }
    let (a, b) = ctx.ordered_pair(0, u64::MAX).unwrap();
    assert!(a <= b);
    assert_eq!(ctx.ordered_pair(5, 5).unwrap(), (5, 5));
    assert!(ctx.ordered_pair(5, 4).is_err());
}
#[test]
fn stream_is_lazy_and_fallible() {
    let mut ctx = FixtureContext::new();
    ctx.sequence_u64(10);
    let values: Vec<_> = ctx
        .stream::<u64>()
        .take(3)
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(values, vec![10, 11, 12]);
    assert_eq!(ctx.build::<u64>(), 13);
}
