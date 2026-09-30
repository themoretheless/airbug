//! Synthetic violin fixture; no workloads are measured.
use airbug_bench::{density::Series, violin, viz::charts::AxisScale};
fn main() -> airbug_bench::Result<()> {
    let populations = [
        Series {
            label: "sort / 64",
            values: &[8., 9., 10., 10., 11., 12.],
        },
        Series {
            label: "sort / 1024",
            values: &[40., 50., 55., 60., 80., 90.],
        },
        Series {
            label: "constant",
            values: &[25.; 5],
        },
        Series {
            label: "missing",
            values: &[],
        },
    ];
    let mut charts = violin::figure(
        &populations,
        "Linear violin summary",
        "ns/op",
        AxisScale::Linear,
    )?;
    charts.push_str(&violin::figure(
        &populations,
        "Logarithmic violin summary",
        "ns/op",
        AxisScale::Logarithmic,
    )?);
    println!(
        "{}",
        airbug_bench::report::html("# Synthetic violin summaries\nNot a performance measurement.")
            .replace("<!--CHARTS-->", &charts)
            .replace("<!--DETAILS-->", "")
    );
    Ok(())
}
