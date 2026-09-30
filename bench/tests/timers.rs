use airbug_bench::{Config, DropPolicy, Sampling, Suite, timer::Timer, workloads::LocalExecutor};
use std::time::{Duration, Instant};

#[test]
fn cpu_suite_local_async_and_worker_intervals_use_the_selected_clock() {
    let timer = match Timer::cpu() {
        Ok(timer) => timer,
        Err(error) => {
            eprintln!("CPU timer unavailable on this host: {error}");
            return;
        }
    };
    let mut suite = Suite::new("cpu");
    suite.group_with_sampling(
        "cases",
        Sampling {
            iterations: Some(1),
            ..Default::default()
        },
        |suite| {
            suite.bench("local", || std::thread::sleep(Duration::from_millis(2)));
            suite.bench_async("async", LocalExecutor, || async {
                std::thread::sleep(Duration::from_millis(2))
            });
            suite.bench_threads_with_local_input(
                "workers",
                2,
                || (),
                |_| std::thread::sleep(Duration::from_millis(2)),
                DropPolicy::OutsideTiming,
            );
        },
    );
    suite.config(Config {
        samples: 2,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.timer(timer);
    let start = Instant::now();
    let run = suite.run("").unwrap();
    let wall = start.elapsed().as_nanos();
    run.validate().unwrap();
    assert_eq!(run.provenance["timer.clock"], "cpu");
    assert_eq!(
        run.provenance["timer.frequency_hz"],
        timer.frequency_hz().unwrap().to_string()
    );
    assert_eq!(run.provenance["timer.cpu.calibration.trials"], "256");
    assert_eq!(
        run.provenance["timer.cpu.calibration.loop_iterations"],
        "10000"
    );
    assert_eq!(
        run.provenance["timer.overhead_policy"],
        "raw; no subtraction"
    );
    assert_eq!(run.observations.len(), 6);
    for observation in &run.observations {
        let elapsed = observation.value.as_ref().unwrap().parse::<u128>().unwrap();
        assert!(elapsed >= 1_000_000, "operation's sleep must be included");
        assert!(
            elapsed <= wall * 2,
            "counter frequency must agree with wall time"
        );
    }
    assert!(run.cases.iter().all(|c| c.contract["timer.clock"] == "cpu"));
    suite.timer(Timer::os());
    let os = suite.run("").unwrap();
    assert_eq!(os.provenance["timer.clock"], "os");
    assert!(
        os.provenance
            .contains_key("timer.os.calibration.loop_batch_ns")
    );
    assert!(
        !os.provenance
            .contains_key("timer.cpu.calibration.loop_batch_ns")
    );
    assert!(!os.provenance.contains_key("timer.frequency_hz"));
    assert!(os.cases.iter().all(|c| c.contract["timer.clock"] == "os"));
}

#[cfg(feature = "macros")]
#[airbug_bench::group]
mod imported_clocks {
    #[bench]
    fn inherited() {}
    #[bench(timer = "os")]
    fn explicit_os() {}
}
#[cfg(feature = "macros")]
#[airbug_bench::group(timer = "os")]
mod imported_os {
    #[bench]
    fn nearest_os() {}
}
#[cfg(feature = "macros")]
#[airbug_bench::group(timer = "cpu", groups = [crate::imported_clocks, crate::imported_os])]
mod cpu_default {
    #[bench]
    fn inherited() {}
    #[group(timer = "os")]
    mod nested {
        #[bench]
        fn nearest_os() {}
        #[bench(timer = "cpu")]
        fn explicit_cpu() {}
    }
}

#[cfg(feature = "macros")]
#[test]
fn attributes_inherit_clocks_across_inline_and_imported_groups() {
    let mut suite = Suite::new("clocks");
    cpu_default::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 6);
    // Selecting only OS cases must never require CPU availability.
    let selection = airbug_bench::Selection::default()
        .with_regex("os$")
        .unwrap();
    let os = suite.test_selected(&selection).unwrap();
    assert_eq!(os.cases.len(), 3);
    assert!(os.cases.iter().all(|c| c.contract["timer.clock"] == "os"));
    if let Err(error) = Timer::cpu() {
        eprintln!("CPU timer unavailable: {error}");
        return;
    }
    let run = suite
        .test_selected(&airbug_bench::Selection::default())
        .unwrap();
    assert_eq!(run.provenance["timer.clock"], "mixed");
    for case in run.cases {
        let expected = if case.id.ends_with("os") { "os" } else { "cpu" };
        assert_eq!(case.contract["timer.clock"], expected, "{}", case.id);
    }
}

#[cfg(feature = "macros")]
#[airbug_bench::group]
mod imported_overhead {
    #[bench]
    fn inherited() {}
    #[bench(overhead = "raw")]
    fn explicit_raw() {}
}
#[cfg(feature = "macros")]
#[airbug_bench::group(overhead = "raw")]
mod imported_raw {
    #[bench]
    fn nearest_raw() {}
}
#[cfg(feature = "macros")]
#[airbug_bench::group(overhead = "subtract", groups = [crate::imported_overhead, crate::imported_raw], samples = 1, iterations = 1, warmup_ms = 0)]
mod compensated_default {
    #[bench]
    fn inherited() {}
    #[group(overhead = "raw")]
    mod nested {
        #[bench]
        fn nearest_raw() {}
        #[bench(overhead = "subtract")]
        fn explicit_subtract() {}
    }
}
#[cfg(feature = "macros")]
#[test]
fn overhead_attributes_preserve_explicit_raw_and_nearest_imported_defaults() {
    let mut suite = Suite::new("overhead");
    compensated_default::__airbug_register_group(&mut suite);
    assert_eq!(suite.list("").len(), 6);
    let run = suite.run("").unwrap();
    assert_eq!(
        run.provenance["timer.overhead_policy"],
        "subtract requested; see case policy"
    );
    for case in &run.cases {
        let compensated = !case.id.ends_with("raw");
        assert_eq!(
            case.contract["timer.overhead_policy"],
            if compensated { "subtract-v1" } else { "raw" }
        );
        assert_eq!(
            case.metrics.iter().any(|m| m.id == "wall.adjusted"),
            compensated
        );
    }
    let smoke = suite
        .test_selected(&airbug_bench::Selection::default())
        .unwrap();
    assert!(
        smoke
            .cases
            .iter()
            .all(|c| c.contract["timer.overhead_policy"] == "raw")
    );
    assert!(
        !smoke
            .observations
            .iter()
            .any(|o| o.metric == "wall.adjusted")
    );
}
