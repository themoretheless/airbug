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

/// Images of closed intervals in one shared display unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FormattedIntervals {
    pub bounds: Vec<[f64; 2]>,
    pub unit: String,
}

/// User-defined presentation. Observation tables use values normalized per operation.
/// Statistical reports also supply estimates, deviations, outlier fences and regression
/// totals. A shared typical value lets related series use consistent display units.
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
    /// Convert values to a shared display unit, preserving input order and length.
    /// Linear conversions change statistical report units directly. Increasing
    /// nonlinear/offset transforms produce a separate labelled display while raw
    /// statistics retain their units. Its standard errors require retained bootstrap
    /// draws. Nonlinear regression plots transform the model curve and its bounds.
    /// Observation-only formatting and throughput have separate validation rules.
    fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues>;
    /// Optional bounds for the image of each closed interval under `scale_values`.
    /// Return one finite ordered [minimum, maximum] pair per input, in the same
    /// unit selected by `scale_values` for this typical value. Include interior
    /// extrema, not only endpoint images. The implementor guarantees containment
    /// for every value in each interval; finite sampling cannot prove it.
    ///
    /// Regression plots use these bounds when supplied. None uses endpoint images
    /// and requires a monotone transform over the full regression domain.
    fn scale_intervals(
        &self,
        _typical: f64,
        _intervals: &[[f64; 2]],
    ) -> Result<Option<FormattedIntervals>> {
        Ok(None)
    }

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
        || scaled
            .values
            .iter()
            .any(|v| !v.is_finite() || (*v < 0.0 && !matches!(format, Format::Human)))
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

/// Machine-formatted CSV enriched from the validated source run. Older saved
/// formatter snapshots need no migration: operations and contracts come from raw
/// observations, never from the current benchmark registration. `value` remains
/// formatter-scaled per-operation data for batch totals; `raw_value` is unchanged.
pub fn saved_report_csv(run: &crate::Run) -> Result<String> {
    run.validate()?;
    let metrics = load_formatted(run)?;
    let mut output = String::from(
        "case,metric,variant,process,sequence,value,unit,normalized_per_operation,unavailable_reason,operations,pair,raw_value,raw_unit,case_contract,work_totals,worker_work_totals\n",
    );
    let wall: std::collections::BTreeMap<_, _> = run
        .observations
        .iter()
        .filter(|observation| observation.metric == "wall")
        .map(|observation| {
            (
                (
                    observation.case.as_str(),
                    observation.variant.as_str(),
                    observation.process,
                    observation.pair,
                    observation.sequence,
                    observation.operations,
                ),
                observation,
            )
        })
        .collect();
    for metric in metrics {
        let case = run
            .cases
            .iter()
            .find(|case| case.id == metric.case)
            .unwrap();
        let observations = run.observations.iter().filter(|observation| {
            observation.case == metric.case && observation.metric == metric.metric.id
        });
        // load_formatted validates the count and identity of every row in order.
        for (row, raw) in metric.machine.iter().zip(observations) {
            let work = wall
                .get(&(
                    raw.case.as_str(),
                    raw.variant.as_str(),
                    raw.process,
                    raw.pair,
                    raw.sequence,
                    raw.operations,
                ))
                .copied()
                .unwrap_or(raw);
            let fields = [
                metric.case.clone(),
                metric.metric.id.clone(),
                row.variant.clone(),
                row.process.to_string(),
                row.sequence.to_string(),
                row.value.map(|value| value.to_string()).unwrap_or_default(),
                row.unit.clone(),
                (metric.metric.statistic == "batch_total").to_string(),
                row.unavailable_reason.clone().unwrap_or_default(),
                raw.operations.to_string(),
                raw.pair.map(|pair| pair.to_string()).unwrap_or_default(),
                raw.value.clone().unwrap_or_default(),
                metric.metric.unit.clone(),
                serde_json::to_string(&case.contract)?,
                serde_json::to_string(&work.work_totals)?,
                serde_json::to_string(&work.worker_work_totals)?,
            ];
            output.push_str(
                &fields
                    .iter()
                    .map(|value| format!("\"{}\"", value.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            output.push('\n');
        }
    }
    Ok(output)
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
        for (rows, allow_negative) in [(&formatted.human, true), (&formatted.machine, false)]
            .into_iter()
            .chain(
                formatted
                    .throughput
                    .values()
                    .map(|t| (&t.observations, false)),
            )
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
                    || row
                        .value
                        .is_some_and(|v| !v.is_finite() || (v < 0.0 && !allow_negative))
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
        )?);
        for (counter, throughput) in &metric.throughput {
            output.push_str(&formatted_chart_rows(
                &format!("{title} — formatted throughput: {counter}"),
                &throughput.observations,
            )?);
        }
    }
    Ok(output)
}

fn formatted_chart_rows(title: &str, rows: &[FormattedObservation]) -> Result<String> {
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
        output.push_str(&formatted_distribution(title, &unit, &series)?);
    }
    Ok(output)
}

fn formatted_distribution(
    title: &str,
    unit: &str,
    series: &[crate::viz::charts::Series],
) -> Result<String> {
    let values: Vec<Vec<f64>> = series
        .iter()
        .map(|series| series.points.iter().map(|(_, value)| *value).collect())
        .collect();
    let populations: Vec<_> = series
        .iter()
        .zip(&values)
        .map(|(series, values)| crate::density::Series {
            label: &series.label,
            values,
        })
        .collect();
    crate::density::figure_with_outliers(
        &populations,
        &format!("{title} — distribution ({unit})"),
        unit,
    )
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
    for allocation in &baseline.worker_timings {
        let mut allocation = allocation.clone();
        allocation.variant = "baseline".into();
        combined.worker_timings.push(allocation);
    }
    format_observations(&combined, case, metric, formatter, format)
}

/// Relative changes in a formatter's fixed-work throughput of mean/median
/// measurement. Original sampling units are resampled first; the formatter then
/// transforms each statistic pair on one shared scale. This is not the mean of
/// per-observation rates and makes no reciprocal/linear assumption about callbacks.
pub fn relative_throughput(
    baseline: &crate::Run,
    candidate: &crate::Run,
    case: &str,
    metric: &str,
    formatter: &dyn ValueFormatter,
    config: &crate::bootstrap::Config,
) -> Result<Vec<crate::relative::ThroughputChange>> {
    config.validate()?;
    crate::analysis::validate_comparison(baseline, Some(candidate), 0., 0.05)?;
    let case = candidate
        .cases
        .iter()
        .find(|c| c.id == case)
        .ok_or_else(|| crate::error("unknown comparison case"))?;
    let metric = case
        .metrics
        .iter()
        .find(|m| m.id == metric)
        .ok_or_else(|| crate::error("unknown comparison metric"))?;
    let samples = crate::relative::comparison_samples(baseline, candidate, &case.id, metric);
    let mut result = Vec::new();
    for (unit, count) in crate::report::work_counters(case)? {
        let mut output_unit = None;
        let report = match (count, &samples) {
            (Some(work), Ok((before, after, _))) if work > 0 => {
                let typical = before.iter().chain(after).copied().fold(0., f64::max);
                crate::relative::bootstrap_by(before, after, config, |before, after, scale| {
                    let rates = formatter.scale_throughputs(
                        typical,
                        work as f64,
                        unit,
                        &[before * scale, after * scale],
                    )?;
                    if rates.values.len() != 2 || rates.unit.is_empty() {
                        return Err(crate::error(
                            "relative throughput formatter returned invalid count or unit",
                        ));
                    }
                    if output_unit.as_ref().is_some_and(|unit| *unit != rates.unit) {
                        return Err(crate::error(
                            "relative throughput formatter changed units across resamples",
                        ));
                    }
                    output_unit = Some(rates.unit);
                    let before = rates.values[0];
                    let after = rates.values[1];
                    if !before.is_finite() || !after.is_finite() || before <= 0. || after < 0. {
                        return Ok(None);
                    }
                    let change = (after / before - 1.) * 100.;
                    Ok(change.is_finite().then_some(change))
                })
            }
            (None, _) => Err(crate::error(
                "Dynamic work requires a paired throughput comparison",
            )),
            (Some(0), _) => Err(crate::error(
                "Zero work has no defined relative throughput change",
            )),
            (_, Err(reason)) => Err(crate::error(reason.to_string())),
            _ => unreachable!(),
        };
        let method = Some("Fixed work: formatter applied to each resampled mean/median measurement pair on a shared scale; percentile interval of resulting rate changes, not a mean of per-observation rates. Positive means higher throughput.".into());
        for (name, statistic) in [
            ("mean", report.as_ref().ok().map(|r| &r.mean)),
            ("median", report.as_ref().ok().map(|r| &r.median)),
        ] {
            result.push(crate::relative::ThroughputChange {
                method: method.clone(),
                unit: output_unit.clone().unwrap_or_else(|| unit.to_owned()),
                statistic: name.into(),
                point_percent: statistic.and_then(|s| s.point_percent),
                interval_percent: statistic.and_then(|s| s.interval_percent),
                unavailable_reason: match &report {
                    Err(reason) => Some(reason.to_string()),
                    Ok(_) => statistic.and_then(|s| s.unavailable_reason.clone()),
                },
            });
        }
    }
    Ok(result)
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
        comparison_rows_chart(&format!("{case} / {metric} — formatted comparison"), &rows)?;
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
        )?);
    }
    Ok(output)
}

fn comparison_rows_chart(title: &str, rows: &[FormattedObservation]) -> Result<String> {
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
    let mut output = crate::viz::charts::scatter(
        &format!("{title} ({unit})"),
        &series,
        1000,
        "One formatter scale selected from both raw populations. X is sequence within each process, not a paired measurement. Missing values omitted; at most 1000 points per series.",
    );
    output.push_str(&formatted_distribution(title, unit, &series)?);
    Ok(output)
}

/// Compare original process fits through one shared formatter invocation per variant.
/// Callers validate run comparability before analysis; metric contracts are checked
/// here as well. Process identities include their side, so matching IDs stay separate.
pub fn regression_comparison_chart(
    baseline: &crate::bootstrap::Report,
    candidate: &crate::bootstrap::Report,
    case: &str,
    metric: &Metric,
    formatter: &dyn ValueFormatter,
) -> Result<String> {
    let mut output = String::new();
    for (title, presentation) in
        regression_comparison_presentations(baseline, candidate, case, metric, formatter)?
    {
        output.push_str(&format!("<p>Original slope confidence levels: baseline {}, candidate {}. Each curve transforms its own process fit and confidence bounds.</p>",
            baseline.config_for_case(case).confidence_level, candidate.config_for_case(case).confidence_level));
        output.push_str(&presentation.figure(&title)?);
    }
    Ok(output)
}

pub(crate) fn regression_comparison_presentations(
    baseline: &crate::bootstrap::Report,
    candidate: &crate::bootstrap::Report,
    case: &str,
    metric: &Metric,
    formatter: &dyn ValueFormatter,
) -> Result<Vec<(String, crate::regression::ComparisonPresentation)>> {
    let mut output = Vec::new();
    for row in candidate
        .rows
        .iter()
        .filter(|r| r.case == case && r.metric == metric.id)
    {
        let Some(old) = baseline
            .rows
            .iter()
            .find(|r| r.case == case && r.metric == metric.id && r.variant == row.variant)
        else {
            continue;
        };
        if old.metric_contract.as_ref() != Some(metric)
            || row.metric_contract.as_ref() != Some(metric)
            || old.unit != metric.unit
            || row.unit != metric.unit
        {
            return Err(crate::error(
                "regression comparison metric contracts differ",
            ));
        }
        if old.regressions.is_empty() || row.regressions.is_empty() {
            continue;
        }
        let labeled: Vec<_> = [("baseline", old), ("candidate", row)]
            .into_iter()
            .flat_map(|(side, row)| {
                row.regressions
                    .iter()
                    .map(move |r| (format!("{side} process {}", r.process), r))
            })
            .collect();
        let series: Vec<_> = labeled
            .iter()
            .map(|(label, r)| crate::regression::Series {
                label,
                samples: &r.samples,
                fit: &r.fit,
            })
            .collect();
        output.push((
            format!(
                "{case} / {} / {} — formatted regression comparison",
                metric.id, row.variant
            ),
            crate::regression::ComparisonPresentation::new(&series, formatter)?,
        ));
    }
    Ok(output)
}

/// Attach saved display coordinates to the selected metric's regression plots.
/// All process curves share a formatter scale. Tables, slope distributions and
/// raw estimates retain their original units; only regression geometry changes.
/// The result can be serialized and rendered with `bootstrap::html` after reload.
pub fn format_regression_metric(
    report: &crate::bootstrap::Report,
    case: &str,
    metric: &Metric,
    formatter: &dyn ValueFormatter,
) -> Result<crate::bootstrap::Report> {
    let mut result = report.clone();
    result
        .rows
        .retain(|row| row.case == case && row.metric == metric.id);
    result
        .presentation
        .retain(|entry| entry.case == case && entry.metric == metric.id);
    if result
        .rows
        .iter()
        .any(|row| row.metric_contract.as_ref() != Some(metric))
    {
        return Err(crate::error(
            "formatter metric contract differs from bootstrap report",
        ));
    }
    let series: Vec<_> = result
        .rows
        .iter()
        .flat_map(|row| row.regressions.iter())
        .map(|r| crate::regression::Series {
            label: "process",
            samples: &r.samples,
            fit: &r.fit,
        })
        .collect();
    if series.is_empty() {
        return Ok(result);
    }
    let presentations = crate::regression::format_series(&series, formatter)?;
    for (regression, presentation) in result
        .rows
        .iter_mut()
        .flat_map(|row| &mut row.regressions)
        .zip(presentations)
    {
        regression.presentation = Some(presentation);
    }
    Ok(result)
}

/// Format absolute bootstrap estimates and retained draws on one shared scale.
/// Linear conversions change estimate, outlier and regression units together.
/// Increasing nonlinear conversions attach a separate display of each statistic
/// and regression model; raw estimates and classifications keep original units.
/// The original report is unchanged.
pub fn format_bootstrap_metric(
    report: &crate::bootstrap::Report,
    case: &str,
    metric: &Metric,
    formatter: &dyn ValueFormatter,
) -> Result<crate::bootstrap::Report> {
    let mut result = report.clone();
    result.presentation.clear();
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
    let raw_rows = result.rows.clone();
    let typical = result
        .rows
        .iter()
        .filter_map(|row| row.estimates.as_ref())
        .map(|estimates| estimates.mean.point)
        .fold(0.0f64, f64::max);
    let mut references = Vec::new();
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
                ]);
                references.extend(estimate.standard_error.iter_mut());
            }
            references.extend([&mut estimates.minimum, &mut estimates.maximum]);
        }
        for regression in &mut row.regressions {
            // These coordinates were bound to the previous units and fit.
            regression.presentation = None;
            let slope = &mut regression.fit.slope;
            references.extend([&mut slope.point, &mut slope.lower, &mut slope.upper]);
            references.extend(slope.standard_error.iter_mut());
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
    let mut values: Vec<_> = references.iter().map(|value| **value).collect();
    if values.is_empty() {
        for (row, raw) in result.rows.iter_mut().zip(&raw_rows) {
            row.throughput = format_bootstrap_rates(raw, typical, formatter)?;
        }
        return Ok(result);
    }
    // Constant samples alone cannot distinguish a unit conversion from a curve.
    // Probe intermediate totals as well; formatter order preservation remains a
    // contract between evaluated points for arbitrary user functions.
    let anchor = values.iter().copied().fold(0.0f64, f64::max);
    values.extend([0., anchor * 0.5, anchor * 0.25]);
    let scaled = formatter.scale_values(typical, &values)?;
    if scaled.values.len() != values.len() || scaled.unit.trim().is_empty() {
        return Err(crate::error("invalid bootstrap formatter output"));
    }
    // Only a linear change of units may replace the numerical statistics.
    // Other monotone transforms get a separate, explicitly labelled display.
    let linear = if let Some((anchor, converted)) = values
        .iter()
        .zip(&scaled.values)
        .filter(|(value, _)| **value > 0.0)
        .max_by(|a, b| a.0.total_cmp(b.0))
    {
        *converted > 0.0
            && values.iter().zip(&scaled.values).all(|(raw, display)| {
                if *raw == 0.0 {
                    return *display == 0.0;
                }
                let expected = raw / anchor;
                let actual = display / converted;
                actual.is_finite() && (actual - expected).abs() <= 1e-12 * expected.abs()
            })
    } else {
        scaled.values.iter().all(|value| *value == 0.0)
    };
    if !linear {
        drop(references);
        result.rows = raw_rows;
        for row in &mut result.rows {
            row.throughput = format_bootstrap_rates(row, typical, formatter)?;
        }
        let result = crate::statistic_format::format(result, formatter)?;
        return format_regression_metric(&result, case, metric, formatter);
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
    for (row, raw) in result.rows.iter_mut().zip(&raw_rows) {
        row.throughput = format_bootstrap_rates(raw, typical, formatter)?;
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

fn format_bootstrap_rates(
    row: &crate::bootstrap::Row,
    typical: f64,
    formatter: &dyn ValueFormatter,
) -> Result<Vec<crate::bootstrap::ThroughputEstimate>> {
    let mut output = Vec::new();
    for (unit, count) in &row.work_counters {
        let mut estimates = vec![
            (
                "inverse mean measurement".to_owned(),
                row.estimates.as_ref().map(|e| &e.mean),
            ),
            (
                "inverse median measurement".to_owned(),
                row.estimates.as_ref().map(|e| &e.median),
            ),
        ];
        estimates.extend(row.regressions.iter().map(|r| {
            (
                format!("inverse slope (process {})", r.process),
                Some(&r.fit.slope),
            )
        }));
        for (statistic, estimate) in estimates {
            let mut rate = crate::bootstrap::ThroughputEstimate {
                statistic,
                unit: unit.clone(),
                values: None,
                display: None,
                unavailable_reason: None,
            };
            match (count, estimate) {
                (Some(work), Some(estimate))
                    if [estimate.point, estimate.lower, estimate.upper]
                        .iter()
                        .all(|v| v.is_finite() && *v > 0.) =>
                {
                    let scaled = formatter.scale_throughputs(
                        typical,
                        *work as f64,
                        unit,
                        &[estimate.point, estimate.upper, estimate.lower],
                    )?;
                    if scaled.values.len() != 3
                        || scaled.unit.is_empty()
                        || scaled.values.iter().any(|v| !v.is_finite() || *v < 0.)
                        || scaled.values[1] > scaled.values[2]
                    {
                        return Err(crate::error(
                            "invalid bootstrap throughput formatter output: expected three finite nonnegative values and ordered bounds",
                        ));
                    }
                    let values = [scaled.values[0], scaled.values[1], scaled.values[2]];
                    let mut display = values.map(|v| v.to_string());
                    let mut custom = false;
                    for (index, value) in values.iter().enumerate() {
                        if let Some(text) = formatter.format_throughput(*value, &scaled.unit)? {
                            display[index] = text;
                            custom = true;
                        }
                    }
                    rate.values = Some(values);
                    rate.display = custom.then_some(display);
                    rate.unit = scaled.unit;
                }
                (None, _) => {
                    rate.unavailable_reason =
                        Some("Dynamic work requires paired throughput analysis.".into())
                }
                _ => {
                    rate.unavailable_reason = Some(
                        "Throughput requires a positive finite measurement estimate and bounds."
                            .into(),
                    )
                }
            }
            output.push(rate);
        }
    }
    Ok(output)
}
