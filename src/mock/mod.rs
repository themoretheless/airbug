//! Strict thread-safe mocks. User matchers, capture cloning and answers run outside locks.
mod capture;
mod expectation;
mod journal;
mod matcher;
mod verify;

pub use capture::Capture;
pub use expectation::ExpectationBuilder;
pub use journal::CallRecord;
pub use matcher::Matcher;
pub use verify::{VerificationErrors, VerifyMocks, with_mocks, with_mocks_async};

use crate::order::SequenceStep;
use journal::Journal;
use std::{
    fmt,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockError {
    pub method: String,
    pub details: Vec<String>,
}
impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mock {}: {}", self.method, self.details.join("; "))
    }
}
impl std::error::Error for MockError {}

type Answer<A, R> = Arc<dyn Fn(A) -> R + Send + Sync>;
type Recorder<A> = Arc<dyn Fn(&A) + Send + Sync>;

struct Expectation<A, R> {
    label: String,
    matcher: Matcher<A>,
    answer: Answer<A, R>,
    min: usize,
    max: usize,
    actual: usize,
    repeatable: bool,
    capture: Option<Recorder<A>>,
    sequence: Option<SequenceStep>,
}
impl<A, R> Expectation<A, R> {
    /// Describes an unmet call count, or None when the count is within range.
    fn count_failure(&self) -> Option<String> {
        if self.actual >= self.min && self.actual <= self.max {
            return None;
        }
        Some(if self.min == self.max {
            format!(
                "{} expected {} calls, actual {}",
                self.label, self.min, self.actual
            )
        } else {
            format!(
                "{} expected {}..={} calls, actual {}",
                self.label, self.min, self.max, self.actual
            )
        })
    }
}

struct State<A, R> {
    expectations: Vec<Expectation<A, R>>,
    failures: Vec<String>,
    started: bool,
    journal: Journal,
    in_flight: usize,
}

pub struct Mock<A, R> {
    name: Arc<str>,
    state: Arc<Mutex<State<A, R>>>,
}
impl<A, R> Clone for Mock<A, R> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            state: self.state.clone(),
        }
    }
}

/// Marks a call as in flight, so `reset_counts` can refuse to run underneath it.
struct Active<'a, A, R>(&'a Mock<A, R>);
impl<A, R> Drop for Active<'_, A, R> {
    fn drop(&mut self) {
        self.0.state.lock().expect("mock lock poisoned").in_flight -= 1;
    }
}

impl<A: fmt::Debug + 'static, R: 'static> Mock<A, R> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Arc::from(name.into()),
            state: Arc::new(Mutex::new(State {
                expectations: Vec::new(),
                failures: Vec::new(),
                started: false,
                journal: Journal::default(),
                in_flight: 0,
            })),
        }
    }
    /// First matching rule wins even if exhausted. Counts default to exactly one.
    pub fn expect(
        &self,
        label: impl Into<String>,
        matcher: impl Fn(&A) -> bool + Send + Sync + 'static,
    ) -> ExpectationBuilder<A, R> {
        self.expect_matcher(Matcher::new(label, matcher))
    }
    pub fn expect_matcher(&self, matcher: Matcher<A>) -> ExpectationBuilder<A, R> {
        ExpectationBuilder {
            mock: self.clone(),
            label: matcher.name().to_owned(),
            matcher,
            min: 1,
            max: 1,
            capture: None,
            sequence: None,
        }
    }
    /// Opt-in bounded journal; zero disables it. Contains Debug representations.
    pub fn journal_capacity(&self, capacity: usize) {
        self.state
            .lock()
            .expect("mock lock poisoned")
            .journal
            .set_capacity(capacity);
    }
    pub fn calls(&self) -> Vec<CallRecord> {
        self.state
            .lock()
            .expect("mock lock poisoned")
            .journal
            .records()
    }
    fn error(&self, details: Vec<String>) -> MockError {
        MockError {
            method: self.name.to_string(),
            details,
        }
    }
    pub fn try_call(&self, args: A) -> Result<R, MockError> {
        let matchers: Vec<_> = {
            let mut state = self.state.lock().expect("mock lock poisoned");
            state.started = true;
            state.in_flight += 1;
            for expectation in &state.expectations {
                if let Some(step) = &expectation.sequence {
                    step.seal();
                }
            }
            state
                .expectations
                .iter()
                .map(|e| e.matcher.clone())
                .collect()
        };
        let _active = Active(self);
        let matched = matchers.iter().position(|matcher| matcher.matches(&args));
        let arguments = format!("{args:?}");
        let (answer, capture) = {
            let mut state = self.state.lock().expect("mock lock poisoned");
            let (result, label) = match matched {
                Some(index) => {
                    let expectation = &mut state.expectations[index];
                    let label = Some(expectation.label.clone());
                    (expectation.admit(&arguments), label)
                }
                None => (Err(format!("unexpected arguments {arguments}")), None),
            };
            state.journal.push(CallRecord {
                arguments,
                expectation: label,
                accepted: result.is_ok(),
            });
            match result {
                Ok(answer) => answer,
                Err(detail) => {
                    state.failures.push(detail.clone());
                    return Err(self.error(vec![detail]));
                }
            }
        };
        if let Some(capture) = capture {
            capture(&args);
        }
        Ok(answer(args))
    }
    #[track_caller]
    pub fn call(&self, args: A) -> R {
        self.try_call(args).unwrap_or_else(|e| panic!("{e}"))
    }
    pub fn verify(&self) -> Result<(), MockError> {
        let state = self.state.lock().expect("mock lock poisoned");
        let mut details = state.failures.clone();
        details.extend(
            state
                .expectations
                .iter()
                .filter_map(Expectation::count_failure),
        );
        if details.is_empty() {
            Ok(())
        } else {
            Err(self.error(details))
        }
    }
    /// Start a new counting phase; reusable answers only. External captures are independent.
    pub fn reset_counts(&self) -> Result<(), MockError> {
        let mut state = self.state.lock().expect("mock lock poisoned");
        if state.in_flight != 0
            || state
                .expectations
                .iter()
                .any(|e| !e.repeatable || e.sequence.is_some())
        {
            return Err(self.error(vec![
                "reset requires idle reusable unordered answers".into(),
            ]));
        }
        for e in &mut state.expectations {
            e.actual = 0;
        }
        state.failures.clear();
        state.journal.clear();
        Ok(())
    }
    #[track_caller]
    pub fn assert_verified(&self) {
        self.verify().unwrap_or_else(|e| panic!("{e}"));
    }
}

impl<A: fmt::Debug + 'static, R: 'static> Expectation<A, R> {
    /// Accept one call against this rule, yielding its answer and capture hook.
    #[allow(clippy::type_complexity)]
    fn admit(&mut self, arguments: &str) -> Result<(Answer<A, R>, Option<Recorder<A>>), String> {
        if self.actual >= self.max {
            let detail = format!(
                "{} exceeded {} calls with {arguments}",
                self.label, self.max
            );
            if let Some(step) = &self.sequence {
                step.record_excess(&detail);
            }
            return Err(detail);
        }
        if let Some(step) = &self.sequence {
            step.admit()?;
        }
        self.actual += 1;
        Ok((self.answer.clone(), self.capture.clone()))
    }
}

impl<A: fmt::Debug + 'static, R: 'static> VerifyMocks for Mock<A, R> {
    fn verify_mocks(&self) -> Result<(), VerificationErrors> {
        self.verify()
            .map_err(|error| VerificationErrors(vec![error]))
    }
}
