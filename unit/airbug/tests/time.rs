use airbug::time::{Clock, Eventually, ManualClock};
use std::time::{Duration, SystemTime};
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
fn nonadvancing_clock_fails_instead_of_hanging() {
    struct Frozen;
    impl Clock for Frozen {
        fn elapsed(&self) -> Duration {
            Duration::ZERO
        }
        fn wall_time(&self) -> SystemTime {
            SystemTime::UNIX_EPOCH
        }
        fn sleep(&self, _: Duration) -> Result<(), airbug::time::ClockError> {
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
        fn sleep(&self, duration: Duration) -> Result<(), airbug::time::ClockError> {
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
