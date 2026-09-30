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
        fn format_value(&self, value: f64, unit: &str) -> Result<Option<String>> {
            Ok(Some(format!("value=<{value}> | {unit}")))
        }
        fn format_throughput(&self, value: f64, unit: &str) -> Result<Option<String>> {
            Ok(Some(format!("rate={value} {unit}")))
        }
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
    assert_eq!(human[0].display.as_deref(), Some("value=<1> | k-units|<x>"));
    assert!(human[2].display.is_none());
    assert!(machine.iter().all(|r| r.display.is_none()));
    assert!(airbug_bench::measurement::markdown(&human).contains("value=&lt;1&gt; &#124;"));

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
    assert_eq!(
        throughput[0].display.as_deref(),
        Some("rate=0.004 items/unit")
    );
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
    suite.work_units("items", 4);
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
    assert_eq!(
        formatted[0].throughput["items"].observations[0].value,
        Some(0.002)
    );
    assert_eq!(
        formatted[0].throughput["items"].observations[0].unit,
        "items/unit"
    );
    airbug_bench::measurement::save_formatted(&mut saved, &formatted).unwrap();
    let decoded: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
    assert_eq!(
        airbug_bench::measurement::load_formatted(&decoded).unwrap()[0].human[0]
            .display
            .as_deref(),
        Some("value=<2> | k-units|<x>")
    );
    assert_eq!(
        airbug_bench::measurement::load_formatted(&decoded).unwrap()[0].human[0].value,
        Some(2.0)
    );
    assert!(
        airbug_bench::report::html_run(&decoded)
            .unwrap()
            .contains("Formatted metric:")
    );
    let chart = airbug_bench::measurement::charts(&decoded).unwrap();
    assert!(chart.contains("formatted observations (k-units|&lt;x&gt;)"));
    assert_eq!(chart.matches("<circle").count(), 2);
    assert!(chart.contains("formatted throughput: items (items/unit)"));
    assert!(
        airbug_bench::report::html_run(&decoded)
            .unwrap()
            .contains("formatted observations")
    );
    let mut stale_counter = decoded.clone();
    stale_counter.cases[0]
        .contract
        .insert("work.counter.items".into(), "8".into());
    assert!(
        airbug_bench::measurement::load_formatted(&stale_counter)
            .unwrap_err()
            .to_string()
            .contains("work counter changed")
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

#[test]
fn comparison_formatter_chooses_one_scale_from_both_runs() {
    use airbug_bench::measurement::{Format, FormattedValues, ValueFormatter, format_comparison};
    struct Adaptive;
    impl ValueFormatter for Adaptive {
        fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues> {
            let (divisor, unit) = if typical >= 1000.0 {
                (1000.0, "k-units")
            } else {
                (1.0, "units")
            };
            Ok(FormattedValues {
                values: values.iter().map(|v| v / divisor).collect(),
                unit: unit.into(),
            })
        }
        fn scale_throughputs(&self, _: f64, _: f64, _: &str, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
        fn scale_for_machines(&self, values: &[f64]) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.to_vec(),
                unit: "units".into(),
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
    recorder.observe("case", "counter", 900).unwrap();
    let baseline = recorder.finish().unwrap();
    let mut candidate = baseline.clone();
    candidate.observations[0].operations = 2;
    candidate.observations[0].value = Some("2200".into());
    let before = (
        serde_json::to_vec(&baseline).unwrap(),
        serde_json::to_vec(&candidate).unwrap(),
    );
    let rows = format_comparison(
        &baseline,
        &candidate,
        "case",
        "counter",
        &Adaptive,
        Format::Human,
    )
    .unwrap();
    assert!(rows.iter().all(|r| r.unit == "k-units"));
    assert_eq!(
        rows.iter().find(|r| r.variant == "baseline").unwrap().value,
        Some(0.9)
    );
    assert_eq!(
        rows.iter()
            .find(|r| r.variant == "candidate")
            .unwrap()
            .value,
        Some(1.1)
    );
    assert_eq!(
        before,
        (
            serde_json::to_vec(&baseline).unwrap(),
            serde_json::to_vec(&candidate).unwrap()
        )
    );
    let chart = airbug_bench::measurement::comparison_chart(
        &baseline, &candidate, "case", "counter", &Adaptive,
    )
    .unwrap();
    assert!(chart.contains("formatted comparison (k-units)"));
    assert_eq!(chart.matches("<circle").count(), 2);
    candidate.cases[0].metrics[0].unit = "different".into();
    assert!(
        format_comparison(
            &baseline,
            &candidate,
            "case",
            "counter",
            &Adaptive,
            Format::Human
        )
        .is_err()
    );
}

#[test]
fn dynamic_formatter_matches_work_by_identity_and_invalidates_saved_work() {
    use airbug_bench::measurement::{self, Format, FormattedValues, ValueFormatter};
    struct Rates;
    impl ValueFormatter for Rates {
        fn scale_values(&self, _: f64, values: &[f64]) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.to_vec(),
                unit: "units/op".into(),
            })
        }
        fn scale_for_machines(&self, values: &[f64]) -> Result<FormattedValues> {
            self.scale_values(0.0, values)
        }
        fn scale_throughputs(
            &self,
            _: f64,
            work: f64,
            unit: &str,
            values: &[f64],
        ) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.iter().map(|v| work / v).collect(),
                unit: format!("{unit}/unit"),
            })
        }
    }
    let counter = Rc::new(Cell::new(0));
    let work = counter.clone();
    let mut suite = Suite::new("dynamic");
    let counters = airbug_bench::counters::InputCounters::new(&["items"]).unwrap();
    let inputs = counters.clone();
    let mut size = 0u64;
    suite
        .bench_measured_with_input(
            "case",
            Counter {
                value: counter,
                waves: Rc::new(RefCell::new(vec![])),
                invalid: false,
            },
            BatchPolicy::PerIteration,
            move || {
                size += 1;
                inputs.add("items", size);
                size
            },
            move |_| work.set(work.get() + 3),
            airbug_bench::DropPolicy::OutsideTiming,
        )
        .unwrap();
    suite.input_counters(counters);
    suite.formatter("instructions", Rates).unwrap();
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
    let mut run = suite.run("").unwrap();
    let live = suite.formatted_metrics(&run).unwrap();
    let rates = &live[0].throughput["items"].observations;
    assert_eq!(
        rates.iter().map(|r| r.value).collect::<Vec<_>>(),
        [Some(1.0), Some(8.0 / 3.0)]
    );
    let case = run.cases[0].id.clone();
    run.cases[0]
        .contract
        .insert("work.input.items".into(), "batch_total".into());
    let first_sequence = run.observations[0].sequence;
    for o in &mut run.observations {
        let first = o.sequence == first_sequence;
        o.operations = if first { 2 } else { 5 };
        if o.metric == "wall" {
            o.work_totals
                .insert("items".into(), if first { "6" } else { "50" }.into());
        } else {
            o.value = Some(if first { "20" } else { "100" }.into());
        }
    }
    // Reordering wall observations must not zip them to unrelated metric rows.
    run.observations.reverse();
    let raw = serde_json::to_vec(&run).unwrap();
    let formatted = suite.formatted_metrics(&run).unwrap();
    let rates = &formatted[0].throughput["items"];
    assert_eq!(rates.work_per_operation, None);
    for row in &rates.observations {
        assert_eq!(
            row.value,
            Some(if row.sequence == first_sequence {
                0.3
            } else {
                0.5
            })
        );
        assert_eq!(row.unit, "items/unit");
    }
    assert_eq!(raw, serde_json::to_vec(&run).unwrap());
    measurement::save_formatted(&mut run, &formatted).unwrap();
    let restored: airbug_bench::Run =
        serde_json::from_slice(&serde_json::to_vec(&run).unwrap()).unwrap();
    assert_eq!(
        measurement::load_formatted(&restored).unwrap()[0].throughput["items"]
            .observations
            .len(),
        2
    );
    let charts = measurement::charts(&restored).unwrap();
    assert!(charts.contains("formatted throughput: items (items/unit)"));
    assert_eq!(charts.matches("<circle").count(), 4);
    let comparison =
        measurement::comparison_chart(&restored, &restored, &case, "instructions", &Rates).unwrap();
    assert!(comparison.contains("formatted throughput comparison: items (items/unit)"));
    assert_eq!(comparison.matches("<circle").count(), 8);

    let mut changed = run.clone();
    changed
        .observations
        .iter_mut()
        .find(|o| o.metric == "wall")
        .unwrap()
        .work_totals
        .insert("items".into(), "99".into());
    assert!(
        measurement::load_formatted(&changed)
            .unwrap_err()
            .to_string()
            .contains("input work observations changed")
    );
    let mut mismatch = run.clone();
    mismatch
        .observations
        .iter_mut()
        .find(|o| o.metric == "wall")
        .unwrap()
        .operations += 1;
    assert!(
        suite
            .formatted_metrics(&mismatch)
            .unwrap_err()
            .to_string()
            .contains("operation counts differ")
    );
    let mut missing = run.clone();
    let index = missing
        .observations
        .iter()
        .position(|o| o.metric == "wall")
        .unwrap();
    missing.observations.remove(index);
    assert!(
        suite
            .formatted_metrics(&missing)
            .unwrap_err()
            .to_string()
            .contains("matching wall")
    );
    let mut empty = run.clone();
    empty.observations.clear();
    empty.cases[0].contract.remove("work.input.items");
    assert!(
        measurement::format_observations(
            &empty,
            &case,
            "instructions",
            &Rates,
            Format::InputThroughput { unit: "items" }
        )
        .is_err()
    );
}

#[test]
fn dynamic_formatter_requires_shared_units_or_population_override() {
    use airbug_bench::measurement::{FormattedValues, ValueFormatter};
    struct Adaptive(bool);
    impl ValueFormatter for Adaptive {
        fn scale_values(&self, _: f64, values: &[f64]) -> Result<FormattedValues> {
            self.scale_for_machines(values)
        }
        fn scale_for_machines(&self, values: &[f64]) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.to_vec(),
                unit: "units".into(),
            })
        }
        fn scale_throughputs(
            &self,
            _: f64,
            work: f64,
            _: &str,
            values: &[f64],
        ) -> Result<FormattedValues> {
            let scale = if work >= 1000.0 { 1000.0 } else { 1.0 };
            Ok(FormattedValues {
                values: values.iter().map(|v| work / v / scale).collect(),
                unit: if scale == 1.0 {
                    "items/unit"
                } else {
                    "k-items/unit"
                }
                .into(),
            })
        }
        fn scale_input_throughputs(
            &self,
            typical: f64,
            work: &[f64],
            unit: &str,
            values: &[f64],
        ) -> Result<FormattedValues> {
            if !self.0 {
                // Exercise the default implementation through a delegate without an override.
                struct DefaultRates;
                impl ValueFormatter for DefaultRates {
                    fn scale_values(&self, t: f64, v: &[f64]) -> Result<FormattedValues> {
                        Adaptive(false).scale_values(t, v)
                    }
                    fn scale_for_machines(&self, v: &[f64]) -> Result<FormattedValues> {
                        Adaptive(false).scale_for_machines(v)
                    }
                    fn scale_throughputs(
                        &self,
                        t: f64,
                        w: f64,
                        u: &str,
                        v: &[f64],
                    ) -> Result<FormattedValues> {
                        Adaptive(false).scale_throughputs(t, w, u, v)
                    }
                }
                return DefaultRates.scale_input_throughputs(typical, work, unit, values);
            }
            Ok(FormattedValues {
                values: work
                    .iter()
                    .zip(values)
                    .map(|(w, v)| w / v / 1000.0)
                    .collect(),
                unit: "k-items/unit".into(),
            })
        }
    }
    let work = [10.0, 2000.0];
    let values = [2.0, 4.0];
    assert!(
        Adaptive(false)
            .scale_input_throughputs(4.0, &work, "items", &values)
            .unwrap_err()
            .to_string()
            .contains("shared unit")
    );
    let shared = Adaptive(true)
        .scale_input_throughputs(4.0, &work, "items", &values)
        .unwrap();
    assert_eq!(shared.values, [0.005, 0.5]);
    assert_eq!(shared.unit, "k-items/unit");
}

#[test]
fn process_formatting_preserves_units_and_requires_every_worker() {
    use airbug_bench::measurement::{
        self, FormattedMetric, FormattedObservation, ProcessFormatting,
    };
    let metric = Metric::duration("wall", "fixture", "batch_total");
    let mut recorder = airbug_bench::Recorder::new();
    recorder
        .case(airbug_bench::Case {
            id: "case".into(),
            contract: Default::default(),
            metrics: vec![metric.clone()],
        })
        .unwrap();
    recorder.observe("case", "wall", 1000).unwrap();
    let mut worker = recorder.finish().unwrap();
    let row = FormattedObservation {
        variant: "candidate".into(),
        process: 0,
        sequence: worker.observations[0].sequence,
        value: Some(1000.0),
        unit: "ns".into(),
        unavailable_reason: None,
        display: None,
    };
    let mut snapshot = FormattedMetric {
        case: "case".into(),
        metric,
        human: vec![row.clone()],
        machine: vec![row],
        throughput: Default::default(),
    };
    measurement::save_formatted(&mut worker, &[snapshot.clone()]).unwrap();
    let mut collector = ProcessFormatting::default();
    collector.observe(&worker, 0, "baseline").unwrap();
    snapshot.human[0].value = Some(1.0);
    snapshot.human[0].unit = "us".into();
    let mut second = worker.clone();
    measurement::save_formatted(&mut second, &[snapshot]).unwrap();
    collector.observe(&second, 1, "candidate").unwrap();
    let mut combined = worker.clone();
    combined.observations[0].variant = "baseline".into();
    let mut observation = second.observations[0].clone();
    observation.process = 1;
    combined.observations.push(observation);
    collector.save(&mut combined).unwrap();
    let saved = measurement::load_formatted(&combined).unwrap();
    assert_eq!(
        saved[0]
            .human
            .iter()
            .map(|r| (r.variant.as_str(), r.process, r.value, r.unit.as_str()))
            .collect::<Vec<_>>(),
        [
            ("baseline", 0, Some(1000.0), "ns"),
            ("candidate", 1, Some(1.0), "us")
        ]
    );
    let charts = measurement::charts(&combined).unwrap();
    assert!(charts.contains("formatted observations (ns)"));
    assert!(charts.contains("formatted observations (us)"));
    let mut absent = second.clone();
    absent.provenance.clear();
    for missing_first in [false, true] {
        let mut partial = ProcessFormatting::default();
        partial
            .observe(if missing_first { &absent } else { &worker }, 0, "baseline")
            .unwrap();
        partial
            .observe(
                if missing_first { &worker } else { &absent },
                1,
                "candidate",
            )
            .unwrap();
        partial.save(&mut combined).unwrap();
        assert!(measurement::load_formatted(&combined).unwrap().is_empty());
    }
    let mut stale = worker.clone();
    stale.observations[0].value = Some("2000".into());
    assert!(
        ProcessFormatting::default()
            .observe(&stale, 0, "candidate")
            .is_err()
    );
}

struct ManualValue;
impl Measurement for ManualValue {
    type Start = ();
    type Value = Vec<u64>;
    fn metric(&self) -> Metric {
        let mut metric = Metric::duration("work", "caller custom total", "batch_total");
        metric.unit = "units".into();
        metric
    }
    fn start(&mut self) -> Result<()> {
        panic!("caller owns start")
    }
    fn end(&mut self, _: ()) -> Result<Vec<u64>> {
        panic!("caller owns end")
    }
    fn zero(&self) -> Vec<u64> {
        panic!("caller owns accumulation")
    }
    fn add(&self, _: Vec<u64>, _: Vec<u64>) -> Result<Vec<u64>> {
        panic!("caller owns accumulation")
    }
    fn to_f64(&self, value: &Vec<u64>) -> Result<f64> {
        Ok(value.iter().sum::<u64>() as f64)
    }
}
#[test]
fn caller_defined_typed_totals_support_sync_async_and_keep_wall_separate() {
    for asynchronous in [false, true] {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let recorded = calls.clone();
        let executors = Rc::new(Cell::new(0));
        let created = executors.clone();
        let mut suite = Suite::new("manual");
        if asynchronous {
            suite
                .bench_async_measured_custom(
                    "value",
                    ManualValue,
                    move || {
                        created.set(created.get() + 1);
                        airbug_bench::workloads::LocalExecutor
                    },
                    async move |n| {
                        let mut first = true;
                        std::future::poll_fn(|cx| {
                            if std::mem::take(&mut first) {
                                cx.waker().wake_by_ref();
                                std::task::Poll::Pending
                            } else {
                                std::task::Poll::Ready(())
                            }
                        })
                        .await;
                        recorded.borrow_mut().push(n);
                        vec![n * 2, n * 3]
                    },
                )
                .unwrap();
        } else {
            suite
                .bench_measured_custom("value", ManualValue, move |n| {
                    recorded.borrow_mut().push(n);
                    vec![n * 2, n * 3]
                })
                .unwrap();
        }
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
        assert!(calls.borrow().is_empty());
        assert_eq!(executors.get(), 0);
        let run = suite.run("").unwrap();
        run.validate().unwrap();
        assert_eq!(*calls.borrow(), [130, 130]);
        assert_eq!(executors.get(), usize::from(asynchronous));
        assert_eq!(run.cases[0].contract["measurement.batch_policy"], "caller");
        for row in run.observations.iter().filter(|o| o.metric == "work") {
            assert_eq!(row.operations, 130);
            assert_eq!(row.value.as_deref(), Some("650"));
        }
        assert_eq!(
            run.observations
                .iter()
                .filter(|o| o.metric == "wall")
                .count(),
            2
        );
    }
}

#[cfg(feature = "macros")]
#[airbug_bench::bench(custom = true, measurement = ManualValue, args = [2u64, 7], iterations = 3, samples = 1, warmup_ms = 0)]
fn attributed_manual_total(n: u64, value: u64) -> Vec<u64> {
    vec![n * value]
}
#[cfg(feature = "macros")]
#[airbug_bench::bench(custom = true, measurement = ManualValue, iterations = 3, samples = 1, warmup_ms = 0)]
async fn attributed_async_manual_total(n: u64) -> Vec<u64> {
    vec![n * 9]
}
#[cfg(feature = "macros")]
#[test]
fn attributes_register_caller_defined_custom_values() {
    let mut suite = Suite::new("manual");
    register_attributed_manual_total(&mut suite);
    register_attributed_async_manual_total(&mut suite);
    let run = suite.run("").unwrap();
    run.validate().unwrap();
    assert_eq!(
        run.observations
            .iter()
            .filter(|o| o.metric == "work")
            .map(|o| o.value.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["6", "21", "27"]
    );
}

struct ConvertedTotal;
impl Measurement for ConvertedTotal {
    type Start = ();
    type Value = Result<f64>;
    fn metric(&self) -> Metric {
        ManualValue.metric()
    }
    fn start(&mut self) -> Result<()> {
        unreachable!()
    }
    fn end(&mut self, _: ()) -> Result<Self::Value> {
        unreachable!()
    }
    fn zero(&self) -> Self::Value {
        unreachable!()
    }
    fn add(&self, _: Self::Value, _: Self::Value) -> Result<Self::Value> {
        unreachable!()
    }
    fn to_f64(&self, value: &Self::Value) -> Result<f64> {
        value
            .as_ref()
            .copied()
            .map_err(|e| airbug_bench::error(e.to_string()))
    }
}
#[test]
fn manual_conversion_failure_never_reuses_a_previous_sample() {
    for asynchronous in [false, true] {
        let values = Rc::new(RefCell::new(std::collections::VecDeque::from([
            Ok(6.0),
            Ok(f64::NAN),
            Ok(f64::INFINITY),
            Ok(-1.0),
            Err(airbug_bench::error("counter unavailable")),
            Ok(15.0),
        ])));
        let pending = values.clone();
        let mut suite = Suite::new("conversion");
        if asynchronous {
            suite
                .bench_async_measured_custom(
                    "value",
                    ConvertedTotal,
                    || airbug_bench::workloads::LocalExecutor,
                    async move |_| pending.borrow_mut().pop_front().unwrap(),
                )
                .unwrap();
        } else {
            suite
                .bench_measured_custom("value", ConvertedTotal, move |_| {
                    pending.borrow_mut().pop_front().unwrap()
                })
                .unwrap();
        }
        suite.sampling(Sampling {
            iterations: Some(3),
            ..Default::default()
        });
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 3,
        });
        let first = suite.run("").unwrap();
        assert_eq!(
            first
                .observations
                .iter()
                .find(|o| o.metric == "work")
                .unwrap()
                .value
                .as_deref(),
            Some("6")
        );
        for expected in [
            "finite nonnegative",
            "finite nonnegative",
            "finite nonnegative",
            "counter unavailable",
        ] {
            assert!(suite.run("").unwrap_err().to_string().contains(expected));
        }
        let recovered = suite.run("").unwrap();
        recovered.validate().unwrap();
        let rows: Vec<_> = recovered
            .observations
            .iter()
            .filter(|o| o.metric == "work")
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].value.as_deref(), Some("15"));
        assert_eq!(rows[0].operations, 3);
        assert!(values.borrow().is_empty());
    }
}

#[test]
fn machine_csv_preserves_missing_rows_text_and_integer_identity() {
    use airbug_bench::measurement::{FormattedMetric, FormattedObservation, report_csv};
    let mut metric = ManualValue.metric();
    metric.id = "counter,\"ticks\"".into();
    let row = FormattedObservation {
        variant: "вариант\nB".into(),
        process: u32::MAX,
        sequence: u64::MAX,
        value: Some(1.25),
        unit: "k\"units".into(),
        unavailable_reason: None,
        display: Some("human text must not enter machine CSV".into()),
    };
    let missing = FormattedObservation {
        value: None,
        sequence: 7,
        unavailable_reason: Some("missing,\n\"counter\"".into()),
        ..row.clone()
    };
    let formatted = FormattedMetric {
        case: "case,one".into(),
        metric,
        human: vec![],
        machine: vec![row, missing],
        throughput: Default::default(),
    };
    let csv = report_csv(&[formatted]);
    assert!(csv.starts_with("case,metric,variant,process,sequence,value,unit,normalized_per_operation,unavailable_reason\n"));
    assert!(csv.contains("\"case,one\",\"counter,\"\"ticks\"\"\",\"вариант\nB\",4294967295,18446744073709551615,1.25,\"k\"\"units\",true,\"\"\n"));
    assert!(csv.contains(",7,,\"k\"\"units\",true,\"missing,\n\"\"counter\"\"\"\n"));
    assert!(!csv.contains("human text"));
    assert_eq!(report_csv(&[]).lines().count(), 1);
}

#[test]
fn bootstrap_formatter_scales_estimates_intervals_and_draws_without_mutating_raw_report() {
    use airbug_bench::measurement::{FormattedValues, ValueFormatter, format_bootstrap_metric};
    struct Scale;
    impl ValueFormatter for Scale {
        fn scale_values(&self, typical: f64, values: &[f64]) -> Result<FormattedValues> {
            assert_eq!(typical, 5000.0);
            Ok(FormattedValues {
                values: values.iter().map(|v| v / 1000.0).collect(),
                unit: "k<units>".into(),
            })
        }
        fn scale_throughputs(&self, _: f64, _: f64, _: &str, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
        fn scale_for_machines(&self, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
    }
    let mut suite = Suite::new("bootstrap");
    let mut sample = 0;
    suite
        .bench_measured_custom("value", ManualValue, move |n| {
            sample += 1;
            vec![n * sample * 1000]
        })
        .unwrap();
    suite.sampling(Sampling {
        iterations: Some(3),
        ..Default::default()
    });
    suite.config(Config {
        samples: 4,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 3,
    });
    let run = suite.run("").unwrap();
    let mut report = airbug_bench::bootstrap::analyze_with_distributions(
        &run,
        &airbug_bench::bootstrap::Config {
            resamples: 64,
            ..Default::default()
        },
    )
    .unwrap();
    let samples: Vec<_> = [1.0, 2.0, 4.0, 8.0]
        .into_iter()
        .map(|operations| airbug_bench::regression::Sample {
            operations,
            total: operations * 2500.0,
        })
        .collect();
    let (fit, draws) = airbug_bench::regression::fit_with_distribution(
        &samples,
        &airbug_bench::bootstrap::Config {
            resamples: 64,
            ..Default::default()
        },
    )
    .unwrap();
    report
        .rows
        .iter_mut()
        .find(|row| row.metric == "work")
        .unwrap()
        .regressions
        .push(airbug_bench::bootstrap::ProcessRegression {
            process: 7,
            fit,
            slope_distribution: Some(draws),
            samples,
        });
    let mut second = report
        .rows
        .iter()
        .find(|r| r.metric == "work")
        .unwrap()
        .clone();
    second.variant = "other".into();
    second.estimates.as_mut().unwrap().mean.point = 5000.0;
    report.rows.push(second);
    let original = serde_json::to_vec(&report).unwrap();
    let formatted =
        format_bootstrap_metric(&report, "bootstrap/value", &ManualValue.metric(), &Scale).unwrap();
    assert_eq!(formatted.rows.len(), 2);
    for row in &formatted.rows {
        assert_eq!(row.unit, "k<units>");
        assert_eq!(row.regressions.len(), 1);
        let raw = report
            .rows
            .iter()
            .find(|r| r.metric == "work" && r.variant == row.variant)
            .unwrap();
        for (actual, expected) in row.regressions.iter().zip(&raw.regressions) {
            assert_eq!(actual.process, expected.process);
            assert_eq!(actual.fit.r_squared, expected.fit.r_squared);
            assert_eq!(actual.fit.samples, expected.fit.samples);
            let (a, e) = (&actual.fit.slope, &expected.fit.slope);
            assert_eq!(
                [a.point, a.lower, a.upper, a.standard_error],
                [
                    e.point / 1000.0,
                    e.lower / 1000.0,
                    e.upper / 1000.0,
                    e.standard_error / 1000.0
                ]
            );
            assert_eq!(
                actual.slope_distribution.as_ref().unwrap(),
                &expected
                    .slope_distribution
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|v| v / 1000.0)
                    .collect::<Vec<_>>()
            );
            for (a, e) in actual.samples.iter().zip(&expected.samples) {
                assert_eq!(a.operations, e.operations);
                assert_eq!(a.total, e.total / 1000.0);
            }
        }
        let outliers = row.outliers.as_ref().unwrap();
        let raw_outliers = raw.outliers.as_ref().unwrap();
        assert_eq!(outliers.counts, raw_outliers.counts);
        assert_eq!(outliers.q1, raw_outliers.q1 / 1000.0);
        assert_eq!(outliers.q3, raw_outliers.q3 / 1000.0);
        assert_eq!(
            outliers.fences,
            raw_outliers.fences.map(|v| v.map(|v| v / 1000.0))
        );
        assert!(outliers.fences.iter().flatten().any(|v| *v < 0.0));
        for (point, raw_point) in outliers.points.iter().zip(&raw_outliers.points) {
            assert_eq!(
                (point.index, point.label),
                (raw_point.index, raw_point.label)
            );
            assert_eq!(point.value, raw_point.value / 1000.0);
        }
        let (actual, expected) = (
            row.estimates.as_ref().unwrap(),
            raw.estimates.as_ref().unwrap(),
        );
        for (a, e) in [
            (&actual.mean, &expected.mean),
            (&actual.median, &expected.median),
            (&actual.standard_deviation, &expected.standard_deviation),
            (
                &actual.median_absolute_deviation,
                &expected.median_absolute_deviation,
            ),
        ] {
            assert_eq!(
                [a.point, a.lower, a.upper, a.standard_error],
                [
                    e.point / 1000.0,
                    e.lower / 1000.0,
                    e.upper / 1000.0,
                    e.standard_error / 1000.0
                ]
            );
        }
        assert_eq!(
            row.distributions.as_ref().unwrap().mean,
            raw.distributions
                .as_ref()
                .unwrap()
                .mean
                .iter()
                .map(|v| v / 1000.0)
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(original, serde_json::to_vec(&report).unwrap());
    assert!(airbug_bench::bootstrap::html(&formatted).contains("k&lt;units&gt;"));
    struct Invalid(fn(f64) -> f64);
    impl ValueFormatter for Invalid {
        fn scale_values(&self, _: f64, values: &[f64]) -> Result<FormattedValues> {
            Ok(FormattedValues {
                values: values.iter().map(|v| (self.0)(*v)).collect(),
                unit: "invalid".into(),
            })
        }
        fn scale_throughputs(&self, _: f64, _: f64, _: &str, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
        fn scale_for_machines(&self, _: &[f64]) -> Result<FormattedValues> {
            unreachable!()
        }
    }
    for transform in [
        (|v| v + 100_000.0) as fn(f64) -> f64,
        |v| v * v,
        |_| 0.0,
        |_| f64::INFINITY,
    ] {
        assert!(
            format_bootstrap_metric(
                &report,
                "bootstrap/value",
                &ManualValue.metric(),
                &Invalid(transform)
            )
            .is_err()
        );
    }
    let mut wrong = ManualValue.metric();
    wrong.scope = "different".into();
    assert!(format_bootstrap_metric(&report, "bootstrap/value", &wrong, &Scale).is_err());
}
