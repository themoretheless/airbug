//! Immutable direct-Cargo history. Only entries with a final commit marker are eligible.
use crate::{Result, Run, Status, analysis::Comparison, error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// Practical change threshold is a percentage; significance is a probability.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ComparisonConfig {
    pub noise_threshold_percent: f64,
    pub significance_level: f64,
    #[serde(default)]
    pub hypothesis: crate::hypothesis::Config,
    #[serde(default)]
    pub capture_distribution: bool,
}
/// Sparse settings for suite/group/case inheritance. Explicit zero margins and
/// seeds remain overrides; omitted fields inherit independently.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ComparisonOptions {
    pub noise_threshold_percent: Option<f64>,
    pub significance_level: Option<f64>,
    pub resamples: Option<usize>,
    pub seed: Option<u64>,
}
impl ComparisonOptions {
    pub fn inherit(&mut self, defaults: Self) {
        self.noise_threshold_percent = self
            .noise_threshold_percent
            .or(defaults.noise_threshold_percent);
        self.significance_level = self.significance_level.or(defaults.significance_level);
        self.resamples = self.resamples.or(defaults.resamples);
        self.seed = self.seed.or(defaults.seed);
    }
    pub fn resolve(self, fallback: ComparisonConfig) -> Result<ComparisonConfig> {
        let resolved = ComparisonConfig {
            noise_threshold_percent: self
                .noise_threshold_percent
                .unwrap_or(fallback.noise_threshold_percent),
            significance_level: self
                .significance_level
                .unwrap_or(fallback.significance_level),
            capture_distribution: fallback.capture_distribution,
            hypothesis: crate::hypothesis::Config {
                resamples: self.resamples.unwrap_or(fallback.hypothesis.resamples),
                seed: self.seed.unwrap_or(fallback.hypothesis.seed),
            },
        };
        resolved.validate()?;
        Ok(resolved)
    }
}
impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            noise_threshold_percent: 5.0,
            significance_level: 0.05,
            hypothesis: crate::hypothesis::Config::default(),
            capture_distribution: false,
        }
    }
}
impl ComparisonConfig {
    pub fn validate(self) -> Result<()> {
        self.hypothesis.validate()?;
        if !self.noise_threshold_percent.is_finite()
            || self.noise_threshold_percent < 0.0
            || !self.significance_level.is_finite()
            || self.significance_level <= 0.0
            || self.significance_level >= 1.0
        {
            return Err(error(
                "noise threshold must be a finite nonnegative percent; significance level must be between 0 and 1",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviousStatus {
    FirstRun,
    Compared,
    Incompatible,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviousCase {
    pub case: String,
    pub previous_run: Option<String>,
    pub status: PreviousStatus,
    pub reason: Option<String>,
    pub comparisons: Vec<Comparison>,
    /// Relative estimates for automatic history; resample draws are not retained.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relative: Vec<crate::relative::Comparison>,
    #[serde(default)]
    pub config: ComparisonConfig,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryReport {
    #[serde(default)]
    pub config: ComparisonConfig,
    pub current_run: String,
    pub cases: Vec<PreviousCase>,
}
#[derive(Serialize, Deserialize)]
struct Commit {
    version: u32,
    run_sha256: String,
    comparison_sha256: String,
}

/// Separate logical suites share a root without mixing their histories.
pub struct History {
    directory: PathBuf,
}
impl History {
    pub fn new(root: impl AsRef<Path>, namespace: &str) -> Self {
        Self {
            directory: root
                .as_ref()
                .join(crate::model::hex(&Sha256::digest(namespace.as_bytes()))),
        }
    }
    /// Compare against the latest committed result containing each case, then
    /// publish this complete run. Failed/smoke runs are never accepted. Concurrent
    /// callers compare against their own snapshot of completed entries.
    pub fn record(&self, run: &Run) -> Result<HistoryReport> {
        self.record_with_config(run, ComparisonConfig::default())
    }
    pub fn record_with_config(&self, run: &Run, config: ComparisonConfig) -> Result<HistoryReport> {
        self.record_with_overrides(run, config, &BTreeMap::new())
    }
    pub fn record_with_overrides(
        &self,
        run: &Run,
        config: ComparisonConfig,
        overrides: &BTreeMap<String, ComparisonOptions>,
    ) -> Result<HistoryReport> {
        self.record_with_export(run, config, overrides, None)
    }
    /// Write the requested comparison export before publishing this run into
    /// history, so export failures cannot advance the previous-run baseline.
    pub fn record_with_export(
        &self,
        run: &Run,
        config: ComparisonConfig,
        overrides: &BTreeMap<String, ComparisonOptions>,
        export: Option<&Path>,
    ) -> Result<HistoryReport> {
        self.record_with_export_plots(run, config, overrides, export, true)
    }
    /// Control SVG generation without changing numerical comparisons or publication order.
    pub fn record_with_export_plots(
        &self,
        run: &Run,
        config: ComparisonConfig,
        overrides: &BTreeMap<String, ComparisonOptions>,
        export: Option<&Path>,
        plots: bool,
    ) -> Result<HistoryReport> {
        self.record_with_analysis(
            &mut run.clone(),
            config,
            overrides,
            (export, plots, None),
            |_, _, _, _| Ok(()),
        )
    }

    /// Enrich relative estimates and candidate provenance on a temporary copy
    /// before exporting and committing history. The callback must preserve raw
    /// observations and contracts. Failure leaves the caller and previous entry intact.
    pub(crate) fn record_with_analysis(
        &self,
        run: &mut Run,
        config: ComparisonConfig,
        overrides: &BTreeMap<String, ComparisonOptions>,
        output: (Option<&Path>, bool, Option<&Path>),
        mut format: impl FnMut(
            &Run,
            &mut Run,
            &crate::bootstrap::Config,
            &mut [crate::relative::Comparison],
        ) -> Result<()>,
    ) -> Result<HistoryReport> {
        let (export, plots, run_export) = output;
        validate_overrides(run, config, overrides)?;
        config.validate()?;
        run.validate()?;
        if run.status != Status::Complete
            || run.cases.iter().any(|case| {
                case.contract
                    .get("execution.mode")
                    .is_some_and(|mode| mode == "test_once")
            })
        {
            return Err(error(
                "history requires a complete benchmark run, not a smoke test",
            ));
        }
        if run.observations.iter().any(|o| o.variant != "candidate") {
            return Err(error("direct history requires candidate-only observations"));
        }
        let mut previous = BTreeMap::new();
        let mut paths = match fs::read_dir(&self.directory) {
            Ok(entries) => entries
                .map(|entry| entry.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => vec![],
            Err(err) => return Err(err.into()),
        };
        paths.sort();
        for path in paths.into_iter().rev() {
            if !path.join("COMMITTED").is_file() {
                continue;
            }
            let commit: Commit = serde_json::from_slice(&fs::read(path.join("COMMITTED"))?)?;
            if commit.version != 1
                || crate::model::hash_file(&path.join("run.json"))? != commit.run_sha256
                || crate::model::hash_file(&path.join("comparison.json"))?
                    != commit.comparison_sha256
            {
                return Err(error(
                    "history artifact changed after commit or has unsupported version",
                ));
            }
            let old = Run::load(path.join("run.json"))?;
            if old.status != Status::Complete {
                return Err(error("committed history contains an incomplete run"));
            }
            for case in &run.cases {
                if !previous.contains_key(&case.id) && old.cases.iter().any(|c| c.id == case.id) {
                    previous.insert(case.id.clone(), one_case(&old, &case.id));
                }
            }
            if previous.len() == run.cases.len() {
                break;
            }
        }
        let mut persisted = run.clone();
        let mut report = compare_cases(run, &previous, config, overrides)?;
        for case in &mut report.cases {
            if matches!(case.status, PreviousStatus::Compared) {
                if let Some(old) = previous.get(&case.case) {
                    let settings = crate::bootstrap::Config {
                        resamples: case.config.hypothesis.resamples,
                        seed: case.config.hypothesis.seed,
                        confidence_level: (1. - case.config.significance_level)
                            .clamp(f64::EPSILON, 1. - f64::EPSILON),
                    };
                    case.relative =
                        crate::relative::compare_runs(old, &one_case(run, &case.case), &settings)?;
                    format(old, &mut persisted, &settings, &mut case.relative)?;
                    crate::relative_format::save(old, &mut persisted, &settings, &case.relative)?;
                    for row in &mut case.relative {
                        if let Some(report) = &mut row.report {
                            report.mean.draws_percent.clear();
                            report.median.draws_percent.clear();
                        }
                    }
                }
            }
        }
        if let Some(path) = export {
            report.export_with_plots(path, plots)?;
        }
        if let Some(path) = run_export {
            crate::model::write_new(path, &persisted)?;
        }
        fs::create_dir_all(&self.directory)?;
        // Generated ID rather than run.id: caller-supplied IDs are data, never paths.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let destination = self.directory.join(format!(
            "{}-{:016x}",
            Run::new().id,
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&destination)?;
        crate::model::write_new(&destination.join("run.json"), &persisted)?;
        crate::model::write_new(&destination.join("comparison.json"), &report)?;
        crate::model::write_new(
            &destination.join("COMMITTING"),
            &Commit {
                version: 1,
                run_sha256: crate::model::hash_file(&destination.join("run.json"))?,
                comparison_sha256: crate::model::hash_file(&destination.join("comparison.json"))?,
            },
        )?;
        fs::rename(
            destination.join("COMMITTING"),
            destination.join("COMMITTED"),
        )?;
        *run = persisted;
        Ok(report)
    }
}
/// Compare a loaded baseline snapshot without changing history or artifacts.
pub fn compare_baseline(run: &Run, baseline: Option<&Run>) -> Result<HistoryReport> {
    run.validate()?;
    if let Some(old) = baseline {
        old.validate()?;
    }
    let previous = baseline
        .map(|old| {
            old.cases
                .iter()
                .map(|case| (case.id.clone(), one_case(old, &case.id)))
                .collect()
        })
        .unwrap_or_default();
    compare_cases(
        run,
        &previous,
        ComparisonConfig::default(),
        &BTreeMap::new(),
    )
}
/// Compare per-case baseline snapshots without assigning one run's environment
/// to observations that came from another run.
pub fn compare_case_baselines(
    run: &Run,
    previous: &BTreeMap<String, Run>,
) -> Result<HistoryReport> {
    compare_case_baselines_with_config(run, previous, ComparisonConfig::default())
}
pub fn compare_case_baselines_with_config(
    run: &Run,
    previous: &BTreeMap<String, Run>,
    config: ComparisonConfig,
) -> Result<HistoryReport> {
    compare_case_baselines_with_overrides(run, previous, config, &BTreeMap::new())
}
pub fn compare_case_baselines_with_overrides(
    run: &Run,
    previous: &BTreeMap<String, Run>,
    config: ComparisonConfig,
    overrides: &BTreeMap<String, ComparisonOptions>,
) -> Result<HistoryReport> {
    validate_overrides(run, config, overrides)?;
    config.validate()?;
    run.validate()?;
    let previous = previous
        .iter()
        .map(|(id, old)| {
            old.validate()?;
            if !old.cases.iter().any(|case| case.id == *id) {
                return Err(error(format!("baseline snapshot is missing case {id:?}")));
            }
            Ok((id.clone(), one_case(old, id)))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    compare_cases(run, &previous, config, overrides)
}
fn compare_cases(
    run: &Run,
    previous: &BTreeMap<String, Run>,
    config: ComparisonConfig,
    overrides: &BTreeMap<String, ComparisonOptions>,
) -> Result<HistoryReport> {
    let mut report = HistoryReport {
        config,
        current_run: run.id.clone(),
        cases: vec![],
    };
    for case in &run.cases {
        let config = overrides
            .get(&case.id)
            .copied()
            .unwrap_or_default()
            .resolve(config)?;
        let mut row = PreviousCase {
            config,
            case: case.id.clone(),
            previous_run: None,
            status: PreviousStatus::FirstRun,
            reason: None,
            comparisons: vec![],
            relative: vec![],
        };
        if let Some(old) = previous.get(&case.id) {
            row.previous_run = Some(old.id.clone());
            let compare = if config.capture_distribution {
                crate::analysis::compare_with_distribution
            } else {
                crate::analysis::compare_with_hypothesis
            };
            match compare(
                old,
                Some(&one_case(run, &case.id)),
                config.noise_threshold_percent,
                config.significance_level / run.cases.len().max(1) as f64,
                config.hypothesis,
            ) {
                Ok(comparisons) => {
                    row.status = PreviousStatus::Compared;
                    row.comparisons = comparisons;
                }
                Err(err) => {
                    row.status = PreviousStatus::Incompatible;
                    row.reason = Some(err.to_string());
                }
            }
        }
        report.cases.push(row);
    }
    Ok(report)
}
fn validate_overrides(
    run: &Run,
    config: ComparisonConfig,
    overrides: &BTreeMap<String, ComparisonOptions>,
) -> Result<()> {
    config.validate()?;
    for case in &run.cases {
        overrides
            .get(&case.id)
            .copied()
            .unwrap_or_default()
            .resolve(config)?;
    }
    Ok(())
}
pub(crate) fn one_case(run: &Run, id: &str) -> Run {
    let mut selected = run.clone();
    selected.cases.retain(|c| c.id == id);
    // Realized sample counts and adaptive stop reasons are outcomes, not
    // workload semantics. Preserve every lifecycle/configuration contract.
    for case in &mut selected.cases {
        case.contract.remove("samples");
        case.contract.remove("quick.stop");
    }
    selected.observations.retain(|o| o.case == id);
    selected.worker_allocations.retain(|w| w.case == id);
    selected.worker_timings.retain(|w| w.case == id);
    selected
}
impl HistoryReport {
    /// Export JSON and, when null draws were requested, an adjacent HTML report.
    /// Call before publishing history or replacing a named baseline.
    pub fn export(&self, json_path: &Path) -> Result<()> {
        self.export_with_plots(json_path, true)
    }
    pub fn export_with_plots(&self, json_path: &Path, plots: bool) -> Result<()> {
        let capture = self.config.capture_distribution
            || self
                .cases
                .iter()
                .any(|case| case.config.capture_distribution);
        let html = if capture && plots {
            let rows: Vec<_> = self
                .cases
                .iter()
                .flat_map(|case| case.comparisons.iter().cloned())
                .collect();
            Some(crate::hypothesis_plot::html(&rows, "Benchmark comparison")?)
        } else {
            None
        };
        crate::model::write_new(json_path, self)?;
        if let Some(html) = html {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(json_path.with_extension("html"))?;
            file.write_all(html.as_bytes())?;
            file.sync_all()?;
        }
        Ok(())
    }
    pub fn markdown(&self) -> String {
        self.markdown_impl(true)
    }
    pub(crate) fn concise_markdown(&self) -> String {
        self.markdown_impl(false)
    }
    fn markdown_impl(&self, include_relative: bool) -> String {
        let mut output = format!(
            "## Previous run comparison\n\nNoise threshold: {}%; family significance level: {}.\n\n",
            self.config.noise_threshold_percent, self.config.significance_level
        );
        for case in &self.cases {
            // Debug formatting escapes control characters in arbitrary case names.
            output.push_str(&format!("Case {:?}: {:?}; noise threshold {}%; significance {}; hypothesis resamples {}, seed {}", case.case, case.status, case.config.noise_threshold_percent, case.config.significance_level, case.config.hypothesis.resamples, case.config.hypothesis.seed));
            if let Some(reason) = &case.reason {
                output.push_str(&format!(" ({reason})"));
            }
            output.push_str("\n\n");
            if !case.comparisons.is_empty() {
                output.push_str(&crate::report::comparison(&case.comparisons));
            }
            if include_relative && !case.relative.is_empty() {
                output.push_str(&crate::relative::markdown(&case.relative));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(cases: &[(&str, u128)]) -> Run {
        let mut recorder = crate::Recorder::new();
        for &(id, value) in cases {
            recorder
                .case(crate::Case {
                    id: id.into(),
                    contract: Default::default(),
                    metrics: vec![crate::Metric::duration("wall", "test", "batch_total")],
                })
                .unwrap();
            recorder.observe(id, "wall", value).unwrap();
        }
        recorder.finish().unwrap()
    }
    #[test]
    fn relative_formatter_failure_does_not_publish_or_advance_history() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path().join("history"), "formatter");
        let first = run(&[("a", 10)]);
        history.record(&first).unwrap();
        let output = root.path().join("comparison.json");
        let mut candidate = run(&[("a", 20)]);
        let original = serde_json::to_vec(&candidate).unwrap();
        let result = history.record_with_analysis(
            &mut candidate,
            ComparisonConfig::default(),
            &BTreeMap::new(),
            (Some(&output), false, None),
            |_, candidate, _, _| {
                candidate
                    .provenance
                    .insert("partial-display".into(), "uncommitted".into());
                Err(error("formatter failed"))
            },
        );
        assert!(result.unwrap_err().to_string().contains("formatter failed"));
        assert_eq!(original, serde_json::to_vec(&candidate).unwrap());
        assert!(!output.exists());
        let next = history.record(&run(&[("a", 30)])).unwrap();
        assert_eq!(
            next.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
    }

    #[test]
    fn run_export_failure_does_not_advance_history_or_mutate_caller() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path().join("history"), "run-export");
        let first = run(&[("a", 10)]);
        history.record(&first).unwrap();
        let mut candidate = run(&[("a", 20)]);
        let original = serde_json::to_vec(&candidate).unwrap();
        let output = root.path().join("run.json");
        std::fs::write(&output, "existing artifact").unwrap();
        let result = history.record_with_analysis(
            &mut candidate,
            ComparisonConfig::default(),
            &BTreeMap::new(),
            (None, false, Some(&output)),
            |_, _, _, _| Ok(()),
        );
        assert!(result.is_err());
        assert_eq!(original, serde_json::to_vec(&candidate).unwrap());
        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            "existing artifact"
        );
        let next = history.record(&run(&[("a", 30)])).unwrap();
        assert_eq!(
            next.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
    }

    #[test]
    fn html_export_failure_preserves_history_and_existing_artifact() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path().join("history"), "html-export");
        let first = run(&[("a", 10)]);
        history.record(&first).unwrap();
        let config = ComparisonConfig {
            capture_distribution: true,
            ..Default::default()
        };
        let blocked = root.path().join("blocked.json");
        fs::write(blocked.with_extension("html"), b"keep").unwrap();
        assert!(
            history
                .record_with_export(&run(&[("a", 20)]), config, &BTreeMap::new(), Some(&blocked))
                .is_err()
        );
        assert_eq!(fs::read(blocked.with_extension("html")).unwrap(), b"keep");
        let destination = root.path().join("comparison.json");
        let report = history
            .record_with_export(
                &run(&[("a", 30)]),
                config,
                &BTreeMap::new(),
                Some(&destination),
            )
            .unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
        let html = fs::read_to_string(destination.with_extension("html")).unwrap();
        assert!(html.contains("<svg "));
        assert!(html.contains("at least two independent units"));
    }
    #[test]
    fn history_without_plots_retains_json_and_advances_previous_run() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path().join("history"), "no-plots");
        let first = run(&[("a", 10)]);
        history.record(&first).unwrap();
        let second = run(&[("a", 20)]);
        let output = root.path().join("comparison.json");
        let config = ComparisonConfig {
            capture_distribution: true,
            ..Default::default()
        };
        let report = history
            .record_with_export_plots(&second, config, &BTreeMap::new(), Some(&output), false)
            .unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&fs::read(output).unwrap()).unwrap()
        );
        assert!(!root.path().join("comparison.html").exists());
        let next = history.record(&run(&[("a", 30)])).unwrap();
        assert_eq!(
            next.cases[0].previous_run.as_deref(),
            Some(second.id.as_str())
        );
    }

    #[test]
    fn comparison_export_failure_does_not_advance_history() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path().join("history"), "export");
        let first = run(&[("a", 10)]);
        history.record(&first).unwrap();
        let blocked = root.path().join("comparison.json");
        std::fs::write(&blocked, b"existing").unwrap();
        assert!(
            history
                .record_with_export(
                    &run(&[("a", 20)]),
                    ComparisonConfig::default(),
                    &BTreeMap::new(),
                    Some(&blocked)
                )
                .is_err()
        );
        assert_eq!(std::fs::read(&blocked).unwrap(), b"existing");
        let destination = root.path().join("export.json");
        let report = history
            .record_with_export(
                &run(&[("a", 30)]),
                ComparisonConfig::default(),
                &BTreeMap::new(),
                Some(&destination),
            )
            .unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
        let saved: HistoryReport =
            serde_json::from_slice(&std::fs::read(destination).unwrap()).unwrap();
        assert_eq!(saved.current_run, report.current_run);
    }
    #[test]
    fn case_overrides_are_recorded_and_invalid_options_do_not_publish() {
        let current = run(&[("a", 10), ("b", 20)]);
        let options = BTreeMap::from([(
            "a".into(),
            ComparisonOptions {
                noise_threshold_percent: Some(0.0),
                significance_level: Some(0.01),
                seed: Some(7),
                resamples: Some(128),
            },
        )]);
        let report = compare_case_baselines_with_overrides(
            &current,
            &BTreeMap::new(),
            ComparisonConfig::default(),
            &options,
        )
        .unwrap();
        assert_eq!(report.cases[0].config.noise_threshold_percent, 0.0);
        assert_eq!(report.cases[0].config.significance_level, 0.01);
        assert_eq!(report.cases[0].config.hypothesis.seed, 7);
        assert_eq!(report.cases[1].config.noise_threshold_percent, 5.0);
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "scope");
        let invalid = BTreeMap::from([(
            "b".into(),
            ComparisonOptions {
                resamples: Some(0),
                ..Default::default()
            },
        )]);
        assert!(
            history
                .record_with_overrides(&current, ComparisonConfig::default(), &invalid)
                .is_err()
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        let recorded = history
            .record_with_overrides(&current, ComparisonConfig::default(), &options)
            .unwrap();
        assert_eq!(recorded.cases[0].config.hypothesis.resamples, 128);
    }
    #[test]
    fn comparison_configuration_changes_decisions_and_rejects_invalid_values() {
        let mut old = run(&[("a", 100)]);
        let mut current = run(&[("a", 110)]);
        for run in [&mut old, &mut current] {
            let observation = run.observations[0].clone();
            run.observations = (0..20)
                .map(|process| {
                    let mut o = observation.clone();
                    o.process = process;
                    o
                })
                .collect();
        }
        let previous = BTreeMap::from([("a".into(), old)]);
        let decision = |config| {
            compare_case_baselines_with_config(&current, &previous, config)
                .unwrap()
                .cases[0]
                .comparisons[0]
                .decision
                .clone()
        };
        assert_eq!(
            decision(ComparisonConfig::default()),
            crate::analysis::Decision::Regression
        );
        assert_eq!(
            decision(ComparisonConfig {
                noise_threshold_percent: 20.0,
                ..ComparisonConfig::default()
            }),
            crate::analysis::Decision::WithinMargin
        );
        assert_eq!(
            decision(ComparisonConfig {
                significance_level: 1e-12,
                ..ComparisonConfig::default()
            }),
            crate::analysis::Decision::Inconclusive
        );
        for value in [f64::NAN, f64::INFINITY, -1.0, 1.0, 0.0] {
            assert!(
                ComparisonConfig {
                    significance_level: value,
                    ..ComparisonConfig::default()
                }
                .validate()
                .is_err()
            );
        }
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("must-not-exist");
        assert!(
            History::new(&destination, "test")
                .record_with_config(
                    &current,
                    ComparisonConfig {
                        noise_threshold_percent: f64::INFINITY,
                        ..ComparisonConfig::default()
                    }
                )
                .is_err()
        );
        assert!(!destination.exists());
    }
    #[test]
    fn case_baselines_compare_each_original_environment_and_run_identity() {
        let current = run(&[("a", 20), ("b", 30)]);
        let old_a = run(&[("a", 10)]);
        let mut old_b = run(&[("b", 15)]);
        old_b.environment.insert("os".into(), "different-os".into());
        let previous = BTreeMap::from([("a".into(), old_a.clone()), ("b".into(), old_b.clone())]);
        let report = compare_case_baselines(&current, &previous).unwrap();
        assert!(matches!(report.cases[0].status, PreviousStatus::Compared));
        assert!(matches!(
            report.cases[1].status,
            PreviousStatus::Incompatible
        ));
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(old_a.id.as_str())
        );
        assert_eq!(
            report.cases[1].previous_run.as_deref(),
            Some(old_b.id.as_str())
        );
        assert!(
            compare_case_baselines(&current, &BTreeMap::from([("absent".into(), old_a)])).is_err()
        );
    }
    #[test]
    fn automatic_history_retains_relative_summaries_without_draws() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "relative");
        let mut baseline = run(&[("a", 10)]);
        baseline.cases[0]
            .contract
            .insert("work.counter.items".into(), "2".into());
        let mut observation = baseline.observations[0].clone();
        observation.sequence = 1;
        baseline.observations.push(observation);
        let settings = ComparisonConfig {
            significance_level: 0.1,
            hypothesis: crate::hypothesis::Config {
                resamples: 64,
                seed: 7,
            },
            ..Default::default()
        };
        assert!(
            history
                .record_with_config(&baseline, settings)
                .unwrap()
                .cases[0]
                .relative
                .is_empty()
        );
        let mut candidate = baseline.clone();
        candidate.id = "next".into();
        for observation in &mut candidate.observations {
            observation.value = Some("20".into());
        }
        let report = history.record_with_config(&candidate, settings).unwrap();
        let row = &report.cases[0].relative[0];
        let estimate = row.report.as_ref().unwrap();
        assert_eq!(estimate.mean.point_percent, Some(100.));
        assert_eq!(estimate.mean.interval_percent, Some((100., 100.)));
        assert_eq!(estimate.config.confidence_level, 0.9);
        assert_eq!((estimate.config.resamples, estimate.config.seed), (64, 7));
        assert_eq!(row.throughput[0].point_percent, Some(-50.));
        assert!(estimate.mean.draws_percent.is_empty());
        assert!(report.markdown().contains("Relative bootstrap estimates"));
        let encoded = serde_json::to_string(&report).unwrap();
        assert!(!encoded.contains("draws_percent"));
        let restored: HistoryReport = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            restored.cases[0].relative[0].report.as_ref().unwrap().mean,
            estimate.mean
        );
        let mut legacy = serde_json::to_value(&report).unwrap();
        legacy["cases"][0]
            .as_object_mut()
            .unwrap()
            .remove("relative");
        assert!(
            serde_json::from_value::<HistoryReport>(legacy)
                .unwrap()
                .cases[0]
                .relative
                .is_empty()
        );
    }

    #[test]
    fn previous_is_per_case_and_does_not_skip_incompatible_latest_results() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "suite");
        let first = run(&[("a", 100), ("b", 200)]);
        assert!(
            history
                .record(&first)
                .unwrap()
                .cases
                .iter()
                .all(|c| matches!(c.status, PreviousStatus::FirstRun))
        );
        let second = run(&[("a", 120)]);
        let report = history.record(&second).unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
        assert!((report.cases[0].comparisons[0].change_percent.unwrap() - 20.0).abs() < 1e-10);
        assert!(matches!(
            report.cases[0].comparisons[0].decision,
            crate::analysis::Decision::Inconclusive
        ));
        let third = run(&[("a", 120), ("b", 180)]);
        let report = history.record(&third).unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(second.id.as_str())
        );
        assert_eq!(
            report.cases[1].previous_run.as_deref(),
            Some(first.id.as_str())
        );
        let mut changed = run(&[("a", 50)]);
        changed.cases[0]
            .contract
            .insert("lifecycle".into(), "changed".into());
        assert!(matches!(
            history.record(&changed).unwrap().cases[0].status,
            PreviousStatus::Incompatible
        ));
        let report = history.record(&second).unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(changed.id.as_str())
        );
        assert!(matches!(
            report.cases[0].status,
            PreviousStatus::Incompatible
        ));
    }
    #[test]
    fn failed_smoke_partial_and_other_namespaces_do_not_replace_history() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "../suite");
        let mut first = run(&[("a", 100)]);
        first.id = "../../not-a-path".into();
        history.record(&first).unwrap();
        let interrupted = history.directory.join("zz-interrupted");
        fs::create_dir(&interrupted).unwrap();
        fs::write(interrupted.join("run.json"), "incomplete JSON").unwrap();
        let mut failed = run(&[("a", 30)]);
        failed.status = Status::Failed;
        assert!(history.record(&failed).is_err());
        let mut smoke = run(&[("a", 10)]);
        smoke.cases[0]
            .contract
            .insert("execution.mode".into(), "test_once".into());
        assert!(history.record(&smoke).is_err());
        let current = run(&[("a", 110)]);
        let report = history.record(&current).unwrap();
        assert_eq!(
            report.cases[0].previous_run.as_deref(),
            Some(first.id.as_str())
        );
        assert!(matches!(
            History::new(root.path(), "other")
                .record(&current)
                .unwrap()
                .cases[0]
                .status,
            PreviousStatus::FirstRun
        ));
        // A committed artifact is authoritative: corruption is surfaced, never
        // silently replaced by an older run or a new first-run baseline.
        fs::write(interrupted.join("COMMITTED"), "").unwrap();
        assert!(history.record(&current).is_err());
    }

    #[test]
    fn valid_json_edits_are_detected_before_publishing_another_entry() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "integrity");
        let first = run(&[("a", 100)]);
        history.record(&first).unwrap();
        let entry = fs::read_dir(&history.directory)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let mut edited = first.clone();
        edited.observations[0].value = Some("999".into());
        fs::write(entry.join("run.json"), serde_json::to_vec(&edited).unwrap()).unwrap();
        assert!(
            history
                .record(&first)
                .unwrap_err()
                .to_string()
                .contains("artifact changed")
        );
        assert_eq!(fs::read_dir(&history.directory).unwrap().count(), 1);
    }

    #[test]
    fn realized_sampling_outcomes_can_differ_but_environments_cannot() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "outcomes");
        let mut old = run(&[("a", 100)]);
        old.cases[0].contract.insert("samples".into(), "30".into());
        old.cases[0]
            .contract
            .insert("quick.stop".into(), "stable".into());
        history.record(&old).unwrap();
        let mut current = run(&[("a", 120)]);
        current.cases[0]
            .contract
            .insert("samples".into(), "40".into());
        current.cases[0]
            .contract
            .insert("quick.stop".into(), "max_time".into());
        assert!(matches!(
            history.record(&current).unwrap().cases[0].status,
            PreviousStatus::Compared
        ));
        current
            .environment
            .insert("arch".into(), "different".into());
        let changed = history.record(&current).unwrap();
        assert!(matches!(
            changed.cases[0].status,
            PreviousStatus::Incompatible
        ));
        assert!(
            changed.cases[0]
                .reason
                .as_ref()
                .unwrap()
                .contains("environments")
        );
    }

    #[test]
    fn concurrent_writers_publish_distinct_complete_entries() {
        let root = tempfile::tempdir().unwrap();
        let history = History::new(root.path(), "concurrent");
        let barrier = std::sync::Barrier::new(4);
        let ids = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|i| {
                    let history = &history;
                    let barrier = &barrier;
                    scope.spawn(move || {
                        let current = run(&[("a", 100 + i)]);
                        barrier.wait();
                        history.record(&current).unwrap();
                        current.id
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(fs::read_dir(&history.directory).unwrap().count(), 4);
        let report = history.record(&run(&[("a", 110)])).unwrap();
        assert!(ids.contains(report.cases[0].previous_run.as_ref().unwrap()));
        assert!(matches!(report.cases[0].status, PreviousStatus::Compared));
    }
}
