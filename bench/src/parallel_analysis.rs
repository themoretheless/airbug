//! Ordered analysis chunks with exactly the sequential resampling RNG stream.
use crate::{Result, Seeded, error};

pub(crate) fn workers(resamples: usize, units: usize) -> usize {
    if !cfg!(feature = "parallel-analysis") || resamples < 2048 || units < 32 {
        return 1;
    }
    std::thread::available_parallelism().map_or(1, |n| n.get().min(8))
}

pub(crate) fn chunks<R: Send>(
    resamples: usize,
    seed: u64,
    dimensions: &[usize],
    workers: usize,
    work: impl Fn(usize, Seeded) -> Result<R> + Sync,
) -> Result<Vec<R>> {
    let workers = workers.max(1).min(resamples.max(1));
    if workers == 1 {
        return work(resamples, Seeded::new(seed)).map(|result| vec![result]);
    }
    let mut plans = Vec::with_capacity(workers);
    let mut rng = Seeded::new(seed);
    for index in 0..workers {
        let count = resamples / workers + usize::from(index < resamples % workers);
        plans.push((count, rng.clone()));
        if index + 1 < workers {
            // Rejection sampling consumes a variable number of RNG words. Walk
            // it exactly; arithmetic skip-ahead would change seeded results.
            for _ in 0..count {
                for &length in dimensions {
                    for _ in 0..length {
                        crate::bootstrap::draw_index(&mut rng, length);
                    }
                }
            }
        }
    }
    ordered(plans, |(count, rng)| work(count, rng))
}

pub(crate) fn density_workers(points: usize, samples: usize) -> usize {
    if !cfg!(feature = "parallel-analysis")
        || points < 64
        || points.saturating_mul(samples) < 262_144
    {
        return 1;
    }
    std::thread::available_parallelism().map_or(1, |n| n.get().min(8).min(points / 32))
}

pub(crate) fn ordered<T: Send, R: Send>(
    jobs: Vec<T>,
    work: impl Fn(T) -> Result<R> + Sync,
) -> Result<Vec<R>> {
    if jobs.len() <= 1 {
        return jobs.into_iter().map(work).collect();
    }
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(jobs.len());
        let mut spawn_error = None;
        for (index, job) in jobs.into_iter().enumerate() {
            let work = &work;
            match std::thread::Builder::new()
                .name(format!("airbug-analysis-{index}"))
                .spawn_scoped(scope, move || work(job))
            {
                Ok(handle) => handles.push(handle),
                Err(failure) => {
                    spawn_error = Some(failure);
                    break;
                }
            }
        }
        // Join every worker even after an error/panic, then return the first
        // error in input order. No analysis thread outlives this call.
        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| error("statistical analysis worker panicked"))?
            })
            .collect();
        if let Some(failure) = spawn_error {
            return Err(failure.into());
        }
        results.into_iter().collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seeded_chunks_preserve_order_and_rejection_sampling() {
        // This seed makes the first SplitMix64 value zero, forcing rejection
        // for a three-element input. Chunk boundaries must account for it.
        for seed in [0, 42, 0u64.wrapping_sub(0x9e3779b97f4a7c15)] {
            let work = |count, mut rng: Seeded| -> Result<Vec<usize>> {
                Ok((0..count * 3)
                    .map(|_| crate::bootstrap::draw_index(&mut rng, 3))
                    .collect())
            };
            let serial = chunks(19, seed, &[3], 1, work).unwrap().concat();
            for workers in [2, 3, 8] {
                assert_eq!(
                    chunks(19, seed, &[3], workers, work).unwrap().concat(),
                    serial
                );
            }
        }
    }
    #[test]
    fn workers_are_joined_after_failure() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let completed = AtomicUsize::new(0);
        let result = chunks(8, 0, &[2], 4, |_, _| -> Result<()> {
            let index = completed.fetch_add(1, Ordering::SeqCst);
            if index == 0 {
                panic!("analysis panic")
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(completed.load(Ordering::SeqCst), 4);
    }
}
