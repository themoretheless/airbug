//! Descriptive diagnostics and a separately planned confirmatory design. No automatic rerun-until-pass.
use crate::{
    analysis::{median, median_interval},
    error, Result, Run, Status,
};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub processes: usize,
    pub median: f64,
    pub relative_mad_percent: Option<f64>,
    pub first_to_last_half_percent: Option<f64>,
    pub flags: Vec<String>,
}
pub type ProcessValues = BTreeMap<(String, String, String), BTreeMap<u32, f64>>;
pub fn process_values(r: &Run) -> Result<ProcessValues> {
    r.validate()?;
    let mut groups: BTreeMap<_, BTreeMap<u32, Vec<f64>>> = BTreeMap::new();
    for o in &r.observations {
        let Some(mut n) = o.number()? else { continue };
        let metric = r
            .cases
            .iter()
            .find(|c| c.id == o.case)
            .unwrap()
            .metrics
            .iter()
            .find(|m| m.id == o.metric)
            .unwrap();
        if metric.statistic == "batch_total" {
            n /= o.operations as f64;
        }
        groups
            .entry((o.case.clone(), o.metric.clone(), o.variant.clone()))
            .or_default()
            .entry(o.process)
            .or_default()
            .push(n);
    }
    Ok(groups
        .into_iter()
        .map(|(k, g)| (k, g.into_iter().map(|(p, v)| (p, median(&v))).collect()))
        .collect())
}
pub fn diagnose(r: &Run) -> Result<Vec<Diagnostic>> {
    let mut rows = vec![];
    for ((case, metric, variant), values) in process_values(r)? {
        let v: Vec<_> = values.values().copied().collect();
        let m = median(&v);
        let mad = median(&v.iter().map(|n| (n - m).abs()).collect::<Vec<_>>());
        let relative = (m > 0.).then_some(mad / m * 100.);
        let drift = if v.len() >= 4 {
            let middle = v.len() / 2;
            let first = median(&v[..middle]);
            (first > 0.).then(|| (median(&v[middle..]) / first - 1.) * 100.)
        } else {
            None
        };
        let mut flags = vec![];
        if relative.is_some_and(|x| x > 5.) {
            flags.push("between-process spread exceeds 5% MAD".into());
        }
        if drift.is_some_and(|x| x.abs() > 5.) {
            flags.push("chronological half-to-half shift exceeds 5%".into());
        }
        if v.len() < 6 {
            flags.push("few independent processes; diagnostics limited".into());
        }
        rows.push(Diagnostic {
            case,
            metric,
            variant,
            processes: v.len(),
            median: m,
            relative_mad_percent: relative,
            first_to_last_half_percent: drift,
            flags,
        });
    }
    Ok(rows)
}
#[derive(Debug, Serialize)]
pub struct OrderEffect {
    pub case: String,
    pub metric: String,
    pub ab_pairs: usize,
    pub ba_pairs: usize,
    pub ab_ratio: Option<f64>,
    pub ba_ratio: Option<f64>,
    pub flagged: bool,
}
pub fn order_effects(r: &Run) -> Result<Vec<OrderEffect>> {
    let values = process_values(r)?;
    let mut out = vec![];
    for c in &r.cases {
        for m in &c.metrics {
            let mut orders: [Vec<f64>; 2] = Default::default();
            let mut pairs: BTreeMap<u32, BTreeMap<&str, u32>> = BTreeMap::new();
            for o in r
                .observations
                .iter()
                .filter(|o| o.case == c.id && o.metric == m.id)
            {
                if let Some(p) = o.pair {
                    pairs.entry(p).or_default().insert(&o.variant, o.process);
                }
            }
            for variants in pairs.values() {
                if let (Some(a), Some(b)) = (variants.get("baseline"), variants.get("candidate")) {
                    let x = values
                        .get(&(c.id.clone(), m.id.clone(), "baseline".into()))
                        .and_then(|v| v.get(a));
                    let y = values
                        .get(&(c.id.clone(), m.id.clone(), "candidate".into()))
                        .and_then(|v| v.get(b));
                    if let (Some(x), Some(y)) = (x, y) {
                        if *x > 0. {
                            orders[usize::from(a > b)].push(y / x);
                        }
                    }
                }
            }
            let ab = (!orders[0].is_empty()).then(|| median(&orders[0]));
            let ba = (!orders[1].is_empty()).then(|| median(&orders[1]));
            out.push(OrderEffect {
                case: c.id.clone(),
                metric: m.id.clone(),
                ab_pairs: orders[0].len(),
                ba_pairs: orders[1].len(),
                ab_ratio: ab,
                ba_ratio: ba,
                flagged: orders.iter().all(|v| v.len() >= 3)
                    && ab
                        .zip(ba)
                        .is_some_and(|(a, b)| b > 0. && (a / b - 1.).abs() > 0.05),
            });
        }
    }
    Ok(out)
}
#[derive(Debug, Serialize)]
pub struct Pilot {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub observed_processes: usize,
    pub proposed_processes: Option<usize>,
    pub reason: String,
}
/// Heuristic precision planning from an exact pilot median interval. Confirmation MUST be new data.
pub fn pilot(r: &Run, target_percent: f64, max_processes: usize) -> Result<Vec<Pilot>> {
    if r.status != Status::Complete
        || !target_percent.is_finite()
        || target_percent <= 0.
        || target_percent > 100.
        || !(6..=10000).contains(&max_processes)
    {
        return Err(error(
            "complete pilot, target 0..100%, max-processes 6..10000 required",
        ));
    }
    if r.observations
        .iter()
        .any(|o| o.availability != crate::Availability::Available)
    {
        return Err(error("pilot requires all planned observations available"));
    }
    let mut out = vec![];
    for ((case, metric, variant), g) in process_values(r)? {
        let values: Vec<_> = g.values().copied().collect();
        let m = median(&values);
        let (proposed, reason) = if let Some((l, h)) =
            median_interval(&values, 0.05).filter(|_| m > 0.)
        {
            let half = (h - l) / 2. / m * 100.;
            let n =
                ((values.len() as f64 * (half / target_percent).powi(2)).ceil() as usize).max(6);
            (Some(n.min(max_processes)),format!("Heuristic sqrt(n) extrapolation; {}. Fixed new confirmation only; no guarantee under drift/dependence",if n>max_processes{"requested precision exceeds process cap"}else{"not a promised precision or power"}))
        } else {
            (None,"Pilot too small or zero median; obtain a separately declared pilot with >=6 independent processes. Do not pool pilot with confirmation.".into())
        };
        out.push(Pilot {
            case,
            metric,
            variant,
            observed_processes: values.len(),
            proposed_processes: proposed,
            reason,
        });
    }
    Ok(out)
}
#[derive(Debug, Serialize)]
pub struct Deadline {
    pub hz: f64,
    pub budget_ns: f64,
    pub observations: usize,
    pub over_budget: usize,
    pub over_budget_percent: f64,
    pub p95_ns: f64,
}
/// Only raw individual frame durations are accepted; no inference of compositor-dropped frames.
pub fn deadlines(r: &Run, case: &str, metric: &str, hz: &[f64]) -> Result<Vec<Deadline>> {
    r.validate()?;
    if r.status != Status::Complete {
        return Err(error("deadlines require complete run"));
    }
    let m = r
        .cases
        .iter()
        .find(|c| c.id == case)
        .and_then(|c| c.metrics.iter().find(|m| m.id == metric))
        .ok_or_else(|| error("frame metric missing"))?;
    if m.unit != "ns" || m.statistic != "individual frame" {
        return Err(error(
            "deadlines require raw individual frame duration in ns",
        ));
    }
    let mut v = vec![];
    for o in r
        .observations
        .iter()
        .filter(|o| o.case == case && o.metric == metric && o.variant == "candidate")
    {
        v.push(
            o.number()?
                .ok_or_else(|| error("frame duration unavailable"))?,
        );
    }
    if v.is_empty() || hz.is_empty() {
        return Err(error("frame observations/rates missing"));
    }
    v.sort_by(f64::total_cmp);
    let p95 = v[((v.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)];
    hz.iter()
        .map(|hz| {
            if !hz.is_finite() || *hz <= 0. || *hz > 1000. {
                return Err(error("refresh rate must be 0..1000 Hz"));
            }
            let budget = 1e9 / hz;
            let over = v.iter().filter(|v| **v > budget).count();
            Ok(Deadline {
                hz: *hz,
                budget_ns: budget,
                observations: v.len(),
                over_budget: over,
                over_budget_percent: over as f64 / v.len() as f64 * 100.,
                p95_ns: p95,
            })
        })
        .collect()
}
