//! A render loop reporting its frame boundaries, without a GPU: the "device" is simulated,
//! the boundaries and the timestamp adapter are the real ones.
//!
//! `cargo airbug-bench run --protocol --program target/release/examples/frames -o .airbug-bench/frames`
use airbug_bench::{
    Result,
    frames::{Frames, GpuSpan},
};
use std::collections::BTreeMap;

fn main() -> Result<()> {
    let contract = BTreeMap::from([("renderer".to_string(), "simulated; no GPU".to_string())]);
    // 20 warm-up frames, as Forma's perf_bench does, then 60 measured ones.
    let mut frames = Frames::new("scene/offscreen", contract, 20, true)?;
    let mut ticks = 0u64;
    let mut pending: Option<GpuSpan> = None;
    for _ in 0..80 {
        let frame = frames.begin();
        std::hint::black_box((0..20_000u64).sum::<u64>()); // build and encode commands
        frames.submitted(frame)?;
        std::hint::black_box((0..5_000u64).sum::<u64>()); // wait for the device
        frames.completed(frame)?;
        // Timestamp queries resolve a frame later, as a mapped readback buffer does.
        if let Some(span) = pending.take() {
            frames.gpu(span)?;
        }
        pending = Some(GpuSpan {
            frame,
            begin_ticks: ticks,
            end_ticks: ticks + 4_000,
            period_ns: 1.0,
        });
        ticks += 10_000;
    }
    // The last frame's result never arrives: it is reported as incomplete, not as zero.
    let run = frames.finish()?;
    println!("BENCH_RESULT={}", serde_json::to_string(&run)?);
    Ok(())
}
