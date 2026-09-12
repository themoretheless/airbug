use std::fmt::Debug;

/// Borrows the subject so assertions never consume test data.
pub fn assert_that<T: ?Sized>(actual: &T) -> Assertion<'_, T> {
    Assertion {
        actual,
        reason: None,
    }
}
pub struct Assertion<'a, T: ?Sized> {
    actual: &'a T,
    reason: Option<&'a str>,
}
impl<'a, T: ?Sized> Assertion<'a, T> {
    pub fn because(mut self, reason: &'a str) -> Self {
        self.reason = Some(reason);
        self
    }
    #[track_caller]
    fn check(&self, passed: bool, expectation: impl std::fmt::Display) {
        assert!(
            passed,
            "{expectation}{}",
            self.reason
                .map(|r| format!(" because {r}"))
                .unwrap_or_default()
        );
    }
}
impl<T: Debug + PartialEq + ?Sized> Assertion<'_, T> {
    #[track_caller]
    pub fn is_equal_to(self, expected: &T) -> Self {
        let passed = self.actual == expected;
        if !passed && crate::report::is_enabled() {
            crate::report::comparison(
                self.reason.unwrap_or("is_equal_to"),
                &format!("{expected:#?}"),
                &format!("{:#?}", self.actual),
                passed,
            );
        }
        self.check(
            passed,
            format_args!("expected {:?} to equal {:?}", self.actual, expected),
        );
        self
    }
    #[track_caller]
    pub fn is_not_equal_to(self, expected: &T) -> Self {
        self.check(
            self.actual != expected,
            format_args!("expected {:?} not to equal {:?}", self.actual, expected),
        );
        self
    }
}
impl<T: Debug + PartialOrd> Assertion<'_, T> {
    #[track_caller]
    pub fn is_greater_than(self, expected: &T) -> Self {
        self.check(
            self.actual > expected,
            format_args!(
                "expected {:?} to be greater than {:?}",
                self.actual, expected
            ),
        );
        self
    }
}
impl Assertion<'_, bool> {
    #[track_caller]
    pub fn is_true(self) -> Self {
        self.check(*self.actual, "expected true, actual false");
        self
    }
    #[track_caller]
    pub fn is_false(self) -> Self {
        self.check(!*self.actual, "expected false, actual true");
        self
    }
}
impl<T: AsRef<str> + ?Sized> Assertion<'_, T> {
    #[track_caller]
    pub fn contains_text(self, expected: &str) -> Self {
        self.check(
            self.actual.as_ref().contains(expected),
            format_args!(
                "expected {:?} to contain {:?}",
                self.actual.as_ref(),
                expected
            ),
        );
        self
    }
}
impl<T: Debug> Assertion<'_, Option<T>> {
    #[track_caller]
    pub fn is_some(self) -> Self {
        self.check(self.actual.is_some(), "expected Some, actual None");
        self
    }
    #[track_caller]
    pub fn is_none(self) -> Self {
        self.check(
            self.actual.is_none(),
            format_args!("expected None, actual {:?}", self.actual),
        );
        self
    }
}
impl<T: Debug, E: Debug> Assertion<'_, Result<T, E>> {
    #[track_caller]
    pub fn is_ok(self) -> Self {
        self.check(
            self.actual.is_ok(),
            format_args!("expected Ok, actual {:?}", self.actual),
        );
        self
    }
    #[track_caller]
    pub fn is_err(self) -> Self {
        self.check(
            self.actual.is_err(),
            format_args!("expected Err, actual {:?}", self.actual),
        );
        self
    }
}
impl<T: Debug + PartialEq> Assertion<'_, [T]> {
    #[track_caller]
    pub fn contains(self, expected: &T) -> Self {
        self.check(
            self.actual.contains(expected),
            format_args!("expected {:?} to contain {:?}", self.actual, expected),
        );
        self
    }
    #[track_caller]
    pub fn has_length(self, expected: usize) -> Self {
        self.check(
            self.actual.len() == expected,
            format_args!(
                "expected length {expected}, actual {} for {:?}",
                self.actual.len(),
                self.actual
            ),
        );
        self
    }
    #[track_caller]
    pub fn is_empty(self) -> Self {
        self.has_length(0)
    }
}

impl<T: Debug + PartialOrd> Assertion<'_, T> {
    #[track_caller]
    pub fn is_less_than(self, expected: &T) -> Self {
        self.check(
            self.actual < expected,
            format_args!("expected {:?} to be less than {:?}", self.actual, expected),
        );
        self
    }
    #[track_caller]
    pub fn is_between(self, min: &T, max: &T) -> Self {
        self.check(
            self.actual >= min && self.actual <= max,
            format_args!(
                "expected {:?} in inclusive range {:?}..={:?}",
                self.actual, min, max
            ),
        );
        self
    }
}
impl<T: AsRef<str> + ?Sized> Assertion<'_, T> {
    #[track_caller]
    pub fn starts_with(self, prefix: &str) -> Self {
        self.check(
            self.actual.as_ref().starts_with(prefix),
            format_args!(
                "expected {:?} to start with {:?}",
                self.actual.as_ref(),
                prefix
            ),
        );
        self
    }
    #[track_caller]
    pub fn ends_with(self, suffix: &str) -> Self {
        self.check(
            self.actual.as_ref().ends_with(suffix),
            format_args!(
                "expected {:?} to end with {:?}",
                self.actual.as_ref(),
                suffix
            ),
        );
        self
    }
}
impl<'a, T: Debug> Assertion<'a, Option<T>> {
    #[track_caller]
    pub fn value(self) -> Assertion<'a, T> {
        let checked = self.is_some();
        Assertion {
            actual: checked.actual.as_ref().expect("checked Some"),
            reason: checked.reason,
        }
    }
}

impl<'a, T: Debug, E: Debug> Assertion<'a, Result<T, E>> {
    #[track_caller]
    pub fn value(self) -> Assertion<'a, T> {
        let checked = self.is_ok();
        Assertion {
            actual: checked.actual.as_ref().expect("checked Ok"),
            reason: checked.reason,
        }
    }
}
impl<T: Debug + PartialEq> Assertion<'_, Vec<T>> {
    #[track_caller]
    pub fn contains(self, expected: &T) -> Self {
        Assertion {
            actual: self.actual.as_slice(),
            reason: self.reason,
        }
        .contains(expected);
        self
    }
    #[track_caller]
    pub fn has_length(self, expected: usize) -> Self {
        Assertion {
            actual: self.actual.as_slice(),
            reason: self.reason,
        }
        .has_length(expected);
        self
    }
    #[track_caller]
    pub fn is_empty(self) -> Self {
        self.has_length(0)
    }
}
impl Assertion<'_, f64> {
    /// Absolute tolerance; both values and tolerance must be finite, tolerance nonnegative.
    #[track_caller]
    pub fn is_close_to(self, expected: f64, tolerance: f64) -> Self {
        self.check(
            self.actual.is_finite()
                && expected.is_finite()
                && tolerance.is_finite()
                && tolerance >= 0.0
                && (*self.actual - expected).abs() <= tolerance,
            format_args!(
                "expected {:?} within {:?} of {:?} (finite values, nonnegative tolerance)",
                self.actual, tolerance, expected
            ),
        );
        self
    }
}
