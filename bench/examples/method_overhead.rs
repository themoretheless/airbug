//! Method experiment: what the harness adds to a measurement.
//!
//! The same small workload is timed twice in one process — by a handwritten batch loop around
//! `Instant`, and by `Suite::bench` — in alternating rounds, so drift hits both alike. The
//! difference of the medians is the harness's own cost per operation.
//!
//! `cargo run --release -p airbug-bench --example method_overhead`
use airbug_bench::{Result, Suite};
use std::{hint::black_box, time::Instant};

const ROUNDS: usize = 5;
const SAMPLES: usize = 30;

fn work() -> u64 {
    let mut state = 0u64;
    for i in 0..256u64 {
        state = black_box(state.wrapping_mul(31).wrapping_add(i));
    }
    state
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Batches of about a millisecond, ns per operation.
fn handwritten(operation: fn() -> u64) -> Vec<f64> {
    let mut iterations = 1u64;
    loop {
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(operation());
        }
        if start.elapsed().as_micros() >= 1000 {
            break;
        }
        iterations *= 2;
    }
    (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..iterations {
                black_box(operation());
            }
            start.elapsed().as_nanos() as f64 / iterations as f64
        })
        .collect()
}

fn with_suite(operation: fn() -> u64) -> Result<Vec<f64>> {
    let mut suite = Suite::new("overhead");
    suite.bench("work", operation);
    let run = suite.run("")?;
    Ok(run
        .observations
        .iter()
        .filter(|o| o.metric == "wall")
        .filter_map(|o| Some(o.value.as_deref()?.parse::<f64>().ok()? / o.operations as f64))
        .collect())
}

fn main() -> Result<()> {
    let (mut by_hand, mut by_suite) = (Vec::new(), Vec::new());
    for _ in 0..ROUNDS {
        by_hand.push(median(handwritten(work)));
        by_suite.push(median(with_suite(work)?));
    }
    let (hand, suite) = (median(by_hand.clone()), median(by_suite.clone()));
    println!("workload: 256 multiply-adds; {ROUNDS} alternating rounds");
    println!("handwritten loop   {hand:8.2} ns/op   rounds {by_hand:.2?}");
    println!("Suite::bench       {suite:8.2} ns/op   rounds {by_suite:.2?}");
    println!(
        "harness cost       {:+8.2} ns/op   ({:+.2}%)",
        suite - hand,
        (suite - hand) / hand * 100.0
    );
    Ok(())
}
