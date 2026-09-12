//! Distribution-free order-statistic intervals for the median of independent
//! process pairs. No significance claims from intra-process batches.
use crate::{error, Availability, Direction, Metric, Result, Run, Status};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub fn median(v: &[f64]) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 0 {
        v[n / 2 - 1] / 2.0 + v[n / 2] / 2.0
    } else {
        v[n / 2]
    }
}
/// Exact binomial coverage under independent observations from a continuous
/// population. Ties make the interval conservative. None = insufficient units.
pub fn median_interval(values: &[f64], alpha: f64) -> Option<(f64, f64)> {
    if values.is_empty() || !(0.0..1.0).contains(&alpha) || values.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n = values.len();
    // Stable binomial PMF: normalize relative probabilities around the mode.
    let m = n / 2;
    let mut pmf = vec![0.0; n + 1];
    pmf[m] = 1.0;
    for j in (1..=m).rev() {
        pmf[j - 1] = pmf[j] * j as f64 / (n - j + 1) as f64;
    }
    for j in m..n {
        pmf[j + 1] = pmf[j] * (n - j) as f64 / (j + 1) as f64;
    }
    let total: f64 = pmf.iter().sum();
    let mut tail = 0.0;
    let mut k = None;
    for (j, p) in pmf.iter().enumerate().take(n.div_ceil(2)) {
        tail += p / total;
        if 2.0 * tail <= alpha {
            k = Some(j);
        } else {
            break;
        }
    }
    let k = k?;
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    Some((v[k], v[n - 1 - k]))
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Regression,
    Improvement,
    WithinMargin,
    Inconclusive,
    Unavailable,
    Neutral,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub case: String,
    pub metric: String,
    pub unit: String,
    pub scope: String,
    pub baseline: Option<f64>,
    pub candidate: Option<f64>,
    pub change_percent: Option<f64>,
    pub interval_percent: Option<(f64, f64)>,
    pub independent_units: usize,
    pub decision: Decision,
    pub note: String,
}
fn values(
    run: &Run,
    case: &str,
    metric: &Metric,
    variant: &str,
) -> Result<BTreeMap<u32, (Option<u32>, f64)>> {
    let mut groups: BTreeMap<u32, (Option<u32>, Vec<f64>)> = BTreeMap::new();
    for o in run
        .observations
        .iter()
        .filter(|o| o.case == case && o.metric == metric.id && o.variant == variant)
    {
        if o.availability != Availability::Available {
            return Err(error("metric has unavailable observations"));
        }
        let n = o.number()?.ok_or_else(|| error("missing numeric value"))?;
        let n = if metric.statistic == "batch_total" {
            n / o.operations as f64
        } else {
            n
        };
        let group = groups.entry(o.process).or_insert((o.pair, vec![]));
        if group.0 != o.pair {
            return Err(error("process has inconsistent pair IDs"));
        }
        group.1.push(n);
    }
    if groups.is_empty() {
        return Err(error("variant missing"));
    }
    Ok(groups
        .into_iter()
        .map(|(id, (pair, v))| (id, (pair, median(&v))))
        .collect())
}
pub fn compare(a: &Run, b: Option<&Run>, threshold: f64, alpha: f64) -> Result<Vec<Comparison>> {
    a.validate()?;
    let allowed = |r: &Run, paired: bool| {
        let variants: BTreeSet<_> = r.observations.iter().map(|o| o.variant.as_str()).collect();
        if paired {
            variants == BTreeSet::from(["baseline", "candidate"])
        } else {
            variants == BTreeSet::from(["candidate"])
        }
    };
    if !allowed(a, b.is_none()) || b.is_some_and(|r| !allowed(r, false)) {
        return Err(error("comparison variant set is incomplete or unexpected"));
    }
    let other = b.unwrap_or(a);
    other.validate()?;
    if a.status != Status::Complete || other.status != Status::Complete {
        return Err(error("comparison requires complete runs"));
    }
    if !threshold.is_finite()
        || !(0.0..100.0).contains(&threshold)
        || !alpha.is_finite()
        || alpha <= 0.0
        || alpha >= 1.0
    {
        return Err(error(
            "threshold must be 0..100 percent; alpha must be 0..1",
        ));
    }
    if a.environment != other.environment {
        return Err(error("incompatible environments"));
    }
    let ac: BTreeMap<_, _> = a.cases.iter().map(|c| (&c.id, c)).collect();
    let bc: BTreeMap<_, _> = other.cases.iter().map(|c| (&c.id, c)).collect();
    if ac != bc {
        return Err(error("incompatible cases, metrics or workload contracts"));
    }
    let family = a
        .cases
        .iter()
        .map(|c| {
            c.metrics
                .iter()
                .filter(|m| m.direction != Direction::Neutral)
                .count()
        })
        .sum::<usize>()
        .max(1);
    let corrected = alpha / family as f64;
    let mut result = vec![];
    for c in &a.cases {
        for m in &c.metrics {
            let mut row = Comparison {
                case: c.id.clone(),
                metric: m.id.clone(),
                unit: m.unit.clone(),
                scope: m.scope.clone(),
                baseline: None,
                candidate: None,
                change_percent: None,
                interval_percent: None,
                independent_units: 0,
                decision: Decision::Inconclusive,
                note: String::new(),
            };
            let va = values(
                a,
                &c.id,
                m,
                if b.is_some() { "candidate" } else { "baseline" },
            );
            let vb = values(other, &c.id, m, "candidate");
            let (va, vb) = match (va, vb) {
                (Ok(a), Ok(b)) => (a, b),
                (a, b) => {
                    row.decision = Decision::Unavailable;
                    row.note = format!(
                        "{}; {}",
                        a.err().map(|e| e.to_string()).unwrap_or_default(),
                        b.err().map(|e| e.to_string()).unwrap_or_default()
                    );
                    result.push(row);
                    continue;
                }
            };
            let av: Vec<_> = va.values().map(|x| x.1).collect();
            let bv: Vec<_> = vb.values().map(|x| x.1).collect();
            let am = median(&av);
            let bm = median(&bv);
            row.baseline = Some(am);
            row.candidate = Some(bm);
            if am > 0.0 {
                row.change_percent = Some((bm / am - 1.0) * 100.0);
            }
            row.independent_units = av.len().min(bv.len());
            if b.is_none() {
                let to_pairs =
                    |v: &BTreeMap<u32, (Option<u32>, f64)>| -> Result<BTreeMap<u32, f64>> {
                        let mut p = BTreeMap::new();
                        for (id, n) in v.values() {
                            let id = id.ok_or_else(|| error("missing pair ID"))?;
                            if p.insert(id, *n).is_some() {
                                return Err(error("duplicate process for pair"));
                            }
                        }
                        Ok(p)
                    };
                let pa = to_pairs(&va)?;
                let pb = to_pairs(&vb)?;
                if pa.keys().collect::<BTreeSet<_>>() != pb.keys().collect::<BTreeSet<_>>() {
                    return Err(error("incomplete A/B pairs"));
                }
                let ratios: Vec<_> = pa
                    .iter()
                    .filter(|(_, a)| **a > 0.0)
                    .map(|(id, a)| pb[id] / a - 1.0)
                    .collect();
                if ratios.len() == pa.len() {
                    row.change_percent = Some(median(&ratios) * 100.0);
                    row.interval_percent =
                        median_interval(&ratios, corrected).map(|(l, h)| (l * 100.0, h * 100.0));
                }
                row.note="Median of process-pair ratios; exact median interval, Bonferroni family correction. Independence across pairs assumed; background drift is not controlled.".into();
            } else {
                if let (Some((al, ah)), Some((bl, bh))) = (
                    median_interval(&av, corrected / 2.0),
                    median_interval(&bv, corrected / 2.0),
                ) {
                    if al > 0.0 && ah > 0.0 {
                        row.interval_percent =
                            Some(((bl / ah - 1.0) * 100.0, (bh / al - 1.0) * 100.0));
                    }
                }
                row.note="Ratio of process medians; conservative simultaneous median intervals, Bonferroni correction. Historical runs may be confounded by environment drift.".into();
            }
            if row.change_percent.is_some_and(|v| !v.is_finite())
                || row
                    .interval_percent
                    .is_some_and(|(l, h)| !l.is_finite() || !h.is_finite())
            {
                row.change_percent = None;
                row.interval_percent = None;
                row.note
                    .push_str(" Relative effect overflow: no numeric ratio reported.");
            }
            if m.direction == Direction::Neutral {
                row.decision = Decision::Neutral;
            } else if let Some((lo, hi)) = row.interval_percent {
                row.decision = if lo > threshold {
                    if m.direction == Direction::Lower {
                        Decision::Regression
                    } else {
                        Decision::Improvement
                    }
                } else if hi < -threshold {
                    if m.direction == Direction::Lower {
                        Decision::Improvement
                    } else {
                        Decision::Regression
                    }
                } else if lo >= -threshold && hi <= threshold {
                    Decision::WithinMargin
                } else {
                    Decision::Inconclusive
                };
            }
            result.push(row);
        }
    }
    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct MultiComparison {
    pub reference: String,
    pub variant: String,
    pub comparison: Comparison,
}
/// Every variant against one predeclared reference; family correction spans variants AND metrics.
/// No total ranking is implied by intervals overlapping or an inconclusive comparison.
pub fn compare_multi(
    run: &Run,
    reference: &str,
    threshold: f64,
    alpha: f64,
) -> Result<Vec<MultiComparison>> {
    run.validate()?;
    let names: BTreeSet<_> = run
        .observations
        .iter()
        .map(|o| o.variant.as_str())
        .collect();
    if !names.contains(reference) || names.len() < 2 {
        return Err(error("reference and at least one alternative required"));
    }
    let mut result = vec![];
    for name in names.iter().filter(|n| **n != reference) {
        let mut pair = run.clone();
        pair.observations
            .retain(|o| o.variant == reference || o.variant == *name);
        for o in &mut pair.observations {
            o.variant = if o.variant == reference {
                "baseline"
            } else {
                "candidate"
            }
            .into();
        }
        for comparison in compare(&pair, None, threshold, alpha / (names.len() - 1) as f64)? {
            result.push(MultiComparison {
                reference: reference.into(),
                variant: (*name).into(),
                comparison,
            });
        }
    }
    Ok(result)
}
