use rbench::analysis::{compare, median_interval, Decision};
use rbench::*;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
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
    assert!(rbench::report::markdown(&r).unwrap().contains("no adapter"));
}
#[test]
fn report_escapes_untrusted_names() {
    let html = rbench::report::html("<script>alert(1)</script>");
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
    let a = rbench::alloc::TrackingAllocator::new(FailRealloc);
    unsafe {
        let l = Layout::from_size_align(32, 8).unwrap();
        let p = a.alloc(l);
        assert!(!p.is_null());
        assert!(a.realloc(p, l, 64).is_null());
        assert_eq!(a.snapshot().live_bytes, 32);
        assert_eq!(a.snapshot().reallocations, 0);
        a.dealloc(p, l);
        assert_eq!(a.snapshot().live_bytes, 0);
    }
}
#[test]
fn allocator_zeroed_shrink_and_peak() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let a = rbench::alloc::TrackingAllocator::new(System);
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
    assert!(s
        .run("")
        .unwrap_err()
        .to_string()
        .contains("pre-validation"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[test]
fn matrix_rejects_bad_axes_without_registration() {
    let mut s = Suite::new("bad");
    assert!(s
        .matrix("x", &[("n", &["1", "1"])], |_, _, _| panic!(
            "must not register"
        ))
        .is_err());
    assert!(s
        .matrix("x", &[("n", &[])], |_, _, _| panic!("must not register"))
        .is_err());
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
    .work_units("bytes", 0);
    assert!(s.run("").is_err());
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
    let plot = report::plot("<script>", &[(0., 1.), (1., 2.)]);
    assert!(!plot.contains("<script>"));
    assert!(plot.contains("&lt;script&gt;"));
}

#[test]
fn lifecycle_helpers_join_and_cancel() {
    use rbench::workloads::*;
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
    assert!(parallel(4, |i| {
        if i == 2 {
            panic!("controlled worker failure")
        }
        i
    })
    .is_err());
    assert!(parallel(0, |_| 0).is_err());
    let cancellation = Cancellation::default();
    let p = pipeline((0..100).collect(), 2, &cancellation, |i| i * 2).unwrap();
    assert_eq!(p.outputs, (0..100).map(|i| i * 2).collect::<Vec<_>>());
    assert_eq!(p.latency_ns.len(), 100);
    assert_eq!(p.processed, 100);
    assert!(pipeline(vec![1], 0, &cancellation, |i| i).is_err());
    let c = cancellation.clone();
    assert!(pipeline((0..100).collect(), 1, &cancellation, move |i| {
        if i == 3 {
            c.cancel();
        }
        i
    })
    .is_err());
}
#[test]
fn async_wakeup_and_cold_first_invocation() {
    use rbench::workloads::*;
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
    assert!(diagnostics::diagnose(&drift)
        .unwrap()
        .iter()
        .all(|d| d.flags.iter().any(|s| s.contains("chronological"))));
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
#[rbench::bench]
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
    assert!(report::throughput(&plain.finish().unwrap()).unwrap().is_empty());
}
