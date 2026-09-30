//! Timed profiling sessions, separate from statistical benchmark measurements.
use crate::{Result, error};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    time::{Duration, Instant},
};

/// Hooks run only in profiling mode. `stop` is attempted exactly once after each
/// attempted `start`, including start/work errors and unwinding panics. A profiler
/// should tolerate partially initialized state in `stop`.
pub trait Profiler {
    fn start(&mut self, case: &str, directory: &Path) -> Result<()>;
    fn stop(&mut self, case: &str, directory: &Path) -> Result<()>;
}
/// External profilers need no in-process hooks.
pub struct ExternalProfiler;
impl Profiler for ExternalProfiler {
    fn start(&mut self, _: &str, _: &Path) -> Result<()> {
        Ok(())
    }
    fn stop(&mut self, _: &str, _: &Path) -> Result<()> {
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileStats {
    pub batches: u64,
    pub operations: String,
    /// Actual elapsed wall time, including setup and synchronization.
    pub elapsed_ns: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileCase {
    pub id: String,
    /// Relative artifact directory within this profiling session.
    pub directory: String,
    pub stats: Option<ProfileStats>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileReport {
    pub complete: bool,
    pub requested_ns_per_case: String,
    pub cases: Vec<ProfileCase>,
}

pub(crate) fn validate_duration(duration: Duration) -> Result<()> {
    if duration.is_zero() || Instant::now().checked_add(duration).is_none() {
        return Err(error(
            "profile time must be positive and representable by the host clock",
        ));
    }
    Ok(())
}
pub(crate) fn attempt<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic");
            Err(error(format!("profiling panic: {message}")))
        }
    }
}
pub(crate) fn session<T>(
    profiler: &mut dyn Profiler,
    case: &str,
    directory: &Path,
    work: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let result = attempt(|| profiler.start(case, directory)).and_then(|_| attempt(work));
    let stopped = attempt(|| profiler.stop(case, directory));
    match (result, stopped) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(stop)) => Err(error(format!("profiler stop failed: {stop}"))),
        (Err(work), Err(stop)) => Err(error(format!("{work}; profiler stop also failed: {stop}"))),
    }
}
/// The wall clock controls duration even for caller-reported custom measurements.
/// Cancellation is cooperative between complete batches; a long operation can
/// overrun the requested duration. No samples or statistical estimates are made.
pub(crate) fn repeat(
    duration: Duration,
    cap: u64,
    fixed: Option<u64>,
    workers: u64,
    mut work: impl FnMut(u64) -> Result<u128>,
) -> Result<ProfileStats> {
    validate_duration(duration)?;
    if cap == 0 || workers == 0 || fixed.is_some_and(|n| n == 0 || n > cap) {
        return Err(error("invalid profiling iteration count"));
    }
    let start = Instant::now();
    let mut batches = 0u64;
    let mut operations = 0u128;
    let mut n = fixed.unwrap_or(1);
    let target = duration.min(Duration::from_millis(10)).as_nanos().max(1);
    while batches == 0 || start.elapsed() < duration {
        let before = Instant::now();
        work(n)?;
        let actual_ns = before.elapsed().as_nanos().max(1);
        batches = batches
            .checked_add(1)
            .ok_or_else(|| error("profiling batch count overflow"))?;
        operations = operations
            .checked_add(n as u128 * workers as u128)
            .ok_or_else(|| error("profiling operation count overflow"))?;
        if fixed.is_none() {
            n = (n as u128 * target / actual_ns).clamp(1, cap as u128) as u64;
        }
    }
    Ok(ProfileStats {
        batches,
        operations: operations.to_string(),
        elapsed_ns: start.elapsed().as_nanos().to_string(),
    })
}
