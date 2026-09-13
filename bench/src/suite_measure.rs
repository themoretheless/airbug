//! Hot-path batch helpers for Suite (kept separate from registration API).
use crate::suite::DropPolicy;
use std::hint::black_box;
use std::time::Instant;

pub(crate) fn input_lifecycle(drop: DropPolicy) -> &'static str {
    match drop {
        DropPolicy::InsideTiming => {
            "fresh input per operation; input setup/drop excluded; output drop included; chunks of <=64"
        }
        DropPolicy::OutsideTiming => {
            "fresh input per operation; input setup/drop and output drop excluded; chunks of <=64"
        }
    }
}

pub(crate) fn input_batch<I, O>(
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(&mut I) -> O,
    drop: DropPolicy,
) -> u128 {
    let mut remaining = n;
    let mut total = 0;
    while remaining > 0 {
        let count = remaining.min(64);
        let mut inputs: Vec<I> = (0..count).map(|_| setup()).collect();
        let mut outputs = Vec::with_capacity(count as usize);
        let start = Instant::now();
        for input in &mut inputs {
            match drop {
                DropPolicy::InsideTiming => {
                    black_box(f(black_box(input)));
                }
                DropPolicy::OutsideTiming => outputs.push(black_box(f(black_box(input)))),
            }
        }
        total += start.elapsed().as_nanos();
        black_box(&outputs);
        remaining -= count;
    }
    total
}
