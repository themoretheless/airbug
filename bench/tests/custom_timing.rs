use airbug_bench::{Config, Run, Sampling, Suite, workloads::LocalExecutor};
use std::{cell::RefCell, time::Duration};

#[test]
fn custom_batches_preserve_zero_and_wide_durations_without_wall_time_substitution() {
    for asynchronous in [false, true] {
        let calls = RefCell::new(Vec::new());
        let mut suite = Suite::new("custom");
        let measure = |n| {
            let mut calls = calls.borrow_mut();
            calls.push(n);
            if calls.len() == 1 {
                Duration::ZERO
            } else {
                Duration::new(u64::MAX, 999_999_999)
            }
        };
        if asynchronous {
            suite.bench_async_custom(
                "case",
                || LocalExecutor,
                async move |n| {
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
                    measure(n)
                },
            );
        } else {
            suite.bench_custom("case", measure);
        }
        suite.sampling(Sampling {
            iterations: Some(7),
            ..Default::default()
        });
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 7,
        });
        assert_eq!(suite.list("").len(), 1);
        assert!(calls.borrow().is_empty());
        let run = suite.run("").unwrap();
        assert_eq!(*calls.borrow(), [7, 7]);
        assert_eq!(run.observations[0].value.as_deref(), Some("0"));
        let maximum = Duration::new(u64::MAX, 999_999_999).as_nanos().to_string();
        assert_eq!(run.observations[1].value.as_deref(), Some(maximum.as_str()));
        assert!(run.observations.iter().all(|o| o.operations == 7));
        let decoded: Run = serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
        assert_eq!(decoded.observations[1].value, run.observations[1].value);
    }
}
