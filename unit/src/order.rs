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

/// Stub for completion-order waits that [`CallSequence`] deliberately does not provide.
///
/// Admission order is covered by [`CallSequence`]. Waiting until async answers
/// *finish* belongs to the application (join handles, `.await`, barriers you own).
/// Methods here return a clear error / panic so call sites do not mistake this
/// for a working completion API.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompletionBarrier;

impl CompletionBarrier {
    /// Not implemented. Use application join/await for completion ordering.
    pub fn await_done(&self) -> Result<(), &'static str> {
        Err("not implemented: use application join/await")
    }

    /// Same gap as [`await_done`](Self::await_done), as a panic.
    #[track_caller]
    pub fn await_done_unchecked(&self) -> ! {
        panic!("not implemented: use application join/await")
    }
}
