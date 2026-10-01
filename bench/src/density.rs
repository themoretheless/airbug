//! Gaussian kernel density estimates for benchmark distributions.
use crate::{Result, error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Density {
    Curve {
        bandwidth: f64,
        points: Vec<(f64, f64)>,
        samples: usize,
    },
    /// A point mass has no ordinary continuous density; do not invent a width.
    Constant {
        value: f64,
        samples: usize,
    },
    Empty,
}

/// Gaussian KDE using sigma * (4 / (3*n))^(1/5), with unbiased sample sigma.
/// Coordinates retain input units and density has reciprocal input units.
/// The grid spans the observed range plus four bandwidths on each side.
pub fn gaussian(values: &[f64], resolution: usize) -> Result<Density> {
    if !(16..=4096).contains(&resolution) {
        return Err(error("density resolution must be 16..=4096"));
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(error("density requires finite samples"));
    }
    let Some(&first) = values.first() else {
        return Ok(Density::Empty);
    };
    if values.iter().all(|value| *value == first) {
        return Ok(Density::Constant {
            value: first,
            samples: values.len(),
        });
    }
    let scale = values.iter().map(|v| v.abs()).fold(0., f64::max);
    let normalized: Vec<_> = values.iter().map(|value| value / scale).collect();
    let mean = normalized.iter().sum::<f64>() / values.len() as f64;
    let variance = normalized
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;
    let width = variance.sqrt() * (4. / (3. * values.len() as f64)).powf(0.2);
    let bandwidth = width * scale;
    if !bandwidth.is_finite() || bandwidth <= 0. || width <= 0. {
        return Err(error("density bandwidth exceeds numeric precision"));
    }
    let lo = normalized.iter().copied().fold(f64::INFINITY, f64::min) - 4. * width;
    let hi = normalized.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 4. * width;
    let point = |index: usize| -> Result<(f64, f64)> {
        let x = lo + (hi - lo) * index as f64 / (resolution - 1) as f64;
        let sum = normalized
            .iter()
            .map(|sample| {
                let z = (x - sample) / width;
                (-0.5 * z * z).exp()
            })
            .sum::<f64>();
        let density = (sum / values.len() as f64 / (2. * std::f64::consts::PI).sqrt()) / bandwidth;
        let x = x * scale;
        if !x.is_finite() || !density.is_finite() {
            return Err(error("density grid exceeds numeric precision"));
        }
        Ok((x, density))
    };
    let workers = crate::parallel_analysis::density_workers(resolution, values.len());
    let points = if workers == 1 {
        (0..resolution).map(point).collect::<Result<Vec<_>>>()?
    } else {
        let jobs: Vec<_> = (0..workers)
            .map(|worker| resolution * worker / workers..resolution * (worker + 1) / workers)
            .collect();
        let mut points = Vec::with_capacity(resolution);
        for chunk in crate::parallel_analysis::ordered(jobs, |range| {
            range.map(point).collect::<Result<Vec<_>>>()
        })? {
            points.extend(chunk);
        }
        points
    };
    Ok(Density::Curve {
        bandwidth,
        points,
        samples: values.len(),
    })
}

/// One sample population; callers must keep measurement units consistent.
pub struct Series<'a> {
    pub label: &'a str,
    pub values: &'a [f64],
}

/// Gaussian density curves on common axes. Constant populations are marked at
/// y=0 and described as point masses, not assigned arbitrary finite densities.
pub fn figure(series: &[Series<'_>], title: &str, unit: &str) -> Result<String> {
    render(series, title, unit, false, None, None)
}

/// Add a rug of observations at zero, classified by Tukey fences. All values
/// remain in the KDE; rug positions are observations, not zero density estimates.
pub fn figure_with_outliers(series: &[Series<'_>], title: &str, unit: &str) -> Result<String> {
    render(series, title, unit, true, None, None)
}

/// Plot retained bootstrap draws with the independently computed point and interval.
pub fn estimate_figure(
    values: &[f64],
    estimate: &crate::bootstrap::Estimate,
    title: &str,
    unit: &str,
) -> Result<String> {
    if values.is_empty() {
        return Err(error("bootstrap density requires retained draws"));
    }
    if ![estimate.point, estimate.lower, estimate.upper]
        .iter()
        .all(|v| v.is_finite())
        || estimate.lower > estimate.upper
    {
        return Err(error(
            "bootstrap density requires a finite ordered interval",
        ));
    }
    render(
        &[Series {
            label: "bootstrap draws",
            values,
        }],
        title,
        unit,
        false,
        Some(estimate),
        None,
    )
}

/// Relative bootstrap density with a symmetric practical-change region in percent.
pub fn relative_figure(
    values: &[f64],
    estimate: &crate::bootstrap::Estimate,
    title: &str,
    threshold_percent: f64,
) -> Result<String> {
    if !threshold_percent.is_finite() || threshold_percent < 0. {
        return Err(error(
            "relative chart threshold must be finite and nonnegative",
        ));
    }
    // Apply the same point/interval validation as absolute distributions.
    if values.is_empty()
        || ![estimate.point, estimate.lower, estimate.upper]
            .iter()
            .all(|v| v.is_finite())
        || estimate.lower > estimate.upper
    {
        return Err(error(
            "relative density requires draws and a finite ordered interval",
        ));
    }
    render(
        &[Series {
            label: "relative bootstrap draws",
            values,
        }],
        title,
        "%",
        false,
        Some(estimate),
        Some(threshold_percent),
    )
}

fn render(
    series: &[Series<'_>],
    title: &str,
    unit: &str,
    outliers: bool,
    estimate: Option<&crate::bootstrap::Estimate>,
    noise: Option<f64>,
) -> Result<String> {
    use crate::report::escape;
    use crate::viz::{Domain, Palette, Plot};
    let densities: Vec<_> = series
        .iter()
        .map(|s| gaussian(s.values, 256))
        .collect::<Result<_>>()?;
    let mut xs = Vec::new();
    let mut peak = 0f64;
    let mut details = String::from("<dl>");
    for (series, density) in series.iter().zip(&densities) {
        details.push_str(&format!("<dt>{}</dt><dd>", escape(series.label)));
        match density {
            Density::Curve {
                points,
                bandwidth,
                samples,
            } => {
                xs.extend(points.iter().map(|p| p.0));
                peak = points.iter().map(|p| p.1).fold(peak, f64::max);
                details.push_str(&format!(
                    "{samples} samples; Gaussian KDE bandwidth {bandwidth:.6e} {}.",
                    escape(unit)
                ));
            }
            Density::Constant { value, samples } => {
                xs.push(*value);
                details.push_str(&format!("{samples} identical samples: point mass at {value:.6e} {}. Marker at zero is not a density estimate.", escape(unit)));
            }
            Density::Empty => details.push_str("No samples; no density estimate."),
        }
        details.push_str("</dd>");
    }
    details.push_str("</dl>");
    if xs.is_empty() {
        return Ok(format!(
            "<section><h3>{}</h3>{details}</section>",
            escape(title)
        ));
    }
    if let Some(estimate) = estimate {
        xs.extend([estimate.point, estimate.lower, estimate.upper]);
        details.push_str(&format!("<p>Point estimate: {:.6e}; confidence interval: [{:.6e}, {:.6e}] {}. Shading shows the reported bootstrap interval; the red line marks the original point estimate. KDE smoothing does not define these bounds.</p>", estimate.point, estimate.lower, estimate.upper, escape(unit)));
    }
    if let Some(threshold) = noise {
        xs.extend([-threshold, 0., threshold]);
        details.push_str(&format!("<p>Practical noise region: [-{threshold:.6e}, {threshold:.6e}] %. Amber shading marks this region; the zero line means no change. Positive values mean an increase in the measured quantity. Confidence intervals are per statistic, not family-adjusted decisions.</p>"));
    }
    let x = Domain::fit(xs);
    let (lo, hi) = x.ends();
    if !lo.is_finite() || !hi.is_finite() || !(hi - lo).is_finite() {
        return Err(error("density chart domain exceeds numeric precision"));
    }
    let mut plot = Plot::new(title).x(x).y(Domain::Linear { min: 0., max: if peak > 0. { peak } else { 1. } })
        .notes(format!("X: {}. Y: probability density (1/{}). Gaussian kernels with Silverman bandwidth; four bandwidths of padding. Each curve is normalized independently. Point masses are identified below.", unit, unit));
    if series.len() > 1 {
        plot.legend_wrapped(
            &series
                .iter()
                .enumerate()
                .map(|(i, s)| (s.label.to_string(), Palette::LIGHT.series(i)))
                .collect::<Vec<_>>(),
        );
    }
    if let Some(threshold) = noise {
        let top = if peak > 0. { peak } else { 1. };
        plot.polygon(
            &[
                (-threshold, 0.),
                (threshold, 0.),
                (threshold, top),
                (-threshold, top),
            ],
            "#b45309",
            "Practical noise region",
        );
        plot.path_labeled(&[(0., 0.), (0., top)], "#64748b", 1., "No change");
    }
    if let Some(estimate) = estimate {
        let top = if peak > 0. { peak } else { 1. };
        plot.polygon(
            &[
                (estimate.lower, 0.),
                (estimate.upper, 0.),
                (estimate.upper, top),
                (estimate.lower, top),
            ],
            "#397ec0",
            "Bootstrap confidence interval",
        );
        plot.path_labeled(
            &[(estimate.point, 0.), (estimate.point, top)],
            "#be123c",
            2.,
            "Original point estimate",
        );
    }
    for (i, (series, density)) in series.iter().zip(&densities).enumerate() {
        let color = Palette::LIGHT.series(i);
        match density {
            Density::Curve { points, .. } => {
                plot.path_labeled(points, color, 2., series.label);
            }
            Density::Constant { value, .. } => {
                plot.points_labeled(
                    &[(*value, 0.)],
                    color,
                    4.,
                    &format!("{}: point mass", series.label),
                );
            }
            Density::Empty => {}
        }
    }
    if outliers {
        for (index, series) in series.iter().enumerate() {
            if series.values.is_empty() {
                continue;
            }
            let classification = crate::outliers::classify(series.values)?;
            for point in classification.points {
                use crate::outliers::Label;
                let color = match point.label {
                    Label::Normal => Palette::LIGHT.series(index),
                    Label::LowMild | Label::HighMild => "#b45309",
                    Label::LowSevere | Label::HighSevere => "#be123c",
                };
                plot.points_labeled(
                    &[(point.value, 0.)],
                    color,
                    2.,
                    &format!(
                        "{}: observation {} ({})",
                        series.label,
                        point.index,
                        point.label.name()
                    ),
                );
            }
        }
        details.push_str("<p>Observation rug at zero: series color = normal, amber = mild outlier, red = severe outlier (Tukey fences). Rug markers are not density estimates. No observations discarded.</p>");
    }
    Ok(format!("{}{details}", plot.figure()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn parallel_density_preserves_every_point_bit_for_bit() {
        use super::*;
        for factor in [1.0, 1e200] {
            let values: Vec<_> = (0..1025)
                .map(|n| (((n * 17) % 509) as f64 - 254.0) * factor)
                .collect();
            let resolution = 257;
            let Density::Curve {
                bandwidth,
                points,
                samples,
            } = gaussian(&values, resolution).unwrap()
            else {
                panic!("curve expected")
            };
            let scale = values.iter().map(|v| v.abs()).fold(0.0, f64::max);
            let normalized: Vec<_> = values.iter().map(|v| v / scale).collect();
            let mean = normalized.iter().sum::<f64>() / values.len() as f64;
            let variance = normalized.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / (values.len() - 1) as f64;
            let width = variance.sqrt() * (4.0 / (3.0 * values.len() as f64)).powf(0.2);
            assert_eq!(bandwidth.to_bits(), (width * scale).to_bits());
            assert_eq!(samples, values.len());
            let lo = normalized.iter().copied().fold(f64::INFINITY, f64::min) - 4.0 * width;
            let hi = normalized.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 4.0 * width;
            for (index, (actual_x, actual_y)) in points.iter().enumerate() {
                let x = lo + (hi - lo) * index as f64 / (resolution - 1) as f64;
                let sum = normalized
                    .iter()
                    .map(|v| {
                        let z = (x - v) / width;
                        (-0.5 * z * z).exp()
                    })
                    .sum::<f64>();
                let y =
                    (sum / values.len() as f64 / (2.0 * std::f64::consts::PI).sqrt()) / bandwidth;
                assert_eq!(actual_x.to_bits(), (x * scale).to_bits());
                assert_eq!(actual_y.to_bits(), y.to_bits());
            }
            assert_eq!(points.len(), resolution);
        }
    }
    use super::*;
    #[test]
    fn estimate_overlay_keeps_external_point_and_degenerate_intervals() {
        let estimate = crate::bootstrap::Estimate {
            point: 50.,
            lower: 1.,
            upper: 3.,
            standard_error: Some(1.),
        };
        let chart = estimate_figure(&[1., 2., 3.], &estimate, "bootstrap", "ns").unwrap();
        assert_eq!(chart.matches("<polygon").count(), 1);
        assert!(chart.contains("Original point estimate"));
        assert!(chart.contains("5.000000e1"));
        assert!(chart.contains("KDE smoothing does not define these bounds"));
        let Density::Curve { points, .. } = gaussian(&[1., 2., 3.], 256).unwrap() else {
            panic!()
        };
        let mut xs: Vec<_> = points.iter().map(|p| p.0).collect();
        xs.extend([50., 1., 3.]);
        let plot = crate::viz::Plot::new("oracle").x(crate::viz::Domain::fit(xs));
        assert!(chart.contains(&format!("M{:.2} ", plot.xpx(50.).unwrap())));
        let constant = crate::bootstrap::Estimate {
            point: 7.,
            lower: 7.,
            upper: 7.,
            standard_error: Some(0.),
        };
        let chart = estimate_figure(&[7.; 10], &constant, "constant", "ns").unwrap();
        assert!(chart.contains("point mass"));
        assert!(!chart.contains("NaN"));
        assert!(estimate_figure(&[], &estimate, "empty", "ns").is_err());
        let reversed = crate::bootstrap::Estimate {
            lower: 4.,
            upper: 2.,
            ..estimate
        };
        assert!(estimate_figure(&[1., 2.], &reversed, "invalid", "ns").is_err());
    }

    #[test]
    fn outlier_rug_preserves_density_and_labels_extreme_observations() {
        let values = [1., 2., 3., 4., 100.];
        let series = [Series {
            label: "<sample>",
            values: &values,
        }];
        let plain = figure(&series, "density", "ns").unwrap();
        let marked = figure_with_outliers(&series, "density", "ns").unwrap();
        let path = plain
            .split("pathLength=\"100\"")
            .next()
            .unwrap()
            .rsplit("<path")
            .next()
            .unwrap();
        assert!(marked.contains(path));
        assert_eq!(marked.matches("<circle").count(), values.len());
        assert!(marked.contains("&lt;sample&gt;: observation 4 (high severe)"));
        assert!(marked.contains("No observations discarded"));
        assert!(!marked.contains("<sample>"));
    }

    #[test]
    fn overlay_preserves_constant_empty_and_escaped_series() {
        let html = figure(
            &[
                Series {
                    label: "<baseline>",
                    values: &[1., 2., 3.],
                },
                Series {
                    label: "candidate",
                    values: &[4., 5., 6.],
                },
                Series {
                    label: "constant",
                    values: &[7.; 3],
                },
                Series {
                    label: "empty",
                    values: &[],
                },
            ],
            "<density>",
            "ns",
        )
        .unwrap();
        assert_eq!(html.matches("class=\"dv\"").count(), 2);
        assert_eq!(html.matches("<circle").count(), 1);
        assert!(html.contains("&lt;baseline&gt;"));
        assert!(!html.contains("<baseline>"));
        assert!(html.contains("point mass") && html.contains("No samples"));
        assert!(html.contains("1/ns"));
        assert!(
            !figure(
                &[Series {
                    label: "empty",
                    values: &[]
                }],
                "empty",
                "ns"
            )
            .unwrap()
            .contains("<svg")
        );
        assert!(
            figure(
                &[Series {
                    label: "invalid",
                    values: &[f64::NAN]
                }],
                "invalid",
                "ns"
            )
            .is_err()
        );
    }

    #[test]
    fn gaussian_normalizes_and_scales_density_inversely() {
        let values = [-2., -1., 0., 1., 2.];
        let Density::Curve {
            bandwidth, points, ..
        } = gaussian(&values, 2048).unwrap()
        else {
            panic!()
        };
        let expected = 2.5f64.sqrt() * (4.0f64 / 15.).powf(0.2);
        assert!((bandwidth - expected).abs() < 1e-12);
        let area: f64 = points
            .windows(2)
            .map(|p| (p[1].0 - p[0].0) * (p[0].1 + p[1].1) / 2.)
            .sum();
        assert!((area - 1.).abs() < 0.0001, "{area}");
        let Density::Curve { points: scaled, .. } =
            gaussian(&values.map(|v| v * 1e200), 2048).unwrap()
        else {
            panic!()
        };
        for ((x, y), (sx, sy)) in points.iter().zip(scaled) {
            assert!((x - sx / 1e200).abs() < 1e-12);
            assert!((y - sy * 1e200).abs() < 1e-12);
        }
        assert!((points[0].1 - points.last().unwrap().1).abs() < 1e-12);
    }
    #[test]
    fn degenerate_and_invalid_samples_are_explicit() {
        assert!(matches!(gaussian(&[], 100).unwrap(), Density::Empty));
        assert!(matches!(
            gaussian(&[7.; 4], 100).unwrap(),
            Density::Constant {
                value: 7.,
                samples: 4
            }
        ));
        assert!(matches!(
            gaussian(&[0.], 100).unwrap(),
            Density::Constant { .. }
        ));
        assert!(gaussian(&[f64::NAN], 100).is_err());
        assert!(gaussian(&[1., 2.], 1).is_err());
        assert!(gaussian(&[-f64::MAX, f64::MAX], 100).is_err());
    }
}
