//! Native-style checks and structured failures, without fluent syntax.
use std::{
    error::Error,
    fmt::Debug,
    time::{Duration, Instant, SystemTime},
};
/// Soft assertion batch: accumulate mismatches, then [`CheckReport::assert`].
pub type SoftAssert = CheckReport;
/// One mismatch recorded by a [`CheckReport`], already rendered to strings so
/// the report can hold failures about differently typed fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckFailure {
    /// Where the mismatch is, as given to [`CheckReport::equal`], e.g. `order.total`.
    pub path: String,
    /// `Debug` rendering of the expected value.
    pub expected: String,
    /// `Debug` rendering of the actual value.
    pub actual: String,
}
/// Accumulates mismatches so one test run reports every wrong field at once,
/// instead of stopping at the first `assert_eq!`.
///
/// Record with [`equal`](CheckReport::equal) or
/// [`field_equal`](CheckReport::field_equal), then finish with
/// [`assert`](CheckReport::assert).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckReport {
    /// Recorded mismatches, in the order they were checked.
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
    /// True when nothing has been recorded as failing.
    pub fn is_success(&self) -> bool {
        self.failures.is_empty()
    }
    /// Record a mismatch at `path` unless the two values are equal.
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
    /// [`equal`](CheckReport::equal) on a field picked out of `actual`, so a
    /// chain of checks over one struct does not repeat the binding.
    pub fn field_equal<T, F: PartialEq + Debug + ?Sized>(
        &mut self,
        actual: &T,
        path: impl Into<String>,
        select: impl FnOnce(&T) -> &F,
        expected: &F,
    ) -> &mut Self {
        self.equal(path, select(actual), expected)
    }
    /// Panic listing every recorded mismatch, or return if there are none.
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
/// Panic on the first value that repeats an earlier one. O(n²).
#[track_caller]
pub fn assert_unique<T: PartialEq + Debug>(values: &[T]) {
    for (i, value) in values.iter().enumerate() {
        if let Some(j) = values[..i].iter().position(|v| v == value) {
            panic!("duplicate {value:?} at indexes {j} and {i}");
        }
    }
}
/// Panic at the first adjacent pair that is out of ascending order. Values that
/// do not compare at all (such as NaN) also fail.
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
/// Panic at the first element failing `predicate`, naming its index.
#[track_caller]
pub fn assert_all<T: Debug>(values: &[T], predicate: impl Fn(&T) -> bool) {
    for (i, value) in values.iter().enumerate() {
        assert!(predicate(value), "predicate failed at [{i}]: {value:?}");
    }
}
/// Panic unless exactly `expected` elements satisfy `predicate`.
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
/// Multiset superset: the arguments of [`assert_subset`] the other way round.
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
/// Convert CRLF and lone CR to LF, so text compares the same on every platform.
pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
/// Compare two strings after [`normalize_newlines`], so a checkout that
/// rewrote line endings does not fail the test.
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
/// Panic unless `|actual - expected| <= tolerance`.
#[track_caller]
pub fn assert_duration_eq(actual: Duration, expected: Duration, tolerance: Duration) {
    let diff = actual.abs_diff(expected);
    assert!(
        diff <= tolerance,
        "expected duration {actual:?} within {tolerance:?} of {expected:?} (diff {diff:?})"
    );
}
/// Panic unless two [`SystemTime`] values differ by at most `tolerance`.
#[track_caller]
pub fn assert_near(actual: SystemTime, expected: SystemTime, tolerance: Duration) {
    let diff = match actual.duration_since(expected) {
        Ok(d) => d,
        Err(e) => e.duration(),
    };
    assert!(
        diff <= tolerance,
        "expected system time {actual:?} within {tolerance:?} of {expected:?} (diff {diff:?})"
    );
}
/// Panic unless two [`Instant`] values differ by at most `tolerance`.
#[track_caller]
pub fn assert_instant_near(actual: Instant, expected: Instant, tolerance: Duration) {
    let diff = if actual >= expected {
        actual.duration_since(expected)
    } else {
        expected.duration_since(actual)
    };
    assert!(
        diff <= tolerance,
        "expected instant within {tolerance:?} (diff {diff:?})"
    );
}
/// Assert that a dotted JSON path (e.g. `a.b.0`) equals `expected`.
#[cfg(feature = "json")]
#[track_caller]
pub fn assert_json_path(
    value: &serde_json::Value,
    path: &str,
    expected: &serde_json::Value,
) {
    let mut current = value;
    for segment in path.split('.') {
        assert!(
            !segment.is_empty(),
            "json path {path:?} has an empty segment"
        );
        current = if let Ok(index) = segment.parse::<usize>() {
            current.get(index).unwrap_or_else(|| {
                panic!("json path {path:?}: missing array index {index} at {current}")
            })
        } else {
            current.get(segment).unwrap_or_else(|| {
                panic!("json path {path:?}: missing field {segment:?} at {current}")
            })
        };
    }
    assert_eq!(
        current, expected,
        "json path {path:?}: expected {expected}, actual {current}"
    );
}
