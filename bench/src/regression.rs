//! Through-origin least-squares fits for batch totals against operation counts.
use crate::{
    Result, Seeded,
    bootstrap::{Config, Estimate},
    error,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sample {
    pub operations: f64,
    pub total: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    /// Slope in metric units per operation; model is total = slope * operations.
    pub slope: Estimate,
    /// Centered R² may be negative for a through-origin model; null when totals are constant.
    pub r_squared: Option<f64>,
    pub samples: usize,
}
/// Scatter, through-origin fitted line and slope confidence wedge for one process.
/// The wedge is uncertainty in slope, not a prediction interval for a future batch.
pub fn figure(samples: &[Sample], fit: &Fit, title: &str, unit: &str) -> String {
    comparison_figure(
        &[Series {
            label: "process",
            samples,
            fit,
        }],
        title,
        unit,
    )
}

/// One independently fitted process. Callers must supply compatible metric units.
pub struct Series<'a> {
    pub label: &'a str,
    pub samples: &'a [Sample],
    pub fit: &'a Fit,
}

/// Overlay independent fits on shared axes without pooling samples or extrapolating
/// past each process's largest observed operation count.
pub fn comparison_figure(series: &[Series<'_>], title: &str, unit: &str) -> String {
    use crate::viz::{Domain, Palette, Plot};
    let unavailable = || {
        format!(
            "<p>{}: regression chart unavailable (missing or invalid paired samples).</p>",
            crate::report::escape(title)
        )
    };
    if series.is_empty() {
        return unavailable();
    }
    let (mut xmax, mut ymin, mut ymax) = (0f64, 0f64, 0f64);
    let mut endpoints = Vec::new();
    for row in series {
        if row.samples.is_empty()
            || row
                .samples
                .iter()
                .any(|s| !s.operations.is_finite() || s.operations <= 0. || !s.total.is_finite())
        {
            return unavailable();
        }
        let end = row.samples.iter().map(|s| s.operations).fold(0., f64::max);
        let [low, point, high] = [
            row.fit.slope.lower,
            row.fit.slope.point,
            row.fit.slope.upper,
        ]
        .map(|s| s * end);
        if ![low, point, high].iter().all(|v| v.is_finite()) || low > point || point > high {
            return unavailable();
        }
        xmax = xmax.max(end);
        ymin = row
            .samples
            .iter()
            .map(|s| s.total)
            .fold(ymin.min(low), f64::min);
        ymax = row
            .samples
            .iter()
            .map(|s| s.total)
            .fold(ymax.max(high), f64::max);
        endpoints.push((end, low, point, high));
    }
    if !(ymax - ymin).is_finite() {
        return unavailable();
    }
    let mut plot = Plot::new(title)
        .x(Domain::Linear { min: 0., max: xmax })
        .y(Domain::Linear { min: ymin, max: ymax })
        .notes(format!("X: operations per batch. Y: total ({unit}). Shading: bootstrap confidence interval for the through-origin slope; not a prediction interval. Each series is an independent process fit; samples are not pooled."));
    if series.len() > 1 {
        plot = plot.margins(64., 16., 30., 34.);
        plot.legend_wrapped(
            &series
                .iter()
                .enumerate()
                .map(|(i, row)| (row.label.to_string(), Palette::LIGHT.series(i)))
                .collect::<Vec<_>>(),
        );
    }
    for (i, (row, &(end, low, point, high))) in series.iter().zip(&endpoints).enumerate() {
        let color = Palette::LIGHT.series(i);
        plot.polygon(
            &[(0., 0.), (end, low), (end, high)],
            color,
            &format!("{}: Slope confidence interval", row.label),
        );
        plot.path_labeled(&[(0., 0.), (end, point)], color, 2., row.label);
        let points: Vec<_> = row
            .samples
            .iter()
            .map(|s| (s.operations, s.total))
            .collect();
        plot.points_labeled(&points, color, 2.5, row.label);
    }
    plot.figure()
}

fn slope(samples: &[Sample]) -> f64 {
    let numerator = samples.iter().map(|s| s.operations * s.total).sum::<f64>();
    let denominator = samples
        .iter()
        .map(|s| s.operations * s.operations)
        .sum::<f64>();
    numerator / denominator
}
/// Bootstrap resamples (operations,total) pairs together. This does not remove
/// within-process autocorrelation; independent-process comparison remains separate.
pub fn fit(samples: &[Sample], config: &Config) -> Result<Fit> {
    Ok(fit_impl(samples, config, false)?.0)
}

/// Retain paired-bootstrap slope draws in original units and generation order.
pub fn fit_with_distribution(samples: &[Sample], config: &Config) -> Result<(Fit, Vec<f64>)> {
    let (fit, draws) = fit_impl(samples, config, true)?;
    Ok((fit, draws.unwrap()))
}
fn fit_impl(samples: &[Sample], config: &Config, capture: bool) -> Result<(Fit, Option<Vec<f64>>)> {
    config.validate()?;
    if samples.len() < 2
        || samples
            .iter()
            .any(|s| !s.operations.is_finite() || s.operations <= 0.0 || !s.total.is_finite())
    {
        return Err(error(
            "regression requires at least two finite pairs with positive operation counts",
        ));
    }
    let xscale = samples.iter().map(|s| s.operations).fold(0.0, f64::max);
    let yscale = samples
        .iter()
        .map(|s| s.total.abs())
        .fold(0.0, f64::max)
        .max(f64::MIN_POSITIVE);
    let normalized: Vec<_> = samples
        .iter()
        .map(|s| Sample {
            operations: s.operations / xscale,
            total: s.total / yscale,
        })
        .collect();
    if normalized
        .iter()
        .any(|s| s.operations * s.operations == 0.0)
    {
        return Err(error(
            "operation-count range exceeds regression numeric precision",
        ));
    }
    let point = slope(&normalized);
    let mean = normalized.iter().map(|s| s.total).sum::<f64>() / samples.len() as f64;
    let residual = normalized
        .iter()
        .map(|s| (s.total - point * s.operations).powi(2))
        .sum::<f64>();
    let variation = normalized
        .iter()
        .map(|s| (s.total - mean).powi(2))
        .sum::<f64>();
    let r_squared = (variation > 0.0 && samples.iter().any(|s| s.total != samples[0].total))
        .then(|| 1.0 - residual / variation)
        .filter(|v| v.is_finite());
    let mut rng = Seeded::new(config.seed);
    let mut sample = vec![normalized[0]; samples.len()];
    let mut distribution = Vec::with_capacity(config.resamples);
    for _ in 0..config.resamples {
        for s in &mut sample {
            *s = normalized[crate::bootstrap::draw_index(&mut rng, samples.len())];
        }
        let value = slope(&sample);
        if !value.is_finite() {
            return Err(error(
                "bootstrap regression produced an unrepresentable slope",
            ));
        }
        distribution.push(value);
    }
    let average = distribution.iter().sum::<f64>() / distribution.len() as f64;
    let standard_error = crate::bootstrap::deviation(&distribution, average);
    let raw = capture.then(|| distribution.clone());
    distribution.sort_by(f64::total_cmp);
    let tail = (1.0 - config.confidence_level) / 2.0;
    let rescale = |value: f64| {
        let result = value * yscale / xscale;
        if result.is_finite() {
            result
        } else {
            value / xscale * yscale
        }
    };
    let retained = raw.map(|values| values.into_iter().map(rescale).collect::<Vec<_>>());
    if retained
        .as_ref()
        .is_some_and(|values| values.iter().any(|v| !v.is_finite()))
    {
        return Err(error(
            "regression distribution exceeds finite numeric range",
        ));
    }
    let estimate = Estimate {
        point: rescale(point),
        lower: rescale(crate::bootstrap::quantile(&distribution, tail)),
        upper: rescale(crate::bootstrap::quantile(&distribution, 1.0 - tail)),
        standard_error: rescale(standard_error),
    };
    if [
        estimate.point,
        estimate.lower,
        estimate.upper,
        estimate.standard_error,
    ]
    .iter()
    .any(|v| !v.is_finite())
    {
        return Err(error("regression estimate exceeds finite numeric range"));
    }
    Ok((
        Fit {
            slope: estimate,
            r_squared,
            samples: samples.len(),
        },
        retained,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_slope_draws_match_paired_oracle_and_existing_fit() {
        let samples = [
            Sample {
                operations: 1.,
                total: 2.,
            },
            Sample {
                operations: 2.,
                total: 8.,
            },
        ];
        let config = config();
        let (result, draws) = fit_with_distribution(&samples, &config).unwrap();
        assert_eq!(
            serde_json::to_value(&result).unwrap(),
            serde_json::to_value(fit(&samples, &config).unwrap()).unwrap()
        );
        assert_eq!(draws.len(), config.resamples);
        assert_eq!(draws, fit_with_distribution(&samples, &config).unwrap().1);
        // All four ordered paired resamples yield slopes 2, 18/5, 18/5, 4.
        for draw in &draws {
            assert!(
                [2., 3.6, 4.]
                    .iter()
                    .any(|expected| (draw - expected).abs() < 1e-12)
            );
        }
        for expected in [2., 3.6, 4.] {
            assert!(draws.iter().any(|draw| (draw - expected).abs() < 1e-12));
        }
        let restored: Vec<f64> =
            serde_json::from_str(&serde_json::to_string(&draws).unwrap()).unwrap();
        assert_eq!(draws, restored);
        let mut sorted = draws;
        sorted.sort_by(f64::total_cmp);
        let tail = (1. - config.confidence_level) / 2.;
        assert!((crate::bootstrap::quantile(&sorted, tail) - result.slope.lower).abs() < 1e-12);
        assert!(
            (crate::bootstrap::quantile(&sorted, 1. - tail) - result.slope.upper).abs() < 1e-12
        );
    }

    #[test]
    fn chart_and_paired_samples_roundtrip_with_legacy_fallback() {
        let samples = vec![
            Sample {
                operations: 1.,
                total: 2.,
            },
            Sample {
                operations: 2.,
                total: 4.,
            },
        ];
        let fit = Fit {
            slope: Estimate {
                point: 2.,
                lower: 1.,
                upper: 3.,
                standard_error: 0.5,
            },
            r_squared: Some(1.),
            samples: 2,
        };
        let chart = figure(&samples, &fit, "<regression>", "ns");
        assert_eq!(chart.matches("<circle").count(), 2);
        assert_eq!(chart.matches("<polygon").count(), 1);
        assert!(chart.contains("&lt;regression&gt;"));
        assert!(chart.contains("not a prediction interval"));
        let plot = crate::viz::Plot::new("geometry")
            .x(crate::viz::Domain::Linear { min: 0., max: 2. })
            .y(crate::viz::Domain::Linear { min: 0., max: 6. });
        let vertices = format!(
            "{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
            plot.xpx(0.).unwrap(),
            plot.ypx(0.).unwrap(),
            plot.xpx(2.).unwrap(),
            plot.ypx(2.).unwrap(),
            plot.xpx(2.).unwrap(),
            plot.ypx(6.).unwrap()
        );
        assert!(chart.contains(&vertices));
        let row = crate::bootstrap::ProcessRegression {
            process: 7,
            slope_distribution: None,
            fit,
            samples,
        };
        let mut json = serde_json::to_value(&row).unwrap();
        let decoded: crate::bootstrap::ProcessRegression =
            serde_json::from_value(json.clone()).unwrap();
        assert_eq!(decoded.samples, row.samples);
        json.as_object_mut().unwrap().remove("samples");
        let legacy: crate::bootstrap::ProcessRegression = serde_json::from_value(json).unwrap();
        assert!(legacy.samples.is_empty());
        assert!(figure(&legacy.samples, &legacy.fit, "old", "ns").contains("unavailable"));
    }

    #[test]
    fn overlay_shares_axes_and_keeps_each_fit_in_its_observed_range() {
        let baseline = [
            Sample {
                operations: 1.,
                total: 2.,
            },
            Sample {
                operations: 2.,
                total: 4.,
            },
        ];
        let candidate = [
            Sample {
                operations: 2.,
                total: 6.,
            },
            Sample {
                operations: 4.,
                total: 12.,
            },
        ];
        let a = fit(&baseline, &config()).unwrap();
        let b = fit(&candidate, &config()).unwrap();
        let chart = comparison_figure(
            &[
                Series {
                    label: "<baseline>",
                    samples: &baseline,
                    fit: &a,
                },
                Series {
                    label: "candidate",
                    samples: &candidate,
                    fit: &b,
                },
            ],
            "overlay",
            "ns",
        );
        assert_eq!(chart.matches("<circle").count(), 4);
        assert_eq!(chart.matches("<polygon").count(), 2);
        assert!(chart.contains("&lt;baseline&gt;"));
        assert!(chart.contains("&lt;baseline&gt;: x = 1, y = 2"));
        assert!(chart.contains("candidate: x = 4, y = 12"));
        let plot = crate::viz::Plot::new("geometry")
            .margins(64., 16., 30., 34.)
            .x(crate::viz::Domain::Linear { min: 0., max: 4. })
            .y(crate::viz::Domain::Linear { min: 0., max: 12. });
        for (x, y) in [(2., 4.), (4., 12.)] {
            assert!(chart.contains(&format!(
                "L{:.2} {:.2}",
                plot.xpx(x).unwrap(),
                plot.ypx(y).unwrap()
            )));
        }
        let names: Vec<_> = (0..8).map(|i| format!("candidate process {i}")).collect();
        let crowded: Vec<_> = names
            .iter()
            .map(|label| Series {
                label,
                samples: &candidate,
                fit: &b,
            })
            .collect();
        let crowded = comparison_figure(&crowded, "crowded", "ns");
        for name in &names {
            assert!(crowded.contains(&format!("{name}: x = 4, y = 12")));
        }
        let mut bad = b.clone();
        bad.slope.upper = f64::MAX;
        assert!(
            comparison_figure(
                &[Series {
                    label: "overflow",
                    samples: &candidate,
                    fit: &bad
                }],
                "bad",
                "ns"
            )
            .contains("unavailable")
        );
    }

    fn config() -> Config {
        Config {
            resamples: 500,
            ..Default::default()
        }
    }
    #[test]
    fn exact_line_and_known_noisy_fit() {
        let samples: Vec<_> = (1..=10)
            .map(|n| Sample {
                operations: n as f64,
                total: n as f64 * 7.0,
            })
            .collect();
        let result = fit(&samples, &config()).unwrap();
        assert!((result.slope.point - 7.0).abs() < 1e-12);
        assert!((result.slope.lower - 7.0).abs() < 1e-12);
        assert!((result.slope.upper - 7.0).abs() < 1e-12);
        assert!(result.slope.standard_error < 1e-12);
        assert_eq!(result.r_squared, Some(1.0));
        let result = fit(
            &[
                Sample {
                    operations: 1.0,
                    total: 2.0,
                },
                Sample {
                    operations: 2.0,
                    total: 3.0,
                },
                Sample {
                    operations: 3.0,
                    total: 5.0,
                },
            ],
            &config(),
        )
        .unwrap();
        assert!((result.slope.point - 23.0 / 14.0).abs() < 1e-12);
        assert!((result.r_squared.unwrap() - 187.0 / 196.0).abs() < 1e-12);
        assert!(result.slope.lower <= 23.0 / 14.0 && result.slope.upper >= 23.0 / 14.0);
    }
    #[test]
    fn constant_totals_and_invalid_inputs() {
        let samples = [
            Sample {
                operations: 1.0,
                total: 5.0,
            },
            Sample {
                operations: 2.0,
                total: 5.0,
            },
        ];
        assert_eq!(fit(&samples, &config()).unwrap().r_squared, None);
        assert!(fit(&samples[..1], &config()).is_err());
        assert!(
            fit(
                &[Sample {
                    operations: 0.0,
                    total: 5.0
                }; 2],
                &config()
            )
            .is_err()
        );
        assert!(
            fit(
                &[Sample {
                    operations: 1.0,
                    total: f64::NAN
                }; 2],
                &config()
            )
            .is_err()
        );
        let huge = [
            Sample {
                operations: 1e200,
                total: 2e200,
            },
            Sample {
                operations: 2e200,
                total: 4e200,
            },
        ];
        assert!((fit(&huge, &config()).unwrap().slope.point - 2.0).abs() < 1e-12);
    }
}
