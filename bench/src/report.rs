use crate::{
    Result, Run,
    analysis::{Comparison, median},
    viz::{self, charts},
};
use std::collections::BTreeMap;
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace(['\n', '\r'], " ")
        .replace('`', "&#96;")
}
/// Derived batch throughput for one case/variant.
///
/// `values` are per-observation useful-work rates in `unit` per second, computed
/// from positive `wall` batches and the case's declared work units. For `bytes`
/// the rate is scaled to MiB/s and `unit` is `MiB`. This is batch throughput, not
/// a distribution of individual-operation latencies.
#[derive(Clone, Debug)]
pub struct ThroughputSeries {
    pub case: String,
    pub variant: String,
    pub unit: String,
    pub values: Vec<f64>,
}
/// Derive work-unit throughput series for every case that declared work units.
///
/// Cases without `work.unit`/`work.count`, and zero/unavailable durations, are
/// omitted; the underlying observations are never modified.
pub fn throughput(run: &Run) -> Result<Vec<ThroughputSeries>> {
    let mut series = Vec::new();
    for c in &run.cases {
        if let (Some(unit), Some(count)) = (
            c.contract.get("work.unit"),
            c.contract
                .get("work.count")
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|n| *n > 0),
        ) {
            let mut groups: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
            for o in run
                .observations
                .iter()
                .filter(|o| o.case == c.id && o.metric == "wall")
            {
                if let Some(n) = o.number()?.filter(|n| *n > 0.) {
                    groups
                        .entry(&o.variant)
                        .or_default()
                        .push(count as f64 * 1e9 / (n / o.operations as f64));
                }
            }
            let is_bytes = unit == "bytes";
            let display_unit = if is_bytes { "MiB" } else { unit.as_str() };
            for (variant, mut values) in groups {
                if is_bytes {
                    for v in &mut values {
                        *v /= 1048576.0;
                    }
                }
                series.push(ThroughputSeries {
                    case: c.id.clone(),
                    variant: variant.to_string(),
                    unit: display_unit.to_string(),
                    values,
                });
            }
        }
    }
    Ok(series)
}
pub fn markdown(run: &Run) -> Result<String> {
    run.validate()?;
    let mut out = format!(
        "# bench {}\n\nStatus: {:?}\n\n| Case | Metric | Median | Unit | Scope / statistic | Observations | Processes |\n|---|---|---:|---|---|---:|---:|\n",
        escape(&run.id),
        run.status
    );
    for c in &run.cases {
        for m in &c.metrics {
            let mut groups: BTreeMap<&str, Vec<_>> = BTreeMap::new();
            for o in run
                .observations
                .iter()
                .filter(|o| o.case == c.id && o.metric == m.id)
            {
                groups.entry(&o.variant).or_default().push(o);
            }
            for (variant, rows) in groups {
                let mut values = vec![];
                let mut processes = std::collections::BTreeSet::new();
                let mut unavailable = vec![];
                for o in &rows {
                    processes.insert(o.process);
                    match o.number()? {
                        Some(n) => values.push(if m.statistic == "batch_total" {
                            n / o.operations as f64
                        } else {
                            n
                        }),
                        None => unavailable.push(format!("{:?}", o.availability)),
                    }
                }
                unavailable.sort();
                unavailable.dedup();
                let value = if unavailable.is_empty() && !values.is_empty() {
                    format!("{:.4}", median(&values))
                } else {
                    format!("n/a ({})", escape(&unavailable.join(", ")))
                };
                out.push_str(&format!(
                    "| {} [{}] | {} | {} | {} | {} / {} | {} | {} |\n",
                    escape(&c.id),
                    escape(variant),
                    escape(&m.id),
                    value,
                    escape(&m.unit),
                    escape(&m.scope),
                    escape(&m.statistic),
                    rows.len(),
                    processes.len()
                ));
            }
        }
    }
    let mut rates = String::new();
    for s in throughput(run)? {
        rates.push_str(&format!(
            "| {} [{}] | {:.4} | {}/s |\n",
            escape(&s.case),
            escape(&s.variant),
            median(&s.values),
            escape(&s.unit)
        ));
    }
    if !rates.is_empty() {
        out.push_str("\n# Useful work throughput\n\n| Case | Median batch throughput | Unit |\n|---|---:|---|\n");
        out.push_str(&rates);
        out.push_str("\nDerived from positive wall batches and declared units per operation; zero/unavailable durations omitted.\n");
    }
    out.push_str("\nMedians above summarize observations, not pooled operation-latency percentiles. Comparisons aggregate within each process first.\n");
    for n in &run.notes {
        out.push_str(&format!("\n- {}\n", escape(n)));
    }
    Ok(out)
}
pub fn comparison(rows: &[Comparison]) -> String {
    let summary = crate::analysis::summarize(rows);
    let mut ordered: Vec<_> = rows.iter().collect();
    ordered.sort_by_key(|r| match r.decision {
        crate::analysis::Decision::Regression => 0,
        crate::analysis::Decision::Unavailable => 1,
        crate::analysis::Decision::Inconclusive => 2,
        crate::analysis::Decision::Improvement => 3,
        _ => 4,
    });
    let rows = ordered;
    let mut out = String::from(
        "# bench comparison\n\n| Case | Metric | A | B | Change % | Interval % | Independent units | Decision |\n|---|---|---:|---:|---:|---|---:|---|\n",
    );
    out = out.replacen(
        "# bench comparison\n\n",
        &format!(
            "# bench comparison\n\n{} metrics · {} regressions · {} unresolved. Regressions and unresolved results appear first.\n\n",
            summary.rows, summary.regressions, summary.unresolved
        ),
        1,
    );
    let n = |v: Option<f64>| v.map(|x| format!("{x:.4}")).unwrap_or("n/a".into());
    for r in &rows {
        out.push_str(&format!(
            "| {} | {} ({}) | {} | {} | {} | {} | {} | {:?} |\n",
            escape(&r.case),
            escape(&r.metric),
            escape(&r.unit),
            n(r.baseline),
            n(r.candidate),
            n(r.change_percent),
            r.interval_percent
                .map(|(l, h)| format!("[{l:.3}, {h:.3}]"))
                .unwrap_or("insufficient data".into()),
            r.independent_units,
            r.decision
        ));
    }
    out.push_str("\nWithinMargin concerns the declared median metric, not tail latency or all possible workloads.\n");
    for r in &rows {
        out.push_str(&format!(
            "\n- {} / {}: {} Scope: {}.\n",
            escape(&r.case),
            escape(&r.metric),
            escape(&format!("{} {}", r.note, advice(r))),
            escape(&r.scope)
        ));
    }
    out
}
fn html_escape(s: &str) -> String {
    viz::esc(s)
}
/// Render the report's small Markdown subset, never arbitrary HTML.
pub fn html_fragment(markdown: &str) -> String {
    let mut body = String::new();
    let mut table = false;
    let mut header = false;
    for line in markdown.lines() {
        if line.starts_with('|') {
            if line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) {
                continue;
            }
            if !table {
                body.push_str("<div class=\"table-wrap\"><table><thead>");
                table = true;
                header = true;
            }
            body.push_str("<tr>");
            for cell in line.trim_matches('|').split('|') {
                let cell = cell
                    .trim()
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&#124;", "|")
                    .replace("&#96;", "`")
                    .replace("&amp;", "&");
                let text = html_escape(&cell);
                if header {
                    body.push_str(&format!("<th><button type=\"button\">{text}</button></th>"));
                } else {
                    body.push_str(&format!("<td>{text}</td>"));
                }
            }
            body.push_str("</tr>");
            if header {
                body.push_str("</thead><tbody>");
                header = false;
            }
        } else {
            if table {
                body.push_str("</tbody></table></div>");
                table = false;
            }
            if let Some(h) = line.strip_prefix("# ") {
                body.push_str(&format!("<h1>{}</h1>", html_escape(h)));
            } else if !line.trim().is_empty() {
                body.push_str(&format!("<p>{}</p>", html_escape(line)));
            }
        }
    }
    if table {
        body.push_str("</tbody></table></div>");
    }
    body
}
/// Self-contained page for the report Markdown subset.
pub fn html(markdown: &str) -> String {
    include_str!("report-template.html").replace("<!--CONTENT-->", &html_fragment(markdown))
}
pub fn html_run(run: &Run) -> Result<String> {
    let mut text = markdown(run)?;
    text.push_str("\n# Measurement context\n");
    for (k, v) in &run.environment {
        text.push_str(&format!("{k}: {v}\n"));
    }
    let mut result = html(&text);
    let mut details = String::from("<section><h2>Case contracts</h2>");
    for c in &run.cases {
        details.push_str(&format!(
            "<details><summary>{}</summary><dl>",
            html_escape(&c.id)
        ));
        for (k, v) in &c.contract {
            details.push_str(&format!(
                "<dt>{}</dt><dd>{}</dd>",
                html_escape(k),
                html_escape(v)
            ));
        }
        details.push_str("</dl></details>");
    }
    details.push_str("</section>");
    result = result.replace("<!--DETAILS-->", &details);
    result = result.replace("<!--CHARTS-->", &raw_charts(run)?);
    Ok(result)
}

pub fn advice(r: &Comparison) -> &'static str {
    use crate::analysis::Decision;
    match r.decision {
        Decision::Unavailable => {
            "Next: inspect missing capability/data; repeating unchanged unsupported cases will not help."
        }
        Decision::Inconclusive if r.baseline == Some(0.) => {
            "Next: use an absolute budget; a percentage of zero is undefined."
        }
        Decision::Inconclusive if r.interval_percent.is_none() => {
            "Next: predeclare a new experiment with more independent processes/pairs; more inner iterations do not increase independent units."
        }
        Decision::Inconclusive => {
            "Next: inspect raw process plots and environment drift; use a separately planned larger experiment if needed. Do not rerun until a desired verdict appears."
        }
        _ => "",
    }
}
/// Dependency-free SVG scatterplot of one series. Empty or all-non-finite input plots nothing.
pub fn plot(label: &str, points: &[(f64, f64)]) -> String {
    let clean: Vec<_> = points
        .iter()
        .copied()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .collect();
    if clean.is_empty() {
        return String::new();
    }
    charts::dot_plot(label, &clean, 1000, "")
}
/// Effect and distribution charts that explain a set of comparisons.
///
/// The forest plot shows every finite interval estimate; per-unit strip charts appear only
/// for crossover runs where both variants share a `run.json`, because that is the case where
/// independent units can be paired without assumptions.
pub fn comparison_charts(
    run: &Run,
    rows: &[Comparison],
    threshold: f64,
    max_unit_charts: usize,
) -> String {
    let forest_rows: Vec<_> = rows
        .iter()
        .filter(|r| {
            r.change_percent.is_some_and(f64::is_finite)
                && r.interval_percent
                    .is_some_and(|(l, h)| l.is_finite() && h.is_finite())
        })
        .take(32)
        .map(|r| charts::Interval {
            label: format!("{} / {} ({})", r.case, r.metric, r.unit),
            point: r.change_percent.unwrap(),
            low: r.interval_percent.unwrap().0,
            high: r.interval_percent.unwrap().1,
            decision: r.decision.clone(),
        })
        .collect();
    let mut out = String::new();
    if forest_rows.is_empty() {
        out.push_str(
            "<p>No finite effect intervals available. Missing intervals are not zero changes.</p>",
        );
    } else {
        out.push_str(&charts::forest(
            "Effect estimates and confidence intervals",
            &forest_rows,
            threshold,
            "Positive means a larger candidate metric, not necessarily slower: colors follow each metric's declared direction. Shaded band and dashed lines: the declared practical margin. At most 32 finite intervals shown; every decision stays in the tables above.",
        ));
        let bars_rows: Vec<_> = forest_rows
            .iter()
            .map(|r| (r.label.clone(), r.point, r.decision.clone()))
            .collect();
        out.push_str(&charts::bars(
            "Point estimates by metric",
            &bars_rows,
            "%",
            "Bars are the point estimate only; read the interval above before deciding.",
        ));
    }
    let crossover = {
        let mut variants = std::collections::BTreeSet::new();
        for o in &run.observations {
            variants.insert(o.variant.as_str());
        }
        variants == std::collections::BTreeSet::from(["baseline", "candidate"])
    };
    if crossover {
        let mut drawn = 0;
        for r in rows.iter().filter(|r| r.independent_units >= 2) {
            if drawn >= max_unit_charts {
                out.push_str(&format!(
                    "<p>{} more per-metric unit charts are available in the JSON report.</p>",
                    rows.iter().filter(|r| r.independent_units >= 2).count() - drawn
                ));
                break;
            }
            let Ok(units) = crate::analysis::pairs(run, None, &r.case, &r.metric) else {
                continue;
            };
            let lanes = [
                charts::Lane {
                    label: "baseline".into(),
                    values: units.iter().map(|u| u.baseline).collect(),
                    median: Some(median(
                        &units.iter().map(|u| u.baseline).collect::<Vec<_>>(),
                    )),
                    color: Some(viz::Palette::LIGHT.series(0)),
                },
                charts::Lane {
                    label: "candidate".into(),
                    values: units.iter().map(|u| u.candidate).collect(),
                    median: Some(median(
                        &units.iter().map(|u| u.candidate).collect::<Vec<_>>(),
                    )),
                    color: Some(viz::Palette::LIGHT.series(1)),
                },
            ];
            let deltas: Vec<_> = units
                .iter()
                .filter_map(|u| u.change_percent.map(|c| (u.unit as f64, c)))
                .collect();
            out.push_str(&charts::strip(
                &format!("{} / {} — independent units", r.case, r.metric),
                &lanes,
                &r.unit,
                "One dot per independent unit (a crossover pair), lanes are variants. This shows spread across units; it is not a confidence interval.",
            ));
            if !deltas.is_empty() {
                out.push_str(&charts::dot_plot(
                    &format!("{} / {} — change per unit", r.case, r.metric),
                    &deltas,
                    200,
                    "x = pair id, y = candidate relative to baseline, in percent. One point per independent unit; this is a view of spread, not an interval estimate.",
                ));
            }
            drawn += 1;
        }
    }
    if out.is_empty() {
        return String::new();
    }
    format!("<section class=\"charts\"><h3>Effect charts</h3>{out}</section>")
}
fn raw_charts(run: &Run) -> Result<String> {
    let mut charts = String::from(
        "<section><h2>Raw observation plots</h2><p>x = observation sequence within each process; y = value (wall batches normalized per operation). Each plot is a separate process, with its own scale. At most 64 plots, at most 1000 displayed points each; JSON/CSV retains all values. These are diagnostics, not confidence intervals.</p>",
    );
    type Series = BTreeMap<(String, String, String, u32), Vec<(f64, f64)>>;
    let mut groups: Series = BTreeMap::new();
    for o in &run.observations {
        if let Some(mut v) = o.number()? {
            let m = run
                .cases
                .iter()
                .find(|c| c.id == o.case)
                .unwrap()
                .metrics
                .iter()
                .find(|m| m.id == o.metric)
                .unwrap();
            if m.statistic == "batch_total" {
                v /= o.operations as f64;
            }
            groups
                .entry((
                    o.case.clone(),
                    format!("{} ({})", o.metric, m.unit),
                    o.variant.clone(),
                    o.process,
                ))
                .or_default()
                .push((o.sequence as f64, v));
        }
    }
    for ((case, metric, variant, process), mut values) in groups.into_iter().take(64) {
        values.sort_by(|a, b| a.0.total_cmp(&b.0));
        charts.push_str(&format!(
            "<details><summary>{}</summary>{}</details>",
            html_escape(&format!(
                "{case} / {metric} / {variant} / process {process}"
            )),
            plot(
                &format!("{case} / {metric} / {variant} / process {process}"),
                &values
            )
        ));
    }
    charts.push_str("</section>");
    Ok(charts)
}
