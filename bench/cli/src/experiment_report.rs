//! Offline report assembly: discovery is separate from statistical comparisons; no pooling.
use crate::{artifacts, project};
use airbug_bench::{
    analysis::{self, Comparison, Decision},
    model::hash_file,
    report, *,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
#[derive(Serialize)]
pub struct Entry {
    pub label: String,
    pub source: String,
    pub sha256: Option<String>,
    pub baseline_source: Option<String>,
    pub baseline_sha256: Option<String>,
    pub status: String,
    pub outcome: String,
    pub issues: Vec<String>,
    pub comparisons: Vec<Row>,
    pub unavailable_observations: usize,
    pub run: Option<Run>,
    #[serde(skip)]
    baseline_run: Option<Run>,
}
#[derive(Serialize)]
pub struct Row {
    pub variant: String,
    #[serde(flatten)]
    pub comparison: Comparison,
}
#[derive(Serialize)]
pub struct Document {
    pub schema: u32,
    pub title: String,
    pub threshold_percent: f64,
    pub alpha_family: f64,
    pub family_run_count: usize,
    pub counts: BTreeMap<String, usize>,
    pub entries: Vec<Entry>,
}
pub struct Options<'a> {
    pub source: &'a Path,
    pub baseline: Option<&'a Path>,
    pub store: &'a Path,
    pub title: &'a str,
    pub threshold: f64,
    pub alpha: f64,
}
pub(crate) fn files(root: &Path) -> Result<Vec<(String, PathBuf)>> {
    if root.is_file() {
        return Ok(vec![(
            root.file_name().unwrap().to_string_lossy().into(),
            root.into(),
        )]);
    }
    if !root.is_dir() {
        return Err(error("report source is not a file or directory"));
    }
    let mut found = vec![];
    let mut visited = 0;
    fn walk(
        root: &Path,
        p: &Path,
        depth: usize,
        visited: &mut usize,
        found: &mut Vec<(String, PathBuf)>,
    ) -> Result<()> {
        *visited += 1;
        if *visited > 20000 || depth > 12 {
            return Err(error(
                "report discovery limit exceeded; choose a narrower source",
            ));
        }
        let run = p.join("run.json");
        if fs::symlink_metadata(&run).is_ok() {
            if run.is_symlink() || !run.is_file() {
                return Err(error("run.json must be a regular file, not a symlink"));
            }
            let label = p.strip_prefix(root)?.to_string_lossy();
            found.push((
                if label.is_empty() {
                    "run".into()
                } else {
                    label.into()
                },
                run,
            ));
            return Ok(());
        }
        // Preserve interrupted experiments that have not yet published a run artifact.
        if p.join("status.json").is_file() && p.join("plan.json").is_file() {
            found.push((p.strip_prefix(root)?.to_string_lossy().into(), run));
            return Ok(());
        }
        for e in fs::read_dir(p)? {
            let e = e?;
            if e.file_type()?.is_dir()
                && !matches!(
                    e.file_name().to_str(),
                    Some(
                        "target"
                            | ".git"
                            | "checkouts"
                            | "baselines"
                            | "baseline-views"
                            | "notes"
                            | "quarantine"
                            | "logs"
                    )
                )
            {
                walk(root, &e.path(), depth + 1, visited, found)?;
            }
        }
        Ok(())
    }
    walk(root, root, 0, &mut visited, &mut found)?;
    found.sort_by(|a, b| a.0.cmp(&b.0));
    if found.is_empty() {
        return Err(error("no run artifacts found in experiment"));
    }
    if found.len() > 256 {
        return Err(error(
            "report limited to 256 runs; choose a narrower experiment",
        ));
    }
    Ok(found)
}
fn read(path: &Path, store: &Path) -> Result<(Run, String)> {
    if path.is_symlink() {
        return Err(error("run symlink refused"));
    }
    if fs::metadata(path)?.len() > 64 * 1024 * 1024 {
        return Err(error("individual report input exceeds 64 MiB"));
    }
    let before = hash_file(path)?;
    let mut r = Run::load(path)?;
    for note in artifacts::notes(store, path)? {
        r.notes.push(format!("User note: {}", note.text));
    }
    if before != hash_file(path)? {
        return Err(error("run changed while reporting"));
    }
    Ok((r, before))
}
pub fn build(o: Options<'_>) -> Result<Document> {
    if !o.threshold.is_finite()
        || !(0.0..100.0).contains(&o.threshold)
        || !o.alpha.is_finite()
        || !(0.0..1.0).contains(&o.alpha)
        || o.alpha == 0.
    {
        return Err(error("threshold 0..100 and alpha 0..1 required"));
    }
    let source = project::resolve_report(o.store, o.source)?;
    let candidates = files(&source)?;
    let baseline = o
        .baseline
        .map(|p| project::resolve_report(o.store, p))
        .transpose()?;
    let baselines = baseline.as_deref().map(files).transpose()?;
    if baselines
        .as_ref()
        .is_some_and(|b| b.len() == 1 && candidates.len() > 1)
    {
        return Err(error(
            "a single baseline cannot be broadcast across a collection; supply a matching baseline collection",
        ));
    }
    if baselines
        .as_ref()
        .is_some_and(|b| b.len() > 1 && candidates.len() == 1)
    {
        return Err(error("a single candidate needs a single baseline run"));
    }
    let family = candidates.len();
    let mut entries = vec![];
    let mut total_bytes = baselines
        .as_ref()
        .map(|b| {
            b.iter()
                .map(|(_, p)| fs::metadata(p).map(|m| m.len()).unwrap_or(0))
                .sum::<u64>()
        })
        .unwrap_or(0);
    for (label, path) in &candidates {
        total_bytes += fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if total_bytes > 128 * 1024 * 1024 {
            return Err(error("report inputs exceed 128 MiB total"));
        }
        let mut entry = Entry {
            label: label.clone(),
            source: path.to_string_lossy().into(),
            sha256: None,
            baseline_source: None,
            baseline_sha256: None,
            status: "invalid".into(),
            outcome: "error".into(),
            issues: vec![],
            comparisons: vec![],
            unavailable_observations: 0,
            run: None,
            baseline_run: None,
        };
        match read(path, o.store) {
            Err(e) => {
                if !path.exists() {
                    entry.status = "incomplete".into();
                    entry.issues.push(
                        "Run artifact not published: experiment may be active or interrupted."
                            .into(),
                    );
                }
                entry.issues.push(e.to_string());
            }
            Ok((r, sha)) => {
                entry.sha256 = Some(sha);
                entry.status = format!("{:?}", r.status).to_lowercase();
                entry.unavailable_observations = r
                    .observations
                    .iter()
                    .filter(|x| x.availability != Availability::Available)
                    .count();
                if r.status != Status::Complete {
                    entry.issues.push(
                        "Run is not complete; partial observations are diagnostic only.".into(),
                    );
                } else if r
                    .provenance
                    .get("user.session.policy")
                    .is_some_and(|s| s.contains("profiler"))
                {
                    entry.outcome = "diagnostic".into();
                    entry.issues.push("Profiler replay is perturbed diagnostic evidence, excluded from comparisons.".into());
                } else {
                    let variants: BTreeSet<_> =
                        r.observations.iter().map(|x| x.variant.as_str()).collect();
                    let comparison = (|| -> Result<Vec<Row>> {
                        if let Some(b) = &baselines {
                            let p = if candidates.len() == 1 && b.len() == 1 {
                                &b[0].1
                            } else {
                                &b.iter()
                                    .find(|(name, _)| name == label)
                                    .ok_or_else(|| error("matching baseline path absent"))?
                                    .1
                            };
                            let (b, sha) = read(p, o.store)?;
                            entry.baseline_source = Some(p.to_string_lossy().into());
                            entry.baseline_sha256 = Some(sha);
                            if b.provenance
                                .get("user.session.policy")
                                .is_some_and(|s| s.contains("profiler"))
                            {
                                return Err(error(
                                    "diagnostic profiler run cannot serve as baseline",
                                ));
                            }
                            let comparisons = analysis::compare(
                                &b,
                                Some(&r),
                                o.threshold,
                                o.alpha / family as f64,
                            )?;
                            entry.baseline_run = Some(b);
                            Ok(comparisons
                                .into_iter()
                                .map(|comparison| Row {
                                    variant: "candidate".into(),
                                    comparison,
                                })
                                .collect())
                        } else if variants.contains("baseline") && variants.len() > 1 {
                            Ok(analysis::compare_multi(
                                &r,
                                "baseline",
                                o.threshold,
                                o.alpha / family as f64,
                            )?
                            .into_iter()
                            .map(|r| Row {
                                variant: r.variant,
                                comparison: r.comparison,
                            })
                            .collect())
                        } else {
                            Ok(vec![])
                        }
                    })();
                    match comparison {
                        Err(e) => entry.issues.push(format!("Comparison unavailable: {e}")),
                        Ok(rows) => {
                            entry.outcome = if rows
                                .iter()
                                .any(|r| r.comparison.decision == Decision::Regression)
                            {
                                "regression"
                            } else if rows
                                .iter()
                                .any(|r| r.comparison.decision == Decision::Unavailable)
                                || entry.unavailable_observations > 0
                            {
                                "unavailable"
                            } else if rows
                                .iter()
                                .any(|r| r.comparison.decision == Decision::Inconclusive)
                            {
                                "inconclusive"
                            } else if rows.is_empty()
                                || rows
                                    .iter()
                                    .all(|r| r.comparison.decision == Decision::Neutral)
                            {
                                "uncompared"
                            } else {
                                "passed_comparison"
                            }
                            .into();
                            entry.comparisons = rows;
                        }
                    }
                }
                entry.run = Some(r);
            }
        }
        entries.push(entry);
    }
    if let Some(baselines) = baselines {
        for (label, path) in baselines {
            if candidates.len() > 1 && !candidates.iter().any(|(name, _)| *name == label) {
                entries.push(Entry {
                    label: format!("baseline-only/{label}"),
                    source: path.to_string_lossy().into(),
                    sha256: None,
                    baseline_source: None,
                    baseline_sha256: None,
                    status: "missing_candidate".into(),
                    outcome: "error".into(),
                    issues: vec![
                        "Baseline target has no matching candidate; it was not silently omitted."
                            .into(),
                    ],
                    comparisons: vec![],
                    unavailable_observations: 0,
                    run: None,
                    baseline_run: None,
                });
            }
        }
    }
    let rank = |outcome: &str| match outcome {
        "regression" => 0,
        "error" => 1,
        "unavailable" => 2,
        "inconclusive" => 3,
        "uncompared" => 4,
        "diagnostic" => 5,
        _ => 6,
    };
    entries.sort_by(|a, b| (rank(&a.outcome), &a.label).cmp(&(rank(&b.outcome), &b.label)));
    let mut counts = BTreeMap::new();
    for e in &entries {
        *counts.entry(e.outcome.clone()).or_default() += 1;
    }
    Ok(Document {
        schema: 1,
        title: o.title.into(),
        threshold_percent: o.threshold,
        alpha_family: o.alpha,
        family_run_count: family,
        counts,
        entries,
    })
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn outcome_label(value: &str) -> &str {
    match value {
        "regression" => "Regression",
        "error" => "Report or run error",
        "unavailable" => "Metric unavailable",
        "inconclusive" => "Inconclusive",
        "uncompared" => "No comparison",
        "diagnostic" => "Profiler run",
        "passed_comparison" => "Comparison passed",
        _ => value,
    }
}
impl Document {
    fn summary(&self) -> String {
        let mut out = format!(
            "# {}\n\n{} report entries. Practical margin: {}%. Family confidence: {:.1}%.\n\n",
            report::escape(&self.title),
            self.entries.len(),
            self.threshold_percent,
            (1. - self.alpha_family) * 100.
        );
        out.push_str("Independent experiments are never pooled. Family correction covers all candidate runs, compared variants and metrics. Uncompared/diagnostic/incomplete/unsupported does not mean passed.\n\n| Run | Status | Outcome | Cases | Observations | Missing observations |\n|---|---|---|---:|---:|---:|\n");
        for e in &self.entries {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                report::escape(&e.label),
                report::escape(&e.status),
                outcome_label(&e.outcome),
                e.run.as_ref().map_or(0, |r| r.cases.len()),
                e.run.as_ref().map_or(0, |r| r.observations.len()),
                e.unavailable_observations
            ));
        }
        out
    }
    pub fn markdown(&self) -> Result<String> {
        let mut out = self.summary();
        for e in &self.entries {
            out.push_str(&format!(
                "\n# {} — {}\n\nSource: {}\nSHA256: {}\n",
                report::escape(&e.label),
                outcome_label(&e.outcome),
                report::escape(&e.source),
                e.sha256.as_deref().unwrap_or("unavailable")
            ));
            if let Some(base) = &e.baseline_source {
                out.push_str(&format!(
                    "\nBaseline: {}\nBaseline SHA256: {}\n",
                    report::escape(base),
                    e.baseline_sha256.as_deref().unwrap_or("unavailable")
                ));
            }
            for issue in &e.issues {
                out.push_str(&format!("\n{}\n", report::escape(issue)));
            }
            for row in &e.comparisons {
                out.push_str(&format!(
                    "\nVariant: {}\n{}",
                    report::escape(&row.variant),
                    report::comparison(std::slice::from_ref(&row.comparison))
                ));
            }
            if let Some(r) = &e.run {
                out.push_str(&report::markdown(r)?);
            }
        }
        Ok(out)
    }
    /// Numerical report without bootstrap chart analysis or SVG generation.
    pub fn html_without_plots(&self) -> Result<String> {
        Ok(report::html(&self.markdown()?)
            .replace("<!--CHARTS-->", "")
            .replace("<!--DETAILS-->", ""))
    }
    pub fn html(&self) -> Result<String> {
        self.html_with_summary(None)
    }
    pub fn html_with_summary(
        &self,
        summary_plot: Option<&report::SummaryPlot<'_>>,
    ) -> Result<String> {
        self.html_with_scale(summary_plot, summary_plot.map(|s| s.scale))
    }
    pub fn html_with_scale(
        &self,
        summary_plot: Option<&report::SummaryPlot<'_>>,
        scale: Option<airbug_bench::viz::charts::AxisScale>,
    ) -> Result<String> {
        let mut content =
            String::from("<section class=\"cards\" aria-label=\"Experiment summary\">");
        for state in [
            "regression",
            "error",
            "unavailable",
            "inconclusive",
            "uncompared",
            "diagnostic",
            "passed_comparison",
        ] {
            content.push_str(&format!(
                "<div class=\"card {state}\"><strong>{}</strong><span>{}</span></div>",
                self.counts.get(state).unwrap_or(&0),
                outcome_label(state)
            ));
        }
        content.push_str("</section>");
        let summary = report::html_fragment(&self.summary());
        let mut rest = summary.as_str();
        let mut decorated = String::new();
        if let Some((head, body)) = rest.split_once("<tbody>") {
            decorated.push_str(head);
            decorated.push_str("<tbody>");
            rest = body;
            for e in &self.entries {
                if let Some((before, after)) = rest.split_once("<tr>") {
                    decorated.push_str(before);
                    decorated.push_str(&format!("<tr data-outcome=\"{}\">", e.outcome));
                    rest = after;
                }
            }
        }
        decorated.push_str(rest);
        content.push_str(&decorated);
        content.push_str("<nav aria-label=\"Run navigation\">");
        for (i, e) in self.entries.iter().enumerate() {
            content.push_str(&format!("<a href=\"#run-{i}\">{}</a>", esc(&e.label)));
        }
        content.push_str("</nav>");
        let mut chart_budget = 64;
        let per_run = (64
            / self
                .entries
                .iter()
                .filter(|e| e.run.is_some())
                .count()
                .max(1))
        .max(1);
        for (i, e) in self.entries.iter().enumerate() {
            content.push_str(&format!("<article id=\"run-{i}\" class=\"run-card\" data-outcome=\"{}\"><h2>{} <small>{}</small></h2><p>Source: {}<br>SHA256: {}</p>",e.outcome,esc(&e.label),outcome_label(&e.outcome),esc(&e.source),e.sha256.as_deref().unwrap_or("unavailable")));
            if let Some(base) = &e.baseline_source {
                content.push_str(&format!(
                    "<p>Baseline: {}<br>Baseline SHA256: {}</p>",
                    esc(base),
                    e.baseline_sha256.as_deref().unwrap_or("unavailable")
                ));
            }
            for issue in &e.issues {
                content.push_str(&format!("<p class=\"issue\">{}</p>", esc(issue)));
            }
            let config = airbug_bench::bootstrap::Config {
                confidence_level: (1. - self.alpha_family).clamp(f64::EPSILON, 1. - f64::EPSILON),
                ..Default::default()
            };
            let candidate_analysis = e
                .run
                .as_ref()
                .map(|run| airbug_bench::bootstrap::analyze(run, &config));
            if let Some(analysis) = &candidate_analysis {
                match analysis {
                    Ok(report) => {
                        let scales = e
                            .run
                            .as_ref()
                            .map(airbug_bench::presentation::load_scales)
                            .transpose();
                        match scales {
                            Ok(scales) => content.push_str(
                                &airbug_bench::bootstrap::violin_charts_with_case_scales(
                                    report,
                                    scale,
                                    &scales.unwrap_or_default(),
                                ),
                            ),
                            Err(err) => content.push_str(&format!(
                                "<p>Violin summaries unavailable: {}</p>",
                                esc(&err.to_string())
                            )),
                        }
                    }
                    Err(err) => content.push_str(&format!(
                        "<p>Violin summaries unavailable: {}</p>",
                        esc(&err.to_string())
                    )),
                }
            }
            if !e.comparisons.is_empty() {
                if let (Some(baseline), Some(candidate)) = (&e.baseline_run, &e.run) {
                    content.push_str(&report::iteration_comparison(baseline, candidate)?);
                    let retained = analysis::compare_with_distribution(
                        baseline,
                        Some(candidate),
                        self.threshold_percent,
                        self.alpha_family / self.family_run_count.max(1) as f64,
                        airbug_bench::hypothesis::Config::default(),
                    )?;
                    content.push_str(&airbug_bench::hypothesis_plot::fragment(&retained)?);
                    content.push_str(&airbug_bench::relative::charts(
                        &airbug_bench::relative::compare_runs(baseline, candidate, &config)?,
                        self.threshold_percent,
                    )?);
                    let baseline = airbug_bench::bootstrap::analyze(baseline, &config)?;
                    if let Some(Ok(candidate)) = &candidate_analysis {
                        content.push_str(&airbug_bench::bootstrap::comparison_charts(
                            &baseline, candidate,
                        ));
                    }
                }
                if let Some(r) = &e.run {
                    let comparisons: Vec<_> = e
                        .comparisons
                        .iter()
                        .map(|row| row.comparison.clone())
                        .collect();
                    content.push_str(&report::comparison_charts(
                        r,
                        &comparisons,
                        self.threshold_percent,
                        8,
                    ));
                }
            }
            for row in &e.comparisons {
                content.push_str(&format!(
                    "<h3>{}</h3>{}",
                    esc(&row.variant),
                    report::html_fragment(&report::comparison(std::slice::from_ref(
                        &row.comparison
                    )))
                ));
            }
            if let Some(r) = &e.run {
                match airbug_bench::measurement::charts(r) {
                    Ok(charts) => content.push_str(&charts),
                    Err(err) => content.push_str(&format!(
                        "<p>Formatted plots unavailable: {}</p>",
                        esc(&err.to_string())
                    )),
                }
                if let Some(summary) = summary_plot {
                    content.push_str(&report::parameter_charts(r, summary)?);
                }
                content.push_str("<details><summary>Measurements, scopes and notes</summary>");
                content.push_str(&report::html_fragment(&report::markdown(r)?));
                content.push_str("</details>");
                content
                    .push_str("<details><summary>Environment and workload contracts</summary><dl>");
                for (k, v) in &r.environment {
                    content.push_str(&format!("<dt>{}</dt><dd>{}</dd>", esc(k), esc(v)));
                }
                content.push_str("</dl>");
                for c in &r.cases {
                    content.push_str(&format!("<h3>{}</h3><dl>", esc(&c.id)));
                    for (k, v) in &c.contract {
                        content.push_str(&format!("<dt>{}</dt><dd>{}</dd>", esc(k), esc(v)));
                    }
                    content.push_str("</dl>");
                }
                content.push_str("</details>");
                let mut groups: Vec<_> = diagnostics::process_values(r)?.into_iter().collect();
                groups.sort_by_key(|((case, metric, variant), _)| {
                    (
                        match metric.as_str() {
                            "frame.completed" | "wall" | "gpu.duration" | "first.completed" => 0,
                            "cpu.submit" | "frame.interval" => 1,
                            _ => 2,
                        },
                        case.clone(),
                        metric.clone(),
                        variant.clone(),
                    )
                });
                let matrix: Vec<_> = groups
                    .iter()
                    .map(|((case, metric, variant), values)| {
                        let unit = r
                            .cases
                            .iter()
                            .find(|c| c.id == *case)
                            .and_then(|c| c.metrics.iter().find(|m| m.id == *metric))
                            .map(|m| format!(" ({})", m.unit))
                            .unwrap_or_default();
                        (
                            format!("{case} / {metric}{unit} / {variant}"),
                            values.iter().map(|(p, v)| (*p, *v)).collect(),
                        )
                    })
                    .collect();
                content.push_str(&report::process_heat(&matrix));
                for ((case, metric, variant), values) in groups.into_iter().take(per_run) {
                    if chart_budget == 0 {
                        break;
                    }
                    chart_budget -= 1;
                    let descriptor = r
                        .cases
                        .iter()
                        .find(|c| c.id == case)
                        .unwrap()
                        .metrics
                        .iter()
                        .find(|m| m.id == metric)
                        .unwrap();
                    let label = format!(
                        "{case} / {metric} / {variant} — process medians ({})",
                        descriptor.unit
                    );
                    let points: Vec<_> = values.into_iter().map(|(p, v)| (p as f64, v)).collect();
                    content.push_str(&format!(
                        "<details><summary>{}</summary>{}</details>",
                        esc(&label),
                        report::plot(&label, &points)
                    ));
                }
            }
            content.push_str("</article>");
        }
        content.push_str("<p>Charts show at most 64 series of process medians. Unavailable observations are excluded from plotted numeric summaries, counted above, and retained in measurement tables. The JSON report preserves all loaded observations.</p>");
        let page = report::html("")
            .replace("<!--CHARTS-->", "")
            .replace("<!--DETAILS-->", "");
        // Only trusted generated markup is inserted; all source values are escaped above.
        Ok(page.replace("<footer>", &format!("{content}<footer>")))
    }
}
