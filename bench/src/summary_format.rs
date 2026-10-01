//! Portable numeric formatter results for one requested parameter summary.
use crate::{
    Metric, Result, Run, error,
    measurement::{FormattedValues, ValueFormatter},
    report::{SummaryFormatters, SummaryPlot},
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
const KEY: &str = "airbug.presentation.parameter-summary.v1";

#[derive(Clone, Serialize, Deserialize)]
struct Conversion {
    typical: f64,
    #[serde(default)]
    work: Option<(f64, String)>,
    values: Vec<f64>,
    formatted: FormattedValues,
}
#[derive(Serialize, Deserialize)]
struct SavedMetric {
    case: String,
    metric: Metric,
    conversions: Vec<Conversion>,
}
#[derive(Serialize, Deserialize)]
struct Snapshot {
    source: String,
    parameter: String,
    estimator: String,
    metrics: Vec<SavedMetric>,
}
fn estimator(config: &SummaryPlot<'_>) -> &'static str {
    match config.estimator {
        crate::summary::Estimator::Mean => "mean",
        crate::summary::Estimator::ProcessMedian => "process-median",
    }
}
fn source(run: &Run) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(crate::model::hex(&Sha256::digest(serde_json::to_vec(&(
        &run.cases,
        &run.observations,
    ))?)))
}
struct Recorder<'a> {
    inner: &'a dyn ValueFormatter,
    calls: RefCell<Vec<Conversion>>,
}
impl ValueFormatter for Recorder<'_> {
    fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues> {
        let formatted = self.inner.scale_values(typical, values)?;
        let mut calls = self.calls.borrow_mut();
        if let Some(previous) = calls
            .iter()
            .find(|call| call.work.is_none() && call.typical == typical && call.values == values)
        {
            if previous.formatted != formatted {
                return Err(error(
                    "summary formatter returned inconsistent values for identical inputs",
                ));
            }
        } else {
            calls.push(Conversion {
                typical,
                work: None,
                values: values.to_vec(),
                formatted: formatted.clone(),
            });
        }
        Ok(formatted)
    }
    fn scale_throughputs(
        &self,
        typical: f64,
        work: f64,
        unit: &str,
        values: &[f64],
    ) -> Result<FormattedValues> {
        let formatted = self.inner.scale_throughputs(typical, work, unit, values)?;
        let key = Some((work, unit.to_owned()));
        let mut calls = self.calls.borrow_mut();
        if let Some(previous) = calls
            .iter()
            .find(|call| call.work == key && call.typical == typical && call.values == values)
        {
            if previous.formatted != formatted {
                return Err(error(
                    "summary throughput formatter returned inconsistent values for identical inputs",
                ));
            }
        } else {
            calls.push(Conversion {
                typical,
                work: key,
                values: values.to_vec(),
                formatted: formatted.clone(),
            });
        }
        Ok(formatted)
    }
    fn scale_for_machines(&self, _: &[f64]) -> Result<FormattedValues> {
        Err(error("summary machine conversion not recorded"))
    }
}
impl ValueFormatter for SavedMetric {
    fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues> {
        self.conversions
            .iter()
            .find(|call| call.work.is_none() && call.typical == typical && call.values == values)
            .map(|call| call.formatted.clone())
            .ok_or_else(|| error("saved summary conversion does not match requested estimate"))
    }
    fn scale_throughputs(
        &self,
        typical: f64,
        work: f64,
        unit: &str,
        values: &[f64],
    ) -> Result<FormattedValues> {
        let key = Some((work, unit.to_owned()));
        self.conversions.iter().find(|call| call.work == key && call.typical == typical && call.values == values)
            .map(|call| call.formatted.clone()).ok_or_else(|| error("saved summary has no matching throughput conversion; regenerate with live formatters"))
    }
    fn scale_for_machines(&self, _: &[f64]) -> Result<FormattedValues> {
        Err(error("saved summary has no machine conversion"))
    }
}
/// Validate and capture aggregate conversions before changing provenance. This
/// records numbers and units, never callback code or trusted HTML.
pub fn save(
    run: &mut Run,
    config: &SummaryPlot<'_>,
    formatters: &SummaryFormatters<'_>,
) -> Result<()> {
    if formatters.is_empty() {
        run.provenance.remove(KEY);
        return Ok(());
    }
    let recorders: Vec<_> = formatters
        .iter()
        .map(|(_, _, formatter)| Recorder {
            inner: *formatter,
            calls: RefCell::new(Vec::new()),
        })
        .collect();
    let refs: Vec<_> = formatters
        .iter()
        .zip(&recorders)
        .map(|((case, metric, _), recorder)| (*case, *metric, recorder as &dyn ValueFormatter))
        .collect();
    crate::report::parameter_charts_with_formatters(run, config, &refs)?;
    let metrics = formatters
        .iter()
        .zip(recorders)
        .map(|((case, metric, _), recorder)| SavedMetric {
            case: (*case).into(),
            metric: (*metric).clone(),
            conversions: recorder.calls.into_inner(),
        })
        .filter(|metric| !metric.conversions.is_empty())
        .collect();
    let saved = Snapshot {
        source: source(run)?,
        parameter: config.parameter.into(),
        estimator: estimator(config).into(),
        metrics,
    };
    run.provenance
        .insert(KEY.into(), serde_json::to_string(&saved)?);
    Ok(())
}
/// Restore only the recorded parameter/estimator. Changing axis scale is safe;
/// changing the statistical population requires live callbacks again.
pub fn charts(run: &Run, config: &SummaryPlot<'_>) -> Result<Option<String>> {
    let Some(raw) = run.provenance.get(KEY) else {
        return Ok(None);
    };
    let saved: Snapshot = serde_json::from_str(raw)?;
    if saved.source != source(run)? {
        return Err(error("stale parameter-summary formatting snapshot"));
    }
    if saved.parameter != config.parameter || saved.estimator != estimator(config) {
        return Err(error(
            "saved parameter-summary formatting covers another parameter or estimator; regenerate with live formatters",
        ));
    }
    let refs: Vec<_> = saved
        .metrics
        .iter()
        .map(|metric| {
            (
                metric.case.as_str(),
                &metric.metric,
                metric as &dyn ValueFormatter,
            )
        })
        .collect();
    Ok(Some(crate::report::parameter_charts_with_formatters(
        run, config, &refs,
    )?))
}
