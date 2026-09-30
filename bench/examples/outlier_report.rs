//! Render a deterministic synthetic analysis report for visual inspection.
use airbug_bench::{bootstrap, outliers};
fn main() -> airbug_bench::Result<()> {
    let values = [
        -6., -5., -3., -2., 0., 1., 1., 1., 2., 2., 2., 2., 3., 3., 3., 3., 4., 6., 7., 9., 10.,
    ];
    let config = bootstrap::Config {
        resamples: 500,
        ..Default::default()
    };
    let captured = bootstrap::estimate_with_distributions(&values, &config)?;
    let report = bootstrap::Report {
        config: config.clone(), method: "synthetic_example_percentile_bootstrap".into(),
        rows: vec![bootstrap::Row {
            metric_contract: None,
            case: "synthetic/outlier-demo".into(), metric: "value".into(), variant: "example".into(), unit: "units".into(),
            resampling_unit: "synthetic_observation".into(), units: values.len(),
            estimates: Some(captured.estimates),
            distributions: Some(captured.distributions), regressions: vec![], outliers: Some(outliers::classify(&values)?),
            note: "Synthetic fixture demonstrating both mild and severe outliers; not a performance measurement.".into(),
        }],
    };
    println!("{}", bootstrap::html(&report));
    Ok(())
}
