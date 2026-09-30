//! Descriptive estimates for input-size summary charts.
use crate::{Result, Run, analysis};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Descriptive chart estimator; comparison decisions are unaffected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Estimator {
    #[default]
    ProcessMedian,
    Mean,
}

/// Arithmetic mean of normalized observations, as used by Criterion's input
/// summary lines. Batches carry equal weight; this is not a process-level test.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeanEstimate {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub unit: String,
    pub observations: usize,
    pub value: Option<f64>,
    pub unavailable_reason: Option<String>,
}

/// Complete populations only. Normalize batch totals before averaging, rather
/// than dividing the sum of totals by the sum of operation counts.
pub fn mean_estimates(run: &Run) -> Result<Vec<MeanEstimate>> {
    run.validate()?;
    let variants: BTreeSet<_> = run
        .observations
        .iter()
        .map(|o| o.variant.as_str())
        .collect();
    let mut rows = Vec::new();
    for case in &run.cases {
        for metric in &case.metrics {
            for variant in &variants {
                let observations: Vec<_> = run
                    .observations
                    .iter()
                    .filter(|o| o.case == case.id && o.metric == metric.id && o.variant == *variant)
                    .collect();
                let mut row = MeanEstimate {
                    case: case.id.clone(),
                    metric: metric.id.clone(),
                    variant: (*variant).into(),
                    unit: metric.unit.clone(),
                    observations: observations.len(),
                    value: None,
                    unavailable_reason: None,
                };
                if let Err(err) = analysis::values(run, &case.id, metric, variant) {
                    row.unavailable_reason = Some(err.to_string());
                } else {
                    let values: Vec<_> = observations
                        .iter()
                        .map(|o| {
                            let value = o
                                .number()?
                                .ok_or_else(|| crate::error("missing numeric value"))?;
                            Ok(if metric.statistic == "batch_total" {
                                value / o.operations as f64
                            } else {
                                value
                            })
                        })
                        .collect::<Result<_>>()?;
                    let scale = values.iter().copied().fold(0., f64::max);
                    row.value = Some(if scale == 0. {
                        0.
                    } else {
                        (values.iter().map(|v| v / scale).sum::<f64>() / values.len() as f64)
                            * scale
                    });
                }
                rows.push(row);
            }
        }
    }
    Ok(rows)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeanThroughput {
    pub case: String,
    pub variant: String,
    pub unit: String,
    pub mean_work_per_operation: Option<f64>,
    pub mean_time_ns: Option<f64>,
    pub per_second: Option<f64>,
    pub unavailable_reason: Option<String>,
}

/// Fixed work / mean normalized time matches Criterion summary throughput.
/// For varying inputs, use mean normalized work / mean normalized time; this
/// extension is a ratio of means, never a mean of batch rates. Bytes use MiB/s.
pub fn mean_throughput(run: &Run) -> Result<Vec<MeanThroughput>> {
    let times = mean_estimates(run)?;
    let mut result = Vec::new();
    for time in times.iter().filter(|row| row.metric == "wall") {
        let case = run.cases.iter().find(|c| c.id == time.case).unwrap();
        let metric = case.metrics.iter().find(|m| m.id == "wall").unwrap();
        for (unit, fixed) in crate::report::work_counters(case)? {
            let scale = if unit == "bytes" { 1_048_576. } else { 1. };
            let mut row = MeanThroughput {
                case: time.case.clone(),
                variant: time.variant.clone(),
                unit: if unit == "bytes" {
                    "MiB".into()
                } else {
                    unit.into()
                },
                mean_work_per_operation: None,
                mean_time_ns: time.value,
                per_second: None,
                unavailable_reason: time.unavailable_reason.clone(),
            };
            if metric.unit != "ns" || metric.statistic != "batch_total" {
                row.mean_time_ns = None;
                row.unavailable_reason =
                    Some("Throughput requires wall batch totals in nanoseconds".into());
            }
            if row.unavailable_reason.is_none() {
                let values: Vec<_> = run
                    .observations
                    .iter()
                    .filter(|o| {
                        o.case == time.case && o.metric == "wall" && o.variant == time.variant
                    })
                    .map(|o| {
                        Ok(match fixed {
                            Some(value) => value as f64 / scale,
                            None => {
                                o.work_totals[unit].parse::<u128>()? as f64
                                    / o.operations as f64
                                    / scale
                            }
                        })
                    })
                    .collect::<Result<_>>()?;
                let maximum = values.iter().copied().fold(0., f64::max);
                let work = if maximum == 0. {
                    0.
                } else {
                    values.iter().map(|v| v / maximum).sum::<f64>() / values.len() as f64 * maximum
                };
                row.mean_work_per_operation = Some(work);
                let duration = time.value.unwrap();
                if duration <= 0. {
                    row.unavailable_reason = Some("Mean normalized duration is zero".into());
                } else {
                    let rate = work / duration * 1e9;
                    if rate.is_finite() {
                        row.per_second = Some(rate);
                    } else {
                        row.unavailable_reason =
                            Some("Throughput exceeds finite numeric range".into());
                    }
                }
            }
            result.push(row);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arithmetic_mean_uses_normalized_batches_and_preserves_missing_populations() {
        let mut rec = crate::Recorder::new();
        rec.case(crate::Case {
            id: "case".into(),
            contract: Default::default(),
            metrics: vec![crate::Metric::duration("wall", "operation", "batch_total")],
        })
        .unwrap();
        rec.observe("case", "wall", 1).unwrap();
        let mut run = rec.finish().unwrap();
        let prototype = run.observations[0].clone();
        run.observations = [(1, 2), (10, 40), (100, 1200)]
            .into_iter()
            .enumerate()
            .map(|(i, (operations, total))| {
                let mut o = prototype.clone();
                o.sequence = i as u64;
                o.operations = operations;
                o.value = Some(total.to_string());
                o
            })
            .collect();
        let rows = mean_estimates(&run).unwrap();
        assert_eq!(rows[0].value, Some(6.)); // mean([2,4,12]), not median 4 or weighted 1242/111.
        assert_eq!(rows[0].observations, 3);
        run.cases[0]
            .contract
            .insert("work.counter.items".into(), "2".into());
        run.cases[0]
            .contract
            .insert("work.counter.zero".into(), "0".into());
        run.cases[0]
            .contract
            .insert("work.input.bytes".into(), "batch_total".into());
        for (o, work) in run.observations.iter_mut().zip([1u64, 1, 10]) {
            o.work_totals.insert(
                "bytes".into(),
                (work * o.operations * 1_048_576).to_string(),
            );
        }
        let rates = mean_throughput(&run).unwrap();
        let items = rates.iter().find(|r| r.unit == "items").unwrap();
        assert_eq!(items.mean_time_ns, Some(6.));
        assert_eq!(items.mean_work_per_operation, Some(2.));
        assert!((items.per_second.unwrap() - 1e9 / 3.).abs() < 1e-6);
        let bytes = rates.iter().find(|r| r.unit == "MiB").unwrap();
        assert!((bytes.mean_work_per_operation.unwrap() - 4.).abs() < 1e-12);
        assert!((bytes.per_second.unwrap() - 2e9 / 3.).abs() < 1e-6);
        assert_eq!(
            rates.iter().find(|r| r.unit == "zero").unwrap().per_second,
            Some(0.)
        );
        let mut zeros = run.clone();
        for o in &mut zeros.observations {
            o.value = Some("0".into());
        }
        assert!(
            mean_throughput(&zeros)
                .unwrap()
                .iter()
                .all(|r| r.per_second.is_none())
        );
        let mut unavailable = prototype;
        unavailable.sequence = 3;
        unavailable.value = None;
        unavailable.availability = crate::Availability::Unsupported("fixture".into());
        run.observations.push(unavailable);
        let rows = mean_estimates(&run).unwrap();
        assert!(rows[0].value.is_none());
        assert!(rows[0].unavailable_reason.is_some());
        assert!(
            mean_throughput(&run)
                .unwrap()
                .iter()
                .all(|r| r.per_second.is_none())
        );
    }
}
