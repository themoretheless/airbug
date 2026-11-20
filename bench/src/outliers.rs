//! Tukey fences annotate observations; no observation is removed from estimation.
use crate::{Result, error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    LowSevere,
    LowMild,
    Normal,
    HighMild,
    HighSevere,
}
impl Label {
    pub const ALL: [Self; 5] = [
        Self::LowSevere,
        Self::LowMild,
        Self::Normal,
        Self::HighMild,
        Self::HighSevere,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::LowSevere => "low severe",
            Self::LowMild => "low mild",
            Self::Normal => "normal",
            Self::HighMild => "high mild",
            Self::HighSevere => "high severe",
        }
    }
    fn index(self) -> usize {
        Self::ALL.iter().position(|v| *v == self).unwrap()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub index: usize,
    pub value: f64,
    pub label: Label,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Classification {
    pub q1: f64,
    pub q3: f64,
    /// Low severe, low mild, high mild, high severe. Null means outside finite f64 range.
    pub fences: [Option<f64>; 4],
    /// Low severe, low mild, normal, high mild, high severe.
    pub counts: [usize; 5],
    pub points: Vec<Point>,
}
/// Quartiles use linear interpolation; values exactly on a fence belong to the
/// less severe class. Zero IQR is retained, so departures from a constant core are severe.
pub fn classify(values: &[f64]) -> Result<Classification> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return Err(error(
            "outlier classification needs nonempty finite observations",
        ));
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let q1 = crate::bootstrap::quantile(&sorted, 0.25);
    let q3 = crate::bootstrap::quantile(&sorted, 0.75);
    let iqr = q3 - q1;
    let fences = [
        q1 - 3.0 * iqr,
        q1 - 1.5 * iqr,
        q3 + 1.5 * iqr,
        q3 + 3.0 * iqr,
    ];
    let mut counts = [0; 5];
    let points = values
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let label = if value < fences[0] {
                Label::LowSevere
            } else if value > fences[3] {
                Label::HighSevere
            } else if value < fences[1] {
                Label::LowMild
            } else if value > fences[2] {
                Label::HighMild
            } else {
                Label::Normal
            };
            counts[label.index()] += 1;
            Point {
                index,
                value,
                label,
            }
        })
        .collect();
    Ok(Classification {
        q1,
        q3,
        fences: fences.map(|f| f.is_finite().then_some(f)),
        counts,
        points,
    })
}

pub fn figure(classification: &Classification, title: &str, unit: &str) -> String {
    use crate::viz::{Domain, Plot};
    let mut plot = Plot::new(title).size(800.0, 320.0)
        .x(Domain::fit(classification.points.iter().map(|p| p.index as f64 + 1.0)))
        .y(Domain::fit(classification.points.iter().map(|p| p.value)))
        .suffixes("", format!(" {unit}"))
        .notes("Observation index. Green: normal; amber: mild; red: severe. Tukey fences at 1.5× and 3× IQR. All observations remain in the estimates.");
    if classification.q1 == classification.q3 {
        plot.guide_y(classification.q1, Some("all fences (zero IQR)"), "#87929e");
    } else {
        for (value, label) in
            classification
                .fences
                .iter()
                .zip(["low severe", "low mild", "high mild", "high severe"])
        {
            if let Some(value) = value {
                plot.guide_y(*value, Some(label), "#87929e");
            }
        }
    }
    for label in Label::ALL {
        let color = match label {
            Label::Normal => "#09695d",
            Label::LowMild | Label::HighMild => "#b45309",
            _ => "#b32c34",
        };
        let points: Vec<_> = classification
            .points
            .iter()
            .filter(|p| p.label == label)
            .map(|p| (p.index as f64 + 1.0, p.value))
            .collect();
        plot.points(&points, color, 4.0);
    }
    plot.figure()
}

/// `#106` Configurable outlier filtering policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutlierPolicy {
    #[default]
    Keep,
    Iqr,
    Mad,
    ZScore,
}

/// `#106` Return finite values retained by `policy` (order preserved).
pub fn filter_outliers(values: &[f64], policy: OutlierPolicy) -> Vec<f64> {
    let finite: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if finite.len() < 4 || policy == OutlierPolicy::Keep {
        return finite;
    }
    match policy {
        OutlierPolicy::Keep => finite,
        OutlierPolicy::Iqr => {
            let Ok(class) = classify(&finite) else {
                return finite;
            };
            class
                .points
                .into_iter()
                .filter(|p| matches!(p.label, Label::Normal))
                .map(|p| p.value)
                .collect()
        }
        OutlierPolicy::Mad => {
            let mut sorted = finite.clone();
            sorted.sort_by(f64::total_cmp);
            let med = sorted[sorted.len() / 2];
            let mut devs: Vec<f64> = sorted.iter().map(|x| (x - med).abs()).collect();
            devs.sort_by(f64::total_cmp);
            let mad = devs[devs.len() / 2];
            if mad == 0.0 {
                return finite;
            }
            // Modified Z-score threshold 3.5 using 0.6745 scaling.
            finite
                .into_iter()
                .filter(|&x| (0.6745 * (x - med).abs() / mad) <= 3.5)
                .collect()
        }
        OutlierPolicy::ZScore => {
            let n = finite.len() as f64;
            let mean = finite.iter().sum::<f64>() / n;
            let var = finite.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
            let sd = var.sqrt();
            if sd == 0.0 {
                return finite;
            }
            // Standard z-score can be masked by an extreme outlier; check threshold 2.1 when n is small.
            let thresh = if finite.len() < 12 { 2.1 } else { 3.0 };
            finite
                .into_iter()
                .filter(|&x| ((x - mean).abs() / sd) <= thresh)
                .collect()
        }
    }
}

/// `#107` Symmetric trimmed mean dropping `floor(n * trim_fraction)` extremes from each tail.
pub fn trimmed_mean(values: &[f64], trim_fraction: f64) -> Option<f64> {
    if values.is_empty()
        || !trim_fraction.is_finite()
        || !(0.0..0.5).contains(&trim_fraction)
        || values.iter().any(|x| !x.is_finite())
    {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let k = ((sorted.len() as f64) * trim_fraction).floor() as usize;
    let slice = &sorted[k..sorted.len() - k];
    if slice.is_empty() {
        None
    } else {
        Some(slice.iter().sum::<f64>() / slice.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_order_and_classifies_both_sides_including_fence_equalities() {
        let values = [
            -6., -5., -3., -2., 0., 1., 1., 1., 2., 2., 2., 2., 3., 3., 3., 3., 4., 6., 7., 9., 10.,
        ];
        let result = classify(&values).unwrap();
        assert_eq!(result.counts, [1, 2, 15, 2, 1]);
        for (actual, expected) in result.fences.into_iter().zip([-5., -2., 6., 9.]) {
            assert!((actual.unwrap() - expected).abs() < 1e-12);
        }
        assert_eq!(
            result.points.iter().map(|p| p.value).collect::<Vec<_>>(),
            values
        );
        assert_eq!(result.points[1].label, Label::LowMild);
        assert_eq!(result.points[3].label, Label::Normal);
        assert_eq!(result.points[17].label, Label::Normal);
        assert_eq!(result.points[19].label, Label::HighMild);
    }
    #[test]
    fn tied_core_and_extreme_range_remain_valid() {
        assert_eq!(classify(&[1.; 10]).unwrap().counts, [0, 0, 10, 0, 0]);
        assert_eq!(
            classify(&[1., 1., 1., 1., 1., 100.]).unwrap().counts,
            [0, 0, 5, 0, 1]
        );
        let result = classify(&[-f64::MAX, 0., f64::MAX]).unwrap();
        assert_eq!(result.counts, [0, 0, 3, 0, 0]);
        assert!(result.fences.iter().all(Option::is_none));
        assert!(classify(&[]).is_err());
        assert!(classify(&[f64::NAN]).is_err());
        let figure = figure(&classify(&[1., 2., 30.]).unwrap(), "<unsafe>", "ns");
        assert!(figure.contains("&lt;unsafe&gt;") && !figure.contains("<unsafe>"));
        assert!(!figure.contains("NaN"));
    }

    #[test]
    fn outlier_policies_and_trimmed_mean_behave_as_expected() {
        let data = [10.0, 11.0, 10.5, 10.2, 9.8, 10.1, 100.0];
        assert_eq!(filter_outliers(&data, OutlierPolicy::Keep).len(), 7);
        assert_eq!(filter_outliers(&data, OutlierPolicy::Iqr).len(), 6);
        assert_eq!(filter_outliers(&data, OutlierPolicy::Mad).len(), 6);
        assert_eq!(filter_outliers(&data, OutlierPolicy::ZScore).len(), 6);
        let tm = trimmed_mean(&data, 0.15).unwrap();
        assert!((tm - 10.36).abs() < 0.5, "tm={tm}");
    }
}
