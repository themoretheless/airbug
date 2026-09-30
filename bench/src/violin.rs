//! Horizontal peak-normalized Gaussian violin summaries on shared axes.
use crate::{
    Result,
    density::{Density, Series, gaussian},
    report::escape,
    viz::{Domain, Palette, Plot, charts::AxisScale},
};

/// Each lane retains its own population and bandwidth. Width represents density
/// relative to that lane's peak, never sample count. Log axes transform positions,
/// not the population used to estimate the density.
pub fn figure(series: &[Series<'_>], title: &str, unit: &str, scale: AxisScale) -> Result<String> {
    let mut populations = Vec::new();
    let mut xs = Vec::new();
    let mut details = String::from("<dl>");
    for row in series {
        if row.values.iter().any(|v| !v.is_finite()) {
            return Err(crate::error("violin requires finite samples"));
        }
        details.push_str(&format!("<dt>{}</dt><dd>", escape(row.label)));
        if matches!(scale, AxisScale::Logarithmic) && row.values.iter().any(|v| *v <= 0.) {
            populations.push(Density::Empty);
            details.push_str("Unavailable on logarithmic axes: population contains nonpositive values; no subset was plotted.</dd>");
            continue;
        }
        let mut density = gaussian(row.values, 256)?;
        match &mut density {
            Density::Curve {
                points,
                samples,
                bandwidth,
            } => {
                if matches!(scale, AxisScale::Logarithmic) {
                    points.retain(|p| p.0 > 0.);
                }
                xs.extend(points.iter().map(|p| p.0));
                details.push_str(&format!(
                    "{samples} samples; Gaussian bandwidth {bandwidth:.6e} {}.",
                    escape(unit)
                ));
            }
            Density::Constant { value, samples } => {
                xs.push(*value);
                details.push_str(&format!(
                    "{samples} identical samples; point mass at {value:.6e} {}.",
                    escape(unit)
                ));
            }
            Density::Empty => details.push_str("No samples; no density estimate."),
        }
        populations.push(density);
        details.push_str("</dd>");
    }
    details.push_str("</dl>");
    if xs.is_empty() {
        return Ok(format!(
            "<section><h3>{}</h3>{details}</section>",
            escape(title)
        ));
    }
    let x = match scale {
        AxisScale::Linear => Domain::fit(xs),
        AxisScale::Logarithmic => Domain::Log {
            min: xs.iter().copied().fold(f64::INFINITY, f64::min),
            max: xs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        },
    };
    let (lo, hi) = x.ends();
    if !lo.is_finite() || !hi.is_finite() || !(hi - lo).is_finite() {
        return Err(crate::error("violin domain exceeds numeric precision"));
    }
    let mut plot = Plot::new(title).size(960., 150. + series.len() as f64 * 36.)
        .margins(250., 24., 24., 48.)
        .x(x).y(Domain::Categories { labels: series.iter().map(|s| s.label.to_string()).collect() })
        .notes(format!("X: {unit}. Each violin is mirrored and normalized to its own density peak; width does not encode sample count. Gaussian KDE uses original values. Logarithmic axes clip only nonpositive kernel tails, without renormalizing. Constant populations are point masses."));
    for (index, density) in populations.iter().enumerate() {
        let lane = index as f64;
        let color = Palette::LIGHT.series(index);
        plot.lane_label(lane, series[index].label, 11.);
        match density {
            Density::Curve { points, .. } => {
                let peak = points.iter().map(|p| p.1).fold(0., f64::max);
                if peak > 0. {
                    let outline: Vec<_> = points
                        .iter()
                        .map(|&(x, y)| (x, lane + 0.4 * y / peak))
                        .chain(
                            points
                                .iter()
                                .rev()
                                .map(|&(x, y)| (x, lane - 0.4 * y / peak)),
                        )
                        .collect();
                    plot.polygon(&outline, color, series[index].label);
                    let mut closed = outline;
                    closed.push(closed[0]);
                    plot.path_labeled(&closed, color, 1., series[index].label);
                }
            }
            Density::Constant { value, .. } => {
                plot.path_labeled(
                    &[(*value, lane - 0.4), (*value, lane + 0.4)],
                    color,
                    2.,
                    &format!("{}: point mass", series[index].label),
                );
            }
            Density::Empty => {}
        }
    }
    Ok(format!("{}{details}", plot.figure()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symmetric_geometry_is_peak_normalized_independently() {
        let values = [1., 2., 3., 4.];
        let row = Series {
            label: "population",
            values: &values,
        };
        let html = figure(&[row], "Violin", "ns", AxisScale::Linear).unwrap();
        let Density::Curve { points, .. } = gaussian(&values, 256).unwrap() else {
            panic!()
        };
        let peak = points.iter().map(|p| p.1).fold(0., f64::max);
        let plot = Plot::new("oracle")
            .size(960., 186.)
            .margins(250., 24., 24., 48.)
            .x(Domain::fit(points.iter().map(|p| p.0)))
            .y(Domain::Categories {
                labels: vec!["population".into()],
            });
        let at_peak = points.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
        for direction in [-1., 1.] {
            let x = plot.xpx(at_peak.0).unwrap();
            let y = plot.ypx(direction * 0.4 * at_peak.1 / peak).unwrap();
            assert!(
                html.contains(&format!("{x:.2},{y:.2}")),
                "peak geometry missing"
            );
        }
    }

    #[test]
    fn shared_axes_preserve_empty_constant_and_log_invalid_populations() {
        let series = [
            Series {
                label: "<variable>",
                values: &[1., 2., 3., 4.],
            },
            Series {
                label: "constant",
                values: &[10.; 3],
            },
            Series {
                label: "empty",
                values: &[],
            },
            Series {
                label: "signed",
                values: &[-1., 1.],
            },
        ];
        let linear = figure(&series, "Violin", "ns", AxisScale::Linear).unwrap();
        assert_eq!(linear.matches("<polygon").count(), 2);
        assert!(linear.contains("&lt;variable&gt;"));
        assert!(linear.contains("point mass at 1.000000e1"));
        assert!(linear.contains("No samples"));
        let log = figure(&series, "Violin", "ns", AxisScale::Logarithmic).unwrap();
        assert_eq!(log.matches("<polygon").count(), 1);
        assert!(log.contains("no subset was plotted"));
        assert!(!log.contains("NaN"));
        assert!(
            figure(&[], "empty", "ns", AxisScale::Linear)
                .unwrap()
                .contains("<section>")
        );
        assert!(
            figure(
                &[Series {
                    label: "invalid",
                    values: &[f64::NAN]
                }],
                "bad",
                "ns",
                AxisScale::Linear
            )
            .is_err()
        );
    }
}
