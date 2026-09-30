use airbug_bench::{Config, Selection, Suite, profiling::Profiler};
use std::{cell::RefCell, path::Path, rc::Rc, time::Duration};

struct Hooks {
    events: Rc<RefCell<Vec<String>>>,
    start_fails: bool,
    stop_fails: bool,
}
impl Profiler for Hooks {
    fn start(&mut self, case: &str, directory: &Path) -> airbug_bench::Result<()> {
        assert!(directory.is_dir());
        self.events.borrow_mut().push(format!("start:{case}"));
        std::fs::write(directory.join("trace.txt"), case)?;
        if self.start_fails {
            return Err(std::io::Error::other("start failed").into());
        }
        Ok(())
    }
    fn stop(&mut self, case: &str, directory: &Path) -> airbug_bench::Result<()> {
        assert_eq!(std::fs::read_to_string(directory.join("trace.txt"))?, case);
        self.events.borrow_mut().push(format!("stop:{case}"));
        if self.stop_fails {
            panic!("stop panic");
        }
        Ok(())
    }
}
#[test]
fn timed_profile_uses_wall_time_and_brackets_only_selected_work() {
    let root = tempfile::tempdir().unwrap();
    let events = Rc::new(RefCell::new(vec![]));
    let mut suite = Suite::new("profile/../case");
    suite.profiler(Hooks {
        events: events.clone(),
        start_fails: false,
        stop_fails: false,
    });
    suite.bench_custom("selected", |_| {
        events.borrow_mut().push("work".into());
        std::thread::sleep(Duration::from_millis(1));
        Duration::ZERO // Reported measurement must not control the profiling deadline.
    });
    suite.bench("unselected", || panic!("wrong selection"));
    let selection = Selection {
        pattern: "selected".into(),
        exact: true,
        ..Default::default()
    };
    let selection = Selection {
        pattern: "profile/../case/selected".into(),
        ..selection
    };
    let output = root.path().join("session");
    let report = suite
        .profile_selected(&selection, Duration::from_millis(4), &output)
        .unwrap();
    assert!(report.complete);
    assert_eq!(report.cases.len(), 1);
    assert_eq!(report.cases[0].directory, "0000");
    assert_eq!(
        events.borrow().first().unwrap(),
        "start:profile/../case/selected"
    );
    assert_eq!(
        events.borrow().last().unwrap(),
        "stop:profile/../case/selected"
    );
    let stats = report.cases[0].stats.as_ref().unwrap();
    assert!(stats.elapsed_ns.parse::<u128>().unwrap() >= 4_000_000);
    assert!(stats.batches > 0);
    assert!(output.join("0000/trace.txt").is_file());
    assert!(!root.path().join("case").exists());
    let before = events.borrow().len();
    assert!(
        suite
            .profile_selected(&selection, Duration::from_millis(1), &output)
            .is_err()
    );
    assert_eq!(
        events.borrow().len(),
        before,
        "existing output fails before hooks/work"
    );
    events.borrow_mut().clear();
    suite
        .profile_selected(
            &selection,
            Duration::from_millis(1),
            &root.path().join("second"),
        )
        .unwrap();
    assert_eq!(
        events.borrow().first().unwrap(),
        "start:profile/../case/selected"
    );
    assert_eq!(
        events.borrow().last().unwrap(),
        "stop:profile/../case/selected"
    );
}
#[test]
fn hooks_stop_after_start_error_work_panic_and_stop_panic() {
    for (start_fails, work_panics, stop_fails) in [
        (true, false, false),
        (false, true, false),
        (false, true, true),
        (false, false, true),
    ] {
        let root = tempfile::tempdir().unwrap();
        let events = Rc::new(RefCell::new(vec![]));
        let mut suite = Suite::new("fail");
        suite.profiler(Hooks {
            events: events.clone(),
            start_fails,
            stop_fails,
        });
        suite.bench("case", || {
            events.borrow_mut().push("work".into());
            assert!(!work_panics, "work panic");
        });
        let output = root.path().join("session");
        assert!(
            suite
                .profile_selected(&Selection::default(), Duration::from_millis(1), &output)
                .is_err()
        );
        assert_eq!(events.borrow().first().unwrap(), "start:fail/case");
        assert_eq!(events.borrow().last().unwrap(), "stop:fail/case");
        if start_fails {
            assert_eq!(events.borrow().len(), 2);
        }
        let report: airbug_bench::profiling::ProfileReport =
            serde_json::from_slice(&std::fs::read(output.join("profile.json")).unwrap()).unwrap();
        assert!(!report.complete);
        assert!(report.cases[0].error.is_some());
    }
}
#[test]
fn ordinary_benchmark_does_not_invoke_profiler_hooks() {
    let events = Rc::new(RefCell::new(vec![]));
    let mut suite = Suite::new("normal");
    suite.profiler(Hooks {
        events: events.clone(),
        start_fails: true,
        stop_fails: true,
    });
    suite.bench("case", || ());
    suite.config(Config {
        samples: 1,
        warmup: Duration::ZERO,
        sample_time: Duration::from_nanos(1),
        max_iterations: 1,
    });
    suite.run("").unwrap();
    assert!(events.borrow().is_empty());
}
