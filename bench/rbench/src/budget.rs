//! Declarative exact-case budgets. Missing metrics and uncertainty never silently pass.
use crate::{analysis, error, Result, Run, Status};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetFile {
    pub budgets: Vec<Budget>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub case: String,
    pub metric: String,
    pub unit: String,
    pub max: Option<f64>,
    pub min: Option<f64>,
    pub max_regression_percent: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Outcome {
    pub case: String,
    pub metric: String,
    pub decision: String,
    pub detail: String,
}
fn select(run: &Run, case: &str, metric: &str) -> Result<Run> {
    let mut r = run.clone();
    r.cases.retain(|c| c.id == case);
    for c in &mut r.cases {
        c.metrics.retain(|m| m.id == metric);
    }
    r.observations
        .retain(|o| o.case == case && o.metric == metric);
    r.validate()?;
    Ok(r)
}
pub fn evaluate(config: &BudgetFile, run: &Run, baseline: Option<&Run>) -> Result<Vec<Outcome>> {
    run.validate()?;
    if run.status != Status::Complete {
        return Err(error("budget requires complete run"));
    }
    if config.budgets.is_empty() {
        return Err(error("budget configuration is empty"));
    }
    let mut seen = std::collections::BTreeSet::new();
    let family = config
        .budgets
        .iter()
        .filter(|b| b.max_regression_percent.is_some())
        .count()
        .max(1);
    let mut out = vec![];
    for b in &config.budgets {
        if !seen.insert((&b.case, &b.metric))
            || [b.max, b.min, b.max_regression_percent]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite() || v < 0.)
            || (b.max.is_none() && b.min.is_none() && b.max_regression_percent.is_none())
            || b.min.zip(b.max).is_some_and(|(a, z)| a > z)
        {
            return Err(error(
                "invalid/duplicate budget; use finite nonnegative bounds",
            ));
        }
        let check = || -> Result<(String, String)> {
            let r = select(run, &b.case, &b.metric)?;
            let m = r
                .cases
                .first()
                .and_then(|c| c.metrics.first())
                .ok_or_else(|| error("required case/metric missing"))?;
            if m.unit != b.unit {
                return Err(error(format!(
                    "unit mismatch: expected {}, found {}",
                    b.unit, m.unit
                )));
            }
            let mut count = 0;
            let mut violated = false;
            for o in r.observations.iter().filter(|o| o.variant == "candidate") {
                let n = o
                    .number()?
                    .ok_or_else(|| error("required candidate metric unavailable"))?;
                let n = if m.statistic == "batch_total" {
                    n / o.operations as f64
                } else {
                    n
                };
                violated |= b.max.is_some_and(|v| n > v) || b.min.is_some_and(|v| n < v);
                count += 1;
            }
            if count == 0 {
                return Err(error("candidate observations missing"));
            }
            if violated {
                return Ok((
                    "failed".into(),
                    format!("absolute bound exceeded among {count} candidate observations"),
                ));
            }
            if let Some(threshold) = b.max_regression_percent {
                let a = baseline
                    .map(|v| select(v, &b.case, &b.metric))
                    .transpose()?;
                let rows = if let Some(a) = a.as_ref() {
                    analysis::compare(a, Some(&r), threshold, 0.05 / family as f64)?
                } else {
                    analysis::compare(&r, None, threshold, 0.05 / family as f64)?
                };
                let result = &rows[0];
                use analysis::Decision::*;
                return Ok((
                    match result.decision {
                        Regression => "failed",
                        Improvement | WithinMargin => "passed",
                        _ => "inconclusive",
                    }
                    .into(),
                    format!(
                        "{:?}; interval {:?}; Bonferroni across {family} relative budgets",
                        result.decision, result.interval_percent
                    ),
                ));
            }
            Ok((
                "passed".into(),
                format!("{count} candidate observations satisfy absolute bounds"),
            ))
        };
        let (decision, detail) = match check() {
            Ok(v) => v,
            Err(e) => ("unavailable".into(), e.to_string()),
        };
        out.push(Outcome {
            case: b.case.clone(),
            metric: b.metric.clone(),
            decision,
            detail,
        });
    }
    Ok(out)
}
pub fn exit_code(rows: &[Outcome]) -> i32 {
    if rows.iter().any(|r| r.decision == "failed") {
        1
    } else if rows.iter().any(|r| r.decision != "passed") {
        2
    } else {
        0
    }
}
