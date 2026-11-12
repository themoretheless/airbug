//! Hardware counters next to wall time: one case per event this machine can count.
//!
//! `cargo run --release -p airbug-bench --example perf_counters -- --quick`
use airbug_bench::{
    BatchPolicy, Result, Suite,
    perf::{Event, PerfCounter},
};

fn sort() {
    let mut data: Vec<u32> = (0..4096u32)
        .map(|i| i.wrapping_mul(2_654_435_761))
        .collect();
    data.sort_unstable();
    std::hint::black_box(data);
}

fn main() -> Result<()> {
    let mut suite = Suite::new("perf_counters");
    for event in Event::ALL {
        match PerfCounter::open(event) {
            Ok(counter) => {
                let name = format!("sort/4096/{}", event.id());
                suite.bench_measured(&name, counter, BatchPolicy::SmallInput, sort)?;
            }
            // A machine without a PMU still runs the suite; it just has fewer cases.
            Err(why) => eprintln!("skipped: {why}"),
        }
    }
    suite.bench("sort/4096/wall", sort);
    suite.main()
}
