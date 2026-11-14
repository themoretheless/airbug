//! Shared, strict admission order across mock methods and objects.
use crate::verify::{MockError, VerificationErrors, VerifyMocks};
use std::sync::{Arc, Mutex};

struct Entry {
    label: String,
    required: usize,
    admitted: usize,
}
struct State {
    entries: Vec<Entry>,
    position: usize,
    started: bool,
    failures: Vec<String>,
}
/// Register expectations in desired order with `.in_sequence(&sequence)`.
///
/// **Admission ≠ completion.** [`CallSequence`] only orders when a mock *admits*
/// a call into its answer (count + step advance). An answer may still be running
/// (async task, thread join, etc.) when the next step is admitted. There is no
/// `await_done` on this type: wait for operation completion in the application
/// under test. See [`CompletionBarrier`] for an explicit stub of that gap.
///
/// Verify mocks as well as the sequence.
#[derive(Clone)]
pub struct CallSequence {
    name: Arc<str>,
    state: Arc<Mutex<State>>,
}
impl CallSequence {
    /// An empty sequence; `name` appears in its failure messages.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Arc::from(name.into()),
            state: Arc::new(Mutex::new(State {
                entries: Vec::new(),
                position: 0,
                started: false,
                failures: Vec::new(),
            })),
        }
    }
    pub(crate) fn register(&self, label: String, count: usize) -> Result<SequenceStep, MockError> {
        let mut state = self.state.lock().expect("sequence lock poisoned");
        if state.started {
            return Err(self.error(vec!["configure sequence before its first mock call".into()]));
        }
        let index = state.entries.len();
        state.entries.push(Entry {
            label,
            required: count,
            admitted: 0,
        });
        Ok(SequenceStep {
            sequence: self.clone(),
            index,
        })
    }
    fn error(&self, details: Vec<String>) -> MockError {
        MockError {
            method: format!("sequence {}", self.name),
            details,
        }
    }
    /// Includes missing steps and retained order violations. No reset or Drop verification.
    /// Report out-of-order admissions plus any step that never ran enough times.
    pub fn verify(&self) -> Result<(), MockError> {
        let state = self.state.lock().expect("sequence lock poisoned");
        let mut details = state.failures.clone();
        details.extend(
            state
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.admitted != entry.required)
                .map(|(i, entry)| {
                    format!(
                        "step {} {} expected {} calls, actual {}",
                        i + 1,
                        entry.label,
                        entry.required,
                        entry.admitted
                    )
                }),
        );
        if details.is_empty() {
            Ok(())
        } else {
            Err(self.error(details))
        }
    }
    /// [`verify`](CallSequence::verify), panicking on failure.
    #[track_caller]
    pub fn assert_verified(&self) {
        self.verify().unwrap_or_else(|error| panic!("{error}"));
    }
}
impl VerifyMocks for CallSequence {
    fn verify_mocks(&self) -> Result<(), VerificationErrors> {
        self.verify()
            .map_err(|error| VerificationErrors(vec![error]))
    }
}
#[derive(Clone)]
pub(crate) struct SequenceStep {
    sequence: CallSequence,
    index: usize,
}
impl SequenceStep {
    pub(crate) fn seal(&self) {
        self.sequence
            .state
            .lock()
            .expect("sequence lock poisoned")
            .started = true;
    }
    /// Called while the method's count lock is held. Never runs user code.
    pub(crate) fn admit(&self) -> Result<(), String> {
        let mut state = self.sequence.state.lock().expect("sequence lock poisoned");
        state.started = true;
        if state.position != self.index {
            let expected = match state.entries.get(state.position) {
                Some(entry) => format!(
                    "step {} {} ({}/{})",
                    state.position + 1,
                    entry.label,
                    entry.admitted,
                    entry.required
                ),
                None => "sequence already completed".into(),
            };
            let received = &state.entries[self.index].label;
            let message = format!(
                "sequence {} out of order: expected {expected}; received step {} {received}",
                self.sequence.name,
                self.index + 1
            );
            state.failures.push(message.clone());
            return Err(message);
        }
        let entry = &mut state.entries[self.index];
        entry.admitted += 1;
        if entry.admitted == entry.required {
            state.position += 1;
        }
        Ok(())
    }
    pub(crate) fn record_excess(&self, detail: &str) {
        self.sequence
            .state
            .lock()
            .expect("sequence lock poisoned")
            .failures
            .push(detail.into());
    }
}

/// Partial-order admissions: edges require `before` to be admitted earlier than `after`.
#[derive(Clone, Debug, Default)]
pub struct CallDag {
    edges: Vec<(String, String)>,
    admitted: Vec<String>,
}

impl CallDag {
    /// Empty DAG with no edges or admissions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require `before` to be admitted earlier than `after`.
    pub fn edge(mut self, before: impl Into<String>, after: impl Into<String>) -> Self {
        self.edges.push((before.into(), after.into()));
        self
    }

    /// Record that `name` ran, in chronological order.
    pub fn admit(&mut self, name: impl Into<String>) {
        self.admitted.push(name.into());
    }

    /// Whether `a` was admitted strictly earlier than `b` in this DAG.
    pub fn happened_before(&self, a: &str, b: &str) -> bool {
        let ai = self.admitted.iter().position(|n| n == a);
        let bi = self.admitted.iter().position(|n| n == b);
        matches!((ai, bi), (Some(ai), Some(bi)) if ai < bi)
    }

    /// Check that every admitted node respects all edges.
    pub fn verify(&self) -> Result<(), String> {
        for (index, name) in self.admitted.iter().enumerate() {
            for (before, after) in &self.edges {
                if after != name {
                    continue;
                }
                let ok = self.admitted[..index].iter().any(|n| n == before);
                if !ok {
                    return Err(format!(
                        "happens-before violated: `{before}` must precede `{after}`"
                    ));
                }
            }
        }
        Ok(())
    }

    /// [`verify`](CallDag::verify), panicking on failure.
    #[track_caller]
    pub fn assert(&self) {
        self.verify().unwrap_or_else(|error| panic!("{error}"));
    }
}

/// True when the earliest `seq` for `a` is strictly less than the earliest for `b`.
pub fn happened_before(a: &str, b: &str, events: &[(&str, u64)]) -> bool {
    let seq_a = events
        .iter()
        .filter(|(name, _)| *name == a)
        .map(|(_, seq)| *seq)
        .min();
    let seq_b = events
        .iter()
        .filter(|(name, _)| *name == b)
        .map(|(_, seq)| *seq)
        .min();
    match (seq_a, seq_b) {
        (Some(sa), Some(sb)) => sa < sb,
        _ => false,
    }
}

/// Thread-safe completion barrier for coordinating and verifying when concurrent or `async`
/// operations finish.
///
/// While [`CallSequence`] checks the order in which calls are *admitted*, `CompletionBarrier`
/// tracks actual task completions across threads or async executors: issue guards with
/// [`enter`](Self::enter), record direct completions with [`complete`](Self::complete), await
/// completion asynchronously with [`await_done`](Self::await_done), or block with
/// [`wait_timeout`](Self::wait_timeout). Implements [`VerifyMocks`].
#[derive(Debug, Clone)]
pub struct CompletionBarrier {
    inner: Arc<CompletionInner>,
}

#[derive(Debug)]
struct CompletionInner {
    name: Arc<str>,
    state: Mutex<BarrierState>,
    cv: std::sync::Condvar,
}

#[derive(Debug, Default)]
struct BarrierState {
    expected: Option<usize>,
    expected_order: Option<Vec<String>>,
    completed: Vec<String>,
    in_flight: usize,
    wakers: Vec<std::task::Waker>,
}

impl BarrierState {
    fn is_done(&self) -> bool {
        if self.in_flight > 0 {
            return false;
        }
        match self.expected {
            Some(n) => self.completed.len() >= n,
            None => true,
        }
    }

    fn verify(&self, name: &str) -> Result<(), MockError> {
        let mut details = Vec::new();
        if self.in_flight > 0 {
            details.push(format!("{} task(s) still in flight", self.in_flight));
        }
        if let Some(expected) = self.expected
            && self.completed.len() != expected
        {
            details.push(format!(
                "expected {expected} completions, got {} ({:?})",
                self.completed.len(),
                self.completed
            ));
        }
        if let Some(expected_order) = &self.expected_order
            && &self.completed != expected_order
        {
            details.push(format!(
                "completion order mismatch: expected {expected_order:?}, got {:?}",
                self.completed
            ));
        }
        if details.is_empty() {
            Ok(())
        } else {
            Err(MockError {
                method: format!("barrier:{name}"),
                details,
            })
        }
    }
}

impl Default for CompletionBarrier {
    fn default() -> Self {
        Self {
            inner: Arc::new(CompletionInner {
                name: Arc::from("default"),
                state: Mutex::new(BarrierState::default()),
                cv: std::sync::Condvar::new(),
            }),
        }
    }
}

impl CompletionBarrier {
    /// Create a barrier expecting `expected` completion events.
    pub fn new(expected: usize) -> Self {
        Self::named("barrier", expected)
    }

    /// Create a named barrier expecting `expected` completion events.
    pub fn named(name: impl Into<String>, expected: usize) -> Self {
        Self {
            inner: Arc::new(CompletionInner {
                name: Arc::from(name.into()),
                state: Mutex::new(BarrierState {
                    expected: Some(expected),
                    ..BarrierState::default()
                }),
                cv: std::sync::Condvar::new(),
            }),
        }
    }

    /// Create a barrier that waits until all issued [`CompletionToken`] guards finish.
    pub fn unbounded(name: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(CompletionInner {
                name: Arc::from(name.into()),
                state: Mutex::new(BarrierState::default()),
                cv: std::sync::Condvar::new(),
            }),
        }
    }

    /// Require completions to arrive in the specified exact label sequence.
    pub fn expect_order<I, S>(self, order: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let order: Vec<String> = order.into_iter().map(Into::into).collect();
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.expected.is_none() {
            state.expected = Some(order.len());
        }
        state.expected_order = Some(order);
        drop(state);
        self
    }

    /// Enter an in-flight unit of work labeled `label`.
    ///
    /// Completing explicitly via [`CompletionToken::complete`] or dropping the returned token
    /// records completion and wakes all waiting threads and futures.
    pub fn enter(&self, label: impl Into<String>) -> CompletionToken {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.in_flight += 1;
        CompletionToken {
            inner: Arc::clone(&self.inner),
            label: label.into(),
            finished: false,
        }
    }

    /// Record a completion event for `label` directly and notify all waiters.
    pub fn complete(&self, label: impl Into<String>) {
        let wakers = {
            let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
            state.completed.push(label.into());
            self.inner.cv.notify_all();
            if state.is_done() {
                std::mem::take(&mut state.wakers)
            } else {
                Vec::new()
            }
        };
        for waker in wakers {
            waker.wake();
        }
    }

    /// Return the completion labels recorded so far, in arrival order.
    pub fn completed(&self) -> Vec<String> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .completed
            .clone()
    }

    /// Number of completions recorded so far.
    pub fn completed_count(&self) -> usize {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .completed
            .len()
    }

    /// Return `true` when zero tasks are in flight and any expected count is satisfied.
    pub fn is_done(&self) -> bool {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_done()
    }

    /// Verify completion count and order immediately without blocking.
    pub fn verify(&self) -> Result<(), MockError> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.verify(&self.inner.name)
    }

    /// Block the current thread until the barrier is done or `timeout` elapses, then verify.
    pub fn wait_timeout(&self, timeout: std::time::Duration) -> Result<(), MockError> {
        let deadline = std::time::Instant::now() + timeout;
        let mut guard = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        while !guard.is_done() {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                let mut details = vec![format!(
                    "timed out after {timeout:?} waiting for completion ({} completed, {} in flight)",
                    guard.completed.len(),
                    guard.in_flight
                )];
                if let Err(err) = guard.verify(&self.inner.name) {
                    details.extend(err.details);
                }
                return Err(MockError {
                    method: format!("barrier:{}", self.inner.name),
                    details,
                });
            }
            let (next_guard, wait_res) = self
                .inner
                .cv
                .wait_timeout(guard, remaining)
                .unwrap_or_else(|e| e.into_inner());
            guard = next_guard;
            if wait_res.timed_out() && !guard.is_done() {
                let mut details = vec![format!(
                    "timed out after {timeout:?} waiting for completion ({} completed, {} in flight)",
                    guard.completed.len(),
                    guard.in_flight
                )];
                if let Err(err) = guard.verify(&self.inner.name) {
                    details.extend(err.details);
                }
                return Err(MockError {
                    method: format!("barrier:{}", self.inner.name),
                    details,
                });
            }
        }
        guard.verify(&self.inner.name)
    }

    /// Block the current thread indefinitely until the barrier completes, then verify.
    pub fn wait(&self) -> Result<(), MockError> {
        let guard = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let guard = self
            .inner
            .cv
            .wait_while(guard, |state| !state.is_done())
            .unwrap_or_else(|e| e.into_inner());
        guard.verify(&self.inner.name)
    }

    /// Return a [`std::future::Future`] that resolves once all expected and in-flight
    /// tasks complete, and verifies any configured completion count and order.
    pub fn await_done(&self) -> CompletionDoneFuture {
        CompletionDoneFuture {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl VerifyMocks for CompletionBarrier {
    fn verify_mocks(&self) -> Result<(), VerificationErrors> {
        self.verify().map_err(|e| VerificationErrors(vec![e]))
    }
}

/// Guard returned by [`CompletionBarrier::enter`] that records completion when finished or dropped.
#[derive(Debug)]
pub struct CompletionToken {
    inner: Arc<CompletionInner>,
    label: String,
    finished: bool,
}

impl CompletionToken {
    /// Complete this token explicitly, waking all waiting threads and async futures.
    pub fn complete(mut self) {
        self.finish(true);
    }

    /// Cancel this token without recording a completion label, decrementing in-flight count.
    pub fn cancel(mut self) {
        self.finish(false);
    }

    fn finish(&mut self, record: bool) {
        if self.finished {
            return;
        }
        self.finished = true;
        let wakers = {
            let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
            state.in_flight = state.in_flight.saturating_sub(1);
            if record {
                state.completed.push(std::mem::take(&mut self.label));
            }
            self.inner.cv.notify_all();
            if state.is_done() {
                std::mem::take(&mut state.wakers)
            } else {
                Vec::new()
            }
        };
        for waker in wakers {
            waker.wake();
        }
    }
}

impl Drop for CompletionToken {
    fn drop(&mut self) {
        self.finish(true);
    }
}

/// Future returned by [`CompletionBarrier::await_done`].
#[derive(Debug)]
pub struct CompletionDoneFuture {
    inner: Arc<CompletionInner>,
}

impl std::future::Future for CompletionDoneFuture {
    type Output = Result<(), MockError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.is_done() {
            return std::task::Poll::Ready(state.verify(&self.inner.name));
        }
        let waker = cx.waker();
        if !state
            .wakers
            .iter()
            .any(|existing| existing.will_wake(waker))
        {
            state.wakers.push(waker.clone());
        }
        std::task::Poll::Pending
    }
}
