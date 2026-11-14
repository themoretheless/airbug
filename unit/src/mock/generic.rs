//! Type-erased strict mock for trait methods with generic parameters.
//!
//! [`GenericMock`] stores independent per-`(Args, Return)` expectation lists keyed by
//! `(TypeId::of::<A>(), TypeId::of::<R>())`, enabling `#[airbug::mock]` to verify calls to
//! generic trait methods without requiring `Args: Debug`.

use crate::order::{CallSequence, SequenceStep};
use crate::verify::{MockError, VerificationErrors, VerifyMocks};
use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

type MatcherFn<A> = Arc<dyn Fn(&A) -> bool + Send + Sync>;
type AnswerFn<A, R> = Arc<dyn Fn(A) -> R + Send + Sync>;
type RecorderFn<A> = Arc<dyn Fn(&A) + Send + Sync>;
type AdmittedHandlers<A, R> = (AnswerFn<A, R>, Option<RecorderFn<A>>);

trait GenericSlot: Send + Sync {
    fn verify(&self, method: &str, failures: &mut Vec<String>);
    fn call_count(&self) -> usize;
    fn in_flight(&self) -> usize;
    fn reset_counts(&mut self);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct TypedExpectation<A, R> {
    label: String,
    matcher: MatcherFn<A>,
    answer: AnswerFn<A, R>,
    min: usize,
    max: usize,
    actual: usize,
    sequence: Option<SequenceStep>,
    capture: Option<RecorderFn<A>>,
}

impl<A, R> TypedExpectation<A, R> {
    fn count_failure(&self) -> Option<String> {
        if self.actual >= self.min && self.actual <= self.max {
            return None;
        }
        Some(if self.min == self.max {
            format!(
                "{} ({}) expected {} calls, actual {}",
                self.label,
                type_name::<A>(),
                self.min,
                self.actual
            )
        } else {
            format!(
                "{} ({}) expected {}..={} calls, actual {}",
                self.label,
                type_name::<A>(),
                self.min,
                self.max,
                self.actual
            )
        })
    }

    fn admit(&mut self) -> Result<AdmittedHandlers<A, R>, String> {
        if self.actual >= self.max {
            let detail = format!(
                "{} ({}) exceeded {} calls",
                self.label,
                type_name::<A>(),
                self.max
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
        Ok((Arc::clone(&self.answer), self.capture.clone()))
    }
}

struct TypedSlot<A, R> {
    expectations: Vec<TypedExpectation<A, R>>,
    failures: Vec<String>,
    calls: usize,
    in_flight: usize,
    started: bool,
}

impl<A, R> Default for TypedSlot<A, R> {
    fn default() -> Self {
        Self {
            expectations: Vec::new(),
            failures: Vec::new(),
            calls: 0,
            in_flight: 0,
            started: false,
        }
    }
}

impl<A: 'static, R: 'static> GenericSlot for TypedSlot<A, R> {
    fn verify(&self, _method: &str, failures: &mut Vec<String>) {
        failures.extend(self.failures.iter().cloned());
        for expectation in &self.expectations {
            if let Some(detail) = expectation.count_failure() {
                failures.push(detail);
            }
        }
    }

    fn call_count(&self) -> usize {
        self.calls
    }

    fn in_flight(&self) -> usize {
        self.in_flight
    }

    fn reset_counts(&mut self) {
        self.failures.clear();
        for exp in &mut self.expectations {
            exp.actual = 0;
        }
        self.calls = 0;
        self.started = false;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
struct GenericState {
    slots: HashMap<(TypeId, TypeId), Box<dyn GenericSlot>>,
    unexpected: Vec<String>,
    total_calls: usize,
}

/// Thread-safe strict mock supporting generic method signatures keyed by `(Args, Return)`.
///
/// Generated automatically for generic methods in `#[airbug::mock]` traits, or usable directly
/// in manual mock adapters. Clones share the same underlying state.
#[derive(Clone)]
pub struct GenericMock {
    name: Arc<str>,
    state: Arc<Mutex<GenericState>>,
}

impl GenericMock {
    /// Create a new generic method mock named `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Arc::from(name.into()),
            state: Arc::new(Mutex::new(GenericState::default())),
        }
    }

    /// Add an expectation for calls taking `A` and returning `R` that satisfy `matcher`.
    ///
    /// Rules default to expecting exactly one call and are evaluated in registration order.
    pub fn expect<A: 'static, R: 'static>(
        &self,
        label: impl Into<String>,
        matcher: impl Fn(&A) -> bool + Send + Sync + 'static,
    ) -> GenericExpectationBuilder<'_, A, R> {
        GenericExpectationBuilder {
            mock: self,
            label: label.into(),
            matcher: Arc::new(matcher),
            min: 1,
            max: 1,
            sequence: None,
            capture: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Execute one mocked call with concrete types `(A, R)` without panicking on mismatch.
    pub fn try_call<A: 'static, R: 'static>(&self, args: A) -> Result<R, MockError> {
        let key = (TypeId::of::<A>(), TypeId::of::<R>());
        let (answer, capture) = {
            let mut state = self.state.lock().expect("generic mock lock poisoned");
            state.total_calls += 1;
            let Some(slot_box) = state.slots.get_mut(&key) else {
                let detail = format!(
                    "unexpected generic call for {} -> {} (no expectations configured)",
                    type_name::<A>(),
                    type_name::<R>()
                );
                state.unexpected.push(detail.clone());
                return Err(MockError {
                    method: self.name.to_string(),
                    details: vec![detail],
                });
            };
            let slot = slot_box
                .as_any_mut()
                .downcast_mut::<TypedSlot<A, R>>()
                .expect("type key guarantees slot type");
            slot.started = true;
            slot.calls += 1;
            let mut match_idx = None;
            for (idx, exp) in slot.expectations.iter().enumerate() {
                if (exp.matcher)(&args) {
                    match_idx = Some(idx);
                    break;
                }
            }
            let Some(idx) = match_idx else {
                let detail = format!(
                    "unexpected generic call for {} -> {}",
                    type_name::<A>(),
                    type_name::<R>()
                );
                slot.failures.push(detail.clone());
                return Err(MockError {
                    method: self.name.to_string(),
                    details: vec![detail],
                });
            };
            match slot.expectations[idx].admit() {
                Ok(pair) => {
                    slot.in_flight += 1;
                    pair
                }
                Err(detail) => {
                    slot.failures.push(detail.clone());
                    return Err(MockError {
                        method: self.name.to_string(),
                        details: vec![detail],
                    });
                }
            }
        };

        struct InFlightGuard<'a, A: 'static, R: 'static>(
            &'a GenericMock,
            std::marker::PhantomData<(A, R)>,
        );
        impl<A: 'static, R: 'static> Drop for InFlightGuard<'_, A, R> {
            fn drop(&mut self) {
                let key = (TypeId::of::<A>(), TypeId::of::<R>());
                if let Ok(mut state) = self.0.state.lock()
                    && let Some(slot_box) = state.slots.get_mut(&key)
                    && let Some(slot) = slot_box.as_any_mut().downcast_mut::<TypedSlot<A, R>>()
                {
                    slot.in_flight = slot.in_flight.saturating_sub(1);
                }
            }
        }
        let _guard = InFlightGuard::<A, R>(self, std::marker::PhantomData);
        if let Some(cap) = capture {
            cap(&args);
        }
        Ok(answer(args))
    }

    /// Execute one mocked call with concrete types `(A, R)`, panicking on unexpected calls.
    #[track_caller]
    pub fn call<A: 'static, R: 'static>(&self, args: A) -> R {
        self.try_call(args).unwrap_or_else(|e| panic!("{e}"))
    }

    /// Return the total number of calls observed across all generic type instantiations.
    pub fn calls(&self) -> usize {
        self.state
            .lock()
            .expect("generic mock lock poisoned")
            .total_calls
    }

    /// Return the number of calls observed specifically for `(A, R)`.
    pub fn typed_calls<A: 'static, R: 'static>(&self) -> usize {
        let key = (TypeId::of::<A>(), TypeId::of::<R>());
        let state = self.state.lock().expect("generic mock lock poisoned");
        state.slots.get(&key).map_or(0, |slot| slot.call_count())
    }

    /// Verify all typed expectations and unexpected calls recorded on this mock.
    pub fn verify(&self) -> Result<(), MockError> {
        let state = self.state.lock().expect("generic mock lock poisoned");
        let mut details = state.unexpected.clone();
        for slot in state.slots.values() {
            slot.verify(&self.name, &mut details);
        }
        if details.is_empty() {
            Ok(())
        } else {
            Err(MockError {
                method: self.name.to_string(),
                details,
            })
        }
    }

    /// Reset recorded call counts across all configured type slots when no calls are in flight.
    pub fn reset_counts(&self) -> Result<(), MockError> {
        let mut state = self.state.lock().expect("generic mock lock poisoned");
        if state.slots.values().any(|s| s.in_flight() > 0) {
            return Err(MockError {
                method: self.name.to_string(),
                details: vec!["cannot reset generic mock counts while a call is in flight".into()],
            });
        }
        state.unexpected.clear();
        state.total_calls = 0;
        for slot in state.slots.values_mut() {
            slot.reset_counts();
        }
        Ok(())
    }

    /// Verify and panic with full diagnostic details if any expectation failed.
    #[track_caller]
    pub fn assert_verified(&self) {
        self.verify().unwrap_or_else(|e| panic!("{e}"));
    }
}

impl VerifyMocks for GenericMock {
    fn verify_mocks(&self) -> Result<(), VerificationErrors> {
        self.verify().map_err(|err| VerificationErrors(vec![err]))
    }
}

/// Builder configuring call counts, ordering, capture, and answers for one generic expectation.
pub struct GenericExpectationBuilder<'a, A: 'static, R: 'static> {
    mock: &'a GenericMock,
    label: String,
    matcher: MatcherFn<A>,
    min: usize,
    max: usize,
    sequence: Option<CallSequence>,
    capture: Option<RecorderFn<A>>,
    _marker: std::marker::PhantomData<fn() -> R>,
}

impl<A: 'static, R: 'static> GenericExpectationBuilder<'_, A, R> {
    /// Require exactly `count` matching calls.
    pub fn times(mut self, count: usize) -> Self {
        self.min = count;
        self.max = count;
        self
    }

    /// Require between `min` and `max` (inclusive) matching calls.
    pub fn times_between(mut self, min: usize, max: usize) -> Self {
        assert!(min <= max, "min ({min}) must not exceed max ({max})");
        self.min = min;
        self.max = max;
        self
    }

    /// Require at least `count` matching calls.
    pub fn at_least(mut self, count: usize) -> Self {
        self.min = count;
        self.max = usize::MAX;
        self
    }

    /// Allow at most `count` matching calls.
    pub fn at_most(mut self, count: usize) -> Self {
        self.min = 0;
        self.max = count;
        self
    }

    /// Forbid any matching calls (`times(0)`).
    pub fn never(self) -> Self {
        self.times(0)
    }

    /// Enroll this expectation in a global [`CallSequence`].
    pub fn in_sequence(mut self, sequence: &CallSequence) -> Self {
        assert!(
            self.sequence.is_none(),
            "an expectation can join only one sequence"
        );
        self.sequence = Some(sequence.clone());
        self
    }

    /// Capture each matching call's arguments into a [`Capture`](super::Capture).
    pub fn capture(mut self, capture: &super::Capture<A>) -> Self
    where
        A: Clone + Send + Sync,
    {
        let cap = capture.clone();
        self.capture = Some(Arc::new(move |args: &A| {
            cap.record(args);
        }));
        self
    }

    /// Register the expectation with a dynamic answer closure.
    pub fn returning(self, answer: impl Fn(A) -> R + Send + Sync + 'static) {
        self.finish(Arc::new(answer));
    }

    /// Alias for [`returning`](Self::returning) for readability on generic expectations.
    pub fn answers(self, answer: impl Fn(A) -> R + Send + Sync + 'static) {
        self.returning(answer);
    }

    /// Return a clone of `value` on every matching call.
    pub fn returns(self, value: R)
    where
        R: Clone + Send + Sync + 'static,
    {
        self.returning(move |_| value.clone());
    }

    /// Return `value` on a single matching call without requiring `R: Clone`.
    pub fn returning_once(self, answer: impl FnOnce(A) -> R + Send + 'static) {
        let cell = Mutex::new(Some(answer));
        self.returning(move |args| {
            let f = cell
                .lock()
                .expect("returning_once lock poisoned")
                .take()
                .expect("returning_once called more than once");
            f(args)
        });
    }

    /// Yield a sequence of values across consecutive matching calls.
    pub fn returns_sequence(self, values: impl IntoIterator<Item = R>)
    where
        R: Send + 'static,
    {
        let queue: Mutex<VecDeque<R>> = Mutex::new(values.into_iter().collect());
        self.returning(move |_| {
            queue
                .lock()
                .expect("returns_sequence lock poisoned")
                .pop_front()
                .expect("returns_sequence exhausted")
        });
    }

    fn finish(self, answer: AnswerFn<A, R>) {
        let key = (TypeId::of::<A>(), TypeId::of::<R>());
        let mut state = self.mock.state.lock().expect("generic mock lock poisoned");
        let name = self.mock.name.clone();
        let slot_box = state
            .slots
            .entry(key)
            .or_insert_with(|| Box::<TypedSlot<A, R>>::default());
        let slot = slot_box
            .as_any_mut()
            .downcast_mut::<TypedSlot<A, R>>()
            .expect("slot type matches key");
        assert!(
            !slot.started,
            "cannot configure expectations on {} after calls have started",
            name
        );
        let sequence = self.sequence.map(|seq| {
            let count = if self.min == self.max {
                self.min
            } else {
                panic!("sequenced generic expectations require a fixed call count (.times(n))");
            };
            seq.register(format!("{}::{}", name, self.label), count)
                .unwrap_or_else(|e| panic!("{e}"))
        });
        slot.expectations.push(TypedExpectation {
            label: self.label,
            matcher: self.matcher,
            answer,
            min: self.min,
            max: self.max,
            actual: 0,
            sequence,
            capture: self.capture,
        });
    }
}
