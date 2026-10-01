//! Pair-specific regression displays for comparisons without live formatter code.
use crate::{Metric, Result, Run, bootstrap::Config, measurement::ValueFormatter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const KEY: &str = "airbug.presentation.regression-comparison.v1";
#[derive(Serialize, Deserialize)]
struct Saved {
    case: String,
    metric: String,
    baseline: String,
    candidate: String,
    config: Config,
    charts: Vec<(String, crate::regression::ComparisonPresentation)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    statistics: Option<crate::bootstrap::Report>,
    #[serde(default)]
    statistics_unavailable: bool,
}

/// Store a shared-scale comparison, bound to both original populations and settings.
/// The candidate is changed only after all formatting and serialization succeed.
pub fn save(
    baseline: &Run,
    candidate: &mut Run,
    case: &str,
    metric: &Metric,
    config: &Config,
    formatter: &dyn ValueFormatter,
) -> Result<()> {
    config.validate()?;
    let before = crate::history::one_case(baseline, case);
    let after = crate::history::one_case(candidate, case);
    crate::analysis::validate_comparison(&before, Some(&after), 0., 0.05)?;
    let before_report = crate::bootstrap::analyze_with_distributions(&before, config)?;
    let after_report = crate::bootstrap::analyze_with_distributions(&after, config)?;
    let charts = crate::measurement::regression_comparison_presentations(
        &before_report,
        &after_report,
        case,
        metric,
        formatter,
    )?;
    let statistics = if before_report
        .rows
        .iter()
        .chain(&after_report.rows)
        .filter(|r| r.metric == metric.id)
        .all(|r| r.distributions.is_some())
    {
        Some(crate::statistic_format::comparison_report(
            &before_report,
            &after_report,
            case,
            metric,
            formatter,
        )?)
    } else {
        None
    };
    let mut saved: Vec<Saved> = candidate
        .provenance
        .get(KEY)
        .map(|s| serde_json::from_str(s))
        .transpose()?
        .unwrap_or_default();
    saved.retain(|s| s.case != case || s.metric != metric.id);
    {
        // Validate generated geometry before publishing a snapshot.
        for (_, chart) in &charts {
            chart.validate()?;
        }
        saved.push(Saved {
            case: case.into(),
            metric: metric.id.clone(),
            baseline: crate::relative_format::source(baseline, case, &metric.id)?,
            candidate: crate::relative_format::source(candidate, case, &metric.id)?,
            config: config.clone(),
            charts,
            statistics_unavailable: statistics.is_none(),
            statistics,
        });
    }
    candidate
        .provenance
        .insert(KEY.into(), serde_json::to_string(&saved)?);
    Ok(())
}

/// Render matching saved displays. A changed candidate is an error; another
/// baseline/config gets an explicit unavailable notice. Filtering skips other metrics.
pub fn charts(
    baseline: &Run,
    candidate: &Run,
    fallback: &Config,
    cases: &BTreeMap<String, Config>,
) -> Result<String> {
    fallback.validate()?;
    for config in cases.values() {
        config.validate()?;
    }
    let Some(raw) = candidate.provenance.get(KEY) else {
        return Ok(String::new());
    };
    let saved: Vec<Saved> = serde_json::from_str(raw)?;
    let mut output = String::new();
    for item in saved {
        if !candidate
            .cases
            .iter()
            .any(|c| c.id == item.case && c.metrics.iter().any(|m| m.id == item.metric))
            || !baseline
                .cases
                .iter()
                .any(|c| c.id == item.case && c.metrics.iter().any(|m| m.id == item.metric))
        {
            continue;
        }
        if item.candidate != crate::relative_format::source(candidate, &item.case, &item.metric)? {
            return Err(crate::error("stale saved regression comparison snapshot"));
        }
        let config = cases.get(&item.case).unwrap_or(fallback);
        let reason = if item.baseline
            != crate::relative_format::source(baseline, &item.case, &item.metric)?
        {
            Some("another baseline")
        } else if item.config.resamples != config.resamples
            || item.config.seed != config.seed
            || item.config.confidence_level != config.confidence_level
        {
            Some("different bootstrap settings")
        } else {
            None
        };
        if let Some(reason) = reason {
            output.push_str(&format!("<p>{} / {}: saved regression comparison unavailable: {}. Regenerate with live formatters.</p>",
                crate::report::escape(&item.case), crate::report::escape(&item.metric), reason));
            continue;
        }
        output.push_str(&format!("<p>Saved formatter comparison. Original slope confidence level: {}. Independent process fits; intervals are not family-adjusted.</p>", config.confidence_level));
        if let Some(statistics) = &item.statistics {
            output.push_str(&crate::statistic_format::render_comparison(
                statistics,
                &item.case,
                &item.metric,
            )?);
        } else if item.statistics_unavailable {
            output.push_str(&format!("<p>{} / {}: transformed statistical comparison unavailable: insufficient bootstrap samples.</p>", crate::report::escape(&item.case), crate::report::escape(&item.metric)));
        }
        for (title, chart) in item.charts {
            output.push_str(&chart.figure(&title)?);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Square(bool);
    impl ValueFormatter for Square {
        fn scale_values(
            &self,
            _: f64,
            values: &[f64],
        ) -> Result<crate::measurement::FormattedValues> {
            if self.0 {
                return Err(crate::error("formatter failed"));
            }
            Ok(crate::measurement::FormattedValues {
                values: values.iter().map(|v| v * v).collect(),
                unit: "squared<&>".into(),
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
    fn saved_pair_restores_and_rejects_stale_or_malformed_data_atomically() {
        let metric = Metric::duration("wall", "batch", "batch_total");
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "case".into(),
                contract: BTreeMap::new(),
                metrics: vec![metric.clone()],
            })
            .unwrap();
        recorder.observe("case", "wall", 2).unwrap();
        recorder.observe("case", "wall", 4).unwrap();
        let mut before = recorder.finish().unwrap();
        before.observations[1].operations = 2;
        let mut after = before.clone();
        after.observations[0].value = Some("4".into());
        after.observations[1].value = Some("8".into());
        let config = Config {
            resamples: 32,
            ..Config::default()
        };
        save(
            &before,
            &mut after,
            "case",
            &metric,
            &config,
            &Square(false),
        )
        .unwrap();
        let original = serde_json::to_vec(&after).unwrap();
        let restored: Run = serde_json::from_slice(&original).unwrap();
        let html = charts(&before, &restored, &config, &BTreeMap::new()).unwrap();
        assert!(html.contains("baseline process 0: x = 1, y = 4"));
        assert!(html.contains("candidate process 0: x = 1, y = 16"));
        assert!(html.contains("squared&lt;&amp;&gt;"));
        assert!(html.contains("formatter(mean) comparison"));
        let mut malformed_statistics = restored.clone();
        let mut snapshots: serde_json::Value =
            serde_json::from_str(&malformed_statistics.provenance[KEY]).unwrap();
        snapshots[0]["statistics"]["rows"][0]["estimates"]["mean"]["point"] =
            serde_json::json!(999.);
        malformed_statistics
            .provenance
            .insert(KEY.into(), serde_json::to_string(&snapshots).unwrap());
        assert!(
            charts(&before, &malformed_statistics, &config, &BTreeMap::new())
                .unwrap_err()
                .to_string()
                .contains("does not match source")
        );
        let mut legacy = restored.clone();
        let mut snapshots: serde_json::Value =
            serde_json::from_str(&legacy.provenance[KEY]).unwrap();
        snapshots[0].as_object_mut().unwrap().remove("statistics");
        snapshots[0]
            .as_object_mut()
            .unwrap()
            .remove("statistics_unavailable");
        legacy
            .provenance
            .insert(KEY.into(), serde_json::to_string(&snapshots).unwrap());
        let legacy_html = charts(&before, &legacy, &config, &BTreeMap::new()).unwrap();
        assert!(legacy_html.contains("candidate process 0"));
        assert!(!legacy_html.contains("formatter(mean) comparison"));

        assert!(save(&before, &mut after, "case", &metric, &config, &Square(true)).is_err());
        assert_eq!(serde_json::to_vec(&after).unwrap(), original);
        let changed_config = Config {
            seed: config.seed + 1,
            ..config.clone()
        };
        assert!(
            charts(&before, &after, &changed_config, &BTreeMap::new())
                .unwrap()
                .contains("different bootstrap settings")
        );
        let mut other_before = before.clone();
        other_before.observations[0].value = Some("3".into());
        assert!(
            charts(&other_before, &after, &config, &BTreeMap::new())
                .unwrap()
                .contains("another baseline")
        );
        let mut stale = after.clone();
        stale.observations[0].value = Some("5".into());
        assert!(
            charts(&before, &stale, &config, &BTreeMap::new())
                .unwrap_err()
                .to_string()
                .contains("stale saved")
        );
        let mut bad: serde_json::Value = serde_json::from_str(&after.provenance[KEY]).unwrap();
        bad[0]["charts"][0][1]["series"][0]["presentation"]["coordinates"]["band"] =
            serde_json::json!([]);
        after
            .provenance
            .insert(KEY.into(), serde_json::to_string(&bad).unwrap());
        assert!(
            charts(&before, &after, &config, &BTreeMap::new())
                .unwrap_err()
                .to_string()
                .contains("invalid coordinates")
        );
    }
}
