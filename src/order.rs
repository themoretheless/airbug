//! Shared, strict admission order across mock methods and objects.
use crate::mock::{MockError, VerificationErrors, VerifyMocks};
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
/// Ordering is about admission, not answer completion. Verify mocks as well.
#[derive(Clone)]
pub struct CallSequence {
    name: Arc<str>,
    state: Arc<Mutex<State>>,
}
impl CallSequence {
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
