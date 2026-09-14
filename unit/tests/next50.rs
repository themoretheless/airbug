use airbug::prop::{Prop, PropError, PropFailure};
use airbug::snapshot::{SnapshotError, Snapshots, UpdateMode};
use airbug::time::{Clock, Eventually, ManualClock};
use std::{
    cell::Cell,
    future::Future,
    path::PathBuf,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
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
