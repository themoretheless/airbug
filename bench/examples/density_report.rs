//! Synthetic density fixture; does not measure performance.
use airbug_bench::density::{Series, figure_with_outliers};
fn main() -> airbug_bench::Result<()> {
    let chart = figure_with_outliers(
        &[
            Series {
                label: "baseline",
                values: &[10., 11., 12., 13., 14., 15., 16., 35.],
            },
            Series {
                label: "candidate",
                values: &[14., 15., 16., 17., 18., 19., 20., 21.],
            },
            Series {
                label: "constant",
                values: &[25.; 4],
            },
        ],
        "Synthetic densities",
        "ns",
    )?;
    println!(
        "{}",
        airbug_bench::report::html("# Synthetic density fixture\nNot a performance measurement.")
            .replace("<!--CHARTS-->", &chart)
            .replace("<!--DETAILS-->", "")
    );
    Ok(())
}
