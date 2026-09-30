//! Independent-sample bootstrap of percentage changes in mean and median.
use crate::{Result, Seeded, bootstrap::Config, error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Statistic {
    pub point_percent: Option<f64>,
    /// Generation order. Undefined ratios are retained as null, never discarded.
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
        html.push_str(&format!("<p>Baseline: {} independent units; candidate: {}. Confidence: {:.4}%; resamples: {}; seed: {}.</p>",
            report.baseline_units, report.candidate_units, report.config.confidence_level * 100., report.config.resamples, report.config.seed));
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
                        standard_error: 0.,
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
    let mut rng = Seeded::new(config.seed);
    let mut sample_a = vec![0.; a.len()];
    let mut sample_b = vec![0.; b.len()];
    let mut means = Vec::with_capacity(config.resamples);
    let mut medians = Vec::with_capacity(config.resamples);
    for _ in 0..config.resamples {
        for value in &mut sample_a {
            *value = a[crate::bootstrap::draw_index(&mut rng, a.len())];
        }
        for value in &mut sample_b {
            *value = b[crate::bootstrap::draw_index(&mut rng, b.len())];
        }
        let before = stats(&mut sample_a);
        let after = stats(&mut sample_b);
        means.push(change(before[0], after[0]));
        medians.push(change(before[1], after[1]));
    }
    Ok(Report {
        config: config.clone(),
        baseline_units: a.len(),
        candidate_units: b.len(),
        mean: finish(
            change(original_a[0], original_b[0]),
            means,
            config.confidence_level,
        ),
        median: finish(
            change(original_a[1], original_b[1]),
            medians,
            config.confidence_level,
        ),
    })
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
    pub unavailable_reason: Option<String>,
}

/// Compare complete compatible single-variant runs using normalized process
/// medians. Missing observations invalidate the metric instead of being dropped.
/// Confidence intervals are per statistic, not family-adjusted decisions.
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
                unavailable_reason: None,
            };
            let before = crate::analysis::values(baseline, &case.id, metric, "candidate");
            let after = crate::analysis::values(candidate, &case.id, metric, "candidate");
            match (before, after) {
                (Ok(before), Ok(after)) => {
                    row.baseline_units = before.len();
                    row.candidate_units = after.len();
                    let before: Vec<_> = before.values().map(|v| v.1).collect();
                    let after: Vec<_> = after.values().map(|v| v.1).collect();
                    match bootstrap(&before, &after, config) {
                        Ok(report) => row.report = Some(report),
                        Err(err) => row.unavailable_reason = Some(err.to_string()),
                    }
                }
                (before, after) => {
                    row.unavailable_reason = Some(format!(
                        "baseline: {}; candidate: {}",
                        before
                            .err()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "available".into()),
                        after
                            .err()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "available".into())
                    ))
                }
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
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
            unavailable_reason: None,
        };
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
        row.report = None;
        row.unavailable_reason = Some("missing <observations>".into());
        assert!(
            charts(&[row], 5.)
                .unwrap()
                .contains("missing &lt;observations&gt;")
        );
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
        assert_eq!(rows[0].baseline_units, 1);
        assert!(rows[0].report.is_none());
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
