//! Deterministic chart fixture; this does not measure performance.
use airbug_bench::{
    bootstrap::Config,
    regression::{Sample, Series, comparison_figure, figure, fit},
};

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
