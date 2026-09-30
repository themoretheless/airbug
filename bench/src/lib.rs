//! Benchmarks with explicit lifecycle, raw observations and offline analysis.
//! No GPU, async runtime, network, or global allocator is installed implicitly.
#![allow(clippy::collapsible_if)]
extern crate self as airbug_bench;
pub mod alloc;
pub mod analysis;
pub mod baseline;
pub mod bootstrap;
pub mod budget;
pub mod convenience;
pub mod counters;
pub mod executors;
pub mod profiling;
mod quick;
pub use quick::QuickConfig;
pub mod threads;
pub mod timer;
mod timing;
pub use convenience::{Fixture, Seeded, Selection, SortOrder};
pub mod history;
pub mod hypothesis;
pub mod hypothesis_plot;
pub mod image;
pub mod scenario;
pub use scenario::Recorder;
mod console;
pub mod density;
pub mod measurement;
pub mod model;
mod ordering;
pub mod outliers;
pub mod presentation;
pub mod regression;
pub mod relative;
pub mod report;
mod sampling;
pub use console::{ConsoleColor, ConsoleFormat};
mod suite;
pub mod summary;
pub mod violin;
pub use sampling::{Sampling, SamplingMode};
mod suite_measure;
pub mod viz;
pub use model::*;
pub use suite::{BatchPolicy, Config, ConsoleOutput, DropPolicy, Suite};

/// Standard best-effort optimization barrier for benchmark inputs and outputs.
pub use std::hint::black_box;

/// Pass a value through the optimization barrier, then destroy it at this call site.
/// Useful with lazy iterators when both producing and dropping each item belong in timing.
///
/// ```
/// (0..4).map(|n| vec![n; 32]).for_each(airbug_bench::black_box_drop);
/// ```
#[inline]
pub fn black_box_drop<T>(value: T) {
    drop(black_box(value));
}

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

/// Register a benchmark. `args = [...]` passes Copy + Debug values to a one-argument
/// function. `setup = callable` prepares fresh input outside timing; the benchmark
/// receives a mutable reference or takes ownership, according to its signature.
/// Both options may be combined.
/// `types` and `consts` instantiate generic cases; `threads = [1, 2]` measures
/// concurrent workers. Async functions use LocalExecutor unless `executor` is supplied.
/// `custom = true` passes an iteration count and expects a total Duration.
/// `bytes`/`items`/`chars`/`cycles` declare simultaneous throughput counters;
/// `samples`, `sample_ms`, `warmup_ms` tune sampling.
/// Invalid options and ambiguous lifecycle signatures are rejected.
/// ```compile_fail
/// static ALLOC: airbug_bench::alloc::TrackingAllocator<std::alloc::System> =
///     airbug_bench::alloc::TrackingAllocator::new(std::alloc::System);
/// #[airbug_bench::bench(allocator = &ALLOC, custom = true)]
/// fn unsupported_allocation_policy(_: u64) -> std::time::Duration { std::time::Duration::ZERO }
/// ```
///
/// ```compile_fail
/// #[airbug_bench::suite]
/// mod invalid {
///     #[group(args = [1])]
///     mod nested { #[bench] fn case() {} }
/// }
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(ignore = false)]
/// #[ignore]
/// fn duplicate_ignore() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(types = [(u8, u16, u32)])]
/// fn wrong_tuple<T, U>() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(types = [u8, u16])]
/// fn missing_tuple<T, U>() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(consts = [(1, 2, 3)])]
/// fn wrong_const_tuple<const N: usize, const M: usize>() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(consts = [1, 2])]
/// fn missing_const_tuple<const N: usize, const M: usize>() {}
/// ```
/// ```compile_fail
/// const VALUES: &[usize] = &[];
/// #[airbug_bench::bench(consts = VALUES)]
/// fn empty_external<const N: usize>() {}
/// ```
/// ```compile_fail
/// const VALUES: [usize; 21] = [1; 21];
/// #[airbug_bench::bench(consts = VALUES)]
/// fn oversized_external<const N: usize>() {}
/// ```
/// ```compile_fail
/// const VALUES: &[(usize, bool, char)] = &[(1, true, 'a')];
/// #[airbug_bench::bench(consts = VALUES)]
/// fn extra_external_field<const N: usize, const B: bool>() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(setup_thread = "worker", setup = || 1)]
/// fn missing_workers(value: &mut i32) {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(threads = [2], setup_thread = "elsewhere", setup = || 1)]
/// fn invalid_setup_thread(value: &mut i32) {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(input_bytes = |input: &Vec<u8>| input.len() as u64)]
/// fn counter_without_setup() {}
/// ```
/// ```compile_fail
/// struct StaticOnly;
/// impl From<&'static str> for StaticOnly { fn from(_: &'static str) -> Self { Self } }
/// #[airbug_bench::bench(types = [StaticOnly], args = [String::from("owned")])]
/// fn must_not_extend_lifetime<'input, T: From<&'input str>>(input: &'input str) -> T {
///     T::from(input)
/// }
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(types = [])]
/// fn empty<T>() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(custom = true, setup = || 1)]
/// fn conflicting(n: u64, input: &mut i32) -> std::time::Duration { std::time::Duration::ZERO }
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(threads = [2], setup = || 1, setup_thread = "coordinator")]
/// async fn threaded_async(value: &mut i32) {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(bytes = "not a count")]
/// fn invalid_counter() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(args = [1, 2], args = [3])]
/// fn duplicate(n: usize) {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(types = [u32])]
/// fn missing_type_parameter() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(unknown = 42)]
/// fn unknown() {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(setup = || 1, drop_output = "never")]
/// fn invalid_drop(n: &mut i32) {}
/// ```
/// ```compile_fail
/// #[airbug_bench::bench]
/// fn has_arguments(input: usize) -> usize { input }
/// ```
/// ```compile_fail
/// #[airbug_bench::bench(executor = airbug_bench::workloads::LocalExecutor)]
/// fn synchronous_executor() {}
/// ```
///
/// ```compile_fail
/// #[airbug_bench::group(executor = airbug_bench::workloads::LocalExecutor)]
/// mod invalid_executor_override {
///     #[bench(executor = airbug_bench::workloads::LocalExecutor)]
///     fn synchronous() {}
/// }
/// ```
#[cfg(feature = "macros")]
pub use airbug_bench_macros::bench;
/// Generate a benchmark executable from an inline module of `#[bench]` functions.
/// Use once at the root of a `harness = false` Cargo benchmark target.
/// `groups = [path::to::group]` imports reusable modules declared with `#[group]`.
#[cfg(feature = "macros")]
pub use airbug_bench_macros::{group, suite};
pub mod diagnostics;
pub mod workloads;

#[cfg(feature = "memory")]
pub mod memory;
