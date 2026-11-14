//! Goodness-of-fit normality tests (`#110` Kolmogorov–Smirnov, `#111` Jarque–Bera) and
//! ordinary least-squares trend estimation (`#113`).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalityResult {
    pub test: String,
    pub sample_size: usize,
    pub statistic: f64,
    pub p_value: f64,
    pub is_normal_at_05: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OlsTrend {
    pub sample_size: usize,
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
    pub t_stat: f64,
}

/// Abramowitz & Stegun 7.1.26 rational approximation for `erf(x)`.
fn erf_approx(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let ax = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * ax);
    let poly = (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
        + 0.254829592)
        * t;
    sign * (1.0 - poly * (-ax * ax).exp())
}

fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf_approx(z / std::f64::consts::SQRT_2))
}

fn mean_and_std(values: &[f64]) -> Option<(f64, f64)> {
    if values.len() < 3 || values.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt();
    if std <= 0.0 || !std.is_finite() {
        return None;
    }
    Some((mean, std))
}

/// `#110` One-sample Kolmogorov–Smirnov test against `N(mean, std^2)`.
pub fn ks_normality_test(values: &[f64]) -> Option<NormalityResult> {
    let (mean, std) = mean_and_std(values)?;
    let n = values.len();
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    let nf = n as f64;
    let mut d_max = 0.0f64;
    for (i, &x) in sorted.iter().enumerate() {
        let cdf = normal_cdf((x - mean) / std);
        let d_plus = ((i + 1) as f64 / nf - cdf).abs();
        let d_minus = (cdf - (i as f64 / nf)).abs();
        d_max = d_max.max(d_plus).max(d_minus);
    }

    // Asymptotic Kolmogorov survival series for Q_KS(lambda), lambda = (sqrt(n) + 0.12 + 0.11/sqrt(n)) * D.
    let sn = nf.sqrt();
    let lambda = (sn + 0.12 + 0.11 / sn) * d_max;
    let p_value = if lambda <= 0.0 {
        1.0
    } else {
        let mut sum = 0.0;
        for k in 1..=100 {
            let kf = k as f64;
            let term = (-2.0 * kf * kf * lambda * lambda).exp();
            sum += if k % 2 == 1 { term } else { -term };
        }
        (2.0 * sum).clamp(0.0, 1.0)
    };

    Some(NormalityResult {
        test: "kolmogorov_smirnov".into(),
        sample_size: n,
        statistic: d_max,
        p_value,
        is_normal_at_05: p_value >= 0.05,
    })
}

/// `#111` Jarque–Bera skewness/kurtosis normality test (`p = exp(-JB / 2)` for chi-square df=2).
pub fn jarque_bera_test(values: &[f64]) -> Option<NormalityResult> {
    let (mean, _) = mean_and_std(values)?;
    let n = values.len() as f64;
    let m2 = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    if m2 <= 0.0 {
        return None;
    }
    let m3 = values.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / n;
    let m4 = values.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n;

    let skewness = m3 / m2.powf(1.5);
    let excess_kurtosis = (m4 / (m2 * m2)) - 3.0;
    let jb = (n / 6.0) * (skewness * skewness + 0.25 * excess_kurtosis * excess_kurtosis);
    // Chi-square distribution with 2 degrees of freedom has CDF 1 - exp(-x/2).
    let p_value = (-0.5 * jb).exp().clamp(0.0, 1.0);

    Some(NormalityResult {
        test: "jarque_bera".into(),
        sample_size: values.len(),
        statistic: jb,
        p_value,
        is_normal_at_05: p_value >= 0.05,
    })
}

/// `#113` Ordinary least-squares linear trend over sample index `0..n`.
pub fn ols_slope(values: &[f64]) -> Option<OlsTrend> {
    if values.len() < 3 || values.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n = values.len() as f64;
    let mx = (n - 1.0) / 2.0;
    let my = values.iter().sum::<f64>() / n;
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for (i, &y) in values.iter().enumerate() {
        let dx = i as f64 - mx;
        let dy = y - my;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx <= 0.0 {
        return None;
    }
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let rss: f64 = values
        .iter()
        .enumerate()
        .map(|(i, &y)| {
            let r = y - (intercept + slope * i as f64);
            r * r
        })
        .sum();
    let r_squared = if syy > 0.0 {
        (1.0 - rss / syy).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let se_slope = (rss / ((n - 2.0) * sxx)).max(0.0).sqrt();
    let t_stat = if se_slope > 0.0 {
        slope / se_slope
    } else if slope == 0.0 {
        0.0
    } else {
        slope.signum() * f64::INFINITY
    };

    Some(OlsTrend {
        sample_size: values.len(),
        slope,
        intercept,
        r_squared,
        t_stat,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normality_tests_distinguish_bell_shape_from_heavy_skew() {
        // Quantiles sampled cleanly around N(100, 10^2)
        let normal_like = [
            80.0, 86.0, 90.0, 93.0, 95.0, 97.0, 99.0, 100.0, 101.0, 103.0, 105.0, 107.0, 110.0,
            114.0, 120.0,
        ];
        let ks = ks_normality_test(&normal_like).unwrap();
        let jb = jarque_bera_test(&normal_like).unwrap();
        assert!(ks.is_normal_at_05, "ks={ks:?}");
        assert!(jb.is_normal_at_05, "jb={jb:?}");

        let skewed = [
            1.0, 1.0, 1.1, 1.0, 1.0, 1.2, 1.0, 1.0, 1.1, 1.0, 1.0, 1.0, 1.0, 1.0, 100.0, 200.0,
        ];
        let jb_bad = jarque_bera_test(&skewed).unwrap();
        assert!(!jb_bad.is_normal_at_05, "jb_bad={jb_bad:?}");
    }

    #[test]
    fn ols_recovers_exact_linear_drift() {
        let series: Vec<f64> = (0..10).map(|i| 50.0 + 2.5 * i as f64).collect();
        let trend = ols_slope(&series).unwrap();
        assert!((trend.slope - 2.5).abs() < 1e-12);
        assert!((trend.intercept - 50.0).abs() < 1e-12);
        assert!((trend.r_squared - 1.0).abs() < 1e-12);
    }
}
