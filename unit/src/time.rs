//! Deterministic clocks and cooperative polling. No async executor is installed.
use std::{
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};
/// Why a clock refused an operation, for instance an overflowing advance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockError(
    /// Static description of the refusal.
    pub &'static str,
);
impl fmt::Display for ClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ClockError {}
/// The time source code under test should depend on, so tests can drive it.
///
/// [`ManualClock`] advances only when told to; [`RealClock`] delegates to the
/// operating system. Monotonic [`elapsed`](Clock::elapsed) and wall
/// [`wall_time`](Clock::wall_time) are separate, so a wall-clock jump cannot
/// move a timeout.
pub trait Clock {
    /// Monotonic time since the clock was created. Never goes backwards.
    fn elapsed(&self) -> Duration;
    /// Current wall-clock time, which may jump in either direction.
    fn wall_time(&self) -> SystemTime;
    /// Wait for `duration`, however this clock chooses to represent waiting.
    fn sleep(&self, duration: Duration) -> Result<(), ClockError>;
}
/// A clock that only moves when the test moves it, so no test sleeps.
/// Clones share one timeline.
#[derive(Clone)]
pub struct ManualClock {
    state: Arc<Mutex<(Duration, SystemTime)>>,
}
impl ManualClock {
    /// Start at zero elapsed and the given wall time.
    pub fn new(wall_time: SystemTime) -> Self {
        Self {
            state: Arc::new(Mutex::new((Duration::ZERO, wall_time))),
        }
    }
    /// Move both monotonic and wall time forward. Errors instead of wrapping
    /// on overflow.
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
    /// Jump wall time without touching monotonic time, to model an NTP
    /// correction or a user changing the system clock.
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
/// The system clock: [`Instant`] for elapsed time, [`SystemTime`] for wall
/// time, and a real thread sleep. Use it in production wiring, not in tests.
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
/// One probe result kept for the failure report of [`Eventually::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// Clock time when the probe ran.
    pub elapsed: Duration,
    /// `Debug` rendering of what the probe returned.
    pub value: String,
}
/// A condition that never held, reported with what was actually seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventuallyError {
    /// Why polling stopped, for instance a timeout or a rejected interval.
    pub reason: String,
    /// How many times the probe ran.
    pub attempts: usize,
    /// The most recent observations, capped by
    /// [`Eventually::history_limit`].
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
/// Poll a condition until it holds or a deadline passes, on a [`Clock`] the
/// test controls, so retries need no real waiting.
#[derive(Debug, Clone)]
pub struct Eventually {
    /// Give up once this much clock time has passed.
    pub timeout: Duration,
    /// Wait this long between probes. Must be nonzero.
    pub interval: Duration,
    /// How many observations to keep for the failure report. Defaults to 16.
    pub history_limit: usize,
}
impl Eventually {
    /// A poller with the default history limit of 16 observations.
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

    /// Async sibling of [`check`](Eventually::check): awaits each probe on the
    /// caller's runtime. The clock still controls timeouts and sleep between
    /// attempts (use [`ManualClock`] in tests).
    pub async fn check_async<T, Fut>(
        &self,
        clock: &impl Clock,
        mut probe: impl FnMut() -> Fut,
        predicate: impl Fn(&T) -> bool,
    ) -> Result<T, EventuallyError>
    where
        T: fmt::Debug,
        Fut: core::future::Future<Output = T>,
    {
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
            let value = probe().await;
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

/// Shared clock whose [`sleep`](Clock::sleep) blocks until another thread
/// [`advance`](ParkClock::advance)s enough time or calls [`unpark`](ParkClock::unpark).
#[derive(Clone)]
pub struct ParkClock {
    state: Arc<(Mutex<ParkState>, std::sync::Condvar)>,
}

struct ParkState {
    elapsed: Duration,
    wall: SystemTime,
    unpark: bool,
}

impl ParkClock {
    /// Start at zero elapsed and the given wall time.
    pub fn new(wall_time: SystemTime) -> Self {
        Self {
            state: Arc::new((
                Mutex::new(ParkState {
                    elapsed: Duration::ZERO,
                    wall: wall_time,
                    unpark: false,
                }),
                std::sync::Condvar::new(),
            )),
        }
    }

    /// Move both monotonic and wall time forward, waking sleepers.
    pub fn advance(&self, duration: Duration) -> Result<(), ClockError> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().expect("park clock lock poisoned");
        state.elapsed = state
            .elapsed
            .checked_add(duration)
            .ok_or(ClockError("monotonic overflow"))?;
        state.wall = state
            .wall
            .checked_add(duration)
            .ok_or(ClockError("wall time overflow"))?;
        cvar.notify_all();
        Ok(())
    }

    /// Wake any thread blocked in [`sleep`](Clock::sleep) without advancing time.
    pub fn unpark(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().expect("park clock lock poisoned");
        state.unpark = true;
        cvar.notify_all();
    }
}

impl Clock for ParkClock {
    fn elapsed(&self) -> Duration {
        self.state
            .0
            .lock()
            .expect("park clock lock poisoned")
            .elapsed
    }

    fn wall_time(&self) -> SystemTime {
        self.state.0.lock().expect("park clock lock poisoned").wall
    }

    fn sleep(&self, duration: Duration) -> Result<(), ClockError> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().expect("park clock lock poisoned");
        let target = state
            .elapsed
            .checked_add(duration)
            .ok_or(ClockError("monotonic overflow"))?;
        state.unpark = false;
        while state.elapsed < target && !state.unpark {
            state = cvar.wait(state).expect("park clock lock poisoned");
        }
        state.unpark = false;
        Ok(())
    }
}

/// Absolute monotonic deadline relative to a [`Clock::elapsed`] timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deadline {
    /// Elapsed time at which the deadline expires.
    pub end: Duration,
}

impl Deadline {
    /// `clock.elapsed() + timeout`.
    pub fn from_timeout(clock: &impl Clock, timeout: Duration) -> Result<Self, ClockError> {
        let end = clock
            .elapsed()
            .checked_add(timeout)
            .ok_or(ClockError("deadline overflow"))?;
        Ok(Self { end })
    }

    /// Time left until [`end`](Deadline::end), or zero when expired.
    pub fn remaining(&self, clock: &impl Clock) -> Duration {
        self.end.saturating_sub(clock.elapsed())
    }

    /// Whether `clock.elapsed()` has reached or passed [`end`](Deadline::end).
    pub fn expired(&self, clock: &impl Clock) -> bool {
        clock.elapsed() >= self.end
    }
}

/// Exponential backoff helper for retry loops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retry {
    /// First delay returned by [`next_delay`](Retry::next_delay).
    pub initial: Duration,
    /// Multiplier applied after each delay (at least 1).
    pub factor: u32,
    /// Cap for each delay.
    pub max: Duration,
    current: Duration,
}

impl Retry {
    /// Factor 2, max 60s, starting at `initial`.
    pub fn exponential(initial: Duration) -> Self {
        Self {
            initial,
            factor: 2,
            max: Duration::from_secs(60),
            current: initial,
        }
    }

    /// Override the growth factor (clamped to at least 1).
    pub fn factor(mut self, factor: u32) -> Self {
        self.factor = factor.max(1);
        self
    }

    /// Override the per-delay ceiling.
    pub fn max(mut self, max: Duration) -> Self {
        self.max = max;
        self
    }

    /// Return the next delay and grow `current` by `factor`, capped at `max`.
    pub fn next_delay(&mut self) -> Duration {
        let delay = self.current.min(self.max);
        self.current = self
            .current
            .saturating_mul(self.factor.into())
            .min(self.max);
        delay
    }
}
