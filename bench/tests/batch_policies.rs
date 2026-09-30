use airbug_bench::{BatchPolicy, Config, DropPolicy, Sampling, Suite, workloads::LocalExecutor};
use std::{cell::Cell, rc::Rc, time::Duration};

#[derive(Default)]
struct Counts {
    alive: Cell<usize>,
    peak: Cell<usize>,
    made: Cell<usize>,
    dropped: Cell<usize>,
}
struct Value(Rc<Counts>);
impl Value {
    fn new(counts: &Rc<Counts>) -> Self {
        counts.alive.set(counts.alive.get() + 1);
        counts.peak.set(counts.peak.get().max(counts.alive.get()));
        counts.made.set(counts.made.get() + 1);
        Self(counts.clone())
    }
}
impl Drop for Value {
    fn drop(&mut self) {
        self.0.alive.set(self.0.alive.get() - 1);
        self.0.dropped.set(self.0.dropped.get() + 1);
    }
}
#[test]
fn every_policy_bounds_live_inputs_and_outputs_in_sync_and_async_batches() {
    for (policy, peak) in [
        (BatchPolicy::SmallInput, 101),
        (BatchPolicy::LargeInput, 2),
        (BatchPolicy::PerIteration, 1),
        (BatchPolicy::Batches(1.try_into().unwrap()), 1001),
        (BatchPolicy::Batches(3.try_into().unwrap()), 334),
        (BatchPolicy::Iterations(77.try_into().unwrap()), 77),
    ] {
        for mode in 0..4 {
            let inputs = Rc::new(Counts::default());
            let outputs = Rc::new(Counts::default());
            let setup = || Value::new(&inputs);
            let mut suite = Suite::new("batch-policy");
            match mode {
                0 => {
                    suite.bench_batched_ref(
                        "borrowed",
                        setup,
                        |_| Value::new(&outputs),
                        DropPolicy::OutsideTiming,
                        policy,
                    );
                }
                1 => {
                    suite.bench_batched(
                        "owned",
                        setup,
                        |_| Value::new(&outputs),
                        DropPolicy::OutsideTiming,
                        policy,
                    );
                }
                2 => {
                    suite.bench_async_batched_ref(
                        "async-borrowed",
                        || LocalExecutor,
                        setup,
                        async |_| Value::new(&outputs),
                        DropPolicy::OutsideTiming,
                        policy,
                    );
                }
                _ => {
                    suite.bench_async_batched(
                        "async-owned",
                        || LocalExecutor,
                        setup,
                        async |_| Value::new(&outputs),
                        DropPolicy::OutsideTiming,
                        policy,
                    );
                }
            }
            suite.config(Config {
                samples: 2,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 1001,
            });
            suite.sampling(Sampling {
                iterations: Some(1001),
                ..Default::default()
            });
            assert_eq!(suite.list("").len(), 1);
            assert_eq!(inputs.made.get(), 0);
            let run = suite.run("").unwrap();
            assert_eq!(run.observations.len(), 2);
            assert!(run.observations.iter().all(|o| o.operations == 1001));
            for counts in [&inputs, &outputs] {
                assert_eq!(counts.peak.get(), peak, "mode {mode}, policy {policy:?}");
                assert_eq!(counts.alive.get(), 0);
                assert_eq!(counts.made.get(), 2002);
                assert_eq!(counts.dropped.get(), 2002);
            }
        }
    }
}
