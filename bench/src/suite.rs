use crate::{error, Availability, Case, Metric, Observation, Result, Run, Status};
use std::{
    collections::BTreeMap,
    hint::black_box,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub samples: u32,
    pub warmup: Duration,
    pub sample_time: Duration,
    pub max_iterations: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            samples: 30,
            warmup: Duration::from_millis(50),
            sample_time: Duration::from_millis(5),
            max_iterations: 1_048_576,
        }
    }
}
impl Config {
    fn validate(&self) -> Result<()> {
        if self.samples == 0
            || self.samples > 100_000
            || self.max_iterations == 0
            || self.max_iterations > 1_048_576
            || self.sample_time.is_zero()
            || self.sample_time > Duration::from_secs(60)
            || self.warmup > Duration::from_secs(60)
        {
            return Err(error("invalid benchmark config: samples 1..100000, iterations 1..1048576, time 1ns..60s, warmup <=60s"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub enum DropPolicy {
    InsideTiming,
    OutsideTiming,
}
type Work<'a> = Box<dyn FnMut(u64) -> u128 + 'a>;
struct Entry<'a> {
    case: Case,
    work: Work<'a>,
    max_batch: u64,
    executed: bool,
    verify: Option<Box<dyn FnMut() -> Result<()> + 'a>>,
}
/// Registration is lazy: `--list` never runs workloads.
pub struct Suite<'a> {
    name: String,
    entries: Vec<Entry<'a>>,
    config: Config,
}
impl<'a> Suite<'a> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entries: vec![],
            config: Config::default(),
        }
    }
    pub fn config(&mut self, config: Config) -> &mut Self {
        self.config = config;
        self
    }
    pub fn bench<O: 'a>(&mut self, name: &str, mut f: impl FnMut() -> O + 'a) -> &mut Self {
        self.register(
            name,
            "reused closure state; output drop included",
            1_048_576,
            Box::new(move |n| {
                let start = Instant::now();
                for _ in 0..n {
                    black_box(f());
                }
                start.elapsed().as_nanos()
            }),
        );
        self
    }
    /// Fresh input for EVERY operation. Input generation and input Drop are excluded.
    /// Outputs are buffered only when their Drop is excluded; at most 64 per batch.
    /// Heap allocations owned by input/output are controlled by the caller, not byte-capped.
    pub fn bench_with_input<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(&mut I) -> O + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        self.register(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n| input_batch(n, &mut setup, &mut f, drop)),
        );
        self
    }
    /// Validate one fresh operation before calibration and after measurement, outside timing.
    /// This is a sampled correctness check, not validation of every measured operation.
    pub fn bench_checked<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        setup: impl FnMut() -> I + 'a,
        f: impl FnMut(&mut I) -> O + 'a,
        mut validate: impl FnMut(&I, &O) -> Result<()> + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        use std::{cell::RefCell, rc::Rc};
        let setup = Rc::new(RefCell::new(setup));
        let f = Rc::new(RefCell::new(f));
        let (s, work) = (setup.clone(), f.clone());
        self.register(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n| {
                // Borrow once per batch, before timing; no RefCell checks in the measured loop.
                let mut setup = s.borrow_mut();
                let mut f = work.borrow_mut();
                input_batch(n, &mut *setup, &mut *f, drop)
            }),
        );
        let entry = self.entries.last_mut().unwrap();
        entry.case.contract.insert(
            "validation".into(),
            "fresh operation before calibration and after measurement".into(),
        );
        entry.verify = Some(Box::new(move || {
            let mut input = (setup.borrow_mut())();
            let output = (f.borrow_mut())(&mut input);
            validate(&input, &output)
        }));
        self
    }
    /// Cartesian parameter matrix. Registration stays lazy; caller registers cases using each ID.
    /// Values must be unique per axis; at most 4096 combinations.
    pub fn matrix(
        &mut self,
        name: &str,
        axes: &[(&str, &[&str])],
        mut register: impl FnMut(&mut Self, &str, &BTreeMap<String, String>),
    ) -> Result<&mut Self> {
        let mut combinations = vec![BTreeMap::new()];
        let mut keys = std::collections::BTreeSet::new();
        for (key, values) in axes {
            if key.is_empty()
                || !keys.insert(*key)
                || values.is_empty()
                || values
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != values.len()
                || combinations.len().saturating_mul(values.len()) > 4096
            {
                return Err(error(
                    "invalid matrix: unique nonempty axes and values, <=4096 combinations",
                ));
            }
            combinations = combinations
                .into_iter()
                .flat_map(|row| {
                    values.iter().map(move |v| {
                        let mut r = row.clone();
                        r.insert((*key).to_string(), (*v).to_string());
                        r
                    })
                })
                .collect();
        }
        // Encode delimiters so distinct parameter values cannot produce the same generated ID.
        let encode = |s: &str| {
            s.bytes()
                .map(|b| {
                    if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                        (b as char).to_string()
                    } else {
                        format!("%{b:02X}")
                    }
                })
                .collect::<String>()
        };
        for params in combinations {
            let suffix = params
                .iter()
                .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
                .collect::<Vec<_>>()
                .join(",");
            let id = format!("{name}[{suffix}]");
            let before = self.entries.len();
            register(self, &id, &params);
            for entry in &mut self.entries[before..] {
                for (k, v) in &params {
                    entry.case.contract.insert(format!("param.{k}"), v.clone());
                }
            }
        }
        Ok(self)
    }
    fn register(&mut self, name: &str, lifecycle: &str, max_batch: u64, work: Work<'a>) {
        let contract = BTreeMap::from([
            ("lifecycle".into(), lifecycle.into()),
            ("completion".into(), "synchronous function return".into()),
        ]);
        self.entries.push(Entry {
            case: Case {
                id: format!("{}/{}", self.name, name),
                contract,
                metrics: vec![Metric::duration(
                    "wall",
                    "current thread wall clock; scheduler included",
                    "batch_total",
                )],
            },
            work,
            max_batch,
            executed: false,
            verify: None,
        });
    }
    /// Attach workload identity/parameters to the last registered benchmark.
    pub fn parameter(&mut self, key: &str, value: impl ToString) -> &mut Self {
        if let Some(e) = self.entries.last_mut() {
            e.case
                .contract
                .insert(format!("param.{key}"), value.to_string());
        }
        self
    }
    /// Executor construction happens at the call site. Future creation, polling and output Drop are timed.
    pub fn bench_async<E, F, O>(
        &mut self,
        name: &str,
        mut executor: E,
        mut future: impl FnMut() -> F + 'a,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
        F: std::future::Future<Output = O> + 'a,
        O: 'a,
    {
        self.bench(name, move || executor.block_on(future()));
        self.entries.last_mut().unwrap().case.contract.insert(
            "async.scope".into(),
            "future creation + executor block_on + output drop; executor construction excluded"
                .into(),
        );
        self
    }
    /// First invocation only: select exactly one unchecked case in a fresh worker process.
    /// No cache flushing is implied; external OS/driver caches remain uncontrolled.
    pub fn cold(&mut self) -> &mut Self {
        if let Some(e) = self.entries.last_mut() {
            e.case.contract.insert(
                "temperature".into(),
                "cold; first invocation only; no pilot/warmup; external caches uncontrolled".into(),
            );
        }
        self
    }
    pub fn warm(&mut self) -> &mut Self {
        if let Some(e) = self.entries.last_mut() {
            e.case
                .contract
                .insert("temperature".into(), "warm; calibrated and warmed".into());
        }
        self
    }
    pub fn tag(&mut self, tag: &str) -> &mut Self {
        self.parameter_tag(tag);
        self
    }
    fn parameter_tag(&mut self, tag: &str) {
        if let Some(e) = self.entries.last_mut() {
            e.case.contract.insert(format!("tag.{tag}"), "true".into());
        }
    }
    /// Positive work units per operation: e.g. bytes or elements. Throughput is derived in reports.
    pub fn work_units(&mut self, unit: &str, count: u64) -> &mut Self {
        if let Some(e) = self.entries.last_mut() {
            e.case.contract.insert("work.unit".into(), unit.into());
            e.case
                .contract
                .insert("work.count".into(), count.to_string());
        }
        self
    }
    pub fn seed(&mut self, seed: u64) -> &mut Self {
        self.parameter("seed", seed)
    }
    pub fn bench_fixture<T: 'static, O: 'a>(
        &mut self,
        name: &str,
        fixture: crate::Fixture<T>,
        mut f: impl FnMut(&mut T) -> O + 'a,
    ) -> &mut Self {
        self.register(
            name,
            "shared lazy process fixture; setup/borrow/drop excluded; output drop included",
            1_048_576,
            Box::new(move |n| {
                let mut input = fixture.get().borrow_mut();
                let start = Instant::now();
                for _ in 0..n {
                    black_box(f(black_box(&mut *input)));
                }
                start.elapsed().as_nanos()
            }),
        );
        self
    }
    pub fn list(&self, filter: &str) -> Vec<&str> {
        self.list_selected(&crate::Selection {
            pattern: filter.into(),
            ..Default::default()
        })
    }
    pub fn list_selected(&self, selection: &crate::Selection) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| selection.matches(&e.case))
            .map(|e| e.case.id.as_str())
            .collect()
    }
    pub fn run(&mut self, filter: &str) -> Result<Run> {
        self.run_selected(&crate::Selection {
            pattern: filter.into(),
            ..Default::default()
        })
    }
    pub fn run_selected(&mut self, selection: &crate::Selection) -> Result<Run> {
        selection.validate()?;
        self.config.validate()?;
        let mut ids = std::collections::BTreeSet::new();
        for e in self.entries.iter().filter(|e| selection.matches(&e.case)) {
            if !ids.insert(&e.case.id) {
                return Err(error("duplicate benchmark ID"));
            }
            if e.case.contract.contains_key("work.count")
                && (e
                    .case
                    .contract
                    .get("work.count")
                    .and_then(|n| n.parse::<u64>().ok())
                    .unwrap_or(0)
                    == 0
                    || e.case
                        .contract
                        .get("work.unit")
                        .is_none_or(|s| s.is_empty()))
            {
                return Err(error(
                    "work units require a nonempty unit and positive count",
                ));
            }
        }
        let selected: Vec<_> = self
            .entries
            .iter()
            .filter(|e| selection.matches(&e.case))
            .collect();
        if selected.iter().any(|e| {
            e.case
                .contract
                .get("temperature")
                .is_some_and(|v| v.starts_with("cold;"))
        }) && (selected.len() != 1 || selected[0].verify.is_some())
        {
            return Err(error("cold mode requires exactly one unchecked case; verify correctness in a separate process"));
        }
        let mut run = Run::new();
        run.provenance
            .insert("rbench_version".into(), env!("CARGO_PKG_VERSION").into());
        run.provenance
            .insert("samples".into(), self.config.samples.to_string());
        run.provenance.insert(
            "warmup_ns".into(),
            self.config.warmup.as_nanos().to_string(),
        );
        run.notes.push("Raw batches are not independent process replications. Batch averages are not individual-operation latency. CPU frequency and background load are uncontrolled.".into());
        for e in &mut self.entries {
            if !selection.matches(&e.case) {
                continue;
            }
            if run.cases.iter().any(|c: &Case| c.id == e.case.id) {
                return Err(error("duplicate benchmark ID"));
            }
            let cold = e
                .case
                .contract
                .get("temperature")
                .is_some_and(|v| v.starts_with("cold;"));
            if cold && e.executed {
                return Err(error("cold case already executed; launch a fresh worker"));
            }
            e.executed = true;
            let samples = if cold { 1 } else { self.config.samples };
            e.case
                .contract
                .insert("samples".into(), samples.to_string());
            e.case.contract.insert(
                "warmup_ns".into(),
                (if cold {
                    0
                } else {
                    self.config.warmup.as_nanos()
                })
                .to_string(),
            );
            e.case.contract.insert(
                "sample_target_ns".into(),
                (if cold {
                    0
                } else {
                    self.config.sample_time.as_nanos()
                })
                .to_string(),
            );
            eprintln!("rbench: {} — validating and calibrating", e.case.id);
            if let Some(check) = &mut e.verify {
                check().map_err(|err| error(format!("{} pre-validation: {err}", e.case.id)))?;
            }
            let cap = e.max_batch.min(self.config.max_iterations);
            let mut n = 1;
            if !cold {
                loop {
                    let elapsed = (e.work)(n);
                    if elapsed >= self.config.sample_time.as_nanos() || n >= cap {
                        break;
                    }
                    n = (n * 2).min(cap);
                }
            }
            let start = Instant::now();
            while !cold && start.elapsed() < self.config.warmup {
                (e.work)(n);
            }
            eprintln!("rbench: {} — measuring {} samples", e.case.id, samples);
            run.cases.push(e.case.clone());
            let mut under_target = false;
            for sequence in 0..samples {
                let elapsed = (e.work)(n);
                under_target |= elapsed < self.config.sample_time.as_nanos() / 2;
                run.observations.push(Observation {
                    case: e.case.id.clone(),
                    metric: "wall".into(),
                    variant: "candidate".into(),
                    process: 0,
                    pair: None,
                    sequence: sequence as u64,
                    value: Some(elapsed.to_string()),
                    operations: n,
                    availability: Availability::Available,
                });
            }
            if let Some(check) = &mut e.verify {
                check().map_err(|err| error(format!("{} post-validation: {err}", e.case.id)))?;
            }
            if under_target && !cold {
                run.notes.push(format!("{}: some samples below half the requested duration; iteration cap or workload drift may limit precision",e.case.id));
            }
        }
        run.status = Status::Complete;
        run.validate()?;
        Ok(run)
    }
    /// Minimal executable harness: --list, --filter, --samples, --sample-ms,
    /// --warmup-ms, --json, --output. Errors propagate to the caller's main.
    pub fn main(mut self) -> Result<()> {
        let mut args = std::env::args().skip(1);
        let all_args: Vec<_> = std::env::args().skip(1).collect();
        let mut profile = None;
        for pair in all_args.windows(2) {
            if pair[0] == "--profile" {
                if profile.is_some() {
                    return Err(error("profile specified twice"));
                }
                profile = Some(pair[1].clone());
            }
        }
        if let Some(p) = &profile {
            self.config = Config::profile(p)?;
        }
        let mut selection = crate::Selection::default();
        let mut list = false;
        let mut dry_run = false;
        let mut json = false;
        let mut output = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--dry-run" => dry_run = true,
                "--profile" => {
                    args.next().ok_or_else(|| error("profile requires value"))?;
                }
                "--exact" => selection.exact = true,
                "--glob" => selection.glob = true,
                "--exclude" => selection
                    .exclude
                    .push(args.next().ok_or_else(|| error("exclude requires glob"))?),
                "--tag" => selection
                    .tags
                    .push(args.next().ok_or_else(|| error("tag requires value"))?),
                "--list" => list = true,
                "--json" => json = true,
                "--filter" => {
                    selection.pattern = args
                        .next()
                        .ok_or_else(|| error("--filter requires value"))?
                }
                "--output" => {
                    output = Some(
                        args.next()
                            .ok_or_else(|| error("--output requires directory"))?,
                    )
                }
                "--samples" => {
                    self.config.samples = args
                        .next()
                        .ok_or_else(|| error("--samples requires value"))?
                        .parse()?
                }
                "--sample-ms" => {
                    self.config.sample_time = Duration::from_millis(
                        args.next()
                            .ok_or_else(|| error("--sample-ms requires value"))?
                            .parse()?,
                    )
                }
                "--warmup-ms" => {
                    self.config.warmup = Duration::from_millis(
                        args.next()
                            .ok_or_else(|| error("--warmup-ms requires value"))?
                            .parse()?,
                    )
                }
                "--help" | "-h" => {
                    println!("--list --profile quick|normal|thorough --filter TEXT [--exact|--glob] --exclude GLOB --tag TAG --samples N --sample-ms N --warmup-ms N --json --output NEW_DIRECTORY");
                    return Ok(());
                }
                _ => return Err(error(format!("unknown argument {arg}"))),
            }
        }
        selection.validate()?;
        if dry_run {
            self.config.validate()?;
            let cases: Vec<_> = self
                .entries
                .iter()
                .filter(|e| selection.matches(&e.case))
                .map(|e| &e.case)
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"cases":cases,"samples":self.config.samples,"warmup_ms":self.config.warmup.as_millis(),"target_sample_ms":self.config.sample_time.as_millis(),"target_measured_ms_per_case":self.config.samples as u128*self.config.sample_time.as_millis(),"note":"No work executed. Calibration, setup and validation add wall time; target duration is not a precision guarantee."})
                )?
            );
            return Ok(());
        }
        if list {
            for id in self.list_selected(&selection) {
                println!("{id}");
            }
            return Ok(());
        }
        if cfg!(debug_assertions) {
            return Err(error(
                "benchmarks require an optimized build; use cargo run --release or cargo bench",
            ));
        }
        let run = self.run_selected(&selection)?;
        if let Some(p) = output {
            run.save_new(p)?;
        }
        if json {
            println!("RBENCH_RESULT={}", serde_json::to_string(&run)?);
        } else {
            println!("{}", crate::report::markdown(&run)?);
        }
        Ok(())
    }
}

fn input_lifecycle(drop: DropPolicy) -> &'static str {
    match drop {
        DropPolicy::InsideTiming => "fresh input per operation; input setup/drop excluded; output drop included; chunks of <=64",
        DropPolicy::OutsideTiming => "fresh input per operation; input setup/drop and output drop excluded; chunks of <=64",
    }
}
fn input_batch<I, O>(
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(&mut I) -> O,
    drop: DropPolicy,
) -> u128 {
    let mut remaining = n;
    let mut total = 0;
    while remaining > 0 {
        let count = remaining.min(64);
        let mut inputs: Vec<I> = (0..count).map(|_| setup()).collect();
        let mut outputs = Vec::with_capacity(count as usize);
        let start = Instant::now();
        for input in &mut inputs {
            match drop {
                DropPolicy::InsideTiming => {
                    black_box(f(black_box(input)));
                }
                DropPolicy::OutsideTiming => outputs.push(black_box(f(black_box(input)))),
            }
        }
        total += start.elapsed().as_nanos();
        black_box(&outputs);
        remaining -= count;
    }
    total
}
