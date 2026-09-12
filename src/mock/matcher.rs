//! Named predicates over call arguments.
use std::sync::Arc;

/// Reusable named predicate, with short-circuit composition.
pub struct Matcher<A> {
    name: String,
    predicate: Arc<dyn Fn(&A) -> bool + Send + Sync>,
}
impl<A> Clone for Matcher<A> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            predicate: self.predicate.clone(),
        }
    }
}
impl<A: 'static> Matcher<A> {
    pub fn new(
        name: impl Into<String>,
        predicate: impl Fn(&A) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            predicate: Arc::new(predicate),
        }
    }
    /// The name this matcher reports in expectation labels and failure messages.
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn matches(&self, value: &A) -> bool {
        (self.predicate)(value)
    }
    pub fn and(self, other: Self) -> Self {
        Self::new(format!("({} and {})", self.name, other.name), move |a| {
            self.matches(a) && other.matches(a)
        })
    }
    pub fn or(self, other: Self) -> Self {
        Self::new(format!("({} or {})", self.name, other.name), move |a| {
            self.matches(a) || other.matches(a)
        })
    }
    pub fn negate(self) -> Self {
        Self::new(format!("not {}", self.name), move |a| !self.matches(a))
    }
}
