use crate::{
    analysis::{median, Comparison},
    Result, Run,
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
    let mut out=format!("# rbench {}\n\nStatus: {:?}\n\n| Case | Metric | Median | Unit | Scope / statistic | Observations | Processes |\n|---|---|---:|---|---|---:|---:|\n",escape(&run.id),run.status);
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
    let mut ordered: Vec<_> = rows.iter().collect();
    ordered.sort_by_key(|r| match r.decision {
        crate::analysis::Decision::Regression => 0,
        crate::analysis::Decision::Unavailable => 1,
        crate::analysis::Decision::Inconclusive => 2,
        crate::analysis::Decision::Improvement => 3,
        _ => 4,
    });
    let rows = ordered;
    let mut out=String::from("# rbench comparison\n\n| Case | Metric | A | B | Change % | Interval % | Independent units | Decision |\n|---|---|---:|---:|---:|---|---:|---|\n");
    let regressions = rows
        .iter()
        .filter(|r| r.decision == crate::analysis::Decision::Regression)
        .count();
    let unresolved = rows
        .iter()
        .filter(|r| {
            matches!(
                r.decision,
                crate::analysis::Decision::Inconclusive | crate::analysis::Decision::Unavailable
            )
        })
        .count();
    out=out.replacen("# rbench comparison\n\n",&format!("# rbench comparison\n\n{} metrics · {regressions} regressions · {unresolved} unresolved. Regressions and unresolved results appear first.\n\n",rows.len()),1);
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
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
        Decision::Unavailable=>"Next: inspect missing capability/data; repeating unchanged unsupported cases will not help.",
        Decision::Inconclusive if r.baseline==Some(0.)=>"Next: use an absolute budget; a percentage of zero is undefined.",
        Decision::Inconclusive if r.interval_percent.is_none()=>"Next: predeclare a new experiment with more independent processes/pairs; more inner iterations do not increase independent units.",
        Decision::Inconclusive=>"Next: inspect raw process plots and environment drift; use a separately planned larger experiment if needed. Do not rerun until a desired verdict appears.",
        _=>""
    }
}
/// Dependency-free SVG scatterplot. Fixed numeric coordinates and escaped labels only.
pub fn plot(label: &str, points: &[(f64, f64)]) -> String {
    let points: Vec<_> = points
        .iter()
        .copied()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .collect();
    if points.is_empty() {
        return String::new();
    }
    let xmin = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let xmax = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let ymin = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let ymax = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let mut svg=format!("<figure><figcaption>{}</figcaption><svg role=\"img\" aria-label=\"{}\" viewBox=\"0 0 720 220\" style=\"width:100%;max-width:900px;background:white\"><path d=\"M64 16V180H704\" stroke=\"#a9bdb7\" fill=\"none\"/><text x=\"2\" y=\"25\" font-size=\"11\">{ymax:.3}</text><text x=\"2\" y=\"180\" font-size=\"11\">{ymin:.3}</text><text x=\"64\" y=\"205\" font-size=\"11\">{xmin:.0}</text><text x=\"665\" y=\"205\" font-size=\"11\">{xmax:.0}</text>",html_escape(label),html_escape(label));
    let stride = points.len().div_ceil(1000).max(1);
    for (x, y) in points.iter().step_by(stride) {
        let px = 64. + (x - xmin) / (xmax - xmin).max(1.) * 640.;
        let py = if ymax == ymin {
            98.
        } else {
            180. - (y - ymin) / (ymax - ymin) * 164.
        };
        svg.push_str(&format!("<circle cx=\"{px:.2}\" cy=\"{py:.2}\" r=\"2.3\" fill=\"#09695d\"><title>x={x}, y={y}</title></circle>"));
    }
    svg.push_str("</svg></figure>");
    svg
}
fn raw_charts(run: &Run) -> Result<String> {
    let mut charts=String::from("<section><h2>Raw observation plots</h2><p>x = observation sequence within each process; y = value (wall batches normalized per operation). Each plot is a separate process, with its own scale. At most 64 plots, at most 1000 displayed points each; JSON/CSV retains all values. These are diagnostics, not confidence intervals.</p>");
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
