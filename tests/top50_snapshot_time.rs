use runit::{
    snapshot::{SnapshotError, Snapshots, UpdateMode, text_diff},
    time::{Clock, Eventually, ManualClock},
};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
static NEXT_TEMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "runit-snapshot-{}-{stamp}-{serial}",
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
fn snapshots_verify_never_creates_files() {
    let dir = Temp::new();
    assert!(matches!(
        Snapshots::new(&dir.0).check("missing", "text"),
        Err(SnapshotError::Missing(_))
    ));
    assert!(!dir.0.exists());
}
#[test]
fn snapshots_only_update_explicitly() {
    let dir = Temp::new();
    let snapshots = Snapshots::new(&dir.0);
    snapshots
        .clone()
        .mode(UpdateMode::CreateMissing)
        .check("one", "old")
        .unwrap();
    snapshots.check("one", "old").unwrap();
    assert!(snapshots.check("one", "new").is_err());
    assert!(
        snapshots
            .clone()
            .mode(UpdateMode::CreateMissing)
            .check("one", "new")
            .is_err()
    );
    snapshots.check("one", "old").unwrap();
    snapshots
        .clone()
        .mode(UpdateMode::Overwrite)
        .check("one", "new")
        .unwrap();
    snapshots.check("one", "new").unwrap();
}
#[test]
fn diff_identifies_line_and_values() {
    let diff = text_diff("a\nb", "a\nc");
    assert!(diff.contains("line 2") && diff.contains("b") && diff.contains("c"));
    assert!(!text_diff("a\n", "a").is_empty());
}
#[test]
fn inline_snapshots_compare_without_writes() {
    let dir = Temp::new();
    let snapshots = Snapshots::new(&dir.0).mode(UpdateMode::Overwrite);
    snapshots.inline("a", "a").unwrap();
    assert!(snapshots.inline("a", "b").is_err());
    assert!(!dir.0.exists());
}
#[test]
fn redaction_applies_before_storage_and_diagnostics() {
    let dir = Temp::new();
    let snapshots = Snapshots::new(&dir.0).redact("secret", "<redacted>");
    snapshots
        .clone()
        .mode(UpdateMode::CreateMissing)
        .check("value", "token=secret")
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.0.join("test-value.snap")).unwrap(),
        "token=<redacted>"
    );
    let error = snapshots
        .check("value", "other=secret")
        .unwrap_err()
        .to_string();
    assert!(!error.contains("secret"));
}
#[test]
fn cases_have_separate_snapshot_paths() {
    let dir = Temp::new();
    let snapshots = Snapshots::new(&dir.0).mode(UpdateMode::CreateMissing);
    snapshots.check_case("test", "a", "A").unwrap();
    snapshots.check_case("test", "b", "B").unwrap();
    let verify = snapshots.mode(UpdateMode::Verify);
    verify.check_case("test", "a", "A").unwrap();
    verify.check_case("test", "b", "B").unwrap();
    assert!(verify.check_case("../escape", "x", "x").is_err());
}
#[test]
fn size_limits_and_failed_updates_leave_no_lock() {
    let dir = Temp::new();
    let snapshots = Snapshots::new(&dir.0)
        .mode(UpdateMode::Overwrite)
        .max_bytes(2);
    assert!(matches!(
        snapshots.check("big", "abc"),
        Err(SnapshotError::TooLarge)
    ));
    std::fs::create_dir_all(dir.0.join("test-small.tmp")).unwrap();
    assert!(snapshots.check("small", "a").is_err());
    assert!(!dir.0.join("test-small.lock").exists());
}
#[test]
fn manual_clock_advances_without_sleeping() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    clock.advance(Duration::from_secs(2)).unwrap();
    assert_eq!(clock.elapsed(), Duration::from_secs(2));
    assert_eq!(
        clock.wall_time(),
        SystemTime::UNIX_EPOCH + Duration::from_secs(2)
    );
    let other = clock.clone();
    other.sleep(Duration::from_secs(3)).unwrap();
    assert_eq!(clock.elapsed(), Duration::from_secs(5));
}
#[test]
fn wall_adjustments_do_not_change_monotonic_time() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    clock.advance(Duration::from_secs(5)).unwrap();
    clock.set_wall_time(SystemTime::UNIX_EPOCH);
    assert_eq!(clock.elapsed(), Duration::from_secs(5));
    assert_eq!(clock.wall_time(), SystemTime::UNIX_EPOCH);
}
#[test]
fn eventual_success_and_timeout_keep_bounded_observations() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    let mut calls = 0;
    assert_eq!(
        Eventually::new(Duration::from_secs(10), Duration::from_secs(1))
            .check(
                &clock,
                || {
                    calls += 1;
                    calls
                },
                |v| *v == 3
            )
            .unwrap(),
        3
    );
    assert_eq!(clock.elapsed(), Duration::from_secs(2));
    let poll = Eventually {
        timeout: Duration::from_secs(10),
        interval: Duration::from_secs(3),
        history_limit: 2,
    };
    let error = poll.check(&clock, || false, |v| *v).unwrap_err();
    assert_eq!(error.attempts, 5);
    assert_eq!(error.history.len(), 2);
    assert_eq!(error.history[1].elapsed, Duration::from_secs(10));
}
#[test]
fn polling_rejects_zero_interval_and_handles_zero_timeout() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    assert!(
        Eventually::new(Duration::ZERO, Duration::ZERO)
            .check(&clock, || true, |v| *v)
            .is_err()
    );
    assert!(
        Eventually::new(Duration::ZERO, Duration::from_secs(1))
            .check(&clock, || true, |v| *v)
            .is_ok()
    );
    assert_eq!(
        Eventually::new(Duration::ZERO, Duration::from_secs(1))
            .check(&clock, || false, |v| *v)
            .unwrap_err()
            .attempts,
        1
    );
}

#[test]
fn concurrent_snapshot_updates_never_publish_partial_text() {
    let dir = Temp::new();
    std::fs::create_dir_all(&dir.0).unwrap();
    let snapshots = Snapshots::new(&dir.0).mode(UpdateMode::Overwrite);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
    let workers: Vec<_> = (0..4)
        .map(|i| {
            let snapshots = snapshots.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let text = format!("value-{i}\n").repeat(1000);
                barrier.wait();
                snapshots.check("concurrent", &text)
            })
        })
        .collect();
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert!(results.iter().any(Result::is_ok));
    let stored = std::fs::read_to_string(dir.0.join("test-concurrent.snap")).unwrap();
    assert!((0..4).any(|i| stored == format!("value-{i}\n").repeat(1000)));
    assert!(!dir.0.join("test-concurrent.lock").exists());
}
#[test]
fn redaction_expansion_respects_limit_before_allocation() {
    let dir = Temp::new();
    let snapshots = Snapshots::new(&dir.0).max_bytes(4).redact("x", "long");
    assert!(matches!(
        snapshots.inline("", "xx"),
        Err(SnapshotError::TooLarge)
    ));
}
#[test]
fn nonadvancing_clock_fails_instead_of_hanging() {
    struct Frozen;
    impl Clock for Frozen {
        fn elapsed(&self) -> Duration {
            Duration::ZERO
        }
        fn wall_time(&self) -> SystemTime {
            SystemTime::UNIX_EPOCH
        }
        fn sleep(&self, _: Duration) -> Result<(), runit::time::ClockError> {
            Ok(())
        }
    }
    let error = Eventually::new(Duration::from_secs(1), Duration::from_millis(1))
        .check(&Frozen, || false, |v| *v)
        .unwrap_err();
    assert!(error.reason.contains("no progress"));
}
#[test]
fn successful_probe_after_deadline_is_rejected() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    let result = Eventually::new(Duration::from_secs(1), Duration::from_millis(1)).check(
        &clock,
        || {
            clock.advance(Duration::from_secs(2)).unwrap();
            true
        },
        |v| *v,
    );
    assert!(result.is_err());
}

#[test]
fn slow_predicate_does_not_trigger_another_probe() {
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
    let mut probes = 0;
    let result = Eventually::new(Duration::from_secs(1), Duration::from_millis(1)).check(
        &clock,
        || {
            probes += 1;
            true
        },
        |_| {
            clock.advance(Duration::from_secs(2)).unwrap();
            false
        },
    );
    assert!(result.is_err());
    assert_eq!(probes, 1);
}
#[test]
fn oversleep_does_not_trigger_probe_past_deadline() {
    struct Oversleep(ManualClock);
    impl Clock for Oversleep {
        fn elapsed(&self) -> Duration {
            self.0.elapsed()
        }
        fn wall_time(&self) -> SystemTime {
            self.0.wall_time()
        }
        fn sleep(&self, duration: Duration) -> Result<(), runit::time::ClockError> {
            self.0.advance(duration + Duration::from_secs(2))
        }
    }
    let clock = Oversleep(ManualClock::new(SystemTime::UNIX_EPOCH));
    let mut probes = 0;
    let result = Eventually::new(Duration::from_secs(1), Duration::from_millis(1)).check(
        &clock,
        || {
            probes += 1;
            false
        },
        |v| *v,
    );
    assert!(result.is_err());
    assert_eq!(probes, 1);
}
