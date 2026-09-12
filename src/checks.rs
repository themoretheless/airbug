//! Native-style checks and structured failures, without fluent syntax.
use std::{error::Error, fmt::Debug};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckFailure {
    pub path: String,
    pub expected: String,
    pub actual: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckReport {
    pub failures: Vec<CheckFailure>,
}
impl std::fmt::Display for CheckReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for failure in &self.failures {
            writeln!(
                f,
                "{}: expected {}, actual {}",
                failure.path, failure.expected, failure.actual
            )?;
        }
        Ok(())
    }
}
impl CheckReport {
    pub fn is_success(&self) -> bool {
        self.failures.is_empty()
    }
    pub fn equal<T: PartialEq + Debug + ?Sized>(
        &mut self,
        path: impl Into<String>,
        actual: &T,
        expected: &T,
    ) -> &mut Self {
        if actual != expected {
            self.failures.push(CheckFailure {
                path: path.into(),
                expected: format!("{expected:?}"),
                actual: format!("{actual:?}"),
            });
        }
        self
    }
    pub fn field_equal<T, F: PartialEq + Debug + ?Sized>(
        &mut self,
        actual: &T,
        path: impl Into<String>,
        select: impl FnOnce(&T) -> &F,
        expected: &F,
    ) -> &mut Self {
        self.equal(path, select(actual), expected)
    }
    #[track_caller]
    pub fn assert(&self) {
        if crate::report::is_enabled() {
            for failure in &self.failures {
                crate::report::comparison(&failure.path, &failure.expected, &failure.actual, false);
            }
        }
        assert!(self.is_success(), "{self}");
    }
}
#[track_caller]
pub fn assert_unique<T: PartialEq + Debug>(values: &[T]) {
    for (i, value) in values.iter().enumerate() {
        if let Some(j) = values[..i].iter().position(|v| v == value) {
            panic!("duplicate {value:?} at indexes {j} and {i}");
        }
    }
}
#[track_caller]
pub fn assert_sorted<T: PartialOrd + Debug>(values: &[T]) {
    for (i, pair) in values.windows(2).enumerate() {
        assert!(
            pair[0] <= pair[1],
            "out of order or unordered at {i}: {:?} then {:?}",
            pair[0],
            pair[1]
        );
    }
}
#[track_caller]
pub fn assert_all<T: Debug>(values: &[T], predicate: impl Fn(&T) -> bool) {
    for (i, value) in values.iter().enumerate() {
        assert!(predicate(value), "predicate failed at [{i}]: {value:?}");
    }
}
#[track_caller]
pub fn assert_count<T: Debug>(values: &[T], expected: usize, predicate: impl Fn(&T) -> bool) {
    let count = values.iter().filter(|v| predicate(v)).count();
    assert_eq!(count, expected, "matching count for {values:?}");
}
/// Multiset subset: duplicate multiplicities matter, O(n²).
#[track_caller]
pub fn assert_subset<T: PartialEq + Debug>(subset: &[T], superset: &[T]) {
    let mut used = vec![false; superset.len()];
    for item in subset {
        match superset
            .iter()
            .enumerate()
            .position(|(i, value)| !used[i] && value == item)
        {
            Some(i) => used[i] = true,
            None => panic!("{subset:?} is not a multiset subset of {superset:?}; missing {item:?}"),
        }
    }
}
#[track_caller]
pub fn assert_superset<T: PartialEq + Debug>(superset: &[T], subset: &[T]) {
    assert_subset(subset, superset);
}
/// Relative error scaled by max(abs(actual), abs(expected)); finite values only.
#[track_caller]
pub fn assert_relative_eq(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        actual.is_finite() && expected.is_finite() && tolerance.is_finite() && tolerance >= 0.0,
        "relative comparison requires finite values and nonnegative tolerance"
    );
    let scale = actual.abs().max(expected.abs());
    assert!(
        scale == 0.0 || (actual / scale - expected / scale).abs() <= tolerance,
        "expected {actual:?} within relative tolerance {tolerance:?} of {expected:?}"
    );
}
pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
#[track_caller]
pub fn assert_text_eq(actual: &str, expected: &str) {
    assert_eq!(
        normalize_newlines(actual),
        normalize_newlines(expected),
        "text differs after newline normalization"
    );
}
/// Examine at most 64 links so a cyclic source chain cannot hang a test.
#[track_caller]
pub fn assert_error_chain_contains(error: &(dyn Error + 'static), expected: &str) {
    let mut current = Some(error);
    let mut chain = Vec::new();
    for _ in 0..64 {
        let Some(error) = current else {
            break;
        };
        let message = error.to_string();
        if message.contains(expected) {
            return;
        }
        chain.push(message);
        current = error.source();
    }
    panic!("expected error chain to contain {expected:?}; inspected {chain:?}");
}
/// Assert a string panic payload contains text; non-string payloads fail explicitly.
#[track_caller]
pub fn assert_panics(expected: &str, body: impl FnOnce() + std::panic::UnwindSafe) {
    match std::panic::catch_unwind(body) {
        Ok(()) => panic!("expected panic containing {expected:?}, body returned"),
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied());
            assert!(
                message.is_some_and(|message| message.contains(expected)),
                "expected panic containing {expected:?}, actual {message:?} (None means non-string payload)"
            );
        }
    }
}
