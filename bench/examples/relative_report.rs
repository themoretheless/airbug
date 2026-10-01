//! Synthetic relative distribution fixture; does not measure performance.
use airbug_bench::{bootstrap::Config, relative};
fn main() -> airbug_bench::Result<()> {
    let report = relative::bootstrap(
        &[90., 95., 100., 105., 110.],
        &[95., 100., 105., 110., 115.],
        &Config {
            resamples: 1000,
            confidence_level: 0.95,
            seed: 7,
        },
    )?;
    let chart = relative::charts(
        &[relative::Comparison {
            case: "synthetic".into(),
            metric: "wall".into(),
            unit: "ns".into(),
            scope: "operation".into(),
            resampling_unit: "independent_process_median".into(),
            baseline_units: 5,
            candidate_units: 5,
            report: Some(report),
            throughput: Vec::new(),
            unavailable_reason: None,
        }],
        5.,
    )?;
    println!(
        "{}",
        airbug_bench::report::html(
            "# Synthetic relative distributions\nNot a performance measurement."
        )
        .replace("<!--CHARTS-->", &chart)
        .replace("<!--DETAILS-->", "")
    );
    Ok(())
}
