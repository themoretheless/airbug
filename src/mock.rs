//! Strict thread-safe mocks. User matchers, capture cloning and answers run outside locks.
use crate::order::{CallSequence, SequenceStep};
use std::{
    collections::VecDeque,
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
/// Reusable named predicate, with short-circuit composition.
pub struct Matcher<A> {
    name: String,
    predicate: Arc<dyn Fn(&A) -> bool + Send + Sync>,
}
impl<A> Clone for Matcher<A> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            predicate: self.predicate.clone(),
        }
    }
}
impl<A: 'static> Matcher<A> {
    pub fn new(
        name: impl Into<String>,
        predicate: impl Fn(&A) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            predicate: Arc::new(predicate),
        }
    }
    pub fn matches(&self, value: &A) -> bool {
        (self.predicate)(value)
    }
    pub fn and(self, other: Self) -> Self {
        Self::new(format!("({} and {})", self.name, other.name), move |a| {
            self.matches(a) && other.matches(a)
        })
    }
    pub fn or(self, other: Self) -> Self {
        Self::new(format!("({} or {})", self.name, other.name), move |a| {
            self.matches(a) || other.matches(a)
        })
    }
    pub fn negate(self) -> Self {
        Self::new(format!("not {}", self.name), move |a| !self.matches(a))
    }
}
/// Bounded owned argument capture. Clones share values. Oldest values are evicted.
pub struct Capture<A> {
    values: Arc<Mutex<VecDeque<Arc<A>>>>,
    capacity: usize,
}
impl<A> Clone for Capture<A> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            capacity: self.capacity,
        }
    }
}
impl<A: Clone> Capture<A> {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Arc::new(Mutex::new(VecDeque::new())),
            capacity,
        }
    }
    pub fn record(&self, value: &A) {
        if self.capacity == 0 {
            return;
        }
        let value = Arc::new(value.clone());
        let removed = {
            let mut values = self.values.lock().expect("capture lock poisoned");
            let removed = if values.len() == self.capacity {
                values.pop_front()
            } else {
                None
            };
            values.push_back(value);
            removed
        };
        drop(removed);
    }
    pub fn values(&self) -> Vec<A> {
        let values: Vec<_> = self
            .values
            .lock()
            .expect("capture lock poisoned")
            .iter()
            .cloned()
            .collect();
        values.into_iter().map(|value| (*value).clone()).collect()
    }
    pub fn clear(&self) {
        let removed = std::mem::take(&mut *self.values.lock().expect("capture lock poisoned"));
        drop(removed);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallRecord {
    pub arguments: String,
    pub expectation: Option<String>,
    pub accepted: bool,
}
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
struct State<A, R> {
    expectations: Vec<Expectation<A, R>>,
    failures: Vec<String>,
    started: bool,
    journal: VecDeque<CallRecord>,
    journal_capacity: usize,
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
                journal: VecDeque::new(),
                journal_capacity: 0,
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
            label: matcher.name.clone(),
            matcher,
            min: 1,
            max: 1,
            capture: None,
            sequence: None,
        }
    }
    /// Opt-in bounded journal; zero disables it. Contains Debug representations.
    pub fn journal_capacity(&self, capacity: usize) {
        let mut state = self.state.lock().expect("mock lock poisoned");
        state.journal_capacity = capacity;
        while state.journal.len() > capacity {
            state.journal.pop_front();
        }
    }
    pub fn calls(&self) -> Vec<CallRecord> {
        self.state
            .lock()
            .expect("mock lock poisoned")
            .journal
            .iter()
            .cloned()
            .collect()
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
            let (result, label) = if let Some(index) = matched {
                let expectation = &mut state.expectations[index];
                let label = Some(expectation.label.clone());
                if expectation.actual < expectation.max {
                    let admission = expectation
                        .sequence
                        .as_ref()
                        .map_or(Ok(()), SequenceStep::admit);
                    match admission {
                        Ok(()) => {
                            expectation.actual += 1;
                            (
                                Ok((expectation.answer.clone(), expectation.capture.clone())),
                                label,
                            )
                        }
                        Err(detail) => (Err(detail), label),
                    }
                } else {
                    let detail = format!(
                        "{} exceeded {} calls with {arguments}",
                        expectation.label, expectation.max
                    );
                    if let Some(step) = &expectation.sequence {
                        step.record_excess(&detail);
                    }
                    (Err(detail), label)
                }
            } else {
                (Err(format!("unexpected arguments {arguments}")), None)
            };
            if state.journal_capacity > 0 {
                if state.journal.len() == state.journal_capacity {
                    state.journal.pop_front();
                }
                state.journal.push_back(CallRecord {
                    arguments,
                    expectation: label,
                    accepted: result.is_ok(),
                });
            }
            match result {
                Ok(answer) => answer,
                Err(detail) => {
                    state.failures.push(detail.clone());
                    return Err(MockError {
                        method: self.name.to_string(),
                        details: vec![detail],
                    });
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
                .filter(|e| e.actual < e.min || e.actual > e.max)
                .map(|e| {
                    if e.min == e.max {
                        format!("{} expected {} calls, actual {}", e.label, e.min, e.actual)
                    } else {
                        format!(
                            "{} expected {}..={} calls, actual {}",
                            e.label, e.min, e.max, e.actual
                        )
                    }
                }),
        );
        if details.is_empty() {
            Ok(())
        } else {
            Err(MockError {
                method: self.name.to_string(),
                details,
            })
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
            return Err(MockError {
                method: self.name.to_string(),
                details: vec!["reset requires idle reusable unordered answers".into()],
            });
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
#[must_use = "finish with returns, returning, returning_once or returns_sequence"]
pub struct ExpectationBuilder<A, R> {
    mock: Mock<A, R>,
    label: String,
    matcher: Matcher<A>,
    min: usize,
    max: usize,
    capture: Option<Recorder<A>>,
    sequence: Option<CallSequence>,
}
impl<A: fmt::Debug + 'static, R: 'static> ExpectationBuilder<A, R> {
    pub fn times(self, count: usize) -> Self {
        self.times_between(count, count)
    }
    pub fn times_between(mut self, min: usize, max: usize) -> Self {
        assert!(min <= max, "min must be <= max");
        self.min = min;
        self.max = max;
        self
    }
    pub fn at_least(self, count: usize) -> Self {
        self.times_between(count, usize::MAX)
    }
    pub fn at_most(self, count: usize) -> Self {
        self.times_between(0, count)
    }
    pub fn capture(mut self, capture: &Capture<A>) -> Self
    where
        A: Clone + Send + Sync,
    {
        let capture = capture.clone();
        self.capture = Some(Arc::new(move |args| capture.record(args)));
        self
    }
    /// Join a shared strict sequence. Register terminal answers in desired order.
    /// Requires a positive exact count; ranges cannot define a strict transition.
    pub fn in_sequence(mut self, sequence: &CallSequence) -> Self {
        assert!(
            self.sequence.is_none(),
            "an expectation can join only one sequence"
        );
        self.sequence = Some(sequence.clone());
        self
    }
    fn insert(self, answer: Answer<A, R>, repeatable: bool) {
        let mut state = self.mock.state.lock().expect("mock lock poisoned");
        if state.started {
            drop(state);
            panic!("configure mock expectations before the first call");
        }
        let sequence = if let Some(sequence) = self.sequence {
            if self.min != self.max || self.min == 0 {
                drop(state);
                panic!("ordered expectations require a positive exact count");
            }
            match sequence.register(format!("{} [{}]", self.mock.name, self.label), self.min) {
                Ok(step) => Some(step),
                Err(error) => {
                    drop(state);
                    panic!("{error}");
                }
            }
        } else {
            None
        };
        state.expectations.push(Expectation {
            label: self.label,
            matcher: self.matcher,
            answer,
            min: self.min,
            max: self.max,
            actual: 0,
            repeatable,
            capture: self.capture,
            sequence,
        });
    }
    pub fn returning(self, answer: impl Fn(A) -> R + Send + Sync + 'static) {
        self.insert(Arc::new(answer), true);
    }
    pub fn returns(self, value: R)
    where
        R: Clone + Send + Sync,
    {
        self.returning(move |_| value.clone());
    }
    /// Forces exactly one call, taking ownership of its response factory.
    pub fn returning_once(self, answer: impl FnOnce(A) -> R + Send + 'static) {
        let answer = Mutex::new(Some(answer));
        self.times(1).insert(
            Arc::new(move |args| {
                let answer = answer
                    .lock()
                    .expect("answer lock poisoned")
                    .take()
                    .expect("answer already consumed");
                answer(args)
            }),
            false,
        );
    }
    /// Forces exactly values.len() calls. Concurrent callers consume in answer-lock order.
    pub fn returns_sequence(self, values: impl IntoIterator<Item = R>)
    where
        R: Send,
    {
        let values: VecDeque<_> = values.into_iter().collect();
        let count = values.len();
        let values = Mutex::new(values);
        self.times(count).insert(
            Arc::new(move |_| {
                values
                    .lock()
                    .expect("answer lock poisoned")
                    .pop_front()
                    .expect("sequence exhausted")
            }),
            false,
        );
    }
}
/// Combined errors from multiple mocked methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationErrors(pub Vec<MockError>);
impl fmt::Display for VerificationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index != 0 {
                writeln!(f)?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}
impl std::error::Error for VerificationErrors {}
/// Shared verification interface for one method or a generated trait mock.
pub trait VerifyMocks: Send + Sync {
    fn verify_mocks(&self) -> Result<(), VerificationErrors>;
}
impl<A: fmt::Debug + 'static, R: 'static> VerifyMocks for Mock<A, R> {
    fn verify_mocks(&self) -> Result<(), VerificationErrors> {
        self.verify()
            .map_err(|error| VerificationErrors(vec![error]))
    }
}
/// Run a normal test body, then verify every supplied mock. On panic the original
/// panic propagates without running verification. Await/join workers inside the body.
#[track_caller]
pub fn with_mocks<R>(mocks: &[&dyn VerifyMocks], body: impl FnOnce() -> R) -> R {
    let result = body();
    let errors: Vec<_> = mocks
        .iter()
        .filter_map(|mock| mock.verify_mocks().err())
        .flat_map(|e| e.0)
        .collect();
    assert!(errors.is_empty(), "{}", VerificationErrors(errors));
    result
}

/// Verify after an async body completes, using the caller's executor.
/// Dropping the future before completion does not verify expectations.
pub async fn with_mocks_async<R>(
    mocks: &[&dyn VerifyMocks],
    body: impl std::future::Future<Output = R>,
) -> R {
    let result = body.await;
    let errors: Vec<_> = mocks
        .iter()
        .filter_map(|mock| mock.verify_mocks().err())
        .flat_map(|e| e.0)
        .collect();
    assert!(errors.is_empty(), "{}", VerificationErrors(errors));
    result
}
