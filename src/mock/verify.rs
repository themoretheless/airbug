//! Verifying one or many mocks at the end of a test.
use super::MockError;
use std::fmt;

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
