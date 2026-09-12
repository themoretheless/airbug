#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
pub mod assertions;
pub mod fixture;

pub use assertions::assert_that;
pub use fixture::{FixtureContext, Generate, GenerationError, GenerationErrorKind};

pub mod validation;
pub use validation::{ValidationError, ValidationErrors, Validator};

pub mod mock;
pub use mock::{Mock, MockError};

extern crate self as runit;
pub use mock::{VerificationErrors, VerifyMocks, with_mocks, with_mocks_async};
#[cfg(feature = "macros")]
pub use runit_macros::{Generate, cases, mock};
pub mod native;

pub mod checks;
pub mod snapshot;
pub mod time;
/// Common opt-in imports for ordinary Rust tests.
pub mod prelude {
    pub use crate::checks::*;
    pub use crate::mock::{Capture, Matcher};
    pub use crate::snapshot::{Snapshots, UpdateMode};
    pub use crate::time::{Clock, Eventually, ManualClock};
    pub use crate::{
        CallSequence, FixtureContext, Generate, Mock, Validator, VerifyMocks, assert_contains,
        assert_same_items, assert_that, check_all, with_mocks, with_mocks_async,
    };
    #[cfg(feature = "macros")]
    pub use crate::{cases, mock};
}

pub mod order;
pub use order::CallSequence;

pub mod report;
