//! Offline report assembly: discovery is separate from statistical comparisons; no pooling.
use crate::{artifacts, project};
use rbench::{
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
    let source = project::resolve(o.store, o.source)?;
    let candidates = files(&source)?;
    let baseline = o
        .baseline
        .map(|p| project::resolve(o.store, p))
        .transpose()?;
    let baselines = baseline.as_deref().map(files).transpose()?;
    if baselines
        .as_ref()
        .is_some_and(|b| b.len() == 1 && candidates.len() > 1)
    {
        return Err(error("a single baseline cannot be broadcast across a collection; supply a matching baseline collection"));
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
                            Ok(analysis::compare(
                                &b,
                                Some(&r),
                                o.threshold,
                                o.alpha / family as f64,
                            )?
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
    pub fn html(&self) -> Result<String> {
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
            if !e.comparisons.is_empty() {
                content.push_str(&effects(&e.comparisons, self.threshold_percent));
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

fn effects(rows: &[Row], threshold: f64) -> String {
    let valid: Vec<_> = rows
        .iter()
        .filter(|r| {
            r.comparison
                .interval_percent
                .is_some_and(|(l, h)| l.is_finite() && h.is_finite())
                && r.comparison.change_percent.is_some_and(f64::is_finite)
        })
        .take(32)
        .collect();
    if valid.is_empty() {
        return "<p>No finite effect intervals available. Missing intervals are not zero changes.</p>".into();
    }
    let extent = valid
        .iter()
        .flat_map(|r| {
            let (l, h) = r.comparison.interval_percent.unwrap();
            [l.abs(), h.abs(), r.comparison.change_percent.unwrap().abs()]
        })
        .fold(threshold.max(1.), f64::max);
    let x = |v: f64| 400. + v / extent * 300.;
    let height = valid.len() * 66 + 75;
    let mut out=format!("<details open><summary>Effect estimates and confidence intervals</summary><svg role=\"img\" aria-label=\"Candidate change percent and confidence intervals\" viewBox=\"0 0 800 {height}\" style=\"width:100%;max-width:1000px\"><rect width=\"800\" height=\"{height}\" fill=\"#fff\"/>");
    for value in [-threshold, 0., threshold] {
        out.push_str(&format!("<line x1=\"{:.2}\" x2=\"{:.2}\" y1=\"15\" y2=\"{}\" stroke=\"#a4b4b2\" stroke-dasharray=\"4 4\"/>",x(value),x(value),height-40));
    }
    for (i, r) in valid.iter().enumerate() {
        let y = 35 + i * 66;
        let (l, h) = r.comparison.interval_percent.unwrap();
        let change = r.comparison.change_percent.unwrap();
        let color = match r.comparison.decision {
            Decision::Regression => "#b32c34",
            Decision::Improvement | Decision::WithinMargin => "#09695d",
            _ => "#685286",
        };
        let label = format!(
            "{} / {} / {}",
            r.comparison.case, r.comparison.metric, r.variant
        );
        let short: String = label.chars().take(90).collect();
        out.push_str(&format!("<g><title>{}: {:.3}% [{:.3}, {:.3}]</title><text x=\"20\" y=\"{y}\" font-size=\"12\">{}</text><line x1=\"{:.2}\" x2=\"{:.2}\" y1=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"5\"/><circle cx=\"{:.2}\" cy=\"{}\" r=\"5\" fill=\"{color}\"/></g>",esc(&label),change,l,h,esc(&short),x(l),x(h),y+18,y+18,x(change),y+18));
    }
    for (value, anchor) in [(-extent, "start"), (0., "middle"), (extent, "end")] {
        out.push_str(&format!("<text x=\"{:.2}\" y=\"{}\" text-anchor=\"{anchor}\" font-size=\"12\">{value:.2}%</text>",x(value),height-15));
    }
    out.push_str("</svg><p>Positive means a larger candidate metric, not necessarily slower. Colors follow each metric's declared direction. Dashed lines: zero and practical margin. At most 32 finite intervals shown; all decisions remain in tables.</p></details>");
    out
}
