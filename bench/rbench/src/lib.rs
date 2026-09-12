//! Benchmarks with explicit lifecycle, raw observations and offline analysis.
//! No GPU, async runtime, network, or global allocator is installed implicitly.
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
pub use model::*;
pub use suite::{Config, DropPolicy, Suite};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
    std::io::Error::other(message.into()).into()
}

/// Attribute registration rejects ambiguous lifecycle signatures.
/// ```compile_fail
/// #[rbench::bench]
/// fn has_arguments(input: usize) -> usize { input }
/// ```
/// ```compile_fail
/// #[rbench::bench]
/// async fn implicit_runtime() {}
/// ```
#[cfg(feature = "macros")]
pub use rbench_macros::bench;
pub mod diagnostics;
pub mod workloads;

#[cfg(feature = "memory")]
pub mod memory;
