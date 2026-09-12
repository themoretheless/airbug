//! The failure types every mocked call and every ordered sequence reports, and
//! the helpers that verify a batch of them at the end of a test.
//!
//! This sits below both [`mod@crate::mock`] and [`crate::order`] so those two do not
//! have to depend on each other.
use std::fmt;

/// Everything one mocked method got wrong during a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockError {
    /// Name the mock was created with.
    pub method: String,
    /// Individual failures, in the order they were detected.
    pub details: Vec<String>,
}
impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mock {}: {}", self.method, self.details.join("; "))
    }
}
impl std::error::Error for MockError {}

/// Combined errors from multiple mocked methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationErrors(pub Vec<MockError>);
impl fmt::Display for VerificationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index != 0 {
                writeln!(f)?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}
impl std::error::Error for VerificationErrors {}

/// Shared verification interface for one method or a generated trait mock.
pub trait VerifyMocks: Send + Sync {
    /// Report everything this mock, sequence or generated trait double got
    /// wrong, so [`with_mocks`] can gather failures across all of them.
    fn verify_mocks(&self) -> Result<(), VerificationErrors>;
}

/// Collect the failures of every mock, so one run reports all of them.
fn collect(mocks: &[&dyn VerifyMocks]) -> Vec<MockError> {
    mocks
        .iter()
        .filter_map(|mock| mock.verify_mocks().err())
        .flat_map(|errors| errors.0)
        .collect()
}

#[track_caller]
fn assert_verified(mocks: &[&dyn VerifyMocks]) {
    let errors = collect(mocks);
    assert!(errors.is_empty(), "{}", VerificationErrors(errors));
}

/// Run a normal test body, then verify every supplied mock. On panic the original
/// panic propagates without running verification. Await/join workers inside the body.
#[track_caller]
pub fn with_mocks<R>(mocks: &[&dyn VerifyMocks], body: impl FnOnce() -> R) -> R {
    let result = body();
    assert_verified(mocks);
    result
}

/// Verify after an async body completes, using the caller's executor.
/// Dropping the future before completion does not verify expectations.
pub async fn with_mocks_async<R>(
    mocks: &[&dyn VerifyMocks],
    body: impl std::future::Future<Output = R>,
) -> R {
    let result = body.await;
    assert_verified(mocks);
    result
}
