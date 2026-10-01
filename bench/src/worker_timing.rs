//! Structural and aggregate checks for ordinary per-worker timing records.
use crate::{Availability, Result, Run, WorkerTiming, error};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate(run: &Run) -> Result<()> {
    let mut ids = BTreeSet::new();
    for worker in &run.worker_timings {
        let case = run
            .cases
            .iter()
            .find(|c| c.id == worker.case)
            .ok_or_else(|| error("worker timing refers to unknown case"))?;
        let count: u64 = case
            .contract
            .get("threads")
            .ok_or_else(|| error("worker timing without thread contract"))?
            .parse()?;
        if worker.worker >= count
            || worker.operations == 0
            || !ids.insert((
                &worker.case,
                &worker.variant,
                worker.process,
                worker.sequence,
                worker.wave,
                worker.worker,
            ))
        {
            return Err(error("invalid or duplicate worker timing identity"));
        }
        let raw = worker.wall_ns.parse::<u128>()?;
        if let Some(adjusted) = &worker.adjusted_wall_ns {
            if adjusted.parse::<u128>()? > raw {
                return Err(error("adjusted worker timing exceeds raw interval"));
            }
        }
        if !run.observations.iter().any(|o| {
            o.case == worker.case
                && o.variant == worker.variant
                && o.process == worker.process
                && o.sequence == worker.sequence
                && o.metric == "wall"
                && o.availability == Availability::Available
        }) {
            return Err(error("worker timing has no enclosing sample"));
        }
    }
    for wall in run.observations.iter().filter(|o| o.metric == "wall") {
        let mut waves: BTreeMap<u64, Vec<&WorkerTiming>> = BTreeMap::new();
        for worker in run.worker_timings.iter().filter(|w| {
            w.case == wall.case
                && w.variant == wall.variant
                && w.process == wall.process
                && w.sequence == wall.sequence
        }) {
            waves.entry(worker.wave).or_default().push(worker);
        }
        let case = run.cases.iter().find(|c| c.id == wall.case).unwrap();
        if waves.is_empty() {
            if case
                .contract
                .get("threads.timing_records")
                .is_some_and(|v| v == "wave-v1")
            {
                return Err(error("missing worker timing records"));
            }
            continue;
        }
        let count: u64 = case.contract["threads"].parse()?;
        let adjusted = run.observations.iter().find(|o| {
            o.metric == "wall.adjusted"
                && o.case == wall.case
                && o.variant == wall.variant
                && o.process == wall.process
                && o.sequence == wall.sequence
        });
        let (mut operations, mut elapsed, mut adjusted_elapsed) = (0u128, 0u128, 0u128);
        for (index, (wave, workers)) in waves.iter().enumerate() {
            if *wave != index as u64
                || workers.len() as u64 != count
                || workers
                    .iter()
                    .any(|w| w.operations != workers[0].operations)
            {
                return Err(error("incomplete or inconsistent worker timing wave"));
            }
            let (mut maximum, mut adjusted_maximum) = (0, 0);
            for worker in workers {
                operations = add(operations, worker.operations as u128)?;
                maximum = maximum.max(worker.wall_ns.parse::<u128>()?);
                if adjusted.is_some() {
                    adjusted_maximum = adjusted_maximum.max(
                        worker
                            .adjusted_wall_ns
                            .as_deref()
                            .ok_or_else(|| error("missing adjusted worker timing"))?
                            .parse::<u128>()?,
                    );
                }
            }
            elapsed = add(elapsed, maximum)?;
            adjusted_elapsed = add(adjusted_elapsed, adjusted_maximum)?;
        }
        if operations != wall.operations as u128
            || wall
                .value
                .as_deref()
                .ok_or_else(|| error("missing aggregate wall time"))?
                .parse::<u128>()?
                != elapsed
        {
            return Err(error("worker timing disagrees with aggregate"));
        }
        if let Some(adjusted) = adjusted {
            if adjusted.operations != wall.operations
                || adjusted
                    .value
                    .as_deref()
                    .ok_or_else(|| error("missing adjusted aggregate"))?
                    .parse::<u128>()?
                    != adjusted_elapsed
            {
                return Err(error("adjusted worker timing disagrees with aggregate"));
            }
        }
    }
    Ok(())
}
fn add(a: u128, b: u128) -> Result<u128> {
    a.checked_add(b)
        .ok_or_else(|| error("worker timing aggregate overflow"))
}

pub(crate) fn slot_samples(run: &Run) -> Result<Vec<crate::WorkerSlotSample>> {
    let mut grouped = BTreeMap::new();
    let ordinary: BTreeSet<_> = run
        .worker_timings
        .iter()
        .map(|w| (&w.case, &w.variant, w.process))
        .collect();
    let fallback: Vec<_> = run
        .worker_allocations
        .iter()
        .filter(|w| !ordinary.contains(&(&w.case, &w.variant, w.process)))
        .map(|w| WorkerTiming {
            case: w.case.clone(),
            variant: w.variant.clone(),
            process: w.process,
            sequence: w.sequence,
            wave: w.wave,
            worker: w.worker,
            operations: w.operations,
            wall_ns: w.wall_ns.clone(),
            adjusted_wall_ns: w.adjusted_wall_ns.clone(),
        })
        .collect();
    for timing in run.worker_timings.iter().chain(&fallback) {
        let key = (
            &timing.case,
            &timing.variant,
            timing.process,
            timing.sequence,
            timing.worker,
        );
        let totals = grouped
            .entry(key)
            .or_insert((0u64, 0u64, 0u128, Some(0u128)));
        totals.0 = totals
            .0
            .checked_add(1)
            .ok_or_else(|| error("worker wave count overflow"))?;
        totals.1 = totals
            .1
            .checked_add(timing.operations)
            .ok_or_else(|| error("worker operation count overflow"))?;
        totals.2 = add(totals.2, timing.wall_ns.parse()?)?;
        totals.3 = match (totals.3, timing.adjusted_wall_ns.as_deref()) {
            (Some(sum), Some(value)) => Some(add(sum, value.parse()?)?),
            _ => None,
        };
    }
    grouped
        .into_iter()
        .map(
            |((case, variant, process, sequence, worker), (waves, operations, wall, adjusted))| {
                let contract = &run.cases.iter().find(|c| &c.id == case).unwrap().contract;
                let mut counts = BTreeMap::new();
                if let (Some(unit), Some(count)) =
                    (contract.get("work.unit"), contract.get("work.count"))
                {
                    counts.insert(unit.clone(), count.parse::<u64>()?);
                }
                for (key, value) in contract {
                    if let Some(unit) = key.strip_prefix("work.counter.") {
                        counts.insert(unit.to_owned(), value.parse::<u64>()?);
                    }
                }
                // Input totals can differ between workers. Never distribute aggregate totals.
                counts.retain(|unit, _| !contract.contains_key(&format!("work.input.{unit}")));
                let mut work_totals: BTreeMap<String, String> = counts
                    .into_iter()
                    .map(|(unit, count)| {
                        (
                            unit,
                            (u128::from(count) * u128::from(operations)).to_string(),
                        )
                    })
                    .collect();
                if let Some(totals) = run
                    .observations
                    .iter()
                    .find(|o| {
                        o.metric == "wall"
                            && &o.case == case
                            && &o.variant == variant
                            && o.process == process
                            && o.sequence == sequence
                    })
                    .and_then(|o| o.worker_work_totals.get(&worker))
                {
                    work_totals.extend(totals.clone());
                }
                Ok(crate::WorkerSlotSample {
                    case: case.clone(),
                    variant: variant.clone(),
                    process,
                    sequence,
                    worker,
                    waves,
                    operations,
                    wall_ns: wall.to_string(),
                    adjusted_wall_ns: adjusted.map(|v| v.to_string()),
                    work_totals,
                })
            },
        )
        .collect()
}
