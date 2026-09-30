//! Deterministic custom counter example; values are synthetic, not hardware counters.
use airbug_bench::{BatchPolicy, Metric, Result, Suite, measurement::Measurement};
use std::{cell::Cell, rc::Rc};

struct Counter(Rc<Cell<u64>>);
impl Measurement for Counter {
    type Start = u64;
    type Value = u64;
    fn metric(&self) -> Metric {
        let mut metric = Metric::duration("work_units", "synthetic work counter v1", "batch_total");
        metric.unit = "units".into();
        metric
    }
    fn start(&mut self) -> Result<u64> {
        Ok(self.0.get())
    }
    fn end(&mut self, start: u64) -> Result<u64> {
        Ok(self.0.get() - start)
    }
    fn zero(&self) -> u64 {
        0
    }
    fn add(&self, total: u64, value: u64) -> Result<u64> {
        Ok(total + value)
    }
    fn to_f64(&self, value: &u64) -> Result<f64> {
        Ok(*value as f64)
    }
}
struct Groups;
impl airbug_bench::measurement::ValueFormatter for Groups {
    fn scale_values(
        &self,
        _: f64,
        values: &[f64],
    ) -> Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.iter().map(|v| v / 3.0).collect(),
            unit: "groups".into(),
        })
    }
    fn scale_throughputs(
        &self,
        _: f64,
        work: f64,
        unit: &str,
        values: &[f64],
    ) -> Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.iter().map(|v| work / v).collect(),
            unit: format!("{unit}/unit"),
        })
    }
    fn scale_for_machines(
        &self,
        values: &[f64],
    ) -> Result<airbug_bench::measurement::FormattedValues> {
        Ok(airbug_bench::measurement::FormattedValues {
            values: values.to_vec(),
            unit: "units/op".into(),
        })
    }
}
fn main() -> Result<()> {
    let counter = Rc::new(Cell::new(0));
    let measured = Counter(counter.clone());
    let mut suite = Suite::new("custom_measurement");
    suite.bench_measured("work", measured, BatchPolicy::SmallInput, move || {
        counter.set(counter.get() + 3);
    })?;
    suite.formatter("work_units", Groups)?;
    suite.main()
}
