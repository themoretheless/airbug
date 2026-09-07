//! Reusable validators that collect field errors without panicking.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors(pub Vec<ValidationError>);
impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}: {}", error.field, error.message)?;
        }
        Ok(())
    }
}
impl std::error::Error for ValidationErrors {}

struct ErrorSink {
    errors: Vec<ValidationError>,
    limit: usize,
}
impl ErrorSink {
    fn full(&self) -> bool {
        self.errors.len() >= self.limit
    }
    fn remaining(&self) -> usize {
        self.limit - self.errors.len()
    }
    fn push(&mut self, error: ValidationError) {
        if !self.full() {
            self.errors.push(error);
        }
    }
    fn extend(&mut self, errors: impl IntoIterator<Item = ValidationError>) {
        for error in errors {
            if self.full() {
                break;
            }
            self.push(error);
        }
    }
}
type Rule<T> = Box<dyn Fn(&T, &mut ErrorSink) + Send + Sync>;

/// All rules run, in declaration order. Predicates should be side-effect free.
pub struct Validator<T> {
    rules: Vec<Rule<T>>,
    max_errors: usize,
}
impl<T: 'static> Default for Validator<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: 'static> Validator<T> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            max_errors: usize::MAX,
        }
    }
    pub fn rule_for<F: ?Sized + 'static>(
        self,
        field: impl Into<String>,
        select: impl Fn(&T) -> &F + Send + Sync + 'static,
    ) -> FieldRule<T, F> {
        FieldRule {
            validator: self,
            field: field.into(),
            select: Box::new(select),
            checks: Vec::new(),
            condition: None,
            stop_on_failure: false,
        }
    }
    pub fn validate(&self, value: &T) -> Result<(), ValidationErrors> {
        self.validate_limit(value, self.max_errors)
    }
    fn validate_limit(&self, value: &T, limit: usize) -> Result<(), ValidationErrors> {
        let mut errors = ErrorSink {
            errors: Vec::new(),
            limit: limit.min(self.max_errors),
        };
        for rule in &self.rules {
            if errors.full() {
                break;
            }
            rule(value, &mut errors);
        }
        if errors.errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors(errors.errors))
        }
    }
    /// Positive error budget, shared across fields and children. Later rules are skipped.
    pub fn max_errors(mut self, limit: usize) -> Self {
        assert!(limit > 0, "error limit must be positive");
        self.max_errors = limit;
        self
    }
    pub fn stop_on_first_failure(self) -> Self {
        self.max_errors(1)
    }
    /// Prefixes child errors with the selected field's path.
    pub fn child<U: 'static>(
        mut self,
        field: impl Into<String>,
        select: impl Fn(&T) -> &U + Send + Sync + 'static,
        validator: Validator<U>,
    ) -> Self {
        let field = field.into();
        self.rules.push(Box::new(move |value, errors| {
            if let Err(child_errors) = validator.validate_limit(select(value), errors.remaining()) {
                errors.extend(child_errors.0.into_iter().map(|error| ValidationError {
                    field: format!("{field}.{}", error.field),
                    message: error.message,
                    code: error.code,
                }));
            }
        }));
        self
    }
}
type Condition<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;
struct Check<F: ?Sized> {
    predicate: Box<dyn Fn(&F) -> bool + Send + Sync>,
    message: String,
    code: String,
}
/// Call `done` to attach this field's checks to the validator.
#[must_use = "call done() to attach the field rules"]
pub struct FieldRule<T, F: ?Sized> {
    validator: Validator<T>,
    field: String,
    select: Box<dyn Fn(&T) -> &F + Send + Sync>,
    checks: Vec<Check<F>>,
    condition: Option<Condition<T>>,
    stop_on_failure: bool,
}
impl<T: 'static, F: ?Sized + 'static> FieldRule<T, F> {
    pub fn must(
        mut self,
        predicate: impl Fn(&F) -> bool + Send + Sync + 'static,
        message: impl Into<String>,
    ) -> Self {
        self.checks.push(Check {
            predicate: Box::new(predicate),
            message: message.into(),
            code: "custom".into(),
        });
        self
    }
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.checks
            .last_mut()
            .expect("with_code requires a preceding check")
            .code = code.into();
        self
    }
    /// Replaces the last check's message. Panics if no check has been added.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.checks
            .last_mut()
            .expect("with_message requires a preceding check")
            .message = message.into();
        self
    }
    /// Applies to all checks on this field. Replaces any previous condition.
    pub fn when(mut self, condition: impl Fn(&T) -> bool + Send + Sync + 'static) -> Self {
        self.condition = Some(Box::new(condition));
        self
    }
    pub fn stop_on_first_failure(mut self) -> Self {
        self.stop_on_failure = true;
        self
    }
    pub fn done(mut self) -> Validator<T> {
        self.validator.rules.push(Box::new(move |value, errors| {
            if self
                .condition
                .as_ref()
                .is_some_and(|condition| !condition(value))
            {
                return;
            }
            let field_value = (self.select)(value);
            for check in &self.checks {
                if errors.full() {
                    break;
                }
                if !(check.predicate)(field_value) {
                    errors.push(ValidationError {
                        field: self.field.clone(),
                        message: check.message.clone(),
                        code: check.code.clone(),
                    });
                    if self.stop_on_failure {
                        break;
                    }
                }
            }
        }));
        self.validator
    }
}
impl<T: 'static, F: PartialOrd + fmt::Debug + Send + Sync + 'static> FieldRule<T, F> {
    pub fn greater_than(self, bound: F) -> Self {
        let message = format!("must be greater than {bound:?}");
        self.must(move |value| value > &bound, message)
            .with_code("greater_than")
    }
    pub fn inclusive_between(self, min: F, max: F) -> Self {
        assert!(min <= max, "inclusive_between requires ordered bounds");
        let message = format!("must be between {min:?} and {max:?} (inclusive)");
        self.must(move |value| value >= &min && value <= &max, message)
            .with_code("inclusive_between")
    }
}
impl<T: 'static, F: AsRef<str> + ?Sized + 'static> FieldRule<T, F> {
    pub fn not_empty(self) -> Self {
        self.must(
            |value| !value.as_ref().trim().is_empty(),
            "must not be empty or whitespace",
        )
        .with_code("not_empty")
    }
    /// Counts Unicode scalar values, not bytes or grapheme clusters.
    pub fn length(self, min: usize, max: usize) -> Self {
        assert!(min <= max, "length requires min <= max");
        self.must(
            move |value| (min..=max).contains(&value.as_ref().chars().count()),
            format!("length must be between {min} and {max}"),
        )
        .with_code("length")
    }
}
impl<T: 'static, F: 'static> FieldRule<T, Option<F>> {
    pub fn not_none(self) -> Self {
        self.must(Option::is_some, "must be present")
            .with_code("not_none")
    }
}

impl<T: 'static> Validator<T> {
    /// Validates every element and produces paths such as `items[2].name`.
    pub fn for_each<U: 'static>(
        mut self,
        field: impl Into<String>,
        select: impl Fn(&T) -> &[U] + Send + Sync + 'static,
        validator: Validator<U>,
    ) -> Self {
        let field = field.into();
        self.rules.push(Box::new(move |value, errors| {
            for (index, child) in select(value).iter().enumerate() {
                if errors.full() {
                    break;
                }
                if let Err(child_errors) = validator.validate_limit(child, errors.remaining()) {
                    errors.extend(child_errors.0.into_iter().map(|error| ValidationError {
                        field: format!("{field}[{index}].{}", error.field),
                        message: error.message,
                        code: error.code,
                    }));
                }
            }
        }));
        self
    }
}

impl ValidationErrors {
    pub fn map<E>(self, convert: impl FnMut(ValidationError) -> E) -> Vec<E> {
        self.0.into_iter().map(convert).collect()
    }
}
impl<T: 'static> Validator<T> {
    /// Object-level predicate can compare any number of fields.
    pub fn check(
        mut self,
        path: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        predicate: impl Fn(&T) -> bool + Send + Sync + 'static,
    ) -> Self {
        let field = path.into();
        let code = code.into();
        let message = message.into();
        self.rules.push(Box::new(move |value, errors| {
            if !predicate(value) {
                errors.push(ValidationError {
                    field: field.clone(),
                    code: code.clone(),
                    message: message.clone(),
                });
            }
        }));
        self
    }
    /// Conditionally execute a complete reusable group of rules in declaration order.
    pub fn group_when(
        mut self,
        condition: impl Fn(&T) -> bool + Send + Sync + 'static,
        group: Validator<T>,
    ) -> Self {
        self.rules.push(Box::new(move |value, errors| {
            if condition(value)
                && let Err(group_errors) = group.validate_limit(value, errors.remaining())
            {
                errors.extend(group_errors.0);
            }
        }));
        self
    }
    /// None is skipped; use a separate not_none rule when presence is required.
    pub fn optional<U: 'static>(
        mut self,
        field: impl Into<String>,
        select: impl Fn(&T) -> &Option<U> + Send + Sync + 'static,
        validator: Validator<U>,
    ) -> Self {
        let field = field.into();
        self.rules.push(Box::new(move |value, errors| {
            if let Some(child) = select(value)
                && let Err(child_errors) = validator.validate_limit(child, errors.remaining())
            {
                errors.extend(child_errors.0.into_iter().map(|error| ValidationError {
                    field: format!("{field}.{}", error.field),
                    code: error.code,
                    message: error.message,
                }));
            }
        }));
        self
    }
    /// Report duplicate element indexes using keys; first occurrence remains valid.
    pub fn unique_by<U: 'static, K: Eq + std::hash::Hash + 'static>(
        mut self,
        field: impl Into<String>,
        select: impl Fn(&T) -> &[U] + Send + Sync + 'static,
        key: impl Fn(&U) -> K + Send + Sync + 'static,
    ) -> Self {
        let field = field.into();
        self.rules.push(Box::new(move |value, errors| {
            let mut seen = std::collections::HashSet::new();
            for (index, item) in select(value).iter().enumerate() {
                if errors.full() {
                    break;
                }
                if !seen.insert(key(item)) {
                    errors.push(ValidationError {
                        field: format!("{field}[{index}]"),
                        code: "unique".into(),
                        message: "duplicate key".into(),
                    });
                }
            }
        }));
        self
    }
}
