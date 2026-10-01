use airbug_bench::analysis::{self, Decision, compare, median_interval};
use airbug_bench::*;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
fn paired(n: u32, factor: f64) -> Run {
    let mut r = Run::new();
    r.cases.push(Case {
        id: "case".into(),
        contract: BTreeMap::new(),
        metrics: vec![Metric::duration("latency", "test", "process_total")],
    });
    for p in 0..n {
        for (i, v, f) in [(0, "baseline", 1.0), (1, "candidate", factor)] {
            r.observations.push(Observation {
                worker_work_totals: Default::default(),
                work_totals: Default::default(),
                case: "case".into(),
                metric: "latency".into(),
                variant: v.into(),
                process: 2 * p + i,
                pair: Some(p),
                sequence: 0,
                value: Some(((100.0 + p as f64) * f).to_string()),
                operations: 1,
                availability: Availability::Available,
            });
        }
    }
    r.status = Status::Complete;
    r
}
#[test]
fn exact_interval_small_n_is_inconclusive() {
    assert!(median_interval(&[1.0; 5], 0.05).is_none());
    assert_eq!(median_interval(&[1.0; 6], 0.05), Some((1.0, 1.0)));
}
#[test]
fn paired_effect_and_zero_baseline() {
    let mut r = paired(12, 1.2);
    let rows = compare(&r, None, 5.0, 0.05).unwrap();
    assert_eq!(rows[0].decision, Decision::Regression);
    assert!((rows[0].change_percent.unwrap() - 20.0).abs() < 1e-9);
    for o in r
        .observations
        .iter_mut()
        .filter(|o| o.variant == "baseline")
    {
        o.value = Some("0".into());
    }
    assert_eq!(
        compare(&r, None, 5.0, 0.05).unwrap()[0].decision,
        Decision::Inconclusive
    );
}
#[test]
fn small_pairs_never_use_inner_iterations_as_replications() {
    let mut r = paired(3, 2.0);
    let orig = r.observations.clone();
    for i in 1..100 {
        for o in &orig {
            let mut o = o.clone();
            o.sequence = i;
            r.observations.push(o);
        }
    }
    let rows = compare(&r, None, 5.0, 0.05).unwrap();
    assert_eq!(rows[0].independent_units, 3);
    assert_eq!(rows[0].decision, Decision::Inconclusive);
}
#[test]
fn missing_pair_and_duplicate_are_rejected() {
    let mut r = paired(12, 1.1);
    r.observations.pop();
    assert!(compare(&r, None, 5.0, 0.05).is_err());
    let mut r = paired(12, 1.1);
    r.observations.push(r.observations[0].clone());
    assert!(r.validate().is_err());
}
#[test]
fn incompatible_contract_and_environment_are_rejected() {
    let a = paired(12, 1.0);
    let mut b = a.clone();
    b.environment.insert("cpu".into(), "other".into());
    assert!(compare(&a, Some(&b), 5.0, 0.05).is_err());
    b = a.clone();
    b.cases[0].contract.insert("dpi".into(), "2".into());
    assert!(compare(&a, Some(&b), 5.0, 0.05).is_err());
}
#[test]
fn nonfinite_and_invalid_availability_rejected() {
    let mut r = paired(12, 1.0);
    r.observations[0].value = Some("NaN".into());
    assert!(r.validate().is_err());
    r.observations[0].value = Some("12".into());
    r.observations[0].availability = Availability::Unsupported("x".into());
    assert!(r.validate().is_err());
}
#[test]
fn saves_are_immutable() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("run");
    let r = paired(12, 1.0);
    r.save_new(&p).unwrap();
    assert!(r.save_new(&p).is_err());
    assert_eq!(Run::load(p).unwrap().observations.len(), 24);
}
#[test]
fn registration_and_list_do_not_execute() {
    let n = Arc::new(AtomicUsize::new(0));
    let c = n.clone();
    let mut s = Suite::new("demo");
    s.bench("x", move || {
        c.fetch_add(1, Ordering::SeqCst);
    });
    assert_eq!(s.list("x"), vec!["demo/x"]);
    assert_eq!(n.load(Ordering::SeqCst), 0);
}
#[test]
fn fresh_input_and_drop_exactly_once() {
    struct DropCount(Arc<AtomicUsize>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    let d = drops.clone();
    let mut s = Suite::new("sort");
    s.config(Config {
        samples: 3,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    s.bench_with_input(
        "fresh",
        || vec![3, 1, 2],
        move |v| {
            assert_eq!(v, &[3, 1, 2]);
            v.sort();
            c.fetch_add(1, Ordering::SeqCst);
            DropCount(d.clone())
        },
        DropPolicy::OutsideTiming,
    );
    let r = s.run("").unwrap();
    assert_eq!(r.observations.len(), 3);
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    assert_eq!(drops.load(Ordering::SeqCst), 4);
}
#[test]
fn invalid_config_and_empty_filter() {
    let mut s = Suite::new("x");
    s.bench("one", || 1);
    assert!(s.run("no match").is_err());
    s.config(Config {
        samples: 0,
        ..Config::default()
    });
    let disabled = s.run("").unwrap();
    assert!(disabled.observations.is_empty());
    disabled.validate().unwrap();
    let mut missing_reason = disabled.clone();
    missing_reason.provenance.clear();
    assert!(missing_reason.validate().is_err());
    s.config(Config {
        max_iterations: 0,
        ..Config::default()
    });
    assert!(s.run("").is_err());
}
#[test]
fn recorder_keeps_missing_gpu_reason() {
    let mut s = Recorder::new();
    s.case(Case {
        id: "frame".into(),
        contract: BTreeMap::new(),
        metrics: vec![Metric::duration("gpu", "GPU pass", "pass")],
    })
    .unwrap();
    s.unavailable(
        "frame",
        "gpu",
        Availability::Unsupported("no adapter".into()),
    )
    .unwrap();
    let r = s.finish().unwrap();
    assert_eq!(r.observations[0].value, None);
    assert!(
        airbug_bench::report::markdown(&r)
            .unwrap()
            .contains("no adapter")
    );
}
#[test]
fn report_escapes_untrusted_names() {
    let html = airbug_bench::report::html("<script>alert(1)</script>");
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}
#[test]
fn interval_coverage_seeded_uniform_monte_carlo() {
    let mut seed = 937451u64;
    let mut misses = 0;
    let trials = 2000;
    for _ in 0..trials {
        let values: Vec<_> = (0..24)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                (seed >> 11) as f64 / (1u64 << 53) as f64
            })
            .collect();
        let (l, h) = median_interval(&values, 0.05).unwrap();
        if l > 0.5 || h < 0.5 {
            misses += 1;
        }
    }
    // Predeclared generous Monte Carlo bound; exact order-statistic coverage is conservative.
    assert!(misses < 130, "misses={misses}/{trials}");
    assert!(misses > 10);
}
#[test]
fn allocator_failed_realloc_preserves_live_allocation() {
    use std::alloc::{GlobalAlloc, Layout, System};
    struct FailRealloc;
    unsafe impl GlobalAlloc for FailRealloc {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            unsafe { System.alloc(l) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            unsafe { System.dealloc(p, l) }
        }
        unsafe fn realloc(&self, _: *mut u8, _: Layout, _: usize) -> *mut u8 {
            std::ptr::null_mut()
        }
    }
    let a = airbug_bench::alloc::TrackingAllocator::new(FailRealloc);
    let thread_phase = a.begin_thread_phase().unwrap();
    unsafe {
        let l = Layout::from_size_align(32, 8).unwrap();
        let p = a.alloc(l);
        assert!(!p.is_null());
        assert!(a.realloc(p, l, 64).is_null());
        assert_eq!(a.snapshot().live_bytes, 32);
        assert_eq!(a.snapshot().reallocations, 0);
        assert_eq!(a.snapshot().grow_operations, 0);
        assert_eq!(a.snapshot().grown_bytes, 0);
        assert_eq!(a.snapshot().shrink_operations, 0);
        assert_eq!(a.snapshot().shrunk_bytes, 0);
        let local = thread_phase.finish();
        assert_eq!(local.allocations, 1);
        assert_eq!(local.reallocations, 0);
        assert_eq!(local.grow_operations, 0);
        assert_eq!(local.net_live_bytes, 32);
        a.dealloc(p, l);
        assert_eq!(a.snapshot().live_bytes, 0);
    }
}
#[test]
fn allocator_zeroed_shrink_and_peak() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let a = airbug_bench::alloc::TrackingAllocator::new(System);
    unsafe {
        let l = Layout::from_size_align(64, 8).unwrap();
        let p = a.alloc_zeroed(l);
        assert!(!p.is_null());
        assert_eq!(*p, 0);
        let q = a.realloc(p, l, 16);
        assert!(!q.is_null());
        assert_eq!(a.snapshot().live_bytes, 16);
        assert_eq!(a.snapshot().lifetime_peak_bytes, 64);
        assert_eq!(a.snapshot().requested_bytes, 80);
        a.dealloc(q, Layout::from_size_align(16, 8).unwrap());
        assert_eq!(a.snapshot().live_bytes, 0);
    }
}

#[test]
fn matrix_cartesian_identity_and_lazy_checked_work() {
    let calls = Arc::new(AtomicUsize::new(0));
    let checks = Arc::new(AtomicUsize::new(0));
    let mut s = Suite::new("matrix");
    s.matrix(
        "sort",
        &[("size", &["2", "4"]), ("mode", &["a/b", "a,b"])],
        |s, id, p| {
            let n: usize = p["size"].parse().unwrap();
            let calls = calls.clone();
            let checks = checks.clone();
            s.bench_checked(
                id,
                move || (0..n).rev().collect::<Vec<_>>(),
                move |v| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    v.sort();
                },
                move |v, _| {
                    checks.fetch_add(1, Ordering::SeqCst);
                    assert!(v.windows(2).all(|w| w[0] <= w[1]));
                    Ok(())
                },
                DropPolicy::OutsideTiming,
            );
        },
    )
    .unwrap();
    assert_eq!(s.list("").len(), 4);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(s.list("").iter().any(|id| id.contains("a%2Fb")));
    s.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    let r = s.run("").unwrap();
    assert_eq!(checks.load(Ordering::SeqCst), 8);
    assert!(r
        .cases
        .iter()
        .all(|c| c.contract.contains_key("param.size") && c.contract.contains_key("param.mode")));
}
#[test]
fn validation_failure_stops_before_sampling() {
    let calls = Arc::new(AtomicUsize::new(0));
    let n = calls.clone();
    let mut s = Suite::new("bad");
    s.bench_checked(
        "wrong",
        || (),
        move |_| {
            n.fetch_add(1, Ordering::SeqCst);
        },
        |_, _| Err(error("wrong answer")),
        DropPolicy::InsideTiming,
    );
    assert!(
        s.run("")
            .unwrap_err()
            .to_string()
            .contains("pre-validation")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[test]
fn matrix_rejects_bad_axes_without_registration() {
    let mut s = Suite::new("bad");
    assert!(
        s.matrix("x", &[("n", &["1", "1"])], |_, _, _| panic!(
            "must not register"
        ))
        .is_err()
    );
    assert!(
        s.matrix("x", &[("n", &[])], |_, _, _| panic!("must not register"))
            .is_err()
    );
}
#[test]
fn budgets_missing_units_bounds_and_relative_effect() {
    let r = paired(16, 1.2);
    let config = |extra: &str| {
        serde_json::from_str::<budget::BudgetFile>(&format!(
            r#"{{"budgets":[{{"case":"case","metric":"latency","unit":"ns",{extra}}}]}}"#
        ))
        .unwrap()
    };
    assert_eq!(
        budget::exit_code(&budget::evaluate(&config("\"max\":200"), &r, None).unwrap()),
        0
    );
    assert_eq!(
        budget::exit_code(&budget::evaluate(&config("\"max\":110"), &r, None).unwrap()),
        1
    );
    assert_eq!(
        budget::exit_code(
            &budget::evaluate(&config("\"max_regression_percent\":5"), &r, None).unwrap()
        ),
        1
    );
    let mut b = config("\"max\":200");
    b.budgets[0].unit = "ms".into();
    assert_eq!(
        budget::exit_code(&budget::evaluate(&b, &r, None).unwrap()),
        2
    );
    b.budgets[0].unit = "ns".into();
    b.budgets[0].case = "missing".into();
    assert_eq!(
        budget::exit_code(&budget::evaluate(&b, &r, None).unwrap()),
        2
    );
    let r = paired(2, 1.0);
    assert_eq!(
        budget::exit_code(
            &budget::evaluate(&config("\"max_regression_percent\":5"), &r, None).unwrap()
        ),
        2
    );
}
#[test]
fn rgba_golden_tolerance_dimensions_and_immutable_storage() {
    use image::RgbaImage;
    let expected = RgbaImage {
        width: 2,
        height: 1,
        pixels: vec![0, 0, 0, 255, 100, 100, 100, 255],
    };
    let mut actual = expected.clone();
    actual.pixels[0] = 3;
    let diff = actual.compare(&expected, 0, 0.).unwrap();
    assert!(!diff.accepted);
    assert_eq!(diff.changed_pixels, 1);
    assert!(actual.compare(&expected, 3, 0.).unwrap().accepted);
    assert!(actual.compare(&expected, 0, 50.).unwrap().accepted);
    assert!(actual.compare(&expected, 0, f64::NAN).is_err());
    actual.width = 1;
    actual.height = 2;
    assert!(actual.compare(&expected, 0, 0.).is_err());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("frame.rbimg");
    expected.save_new(&path).unwrap();
    assert!(expected.save_new(&path).is_err());
    assert_eq!(RgbaImage::load(&path).unwrap().pixels, expected.pixels);
    std::fs::write(&path, b"RBIMG001").unwrap();
    assert!(RgbaImage::load(path).is_err());
}
#[test]
fn html_has_real_table_and_escapes_contract_injection() {
    let mut run = paired(6, 1.);
    run.cases[0]
        .contract
        .insert("unsafe".into(), "</dd><script>alert(1)</script>".into());
    let html = report::html_run(&run).unwrap();
    assert!(html.contains("<table>"));
    assert!(html.contains("<details>"));
    assert!(html.contains("id=\"search\""));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)"));
}

#[test]
fn selection_exact_glob_tags_and_exclusions() {
    let mut suite = Suite::new("s");
    suite.bench("parse/a", || 1).tag("cpu");
    suite.bench("parse/b", || 2).tag("gpu");
    let mut sel = Selection {
        pattern: "s/parse/?".into(),
        glob: true,
        tags: vec!["cpu".into()],
        ..Default::default()
    };
    assert_eq!(suite.list_selected(&sel), vec!["s/parse/a"]);
    sel.exclude.push("*/a".into());
    assert!(suite.list_selected(&sel).is_empty());
    sel.exact = true;
    assert!(suite.run_selected(&sel).is_err());
    assert!(convenience::glob("?ест*", "тест/пример"));
    assert!(!convenience::glob("a?", "a"));
}
#[test]
fn shared_fixture_is_lazy_and_initialized_once_across_cases() {
    let setup = Arc::new(AtomicUsize::new(0));
    let count = setup.clone();
    let fixture = Fixture::new(move || {
        count.fetch_add(1, Ordering::SeqCst);
        vec![1u64, 2, 3]
    });
    let mut suite = Suite::new("fixture");
    suite.bench_fixture("a", fixture.clone(), |v| v.len());
    suite.bench_fixture("b", fixture, |v| v.iter().sum::<u64>());
    assert_eq!(suite.list("").len(), 2);
    assert_eq!(setup.load(Ordering::SeqCst), 0);
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.run("").unwrap();
    assert_eq!(setup.load(Ordering::SeqCst), 1);
}
#[test]
fn phases_require_registration_and_preserve_output() {
    let mut r = Recorder::new();
    assert!(r.measure("x", "p", || panic!("must not run")).is_err());
    r.case(Case {
        id: "x".into(),
        contract: Default::default(),
        metrics: vec![],
    })
    .unwrap();
    r.phase("x", "parse", "CPU parse").unwrap();
    assert!(r.phase("x", "parse", "other scope").is_err());
    let result = r.measure("x", "parse", || vec![1, 2, 3]).unwrap();
    assert_eq!(result, vec![1, 2, 3]);
    let r = r.finish().unwrap();
    assert_eq!(r.cases[0].metrics[0].phase, "parse");
    assert_eq!(r.observations.len(), 1);
}
#[test]
fn profiles_seeded_inputs_and_work_units() {
    assert!(Config::profile("fastest").is_err());
    assert_eq!(Config::profile("quick").unwrap().samples, 8);
    assert_eq!(Seeded::new(0).next_u64(), 0xe220a8397b1dcdaf);
    assert_eq!(Seeded::new(42).bytes(31), Seeded::new(42).bytes(31));
    assert_ne!(Seeded::new(42).bytes(31), Seeded::new(43).bytes(31));
    let mut s = Suite::new("units");
    s.bench("x", || {
        std::thread::sleep(Duration::from_micros(10));
        1
    })
    .work_units("", 0);
    assert!(s.run("").is_err());
    // A separate suite avoids retaining the intentionally invalid unit.
    let mut s = Suite::new("units");
    s.bench_custom("x", |n| Duration::from_nanos(n * 100));
    s.work_units("bytes", 64).seed(42);
    s.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    let run = s.run("").unwrap();
    assert_eq!(run.cases[0].contract["param.seed"], "42");
    assert!(report::markdown(&run).unwrap().contains("MiB/s"));
}
#[test]
fn diagnostic_reports_prioritize_regressions_and_plot_raw_data() {
    let r = paired(12, 1.2);
    let rows = compare(&r, None, 5., 0.05).unwrap();
    let report = report::comparison(&rows);
    assert!(report.contains("1 regressions"));
    let small = compare(&paired(2, 1.), None, 5., 0.05).unwrap();
    assert!(report::advice(&small[0]).contains("independent"));
    let html = report::html_run(&r).unwrap();
    assert!(html.contains("<svg"));
    assert!(html.contains("Raw observation plots"));
    assert!(html.contains("Median per process"));
    assert!(html.contains(airbug_bench::viz::REVEAL_CSS));
    assert!(!html.contains("/*VIZ_CSS*/"));
    assert_eq!(html.matches("@keyframes airbug-reveal").count(), 1);
    assert!(html.contains("prefers-reduced-motion"));
    let plot = report::plot("<script>", &[(0., 1.), (1., 2.)]);
    assert!(!plot.contains("<script>"));
    assert!(plot.contains("&lt;script&gt;"));
}

#[test]
fn comparison_charts_plot_intervals_and_units_without_overclaiming() {
    let r = paired(6, 1.2);
    let rows = compare(&r, None, 5., 0.05).unwrap();
    let charts = report::comparison_charts(&r, &rows, 5., 8);
    assert!(charts.contains("<section class=\"charts\">"));
    assert!(charts.contains("Effect estimates and confidence intervals"));
    assert!(charts.contains("Point estimates by metric"));
    assert!(charts.contains("independent units"));
    assert!(charts.contains("change per unit"));
    assert!(charts.contains("it is not a confidence interval"));
    // Six units are enough for a cumulative view; the lines are drawn in, not faded.
    assert!(charts.contains("cumulative share"));
    assert!(charts.contains("pathLength=\"100\" class=\"dv\""));
    let few = report::comparison_charts(
        &paired(4, 1.2),
        &compare(&paired(4, 1.2), None, 5., 0.05).unwrap(),
        5.,
        8,
    );
    assert!(!few.contains("cumulative share"), "{few}");
    assert_eq!(
        charts.matches("<figure").count(),
        charts.matches("role=\"img\" aria-label=").count()
    );

    let mut hostile = rows.clone();
    hostile[0].case = "<img src=x>".into();
    let charts = report::comparison_charts(&r, &hostile, 5., 8);
    assert!(charts.contains("&lt;img src=x&gt;"));
    assert!(!charts.contains("<img src=x>"));

    let mut candidate = r.clone();
    candidate.observations.retain(|o| o.variant == "candidate");
    // An external baseline run carries its own "candidate" variant, so the compared run has
    // a single variant and per-unit crossover charts must not appear.
    let baseline = candidate.clone();
    let external = compare(&candidate, Some(&baseline), 5., 0.05).unwrap();
    let charts = report::comparison_charts(&candidate, &external, 5., 8);
    assert!(charts.contains("<section class=\"charts\">"));
    assert!(!charts.contains("independent units"));
    assert!(!charts.contains("change per unit"));
    // Without intervals the section says so instead of drawing an empty chart.
    assert!(charts.contains("No finite effect intervals available"));
    assert!(!charts.contains("<svg"));
}

#[test]
fn comparison_heat_maps_every_case_against_every_metric() {
    let r = paired(6, 1.2);
    let mut rows = compare(&r, None, 5., 0.05).unwrap();
    let mut extra = rows[0].clone();
    extra.case = "other".into();
    extra.metric = "cpu".into();
    rows.push(extra);
    // `max_unit_charts = 0` keeps the per-unit charts out of this assertion's way.
    let charts = report::comparison_charts(&r, &rows, 5., 0);
    assert!(charts.contains("Change by case and metric, in percent"));
    assert!(charts.contains("case · latency"));
    assert!(charts.contains("other · cpu"));
    assert!(!charts.contains("independent units"));
    let single_metric = report::comparison_charts(&r, &rows[..1], 5., 8);
    assert!(
        !single_metric.contains("Change by case and metric"),
        "one column is a bar chart, not a matrix"
    );
}

#[test]
fn pairs_keep_the_independent_unit_identity_behind_every_dot() {
    let r = paired(4, 1.5);
    let units = analysis::pairs(&r, None, "case", "latency").unwrap();
    assert_eq!(units.len(), 4);
    assert_eq!(units[3].unit, 3);
    assert_eq!(units[3].baseline, 103.);
    assert_eq!(units[3].candidate, 154.5);
    assert_eq!(units[3].change_percent, Some(50.));
    assert!(analysis::pairs(&r, None, "missing", "latency").is_err());
}

#[test]
fn lifecycle_helpers_join_and_cancel() {
    use airbug_bench::workloads::*;
    let active = std::sync::atomic::AtomicUsize::new(0);
    let peak = std::sync::atomic::AtomicUsize::new(0);
    let result = parallel(4, |i| {
        let n = active.fetch_add(1, Ordering::SeqCst) + 1;
        peak.fetch_max(n, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(20));
        active.fetch_sub(1, Ordering::SeqCst);
        i * i
    })
    .unwrap();
    assert_eq!(result.outputs, vec![0, 1, 4, 9]);
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert!(peak.load(Ordering::SeqCst) > 1);
    assert!(
        parallel(4, |i| {
            if i == 2 {
                panic!("controlled worker failure")
            }
            i
        })
        .is_err()
    );
    assert!(parallel(0, |_| 0).is_err());
    let cancellation = Cancellation::default();
    let p = pipeline((0..100).collect(), 2, &cancellation, |i| i * 2).unwrap();
    assert_eq!(p.outputs, (0..100).map(|i| i * 2).collect::<Vec<_>>());
    assert_eq!(p.latency_ns.len(), 100);
    assert_eq!(p.processed, 100);
    assert!(pipeline(vec![1], 0, &cancellation, |i| i).is_err());
    let c = cancellation.clone();
    assert!(
        pipeline((0..100).collect(), 1, &cancellation, move |i| {
            if i == 3 {
                c.cancel();
            }
            i
        })
        .is_err()
    );
}
#[test]
fn async_wakeup_and_cold_first_invocation() {
    use airbug_bench::workloads::*;
    struct Once(bool);
    impl std::future::Future for Once {
        type Output = u32;
        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<u32> {
            if self.0 {
                std::task::Poll::Ready(42)
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        }
    }
    assert_eq!(LocalExecutor.block_on(Once(false)), 42);
    let calls = std::cell::Cell::new(0);
    let mut s = Suite::new("test");
    s.bench_async("async", LocalExecutor, || async {
        calls.set(calls.get() + 1);
        42
    })
    .cold();
    let r = s.run("").unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(r.observations.len(), 1);
    assert_eq!(r.observations[0].operations, 1);
    assert_eq!(r.cases[0].contract["warmup_ns"], "0");
    assert!(s.run("").is_err());
    s.bench("second", || 0);
    assert!(s.run("").is_err());
}
#[test]
fn phase_peak_realloc_and_cross_thread_free() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let a = alloc::TrackingAllocator::new(System);
    let l = Layout::from_size_align(128, 8).unwrap();
    unsafe {
        let p = a.alloc(l);
        assert!(!p.is_null());
        let phase = a.begin_phase().unwrap();
        assert!(a.begin_phase().is_err());
        let p = a.realloc(p, l, 512);
        assert!(!p.is_null());
        let address = p as usize;
        std::thread::scope(|s| {
            let a = &a;
            s.spawn(move || {
                a.dealloc(address as *mut u8, Layout::from_size_align(512, 8).unwrap())
            })
            .join()
            .unwrap();
        });
        let r = phase.finish();
        assert_eq!(r.live_start_bytes, 128);
        assert_eq!(r.live_end_bytes, 0);
        assert_eq!(r.peak_live_bytes, 512);
        assert_eq!(r.reallocations, 1);
        assert_eq!(r.deallocations, 1);
        let next = a.begin_phase().unwrap().finish();
        assert_eq!(next.peak_live_bytes, 0);
        assert_eq!(next.lifetime_peak_bytes, 512);
    }
}
#[test]
fn diagnostics_known_drift_and_order() {
    let stable = paired(12, 1.);
    let d = diagnostics::diagnose(&stable).unwrap();
    assert!(d.iter().all(|v| v.relative_mad_percent.unwrap() < 5.));
    let mut drift = stable.clone();
    for o in &mut drift.observations {
        o.value = Some((100 + o.process * 10).to_string());
    }
    assert!(
        diagnostics::diagnose(&drift)
            .unwrap()
            .iter()
            .all(|d| d.flags.iter().any(|s| s.contains("chronological")))
    );
    let mut ordered = paired(12, 1.);
    for o in &mut ordered.observations {
        let p = o.pair.unwrap();
        let baseline = o.variant == "baseline";
        o.process = 2 * p + u32::from(baseline == (p % 2 == 1));
        o.value = Some(
            if baseline {
                "100"
            } else if p % 2 == 0 {
                "120"
            } else {
                "100"
            }
            .into(),
        );
    }
    let rows = diagnostics::order_effects(&ordered).unwrap();
    assert!(rows[0].flagged);
    assert_eq!(rows[0].ab_pairs, 6);
    assert_eq!(rows[0].ba_pairs, 6);
}
#[test]
fn multi_reference_family_and_deadlines() {
    let mut r = paired(18, 1.2);
    let mut third = vec![];
    for o in &mut r.observations {
        o.process = o.pair.unwrap() * 3 + u32::from(o.variant == "candidate");
        if o.variant == "baseline" {
            let mut c = o.clone();
            c.variant = "third".into();
            c.process += 2;
            third.push(c);
        }
    }
    r.observations.extend(third);
    let rows = analysis::compare_multi(&r, "baseline", 5., 0.05).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].comparison.decision, Decision::Regression);
    assert_eq!(rows[1].comparison.decision, Decision::WithinMargin);
    assert!(diagnostics::deadlines(&r, "case", "latency", &[60.]).is_err());
    let mut r = paired(12, 1.);
    r.cases[0].metrics[0].statistic = "individual frame".into();
    for o in &mut r.observations {
        o.value = Some(
            if o.process % 4 == 0 {
                "20000000"
            } else {
                "10000000"
            }
            .into(),
        );
    }
    let d = diagnostics::deadlines(&r, "case", "latency", &[60., 120.]).unwrap();
    assert_eq!(d[0].over_budget, 0);
    assert_eq!(d[1].over_budget, 12);
}
#[cfg(feature = "macros")]
#[airbug_bench::bench]
fn attributed_work() -> usize {
    std::hint::black_box(7)
}
#[cfg(feature = "macros")]
#[test]
fn macro_uses_builder_identity() {
    let mut s = Suite::new("test");
    register_attributed_work(&mut s);
    s.cold();
    let r = s.run("").unwrap();
    assert_eq!(r.cases[0].id, "test/attributed_work");
}
#[test]
fn pilot_independent_confirmation_coverage() {
    let mut rng = Seeded::new(9182);
    let mut covered = 0;
    let mut false_positive = 0;
    let experiments = 2000;
    for _ in 0..experiments {
        let mut pilot = paired(12, 1.);
        for o in &mut pilot.observations {
            o.value = Some((0.5 + (rng.next_u64() as f64 / u64::MAX as f64)).to_string());
        }
        let n = diagnostics::pilot(&pilot, 10., 100).unwrap()[0]
            .proposed_processes
            .unwrap();
        let v: Vec<_> = (0..n)
            .map(|_| 0.5 + rng.next_u64() as f64 / u64::MAX as f64)
            .collect();
        let (l, h) = median_interval(&v, 0.05).unwrap();
        covered += usize::from(l <= 1. && h >= 1.);
        false_positive += usize::from(l > 1. || h < 1.);
    }
    assert!(covered as f64 / experiments as f64 > 0.94);
    assert!(false_positive as f64 / (experiments as f64) < 0.06);
}

#[test]
fn throughput_derives_rates_and_scales_bytes_to_mib() {
    let mk_case = |id: &str, unit: &str, count: &str| Case {
        id: id.into(),
        contract: BTreeMap::from([
            ("work.unit".to_string(), unit.to_string()),
            ("work.count".to_string(), count.to_string()),
        ]),
        metrics: vec![Metric::duration("wall", "test", "batch_total")],
    };
    let mut rec = Recorder::new();
    rec.case(mk_case("elems", "elements", "10")).unwrap();
    rec.case(mk_case("bytes", "bytes", "1048576")).unwrap();
    // ops=1, 1e9 ns batch -> elems: 10 elements/s; bytes: 1048576 bytes/s = 1.0 MiB/s.
    rec.observe("elems", "wall", 1_000_000_000).unwrap();
    rec.observe("bytes", "wall", 1_000_000_000).unwrap();
    let run = rec.finish().unwrap();
    let series = report::throughput(&run).unwrap();
    assert_eq!(series.len(), 2);
    let elems = series.iter().find(|s| s.case == "elems").unwrap();
    assert_eq!(elems.unit, "elements");
    assert!((elems.values[0] - 10.0).abs() < 1e-6);
    let bytes = series.iter().find(|s| s.case == "bytes").unwrap();
    assert_eq!(bytes.unit, "MiB");
    assert!((bytes.values[0] - 1.0).abs() < 1e-9);
    // Cases without declared work units produce no series.
    let mut plain = Recorder::new();
    plain
        .case(Case {
            id: "plain".into(),
            contract: BTreeMap::new(),
            metrics: vec![Metric::duration("wall", "test", "batch_total")],
        })
        .unwrap();
    plain.observe("plain", "wall", 1_000_000_000).unwrap();
    assert!(
        report::throughput(&plain.finish().unwrap())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn regex_selection_and_exact_exclusions_are_lazy_and_unicode_aware() {
    let mut suite = Suite::new("s");
    for name in ["parse/2", "parse/12", "parse/12/child", "тест/42", "other"] {
        suite.bench(name, || panic!("selection must not execute workload"));
    }
    let mut selection = Selection::default()
        .with_regex(r"^s/(parse|тест)/\d+$")
        .unwrap();
    assert_eq!(
        suite.list_selected(&selection),
        ["s/parse/2", "s/parse/12", "s/тест/42"]
    );
    selection.exclude_exact.push("s/parse/2".into());
    selection.skip_regex(r"/тест/").unwrap();
    assert_eq!(suite.list_selected(&selection), ["s/parse/12"]);
    assert!(Selection::default().with_regex("[").is_err());
    assert!(selection.skip_regex("(").is_err());
    // An unsuccessful mutation preserves the existing valid exclusions.
    assert_eq!(suite.list_selected(&selection), ["s/parse/12"]);
    selection.glob = true;
    assert!(suite.run_selected(&selection).is_err());
    let exact_exclusion = Selection {
        exclude_exact: vec!["s/parse/12".into()],
        ..Default::default()
    };
    assert!(
        suite
            .list_selected(&exact_exclusion)
            .contains(&"s/parse/12/child")
    );
}

#[test]
fn time_budget_distinguishes_reported_intervals_from_full_workload_calls() {
    for measured_only in [false, true] {
        let calls = AtomicUsize::new(0);
        let mut suite = Suite::new("accounting");
        suite.bench_custom("external", |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            // Simulate caller-excluded setup/drop. The reported interval is deterministic.
            std::thread::sleep(Duration::from_millis(15));
            Duration::from_nanos(1)
        });
        suite.sampling(Sampling {
            iterations: Some(1),
            samples: Some(2),
            warmup: Some(Duration::ZERO),
            max_time: Some(Duration::from_millis(5)),
            exclude_external_time: Some(measured_only),
            ..Default::default()
        });
        let run = suite.run("").unwrap();
        let expected = if measured_only { 2 } else { 1 };
        assert_eq!(run.observations.len(), expected);
        assert_eq!(calls.load(Ordering::SeqCst), expected);
        assert!(
            run.observations
                .iter()
                .all(|o| o.value.as_deref() == Some("1"))
        );
        assert_eq!(
            run.notes.iter().any(|n| n.contains("max_time reached")),
            !measured_only
        );
    }
}

#[test]
fn zero_maximum_time_disables_work_and_positive_override_reenables_it() {
    let calls = AtomicUsize::new(0);
    let mut suite = Suite::new("zero-maximum");
    suite.bench_custom("work", |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        Duration::from_nanos(1)
    });
    let disabled = Sampling {
        samples: Some(1),
        iterations: Some(1),
        warmup: Some(Duration::ZERO),
        max_time: Some(Duration::ZERO),
        exclude_external_time: Some(true),
        ..Default::default()
    };
    suite.sampling(disabled.clone());
    assert!(suite.list("").is_empty());
    assert!(suite.run("").unwrap().observations.is_empty());
    assert!(
        suite
            .test_selected(&Selection {
                include_ignored: true,
                ..Default::default()
            })
            .unwrap()
            .observations
            .is_empty()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    suite.sampling(Sampling {
        max_time: Some(Duration::from_nanos(1)),
        ..disabled
    });
    assert_eq!(suite.list(""), ["zero-maximum/work"]);
    assert_eq!(suite.run("").unwrap().observations.len(), 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn zero_duration_measured_budgets_make_progress_without_changing_observations() {
    for (maximum, expected) in [(None, 3), (Some(Duration::from_nanos(2)), 2)] {
        let calls = AtomicUsize::new(0);
        let mut suite = Suite::new("zero-budget");
        suite.bench_custom("zero", |_| {
            let count = calls.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(
                count <= 8,
                "zero durations must not produce an unbounded sample loop"
            );
            Duration::ZERO
        });
        suite.sampling(Sampling {
            iterations: Some(1),
            samples: Some(1),
            warmup: Some(Duration::ZERO),
            min_time: Some(Duration::from_nanos(3)),
            max_time: maximum,
            exclude_external_time: Some(true),
            ..Default::default()
        });
        let run = suite.run("").unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), expected);
        assert_eq!(run.observations.len(), expected);
        assert!(
            run.observations
                .iter()
                .all(|o| o.value.as_deref() == Some("0"))
        );
        assert_eq!(
            run.cases[0].contract["sampling.zero_duration_budget_floor_ns"],
            "1"
        );
        assert!(
            run.notes
                .iter()
                .any(|note| note.contains("1 ns progress floor"))
        );
    }
}

#[test]
fn calibration_and_warmup_consume_the_time_budget() {
    for fixed in [false, true] {
        let calls = AtomicUsize::new(0);
        let mut suite = Suite::new("limits");
        suite.bench_custom("bounded", |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Duration::from_nanos(5)
        });
        suite.sampling(Sampling {
            iterations: fixed.then_some(1),
            samples: Some(2),
            warmup: Some(if fixed {
                Duration::from_secs(60)
            } else {
                Duration::ZERO
            }),
            sample_time: Some(Duration::from_nanos(100)),
            max_time: Some(Duration::from_nanos(9)),
            exclude_external_time: Some(true),
            ..Default::default()
        });
        let error = suite.run("").unwrap_err().to_string();
        assert!(
            error.contains(if fixed {
                "before collecting a sample"
            } else {
                "during calibration"
            }),
            "{error}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
    let calls = AtomicUsize::new(0);
    let mut suite = Suite::new("minimum");
    suite.bench_custom("count_calibration", |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        Duration::from_nanos(5)
    });
    suite.sampling(Sampling {
        samples: Some(1),
        warmup: Some(Duration::ZERO),
        sample_time: Some(Duration::from_nanos(5)),
        min_time: Some(Duration::from_nanos(12)),
        exclude_external_time: Some(true),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(run.observations.len(), 2); // 5 ns calibration + 10 ns collected.
}

#[test]
fn black_box_drop_consumes_lazy_noncopy_outputs_once() {
    struct Output<'a>(&'a AtomicUsize);
    impl Drop for Output<'_> {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let produced = AtomicUsize::new(0);
    let dropped = AtomicUsize::new(0);
    let values = (0..7).map(|_| {
        produced.fetch_add(1, Ordering::SeqCst);
        Output(&dropped)
    });
    assert_eq!(produced.load(Ordering::SeqCst), 0);
    values.for_each(airbug_bench::black_box_drop);
    assert_eq!(produced.load(Ordering::SeqCst), 7);
    assert_eq!(dropped.load(Ordering::SeqCst), 7);
    assert_eq!(
        airbug_bench::black_box(String::from("retained")),
        "retained"
    );
}

#[test]
fn sorting_matches_listing_execution_and_preserves_registration_order() {
    use std::sync::Mutex;
    for (sort, reverse, expected) in [
        (
            SortOrder::Registration,
            false,
            vec!["case10", "case2", "case01", "case1"],
        ),
        (
            SortOrder::Lexical,
            false,
            vec!["case01", "case1", "case10", "case2"],
        ),
        (
            SortOrder::Natural,
            false,
            vec!["case01", "case1", "case2", "case10"],
        ),
        (
            SortOrder::Natural,
            true,
            vec!["case10", "case2", "case1", "case01"],
        ),
    ] {
        let calls = Mutex::new(Vec::new());
        let mut suite = Suite::new("order");
        for name in ["case10", "case2", "case01", "case1"] {
            let calls = &calls;
            suite.bench(name, move || calls.lock().unwrap().push(name));
        }
        let selection = Selection {
            sort,
            reverse,
            ..Default::default()
        };
        let expected_ids: Vec<_> = expected.iter().map(|n| format!("order/{n}")).collect();
        assert_eq!(suite.list_selected(&selection), expected_ids);
        assert!(calls.lock().unwrap().is_empty());
        let run = suite.test_selected(&selection).unwrap();
        assert_eq!(*calls.lock().unwrap(), expected);
        assert_eq!(
            run.cases.iter().map(|c| &c.id).collect::<Vec<_>>(),
            expected_ids.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            suite.list(""),
            ["order/case10", "order/case2", "order/case01", "order/case1"]
        );
    }
    let mut suite = Suite::new("unicode");
    for name in [
        "тест10000000000000000000000000000000000000000",
        "тест9",
        "тест10",
    ] {
        suite.bench(name, || ());
    }
    let selection = Selection {
        sort: SortOrder::Natural,
        ..Default::default()
    };
    assert_eq!(
        suite.list_selected(&selection),
        [
            "unicode/тест9",
            "unicode/тест10",
            "unicode/тест10000000000000000000000000000000000000000"
        ]
    );
}

#[test]
fn source_sort_orders_files_lines_columns_and_unknown_locations() {
    let mut suite = Suite::new("source");
    suite.bench("unknown", || ());
    suite.bench("b", || ()).source_location("b.rs", 1, 1);
    suite.bench("a20", || ()).source_location("a.rs", 20, 1);
    suite.bench("a3c9", || ()).source_location("a.rs", 3, 9);
    suite.bench("a3c1", || ()).source_location("a.rs", 3, 1);
    let selection = Selection {
        sort: SortOrder::Source,
        ..Default::default()
    };
    assert_eq!(
        suite.list_selected(&selection),
        [
            "source/a3c1",
            "source/a3c9",
            "source/a20",
            "source/b",
            "source/unknown"
        ]
    );
}

#[test]
fn bootstrap_uses_process_units_and_normalizes_batch_totals() {
    let config = bootstrap::Config {
        resamples: 200,
        ..Default::default()
    };
    let mut run = paired(8, 1.2);
    let report = bootstrap::analyze(&run, &config).unwrap();
    assert!(
        report
            .rows
            .iter()
            .all(|r| r.units == 8 && r.resampling_unit == "process_median")
    );
    let original = report.rows[0].estimates.clone();
    let mut extra = run.observations[0].clone();
    extra.sequence = 1;
    run.observations.push(extra);
    assert_eq!(
        bootstrap::analyze(&run, &config).unwrap().rows[0].estimates,
        original
    );

    let mut suite = Suite::new("normalized");
    suite.bench_custom("constant", |n| Duration::from_nanos(n * 20));
    suite.sampling(Sampling {
        iterations: Some(5),
        samples: Some(4),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    let report = bootstrap::analyze(&run, &config).unwrap();
    assert_eq!(report.rows[0].estimates.as_ref().unwrap().mean.point, 20.0);
    assert_eq!(report.rows[0].resampling_unit, "normalized_observation");
    assert!(report.rows[0].note.contains("autocorrelation"));
}

#[test]
fn linear_sampling_report_fits_totals_per_process() {
    let mut suite = Suite::new("regression");
    suite.bench_custom("linear", |n| Duration::from_nanos(n * 10));
    suite.sampling(Sampling {
        samples: Some(5),
        sample_time: Some(Duration::from_nanos(100)),
        warmup: Some(Duration::ZERO),
        mode: Some(SamplingMode::Linear),
        ..Default::default()
    });
    let mut run = suite.run("").unwrap();
    let config = bootstrap::Config {
        resamples: 200,
        ..Default::default()
    };
    let report = bootstrap::analyze(&run, &config).unwrap();
    assert_eq!(report.rows[0].regressions.len(), 1);
    let fit = &report.rows[0].regressions[0].fit;
    assert!((fit.slope.point - 10.0).abs() < 1e-12);
    assert!((fit.slope.lower - 10.0).abs() < 1e-12);
    assert!((fit.slope.upper - 10.0).abs() < 1e-12);
    assert_eq!(fit.samples, 5);
    assert_eq!(fit.r_squared, Some(1.0));
    let another: Vec<_> = run
        .observations
        .iter()
        .cloned()
        .map(|mut o| {
            o.process = 1;
            o.value = Some((o.value.unwrap().parse::<u64>().unwrap() * 2).to_string());
            o
        })
        .collect();
    run.observations.extend(another);
    let report = bootstrap::analyze(&run, &config).unwrap();
    assert_eq!(report.rows[0].units, 2);
    assert_eq!(report.rows[0].regressions.len(), 2);
    assert!((report.rows[0].regressions[1].fit.slope.point - 20.0).abs() < 1e-12);
    assert!(bootstrap::markdown(&report).contains("slope (process 1)"));
}

#[test]
fn outlier_reporting_preserves_extreme_observations_in_estimates() {
    let mut run = paired(6, 1.0);
    for observation in &mut run.observations {
        observation.value = Some(
            if observation.pair == Some(5) {
                "10000"
            } else {
                "1"
            }
            .into(),
        );
    }
    let report = bootstrap::analyze(
        &run,
        &bootstrap::Config {
            resamples: 100,
            ..Default::default()
        },
    )
    .unwrap();
    for row in &report.rows {
        assert_eq!(row.outliers.as_ref().unwrap().counts, [0, 0, 5, 0, 1]);
        assert_eq!(row.outliers.as_ref().unwrap().points.len(), 6);
        assert!((row.estimates.as_ref().unwrap().mean.point - 10005.0 / 6.0).abs() < 1e-10);
    }
    let html = bootstrap::html(&report);
    assert!(html.contains("<svg") && html.contains("No observations discarded"));
    assert!(!html.contains("<!--CHARTS-->"));
    assert_eq!(html.matches("all fences (zero IQR)").count(), 2);
}

#[test]
fn byte_format_scaling_is_display_only_and_keeps_custom_units() {
    use report::BytesFormat;
    let mut suite = Suite::new("formats");
    suite
        .bench_custom("one_second", |_| Duration::from_secs(1))
        .work_units("bytes", 1_048_576)
        .work_units("items", 3)
        .work_units("MiB", 2);
    suite.sampling(Sampling {
        samples: Some(2),
        iterations: Some(1),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    let run = suite.run("").unwrap();
    let before = serde_json::to_string(&run).unwrap();
    let decimal = report::throughput_with_format(&run, BytesFormat::Decimal).unwrap();
    let binary = report::throughput_with_format(&run, BytesFormat::Binary).unwrap();
    assert_eq!(
        decimal.iter().find(|s| s.unit == "MB").unwrap().values,
        [1.048576; 2]
    );
    assert!(
        binary
            .iter()
            .any(|s| s.unit == "MiB" && s.values == [1.0; 2])
    );
    // A user-defined counter named MiB must not be mistaken for the bytes counter.
    assert!(
        decimal
            .iter()
            .any(|s| s.unit == "MiB" && s.values == [2.0; 2])
    );
    assert!(
        decimal
            .iter()
            .any(|s| s.unit == "items" && s.values == [3.0; 2])
    );
    assert_eq!(serde_json::to_string(&run).unwrap(), before);
    assert!(
        report::markdown_with_bytes_format(&run, BytesFormat::Decimal)
            .unwrap()
            .contains("MB/s")
    );
    for (count, decimal_unit, binary_unit) in [
        (0, "B", "B"),
        (999, "B", "B"),
        (1000, "KB", "B"),
        (1024, "KB", "KiB"),
        (1_000_000_000, "GB", "MiB"),
    ] {
        let mut run = run.clone();
        run.cases[0].contract = BTreeMap::from([("work.counter.bytes".into(), count.to_string())]);
        assert_eq!(
            report::throughput_with_format(&run, BytesFormat::Decimal).unwrap()[0].unit,
            decimal_unit
        );
        assert_eq!(
            report::throughput_with_format(&run, BytesFormat::Binary).unwrap()[0].unit,
            binary_unit
        );
    }
}

#[test]
fn dynamic_work_totals_exclude_pilot_and_warmup_and_drive_throughput() {
    use airbug_bench::counters::InputCounters;
    use std::cell::RefCell;
    let counts = InputCounters::new(&["bytes", "items"]).unwrap();
    let collect = counts.clone();
    let batches = RefCell::new(vec![]);
    let mut next = 0u64;
    let mut suite = Suite::new("dynamic");
    suite
        .bench_custom("input", |n| {
            let mut sum = 0;
            for _ in 0..n {
                next += 1;
                collect.add("bytes", next);
                collect.add("items", 0);
                sum += next;
            }
            batches.borrow_mut().push(sum);
            Duration::from_nanos(n * 100)
        })
        .input_counters(counts)
        .work_units("bytes", 999);
    suite.config(Config {
        samples: 2,
        warmup: Duration::from_micros(10),
        sample_time: Duration::from_nanos(300),
        max_iterations: 3,
    });
    let run = suite.run("").unwrap();
    assert!(batches.borrow().len() > 2);
    let expected = &batches.borrow()[batches.borrow().len() - 2..];
    for (observation, total) in run.observations.iter().zip(expected) {
        assert_eq!(observation.work_totals["bytes"], total.to_string());
        assert_eq!(observation.work_totals["items"], "0");
    }
    let rates = report::throughput(&run).unwrap();
    let bytes = rates.iter().find(|r| r.unit == "MiB").unwrap();
    for ((rate, total), observation) in bytes.values.iter().zip(expected).zip(&run.observations) {
        let wanted = *total as f64 * 1e9 / observation.number().unwrap().unwrap() / 1_048_576.0;
        assert!((rate - wanted).abs() < 1e-8);
    }
    assert!(
        rates
            .iter()
            .find(|r| r.unit == "items")
            .unwrap()
            .values
            .iter()
            .all(|n| *n == 0.0)
    );
    let encoded = serde_json::to_vec(&run).unwrap();
    let roundtrip: Run = serde_json::from_slice(&encoded).unwrap();
    roundtrip.validate().unwrap();
    assert_eq!(
        roundtrip.observations[0].work_totals,
        run.observations[0].work_totals
    );
    let mut missing = roundtrip.clone();
    missing.observations[0].work_totals.clear();
    assert!(missing.validate().is_err());
    assert!(report::throughput(&missing).is_err());
    let mut invalid = roundtrip;
    invalid.observations[0]
        .work_totals
        .insert("bytes".into(), "-1".into());
    assert!(invalid.validate().is_err());
    assert!(report::throughput(&invalid).is_err());
}

#[test]
fn invalid_dynamic_counter_units_fail_the_run() {
    use airbug_bench::counters::InputCounters;
    assert!(InputCounters::new(&[]).is_err());
    assert!(InputCounters::new(&[""]).is_err());
    assert!(InputCounters::new(&["items", "items"]).is_err());
    let counts = InputCounters::new(&["items"]).unwrap();
    let collect = counts.clone();
    let mut suite = Suite::new("invalid-input-count");
    suite
        .bench_custom("case", move |_| {
            collect.add("unknown", 1);
            Duration::from_nanos(1)
        })
        .input_counters(counts);
    assert!(suite.run("").is_err());
}

#[test]
fn async_workers_poll_concurrently_and_release_after_future_panic() {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        for fail in [false, true] {
            let gate = std::sync::Barrier::new(2);
            let reached = std::sync::atomic::AtomicUsize::new(0);
            let mut suite = Suite::new("async-concurrency");
            suite.bench_async_threads(
                "case",
                2,
                || airbug_bench::workloads::LocalExecutor,
                async || {
                    gate.wait();
                    let id = reached.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    assert!(!(fail && id == 0), "future panic");
                },
                airbug_bench::DropPolicy::OutsideTiming,
            );
            suite.config(Config {
                samples: 1,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 1,
            });
            suite.sampling(airbug_bench::Sampling {
                iterations: Some(1),
                ..Default::default()
            });
            let result = suite.run("");
            assert_eq!(result.is_err(), fail);
            assert_eq!(reached.load(std::sync::atomic::Ordering::SeqCst), 2);
            if let Ok(run) = result {
                assert_eq!(run.observations[0].operations, 2);
            }
        }
        tx.send(()).unwrap();
    });
    rx.recv_timeout(Duration::from_secs(5))
        .expect("async workers must run concurrently and not deadlock on panic");
    handle.join().unwrap();
}

#[test]
fn kind_order_keeps_case_labels_distinct_from_nested_groups() {
    use airbug_bench::{Selection, SortOrder, Suite};
    let mut suite = Suite::new("root");
    suite.group("a", |suite| {
        suite.group("a", |suite| {
            suite.bench("first", || ());
        });
        suite.bench("z", || ());
    });
    suite.bench("z/10", || ());
    suite.bench("z/2", || ());
    let mut selection = Selection {
        sort: SortOrder::Kind,
        ..Default::default()
    };
    let expected = vec!["root/z/2", "root/z/10", "root/a/z", "root/a/a/first"];
    assert_eq!(suite.list_selected(&selection), expected);
    let run = suite.test_selected(&selection).unwrap();
    assert_eq!(
        run.cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    selection.reverse = true;
    assert_eq!(
        suite.list_selected(&selection),
        expected.into_iter().rev().collect::<Vec<_>>()
    );
}

#[test]
fn descriptive_rows_separate_processes_and_do_not_hide_missing_observations() {
    let mut run = paired(3, 1.2);
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    assert_eq!(rows.len(), 6);
    for row in &rows {
        let observation = run
            .observations
            .iter()
            .find(|o| o.variant == row.variant && o.process == row.process)
            .unwrap();
        let summary = row.summary.as_ref().unwrap();
        assert_eq!(summary.count, 1);
        assert_eq!(summary.standard_deviation, None);
        assert_eq!(
            summary.mean,
            observation.number().unwrap().unwrap() / observation.operations as f64
        );
    }
    run.observations[0].value = None;
    run.observations[0].availability = Availability::Unsupported("missing counter".into());
    let rows = airbug_bench::report::descriptive(&run).unwrap();
    assert_eq!(rows.iter().filter(|r| r.summary.is_none()).count(), 1);
    assert_eq!(rows.iter().map(|r| r.unavailable).sum::<usize>(), 1);
}

#[test]
fn descriptive_operation_mean_weights_unequal_batches_and_survives_large_totals() {
    let mut run = paired(1, 1.0);
    run.cases[0].metrics[0].statistic = "batch_total".into();
    run.observations.truncate(1);
    run.observations[0].pair = None;
    run.observations[0].value = Some("10".into());
    let mut second = run.observations[0].clone();
    second.sequence = 1;
    second.operations = 9;
    second.value = Some("180".into());
    run.observations.push(second);
    let rows = report::descriptive(&run).unwrap();
    assert_eq!(rows[0].operations, "10");
    assert_eq!(rows[0].summary.as_ref().unwrap().mean, 15.0);
    assert!((rows[0].operation_weighted_mean.unwrap() - 19.0).abs() < 1e-12);
    for observation in &mut run.observations {
        observation.operations = 1;
        observation.value = Some("1e308".into());
    }
    let rows = report::descriptive(&run).unwrap();
    assert_eq!(rows[0].operation_weighted_mean, Some(1e308));
    run.observations[1].value = None;
    run.observations[1].availability = Availability::Incomplete("missing".into());
    assert!(
        report::descriptive(&run).unwrap()[0]
            .operation_weighted_mean
            .is_none()
    );
}

#[test]
fn associated_work_counts_follow_timing_order_not_counter_order() {
    let mut run = paired(1, 1.0);
    run.cases[0].metrics[0] = Metric::duration("wall", "test", "batch_total");
    run.cases[0]
        .contract
        .insert("work.input.items".into(), "batch_total".into());
    run.cases[0]
        .contract
        .insert("work.counter.bytes".into(), "0".into());
    let mut original = run.observations[0].clone();
    original.metric = "wall".into();
    original.pair = None;
    run.observations.clear();
    for (sequence, (time, work)) in [(10, 100), (40, 3), (20, 50), (30, 10)]
        .into_iter()
        .enumerate()
    {
        let mut observation = original.clone();
        observation.sequence = sequence as u64;
        observation.value = Some(time.to_string());
        observation
            .work_totals
            .insert("items".into(), work.to_string());
        run.observations.push(observation);
    }
    let rows = report::descriptive(&run).unwrap();
    let counts = &rows[0].associated_counters["items"];
    assert_eq!(counts.fastest, 100.0);
    assert_eq!(counts.slowest, 3.0);
    assert_eq!(counts.median, 30.0);
    assert_eq!(counts.operation_mean, 40.75);
    assert_eq!(rows[0].associated_counters["bytes"].operation_mean, 0.0);
    run.observations[1].operations = 100;
    let weighted = report::descriptive(&run).unwrap();
    let counts = &weighted[0].associated_counters["items"];
    assert_eq!(counts.fastest, 0.03);
    assert_eq!(counts.slowest, 10.0);
    assert_eq!(counts.median, 75.0);
    assert!((counts.operation_mean - 163.0 / 103.0).abs() < 1e-12);
    assert!(report::descriptive_markdown(&weighted).contains("Work counts associated with timing"));
    run.observations[1].value = None;
    run.observations[1].availability = Availability::Incomplete("missing timing".into());
    assert!(
        report::descriptive(&run).unwrap()[0]
            .associated_counters
            .is_empty()
    );
}

#[test]
fn allocation_phases_distinguish_growth_shrink_and_freed_bytes() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let allocator = alloc::TrackingAllocator::new(System);
    // Explicit allocator calls isolate the accounting from the test harness.
    unsafe {
        let original = Layout::from_size_align(32, 8).unwrap();
        let pointer = allocator.alloc_zeroed(original);
        assert!(!pointer.is_null());
        let phase = allocator.begin_phase().unwrap();
        let pointer = allocator.realloc(pointer, original, 128);
        assert!(!pointer.is_null());
        let pointer = allocator.realloc(pointer, Layout::from_size_align(128, 8).unwrap(), 16);
        assert!(!pointer.is_null());
        allocator.dealloc(pointer, Layout::from_size_align(16, 8).unwrap());
        let result = phase.finish();
        assert_eq!(result.allocations, 0);
        assert_eq!(result.allocated_bytes, 0);
        assert_eq!(result.deallocations, 1);
        assert_eq!(result.deallocated_bytes, 16);
        assert_eq!(result.reallocations, 2);
        assert_eq!(result.grow_operations, 1);
        assert_eq!(result.shrink_operations, 1);
        assert_eq!(result.grown_bytes, 96);
        assert_eq!(result.shrunk_bytes, 112);
        assert_eq!(result.live_start_bytes, 32);
        assert_eq!(result.live_end_bytes, 0);
        assert_eq!(result.peak_above_start_bytes, 96);
        let snapshot = allocator.snapshot();
        assert_eq!(snapshot.allocated_bytes, 32);
        assert_eq!(
            snapshot.allocated_bytes + snapshot.grown_bytes,
            snapshot.deallocated_bytes + snapshot.shrunk_bytes + snapshot.live_bytes
        );
        let empty = allocator.begin_phase().unwrap().finish();
        assert_eq!(
            (
                empty.grow_operations,
                empty.shrink_operations,
                empty.allocated_bytes,
                empty.deallocated_bytes
            ),
            (0, 0, 0, 0)
        );
    }
}

#[test]
fn summary_scales_inherit_lazily_with_nearest_and_explicit_linear_overrides() {
    use airbug_bench::viz::charts::AxisScale::{Linear, Logarithmic};
    let mut suite = airbug_bench::Suite::new("plots");
    suite.summary_scale(Logarithmic);
    suite.bench("default", || panic!("must stay lazy"));
    suite.with_summary_scale_defaults(Linear, |suite| {
        suite.group("outer", |suite| {
            suite.bench("inherited", || panic!("must stay lazy"));
            suite.with_summary_scale_defaults(Logarithmic, |suite| {
                suite.group("inner", |suite| {
                    suite.bench("inherited", || panic!("must stay lazy"));
                    suite.bench("explicit", || panic!("must stay lazy"));
                    suite.summary_scale_case(Linear);
                });
            });
        });
    });
    suite.bench("after", || panic!("must stay lazy"));
    let scales = suite.summary_scales(&Default::default()).unwrap();
    assert_eq!(scales["plots/default"], Logarithmic);
    assert_eq!(scales["plots/outer/inherited"], Linear);
    assert_eq!(scales["plots/outer/inner/inherited"], Logarithmic);
    assert_eq!(scales["plots/outer/inner/explicit"], Linear);
    assert_eq!(scales["plots/after"], Logarithmic);
}

#[test]
fn suite_results_persist_resolved_summary_scales() {
    use airbug_bench::viz::charts::AxisScale::{Linear, Logarithmic};
    let mut suite = airbug_bench::Suite::new("persisted");
    suite.summary_scale(Logarithmic);
    suite.bench("inherited", || ());
    suite.bench("explicit", || ());
    suite.summary_scale_case(Linear);
    let run = suite.test_selected(&Default::default()).unwrap();
    let restored: airbug_bench::Run =
        serde_json::from_str(&serde_json::to_string(&run).unwrap()).unwrap();
    let scales = airbug_bench::presentation::load_scales(&restored).unwrap();
    assert_eq!(scales["persisted/inherited"], Logarithmic);
    assert_eq!(scales["persisted/explicit"], Linear);
}

#[test]
fn declared_summary_families_persist_and_connect_only_related_inputs() {
    let mut suite = airbug_bench::Suite::new("families");
    for size in [100, 1, 10] {
        suite.bench(&format!("sort/{size}"), || ());
        suite.parameter("size", size.to_string());
        suite.summary_family("sort");
    }
    suite.group("other", |suite| {
        suite.bench("sort", || ());
        suite.parameter("size", "1");
        suite.summary_family("sort");
    });
    let mut run = suite.test_selected(&Default::default()).unwrap();
    for o in &mut run.observations {
        o.value = Some("10".into());
    }
    let restored: airbug_bench::Run =
        serde_json::from_str(&serde_json::to_string(&run).unwrap()).unwrap();
    let families = airbug_bench::presentation::load_families(&restored).unwrap();
    assert_eq!(families["families/sort/100"], "families/sort");
    assert_eq!(families["families/other/sort"], "families/other/sort");
    let config = airbug_bench::report::SummaryPlot {
        estimator: Default::default(),
        parameter: "size",
        scale: airbug_bench::viz::charts::AxisScale::Linear,
    };
    let html = airbug_bench::report::parameter_charts(&restored, &config).unwrap();
    assert!(html.contains("Lines connect observed estimates"));
    assert!(html.contains("family families/sort / candidate"));
    for size in [1, 10, 100] {
        assert!(html.contains(&format!("case families/sort/{size}: x = {size}, y = 10")));
    }
    assert!(html.contains("family families/other/sort / candidate"));
    assert!(!html.contains("line summary unavailable"));
    let mut missing = restored.clone();
    for o in &mut missing.observations {
        if o.case == "families/sort/10" {
            o.value = None;
            o.availability = airbug_bench::Availability::Unsupported("fixture".into());
        }
    }
    let gap = airbug_bench::report::parameter_charts(&missing, &config).unwrap();
    assert!(gap.contains("no line crosses an unavailable case"));
    assert!(!gap.contains("Lines connect observed estimates"));
    assert_eq!(gap.matches("<circle").count(), 3);
    for size in [1, 100] {
        assert!(gap.contains(&format!("case families/sort/{size}: x = {size}, y = 10")));
    }
    assert!(!gap.contains("case families/sort/10: x ="));
    let mut duplicate = restored.clone();
    duplicate.cases[0]
        .contract
        .insert("param.size".into(), "1".into());
    assert!(
        airbug_bench::report::parameter_charts(&duplicate, &config)
            .unwrap()
            .contains("duplicate inputs")
    );
}

#[test]
fn throughput_input_lines_use_equal_process_weights_and_separate_units() {
    let mut suite = airbug_bench::Suite::new("rates");
    for size in [1, 10] {
        suite.bench(&format!("case/{size}"), || ());
        suite.parameter("size", size.to_string());
        suite.summary_family("work");
    }
    let mut run = suite.test_selected(&Default::default()).unwrap();
    for case in &mut run.cases {
        case.contract
            .insert("work.counter.items".into(), "2".into());
        case.contract
            .insert("work.counter.chars".into(), "4".into());
    }
    let originals = run.observations.clone();
    run.observations.clear();
    for original in originals {
        for sequence in 0..6 {
            let mut o = original.clone();
            o.process = if sequence < 5 { 0 } else { 1 };
            o.sequence = sequence;
            o.operations = 1;
            o.value = Some(if sequence < 5 { "2" } else { "1" }.into());
            run.observations.push(o);
        }
    }
    let values = airbug_bench::report::throughput_process_medians(&run).unwrap();
    let item_values: Vec<_> = values
        .iter()
        .filter(|r| r.case == "rates/case/1" && r.unit == "items")
        .map(|r| r.median)
        .collect();
    assert_eq!(item_values, [1e9, 2e9]);
    let config = airbug_bench::report::SummaryPlot {
        estimator: Default::default(),
        parameter: "size",
        scale: airbug_bench::viz::charts::AxisScale::Linear,
    };
    let html = airbug_bench::report::throughput_parameter_charts(&run, &config).unwrap();
    assert!(html.contains("items/s") && html.contains("chars/s"));
    assert_eq!(html.matches("<svg").count(), 2);
    assert!(html.contains("Lines connect observed estimates"));
    assert!(html.contains("1.50e9"));
    for size in [1, 10] {
        assert!(html.contains(&format!("case rates/case/{size}: x = {size}, y =")));
    }
    for o in &mut run.observations {
        o.value = Some("0".into());
    }
    assert!(
        !airbug_bench::report::throughput_parameter_charts(&run, &config)
            .unwrap()
            .contains("<svg")
    );
}

#[test]
fn throughput_families_disconnect_zero_missing_and_unavailable_members() {
    let mut suite = airbug_bench::Suite::new("gaps");
    for size in [1, 2, 3] {
        suite.bench(&format!("case/{size}"), || ());
        suite.parameter("size", size.to_string());
        suite.summary_family("family");
    }
    let mut run = suite.test_selected(&Default::default()).unwrap();
    for case in &mut run.cases {
        case.contract
            .insert("work.counter.items".into(), "1".into());
    }
    for o in &mut run.observations {
        o.value = Some("10".into());
    }
    let config = airbug_bench::report::SummaryPlot {
        estimator: Default::default(),
        parameter: "size",
        scale: airbug_bench::viz::charts::AxisScale::Linear,
    };
    for mode in ["zero", "missing", "unavailable", "parameter"] {
        let mut partial = run.clone();
        if mode == "missing" {
            partial.status = airbug_bench::Status::Incomplete;
        }
        match mode {
            "missing" => partial.observations.retain(|o| o.case != "gaps/case/2"),
            "parameter" => {
                partial.cases[1].contract.remove("param.size");
            }
            _ => {
                for o in &mut partial.observations {
                    if o.case == "gaps/case/2" {
                        if mode == "zero" {
                            o.value = Some("0".into());
                        } else {
                            o.value = None;
                            o.availability =
                                airbug_bench::Availability::Unsupported("fixture".into());
                        }
                    }
                }
            }
        }
        let chart = airbug_bench::report::throughput_parameter_charts(&partial, &config).unwrap();
        assert!(chart.contains("Incomplete throughput families"), "{mode}");
        assert_eq!(chart.matches("<circle").count(), 2, "{mode}");
        // The chart retains axes/points but has no family line path.
        assert!(!chart.contains("pathLength=\"100\""), "{mode}");
    }
}

#[test]
fn name_sort_keeps_real_groups_contiguous_despite_slashes_in_leaf_labels() {
    let mut suite = Suite::new("root");
    suite.bench("a/0", || ());
    suite.group("a", |suite| {
        suite.bench("z/10", || ());
        suite.bench("z/2", || ());
    });
    for (sort, expected) in [
        (
            SortOrder::Natural,
            vec!["root/a/z/2", "root/a/z/10", "root/a/0"],
        ),
        (
            SortOrder::Lexical,
            vec!["root/a/z/10", "root/a/z/2", "root/a/0"],
        ),
    ] {
        let mut selection = Selection {
            sort,
            ..Default::default()
        };
        assert_eq!(suite.list_selected(&selection), expected);
        let run = suite.test_selected(&selection).unwrap();
        assert_eq!(
            run.cases
                .iter()
                .map(|case| case.id.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        selection.reverse = true;
        assert_eq!(
            suite.list_selected(&selection),
            expected.into_iter().rev().collect::<Vec<_>>()
        );
    }
}

#[test]
fn source_sort_uses_group_declarations_before_child_function_locations() {
    let mut suite = Suite::new("root");
    suite.group("late", |suite| {
        suite.group_source_location("groups.rs", 20, 1);
        suite
            .bench("early_child", || ())
            .source_location("a.rs", 1, 1);
    });
    suite
        .bench("middle", || ())
        .source_location("groups.rs", 15, 1);
    suite.group("early", |suite| {
        suite.group_source_location("groups.rs", 10, 1);
        suite
            .bench("late_child", || ())
            .source_location("z.rs", 100, 1);
        suite
            .bench("first_child", || ())
            .source_location("b.rs", 100, 1);
    });
    let mut selection = Selection {
        sort: SortOrder::Source,
        ..Default::default()
    };
    let expected = vec![
        "root/early/first_child",
        "root/early/late_child",
        "root/middle",
        "root/late/early_child",
    ];
    assert_eq!(suite.list_selected(&selection), expected);
    assert_eq!(
        suite
            .test_selected(&selection)
            .unwrap()
            .cases
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    selection.reverse = true;
    assert_eq!(
        suite.list_selected(&selection),
        expected.into_iter().rev().collect::<Vec<_>>()
    );
}

#[test]
fn source_sort_infers_implicit_groups_and_preserves_equal_location_order() {
    let mut suite = Suite::new("root");
    suite.group("unknown", |suite| {
        suite.bench("case", || ());
    });
    suite.group("implicit", |suite| {
        suite.bench("late", || ()).source_location("b.rs", 10, 1);
        suite.group("nested", |suite| {
            suite.bench("first", || ()).source_location("a.rs", 1, 1);
            suite.bench("second", || ()).source_location("a.rs", 1, 1);
        });
    });
    suite.bench("middle", || ()).source_location("a.rs", 20, 1);
    let mut selection = Selection {
        sort: SortOrder::Source,
        ..Default::default()
    };
    let expected = vec![
        "root/implicit/nested/first",
        "root/implicit/nested/second",
        "root/implicit/late",
        "root/middle",
        "root/unknown/case",
    ];
    assert_eq!(suite.list_selected(&selection), expected);
    assert_eq!(
        suite
            .test_selected(&selection)
            .unwrap()
            .cases
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    // Filtering must not relocate the group based on only its remaining children.
    selection
        .exclude_exact
        .push("root/implicit/nested/first".into());
    selection
        .exclude_exact
        .push("root/implicit/nested/second".into());
    assert_eq!(
        suite.list_selected(&selection),
        ["root/implicit/late", "root/middle", "root/unknown/case"]
    );
    selection.reverse = true;
    assert_eq!(
        suite.list_selected(&selection),
        ["root/unknown/case", "root/middle", "root/implicit/late"]
    );
}

#[test]
fn global_builder_counter_override_preserves_other_units_and_is_lazy() {
    let calls = AtomicUsize::new(0);
    let counters = counters::InputCounters::new(&["items", "bytes"]).unwrap();
    let collected = counters.clone();
    let mut suite = Suite::new("global-counters");
    suite
        .bench_custom("dynamic", |n| {
            calls.fetch_add(n as usize, Ordering::SeqCst);
            collected.add("items", 2 * n);
            collected.add("bytes", 4 * n);
            Duration::from_secs(n)
        })
        .input_counters(counters);
    suite.sampling(Sampling {
        samples: Some(1),
        iterations: Some(3),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    suite
        .bench_custom("fixed", Duration::from_secs)
        .work_units("items", 1)
        .work_units("bytes", 9);
    suite.sampling(Sampling {
        samples: Some(1),
        iterations: Some(3),
        warmup: Some(Duration::ZERO),
        ..Default::default()
    });
    suite
        .override_work_units("items", 7)
        .override_work_units("chars", 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let run = suite.run("").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    for case in &run.cases {
        assert_eq!(case.contract["work.counter.items"], "7");
        assert_eq!(case.contract["work.counter.chars"], "0");
        assert!(!case.contract.contains_key("work.input.items"));
    }
    let dynamic = run
        .observations
        .iter()
        .find(|r| r.case.ends_with("dynamic"))
        .unwrap();
    assert_eq!(dynamic.work_totals["bytes"], "12");
    assert!(!dynamic.work_totals.contains_key("items"));
    let rates = report::throughput(&run).unwrap();
    for row in rates.iter().filter(|r| r.unit == "items") {
        assert_eq!(row.values, [7.0]);
    }
    assert_eq!(rates.iter().filter(|r| r.unit == "items").count(), 2);
    assert!(
        rates
            .iter()
            .filter(|r| r.unit == "chars")
            .all(|r| r.values == [0.0])
    );
}

#[test]
fn total_measurement_time_drives_flat_and_linear_collection() {
    for mode in [SamplingMode::Flat, SamplingMode::Linear] {
        let mut suite = Suite::new("total-target");
        suite.bench_custom("work", |n| Duration::from_nanos(n * 100));
        suite.sampling(Sampling {
            samples: Some(3),
            warmup: Some(Duration::ZERO),
            measurement_time: Some(Duration::from_nanos(900)),
            mode: Some(mode),
            ..Default::default()
        });
        let run = suite.run("").unwrap();
        assert_eq!(
            run.cases[0].contract["sampling.measurement_target_ns"],
            "900"
        );
        let ops: Vec<_> = run.observations.iter().map(|o| o.operations).collect();
        assert_eq!(
            ops,
            if mode == SamplingMode::Flat {
                vec![3, 3, 3]
            } else {
                vec![2, 4, 6]
            }
        );
        assert_eq!(
            run.observations
                .iter()
                .map(|o| o.value.as_ref().unwrap().parse::<u128>().unwrap())
                .sum::<u128>(),
            if mode == SamplingMode::Flat {
                900
            } else {
                1200
            }
        );
    }
}

#[test]
fn multiple_inclusive_patterns_union_before_exclusions_and_execute_once() {
    let calls = AtomicUsize::new(0);
    let mut suite = Suite::new("s");
    for name in ["parse/2", "parse/12", "тест/42", "other"] {
        suite.bench(name, || {
            calls.fetch_add(1, Ordering::SeqCst);
        });
    }
    let mut selection = Selection {
        patterns: vec!["parse".into(), "тест".into(), "parse/2".into()],
        exclude_exact: vec!["s/parse/12".into()],
        ..Default::default()
    };
    assert_eq!(suite.list_selected(&selection), ["s/parse/2", "s/тест/42"]);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let run = suite.test_selected(&selection).unwrap();
    assert_eq!(run.cases.len(), 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    selection.patterns = vec!["s/parse/2".into(), "s/тест/42".into()];
    selection.exact = true;
    assert_eq!(suite.list_selected(&selection), ["s/parse/2", "s/тест/42"]);
    selection.exact = false;
    selection.glob = true;
    selection.patterns = vec!["s/parse/?".into(), "s/тест/*".into()];
    assert_eq!(suite.list_selected(&selection), ["s/parse/2", "s/тест/42"]);
    let mut regexes = Selection::default()
        .with_regexes(&[r"^s/parse/\d+$", r"^s/тест/"])
        .unwrap();
    regexes.skip_regex("/12$").unwrap();
    assert_eq!(suite.list_selected(&regexes), ["s/parse/2", "s/тест/42"]);
    assert!(Selection::default().with_regexes(&["valid", "["]).is_err());
}

#[test]
fn groups_keep_shared_borrowed_inputs_lazy_and_select_by_full_identity() {
    use std::cell::Cell;
    // Neither cloning nor a 'static input is needed for shared-input benchmarks.
    struct Input {
        calls: Cell<usize>,
    }
    let input = Input {
        calls: Cell::new(0),
    };
    let borrowed = &input;
    let address = std::ptr::from_ref(borrowed);
    let mut suite = Suite::new("inputs");
    for group in ["first", "second"] {
        suite.group(group, |suite| {
            suite.bench("read/7", move || {
                assert_eq!(std::ptr::from_ref(borrowed), address);
                borrowed.calls.set(borrowed.calls.get() + 1);
            });
            suite.parameter("size", 7);
        });
    }
    suite.validate_registration().unwrap();
    assert_eq!(input.calls.get(), 0);
    assert_eq!(
        suite.list(""),
        ["inputs/first/read/7", "inputs/second/read/7"]
    );
    let selection = Selection {
        pattern: "second".into(),
        ..Default::default()
    };
    assert_eq!(suite.list_selected(&selection), ["inputs/second/read/7"]);
    assert_eq!(input.calls.get(), 0);
    let run = suite.test_selected(&selection).unwrap();
    assert_eq!(run.cases.len(), 1);
    assert_eq!(run.cases[0].id, "inputs/second/read/7");
    assert_eq!(input.calls.get(), 1);
    suite.group("second", |suite| {
        suite.bench("read/7", || panic!("validation must not execute work"));
    });
    let error = suite.validate_registration().unwrap_err().to_string();
    assert!(
        error.contains("duplicate benchmark ID: inputs/second/read/7"),
        "{error}"
    );
    assert_eq!(input.calls.get(), 1);
}
