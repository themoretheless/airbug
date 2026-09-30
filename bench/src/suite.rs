use crate::sampling::{Sampling, Schedule, TimeBudget};
use crate::suite_measure::{input_batch, input_lifecycle};
use crate::timing::Measured;
use crate::{Availability, Case, Metric, Observation, Result, Run, Status, error};
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
    pub(crate) fn validate(&self) -> Result<()> {
        if self.samples == 0
            || self.samples > 100_000
            || self.max_iterations == 0
            || self.max_iterations > 1_048_576
            || self.sample_time.is_zero()
            || self.sample_time > Duration::from_secs(60)
            || self.warmup > Duration::from_secs(60)
        {
            return Err(error(
                "invalid benchmark config: samples 1..100000, iterations 1..1048576, time 1ns..60s, warmup <=60s",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub enum DropPolicy {
    InsideTiming,
    OutsideTiming,
}
/// Controls how many fresh inputs and deferred outputs coexist. Counts do not
/// cap heap bytes owned by those values. Smaller batches incur more timer reads.
#[derive(Clone, Copy, Debug)]
pub enum BatchPolicy {
    PerIteration,
    /// About ten batches per sample, suitable for small inputs.
    SmallInput,
    /// About a thousand batches per sample, reducing live input memory.
    LargeInput,
    Iterations(std::num::NonZeroU64),
    Batches(std::num::NonZeroU64),
}
impl BatchPolicy {
    pub(crate) fn size(self, iterations: u64) -> u64 {
        match self {
            Self::PerIteration => 1,
            Self::SmallInput => iterations.div_ceil(10).max(1),
            Self::LargeInput => iterations.div_ceil(1000).max(1),
            Self::Iterations(n) => n.get(),
            Self::Batches(n) => iterations.div_ceil(n.get()).max(1),
        }
    }
}
type Work<'a> = Box<dyn FnMut(u64, crate::timer::Timer) -> Result<Measured> + 'a>;

struct Entry<'a> {
    ordering_group: String,
    ordering_family: Option<String>,
    ordering_name: String,
    input_counters: Option<crate::counters::InputCounters>,
    measurement_sample: Option<(String, std::rc::Rc<std::cell::Cell<Option<f64>>>)>,
    formatters: BTreeMap<String, Box<dyn crate::measurement::ValueFormatter + 'a>>,
    allocation_sample: Option<std::rc::Rc<std::cell::Cell<Option<crate::alloc::ThreadStats>>>>,
    worker_allocations: Option<std::rc::Rc<std::cell::RefCell<Vec<crate::WorkerAllocation>>>>,
    case: Case,
    work: Work<'a>,
    sampling: Sampling,
    comparison: crate::history::ComparisonOptions,
    bootstrap: crate::bootstrap::Options,
    summary_scale: Option<crate::viz::charts::AxisScale>,
    summary_family: Option<String>,
    quick: Option<Option<crate::QuickConfig>>,
    operations_per_iteration: u64,
    runtime_workers: Option<std::rc::Rc<std::cell::Cell<usize>>>,
    max_batch: u64,
    executed: bool,
    caller_timed: bool,
    overhead: Option<bool>,
    timer_kind: Option<crate::timer::TimerKind>,
    source: Option<(String, u32, u32)>,
    verify: Option<Box<dyn FnMut() -> Result<()> + 'a>>,
}
/// Human console detail. Structured output and persisted artifacts are unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsoleOutput {
    Quiet,
    #[default]
    Normal,
    Verbose,
}
/// Registration is lazy: `--list` never runs workloads.
pub struct Suite<'a> {
    profiler: Option<Box<dyn crate::profiling::Profiler + 'a>>,
    name: String,
    entries: Vec<Entry<'a>>,
    group_sources: BTreeMap<String, (String, u32, u32)>,
    config: Config,
    quick: Option<crate::QuickConfig>,
    quick_cli: bool,
    timer: crate::timer::Timer,
    timer_cli: Option<crate::timer::TimerKind>,
    overhead: bool,
    overhead_cli: bool,
    comparison_config: crate::history::ComparisonConfig,
    summary_scale: crate::viz::charts::AxisScale,
    plots: bool,
    registration_threads: Option<Vec<usize>>,
    registration_drop: Option<DropPolicy>,
    registration_batch: Option<BatchPolicy>,
    console_output: ConsoleOutput,
    console_color: crate::ConsoleColor,
    console_format: crate::ConsoleFormat,
}
impl<'a> Suite<'a> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            profiler: None,
            name: name.into(),
            entries: vec![],
            group_sources: BTreeMap::new(),
            config: Config::default(),
            quick: None,
            quick_cli: false,
            timer: crate::timer::Timer::os(),
            timer_cli: None,
            overhead: false,
            overhead_cli: false,
            comparison_config: crate::history::ComparisonConfig::default(),
            summary_scale: Default::default(),
            plots: true,
            registration_threads: None,
            registration_drop: None,
            registration_batch: None,
            console_output: ConsoleOutput::Normal,
            console_color: crate::ConsoleColor::Auto,
            console_format: crate::ConsoleFormat::Table,
        }
    }
    #[doc(hidden)]
    pub fn registration_batch_policy(&self) -> BatchPolicy {
        self.registration_batch.unwrap_or(BatchPolicy::Iterations(
            std::num::NonZeroU64::new(64).unwrap(),
        ))
    }
    #[doc(hidden)]
    pub fn has_registration_batch_policy(&self) -> bool {
        self.registration_batch.is_some()
    }
    /// Apply a batch default while registering imported groups.
    pub fn with_batch_defaults(
        &mut self,
        policy: BatchPolicy,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let previous = self.registration_batch.replace(policy);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| register(self)));
        self.registration_batch = previous;
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
        self
    }
    #[doc(hidden)]
    pub fn registration_drop_policy(&self) -> DropPolicy {
        self.registration_drop.unwrap_or(DropPolicy::InsideTiming)
    }
    #[doc(hidden)]
    pub fn has_registration_drop_policy(&self) -> bool {
        self.registration_drop.is_some()
    }
    /// Apply an output destruction default while registering imported groups.
    pub fn with_drop_defaults(
        &mut self,
        policy: DropPolicy,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let previous = self.registration_drop.replace(policy);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| register(self)));
        self.registration_drop = previous;
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
        self
    }
    /// Set the worker matrix used by attribute registration. Call before registering
    /// groups. Sequential cases retain their execution model.
    pub fn registration_threads(&mut self, counts: &[usize]) -> Result<&mut Self> {
        if counts.is_empty() || counts.iter().any(|n| *n > 256) {
            return Err(error("thread matrix requires at least one count in 0..256"));
        }
        self.registration_threads = Some(crate::threads::counts(counts));
        Ok(self)
    }
    #[doc(hidden)]
    pub fn resolve_thread_counts<T: crate::threads::ThreadCounts<M>, M>(
        &self,
        counts: T,
    ) -> Vec<usize> {
        self.registration_threads
            .clone()
            .unwrap_or_else(|| crate::threads::counts(counts))
    }
    /// Configure runtime worker choices before registering attributed cases.
    /// Generated `#[suite]` entrypoints use this so names match the actual matrix.
    pub fn main_registered(mut self, register: impl FnOnce(&mut Self)) -> Result<()> {
        let cli: Vec<String> = std::env::args().skip(1).collect();
        let mut args = crate::sampling::environment_args(&cli, |name| match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(error) => Err(crate::error(error.to_string())),
        })?;
        args.extend(cli);
        if let Some(counts) = crate::threads::from_args(&args)? {
            self.registration_threads(&counts)?;
        }
        register(&mut self);
        self.main()
    }

    /// Select table or hierarchical tree layout for human results and case lists.
    pub fn console_format(&mut self, format: crate::ConsoleFormat) -> &mut Self {
        self.console_format = format;
        self
    }
    /// Terminal color policy. Auto respects stdout TTY detection and NO_COLOR.
    pub fn console_color(&mut self, color: crate::ConsoleColor) -> &mut Self {
        self.console_color = color;
        self
    }
    /// Set console detail; --quiet/--verbose override this default in `main`.
    pub fn console_output(&mut self, output: ConsoleOutput) -> &mut Self {
        self.console_output = output;
        self
    }
    /// Enable or disable charts produced by `main`, without changing measurements.
    /// This is a suite-wide presentation default; CLI --plots/--no-plots override it.
    pub fn plots(&mut self, enabled: bool) -> &mut Self {
        self.plots = enabled;
        self
    }
    /// Register an inline group without changing the surrounding suite's identity.
    pub fn group(&mut self, name: &str, register: impl FnOnce(&mut Self)) -> &mut Self {
        let parent = std::mem::take(&mut self.name);
        self.name = format!("{parent}/{name}");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| register(self)));
        self.name = parent;
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
        self
    }
    /// Register a group with fallback sampling settings. Explicit child settings
    /// take precedence, including those in groups imported across crate boundaries.
    pub fn group_with_sampling(
        &mut self,
        name: &str,
        defaults: Sampling,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        self.group_with_defaults(name, defaults, None, &[], register)
    }
    /// Fallback sampling, ignore and fixed counters for newly registered cases.
    /// Explicit child values, including false and zero, always win.
    pub fn group_with_defaults(
        &mut self,
        name: &str,
        sampling: Sampling,
        ignored: Option<bool>,
        counters: &[(&str, u64)],
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        self.group(name, register);
        for entry in &mut self.entries[first..] {
            entry.sampling.inherit(&sampling);
            let contract = &mut entry.case.contract;
            if let Some(value) = ignored {
                contract
                    .entry("ignored".into())
                    .or_insert_with(|| value.to_string());
            }
            for &(unit, count) in counters {
                let key = format!("work.counter.{unit}");
                if !contract.contains_key(&key)
                    && !contract.contains_key(&format!("work.input.{unit}"))
                {
                    contract.insert(key, count.to_string());
                    if !contract.contains_key("work.unit") {
                        contract.insert("work.unit".into(), unit.into());
                        contract.insert("work.count".into(), count.to_string());
                    }
                }
            }
        }
        self
    }
    /// Mark the last case ignored by default. Selection can include or isolate it.
    pub fn ignore(&mut self, ignored: bool) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry
                .case
                .contract
                .insert("ignored".into(), ignored.to_string());
        }
        self
    }
    /// Resolve registered comparison defaults without executing any workload.
    pub fn comparison_settings(
        &self,
        selection: &crate::Selection,
    ) -> Result<std::collections::BTreeMap<String, crate::history::ComparisonConfig>> {
        selection.validate()?;
        self.selected_indices(selection)
            .into_iter()
            .map(|index| {
                let entry = &self.entries[index];
                Ok((
                    entry.case.id.clone(),
                    entry.comparison.resolve(self.comparison_config)?,
                ))
            })
            .collect()
    }
    pub fn comparison_config(&mut self, config: crate::history::ComparisonConfig) -> &mut Self {
        self.comparison_config = config;
        self
    }
    pub fn bootstrap_case(&mut self, options: crate::bootstrap::Options) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.bootstrap = options;
        }
        self
    }
    pub fn with_bootstrap_defaults(
        &mut self,
        options: crate::bootstrap::Options,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            entry.bootstrap.inherit(options);
        }
        self
    }
    /// Resolve selected analysis defaults with per-field CLI overrides before measurement.
    pub fn bootstrap_settings(
        &self,
        selection: &crate::Selection,
        fallback: &crate::bootstrap::Config,
        overrides: crate::bootstrap::Options,
    ) -> Result<BTreeMap<String, crate::bootstrap::Config>> {
        selection.validate()?;
        self.selected_indices(selection)
            .into_iter()
            .map(|index| {
                let entry = &self.entries[index];
                let mut options = overrides;
                options.inherit(entry.bootstrap);
                Ok((entry.case.id.clone(), options.resolve(fallback)?))
            })
            .collect()
    }
    pub fn comparison_case(&mut self, options: crate::history::ComparisonOptions) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.comparison = options;
        }
        self
    }
    pub fn with_comparison_defaults(
        &mut self,
        options: crate::history::ComparisonOptions,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            entry.comparison.inherit(options);
        }
        self
    }
    /// Declare the last case's input-summary family. Names are scoped to the
    /// current group so unrelated groups cannot accidentally share a line.
    pub fn summary_family(&mut self, name: &str) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.summary_family = Some(format!("{}/{name}", self.name));
        }
        self
    }
    /// Fallback family name for newly registered cases, scoped per child group.
    pub fn with_summary_family_defaults(
        &mut self,
        name: &str,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            if entry.summary_family.is_none() {
                entry.summary_family = Some(format!("{}/{name}", entry.ordering_group));
            }
        }
        self
    }
    /// Default summary axis for this suite; explicit case/group settings win.
    pub fn summary_scale(&mut self, scale: crate::viz::charts::AxisScale) -> &mut Self {
        self.summary_scale = scale;
        self
    }
    /// Override the summary scale of the most recently registered case.
    pub fn summary_scale_case(&mut self, scale: crate::viz::charts::AxisScale) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.summary_scale = Some(scale);
        }
        self
    }
    /// Fallback for cases registered by this closure, including imported groups.
    /// Inner defaults and explicit linear overrides take precedence.
    pub fn with_summary_scale_defaults(
        &mut self,
        scale: crate::viz::charts::AxisScale,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            entry.summary_scale.get_or_insert(scale);
        }
        self
    }
    /// Resolve presentation settings without running any workloads.
    pub fn summary_scales(
        &self,
        selection: &crate::Selection,
    ) -> Result<std::collections::BTreeMap<String, crate::viz::charts::AxisScale>> {
        selection.validate()?;
        Ok(self
            .selected_indices(selection)
            .into_iter()
            .map(|i| {
                let entry = &self.entries[i];
                (
                    entry.case.id.clone(),
                    entry.summary_scale.unwrap_or(self.summary_scale),
                )
            })
            .collect())
    }
    /// Select a validated clock for measured intervals. Budgets use OS time.
    pub fn timer(&mut self, timer: crate::timer::Timer) -> &mut Self {
        self.timer = timer;
        self
    }
    /// Also emit wall.adjusted after subtracting calibrated harness costs.
    pub fn compensate_overhead(&mut self, enabled: bool) -> &mut Self {
        self.overhead = enabled;
        self
    }
    /// Override compensation for the last case, including an explicit disable.
    pub fn compensate_case(&mut self, enabled: bool) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.overhead = Some(enabled);
        }
        self
    }
    /// Fill missing compensation settings on newly registered imported cases.
    pub fn with_overhead_defaults(
        &mut self,
        enabled: bool,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            entry.overhead.get_or_insert(enabled);
        }
        self
    }
    fn overhead_for(&self, entry: &Entry<'_>) -> bool {
        if self.overhead_cli {
            self.overhead
        } else {
            entry.overhead.unwrap_or(self.overhead)
        }
    }
    /// Select a clock lazily for the last registered case.
    pub fn timer_case(&mut self, kind: crate::timer::TimerKind) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.timer_kind = Some(kind);
        }
        self
    }
    /// Imported groups inherit only when no nearer default or case setting exists.
    pub fn with_timer_defaults(
        &mut self,
        kind: crate::timer::TimerKind,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            entry.timer_kind.get_or_insert(kind);
        }
        self
    }
    fn timer_name(&self, entry: &Entry<'_>) -> &'static str {
        self.timer_cli
            .or(entry.timer_kind)
            .map_or(self.timer.name(), |kind| kind.name())
    }
    fn resolve_timers(&self, indices: &[usize]) -> Result<BTreeMap<usize, crate::timer::Timer>> {
        indices
            .iter()
            .map(|&index| {
                let timer = match self.timer_cli.or(self.entries[index].timer_kind) {
                    Some(kind) => kind.resolve()?,
                    None => self.timer,
                };
                Ok((index, timer))
            })
            .collect()
    }
    pub fn config(&mut self, config: Config) -> &mut Self {
        self.config = config;
        self
    }
    /// Enable adaptive batches for this suite. Skips calibration/warmup, retains
    /// real samples, and stops after a stable adjacent pair or a wall-time limit.
    pub fn quick(&mut self, config: crate::QuickConfig) -> &mut Self {
        self.quick = Some(config);
        self
    }
    /// Override quick mode for the last case; None explicitly disables it.
    pub fn quick_case(&mut self, config: Option<crate::QuickConfig>) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.quick = Some(config);
        }
        self
    }
    /// Fill missing quick settings on newly registered imported cases.
    pub fn with_quick_defaults(
        &mut self,
        config: Option<crate::QuickConfig>,
        register: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let first = self.entries.len();
        register(self);
        for entry in &mut self.entries[first..] {
            entry.quick.get_or_insert(config);
        }
        self
    }
    fn quick_for(&self, entry: &Entry<'_>) -> Option<crate::QuickConfig> {
        if self.quick_cli {
            self.quick
        } else {
            entry.quick.unwrap_or(self.quick)
        }
    }
    fn validate_case_quick(&self, entry: &Entry<'_>, test_mode: bool) -> Result<()> {
        if let Some(quick) = self.quick_for(entry) {
            quick.validate()?;
            let cold = entry
                .case
                .contract
                .get("temperature")
                .is_some_and(|v| v.starts_with("cold;"));
            if !test_mode
                && !cold
                && (entry.sampling.iterations.is_some()
                    || entry
                        .sampling
                        .mode
                        .is_some_and(|m| m != crate::SamplingMode::Flat))
            {
                return Err(error(
                    "adaptive quick mode cannot combine with fixed iterations or linear/auto sampling",
                ));
            }
        }
        Ok(())
    }
    pub fn bench<O: 'a>(&mut self, name: &str, mut f: impl FnMut() -> O + 'a) -> &mut Self {
        self.register(
            name,
            "reused closure state; output drop included",
            1_048_576,
            Box::new(move |n, timer| {
                let start = timer.start();
                for _ in 0..n {
                    black_box(f());
                }
                start.measure(n)
            }),
        );
        self
    }
    /// Measure a local synchronous function and this allocator's events over
    /// the same operation loop. Output destruction is included. Allocations on
    /// other threads and report construction are excluded.
    pub fn bench_allocated<A: std::alloc::GlobalAlloc + 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        mut f: impl FnMut() -> O + 'a,
    ) -> &mut Self {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        self.register_fallible(
            name,
            "local allocation-counted loop; output drop included",
            1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let phase = allocator.begin_thread_phase()?;
                let start = timer.start();
                for _ in 0..n {
                    black_box(f());
                }
                let elapsed = start.elapsed_ns()?;
                let counts = phase.finish();
                let elapsed = timer.measured(elapsed, n, Some(&counts))?;
                if counts.overflowed {
                    return Err(error("thread allocation counters overflowed"));
                }
                captured.set(Some(counts));
                Ok(elapsed)
            }),
        );
        self.attach_allocation_sample(sample);
        self
    }
    /// Count allocator events on the thread polling each future. Executor
    /// creation is lazy and excluded; future creation, polling and output drop
    /// are included. Tasks spawned on other threads are not counted.
    pub fn bench_async_allocated<A, E, O>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        mut executor: impl FnMut() -> E + 'a,
        mut f: impl AsyncFnMut() -> O + 'a,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + 'a,
        E: crate::workloads::Executor + 'a,
        O: 'a,
    {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        let mut state = None;
        self.register_fallible(
            name,
            "async allocation-counted loop; output drop included",
            1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let executor = state.get_or_insert_with(&mut executor);
                let phase = allocator.begin_thread_phase()?;
                let start = timer.start();
                for _ in 0..n {
                    black_box(executor.block_on(f()));
                }
                let elapsed = start.elapsed_ns()?;
                let counts = phase.finish();
                let elapsed = timer.measured(elapsed, n, Some(&counts))?;
                if counts.overflowed {
                    return Err(error("thread allocation counters overflowed"));
                }
                captured.set(Some(counts));
                Ok(elapsed)
            }),
        );
        self.attach_allocation_sample(sample);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("async.executor".into(), std::any::type_name::<E>().into());
        contract.insert("async.scope".into(), "lazy executor construction excluded; future creation/polling and output drop included; calling-thread allocations only".into());
        self
    }
    /// Fresh borrowed inputs with allocation accounting restricted to the
    /// measured batches. Setup/input destruction and buffer allocation excluded.
    pub fn bench_allocated_batched_ref<A: std::alloc::GlobalAlloc + 'a, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(&mut I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        self.register_fallible(
            name,
            "fresh input; setup/input drop excluded; allocated batches",
            1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let (elapsed, counts) = allocator.measure_input_batch(
                    timer,
                    n,
                    batch.size(n),
                    &mut setup,
                    &mut f,
                    drop,
                )?;
                captured.set(Some(counts));
                Ok(elapsed)
            }),
        );
        self.attach_allocation_sample(sample);
        self.batch_contract(drop, batch);
        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(), "current-thread measured batches; setup/input drop and buffers excluded; output drop follows output.drop; peak is maximum batch net growth".into());
        self
    }
    /// Fresh owned input. Destruction performed by the operation is counted.
    pub fn bench_allocated_batched<A: std::alloc::GlobalAlloc + 'a, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self {
        self.bench_allocated_batched_ref(
            name,
            allocator,
            move || Some(setup()),
            move |input| f(input.take().expect("fresh owned input")),
            drop,
            batch,
        );
        self.owned_input_contract(drop);
        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(), "current-thread measured batches; setup and buffers excluded; consumed input drop included; output drop follows output.drop; peak is maximum batch net growth".into());
        self
    }
    /// Async allocation-counted batches. Executor construction, setup and harness
    /// buffers are excluded; future polling is counted on the calling thread.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_async_allocated_batched_ref<A, E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        mut executor: impl FnMut() -> E + 'a,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl for<'i> AsyncFnMut(&'i mut I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + 'a,
        E: crate::workloads::Executor + 'a,
    {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        let mut state = None;

        self.register_fallible(
            name,
            "async fresh input; allocation-counted batches",
            1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let executor = state.get_or_insert_with(&mut executor);
                let (elapsed, counts) = allocator.measure_input_batch(
                    timer,
                    n,
                    batch.size(n),
                    &mut setup,
                    &mut |input| executor.block_on(f(input)),
                    drop,
                )?;
                captured.set(Some(counts));
                Ok(elapsed)
            }),
        );
        self.attach_allocation_sample(sample);
        self.async_batch_contract::<E>(drop, batch);

        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(),
            "polling-thread measured batches; executor construction, setup and buffers excluded; input drop excluded; output drop follows output.drop; peak is maximum batch net growth".into());
        self
    }
    /// Async allocation-counted batches. Executor construction, setup and harness
    /// buffers are excluded; future polling is counted on the calling thread.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_async_allocated_batched<A, E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        mut executor: impl FnMut() -> E + 'a,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl AsyncFnMut(I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + 'a,
        E: crate::workloads::Executor + 'a,
    {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        let mut state = None;
        let mut setup = move || Some(setup());
        self.register_fallible(
            name,
            "async fresh input; allocation-counted batches",
            1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let executor = state.get_or_insert_with(&mut executor);
                let (elapsed, counts) = allocator.measure_input_batch(
                    timer,
                    n,
                    batch.size(n),
                    &mut setup,
                    &mut |input| executor.block_on(f(input.take().expect("fresh owned input"))),
                    drop,
                )?;
                captured.set(Some(counts));
                Ok(elapsed)
            }),
        );
        self.attach_allocation_sample(sample);
        self.async_batch_contract::<E>(drop, batch);
        self.owned_input_contract(drop);
        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(),
            "polling-thread measured batches; executor construction, setup and buffers excluded; consumed input drop included; output drop follows output.drop; peak is maximum batch net growth".into());
        self
    }
    fn attach_allocation_sample(
        &mut self,
        sample: std::rc::Rc<std::cell::Cell<Option<crate::alloc::ThreadStats>>>,
    ) {
        let entry = self.entries.last_mut().unwrap();
        entry.allocation_sample = Some(sample);
        entry.case.contract.insert(
            "alloc.scope".into(),
            "current-thread allocator events during operation loop; output drop included".into(),
        );
        for (id, unit, statistic) in crate::alloc::ThreadStats::METRICS {
            let mut metric =
                Metric::duration(id, "current thread Rust allocator events", statistic);
            metric.unit = unit.into();
            entry.case.metrics.push(metric);
        }
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
            Box::new(move |n, timer| input_batch(timer, n, &mut setup, &mut f, drop)),
        );
        self
    }
    /// Fresh owned input, created outside timing and moved into the operation.
    /// Input destruction performed by the operation remains inside timing.
    pub fn bench_with_owned_input<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(I) -> O + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        self.register(
            name,
            "fresh owned input; setup excluded; consumed input drop inside operation",
            1_048_576,
            Box::new(move |n, timer| {
                crate::suite_measure::owned_input_batch(timer, n, &mut setup, &mut f, drop)
            }),
        );
        self.owned_input_contract(drop);
        self
    }
    /// Fresh borrowed inputs with an explicit memory/timer-overhead tradeoff.
    pub fn bench_batched_ref<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(&mut I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self {
        self.register(
            name,
            "fresh borrowed input; setup/input drop excluded; explicit batch policy",
            1_048_576,
            Box::new(move |n, timer| {
                crate::suite_measure::input_batch_sized(
                    timer,
                    n,
                    &mut setup,
                    &mut f,
                    drop,
                    batch.size(n),
                )
            }),
        );
        self.batch_contract(drop, batch);
        self
    }
    /// Fresh inputs moved into the operation with an explicit batch policy.
    pub fn bench_batched<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self {
        self.register(
            name,
            "fresh owned input; setup excluded; consumed input drop inside operation",
            1_048_576,
            Box::new(move |n, timer| {
                crate::suite_measure::owned_input_batch_sized(
                    timer,
                    n,
                    &mut setup,
                    &mut f,
                    drop,
                    batch.size(n),
                )
            }),
        );
        self.owned_input_contract(drop);
        self.batch_contract(drop, batch);
        self
    }
    fn batch_contract(&mut self, drop: DropPolicy, batch: BatchPolicy) {
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("batch.policy".into(), format!("{batch:?}"));
        contract.insert(
            "output.drop".into(),
            match drop {
                DropPolicy::InsideTiming => "inside",
                DropPolicy::OutsideTiming => "outside",
            }
            .into(),
        );
    }
    fn owned_input_contract(&mut self, drop: DropPolicy) {
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert(
            "input.ownership".into(),
            "moved; setup excluded; consumed input drop inside operation".into(),
        );
        contract.insert(
            "output.drop".into(),
            match drop {
                DropPolicy::InsideTiming => "inside",
                DropPolicy::OutsideTiming => "outside",
            }
            .into(),
        );
    }
    /// Async owned inputs with lazy executor construction outside the timed loop.
    pub fn bench_async_with_owned_input<E, I, F, O>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl FnMut(I) -> F + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
        I: 'a,
        F: std::future::Future<Output = O> + 'a,
        O: 'a,
    {
        let mut state = None;
        self.register(
            name,
            "fresh owned input; setup excluded; consumed input drop inside operation",
            1_048_576,
            Box::new(move |n, timer| {
                let executor = state.get_or_insert_with(&mut executor);
                crate::suite_measure::owned_input_batch(
                    timer,
                    n,
                    &mut setup,
                    &mut |input| executor.block_on(f(input)),
                    drop,
                )
            }),
        );
        self.owned_input_contract(drop);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("async.executor".into(), std::any::type_name::<E>().into());
        contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; future creation + polling included".into(),
        );
        self
    }
    /// Async owned inputs with explicit batching; each future finishes before the next starts.
    pub fn bench_async_batched<E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        setup: impl FnMut() -> I + 'a,
        mut f: impl AsyncFnMut(I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
    {
        // Initialize before entering the synchronous batch helper's timer.
        let mut state = None;
        let mut setup = setup;
        self.register(
            name,
            "fresh owned input; setup excluded; consumed input drop inside operation",
            1_048_576,
            Box::new(move |n, timer| {
                let executor = state.get_or_insert_with(&mut executor);
                crate::suite_measure::owned_input_batch_sized(
                    timer,
                    n,
                    &mut setup,
                    &mut |input| executor.block_on(f(input)),
                    drop,
                    batch.size(n),
                )
            }),
        );
        self.owned_input_contract(drop);
        self.async_batch_contract::<E>(drop, batch);
        self
    }

    /// Async borrowed inputs with explicit batching; setup and input drop are excluded.
    pub fn bench_async_batched_ref<E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl for<'i> AsyncFnMut(&'i mut I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
    {
        let mut state = None;
        self.register(
            name,
            "fresh borrowed input; setup/input drop excluded; explicit batch policy",
            1_048_576,
            Box::new(move |n, timer| {
                let executor = state.get_or_insert_with(&mut executor);
                crate::suite_measure::input_batch_sized(
                    timer,
                    n,
                    &mut setup,
                    &mut |input: &mut I| executor.block_on(f(input)),
                    drop,
                    batch.size(n),
                )
            }),
        );
        self.async_batch_contract::<E>(drop, batch);
        self
    }
    fn async_batch_contract<E>(&mut self, drop: DropPolicy, batch: BatchPolicy) {
        self.batch_contract(drop, batch);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("async.executor".into(), std::any::type_name::<E>().into());
        contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; future creation + polling included".into(),
        );
    }
    /// Concurrent owned inputs. Preparation is outside timing, consumption is inside.
    pub fn bench_threads_with_owned_input<I: Send + 'a, O: Send + 'a>(
        &mut self,
        name: &str,
        workers: usize,
        mut setup: impl FnMut() -> I + 'a,
        f: impl Fn(I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        self.bench_threads_with_input(
            name,
            workers,
            move || Some(setup()),
            move |input| f(input.take().expect("fresh input")),
            drop,
        );
        self.entries.last_mut().unwrap().case.contract.insert(
            "lifecycle".into(),
            "concurrent fresh owned input; setup excluded; consumed input drop inside operation"
                .into(),
        );
        self.owned_input_contract(drop);
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
            Box::new(move |n, timer| {
                // Borrow once per batch, before timing; no RefCell checks in the measured loop.
                let mut setup = s.borrow_mut();
                let mut f = work.borrow_mut();
                input_batch(timer, n, &mut *setup, &mut *f, drop)
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
        self.register_fallible(name, lifecycle, max_batch, work);
    }
    fn register_fallible(&mut self, name: &str, lifecycle: &str, max_batch: u64, work: Work<'a>) {
        let contract = BTreeMap::from([
            ("lifecycle".into(), lifecycle.into()),
            ("completion".into(), "synchronous function return".into()),
        ]);
        self.entries.push(Entry {
            runtime_workers: None,
            comparison: crate::history::ComparisonOptions::default(),
            bootstrap: Default::default(),
            summary_scale: None,
            summary_family: None,
            ordering_group: self.name.clone(),
            ordering_family: None,
            ordering_name: name.into(),
            input_counters: None,
            measurement_sample: None,
            formatters: BTreeMap::new(),
            allocation_sample: None,
            worker_allocations: None,
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
            sampling: Sampling::default(),
            quick: None,
            operations_per_iteration: 1,
            max_batch,
            executed: false,
            caller_timed: false,
            overhead: None,
            timer_kind: None,
            source: None,
            verify: None,
        });
    }
    /// Group the last case into a generic family for kind ordering.
    /// This is display metadata: it does not change case IDs or measurement contracts.
    pub fn ordering_family(&mut self, name: &str) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.ordering_family = Some(name.into());
        }
        self
    }
    /// Set the source location of the current group. Attribute groups supply this
    /// automatically; builders can call it inside `group` registration closures.
    pub fn group_source_location(&mut self, file: &str, line: u32, column: u32) -> &mut Self {
        self.group_sources
            .insert(self.name.clone(), (file.into(), line, column));
        self
    }
    /// Set display ordering metadata for the last case. Attributes supply this automatically.
    /// Source locations do not enter measurement contracts or affect result compatibility.
    pub fn source_location(&mut self, file: &str, line: u32, column: u32) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.source = Some((file.into(), line, column));
        }
        self
    }
    /// Configure the last case. CLI sampling flags override these defaults.
    pub fn sampling(&mut self, sampling: Sampling) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            entry.sampling = sampling;
        }
        self
    }

    /// Set presentation for a metric of the last registered case.
    pub fn formatter(
        &mut self,
        metric: &str,
        formatter: impl crate::measurement::ValueFormatter + 'a,
    ) -> Result<&mut Self> {
        let entry = self
            .entries
            .last_mut()
            .ok_or_else(|| error("register a case before its formatter"))?;
        if !entry.case.metrics.iter().any(|m| m.id == metric) {
            return Err(error("formatter metric is not registered on this case"));
        }
        entry.formatters.insert(metric.into(), Box::new(formatter));
        Ok(self)
    }

    /// Apply registered formatters without modifying raw data. Loaded runs must
    /// have the same complete metric contract as the registered formatter target.
    /// Absolute bootstrap estimates in registered formatter units. Raw statistical
    /// output remains separate; regression and outlier graphs keep their raw units.
    pub fn formatted_bootstrap(
        &self,
        report: &crate::bootstrap::Report,
    ) -> Result<crate::bootstrap::Report> {
        let mut result = report.clone();
        result.rows.clear();
        for entry in &self.entries {
            for (id, formatter) in &entry.formatters {
                let metric = entry
                    .case
                    .metrics
                    .iter()
                    .find(|metric| metric.id == *id)
                    .ok_or_else(|| error("formatted metric absent from registered case"))?;
                result.rows.extend(
                    crate::measurement::format_bootstrap_metric(
                        report,
                        &entry.case.id,
                        metric,
                        formatter.as_ref(),
                    )?
                    .rows,
                );
            }
        }
        Ok(result)
    }

    pub fn formatted_metrics(&self, run: &Run) -> Result<Vec<crate::measurement::FormattedMetric>> {
        let mut result = Vec::new();
        for case in &run.cases {
            if let Some(entry) = self.entries.iter().find(|e| e.case.id == case.id) {
                for (id, formatter) in &entry.formatters {
                    let metric = case
                        .metrics
                        .iter()
                        .find(|m| m.id == *id)
                        .ok_or_else(|| error("formatted metric absent from loaded run"))?;
                    if entry.case.metrics.iter().find(|m| m.id == *id) != Some(metric) {
                        return Err(error("formatter metric contract differs from loaded run"));
                    }
                    result.push(crate::measurement::FormattedMetric {
                        case: case.id.clone(),
                        metric: metric.clone(),
                        throughput: crate::report::work_counters(case)?
                            .into_iter()
                            .map(|(unit, count)| {
                                Ok((
                                    unit.to_string(),
                                    crate::measurement::FormattedThroughput {
                                        work_per_operation: count,
                                        observations: crate::measurement::format_observations(
                                            run,
                                            &case.id,
                                            id,
                                            formatter.as_ref(),
                                            match count {
                                                Some(count) => {
                                                    crate::measurement::Format::Throughput {
                                                        work: count as f64,
                                                        unit,
                                                    }
                                                }
                                                None => {
                                                    crate::measurement::Format::InputThroughput {
                                                        unit,
                                                    }
                                                }
                                            },
                                        )?,
                                    },
                                ))
                            })
                            .collect::<Result<_>>()?,
                        human: crate::measurement::format_observations(
                            run,
                            &case.id,
                            id,
                            formatter.as_ref(),
                            crate::measurement::Format::Human,
                        )?,
                        machine: crate::measurement::format_observations(
                            run,
                            &case.id,
                            id,
                            formatter.as_ref(),
                            crate::measurement::Format::Machine,
                        )?,
                    });
                }
            }
        }
        Ok(result)
    }

    /// Measure a synchronous workload with a custom value alongside wall time.
    /// Output destruction is included. Each wave gets its own start/end pair;
    /// the measurement's accumulator combines waves before numeric conversion.
    pub fn bench_measured<M: crate::measurement::Measurement + 'a, O: 'a>(
        &mut self,
        name: &str,
        measurement: M,
        batch: BatchPolicy,
        mut work: impl FnMut() -> O + 'a,
    ) -> Result<&mut Self> {
        self.bench_measured_with_input(
            name,
            measurement,
            batch,
            || (),
            move |_| work(),
            DropPolicy::InsideTiming,
        )
    }

    /// Custom measurements of sequential async operations without input setup.
    pub fn bench_async_measured<M, E, O>(
        &mut self,
        name: &str,
        measurement: M,
        batch: BatchPolicy,
        executor: impl FnMut() -> E + 'a,
        mut work: impl AsyncFnMut() -> O + 'a,
    ) -> Result<&mut Self>
    where
        M: crate::measurement::Measurement + 'a,
        E: crate::workloads::Executor + 'a,
        O: 'a,
    {
        self.bench_async_measured_with_input(
            name,
            measurement,
            batch,
            executor,
            || (),
            async move |_| work().await,
            DropPolicy::InsideTiming,
        )
    }

    /// Async custom measurement with fresh owned inputs. Returned values follow
    /// output policy; inputs destroyed by the future are part of the measurement.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_async_measured_with_owned_input<M, E, I, O>(
        &mut self,
        name: &str,
        measurement: M,
        batch: BatchPolicy,
        executor: impl FnMut() -> E + 'a,
        mut setup: impl FnMut() -> I + 'a,
        mut work: impl AsyncFnMut(I) -> O + 'a,
        drop_output: DropPolicy,
    ) -> Result<&mut Self>
    where
        M: crate::measurement::Measurement + 'a,
        E: crate::workloads::Executor + 'a,
        I: 'a,
        O: 'a,
    {
        self.bench_async_measured_with_input(
            name,
            measurement,
            batch,
            executor,
            move || Some(setup()),
            async move |input| work(input.take().expect("fresh measured input")).await,
            drop_output,
        )?;
        self.owned_input_contract(drop_output);
        self.entries.last_mut().unwrap().case.contract.insert("lifecycle".into(),
            "async custom measurement; setup outside; consumed input destruction inside; returned output follows policy".into());
        Ok(self)
    }

    /// Custom measurements of async operations on fresh borrowed inputs.
    /// Executor creation is lazy and outside both clocks. Futures complete sequentially.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_async_measured_with_input<M, E, I, O>(
        &mut self,
        name: &str,
        measurement: M,
        batch: BatchPolicy,
        mut executor: impl FnMut() -> E + 'a,
        setup: impl FnMut() -> I + 'a,
        mut work: impl for<'i> AsyncFnMut(&'i mut I) -> O + 'a,
        drop_output: DropPolicy,
    ) -> Result<&mut Self>
    where
        M: crate::measurement::Measurement + 'a,
        E: crate::workloads::Executor + 'a,
        I: 'a,
        O: 'a,
    {
        let state = std::rc::Rc::new(std::cell::RefCell::new(None::<E>));
        let execution = state.clone();
        self.bench_measured_with_input(
            name,
            measurement,
            batch,
            setup,
            move |input| {
                execution
                    .borrow_mut()
                    .as_mut()
                    .expect("initialized executor")
                    .block_on(work(input))
            },
            drop_output,
        )?;
        let entry = self.entries.last_mut().unwrap();
        let mut measured = std::mem::replace(&mut entry.work, Box::new(|_, _| unreachable!()));
        entry.work = Box::new(move |n, timer| {
            state.borrow_mut().get_or_insert_with(&mut executor);
            measured(n, timer)
        });
        self.async_batch_contract::<E>(drop_output, batch);
        Ok(self)
    }

    /// Move a fresh input into each operation. Setup is excluded; destruction by
    /// the operation is included. Returned input destruction follows output policy.
    pub fn bench_measured_with_owned_input<
        M: crate::measurement::Measurement + 'a,
        I: 'a,
        O: 'a,
    >(
        &mut self,
        name: &str,
        measurement: M,
        batch: BatchPolicy,
        mut setup: impl FnMut() -> I + 'a,
        mut work: impl FnMut(I) -> O + 'a,
        drop_output: DropPolicy,
    ) -> Result<&mut Self> {
        self.bench_measured_with_input(
            name,
            measurement,
            batch,
            move || Some(setup()),
            move |input| work(input.take().expect("fresh measured input")),
            drop_output,
        )?;
        self.owned_input_contract(drop_output);
        self.entries.last_mut().unwrap().case.contract.insert(
            "lifecycle".into(),
            "custom measurement waves; setup outside; consumed input destruction inside; returned output follows policy".into(),
        );
        Ok(self)
    }

    /// Fresh mutable inputs with setup and input destruction outside both measurements.
    /// Output destruction follows `drop_output`. Each wave accumulates one custom value.
    pub fn bench_measured_with_input<M: crate::measurement::Measurement + 'a, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        mut measurement: M,
        batch: BatchPolicy,
        mut setup: impl FnMut() -> I + 'a,
        mut work: impl FnMut(&mut I) -> O + 'a,
        drop_output: DropPolicy,
    ) -> Result<&mut Self> {
        let metric = measurement.metric();
        if metric.id.is_empty()
            || metric.id == "wall"
            || metric.id.starts_with("wall.")
            || metric.id.starts_with("alloc.")
            || metric.statistic != "batch_total"
        {
            return Err(error(
                "custom measurement requires a distinct metric id and batch_total statistic",
            ));
        }
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        self.register(
            name,
            "custom measurement waves; fresh input setup/drop outside; output destruction follows policy",
            1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let mut total = measurement.zero();
                let mut elapsed = Measured::default();
                let mut remaining = n;
                let wave_size = batch.size(n);
                while remaining > 0 {
                    let count = remaining.min(wave_size);
                    let mut inputs: Vec<_> = (0..count).map(|_| setup()).collect();
                    let mut outputs = match drop_output {
                        DropPolicy::InsideTiming => Vec::new(),
                        DropPolicy::OutsideTiming => Vec::with_capacity(inputs.len()),
                    };
                    let wall = timer.start();
                    let start = measurement.start()?;
                    for input in &mut inputs {
                        let output = std::hint::black_box(work(std::hint::black_box(input)));
                        match drop_output {
                            DropPolicy::InsideTiming => drop(output),
                            DropPolicy::OutsideTiming => outputs.push(output),
                        }
                    }
                    let value = measurement.end(start)?;
                    elapsed = elapsed.add(wall.measure(count)?)?;
                    drop(outputs);
                    drop(inputs);
                    total = measurement.add(total, value)?;
                    remaining -= count;
                }
                let value = measurement.to_f64(&total)?;
                if !value.is_finite() || value < 0.0 {
                    return Err(error(
                        "custom measurement must convert to a finite nonnegative value",
                    ));
                }
                captured.set(Some(value));
                Ok(elapsed)
            }),
        );
        let entry = self.entries.last_mut().unwrap();
        entry.measurement_sample = Some((metric.id.clone(), sample));
        entry.case.metrics.push(metric);
        entry
            .case
            .contract
            .insert("measurement.type".into(), std::any::type_name::<M>().into());
        entry
            .case
            .contract
            .insert("measurement.batch_policy".into(), format!("{batch:?}"));
        self.batch_contract(drop_output, batch);
        Ok(self)
    }

    /// A caller-defined batch returning a typed custom total for exactly `n`
    /// operations. Measurement start/end/zero/add are not invoked. Wall time is
    /// measured independently for scheduling; conversion is outside that interval.
    pub fn bench_measured_custom<M: crate::measurement::Measurement + 'a>(
        &mut self,
        name: &str,
        measurement: M,
        mut measure: impl FnMut(u64) -> M::Value + 'a,
    ) -> Result<&mut Self> {
        let metric = measurement.metric();
        if metric.id.is_empty()
            || metric.id == "wall"
            || metric.id.starts_with("wall.")
            || metric.id.starts_with("alloc.")
            || metric.statistic != "batch_total"
        {
            return Err(error(
                "custom measurement requires a distinct metric id and batch_total statistic",
            ));
        }
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        self.register(name, "caller-defined custom value; caller owns measurement boundaries; wall measures complete batch", 1_048_576,
            Box::new(move |n, timer| {
                captured.set(None);
                let wall = timer.start();
                let total = measure(n);
                let elapsed = wall.measure(n)?;
                let value = measurement.to_f64(&total)?;
                if !value.is_finite() || value < 0.0 {
                    return Err(error("custom measurement must convert to a finite nonnegative value"));
                }
                captured.set(Some(value));
                Ok(elapsed)
            }));
        let entry = self.entries.last_mut().unwrap();
        entry.measurement_sample = Some((metric.id.clone(), sample));
        entry.case.metrics.push(metric);
        entry
            .case
            .contract
            .insert("measurement.type".into(), std::any::type_name::<M>().into());
        entry
            .case
            .contract
            .insert("measurement.batch_policy".into(), "caller".into());
        Ok(self)
    }

    /// Async caller-defined custom total. Executor construction is lazy and
    /// excluded from the independent batch timer; the future completes before conversion.
    pub fn bench_async_measured_custom<M, E>(
        &mut self,
        name: &str,
        measurement: M,
        mut executor: impl FnMut() -> E + 'a,
        mut measure: impl AsyncFnMut(u64) -> M::Value + 'a,
    ) -> Result<&mut Self>
    where
        M: crate::measurement::Measurement + 'a,
        E: crate::workloads::Executor + 'a,
    {
        let state = std::rc::Rc::new(std::cell::RefCell::new(None::<E>));
        let execution = state.clone();
        self.bench_measured_custom(name, measurement, move |n| {
            execution
                .borrow_mut()
                .as_mut()
                .expect("initialized executor")
                .block_on(measure(n))
        })?;
        let entry = self.entries.last_mut().unwrap();
        let mut measured = std::mem::replace(&mut entry.work, Box::new(|_, _| unreachable!()));
        entry.work = Box::new(move |n, timer| {
            state.borrow_mut().get_or_insert_with(&mut executor);
            measured(n, timer)
        });
        entry
            .case
            .contract
            .insert("async.executor".into(), std::any::type_name::<E>().into());
        entry.case.contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; caller future completes before conversion".into(),
        );
        Ok(self)
    }

    /// A caller-timed batch: execute exactly `n` operations and return their total duration.
    /// Setup, coordination and synchronization are the caller's responsibility.
    pub fn bench_custom(
        &mut self,
        name: &str,
        mut measure: impl FnMut(u64) -> Duration + 'a,
    ) -> &mut Self {
        self.register(
            name,
            "caller-timed batch; caller defines setup/drop/completion boundaries",
            1_048_576,
            Box::new(move |n, _timer| Ok(Measured::raw(measure(n).as_nanos()))),
        );
        self.entries.last_mut().unwrap().caller_timed = true;
        self.entries.last_mut().unwrap().case.metrics[0] =
            Metric::duration("wall", "caller-reported duration", "batch_total");
        self
    }

    /// An asynchronous caller-timed batch. The future must complete exactly `n`
    /// operations and return their total duration. Executor construction is lazy;
    /// the caller owns all reported setup, polling and destruction boundaries.
    pub fn bench_async_custom<E>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        mut measure: impl AsyncFnMut(u64) -> Duration + 'a,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
    {
        let mut state = None;
        self.bench_custom(name, move |n| {
            state.get_or_insert_with(&mut executor).block_on(measure(n))
        });
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("async.executor".into(), std::any::type_name::<E>().into());
        contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; future completed; caller defines reported interval".into(),
        );
        self
    }

    /// Lazy executor construction, before any timed loop. Futures run to completion.
    pub fn bench_async_factory<E, F, O>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        mut future: impl FnMut() -> F + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
        F: std::future::Future<Output = O> + 'a,
        O: 'a,
    {
        let mut state = None;
        self.register(
            name,
            crate::suite_measure::input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let executor = state.get_or_insert_with(&mut executor);
                input_batch(
                    timer,
                    n,
                    &mut || (),
                    &mut |_: &mut ()| executor.block_on(future()),
                    drop,
                )
            }),
        );
        self.entries
            .last_mut()
            .unwrap()
            .case
            .contract
            .insert("async.executor".into(), std::any::type_name::<E>().into());
        self.entries.last_mut().unwrap().case.contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; future creation + polling included".into(),
        );
        self
    }

    /// Async operation borrowing one owned argument repeatedly, without cloning it.
    /// The argument and executor are dropped outside the measured batches.
    pub fn bench_async_with_value<E, I, O>(
        &mut self,
        name: &str,
        executor: impl FnMut() -> E + 'a,
        input: I,
        f: impl for<'i> AsyncFnMut(&'i I) -> O + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
        I: 'a,
        O: 'a,
    {
        self.bench_async_with_value_batched(
            name,
            executor,
            input,
            f,
            drop,
            BatchPolicy::Iterations(std::num::NonZeroU64::new(64).unwrap()),
        )
    }

    /// Reuse an owned async argument with an explicit output batch policy.
    pub fn bench_async_with_value_batched<E, I, O>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        input: I,
        mut f: impl for<'i> AsyncFnMut(&'i I) -> O + 'a,
        drop: DropPolicy,
        batch: BatchPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
        I: 'a,
        O: 'a,
    {
        let mut state = None;
        self.register(
            name,
            "reused borrowed input; input drop excluded",
            1_048_576,
            Box::new(move |n, timer| {
                let executor = state.get_or_insert_with(&mut executor);
                crate::suite_measure::input_batch_sized(
                    timer,
                    n,
                    &mut || (),
                    &mut |_: &mut ()| executor.block_on(f(black_box(&input))),
                    drop,
                    batch.size(n),
                )
            }),
        );
        let entry = self.entries.last_mut().unwrap();
        entry
            .case
            .contract
            .insert("async.executor".into(), std::any::type_name::<E>().into());
        entry.case.contract.insert(
            "output.drop".into(),
            match drop {
                DropPolicy::InsideTiming => "inside",
                DropPolicy::OutsideTiming => "outside",
            }
            .into(),
        );
        entry.case.contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; future creation + polling included".into(),
        );
        self.batch_contract(drop, batch);
        self
    }

    /// Async operation borrowing fresh input; preparation and input drop are excluded.
    pub fn bench_async_with_input<E, I, O>(
        &mut self,
        name: &str,
        mut executor: impl FnMut() -> E + 'a,
        mut setup: impl FnMut() -> I + 'a,
        mut f: impl for<'i> AsyncFnMut(&'i mut I) -> O + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
        I: 'a,
        O: 'a,
    {
        let mut state = None;
        self.register(
            name,
            crate::suite_measure::input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let executor = state.get_or_insert_with(&mut executor);
                input_batch(
                    timer,
                    n,
                    &mut setup,
                    &mut |input: &mut I| executor.block_on(f(input)),
                    drop,
                )
            }),
        );
        self.entries
            .last_mut()
            .unwrap()
            .case
            .contract
            .insert("async.executor".into(), std::any::type_name::<E>().into());
        self.entries.last_mut().unwrap().case.contract.insert(
            "async.scope".into(),
            "lazy executor construction excluded; future creation + polling included".into(),
        );
        self
    }

    /// Concurrent operations. Spawning/joining threads is excluded; scheduling and start
    /// synchronization are included. Each worker runs n operations; normalization uses n*workers.
    pub fn bench_threads<O>(
        &mut self,
        name: &str,
        workers: usize,
        f: impl Fn() -> O + Sync + 'a,
    ) -> &mut Self {
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            "concurrent batch; spawn/join excluded; output drop included",
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                crate::suite_measure::parallel_batch(
                    timer,
                    workers,
                    || (),
                    |_: &mut (), start| {
                        for _ in 0..n {
                            black_box(f());
                        }
                        start.measure(n)
                    },
                )
            }),
        );
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        self
    }

    /// Construct and destroy inputs on the worker that measures them. Inputs and
    /// outputs need not be Send. Setup is shared, so captured factory state must be Sync.
    pub fn bench_threads_with_local_input<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        workers: usize,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl Fn(&mut I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                let mut remaining = n;
                let mut total = Measured::default();
                while remaining > 0 {
                    let count = remaining.min(64);
                    total = total.add(crate::suite_measure::parallel_local_batch(
                        timer,
                        workers,
                        || {
                            (
                                (0..count).map(|_| setup()).collect::<Vec<_>>(),
                                Vec::with_capacity(count as usize),
                            )
                        },
                        |(inputs, outputs), start| {
                            for input in inputs {
                                match drop {
                                    DropPolicy::InsideTiming => {
                                        black_box(f(black_box(input)));
                                    }
                                    DropPolicy::OutsideTiming => {
                                        outputs.push(black_box(f(black_box(input))))
                                    }
                                }
                            }
                            let elapsed = start.measure(count)?;
                            black_box(&outputs);
                            Ok(elapsed)
                        },
                    )?)?;
                    remaining -= count;
                }
                Ok(total)
            }),
        );
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("threads.setup".into(), "worker".into());
        contract.insert("threads.scope".into(), "sum of synchronized wave intervals; <=64 operations per worker per wave; worker-local setup/drop and spawn/join excluded; values retained until all worker timers stop".into());
        self
    }
    /// Allocation accounting on each worker, with fresh worker-local inputs.
    /// Peak is the maximum across waves of the sum of worker peaks: an upper
    /// bound, not a simultaneous process-wide live-memory measurement.
    pub fn bench_threads_allocated_with_local_input<A, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl Fn(&mut I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + Sync + 'a,
    {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        let worker_samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let captured_workers = worker_samples.clone();
        let case_id = format!("{}/{}", self.name, name);
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                captured.set(None);
                captured_workers.borrow_mut().clear();
                let mut wave_index = 0;
                if workers == 0 || workers > 256 {
                    return Err(error("workers must be 1..256"));
                }
                let mut remaining = n;
                let mut total = Measured::default();
                let mut counts = crate::alloc::ThreadStats::default();
                while remaining > 0 {
                    let count = remaining.min(64);
                    let next_worker = std::sync::atomic::AtomicU32::new(0);
                    let results = std::sync::Mutex::new(Vec::with_capacity(workers));
                    total = total.add(crate::suite_measure::parallel_local_batch(
                        timer,
                        workers,
                        || {
                            (
                                next_worker.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                                (0..count).map(|_| setup()).collect::<Vec<_>>(),
                                Vec::with_capacity(count as usize),
                            )
                        },
                        |(worker, inputs, outputs), start| {
                            let phase = match allocator.begin_thread_phase() {
                                Ok(phase) => phase,
                                Err(error) => {
                                    results.lock().unwrap().push(Err(error));
                                    return Ok(Measured::default());
                                }
                            };
                            for input in inputs {
                                let output = black_box(f(black_box(input)));
                                match drop {
                                    DropPolicy::InsideTiming => std::mem::drop(output),
                                    DropPolicy::OutsideTiming => outputs.push(output),
                                }
                            }
                            let elapsed = start.elapsed_ns()?;
                            let stats = phase.finish();
                            let elapsed = timer.measured(elapsed, count, Some(&stats))?;
                            results.lock().unwrap().push(Ok((*worker, elapsed, stats)));
                            Ok(elapsed)
                        },
                    )?)?;
                    let mut wave = crate::alloc::ThreadStats::default();
                    let mut peak = 0u128;
                    let mut peak_count = 0u128;
                    for result in results.into_inner().unwrap() {
                        let (worker, elapsed, stats) = result?;
                        captured_workers.borrow_mut().push(crate::WorkerAllocation {
                            case: case_id.clone(),
                            variant: "candidate".into(),
                            process: 0,
                            sequence: 0,
                            wave: wave_index,
                            worker,
                            operations: count,
                            wall_ns: elapsed.raw_ns.to_string(),
                            adjusted_wall_ns: Some(elapsed.adjusted_ns.to_string()),
                            metrics: crate::alloc::ThreadStats::METRICS
                                .into_iter()
                                .zip(stats.metric_values())
                                .map(|((id, _, _), value)| (id.into(), value.to_string()))
                                .collect(),
                        });
                        peak = peak
                            .checked_add(stats.peak_above_start_bytes)
                            .ok_or_else(|| error("worker allocation peak overflowed"))?;
                        peak_count = peak_count
                            .checked_add(stats.peak_above_start_count)
                            .ok_or_else(|| error("worker allocation count peak overflowed"))?;
                        wave.append_batch(stats);
                    }
                    wave.peak_above_start_bytes = peak;
                    wave.peak_above_start_count = peak_count;
                    counts.append_batch(wave);
                    remaining -= count;
                    wave_index += 1;
                }
                if counts.overflowed {
                    return Err(error("worker allocation counters overflowed"));
                }
                captured.set(Some(counts));
                Ok(total)
            }),
        );
        self.attach_allocation_sample(sample);
        self.entries.last_mut().unwrap().worker_allocations = Some(worker_samples);
        self.entries
            .last_mut()
            .unwrap()
            .case
            .contract
            .insert("alloc.worker_records".into(), "wave-v1".into());
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("threads.setup".into(), "worker".into());
        contract.insert("threads.scope".into(), "sum of synchronized wave intervals; <=64 operations per worker per wave; worker-local setup/drop and spawn/join excluded; values retained until all worker timers stop".into());
        contract.insert("alloc.scope".into(), "sum of worker operation-loop events; setup, buffers, input drop and spawn/join excluded; output drop follows output.drop; peak is maximum wave sum of worker peaks (upper bound)".into());
        contract.insert(
            "output.drop".into(),
            if matches!(drop, DropPolicy::InsideTiming) {
                "inside"
            } else {
                "outside"
            }
            .into(),
        );
        self
    }

    /// Allocation-counted owned worker inputs. Setup is excluded; destruction
    /// of consumed inputs by the operation is included in timing and accounting.
    pub fn bench_threads_allocated_with_local_owned_input<A, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl Fn(I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + Sync + 'a,
    {
        self.bench_threads_allocated_with_local_input(
            name,
            allocator,
            workers,
            move || Some(setup()),
            move |input| f(input.take().expect("fresh worker input")),
            drop,
        );
        self.owned_input_contract(drop);
        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(),
            "sum of worker operation-loop events; setup, buffers and spawn/join excluded; consumed input drop included; output drop follows output.drop; peak is maximum wave sum of worker peaks (upper bound)".into());
        self
    }

    /// Allocation accounting on each worker, with fresh worker-local inputs.
    /// Peak is the maximum across waves of the sum of worker peaks: an upper
    /// bound, not a simultaneous process-wide live-memory measurement.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_async_threads_allocated_with_input<A, E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        executor: impl Fn() -> E + Sync + 'a,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl for<'i> AsyncFn(&'i mut I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + Sync + 'a,
        E: crate::workloads::Executor + 'a,
    {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        let worker_samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let captured_workers = worker_samples.clone();
        let case_id = format!("{}/{}", self.name, name);
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                captured.set(None);
                captured_workers.borrow_mut().clear();
                let mut wave_index = 0;
                if workers == 0 || workers > 256 {
                    return Err(error("workers must be 1..256"));
                }
                let mut remaining = n;
                let mut total = Measured::default();
                let mut counts = crate::alloc::ThreadStats::default();
                while remaining > 0 {
                    let count = remaining.min(64);
                    let next_worker = std::sync::atomic::AtomicU32::new(0);
                    let results = std::sync::Mutex::new(Vec::with_capacity(workers));
                    total = total.add(crate::suite_measure::parallel_local_batch(
                        timer,
                        workers,
                        || {
                            (
                                next_worker.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                                executor(),
                                (0..count).map(|_| setup()).collect::<Vec<_>>(),
                                Vec::with_capacity(count as usize),
                            )
                        },
                        |(worker, executor, inputs, outputs), start| {
                            let phase = match allocator.begin_thread_phase() {
                                Ok(phase) => phase,
                                Err(error) => {
                                    results.lock().unwrap().push(Err(error));
                                    return Ok(Measured::default());
                                }
                            };
                            for input in inputs {
                                let output = black_box(executor.block_on(f(black_box(input))));
                                match drop {
                                    DropPolicy::InsideTiming => std::mem::drop(output),
                                    DropPolicy::OutsideTiming => outputs.push(output),
                                }
                            }
                            let elapsed = start.elapsed_ns()?;
                            let stats = phase.finish();
                            let elapsed = timer.measured(elapsed, count, Some(&stats))?;
                            results.lock().unwrap().push(Ok((*worker, elapsed, stats)));
                            Ok(elapsed)
                        },
                    )?)?;
                    let mut wave = crate::alloc::ThreadStats::default();
                    let mut peak = 0u128;
                    let mut peak_count = 0u128;
                    for result in results.into_inner().unwrap() {
                        let (worker, elapsed, stats) = result?;
                        captured_workers.borrow_mut().push(crate::WorkerAllocation {
                            case: case_id.clone(),
                            variant: "candidate".into(),
                            process: 0,
                            sequence: 0,
                            wave: wave_index,
                            worker,
                            operations: count,
                            wall_ns: elapsed.raw_ns.to_string(),
                            adjusted_wall_ns: Some(elapsed.adjusted_ns.to_string()),
                            metrics: crate::alloc::ThreadStats::METRICS
                                .into_iter()
                                .zip(stats.metric_values())
                                .map(|((id, _, _), value)| (id.into(), value.to_string()))
                                .collect(),
                        });
                        peak = peak
                            .checked_add(stats.peak_above_start_bytes)
                            .ok_or_else(|| error("worker allocation peak overflowed"))?;
                        peak_count = peak_count
                            .checked_add(stats.peak_above_start_count)
                            .ok_or_else(|| error("worker allocation count peak overflowed"))?;
                        wave.append_batch(stats);
                    }
                    wave.peak_above_start_bytes = peak;
                    wave.peak_above_start_count = peak_count;
                    counts.append_batch(wave);
                    remaining -= count;
                    wave_index += 1;
                }
                if counts.overflowed {
                    return Err(error("worker allocation counters overflowed"));
                }
                captured.set(Some(counts));
                Ok(total)
            }),
        );
        self.attach_allocation_sample(sample);
        self.entries.last_mut().unwrap().worker_allocations = Some(worker_samples);
        self.entries
            .last_mut()
            .unwrap()
            .case
            .contract
            .insert("alloc.worker_records".into(), "wave-v1".into());
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("async.executor".into(), std::any::type_name::<E>().into());
        contract.insert("async.scope".into(), "one executor per worker per wave; construction/drop excluded; sequential future creation/polling per worker included; workers concurrent".into());
        contract.insert("threads.setup".into(), "worker".into());
        contract.insert("threads.scope".into(), "sum of synchronized wave intervals; <=64 operations per worker per wave; worker-local setup/drop and spawn/join excluded; values retained until all worker timers stop".into());
        contract.insert("alloc.scope".into(), "sum of worker operation-loop events; executor construction/drop, setup, buffers, input drop and spawn/join excluded; output drop follows output.drop; peak is maximum wave sum of worker peaks (upper bound)".into());
        contract.insert(
            "output.drop".into(),
            if matches!(drop, DropPolicy::InsideTiming) {
                "inside"
            } else {
                "outside"
            }
            .into(),
        );
        self
    }

    /// Allocation-counted owned worker inputs. Setup is excluded; destruction
    /// of consumed inputs by the operation is included in timing and accounting.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_async_threads_allocated_with_owned_input<A, E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        executor: impl Fn() -> E + Sync + 'a,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl AsyncFn(I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + Sync + 'a,
        E: crate::workloads::Executor + 'a,
    {
        self.bench_async_threads_allocated_with_input(
            name,
            allocator,
            workers,
            executor,
            move || Some(setup()),
            async move |input| f(input.take().expect("fresh worker input")).await,
            drop,
        );
        self.owned_input_contract(drop);
        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(),
            "sum of worker operation-loop events; executor construction/drop, setup, buffers and spawn/join excluded; consumed input drop included; output drop follows output.drop; peak is maximum wave sum of worker peaks (upper bound)".into());
        self
    }

    /// Allocation accounting on each worker, with fresh coordinator-prepared inputs.
    /// Peak is the maximum across waves of the sum of worker peaks: an upper
    /// bound, not a simultaneous process-wide live-memory measurement.
    pub fn bench_threads_allocated_with_input<A, I: Send + 'a, O: Send + 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        mut setup: impl FnMut() -> I + 'a,
        f: impl Fn(&mut I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + Sync + 'a,
    {
        let sample = std::rc::Rc::new(std::cell::Cell::new(None));
        let captured = sample.clone();
        let worker_samples = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let captured_workers = worker_samples.clone();
        let case_id = format!("{}/{}", self.name, name);
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                captured.set(None);
                captured_workers.borrow_mut().clear();
                let mut wave_index = 0;
                if workers == 0 || workers > 256 {
                    return Err(error("workers must be 1..256"));
                }
                let mut remaining = n;
                let mut total = Measured::default();
                let mut counts = crate::alloc::ThreadStats::default();
                while remaining > 0 {
                    let count = remaining.min(64);
                    let next_worker = std::sync::atomic::AtomicU32::new(0);
                    let results = std::sync::Mutex::new(Vec::with_capacity(workers));
                    total = total.add(crate::suite_measure::parallel_batch(
                        timer,
                        workers,
                        || {
                            (
                                next_worker.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                                (0..count).map(|_| setup()).collect::<Vec<_>>(),
                                Vec::with_capacity(count as usize),
                            )
                        },
                        |(worker, inputs, outputs), start| {
                            let phase = match allocator.begin_thread_phase() {
                                Ok(phase) => phase,
                                Err(error) => {
                                    results.lock().unwrap().push(Err(error));
                                    return Ok(Measured::default());
                                }
                            };
                            for input in inputs {
                                let output = black_box(f(black_box(input)));
                                match drop {
                                    DropPolicy::InsideTiming => std::mem::drop(output),
                                    DropPolicy::OutsideTiming => outputs.push(output),
                                }
                            }
                            let elapsed = start.elapsed_ns()?;
                            let stats = phase.finish();
                            let elapsed = timer.measured(elapsed, count, Some(&stats))?;
                            results.lock().unwrap().push(Ok((*worker, elapsed, stats)));
                            Ok(elapsed)
                        },
                    )?)?;
                    let mut wave = crate::alloc::ThreadStats::default();
                    let mut peak = 0u128;
                    let mut peak_count = 0u128;
                    for result in results.into_inner().unwrap() {
                        let (worker, elapsed, stats) = result?;
                        captured_workers.borrow_mut().push(crate::WorkerAllocation {
                            case: case_id.clone(),
                            variant: "candidate".into(),
                            process: 0,
                            sequence: 0,
                            wave: wave_index,
                            worker,
                            operations: count,
                            wall_ns: elapsed.raw_ns.to_string(),
                            adjusted_wall_ns: Some(elapsed.adjusted_ns.to_string()),
                            metrics: crate::alloc::ThreadStats::METRICS
                                .into_iter()
                                .zip(stats.metric_values())
                                .map(|((id, _, _), value)| (id.into(), value.to_string()))
                                .collect(),
                        });
                        peak = peak
                            .checked_add(stats.peak_above_start_bytes)
                            .ok_or_else(|| error("worker allocation peak overflowed"))?;
                        peak_count = peak_count
                            .checked_add(stats.peak_above_start_count)
                            .ok_or_else(|| error("worker allocation count peak overflowed"))?;
                        wave.append_batch(stats);
                    }
                    wave.peak_above_start_bytes = peak;
                    wave.peak_above_start_count = peak_count;
                    counts.append_batch(wave);
                    remaining -= count;
                    wave_index += 1;
                }
                if counts.overflowed {
                    return Err(error("worker allocation counters overflowed"));
                }
                captured.set(Some(counts));
                Ok(total)
            }),
        );
        self.attach_allocation_sample(sample);
        self.entries.last_mut().unwrap().worker_allocations = Some(worker_samples);
        self.entries
            .last_mut()
            .unwrap()
            .case
            .contract
            .insert("alloc.worker_records".into(), "wave-v1".into());
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("threads.setup".into(), "coordinator".into());
        contract.insert("threads.scope".into(), "sum of synchronized wave intervals; <=64 operations per worker per wave; coordinator setup/drop and spawn/join excluded; values retained until all worker timers stop".into());
        contract.insert("alloc.scope".into(), "sum of worker operation-loop events; setup, buffers, input drop and spawn/join excluded; output drop follows output.drop; peak is maximum wave sum of worker peaks (upper bound)".into());
        contract.insert(
            "output.drop".into(),
            if matches!(drop, DropPolicy::InsideTiming) {
                "inside"
            } else {
                "outside"
            }
            .into(),
        );
        self
    }

    /// Allocation-counted owned coordinator-prepared inputs. Setup is excluded; destruction
    /// of consumed inputs by the operation is included in timing and accounting.
    pub fn bench_threads_allocated_with_owned_input<A, I: Send + 'a, O: Send + 'a>(
        &mut self,
        name: &str,
        allocator: &'a crate::alloc::TrackingAllocator<A>,
        workers: usize,
        mut setup: impl FnMut() -> I + 'a,
        f: impl Fn(I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        A: std::alloc::GlobalAlloc + Sync + 'a,
    {
        self.bench_threads_allocated_with_input(
            name,
            allocator,
            workers,
            move || Some(setup()),
            move |input| f(input.take().expect("fresh worker input")),
            drop,
        );
        self.owned_input_contract(drop);
        self.entries.last_mut().unwrap().case.contract.insert("alloc.scope".into(),
            "sum of worker operation-loop events; setup, buffers and spawn/join excluded; consumed input drop included; output drop follows output.drop; peak is maximum wave sum of worker peaks (upper bound)".into());
        self
    }

    /// Move worker-local inputs into each operation. Consumed input drop is timed.
    pub fn bench_threads_with_local_owned_input<I: 'a, O: 'a>(
        &mut self,
        name: &str,
        workers: usize,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl Fn(I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        self.bench_threads_with_local_input(
            name,
            workers,
            move || Some(setup()),
            move |input| f(input.take().expect("fresh worker input")),
            drop,
        );
        self.entries.last_mut().unwrap().case.contract.insert(
            "lifecycle".into(),
            "worker-local fresh owned input; setup excluded; consumed input drop inside operation"
                .into(),
        );
        self.owned_input_contract(drop);
        self
    }

    /// Each worker constructs its executor and fresh inputs before the common
    /// start. Futures run sequentially on that worker; workers run concurrently.
    /// Executors, inputs, futures and outputs stay on their origin worker.
    pub fn bench_async_threads_with_input<E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        workers: usize,
        executor: impl Fn() -> E + Sync + 'a,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl for<'i> AsyncFn(&'i mut I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
    {
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                let mut remaining = n;
                let mut total = Measured::default();
                while remaining > 0 {
                    let count = remaining.min(64);
                    total = total.add(crate::suite_measure::parallel_local_batch(
                        timer,
                        workers,
                        || {
                            (
                                executor(),
                                (0..count).map(|_| setup()).collect::<Vec<_>>(),
                                Vec::with_capacity(count as usize),
                            )
                        },
                        |(executor, inputs, outputs), start| {
                            for input in inputs {
                                let output = executor.block_on(f(black_box(input)));
                                match drop {
                                    DropPolicy::InsideTiming => {
                                        black_box(output);
                                    }
                                    DropPolicy::OutsideTiming => outputs.push(black_box(output)),
                                }
                            }
                            let elapsed = start.measure(count)?;
                            black_box(&outputs);
                            Ok(elapsed)
                        },
                    )?)?;
                    remaining -= count;
                }
                Ok(total)
            }),
        );
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        let contract = &mut self.entries.last_mut().unwrap().case.contract;
        contract.insert("threads.setup".into(), "worker".into());
        contract.insert("threads.scope".into(), "sum of synchronized wave intervals; <=64 operations per worker per wave; worker-local setup/drop and spawn/join excluded; values retained until all worker timers stop".into());
        contract.insert("async.executor".into(), std::any::type_name::<E>().into());
        contract.insert("async.scope".into(), "one executor per worker per wave; construction/drop excluded; sequential future creation/polling per worker included; workers concurrent".into());
        self
    }
    /// Concurrent async operations consuming worker-local fresh inputs.
    pub fn bench_async_threads_with_owned_input<E, I: 'a, O: 'a>(
        &mut self,
        name: &str,
        workers: usize,
        executor: impl Fn() -> E + Sync + 'a,
        setup: impl Fn() -> I + Sync + 'a,
        f: impl AsyncFn(I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
    {
        self.bench_async_threads_with_input(
            name,
            workers,
            executor,
            move || Some(setup()),
            async move |input| f(input.take().expect("fresh worker input")).await,
            drop,
        );
        self.entries.last_mut().unwrap().case.contract.insert(
            "lifecycle".into(),
            "worker-local fresh owned input; setup excluded; consumed input drop inside operation"
                .into(),
        );
        self.owned_input_contract(drop);
        self
    }
    /// Concurrent async operations without fresh input setup.
    pub fn bench_async_threads<E, O: 'a>(
        &mut self,
        name: &str,
        workers: usize,
        executor: impl Fn() -> E + Sync + 'a,
        f: impl AsyncFn() -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self
    where
        E: crate::workloads::Executor + 'a,
    {
        self.bench_async_threads_with_input(
            name,
            workers,
            executor,
            || (),
            async move |_| f().await,
            drop,
        )
    }

    /// Fresh inputs for concurrent workers, prepared before spawning. At most 64 inputs
    /// per worker per wave; each sample sums wave durations. Input and optional output
    /// destruction happen after every worker completes its wave.
    pub fn bench_threads_with_input<I: Send + 'a, O: Send + 'a>(
        &mut self,
        name: &str,
        workers: usize,
        mut setup: impl FnMut() -> I + 'a,
        f: impl Fn(&mut I) -> O + Sync + 'a,
        drop: DropPolicy,
    ) -> &mut Self {
        let runtime_workers = std::rc::Rc::new(std::cell::Cell::new(workers));
        let captured_count = runtime_workers.clone();
        self.register_fallible(
            name,
            crate::suite_measure::input_lifecycle(drop),
            1_048_576,
            Box::new(move |n, timer| {
                let workers = captured_count.get();
                let mut remaining = n;
                let mut total = Measured::default();
                while remaining > 0 {
                    let count = remaining.min(64);
                    total = total.add(crate::suite_measure::parallel_batch(
                        timer,
                        workers,
                        || {
                            (
                                (0..count).map(|_| setup()).collect::<Vec<_>>(),
                                Vec::with_capacity(count as usize),
                            )
                        },
                        |(inputs, outputs), start| {
                            for input in inputs {
                                match drop {
                                    DropPolicy::InsideTiming => {
                                        black_box(f(black_box(input)));
                                    }
                                    DropPolicy::OutsideTiming => {
                                        outputs.push(black_box(f(black_box(input))))
                                    }
                                }
                            }
                            let elapsed = start.measure(count)?;
                            black_box(&outputs);
                            Ok(elapsed)
                        },
                    )?)?;
                    remaining -= count;
                }
                Ok(total)
            }),
        );
        self.thread_contract(workers);
        self.entries.last_mut().unwrap().runtime_workers = Some(runtime_workers);
        self.entries.last_mut().unwrap().case.contract.insert(
            "threads.scope".into(),
            "sum of synchronized wave intervals; <=64 operations per worker per wave; spawn/join and inter-wave setup/drop excluded; total operations across workers".into(),
        );
        self
    }
    /// Override worker counts for registered concurrent cases. Sequential cases keep
    /// their execution model; their closures need not be thread safe.
    /// Zero selects available host parallelism, as in the `threads` attribute.
    pub fn thread_count(&mut self, workers: usize) -> Result<&mut Self> {
        let workers = if workers == 0 {
            crate::threads::available()
        } else {
            workers
        };
        if !(1..=256).contains(&workers) {
            return Err(error("workers must be 1..256"));
        }
        for entry in &mut self.entries {
            if let Some(count) = &entry.runtime_workers {
                count.set(workers);
                entry.operations_per_iteration = workers as u64;
                entry
                    .case
                    .contract
                    .insert("threads".into(), workers.to_string());
            }
        }
        Ok(self)
    }

    fn thread_contract(&mut self, workers: usize) {
        let entry = self.entries.last_mut().unwrap();
        entry.operations_per_iteration = workers as u64;
        entry
            .case
            .contract
            .insert("threads".into(), workers.to_string());
        entry.case.contract.insert("threads.scope".into(), "spawn/join excluded; common start to last worker completion; scheduling included; total operations across workers".into());
        entry.case.metrics[0] = Metric::duration(
            "wall",
            "concurrent batch wall clock; scheduler included",
            "batch_total",
        );
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
        self.entries
            .last_mut()
            .unwrap()
            .case
            .contract
            .insert("async.executor".into(), std::any::type_name::<E>().into());
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
            e.case
                .contract
                .insert(format!("work.counter.{unit}"), count.to_string());
            e.case.contract.insert("work.unit".into(), unit.into());
            e.case
                .contract
                .insert("work.count".into(), count.to_string());
        }
        self
    }
    /// Replace a counter across all registered cases, including dynamic declarations.
    fn override_work_units(&mut self, unit: &str, count: u64) {
        for entry in &mut self.entries {
            entry.case.contract.remove(&format!("work.input.{unit}"));
            entry
                .case
                .contract
                .insert(format!("work.counter.{unit}"), count.to_string());
            if entry
                .case
                .contract
                .get("work.unit")
                .is_some_and(|v| v == unit)
            {
                entry
                    .case
                    .contract
                    .insert("work.count".into(), count.to_string());
            }
        }
    }
    /// Attach actual work totals accumulated during input preparation. Dynamic
    /// totals replace fixed counters of the same unit in throughput reports.
    pub fn input_counters(&mut self, counters: crate::counters::InputCounters) -> &mut Self {
        if let Some(entry) = self.entries.last_mut() {
            for unit in counters.snapshot().expect("fresh input counters").keys() {
                entry
                    .case
                    .contract
                    .insert(format!("work.input.{unit}"), "batch_total".into());
            }
            let shared = counters.clone();
            let mut work = std::mem::replace(&mut entry.work, Box::new(|_, _| unreachable!()));
            entry.work = Box::new(move |n, timer| {
                shared.reset();
                let elapsed = work(n, timer)?;
                shared.snapshot()?;
                Ok(elapsed)
            });
            entry.input_counters = Some(counters);
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
            Box::new(move |n, timer| {
                let mut input = fixture.get().borrow_mut();
                let start = timer.start();
                for _ in 0..n {
                    black_box(f(black_box(&mut *input)));
                }
                start.measure(n)
            }),
        );
        self
    }
    /// Validate registered identities without executing setup or workloads.
    pub fn validate_registration(&self) -> Result<()> {
        let mut ids = std::collections::BTreeSet::new();
        for entry in &self.entries {
            if !ids.insert(&entry.case.id) {
                return Err(error(format!("duplicate benchmark ID: {}", entry.case.id)));
            }
        }
        Ok(())
    }
    pub fn list(&self, filter: &str) -> Vec<&str> {
        self.list_selected(&crate::Selection {
            pattern: filter.into(),
            ..Default::default()
        })
    }
    fn selected_indices(&self, selection: &crate::Selection) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| selection.matches(&entry.case))
            .map(|(i, _)| i)
            .collect();
        // Each key follows actual group nodes, then the leaf. This keeps children
        // together even when their functions live in different source files.
        let source_keys: Vec<_> = if selection.sort == crate::SortOrder::Source {
            let mut first_groups = BTreeMap::new();
            let mut inferred_sources = BTreeMap::new();
            for entry in &self.entries {
                let mut source = entry.source.as_ref();
                let mut path = entry.ordering_group.as_str();
                while path != self.name {
                    source = self.group_sources.get(path).or(source);
                    if let Some(source) = source {
                        inferred_sources
                            .entry(path.to_owned())
                            .and_modify(|previous: &mut &(String, u32, u32)| {
                                if source < *previous {
                                    *previous = source;
                                }
                            })
                            .or_insert(source);
                    }
                    let Some((parent, _)) = path.rsplit_once('/') else {
                        break;
                    };
                    path = parent;
                }
            }
            self.entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let mut key = Vec::new();
                    let mut path = self.name.clone();
                    let relative = entry
                        .ordering_group
                        .strip_prefix(&self.name)
                        .unwrap_or(&entry.ordering_group)
                        .trim_start_matches('/');
                    for segment in relative.split('/').filter(|s| !s.is_empty()) {
                        path.push('/');
                        path.push_str(segment);
                        let source = self
                            .group_sources
                            .get(&path)
                            .or_else(|| inferred_sources.get(&path).copied());
                        let first = *first_groups.entry(path.clone()).or_insert(index);
                        key.push((source.is_none(), source, first));
                    }
                    key.push((entry.source.is_none(), entry.source.as_ref(), index));
                    key
                })
                .collect()
        } else {
            Vec::new()
        };
        indices.sort_by(|&a, &b| {
            if selection.sort == crate::SortOrder::Kind {
                let a = &self.entries[a];
                let b = &self.entries[b];
                return crate::ordering::kind_cmp(
                    &a.ordering_group,
                    a.ordering_family.as_deref(),
                    &a.ordering_name,
                    &b.ordering_group,
                    b.ordering_family.as_deref(),
                    &b.ordering_name,
                );
            }
            if matches!(
                selection.sort,
                crate::SortOrder::Natural | crate::SortOrder::Lexical
            ) {
                let a = &self.entries[a];
                let b = &self.entries[b];
                return crate::ordering::hierarchy_cmp(
                    (
                        &a.ordering_group,
                        a.ordering_family.as_deref(),
                        &a.ordering_name,
                    ),
                    (
                        &b.ordering_group,
                        b.ordering_family.as_deref(),
                        &b.ordering_name,
                    ),
                    selection.sort,
                );
            }
            if selection.sort == crate::SortOrder::Source {
                return source_keys[a].cmp(&source_keys[b]);
            }
            selection
                .sort
                .compare(&self.entries[a].case.id, &self.entries[b].case.id)
        });
        if selection.reverse {
            indices.reverse();
        }
        indices
    }
    pub fn list_selected(&self, selection: &crate::Selection) -> Vec<&str> {
        self.selected_indices(selection)
            .into_iter()
            .map(|i| self.entries[i].case.id.as_str())
            .collect()
    }
    pub fn run(&mut self, filter: &str) -> Result<Run> {
        self.run_selected(&crate::Selection {
            pattern: filter.into(),
            ..Default::default()
        })
    }
    pub fn run_selected(&mut self, selection: &crate::Selection) -> Result<Run> {
        self.run_selected_live(selection, None, false)
    }

    /// Execute one operation per selected case (per worker for concurrent cases).
    /// Registered correctness checks run once instead of measuring the operation again.
    pub fn test_selected(&mut self, selection: &crate::Selection) -> Result<Run> {
        self.run_selected_live(selection, None, true)
    }

    /// Install hooks used only by `profile_selected` or `--profile-time-ms`.
    pub fn profiler(&mut self, profiler: impl crate::profiling::Profiler + 'a) -> &mut Self {
        self.profiler = Some(Box::new(profiler));
        self
    }
    pub fn profile_selected(
        &mut self,
        selection: &crate::Selection,
        duration: Duration,
        output: &std::path::Path,
    ) -> Result<crate::profiling::ProfileReport> {
        self.profile_selected_live(selection, duration, output, None)
    }
    fn profile_selected_live(
        &mut self,
        selection: &crate::Selection,
        duration: Duration,
        output: &std::path::Path,
        mut live: Option<&mut airbug::report::live::Run>,
    ) -> Result<crate::profiling::ProfileReport> {
        use crate::profiling::{self, ProfileCase, ProfileReport};
        selection.validate()?;
        profiling::validate_duration(duration)?;
        self.config.validate()?;
        let indices = self.selected_indices(selection);
        if indices.is_empty() {
            return Err(error("no benchmarks selected for profiling"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for &index in &indices {
            let e = &self.entries[index];
            e.sampling
                .validate(&self.config, e.max_batch.min(self.config.max_iterations))?;
            if !ids.insert(&e.case.id) {
                return Err(error("duplicate benchmark ID"));
            }
            if !(1..=256).contains(&e.operations_per_iteration) {
                return Err(error("workers must be 1..256"));
            }
            if e.case
                .contract
                .get("temperature")
                .is_some_and(|t| t.starts_with("cold;"))
            {
                return Err(error("timed profiling cannot repeat a cold case"));
            }
        }
        let timers = self.resolve_timers(&indices)?;
        if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir(output)?;
        if let Some(live) = live.as_deref_mut() {
            for &index in &indices {
                live.add_case(&self.name, &self.entries[index].case.id, false);
            }
            live.save()?;
        }
        let mut profiler = self
            .profiler
            .take()
            .unwrap_or_else(|| Box::new(profiling::ExternalProfiler));
        let result = (|| {
            let mut report = ProfileReport {
                complete: false,
                requested_ns_per_case: duration.as_nanos().to_string(),
                cases: vec![],
            };
            for (position, index) in indices.into_iter().enumerate() {
                let timer = timers[&index];
                let entry = &mut self.entries[index];
                let directory = format!("{position:04}");
                let path = output.join(&directory);
                std::fs::create_dir(&path)?;
                if let Some(live) = live.as_deref_mut() {
                    live.start_case(position)?;
                }
                if self.console_output != ConsoleOutput::Quiet {
                    eprintln!("profile: {} — {} ms", entry.case.id, duration.as_millis());
                }
                let outcome = profiling::attempt(|| {
                    if let Some(check) = &mut entry.verify {
                        check()?;
                    }
                    entry.executed = true;
                    let stats = profiling::session(&mut *profiler, &entry.case.id, &path, || {
                        profiling::repeat(
                            duration,
                            entry.max_batch.min(self.config.max_iterations),
                            entry.sampling.iterations,
                            entry.operations_per_iteration,
                            &mut |n| (entry.work)(n, timer).map(|m| m.raw_ns),
                        )
                    })?;
                    if let Some(check) = &mut entry.verify {
                        check()?;
                    }
                    Ok(stats)
                });
                let failure = outcome.as_ref().err().map(ToString::to_string);
                report.cases.push(ProfileCase {
                    id: entry.case.id.clone(),
                    directory,
                    stats: outcome.ok(),
                    error: failure.clone(),
                });
                std::fs::write(
                    path.join("summary.json"),
                    serde_json::to_vec_pretty(report.cases.last().unwrap())?,
                )?;
                if let Some(live) = live.as_deref_mut() {
                    live.finish_case(
                        position,
                        if failure.is_some() {
                            "failed"
                        } else {
                            "passed"
                        },
                        failure.as_deref().unwrap_or(
                            "Timed profiling completed; no statistical benchmark estimates.",
                        ),
                        None,
                    )?;
                }
                if let Some(failure) = failure {
                    std::fs::write(
                        output.join("profile.json"),
                        serde_json::to_vec_pretty(&report)?,
                    )?;
                    return Err(error(failure));
                }
            }
            report.complete = true;
            std::fs::write(
                output.join("profile.json"),
                serde_json::to_vec_pretty(&report)?,
            )?;
            Ok(report)
        })();
        self.profiler = Some(profiler);
        result
    }

    fn run_selected_live(
        &mut self,
        selection: &crate::Selection,
        mut live: Option<&mut airbug::report::live::Run>,
        test_mode: bool,
    ) -> Result<Run> {
        selection.validate()?;
        self.config.validate()?;
        if let Some(quick) = self.quick {
            quick.validate()?;
        }

        let mut ids = std::collections::BTreeSet::new();
        for e in self.entries.iter().filter(|e| selection.matches(&e.case)) {
            self.validate_case_quick(e, test_mode)?;
            e.sampling
                .validate(&self.config, e.max_batch.min(self.config.max_iterations))?;
            if e.operations_per_iteration == 0 || e.operations_per_iteration > 256 {
                return Err(error("workers must be 1..256"));
            }
            if !ids.insert(&e.case.id) {
                return Err(error("duplicate benchmark ID"));
            }
            for (key, value) in &e.case.contract {
                if let Some(unit) = key.strip_prefix("work.counter.") {
                    if unit.is_empty() || value.parse::<u64>().is_err() {
                        return Err(error(
                            "work units require a nonempty unit and an unsigned count",
                        ));
                    }
                }
            }
            if e.case.contract.contains_key("work.count")
                && (e
                    .case
                    .contract
                    .get("work.count")
                    .and_then(|n| n.parse::<u64>().ok())
                    .is_none()
                    || e.case
                        .contract
                        .get("work.unit")
                        .is_none_or(|s| s.is_empty()))
            {
                return Err(error(
                    "work units require a nonempty unit and an unsigned count",
                ));
            }
        }
        let indices = self.selected_indices(selection);
        let selected: Vec<_> = indices.iter().map(|&i| &self.entries[i]).collect();
        if !test_mode
            && selected.iter().any(|e| {
                e.case
                    .contract
                    .get("temperature")
                    .is_some_and(|v| v.starts_with("cold;"))
            })
            && (selected.len() != 1 || selected[0].verify.is_some())
        {
            return Err(error(
                "cold mode requires exactly one unchecked case; verify correctness in a separate process",
            ));
        }
        let timers = self.resolve_timers(&indices)?;
        let clock_names: std::collections::BTreeSet<_> =
            timers.values().map(|t| t.name()).collect();
        let clock_name = if clock_names.len() == 1 {
            *clock_names.first().unwrap()
        } else {
            "mixed"
        };
        let mut run = Run::new();
        run.provenance
            .insert("timer.clock".into(), clock_name.into());
        if let Some(hz) = timers.values().find_map(|timer| timer.frequency_hz()) {
            run.provenance
                .insert("timer.frequency_hz".into(), hz.to_string());
        }
        // Cold/smoke runs must not acquire extra timer/loop warmup work.
        let calibrate = !test_mode
            && !selected.iter().any(|entry| {
                entry
                    .case
                    .contract
                    .get("temperature")
                    .is_some_and(|value| value.starts_with("cold;"))
            });
        run.provenance.insert(
            "timer.overhead_policy".into(),
            if selected.iter().any(|entry| self.overhead_for(entry)) {
                "subtract requested; see case policy"
            } else {
                "raw; no subtraction"
            }
            .into(),
        );
        if calibrate {
            let mut seen = std::collections::BTreeSet::new();
            for (&index, &timer) in &timers {
                if self.entries[index].caller_timed || !seen.insert(timer.name()) {
                    continue;
                }
                let calibration = timer.cached_calibration()?;
                let prefix = format!("timer.{}.calibration", timer.name());
                run.provenance
                    .insert(format!("{prefix}.trials"), calibration.trials.to_string());
                run.provenance.insert(
                    format!("{prefix}.resolution_trials"),
                    calibration.resolution_trials.to_string(),
                );
                run.provenance.insert(
                    format!("{prefix}.resolution_delay_iterations"),
                    calibration.resolution_delay_iterations.to_string(),
                );
                run.provenance.insert(
                    format!("{prefix}.resolution_status"),
                    if calibration.observed_resolution_ns.is_some() {
                        "observed"
                    } else {
                        "unresolved"
                    }
                    .into(),
                );
                run.provenance.insert(
                    format!("{prefix}.empty_interval_ns"),
                    calibration.empty_interval_ns.to_string(),
                );
                if let Some(resolution) = calibration.observed_resolution_ns {
                    run.provenance.insert(
                        format!("{prefix}.observed_resolution_ns"),
                        resolution.to_string(),
                    );
                }
                run.provenance.insert(
                    format!("{prefix}.loop_batch_ns"),
                    calibration.loop_batch_ns.to_string(),
                );
                run.provenance.insert(
                    format!("{prefix}.loop_iterations"),
                    calibration.loop_iterations.to_string(),
                );
                run.provenance.insert(
                    format!("{prefix}.loop_trials"),
                    calibration.loop_trials.to_string(),
                );
            }
        }
        if test_mode {
            run.notes.push("Test mode: one operation per worker, no calibration or warmup; durations are smoke-test diagnostics, not benchmark estimates.".into());
        }
        run.provenance
            .insert("bench_version".into(), env!("CARGO_PKG_VERSION").into());
        run.provenance
            .insert("samples".into(), self.config.samples.to_string());
        run.provenance.insert(
            "warmup_ns".into(),
            self.config.warmup.as_nanos().to_string(),
        );
        run.notes.push("Raw batches are not independent process replications. Batch averages are not individual-operation latency. CPU frequency and background load are uncontrolled.".into());
        if let Some(live) = live.as_deref_mut() {
            for entry in &selected {
                live.add_case(&self.name, &entry.case.id, false);
            }
            live.save()?;
        }
        for (live_index, index) in indices.into_iter().enumerate() {
            let timer = timers[&index];
            let case_quick = self.quick_for(&self.entries[index]);
            let case_overhead = self.overhead_for(&self.entries[index]);
            let e = &mut self.entries[index];
            if run.cases.iter().any(|c: &Case| c.id == e.case.id) {
                return Err(error("duplicate benchmark ID"));
            }
            e.case.contract.insert(
                "timer.clock".into(),
                if e.caller_timed {
                    "caller"
                } else {
                    timer.name()
                }
                .into(),
            );
            let config = e.sampling.apply(&self.config);
            let cold = e
                .case
                .contract
                .get("temperature")
                .is_some_and(|v| v.starts_with("cold;"));
            if cold && e.executed {
                return Err(error("cold case already executed; launch a fresh worker"));
            }
            e.executed = true;
            let once = cold || test_mode;
            let compensate = case_overhead && !once && !e.caller_timed;
            let timer = if compensate {
                timer.with_correction()?
            } else {
                timer
            };
            if let Some(correction) = timer.correction().filter(|_| compensate) {
                let prefix = format!("timer.{}.correction", timer.name());
                run.provenance.insert(
                    format!("{prefix}.loop_batch_ns"),
                    correction.loop_batch_ns.to_string(),
                );
                run.provenance.insert(
                    format!("{prefix}.loop_iterations"),
                    correction.loop_iterations.to_string(),
                );
                run.provenance.insert(
                    format!("{prefix}.tally_iterations"),
                    correction.tally_iterations.to_string(),
                );
                for (event, cost) in ["alloc", "dealloc", "grow", "shrink"]
                    .into_iter()
                    .zip(correction.tally_batch_ns)
                {
                    run.provenance
                        .insert(format!("{prefix}.{event}_batch_ns"), cost.to_string());
                }
            }
            e.case.metrics.retain(|metric| metric.id != "wall.adjusted");
            if compensate {
                e.case.metrics.push(Metric::duration(
                    "wall.adjusted",
                    "calibrated loop and allocation bookkeeping subtracted; clamped per interval",
                    "batch_total",
                ));
            }
            e.case.contract.insert(
                "timer.overhead_policy".into(),
                if compensate { "subtract-v1" } else { "raw" }.into(),
            );
            let quick = if once { None } else { case_quick };
            e.case.contract.retain(|key, _| !key.starts_with("quick."));
            if quick.is_some()
                && (e.sampling.iterations.is_some()
                    || e.sampling
                        .mode
                        .is_some_and(|m| m != crate::SamplingMode::Flat))
            {
                return Err(error(
                    "adaptive quick mode cannot combine with fixed iterations or linear/auto sampling",
                ));
            }
            e.case.contract.insert(
                "execution.mode".into(),
                if test_mode { "test_once" } else { "benchmark" }.into(),
            );
            let samples = if once { 1 } else { config.samples };
            e.case
                .contract
                .insert("samples".into(), samples.to_string());
            e.case.contract.insert(
                "warmup_ns".into(),
                (if once { 0 } else { config.warmup.as_nanos() }).to_string(),
            );
            e.case.contract.insert(
                "sample_target_ns".into(),
                (if once {
                    0
                } else {
                    config.sample_time.as_nanos()
                })
                .to_string(),
            );
            if self.console_output != ConsoleOutput::Quiet {
                eprintln!("bench: {} — validating and calibrating", e.case.id);
            }
            if let Some(live) = live.as_deref_mut() {
                live.start_case(live_index)?;
            }
            if !test_mode {
                if let Some(check) = &mut e.verify {
                    check().map_err(|err| error(format!("{} pre-validation: {err}", e.case.id)))?;
                }
            }
            let cap = e.max_batch.min(config.max_iterations);
            let mut n = if once {
                1
            } else {
                e.sampling.iterations.unwrap_or(1)
            };
            let mut budget = TimeBudget::new(&e.sampling);
            let mut calibrated_ns = 0;
            if !once && quick.is_none() && e.sampling.iterations.is_none() {
                loop {
                    if budget.expired() {
                        return Err(error(format!(
                            "{}: max_time exhausted during calibration",
                            e.case.id
                        )));
                    }
                    let start = Instant::now();
                    calibrated_ns = (e.work)(n, timer)?.raw_ns;
                    budget.add(calibrated_ns, start.elapsed());
                    if calibrated_ns >= config.sample_time.as_nanos() || n >= cap {
                        break;
                    }
                    n = (n * 2).min(cap);
                }
            }
            let start = Instant::now();
            while !once && quick.is_none() && start.elapsed() < config.warmup && !budget.expired() {
                let call_start = Instant::now();
                let elapsed = (e.work)(n, timer)?.raw_ns;
                budget.add(elapsed, call_start.elapsed());
            }
            let schedule = Schedule::new(&e.sampling, &config, n, calibrated_ns, cap);
            e.sampling.record(&mut e.case.contract);
            e.case.contract.insert(
                "sampling.mode".into(),
                if once { "once" } else { schedule.mode.as_str() }.into(),
            );
            if let Some(q) = quick {
                e.case
                    .contract
                    .insert("sampling.mode".into(), "quick_adaptive".into());
                e.case.contract.insert("warmup_ns".into(), "0".into());
                e.case.contract.insert(
                    "quick.min_time_ns".into(),
                    q.min_time.as_nanos().to_string(),
                );
                e.case.contract.insert(
                    "quick.max_time_ns".into(),
                    q.max_time.as_nanos().to_string(),
                );
                e.case.contract.insert(
                    "quick.relative_deviation".into(),
                    q.relative_deviation.to_string(),
                );
            }
            if self.console_output != ConsoleOutput::Quiet {
                eprintln!("bench: {} — measuring {} samples", e.case.id, samples);
            }
            run.cases.push(e.case.clone());
            let mut under_target = false;
            let mut collected = 0u32;
            let mut operation_range = (u64::MAX, 0u64);
            let quick_start = Instant::now();
            let mut previous_quick = None;
            let mut quick_n = 1;
            while if once {
                collected == 0
            } else if quick.is_some() {
                collected == 0 || !budget.expired()
            } else {
                budget.needs_samples(collected, samples)
            } {
                if collected == 100_000 && quick.is_some() {
                    run.cases
                        .last_mut()
                        .unwrap()
                        .contract
                        .insert("quick.stop".into(), "sample_limit".into());
                    break;
                }
                if collected == 100_000 {
                    return Err(error(format!(
                        "{}: min_time not reached within 100000 samples",
                        e.case.id
                    )));
                }
                let n = if once {
                    1
                } else if quick.is_some() {
                    quick_n
                } else {
                    schedule.operations(collected)
                };
                let call_start = Instant::now();
                let measured = if let (true, Some(check)) = (test_mode, e.verify.as_mut()) {
                    if let Some(counters) = &e.input_counters {
                        counters.reset();
                    }
                    let start = timer.start();
                    check()?;
                    start.measure(1)?
                } else {
                    (e.work)(n, timer)?
                };
                let elapsed = measured.raw_ns;
                budget.add(elapsed, call_start.elapsed());
                under_target |= elapsed < config.sample_time.as_nanos() / 2;
                let operations = n * e.operations_per_iteration;
                operation_range.0 = operation_range.0.min(operations);
                operation_range.1 = operation_range.1.max(operations);
                run.observations.push(Observation {
                    work_totals: e
                        .input_counters
                        .as_ref()
                        .map(|c| -> Result<_> {
                            let mut totals = c.snapshot()?;
                            totals.retain(|unit, _| {
                                e.case.contract.contains_key(&format!("work.input.{unit}"))
                            });
                            Ok(totals)
                        })
                        .transpose()?
                        .unwrap_or_default(),
                    case: e.case.id.clone(),
                    metric: "wall".into(),
                    variant: "candidate".into(),
                    process: 0,
                    pair: None,
                    sequence: collected as u64,
                    value: Some(elapsed.to_string()),
                    operations,
                    availability: Availability::Available,
                });
                if compensate {
                    let mut adjusted = run.observations.last().unwrap().clone();
                    adjusted.metric = "wall.adjusted".into();
                    adjusted.value = Some(measured.adjusted_ns.to_string());
                    run.observations.push(adjusted);
                }
                if let Some((metric, sample)) = &e.measurement_sample {
                    run.observations.push(Observation {
                        work_totals: Default::default(),
                        case: e.case.id.clone(),
                        metric: metric.clone(),
                        variant: "candidate".into(),
                        process: 0,
                        pair: None,
                        sequence: collected as u64,
                        value: Some(
                            sample
                                .get()
                                .ok_or_else(|| error("custom measurement sample missing"))?
                                .to_string(),
                        ),
                        operations,
                        availability: Availability::Available,
                    });
                }
                if let Some(sample) = &e.allocation_sample {
                    let counts = sample
                        .get()
                        .ok_or_else(|| error("allocation sample missing"))?;
                    for ((id, _, _), value) in crate::alloc::ThreadStats::METRICS
                        .into_iter()
                        .zip(counts.metric_values())
                    {
                        run.observations.push(Observation {
                            work_totals: Default::default(),
                            case: e.case.id.clone(),
                            metric: id.into(),
                            variant: "candidate".into(),
                            process: 0,
                            pair: None,
                            sequence: collected as u64,
                            value: Some(value.to_string()),
                            operations,
                            availability: Availability::Available,
                        });
                    }
                }
                if let Some(workers) = &e.worker_allocations {
                    for mut worker in workers.borrow_mut().drain(..) {
                        worker.sequence = collected as u64;
                        if !compensate {
                            worker.adjusted_wall_ns = None;
                        }
                        run.worker_allocations.push(worker);
                    }
                }
                collected += 1;
                if let Some(q) = quick {
                    if budget.expired() {
                        run.cases
                            .last_mut()
                            .unwrap()
                            .contract
                            .insert("quick.stop".into(), "time_budget".into());
                        break;
                    }
                    if let Some(reason) =
                        q.stop(previous_quick, (n, elapsed), quick_start.elapsed())
                    {
                        if reason != "stable" || budget.minimum_met() {
                            run.cases
                                .last_mut()
                                .unwrap()
                                .contract
                                .insert("quick.stop".into(), reason.into());
                            break;
                        }
                    }
                    previous_quick = Some((n, elapsed));
                    quick_n = n.saturating_mul(2).min(cap);
                }
            }
            if quick.is_some() {
                let contract = &mut run.cases.last_mut().unwrap().contract;
                contract.insert("samples".into(), collected.to_string());
                contract
                    .entry("quick.stop".into())
                    .or_insert_with(|| "time_budget".into());
            }
            if collected == 0 {
                return Err(error(format!(
                    "{}: max_time exhausted before collecting a sample",
                    e.case.id
                )));
            }
            if !once && budget.expired() {
                run.notes.push(format!("{}: max_time reached after {collected} samples; requested {samples}. Limits are checked between workload calls.", e.case.id));
            }
            if quick.is_none() && collected > samples {
                run.notes.push(format!("{}: collected {collected} samples to meet min_time (requested {samples}); linear extensions repeat the last planned size.", e.case.id));
            }
            if !test_mode {
                if let Some(check) = &mut e.verify {
                    check()
                        .map_err(|err| error(format!("{} post-validation: {err}", e.case.id)))?;
                }
            }
            if under_target
                && !once
                && quick.is_none()
                && schedule.mode == crate::SamplingMode::Flat
            {
                run.notes.push(format!("{}: some samples below half the requested duration; iteration cap or workload drift may limit precision",e.case.id));
            }
            let operations = format!("{}..{}", operation_range.0, operation_range.1);
            if let Some(live) = live.as_deref_mut() {
                let mut batch_averages: Vec<f64> = run
                    .observations
                    .iter()
                    .filter(|o| o.case == e.case.id && o.metric == "wall")
                    .filter_map(|o| {
                        Some(o.value.as_ref()?.parse::<f64>().ok()? / o.operations as f64)
                    })
                    .collect();
                batch_averages.sort_by(f64::total_cmp);
                let median = if batch_averages.is_empty() {
                    None
                } else {
                    let mid = batch_averages.len() / 2;
                    Some(if batch_averages.len().is_multiple_of(2) {
                        (batch_averages[mid - 1] + batch_averages[mid]) / 2.0
                    } else {
                        batch_averages[mid]
                    })
                };
                live.finish_case(live_index, "passed", &format!("{collected} samples; {operations} operations per batch. Median of batch averages, not individual-operation latency."), median)?;
            }
        }
        run.status = Status::Complete;
        crate::presentation::save_scales(&mut run, &self.summary_scales(selection)?)?;
        let families = self
            .entries
            .iter()
            .filter_map(|e| {
                e.summary_family
                    .as_ref()
                    .map(|f| (e.case.id.clone(), f.clone()))
            })
            .collect();
        crate::presentation::save_families(&mut run, &families)?;
        run.validate()?;
        Ok(run)
    }
    /// Minimal executable harness: --list, --filter, --samples, --sample-ms,
    /// --warmup-ms, --json, --output. Errors propagate to the caller's main.
    pub fn main(mut self) -> Result<()> {
        let all_args: Vec<_> = std::env::args().skip(1).collect();
        let environment_args =
            crate::sampling::environment_args(&all_args, |name| match std::env::var(name) {
                Ok(value) => Ok(Some(value)),
                Err(std::env::VarError::NotPresent) => Ok(None),
                Err(error_value) => Err(error(format!("{name}: {error_value}"))),
            })?;
        let effective_args: Vec<_> = environment_args
            .into_iter()
            .chain(all_args.iter().cloned())
            .collect();
        if let Some(requested) = crate::threads::from_args(&effective_args)? {
            crate::threads::validate_registered_request(
                &requested,
                self.registration_threads.as_deref(),
            )?;
        }
        let mut args = effective_args.into_iter();
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
            for entry in &mut self.entries {
                entry.sampling.samples = None;
                entry.sampling.warmup = None;
                entry.sampling.sample_time = None;
            }
        }
        let mut selection = crate::Selection::default();
        let mut list = false;
        let mut dry_run = false;
        let mut test_mode = false;
        let mut json = false;
        let mut color = self.console_color;
        let mut color_cli = false;
        let mut format_cli = false;
        macro_rules! println {
            ($($arg:tt)*) => {{
                use std::io::IsTerminal;
                let text = format!($($arg)*);
                let enabled = !json && !list && !dry_run && color.enabled(
                    std::io::stdout().is_terminal(), std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()));
                ::std::println!("{}", crate::console::render(&text, enabled));
            }};
        }

        let mut no_plots = !self.plots;
        let mut plots_cli = false;
        let mut console_cli = false;
        let mut output = None;
        let mut summary_parameter = None;
        let mut summary_estimator = None;
        let mut summary_scale = None;
        let mut filter_set = false;
        let mut regex_filter = false;
        let mut bootstrap: Option<crate::bootstrap::Config> = None;
        let mut bootstrap_cli = crate::bootstrap::Options::default();
        let mut bootstrap_distributions = false;
        let mut bytes_format = None;
        let mut comparison_config = self.comparison_config;
        let mut significance_set = false;
        let mut noise_set = false;
        let mut hypothesis_resamples_set = false;
        let mut hypothesis_seed_set = false;
        let mut history_enabled = std::env::var_os("AIRBUG_BENCH_HISTORY").is_none_or(|v| v != "0");
        let mut history_root =
            std::env::var_os("AIRBUG_BENCH_HISTORY_DIR").map(std::path::PathBuf::from);
        let mut history_root_set = false;
        let mut discard = false;
        let mut load_baseline = None;
        let mut named_baseline: Option<(String, bool)> = None;
        let mut save_baseline: Option<(String, crate::baseline::SaveMode)> = None;
        let mut baseline_store = std::path::PathBuf::from(".bench");
        let mut baseline_store_set = false;
        let mut profile_time = None;
        let mut timer_kind = None;
        let mut overhead_set = false;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                // Cargo supplies this even for targets with harness = false.
                "--bench" => {}
                "--dry-run" => dry_run = true,
                "--output-format" => {
                    if format_cli {
                        return Err(error("output format specified twice"));
                    }
                    format_cli = true;
                    self.console_format = crate::ConsoleFormat::parse(
                        &args
                            .next()
                            .ok_or_else(|| error("--output-format requires table or tree"))?,
                    )?;
                }
                "--color" => {
                    if color_cli {
                        return Err(error("color specified twice"));
                    }
                    color_cli = true;
                    color = crate::ConsoleColor::parse(
                        &args
                            .next()
                            .ok_or_else(|| error("--color requires auto, always or never"))?,
                    )?;
                }
                "--quiet" | "-q" | "--verbose" | "-v" => {
                    if console_cli {
                        return Err(error(
                            "console output mode specified twice; choose --quiet or --verbose",
                        ));
                    }
                    console_cli = true;
                    self.console_output = if matches!(arg.as_str(), "--quiet" | "-q") {
                        ConsoleOutput::Quiet
                    } else {
                        ConsoleOutput::Verbose
                    };
                }
                "--no-history" => history_enabled = false,
                "--plots" | "--no-plots" => {
                    if plots_cli {
                        return Err(error("plot mode specified twice"));
                    }
                    plots_cli = true;
                    no_plots = arg == "--no-plots";
                }
                "--discard" => discard = true,
                "--load-baseline" => {
                    if load_baseline.is_some() {
                        return Err(error("loaded baseline specified twice"));
                    }
                    load_baseline = Some(
                        args.next()
                            .filter(|v| !v.starts_with('-'))
                            .ok_or_else(|| error("--load-baseline requires a name"))?,
                    );
                }
                "--save-baseline" | "--replace-baseline" | "--retain-baseline" => {
                    if save_baseline.is_some() {
                        return Err(error("baseline save mode specified twice"));
                    }
                    let name = args
                        .next()
                        .filter(|v| !v.starts_with('-'))
                        .ok_or_else(|| error(format!("{arg} requires a name")))?;
                    let mode = match arg.as_str() {
                        "--replace-baseline" => crate::baseline::SaveMode::Replace,
                        "--retain-baseline" => crate::baseline::SaveMode::Retain,
                        _ => crate::baseline::SaveMode::Create,
                    };
                    save_baseline = Some((name, mode));
                }
                "--baseline" | "--baseline-lenient" => {
                    if named_baseline.is_some() {
                        return Err(error("baseline specified twice"));
                    }
                    let name = args
                        .next()
                        .filter(|v| !v.starts_with('-'))
                        .ok_or_else(|| error(format!("{arg} requires a name")))?;
                    named_baseline = Some((name, arg == "--baseline"));
                }
                "--baseline-store" => {
                    if baseline_store_set {
                        return Err(error("baseline store specified twice"));
                    }
                    baseline_store_set = true;
                    baseline_store = args
                        .next()
                        .filter(|v| !v.starts_with('-'))
                        .ok_or_else(|| error("--baseline-store requires a directory"))?
                        .into();
                }
                "--history-dir" => {
                    if history_root_set {
                        return Err(error("history directory specified twice"));
                    }
                    history_root_set = true;
                    history_root = Some(
                        args.next()
                            .filter(|v| !v.starts_with('-'))
                            .ok_or_else(|| error("--history-dir requires a directory"))?
                            .into(),
                    );
                }
                "--test" => test_mode = true,
                "--overhead" => {
                    if overhead_set {
                        return Err(error("overhead policy specified twice"));
                    }
                    overhead_set = true;
                    self.overhead_cli = true;
                    let policy = args
                        .next()
                        .filter(|v| !v.starts_with('-'))
                        .ok_or_else(|| error("--overhead requires raw or subtract"))?;
                    self.overhead = match policy.as_str() {
                        "raw" => false,
                        "subtract" => true,
                        _ => return Err(error("overhead policy must be raw or subtract")),
                    };
                }
                "--timer" => {
                    if timer_kind.is_some() {
                        return Err(error("timer specified twice"));
                    }
                    timer_kind = Some(crate::timer::TimerKind::parse(
                        &args
                            .next()
                            .filter(|value| !value.starts_with('-'))
                            .ok_or_else(|| error("--timer requires os or cpu"))?,
                    )?);
                }
                "--quick" => {
                    self.quick_cli = true;
                    self.quick.get_or_insert_with(Default::default);
                }
                "--quick-min-ms" | "--quick-max-ms" | "--quick-relative-deviation" => {
                    let value = args
                        .next()
                        .ok_or_else(|| error(format!("{arg} requires a value")))?;
                    self.quick_cli = true;
                    let quick = self.quick.get_or_insert_with(Default::default);
                    match arg.as_str() {
                        "--quick-min-ms" => quick.min_time = Duration::from_millis(value.parse()?),
                        "--quick-max-ms" => quick.max_time = Duration::from_millis(value.parse()?),
                        _ => quick.relative_deviation = value.parse()?,
                    }
                }
                "--profile-time-ms" => {
                    if profile_time.is_some() {
                        return Err(error("profile time specified twice"));
                    }
                    let millis: u64 = args
                        .next()
                        .ok_or_else(|| error("--profile-time-ms requires a duration"))?
                        .parse()?;
                    let duration = Duration::from_millis(millis);
                    crate::profiling::validate_duration(duration)?;
                    profile_time = Some(duration);
                }
                "--profile" => {
                    args.next().ok_or_else(|| error("profile requires value"))?;
                }
                "--exact" => selection.exact = true,
                "--glob" => selection.glob = true,
                "--regex" => regex_filter = true,
                "--threads" => {
                    let counts = crate::threads::parse_list(
                        &args
                            .next()
                            .ok_or_else(|| error("--threads requires a worker list"))?,
                    )?;
                    if self.registration_threads.is_none() {
                        if counts.len() != 1 {
                            return Err(error(
                                "thread lists require main_registered so the matrix is configured before registration",
                            ));
                        }
                        self.thread_count(counts[0])?;
                    }
                }
                "--bytes-count" | "--items-count" | "--chars-count" | "--cycles-count"
                | "--bits-count" => {
                    let unit = arg.trim_start_matches("--").trim_end_matches("-count");
                    let count = args
                        .next()
                        .ok_or_else(|| error(format!("{arg} requires an integer")))?
                        .parse::<u64>()?;
                    self.override_work_units(unit, count);
                }
                "--bytes-format" => {
                    bytes_format = Some(crate::report::BytesFormat::parse(
                        &args
                            .next()
                            .ok_or_else(|| error("--bytes-format requires decimal or binary"))?,
                    )?)
                }
                "--bootstrap-distributions" => {
                    if bootstrap_distributions {
                        return Err(error("--bootstrap-distributions specified twice"));
                    }
                    bootstrap_distributions = true;
                    bootstrap.get_or_insert_default();
                }
                "--resamples" => {
                    bootstrap.get_or_insert_default().resamples = args
                        .next()
                        .ok_or_else(|| error("--resamples requires a count"))?
                        .parse()?;
                    bootstrap_cli.resamples = Some(bootstrap.as_ref().unwrap().resamples);
                }
                "--hypothesis-distribution" => comparison_config.capture_distribution = true,
                "--hypothesis-resamples" => {
                    if hypothesis_resamples_set {
                        return Err(error("hypothesis resamples specified twice"));
                    }
                    hypothesis_resamples_set = true;
                    comparison_config.hypothesis.resamples = args
                        .next()
                        .ok_or_else(|| error("--hypothesis-resamples requires an integer"))?
                        .parse()?;
                }
                "--hypothesis-seed" => {
                    if hypothesis_seed_set {
                        return Err(error("hypothesis seed specified twice"));
                    }
                    hypothesis_seed_set = true;
                    comparison_config.hypothesis.seed = args
                        .next()
                        .ok_or_else(|| error("--hypothesis-seed requires an integer"))?
                        .parse()?;
                }
                "--significance-level" | "--noise-threshold-percent" => {
                    let seen = if arg == "--significance-level" {
                        &mut significance_set
                    } else {
                        &mut noise_set
                    };
                    if *seen {
                        return Err(error(format!("{arg} specified twice")));
                    }
                    *seen = true;
                    let value = args
                        .next()
                        .ok_or_else(|| error(format!("{arg} requires a number")))?
                        .parse()?;
                    if arg == "--significance-level" {
                        comparison_config.significance_level = value;
                    } else {
                        comparison_config.noise_threshold_percent = value;
                    }
                }
                "--confidence-level" => {
                    bootstrap.get_or_insert_default().confidence_level = args
                        .next()
                        .ok_or_else(|| error("--confidence-level requires a fraction"))?
                        .parse()?;
                    bootstrap_cli.confidence_level =
                        Some(bootstrap.as_ref().unwrap().confidence_level);
                }
                "--analysis-seed" => {
                    bootstrap.get_or_insert_default().seed = args
                        .next()
                        .ok_or_else(|| error("--analysis-seed requires an integer"))?
                        .parse()?;
                    bootstrap_cli.seed = Some(bootstrap.as_ref().unwrap().seed);
                }
                "--sort" => {
                    selection.sort = crate::SortOrder::parse(
                        &args
                            .next()
                            .ok_or_else(|| error("--sort requires an order"))?,
                    )?
                }
                "--reverse" => selection.reverse = true,
                "--forward" => selection.reverse = false,
                "--exclude-exact" => selection.exclude_exact.push(
                    args.next()
                        .ok_or_else(|| error("--exclude-exact requires a path"))?,
                ),
                "--exclude-regex" => {
                    selection.skip_regex(
                        &args
                            .next()
                            .ok_or_else(|| error("--exclude-regex requires an expression"))?,
                    )?;
                }
                "--exclude" => selection
                    .exclude
                    .push(args.next().ok_or_else(|| error("exclude requires glob"))?),
                "--tag" => selection
                    .tags
                    .push(args.next().ok_or_else(|| error("tag requires value"))?),
                "--list" => list = true,
                "--include-ignored" => selection.include_ignored = true,
                "--ignored" => selection.only_ignored = true,
                "--json" => json = true,
                "--filter" => {
                    if filter_set {
                        return Err(error("benchmark filter specified twice"));
                    }
                    filter_set = true;
                    selection.pattern = args
                        .next()
                        .ok_or_else(|| error("--filter requires value"))?
                }
                "--summary-parameter" => {
                    if summary_parameter.is_some() {
                        return Err(error("--summary-parameter specified twice"));
                    }
                    summary_parameter = Some(
                        args.next()
                            .filter(|v| !v.is_empty())
                            .ok_or_else(|| error("--summary-parameter requires a name"))?,
                    );
                }
                "--summary-estimator" => {
                    if summary_estimator.is_some() {
                        return Err(error("summary estimator specified twice"));
                    }
                    summary_estimator = Some(match args.next().as_deref() {
                        Some("mean") => crate::summary::Estimator::Mean,
                        Some("process-median") => crate::summary::Estimator::ProcessMedian,
                        _ => {
                            return Err(error(
                                "--summary-estimator requires process-median or mean",
                            ));
                        }
                    });
                }
                "--summary-scale" => {
                    if summary_scale.is_some() {
                        return Err(error("--summary-scale specified twice"));
                    }
                    summary_scale = Some(match args.next().as_deref() {
                        Some("linear") => crate::viz::charts::AxisScale::Linear,
                        Some("logarithmic") => crate::viz::charts::AxisScale::Logarithmic,
                        _ => return Err(error("--summary-scale requires linear or logarithmic")),
                    });
                }
                "--output" => {
                    output = Some(
                        args.next()
                            .ok_or_else(|| error("--output requires directory"))?,
                    )
                }
                "--iterations" => {
                    let value: u64 = args
                        .next()
                        .ok_or_else(|| error("--iterations requires value"))?
                        .parse()?;
                    for entry in &mut self.entries {
                        entry.sampling.iterations = Some(value);
                    }
                }
                "--sampling" => {
                    let mode = crate::SamplingMode::parse(
                        &args
                            .next()
                            .ok_or_else(|| error("--sampling requires value"))?,
                    )?;
                    for entry in &mut self.entries {
                        entry.sampling.mode = Some(mode);
                    }
                }
                "--min-time-ms" | "--max-time-ms" => {
                    let value = Duration::from_millis(
                        args.next()
                            .ok_or_else(|| error("time limit requires milliseconds"))?
                            .parse()?,
                    );
                    for entry in &mut self.entries {
                        if arg == "--min-time-ms" {
                            entry.sampling.min_time = Some(value);
                        } else {
                            entry.sampling.max_time = Some(value);
                        }
                    }
                }
                "--exclude-external-time" | "--include-external-time" => {
                    for entry in &mut self.entries {
                        entry.sampling.exclude_external_time =
                            Some(arg == "--exclude-external-time");
                    }
                }
                "--samples" => {
                    for entry in &mut self.entries {
                        entry.sampling.samples = None;
                    }
                    self.config.samples = args
                        .next()
                        .ok_or_else(|| error("--samples requires value"))?
                        .parse()?
                }
                "--sample-ms" => {
                    for entry in &mut self.entries {
                        entry.sampling.sample_time = None;
                    }
                    self.config.sample_time = Duration::from_millis(
                        args.next()
                            .ok_or_else(|| error("--sample-ms requires value"))?
                            .parse()?,
                    )
                }
                "--warmup-ms" => {
                    for entry in &mut self.entries {
                        entry.sampling.warmup = None;
                    }
                    self.config.warmup = Duration::from_millis(
                        args.next()
                            .ok_or_else(|| error("--warmup-ms requires value"))?
                            .parse()?,
                    )
                }
                "--help" | "-h" => {
                    println!(
                        "Airbug benchmarks

Start here:
  cargo bench                       Measure, compare with previous run, save HTML report
  cargo bench FILTER                Run matching cases
  cargo bench -- --list             List cases without measuring
  cargo bench -- --profile quick    Short exploratory run
  cargo bench -- --discard          Measure without saving results
  cargo bench -- --quiet            Results without progress or detailed tables
  cargo bench -- --verbose          Include environment and case contracts
  cargo bench -- --output-format tree  Hierarchical results and lists
  cargo bench -- --color auto       Color: auto, always or never (auto respects NO_COLOR)

Reports: target/airbug-bench/reports/<run-id>/report.html
Use --output NEW_DIRECTORY to choose a destination.
Sampling environment: AIRBUG_BENCH_SAMPLES, AIRBUG_BENCH_ITERATIONS, AIRBUG_BENCH_WARMUP_MS,
AIRBUG_BENCH_SAMPLE_MS, AIRBUG_BENCH_SAMPLING, AIRBUG_BENCH_MIN_TIME_MS,
AIRBUG_BENCH_MAX_TIME_MS, AIRBUG_BENCH_EXCLUDE_EXTERNAL_TIME.
Also: AIRBUG_BENCH_TIMER, AIRBUG_BENCH_SORT, AIRBUG_BENCH_REVERSE, AIRBUG_BENCH_BYTES_FORMAT.
CLI flags take precedence; --forward overrides reverse sorting from the environment.
Concurrent cases: --threads N[,N...] (repeatable; 0 = available parallelism), AIRBUG_BENCH_THREADS.
Counter overrides: --bytes-count N, --items-count N, --chars-count N, --cycles-count N, --bits-count N.
Environment: AIRBUG_BENCH_BYTES_COUNT, AIRBUG_BENCH_ITEMS_COUNT, AIRBUG_BENCH_CHARS_COUNT, AIRBUG_BENCH_CYCLES_COUNT, AIRBUG_BENCH_BITS_COUNT.
The first run establishes a baseline; insufficient evidence stays inconclusive.

All options:"
                    );
                    println!(
                        "[FILTER] --bench --list --test --discard --load-baseline NAME --save-baseline NAME --replace-baseline NAME --retain-baseline NAME --baseline NAME --baseline-lenient NAME --baseline-store DIRECTORY --history-dir DIRECTORY --no-history --plots|--no-plots --timer os|cpu --overhead raw|subtract --quick [--quick-min-ms N --quick-max-ms N --quick-relative-deviation F] --profile-time-ms N --include-ignored --ignored --profile quick|normal|thorough --filter TEXT [--exact|--glob|--regex] --exclude GLOB --exclude-exact PATH --exclude-regex REGEX --sort registration|lexical|natural|source|kind --reverse --tag TAG --samples N --iterations N --sampling flat|linear|auto --min-time-ms N --max-time-ms N [--exclude-external-time|--include-external-time] --sample-ms N --warmup-ms N --resamples N --bootstrap-distributions --confidence-level FRACTION --significance-level FRACTION --noise-threshold-percent PERCENT --hypothesis-resamples N --hypothesis-seed N --hypothesis-distribution --analysis-seed N --bytes-format decimal|binary --summary-parameter NAME --summary-estimator process-median|mean --summary-scale linear|logarithmic --json --output NEW_DIRECTORY"
                    );
                    return Ok(());
                }
                _ if !arg.starts_with('-') => {
                    if filter_set {
                        return Err(error("benchmark filter specified twice"));
                    }
                    filter_set = true;
                    selection.pattern = arg;
                }
                _ => return Err(error(format!("unknown argument {arg}"))),
            }
        }
        if no_plots
            && (summary_parameter.is_some()
                || summary_scale.is_some()
                || summary_estimator.is_some())
        {
            return Err(error("--no-plots conflicts with summary chart options"));
        }
        if summary_estimator.is_some() && summary_parameter.is_none() {
            return Err(error("--summary-estimator requires --summary-parameter"));
        }
        if (summary_parameter.is_some() || summary_scale.is_some())
            && (output.is_none() || profile_time.is_some() || test_mode)
        {
            return Err(error(
                "summary charts require --output and a measurement or loaded baseline",
            ));
        }
        if summary_scale.is_some() {
            bootstrap.get_or_insert_default();
        }
        if bootstrap_distributions
            && (test_mode || profile_time.is_some() || (!json && output.is_none()))
        {
            return Err(error(
                "--bootstrap-distributions requires --json or --output and a measurement or loaded baseline",
            ));
        }
        comparison_config.validate()?;
        self.timer_cli = timer_kind;
        if let Some(config) = &bootstrap {
            config.validate()?;
        }
        if regex_filter {
            let pattern = selection.pattern.clone();
            selection = selection.with_regex(&pattern)?;
        }
        selection.validate()?;
        if self
            .selected_indices(&selection)
            .iter()
            .any(|&i| !self.entries[i].bootstrap.is_empty())
        {
            bootstrap.get_or_insert_default();
        }
        let bootstrap_overrides = if let Some(config) = &bootstrap {
            self.bootstrap_settings(&selection, config, bootstrap_cli)?
        } else {
            BTreeMap::new()
        };
        let mut comparison_overrides = std::collections::BTreeMap::new();
        for index in self.selected_indices(&selection) {
            let entry = &self.entries[index];
            let mut options = crate::history::ComparisonOptions {
                noise_threshold_percent: noise_set
                    .then_some(comparison_config.noise_threshold_percent),
                significance_level: significance_set
                    .then_some(comparison_config.significance_level),
                resamples: hypothesis_resamples_set
                    .then_some(comparison_config.hypothesis.resamples),
                seed: hypothesis_seed_set.then_some(comparison_config.hypothesis.seed),
            };
            options.inherit(entry.comparison);
            options.resolve(comparison_config)?;
            comparison_overrides.insert(entry.case.id.clone(), options);
        }

        for index in self.selected_indices(&selection) {
            let entry = &self.entries[index];
            self.validate_case_quick(entry, test_mode)?;
            if profile_time.is_some() && self.quick_for(entry).is_some() {
                return Err(error(
                    "adaptive quick mode cannot combine with timed profiling",
                ));
            }
        }
        if profile_time.is_some()
            && self
                .selected_indices(&selection)
                .iter()
                .any(|&i| self.overhead_for(&self.entries[i]))
        {
            return Err(error(
                "overhead compensation cannot combine with timed profiling",
            ));
        }
        if profile_time.is_some() && (test_mode || bootstrap.is_some() || bytes_format.is_some()) {
            return Err(error(
                "timed profiling cannot combine with test mode or statistical/throughput report options",
            ));
        }
        if comparison_config.capture_distribution
            && (test_mode
                || profile_time.is_some()
                || (named_baseline.is_none()
                    && (!history_enabled || load_baseline.is_some() || discard)))
        {
            return Err(error(
                "--hypothesis-distribution requires a benchmark comparison with history or a named baseline",
            ));
        }
        if load_baseline.is_some()
            && (test_mode || profile_time.is_some() || save_baseline.is_some())
        {
            return Err(error(
                "--load-baseline cannot combine with test, profiling or baseline saving",
            ));
        }
        if save_baseline.is_some() && (discard || test_mode || profile_time.is_some()) {
            return Err(error(
                "saving a baseline requires a non-discard benchmark measurement",
            ));
        }
        if discard && (output.is_some() || profile_time.is_some() || named_baseline.is_some()) {
            return Err(error(
                "--discard cannot combine with --output, timed profiling or named baseline comparison",
            ));
        }
        if named_baseline.is_some() && (test_mode || profile_time.is_some()) {
            return Err(error(
                "named baseline comparison requires a benchmark measurement",
            ));
        }
        if dry_run {
            let clocks: std::collections::BTreeSet<_> = self
                .selected_indices(&selection)
                .into_iter()
                .map(|i| self.timer_name(&self.entries[i]))
                .collect();
            let preview_timer = if clocks.len() == 1 {
                *clocks.first().unwrap()
            } else {
                "mixed"
            };
            let policies: std::collections::BTreeSet<_> = self
                .selected_indices(&selection)
                .into_iter()
                .map(|i| self.overhead_for(&self.entries[i]))
                .collect();
            let preview_overhead = if policies.len() > 1 {
                "mixed"
            } else if policies.contains(&true) {
                "subtract"
            } else {
                "raw"
            };
            let preview_quick = if test_mode { None } else { self.quick };
            let has_quick = !test_mode
                && self
                    .selected_indices(&selection)
                    .into_iter()
                    .any(|i| self.quick_for(&self.entries[i]).is_some());
            self.validate_registration()?;
            self.config.validate()?;
            let cases: Vec<_> = self
                .selected_indices(&selection)
                .into_iter()
                .map(|i| {
                    let e = &self.entries[i];
                    let config = e.sampling.apply(&self.config);
                    e.sampling
                        .validate(&self.config, e.max_batch.min(config.max_iterations))?;
                    let mut case = e.case.clone();
                    let compensate = self.overhead_for(e) && !test_mode && !e.caller_timed && !case.contract.get("temperature").is_some_and(|v| v.starts_with("cold;"));
                    case.metrics.retain(|metric| metric.id != "wall.adjusted");
                    if compensate {
                        case.metrics.push(Metric::duration("wall.adjusted", "calibrated loop and allocation bookkeeping subtracted; clamped per interval", "batch_total"));
                    }
                    case.contract.insert("timer.overhead_policy".into(), if compensate { "subtract-v1" } else { "raw" }.into());
                    case.contract.insert(
                        "timer.clock".into(),
                        if e.caller_timed {
                            "caller"
                        } else {
                            self.timer_name(e)
                        }
                        .into(),
                    );
                    case.contract
                        .retain(|key, _| !key.starts_with("quick.") && key != "sampling.mode");
                    e.sampling.record(&mut case.contract);
                    case.contract
                        .insert("samples".into(), config.samples.to_string());
                    case.contract
                        .insert("warmup_ns".into(), config.warmup.as_nanos().to_string());
                    case.contract.insert(
                        "sample_target_ns".into(),
                        config.sample_time.as_nanos().to_string(),
                    );
                    if let Some(q) = self.quick_for(e).filter(|_| !test_mode) {
                        case.contract.insert(
                            "quick.min_time_ns".into(),
                            q.min_time.as_nanos().to_string(),
                        );
                        case.contract.insert(
                            "quick.max_time_ns".into(),
                            q.max_time.as_nanos().to_string(),
                        );
                        case.contract.insert(
                            "quick.relative_deviation".into(),
                            q.relative_deviation.to_string(),
                        );
                        case.contract
                            .insert("sampling.mode".into(), "quick_adaptive".into());
                        case.contract.insert("warmup_ns".into(), "0".into());
                        case.contract.remove("samples");
                        case.contract.remove("sample_target_ns");
                    }
                    Ok(case)
                })
                .collect::<Result<Vec<_>>>()?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"cases":cases,"timer":preview_timer,"overhead":preview_overhead,"profile_time_ms":profile_time.map(|d| d.as_millis()),"adaptive_quick":preview_quick.map(|q| serde_json::json!({"min_time_ms":q.min_time.as_millis(),"max_time_ms":q.max_time.as_millis(),"relative_deviation":q.relative_deviation})),"samples":(!has_quick).then_some(self.config.samples),"warmup_ms":if has_quick {0} else {self.config.warmup.as_millis()},"target_sample_ms":(!has_quick).then_some(self.config.sample_time.as_millis()),"target_measured_ms_per_case":(!has_quick).then_some(self.config.samples as u128*self.config.sample_time.as_millis()),"note":"No work executed. Calibration, setup and validation add wall time; target duration is not a precision guarantee."})
                )?
            );
            return Ok(());
        }
        if list {
            let names = self.list_selected(&selection);
            if self.console_format == crate::ConsoleFormat::Tree && !json {
                println!("{}", crate::console::tree_names(names));
            } else {
                for id in names {
                    println!("{id}");
                }
            }
            return Ok(());
        }
        if cfg!(debug_assertions) && !test_mode && load_baseline.is_none() {
            return Err(error(
                "benchmarks require an optimized build; use cargo run --release or cargo bench",
            ));
        }
        if let Some((name, mode)) = &save_baseline {
            crate::baseline::Store::new(&baseline_store).prepare_case_save_for(
                name,
                *mode,
                &self.list_selected(&selection),
            )?;
        }
        let baseline_snapshot = named_baseline
            .as_ref()
            .map(|(name, strict)| {
                let ids = self.list_selected(&selection);
                crate::baseline::Store::new(&baseline_store)
                    .prepare_case_comparison(name, &ids, *strict)
            })
            .transpose()?
            .unwrap_or_default();
        let loaded_runs = load_baseline
            .as_ref()
            .map(|name| {
                let ids = self.list_selected(&selection);
                crate::baseline::Store::new(&baseline_store).load_selected_runs(name, &ids)
            })
            .transpose()?;
        if loaded_runs.is_none() {
            self.resolve_timers(&self.selected_indices(&selection))?;
        }
        let mut live = if loaded_runs.is_some()
            || discard
            || std::env::var_os("AIRBUG_DASHBOARD").is_some_and(|v| v == "0")
        {
            None
        } else {
            let root = airbug::report::live::project_root()?;
            let live = airbug::report::live::Run::new(
                &root.join("target/airbug-report"),
                "bench",
                &self.name,
            )?;
            if self.console_output != ConsoleOutput::Quiet {
                eprintln!(
                    "Airbug run: {} · history: {}",
                    live.id,
                    live.directory.display()
                );
            }
            Some(live)
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
            if let Some(duration) = profile_time {
                let directory = output
                    .as_ref()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| {
                        std::path::PathBuf::from("target/airbug-profile").join(Run::new().id)
                    });
                let report =
                    self.profile_selected_live(&selection, duration, &directory, live.as_mut())?;
                if json {
                    println!("PROFILE_RESULT={}", serde_json::to_string(&report)?);
                } else {
                    println!(
                        "Profiled {} cases. Artifacts: {}",
                        report.cases.len(),
                        directory.display()
                    );
                }
                return Ok(());
            }
            let runs = match loaded_runs.as_ref() {
                Some(runs) => runs.clone(),
                None => vec![self.run_selected_live(&selection, live.as_mut(), test_mode)?],
            };
            let separate = runs.len() > 1;
            if separate {
                if let Some(path) = &output {
                    std::fs::create_dir(path)?;
                }
            }
            for (index, mut run) in runs.into_iter().enumerate() {
                let mut scales = self.summary_scales(&selection)?;
                if let Some(scale) = summary_scale {
                    scales.values_mut().for_each(|value| *value = scale);
                }
                crate::presentation::save_scales(&mut run, &scales)?;
                let automatic_output = if output.is_none()
                    && history_enabled
                    && !discard
                    && !test_mode
                    && loaded_runs.is_none()
                {
                    let root = match history_root.as_ref() {
                        Some(root) => root.join("reports"),
                        None => airbug::report::live::project_root()?
                            .join("target/airbug-bench/reports"),
                    };
                    std::fs::create_dir_all(&root)?;
                    Some(root.join(&run.id))
                } else {
                    None
                };
                let output = output
                    .as_ref()
                    .map(|path| {
                        let root = std::path::PathBuf::from(path);
                        if separate {
                            root.join(index.to_string())
                        } else {
                            root
                        }
                    })
                    .or(automatic_output);
                if loaded_runs.is_none() {
                    if let Some(config) = &bootstrap {
                        let settings = bootstrap_overrides
                            .iter()
                            .filter(|(id, _)| run.cases.iter().any(|case| &case.id == *id))
                            .map(|(id, value)| (id.clone(), value.clone()))
                            .collect();
                        crate::bootstrap::save_settings(&mut run, config, &settings)?;
                    }
                }
                let summaries = crate::report::descriptive(&run)?;
                let formatted = self.formatted_metrics(&run)?;
                if !formatted.is_empty() {
                    crate::measurement::save_formatted(&mut run, &formatted)?;
                }
                let estimates = bootstrap
                    .as_ref()
                    .map(|config| {
                        let overrides = bootstrap_overrides
                            .iter()
                            .filter(|(id, _)| run.cases.iter().any(|case| &case.id == *id))
                            .map(|(id, config)| (id.clone(), config.clone()))
                            .collect();
                        crate::bootstrap::analyze_with_case_configs(
                            &run,
                            config,
                            &overrides,
                            bootstrap_distributions,
                        )
                    })
                    .transpose()?;
                let rates = bytes_format
                    .map(|format| crate::report::throughput_with_format(&run, format))
                    .transpose()?;
                if let Some(p) = &output {
                    run.save_new(p)?;
                    if !formatted.is_empty() {
                        crate::model::write_new(&p.join("formatted.json"), &formatted)?;
                        std::fs::write(
                            p.join("formatted.csv"),
                            crate::measurement::report_csv(&formatted),
                        )?;
                    }
                    crate::model::write_new(
                        &std::path::Path::new(p).join("summary.json"),
                        &summaries,
                    )?;
                    if let Some(rates) = &rates {
                        crate::model::write_new(
                            &std::path::Path::new(p).join("throughput.json"),
                            rates,
                        )?;
                    }
                    if let Some(report) = &estimates {
                        let formatted_report = self.formatted_bootstrap(report)?;
                        if !formatted_report.rows.is_empty() {
                            crate::model::write_new(
                                &p.join("formatted-estimates.json"),
                                &formatted_report,
                            )?;
                            let html = if no_plots {
                                crate::report::html(&crate::bootstrap::markdown(&formatted_report))
                            } else {
                                crate::bootstrap::html(&formatted_report)
                            };
                            std::fs::write(p.join("formatted-estimates.html"), html)?;
                        }
                        crate::model::write_new(
                            &std::path::Path::new(p).join("estimates.json"),
                            report,
                        )?;
                        let mut html = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(std::path::Path::new(p).join("estimates.html"))?;
                        std::io::Write::write_all(
                            &mut html,
                            (if no_plots {
                                crate::report::html(&crate::bootstrap::markdown(report))
                            } else {
                                crate::bootstrap::html_with_case_scales(
                                    report,
                                    summary_scale,
                                    &self.summary_scales(&selection)?,
                                )
                            })
                            .as_bytes(),
                        )?;
                    }
                }
                let named_comparison = if named_baseline.is_some() {
                    Some(crate::history::compare_case_baselines_with_overrides(
                        &run,
                        &baseline_snapshot,
                        comparison_config,
                        &comparison_overrides,
                    )?)
                } else {
                    None
                };
                if let (Some(path), Some(report)) = (&output, &named_comparison) {
                    report.export_with_plots(&path.join("comparison.json"), !no_plots)?;
                    if let (Some(config), Some(candidate)) = (&bootstrap, &estimates) {
                        let mut charts = String::new();
                        let mut relative_reports = Vec::new();
                        for case in &report.cases {
                            if matches!(case.status, crate::history::PreviousStatus::Compared) {
                                if let Some(baseline) = baseline_snapshot.get(&case.case) {
                                    let config =
                                        bootstrap_overrides.get(&case.case).unwrap_or(config);
                                    let relative = crate::relative::compare_runs(
                                        &crate::history::one_case(baseline, &case.case),
                                        &crate::history::one_case(&run, &case.case),
                                        config,
                                    )?;
                                    if !no_plots {
                                        charts.push_str(&crate::relative::charts(
                                            &relative,
                                            case.config.noise_threshold_percent,
                                        )?);
                                        let baseline = crate::bootstrap::analyze(baseline, config)?;
                                        charts.push_str(&crate::bootstrap::comparison_charts(
                                            &baseline, candidate,
                                        ));
                                    }
                                    relative_reports.extend(relative);
                                }
                            } else {
                                charts.push_str(&format!(
                                    "<p>{}: {}</p>",
                                    crate::report::escape(&case.case),
                                    crate::report::escape(
                                        case.reason
                                            .as_deref()
                                            .unwrap_or("No baseline for this case")
                                    )
                                ));
                            }
                        }
                        if no_plots {
                            charts.push_str(&crate::report::html_fragment(&report.markdown()));
                        } else if charts.is_empty() {
                            charts.push_str("<p>No matching per-process regressions with varying operation counts.</p>");
                        }
                        let relative_file = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(path.join("relative-distributions.json"))?;
                        serde_json::to_writer_pretty(relative_file, &relative_reports)?;
                        let html = crate::report::html("# Baseline statistical comparison")
                            .replace("<!--CHARTS-->", &charts)
                            .replace("<!--DETAILS-->", "");
                        let mut file = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(path.join("regression-comparison.html"))?;
                        std::io::Write::write_all(&mut file, html.as_bytes())?;
                    }
                }
                let previous = if history_enabled
                    && loaded_runs.is_none()
                    && !discard
                    && !test_mode
                    && named_baseline.is_none()
                {
                    let root = match history_root.as_ref() {
                        Some(path) => path.clone(),
                        None => std::env::current_dir()?.join("target/airbug-bench/history"),
                    };
                    let executable = std::env::current_exe()?;
                    let namespace = format!(
                        "{}\0{}",
                        self.name,
                        executable.file_name().unwrap_or_default().to_string_lossy()
                    );
                    Some(
                        crate::history::History::new(root, &namespace).record_with_export_plots(
                            &run,
                            comparison_config,
                            &comparison_overrides,
                            output
                                .as_ref()
                                .map(|path| path.join("comparison.json"))
                                .as_deref(),
                            !no_plots,
                        )?,
                    )
                } else {
                    None
                };
                if let Some(path) = &output {
                    let mut html = if no_plots {
                        crate::report::html(&crate::report::markdown(&run)?)
                    } else {
                        crate::report::html_run_with_summary(
                            &run,
                            summary_parameter.as_deref().map(|parameter| {
                                crate::report::SummaryPlot {
                                    estimator: summary_estimator.unwrap_or_default(),
                                    parameter,
                                    scale: summary_scale.unwrap_or_default(),
                                }
                            }),
                        )?
                    };
                    if !no_plots {
                        if let Some(comparison) = &named_comparison {
                            let mut charts = String::new();
                            for case in &comparison.cases {
                                if matches!(case.status, crate::history::PreviousStatus::Compared) {
                                    if let Some(baseline) = baseline_snapshot.get(&case.case) {
                                        charts.push_str(&crate::report::iteration_comparison(
                                            &crate::history::one_case(baseline, &case.case),
                                            &crate::history::one_case(&run, &case.case),
                                        )?);
                                        if let Some(entry) =
                                            self.entries.iter().find(|e| e.case.id == case.case)
                                        {
                                            for (metric, formatter) in &entry.formatters {
                                                charts.push_str(
                                                    &crate::measurement::comparison_chart(
                                                        &crate::history::one_case(
                                                            baseline, &case.case,
                                                        ),
                                                        &crate::history::one_case(&run, &case.case),
                                                        &case.case,
                                                        metric,
                                                        formatter.as_ref(),
                                                    )?,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            html = html.replace("</body>", &format!("{charts}</body>"));
                        }
                    }
                    if let Some(comparison) = named_comparison.as_ref().or(previous.as_ref()) {
                        html = html.replace(
                            "</body>",
                            &format!(
                                "{}</body>",
                                crate::report::html_fragment(&comparison.markdown())
                            ),
                        );
                    }
                    let mut navigation = String::from("<nav aria-label=\"Run results\">");
                    for (file, label) in [
                        ("estimates.html", "Statistical estimates"),
                        ("formatted-estimates.html", "Custom metric estimates"),
                        ("regression-comparison.html", "Regression comparison"),
                        ("formatted.csv", "Download custom metrics (CSV)"),
                        ("run.json", "Download raw data (JSON)"),
                    ] {
                        if path.join(file).is_file() {
                            navigation.push_str(&format!("<a href=\"{file}\">{label}</a> "));
                        }
                    }
                    navigation.push_str("</nav>");
                    html = html.replacen("<main>", &format!("<main>{navigation}"), 1);
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path.join("report.html"))?;
                    std::io::Write::write_all(&mut file, html.as_bytes())?;
                    if !json {
                        println!("Report: {}", path.join("report.html").display());
                    } else {
                        println!(
                            "BENCH_REPORT={}",
                            serde_json::to_string(&path.join("report.html"))?
                        );
                    }
                }
                let saved_baseline = save_baseline
                    .as_ref()
                    .map(|(name, mode)| {
                        crate::baseline::Store::new(&baseline_store).save_cases(name, &run, *mode)
                    })
                    .transpose()?;
                if json {
                    println!("BENCH_RESULT={}", serde_json::to_string(&run)?);
                    println!("BENCH_SUMMARY={}", serde_json::to_string(&summaries)?);
                    if !formatted.is_empty() {
                        println!("BENCH_FORMATTED={}", serde_json::to_string(&formatted)?);
                    }
                    if let Some(saved) = &saved_baseline {
                        println!(
                            "BENCH_BASELINE_SAVED={}",
                            serde_json::to_string(&serde_json::json!({
                                "name": save_baseline.as_ref().unwrap().0,
                                "retained": saved.retained,
                                "manifest": saved.manifest,
                                "updated_cases": saved.updated_cases,
                            }))?
                        );
                    }
                    if let Some(report) = &named_comparison {
                        println!(
                            "BENCH_BASELINE={}",
                            serde_json::to_string(&serde_json::json!({
                                "name": named_baseline.as_ref().unwrap().0,
                                "report": report,
                            }))?
                        );
                    }
                    if let Some(previous) = &previous {
                        println!("BENCH_COMPARISON={}", serde_json::to_string(previous)?);
                    }
                    if let Some(rates) = &rates {
                        println!("BENCH_THROUGHPUT={}", serde_json::to_string(rates)?);
                    }
                    if let Some(report) = &estimates {
                        println!("BENCH_ESTIMATES={}", serde_json::to_string(report)?);
                    }
                } else if self.console_output == ConsoleOutput::Quiet
                    && self.console_format != crate::ConsoleFormat::Tree
                {
                    for row in &summaries {
                        let value = row
                            .summary
                            .as_ref()
                            .map(|s| s.median.to_string())
                            .unwrap_or_else(|| "unavailable".into());
                        println!(
                            "{} / {} [{} process {}]: median {} {}{}; missing {}",
                            row.case,
                            row.metric,
                            row.variant,
                            row.process,
                            value,
                            row.unit,
                            if row.normalized_per_operation {
                                "/op"
                            } else {
                                ""
                            },
                            row.unavailable
                        );
                    }
                    if let Some(report) = &named_comparison {
                        println!("{}", report.markdown());
                    }
                    if let Some(report) = &previous {
                        println!("{}", report.markdown());
                    }
                } else {
                    if self.console_output == ConsoleOutput::Verbose {
                        eprintln!(
                            "Benchmark environment: {}",
                            serde_json::to_string_pretty(&run.environment)?
                        );
                        for case in &run.cases {
                            eprintln!(
                                "Benchmark contract {}: {}",
                                case.id,
                                serde_json::to_string_pretty(&case.contract)?
                            );
                        }
                    }
                    if self.console_format == crate::ConsoleFormat::Tree {
                        println!("{}", crate::console::tree(&summaries));
                    } else {
                        println!(
                            "{}",
                            match bytes_format {
                                Some(format) =>
                                    crate::report::markdown_with_bytes_format(&run, format)?,
                                None => crate::report::markdown(&run)?,
                            }
                        );
                        println!("{}", crate::report::descriptive_markdown(&summaries));
                    }
                    if let Some(saved) = &saved_baseline {
                        println!(
                            "{} baseline {:?}: {}",
                            if saved.retained { "Retained" } else { "Saved" },
                            save_baseline.as_ref().unwrap().0,
                            saved.manifest.display()
                        );
                    }
                    if let Some(report) = &named_comparison {
                        println!(
                            "Named baseline {:?}\n{}",
                            named_baseline.as_ref().unwrap().0,
                            report.markdown().replacen(
                                "Previous run comparison",
                                "Named baseline comparison",
                                1
                            )
                        );
                    }
                    if let Some(previous) = &previous {
                        println!("{}", previous.markdown());
                    }
                    if let Some(report) = &estimates {
                        println!("{}", crate::bootstrap::markdown(report));
                    }
                }
            }
            Ok(())
        }));
        let result = match result {
            Ok(result) => result,
            Err(payload) => {
                if let Some(live) = live.as_mut() {
                    let message = payload
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| payload.downcast_ref::<&str>().copied())
                        .unwrap_or("Benchmark panicked with a non-string payload");
                    let _ = live.finish(false, message);
                }
                std::panic::resume_unwind(payload);
            }
        };
        if let Some(live) = live.as_mut() {
            live.finish(
                result.is_ok(),
                &match &result {
                    Ok(_) => "Benchmarks finished".into(),
                    Err(error) => error.to_string(),
                },
            )?;
        }
        result
    }
}

#[cfg(test)]
#[path = "suite_timer_tests.rs"]
mod timer_tests;

#[cfg(test)]
mod comparison_scope_tests {
    use super::*;
    use crate::history::{ComparisonConfig, ComparisonOptions};
    #[test]
    fn nested_defaults_preserve_nearest_fields_explicit_zero_and_siblings() {
        let mut suite = Suite::new("scope");
        suite.comparison_config(ComparisonConfig {
            significance_level: 0.02,
            ..Default::default()
        });
        suite.with_comparison_defaults(
            ComparisonOptions {
                seed: Some(91),
                noise_threshold_percent: Some(8.0),
                ..Default::default()
            },
            |suite| {
                suite.bench("outer", || panic!("registration must stay lazy"));
                suite.with_comparison_defaults(
                    ComparisonOptions {
                        noise_threshold_percent: Some(2.0),
                        resamples: Some(128),
                        ..Default::default()
                    },
                    |suite| {
                        suite.bench("inner", || ());
                        suite
                            .bench("explicit", || ())
                            .comparison_case(ComparisonOptions {
                                noise_threshold_percent: Some(0.0),
                                seed: Some(0),
                                ..Default::default()
                            });
                    },
                );
            },
        );
        suite.bench("sibling", || ());
        let settings: Vec<_> = suite
            .entries
            .iter()
            .map(|entry| entry.comparison.resolve(suite.comparison_config).unwrap())
            .collect();
        assert_eq!(settings[0].noise_threshold_percent, 8.0);
        assert_eq!(settings[1].noise_threshold_percent, 2.0);
        assert_eq!(settings[1].hypothesis.seed, 91);
        assert_eq!(settings[2].noise_threshold_percent, 0.0);
        assert_eq!(settings[2].hypothesis.seed, 0);
        assert_eq!(settings[2].hypothesis.resamples, 128);
        assert_eq!(settings[3].noise_threshold_percent, 5.0);
        assert_eq!(settings[3].hypothesis.seed, 0);
        assert!(
            settings
                .iter()
                .all(|config| config.significance_level == 0.02)
        );
    }
}

#[cfg(test)]
mod counter_override_tests {
    use super::*;
    #[test]
    fn runtime_threads_change_execution_and_normalization() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = AtomicUsize::new(0);
        let mut suite = Suite::new("workers");
        suite.bench_threads("parallel", 2, || {
            calls.fetch_add(1, Ordering::SeqCst);
        });
        suite.sampling(Sampling {
            iterations: Some(3),
            ..Default::default()
        });
        suite.bench("sequential", || ());
        suite.sampling(Sampling {
            iterations: Some(3),
            ..Default::default()
        });
        suite.thread_count(0).unwrap();
        assert_eq!(
            suite.entries[0].operations_per_iteration,
            crate::threads::available() as u64
        );
        assert_eq!(
            suite.entries[0].runtime_workers.as_ref().unwrap().get(),
            crate::threads::available()
        );
        assert!(suite.thread_count(257).is_err());
        suite.thread_count(4).unwrap();
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 3,
        });
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 12);
        assert_eq!(run.cases[0].contract["threads"], "4");
        assert!(!run.cases[1].contract.contains_key("threads"));
        assert_eq!(
            run.observations
                .iter()
                .find(|o| o.case.ends_with("parallel"))
                .unwrap()
                .operations,
            12
        );
        assert_eq!(
            run.observations
                .iter()
                .find(|o| o.case.ends_with("sequential"))
                .unwrap()
                .operations,
            3
        );
    }

    #[test]
    fn runtime_counter_replaces_only_its_unit_in_fixed_and_dynamic_cases() {
        let counters = crate::counters::InputCounters::new(&["items", "bytes"]).unwrap();
        let setup_counters = counters.clone();
        let mut suite = Suite::new("override");
        suite.bench_with_input(
            "dynamic",
            move || {
                setup_counters.add("items", 2);
                setup_counters.add("bytes", 4);
            },
            |_| (),
            DropPolicy::InsideTiming,
        );
        suite.input_counters(counters);
        suite.sampling(crate::Sampling {
            iterations: Some(3),
            ..Default::default()
        });
        suite.bench("fixed", || ());
        suite.work_units("items", 1);
        suite.work_units("bytes", 9);
        suite.sampling(crate::Sampling {
            iterations: Some(3),
            ..Default::default()
        });
        suite.override_work_units("items", 7);
        suite.override_work_units("chars", 0);
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 3,
        });
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        for case in &run.cases {
            let counts = crate::report::work_counters(case).unwrap();
            assert_eq!(counts["items"], Some(7));
            assert_eq!(counts["chars"], Some(0));
            assert_eq!(
                counts["bytes"],
                if case.id.ends_with("dynamic") {
                    None
                } else {
                    Some(9)
                }
            );
            assert!(!case.contract.contains_key("work.input.items"));
        }
        let dynamic = run
            .observations
            .iter()
            .find(|o| o.case.ends_with("dynamic"))
            .unwrap();
        assert_eq!(dynamic.work_totals["bytes"], "12");
        assert!(!dynamic.work_totals.contains_key("items"));
        assert_eq!(run.cases[1].contract["work.count"], "9");
    }
}
