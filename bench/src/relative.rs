//! Independent-sample bootstrap of percentage changes in mean and median.
use crate::{Result, Seeded, bootstrap::Config, error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Statistic {
    pub point_percent: Option<f64>,
    /// Generation order. Undefined ratios are retained as null, never discarded.
    /// Empty when only summary estimates were retained (automatic history).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub draws_percent: Vec<Option<f64>>,
    pub interval_percent: Option<(f64, f64)>,
    pub undefined_draws: usize,
    pub unavailable_reason: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub config: Config,
    pub baseline_units: usize,
    pub candidate_units: usize,
    pub mean: Statistic,
    pub median: Statistic,
}
/// HTML figures for relative mean/median changes. Undefined ratios suppress the
/// entire chart: plotting only finite draws would silently condition the density.
pub fn charts(rows: &[Comparison], threshold_percent: f64) -> Result<String> {
    use crate::report::escape;
    if !threshold_percent.is_finite() || threshold_percent < 0. {
        return Err(error(
            "relative chart threshold must be finite and nonnegative",
        ));
    }
    let mut html = String::from("<section><h2>Relative bootstrap distributions</h2>");
    for row in rows {
        let label = format!(
            "{} / {} ({}, {})",
            row.case, row.metric, row.unit, row.resampling_unit
        );
        html.push_str(&format!("<h3>{}</h3>", escape(&label)));
        let Some(report) = &row.report else {
            html.push_str(&format!(
                "<p>Unavailable: {}</p>",
                escape(
                    row.unavailable_reason
                        .as_deref()
                        .unwrap_or("No relative report")
                )
            ));
            continue;
        };
        html.push_str(&format!("<p>Baseline: {} sampling units; candidate: {}. Confidence: {:.4}%; resamples: {}; seed: {}.</p>",
            report.baseline_units, report.candidate_units, report.config.confidence_level * 100., report.config.resamples, report.config.seed));
        if row.resampling_unit == "normalized_observation" {
            html.push_str("<p>Exploratory within-process comparison: batches may be autocorrelated. This interval does not establish between-process reproducibility.</p>");
        }
        if report.config.resamples == 1 {
            html.push_str("<p>One resample: interval endpoints reflect a single draw and do not establish sampling uncertainty.</p>");
        }
        for (name, stat) in [("mean", &report.mean), ("median", &report.median)] {
            let title = format!("{label}: relative {name} change");
            let values: Option<Vec<f64>> = stat.draws_percent.iter().copied().collect();
            match (stat.point_percent, stat.interval_percent, values) {
                (Some(point), Some((lower, upper)), Some(values))
                    if stat.unavailable_reason.is_none() =>
                {
                    let estimate = crate::bootstrap::Estimate {
                        point,
                        lower,
                        upper,
                        standard_error: None,
                    };
                    match crate::density::relative_figure(
                        &values,
                        &estimate,
                        &title,
                        threshold_percent,
                    ) {
                        Ok(chart) => html.push_str(&chart),
                        Err(err) => html.push_str(&format!(
                            "<p>{}: unavailable: {}</p>",
                            escape(&title),
                            escape(&err.to_string())
                        )),
                    }
                }
                _ => html.push_str(&format!(
                    "<p>{}: unavailable: {} ({} undefined draws).</p>",
                    escape(&title),
                    escape(
                        stat.unavailable_reason
                            .as_deref()
                            .unwrap_or("Incomplete relative distribution")
                    ),
                    stat.undefined_draws
                )),
            }
        }
    }
    html.push_str("</section>");
    Ok(html)
}

/// Text counterpart of the relative distribution plots. These intervals describe
/// individual statistics, not the family-adjusted regression decision.
pub fn markdown(rows: &[Comparison]) -> String {
    use crate::report::escape;
    let mut text = String::from(
        "## Relative bootstrap estimates\n\nChanges describe mean and median of the labelled sampling units. Intervals are per statistic, not family-adjusted decisions.\n\n| Case / metric | Statistic | Change % | Interval % | Baseline / candidate units |\n|---|---|---:|---|---|\n",
    );
    let mut notes = String::new();
    for row in rows {
        let label = escape(&format!("{} / {} ({})", row.case, row.metric, row.unit));
        for rate in &row.throughput {
            let point = rate
                .point_percent
                .map(|v| format!("{v:.6}"))
                .unwrap_or_else(|| "unavailable".into());
            let interval = rate
                .interval_percent
                .map(|(l, h)| format!("[{l:.6}, {h:.6}]"))
                .unwrap_or_else(|| "unavailable".into());
            text.push_str(&format!(
                "| {label} | throughput {} ({}) | {point} | {interval} | {} / {} |\n",
                escape(&rate.statistic),
                escape(&rate.unit),
                row.baseline_units,
                row.candidate_units
            ));
            if let Some(reason) = &rate.unavailable_reason {
                notes.push_str(&format!(
                    "\n- {label}, throughput {}: {}\n",
                    escape(&rate.unit),
                    escape(reason)
                ));
            }
        }
        for rate in &row.throughput {
            if let Some(method) = &rate.method {
                notes.push_str(&format!(
                    "\n- {label}, throughput {}: {}\n",
                    escape(&rate.unit),
                    escape(method)
                ));
            }
        }
        if row.throughput.iter().any(|rate| rate.method.is_none()) {
            notes.push_str(&format!("\n- {label}: throughput change assumes equal fixed work per operation and reverses duration-change interval bounds. Positive means higher throughput.\n"));
        }
        let Some(report) = &row.report else {
            notes.push_str(&format!(
                "\n- {label}: unavailable: {}.\n",
                escape(
                    row.unavailable_reason
                        .as_deref()
                        .unwrap_or("missing relative estimates")
                )
            ));
            continue;
        };
        notes.push_str(&format!(
            "\n- {label}: confidence {:.2}%; resamples {}; seed {}; sampling unit {}.\n",
            report.config.confidence_level * 100.,
            report.config.resamples,
            report.config.seed,
            escape(&row.resampling_unit)
        ));
        if row.resampling_unit == "normalized_observation" {
            notes.push_str("Exploratory within-process comparison: batches may be autocorrelated. This interval does not establish between-process reproducibility.\n");
        }
        if report.config.resamples == 1 {
            notes.push_str(
                "One resample: interval endpoints do not establish sampling uncertainty.\n",
            );
        }
        for (name, statistic) in [("mean", &report.mean), ("median", &report.median)] {
            let point = statistic
                .point_percent
                .map(|value| format!("{value:.6}"))
                .unwrap_or_else(|| "unavailable".into());
            let interval = statistic
                .interval_percent
                .map(|(low, high)| format!("[{low:.6}, {high:.6}]"))
                .unwrap_or_else(|| "unavailable".into());
            text.push_str(&format!(
                "| {label} | {name} | {point} | {interval} | {} / {} |\n",
                report.baseline_units, report.candidate_units
            ));
            if let Some(reason) = &statistic.unavailable_reason {
                notes.push_str(&format!(
                    "\n- {label}, {name}: {}; {} undefined draws retained.\n",
                    escape(reason),
                    statistic.undefined_draws
                ));
            }
        }
    }
    text.push_str(&notes);
    text
}

fn stats(values: &mut [f64]) -> [f64; 2] {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.sort_by(f64::total_cmp);
    [mean, crate::bootstrap::quantile(values, 0.5)]
}
fn change(before: f64, after: f64) -> Option<f64> {
    if before == 0. {
        return None;
    }
    let value = (after / before - 1.) * 100.;
    value.is_finite().then_some(value)
}
fn finish(point: Option<f64>, draws: Vec<Option<f64>>, confidence: f64) -> Statistic {
    let undefined = draws.iter().filter(|v| v.is_none()).count();
    let reason = if point.is_none() {
        Some("Original relative change has zero denominator or exceeds finite range".into())
    } else if undefined > 0 {
        Some(
            "Bootstrap includes undefined ratios; no conditional-on-finite interval is reported"
                .into(),
        )
    } else {
        None
    };
    let interval = if reason.is_none() {
        let mut sorted: Vec<_> = draws.iter().map(|v| v.unwrap()).collect();
        sorted.sort_by(f64::total_cmp);
        let tail = (1. - confidence) / 2.;
        Some((
            crate::bootstrap::quantile(&sorted, tail),
            crate::bootstrap::quantile(&sorted, 1. - tail),
        ))
    } else {
        None
    };
    Statistic {
        point_percent: point,
        draws_percent: draws,
        interval_percent: interval,
        undefined_draws: undefined,
        unavailable_reason: reason,
    }
}
/// Resample each population independently, preserving its own sample size.
/// Inputs must be independent statistical units; batch autocorrelation is not removed.
/// Percentage change is `(candidate / baseline - 1) * 100`.
pub fn bootstrap(baseline: &[f64], candidate: &[f64], config: &Config) -> Result<Report> {
    bootstrap_by(baseline, candidate, config, |before, after, _| {
        Ok(change(before, after))
    })
}

/// Resample original measurement units and transform each pair of mean/median
/// estimates. `scale` restores the shared normalization used for stable arithmetic.
pub(crate) fn bootstrap_by(
    baseline: &[f64],
    candidate: &[f64],
    config: &Config,
    mut transform: impl FnMut(f64, f64, f64) -> Result<Option<f64>>,
) -> Result<Report> {
    config.validate()?;
    if baseline.len() < 2
        || candidate.len() < 2
        || baseline.iter().chain(candidate).any(|v| !v.is_finite())
    {
        return Err(error(
            "relative bootstrap requires two finite units per population",
        ));
    }
    let scale = baseline
        .iter()
        .chain(candidate)
        .map(|v| v.abs())
        .fold(0., f64::max)
        .max(f64::MIN_POSITIVE);
    let a: Vec<_> = baseline.iter().map(|v| v / scale).collect();
    let b: Vec<_> = candidate.iter().map(|v| v / scale).collect();
    let original_a = stats(&mut a.clone());
    let original_b = stats(&mut b.clone());
    let mut means = crate::bootstrap::resample_buffer(config.resamples)?;
    let mut medians = crate::bootstrap::resample_buffer(config.resamples)?;
    let sample_stats = |rng: &mut Seeded, sample_a: &mut [f64], sample_b: &mut [f64]| {
        for value in sample_a.iter_mut() {
            *value = a[crate::bootstrap::draw_index(rng, a.len())];
        }
        for value in sample_b.iter_mut() {
            *value = b[crate::bootstrap::draw_index(rng, b.len())];
        }
        (stats(sample_a), stats(sample_b))
    };
    let workers =
        crate::parallel_analysis::workers(config.resamples, a.len().saturating_add(b.len()));
    if workers == 1 {
        let mut rng = Seeded::new(config.seed);
        let mut sample_a = vec![0.; a.len()];
        let mut sample_b = vec![0.; b.len()];
        for _ in 0..config.resamples {
            let (before, after) = sample_stats(&mut rng, &mut sample_a, &mut sample_b);
            means.push(transform(before[0], after[0], scale)?);
            medians.push(transform(before[1], after[1], scale)?);
        }
    } else {
        let chunks = crate::parallel_analysis::chunks(
            config.resamples,
            config.seed,
            &[a.len(), b.len()],
            workers,
            |count, mut rng| {
                let mut rows = crate::bootstrap::resample_buffer(count)?;
                let mut sample_a = vec![0.; a.len()];
                let mut sample_b = vec![0.; b.len()];
                for _ in 0..count {
                    rows.push(sample_stats(&mut rng, &mut sample_a, &mut sample_b));
                }
                Ok(rows)
            },
        )?;
        // User transforms stay on the caller, in draw order, without Send bounds.
        for chunk in chunks {
            for (before, after) in chunk {
                means.push(transform(before[0], after[0], scale)?);
                medians.push(transform(before[1], after[1], scale)?);
            }
        }
    }
    Ok(Report {
        config: config.clone(),
        baseline_units: a.len(),
        candidate_units: b.len(),
        mean: finish(
            transform(original_a[0], original_b[0], scale)?,
            means,
            config.confidence_level,
        ),
        median: finish(
            transform(original_a[1], original_b[1], scale)?,
            medians,
            config.confidence_level,
        ),
    })
}

/// Change in fixed-work throughput derived from a measurement statistic.
/// With no method label, duration interval endpoints are reciprocally transformed;
/// custom formatter methods describe their own resampling transformation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThroughputChange {
    /// None denotes reciprocal duration-change bounds used by ordinary wall time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    pub unit: String,
    pub statistic: String,
    pub point_percent: Option<f64>,
    pub interval_percent: Option<(f64, f64)>,
    pub unavailable_reason: Option<String>,
}
fn inverse_change(percent: f64) -> Option<f64> {
    if !percent.is_finite() || percent <= -100.0 {
        return None;
    }
    let value = -(percent / (100.0 + percent)) * 100.0;
    value.is_finite().then_some(value)
}
fn throughput_change(
    unit: &str,
    name: &str,
    statistic: Option<&Statistic>,
    reason: Option<&str>,
) -> ThroughputChange {
    let mut result = ThroughputChange {
        method: None,
        unit: format!("{unit}/s"),
        statistic: name.into(),
        point_percent: None,
        interval_percent: None,
        unavailable_reason: reason.map(str::to_owned),
    };
    if reason.is_some() {
        return result;
    }
    if let Some(statistic) = statistic {
        result.point_percent = statistic.point_percent.and_then(inverse_change);
        result.interval_percent = statistic
            .interval_percent
            .and_then(|(lower, upper)| inverse_change(upper).zip(inverse_change(lower)));
        if result.point_percent.is_none() || result.interval_percent.is_none() {
            result.unavailable_reason = Some(statistic.unavailable_reason.clone().unwrap_or_else(|| "Throughput change requires a positive finite duration ratio for the point and both bounds.".into()));
        }
    } else {
        result.unavailable_reason = Some("Duration change estimate unavailable.".into());
    }
    result
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comparison {
    pub case: String,
    pub metric: String,
    pub unit: String,
    pub scope: String,
    pub resampling_unit: String,
    pub baseline_units: usize,
    pub candidate_units: usize,
    pub report: Option<Report>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub throughput: Vec<ThroughputChange>,
    pub unavailable_reason: Option<String>,
}

pub(crate) fn comparison_samples(
    baseline: &crate::Run,
    candidate: &crate::Run,
    case: &str,
    metric: &crate::Metric,
) -> Result<(Vec<f64>, Vec<f64>, &'static str)> {
    let before = crate::analysis::values(baseline, case, metric, "candidate");
    let after = crate::analysis::values(candidate, case, metric, "candidate");
    let (before, after) = match (before, after) {
        (Ok(before), Ok(after)) => (before, after),
        (before, after) => {
            return Err(error(format!(
                "baseline: {}; candidate: {}",
                before
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "available".into()),
                after
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "available".into())
            )));
        }
    };
    if before.len() == 1 && after.len() == 1 {
        let samples = |run: &crate::Run| -> Result<Vec<f64>> {
            run.observations
                .iter()
                .filter(|o| o.case == case && o.metric == metric.id && o.variant == "candidate")
                .map(|o| {
                    let value = o
                        .number()?
                        .ok_or_else(|| error("missing numeric observation"))?;
                    Ok(if metric.statistic == "batch_total" {
                        value / o.operations as f64
                    } else {
                        value
                    })
                })
                .collect()
        };
        Ok((
            samples(baseline)?,
            samples(candidate)?,
            "normalized_observation",
        ))
    } else {
        Ok((
            before.values().map(|v| v.1).collect(),
            after.values().map(|v| v.1).collect(),
            "independent_process_median",
        ))
    }
}

/// Compare saved runs with per-case bootstrap settings from a candidate analysis.
/// Measurement compatibility is checked before selecting individual cases.
pub fn compare_runs_with_case_configs(
    baseline: &crate::Run,
    candidate: &crate::Run,
    fallback: &Config,
    cases: &std::collections::BTreeMap<String, Config>,
) -> Result<Vec<Comparison>> {
    fallback.validate()?;
    crate::analysis::validate_comparison(baseline, Some(candidate), 0., 0.05)?;
    for config in cases.values() {
        config.validate()?;
    }
    let mut rows = Vec::new();
    for case in &candidate.cases {
        rows.extend(compare_runs(
            &crate::history::one_case(baseline, &case.id),
            &crate::history::one_case(candidate, &case.id),
            cases.get(&case.id).unwrap_or(fallback),
        )?);
    }
    Ok(rows)
}

/// Compare complete compatible single-variant runs using normalized process
/// medians. If both runs have one process, compare normalized batches instead,
/// explicitly as exploratory within-process evidence. Missing observations invalidate
/// the metric. Confidence intervals are per statistic, not family-adjusted decisions.
pub fn compare_runs(
    baseline: &crate::Run,
    candidate: &crate::Run,
    config: &Config,
) -> Result<Vec<Comparison>> {
    config.validate()?;
    crate::analysis::validate_comparison(baseline, Some(candidate), 0., 0.05)?;
    let mut rows = Vec::new();
    for case in &baseline.cases {
        for metric in &case.metrics {
            let mut row = Comparison {
                case: case.id.clone(),
                metric: metric.id.clone(),
                unit: metric.unit.clone(),
                scope: metric.scope.clone(),
                resampling_unit: "independent_process_median".into(),
                baseline_units: 0,
                candidate_units: 0,
                report: None,
                throughput: Vec::new(),
                unavailable_reason: None,
            };
            let mut nonnegative_durations = false;
            match comparison_samples(baseline, candidate, &case.id, metric) {
                Ok((before, after, unit)) => {
                    row.resampling_unit = unit.into();
                    row.baseline_units = before.len();
                    row.candidate_units = after.len();
                    nonnegative_durations = before.iter().chain(&after).all(|value| *value >= 0.0);
                    match bootstrap(&before, &after, config) {
                        Ok(report) => row.report = Some(report),
                        Err(err) => row.unavailable_reason = Some(err.to_string()),
                    }
                }
                Err(err) => row.unavailable_reason = Some(err.to_string()),
            }

            if metric.id == "wall" && metric.unit == "ns" && metric.statistic == "batch_total" {
                for (unit, count) in crate::report::work_counters(case)? {
                    let reason = match count {
                        None => Some("Dynamic work counts require paired throughput comparison."),
                        Some(0) => Some("Zero work has no defined relative throughput change."),
                        Some(_) if row.report.is_some() && !nonnegative_durations => {
                            Some("Throughput requires nonnegative duration observations.")
                        }
                        Some(_) => None,
                    };
                    for (name, statistic) in [
                        (
                            "inverse mean duration",
                            row.report.as_ref().map(|r| &r.mean),
                        ),
                        (
                            "inverse median duration",
                            row.report.as_ref().map(|r| &r.median),
                        ),
                    ] {
                        row.throughput
                            .push(throughput_change(unit, name, statistic, reason));
                    }
                }
            }
            rows.push(row);
        }
    }
    crate::relative_format::restore(baseline, candidate, config, &mut rows)?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parallel_relative_preserves_stateful_transform_order_and_errors() {
        use super::*;
        use std::{cell::RefCell, rc::Rc};
        let a: Vec<_> = (0..37).map(|n| ((n * 17) % 43) as f64).collect();
        let b: Vec<_> = (0..41).map(|n| ((n * 19) % 47) as f64).collect();
        let scale = a.iter().chain(&b).copied().fold(0.0, f64::max);
        let na: Vec<_> = a.iter().map(|v| v / scale).collect();
        let nb: Vec<_> = b.iter().map(|v| v / scale).collect();
        for seed in [91, 0u64.wrapping_sub(0x9e3779b97f4a7c15)] {
            let config = Config {
                resamples: 2051,
                seed,
                ..Config::default()
            };
            let mut expected = Vec::new();
            let mut rng = Seeded::new(seed);
            let mut sa = vec![0.0; a.len()];
            let mut sb = vec![0.0; b.len()];
            for _ in 0..config.resamples {
                for v in &mut sa {
                    *v = na[crate::bootstrap::draw_index(&mut rng, a.len())];
                }
                for v in &mut sb {
                    *v = nb[crate::bootstrap::draw_index(&mut rng, b.len())];
                }
                let before = stats(&mut sa);
                let after = stats(&mut sb);
                expected.extend([(before[0], after[0], scale), (before[1], after[1], scale)]);
            }
            let before = stats(&mut na.clone());
            let after = stats(&mut nb.clone());
            expected.extend([(before[0], after[0], scale), (before[1], after[1], scale)]);
            let seen = Rc::new(RefCell::new(Vec::new()));
            let caller = std::thread::current().id();
            bootstrap_by(&a, &b, &config, |x, y, scale| {
                assert_eq!(std::thread::current().id(), caller);
                seen.borrow_mut().push((x, y, scale));
                Ok(Some(seen.borrow().len() as f64))
            })
            .unwrap();
            assert_eq!(*seen.borrow(), expected);
            let mut calls = 0;
            let failure = bootstrap_by(&a, &b, &config, |_, _, _| {
                calls += 1;
                if calls == 17 {
                    return Err(error("transform failure"));
                }
                Ok(Some(1.0))
            })
            .unwrap_err();
            assert!(failure.to_string().contains("transform failure"));
            assert_eq!(calls, 17);
        }
    }
    #[test]
    fn throughput_change_reverses_bounds_and_handles_undefined_ratios() {
        let statistic = super::Statistic {
            point_percent: Some(-50.),
            interval_percent: Some((-75., -25.)),
            draws_percent: vec![],
            undefined_draws: 0,
            unavailable_reason: None,
        };
        let rate = super::throughput_change("items", "mean", Some(&statistic), None);
        assert_eq!(rate.point_percent, Some(100.));
        let (low, high) = rate.interval_percent.unwrap();
        assert!((low - 100. / 3.).abs() < 1e-12);
        assert_eq!(high, 300.);
        assert_eq!(super::inverse_change(100.), Some(-50.));
        assert_eq!(super::inverse_change(0.), Some(0.));
        for value in [-100., -200., f64::NAN, f64::INFINITY] {
            assert!(super::inverse_change(value).is_none());
        }
        let missing = super::Statistic {
            interval_percent: None,
            unavailable_reason: Some("undefined draw".into()),
            ..statistic
        };
        let rate = super::throughput_change("items", "mean", Some(&missing), None);
        assert_eq!(rate.point_percent, Some(100.));
        assert!(rate.interval_percent.is_none());
        assert_eq!(rate.unavailable_reason.as_deref(), Some("undefined draw"));
    }

    use super::*;
    #[test]
    fn relative_charts_preserve_noise_bounds_and_undefined_populations() {
        let config = Config {
            resamples: 128,
            seed: 7,
            confidence_level: 0.95,
        };
        let mut row = Comparison {
            case: "<case>".into(),
            metric: "wall".into(),
            unit: "ns".into(),
            scope: "operation".into(),
            resampling_unit: "independent_process_median".into(),
            baseline_units: 2,
            candidate_units: 2,
            report: Some(bootstrap(&[2., 2.], &[3., 3.], &config).unwrap()),
            throughput: Vec::new(),
            unavailable_reason: None,
        };
        let text = markdown(std::slice::from_ref(&row));
        assert!(text.contains("mean | 50.000000 | [50.000000, 50.000000] | 2 / 2"));
        assert!(text.contains("median | 50.000000"));
        assert!(text.contains("confidence 95.00%; resamples 128; seed 7"));
        assert!(text.contains("&lt;case&gt;"));
        let html = charts(std::slice::from_ref(&row), 5.).unwrap();
        assert!(html.contains("&lt;case&gt;") && !html.contains("<case>"));
        assert_eq!(html.matches("<polygon").count(), 4);
        assert!(html.contains("[-5.000000e0, 5.000000e0]"));
        assert!(html.contains("point mass at 5.000000e1"));
        assert!(html.contains("Confidence: 95.0000%"));
        assert!(!html.contains("NaN"));
        assert!(charts(std::slice::from_ref(&row), 0.).is_ok());
        assert!(charts(std::slice::from_ref(&row), -1.).is_err());
        row.report = Some(bootstrap(&[0., 1.], &[1., 2.], &config).unwrap());
        let html = charts(std::slice::from_ref(&row), 5.).unwrap();
        assert!(!html.contains("<svg"));
        assert!(html.contains("undefined ratios"));
        let text = markdown(std::slice::from_ref(&row));
        assert!(text.contains("undefined draws retained"));
        assert!(text.contains("| unavailable |"));
        row.report = None;
        row.unavailable_reason = Some("missing <observations>".into());
        assert!(markdown(std::slice::from_ref(&row)).contains("missing &lt;observations&gt;"));
        assert!(
            charts(&[row], 5.)
                .unwrap()
                .contains("missing &lt;observations&gt;")
        );
    }

    #[test]
    fn case_comparisons_restore_distinct_bootstrap_settings() {
        let mut recorder = crate::Recorder::new();
        for id in ["a", "b"] {
            recorder
                .case(crate::Case {
                    id: id.into(),
                    contract: Default::default(),
                    metrics: vec![crate::Metric::duration("wall", "process", "batch_total")],
                })
                .unwrap();
            recorder.observe(id, "wall", 2).unwrap();
            recorder.observe(id, "wall", 4).unwrap();
        }
        let before = recorder.finish().unwrap();
        let mut after = before.clone();
        for observation in &mut after.observations {
            observation.value = Some("8".into());
        }
        let fallback = super::Config {
            resamples: 32,
            seed: 7,
            confidence_level: 0.9,
        };
        let special = super::Config {
            resamples: 64,
            seed: 13,
            confidence_level: 0.8,
        };
        let cases = [("b".into(), special.clone())].into_iter().collect();
        let rows =
            super::compare_runs_with_case_configs(&before, &after, &fallback, &cases).unwrap();
        assert_eq!(rows.len(), 2);
        for row in rows {
            let config = if row.case == "b" { &special } else { &fallback };
            let report = row.report.unwrap();
            assert_eq!(report.config.resamples, config.resamples);
            assert_eq!(report.config.seed, config.seed);
            assert_eq!(report.config.confidence_level, config.confidence_level);
        }
    }

    #[test]
    fn saved_runs_use_equal_process_weights_and_reject_incompatible_data() {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "case".into(),
                contract: Default::default(),
                metrics: vec![crate::Metric::duration("wall", "process", "batch_total")],
            })
            .unwrap();
        recorder.observe("case", "wall", 1).unwrap();
        let mut a = recorder.finish().unwrap();
        let original = a.observations[0].clone();
        a.observations.clear();
        for i in 0..6 {
            let mut o = original.clone();
            o.process = if i < 5 { 0 } else { 1 };
            o.sequence = i;
            o.operations = 2;
            o.value = Some(if i < 5 { "4" } else { "8" }.into());
            a.observations.push(o);
        }
        let mut b = a.clone();
        for o in &mut b.observations {
            o.value = Some(if o.process == 0 { "8" } else { "16" }.into());
        }
        let config = Config {
            resamples: 128,
            ..Default::default()
        };
        let rows = compare_runs(&a, &b, &config).unwrap();
        assert_eq!((rows[0].baseline_units, rows[0].candidate_units), (2, 2));
        let expected = bootstrap(&[2., 4.], &[4., 8.], &config).unwrap();
        assert_eq!(rows[0].report.as_ref().unwrap().mean, expected.mean);
        let legacy = serde_json::to_value(&rows[0]).unwrap();
        assert!(legacy.get("throughput").is_none());
        assert!(
            serde_json::from_value::<Comparison>(legacy)
                .unwrap()
                .throughput
                .is_empty()
        );
        for count in ["2", "0"] {
            for run in [&mut a, &mut b] {
                run.cases[0]
                    .contract
                    .insert("work.counter.items".into(), count.into());
            }
            let rows = compare_runs(&a, &b, &config).unwrap();
            assert_eq!(rows[0].throughput.len(), 2);
            if count == "2" {
                assert_eq!(rows[0].throughput[0].point_percent, Some(-50.));
                let restored: Comparison =
                    serde_json::from_str(&serde_json::to_string(&rows[0]).unwrap()).unwrap();
                assert_eq!(restored.throughput, rows[0].throughput);
            } else {
                assert!(rows[0].throughput[0].point_percent.is_none());
                assert!(markdown(&rows).contains("Zero work"));
            }
        }
        b.cases[0]
            .contract
            .insert("work.counter.items".into(), "3".into());
        assert!(compare_runs(&a, &b, &config).is_err());
        b.cases = a.cases.clone();
        b.cases[0].metrics[0].unit = "bytes".into();
        assert!(compare_runs(&a, &b, &config).is_err());
        b.cases = a.cases.clone();
        b.observations[0].value = None;
        b.observations[0].availability = crate::Availability::Unsupported("missing".into());
        let rows = compare_runs(&a, &b, &config).unwrap();
        assert!(rows[0].report.is_none());
        assert!(
            rows[0]
                .unavailable_reason
                .as_ref()
                .unwrap()
                .contains("unavailable")
        );
        let mut one = a.clone();
        one.observations.retain(|o| o.process == 0);
        let rows = compare_runs(&one, &one, &config).unwrap();
        assert_eq!(rows[0].baseline_units, 5);
        assert_eq!(rows[0].resampling_unit, "normalized_observation");
        assert_eq!(
            rows[0].report.as_ref().unwrap().mean.point_percent,
            Some(0.)
        );
        assert!(markdown(&rows).contains("batches may be autocorrelated"));
        assert!(
            charts(&rows, 5.)
                .unwrap()
                .contains("between-process reproducibility")
        );
        let mut changed = one.clone();
        for (index, observation) in changed.observations.iter_mut().enumerate() {
            observation.operations = (index + 1) as u64;
            observation.value = Some((4 * observation.operations).to_string());
        }
        let rows = compare_runs(&one, &changed, &config).unwrap();
        assert_eq!(
            rows[0].report.as_ref().unwrap().mean.point_percent,
            Some(100.)
        );
        let mixed = compare_runs(&one, &a, &config).unwrap();
        assert_eq!(mixed[0].resampling_unit, "independent_process_median");
        assert!(mixed[0].report.is_none());
        one.observations.truncate(1);
        assert!(
            compare_runs(&one, &one, &config).unwrap()[0]
                .report
                .is_none()
        );
    }

    #[test]
    fn independent_resamples_match_two_point_oracle_and_scale() {
        let config = Config {
            resamples: 2048,
            seed: 7,
            ..Default::default()
        };
        let report = bootstrap(&[1., 2.], &[2., 4.], &config).unwrap();
        assert_eq!(report.mean.point_percent, Some(100.));
        assert_eq!(report.mean, report.median);
        let possible: Vec<_> = [1., 1.5, 2.]
            .iter()
            .flat_map(|a| [2., 3., 4.].map(|b| (b / a - 1.) * 100.))
            .collect();
        for draw in &report.mean.draws_percent {
            assert!(
                possible
                    .iter()
                    .any(|expected| (draw.unwrap() - expected).abs() < 1e-10)
            );
        }
        // Reusing the same random indices for both groups would yield only 100%.
        assert!(report.mean.draws_percent.contains(&Some(0.)));
        assert!(report.mean.draws_percent.contains(&Some(300.)));
        let scaled = bootstrap(&[1e200, 2e200], &[2e200, 4e200], &config).unwrap();
        assert_eq!(report.mean, scaled.mean);
        assert_eq!(
            report.mean,
            bootstrap(&[1., 2.], &[2., 4.], &config).unwrap().mean
        );
        let restored: Report =
            serde_json::from_str(&serde_json::to_string(&report).unwrap()).unwrap();
        assert_eq!(report.mean, restored.mean);
    }
    #[test]
    fn undefined_ratios_are_preserved_without_biased_intervals() {
        let config = Config {
            resamples: 256,
            ..Default::default()
        };
        let report = bootstrap(&[0., 1.], &[1., 2.], &config).unwrap();
        assert!(report.mean.point_percent.is_some());
        assert!(report.mean.undefined_draws > 0);
        assert!(report.mean.undefined_draws < config.resamples);
        assert_eq!(report.mean.draws_percent.len(), config.resamples);
        assert!(report.mean.interval_percent.is_none());
        assert!(report.mean.unavailable_reason.is_some());
        let fixed = bootstrap(&[2.; 3], &[3.; 5], &config).unwrap();
        assert_eq!(fixed.mean.interval_percent, Some((50., 50.)));
        assert!(bootstrap(&[1.], &[2., 3.], &config).is_err());
        assert!(bootstrap(&[f64::NAN, 1.], &[2., 3.], &config).is_err());
    }
}
