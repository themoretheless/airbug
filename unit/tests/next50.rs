use airbug::checks::{SoftAssert, assert_duration_eq};
use airbug::fixture::FixtureContext;
use airbug::native::{EnvScope, TempDir};
use airbug::order::{CallDag, happened_before};
use airbug::prop::{Prop, PropError, PropFailure};
use airbug::snapshot::{SnapshotError, Snapshots, UpdateMode};
use airbug::strategy::Strategy;
use airbug::time::{Clock, Deadline, Eventually, ManualClock, ParkClock, Retry};
use airbug::validation::{ValidationCancel, Validator, validate_parallel};
use std::{
    cell::Cell,
    future::Future,
    path::PathBuf,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
    time::{Duration, SystemTime},
};

fn noop_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
    )
}

fn block_on<F: Future>(fut: F) -> F::Output {
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = Box::pin(fut);
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("future pending without async runtime"),
    }
}

static NEXT_TEMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new(tag: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "airbug-next50-{tag}-{}-{stamp}-{serial}",
            std::process::id()
        )))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn obsolete_snapshots_are_detected() {
    let dir = Temp::new("obs");
    let snaps = Snapshots::new(&dir.0).mode(UpdateMode::CreateMissing);
    snaps.check("keep", "a").unwrap();
    snaps.check("gone", "b").unwrap();
    let obsolete = snaps.list_obsolete(&["keep"]).unwrap();
    assert_eq!(obsolete.len(), 1);
    assert!(obsolete[0].ends_with("test-gone.snap"));
    assert!(matches!(
        snaps.assert_no_obsolete(&["keep"]),
        Err(SnapshotError::Obsolete { .. })
    ));
    snaps.assert_no_obsolete(&["keep", "gone"]).unwrap();
}

#[cfg(feature = "json")]
#[test]
fn json_snapshot_is_canonical() {
    let dir = Temp::new("json");
    let snaps = Snapshots::new(&dir.0).mode(UpdateMode::CreateMissing);
    let a = serde_json::json!({"b": 1, "a": {"z": 2, "y": 3}});
    let b = serde_json::json!({"a": {"y": 3, "z": 2}, "b": 1});
    snaps.check_json("order", &a).unwrap();
    snaps.check_json("order", &b).unwrap();
}

#[cfg(feature = "json")]
#[test]
fn json_mode_compact_is_stable() {
    use airbug::snapshot::JsonMode;
    let dir = Temp::new("json-compact");
    let snaps = Snapshots::new(&dir.0)
        .mode(UpdateMode::CreateMissing)
        .json_mode(JsonMode::Compact);
    let value = serde_json::json!({"b": 1, "a": 2});
    snaps.check_json("compact", &value).unwrap();
    let text = std::fs::read_to_string(dir.0.join("test-compact.snap")).unwrap();
    assert_eq!(text, "{\"a\":2,\"b\":1}\n");
}

#[test]
fn check_bytes_stores_lowercase_hex() {
    let dir = Temp::new("bytes");
    let snaps = Snapshots::new(&dir.0).mode(UpdateMode::CreateMissing);
    snaps.check_bytes("payload", &[0x0a, 0xff, 0x00]).unwrap();
    let text = std::fs::read_to_string(dir.0.join("test-payload.snap")).unwrap();
    assert_eq!(text, "0aff00");
    snaps.check_bytes("payload", &[0x0a, 0xff, 0x00]).unwrap();
}

#[test]
fn prop_for_all_and_replay() {
    Prop::new(7)
        .trials(20)
        .for_all::<u64, _>(|n| Ok(n == n))
        .unwrap();

    let dir = Temp::new("prop");
    let path = dir.0.join("fail.seed");
    let err = Prop::new(1)
        .trials(5)
        .persist_failures(&path)
        .for_all::<u64, _>(|_| Ok(false))
        .unwrap_err();
    let PropError::Failed(fail) = err else {
        panic!("expected Failed");
    };
    assert!(path.is_file());
    let seed = PropFailure::read_seed_file(&path).unwrap();
    assert_eq!(seed, fail.seed);
    let replay = Prop::replay::<u64, _>(seed, |_| Ok(false)).unwrap_err();
    assert!(matches!(replay, PropError::Failed(_)));
}

#[test]
fn prop_for_all2_draws_pair() {
    // Both halves are trivially true by design; this exercises for_all2's pair plumbing.
    #[allow(clippy::overly_complex_bool_expr)]
    Prop::new(3)
        .trials(10)
        .for_all2::<u64, bool, _>(|a, b| Ok(*a == *a && (*b || !*b)))
        .unwrap();
}

#[test]
fn choose_weighted_respects_nonzero_weights() {
    let mut ctx = FixtureContext::with_seed(9);
    let values = ["a", "b", "c"];
    let picked = ctx.choose_weighted(&values, &[0, 10, 0]).unwrap();
    assert_eq!(*picked, "b");
    assert!(ctx.choose_weighted::<&str>(&[], &[]).is_err());
    assert!(ctx.choose_weighted(&values, &[1, 2]).is_err());
    assert!(ctx.choose_weighted(&values, &[0, 0, 0]).is_err());
}

#[test]
fn soft_assert_alias_and_duration_eq() {
    let mut soft: SoftAssert = SoftAssert::default();
    soft.equal("n", &1, &1);
    soft.assert();
    assert_duration_eq(
        Duration::from_millis(105),
        Duration::from_millis(100),
        Duration::from_millis(10),
    );
}

#[test]
fn temp_dir_and_env_scope() {
    let temp = TempDir::new().unwrap();
    assert!(temp.path().is_dir());
    let key = format!("AIRBUG_NEXT50_ENV_{}", std::process::id());
    assert!(std::env::var_os(&key).is_none());
    {
        let _guard = EnvScope::set(&key, "scoped");
        assert_eq!(std::env::var(&key).unwrap(), "scoped");
    }
    assert!(std::env::var_os(&key).is_none());
    let kept = temp.path_buf();
    drop(temp);
    assert!(!kept.exists());
}

#[test]
fn eventually_check_async_succeeds() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    let ticks = Cell::new(0u8);
    let result = block_on(
        Eventually::new(Duration::from_secs(5), Duration::from_secs(1)).check_async(
            &clock,
            || {
                ticks.set(ticks.get() + 1);
                let value = ticks.get();
                async move { value }
            },
            |v| *v >= 2,
        ),
    );
    assert_eq!(result.unwrap(), 2);
    assert_eq!(clock.elapsed(), Duration::from_secs(1));
}

#[test]
fn strategy_map_filter_and_for_all_strategy() {
    let strategy = Strategy::<u64>::from_generate()
        .map(|n| n % 10)
        .filter(|n| *n >= 3);
    let mut ctx = FixtureContext::with_seed(11);
    let value = strategy.draw(&mut ctx).unwrap();
    assert!((3..10).contains(&value));

    Prop::new(3)
        .trials(15)
        .for_all_strategy(strategy, |n| Ok(*n >= 3 && *n < 10))
        .unwrap();
}

#[test]
fn validation_cancel_async_and_parallel() {
    let cancel = ValidationCancel::new();
    assert!(!cancel.is_cancelled());
    cancel.cancel();
    assert!(cancel.is_cancelled());

    let v = Validator::<i32>::new()
        .rule_for("n", |n| n)
        .greater_than(0)
        .done();
    assert!(block_on(v.validate_async(&1)).is_ok());
    assert!(block_on(v.validate_async(&0)).is_err());

    let a = Validator::<i32>::new()
        .rule_for("a", |n| n)
        .greater_than(0)
        .done();
    let b = Validator::<i32>::new()
        .rule_for("b", |n| n)
        .inclusive_between(10, 20)
        .done();
    let err = validate_parallel(&[&a, &b], &5).unwrap_err();
    assert_eq!(err.0.len(), 1);
    assert_eq!(err.0[0].field, "b");
    validate_parallel(&[&a], &5).unwrap();
}

#[test]
fn park_clock_deadline_and_retry() {
    let clock = ParkClock::new(SystemTime::UNIX_EPOCH);
    let sleeper = clock.clone();
    let advancer = clock.clone();
    let handle = thread::spawn(move || {
        sleeper.sleep(Duration::from_secs(5)).unwrap();
    });
    thread::sleep(Duration::from_millis(20));
    advancer.advance(Duration::from_secs(2)).unwrap();
    thread::sleep(Duration::from_millis(20));
    advancer.advance(Duration::from_secs(3)).unwrap();
    handle.join().unwrap();
    assert_eq!(clock.elapsed(), Duration::from_secs(5));

    let unpark = ParkClock::new(SystemTime::UNIX_EPOCH);
    let sleeper = unpark.clone();
    let waker = unpark.clone();
    let handle = thread::spawn(move || {
        sleeper.sleep(Duration::from_secs(60)).unwrap();
    });
    thread::sleep(Duration::from_millis(20));
    waker.unpark();
    handle.join().unwrap();

    let manual = ManualClock::new(SystemTime::UNIX_EPOCH);
    manual.advance(Duration::from_secs(3)).unwrap();
    let deadline = Deadline::from_timeout(&manual, Duration::from_secs(5)).unwrap();
    assert_eq!(deadline.remaining(&manual), Duration::from_secs(5));
    assert!(!deadline.expired(&manual));
    manual.advance(Duration::from_secs(5)).unwrap();
    assert!(deadline.expired(&manual));
    assert_eq!(deadline.remaining(&manual), Duration::ZERO);

    let mut retry = Retry::exponential(Duration::from_millis(10)).max(Duration::from_millis(50));
    assert_eq!(retry.next_delay(), Duration::from_millis(10));
    assert_eq!(retry.next_delay(), Duration::from_millis(20));
    assert_eq!(retry.next_delay(), Duration::from_millis(40));
    assert_eq!(retry.next_delay(), Duration::from_millis(50));
}

#[test]
fn call_dag_and_happened_before() {
    let mut dag = CallDag::new().edge("charge", "save").edge("save", "email");
    dag.admit("charge");
    dag.admit("save");
    dag.admit("email");
    dag.assert();
    assert!(dag.happened_before("charge", "email"));

    let mut bad = CallDag::new().edge("a", "b");
    bad.admit("b");
    assert!(bad.verify().is_err());

    let events = [("write", 1u64), ("read", 3u64), ("write", 5u64)];
    assert!(happened_before("write", "read", &events));
    assert!(!happened_before("read", "write", &events));
}

#[test]
fn prop_shrink_from_minimizes_u64() {
    use airbug::prop::Shrink;
    assert!(10u64.shrink().contains(&5));
    let minimal = Prop::shrink_from::<u64, _>(42, |_| Ok(false)).unwrap();
    assert_eq!(minimal, 0);
}

#[test]
fn prop_shrink_from_string() {
    let fail_seed = (0..200u64)
        .find(|&s| {
            let mut ctx = FixtureContext::with_seed(s);
            let value: String = airbug::Generate::generate(&mut ctx).unwrap();
            !value.is_empty()
        })
        .expect("non-empty string seed");
    let minimal = Prop::shrink_from::<String, _>(fail_seed, |_| Ok(false)).unwrap();
    assert_eq!(minimal, "");
}

#[test]
fn pending_response_stays_pending() {
    use airbug::mock::{Mock, pending_response};
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn raw() -> RawWaker {
        fn clone(_: *const ()) -> RawWaker {
            raw()
        }
        fn wake(_: *const ()) {}
        fn wake_by_ref(_: *const ()) {}
        fn drop(_: *const ()) {}
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
        )
    }
    let waker = unsafe { Waker::from_raw(raw()) };
    let mut cx = Context::from_waker(&waker);
    let mut a = Box::pin(pending_response::<u32>());
    let mut b = Box::pin(Mock::<(), ()>::pending::<()>());
    assert!(matches!(a.as_mut().poll(&mut cx), Poll::Pending));
    assert!(matches!(b.as_mut().poll(&mut cx), Poll::Pending));
}

#[test]
fn map_args_and_borrowed_helpers() {
    use airbug::mock::borrowed;
    let mock = airbug::Mock::<String, usize>::new("len");
    mock.expect("hello", |s| s == "HELLO")
        .map_args(|s| s.to_ascii_uppercase())
        .at_least(1)
        .returns(5);
    assert_eq!(mock.call("hello".into()), 5);
    assert_eq!(borrowed::call_to_owned(&mock, "hello"), 5);
    assert_eq!(
        borrowed::map_args(&mock, "hello", |s: &str| s.to_owned()),
        5
    );

    let once = airbug::Mock::<String, usize>::new("once");
    once.expect("x", |s| s == "x").returns(1);
    assert_eq!(borrowed::to_owned_arg("x"), String::from("x"));
    assert_eq!(borrowed::call_to_owned(&once, "x"), 1);
}

#[test]
fn completion_barrier_documents_gap() {
    use airbug::order::CompletionBarrier;
    let barrier = CompletionBarrier;
    assert_eq!(
        barrier.await_done(),
        Err("not implemented: use application join/await")
    );
}

#[test]
fn inline_source_is_unsupported_stub() {
    let dir = Temp::new("inline");
    let snaps = Snapshots::new(&dir.0).mode(UpdateMode::InlineSource);
    assert!(matches!(
        snaps.check("x", "y"),
        Err(SnapshotError::Unsupported(_))
    ));
}

#[test]
fn report_flaky_is_callable() {
    airbug::report::flaky("next50 batch3");
}
