//! Clap CLI surface for cargo-airbug-bench.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cargo airbug-bench",
    version,
    about = "Reproducible local benchmarks, explicit metrics, offline A/B reports"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Action,
    /// Storage for named baselines.
    #[arg(long, global = true, default_value = ".airbug-bench")]
    pub(crate) store: PathBuf,
}
#[derive(Subcommand)]
pub(crate) enum Action {
    /// Browse saved experiments in a local web interface.
    Serve {
        #[arg(default_value = ".airbug-bench")]
        root: PathBuf,
        #[arg(long, default_value_t = 8787)]
        port: u16,
    },
    /// Execute a Cartesian matrix of explicit worker CLI arguments, sequentially.
    Matrix {
        #[arg(long)]
        plan: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Search a bounded first-parent history; audit every commit to detect nonmonotonic regressions.
    Bisect {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        #[arg(long)]
        good: String,
        #[arg(long)]
        bad: String,
        #[arg(long)]
        target: String,
        #[arg(long, default_value = "Cargo.toml")]
        manifest_path: PathBuf,
        #[arg(long, default_value_t = 12)]
        repetitions: u32,
        #[arg(long, default_value_t = 16)]
        max_commits: usize,
        #[arg(long)]
        offline: bool,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Continue an interrupted allowlisted experiment as a new linked session.
    Resume {
        run: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Replay one recorded case under an explicit profiler executable.
    Profile {
        run: PathBuf,
        #[arg(long)]
        case: String,
        #[arg(long)]
        profiler: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Inventory storage; --apply moves eligible owned runs into reversible quarantine.
    Retention {
        #[arg(long, default_value_t = 20)]
        keep: usize,
        #[arg(long)]
        apply: bool,
    },
    /// Diagnose between-process spread, chronological drift and AB/BA effects.
    Diagnose { run: PathBuf },
    /// Plan a NEW confirmation experiment; the pilot is never pooled automatically.
    Pilot {
        run: PathBuf,
        #[arg(long, default_value_t = 5.0)]
        precision: f64,
        #[arg(long, default_value_t = 200)]
        max_processes: usize,
    },
    /// Raw frame deadline exceedance; does not infer compositor-dropped frames.
    Deadlines {
        run: PathBuf,
        #[arg(long)]
        case: String,
        #[arg(long)]
        metric: String,
        #[arg(long, value_delimiter = ',', default_value = "60,120,144")]
        hz: Vec<f64>,
    },
    /// Compare all variants against one reference with family-wise correction.
    Multi {
        run: PathBuf,
        #[arg(long, default_value = "baseline")]
        reference: String,
        #[arg(long, default_value_t = 5.0)]
        threshold: f64,
        #[arg(long, default_value_t = 0.05)]
        alpha: f64,
    },
    /// Prepare a PR comment from a completed base/head run; never publishes.
    PrReport {
        run: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Preview the exact categories and sizes included in a bundle.
    ExportPreview { run: PathBuf },

    /// Show the effort and limitations of named worker sampling profiles.
    Profiles,
    /// List recorded runs chronologically; last resolves the latest complete run.
    History {
        #[arg(long)]
        json: bool,
        /// Keep only runs with this status (case-insensitive): complete, failed, cancelled, incomplete, invalid.
        #[arg(long)]
        status: Option<String>,
        /// Keep only the most recent N runs.
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Show exact environment, provenance and workload-contract differences.
    Context {
        baseline: PathBuf,
        candidate: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Descriptive performance history of an exact case/metric.
    Trend {
        #[arg(long)]
        case: String,
        #[arg(long)]
        metric: String,
        #[arg(long)]
        json: bool,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Export raw observations, descriptors and availability.
    Export {
        run: PathBuf,
        #[arg(long)]
        format: String,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Produce a portable JSON bundle of run, report and notes (no binaries/logs).
    Bundle {
        run: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Verify and unpack a bundle into a new directory.
    Unpack {
        bundle: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Append a note without altering immutable run data.
    Note { run: PathBuf, text: String },
    /// Read the notes attached to an unchanged run.
    Notes {
        run: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Check binaries, fixtures, cwd, deadlines and output without executing workers.
    Preflight {
        #[arg(long)]
        plan: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Compare local committed Git snapshots; current checkout remains untouched.
    GitCompare {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        #[arg(long)]
        baseline: String,
        #[arg(long)]
        candidate: String,
        #[arg(long)]
        target: String,
        #[arg(long, default_value = "Cargo.toml")]
        manifest_path: PathBuf,
        #[arg(long, default_value_t = 12)]
        repetitions: u32,
        #[arg(long)]
        offline: bool,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Write a manually triggered GitHub Actions smoke workflow, never overwrite.
    Ci {
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Scaffold a checked parameterized benchmark in an existing Cargo package.
    Init {
        #[arg(long, default_value = "Cargo.toml")]
        manifest_path: PathBuf,
        #[arg(long)]
        library_path: Option<PathBuf>,
    },
    /// Discover Cargo workspace benchmark targets without executing them.
    Discover {
        #[arg(long)]
        manifest_path: Option<PathBuf>,
        #[arg(long)]
        offline: bool,
        #[arg(long)]
        json: bool,
    },
    /// Build all registered workspace targets first, then measure them sequentially.
    Bench {
        #[arg(long)]
        manifest_path: Option<PathBuf>,
        #[arg(long)]
        offline: bool,
        /// Select exact package/target (may repeat); only registered targets are runnable.
        #[arg(long)]
        target: Vec<String>,
        #[arg(long, default_value_t = 12)]
        repetitions: u32,
        #[arg(long, default_value_t = 60000)]
        timeout_ms: u64,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Register or list immutable named baseline references.
    Baseline {
        #[command(subcommand)]
        action: BaselineAction,
    },
    /// Evaluate exact-case absolute and relative budgets from JSON.
    Gate {
        run: PathBuf,
        #[arg(long, default_value = "bench.json")]
        config: PathBuf,
        #[arg(long)]
        baseline: Option<PathBuf>,
        #[arg(long)]
        json: bool,
        #[arg(long, value_enum, default_value = "fail")]
        uncertainty: Uncertainty,
    },
    /// Execute a JSON plan or one program; output directory must not exist.
    Run {
        #[arg(long)]
        plan: Option<PathBuf>,
        #[arg(long)]
        program: Option<PathBuf>,
        #[arg(long)]
        baseline: Option<PathBuf>,
        #[arg(long, default_value_t = 12)]
        repetitions: u32,
        #[arg(long, default_value_t = 60000)]
        timeout_ms: u64,
        #[arg(long)]
        protocol: bool,
        #[arg(long)]
        dry_run: bool,
        /// Start and open the live interface even without an interactive terminal.
        #[arg(long, conflicts_with = "no_ui")]
        ui: bool,
        /// Disable the automatic interface (CI/headless runs).
        #[arg(long)]
        no_ui: bool,
        /// Print the interface URL without launching a browser.
        #[arg(long)]
        no_open: bool,
        /// One diagnostic memory-profile run; requires an instrumented Rust worker.
        #[arg(long)]
        memory: bool,
        /// airbug-hub base URL (or env AIRBUG_HUB); registers a GUID run before measuring.
        #[arg(long, env = "AIRBUG_HUB")]
        hub: Option<String>,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Compare paired run, or two independent runs. Nonzero exit with --check on uncertain results.
    Compare {
        baseline: PathBuf,
        candidate: Option<PathBuf>,
        #[arg(long, default_value_t = 5.0)]
        threshold: f64,
        #[arg(long, default_value_t = 0.05)]
        alpha: f64,
        #[arg(long)]
        json: bool,
        #[arg(long, value_enum, default_value = "fail")]
        uncertainty: Uncertainty,
        #[arg(long)]
        check: bool,
        #[arg(long, default_value = "")]
        filter: String,
        #[arg(long, default_value = "")]
        metric: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Check every available observation against absolute lower/upper bounds (no statistical inference).
    Check {
        run: PathBuf,
        #[arg(long)]
        metric: String,
        /// Absolute upper bound; fail if any observation exceeds it.
        #[arg(long)]
        max: Option<f64>,
        /// Absolute lower bound; fail if any observation falls below it.
        #[arg(long)]
        min: Option<f64>,
        /// Only check cases whose id contains this substring.
        #[arg(long, default_value = "")]
        filter: String,
    },
    /// Import completed legacy Forma offscreen or paired directories.
    ImportForma {
        source: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Build an offline report for one run or an experiment tree; .html/.json/.md select format.
    Report {
        run: PathBuf,
        /// Baseline run or collection matched by relative target paths.
        #[arg(long)]
        baseline: Option<PathBuf>,
        #[arg(long, default_value = "Benchmark experiment")]
        title: String,
        #[arg(long, default_value_t = 5.0)]
        threshold: f64,
        #[arg(long, default_value_t = 0.05)]
        alpha: f64,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// List cases already recorded in a run, without executing workloads.
    List {
        run: PathBuf,
        #[arg(long, default_value = "")]
        filter: String,
        /// Emit cases with their metrics and observation counts as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Derive work-unit throughput (units/s; MiB/s for bytes) from wall batches; optional budget.
    Throughput {
        run: PathBuf,
        #[arg(long, default_value = "")]
        filter: String,
        /// Fail if any per-observation throughput exceeds this rate.
        #[arg(long)]
        max: Option<f64>,
        /// Fail if any per-observation throughput falls below this rate.
        #[arg(long)]
        min: Option<f64>,
        #[arg(long)]
        json: bool,
    },
    /// Build benchmark executables without measuring (for project harness=false benches).
    Build {
        #[arg(long)]
        manifest_path: Option<PathBuf>,
        #[arg(long)]
        offline: bool,
    },
    /// Print available host capabilities; does not change system settings.
    Doctor {
        /// Emit capabilities as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print a shell completion script to stdout; does not modify the shell configuration.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
#[derive(Clone, Copy, clap::ValueEnum)]
pub(crate) enum Uncertainty {
    Fail,
    Warn,
    Record,
}

#[derive(Subcommand)]
pub(crate) enum BaselineAction {
    Save { name: String, run: PathBuf },
    List,
}
