//! Saved display transformations of statistical estimates, separate from raw statistics.
use crate::{
    Result,
    bootstrap::{Estimate, Report, Row},
    measurement::ValueFormatter,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntervalMethod {
    #[default]
    TransformedBounds,
    TransformedBootstrapPercentiles,
}
impl IntervalMethod {
    fn label(self) -> &'static str {
        match self {
            Self::TransformedBounds => "transformed original bounds",
            Self::TransformedBootstrapPercentiles => "percentiles of transformed bootstrap draws",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayStatistic {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub statistic: String,
    pub unit: String,
    pub estimate: Estimate,
    #[serde(default)]
    pub interval_method: IntervalMethod,
    pub draws: Vec<f64>,
    source_sha256: String,
}

fn source_hash(report: &Report, row: &Row) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut source = row.clone();
    for regression in &mut source.regressions {
        regression.presentation = None;
    }
    Ok(crate::model::hex(&Sha256::digest(serde_json::to_vec(&(
        &source,
        report.config_for_case(&row.case),
    ))?)))
}

/// Attach display estimates without changing raw estimates or classifications.
/// Nonmonotone transforms use percentile bounds of retained or reproducible draws.
/// Decreasing transforms reverse the displayed interval endpoints.
/// Standard errors are recomputed from transformed draws, never transformed directly.
pub(crate) fn format(mut report: Report, formatter: &dyn ValueFormatter) -> Result<Report> {
    report.presentation.clear();
    let mut entries = Vec::new();
    let mut values = Vec::new();
    for row in &report.rows {
        let source_sha256 = source_hash(&report, row)?;
        let mut add = |name: String, estimate: &Estimate, draws: &[f64]| -> Result<()> {
            let start = values.len();
            values.extend([estimate.point, estimate.lower, estimate.upper]);
            values.extend_from_slice(draws);
            entries.push((
                DisplayStatistic {
                    case: row.case.clone(),
                    metric: row.metric.clone(),
                    variant: row.variant.clone(),
                    statistic: name,
                    unit: String::new(),
                    estimate: estimate.clone(),
                    interval_method: IntervalMethod::TransformedBounds,
                    draws: Vec::new(),
                    source_sha256: source_sha256.clone(),
                },
                start,
                values.len(),
            ));
            Ok(())
        };
        if let Some(e) = &row.estimates {
            let d = row.distributions.as_ref();
            for (name, estimate, draws) in [
                ("mean", &e.mean, d.map(|d| d.mean.as_slice()).unwrap_or(&[])),
                (
                    "median",
                    &e.median,
                    d.map(|d| d.median.as_slice()).unwrap_or(&[]),
                ),
                (
                    "standard deviation",
                    &e.standard_deviation,
                    d.map(|d| d.standard_deviation.as_slice()).unwrap_or(&[]),
                ),
                (
                    "scaled MAD",
                    &e.median_absolute_deviation,
                    d.map(|d| d.median_absolute_deviation.as_slice())
                        .unwrap_or(&[]),
                ),
            ] {
                add(name.into(), estimate, draws)?;
            }
        }
        for r in &row.regressions {
            add(
                format!("slope (process {})", r.process),
                &r.fit.slope,
                r.slope_distribution.as_deref().unwrap_or(&[]),
            )?;
        }
    }
    if values.is_empty() {
        return Ok(report);
    }
    let typical = values.iter().map(|v| v.abs()).fold(0., f64::max);
    let scaled = formatter.scale_values(typical, &values)?;
    if scaled.values.len() != values.len()
        || scaled.unit.trim().is_empty()
        || values.iter().chain(&scaled.values).any(|v| !v.is_finite())
    {
        return Err(crate::error("invalid statistical display formatter output"));
    }
    let mut order: Vec<_> = values
        .iter()
        .copied()
        .zip(scaled.values.iter().copied())
        .collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0));
    let increasing = order.windows(2).all(|w| w[0].1 <= w[1].1);
    let decreasing = order.windows(2).all(|w| w[0].1 >= w[1].1);
    if order
        .windows(2)
        .any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1)
        || (order[0].0 != order[order.len() - 1].0 && order.iter().all(|p| p.1 == order[0].1))
    {
        return Err(crate::error(
            "statistical display formatter must preserve equal values and not collapse the range",
        ));
    }
    if !increasing && !decreasing && entries.iter().any(|(_, start, end)| end - start == 3) {
        let configs: Vec<_> = report
            .rows
            .iter()
            .map(|row| report.config_for_case(&row.case).clone())
            .collect();
        for (row, config) in report.rows.iter_mut().zip(configs) {
            if let Some(estimates) = &row.estimates {
                if row.distributions.is_none() {
                    let points = &row
                        .outliers
                        .as_ref()
                        .ok_or_else(|| {
                            crate::error(
                                "cannot reproduce bootstrap draws: source observations are missing",
                            )
                        })?
                        .points;
                    if points.len() != row.units
                        || points.iter().enumerate().any(|(i, p)| p.index != i)
                    {
                        return Err(crate::error(
                            "cannot reproduce bootstrap draws: incomplete source observations",
                        ));
                    }
                    let values: Vec<_> = points.iter().map(|p| p.value).collect();
                    let regenerated =
                        crate::bootstrap::estimate_with_distributions(&values, &config)?;
                    if &regenerated.estimates != estimates {
                        return Err(crate::error(
                            "cannot reproduce bootstrap draws: source estimates do not match observations and settings",
                        ));
                    }
                    row.distributions = Some(regenerated.distributions);
                }
            }
            for regression in &mut row.regressions {
                if regression.slope_distribution.is_none() {
                    let (fit, draws) =
                        crate::regression::fit_with_distribution(&regression.samples, &config)?;
                    if fit.slope != regression.fit.slope {
                        return Err(crate::error(
                            "cannot reproduce bootstrap draws: source slope does not match samples and settings",
                        ));
                    }
                    regression.slope_distribution = Some(draws);
                }
            }
        }
        // Re-evaluate the shared display scale with the complete draw population.
        if report.rows.iter().any(|row| {
            row.distributions.as_ref().is_some_and(|d| {
                d.mean.is_empty()
                    || d.median.is_empty()
                    || d.standard_deviation.is_empty()
                    || d.median_absolute_deviation.is_empty()
            }) || row
                .regressions
                .iter()
                .any(|r| r.slope_distribution.as_ref().is_some_and(Vec::is_empty))
        }) {
            return Err(crate::error(
                "cannot reproduce bootstrap draws: empty retained distribution",
            ));
        }
        return format(report, formatter);
    }
    for (mut entry, start, end) in entries {
        let transformed = &scaled.values[start..end];
        entry.unit.clone_from(&scaled.unit);
        let (lower, upper) = if increasing || decreasing {
            (
                transformed[1].min(transformed[2]),
                transformed[1].max(transformed[2]),
            )
        } else {
            if transformed.len() == 3 {
                return Err(crate::error(
                    "nonmonotone statistical display requires retained bootstrap draws",
                ));
            }
            let mut draws = transformed[3..].to_vec();
            draws.sort_by(f64::total_cmp);
            let config = report.config_for_case(&entry.case);
            config.validate()?;
            let tail = (1. - config.confidence_level) / 2.;
            entry.interval_method = IntervalMethod::TransformedBootstrapPercentiles;
            (
                crate::bootstrap::quantile(&draws, tail),
                crate::bootstrap::quantile(&draws, 1. - tail),
            )
        };

        entry.estimate = Estimate {
            point: transformed[0],
            lower,
            upper,
            standard_error: if transformed.len() > 4 {
                crate::bootstrap::describe(&transformed[3..])?.standard_deviation
            } else {
                None
            },
        };
        if entry.estimate.lower > entry.estimate.upper {
            return Err(crate::error("invalid statistical display interval"));
        }
        entry.draws.extend_from_slice(&transformed[3..]);
        report.presentation.push(entry);
    }
    Ok(report)
}

impl DisplayStatistic {
    fn validate(&self, report: &Report) -> Result<()> {
        let row = report
            .rows
            .iter()
            .find(|r| r.case == self.case && r.metric == self.metric && r.variant == self.variant)
            .ok_or_else(|| crate::error("saved statistical display has no source row"))?;
        if source_hash(report, row)? != self.source_sha256 {
            return Err(crate::error(
                "saved statistical display does not match source data or settings",
            ));
        }
        let e = &self.estimate;
        if self.unit.trim().is_empty()
            || e.lower > e.upper
            || [e.point, e.lower, e.upper]
                .iter()
                .chain(e.standard_error.iter())
                .chain(&self.draws)
                .any(|v| !v.is_finite())
            || e.standard_error.is_some_and(|v| v < 0.)
        {
            return Err(crate::error("invalid saved statistical display values"));
        }
        Ok(())
    }
}

pub(crate) fn markdown(report: &Report) -> String {
    if report.presentation.is_empty() {
        return String::new();
    }
    use crate::report::escape;
    let mut out = String::from(
        "\n## Transformed statistical display\n\nEach row is formatter(original statistic), not a statistic recomputed from transformed observations. Monotone intervals transform original bounds; nonmonotone intervals use percentiles of transformed bootstrap draws. Each row identifies its interval method. Standard error is the sample deviation of transformed bootstrap draws; unavailable when draws were not retained. Original statistics above retain their original units.\n\n| Case / metric / variant | Transformation | Estimate | Interval | Standard error | Unit |\n|---|---|---:|---|---:|---|\n",
    );
    for entry in &report.presentation {
        let label = escape(&format!(
            "{} / {} / {}",
            entry.case, entry.metric, entry.variant
        ));
        match entry.validate(report) {
            Ok(()) => out.push_str(&format!(
                "| {label} | formatter({}) | {:.6} | [{:.6}, {:.6}] ({}) | {} | {} |\n",
                escape(&entry.statistic),
                entry.estimate.point,
                entry.estimate.lower,
                entry.estimate.upper,
                entry.interval_method.label(),
                entry
                    .estimate
                    .standard_error
                    .map(|v| format!("{v:.6}"))
                    .unwrap_or_else(|| "unavailable".into()),
                escape(&entry.unit)
            )),
            Err(err) => out.push_str(&format!(
                "| {label} | formatter({}) | statistical display unavailable: {} | — | — | {} |\n",
                escape(&entry.statistic),
                escape(&err.to_string()),
                escape(&entry.unit)
            )),
        }
    }
    out
}

pub(crate) fn charts(report: &Report) -> String {
    let mut out = String::new();
    for entry in &report.presentation {
        let title = format!(
            "{} / {} / {} — formatter({})",
            entry.case, entry.metric, entry.variant, entry.statistic
        );
        let chart = entry.validate(report).and_then(|()| {
            if entry.draws.is_empty() {
                return Ok(String::new());
            }
            crate::density::estimate_figure(&entry.draws, &entry.estimate, &title, &entry.unit)
        });
        match chart {
            Ok(chart) => out.push_str(&chart),
            Err(err) => out.push_str(&format!(
                "<p>{}: statistical display unavailable: {}</p>",
                crate::report::escape(&title),
                crate::report::escape(&err.to_string())
            )),
        }
    }
    out
}

/// Compare transformed bootstrap distributions on one formatter-selected scale.
/// Both reports must retain bootstrap draws. Original statistics are unchanged.
pub fn comparison_charts(
    baseline: &Report,
    candidate: &Report,
    case: &str,
    metric: &crate::Metric,
    formatter: &dyn ValueFormatter,
) -> Result<String> {
    render_comparison(
        &comparison_report(baseline, candidate, case, metric, formatter)?,
        case,
        &metric.id,
    )
}

pub(crate) fn comparison_report(
    baseline: &Report,
    candidate: &Report,
    case: &str,
    metric: &crate::Metric,
    formatter: &dyn ValueFormatter,
) -> Result<Report> {
    let mut combined = baseline.clone();
    combined.rows.clear();
    combined.case_configs.clear();
    combined.presentation.clear();
    for (side, report) in [("baseline", baseline), ("candidate", candidate)] {
        let mut found = false;
        for row in report
            .rows
            .iter()
            .filter(|r| r.case == case && r.metric == metric.id)
        {
            if row.metric_contract.as_ref() != Some(metric) || row.unit != metric.unit {
                return Err(crate::error(
                    "statistical comparison metric contract mismatch",
                ));
            }
            if row.distributions.is_none() {
                return Err(crate::error(
                    "statistical comparison requires retained bootstrap draws",
                ));
            }
            found = true;
            let mut row = row.clone();
            row.case = side.into();
            combined.rows.push(row);
        }
        if !found {
            return Err(crate::error("statistical comparison case/metric absent"));
        }
        combined
            .case_configs
            .insert(side.into(), report.config_for_case(case).clone());
    }
    for old in combined.rows.iter().filter(|r| r.case == "baseline") {
        if let Some(new) = combined
            .rows
            .iter()
            .find(|r| r.case == "candidate" && r.variant == old.variant)
        {
            if old.resampling_unit != new.resampling_unit {
                return Err(crate::error(
                    "statistical comparison resampling populations differ",
                ));
            }
        }
    }
    // One call selects units from both populations; matching unit strings from
    // independently formatted reports would not establish a common scale.
    format(combined, formatter)
}

pub(crate) fn render_comparison(shown: &Report, case: &str, metric: &str) -> Result<String> {
    for entry in &shown.presentation {
        entry.validate(shown)?;
    }

    let mut out = String::new();
    for old in shown.presentation.iter().filter(|p| p.case == "baseline") {
        let Some(new) = shown.presentation.iter().find(|p| {
            p.case == "candidate" && p.variant == old.variant && p.statistic == old.statistic
        }) else {
            continue;
        };
        if old.unit != new.unit {
            return Err(crate::error("saved statistical comparison units differ"));
        }
        if old.draws.is_empty() || new.draws.is_empty() {
            continue;
        }
        out.push_str(&crate::density::figure(
            &[
                crate::density::Series {
                    label: "baseline",
                    values: &old.draws,
                },
                crate::density::Series {
                    label: "candidate",
                    values: &new.draws,
                },
            ],
            &format!(
                "{case} / {} / {} — formatter({}) comparison",
                metric, old.variant, old.statistic
            ),
            &old.unit,
        )?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::FormattedValues;
    use std::cell::Cell;

    struct AdaptiveSquare {
        calls: Cell<usize>,
    }
    impl ValueFormatter for AdaptiveSquare {
        fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues> {
            self.calls.set(self.calls.get() + 1);
            let divisor = if typical >= 1000. { 1000. } else { 1. };
            Ok(FormattedValues {
                values: values.iter().map(|v| (v / divisor).powi(2)).collect(),
                unit: if divisor == 1000. {
                    "squared kilo-units"
                } else {
                    "squared units"
                }
                .into(),
            })
        }
        fn scale_for_machines(&self, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
        fn scale_throughputs(&self, _: f64, _: f64, _: &str, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
    }

    #[test]
    fn comparison_uses_joint_scale_and_preserves_process_draws_and_settings() {
        let metric = crate::Metric::duration("wall", "batch", "batch_total");
        let fixture = |factor: u128, seed| {
            let mut recorder = crate::Recorder::new();
            recorder
                .case(crate::Case {
                    id: "case".into(),
                    contract: Default::default(),
                    metrics: vec![metric.clone()],
                })
                .unwrap();
            for value in [2, 4, 6] {
                recorder.observe("case", "wall", value * factor).unwrap();
            }
            let mut run = recorder.finish().unwrap();
            for (process, row) in run.observations.iter_mut().enumerate() {
                row.process = process as u32;
            }
            crate::bootstrap::analyze_with_distributions(
                &run,
                &crate::bootstrap::Config {
                    resamples: 64,
                    confidence_level: 0.9,
                    seed,
                },
            )
            .unwrap()
        };
        let before = fixture(1, 7);
        let after = fixture(1000, 13);
        assert_eq!(before.rows[0].resampling_unit, "process_median");
        let original = serde_json::to_vec(&(&before, &after)).unwrap();
        let formatter = AdaptiveSquare {
            calls: Cell::new(0),
        };
        let shown = comparison_report(&before, &after, "case", &metric, &formatter).unwrap();
        assert_eq!(formatter.calls.get(), 1);
        for (side, source) in [("baseline", &before), ("candidate", &after)] {
            assert_eq!(shown.config_for_case(side).seed, source.config.seed);
            let entry = shown
                .presentation
                .iter()
                .find(|p| p.case == side && p.statistic == "mean")
                .unwrap();
            let raw = source.rows[0].estimates.as_ref().unwrap();
            assert_eq!(entry.unit, "squared kilo-units");
            assert_eq!(entry.estimate.point, (raw.mean.point / 1000.).powi(2));
            assert_eq!(entry.estimate.lower, (raw.mean.lower / 1000.).powi(2));
            assert_eq!(entry.estimate.upper, (raw.mean.upper / 1000.).powi(2));
            let expected: Vec<_> = source.rows[0]
                .distributions
                .as_ref()
                .unwrap()
                .mean
                .iter()
                .map(|v| (v / 1000.).powi(2))
                .collect();
            assert_eq!(entry.draws, expected);
            let mean = expected.iter().sum::<f64>() / expected.len() as f64;
            let standard_error = (expected.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / (expected.len() - 1) as f64)
                .sqrt();
            assert!(
                (entry.estimate.standard_error.unwrap() - standard_error).abs()
                    <= 1e-12 * standard_error.max(1.)
            );
        }
        let restored: Report =
            serde_json::from_slice(&serde_json::to_vec(&shown).unwrap()).unwrap();
        assert!(
            render_comparison(&restored, "case", "wall")
                .unwrap()
                .contains("squared kilo-units")
        );
        assert_eq!(formatter.calls.get(), 1);
        assert_eq!(serde_json::to_vec(&(&before, &after)).unwrap(), original);
    }
}

#[cfg(test)]
mod nonmonotone_tests {
    use super::*;
    use crate::measurement::FormattedValues;
    struct Fold;
    impl ValueFormatter for Fold {
        fn scale_values(&self, _: f64, values: &[f64]) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.iter().map(|v| (v - 4.).powi(2)).collect(),
                unit: "folded".into(),
            })
        }
        fn scale_intervals(
            &self,
            _: f64,
            intervals: &[[f64; 2]],
        ) -> Result<Option<crate::measurement::FormattedIntervals>> {
            Ok(Some(crate::measurement::FormattedIntervals {
                bounds: intervals
                    .iter()
                    .map(|&[a, b]| {
                        let left = (a - 4.).powi(2);
                        let right = (b - 4.).powi(2);
                        [
                            if a <= 4. && b >= 4. {
                                0.
                            } else {
                                left.min(right)
                            },
                            left.max(right),
                        ]
                    })
                    .collect(),
                unit: "folded".into(),
            }))
        }
        fn scale_for_machines(&self, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
        fn scale_throughputs(&self, _: f64, _: f64, _: &str, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
    }
    #[test]
    fn folded_interval_uses_draw_quantiles_instead_of_endpoint_images() {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "case".into(),
                contract: Default::default(),
                metrics: vec![crate::Metric::duration("wall", "batch", "batch_total")],
            })
            .unwrap();
        for value in [2, 4, 6] {
            recorder.observe("case", "wall", value).unwrap();
        }
        let run = recorder.finish().unwrap();
        let config = crate::bootstrap::Config {
            resamples: 256,
            confidence_level: 0.9,
            seed: 11,
        };
        let original = crate::bootstrap::analyze_with_distributions(&run, &config).unwrap();
        let source_bytes = serde_json::to_vec(&original).unwrap();
        let displayed = crate::measurement::format_bootstrap_metric(
            &original,
            "case",
            &run.cases[0].metrics[0],
            &Fold,
        )
        .unwrap();
        let entry = displayed
            .presentation
            .iter()
            .find(|p| p.statistic == "mean")
            .unwrap();
        let raw = original.rows[0].estimates.as_ref().unwrap();
        assert_eq!(
            entry.interval_method,
            IntervalMethod::TransformedBootstrapPercentiles
        );
        assert_eq!(entry.estimate.point, 0.);
        let mut expected: Vec<_> = original.rows[0]
            .distributions
            .as_ref()
            .unwrap()
            .mean
            .iter()
            .map(|v| (v - 4.).powi(2))
            .collect();
        assert_eq!(entry.draws, expected);
        expected.sort_by(f64::total_cmp);
        for (actual, probability) in [(entry.estimate.lower, 0.05), (entry.estimate.upper, 0.95)] {
            let position = probability * (expected.len() - 1) as f64;
            let lo = position.floor() as usize;
            let hi = position.ceil() as usize;
            let oracle = expected[lo] + (expected[hi] - expected[lo]) * position.fract();
            assert!((actual - oracle).abs() < 1e-12);
        }
        assert!(entry.estimate.lower < (raw.mean.lower - 4.).powi(2));
        let restored: Report =
            serde_json::from_slice(&serde_json::to_vec(&displayed).unwrap()).unwrap();
        assert!(markdown(&restored).contains("percentiles of transformed bootstrap draws"));
        assert_eq!(serde_json::to_vec(&original).unwrap(), source_bytes);
        let without_draws = crate::bootstrap::analyze(&run, &config).unwrap();
        let source = serde_json::to_vec(&without_draws).unwrap();
        let loaded: Report = serde_json::from_slice(&source).unwrap();
        let regenerated = crate::measurement::format_bootstrap_metric(
            &loaded,
            "case",
            &run.cases[0].metrics[0],
            &Fold,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_vec(&regenerated).unwrap(),
            serde_json::to_vec(&displayed).unwrap()
        );
        assert_eq!(serde_json::to_vec(&without_draws).unwrap(), source);
        let samples = vec![
            crate::regression::Sample {
                operations: 1.,
                total: 2.,
            },
            crate::regression::Sample {
                operations: 2.,
                total: 8.,
            },
            crate::regression::Sample {
                operations: 3.,
                total: 18.,
            },
        ];
        let (fit, draws) = crate::regression::fit_with_distribution(&samples, &config).unwrap();
        let mut with_slope = without_draws.clone();
        with_slope.rows[0]
            .regressions
            .push(crate::bootstrap::ProcessRegression {
                process: 0,
                presentation: None,
                fit,
                slope_distribution: None,
                samples,
            });
        let with_slope: Report =
            serde_json::from_slice(&serde_json::to_vec(&with_slope).unwrap()).unwrap();
        let mut retained = with_slope.clone();
        retained.rows[0].distributions = original.rows[0].distributions.clone();
        retained.rows[0].regressions[0].slope_distribution = Some(draws);
        assert_eq!(
            serde_json::to_vec(
                &crate::measurement::format_bootstrap_metric(
                    &with_slope,
                    "case",
                    &run.cases[0].metrics[0],
                    &Fold
                )
                .unwrap()
            )
            .unwrap(),
            serde_json::to_vec(
                &crate::measurement::format_bootstrap_metric(
                    &retained,
                    "case",
                    &run.cases[0].metrics[0],
                    &Fold
                )
                .unwrap()
            )
            .unwrap()
        );
        let mut changed_settings = without_draws.clone();
        changed_settings.config.seed += 1;
        assert!(
            format(changed_settings, &Fold)
                .unwrap_err()
                .to_string()
                .contains("source estimates do not match")
        );
        let mut truncated = without_draws.clone();
        truncated.rows[0].outliers.as_mut().unwrap().points.pop();
        assert!(
            format(truncated, &Fold)
                .unwrap_err()
                .to_string()
                .contains("incomplete source observations")
        );
        let mut missing = without_draws.clone();
        missing.rows[0].outliers = None;
        assert!(
            format(missing, &Fold)
                .unwrap_err()
                .to_string()
                .contains("source observations are missing")
        );
        let mut inconsistent = without_draws;
        inconsistent.rows[0].outliers.as_mut().unwrap().points[0].value += 0.5;
        assert!(
            format(inconsistent, &Fold)
                .unwrap_err()
                .to_string()
                .contains("source estimates do not match")
        );
    }
}
