use airbug_bench::{BatchPolicy, Config, DropPolicy, Sampling, Suite, workloads::LocalExecutor};
use std::{cell::Cell, rc::Rc, time::Duration};

#[test]
fn async_batch_panic_cleans_inputs_and_retained_outputs_after_pending() {
    struct Counted(Rc<Cell<usize>>);
    impl Drop for Counted {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    for owned in [false, true] {
        let inputs = Rc::new(Cell::new(0));
        let outputs = Rc::new(Cell::new(0));
        let calls = Cell::new(0);
        let setup = || Counted(inputs.clone());
        let operation = async || {
            let mut pending = true;
            std::future::poll_fn(|cx| {
                if std::mem::take(&mut pending) {
                    cx.waker().wake_by_ref();
                    std::task::Poll::Pending
                } else {
                    std::task::Poll::Ready(())
                }
            })
            .await;
            calls.set(calls.get() + 1);
            assert_ne!(calls.get(), 3, "intentional future panic");
            Counted(outputs.clone())
        };
        let mut suite = Suite::new("async-cleanup");
        let batch = BatchPolicy::Iterations(5.try_into().unwrap());
        if owned {
            suite.bench_async_batched(
                "owned",
                || LocalExecutor,
                setup,
                async |input| {
                    let output = operation().await;
                    drop(input);
                    output
                },
                DropPolicy::OutsideTiming,
                batch,
            );
        } else {
            suite.bench_async_batched_ref(
                "borrowed",
                || LocalExecutor,
                setup,
                async |_| operation().await,
                DropPolicy::OutsideTiming,
                batch,
            );
        }
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 9,
        });
        suite.sampling(Sampling {
            iterations: Some(9),
            ..Default::default()
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| suite.run("")));
        assert!(result.is_err() || result.unwrap().is_err());
        assert_eq!(calls.get(), 3);
        assert_eq!(inputs.get(), 5);
        assert_eq!(outputs.get(), 2);
    }
}
