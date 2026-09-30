//! User-defined measurements independent of the clock used for scheduling.
use crate::{Metric, Result};

/// A measurement may retain an arbitrary start state and accumulate a custom value.
/// `metric` must describe a nonnegative batch total. Implementations must include
/// configuration affecting values in the metric scope to preserve comparability.
pub trait Measurement {
    type Start;
    type Value;
    fn metric(&self) -> Metric;
    fn start(&mut self) -> Result<Self::Start>;
    fn end(&mut self, start: Self::Start) -> Result<Self::Value>;
    fn zero(&self) -> Self::Value;
    fn add(&self, total: Self::Value, value: Self::Value) -> Result<Self::Value>;
    fn to_f64(&self, value: &Self::Value) -> Result<f64>;
}

/// Scaled values and their display unit. Scaling must preserve input order/count.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FormattedValues {
    pub values: Vec<f64>,
    pub unit: String,
}

/// User-defined presentation. Values supplied here are normalized per operation.
/// A shared typical value allows multiple series to use consistent display units.
pub trait ValueFormatter {
    /// Optional human text for an already scaled value. None keeps numeric rendering.
    /// The unit and number remain available separately for charts and machine consumers.
    fn format_value(&self, _value: f64, _unit: &str) -> Result<Option<String>> {
        Ok(None)
    }
    /// Optional human text for an already converted throughput value.
    fn format_throughput(&self, value: f64, unit: &str) -> Result<Option<String>> {
        self.format_value(value, unit)
    }
    fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues>;
    fn scale_throughputs(
        &self,
        typical: f64,
        work: f64,
        work_unit: &str,
        values: &[f64],
    ) -> Result<FormattedValues>;
    /// Work varies per observation. Override to choose a shared unit across all rates.
    /// The default reuses fixed-work formatting and rejects inconsistent units.
    fn scale_input_throughputs(
        &self,
        typical: f64,
        work: &[f64],
        work_unit: &str,
        values: &[f64],
    ) -> Result<FormattedValues> {
        if work.len() != values.len() {
            return Err(crate::error("work/value count mismatch"));
        }
        let mut output = FormattedValues {
            values: Vec::new(),
            unit: String::new(),
        };
        for (&work, &value) in work.iter().zip(values) {
            let row = self.scale_throughputs(typical, work, work_unit, &[value])?;
            if row.values.len() != 1 || (!output.unit.is_empty() && output.unit != row.unit) {
                return Err(crate::error(
                    "input throughput formatter must return one shared unit; override scale_input_throughputs",
                ));
            }
            output.unit = row.unit;
            output.values.extend(row.values);
        }
        Ok(output)
    }
    fn scale_for_machines(&self, values: &[f64]) -> Result<FormattedValues>;
}

#[derive(Clone, Copy, Debug)]
pub enum Format<'a> {
    Human,
    Machine,
    Throughput { work: f64, unit: &'a str },
    InputThroughput { unit: &'a str },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormattedObservation {
    pub variant: String,
    pub process: u32,
    pub sequence: u64,
    pub value: Option<f64>,
    pub unit: String,
    pub unavailable_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// Format one metric without modifying the saved run. The complete population
/// selects a shared scale across variants/processes; missing rows remain missing.
pub fn format_observations(
    run: &crate::Run,
    case: &str,
    metric: &str,
    formatter: &dyn ValueFormatter,
    format: Format<'_>,
) -> Result<Vec<FormattedObservation>> {
    run.validate()?;
    let contract = run
        .cases
        .iter()
        .find(|c| c.id == case)
        .and_then(|c| c.metrics.iter().find(|m| m.id == metric))
        .ok_or_else(|| crate::error("unknown case or metric for formatting"))?;
    let observations: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case == case && o.metric == metric)
        .collect();
    let values: Vec<f64> = observations
        .iter()
        .filter_map(|o| match o.number() {
            Ok(Some(value)) => Some(Ok(if contract.statistic == "batch_total" {
                value / o.operations as f64
            } else {
                value
            })),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<_>>()?;
    if let Format::Throughput { work, unit } = format {
        if !work.is_finite() || work < 0.0 || unit.is_empty() {
            return Err(crate::error(
                "throughput requires finite nonnegative work and a unit",
            ));
        }
    }
    if let Format::InputThroughput { unit } = format {
        let case_contract = run.cases.iter().find(|c| c.id == case).unwrap();
        if case_contract
            .contract
            .get(&format!("work.input.{unit}"))
            .map(String::as_str)
            != Some("batch_total")
        {
            return Err(crate::error(
                "input throughput requires a declared dynamic counter",
            ));
        }
    }
    let scaled = if values.is_empty() {
        FormattedValues {
            values: vec![],
            unit: contract.unit.clone(),
        }
    } else {
        let typical = values.iter().copied().fold(0.0_f64, f64::max);
        match format {
            Format::Human => formatter.scale_values(typical, &values)?,
            Format::Machine => formatter.scale_for_machines(&values)?,
            Format::InputThroughput { unit } => {
                let work = observations
                    .iter()
                    .filter(|o| matches!(o.availability, crate::Availability::Available))
                    .map(|o| {
                        let wall = run
                            .observations
                            .iter()
                            .find(|w| {
                                w.case == o.case
                                    && w.metric == "wall"
                                    && w.variant == o.variant
                                    && w.process == o.process
                                    && w.sequence == o.sequence
                                    && w.pair == o.pair
                            })
                            .ok_or_else(|| {
                                crate::error("matching wall work observation missing")
                            })?;
                        if wall.operations != o.operations {
                            return Err(crate::error(
                                "custom metric and work operation counts differ",
                            ));
                        }
                        let total = wall
                            .work_totals
                            .get(unit)
                            .ok_or_else(|| crate::error("input work total missing"))?
                            .parse::<u128>()?;
                        Ok(total as f64 / o.operations as f64)
                    })
                    .collect::<Result<Vec<_>>>()?;
                formatter.scale_input_throughputs(typical, &work, unit, &values)?
            }
            Format::Throughput { work, unit } => {
                formatter.scale_throughputs(typical, work, unit, &values)?
            }
        }
    };
    if scaled.values.len() != values.len()
        || scaled.unit.is_empty()
        || scaled.values.iter().any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err(crate::error(
            "formatter returned invalid values, unit or observation count",
        ));
    }
    let mut converted = scaled.values.into_iter();
    observations
        .into_iter()
        .map(|o| {
            let available = o.number()?.is_some();
            let value = if available { converted.next() } else { None };
            let display = match (value, format) {
                (Some(value), Format::Human) => formatter.format_value(value, &scaled.unit)?,
                (Some(value), Format::Throughput { .. } | Format::InputThroughput { .. }) => {
                    formatter.format_throughput(value, &scaled.unit)?
                }
                _ => None,
            };
            Ok(FormattedObservation {
                display,
                variant: o.variant.clone(),
                process: o.process,
                sequence: o.sequence,
                value,
                unit: scaled.unit.clone(),
                unavailable_reason: (!available).then(|| format!("{:?}", o.availability)),
            })
        })
        .collect()
}

/// Table suitable for console output or `report::html`. Machine-scaled rows can
/// also be serialized directly to JSON without changing original observations.
pub fn markdown(rows: &[FormattedObservation]) -> String {
    let mut result = String::from(
        "| Variant | Process | Sequence | Value | Unit | Availability | Display |\n|---|---|---|---|---|---|---|\n",
    );
    for row in rows {
        result.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            crate::report::escape(&row.variant),
            row.process,
            row.sequence,
            row.value.map(|v| v.to_string()).unwrap_or_default(),
            crate::report::escape(&row.unit),
            crate::report::escape(row.unavailable_reason.as_deref().unwrap_or("available")),
            crate::report::escape(row.display.as_deref().unwrap_or(""))
        ));
    }
    result
}

/// Derived presentation export; original metric contract and observations remain in run.json.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormattedMetric {
    pub case: String,
    pub metric: Metric,
    pub human: Vec<FormattedObservation>,
    pub machine: Vec<FormattedObservation>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub throughput: std::collections::BTreeMap<String, FormattedThroughput>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormattedThroughput {
    /// None means work is read from each matching wall observation.
    pub work_per_operation: Option<u64>,
    pub observations: Vec<FormattedObservation>,
}
/// CSV of machine-formatted observations. Batch totals have already been
/// normalized per operation before formatter conversion; raw data stays in run.json.
/// Text fields use standard CSV quoting and retain their exact contents.
pub fn report_csv(metrics: &[FormattedMetric]) -> String {
    fn quoted(value: &str) -> String {
        format!("\"{}\"", value.replace('"', "\"\""))
    }
    let mut output = String::from(
        "case,metric,variant,process,sequence,value,unit,normalized_per_operation,unavailable_reason\n",
    );
    for metric in metrics {
        for row in &metric.machine {
            let fields = [
                quoted(&metric.case),
                quoted(&metric.metric.id),
                quoted(&row.variant),
                row.process.to_string(),
                row.sequence.to_string(),
                row.value.map(|value| value.to_string()).unwrap_or_default(),
                quoted(&row.unit),
                (metric.metric.statistic == "batch_total").to_string(),
                quoted(row.unavailable_reason.as_deref().unwrap_or_default()),
            ];
            output.push_str(&fields.join(","));
            output.push('\n');
        }
    }
    output
}

pub fn report_markdown(metrics: &[FormattedMetric]) -> String {
    let mut output = String::new();
    for metric in metrics {
        output.push_str(&format!(
            "\n## Formatted metric: {} / {}\n\n{}",
            crate::report::escape(&metric.case),
            crate::report::escape(&metric.metric.id),
            markdown(&metric.human)
        ));
        for (unit, throughput) in &metric.throughput {
            output.push_str(&format!(
                "\n### Formatted throughput: {} per operation\n\n{}",
                crate::report::escape(unit),
                markdown(&throughput.observations)
            ));
        }
    }
    output
}

const PRESENTATION_KEY: &str = "airbug.presentation.formatted.v1";
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedMetric {
    source_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    work_source_sha256: Option<String>,
    presentation: FormattedMetric,
}
fn source_hash(run: &crate::Run, case: &str, metric: &Metric) -> Result<String> {
    use sha2::{Digest, Sha256};
    let observations: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case == case && o.metric == metric.id)
        .collect();
    let bytes = serde_json::to_vec(&(case, metric, observations))?;
    Ok(crate::model::hex(&Sha256::digest(bytes)))
}

fn work_source_hash(run: &crate::Run, case: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let wall: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case == case && o.metric == "wall")
        .collect();
    Ok(crate::model::hex(&Sha256::digest(serde_json::to_vec(
        &wall,
    )?)))
}

/// Save presentation snapshots beside raw data, outside measurement contracts.
pub fn save_formatted(run: &mut crate::Run, metrics: &[FormattedMetric]) -> Result<()> {
    run.validate()?;
    let saved: Vec<_> = metrics
        .iter()
        .map(|metric| {
            Ok(SavedMetric {
                source_sha256: source_hash(run, &metric.case, &metric.metric)?,
                work_source_sha256: if metric
                    .throughput
                    .values()
                    .any(|t| t.work_per_operation.is_none())
                {
                    Some(work_source_hash(run, &metric.case)?)
                } else {
                    None
                },
                presentation: metric.clone(),
            })
        })
        .collect::<Result<_>>()?;
    let json = serde_json::to_string(&saved)?;
    let mut candidate = run.clone();
    candidate
        .provenance
        .insert(PRESENTATION_KEY.into(), json.clone());
    load_formatted(&candidate)?;
    run.provenance.insert(PRESENTATION_KEY.into(), json);
    Ok(())
}

/// Restore snapshots only if their source observations and metric contract still match.
/// Filtered-out cases/metrics are ignored; edited observations invalidate their snapshot.
pub fn load_formatted(run: &crate::Run) -> Result<Vec<FormattedMetric>> {
    let Some(json) = run.provenance.get(PRESENTATION_KEY) else {
        return Ok(vec![]);
    };
    let saved: Vec<SavedMetric> = serde_json::from_str(json)?;
    let mut result = Vec::new();
    for saved in saved {
        let formatted = saved.presentation;
        let Some(metric) = run
            .cases
            .iter()
            .find(|c| c.id == formatted.case)
            .and_then(|c| c.metrics.iter().find(|m| m.id == formatted.metric.id))
        else {
            continue;
        };
        if metric != &formatted.metric
            || saved.source_sha256 != source_hash(run, &formatted.case, metric)?
        {
            return Err(crate::error(
                "saved formatting is stale: metric or observations changed",
            ));
        }
        let observations: Vec<_> = run
            .observations
            .iter()
            .filter(|o| o.case == formatted.case && o.metric == metric.id)
            .collect();
        let case = run.cases.iter().find(|c| c.id == formatted.case).unwrap();
        let counters = crate::report::work_counters(case)?;
        for (unit, throughput) in &formatted.throughput {
            if counters.get(unit.as_str()) != Some(&throughput.work_per_operation) {
                return Err(crate::error(
                    "saved throughput formatting is stale: work counter changed",
                ));
            }
        }
        if formatted
            .throughput
            .values()
            .any(|t| t.work_per_operation.is_none())
            && saved.work_source_sha256.as_deref()
                != Some(work_source_hash(run, &formatted.case)?.as_str())
        {
            return Err(crate::error(
                "saved throughput formatting is stale: input work observations changed",
            ));
        }
        for rows in [&formatted.human, &formatted.machine]
            .into_iter()
            .chain(formatted.throughput.values().map(|t| &t.observations))
        {
            if rows.len() != observations.len() {
                return Err(crate::error("saved formatting row count mismatch"));
            }
            for (row, observation) in rows.iter().zip(&observations) {
                if row.variant != observation.variant
                    || row.process != observation.process
                    || row.sequence != observation.sequence
                    || row.unit.is_empty()
                    || (row.value.is_none() && row.display.is_some())
                    || row.value.is_some() != observation.number()?.is_some()
                    || row.value.is_some_and(|v| !v.is_finite() || v < 0.0)
                {
                    return Err(crate::error("invalid saved formatting row"));
                }
            }
        }
        result.push(formatted);
    }
    Ok(result)
}

/// Collect validated worker snapshots while the runner assigns global identities.
/// Only metrics formatted by every worker survive. Display units are preserved;
/// independently chosen scales are never treated as one common numeric axis.
#[derive(Default)]
pub struct ProcessFormatting {
    metrics: Option<Vec<FormattedMetric>>,
}
impl ProcessFormatting {
    pub fn observe(&mut self, worker: &crate::Run, process: u32, variant: &str) -> Result<()> {
        let mut next = load_formatted(worker)?;
        for metric in &mut next {
            for rows in [&mut metric.human, &mut metric.machine]
                .into_iter()
                .chain(metric.throughput.values_mut().map(|t| &mut t.observations))
            {
                for row in rows {
                    row.process = process;
                    row.variant = variant.to_owned();
                }
            }
        }
        let Some(previous) = &mut self.metrics else {
            self.metrics = Some(next);
            return Ok(());
        };
        previous.retain_mut(|metric| {
            let Some(incoming) = next
                .iter()
                .find(|m| m.case == metric.case && m.metric == metric.metric)
            else {
                return false;
            };
            metric.human.extend(incoming.human.iter().cloned());
            metric.machine.extend(incoming.machine.iter().cloned());
            metric.throughput.retain(|unit, throughput| {
                let Some(other) = incoming.throughput.get(unit) else {
                    return false;
                };
                if throughput.work_per_operation != other.work_per_operation {
                    return false;
                }
                throughput
                    .observations
                    .extend(other.observations.iter().cloned());
                true
            });
            true
        });
        Ok(())
    }
    /// Call after all corresponding raw observations have been rebased and appended.
    pub fn save(&self, run: &mut crate::Run) -> Result<()> {
        if let Some(metrics) = &self.metrics {
            save_formatted(run, metrics)?;
        }
        Ok(())
    }
}

/// Plot saved human-scaled values without reinterpreting raw metric units.
/// Distinct display units get separate axes, including across processes/variants.
pub fn charts(run: &crate::Run) -> Result<String> {
    let formatted = load_formatted(run)?;
    let mut output = String::new();
    for metric in formatted {
        let title = format!("{} / {}", metric.case, metric.metric.id);
        output.push_str(&formatted_chart_rows(
            &format!("{title} — formatted observations"),
            &metric.human,
        ));
        for (counter, throughput) in &metric.throughput {
            output.push_str(&formatted_chart_rows(
                &format!("{title} — formatted throughput: {counter}"),
                &throughput.observations,
            ));
        }
    }
    Ok(output)
}

fn formatted_chart_rows(title: &str, rows: &[FormattedObservation]) -> String {
    let mut groups: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, Vec<(f64, f64)>>,
    > = Default::default();
    for row in rows {
        if let Some(value) = row.value {
            groups
                .entry(row.unit.clone())
                .or_default()
                .entry(format!("{} / process {}", row.variant, row.process))
                .or_default()
                .push((row.sequence as f64, value));
        }
    }
    let mut output = String::new();
    for (unit, series) in groups {
        let series: Vec<_> = series
            .into_iter()
            .map(|(label, points)| crate::viz::charts::Series::new(label, points))
            .collect();
        output.push_str(&crate::viz::charts::scatter(
            &format!("{title} ({unit})"), &series, 1000,
            "Saved formatter output; X is sequence within each process. Missing values are not plotted. At most 1000 points per series; run.json retains all saved rows. These are descriptive observations, not confidence intervals.",
        ));
    }
    output
}

/// Reformat compatible independent runs together so baseline and candidate use
/// one typical value and unit choice. Never overlays independently scaled snapshots.
pub fn format_comparison(
    baseline: &crate::Run,
    candidate: &crate::Run,
    case: &str,
    metric: &str,
    formatter: &dyn ValueFormatter,
    format: Format<'_>,
) -> Result<Vec<FormattedObservation>> {
    crate::analysis::validate_comparison(baseline, Some(candidate), 5.0, 0.05)?;
    let mut combined = candidate.clone();
    for observation in &baseline.observations {
        let mut observation = observation.clone();
        observation.variant = "baseline".into();
        combined.observations.push(observation);
    }
    for allocation in &baseline.worker_allocations {
        let mut allocation = allocation.clone();
        allocation.variant = "baseline".into();
        combined.worker_allocations.push(allocation);
    }
    format_observations(&combined, case, metric, formatter, format)
}

/// Compare observations in a unit chosen from both raw populations.
pub fn comparison_chart(
    baseline: &crate::Run,
    candidate: &crate::Run,
    case: &str,
    metric: &str,
    formatter: &dyn ValueFormatter,
) -> Result<String> {
    let rows = format_comparison(baseline, candidate, case, metric, formatter, Format::Human)?;
    let mut output =
        comparison_rows_chart(&format!("{case} / {metric} — formatted comparison"), &rows);
    let case_contract = candidate
        .cases
        .iter()
        .find(|c| c.id == case)
        .ok_or_else(|| crate::error("unknown comparison case"))?;
    for (counter, count) in crate::report::work_counters(case_contract)? {
        let format = match count {
            Some(work) => Format::Throughput {
                work: work as f64,
                unit: counter,
            },
            None => Format::InputThroughput { unit: counter },
        };
        let rows = format_comparison(baseline, candidate, case, metric, formatter, format)?;
        output.push_str(&comparison_rows_chart(
            &format!("{case} / {metric} — formatted throughput comparison: {counter}"),
            &rows,
        ));
    }
    Ok(output)
}

fn comparison_rows_chart(title: &str, rows: &[FormattedObservation]) -> String {
    let unit = rows
        .first()
        .map(|r| r.unit.as_str())
        .unwrap_or("unavailable");
    let mut groups: std::collections::BTreeMap<String, Vec<(f64, f64)>> = Default::default();
    for row in rows {
        if let Some(value) = row.value {
            groups
                .entry(format!("{} / process {}", row.variant, row.process))
                .or_default()
                .push((row.sequence as f64, value));
        }
    }
    let series: Vec<_> = groups
        .into_iter()
        .map(|(label, points)| crate::viz::charts::Series::new(label, points))
        .collect();
    crate::viz::charts::scatter(
        &format!("{title} ({unit})"),
        &series,
        1000,
        "One formatter scale selected from both raw populations. X is sequence within each process, not a paired measurement. Missing values omitted; at most 1000 points per series.",
    )
}

/// Format absolute bootstrap estimates and retained draws on one shared scale.
/// Outlier values and fences share the estimate scale; original classifications are
/// retained. Regression totals, slopes and slope draws use the same positive linear
/// unit conversion; operation counts and dimensionless fit quality stay unchanged.
/// The original report is unchanged.
pub fn format_bootstrap_metric(
    report: &crate::bootstrap::Report,
    case: &str,
    metric: &Metric,
    formatter: &dyn ValueFormatter,
) -> Result<crate::bootstrap::Report> {
    let mut result = report.clone();
    result
        .rows
        .retain(|row| row.case == case && row.metric == metric.id);
    if result
        .rows
        .iter()
        .any(|row| row.metric_contract.as_ref() != Some(metric))
    {
        return Err(crate::error(
            "formatter metric contract differs from bootstrap report",
        ));
    }
    let typical = result
        .rows
        .iter()
        .filter_map(|row| row.estimates.as_ref())
        .map(|estimates| estimates.mean.point)
        .fold(0.0f64, f64::max);
    let mut references = Vec::new();
    let mut signed_positions = Vec::new();
    for row in &mut result.rows {
        if let Some(estimates) = &mut row.estimates {
            for estimate in [
                &mut estimates.mean,
                &mut estimates.median,
                &mut estimates.standard_deviation,
                &mut estimates.median_absolute_deviation,
            ] {
                references.extend([
                    &mut estimate.point,
                    &mut estimate.lower,
                    &mut estimate.upper,
                    &mut estimate.standard_error,
                ]);
            }
            references.extend([&mut estimates.minimum, &mut estimates.maximum]);
        }
        for regression in &mut row.regressions {
            let slope = &mut regression.fit.slope;
            references.extend([
                &mut slope.point,
                &mut slope.lower,
                &mut slope.upper,
                &mut slope.standard_error,
            ]);
            references.extend(
                regression
                    .samples
                    .iter_mut()
                    .map(|sample| &mut sample.total),
            );
            if let Some(draws) = &mut regression.slope_distribution {
                references.extend(draws.iter_mut());
            }
        }
        if let Some(outliers) = &mut row.outliers {
            references.extend([&mut outliers.q1, &mut outliers.q3]);
            references.extend(outliers.points.iter_mut().map(|point| &mut point.value));
            for fence in outliers.fences.iter_mut().flatten() {
                signed_positions.push(references.len());
                references.push(fence);
            }
        }
        if let Some(draws) = &mut row.distributions {
            references.extend(draws.mean.iter_mut());
            references.extend(draws.median.iter_mut());
            references.extend(draws.standard_deviation.iter_mut());
            references.extend(draws.median_absolute_deviation.iter_mut());
        }
    }
    let values: Vec<_> = references.iter().map(|value| **value).collect();
    if values.is_empty() {
        return Ok(result);
    }
    let scaled = formatter.scale_values(typical, &values)?;
    if scaled.values.len() != values.len()
        || scaled.unit.is_empty()
        || scaled.values.iter().enumerate().any(|(index, value)| {
            !value.is_finite() || (*value < 0.0 && !signed_positions.contains(&index))
        })
    {
        return Err(crate::error("invalid bootstrap formatter output"));
    }
    // Statistical estimates and through-origin fits are valid under a positive
    // linear unit conversion, not an offset or a nonlinear transformation.
    if let Some((anchor, converted)) = values
        .iter()
        .zip(&scaled.values)
        .filter(|(value, _)| **value > 0.0)
        .max_by(|a, b| a.0.total_cmp(b.0))
    {
        if *converted <= 0.0
            || values.iter().zip(&scaled.values).any(|(raw, display)| {
                if *raw == 0.0 {
                    return *display != 0.0;
                }
                let expected = raw / anchor;
                let actual = display / converted;
                !actual.is_finite() || (actual - expected).abs() > 1e-12 * expected.abs()
            })
        {
            return Err(crate::error(
                "bootstrap formatter must use a positive linear scale",
            ));
        }
    } else if scaled.values.iter().any(|value| *value != 0.0) {
        return Err(crate::error("bootstrap formatter must preserve zero"));
    }
    // A formatter changes units, so observation and fence ordering must survive.
    let mut ordering: Vec<_> = values
        .iter()
        .copied()
        .zip(scaled.values.iter().copied())
        .collect();
    ordering.sort_by(|a, b| a.0.total_cmp(&b.0));
    if ordering
        .windows(2)
        .any(|pair| pair[0].1 > pair[1].1 || (pair[0].0 == pair[1].0 && pair[0].1 != pair[1].1))
    {
        return Err(crate::error(
            "bootstrap formatter must preserve value order",
        ));
    }
    for (destination, value) in references.into_iter().zip(scaled.values) {
        *destination = value;
    }
    for row in &mut result.rows {
        if let Some(estimates) = &row.estimates {
            if [
                &estimates.mean,
                &estimates.median,
                &estimates.standard_deviation,
                &estimates.median_absolute_deviation,
            ]
            .iter()
            .any(|e| e.lower > e.upper)
                || estimates.minimum > estimates.maximum
            {
                return Err(crate::error(
                    "bootstrap formatter must preserve interval order",
                ));
            }
        }
        row.unit.clone_from(&scaled.unit);
        if let Some(contract) = &mut row.metric_contract {
            contract.unit.clone_from(&scaled.unit);
        }
        row.note.push_str(
            " Derived formatter scale; original numerical estimates retained separately.",
        );
    }
    Ok(result)
}
