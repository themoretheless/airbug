//! Configuring what a mock accepts and what it answers.
use super::{Answer, Capture, Expectation, Matcher, Mock, Recorder};
use crate::order::CallSequence;
use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex},
};

/// A rule being configured on a [`Mock`]: how many calls it accepts, whether it
/// captures arguments, whether it belongs to an ordered sequence.
///
/// Nothing is registered until a terminal method
/// ([`returns`](ExpectationBuilder::returns),
/// [`returning`](ExpectationBuilder::returning),
/// [`returning_once`](ExpectationBuilder::returning_once) or
/// [`returns_sequence`](ExpectationBuilder::returns_sequence)) supplies the answer.
#[must_use = "finish with returns, returning, returning_once or returns_sequence"]
pub struct ExpectationBuilder<A, R> {
    pub(super) mock: Mock<A, R>,
    pub(super) label: String,
    pub(super) matcher: Matcher<A>,
    pub(super) min: usize,
    pub(super) max: usize,
    pub(super) capture: Option<Recorder<A>>,
    pub(super) sequence: Option<CallSequence>,
}
impl<A: fmt::Debug + 'static, R: 'static> ExpectationBuilder<A, R> {
    /// Require exactly `count` calls.
    pub fn times(self, count: usize) -> Self {
        self.times_between(count, count)
    }
    /// Require between `min` and `max` calls, inclusive. Panics if `min > max`.
    pub fn times_between(mut self, min: usize, max: usize) -> Self {
        assert!(min <= max, "min must be <= max");
        self.min = min;
        self.max = max;
        self
    }
    /// Require at least `count` calls, with no upper bound.
    pub fn at_least(self, count: usize) -> Self {
        self.times_between(count, usize::MAX)
    }
    /// Allow up to `count` calls, including none at all.
    pub fn at_most(self, count: usize) -> Self {
        self.times_between(0, count)
    }
    /// Record the arguments of every accepted call into `capture`. Cloning
    /// happens after the mock's lock is released.
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
    /// Answer each call by running `answer` on its arguments.
    pub fn returning(self, answer: impl Fn(A) -> R + Send + Sync + 'static) {
        self.insert(Arc::new(answer), true);
    }
    /// Answer every call with a clone of `value`.
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
