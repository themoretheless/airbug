//! Anscombe's Quartet & summary-moment blindspot detector (`#112`).
use serde::{Deserialize, Serialize};

/// Canonical Anscombe's Quartet: four 11-point `(x, y)` datasets sharing nearly identical
/// mean, variance, correlation, and OLS regression line despite radically different distributions.
pub const ANSCOMBE_QUARTET: [[(f64, f64); 11]; 4] = [
    [
        (10.0, 8.04),
        (8.0, 6.95),
        (13.0, 7.58),
        (9.0, 8.81),
        (11.0, 8.33),
        (14.0, 9.96),
        (6.0, 7.24),
        (4.0, 4.26),
        (12.0, 10.84),
        (7.0, 4.82),
        (5.0, 5.68),
    ],
    [
        (10.0, 9.14),
        (8.0, 8.14),
        (13.0, 8.74),
        (9.0, 8.77),
        (11.0, 9.26),
        (14.0, 8.10),
        (6.0, 6.13),
        (4.0, 3.10),
        (12.0, 9.13),
        (7.0, 7.26),
        (5.0, 4.74),
    ],
    [
        (10.0, 7.46),
        (8.0, 6.77),
        (13.0, 12.74),
        (9.0, 7.11),
        (11.0, 7.81),
        (14.0, 8.84),
        (6.0, 6.08),
        (4.0, 5.39),
        (12.0, 8.15),
        (7.0, 6.42),
        (5.0, 5.73),
    ],
    [
        (8.0, 6.58),
        (8.0, 5.76),
        (8.0, 7.71),
        (8.0, 8.84),
        (8.0, 8.47),
        (8.0, 7.04),
        (8.0, 5.25),
        (19.0, 12.50),
        (8.0, 5.56),
        (8.0, 7.91),
        (8.0, 6.89),
    ],
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MomentBlindspotReport {
    pub mean_a: f64,
    pub mean_b: f64,
    pub std_a: f64,
    pub std_b: f64,
    pub median_a: f64,
    pub median_b: f64,
    pub ks_distance: f64,
    pub blindspot_detected: bool,
}

fn summary(values: &[f64]) -> Option<(f64, f64, f64)> {
    if values.len() < 2 || values.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let median = sorted[sorted.len() / 2];
    Some((mean, var.sqrt(), median))
}

fn empirical_ks(a: &[f64], b: &[f64]) -> f64 {
    let mut sa = a.to_vec();
    let mut sb = b.to_vec();
    sa.sort_by(f64::total_cmp);
    sb.sort_by(f64::total_cmp);
    let (na, nb) = (sa.len() as f64, sb.len() as f64);
    let (mut ia, mut ib) = (0usize, 0usize);
    let mut max_d = 0.0f64;
    while ia < sa.len() && ib < sb.len() {
        let va = sa[ia];
        let vb = sb[ib];
        let x = va.min(vb);
        while ia < sa.len() && sa[ia] <= x {
            ia += 1;
        }
        while ib < sb.len() && sb[ib] <= x {
            ib += 1;
        }
        max_d = max_d.max((ia as f64 / na - ib as f64 / nb).abs());
    }
    max_d
}

/// `#112` Flag pairs whose means & standard deviations nearly match while their distributions diverge.
pub fn detect_moment_blindspot(a: &[f64], b: &[f64]) -> Option<MomentBlindspotReport> {
    let (mean_a, std_a, median_a) = summary(a)?;
    let (mean_b, std_b, median_b) = summary(b)?;
    let ks_distance = empirical_ks(a, b);
    let mean_scale = mean_a.abs().max(mean_b.abs()).max(1e-9);
    let std_scale = std_a.abs().max(std_b.abs()).max(1e-9);
    let mean_close = ((mean_a - mean_b).abs() / mean_scale) < 0.02;
    let std_close = ((std_a - std_b).abs() / std_scale) < 0.05;
    let shape_diff = ks_distance >= 0.25 || ((median_a - median_b).abs() / mean_scale) >= 0.05;
    Some(MomentBlindspotReport {
        mean_a,
        mean_b,
        std_a,
        std_b,
        median_a,
        median_b,
        ks_distance,
        blindspot_detected: mean_close && std_close && shape_diff,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bimodal_vs_unimodal_blindspot_and_validates_quartet_moments() {
        // Check classical Anscombe y means (~7.50) across all four datasets
        for ds in ANSCOMBE_QUARTET {
            let ys: Vec<f64> = ds.iter().map(|&(_, y)| y).collect();
            let mean_y = ys.iter().sum::<f64>() / ys.len() as f64;
            assert!((mean_y - 7.5009).abs() < 0.01, "mean_y={mean_y}");
        }

        // Unimodal symmetric around 50 vs bimodal [30, 70] sharing mean 50 and std ~21.38.
        let a: Vec<f64> = vec![21.7157, 21.7157, 50.0, 50.0, 50.0, 50.0, 78.2843, 78.2843];
        let b: Vec<f64> = vec![30.0, 30.0, 30.0, 30.0, 70.0, 70.0, 70.0, 70.0];
        let report = detect_moment_blindspot(&a, &b).unwrap();
        assert!(report.blindspot_detected, "report={report:?}");
    }
}
