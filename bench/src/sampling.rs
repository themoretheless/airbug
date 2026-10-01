//! Sampling schedules and workload-time budgets. No wall clock is read by the planner.
use crate::{Config, Result, error};
use std::{collections::BTreeMap, time::Duration};

/// Parse decimal milliseconds without passing through floating point.
/// CLI and attributes use one-nanosecond resolution; extra fractional zeroes are harmless.
pub fn milliseconds(value: &str) -> Result<Duration> {
    let invalid = || {
        error(format!(
            "invalid millisecond duration {value:?}; use a nonnegative decimal with nanosecond precision"
        ))
    };
    let unsigned = value.strip_prefix('+').unwrap_or(value);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if (whole.is_empty() && fraction.is_empty())
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || (fraction.len() > 6 && fraction.as_bytes()[6..].iter().any(|b| *b != b'0'))
    {
        return Err(invalid());
    }
    let whole = if whole.is_empty() {
        0
    } else {
        whole.parse::<u128>().map_err(|_| invalid())?
    };
    let mut nanos = 0u32;
    for index in 0..6 {
        nanos =
            nanos * 10 + u32::from(fraction.as_bytes().get(index).copied().unwrap_or(b'0') - b'0');
    }
    let total = whole
        .checked_mul(1_000_000)
        .and_then(|n| n.checked_add(u128::from(nanos)))
        .ok_or_else(invalid)?;
    let seconds = u64::try_from(total / 1_000_000_000).map_err(|_| invalid())?;
    Ok(Duration::new(seconds, (total % 1_000_000_000) as u32))
}

/// Preserve existing integer previews and represent fractional/large values exactly.
pub(crate) fn milliseconds_json(nanos: u128) -> serde_json::Value {
    let whole = nanos / 1_000_000;
    let fraction = nanos % 1_000_000;
    if fraction == 0 {
        u64::try_from(whole)
            .map(serde_json::Value::from)
            .unwrap_or_else(|_| serde_json::Value::String(whole.to_string()))
    } else {
        serde_json::Value::String(
            format!("{whole}.{fraction:06}")
                .trim_end_matches('0')
                .to_owned(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SamplingMode {
    #[default]
    Flat,
    Linear,
    Auto,
}
impl SamplingMode {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "flat" => Ok(Self::Flat),
            "linear" => Ok(Self::Linear),
            "auto" => Ok(Self::Auto),
            _ => Err(error("sampling mode must be flat, linear or auto")),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Linear => "linear",
            Self::Auto => "auto",
        }
    }
}

/// Unit used by the requested sample budget. Whole worker batches always finish.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SampleCountUnit {
    #[default]
    Batches,
    Workers,
}
impl SampleCountUnit {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "batches" => Ok(Self::Batches),
            "workers" => Ok(Self::Workers),
            _ => Err(error("sample count unit must be batches or workers")),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Batches => "batches",
            Self::Workers => "workers",
        }
    }
}

/// Per-case defaults. An explicit CLI setting overrides only the corresponding field.
#[derive(Clone, Debug, Default)]
pub struct Sampling {
    pub samples: Option<u64>,
    pub sample_count_unit: Option<SampleCountUnit>,
    pub worker_start: Option<crate::timer::WorkerStart>,
    pub warmup: Option<Duration>,
    pub sample_time: Option<Duration>,
    /// Target time for all measured samples of one case, excluding warmup and
    /// calibration. Overrides the base per-sample target; actual time may differ
    /// due to indivisible operations, changing costs and iteration caps.
    pub measurement_time: Option<Duration>,
    /// Exact operations per worker in every sample; skips calibration.
    pub iterations: Option<u64>,
    pub mode: Option<SamplingMode>,
    /// Workload time across calibration, warmup and collection; max takes priority.
    pub min_time: Option<Duration>,
    pub max_time: Option<Duration>,
    /// Count measured intervals rather than setup/drop-inclusive workload calls.
    /// Zero-duration calls advance the budget by 1 ns without changing observations.
    pub exclude_external_time: Option<bool>,
}
impl Sampling {
    pub(crate) fn disabled(&self, base: &Config) -> bool {
        self.samples.unwrap_or(base.samples) == 0
            || self.iterations == Some(0)
            || self.max_time.is_some_and(|time| time.is_zero())
    }

    pub(crate) fn inherit(&mut self, parent: &Self) {
        if self.sample_time.is_none() && self.measurement_time.is_none() {
            self.sample_time = parent.sample_time;
            self.measurement_time = parent.measurement_time;
        }
        macro_rules! inherit { ($($field:ident),*) => { $(if self.$field.is_none() { self.$field = parent.$field; })* }; }
        inherit!(
            samples,
            sample_count_unit,
            worker_start,
            warmup,
            iterations,
            mode,
            min_time,
            max_time,
            exclude_external_time
        );
    }

    pub(crate) fn for_workers(&self, base: &Config, workers: u64) -> Self {
        let mut effective = self.clone();
        if self.sample_count_unit == Some(SampleCountUnit::Workers) {
            effective.samples = Some(
                self.samples
                    .unwrap_or(base.samples)
                    .div_ceil(workers.max(1)),
            );
        }
        effective
    }

    pub(crate) fn apply(&self, base: &Config) -> Config {
        let samples = self.samples.unwrap_or(base.samples);
        let sample_time = self
            .measurement_time
            .map(|total| {
                let nanos = total.as_nanos().div_ceil(u128::from(samples.max(1)));
                Duration::new(
                    (nanos / 1_000_000_000) as u64,
                    (nanos % 1_000_000_000) as u32,
                )
            })
            .unwrap_or_else(|| self.sample_time.unwrap_or(base.sample_time));
        Config {
            samples,
            warmup: self.warmup.unwrap_or(base.warmup),
            sample_time,
            max_iterations: base.max_iterations,
        }
    }
    pub(crate) fn validate(&self, base: &Config, cap: u64) -> Result<()> {
        if self.measurement_time.is_some() && self.sample_time.is_some() {
            return Err(error(
                "measurement_time and sample_time are mutually exclusive",
            ));
        }
        let config = self.apply(base);
        config.validate()?;
        if self.disabled(base) {
            return Ok(());
        }
        if self.iterations.is_some_and(|n| n == 0 || n > cap) {
            return Err(error(format!("iterations must be 1..{cap} for this case")));
        }
        if self.mode == Some(SamplingMode::Linear) {
            if self.iterations.is_some() {
                return Err(error(
                    "fixed iterations cannot be combined with linear sampling",
                ));
            }
            if cap < config.samples {
                return Err(error("linear sampling requires max_iterations >= samples"));
            }
        }
        Ok(())
    }
    pub(crate) fn record_worker_start(&self, case: &mut crate::Case) {
        if !case.contract.contains_key("threads") {
            return;
        }
        case.contract
            .insert("threads.executor".into(), "os-pool-v1".into());
        case.contract.insert(
            "threads.worker_lifetime".into(),
            "caller for one worker; persistent pool for multiple workers; rebuilt when worker count changes".into(),
        );
        let policy = self.worker_start.unwrap_or_default();
        case.contract
            .insert("threads.timer_start".into(), policy.as_str().into());
        case.contract.insert(
            "threads.timer_aggregation".into(),
            "sum of per-wave maximum worker intervals".into(),
        );
        if let Some(metric) = case.metrics.iter_mut().find(|m| m.id == "wall") {
            metric.scope = match policy {
                crate::timer::WorkerStart::Shared => "concurrent batch wall clock; scheduler included",
                crate::timer::WorkerStart::Local => "sum of maximum worker-local intervals per wave; wakeup before local start excluded",
            }.into();
        }
    }

    pub(crate) fn record(&self, contract: &mut BTreeMap<String, String>) {
        if let Some(target) = self.measurement_time {
            contract.insert(
                "sampling.measurement_target_ns".into(),
                target.as_nanos().to_string(),
            );
        } else {
            contract.remove("sampling.measurement_target_ns");
        }
        contract.insert(
            "sampling.count_unit".into(),
            self.sample_count_unit.unwrap_or_default().as_str().into(),
        );
        contract.insert(
            "sampling.requested_mode".into(),
            self.mode.unwrap_or_default().as_str().into(),
        );
        contract.insert(
            "sampling.iterations".into(),
            self.iterations
                .map(|n| n.to_string())
                .unwrap_or_else(|| "calibrated".into()),
        );
        contract.insert(
            "sampling.min_time_ns".into(),
            self.min_time.unwrap_or_default().as_nanos().to_string(),
        );
        contract.insert(
            "sampling.max_time_ns".into(),
            self.max_time
                .map(|v| v.as_nanos().to_string())
                .unwrap_or_else(|| "unlimited".into()),
        );
        contract.remove("sampling.zero_duration_budget_floor_ns");
        if self.exclude_external_time.unwrap_or(false) {
            contract.insert("sampling.zero_duration_budget_floor_ns".into(), "1".into());
        }
        contract.insert(
            "sampling.time_accounting".into(),
            if self.exclude_external_time.unwrap_or(false) {
                "measured"
            } else {
                "workload_wall"
            }
            .into(),
        );
    }
}

/// Translate runtime environment defaults to the same parser path as CLI values.
/// Explicit CLI flags (including profile fields) suppress the corresponding env value.
pub(crate) fn environment_args(
    cli: &[String],
    mut lookup: impl FnMut(&str) -> Result<Option<String>>,
) -> Result<Vec<String>> {
    if cli
        .iter()
        .any(|v| matches!(v.as_str(), "--help" | "-h" | "--help-all"))
    {
        return Ok(Vec::new());
    }
    let mut args = Vec::new();
    let profile = cli.iter().any(|v| v == "--profile");
    for (name, flag) in [
        ("AIRBUG_BENCH_SAMPLES", "--samples"),
        ("AIRBUG_BENCH_SAMPLE_COUNT_UNIT", "--sample-count-unit"),
        ("AIRBUG_BENCH_WORKER_START", "--worker-start"),
        ("AIRBUG_BENCH_ITERATIONS", "--iterations"),
        ("AIRBUG_BENCH_WARMUP_MS", "--warmup-ms"),
        ("AIRBUG_BENCH_SAMPLE_MS", "--sample-ms"),
        ("AIRBUG_BENCH_MEASUREMENT_MS", "--measurement-ms"),
        ("AIRBUG_BENCH_SAMPLING", "--sampling"),
        ("AIRBUG_BENCH_MIN_TIME_MS", "--min-time-ms"),
        ("AIRBUG_BENCH_MAX_TIME_MS", "--max-time-ms"),
        ("AIRBUG_BENCH_TIMER", "--timer"),
        ("AIRBUG_BENCH_SORT", "--sort"),
        ("AIRBUG_BENCH_REVERSE", "--reverse"),
        ("AIRBUG_BENCH_BYTES_FORMAT", "--bytes-format"),
        ("AIRBUG_BENCH_THREADS", "--threads"),
        ("AIRBUG_BENCH_BYTES_COUNT", "--bytes-count"),
        ("AIRBUG_BENCH_ITEMS_COUNT", "--items-count"),
        ("AIRBUG_BENCH_CHARS_COUNT", "--chars-count"),
        ("AIRBUG_BENCH_CYCLES_COUNT", "--cycles-count"),
        ("AIRBUG_BENCH_BITS_COUNT", "--bits-count"),
        (
            "AIRBUG_BENCH_EXCLUDE_EXTERNAL_TIME",
            "--exclude-external-time",
        ),
    ] {
        if cli.iter().any(|v| v == flag)
            || (profile
                && matches!(
                    flag,
                    "--samples" | "--warmup-ms" | "--sample-ms" | "--measurement-ms"
                ))
            || (matches!(flag, "--sample-ms" | "--measurement-ms")
                && cli
                    .iter()
                    .any(|v| matches!(v.as_str(), "--sample-ms" | "--measurement-ms")))
            || (flag == "--exclude-external-time"
                && cli.iter().any(|v| v == "--include-external-time"))
            || (flag == "--reverse" && cli.iter().any(|v| v == "--forward"))
        {
            continue;
        }
        let Some(value) = lookup(name)? else {
            continue;
        };
        if matches!(flag, "--exclude-external-time" | "--reverse") {
            let enabled = match value.as_str() {
                "true" | "1" => true,
                "false" | "0" => false,
                _ => return Err(error(format!("{name} must be true, false, 1 or 0"))),
            };
            let selected = match (flag, enabled) {
                ("--reverse", true) => "--reverse",
                ("--reverse", false) => "--forward",
                (_, true) => "--exclude-external-time",
                (_, false) => "--include-external-time",
            };
            args.push(selected.into());
        } else {
            let validation = match flag {
                "--threads" => crate::threads::parse_list(&value).map(|_| ()),
                "--warmup-ms" | "--sample-ms" | "--measurement-ms" | "--min-time-ms"
                | "--max-time-ms" => milliseconds(&value).map(|_| ()),
                "--sampling" => SamplingMode::parse(&value).map(|_| ()),
                "--sample-count-unit" => SampleCountUnit::parse(&value).map(|_| ()),
                "--timer" => crate::timer::TimerKind::parse(&value).map(|_| ()),
                "--worker-start" => crate::timer::WorkerStart::parse(&value).map(|_| ()),
                "--sort" => crate::SortOrder::parse(&value).map(|_| ()),
                "--bytes-format" => crate::report::BytesFormat::parse(&value).map(|_| ()),
                _ => value
                    .parse::<u64>()
                    .map(|_| ())
                    .map_err(|e| error(e.to_string())),
            };
            validation.map_err(|e| error(format!("{name}: {e}")))?;
            args.extend([flag.into(), value]);
        }
    }
    Ok(args)
}

pub(crate) struct Schedule {
    pub mode: SamplingMode,
    base: u64,
    samples: u64,
    cap: u64,
}
// Exact 128-bit products represented as high/low limbs. Scheduling happens
// outside measurement; avoid f64 rounding and saturating intermediate products.
fn wide_product(a: u128, b: u128) -> [u128; 2] {
    let mask = u64::MAX as u128;
    let (a0, a1) = (a & mask, a >> 64);
    let (b0, b1) = (b & mask, b >> 64);
    let low = a0 * b0;
    let cross0 = a0 * b1;
    let cross1 = a1 * b0;
    let middle = (low >> 64) + (cross0 & mask) + (cross1 & mask);
    [
        a1 * b1 + (cross0 >> 64) + (cross1 >> 64) + (middle >> 64),
        (middle << 64) | (low & mask),
    ]
}

fn target_iterations(target: u128, calibrated: u64, elapsed: u128, factor: u128, cap: u64) -> u64 {
    let factor = factor.max(1);
    let numerator = wide_product(target, calibrated as u128);
    let mut low = 1u64;
    // Every caller bounds count * factor to u128; also enforce it here.
    let mut high = (cap as u128).min(u128::MAX / factor).max(1) as u64;
    while low < high {
        let middle = low + (high - low) / 2;
        if wide_product(elapsed.max(1), factor * middle as u128) >= numerator {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

impl Schedule {
    pub fn new(
        sampling: &Sampling,
        config: &Config,
        calibrated_n: u64,
        elapsed_ns: u128,
        cap: u64,
    ) -> Self {
        if let Some(iterations) = sampling.iterations {
            return Self {
                mode: SamplingMode::Flat,
                base: iterations,
                samples: config.samples,
                cap,
            };
        }
        let samples = config.samples as u128;
        let triangle = samples * (samples + 1) / 2;
        let target = sampling
            .measurement_time
            .map(|time| time.as_nanos())
            .unwrap_or_else(|| config.sample_time.as_nanos().saturating_mul(samples));
        let elapsed = elapsed_ns.max(1);
        let mut mode = sampling.mode.unwrap_or_default();
        if mode == SamplingMode::Auto {
            // Prefer a linear schedule only if its one-operation minimum fits within
            // twice the target time and all increasing counts fit the iteration cap.
            mode = if cap >= config.samples
                && wide_product(triangle, elapsed) <= wide_product(target, calibrated_n as u128 * 2)
            {
                SamplingMode::Linear
            } else {
                SamplingMode::Flat
            };
        }
        let base = if mode == SamplingMode::Linear {
            target_iterations(
                target,
                calibrated_n,
                elapsed,
                triangle,
                (cap / config.samples).max(1),
            )
        } else if sampling.measurement_time.is_some() {
            target_iterations(target, calibrated_n, elapsed, samples, cap)
        } else {
            calibrated_n.clamp(1, cap)
        };
        Self {
            mode,
            base,
            samples: config.samples,
            cap,
        }
    }
    pub fn operations(&self, sequence: u64) -> u64 {
        let multiplier = if self.mode == SamplingMode::Linear {
            (sequence.saturating_add(1)).min(self.samples)
        } else {
            1
        };
        self.base.saturating_mul(multiplier).min(self.cap)
    }
}

/// Accounting includes workload calls during calibration and warmup, matching collection.
/// Limits are cooperative at call boundaries; a running workload is never interrupted.
pub(crate) struct TimeBudget {
    pub elapsed_ns: u128,
    pub zero_duration_steps: u64,
    min_ns: u128,
    max_ns: Option<u128>,
    measured_only: bool,
}
impl TimeBudget {
    pub fn new(options: &Sampling) -> Self {
        Self {
            elapsed_ns: 0,
            zero_duration_steps: 0,
            min_ns: options.min_time.unwrap_or_default().as_nanos(),
            max_ns: options.max_time.map(|d| d.as_nanos()),
            measured_only: options.exclude_external_time.unwrap_or(false),
        }
    }
    pub fn add(&mut self, measured_ns: u128, wall: Duration) {
        self.elapsed_ns = self.elapsed_ns.saturating_add(if self.measured_only {
            // Budget progress must survive timer quantization and custom zero
            // durations. This changes accounting only, never recorded observations.
            if measured_ns == 0 {
                self.zero_duration_steps = self.zero_duration_steps.saturating_add(1);
            }
            measured_ns.max(1)
        } else {
            wall.as_nanos()
        });
    }
    pub fn minimum_met(&self) -> bool {
        self.elapsed_ns >= self.min_ns
    }
    pub fn expired(&self) -> bool {
        self.max_ns.is_some_and(|max| self.elapsed_ns >= max)
    }
    pub fn needs_samples(&self, collected: u64, requested: u64) -> bool {
        !self.expired() && (collected < requested || self.elapsed_ns < self.min_ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wide_products_preserve_full_precision() {
        assert_eq!(wide_product(u128::MAX, u128::MAX), [u128::MAX - 1, 1]);
        assert_eq!(wide_product(1u128 << 127, 2), [1, 0]);
        assert_eq!(wide_product(u128::MAX, 0), [0, 0]);
        for a in [0, 1, 3, u64::MAX as u128, 1u128 << 64, u128::MAX] {
            for b in [0, 1, 7, u64::MAX as u128, u128::MAX] {
                assert_eq!(wide_product(a, b), wide_product(b, a));
                if let Some(product) = a.checked_mul(b) {
                    assert_eq!(wide_product(a, b), [0, product]);
                }
            }
        }
    }

    #[test]
    fn target_counts_round_up_and_bound_without_saturation() {
        assert_eq!(target_iterations(900, 4, 400, 3, 100), 3);
        assert_eq!(target_iterations(901, 4, 400, 3, 100), 4);
        assert_eq!(target_iterations(900, 4, 400, 6, 100), 2);
        assert_eq!(target_iterations(900, 4, 400, 3, 2), 2);
        assert_eq!(target_iterations(0, 4, 400, 3, 100), 1);
        // Both products exceed u128; saturating them would incorrectly yield 1.
        assert_eq!(
            target_iterations(u128::MAX, u64::MAX, u128::MAX, 1, u64::MAX),
            u64::MAX
        );
        assert_eq!(
            target_iterations(u128::MAX, u64::MAX, u128::MAX, 3, u64::MAX),
            u64::MAX / 3
        );
    }

    #[test]
    fn long_durations_validate_and_schedules_remain_bounded() {
        for duration in [
            Duration::from_secs(61),
            Duration::from_secs(3600),
            Duration::MAX,
        ] {
            let config = Config {
                samples: 4,
                sample_time: duration,
                warmup: duration,
                max_iterations: 100,
            };
            config.validate().unwrap();
            for mode in [SamplingMode::Flat, SamplingMode::Linear, SamplingMode::Auto] {
                let options = Sampling {
                    mode: Some(mode),
                    ..Default::default()
                };
                options.validate(&config, 100).unwrap();
                let schedule = Schedule::new(&options, &config, 10, 100, 100);
                assert!((0..4).all(|n| (1..=100).contains(&schedule.operations(n))));
            }
        }
        assert!(
            Config {
                sample_time: Duration::ZERO,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn sample_counts_cover_u64_without_planner_overflow() {
        for samples in [100_001, u32::MAX as u64 + 1, u64::MAX] {
            let config = Config {
                samples,
                sample_time: Duration::MAX,
                ..Default::default()
            };
            config.validate().unwrap();
            for mode in [SamplingMode::Flat, SamplingMode::Auto] {
                let options = Sampling {
                    mode: Some(mode),
                    ..Default::default()
                };
                options.validate(&config, config.max_iterations).unwrap();
                let schedule =
                    Schedule::new(&options, &config, 4, u128::MAX, config.max_iterations);
                for sequence in [0, samples - 1, u64::MAX] {
                    assert!((1..=config.max_iterations).contains(&schedule.operations(sequence)));
                }
            }
            let budget = TimeBudget::new(&Sampling::default());
            assert!(budget.needs_samples(samples - 1, samples));
            assert!(!budget.needs_samples(samples, samples));
        }
        assert!(
            Config {
                samples: 0,
                ..Default::default()
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn collection_and_minimum_time_can_cross_one_hundred_thousand_samples() {
        for (requested, minimum, expected) in [(100_001, 0, 100_001), (1, 100_002, 100_002)] {
            let calls = std::rc::Rc::new(std::cell::Cell::new(0u64));
            let captured = calls.clone();
            let mut suite = crate::Suite::new("large-sample-count");
            suite.config(Config {
                samples: requested,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 1,
            });
            suite.bench_custom("constant", move |n| {
                assert_eq!(n, 1);
                captured.set(captured.get() + 1);
                Duration::from_nanos(1)
            });
            suite.sampling(Sampling {
                iterations: Some(1),
                min_time: Some(Duration::from_nanos(minimum)),
                exclude_external_time: Some(true),
                ..Default::default()
            });
            let run = suite.run("").unwrap();
            assert_eq!(calls.get(), expected);
            assert_eq!(run.observations.len() as u64, expected);
            assert_eq!(run.observations.last().unwrap().sequence, expected - 1);
            assert!(
                run.observations
                    .iter()
                    .all(|o| o.operations == 1 && o.value.as_deref() == Some("1"))
            );
            run.validate().unwrap();
            if minimum > 0 {
                assert!(run.notes.iter().any(|n| n.contains("to meet min_time")));
            }
        }
    }

    #[test]
    fn total_measurement_target_preserves_precision_and_nearest_time_override() {
        let base = Config {
            samples: 3,
            ..Config::default()
        };
        let total = Sampling {
            measurement_time: Some(Duration::from_nanos(7)),
            ..Default::default()
        };
        assert_eq!(total.apply(&base).sample_time, Duration::from_nanos(3));
        let linear = Sampling {
            mode: Some(SamplingMode::Linear),
            ..total.clone()
        };
        let config = linear.apply(&base);
        let schedule = Schedule::new(&linear, &config, 6, 7, 100);
        assert_eq!(
            (0..3).map(|i| schedule.operations(i)).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        let mut child = Sampling {
            sample_time: Some(Duration::from_nanos(2)),
            ..Default::default()
        };
        child.inherit(&total);
        assert!(child.measurement_time.is_none());
        assert_eq!(child.apply(&base).sample_time, Duration::from_nanos(2));
        let mut inherited = Sampling::default();
        inherited.inherit(&total);
        assert_eq!(inherited.measurement_time, total.measurement_time);
        let mut contract = BTreeMap::new();
        inherited.record(&mut contract);
        assert_eq!(contract["sampling.measurement_target_ns"], "7");
        child.record(&mut contract);
        assert!(!contract.contains_key("sampling.measurement_target_ns"));
        let workers = Sampling {
            samples: Some(5),
            sample_count_unit: Some(SampleCountUnit::Workers),
            ..total.clone()
        }
        .for_workers(&base, 3);
        assert_eq!(workers.apply(&base).samples, 2);
        assert_eq!(workers.apply(&base).sample_time, Duration::from_nanos(4));
        let maximum = Sampling {
            samples: Some(1),
            measurement_time: Some(Duration::MAX),
            ..Default::default()
        };
        assert_eq!(maximum.apply(&base).sample_time, Duration::MAX);
        assert!(
            Sampling {
                measurement_time: Some(Duration::ZERO),
                ..Default::default()
            }
            .validate(&base, 100)
            .is_err()
        );
        assert!(
            Sampling {
                sample_time: Some(Duration::from_nanos(1)),
                ..total
            }
            .validate(&base, 100)
            .is_err()
        );
    }

    #[test]
    fn linear_and_auto_schedules_use_cost_and_respect_caps() {
        let config = Config {
            samples: 4,
            sample_time: Duration::from_nanos(100),
            max_iterations: 100,
            ..Config::default()
        };
        let options = Sampling {
            mode: Some(SamplingMode::Linear),
            ..Default::default()
        };
        let linear = Schedule::new(&options, &config, 10, 100, 100);
        assert_eq!(
            (0..4).map(|i| linear.operations(i)).collect::<Vec<_>>(),
            [4, 8, 12, 16]
        );
        assert_eq!(linear.operations(4), 16); // Extension to satisfy min_time repeats the final size.
        let auto = Sampling {
            mode: Some(SamplingMode::Auto),
            ..Default::default()
        };
        assert_eq!(
            Schedule::new(&auto, &config, 10, 100, 100).mode,
            SamplingMode::Linear
        );
        assert_eq!(
            Schedule::new(&auto, &config, 1, 10_000, 100).mode,
            SamplingMode::Flat
        );
        assert_eq!(
            Schedule::new(&auto, &config, 10, 100, 2).mode,
            SamplingMode::Flat
        );
        let capped = Schedule::new(&options, &config, 100, 1, 100);
        assert_eq!(capped.operations(3), 100);
    }
    #[test]
    fn budgets_distinguish_external_time_and_maximum_wins() {
        let options = Sampling {
            min_time: Some(Duration::from_nanos(100)),
            max_time: Some(Duration::from_nanos(50)),
            ..Default::default()
        };
        let mut wall = TimeBudget::new(&options);
        wall.add(10, Duration::from_nanos(60));
        assert!(wall.expired());
        assert!(!wall.needs_samples(0, 30));
        let mut measured = TimeBudget::new(&Sampling {
            exclude_external_time: Some(true),
            ..options
        });
        measured.add(10, Duration::from_nanos(60));
        assert!(!measured.expired());
        assert!(measured.needs_samples(30, 30));
        measured.add(40, Duration::ZERO);
        assert!(measured.expired());
    }
}

#[cfg(test)]
mod environment_tests {
    use super::*;
    #[test]
    fn environment_timer_sort_direction_and_byte_format() {
        let vars = BTreeMap::from([
            ("AIRBUG_BENCH_TIMER", "os"),
            ("AIRBUG_BENCH_SORT", "natural"),
            ("AIRBUG_BENCH_REVERSE", "true"),
            ("AIRBUG_BENCH_BYTES_FORMAT", "binary"),
        ]);
        assert_eq!(
            environment_args(&[], |k| Ok(vars.get(k).map(|v| v.to_string()))).unwrap(),
            [
                "--timer",
                "os",
                "--sort",
                "natural",
                "--reverse",
                "--bytes-format",
                "binary"
            ]
        );
        let cli = [
            "--timer",
            "os",
            "--sort",
            "source",
            "--forward",
            "--bytes-format",
            "decimal",
        ]
        .map(String::from);
        assert!(
            environment_args(&cli, |k| Ok(vars.contains_key(k).then(|| "invalid".into())))
                .unwrap()
                .is_empty()
        );
        for key in vars.keys() {
            assert!(
                environment_args(&[], |k| Ok((k == *key).then(|| "invalid".into())))
                    .unwrap_err()
                    .to_string()
                    .contains(key)
            );
        }
        assert_eq!(
            environment_args(&[], |k| Ok(
                (k == "AIRBUG_BENCH_REVERSE").then(|| "false".into())
            ))
            .unwrap(),
            ["--forward"]
        );
    }
    #[test]
    fn environment_sampling_precedence_and_errors() {
        let vars = BTreeMap::from([
            ("AIRBUG_BENCH_SAMPLES", "7"),
            ("AIRBUG_BENCH_ITERATIONS", "13"),
            ("AIRBUG_BENCH_WARMUP_MS", "0"),
            ("AIRBUG_BENCH_SAMPLE_MS", "2"),
            ("AIRBUG_BENCH_SAMPLING", "flat"),
            ("AIRBUG_BENCH_MIN_TIME_MS", "0"),
            ("AIRBUG_BENCH_MAX_TIME_MS", "99"),
            ("AIRBUG_BENCH_EXCLUDE_EXTERNAL_TIME", "false"),
        ]);
        let get = |key: &str| Ok(vars.get(key).map(|v| v.to_string()));
        let args = environment_args(&[], get).unwrap();
        assert_eq!(
            args,
            [
                "--samples",
                "7",
                "--iterations",
                "13",
                "--warmup-ms",
                "0",
                "--sample-ms",
                "2",
                "--sampling",
                "flat",
                "--min-time-ms",
                "0",
                "--max-time-ms",
                "99",
                "--include-external-time"
            ]
        );
        let cli = [
            "--profile",
            "quick",
            "--iterations",
            "3",
            "--exclude-external-time",
        ]
        .map(String::from);
        assert_eq!(
            environment_args(&cli, get).unwrap(),
            [
                "--sampling",
                "flat",
                "--min-time-ms",
                "0",
                "--max-time-ms",
                "99"
            ]
        );
        for key in vars.keys() {
            let error = environment_args(&[], |name| Ok((name == *key).then(|| "invalid".into())))
                .unwrap_err();
            assert!(error.to_string().contains(key));
        }
        let cli = ["--samples".into(), "2".into()];
        assert!(
            environment_args(&cli, |name| Ok(
                (name == "AIRBUG_BENCH_SAMPLES").then(|| "invalid".into())
            ))
            .unwrap()
            .is_empty()
        );
    }
}

#[cfg(test)]
mod worker_budget_tests {
    use super::*;
    #[test]
    fn worker_budget_rounding_is_overflow_safe_and_used_for_linear_validation() {
        let base = Config::default();
        let options = Sampling {
            samples: Some(u64::MAX),
            sample_count_unit: Some(SampleCountUnit::Workers),
            ..Default::default()
        };
        assert_eq!(
            options.for_workers(&base, 2).apply(&base).samples,
            u64::MAX / 2 + 1
        );
        assert_eq!(options.for_workers(&base, u64::MAX).apply(&base).samples, 1);
        let linear = Sampling {
            samples: Some(5),
            mode: Some(SamplingMode::Linear),
            sample_count_unit: Some(SampleCountUnit::Workers),
            ..Default::default()
        };
        linear.for_workers(&base, 3).validate(&base, 2).unwrap();
        assert!(linear.for_workers(&base, 1).validate(&base, 2).is_err());
    }
}

#[cfg(test)]
mod millisecond_tests {
    use super::*;
    #[test]
    fn decimal_milliseconds_preserve_nanoseconds_and_full_duration_domain() {
        for (text, expected) in [
            ("0", Duration::ZERO),
            (".5", Duration::from_micros(500)),
            ("+1.000001", Duration::from_nanos(1_000_001)),
            ("0.000001000", Duration::from_nanos(1)),
            ("18446744073709551615", Duration::from_millis(u64::MAX)),
            ("18446744073709551615999.999999", Duration::MAX),
        ] {
            assert_eq!(milliseconds(text).unwrap(), expected, "{text}");
        }
        for text in [
            "",
            ".",
            "-1",
            "NaN",
            "inf",
            "1.2.3",
            " 1",
            "1e-3",
            "0.0000001",
            "18446744073709551616000",
            "999999999999999999999999999999999999999999",
        ] {
            assert!(milliseconds(text).is_err(), "{text}");
        }
    }
}
