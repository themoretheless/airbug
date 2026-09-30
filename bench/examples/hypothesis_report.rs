//! Deterministic synthetic fixture for distribution-chart visual verification.
fn main() -> airbug_bench::Result<()> {
    let baseline: Vec<_> = (1..=20).map(f64::from).collect();
    let candidate: Vec<_> = (3..=22).map(f64::from).collect();
    let distribution = airbug_bench::hypothesis::welch_distribution(
        &baseline,
        &candidate,
        airbug_bench::hypothesis::Config {
            resamples: 10_000,
            seed: 7,
        },
    )?;
    println!(
        "<!doctype html><meta charset=\"utf-8\"><title>Welch distribution example</title><body style=\"margin:32px;font:16px system-ui;background:#f3f5f8\"><h1>Synthetic comparison</h1><p>Deterministic fixture; not a performance measurement.</p>{}</body>",
        airbug_bench::hypothesis_plot::svg(&distribution, "Synthetic Welch distribution")?
    );
    Ok(())
}
