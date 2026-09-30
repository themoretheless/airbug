//! Deterministic report fixture; no workload or performance measurements.
use airbug_bench::{Case, Metric, Recorder, presentation, report, viz::charts::AxisScale};
use std::collections::BTreeMap;
fn main() -> airbug_bench::Result<()> {
    let mut recorder = Recorder::new();
    let mut families = BTreeMap::new();
    for (name, factor) in [("sort A", 2), ("sort B", 3)] {
        for size in [1024, 16, 128] {
            let id = format!("{name}/{size}");
            recorder.case(Case {
                id: id.clone(),
                contract: BTreeMap::from([
                    ("param.size".into(), size.to_string()),
                    ("work.counter.bytes".into(), size.to_string()),
                ]),
                metrics: vec![Metric::duration("wall", "operation", "batch_total")],
            })?;
            recorder.observe(&id, "wall", size * factor)?;
            families.insert(id, name.into());
        }
    }
    let mut run = recorder.finish()?;
    presentation::save_families(&mut run, &families)?;
    println!(
        "{}",
        report::html_run_with_summary(
            &run,
            Some(report::SummaryPlot {
                estimator: Default::default(),
                parameter: "size",
                scale: AxisScale::Logarithmic
            })
        )?
    );
    Ok(())
}
