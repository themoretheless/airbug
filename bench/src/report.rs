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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThroughputSeries {
    pub case: String,
    pub variant: String,
    pub unit: String,
    pub values: Vec<f64>,
}
/// Derive work-unit throughput series for every case that declared work units.
///
/// Reads multiple `work.counter.<unit>` entries and legacy `work.unit`/`work.count`.
/// Cases without counters, and zero/unavailable durations, are omitted;
/// the underlying observations are never modified.
pub fn throughput(run: &Run) -> Result<Vec<ThroughputSeries>> {
    throughput_impl(run, None, None)
}
pub(crate) fn work_counters(c: &crate::Case) -> Result<BTreeMap<&str, Option<u64>>> {
    let mut counters: BTreeMap<&str, Option<u64>> = BTreeMap::new();
    for (key, value) in &c.contract {
        if let Some(unit) = key.strip_prefix("work.counter.") {
            if unit.is_empty() {
                return Err(crate::error("work counter unit must not be empty"));
            }
            counters.insert(unit, Some(value.parse::<u64>()?));
        }
    }
    if let Some(count) = c.contract.get("work.count") {
        let unit = c
            .contract
            .get("work.unit")
            .filter(|u| !u.is_empty())
            .ok_or_else(|| crate::error("work counter unit must not be empty"))?;
        counters.entry(unit).or_insert(Some(count.parse::<u64>()?));
    }
    for (key, value) in &c.contract {
        if let Some(unit) = key.strip_prefix("work.input.") {
            if unit.is_empty() || value != "batch_total" {
                return Err(crate::error("invalid input counter declaration"));
            }
            counters.insert(unit, None);
        }
    }
    Ok(counters)
}
fn throughput_impl(
    run: &Run,
    format: Option<BytesFormat>,
    process: Option<u32>,
) -> Result<Vec<ThroughputSeries>> {
    let mut series = Vec::new();
    for c in &run.cases {
        let counters = work_counters(c)?;
        for (unit, count) in counters {
            let mut groups: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
            for o in run.observations.iter().filter(|o| {
                o.case == c.id && o.metric == "wall" && process.is_none_or(|p| o.process == p)
            }) {
                o.validate_work_totals(c)?;
                if let Some(n) = o.number()?.filter(|n| *n > 0.) {
                    let total = match count {
                        Some(count) => count as f64 * o.operations as f64,
                        None => o.work_totals[unit].parse::<u128>()? as f64,
                    };
                    let rate = if total == 0.0 { 0.0 } else { total * 1e9 / n };
                    if !rate.is_finite() {
                        return Err(crate::error("throughput exceeds finite numeric range"));
                    }
                    groups.entry(&o.variant).or_default().push(rate);
                }
            }
            let is_bytes = unit == "bytes";
            for (variant, mut values) in groups {
                let (scale, display_unit) = if is_bytes {
                    match format {
                        Some(format) => byte_scale(&values, format),
                        None => (1_048_576.0, "MiB"),
                    }
                } else {
                    (1.0, unit)
                };
                for value in &mut values {
                    *value /= scale;
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
/// One process contributes one median batch throughput in a fixed display unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProcessThroughput {
    pub case: String,
    pub variant: String,
    pub unit: String,
    pub process: u32,
    pub median: f64,
}
pub fn throughput_process_medians(run: &Run) -> Result<Vec<ProcessThroughput>> {
    run.validate()?;
    let processes: std::collections::BTreeSet<_> =
        run.observations.iter().map(|o| o.process).collect();
    let mut result = Vec::new();
    for process in processes {
        for series in throughput_impl(run, None, Some(process))? {
            result.push(ProcessThroughput {
                case: series.case,
                variant: series.variant,
                unit: series.unit,
                process,
                median: median(&series.values),
            });
        }
    }
    Ok(result)
}

/// Display-only scaling; raw observations and work counters remain unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytesFormat {
    Decimal,
    Binary,
}
impl BytesFormat {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "decimal" => Ok(Self::Decimal),
            "binary" => Ok(Self::Binary),
            _ => Err(crate::error("bytes format must be decimal or binary")),
        }
    }
}
/// Choose one prefix per series using its median rate, retaining every observation.
pub fn throughput_with_format(run: &Run, format: BytesFormat) -> Result<Vec<ThroughputSeries>> {
    throughput_impl(run, Some(format), None)
}
fn byte_scale(values: &[f64], format: BytesFormat) -> (f64, &'static str) {
    let (base, units) = match format {
        BytesFormat::Decimal => (1000.0, ["B", "KB", "MB", "GB", "TB", "PB", "EB"]),
        BytesFormat::Binary => (1024.0, ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"]),
    };
    let typical = if values.is_empty() {
        0.0
    } else {
        median(values)
    };
    let mut scale = 1.0;
    let mut prefix = 0;
    while prefix + 1 < units.len() && typical >= scale * base {
        scale *= base;
        prefix += 1;
    }
    (scale, units[prefix])
}
pub fn markdown(run: &Run) -> Result<String> {
    markdown_impl(run, None)
}
pub fn markdown_with_bytes_format(run: &Run, format: BytesFormat) -> Result<String> {
    markdown_impl(run, Some(format))
}
fn markdown_impl(run: &Run, format: Option<BytesFormat>) -> Result<String> {
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
    for s in match format {
        Some(format) => throughput_with_format(run, format)?,
        None => throughput(run)?,
    } {
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
    match crate::measurement::load_formatted(run) {
        Ok(formatted) => out.push_str(&crate::measurement::report_markdown(&formatted)),
        Err(err) => out.push_str(&format!(
            "\nFormatted metrics unavailable: {}\n",
            escape(&err.to_string())
        )),
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
///
/// Carries [`viz::REVEAL_CSS`] so every figure built by [`viz::Plot::figure`] draws itself in;
/// pages that poll and repaint (the live UI) must not include it.
pub fn html(markdown: &str) -> String {
    include_str!("report-template.html")
        .replace("<!--CONTENT-->", &html_fragment(markdown))
        .replace("/*VIZ_CSS*/", viz::REVEAL_CSS)
}
pub fn html_run(run: &Run) -> Result<String> {
    html_run_with_summary(run, None)
}

/// Numeric input parameter for summary charts. Values come from `param.<name>`
/// contracts, never from parsing display names. Both axes use the chosen scale.
pub struct SummaryPlot<'a> {
    pub parameter: &'a str,
    pub estimator: crate::summary::Estimator,
    pub scale: charts::AxisScale,
}

/// Standard run report with optional input-size summaries.
pub fn html_run_with_summary(run: &Run, summary: Option<SummaryPlot<'_>>) -> Result<String> {
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
    let mut plots = raw_charts(run)?;
    if let Some(summary) = summary {
        plots.push_str(&parameter_charts(run, &summary)?);
    }
    result = result.replace("<!--CHARTS-->", &plots);
    Ok(result)
}

/// Plot medians of independent process medians against an explicit numeric input.
/// Metric contracts are kept separate, including unit, scope, phase and statistic.
pub fn parameter_charts(run: &Run, config: &SummaryPlot<'_>) -> Result<String> {
    run.validate()?;
    let means: BTreeMap<_, _> = if config.estimator == crate::summary::Estimator::Mean {
        crate::summary::mean_estimates(run)?
            .into_iter()
            .filter_map(|r| r.value.map(|v| ((r.case, r.metric, r.variant), v)))
            .collect()
    } else {
        BTreeMap::new()
    };
    let mut values = BTreeMap::new();
    let families = crate::presentation::load_families(run)?;
    let key = format!("param.{}", config.parameter);
    let mut groups: BTreeMap<String, (String, Vec<charts::Series>)> = BTreeMap::new();
    let mut skipped = 0;
    let mut incomplete = std::collections::BTreeSet::new();
    let mut unavailable = String::new();
    let variants: std::collections::BTreeSet<_> = run
        .observations
        .iter()
        .map(|o| o.variant.as_str())
        .collect();
    for case in &run.cases {
        for metric in &case.metrics {
            for variant in &variants {
                match crate::analysis::values(run, &case.id, metric, variant) {
                    Ok(processes) => {
                        values.insert(
                            (case.id.clone(), metric.id.clone(), (*variant).to_string()),
                            processes
                                .into_iter()
                                .map(|(p, (_, v))| (p, v))
                                .collect::<BTreeMap<_, _>>(),
                        );
                    }
                    Err(err) => {
                        if let Some(family) = families.get(&case.id) {
                            incomplete.insert((
                                serde_json::to_string(metric)?,
                                format!("family {family} / {variant}"),
                            ));
                        }
                        unavailable.push_str(&format!(
                            "<p>{} / {} / {}: summary unavailable: {}</p>",
                            escape(&case.id),
                            escape(&metric.id),
                            escape(variant),
                            escape(&err.to_string())
                        ));
                    }
                }
            }
        }
    }

    for ((case_id, metric_id, variant), processes) in values {
        let case = run.cases.iter().find(|case| case.id == case_id).unwrap();
        let Some(x) = case
            .contract
            .get(&key)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
        else {
            skipped += 1;
            if let Some(family) = families.get(&case_id) {
                let metric = case.metrics.iter().find(|m| m.id == metric_id).unwrap();
                incomplete.insert((
                    serde_json::to_string(metric)?,
                    format!("family {family} / {variant}"),
                ));
            }
            continue;
        };
        if processes.is_empty() {
            continue;
        }
        let metric = case
            .metrics
            .iter()
            .find(|metric| metric.id == metric_id)
            .unwrap();
        let ys: Vec<_> = processes.values().copied().collect();
        let estimate = if config.estimator == crate::summary::Estimator::Mean {
            means[&(case_id.clone(), metric_id.clone(), variant.clone())]
        } else {
            median(&ys)
        };
        let label = format!(
            "{} vs {} ({}; {}; {}; {})",
            config.parameter, metric.id, metric.unit, metric.scope, metric.phase, metric.statistic
        );
        let group = groups
            .entry(serde_json::to_string(metric)?)
            .or_insert_with(|| (label, vec![]));
        let label = families
            .get(&case_id)
            .map(|family| format!("family {family} / {variant}"))
            .unwrap_or_else(|| format!("case {case_id} / {variant}"));
        if let Some(series) = group.1.iter_mut().find(|s| s.label == label) {
            series.points.push((x, estimate));
        } else {
            group
                .1
                .push(charts::Series::new(label, vec![(x, estimate)]));
        }
    }
    let estimator_label = match config.estimator {
        crate::summary::Estimator::ProcessMedian => "median of independent process medians",
        crate::summary::Estimator::Mean => {
            "arithmetic mean of normalized observations; each batch has equal weight"
        }
    };
    let mut output = format!(
        "<section><h2>Input parameter summaries</h2><p>X: {}. Y: {estimator_label}. Explicit families connect cases within each variant; other cases remain separate. Missing or nonnumeric parameter: {skipped} series omitted.</p>",
        escape(config.parameter)
    );
    output.push_str(&unavailable);
    if !incomplete.is_empty() {
        output.push_str("<p>Incomplete families are shown as separate points; no line crosses an unavailable case.</p>");
    }
    for (contract, (label, series)) in groups {
        let series: Vec<_> = series
            .into_iter()
            .flat_map(|s| {
                if incomplete.contains(&(contract.clone(), s.label.clone())) {
                    s.points
                        .into_iter()
                        .map(|p| charts::Series::new(&s.label, vec![p]))
                        .collect()
                } else {
                    vec![s]
                }
            })
            .collect();
        if series.iter().any(|s| s.points.len() > 1) {
            match charts::line_scaled(
                &label,
                &series,
                "Metric contracts are kept separate. Batch totals are normalized per operation.",
                config.scale,
                config.scale,
            ) {
                Ok(chart) => output.push_str(&chart),
                Err(err) => output.push_str(&format!(
                    "<p>{}: line summary unavailable: {}</p>",
                    escape(&label),
                    escape(&err.to_string())
                )),
            }
            continue;
        }
        output.push_str(&charts::scatter_scaled(
            &label,
            &series,
            1000,
            "Metric contracts are not pooled. Batch totals are normalized per operation.",
            config.scale,
            config.scale,
        ));
    }
    output.push_str("</section>");
    output.push_str(&throughput_parameter_charts(run, config)?);
    Ok(output)
}

/// Throughput versus numeric input, using equal weights for independent processes.
pub fn throughput_parameter_charts(run: &Run, config: &SummaryPlot<'_>) -> Result<String> {
    let families = crate::presentation::load_families(run)?;
    let key = format!("param.{}", config.parameter);
    let mut values: BTreeMap<(String, String, String), Vec<f64>> = BTreeMap::new();
    if config.estimator == crate::summary::Estimator::Mean {
        for row in crate::summary::mean_throughput(run)? {
            if let Some(value) = row.per_second {
                values
                    .entry((row.case, row.variant, row.unit))
                    .or_default()
                    .push(value);
            }
        }
    } else {
        for row in throughput_process_medians(run)? {
            values
                .entry((row.case, row.variant, row.unit))
                .or_default()
                .push(row.median);
        }
    }
    let mut groups: BTreeMap<(String, String), Vec<charts::Series>> = BTreeMap::new();
    let mut members: BTreeMap<(String, String), Vec<(String, String)>> = BTreeMap::new();
    let variants: std::collections::BTreeSet<_> =
        run.observations.iter().map(|o| o.variant.clone()).collect();
    for case in &run.cases {
        if let Some(metric) = case.metrics.iter().find(|m| m.id == "wall") {
            if let Some(family) = families.get(&case.id) {
                for variant in &variants {
                    members
                        .entry((
                            serde_json::to_string(metric)?,
                            format!("family {family} / {variant}"),
                        ))
                        .or_default()
                        .push((case.id.clone(), variant.clone()));
                }
            }
        }
    }
    let mut available = std::collections::BTreeSet::new();
    let mut skipped = 0;
    for ((case_id, variant, unit), medians) in values {
        let case = run.cases.iter().find(|c| c.id == case_id).unwrap();
        let Some(x) = case
            .contract
            .get(&key)
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| v.is_finite())
        else {
            skipped += 1;
            continue;
        };
        let Some(metric) = case
            .metrics
            .iter()
            .find(|m| m.id == "wall" && m.unit == "ns" && m.statistic == "batch_total")
        else {
            skipped += 1;
            continue;
        };
        // A partial population or zero duration cannot define a complete rate summary.
        if config.estimator == crate::summary::Estimator::ProcessMedian
            && run
                .observations
                .iter()
                .filter(|o| o.case == case_id && o.variant == variant && o.metric == "wall")
                .any(|o| !matches!(o.number(), Ok(Some(n)) if n > 0.))
        {
            skipped += 1;
            continue;
        }
        available.insert((case_id.clone(), variant.clone(), unit.clone()));
        let label = families
            .get(&case_id)
            .map(|f| format!("family {f} / {variant}"))
            .unwrap_or_else(|| format!("case {case_id} / {variant}"));
        let series = groups
            .entry((serde_json::to_string(metric)?, unit))
            .or_default();
        let point = (x, median(&medians));
        if let Some(row) = series.iter_mut().find(|s| s.label == label) {
            row.points.push(point);
        } else {
            series.push(charts::Series::new(label, vec![point]));
        }
    }
    let estimator_label = match config.estimator {
        crate::summary::Estimator::ProcessMedian => {
            "median of process median batch rates. Every process has equal weight"
        }
        crate::summary::Estimator::Mean => {
            "mean normalized work divided by mean normalized time. Every batch has equal weight; this is a ratio of means, not a mean of rates"
        }
    };
    let mut output = format!(
        "<section><h2>Throughput input summaries</h2><p>X: {}. Y: {estimator_label}. Missing numeric input, unsupported wall contract or incomplete rates: {skipped} series omitted.</p>",
        escape(config.parameter)
    );
    if groups.is_empty() {
        output.push_str("<p>No plottable throughput series: a declared work counter, numeric input and complete positive wall durations are required.</p>");
    }
    for ((contract, unit), series) in groups {
        let mut disconnected = false;
        let series: Vec<_> = series
            .into_iter()
            .flat_map(|row| {
                let incomplete = members
                    .get(&(contract.clone(), row.label.clone()))
                    .is_some_and(|cases| {
                        cases.iter().any(|(case, variant)| {
                            !available.contains(&(case.clone(), variant.clone(), unit.clone()))
                        })
                    });
                if incomplete {
                    disconnected = true;
                    row.points
                        .into_iter()
                        .map(|point| charts::Series::new(&row.label, vec![point]))
                        .collect()
                } else {
                    vec![row]
                }
            })
            .collect();
        if disconnected {
            output.push_str("<p>Incomplete throughput families are shown as separate points; no line crosses an unavailable case.</p>");
        }
        let title = format!("{} vs throughput ({unit}/s)", config.parameter);
        match charts::line_scaled(
            &title,
            &series,
            "Fixed unit scaling; work counters are not pooled across units.",
            config.scale,
            config.scale,
        ) {
            Ok(chart) => output.push_str(&chart),
            Err(err) => output.push_str(&format!(
                "<p>{}: unavailable: {}</p>",
                escape(&title),
                escape(&err.to_string())
            )),
        }
    }
    output.push_str("</section>");
    Ok(output)
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
    let mut cases: Vec<String> = Vec::new();
    let mut columns: Vec<String> = Vec::new();
    for r in rows {
        if !cases.contains(&r.case) {
            cases.push(r.case.clone());
        }
        let column = format!("{} ({})", r.metric, r.unit);
        if !columns.contains(&column) {
            columns.push(column);
        }
    }
    // An effect heat without any interval behind it would overclaim, so it shares the gate the
    // forest and bar charts use; the matrix itself covers every case, not just the 32 rows.
    if !forest_rows.is_empty() && columns.len() > 1 {
        let mut heat: Vec<_> = cases
            .iter()
            .map(|c| charts::HeatRow {
                label: c.clone(),
                cells: vec![None; columns.len()],
            })
            .collect();
        for r in rows {
            let (Some(row), Some(col)) = (
                cases.iter().position(|c| *c == r.case),
                columns
                    .iter()
                    .position(|m| *m == format!("{} ({})", r.metric, r.unit)),
            ) else {
                continue;
            };
            heat[row].cells[col] = r.change_percent.filter(|v| v.is_finite());
        }
        out.push_str(&charts::heatmap(
            "Change by case and metric, in percent",
            &columns,
            &heat,
            charts::Heat::Signed { center: 0. },
            "%",
            240,
            "One cell per case and metric: the point estimate of the change against the baseline, shaded on a ramp centered at zero. Colors follow each metric's declared direction, so red is a regression wherever the axis points. Cells carry no interval; read the forest above for significance.",
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
            // Below five units a cumulative line is the same dots with a second axis.
            if units.len() >= 5 {
                out.push_str(&charts::ecdf(
                    &format!("{} / {} — cumulative share", r.case, r.metric),
                    &lanes,
                    &r.unit,
                    "Share of independent units at or below the value on the x axis, one line per variant. Where the strip above shows which units were slow, this asks how much of a variant fits under a budget. A line crossing another is not a significance test.",
                ));
            }
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
/// Process matrix: one row per series, one cell per process slot.
///
/// Each row is shaded on its own range, so series measured in different units still show their
/// spread; the median of a process's observations fills the cell, which keeps a chatty process
/// from outweighing its siblings. Empty when the matrix has a single column — one process is a
/// number, not a picture.
pub fn process_heat(series: &[(String, Vec<(u32, f64)>)]) -> String {
    let mut slots: Vec<u32> = Vec::new();
    for (_, values) in series {
        for (process, _) in values {
            if !slots.contains(process) {
                slots.push(*process);
            }
        }
    }
    slots.sort_unstable();
    if slots.len() < 2 {
        return String::new();
    }
    let columns: Vec<String> = slots.iter().map(|p| format!("#{p}")).collect();
    let rows: Vec<_> = series
        .iter()
        .map(|(label, values)| charts::HeatRow {
            label: label.clone(),
            cells: slots
                .iter()
                .map(|slot| {
                    let ys: Vec<f64> = values
                        .iter()
                        .filter(|(p, _)| p == slot)
                        .map(|(_, y)| *y)
                        .collect();
                    if ys.is_empty() {
                        None
                    } else {
                        Some(crate::analysis::median(&ys))
                    }
                })
                .collect(),
        })
        .collect();
    charts::heatmap(
        "Median per process",
        &columns,
        &rows,
        charts::Heat::RowLocal,
        "",
        240,
        "One row per case, metric and variant; one cell per process, shaded on that row's own range. Read it for a process that drifts away from its siblings, not for levels: rows carry different metrics and units, so they are normalized separately. A blank means that process has no value for the row.",
    )
}
/// Baseline/candidate normalized observations on shared axes, without pooling processes.
pub fn iteration_comparison(baseline: &Run, candidate: &Run) -> Result<String> {
    crate::analysis::validate_comparison(baseline, Some(candidate), 5.0, 0.05)?;
    let mut output = String::from(
        "<section><h2>Iteration time comparison</h2><p>X is the recorded sequence within each process; Y is normalized value per operation for batch totals. Processes retain separate series on shared axes. Missing observations are omitted and counted. These points are diagnostics, not confidence intervals.</p>",
    );
    for case in &candidate.cases {
        for metric in &case.metrics {
            let mut groups: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
            let mut missing = 0;
            for (name, run) in [("baseline", baseline), ("candidate", candidate)] {
                for observation in run
                    .observations
                    .iter()
                    .filter(|o| o.case == case.id && o.metric == metric.id)
                {
                    if let Some(mut value) = observation.number()? {
                        if metric.statistic == "batch_total" {
                            value /= observation.operations as f64;
                        }
                        groups
                            .entry(format!(
                                "{name} / {} / process {}",
                                observation.variant, observation.process
                            ))
                            .or_default()
                            .push((observation.sequence as f64, value));
                    } else {
                        missing += 1;
                    }
                }
            }
            let series: Vec<_> = groups
                .into_iter()
                .map(|(label, points)| charts::Series::new(label, points))
                .collect();
            output.push_str(&charts::scatter(
                &format!("{} / {} ({}) — iteration time comparison", case.id, metric.id, metric.unit),
                &series,
                usize::MAX,
                &format!("All available observations; {missing} unavailable observations omitted. Sequence indices do not imply paired measurements."),
            ));
        }
    }
    output.push_str("</section>");
    Ok(output)
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
    let entries: Vec<_> = groups.into_iter().collect();
    let mut series: Vec<(String, Vec<(u32, f64)>)> = Vec::new();
    for ((case, metric, variant, process), values) in &entries {
        let label = format!("{case} / {metric} / {variant}");
        let cell = crate::analysis::median(&values.iter().map(|(_, y)| *y).collect::<Vec<_>>());
        match series.iter_mut().find(|(l, _)| *l == label) {
            Some((_, values)) => values.push((*process, cell)),
            None => series.push((label, vec![(*process, cell)])),
        }
    }
    charts.push_str(&process_heat(&series));
    for ((case, metric, variant, process), mut values) in entries.into_iter().take(64) {
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

/// Work per operation associated with duration-ranked samples, not independently
/// ranked work counters. The median averages the central one or two samples.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssociatedCounter {
    pub fastest: f64,
    pub slowest: f64,
    pub median: f64,
    pub operation_mean: f64,
}
fn associated_counters(
    run: &Run,
    case: &crate::Case,
    variant: &str,
    process: u32,
) -> Result<BTreeMap<String, AssociatedCounter>> {
    let mut units = BTreeMap::new();
    if let (Some(unit), Some(count)) = (
        case.contract.get("work.unit"),
        case.contract.get("work.count"),
    ) {
        units.insert(unit.clone(), Some(count.parse::<u64>()?));
    }
    for (key, value) in &case.contract {
        if let Some(unit) = key.strip_prefix("work.counter.") {
            units.insert(unit.into(), Some(value.parse::<u64>()?));
        }
    }
    for key in case.contract.keys() {
        if let Some(unit) = key.strip_prefix("work.input.") {
            units.insert(unit.into(), None);
        }
    }
    let mut samples = Vec::new();
    for o in run.observations.iter().filter(|o| {
        o.case == case.id && o.metric == "wall" && o.variant == variant && o.process == process
    }) {
        let Some(time) = o.number()? else {
            return Ok(BTreeMap::new());
        };
        let normalized = if case
            .metrics
            .iter()
            .any(|m| m.id == "wall" && m.statistic == "batch_total")
        {
            time / o.operations as f64
        } else {
            time
        };
        samples.push((normalized, o));
    }
    if samples.is_empty() {
        return Ok(BTreeMap::new());
    }
    samples.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1.sequence.cmp(&b.1.sequence))
    });
    let operations: f64 = samples.iter().map(|(_, o)| o.operations as f64).sum();
    let mut result = BTreeMap::new();
    for (unit, fixed) in units {
        let values: Vec<f64> = samples
            .iter()
            .map(|(_, o)| match fixed {
                Some(count) => Ok(count as f64),
                None => Ok(o
                    .work_totals
                    .get(&unit)
                    .ok_or_else(|| crate::error("missing input work total"))?
                    .parse::<u128>()? as f64
                    / o.operations as f64),
            })
            .collect::<Result<_>>()?;
        let middle = values.len() / 2;
        let median = if values.len().is_multiple_of(2) {
            values[middle - 1] / 2.0 + values[middle] / 2.0
        } else {
            values[middle]
        };
        let operation_mean = values
            .iter()
            .zip(&samples)
            .map(|(v, (_, o))| v * (o.operations as f64 / operations))
            .sum();
        result.insert(
            unit,
            AssociatedCounter {
                fastest: values[0],
                slowest: *values.last().unwrap(),
                median,
                operation_mean,
            },
        );
    }
    Ok(result)
}

/// Allocation metrics paired with samples selected by normalized wall time.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssociatedAllocation {
    pub unit: String,
    pub normalized_per_operation: bool,
    pub fastest: f64,
    pub slowest: f64,
    pub median: f64,
    /// Operation-weighted for totals; sample-weighted for peaks.
    pub mean: f64,
    /// Additional per-operation view of an absolute sample peak.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_operation: Option<AssociatedCounter>,
}
fn associated_allocations(
    run: &Run,
    case: &crate::Case,
    variant: &str,
    process: u32,
) -> Result<BTreeMap<String, Option<AssociatedAllocation>>> {
    let wall: Vec<_> = run
        .observations
        .iter()
        .filter(|o| {
            o.case == case.id && o.metric == "wall" && o.variant == variant && o.process == process
        })
        .collect();
    let wall_total = case
        .metrics
        .iter()
        .any(|m| m.id == "wall" && m.statistic == "batch_total");
    let mut times = Vec::new();
    for o in &wall {
        if let Some(value) = o.number()? {
            times.push((
                if wall_total {
                    value / o.operations as f64
                } else {
                    value
                },
                *o,
            ));
        }
    }
    times.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1.sequence.cmp(&b.1.sequence))
    });
    let mut result = BTreeMap::new();
    for metric in case.metrics.iter().filter(|m| m.id.starts_with("alloc.")) {
        let samples: BTreeMap<_, _> = run
            .observations
            .iter()
            .filter(|o| {
                o.case == case.id
                    && o.metric == metric.id
                    && o.variant == variant
                    && o.process == process
            })
            .map(|o| (o.sequence, o))
            .collect();
        if wall.is_empty() || times.len() != wall.len() || samples.len() != wall.len() {
            result.insert(metric.id.clone(), None);
            continue;
        }
        let normalized = metric.statistic == "batch_total";
        let mut values = Vec::new();
        for (_, timing) in &times {
            let Some(sample) = samples.get(&timing.sequence) else {
                break;
            };
            if sample.operations != timing.operations {
                return Err(crate::error(
                    "allocation and timing operation counts differ",
                ));
            }
            let Some(value) = sample.number()? else {
                break;
            };
            values.push(if normalized {
                value / sample.operations as f64
            } else {
                value
            });
        }
        if values.len() != wall.len() {
            result.insert(metric.id.clone(), None);
            continue;
        }
        let middle = values.len() / 2;
        let median = if values.len().is_multiple_of(2) {
            values[middle - 1] / 2.0 + values[middle] / 2.0
        } else {
            values[middle]
        };
        let total: f64 = times.iter().map(|(_, o)| o.operations as f64).sum();
        let scale = values.iter().copied().fold(f64::MIN_POSITIVE, f64::max);
        let mean = values
            .iter()
            .zip(&times)
            .map(|(v, (_, o))| {
                (v / scale)
                    * if normalized {
                        o.operations as f64 / total
                    } else {
                        1.0 / values.len() as f64
                    }
            })
            .sum::<f64>()
            * scale;
        if !mean.is_finite() {
            return Err(crate::error(
                "associated allocation mean exceeds finite numeric range",
            ));
        }
        let per_operation = if metric.statistic == "sample_peak" {
            let per_op: Vec<_> = values
                .iter()
                .zip(&times)
                .map(|(v, (_, o))| v / o.operations as f64)
                .collect();
            // Sum sample peaks / total operations, scaled to avoid sum overflow.
            let operation_mean = values.iter().map(|v| (v / scale) / total).sum::<f64>() * scale;
            if !operation_mean.is_finite() {
                return Err(crate::error(
                    "normalized allocation peak mean exceeds finite numeric range",
                ));
            }
            Some(AssociatedCounter {
                fastest: per_op[0],
                slowest: *per_op.last().unwrap(),
                median: if per_op.len().is_multiple_of(2) {
                    per_op[middle - 1] / 2.0 + per_op[middle] / 2.0
                } else {
                    per_op[middle]
                },
                operation_mean,
            })
        } else {
            None
        };
        result.insert(
            metric.id.clone(),
            Some(AssociatedAllocation {
                unit: metric.unit.clone(),
                normalized_per_operation: normalized,
                fastest: values[0],
                slowest: *values.last().unwrap(),
                median,
                mean,
                per_operation,
            }),
        );
    }
    Ok(result)
}

/// Identity of one measured worker wave; slots are not persistent threads.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WorkerSampleRef {
    pub sequence: u64,
    pub wave: u64,
    pub worker: u32,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WorkerAllocationSummary {
    pub samples: usize,
    pub fastest: WorkerSampleRef,
    pub slowest: WorkerSampleRef,
    pub median_samples: Vec<WorkerSampleRef>,
    pub allocations: BTreeMap<String, Option<AssociatedAllocation>>,
}
fn worker_allocation_summary(
    run: &Run,
    case: &crate::Case,
    variant: &str,
    process: u32,
) -> Result<Option<WorkerAllocationSummary>> {
    let mut workers: Vec<_> = run
        .worker_allocations
        .iter()
        .filter(|w| w.case == case.id && w.variant == variant && w.process == process)
        .collect();
    if workers.is_empty() {
        return Ok(None);
    }
    workers.sort_by(|a, b| {
        let a_time = a.wall_ns.parse::<f64>().unwrap() / a.operations as f64;
        let b_time = b.wall_ns.parse::<f64>().unwrap() / b.operations as f64;
        a_time
            .total_cmp(&b_time)
            .then_with(|| (a.sequence, a.wave, a.worker).cmp(&(b.sequence, b.wave, b.worker)))
    });
    let identity = |w: &crate::WorkerAllocation| WorkerSampleRef {
        sequence: w.sequence,
        wave: w.wave,
        worker: w.worker,
    };
    // Adapt validated worker records to the shared paired-summary estimator.
    // This temporary view is not persisted and is not a second aggregate run.
    let mut view = Run::new();
    for (index, worker) in workers.iter().enumerate() {
        for (metric, value) in std::iter::once(("wall", &worker.wall_ns)).chain(
            worker
                .metrics
                .iter()
                .map(|(id, value)| (id.as_str(), value)),
        ) {
            view.observations.push(crate::Observation {
                work_totals: BTreeMap::new(),
                case: case.id.clone(),
                metric: metric.into(),
                variant: variant.into(),
                process,
                pair: None,
                sequence: index as u64,
                value: Some(value.clone()),
                operations: worker.operations,
                availability: crate::Availability::Available,
            });
        }
    }
    let middle = workers.len() / 2;
    let median_samples = if workers.len().is_multiple_of(2) {
        vec![identity(workers[middle - 1]), identity(workers[middle])]
    } else {
        vec![identity(workers[middle])]
    };
    Ok(Some(WorkerAllocationSummary {
        samples: workers.len(),
        fastest: identity(workers[0]),
        slowest: identity(workers[workers.len() - 1]),
        median_samples,
        allocations: associated_allocations(&view, case, variant, process)?,
    }))
}

/// Descriptive values stay separated by process; normalized batches are not
/// individual-operation latency samples or independent process repetitions.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DescriptiveRow {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub process: u32,
    pub unit: String,
    pub normalized_per_operation: bool,
    pub operations: String,
    /// Mean normalized batch value weighted by actual operation counts.
    pub operation_weighted_mean: Option<f64>,
    pub unavailable: usize,
    pub associated_counters: BTreeMap<String, AssociatedCounter>,
    pub associated_allocations: BTreeMap<String, Option<AssociatedAllocation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_allocations: Option<WorkerAllocationSummary>,
    pub summary: Option<crate::bootstrap::Summary>,
}
pub fn descriptive(run: &Run) -> Result<Vec<DescriptiveRow>> {
    #[derive(Default)]
    struct Group {
        values: Vec<f64>,
        weights: Vec<u64>,
        missing: usize,
        operations: u128,
    }
    run.validate()?;
    let mut result = Vec::new();
    for case in &run.cases {
        for metric in &case.metrics {
            let mut groups: BTreeMap<(&str, u32), Group> = BTreeMap::new();
            for observation in run
                .observations
                .iter()
                .filter(|o| o.case == case.id && o.metric == metric.id)
            {
                let group = groups
                    .entry((&observation.variant, observation.process))
                    .or_default();
                group.operations = group
                    .operations
                    .checked_add(observation.operations as u128)
                    .ok_or_else(|| crate::error("summary operation count overflow"))?;
                if let Some(value) = observation.number()? {
                    group.weights.push(observation.operations);
                    group.values.push(if metric.statistic == "batch_total" {
                        value / observation.operations as f64
                    } else {
                        value
                    });
                } else {
                    group.missing += 1;
                }
            }
            for ((variant, process), group) in groups {
                let Group {
                    values,
                    weights,
                    missing: unavailable,
                    operations,
                } = group;
                let operation_weighted_mean = if metric.statistic == "batch_total"
                    && unavailable == 0
                    && !values.is_empty()
                {
                    let scale = values
                        .iter()
                        .map(|v| v.abs())
                        .fold(f64::MIN_POSITIVE, f64::max);
                    let normalized: f64 = values
                        .iter()
                        .zip(&weights)
                        .map(|(value, weight)| {
                            (value / scale) * (*weight as f64 / operations as f64)
                        })
                        .sum();
                    let mean = normalized * scale;
                    if !mean.is_finite() {
                        return Err(crate::error("weighted mean exceeds finite numeric range"));
                    }
                    Some(mean)
                } else {
                    None
                };
                result.push(DescriptiveRow {
                    case: case.id.clone(),
                    metric: metric.id.clone(),
                    variant: variant.into(),
                    process,
                    unit: metric.unit.clone(),
                    normalized_per_operation: metric.statistic == "batch_total",
                    operations: operations.to_string(),
                    operation_weighted_mean,
                    unavailable,
                    associated_counters: if metric.id == "wall" {
                        associated_counters(run, case, variant, process)?
                    } else {
                        BTreeMap::new()
                    },
                    associated_allocations: if metric.id == "wall" {
                        associated_allocations(run, case, variant, process)?
                    } else {
                        BTreeMap::new()
                    },
                    worker_allocations: if metric.id == "wall" {
                        worker_allocation_summary(run, case, variant, process)?
                    } else {
                        None
                    },
                    summary: if unavailable > 0 || values.is_empty() {
                        None
                    } else {
                        Some(crate::bootstrap::describe(&values)?)
                    },
                });
            }
        }
    }
    Ok(result)
}
pub fn descriptive_markdown(rows: &[DescriptiveRow]) -> String {
    let mut text = String::from(
        "\n## Descriptive statistics\n\nPer-process observations; batch totals normalized per operation. No confidence intervals.\n\n| Case / variant | Metric | Process | Count | Min | Max | Sample mean | Operation mean | Median | SD | Scaled MAD | Unit |\n|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n",
    );
    for row in rows {
        if let Some(s) = &row.summary {
            text.push_str(&format!(
                "| {} / {} | {} | {} | {} | {:.4} | {:.4} | {:.4} | {} | {:.4} | {} | {:.4} | {}{} |\n",
                escape(&row.case),
                escape(&row.variant),
                escape(&row.metric),
                row.process,
                s.count,
                s.minimum,
                s.maximum,
                s.mean,
                row.operation_weighted_mean.map(|v| format!("{v:.4}")).unwrap_or_else(|| "n/a".into()),
                s.median,
                s.standard_deviation
                    .map(|v| format!("{v:.4}"))
                    .unwrap_or_else(|| "n/a".into()),
                s.median_absolute_deviation,
                escape(&row.unit),
                if row.normalized_per_operation {
                    "/op"
                } else {
                    ""
                }
            ));
        } else {
            text.push_str(&format!(
                "| {} / {} | {} | {} | n/a ({} unavailable) | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |\n",
                escape(&row.case),
                escape(&row.variant),
                escape(&row.metric),
                row.process,
                row.unavailable
            ));
        }
    }
    if rows.iter().any(|row| !row.associated_counters.is_empty()) {
        text.push_str("\n### Work counts associated with timing\n\nCounts per operation, selected by normalized duration.\n\n| Case / variant | Process | Unit | Fastest | Slowest | Median-time samples | Operation mean |\n|---|---:|---|---:|---:|---:|---:|\n");
        for row in rows {
            for (unit, counts) in &row.associated_counters {
                text.push_str(&format!(
                    "| {} / {} | {} | {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                    escape(&row.case),
                    escape(&row.variant),
                    row.process,
                    escape(unit),
                    counts.fastest,
                    counts.slowest,
                    counts.median,
                    counts.operation_mean
                ));
            }
        }
    }
    if rows
        .iter()
        .any(|row| !row.associated_allocations.is_empty())
    {
        text.push_str("\n### Allocations associated with timing\n\nTotals are per operation; peaks show both absolute sample values and a separately labelled per-operation view.\n\n| Case / variant | Process | Metric | Fastest | Slowest | Median-time samples | Mean | Unit |\n|---|---:|---|---:|---:|---:|---:|---|\n");
        for row in rows {
            for (metric, value) in &row.associated_allocations {
                if let Some(v) = value {
                    text.push_str(&format!(
                        "| {} / {} | {} | {} | {:.4} | {:.4} | {:.4} | {:.4} | {}{} |\n",
                        escape(&row.case),
                        escape(&row.variant),
                        row.process,
                        escape(metric),
                        v.fastest,
                        v.slowest,
                        v.median,
                        v.mean,
                        escape(&v.unit),
                        if v.normalized_per_operation {
                            "/op"
                        } else {
                            ""
                        }
                    ));
                    if let Some(p) = &v.per_operation {
                        text.push_str(&format!(
                            "| {} / {} | {} | {} (per operation) | {:.4} | {:.4} | {:.4} | {:.4} | {}/op |\n",
                            escape(&row.case), escape(&row.variant), row.process, escape(metric),
                            p.fastest, p.slowest, p.median, p.operation_mean, escape(&v.unit)
                        ));
                    }
                } else {
                    text.push_str(&format!("| {} / {} | {} | {} | n/a | n/a | n/a | n/a | incomplete sample pairing |\n", escape(&row.case), escape(&row.variant), row.process, escape(metric)));
                }
            }
        }
    }
    if rows.iter().any(|row| row.worker_allocations.is_some()) {
        text.push_str("\n### Worker allocations associated with timing\n\nEach worker wave is ranked by its own wall time per operation. Waves and workers are not independent process repetitions. Peaks include absolute and per-operation views.\n\n| Case / variant | Process | Worker waves | Metric | Fastest | Slowest | Median-time samples | Mean | Unit |\n|---|---:|---:|---|---:|---:|---:|---:|---|\n");
        for row in rows {
            if let Some(workers) = &row.worker_allocations {
                for (metric, value) in &workers.allocations {
                    if let Some(v) = value {
                        text.push_str(&format!(
                            "| {} / {} | {} | {} | {} | {:.4} | {:.4} | {:.4} | {:.4} | {}{} |\n",
                            escape(&row.case),
                            escape(&row.variant),
                            row.process,
                            workers.samples,
                            escape(metric),
                            v.fastest,
                            v.slowest,
                            v.median,
                            v.mean,
                            escape(&v.unit),
                            if v.normalized_per_operation {
                                "/op"
                            } else {
                                ""
                            }
                        ));
                        if let Some(p) = &v.per_operation {
                            text.push_str(&format!("| {} / {} | {} | {} | {} (per operation) | {:.4} | {:.4} | {:.4} | {:.4} | {}/op |\n",
                                escape(&row.case), escape(&row.variant), row.process, workers.samples,
                                escape(metric), p.fastest, p.slowest, p.median, p.operation_mean, escape(&v.unit)));
                        }
                    }
                }
            }
        }
    }
    text
}

#[cfg(test)]
mod summary_plot_tests {
    use super::*;
    #[test]
    fn iteration_overlay_normalizes_batches_and_preserves_missing_sequences() {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "input".into(),
                contract: BTreeMap::new(),
                metrics: vec![crate::Metric::duration("wall", "process", "batch_total")],
            })
            .unwrap();
        recorder.observe("input", "wall", 20).unwrap();
        let mut baseline = recorder.finish().unwrap();
        baseline.observations[0].operations = 2;
        let mut candidate = baseline.clone();
        candidate.observations[0].value = Some("80".into());
        candidate.observations[0].operations = 4;
        candidate.observations[0].sequence = 7;
        let mut missing = candidate.observations[0].clone();
        missing.sequence = 8;
        missing.value = None;
        missing.availability = crate::Availability::Unsupported("fixture".into());
        candidate.observations.push(missing);
        let html = iteration_comparison(&baseline, &candidate).unwrap();
        let expected = charts::scatter(
            "input / wall (ns) — iteration time comparison",
            &[
                charts::Series::new(
                    format!(
                        "baseline / {} / process 0",
                        baseline.observations[0].variant
                    ),
                    vec![(baseline.observations[0].sequence as f64, 10.0)],
                ),
                charts::Series::new(
                    format!(
                        "candidate / {} / process 0",
                        candidate.observations[0].variant
                    ),
                    vec![(7.0, 20.0)],
                ),
            ],
            usize::MAX,
            "All available observations; 1 unavailable observations omitted. Sequence indices do not imply paired measurements.",
        );
        assert!(html.contains(&expected));
        assert_eq!(html.matches("<circle").count(), 2);
        candidate.cases[0].metrics[0].unit = "cycles".into();
        assert!(iteration_comparison(&baseline, &candidate).is_err());
    }

    #[test]
    fn parameter_summary_weights_processes_equally_and_normalizes_batches() {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "input".into(),
                contract: BTreeMap::from([("param.size".into(), "16".into())]),
                metrics: vec![crate::Metric::duration("wall", "process", "batch_total")],
            })
            .unwrap();
        recorder.observe("input", "wall", 1).unwrap();
        let mut run = recorder.finish().unwrap();
        let original = run.observations[0].clone();
        run.observations.clear();
        for sequence in 0..6 {
            let mut observation = original.clone();
            observation.process = if sequence < 5 { 0 } else { 1 };
            observation.sequence = sequence;
            observation.operations = 2;
            observation.value = Some(if sequence < 5 { "20" } else { "200" }.into());
            run.observations.push(observation);
        }
        let mut unavailable = original;
        unavailable.process = 2;
        unavailable.value = None;
        unavailable.availability = crate::Availability::Unsupported("fixture".into());
        let config = SummaryPlot {
            estimator: Default::default(),
            parameter: "size",
            scale: charts::AxisScale::Linear,
        };
        let html = parameter_charts(&run, &config).unwrap();
        // Process medians are 10 and 100 ns/op, so the summary is 55, not 10.
        let expected = charts::scatter_scaled(
            "size vs wall (ns; process; measurement; batch_total)",
            &[charts::Series::new(
                format!("input / {}", run.observations[0].variant),
                vec![(16., 55.)],
            )],
            1000,
            "Metric contracts are not pooled. Batch totals are normalized per operation.",
            charts::AxisScale::Linear,
            charts::AxisScale::Linear,
        );
        assert!(html.contains(&expected));
        assert_eq!(html.matches("<circle").count(), 1);
        let mean_config = SummaryPlot {
            estimator: crate::summary::Estimator::Mean,
            ..config
        };
        let mean_html = parameter_charts(&run, &mean_config).unwrap();
        let mean_expected = charts::scatter_scaled(
            "size vs wall (ns; process; measurement; batch_total)",
            &[charts::Series::new("mean", vec![(16., 25.)])],
            1000,
            "Metric contracts are not pooled. Batch totals are normalized per operation.",
            charts::AxisScale::Linear,
            charts::AxisScale::Linear,
        );
        assert!(mean_html.contains(&mean_expected));
        assert!(mean_html.contains("arithmetic mean of normalized observations"));
        run.cases[0]
            .contract
            .insert("work.counter.items".into(), "2".into());
        let throughput_html = throughput_parameter_charts(&run, &mean_config).unwrap();
        let expected = charts::line_scaled(
            "size vs throughput (items/s)",
            &[charts::Series::new(
                format!("case input / {}", run.observations[0].variant),
                vec![(16., 80_000_000.)],
            )],
            "Fixed unit scaling; work counters are not pooled across units.",
            charts::AxisScale::Linear,
            charts::AxisScale::Linear,
        )
        .unwrap();
        assert!(throughput_html.contains(&expected));
        assert!(throughput_html.contains("ratio of means"));
        run.observations.push(unavailable);
        let incomplete = parameter_charts(&run, &config).unwrap();
        assert!(!incomplete.contains("<circle"));
        assert!(incomplete.contains("metric has unavailable observations"));
    }

    #[test]
    fn numeric_contract_summary_keeps_metric_scopes_separate() {
        let mut recorder = crate::Recorder::new();
        for (id, parameter, scope) in [
            ("small", "10", "one"),
            ("large", "100", "two"),
            ("missing", "text", "one"),
        ] {
            recorder
                .case(crate::Case {
                    id: id.into(),
                    contract: BTreeMap::from([("param.<size>".into(), parameter.into())]),
                    metrics: vec![crate::Metric::duration("wall", scope, "batch_total")],
                })
                .unwrap();
            recorder.observe(id, "wall", 100).unwrap();
        }
        let run = recorder.finish().unwrap();
        let config = SummaryPlot {
            estimator: Default::default(),
            parameter: "<size>",
            scale: charts::AxisScale::Logarithmic,
        };
        let charts = parameter_charts(&run, &config).unwrap();
        assert_eq!(charts.matches("<svg").count(), 2);
        assert_eq!(charts.matches("<circle").count(), 2);
        assert!(charts.contains("1 series omitted"));
        assert!(charts.contains("&lt;size&gt;"));
        assert!(!charts.contains("NaN"));
        let document = html_run_with_summary(&run, Some(config)).unwrap();
        assert!(document.contains(&charts));
        assert!(
            !html_run(&run)
                .unwrap()
                .contains("Input parameter summaries")
        );
    }
}
