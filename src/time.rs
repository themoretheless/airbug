//! Deterministic clocks and cooperative polling. No async executor is installed.
use std::{
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockError(pub &'static str);
impl fmt::Display for ClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ClockError {}
pub trait Clock {
    fn elapsed(&self) -> Duration;
    fn wall_time(&self) -> SystemTime;
    fn sleep(&self, duration: Duration) -> Result<(), ClockError>;
}
#[derive(Clone)]
pub struct ManualClock {
    state: Arc<Mutex<(Duration, SystemTime)>>,
}
impl ManualClock {
    pub fn new(wall_time: SystemTime) -> Self {
        Self {
            state: Arc::new(Mutex::new((Duration::ZERO, wall_time))),
        }
    }
    pub fn advance(&self, duration: Duration) -> Result<(), ClockError> {
        let mut state = self.state.lock().expect("clock lock poisoned");
        let elapsed = state
            .0
            .checked_add(duration)
            .ok_or(ClockError("monotonic overflow"))?;
        let wall = state
            .1
            .checked_add(duration)
            .ok_or(ClockError("wall time overflow"))?;
        *state = (elapsed, wall);
        Ok(())
    }
    /// Calendar correction does not change elapsed time or deadlines.
    pub fn set_wall_time(&self, wall_time: SystemTime) {
        self.state.lock().expect("clock lock poisoned").1 = wall_time;
    }
}
impl Clock for ManualClock {
    fn elapsed(&self) -> Duration {
        self.state.lock().expect("clock lock poisoned").0
    }
    fn wall_time(&self) -> SystemTime {
        self.state.lock().expect("clock lock poisoned").1
    }
    fn sleep(&self, duration: Duration) -> Result<(), ClockError> {
        self.advance(duration)
    }
}
pub struct RealClock {
    start: Instant,
}
impl Default for RealClock {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}
impl Clock for RealClock {
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    fn wall_time(&self) -> SystemTime {
        SystemTime::now()
    }
    fn sleep(&self, duration: Duration) -> Result<(), ClockError> {
        std::thread::sleep(duration);
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub elapsed: Duration,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventuallyError {
    pub reason: String,
    pub attempts: usize,
    pub history: Vec<Observation>,
}
impl fmt::Display for EventuallyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} after {} attempts: {:?}",
            self.reason, self.attempts, self.history
        )
    }
}
impl std::error::Error for EventuallyError {}
#[derive(Debug, Clone)]
pub struct Eventually {
    pub timeout: Duration,
    pub interval: Duration,
    pub history_limit: usize,
}
impl Eventually {
    pub fn new(timeout: Duration, interval: Duration) -> Self {
        Self {
            timeout,
            interval,
            history_limit: 16,
        }
    }
    /// Probe synchronously; it must return promptly. A deadline cannot preempt user code.
    pub fn check<T: fmt::Debug>(
        &self,
        clock: &impl Clock,
        mut probe: impl FnMut() -> T,
        predicate: impl Fn(&T) -> bool,
    ) -> Result<T, EventuallyError> {
        let mut history = std::collections::VecDeque::new();
        let mut attempts = 0;
        let fail = |reason: String, attempts, history: std::collections::VecDeque<Observation>| {
            EventuallyError {
                reason,
                attempts,
                history: history.into_iter().collect(),
            }
        };
        if self.interval.is_zero() {
            return Err(fail(
                "poll interval must be positive".into(),
                attempts,
                history,
            ));
        }
        let start = clock.elapsed();
        let deadline = start
            .checked_add(self.timeout)
            .ok_or_else(|| fail("deadline overflow".into(), 0, history.clone()))?;
        loop {
            if attempts > 0 && clock.elapsed() > deadline {
                return Err(fail("deadline exceeded".into(), attempts, history));
            }
            let value = probe();
            attempts += 1;
            let now = clock.elapsed();
            if now < start {
                return Err(fail("clock moved backwards".into(), attempts, history));
            }
            if self.history_limit > 0 {
                if history.len() == self.history_limit {
                    history.pop_front();
                }
                history.push_back(Observation {
                    elapsed: now - start,
                    value: format!("{value:?}"),
                });
            }
            let matched = now <= deadline && predicate(&value);
            let after_predicate = clock.elapsed();
            if after_predicate < now {
                return Err(fail("clock moved backwards".into(), attempts, history));
            }
            let now = after_predicate;
            if matched && now <= deadline {
                return Ok(value);
            }
            if now >= deadline {
                return Err(fail("deadline exceeded".into(), attempts, history));
            }
            clock
                .sleep(self.interval.min(deadline - now))
                .map_err(|e| fail(e.to_string(), attempts, history.clone()))?;
            if clock.elapsed() <= now {
                return Err(fail(
                    "clock sleep made no progress".into(),
                    attempts,
                    history,
                ));
            }
        }
    }
}
