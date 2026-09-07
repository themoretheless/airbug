//! Additive assertions for ordinary #[test] functions.

/// Check collection membership. Each expression is evaluated exactly once.
#[macro_export]
macro_rules! assert_contains {
    ($collection:expr, $expected:expr $(,)?) => {{
        match (&$collection, &$expected) {
            (collection, expected) => assert!(
                collection.contains(expected),
                "expected {:?} to contain {:?}",
                collection,
                expected
            ),
        }
    }};
}

/// Compare slices ignoring order, but preserving duplicate counts. O(n²).
#[track_caller]
pub fn assert_same_items<T: PartialEq + std::fmt::Debug>(actual: &[T], expected: &[T]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "different collection lengths: actual {actual:?}, expected {expected:?}"
    );
    let mut matched = vec![false; expected.len()];
    for item in actual {
        let index = expected
            .iter()
            .enumerate()
            .position(|(index, value)| !matched[index] && item == value);
        match index {
            Some(index) => matched[index] = true,
            None => panic!(
                "different collection contents: actual {actual:?}, expected {expected:?}; unmatched {item:?}"
            ),
        }
    }
}

/// Compare collections without imposing an ordering or requiring Ord/Hash.
#[macro_export]
macro_rules! assert_same_items {
    ($actual:expr, $expected:expr $(,)?) => {{
        $crate::native::assert_same_items(&$actual, &$expected);
    }};
}

/// Evaluate every boolean check and report all false expressions together.
/// User panics propagate immediately; this does not catch assert! panics.
#[macro_export]
macro_rules! check_all {
    ($($check:expr);+ $(;)?) => {{
        let mut failures = ::std::vec::Vec::<&str>::new();
        $(if !$check { failures.push(stringify!($check)); })+
        assert!(failures.is_empty(), "failed checks:\n{}", failures.join("\n"));
    }};
}
