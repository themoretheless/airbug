//! Two-sided Welch tests calibrated by resampling the pooled null population.
//! Callers must supply independent observations, not batches from one process.
use crate::{Result, Seeded, error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Config {
    pub resamples: usize,
    pub seed: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            resamples: 10_000,
            seed: 0,
        }
    }
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if self.resamples == 0 {
            return Err(error("hypothesis resamples must be positive"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WelchTest {
    pub statistic: Option<f64>,
    pub p_value: Option<f64>,
    pub baseline_units: usize,
    pub candidate_units: usize,
    pub resamples: usize,
    pub seed: u64,
    pub extreme_resamples: usize,
    pub degenerate_resamples: usize,
    pub unavailable_reason: Option<String>,
}
/// Nonfinite null statistics are represented explicitly in JSON instead of
/// being dropped or silently encoded as null.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum NullStatistic {
    Finite(f64),
    PositiveInfinity,
    NegativeInfinity,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WelchDistribution {
    pub test: WelchTest,
    pub null_statistics: Vec<NullStatistic>,
}
/// Retain every pooled-null draw in generation order for reproducible exports
/// and plots. Unavailable tests produce an empty distribution and an explicit
/// reason in the test result.
pub fn welch_distribution(
    baseline: &[f64],
    candidate: &[f64],
    config: Config,
) -> Result<WelchDistribution> {
    sample_welch(baseline, candidate, config, true)
}

fn distribution_buffer(count: usize) -> Result<Vec<NullStatistic>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|failure| {
        error(format!(
            "cannot allocate hypothesis distribution for {count} resamples: {failure}"
        ))
    })?;
    Ok(values)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    /// Draws in this bin at least as extreme as the observed absolute statistic.
    pub extreme_count: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NullHistogram {
    pub bins: Vec<HistogramBin>,
    pub observed: Option<f64>,
    pub negative_infinity: usize,
    pub positive_infinity: usize,
    pub total: usize,
}
/// Symmetric bins include the observed statistic and every finite null draw.
/// Infinity counts stay separate; extreme counts are computed from individual
/// draws rather than inferred from bin centers that can straddle a threshold.
pub fn histogram(distribution: &WelchDistribution, count: usize) -> Result<NullHistogram> {
    if !(1..=1000).contains(&count) {
        return Err(error("histogram bins must be 1..=1000"));
    }
    let observed = distribution.test.statistic;
    if observed.is_some_and(|value| !value.is_finite()) {
        return Err(error("observed histogram statistic must be finite"));
    }
    let mut extent = observed.map(f64::abs).unwrap_or(0.0);
    for draw in &distribution.null_statistics {
        if let NullStatistic::Finite(value) = draw {
            if !value.is_finite() {
                return Err(error(
                    "nonfinite histogram value must use an explicit infinity variant",
                ));
            }
            extent = extent.max(value.abs());
        }
    }
    if extent == 0.0 {
        extent = 1.0;
    }
    let boundary = |index: usize| (2.0 * (index as f64 / count as f64) - 1.0) * extent;
    let mut result = NullHistogram {
        bins: (0..count)
            .map(|i| HistogramBin {
                lower: boundary(i),
                upper: boundary(i + 1),
                count: 0,
                extreme_count: 0,
            })
            .collect(),
        observed,
        negative_infinity: 0,
        positive_infinity: 0,
        total: distribution.null_statistics.len(),
    };
    for draw in &distribution.null_statistics {
        match draw {
            NullStatistic::NegativeInfinity => result.negative_infinity += 1,
            NullStatistic::PositiveInfinity => result.positive_infinity += 1,
            NullStatistic::Finite(value) => {
                let index = (((value / extent + 1.0) * 0.5 * count as f64) as usize).min(count - 1);
                let bin = &mut result.bins[index];
                bin.count += 1;
                bin.extreme_count +=
                    usize::from(observed.is_some_and(|observed| value.abs() >= observed.abs()));
            }
        }
    }
    Ok(result)
}
fn moments(values: &[f64]) -> (f64, f64) {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    (mean, variance)
}
fn statistic(a: &[f64], b: &[f64]) -> f64 {
    let (am, av) = moments(a);
    let (bm, bv) = moments(b);
    let error = (av / a.len() as f64 + bv / b.len() as f64).sqrt();
    if error == 0.0 && am == bm {
        0.0
    } else {
        (bm - am) / error
    }
}
fn index(rng: &mut Seeded, n: usize) -> usize {
    let n = n as u64;
    let rejection = n.wrapping_neg() % n;
    loop {
        let value = rng.next_u64();
        if value >= rejection {
            return (value % n) as usize;
        }
    }
}
/// Pooled bootstrap calibration assumes exchangeable observations under the
/// null, and independent observations within each input. The +1 correction
/// prevents reporting a zero Monte Carlo probability. This tests a zero effect,
/// separately from practical-change margins or equivalence decisions.
pub fn welch(baseline: &[f64], candidate: &[f64], config: Config) -> Result<WelchTest> {
    sample_welch(baseline, candidate, config, false).map(|distribution| distribution.test)
}
fn sample_welch(
    baseline: &[f64],
    candidate: &[f64],
    config: Config,
    capture: bool,
) -> Result<WelchDistribution> {
    config.validate()?;
    if baseline.iter().chain(candidate).any(|v| !v.is_finite()) {
        return Err(error("hypothesis observations must be finite"));
    }
    let mut report = WelchTest {
        statistic: None,
        p_value: None,
        baseline_units: baseline.len(),
        candidate_units: candidate.len(),
        resamples: config.resamples,
        seed: config.seed,
        extreme_resamples: 0,
        degenerate_resamples: 0,
        unavailable_reason: None,
    };
    if baseline.len() < 2 || candidate.len() < 2 {
        report.unavailable_reason =
            Some("at least two independent units per side are required".into());
        return Ok(WelchDistribution {
            test: report,
            null_statistics: Vec::new(),
        });
    }
    // Joint scaling preserves the statistic while avoiding overflow in moments.
    let scale = baseline
        .iter()
        .chain(candidate)
        .map(|v| v.abs())
        .fold(0.0, f64::max)
        .max(f64::MIN_POSITIVE);
    let a: Vec<_> = baseline.iter().map(|v| v / scale).collect();
    let b: Vec<_> = candidate.iter().map(|v| v / scale).collect();
    let observed = statistic(&a, &b);
    if !observed.is_finite() {
        report.unavailable_reason =
            Some("zero within-group variance with distinct group means".into());
        return Ok(WelchDistribution {
            test: report,
            null_statistics: Vec::new(),
        });
    }
    report.statistic = Some(observed);
    let pool: Vec<_> = a.iter().chain(&b).copied().collect();
    let workers = crate::parallel_analysis::workers(config.resamples, pool.len());
    let mut null_statistics = if capture && workers > 1 {
        distribution_buffer(config.resamples)?
    } else {
        Vec::new()
    };
    let work = |count, mut rng: Seeded| -> Result<(usize, usize, Vec<NullStatistic>)> {
        let mut values = if capture {
            distribution_buffer(count)?
        } else {
            Vec::new()
        };
        let mut extreme = 0;
        let mut degenerate = 0;
        let mut null_a = vec![0.0; a.len()];
        let mut null_b = vec![0.0; b.len()];
        for _ in 0..count {
            for value in null_a.iter_mut().chain(&mut null_b) {
                *value = pool[index(&mut rng, pool.len())];
            }
            let sampled = statistic(&null_a, &null_b);
            degenerate += usize::from(!sampled.is_finite());
            // Infinite statistics count as extreme; no draws are discarded.
            extreme += usize::from(sampled.abs() >= observed.abs());
            if capture {
                values.push(if sampled == f64::INFINITY {
                    NullStatistic::PositiveInfinity
                } else if sampled == f64::NEG_INFINITY {
                    NullStatistic::NegativeInfinity
                } else {
                    NullStatistic::Finite(sampled)
                });
            }
        }
        Ok((extreme, degenerate, values))
    };
    if workers == 1 {
        let (extreme, degenerate, values) = work(config.resamples, Seeded::new(config.seed))?;
        report.extreme_resamples = extreme;
        report.degenerate_resamples = degenerate;
        null_statistics = values;
    } else {
        for (extreme, degenerate, values) in crate::parallel_analysis::chunks(
            config.resamples,
            config.seed,
            &[pool.len()],
            workers,
            work,
        )? {
            report.extreme_resamples += extreme;
            report.degenerate_resamples += degenerate;
            null_statistics.extend(values);
        }
    }
    report.p_value = Some(monte_carlo_probability(
        report.extreme_resamples,
        config.resamples,
    ));
    Ok(WelchDistribution {
        test: report,
        null_statistics,
    })
}

fn monte_carlo_probability(extreme: usize, resamples: usize) -> f64 {
    // Convert before adding one: usize::MAX is a valid requested count.
    (extreme as f64 + 1.0) / (resamples as f64 + 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parallel_welch_preserves_null_draws_counts_and_streaming_memory() {
        let a = [0.0, 1.0];
        let b = vec![0.0; 31];
        let pool: Vec<_> = a.iter().chain(&b).copied().collect();
        let observed = statistic(&a, &b);
        for seed in [91, 0u64.wrapping_sub(0x9e3779b97f4a7c15)] {
            let config = Config {
                resamples: 8193,
                seed,
            };
            let actual = welch_distribution(&a, &b, config).unwrap();
            let mut rng = Seeded::new(seed);
            let mut sa = [0.0; 2];
            let mut sb = vec![0.0; b.len()];
            let mut expected = Vec::new();
            let mut extreme = 0;
            let mut degenerate = 0;
            for _ in 0..config.resamples {
                for value in sa.iter_mut().chain(&mut sb) {
                    *value = pool[index(&mut rng, pool.len())];
                }
                let value = statistic(&sa, &sb);
                extreme += usize::from(value.abs() >= observed.abs());
                degenerate += usize::from(!value.is_finite());
                expected.push(if value == f64::INFINITY {
                    NullStatistic::PositiveInfinity
                } else if value == f64::NEG_INFINITY {
                    NullStatistic::NegativeInfinity
                } else {
                    NullStatistic::Finite(value)
                });
            }
            assert_eq!(actual.null_statistics, expected);
            assert_eq!(actual.test.extreme_resamples, extreme);
            assert_eq!(actual.test.degenerate_resamples, degenerate);
            assert!(degenerate > 0);
            assert_eq!(
                actual.test.p_value,
                Some((extreme as f64 + 1.0) / (config.resamples as f64 + 1.0))
            );
            let streaming = sample_welch(&a, &b, config, false).unwrap();
            assert_eq!(streaming.test, actual.test);
            assert_eq!(streaming.null_statistics.capacity(), 0);
            assert_eq!(welch(&a, &b, config).unwrap(), actual.test);
        }
    }

    #[test]
    fn full_resample_domain_keeps_probabilities_and_capacity_errors_explicit() {
        let single = welch_distribution(
            &[1.0, 2.0],
            &[2.0, 3.0],
            Config {
                resamples: 1,
                seed: 7,
            },
        )
        .unwrap();
        assert_eq!(single.null_statistics.len(), 1);
        assert_eq!(single.test.resamples, 1);
        assert_eq!(
            single.test.p_value,
            Some((single.test.extreme_resamples + 1) as f64 / 2.0)
        );
        let encoded = serde_json::to_vec(&single).unwrap();
        assert_eq!(
            serde_json::from_slice::<WelchDistribution>(&encoded).unwrap(),
            single
        );
        let large = welch(
            &[1.0, 2.0],
            &[1.0, 2.0],
            Config {
                resamples: 1_000_001,
                seed: 7,
            },
        )
        .unwrap();
        assert_eq!(large.extreme_resamples, 1_000_001);
        assert_eq!(large.p_value, Some(1.0));
        let impossible = Config {
            resamples: usize::MAX,
            seed: 7,
        };
        impossible.validate().unwrap();
        assert!(
            welch_distribution(&[1.0, 2.0], &[2.0, 3.0], impossible)
                .unwrap_err()
                .to_string()
                .contains("cannot allocate hypothesis distribution")
        );
        let missing = welch_distribution(&[], &[], impossible).unwrap();
        assert!(missing.null_statistics.is_empty());
        assert!(missing.test.unavailable_reason.is_some());
        assert_eq!(monte_carlo_probability(usize::MAX, usize::MAX), 1.0);
        assert!(monte_carlo_probability(0, usize::MAX) > 0.0);
        assert!(
            Config {
                resamples: 0,
                seed: 0
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn histogram_conserves_draws_and_exact_tail_counts_at_extreme_ranges() {
        let mut distribution = welch_distribution(
            &[1., 2.],
            &[3., 4.],
            Config {
                resamples: 2000,
                seed: 8,
            },
        )
        .unwrap();
        let chart = histogram(&distribution, 37).unwrap();
        assert_eq!(chart.total, 2000);
        assert_eq!(
            chart.bins.iter().map(|b| b.count).sum::<usize>()
                + chart.negative_infinity
                + chart.positive_infinity,
            chart.total
        );
        assert_eq!(
            chart.bins.iter().map(|b| b.extreme_count).sum::<usize>()
                + chart.negative_infinity
                + chart.positive_infinity,
            distribution.test.extreme_resamples
        );
        distribution.null_statistics = vec![
            NullStatistic::Finite(-f64::MAX),
            NullStatistic::Finite(0.0),
            NullStatistic::Finite(f64::MAX),
            NullStatistic::PositiveInfinity,
            NullStatistic::NegativeInfinity,
        ];
        let chart = histogram(&distribution, 4).unwrap();
        assert_eq!(chart.bins[0].count, 1);
        assert_eq!(chart.bins[2].count, 1);
        assert_eq!(chart.bins[3].count, 1);
        assert!(
            chart
                .bins
                .iter()
                .all(|bin| bin.lower.is_finite() && bin.upper.is_finite())
        );
        assert_eq!(chart.positive_infinity, 1);
        assert_eq!(chart.negative_infinity, 1);
        assert!(histogram(&distribution, 0).is_err());
        distribution
            .null_statistics
            .push(NullStatistic::Finite(f64::NAN));
        assert!(histogram(&distribution, 4).is_err());
    }
    #[test]
    fn exported_distribution_reconstructs_probability_and_preserves_infinities() {
        let config = Config {
            resamples: 2000,
            seed: 8,
        };
        let distribution = welch_distribution(&[1., 2.], &[3., 4.], config).unwrap();
        assert_eq!(
            distribution.test,
            welch(&[1., 2.], &[3., 4.], config).unwrap()
        );
        assert_eq!(distribution.null_statistics.len(), config.resamples);
        let observed = distribution.test.statistic.unwrap().abs();
        let extreme = distribution
            .null_statistics
            .iter()
            .filter(|draw| match draw {
                NullStatistic::Finite(value) => value.abs() >= observed,
                _ => true,
            })
            .count();
        let degenerate = distribution
            .null_statistics
            .iter()
            .filter(|draw| !matches!(draw, NullStatistic::Finite(_)))
            .count();
        assert!(degenerate > 0);
        assert_eq!(extreme, distribution.test.extreme_resamples);
        assert_eq!(degenerate, distribution.test.degenerate_resamples);
        assert_eq!(
            distribution.test.p_value.unwrap(),
            (extreme + 1) as f64 / (config.resamples + 1) as f64
        );
        let encoded = serde_json::to_string(&distribution).unwrap();
        let decoded: WelchDistribution = serde_json::from_str(&encoded).unwrap();
        assert_eq!(distribution, decoded);
        assert!(!encoded.contains("\"finite\",\"value\":null"));
        let unavailable = welch_distribution(&[1.], &[2.], config).unwrap();
        assert!(unavailable.null_statistics.is_empty());
        assert!(unavailable.test.unavailable_reason.is_some());
    }
    #[test]
    fn known_statistic_symmetry_scaling_and_seeded_reproducibility() {
        let a = [1., 2., 3., 4., 5.];
        let b = [2., 3., 4., 5., 6.];
        let config = Config {
            resamples: 2000,
            seed: 17,
        };
        let result = welch(&a, &b, config).unwrap();
        assert!((result.statistic.unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(result, welch(&a, &b, config).unwrap());
        let swapped = welch(&b, &a, config).unwrap();
        assert!((swapped.statistic.unwrap() + 1.0).abs() < 1e-12);
        let scaled = welch(&a.map(|v| v * 1e300), &b.map(|v| v * 1e300), config).unwrap();
        assert!((scaled.statistic.unwrap() - 1.0).abs() < 1e-12);
        assert!(result.p_value.unwrap() > 0.1 && result.p_value.unwrap() < 0.7);
    }
    #[test]
    fn pooled_resampling_matches_exhaustive_small_population() {
        // All 4^4 ordered resamples, evaluated with the closed-form n=2
        // Welch statistic, independently of the implementation's moments.
        let mut extreme = 0;
        for a in 1_i32..=4 {
            for b in 1_i32..=4 {
                for c in 1_i32..=4 {
                    for d in 1_i32..=4 {
                        let numerator = (c + d - a - b).pow(2);
                        let denominator = (a - b).pow(2) + (c - d).pow(2);
                        if (denominator == 0 && numerator != 0)
                            || (denominator != 0 && numerator >= 8 * denominator)
                        {
                            extreme += 1;
                        }
                    }
                }
            }
        }
        let exact = f64::from(extreme) / 256.0;
        let actual = welch(
            &[1., 2.],
            &[3., 4.],
            Config {
                resamples: 50_000,
                seed: 91,
            },
        )
        .unwrap();
        assert!(
            (actual.p_value.unwrap() - exact).abs() < 0.02,
            "{actual:?}; exact={exact}"
        );
        assert!(actual.degenerate_resamples > 0);
    }
    #[test]
    fn unavailable_units_degenerate_populations_and_extreme_effects() {
        let config = Config {
            resamples: 2000,
            seed: 8,
        };
        assert!(welch(&[1.], &[2., 3.], config).unwrap().p_value.is_none());
        assert!(
            welch(&[1., 1.], &[2., 2.], config)
                .unwrap()
                .p_value
                .is_none()
        );
        assert_eq!(
            welch(&[1., 1.], &[1., 1.], config).unwrap().p_value,
            Some(1.0)
        );
        assert!(welch(&[f64::NAN, 1.], &[1., 2.], config).is_err());
        let result = welch(
            &[1., 2., 3., 4., 5., 6.],
            &[101., 102., 103., 104., 105., 106.],
            config,
        )
        .unwrap();
        assert!(result.p_value.unwrap() > 0.0 && result.p_value.unwrap() < 0.01);
    }
}
