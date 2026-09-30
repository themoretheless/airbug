//! Sampling schedules and workload-time budgets. No wall clock is read by the planner.
use crate::{Config, Result, error};
use std::{collections::BTreeMap, time::Duration};

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

/// Per-case defaults. An explicit CLI setting overrides only the corresponding field.
#[derive(Clone, Debug, Default)]
pub struct Sampling {
    pub samples: Option<u32>,
    pub warmup: Option<Duration>,
    pub sample_time: Option<Duration>,
    /// Exact operations per worker in every sample; skips calibration.
    pub iterations: Option<u64>,
    pub mode: Option<SamplingMode>,
    /// Workload time across calibration, warmup and collection; max takes priority.
    pub min_time: Option<Duration>,
    pub max_time: Option<Duration>,
    /// Count only reported measured intervals rather than setup/drop-inclusive workload calls.
    pub exclude_external_time: Option<bool>,
}
impl Sampling {
    pub(crate) fn inherit(&mut self, parent: &Self) {
        macro_rules! inherit { ($($field:ident),*) => { $(if self.$field.is_none() { self.$field = parent.$field; })* }; }
        inherit!(
            samples,
            warmup,
            sample_time,
            iterations,
            mode,
            min_time,
            max_time,
            exclude_external_time
        );
    }

    pub(crate) fn apply(&self, base: &Config) -> Config {
        Config {
            samples: self.samples.unwrap_or(base.samples),
            warmup: self.warmup.unwrap_or(base.warmup),
            sample_time: self.sample_time.unwrap_or(base.sample_time),
            max_iterations: base.max_iterations,
        }
    }
    pub(crate) fn validate(&self, base: &Config, cap: u64) -> Result<()> {
        let config = self.apply(base);
        config.validate()?;
        if self.iterations.is_some_and(|n| n == 0 || n > cap) {
            return Err(error(format!("iterations must be 1..{cap} for this case")));
        }
        if self.mode == Some(SamplingMode::Linear) {
            if self.iterations.is_some() {
                return Err(error(
                    "fixed iterations cannot be combined with linear sampling",
                ));
            }
            if cap < config.samples as u64 {
                return Err(error("linear sampling requires max_iterations >= samples"));
            }
        }
        Ok(())
    }
    pub(crate) fn record(&self, contract: &mut BTreeMap<String, String>) {
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
    if cli.iter().any(|v| matches!(v.as_str(), "--help" | "-h")) {
        return Ok(Vec::new());
    }
    let mut args = Vec::new();
    let profile = cli.iter().any(|v| v == "--profile");
    for (name, flag) in [
        ("AIRBUG_BENCH_SAMPLES", "--samples"),
        ("AIRBUG_BENCH_ITERATIONS", "--iterations"),
        ("AIRBUG_BENCH_WARMUP_MS", "--warmup-ms"),
        ("AIRBUG_BENCH_SAMPLE_MS", "--sample-ms"),
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
            || (profile && matches!(flag, "--samples" | "--warmup-ms" | "--sample-ms"))
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
                "--sampling" => SamplingMode::parse(&value).map(|_| ()),
                "--timer" => crate::timer::TimerKind::parse(&value).map(|_| ()),
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
    samples: u32,
    cap: u64,
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
        let target = config.sample_time.as_nanos().saturating_mul(samples);
        let elapsed = elapsed_ns.max(1);
        let mut mode = sampling.mode.unwrap_or_default();
        if mode == SamplingMode::Auto {
            // Prefer a linear schedule only if its one-operation minimum fits within
            // twice the target time and all increasing counts fit the iteration cap.
            mode = if cap >= config.samples as u64
                && triangle.saturating_mul(elapsed)
                    <= target
                        .saturating_mul(calibrated_n as u128)
                        .saturating_mul(2)
            {
                SamplingMode::Linear
            } else {
                SamplingMode::Flat
            };
        }
        let base = if mode == SamplingMode::Linear {
            target
                .saturating_mul(calibrated_n as u128)
                .div_ceil(elapsed.saturating_mul(triangle))
                .clamp(1, (cap / config.samples as u64).max(1) as u128) as u64
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
    pub fn operations(&self, sequence: u32) -> u64 {
        let multiplier = if self.mode == SamplingMode::Linear {
            (sequence.saturating_add(1)).min(self.samples) as u64
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
    min_ns: u128,
    max_ns: Option<u128>,
    measured_only: bool,
}
impl TimeBudget {
    pub fn new(options: &Sampling) -> Self {
        Self {
            elapsed_ns: 0,
            min_ns: options.min_time.unwrap_or_default().as_nanos(),
            max_ns: options.max_time.map(|d| d.as_nanos()),
            measured_only: options.exclude_external_time.unwrap_or(false),
        }
    }
    pub fn add(&mut self, measured_ns: u128, wall: Duration) {
        self.elapsed_ns = self.elapsed_ns.saturating_add(if self.measured_only {
            measured_ns
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
    pub fn needs_samples(&self, collected: u32, requested: u32) -> bool {
        !self.expired() && (collected < requested || self.elapsed_ns < self.min_ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
