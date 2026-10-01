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
    /// Centered R² at the lower and upper slope confidence bounds, in that order.
    /// These are fit diagnostics, not a confidence interval for R².
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r_squared_at_slope_bounds: Option<[f64; 2]>,
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
        if ![low, point, high].iter().all(|v| v.is_finite()) || low > high {
            return unavailable();
        }
        xmax = xmax.max(end);
        ymin = row
            .samples
            .iter()
            .map(|s| s.total)
            .fold(ymin.min(low).min(point), f64::min);
        ymax = row
            .samples
            .iter()
            .map(|s| s.total)
            .fold(ymax.max(high).max(point), f64::max);
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

/// Render existing fits through a display transformation. The formatter must
/// provide `scale_intervals` or be monotone over the full range of totals.
/// Observed values and a sampled model curve share one formatter call and unit.
/// The original slopes, confidence intervals and fit diagnostics are unchanged.
/// This does not enable nonlinear formatting of other statistical report fields.
pub fn formatted_comparison_figure(
    series: &[Series<'_>],
    title: &str,
    formatter: &dyn crate::measurement::ValueFormatter,
) -> Result<String> {
    let (rows, unit) = display_coordinates(series, formatter)?;
    render_coordinates(
        &rows,
        &series.iter().map(|s| s.label).collect::<Vec<_>>(),
        title,
        &unit,
    )
}

fn render_coordinates(
    rows: &[DisplayCoordinates],
    labels: &[&str],
    title: &str,
    unit: &str,
) -> Result<String> {
    use crate::viz::{Domain, Palette, Plot};

    let mut xmax = 0.0f64;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for row in rows {
        for &(x, y) in row.samples.iter().chain(&row.curve).chain(&row.band) {
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
    }
    if !(ymax - ymin).is_finite() {
        return Err(error("formatted regression range is not finite"));
    }
    let mut plot = Plot::new(title)
        .x(Domain::Linear { min: 0., max: xmax })
        .y(Domain::Linear { min: ymin, max: ymax })
        .notes(format!("X: operations per batch. Y: transformed total ({unit}). Curves and shading are the display transformation of the original through-origin model and its slope confidence bounds, not a refit or a prediction interval. Curves are sampled; each process is fitted independently."));
    if labels.len() > 1 {
        plot = plot.margins(64., 16., 30., 34.);
        plot.legend_wrapped(
            &labels
                .iter()
                .enumerate()
                .map(|(i, row)| (row.to_string(), Palette::LIGHT.series(i)))
                .collect::<Vec<_>>(),
        );
    }
    for (i, (series, row)) in labels.iter().zip(rows).enumerate() {
        let color = Palette::LIGHT.series(i);
        plot.polygon(
            &row.band,
            color,
            &format!("{}: transformed slope confidence bounds", series),
        );
        plot.path_labeled(&row.curve, color, 2., series);
        plot.points_labeled(&row.samples, color, 2.5, series);
    }
    Ok(plot.figure())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DisplayCoordinates {
    samples: Vec<(f64, f64)>,
    curve: Vec<(f64, f64)>,
    band: Vec<(f64, f64)>,
}

/// Serializable regression display, bound to its original paired samples and fit.
/// Rendering needs no formatter instance. Raw statistics stay in the source report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Presentation {
    source_sha256: String,
    unit: String,
    coordinates: DisplayCoordinates,
}

fn source_hash(samples: &[Sample], fit: &Fit) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(crate::model::hex(&Sha256::digest(serde_json::to_vec(&(
        samples, fit,
    ))?)))
}

/// Capture all process displays on one shared formatter scale. The returned order
/// matches the input series. No partially formatted result is returned on error.
pub fn format_series(
    series: &[Series<'_>],
    formatter: &dyn crate::measurement::ValueFormatter,
) -> Result<Vec<Presentation>> {
    let (coordinates, unit) = display_coordinates(series, formatter)?;
    series
        .iter()
        .zip(coordinates)
        .map(|(series, coordinates)| {
            Ok(Presentation {
                source_sha256: source_hash(series.samples, series.fit)?,
                unit: unit.clone(),
                coordinates,
            })
        })
        .collect()
}

impl Presentation {
    /// Validate saved coordinates and reject a snapshot from a different source.
    pub fn figure(
        &self,
        samples: &[Sample],
        fit: &Fit,
        label: &str,
        title: &str,
    ) -> Result<String> {
        self.validate(samples, fit)?;
        render_coordinates(
            std::slice::from_ref(&self.coordinates),
            &[label],
            title,
            &self.unit,
        )
    }

    fn validate(&self, samples: &[Sample], fit: &Fit) -> Result<()> {
        if self.source_sha256 != source_hash(samples, fit)? {
            return Err(error(
                "saved regression presentation does not match source samples or fit",
            ));
        }
        let c = &self.coordinates;
        let end = samples.iter().map(|s| s.operations).fold(0., f64::max);
        let n = c.curve.len();
        if self.unit.trim().is_empty()
            || samples.is_empty()
            || n < 2
            || c.samples.len() != samples.len()
            || c.band.len() != n * 2
            || c.samples
                .iter()
                .zip(samples)
                .any(|(p, s)| p.0 != s.operations)
            || c.curve[0].0 != 0.
            || c.curve[n - 1].0 != end
            || c.curve.windows(2).any(|p| p[0].0 >= p[1].0)
            || c.samples
                .iter()
                .chain(&c.curve)
                .chain(&c.band)
                .any(|&(x, y)| !x.is_finite() || !y.is_finite() || x < 0. || x > end)
            || samples.iter().any(|s| {
                c.curve
                    .binary_search_by(|p| p.0.total_cmp(&s.operations))
                    .is_err()
            })
            || c.curve.iter().enumerate().any(|(i, &(x, _))| {
                let low = c.band[i];
                let high = c.band[2 * n - i - 1];
                low.0 != x || high.0 != x || low.1 > high.1
            })
        {
            return Err(error(
                "saved regression presentation has invalid coordinates",
            ));
        }
        Ok(())
    }
}

/// A complete shared-scale comparison that renders without the original formatter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComparisonPresentation {
    series: Vec<SavedSeries>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SavedSeries {
    label: String,
    samples: Vec<Sample>,
    fit: Fit,
    presentation: Presentation,
}
impl ComparisonPresentation {
    pub fn new(
        series: &[Series<'_>],
        formatter: &dyn crate::measurement::ValueFormatter,
    ) -> Result<Self> {
        let presentations = format_series(series, formatter)?;
        let result = Self {
            series: series
                .iter()
                .zip(presentations)
                .map(|(row, presentation)| SavedSeries {
                    label: row.label.into(),
                    samples: row.samples.to_vec(),
                    fit: row.fit.clone(),
                    presentation,
                })
                .collect(),
        };
        result.validate()?;
        Ok(result)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let unit = &self
            .series
            .first()
            .ok_or_else(|| error("empty saved regression comparison"))?
            .presentation
            .unit;
        let (mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY);
        for row in &self.series {
            row.presentation.validate(&row.samples, &row.fit)?;
            if &row.presentation.unit != unit {
                return Err(error("saved regression comparison units differ"));
            }
            let c = &row.presentation.coordinates;
            for &(_, y) in c.samples.iter().chain(&c.curve).chain(&c.band) {
                ymin = ymin.min(y);
                ymax = ymax.max(y);
            }
        }
        if !(ymax - ymin).is_finite() {
            return Err(error("formatted regression range is not finite"));
        }
        Ok(())
    }

    pub fn figure(&self, title: &str) -> Result<String> {
        self.validate()?;
        let unit = &self.series[0].presentation.unit;
        render_coordinates(
            &self
                .series
                .iter()
                .map(|r| r.presentation.coordinates.clone())
                .collect::<Vec<_>>(),
            &self
                .series
                .iter()
                .map(|r| r.label.as_str())
                .collect::<Vec<_>>(),
            title,
            unit,
        )
    }
}

fn display_coordinates(
    series: &[Series<'_>],
    formatter: &dyn crate::measurement::ValueFormatter,
) -> Result<(Vec<DisplayCoordinates>, String)> {
    if series.is_empty() {
        return Err(error("formatted regression needs at least one series"));
    }
    let mut rows = Vec::with_capacity(series.len());
    for row in series {
        let slope = &row.fit.slope;
        if row.samples.is_empty()
            || row
                .samples
                .iter()
                .any(|s| !s.operations.is_finite() || s.operations <= 0. || !s.total.is_finite())
            || ![slope.lower, slope.point, slope.upper]
                .iter()
                .all(|v| v.is_finite())
            || slope.lower > slope.upper
        {
            return Err(error(
                "formatted regression has invalid samples or slope bounds",
            ));
        }
        let end = row.samples.iter().map(|s| s.operations).fold(0., f64::max);
        // Include every observed abscissa so the curve agrees at each observed x.
        let mut xs: Vec<_> = (0..=64).map(|i| end * (f64::from(i) / 64.)).collect();
        xs.extend(row.samples.iter().map(|s| s.operations));
        xs.sort_by(f64::total_cmp);
        xs.dedup();
        let curve = xs.iter().map(|&x| (x, slope.point * x)).collect();
        let band = xs
            .iter()
            .map(|&x| (x, slope.lower * x))
            .chain(xs.iter().rev().map(|&x| (x, slope.upper * x)))
            .collect();
        rows.push(DisplayCoordinates {
            samples: row
                .samples
                .iter()
                .map(|s| (s.operations, s.total))
                .collect(),
            curve,
            band,
        });
    }
    let intervals: Vec<_> = rows
        .iter()
        .flat_map(|row| {
            let n = row.curve.len();
            (0..n).map(move |i| [row.band[i].1, row.band[2 * n - i - 1].1])
        })
        .collect();
    let mut coordinates: Vec<_> = rows
        .iter_mut()
        .flat_map(|row| {
            row.samples
                .iter_mut()
                .chain(&mut row.curve)
                .chain(&mut row.band)
        })
        .collect();
    let raw: Vec<_> = coordinates.iter().map(|p| p.1).collect();
    if raw.iter().any(|v| !v.is_finite()) {
        return Err(error("regression model overflow before formatting"));
    }
    let typical = raw.iter().map(|v| v.abs()).fold(0., f64::max);
    let scaled = formatter.scale_values(typical, &raw)?;
    if scaled.unit.trim().is_empty()
        || scaled.values.len() != raw.len()
        || scaled.values.iter().any(|v| !v.is_finite())
    {
        return Err(error(
            "regression formatter must return finite values and one nonempty unit",
        ));
    }
    // This checks all evaluated values; monotonicity between them is the
    // formatter's contract, as no finite sample can prove it for an arbitrary fn.
    let mut order: Vec<_> = (0..raw.len()).collect();
    order.sort_by(|&a, &b| raw[a].total_cmp(&raw[b]));
    let increasing = order
        .windows(2)
        .all(|w| scaled.values[w[0]] <= scaled.values[w[1]]);
    let decreasing = order
        .windows(2)
        .all(|w| scaled.values[w[0]] >= scaled.values[w[1]]);
    let supplied_bounds = formatter.scale_intervals(typical, &intervals)?;
    if (supplied_bounds.is_none() && !increasing && !decreasing)
        || order
            .windows(2)
            .any(|w| raw[w[0]] == raw[w[1]] && scaled.values[w[0]] != scaled.values[w[1]])
    {
        return Err(error(
            "regression formatter must be monotone or provide scale_intervals, and preserve equal values",
        ));
    }
    for (point, value) in coordinates.iter_mut().zip(scaled.values) {
        point.1 = value;
    }
    drop(coordinates);
    // Keep polygon storage lower-forward/upper-backward after reversing a scale.
    for row in &mut rows {
        let n = row.curve.len();
        for i in 0..n {
            let opposite = 2 * n - i - 1;
            if row.band[i].1 > row.band[opposite].1 {
                row.band.swap(i, opposite);
            }
        }
    }
    if let Some(bounds) = supplied_bounds {
        if bounds.unit != scaled.unit
            || bounds.bounds.len() != intervals.len()
            || bounds
                .bounds
                .iter()
                .any(|b| !b[0].is_finite() || !b[1].is_finite() || b[0] > b[1])
        {
            return Err(error(
                "regression scale_intervals must return finite ordered bounds in the shared unit",
            ));
        }
        let mut bounds = bounds.bounds.into_iter();
        for row in &mut rows {
            let n = row.curve.len();
            for i in 0..n {
                let opposite = 2 * n - i - 1;
                let [low, high] = bounds.next().unwrap();
                if low > row.band[i].1 || high < row.band[opposite].1 {
                    return Err(error(
                        "regression scale_intervals must contain transformed endpoints",
                    ));
                }
                row.band[i].1 = low;
                row.band[opposite].1 = high;
            }
        }
    }
    Ok((rows, scaled.unit))
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
    let variation = normalized
        .iter()
        .map(|s| (s.total - mean).powi(2))
        .sum::<f64>();
    let goodness = |candidate: f64| {
        let residual = normalized
            .iter()
            .map(|s| (s.total - candidate * s.operations).powi(2))
            .sum::<f64>();
        (variation > 0.0 && samples.iter().any(|s| s.total != samples[0].total))
            .then(|| 1.0 - residual / variation)
            .filter(|v| v.is_finite())
    };
    let r_squared = goodness(point);
    let mut distribution = crate::bootstrap::resample_buffer(config.resamples)?;
    let draw = |count, mut rng: Seeded, output: &mut Vec<f64>| -> Result<()> {
        let mut sample = vec![normalized[0]; samples.len()];
        for _ in 0..count {
            for value in &mut sample {
                *value = normalized[crate::bootstrap::draw_index(&mut rng, samples.len())];
            }
            let value = slope(&sample);
            if !value.is_finite() {
                return Err(error(
                    "bootstrap regression produced an unrepresentable slope",
                ));
            }
            output.push(value);
        }
        Ok(())
    };
    let workers = crate::parallel_analysis::workers(config.resamples, samples.len());
    if workers == 1 {
        draw(
            config.resamples,
            Seeded::new(config.seed),
            &mut distribution,
        )?;
    } else {
        for chunk in crate::parallel_analysis::chunks(
            config.resamples,
            config.seed,
            &[samples.len()],
            workers,
            |count, rng| {
                let mut output = crate::bootstrap::resample_buffer(count)?;
                draw(count, rng, &mut output)?;
                Ok(output)
            },
        )? {
            distribution.extend(chunk);
        }
    }
    let average = distribution.iter().sum::<f64>() / distribution.len() as f64;
    let standard_error =
        (distribution.len() > 1).then(|| crate::bootstrap::deviation(&distribution, average));
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
    let bounds = [
        crate::bootstrap::quantile(&distribution, tail),
        crate::bootstrap::quantile(&distribution, 1.0 - tail),
    ];
    let r_squared_at_slope_bounds = goodness(bounds[0])
        .zip(goodness(bounds[1]))
        .map(|(low, high)| [low, high]);
    let estimate = Estimate {
        point: rescale(point),
        lower: rescale(bounds[0]),
        upper: rescale(bounds[1]),
        standard_error: standard_error.map(rescale),
    };
    if [estimate.point, estimate.lower, estimate.upper]
        .iter()
        .chain(estimate.standard_error.iter())
        .any(|v| !v.is_finite())
    {
        return Err(error("regression estimate exceeds finite numeric range"));
    }
    Ok((
        Fit {
            slope: estimate,
            r_squared,
            r_squared_at_slope_bounds,
            samples: samples.len(),
        },
        retained,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Transform(fn(f64) -> f64);
    impl crate::measurement::ValueFormatter for Transform {
        fn scale_values(
            &self,
            _: f64,
            values: &[f64],
        ) -> Result<crate::measurement::FormattedValues> {
            Ok(crate::measurement::FormattedValues {
                values: values.iter().map(|&v| (self.0)(v)).collect(),
                unit: "display<&>".into(),
            })
        }
        fn scale_throughputs(
            &self,
            _: f64,
            _: f64,
            _: &str,
            _: &[f64],
        ) -> Result<crate::measurement::FormattedValues> {
            unreachable!()
        }
        fn scale_for_machines(&self, _: &[f64]) -> Result<crate::measurement::FormattedValues> {
            unreachable!()
        }
    }

    #[test]
    fn nonmonotone_interval_images_preserve_interior_extrema_and_saved_geometry() {
        struct Fold(u8);
        impl crate::measurement::ValueFormatter for Fold {
            fn scale_values(
                &self,
                t: f64,
                values: &[f64],
            ) -> Result<crate::measurement::FormattedValues> {
                Transform(|v| (v - 1.).powi(2)).scale_values(t, values)
            }
            fn scale_intervals(
                &self,
                _: f64,
                intervals: &[[f64; 2]],
            ) -> Result<Option<crate::measurement::FormattedIntervals>> {
                let mut result = crate::measurement::FormattedIntervals {
                    bounds: intervals
                        .iter()
                        .map(|&[a, b]| {
                            let low = if a <= 1. && b >= 1. {
                                0.
                            } else {
                                (a - 1.).powi(2).min((b - 1.).powi(2))
                            };
                            [low, (a - 1.).powi(2).max((b - 1.).powi(2))]
                        })
                        .collect(),
                    unit: "display<&>".into(),
                };
                match self.0 {
                    1 => {
                        result.bounds.pop();
                    }
                    2 => result.unit = "wrong".into(),
                    3 => result.bounds[0] = [2., 1.],
                    4 => result.bounds[0][0] = f64::NAN,
                    5 => result.bounds[0] = [0., 0.],
                    _ => {}
                }
                Ok(Some(result))
            }
            fn scale_throughputs(
                &self,
                _: f64,
                _: f64,
                _: &str,
                _: &[f64],
            ) -> Result<crate::measurement::FormattedValues> {
                unreachable!()
            }
            fn scale_for_machines(&self, _: &[f64]) -> Result<crate::measurement::FormattedValues> {
                unreachable!()
            }
        }
        let samples = [
            Sample {
                operations: 1.,
                total: 1.,
            },
            Sample {
                operations: 2.,
                total: 2.,
            },
        ];
        let fit = Fit {
            slope: Estimate {
                point: 1.,
                lower: 0.5,
                upper: 1.5,
                standard_error: Some(0.2),
            },
            r_squared: Some(1.),
            r_squared_at_slope_bounds: None,
            samples: 2,
        };
        let series = [Series {
            label: "fold",
            samples: &samples,
            fit: &fit,
        }];
        let before = serde_json::to_vec(&fit).unwrap();
        let saved = format_series(&series, &Fold(0)).unwrap();
        let row = &saved[0].coordinates;
        let i = row.curve.iter().position(|p| p.0 == 1.).unwrap();
        assert_eq!(row.curve[i], (1., 0.));
        assert_eq!(row.band[i], (1., 0.));
        assert_eq!(row.band[row.band.len() - i - 1], (1., 0.25));
        assert_eq!(row.curve[0], (0., 1.));
        let restored: Vec<Presentation> =
            serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert_eq!(
            saved[0].figure(&samples, &fit, "fold", "fold").unwrap(),
            restored[0].figure(&samples, &fit, "fold", "fold").unwrap()
        );
        assert_eq!(serde_json::to_vec(&fit).unwrap(), before);
        for invalid in 1..=5 {
            assert!(
                format_series(&series, &Fold(invalid)).is_err(),
                "invalid bounds {invalid}"
            );
        }
        assert!(
            format_series(&series, &Transform(|v| (v - 1.).powi(2)))
                .unwrap_err()
                .to_string()
                .contains("scale_intervals")
        );
    }

    #[test]
    fn nonlinear_regression_transforms_model_at_observed_x_without_refitting() {
        let samples = [
            Sample {
                operations: 1.,
                total: 1.,
            },
            Sample {
                operations: 2.,
                total: 2.,
            },
        ];
        let fit = Fit {
            slope: Estimate {
                point: 1.,
                lower: 0.5,
                upper: 1.5,
                standard_error: Some(0.2),
            },
            r_squared: Some(1.),
            r_squared_at_slope_bounds: None,
            samples: 2,
        };
        let original = serde_json::to_value(&fit).unwrap();
        let series = [Series {
            label: "process<&>",
            samples: &samples,
            fit: &fit,
        }];
        let (rows, unit) = display_coordinates(&series, &Transform(|v| v * v)).unwrap();
        assert_eq!(unit, "display<&>");
        assert_eq!(rows[0].samples, [(1., 1.), (2., 4.)]);
        assert!(rows[0].curve.contains(&(1., 1.)));
        assert!(rows[0].curve.contains(&(2., 4.)));
        assert!(rows[0].band.contains(&(1., 0.25)));
        assert!(rows[0].band.contains(&(1., 2.25)));
        let (shifted, _) = display_coordinates(&series, &Transform(|v| v - 10.)).unwrap();
        assert_eq!(shifted[0].curve[0], (0., -10.));
        assert_eq!(shifted[0].samples, [(1., -9.), (2., -8.)]);
        let reverse = Transform(|v| 100. - v);
        let (reversed, _) = display_coordinates(&series, &reverse).unwrap();
        assert_eq!(reversed[0].samples, [(1., 99.), (2., 98.)]);
        let n = reversed[0].curve.len();
        for (i, &(x, y)) in reversed[0].curve.iter().enumerate() {
            assert_eq!(y, 100. - x);
            assert_eq!(reversed[0].band[i], (x, 100. - x * 1.5));
            assert_eq!(reversed[0].band[2 * n - i - 1], (x, 100. - x * 0.5));
        }
        let paired = [
            Series {
                label: "baseline",
                samples: &samples,
                fit: &fit,
            },
            Series {
                label: "candidate",
                samples: &samples,
                fit: &fit,
            },
        ];
        let comparison = ComparisonPresentation::new(&paired, &Transform(|v| -v)).unwrap();
        let encoded = serde_json::to_vec(&comparison).unwrap();
        let restored_comparison: ComparisonPresentation = serde_json::from_slice(&encoded).unwrap();
        restored_comparison.validate().unwrap();
        for row in &restored_comparison.series {
            assert_eq!(row.presentation.coordinates.samples, [(1., -1.), (2., -2.)]);
            assert_eq!(row.fit.slope.point, 1.);
        }
        let html = restored_comparison.figure("negative comparison").unwrap();
        assert!(html.contains("baseline") && html.contains("candidate"));
        let saved = format_series(&series, &reverse).unwrap();
        let restored: Vec<Presentation> =
            serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert!(
            restored[0]
                .figure(&samples, &fit, "p", "reverse")
                .unwrap()
                .contains("<svg")
        );
        assert_eq!(serde_json::to_value(&fit).unwrap(), original);
        let html = formatted_comparison_figure(&series, "model", &Transform(|v| v * v)).unwrap();
        assert!(html.contains("<svg"));
        assert!(html.contains("display&lt;&amp;&gt;"));
        assert!(html.contains("not a refit"));
    }

    #[test]
    fn saved_regression_display_roundtrips_in_bootstrap_report_and_rejects_stale_sources() {
        let samples = vec![
            Sample {
                operations: 1.,
                total: 1.,
            },
            Sample {
                operations: 2.,
                total: 2.,
            },
        ];
        let config = Config {
            resamples: 32,
            ..Config::default()
        };
        let fit = fit(&samples, &config).unwrap();
        let mut metric = crate::Metric::duration("work", "test", "batch_total");
        metric.unit = "units".into();
        let report = crate::bootstrap::Report {
            presentation: Vec::new(),
            config,
            case_configs: Default::default(),
            method: "test".into(),
            rows: vec![crate::bootstrap::Row {
                case: "case".into(),
                metric: metric.id.clone(),
                variant: "candidate".into(),
                unit: metric.unit.clone(),
                metric_contract: Some(metric.clone()),
                resampling_unit: "process".into(),
                units: 2,
                estimates: None,
                throughput: vec![],
                work_counters: Default::default(),
                distributions: None,
                regressions: vec![crate::bootstrap::ProcessRegression {
                    process: 0,
                    fit,
                    samples,
                    slope_distribution: None,
                    presentation: None,
                }],
                outliers: None,
                note: String::new(),
            }],
        };
        let mut multiple = report.clone();
        let mut other = report.rows[0].clone();
        other.case = "other".into();
        multiple.rows.push(other);
        let multiple = crate::statistic_format::format(multiple, &Transform(|v| v * v)).unwrap();
        assert_eq!(multiple.presentation.len(), 2);
        let selected = crate::measurement::format_regression_metric(
            &multiple,
            "case",
            &metric,
            &Transform(|v| v * v),
        )
        .unwrap();
        assert_eq!(selected.rows.len(), 1);
        assert_eq!(selected.presentation.len(), 1);
        assert_eq!(selected.presentation[0].case, "case");
        assert!(!crate::bootstrap::html(&selected).contains("statistical display unavailable"));
        assert_eq!(multiple.presentation.len(), 2);
        let absent = crate::measurement::format_regression_metric(
            &multiple,
            "absent",
            &metric,
            &Transform(|v| v * v),
        )
        .unwrap();
        assert!(absent.rows.is_empty());
        assert!(absent.presentation.is_empty());
        let mut candidate = report.clone();
        let candidate_regression = &mut candidate.rows[0].regressions[0];
        for sample in &mut candidate_regression.samples {
            sample.total *= 2.;
        }
        candidate_regression.fit =
            super::fit(&candidate_regression.samples, &candidate.config).unwrap();
        let comparison = crate::measurement::regression_comparison_chart(
            &report,
            &candidate,
            "case",
            &metric,
            &Transform(|v| v * v),
        )
        .unwrap();
        assert!(comparison.contains("baseline process 0: x = 1, y = 1"));
        assert!(comparison.contains("candidate process 0: x = 1, y = 4"));
        assert!(comparison.contains("Original slope confidence levels"));
        candidate.rows[0].metric_contract.as_mut().unwrap().unit = "different".into();
        assert!(
            crate::measurement::regression_comparison_chart(
                &report,
                &candidate,
                "case",
                &metric,
                &Transform(|v| v * v),
            )
            .is_err()
        );
        let original = serde_json::to_value(&report).unwrap();
        let formatted = crate::measurement::format_regression_metric(
            &report,
            "case",
            &metric,
            &Transform(|v| v * v),
        )
        .unwrap();
        assert_eq!(serde_json::to_value(&report).unwrap(), original);
        let json = serde_json::to_vec(&formatted).unwrap();
        let mut restored: crate::bootstrap::Report = serde_json::from_slice(&json).unwrap();
        let html = crate::bootstrap::html(&restored);
        assert!(html.contains("transformed total (display&lt;&amp;&gt;)"));
        assert!(!html.contains("saved regression display unavailable"));
        assert_eq!(restored.rows[0].unit, "units");
        let mut without_display = restored.clone();
        without_display.rows[0].regressions[0].presentation = None;
        assert_eq!(serde_json::to_value(&without_display).unwrap(), original);
        // Replacing numeric report units must clear coordinates bound to old units.
        let linear = crate::measurement::format_bootstrap_metric(
            &restored,
            "case",
            &metric,
            &Transform(|v| v * 2.),
        )
        .unwrap();
        assert!(linear.rows[0].regressions[0].presentation.is_none());
        restored.rows[0].regressions[0].samples[0].total = 3.;
        assert!(crate::bootstrap::html(&restored).contains("saved regression display unavailable"));
        let mut malformed = serde_json::to_value(&formatted).unwrap();
        malformed["rows"][0]["regressions"][0]["presentation"]["coordinates"]["band"] =
            serde_json::json!([]);
        let malformed: crate::bootstrap::Report = serde_json::from_value(malformed).unwrap();
        assert!(crate::bootstrap::html(&malformed).contains("invalid coordinates"));
        let mut wrong_metric = metric.clone();
        wrong_metric.unit = "different".into();
        assert!(
            crate::measurement::format_regression_metric(
                &report,
                "case",
                &wrong_metric,
                &Transform(|v| v)
            )
            .is_err()
        );
    }

    #[test]
    fn comparison_snapshot_rejects_unrepresentable_range_before_rendering() {
        let samples = [
            Sample {
                operations: 1.,
                total: 1.,
            },
            Sample {
                operations: 2.,
                total: 2.,
            },
        ];
        let fit = Fit {
            slope: Estimate {
                point: 1.,
                lower: 0.5,
                upper: 1.5,
                standard_error: None,
            },
            r_squared: None,
            r_squared_at_slope_bounds: None,
            samples: 2,
        };
        let error = ComparisonPresentation::new(
            &[Series {
                label: "p",
                samples: &samples,
                fit: &fit,
            }],
            &Transform(|v| if v == 0. { -f64::MAX } else { f64::MAX }),
        )
        .unwrap_err();
        assert!(error.to_string().contains("range is not finite"));
    }

    #[test]
    fn confidence_bounds_need_not_contain_the_point_estimate() {
        let samples = [
            Sample {
                operations: 1.,
                total: 3.,
            },
            Sample {
                operations: 2.,
                total: 6.,
            },
        ];
        let fit = Fit {
            slope: Estimate {
                point: 3.,
                lower: 1.,
                upper: 2.,
                standard_error: None,
            },
            r_squared: Some(1.),
            r_squared_at_slope_bounds: None,
            samples: 2,
        };
        assert!(!figure(&samples, &fit, "external interval", "units").contains("unavailable"));
        let series = [Series {
            label: "p",
            samples: &samples,
            fit: &fit,
        }];
        let (rows, _) = display_coordinates(&series, &Transform(|v| v * v)).unwrap();
        assert!(rows[0].curve.contains(&(2., 36.)));
        assert!(rows[0].band.contains(&(2., 16.)));
        let saved = format_series(&series, &Transform(|v| v * v)).unwrap();
        assert!(
            saved[0]
                .figure(&samples, &fit, "p", "external interval")
                .is_ok()
        );
    }

    #[test]
    fn nonlinear_regression_rejects_invalid_transform_and_model() {
        let samples = [
            Sample {
                operations: 1.,
                total: 1.,
            },
            Sample {
                operations: 2.,
                total: 2.,
            },
        ];
        let mut fit = Fit {
            slope: Estimate {
                point: 1.,
                lower: 0.5,
                upper: 1.5,
                standard_error: None,
            },
            r_squared: None,
            r_squared_at_slope_bounds: None,
            samples: 2,
        };
        for transform in [Transform(|_| f64::NAN), Transform(|v| (v - 1.).powi(2))] {
            assert!(
                formatted_comparison_figure(
                    &[Series {
                        label: "p",
                        samples: &samples,
                        fit: &fit
                    }],
                    "model",
                    &transform
                )
                .is_err()
            );
        }
        fit.slope.upper = f64::MAX;
        assert!(
            formatted_comparison_figure(
                &[Series {
                    label: "p",
                    samples: &samples,
                    fit: &fit
                }],
                "model",
                &Transform(|v| v)
            )
            .is_err()
        );
        assert!(formatted_comparison_figure(&[], "model", &Transform(|v| v)).is_err());
    }

    #[test]
    fn parallel_regression_preserves_every_sequential_draw() {
        let samples: Vec<_> = (1..=37)
            .map(|n| Sample {
                operations: n as f64,
                total: (n * 5 + (n * 17) % 19) as f64,
            })
            .collect();
        let config = Config {
            resamples: 2051,
            seed: 91,
            ..Config::default()
        };
        let (_, draws) = fit_with_distribution(&samples, &config).unwrap();
        let xscale = samples.iter().map(|s| s.operations).fold(0.0, f64::max);
        let yscale = samples.iter().map(|s| s.total).fold(0.0, f64::max);
        let normalized: Vec<_> = samples
            .iter()
            .map(|s| Sample {
                operations: s.operations / xscale,
                total: s.total / yscale,
            })
            .collect();
        let mut rng = Seeded::new(config.seed);
        let mut sample = vec![normalized[0]; samples.len()];
        let expected: Vec<_> = (0..config.resamples)
            .map(|_| {
                for value in &mut sample {
                    *value = normalized[crate::bootstrap::draw_index(&mut rng, samples.len())];
                }
                slope(&sample) * yscale / xscale
            })
            .collect();
        assert_eq!(draws, expected);
    }

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
                standard_error: Some(0.5),
            },
            r_squared: Some(1.),
            r_squared_at_slope_bounds: Some([1., 1.]),
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
            presentation: None,
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
        assert!(result.slope.standard_error.unwrap() < 1e-12);
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

#[cfg(test)]
mod endpoint_tests {
    use super::*;
    #[test]
    fn slope_endpoint_goodness_matches_residuals_and_is_scale_invariant() {
        let samples = [
            Sample {
                operations: 1.,
                total: 11.,
            },
            Sample {
                operations: 2.,
                total: 21.,
            },
            Sample {
                operations: 4.,
                total: 39.,
            },
            Sample {
                operations: 8.,
                total: 83.,
            },
        ];
        let config = Config {
            resamples: 256,
            ..Default::default()
        };
        let result = fit(&samples, &config).unwrap();
        let endpoints = result.r_squared_at_slope_bounds.unwrap();
        let mean = samples.iter().map(|s| s.total).sum::<f64>() / 4.;
        let variation = samples
            .iter()
            .map(|s| (s.total - mean).powi(2))
            .sum::<f64>();
        for (slope, actual) in [result.slope.lower, result.slope.upper]
            .into_iter()
            .zip(endpoints)
        {
            let residual = samples
                .iter()
                .map(|s| (s.total - slope * s.operations).powi(2))
                .sum::<f64>();
            assert!((actual - (1. - residual / variation)).abs() < 1e-12);
            assert!(actual <= result.r_squared.unwrap() + 1e-12);
        }
        for factor in [1e-200, 1e200] {
            let scaled: Vec<_> = samples
                .iter()
                .map(|s| Sample {
                    operations: s.operations * factor,
                    total: s.total * factor,
                })
                .collect();
            let scaled = fit(&scaled, &config)
                .unwrap()
                .r_squared_at_slope_bounds
                .unwrap();
            assert!(
                scaled
                    .iter()
                    .zip(endpoints)
                    .all(|(a, b)| (a - b).abs() < 1e-12)
            );
        }
        let encoded = serde_json::to_value(&result).unwrap();
        let restored: Fit = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(restored.r_squared_at_slope_bounds, Some(endpoints));
        let mut legacy = encoded;
        legacy
            .as_object_mut()
            .unwrap()
            .remove("r_squared_at_slope_bounds");
        assert!(
            serde_json::from_value::<Fit>(legacy)
                .unwrap()
                .r_squared_at_slope_bounds
                .is_none()
        );
    }
    #[test]
    fn constant_totals_remain_undefined_and_negative_goodness_is_not_clamped() {
        let config = Config {
            resamples: 64,
            ..Default::default()
        };
        let mut samples = [
            Sample {
                operations: 1.,
                total: 10.,
            },
            Sample {
                operations: 2.,
                total: 10.,
            },
            Sample {
                operations: 3.,
                total: 10.,
            },
        ];
        let constant = fit(&samples, &config).unwrap();
        assert_eq!(constant.r_squared, None);
        assert_eq!(constant.r_squared_at_slope_bounds, None);
        samples[1].total = 11.;
        let result = fit(&samples, &config).unwrap();
        assert!(
            result
                .r_squared_at_slope_bounds
                .unwrap()
                .iter()
                .all(|value| *value < 0.)
        );
    }
}
