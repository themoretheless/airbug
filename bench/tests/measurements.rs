use airbug_bench::{
    BatchPolicy, Config, Metric, Result, Sampling, Suite, measurement::Measurement,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

struct Counter {
    value: Rc<Cell<u64>>,
    waves: Rc<RefCell<Vec<u64>>>,
    invalid: bool,
}
impl Measurement for Counter {
    type Start = u64;
    type Value = Vec<u64>;
    fn metric(&self) -> Metric {
        let mut metric = Metric::duration("instructions", "synthetic counter v1", "batch_total");
        metric.unit = "instructions".into();
        metric
    }
    fn start(&mut self) -> Result<u64> {
        Ok(self.value.get())
    }
    fn end(&mut self, start: u64) -> Result<Vec<u64>> {
        let delta = self.value.get() - start;
        self.waves.borrow_mut().push(delta);
        Ok(vec![delta])
    }
    fn zero(&self) -> Vec<u64> {
        vec![]
    }
    fn add(&self, mut total: Vec<u64>, value: Vec<u64>) -> Result<Vec<u64>> {
        total.extend(value);
        Ok(total)
    }
    fn to_f64(&self, value: &Vec<u64>) -> Result<f64> {
        Ok(if self.invalid {
            f64::NAN
        } else {
            value.iter().sum::<u64>() as f64
        })
    }
}

#[test]
fn typed_measurement_accumulates_waves_separately_from_wall_clock() {
    for invalid in [false, true] {
        let value = Rc::new(Cell::new(0));
        let waves = Rc::new(RefCell::new(vec![]));
        let mut suite = Suite::new("custom");
        let work = value.clone();
        suite
            .bench_measured(
                "counter",
                Counter {
                    value,
                    waves: waves.clone(),
                    invalid,
                },
                BatchPolicy::Iterations(64.try_into().unwrap()),
                move || work.set(work.get() + 3),
            )
            .unwrap();
        suite.sampling(Sampling {
            iterations: Some(130),
            ..Default::default()
        });
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 130,
        });
        assert_eq!(suite.list("").len(), 1);
        assert!(waves.borrow().is_empty());
        let result = suite.run("");
        if invalid {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("finite nonnegative")
            );
            continue;
        }
        let run = result.unwrap();
        assert_eq!(*waves.borrow(), [192, 192, 6, 192, 192, 6]);
        let measured: Vec<_> = run
            .observations
            .iter()
            .filter(|o| o.metric == "instructions")
            .collect();
        assert_eq!(measured.len(), 2);
        assert!(
            measured
                .iter()
                .all(|o| o.operations == 130 && o.value.as_deref() == Some("390"))
        );
        assert_eq!(
            run.observations
                .iter()
                .filter(|o| o.metric == "wall")
                .count(),
            2
        );
        let summary = airbug_bench::summary::mean_estimates(&run).unwrap();
        assert_eq!(
            summary
                .iter()
                .find(|r| r.metric == "instructions")
                .unwrap()
                .value,
            Some(3.0)
        );
        let decoded: airbug_bench::Run =
            serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded.cases[0].metrics[1].unit, "instructions");
    }
}

struct DropCount {
    counter: Rc<Cell<u64>>,
    amount: u64,
}
impl Drop for DropCount {
    fn drop(&mut self) {
        self.counter.set(self.counter.get() + self.amount);
    }
}

#[test]
fn measured_fresh_inputs_exclude_setup_and_input_drop_and_control_output_drop() {
    use airbug_bench::DropPolicy;
    for (policy, per_operation) in [
        (DropPolicy::InsideTiming, 10),
        (DropPolicy::OutsideTiming, 3),
    ] {
        let counter = Rc::new(Cell::new(0));
        let waves = Rc::new(RefCell::new(vec![]));
        let setup_counter = counter.clone();
        let work_counter = counter.clone();
        let mut suite = Suite::new("fresh");
        suite
            .bench_measured_with_input(
                "case",
                Counter {
                    value: counter.clone(),
                    waves: waves.clone(),
                    invalid: false,
                },
                BatchPolicy::Iterations(64.try_into().unwrap()),
                move || {
                    setup_counter.set(setup_counter.get() + 10_000);
                    (
                        false,
                        DropCount {
                            counter: setup_counter.clone(),
                            amount: 1000,
                        },
                    )
                },
                move |input| {
                    assert!(!input.0, "input reused between operations");
                    input.0 = true;
                    work_counter.set(work_counter.get() + 3);
                    DropCount {
                        counter: work_counter.clone(),
                        amount: 7,
                    }
                },
                policy,
            )
            .unwrap();
        suite.sampling(Sampling {
            iterations: Some(130),
            ..Default::default()
        });
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 130,
        });
        assert_eq!(suite.list("").len(), 1);
        assert_eq!(counter.get(), 0);
        let run = suite.run("").unwrap();
        assert_eq!(
            *waves.borrow(),
            [
                64 * per_operation,
                64 * per_operation,
                2 * per_operation,
                64 * per_operation,
                64 * per_operation,
                2 * per_operation
            ]
        );
        assert_eq!(counter.get(), 260 * (10_000 + 1000 + 3 + 7));
        for observation in run
            .observations
            .iter()
            .filter(|o| o.metric == "instructions")
        {
            assert_eq!(
                observation.number().unwrap(),
                Some((130 * per_operation) as f64)
            );
        }
    }
}

#[test]
fn owned_measured_inputs_distinguish_consumed_and_returned_destruction() {
    use airbug_bench::DropPolicy;
    for returned in [false, true] {
        for (policy, outside) in [
            (DropPolicy::InsideTiming, false),
            (DropPolicy::OutsideTiming, true),
        ] {
            let counter = Rc::new(Cell::new(0));
            let setup_counter = counter.clone();
            let work_counter = counter.clone();
            let mut suite = Suite::new("owned");
            suite
                .bench_measured_with_owned_input(
                    "case",
                    Counter {
                        value: counter.clone(),
                        waves: Rc::new(RefCell::new(vec![])),
                        invalid: false,
                    },
                    BatchPolicy::Iterations(2.try_into().unwrap()),
                    move || {
                        setup_counter.set(setup_counter.get() + 10_000);
                        DropCount {
                            counter: setup_counter.clone(),
                            amount: 1000,
                        }
                    },
                    move |input| {
                        work_counter.set(work_counter.get() + 3);
                        if returned {
                            Some(input)
                        } else {
                            drop(input);
                            None
                        }
                    },
                    policy,
                )
                .unwrap();
            suite.sampling(Sampling {
                iterations: Some(5),
                ..Default::default()
            });
            suite.config(Config {
                samples: 2,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 5,
            });
            let run = suite.run("").unwrap();
            let per_operation = if returned && outside { 3 } else { 1003 };
            for sample in run
                .observations
                .iter()
                .filter(|o| o.metric == "instructions")
            {
                assert_eq!(sample.number().unwrap(), Some((5 * per_operation) as f64));
            }
            assert_eq!(counter.get(), 10 * 11_003);
            assert!(run.cases[0].contract["input.ownership"].contains("moved"));
        }
    }
}

#[test]
fn async_measurement_excludes_executor_creation_and_includes_future_completion() {
    use airbug_bench::{DropPolicy, workloads::LocalExecutor};
    let counter = Rc::new(Cell::new(0));
    let created = Rc::new(Cell::new(0));
    let runtime_counter = counter.clone();
    let runtime_created = created.clone();
    let setup_counter = counter.clone();
    let mut suite = Suite::new("async");
    suite
        .bench_async_measured_with_input(
            "case",
            Counter {
                value: counter.clone(),
                waves: Rc::new(RefCell::new(vec![])),
                invalid: false,
            },
            BatchPolicy::Iterations(2.try_into().unwrap()),
            move || {
                runtime_counter.set(runtime_counter.get() + 1_000_000);
                runtime_created.set(runtime_created.get() + 1);
                LocalExecutor
            },
            move || setup_counter.clone(),
            async |input: &mut Rc<Cell<u64>>| {
                input.set(input.get() + 2);
                let mut pending = true;
                std::future::poll_fn(|cx| {
                    input.set(input.get() + 3);
                    if std::mem::take(&mut pending) {
                        cx.waker().wake_by_ref();
                        std::task::Poll::Pending
                    } else {
                        std::task::Poll::Ready(())
                    }
                })
                .await;
                DropCount {
                    counter: input.clone(),
                    amount: 7,
                }
            },
            DropPolicy::OutsideTiming,
        )
        .unwrap();
    suite.sampling(Sampling {
        iterations: Some(5),
        ..Default::default()
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 5,
    });
    assert_eq!(suite.list("").len(), 1);
    assert_eq!(created.get(), 0);
    let run = suite.run("").unwrap();
    assert_eq!(created.get(), 1);
    for observation in run
        .observations
        .iter()
        .filter(|o| o.metric == "instructions")
    {
        assert_eq!(observation.number().unwrap(), Some(40.0));
    }
    assert_eq!(counter.get(), 1_000_000 + 10 * (8 + 7));
    assert!(run.cases[0].contract.contains_key("async.executor"));
}

#[test]
fn async_owned_measurements_preserve_destruction_boundaries() {
    use airbug_bench::DropPolicy;
    for returned in [false, true] {
        for (policy, outside) in [
            (DropPolicy::InsideTiming, false),
            (DropPolicy::OutsideTiming, true),
        ] {
            let counter = Rc::new(Cell::new(0));
            let setup_counter = counter.clone();
            let work_counter = counter.clone();
            let mut suite = Suite::new("owned");
            suite
                .bench_async_measured_with_owned_input(
                    "case",
                    Counter {
                        value: counter.clone(),
                        waves: Rc::new(RefCell::new(vec![])),
                        invalid: false,
                    },
                    BatchPolicy::Iterations(2.try_into().unwrap()),
                    || airbug_bench::workloads::LocalExecutor,
                    move || {
                        setup_counter.set(setup_counter.get() + 10_000);
                        DropCount {
                            counter: setup_counter.clone(),
                            amount: 1000,
                        }
                    },
                    async move |input| {
                        work_counter.set(work_counter.get() + 3);
                        if returned {
                            Some(input)
                        } else {
                            drop(input);
                            None
                        }
                    },
                    policy,
                )
                .unwrap();
            suite.sampling(Sampling {
                iterations: Some(5),
                ..Default::default()
            });
            suite.config(Config {
                samples: 2,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 5,
            });
            let run = suite.run("").unwrap();
            let per_operation = if returned && outside { 3 } else { 1003 };
            for sample in run
                .observations
                .iter()
                .filter(|o| o.metric == "instructions")
            {
                assert_eq!(sample.number().unwrap(), Some((5 * per_operation) as f64));
            }
            assert_eq!(counter.get(), 10 * 11_003);
            assert!(run.cases[0].contract["input.ownership"].contains("moved"));
        }
    }
}

#[test]
fn plain_async_measurement_includes_output_drop() {
    let counter = Rc::new(Cell::new(0));
    let work_counter = counter.clone();
    let mut suite = Suite::new("plain_async");
    suite
        .bench_async_measured(
            "case",
            Counter {
                value: counter.clone(),
                waves: Rc::new(RefCell::new(vec![])),
                invalid: false,
            },
            BatchPolicy::PerIteration,
            || airbug_bench::workloads::LocalExecutor,
            async move || {
                work_counter.set(work_counter.get() + 3);
                DropCount {
                    counter: work_counter.clone(),
                    amount: 7,
                }
            },
        )
        .unwrap();
    suite.sampling(Sampling {
        iterations: Some(5),
        ..Default::default()
    });
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 5,
    });
    let run = suite.run("").unwrap();
    assert!(
        run.observations
            .iter()
            .filter(|o| o.metric == "instructions")
            .all(|o| o.number().unwrap() == Some(50.0))
    );
    assert_eq!(counter.get(), 100);
}

#[test]
fn custom_formatters_share_scale_preserve_missing_rows_and_raw_data() {
    use airbug_bench::measurement::{Format, FormattedValues, ValueFormatter, format_observations};
    struct Formatter(bool);
    impl ValueFormatter for Formatter {
        fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues> {
            assert_eq!(typical, 2000.0);
            Ok(FormattedValues {
                values: if self.0 {
                    vec![]
                } else {
                    values.iter().map(|v| v / 1000.0).collect()
                },
                unit: "k-units|<x>".into(),
            })
        }
        fn scale_throughputs(
            &self,
            typical: f64,
            work: f64,
            unit: &str,
            values: &[f64],
        ) -> Result<FormattedValues> {
            assert_eq!(typical, 2000.0);
            Ok(FormattedValues {
                values: values.iter().map(|v| work / v).collect(),
                unit: format!("{unit}/unit"),
            })
        }
        fn scale_for_machines(&self, values: &[f64]) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.to_vec(),
                unit: "units/op".into(),
            })
        }
    }
    let mut recorder = airbug_bench::Recorder::new();
    recorder
        .case(airbug_bench::Case {
            id: "case".into(),
            contract: Default::default(),
            metrics: vec![Metric::duration("counter", "fixture", "batch_total")],
        })
        .unwrap();
    recorder.observe("case", "counter", 2000).unwrap();
    let mut run = recorder.finish().unwrap();
    run.observations[0].operations = 2;
    let mut second = run.observations[0].clone();
    second.variant = "other".into();
    second.operations = 4;
    second.value = Some("8000".into());
    run.observations.push(second.clone());
    second.sequence += 1;
    second.value = None;
    second.availability = airbug_bench::Availability::Unsupported("missing".into());
    run.observations.push(second);
    let original = serde_json::to_vec(&run).unwrap();
    let human =
        format_observations(&run, "case", "counter", &Formatter(false), Format::Human).unwrap();
    assert_eq!(
        human.iter().map(|r| r.value).collect::<Vec<_>>(),
        [Some(1.0), Some(2.0), None]
    );
    let machine =
        format_observations(&run, "case", "counter", &Formatter(false), Format::Machine).unwrap();
    assert_eq!(machine[0].value, Some(1000.0));
    let throughput = format_observations(
        &run,
        "case",
        "counter",
        &Formatter(false),
        Format::Throughput {
            work: 4.0,
            unit: "items",
        },
    )
    .unwrap();
    assert_eq!(throughput[0].value, Some(0.004));
    assert_eq!(throughput[1].value, Some(0.002));
    assert!(
        human[2]
            .unavailable_reason
            .as_ref()
            .unwrap()
            .contains("missing")
    );
    assert!(format_observations(&run, "case", "counter", &Formatter(true), Format::Human).is_err());
    let html = airbug_bench::report::html(&airbug_bench::measurement::markdown(&human));
    assert!(html.contains("&lt;x&gt;"));
    assert!(!html.contains("<x>"));
    assert_eq!(original, serde_json::to_vec(&run).unwrap());
    let counter = Rc::new(Cell::new(0));
    let work = counter.clone();
    let mut suite = Suite::new("formatted");
    assert!(suite.formatter("instructions", Formatter(false)).is_err());
    suite
        .bench_measured(
            "case",
            Counter {
                value: counter,
                waves: Rc::new(RefCell::new(vec![])),
                invalid: false,
            },
            BatchPolicy::PerIteration,
            move || work.set(work.get() + 2000),
        )
        .unwrap();
    suite.formatter("instructions", Formatter(false)).unwrap();
    assert!(suite.formatter("missing", Formatter(false)).is_err());
    suite.sampling(Sampling {
        iterations: Some(1),
        ..Default::default()
    });
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    let mut saved = suite.run("").unwrap();
    assert_eq!(
        suite.formatted_metrics(&saved).unwrap()[0].human[0].value,
        Some(2.0)
    );
    let formatted = suite.formatted_metrics(&saved).unwrap();
    airbug_bench::measurement::save_formatted(&mut saved, &formatted).unwrap();
    let decoded: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
    assert_eq!(
        airbug_bench::measurement::load_formatted(&decoded).unwrap()[0].human[0].value,
        Some(2.0)
    );
    assert!(
        airbug_bench::report::html_run(&decoded)
            .unwrap()
            .contains("Formatted metric:")
    );
    let mut stale = decoded.clone();
    stale
        .observations
        .iter_mut()
        .find(|o| o.metric == "instructions")
        .unwrap()
        .value = Some("9000".into());
    assert!(airbug_bench::measurement::load_formatted(&stale).is_err());
    assert!(
        airbug_bench::report::markdown(&stale)
            .unwrap()
            .contains("saved formatting is stale")
    );
    let mut filtered = decoded;
    filtered.cases[0].metrics.retain(|m| m.id != "instructions");
    filtered.observations.retain(|o| o.metric != "instructions");
    assert!(
        airbug_bench::measurement::load_formatted(&filtered)
            .unwrap()
            .is_empty()
    );
    saved.cases[0]
        .metrics
        .iter_mut()
        .find(|m| m.id == "instructions")
        .unwrap()
        .scope = "different counter".into();
    assert!(
        suite
            .formatted_metrics(&saved)
            .unwrap_err()
            .to_string()
            .contains("contract differs")
    );
}
