//! Deterministic chart fixture; this does not measure performance.
use airbug_bench::{
    bootstrap::Config,
    regression::{Presentation, Sample, Series, comparison_figure, figure, fit, format_series},
};

struct SquaredDisplay;
impl airbug_bench::measurement::ValueFormatter for SquaredDisplay {
    fn scale_values(
        &self,
        _: f64,
        values: &[f64],
    ) -> airbug_bench::Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.iter().map(|v| v * v).collect(),
            unit: "ns²".into(),
        })
    }
    fn scale_throughputs(
        &self,
        _: f64,
        _: f64,
        _: &str,
        _: &[f64],
    ) -> airbug_bench::Result<airbug_bench::measurement::FormattedValues> {
        Err("this example formats only regression totals".into())
    }
    fn scale_for_machines(
        &self,
        values: &[f64],
    ) -> airbug_bench::Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.to_vec(),
            unit: "ns".into(),
        })
    }
}

fn main() -> airbug_bench::Result<()> {
    let samples: Vec<_> = [12., 17., 34., 37., 55., 58., 75., 77.]
        .into_iter()
        .enumerate()
        .map(|(index, total)| Sample {
            operations: (index + 1) as f64 * 10.,
            total,
        })
        .collect();
    let estimate = fit(
        &samples,
        &Config {
            resamples: 2000,
            seed: 7,
            ..Default::default()
        },
    )?;
    let mut chart = figure(
        &samples,
        &estimate,
        "Synthetic regression — one process",
        "ns",
    );
    let labels = [
        "baseline process 0",
        "baseline process 1",
        "candidate process 0",
        "candidate process 1",
        "baseline process 2",
        "baseline process 3",
        "candidate process 2",
        "candidate process 3",
    ];
    let clouds: Vec<Vec<Sample>> = [1., 1.05, 1.3, 1.4, 0.95, 1.1, 1.35, 1.45]
        .iter()
        .map(|factor| {
            samples
                .iter()
                .map(|sample| Sample {
                    operations: sample.operations,
                    total: sample.total * factor,
                })
                .collect()
        })
        .collect();
    let fits: Vec<_> = clouds
        .iter()
        .map(|samples| {
            fit(
                samples,
                &Config {
                    resamples: 2000,
                    seed: 7,
                    ..Default::default()
                },
            )
        })
        .collect::<airbug_bench::Result<_>>()?;
    let series: Vec<_> = labels
        .iter()
        .zip(clouds.iter().zip(&fits))
        .map(|(label, (samples, fit))| Series {
            label,
            samples,
            fit,
        })
        .collect();
    chart.push_str(&comparison_figure(
        &series,
        "Synthetic baseline and candidate — eight processes",
        "ns",
    ));
    // Simulate a saved artifact loaded without the original formatter instance.
    let saved = serde_json::to_vec(&format_series(&series, &SquaredDisplay)?)?;
    let restored: Vec<Presentation> = serde_json::from_slice(&saved)?;
    chart.push_str(&restored[0].figure(
        series[0].samples,
        series[0].fit,
        series[0].label,
        "Synthetic nonlinear display — restored from JSON",
    )?);
    println!(
        "{}",
        airbug_bench::report::html(
            "# Synthetic regression fixture\nNot a performance measurement."
        )
        .replace("<!--CHARTS-->", &chart)
        .replace("<!--DETAILS-->", "")
    );
    Ok(())
}
