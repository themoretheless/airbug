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
    fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues>;
    fn scale_throughputs(
        &self,
        typical: f64,
        work: f64,
        work_unit: &str,
        values: &[f64],
    ) -> Result<FormattedValues>;
    fn scale_for_machines(&self, values: &[f64]) -> Result<FormattedValues>;
}

#[derive(Clone, Copy, Debug)]
pub enum Format<'a> {
    Human,
    Machine,
    Throughput { work: f64, unit: &'a str },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormattedObservation {
    pub variant: String,
    pub process: u32,
    pub sequence: u64,
    pub value: Option<f64>,
    pub unit: String,
    pub unavailable_reason: Option<String>,
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
            Ok(FormattedObservation {
                variant: o.variant.clone(),
                process: o.process,
                sequence: o.sequence,
                value: if available { converted.next() } else { None },
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
        "| Variant | Process | Sequence | Value | Unit | Availability |\n|---|---|---|---|---|---|\n",
    );
    for row in rows {
        result.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            crate::report::escape(&row.variant),
            row.process,
            row.sequence,
            row.value.map(|v| v.to_string()).unwrap_or_default(),
            crate::report::escape(&row.unit),
            crate::report::escape(row.unavailable_reason.as_deref().unwrap_or("available"))
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
    }
    output
}

const PRESENTATION_KEY: &str = "airbug.presentation.formatted.v1";
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedMetric {
    source_sha256: String,
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

/// Save presentation snapshots beside raw data, outside measurement contracts.
pub fn save_formatted(run: &mut crate::Run, metrics: &[FormattedMetric]) -> Result<()> {
    run.validate()?;
    let saved: Vec<_> = metrics
        .iter()
        .map(|metric| {
            Ok(SavedMetric {
                source_sha256: source_hash(run, &metric.case, &metric.metric)?,
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
        for rows in [&formatted.human, &formatted.machine] {
            if rows.len() != observations.len() {
                return Err(crate::error("saved formatting row count mismatch"));
            }
            for (row, observation) in rows.iter().zip(&observations) {
                if row.variant != observation.variant
                    || row.process != observation.process
                    || row.sequence != observation.sequence
                    || row.unit.is_empty()
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
