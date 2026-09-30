//! Seeded percentile bootstrap of descriptive estimates. Intervals assume independent
//! sampling units; within-process batches can be autocorrelated and are labelled as such.
use crate::{Availability, Result, Run, Seeded, Status, error};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub resamples: usize,
    pub confidence_level: f64,
    pub seed: u64,
}
/// Sparse group/case settings. Explicit fields override inherited fields independently.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Options {
    pub resamples: Option<usize>,
    pub confidence_level: Option<f64>,
    pub seed: Option<u64>,
}
impl Options {
    pub fn inherit(&mut self, fallback: Self) {
        self.resamples = self.resamples.or(fallback.resamples);
        self.confidence_level = self.confidence_level.or(fallback.confidence_level);
        self.seed = self.seed.or(fallback.seed);
    }
    pub fn is_empty(self) -> bool {
        self.resamples.is_none() && self.confidence_level.is_none() && self.seed.is_none()
    }
    pub fn resolve(self, fallback: &Config) -> Result<Config> {
        let config = Config {
            resamples: self.resamples.unwrap_or(fallback.resamples),
            confidence_level: self.confidence_level.unwrap_or(fallback.confidence_level),
            seed: self.seed.unwrap_or(fallback.seed),
        };
        config.validate()?;
        Ok(config)
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            resamples: 10_000,
            confidence_level: 0.95,
            seed: 0,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(2..=1_000_000).contains(&self.resamples) {
            return Err(error("bootstrap resamples must be 2..=1000000"));
        }
        if !self.confidence_level.is_finite()
            || self.confidence_level <= 0.0
            || self.confidence_level >= 1.0
        {
            return Err(error("confidence level must be strictly between 0 and 1"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Estimate {
    pub point: f64,
    pub lower: f64,
    pub upper: f64,
    pub standard_error: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Estimates {
    pub mean: Estimate,
    pub median: Estimate,
    pub standard_deviation: Estimate,
    /// Median absolute deviation, scaled by 1.4826 for normal consistency.
    pub median_absolute_deviation: Estimate,
    pub minimum: f64,
    pub maximum: f64,
}

/// Descriptive statistics without resampling. Sample deviation is undefined for
/// one observation; the median absolute deviation is scaled by 1.4826.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summary {
    pub count: usize,
    pub minimum: f64,
    pub maximum: f64,
    pub mean: f64,
    pub median: f64,
    pub standard_deviation: Option<f64>,
    pub median_absolute_deviation: f64,
}
/// Summarize finite observations, including a single observation. Empty input
/// and statistics outside the finite numeric range return an error.
pub fn describe(values: &[f64]) -> Result<Summary> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return Err(error("summary needs at least one finite observation"));
    }
    let scale = values
        .iter()
        .map(|v| v.abs())
        .fold(f64::MIN_POSITIVE, f64::max);
    let mut normalized: Vec<_> = values.iter().map(|v| v / scale).collect();
    let [mean, median, sd, mad] = if values.len() == 1 {
        [normalized[0], normalized[0], 0.0, 0.0]
    } else {
        statistics(&mut normalized)
    };
    let points = [mean * scale, median * scale, sd * scale, mad * scale];
    if points.iter().any(|v| !v.is_finite()) {
        return Err(error("summary exceeds finite numeric range"));
    }
    Ok(Summary {
        count: values.len(),
        minimum: values.iter().copied().min_by(f64::total_cmp).unwrap(),
        maximum: values.iter().copied().max_by(f64::total_cmp).unwrap(),
        mean: points[0],
        median: points[1],
        standard_deviation: (values.len() > 1).then_some(points[2]),
        median_absolute_deviation: points[3],
    })
}

pub(crate) fn quantile(sorted: &[f64], probability: f64) -> f64 {
    let position = probability * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let fraction = position - lower as f64;
    sorted[lower] * (1.0 - fraction) + sorted[position.ceil() as usize] * fraction
}
pub(crate) fn deviation(values: &[f64], mean: f64) -> f64 {
    (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64).sqrt()
}
fn statistics(values: &mut [f64]) -> [f64; 4] {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let sd = deviation(values, mean);
    values.sort_by(f64::total_cmp);
    let median = quantile(values, 0.5);
    let mut differences: Vec<_> = values.iter().map(|x| (x - median).abs()).collect();
    differences.sort_by(f64::total_cmp);
    [mean, median, sd, quantile(&differences, 0.5) * 1.4826]
}
pub(crate) fn draw_index(rng: &mut Seeded, length: usize) -> usize {
    let n = length as u64;
    let threshold = n.wrapping_neg() % n;
    loop {
        let value = rng.next_u64();
        if value >= threshold {
            return (value % n) as usize;
        }
    }
}

/// Bootstrap draws in original units, aligned by resample index.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Distributions {
    pub mean: Vec<f64>,
    pub median: Vec<f64>,
    pub standard_deviation: Vec<f64>,
    pub median_absolute_deviation: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DistributionEstimates {
    pub estimates: Estimates,
    pub distributions: Distributions,
}

/// Resample whole observations with replacement, retaining the original sample size.
/// Normalization avoids overflow in intermediate sums; unrepresentable estimates error.
pub fn estimate(values: &[f64], config: &Config) -> Result<Estimates> {
    Ok(estimate_impl(values, config, false)?.0)
}

/// Retain every draw in generation order, in original units. The four arrays
/// share resample indices so their joint bootstrap dependence is preserved.
pub fn estimate_with_distributions(
    values: &[f64],
    config: &Config,
) -> Result<DistributionEstimates> {
    let (estimates, distributions) = estimate_impl(values, config, true)?;
    Ok(DistributionEstimates {
        estimates,
        distributions: distributions.unwrap(),
    })
}

fn estimate_impl(
    values: &[f64],
    config: &Config,
    capture: bool,
) -> Result<(Estimates, Option<Distributions>)> {
    config.validate()?;
    if values.len() < 2 || values.iter().any(|v| !v.is_finite()) {
        return Err(error("bootstrap needs at least two finite observations"));
    }
    let scale = values
        .iter()
        .map(|v| v.abs())
        .fold(0.0, f64::max)
        .max(f64::MIN_POSITIVE);
    let normalized: Vec<_> = values.iter().map(|v| v / scale).collect();
    let points = statistics(&mut normalized.clone());
    let mut rng = Seeded::new(config.seed);
    let mut distributions: [Vec<f64>; 4] =
        std::array::from_fn(|_| Vec::with_capacity(config.resamples));
    let mut sample = vec![0.0; values.len()];
    for _ in 0..config.resamples {
        for value in &mut sample {
            *value = normalized[draw_index(&mut rng, values.len())];
        }
        for (distribution, statistic) in distributions.iter_mut().zip(statistics(&mut sample)) {
            distribution.push(statistic);
        }
    }
    let retained = if capture {
        let [mean, median, standard_deviation, median_absolute_deviation] = distributions
            .each_ref()
            .map(|values| values.iter().map(|value| value * scale).collect::<Vec<_>>());
        if [
            &mean,
            &median,
            &standard_deviation,
            &median_absolute_deviation,
        ]
        .iter()
        .any(|values| values.iter().any(|v| !v.is_finite()))
        {
            return Err(error("bootstrap distribution exceeds finite numeric range"));
        }
        Some(Distributions {
            mean,
            median,
            standard_deviation,
            median_absolute_deviation,
        })
    } else {
        None
    };
    let tail = (1.0 - config.confidence_level) / 2.0;
    let mut estimates = Vec::with_capacity(4);
    for (mut distribution, point) in distributions.into_iter().zip(points) {
        let mean = distribution.iter().sum::<f64>() / distribution.len() as f64;
        let standard_error = deviation(&distribution, mean) * scale;
        distribution.sort_by(f64::total_cmp);
        let estimate = Estimate {
            point: point * scale,
            lower: quantile(&distribution, tail) * scale,
            upper: quantile(&distribution, 1.0 - tail) * scale,
            standard_error,
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
            return Err(error("bootstrap estimate exceeds finite numeric range"));
        }
        estimates.push(estimate);
    }
    let mut estimates = estimates.into_iter();
    Ok((
        Estimates {
            mean: estimates.next().unwrap(),
            median: estimates.next().unwrap(),
            standard_deviation: estimates.next().unwrap(),
            median_absolute_deviation: estimates.next().unwrap(),
            minimum: values.iter().copied().min_by(f64::total_cmp).unwrap(),
            maximum: values.iter().copied().max_by(f64::total_cmp).unwrap(),
        },
        retained,
    ))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessRegression {
    pub process: u32,
    pub fit: crate::regression::Fit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slope_distribution: Option<Vec<f64>>,
    /// Original paired batches used for this process fit; absent in older reports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub samples: Vec<crate::regression::Sample>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub unit: String,
    /// Full metric semantics for grouping summaries. Legacy rows remain isolated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_contract: Option<crate::Metric>,
    pub resampling_unit: String,
    pub units: usize,
    pub estimates: Option<Estimates>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distributions: Option<Distributions>,
    #[serde(default)]
    pub regressions: Vec<ProcessRegression>,
    #[serde(default)]
    pub outliers: Option<crate::outliers::Classification>,
    pub note: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub config: Config,
    /// Effective per-case settings, overriding the report-wide fallback.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub case_configs: BTreeMap<String, Config>,
    pub method: String,
    pub rows: Vec<Row>,
}
const SAVED_CONFIG: &str = "airbug.analysis.bootstrap.v1";

#[derive(Serialize, Deserialize)]
struct SavedConfig {
    fallback: Config,
    cases: BTreeMap<String, Config>,
}

/// Persist analysis settings independently of measurement contracts and observations.
pub fn save_settings(
    run: &mut Run,
    fallback: &Config,
    cases: &BTreeMap<String, Config>,
) -> Result<()> {
    fallback.validate()?;
    for (id, config) in cases {
        config.validate()?;
        if !run.cases.iter().any(|case| case.id == *id) {
            return Err(error(format!(
                "bootstrap settings refer to unknown case: {id}"
            )));
        }
    }
    let encoded = serde_json::to_string(&SavedConfig {
        fallback: fallback.clone(),
        cases: cases.clone(),
    })?;
    run.provenance.insert(SAVED_CONFIG.into(), encoded);
    Ok(())
}

/// Reanalyze a saved run, restoring its settings before applying explicit overrides.
/// Runs without saved settings use the standard defaults. Case extraction may leave
/// settings for omitted cases; those entries are validated but not analyzed.
pub fn analyze_saved(run: &Run, overrides: Options, capture: bool) -> Result<Report> {
    let saved = run
        .provenance
        .get(SAVED_CONFIG)
        .map(|value| serde_json::from_str::<SavedConfig>(value))
        .transpose()?;
    let (fallback, mut cases) = match saved {
        Some(saved) => {
            saved.fallback.validate()?;
            for config in saved.cases.values() {
                config.validate()?;
            }
            (saved.fallback, saved.cases)
        }
        None => (Config::default(), BTreeMap::new()),
    };
    let fallback = overrides.resolve(&fallback)?;
    cases.retain(|id, _| run.cases.iter().any(|case| case.id == *id));
    for config in cases.values_mut() {
        *config = overrides.resolve(config)?;
    }
    analyze_impl(run, &fallback, &cases, capture)
}

/// Multi-process data is reduced to one median per process before resampling.
/// A single process uses normalized batches, explicitly without process-level claims.
pub fn analyze(run: &Run, config: &Config) -> Result<Report> {
    analyze_impl(run, config, &BTreeMap::new(), false)
}

/// Analyze the same statistical units as `analyze`, retaining absolute draws.
pub fn analyze_with_distributions(run: &Run, config: &Config) -> Result<Report> {
    analyze_impl(run, config, &BTreeMap::new(), true)
}

impl Report {
    pub fn config_for_case(&self, case: &str) -> &Config {
        self.case_configs.get(case).unwrap_or(&self.config)
    }
}

/// Analyze cases with independently configured intervals and resample counts.
/// Unknown case IDs are rejected so a misspelled override cannot silently disappear.
pub fn analyze_with_case_configs(
    run: &Run,
    fallback: &Config,
    overrides: &BTreeMap<String, Config>,
    capture_distributions: bool,
) -> Result<Report> {
    analyze_impl(run, fallback, overrides, capture_distributions)
}

fn analyze_impl(
    run: &Run,
    config: &Config,
    overrides: &BTreeMap<String, Config>,
    capture: bool,
) -> Result<Report> {
    run.validate()?;
    config.validate()?;
    if run.status != Status::Complete {
        return Err(error("bootstrap requires a complete run"));
    }
    for (case, settings) in overrides {
        if !run.cases.iter().any(|entry| entry.id == *case) {
            return Err(error(format!(
                "bootstrap settings refer to unknown case: {case}"
            )));
        }
        settings.validate()?;
    }
    let mut rows = Vec::new();
    for case in &run.cases {
        let config = overrides.get(&case.id).unwrap_or(config);
        for metric in &case.metrics {
            let mut variants: BTreeMap<&str, BTreeMap<u32, Vec<f64>>> = BTreeMap::new();
            let mut batches: BTreeMap<&str, BTreeMap<u32, Vec<crate::regression::Sample>>> =
                BTreeMap::new();
            let mut missing = std::collections::BTreeSet::new();
            for observation in run
                .observations
                .iter()
                .filter(|o| o.case == case.id && o.metric == metric.id)
            {
                let process = variants
                    .entry(&observation.variant)
                    .or_default()
                    .entry(observation.process)
                    .or_default();
                if observation.availability != Availability::Available {
                    missing.insert(observation.variant.as_str());
                }
                if let Some(value) = observation.number()? {
                    if metric.statistic == "batch_total" {
                        batches
                            .entry(&observation.variant)
                            .or_default()
                            .entry(observation.process)
                            .or_default()
                            .push(crate::regression::Sample {
                                operations: observation.operations as f64,
                                total: value,
                            });
                    }
                    process.push(if metric.statistic == "batch_total" {
                        value / observation.operations as f64
                    } else {
                        value
                    });
                }
            }
            for (variant, processes) in variants {
                let multiple = processes.len() > 1;
                let values: Vec<_> = if multiple {
                    processes
                        .values()
                        .filter(|v| !v.is_empty())
                        .map(|v| crate::analysis::median(v))
                        .collect()
                } else {
                    processes.values().flatten().copied().collect()
                };
                let valid = !missing.contains(variant) && values.len() >= 2;
                let mut regressions = Vec::new();
                if !missing.contains(variant) {
                    if let Some(processes) = batches.get(variant) {
                        for (&process, samples) in processes {
                            if samples.len() >= 2
                                && samples
                                    .iter()
                                    .any(|s| s.operations != samples[0].operations)
                            {
                                let (fit, slope_distribution) = if capture {
                                    let (fit, draws) =
                                        crate::regression::fit_with_distribution(samples, config)?;
                                    (fit, Some(draws))
                                } else {
                                    (crate::regression::fit(samples, config)?, None)
                                };
                                regressions.push(ProcessRegression {
                                    process,
                                    fit,
                                    slope_distribution,
                                    samples: samples.clone(),
                                });
                            }
                        }
                    }
                }
                let (estimates, distributions) = if valid {
                    let (estimates, distributions) = estimate_impl(&values, config, capture)?;
                    (Some(estimates), distributions)
                } else {
                    (None, None)
                };
                rows.push(Row { metric_contract: Some(metric.clone()), case: case.id.clone(), metric: metric.id.clone(), variant: variant.into(), unit: metric.unit.clone(),
                resampling_unit: if multiple { "process_median" } else { "normalized_observation" }.into(), units: values.len(),
                regressions,
                outliers: if !missing.contains(variant) && !values.is_empty() { Some(crate::outliers::classify(&values)?) } else { None },
                estimates, distributions,
                note: if !valid { "At least two complete units are required; missing observations are not silently discarded." }
                    else if multiple { "Assumes independent processes. Each process contributes one median." }
                    else { "Exploratory within-process interval; autocorrelation can invalidate coverage. Not individual-operation latency or evidence of between-process reproducibility." }.into() });
            }
        }
    }
    Ok(Report {
        case_configs: overrides.clone(),
        config: config.clone(),
        method: "percentile_bootstrap_linear_quantiles".into(),
        rows,
    })
}

pub fn markdown(report: &Report) -> String {
    use crate::report::escape;
    let mut output = format!(
        "\n# Bootstrap estimates\n\nConfidence: {:.2}%; resamples: {}; seed: {}. Percentile intervals.\n\n| Case / metric / variant | Statistic | Estimate | Interval | Standard error | Unit | Samples |\n|---|---|---:|---|---:|---|---:|\n",
        report.config.confidence_level * 100.0,
        report.config.resamples,
        report.config.seed
    );
    if !report.case_configs.is_empty() {
        output = output.replacen("Confidence:", "Default confidence (cases may override):", 1);
    }
    let mut notes = String::new();
    for row in &report.rows {
        let label = escape(&format!("{} / {} / {}", row.case, row.metric, row.variant));
        if !report.case_configs.is_empty() {
            let config = report.config_for_case(&row.case);
            notes.push_str(&format!(
                "\n- {label}: confidence {:.2}%; resamples {}; seed {}.\n",
                config.confidence_level * 100.0,
                config.resamples,
                config.seed
            ));
        }
        if let Some(estimates) = &row.estimates {
            for (name, estimate) in [
                ("mean", &estimates.mean),
                ("median", &estimates.median),
                ("standard deviation", &estimates.standard_deviation),
                ("scaled MAD", &estimates.median_absolute_deviation),
            ] {
                output.push_str(&format!(
                    "| {label} | {name} | {:.6} | [{:.6}, {:.6}] | {:.6} | {} | {} |\n",
                    estimate.point,
                    estimate.lower,
                    estimate.upper,
                    estimate.standard_error,
                    escape(&row.unit),
                    row.units
                ));
            }
        } else {
            output.push_str(&format!(
                "| {label} | unavailable | | | | {} | {} |\n",
                escape(&row.unit),
                row.units
            ));
        }
        if let Some(outliers) = &row.outliers {
            notes.push_str(&format!("\n- {label}: outliers low severe {}, low mild {}, normal {}, high mild {}, high severe {}. No observations discarded.\n", outliers.counts[0], outliers.counts[1], outliers.counts[2], outliers.counts[3], outliers.counts[4]));
        }
        for regression in &row.regressions {
            let fit = &regression.fit;
            output.push_str(&format!(
                "| {label} | slope (process {}) | {:.6} | [{:.6}, {:.6}] | {:.6} | {}/op | {} |\n",
                regression.process,
                fit.slope.point,
                fit.slope.lower,
                fit.slope.upper,
                fit.slope.standard_error,
                escape(&row.unit),
                fit.samples
            ));
            notes.push_str(&format!("\n- {label}, process {}: through-origin fit, centered R² {}; bootstrap resamples paired batches. Within-process independence is assumed.\n", regression.process, fit.r_squared.map(|r| format!("{r:.6}")).unwrap_or_else(|| "undefined (constant totals)".into())));
        }
        notes.push_str(&format!(
            "\n- {}: {} ({})\n",
            label,
            escape(&row.note),
            escape(&row.resampling_unit)
        ));
    }
    output.push_str(&notes);
    output
}

/// Standalone offline report with estimates and classified observations.
pub fn html(report: &Report) -> String {
    html_with_scale(report, crate::viz::charts::AxisScale::Linear)
}

/// Render descriptive distributions with a shared configurable summary scale.
pub fn html_with_scale(report: &Report, scale: crate::viz::charts::AxisScale) -> String {
    html_with_case_scales(report, Some(scale), &BTreeMap::new())
}

/// Render per-case summary scales; an explicit CLI/global scale overrides all cases.
pub fn html_with_case_scales(
    report: &Report,
    override_scale: Option<crate::viz::charts::AxisScale>,
    scales: &BTreeMap<String, crate::viz::charts::AxisScale>,
) -> String {
    let mut charts: String = report
        .rows
        .iter()
        .filter_map(|row| {
            row.outliers.as_ref().map(|outliers| {
                crate::outliers::figure(
                    outliers,
                    &format!(
                        "{} / {} / {} ({})",
                        row.case, row.metric, row.variant, row.resampling_unit
                    ),
                    &row.unit,
                )
            })
        })
        .collect();
    for row in &report.rows {
        if let (Some(draws), Some(estimates)) = (&row.distributions, &row.estimates) {
            for (name, values, estimate) in [
                ("mean", &draws.mean, &estimates.mean),
                ("median", &draws.median, &estimates.median),
                (
                    "standard deviation",
                    &draws.standard_deviation,
                    &estimates.standard_deviation,
                ),
                (
                    "MAD",
                    &draws.median_absolute_deviation,
                    &estimates.median_absolute_deviation,
                ),
            ] {
                let title = format!(
                    "{} / {} / {} bootstrap {} ({})",
                    row.case, row.metric, row.variant, name, row.resampling_unit
                );
                match crate::density::estimate_figure(values, estimate, &title, &row.unit) {
                    Ok(chart) => charts.push_str(&chart),
                    Err(err) => charts.push_str(&format!(
                        "<p>{}: distribution unavailable: {}</p>",
                        crate::report::escape(&title),
                        crate::report::escape(&err.to_string())
                    )),
                }
            }
        }
        if let Some(outliers) = &row.outliers {
            let values: Vec<_> = outliers.points.iter().map(|point| point.value).collect();
            let label = format!(
                "{} / {} / {} ({}) density",
                row.case, row.metric, row.variant, row.resampling_unit
            );
            charts.push_str(&density_chart(
                &[crate::density::Series {
                    label: &row.resampling_unit,
                    values: &values,
                }],
                &label,
                &row.unit,
            ));
        }
        for regression in &row.regressions {
            if let Some(draws) = &regression.slope_distribution {
                let title = format!(
                    "{} / {} / {} / process {} bootstrap slope",
                    row.case, row.metric, row.variant, regression.process
                );
                match crate::density::estimate_figure(
                    draws,
                    &regression.fit.slope,
                    &title,
                    &format!("{}/operation", row.unit),
                ) {
                    Ok(chart) => charts.push_str(&chart),
                    Err(err) => charts.push_str(&format!(
                        "<p>{}: distribution unavailable: {}</p>",
                        crate::report::escape(&title),
                        crate::report::escape(&err.to_string())
                    )),
                }
            }
            charts.push_str(&crate::regression::figure(
                &regression.samples,
                &regression.fit,
                &format!(
                    "{} / {} / {} / process {} regression",
                    row.case, row.metric, row.variant, regression.process
                ),
                &row.unit,
            ));
        }
    }
    charts.push_str(&violin_charts_with_case_scales(
        report,
        override_scale,
        scales,
    ));
    crate::report::html(&markdown(report))
        .replace("<!--CHARTS-->", &charts)
        .replace("<!--DETAILS-->", "")
}

/// Render saved per-case scales, optionally overridden by a caller.
pub fn violin_charts_with_case_scales(
    report: &Report,
    override_scale: Option<crate::viz::charts::AxisScale>,
    scales: &BTreeMap<String, crate::viz::charts::AxisScale>,
) -> String {
    let mut charts = String::new();
    for scale in [
        crate::viz::charts::AxisScale::Linear,
        crate::viz::charts::AxisScale::Logarithmic,
    ] {
        let selected: Vec<_> = report
            .rows
            .iter()
            .filter(|row| {
                override_scale
                    .or_else(|| scales.get(&row.case).copied())
                    .unwrap_or_default()
                    == scale
            })
            .collect();
        if !selected.is_empty() {
            charts.push_str(&violin_rows(&selected, scale));
        }
    }
    charts
}

/// Group compatible metric contracts and sampling units into violin summaries.
/// Legacy reports without contracts retain independent per-row charts.
pub fn violin_charts(report: &Report, scale: crate::viz::charts::AxisScale) -> String {
    violin_rows(&report.rows.iter().collect::<Vec<_>>(), scale)
}
fn violin_rows(rows: &[&Row], scale: crate::viz::charts::AxisScale) -> String {
    let mut groups: BTreeMap<(String, String), Vec<&Row>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let key = row
            .metric_contract
            .as_ref()
            .and_then(|m| serde_json::to_string(m).ok())
            .unwrap_or_else(|| format!("legacy-row-{index}"));
        groups
            .entry((key, row.resampling_unit.clone()))
            .or_default()
            .push(row);
    }
    let mut output = String::from(
        "<section><h2>Violin summaries</h2><p>Metric contracts and sampling units are kept separate. Process medians give each process equal weight; normalized observations describe within-process variation.</p>",
    );
    for ((_, sampling), rows) in groups {
        let labels: Vec<_> = rows
            .iter()
            .map(|row| format!("{} / {}", row.case, row.variant))
            .collect();
        let values: Vec<Vec<f64>> = rows
            .iter()
            .map(|row| {
                row.outliers
                    .as_ref()
                    .map(|o| o.points.iter().map(|p| p.value).collect())
                    .unwrap_or_default()
            })
            .collect();
        let series: Vec<_> = labels
            .iter()
            .zip(&values)
            .map(|(label, values)| crate::density::Series { label, values })
            .collect();
        let first = rows[0];
        let title = format!("{} violin summary ({sampling})", first.metric);
        match crate::violin::figure(&series, &title, &first.unit, scale) {
            Ok(chart) => output.push_str(&chart),
            Err(err) => output.push_str(&format!(
                "<p>{}: unavailable: {}</p>",
                crate::report::escape(&title),
                crate::report::escape(&err.to_string())
            )),
        }
        for row in rows {
            if row.outliers.is_none() {
                output.push_str(&format!(
                    "<p>{} / {}: {}</p>",
                    crate::report::escape(&row.case),
                    crate::report::escape(&row.variant),
                    crate::report::escape(&row.note)
                ));
            }
        }
    }
    output.push_str("</section>");
    output
}

fn density_chart(series: &[crate::density::Series<'_>], title: &str, unit: &str) -> String {
    crate::density::figure_with_outliers(series, title, unit).unwrap_or_else(|err| {
        format!(
            "<p>{}: density unavailable: {}</p>",
            crate::report::escape(title),
            crate::report::escape(&err.to_string())
        )
    })
}

/// Overlay densities and regressions from already validated compatible runs.
/// Each process remains a separate fitted series, even when process IDs coincide.
pub fn comparison_charts(baseline: &Report, candidate: &Report) -> String {
    let mut output = String::new();
    for row in &candidate.rows {
        let Some(old) = baseline.rows.iter().find(|old| {
            old.case == row.case
                && old.metric == row.metric
                && old.variant == row.variant
                && old.unit == row.unit
        }) else {
            continue;
        };
        if let (Some(before), Some(after)) = (&old.outliers, &row.outliers) {
            let before: Vec<_> = before.points.iter().map(|point| point.value).collect();
            let after: Vec<_> = after.points.iter().map(|point| point.value).collect();
            let old_label = format!("baseline ({})", old.resampling_unit);
            let new_label = format!("candidate ({})", row.resampling_unit);
            output.push_str(&density_chart(
                &[
                    crate::density::Series {
                        label: &old_label,
                        values: &before,
                    },
                    crate::density::Series {
                        label: &new_label,
                        values: &after,
                    },
                ],
                &format!(
                    "{} / {} / {} density comparison",
                    row.case, row.metric, row.variant
                ),
                &row.unit,
            ));
        }
        if old.regressions.is_empty() || row.regressions.is_empty() {
            continue;
        }
        let labeled: Vec<_> = [("baseline", old), ("candidate", row)]
            .into_iter()
            .flat_map(|(side, row)| {
                row.regressions
                    .iter()
                    .map(move |r| (format!("{side} process {}", r.process), r))
            })
            .collect();
        let series: Vec<_> = labeled
            .iter()
            .map(|(label, r)| crate::regression::Series {
                label,
                samples: &r.samples,
                fit: &r.fit,
            })
            .collect();
        output.push_str(&format!("<p>Per-process slope confidence levels: baseline {}, candidate {}. These intervals are not family-adjusted.</p>", baseline.config_for_case(&row.case).confidence_level, candidate.config_for_case(&row.case).confidence_level));
        output.push_str(&crate::regression::comparison_figure(
            &series,
            &format!(
                "{} / {} / {} regression comparison",
                row.case, row.metric, row.variant
            ),
            &row.unit,
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn case_configs_control_estimates_draws_regressions_and_labels() {
        let mut recorder = crate::Recorder::new();
        for id in ["first", "second"] {
            recorder
                .case(crate::Case {
                    id: id.into(),
                    contract: Default::default(),
                    metrics: vec![crate::Metric::duration("wall", "process", "batch_total")],
                })
                .unwrap();
            recorder.observe(id, "wall", 1).unwrap();
        }
        let mut run = recorder.finish().unwrap();
        let originals = std::mem::take(&mut run.observations);
        for original in originals {
            for sequence in 0..5 {
                let mut row = original.clone();
                row.sequence = sequence;
                row.operations = sequence + 1;
                row.value = Some(((sequence + 1).pow(2)).to_string());
                run.observations.push(row);
            }
        }
        let before = serde_json::to_vec(&run).unwrap();
        let fallback = Config {
            resamples: 32,
            confidence_level: 0.8,
            seed: 1,
        };
        let overrides = BTreeMap::from([(
            "second".into(),
            Config {
                resamples: 96,
                confidence_level: 0.99,
                seed: 7,
            },
        )]);
        let report = analyze_with_case_configs(&run, &fallback, &overrides, true).unwrap();
        for row in &report.rows {
            let config = report.config_for_case(&row.case);
            let independent = analyze_with_distributions(&run, config).unwrap();
            let expected = independent
                .rows
                .iter()
                .find(|other| other.case == row.case)
                .unwrap();
            assert_eq!(
                serde_json::to_value(row).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
            assert_eq!(
                row.distributions.as_ref().unwrap().mean.len(),
                config.resamples
            );
            assert_eq!(
                row.regressions[0]
                    .slope_distribution
                    .as_ref()
                    .unwrap()
                    .len(),
                config.resamples
            );
        }
        assert_eq!(before, serde_json::to_vec(&run).unwrap());
        let page = html(&report);
        assert!(page.contains("confidence 80.00%; resamples 32; seed 1"));
        assert!(page.contains("confidence 99.00%; resamples 96; seed 7"));
        let baseline = analyze_with_distributions(&run, &fallback).unwrap();
        assert!(comparison_charts(&baseline, &report).contains("baseline 0.8, candidate 0.99"));
        let restored: Report =
            serde_json::from_slice(&serde_json::to_vec(&report).unwrap()).unwrap();
        assert_eq!(restored.config_for_case("second").resamples, 96);
        let legacy: Report =
            serde_json::from_slice(&serde_json::to_vec(&baseline).unwrap()).unwrap();
        assert!(legacy.case_configs.is_empty());
        assert_eq!(legacy.config_for_case("second").resamples, 32);
        save_settings(&mut run, &fallback, &overrides).unwrap();
        let saved_bytes = serde_json::to_vec(&run).unwrap();
        let saved = analyze_saved(&run, Options::default(), true).unwrap();
        assert_eq!(
            serde_json::to_value(&saved).unwrap(),
            serde_json::to_value(&report).unwrap()
        );
        let changed = analyze_saved(
            &run,
            Options {
                resamples: Some(48),
                ..Default::default()
            },
            true,
        )
        .unwrap();
        assert_eq!(changed.config_for_case("first").confidence_level, 0.8);
        assert_eq!(changed.config_for_case("second").confidence_level, 0.99);
        assert!(
            changed
                .rows
                .iter()
                .all(|row| row.distributions.as_ref().unwrap().mean.len() == 48)
        );
        assert_eq!(saved_bytes, serde_json::to_vec(&run).unwrap());
        let one = crate::history::one_case(&run, "second");
        assert_eq!(
            analyze_saved(&one, Options::default(), false)
                .unwrap()
                .case_configs
                .len(),
            1
        );
        let mut invalid = overrides.clone();
        invalid.insert("missing".into(), fallback.clone());
        assert!(analyze_with_case_configs(&run, &fallback, &invalid, true).is_err());
        invalid.remove("missing");
        invalid.get_mut("second").unwrap().confidence_level = 1.0;
        assert!(analyze_with_case_configs(&run, &fallback, &invalid, false).is_err());
        assert!(save_settings(&mut run, &fallback, &invalid).is_err());
        assert_eq!(saved_bytes, serde_json::to_vec(&run).unwrap());
        run.provenance.insert(SAVED_CONFIG.into(), "{}".into());
        assert!(analyze_saved(&run, Options::default(), false).is_err());
    }

    #[test]
    fn report_capture_uses_process_units_and_reads_legacy_json() {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "case".into(),
                contract: Default::default(),
                metrics: vec![crate::Metric::duration("wall", "process", "batch_total")],
            })
            .unwrap();
        recorder.observe("case", "wall", 1).unwrap();
        let mut run = recorder.finish().unwrap();
        let original = run.observations[0].clone();
        run.observations.clear();
        for sequence in 0..6 {
            let mut o = original.clone();
            o.process = if sequence < 5 { 0 } else { 1 };
            o.sequence = sequence;
            o.operations = 2;
            o.value = Some(if sequence < 5 { "4" } else { "8" }.into());
            run.observations.push(o);
        }
        let config = Config {
            resamples: 128,
            ..Default::default()
        };
        let captured = analyze_with_distributions(&run, &config).unwrap();
        let expected = estimate_with_distributions(&[2., 4.], &config).unwrap();
        assert_eq!(captured.rows[0].units, 2);
        let document = html(&captured);
        assert_eq!(document.matches("Bootstrap confidence interval").count(), 4);
        for statistic in ["mean", "median", "standard deviation", "MAD"] {
            assert!(document.contains(&format!("bootstrap {statistic} (process_median)")));
        }
        assert_eq!(captured.rows[0].resampling_unit, "process_median");
        assert_eq!(
            captured.rows[0].distributions.as_ref(),
            Some(&expected.distributions)
        );
        let ordinary = analyze(&run, &config).unwrap();
        assert_eq!(ordinary.rows[0].estimates, captured.rows[0].estimates);
        let old = serde_json::to_string(&ordinary).unwrap();
        assert!(!old.contains("distributions"));
        let restored: Report = serde_json::from_str(&old).unwrap();
        assert!(restored.rows[0].distributions.is_none());
        let restored: Report =
            serde_json::from_str(&serde_json::to_string(&captured).unwrap()).unwrap();
        assert_eq!(
            restored.rows[0].distributions,
            captured.rows[0].distributions
        );
        let mut varying = run.clone();
        for observation in &mut varying.observations {
            observation.operations = observation.sequence + 1;
            observation.value = Some((observation.operations * 3).to_string());
        }
        let slopes = analyze_with_distributions(&varying, &config).unwrap();
        let regression = &slopes.rows[0].regressions[0];
        let draws = regression.slope_distribution.as_ref().unwrap();
        assert_eq!(draws.len(), config.resamples);
        assert!(draws.iter().all(|value| (value - 3.).abs() < 1e-12));
        assert!(html(&slopes).contains("bootstrap slope"));
        assert!(html(&slopes).contains("ns/operation"));
        let plain = analyze(&varying, &config).unwrap();
        assert!(plain.rows[0].regressions[0].slope_distribution.is_none());
        let restored: Report =
            serde_json::from_str(&serde_json::to_string(&slopes).unwrap()).unwrap();
        assert_eq!(
            restored.rows[0].regressions[0].slope_distribution.as_ref(),
            Some(draws)
        );
        run.observations[0].value = None;
        run.observations[0].availability = crate::Availability::Unsupported("fixture".into());
        let incomplete = analyze_with_distributions(&run, &config).unwrap();
        assert!(incomplete.rows[0].estimates.is_none());
        assert!(incomplete.rows[0].distributions.is_none());
    }

    #[test]
    fn retained_draws_preserve_estimates_joint_samples_and_json_precision() {
        let config = Config {
            resamples: 256,
            seed: 91,
            confidence_level: 0.9,
        };
        let result = estimate_with_distributions(&[2., 4.], &config).unwrap();
        assert_eq!(result.estimates, estimate(&[2., 4.], &config).unwrap());
        assert_eq!(
            result,
            estimate_with_distributions(&[2., 4.], &config).unwrap()
        );
        let d = &result.distributions;
        assert_eq!(d.mean.len(), 256);
        assert_eq!(d.median.len(), 256);
        assert_eq!(d.standard_deviation.len(), 256);
        assert_eq!(d.median_absolute_deviation.len(), 256);
        for i in 0..256 {
            let mean = d.mean[i];
            assert!([2., 3., 4.].contains(&mean));
            assert_eq!(d.median[i], mean);
            let mixed = mean == 3.;
            assert!((d.standard_deviation[i] - if mixed { 2f64.sqrt() } else { 0. }).abs() < 1e-12);
            assert!(
                (d.median_absolute_deviation[i] - if mixed { 1.4826 } else { 0. }).abs() < 1e-12
            );
        }
        let decoded: DistributionEstimates =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(result, decoded);
        for (draws, estimate) in [
            (&d.mean, &result.estimates.mean),
            (&d.median, &result.estimates.median),
            (&d.standard_deviation, &result.estimates.standard_deviation),
            (
                &d.median_absolute_deviation,
                &result.estimates.median_absolute_deviation,
            ),
        ] {
            let mut sorted = draws.clone();
            sorted.sort_by(f64::total_cmp);
            assert!((quantile(&sorted, 0.05) - estimate.lower).abs() < 1e-12);
            assert!((quantile(&sorted, 0.95) - estimate.upper).abs() < 1e-12);
        }
    }

    #[test]
    fn violin_grouping_separates_contracts_sampling_units_and_legacy_rows() {
        let metric = crate::Metric::duration("wall", "operation", "batch_total");
        let row = Row {
            case: "a".into(),
            metric: "wall".into(),
            variant: "candidate".into(),
            unit: "ns".into(),
            metric_contract: Some(metric),
            resampling_unit: "process_median".into(),
            units: 3,
            estimates: None,
            distributions: None,
            regressions: vec![],
            outliers: Some(crate::outliers::classify(&[1., 2., 3.]).unwrap()),
            note: String::new(),
        };
        let mut report = Report {
            case_configs: BTreeMap::new(),
            config: Config::default(),
            method: "fixture".into(),
            rows: vec![row.clone(), row.clone()],
        };
        report.rows[1].case = "b".into();
        let scale = crate::viz::charts::AxisScale::Linear;
        let scales = BTreeMap::from([("a".into(), crate::viz::charts::AxisScale::Logarithmic)]);
        report.rows[0].outliers = Some(crate::outliers::classify(&[0., 1., 2.]).unwrap());
        let configured = html_with_case_scales(&report, None, &scales);
        assert!(
            configured.contains("population contains nonpositive values; no subset was plotted")
        );
        let overridden = html_with_case_scales(&report, Some(scale), &scales);
        assert!(
            !overridden.contains("population contains nonpositive values; no subset was plotted")
        );
        assert_eq!(violin_charts(&report, scale).matches("<svg").count(), 1);
        report.rows[1].metric_contract.as_mut().unwrap().scope = "process".into();
        assert_eq!(violin_charts(&report, scale).matches("<svg").count(), 2);
        report.rows[1] = row.clone();
        report.rows[1].resampling_unit = "normalized_observation".into();
        assert_eq!(violin_charts(&report, scale).matches("<svg").count(), 2);
        for row in &mut report.rows {
            row.metric_contract = None;
        }
        assert_eq!(violin_charts(&report, scale).matches("<svg").count(), 2);
        let mut json = serde_json::to_value(&row).unwrap();
        json.as_object_mut().unwrap().remove("metric_contract");
        let legacy: Row = serde_json::from_value(json).unwrap();
        assert!(legacy.metric_contract.is_none());
        report.rows[1].outliers = None;
        report.rows[1].note = "missing <observation>".into();
        let chart = violin_charts(&report, scale);
        assert_eq!(chart.matches("<svg").count(), 1);
        assert!(chart.contains("missing &lt;observation&gt;"));
    }

    #[test]
    fn density_reports_preserve_population_labels_and_all_observations() {
        let values = [1., 2., 3., 4., 100.];
        let baseline = Report {
            case_configs: BTreeMap::new(),
            config: Config::default(),
            method: "fixture".into(),
            rows: vec![Row {
                metric_contract: None,
                case: "case".into(),
                metric: "wall".into(),
                variant: "default".into(),
                unit: "ns".into(),
                resampling_unit: "process_median".into(),
                units: 5,
                estimates: None,
                distributions: None,
                regressions: vec![],
                outliers: Some(crate::outliers::classify(&values).unwrap()),
                note: String::new(),
            }],
        };
        let expected = crate::density::figure_with_outliers(
            &[crate::density::Series {
                label: "process_median",
                values: &values,
            }],
            "case / wall / default (process_median) density",
            "ns",
        )
        .unwrap();
        assert!(html(&baseline).contains(&expected));
        let mut candidate = baseline.clone();
        candidate.rows[0].resampling_unit = "batch".into();
        candidate.rows[0].outliers = Some(crate::outliers::classify(&[7.; 5]).unwrap());
        let overlay = comparison_charts(&baseline, &candidate);
        assert!(overlay.contains("baseline (process_median)"));
        assert!(overlay.contains("candidate (batch)"));
        assert!(overlay.contains("point mass"));
        assert!(overlay.contains("5 samples; Gaussian KDE"));
        assert!(!overlay.contains("regression comparison"));
    }

    #[test]
    fn descriptive_summary_handles_even_skewed_single_and_invalid_samples() {
        let summary = describe(&[10.0, 1.0, 3.0, 2.0]).unwrap();
        assert_eq!(summary.count, 4);
        assert_eq!((summary.minimum, summary.maximum), (1.0, 10.0));
        assert!((summary.mean - 4.0).abs() < 1e-12);
        assert!((summary.median - 2.5).abs() < 1e-12);
        assert!((summary.standard_deviation.unwrap() - (50.0f64 / 3.0).sqrt()).abs() < 1e-12);
        assert!((summary.median_absolute_deviation - 1.4826).abs() < 1e-12);
        let single = describe(&[7.0]).unwrap();
        assert_eq!(single.mean, 7.0);
        assert_eq!(single.median, 7.0);
        assert_eq!(single.standard_deviation, None);
        assert_eq!(single.median_absolute_deviation, 0.0);
        assert!(describe(&[]).is_err());
        assert!(describe(&[f64::INFINITY]).is_err());
        assert!(describe(&[-f64::MAX, f64::MAX]).is_err());
        assert_eq!(describe(&[1e300; 4]).unwrap().mean, 1e300);
    }
    #[test]
    fn constant_samples_and_seeded_reproducibility() {
        let config = Config {
            resamples: 500,
            ..Default::default()
        };
        let result = estimate(&[7.0; 10], &config).unwrap();
        assert_eq!(
            result.mean,
            Estimate {
                point: 7.0,
                lower: 7.0,
                upper: 7.0,
                standard_error: 0.0
            }
        );
        assert_eq!(result.standard_deviation.point, 0.0);
        assert_eq!(result.median_absolute_deviation.point, 0.0);
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];
        let result = estimate(&values, &config).unwrap();
        assert_eq!(result, estimate(&values, &config).unwrap());
        assert!((result.mean.point - 3.0).abs() < 1e-12);
        assert!((result.standard_deviation.point - 2.5f64.sqrt()).abs() < 1e-12);
        assert!((result.median_absolute_deviation.point - 1.4826).abs() < 1e-12);
        let wider = estimate(
            &values,
            &Config {
                confidence_level: 0.99,
                ..config
            },
        )
        .unwrap();
        assert!(wider.mean.lower <= result.mean.lower && wider.mean.upper >= result.mean.upper);
    }
    #[test]
    fn validates_inputs_and_handles_large_scales() {
        let config = Config {
            resamples: 100,
            ..Default::default()
        };
        assert!(estimate(&[1.0], &config).is_err());
        assert!(estimate(&[1.0, f64::NAN], &config).is_err());
        assert!(
            estimate(
                &[1.0, 2.0],
                &Config {
                    confidence_level: 1.0,
                    ..config.clone()
                }
            )
            .is_err()
        );
        assert!(
            estimate(
                &[1.0, 2.0],
                &Config {
                    resamples: 1,
                    ..config.clone()
                }
            )
            .is_err()
        );
        assert_eq!(estimate(&[1e300; 4], &config).unwrap().mean.point, 1e300);
        assert!(estimate(&[-f64::MAX, f64::MAX], &config).is_err());
    }
    #[test]
    fn seeded_mean_interval_tracks_known_sampling_variance() {
        let mut rng = Seeded::new(42);
        let mut covered = 0;
        for repetition in 0..150 {
            let sample: Vec<_> = (0..40)
                .map(|_| rng.next_u64() as f64 / u64::MAX as f64)
                .collect();
            let result = estimate(
                &sample,
                &Config {
                    resamples: 400,
                    seed: repetition,
                    ..Default::default()
                },
            )
            .unwrap();
            covered += usize::from(result.mean.lower <= 0.5 && result.mean.upper >= 0.5);
            let mean = sample.iter().sum::<f64>() / sample.len() as f64;
            let expected = (sample.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                / (sample.len() * sample.len()) as f64)
                .sqrt();
            assert!((result.mean.standard_error / expected - 1.0).abs() < 0.2);
        }
        assert!((125..=149).contains(&covered), "coverage {covered}/150");
    }
}
