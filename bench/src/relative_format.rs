//! Pair-specific custom throughput results for saved-run comparisons.
use crate::{Result, Run, bootstrap::Config, relative::Comparison};
use serde::{Deserialize, Serialize};
const KEY: &str = "airbug.presentation.relative-throughput.v1";
#[derive(Serialize, Deserialize)]
struct Saved {
    case: String,
    metric: String,
    baseline: String,
    candidate: String,
    config: Config,
    rates: Vec<crate::relative::ThroughputChange>,
}
pub(crate) fn source(run: &Run, case: &str, metric: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let c = run
        .cases
        .iter()
        .find(|c| c.id == case)
        .ok_or("missing comparison case")?;
    let m = c
        .metrics
        .iter()
        .find(|m| m.id == metric)
        .ok_or("missing comparison metric")?;
    let mut contract = c.contract.clone();
    contract.remove("samples");
    contract.remove("quick.stop");
    let observations: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case == case && o.metric == metric)
        .collect();
    Ok(crate::model::hex(&Sha256::digest(serde_json::to_vec(&(
        &contract,
        m,
        observations,
    ))?)))
}
/// Persist only formatter-derived rates. Source hashes are per metric, so case
/// and metric filtering do not invalidate unrelated comparisons.
pub fn save(
    baseline: &Run,
    candidate: &mut Run,
    config: &Config,
    rows: &[Comparison],
) -> Result<()> {
    config.validate()?;
    let mut saved: Vec<Saved> = candidate
        .provenance
        .get(KEY)
        .map(|s| serde_json::from_str(s))
        .transpose()?
        .unwrap_or_default();
    for row in rows
        .iter()
        .filter(|r| r.throughput.iter().any(|r| r.method.is_some()))
    {
        let item = Saved {
            case: row.case.clone(),
            metric: row.metric.clone(),
            baseline: source(baseline, &row.case, &row.metric)?,
            candidate: source(candidate, &row.case, &row.metric)?,
            config: config.clone(),
            rates: row.throughput.clone(),
        };
        saved.retain(|s| s.case != item.case || s.metric != item.metric);
        saved.push(item);
    }
    candidate
        .provenance
        .insert(KEY.into(), serde_json::to_string(&saved)?);
    Ok(())
}
pub(crate) fn restore(
    baseline: &Run,
    candidate: &Run,
    config: &Config,
    rows: &mut [Comparison],
) -> Result<()> {
    let Some(raw) = candidate.provenance.get(KEY) else {
        return Ok(());
    };
    let saved: Vec<Saved> = serde_json::from_str(raw)?;
    for row in rows {
        let Some(s) = saved
            .iter()
            .find(|s| s.case == row.case && s.metric == row.metric)
        else {
            continue;
        };
        if s.candidate != source(candidate, &row.case, &row.metric)? {
            return Err(crate::error("stale custom throughput comparison snapshot"));
        }
        let reason = if s.baseline != source(baseline, &row.case, &row.metric)? {
            Some(
                "Saved custom throughput comparison uses another baseline; regenerate with live formatters.",
            )
        } else if s.config.resamples != config.resamples
            || s.config.seed != config.seed
            || s.config.confidence_level != config.confidence_level
        {
            Some(
                "Saved custom throughput comparison uses different bootstrap settings; regenerate with live formatters.",
            )
        } else {
            None
        };
        row.throughput = s.rates.clone();
        if let Some(reason) = reason {
            for rate in &mut row.throughput {
                rate.point_percent = None;
                rate.interval_percent = None;
                rate.unavailable_reason = Some(reason.into());
            }
        }
    }
    Ok(())
}
