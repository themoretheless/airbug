//! Benchmarks with explicit lifecycle, raw observations and offline analysis.
//! No GPU, async runtime, network, or global allocator is installed implicitly.
#![allow(clippy::collapsible_if)]
pub mod alloc;
pub mod analysis;
pub mod budget;
pub mod convenience;
pub use convenience::{Fixture, Seeded, Selection};
pub mod image;
pub mod scenario;
pub use scenario::Recorder;
pub mod model;
pub mod report;
mod suite;
mod suite_measure;
pub mod viz;
pub use model::*;
pub use suite::{Config, DropPolicy, Suite};

/// Typed library error (binaries may still print `Display`).
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    TryFromSlice(#[from] std::array::TryFromSliceError),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("{0}")]
    Message(String),
}

impl From<&str> for BenchError {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}

impl From<String> for BenchError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

pub type Result<T> = std::result::Result<T, BenchError>;

pub fn error(message: impl Into<String>) -> BenchError {
    BenchError::Message(message.into())
}

/// Attribute registration rejects ambiguous lifecycle signatures.
/// ```compile_fail
/// #[airbug_bench::bench]
/// fn has_arguments(input: usize) -> usize { input }
/// ```
/// ```compile_fail
/// #[airbug_bench::bench]
/// async fn implicit_runtime() {}
/// ```
#[cfg(feature = "macros")]
pub use airbug_bench_macros::bench;
pub mod diagnostics;
pub mod workloads;

#[cfg(feature = "memory")]
pub mod memory;
