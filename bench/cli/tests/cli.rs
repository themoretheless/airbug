static RUNNER_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
use std::{fs, path::Path, process::Command};
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-airbug-bench"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn help_and_cargo_invocation() {
    assert!(cli(&["--help"]).status.success());
    assert!(cli(&["airbug-bench", "doctor"]).status.success());
}
#[test]
fn check_filter_narrows_cases() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    {
        let mut rec = airbug_bench::Recorder::new();
        for (id, val) in [("fast", 10u128), ("slow", 100u128)] {
            rec.case(airbug_bench::Case {
                id: id.into(),
                contract: Default::default(),
                metrics: vec![airbug_bench::Metric::duration("wall", "test", "total")],
            })
            .unwrap();
            rec.observe(id, "wall", val).unwrap();
        }
        rec.finish().unwrap().save_new(&run).unwrap();
    }
    let p = run.to_str().unwrap();
    let code = |args: &[&str]| {
        let mut all = vec!["check", p, "--metric", "wall", "--max", "50"];
        all.extend_from_slice(args);
        cli(&all).status.code()
    };
    assert_eq!(code(&[]), Some(1)); // slow (100) > 50
    assert_eq!(code(&["--filter", "fast"]), Some(0)); // only fast (10)
    assert_eq!(code(&["--filter", "slow"]), Some(1)); // only slow (100)
    assert_eq!(code(&["--filter", "nope"]), Some(2)); // no case matched -> metric not found
}
#[test]
fn notes_json_output() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    simple_run(&run);
    let store = t.path().to_str().unwrap();
    let p = run.to_str().unwrap();
    assert!(
        cli(&["--store", store, "note", p, "hello note"])
            .status
            .success()
    );
    let o = cli(&["--store", store, "notes", p, "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let notes = v.as_array().unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["text"], "hello note");
    assert_eq!(notes[0]["run_sha256"].as_str().unwrap().len(), 64);
}
#[test]
fn context_and_trend_json_outputs() {
    let t = tempfile::tempdir().unwrap();
    let store = t.path();
    let a = simple_run(&store.join("a"));
    // context --json: identical runs -> no differences.
    let o = cli(&[
        "context",
        store.join("a").to_str().unwrap(),
        store.join("a").to_str().unwrap(),
        "--json",
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
    // A differing environment key surfaces exactly one diff.
    let mut b = a.clone();
    b.environment.insert("os".into(), "other-os".into());
    b.save_new(store.join("b")).unwrap();
    let o = cli(&[
        "context",
        store.join("a").to_str().unwrap(),
        store.join("b").to_str().unwrap(),
        "--json",
    ]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let diffs = v.as_array().unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0]["key"], "environment.os");
    assert_eq!(diffs[0]["b"], "other-os");
    // trend --json: one complete run with case "=unsafe,case"/metric "wall" value 42.
    let o = cli(&[
        "--store",
        store.to_str().unwrap(),
        "trend",
        "--case",
        "=unsafe,case",
        "--metric",
        "wall",
        "--json",
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let points = v.as_array().unwrap();
    assert!(!points.is_empty());
    assert_eq!(points[0]["unit"], "ns");
    assert!((points[0]["median"].as_f64().unwrap() - 42.0).abs() < 1e-6);
}
#[test]
fn history_filters_by_status_and_limit() {
    let t = tempfile::tempdir().unwrap();
    let store = t.path();
    let base = simple_run(&store.join("a"));
    simple_run(&store.join("b"));
    let mut failed = base.clone();
    failed.status = airbug_bench::Status::Failed;
    failed.save_new(store.join("c")).unwrap();
    let s = store.to_str().unwrap();
    let count = |args: &[&str]| -> usize {
        let mut all = vec!["--store", s, "history", "--json"];
        all.extend_from_slice(args);
        let o = cli(&all);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        serde_json::from_slice::<serde_json::Value>(&o.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len()
    };
    assert_eq!(count(&[]), 3);
    assert_eq!(count(&["--status", "complete"]), 2);
    assert_eq!(count(&["--status", "FAILED"]), 1); // case-insensitive
    assert_eq!(count(&["--status", "cancelled"]), 0);
    assert_eq!(count(&["--limit", "2"]), 2);
    assert_eq!(count(&["--limit", "0"]), 0);
}
#[test]
fn throughput_derives_and_gates_work_units() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    {
        let mut rec = airbug_bench::Recorder::new();
        rec.case(airbug_bench::Case {
            id: "scan".into(),
            contract: std::collections::BTreeMap::from([
                ("work.unit".to_string(), "elements".to_string()),
                ("work.count".to_string(), "10".to_string()),
            ]),
            metrics: vec![airbug_bench::Metric::duration(
                "wall",
                "test",
                "batch_total",
            )],
        })
        .unwrap();
        // ops=1, 1e9 ns batch -> 10 elements/s.
        rec.observe("scan", "wall", 1_000_000_000).unwrap();
        rec.finish().unwrap().save_new(&run).unwrap();
    }
    let path = run.to_str().unwrap();
    let o = cli(&["throughput", path, "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let rows = v.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["case"], "scan");
    assert_eq!(rows[0]["unit"], "elements");
    assert!((rows[0]["median"].as_f64().unwrap() - 10.0).abs() < 1e-6);
    assert_eq!(rows[0]["samples"], 1);
    // Gating: a floor above 10 fails, below passes; a ceiling below 10 fails.
    assert_eq!(
        cli(&["throughput", path, "--min", "20"]).status.code(),
        Some(1)
    );
    assert_eq!(
        cli(&["throughput", path, "--min", "5"]).status.code(),
        Some(0)
    );
    assert_eq!(
        cli(&["throughput", path, "--max", "5"]).status.code(),
        Some(1)
    );
    // Misuse: inverted range.
    assert_eq!(
        cli(&["throughput", path, "--min", "100", "--max", "10"])
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn doctor_text_and_json_report_capabilities() {
    let o = cli(&["doctor"]);
    assert!(o.status.success());
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(text.contains("bench "));
    assert!(text.contains("Statistics: independent process units required"));
    let o = cli(&["doctor", "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["airbug-bench"], env!("CARGO_PKG_VERSION"));
    assert!(v["os"].is_string() && v["arch"].is_string());
    assert_eq!(v["statistics"], "independent process units required");
    #[cfg(unix)]
    assert_eq!(v["process_tree_cleanup"], "Unix process groups");
}
#[test]
fn check_enforces_min_and_max_bounds() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    simple_run(&run); // one observation, metric "wall", value 42, statistic "total"
    let path = run.to_str().unwrap();
    let check = |args: &[&str]| {
        let mut all = vec!["check", path, "--metric", "wall"];
        all.extend_from_slice(args);
        cli(&all).status.code()
    };
    // Upper bound (existing behaviour).
    assert_eq!(check(&["--max", "42"]), Some(0));
    assert_eq!(check(&["--max", "41"]), Some(1));
    // Lower bound (new).
    assert_eq!(check(&["--min", "42"]), Some(0));
    assert_eq!(check(&["--min", "43"]), Some(1));
    // Range: 42 inside passes, outside fails.
    assert_eq!(check(&["--min", "10", "--max", "100"]), Some(0));
    assert_eq!(check(&["--min", "10", "--max", "20"]), Some(1));
    // Misuse: no bound, and inverted range.
    assert_eq!(check(&[]), Some(2));
    assert_eq!(check(&["--min", "100", "--max", "10"]), Some(2));
}
#[test]
fn list_text_ids_and_json_metrics_with_counts() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    simple_run(&run);
    let path = run.to_str().unwrap();
    // Text mode is unchanged: one case id per line.
    let o = cli(&["list", path]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8(o.stdout).unwrap().trim(), "=unsafe,case");
    // JSON mode reports metrics and observation counts.
    let o = cli(&["list", path, "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let rows = v.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["case"], "=unsafe,case");
    assert_eq!(rows[0]["metrics"][0]["id"], "wall");
    assert_eq!(rows[0]["metrics"][0]["unit"], "ns");
    assert_eq!(rows[0]["observations"], 1);
    assert_eq!(rows[0]["available"], 1);
    assert_eq!(rows[0]["processes"], 1);
    // Filter narrows both modes; a non-match yields an empty JSON array.
    let o = cli(&["list", path, "--json", "--filter", "nomatch"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
}
#[test]
fn completions_generate_per_shell_and_reject_unknown() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let o = cli(&["completions", shell]);
        assert!(
            o.status.success(),
            "{shell}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let script = String::from_utf8(o.stdout).unwrap();
        assert!(!script.is_empty(), "{shell}: empty script");
        // Every generator embeds the completed binary name and the real subcommands.
        assert!(
            script.contains("cargo-airbug-bench"),
            "{shell}: missing binary name"
        );
        assert!(
            script.contains("doctor") && script.contains("compare"),
            "{shell}: missing subcommands"
        );
    }
    // The cargo-subcommand invocation form produces the same output.
    let o = cli(&["airbug-bench", "completions", "zsh"]);
    assert!(o.status.success());
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .starts_with("#compdef cargo-airbug-bench")
    );
    // Generation is read-only and never touches the shell configuration.
    assert!(!cli(&["completions", "tcsh"]).status.success());
}
#[cfg(unix)]
#[test]
fn process_success_failure_timeout_and_immutable_output() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("ok");
    let p = p.to_str().unwrap();
    let o = cli(&[
        "run",
        "--program",
        "/bin/echo",
        "--repetitions",
        "2",
        "--output",
        p,
        "--",
        "hello world",
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r = airbug_bench::Run::load(p).unwrap();
    assert_eq!(r.observations.len(), 2);
    for name in ["report.html", "report.json", "report.md", "progress.json"] {
        assert!(Path::new(p).join(name).is_file(), "{name}");
    }
    let progress: serde_json::Value =
        serde_json::from_slice(&fs::read(Path::new(p).join("progress.json")).unwrap()).unwrap();
    assert_eq!(progress["completed"], 2);
    assert!(
        fs::read_to_string(Path::new(p).join("logs/0-candidate.stdout"))
            .unwrap()
            .contains("hello world")
    );
    assert!(
        !cli(&["run", "--program", "/bin/echo", "--output", p])
            .status
            .success()
    );
    let bad = t.path().join("bad");
    assert!(
        !cli(&[
            "run",
            "--program",
            "/usr/bin/false",
            "--output",
            bad.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert_eq!(
        airbug_bench::Run::load(&bad).unwrap().status,
        airbug_bench::Status::Failed
    );
    let timeout = t.path().join("timeout");
    assert!(
        !cli(&[
            "run",
            "--program",
            "/bin/sleep",
            "--timeout-ms",
            "20",
            "--output",
            timeout.to_str().unwrap(),
            "--",
            "2"
        ])
        .status
        .success()
    );
    assert!(
        fs::read_to_string(timeout.join("run.json"))
            .unwrap()
            .contains("timed out")
    );
}
#[cfg(unix)]
#[test]
fn malformed_protocol_is_not_success() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let t = tempfile::tempdir().unwrap();
    let out = t.path().join("bad");
    let o = cli(&[
        "run",
        "--program",
        "/bin/echo",
        "--protocol",
        "--output",
        out.to_str().unwrap(),
        "--",
        "BENCH_RESULT={}",
    ]);
    assert!(!o.status.success());
    assert_eq!(
        airbug_bench::Run::load(out).unwrap().status,
        airbug_bench::Status::Failed
    );
}
#[test]
fn unknown_plan_fields_fail() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("plan.json");
    fs::write(&p, r#"{"candidate":{"path":"/bin/echo"},"repititions":1}"#).unwrap();
    let o = cli(&[
        "run",
        "--plan",
        p.to_str().unwrap(),
        "--output",
        t.path().join("out").to_str().unwrap(),
    ]);
    assert!(!o.status.success());
    assert!(!t.path().join("out").exists());
}
fn forma_fixture(path: &Path) {
    fs::create_dir(path).unwrap();
    fs::write(path.join("metadata.json"),r#"{"platform":"darwin","osRelease":"25","arch":"arm64","cpu":"fixture","rust":"fixture","logicalCpus":10,"systemRamBytes":1024,"repeats":1,"frames":120}"#).unwrap();
    for s in ["image", "text", "nested"] {
        fs::write(path.join(format!("{s}.ui")), s).unwrap();
        fs::write(path.join(format!("{s}.template.ui")), s).unwrap();
    }
    let specs = [
        ("image", "gpu", "animation", 800, 400),
        ("image", "gpu", "animation", 1920, 1080),
        ("image", "gpu", "animation", 3840, 2160),
        ("image", "gpu", "resize", 3840, 2160),
        ("text", "gpu", "animation", 1920, 1080),
        ("nested", "gpu", "forced", 1920, 1080),
        ("image", "gpu", "forced", 3840, 2160),
        ("image", "gpu", "idle", 800, 400),
        ("image", "cpu", "animation", 800, 400),
        ("image", "cpu", "animation", 3840, 2160),
        ("image", "cpu", "resize", 3840, 2160),
    ];
    let rows:Vec<_>=specs.into_iter().map(|(s,b,m,w,h)|serde_json::json!({"scene":s,"backend":b,"mode":m,"width":w,"height":h,"scale":2,"repetition":1,"frames":if m=="idle"{0}else{120},"adapter":"fixture","render_throughput_fps":100,"completed_ms":{"p95":4.25},"gpu_pass":null})).collect();
    fs::write(
        path.join("results.json"),
        serde_json::to_vec(&rows).unwrap(),
    )
    .unwrap();
}
#[test]
fn forma_import_preserves_scope_and_rejects_missing_rows() {
    let t = tempfile::tempdir().unwrap();
    let src = t.path().join("forma");
    forma_fixture(&src);
    let out = t.path().join("import");
    let o = cli(&[
        "import-forma",
        src.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let run = airbug_bench::Run::load(out).unwrap();
    assert_eq!(run.cases.len(), 11);
    assert!(run.cases.iter().all(|c| {
        c.metrics
            .iter()
            .any(|m| m.id == "gpu.pass.mean" && m.phase == "gpu_diagnostic")
    }));
    assert!(
        run.observations
            .iter()
            .filter(|o| o.metric == "gpu.pass.mean")
            .all(|o| o.value.is_none())
    );
    let p = src.join("results.json");
    let mut rows: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    rows.pop();
    fs::write(p, serde_json::to_vec(&rows).unwrap()).unwrap();
    assert!(
        !cli(&[
            "import-forma",
            src.to_str().unwrap(),
            "-o",
            t.path().join("bad").to_str().unwrap()
        ])
        .status
        .success()
    );
}

#[test]
fn init_discovery_preserves_manifest_and_refuses_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("Cargo.toml");
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "").unwrap();
    fs::write(&manifest,"# keep this comment\n[package]\nname='init-fixture'\nversion='0.1.0'\nedition='2021'\n[workspace]\n").unwrap();
    let o = cli(&["init", "--manifest-path", manifest.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let content = fs::read_to_string(&manifest).unwrap();
    assert!(content.contains("# keep this comment"));
    assert!(content.contains("harness = false"));
    assert!(String::from_utf8_lossy(&o.stdout).contains("Run: cargo bench"));
    let source = fs::read_to_string(dir.path().join("benches/bench.rs")).unwrap();
    assert!(source.contains("#[airbug_bench::suite]"));
    assert!(source.contains("#[bench(args"));
    let check = std::process::Command::new("cargo")
        .args(["check", "--offline", "--bench", "bench", "--manifest-path"])
        .arg(&manifest)
        .env(
            "CARGO_TARGET_DIR",
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/init-smoke"),
        )
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );

    let o = cli(&[
        "discover",
        "--manifest-path",
        manifest.to_str().unwrap(),
        "--offline",
        "--json",
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v[0]["registered"], true);
    assert!(
        !cli(&["init", "--manifest-path", manifest.to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(content, fs::read_to_string(&manifest).unwrap());
}
#[test]
fn retention_protects_case_baseline_artifacts_and_checks_hashes() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let root = tempfile::tempdir().unwrap();
    let protected = root.path().join("protected");
    let unused = root.path().join("unused");
    let run = simple_run(&protected);
    simple_run(&unused);
    for directory in [&protected, &unused] {
        for file in ["plan.json", "schedule.json", "status-final.json"] {
            fs::write(directory.join(file), "{}").unwrap();
        }
    }
    let store = airbug_bench::baseline::Store::new(root.path());
    store
        .save("main", &protected, airbug_bench::baseline::SaveMode::Create)
        .unwrap();
    let reference: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("baselines/main.json")).unwrap())
            .unwrap();
    fs::write(
        root.path().join("baselines/main.cases.json"),
        serde_json::to_vec(&serde_json::json!({
            "version": 1, "cases": {run.cases[0].id.clone(): reference}
        }))
        .unwrap(),
    )
    .unwrap();
    let result = cli(&[
        "--store",
        root.path().to_str().unwrap(),
        "retention",
        "--keep",
        "0",
        "--apply",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(protected.join("run.json").exists());
    assert!(!unused.exists());
    fs::write(protected.join("run.json"), "{}").unwrap();
    let result = cli(&[
        "--store",
        root.path().to_str().unwrap(),
        "retention",
        "--keep",
        "0",
        "--apply",
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("integrity failed"));
    assert!(protected.exists());
}

#[test]
fn runner_save_updates_cargo_case_baseline_without_shadow_reference() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let root = tempfile::tempdir().unwrap();
    let a = simple_run(&root.path().join("a"));
    let mut b = a.clone();
    b.id = "other-run".into();
    b.cases[0].id = "other-case".into();
    b.observations[0].case = "other-case".into();
    let store = airbug_bench::baseline::Store::new(root.path());
    store
        .save_cases("main", &a, airbug_bench::baseline::SaveMode::Create)
        .unwrap();
    store
        .save_cases("main", &b, airbug_bench::baseline::SaveMode::Replace)
        .unwrap();
    let mut replacement = a.clone();
    replacement.observations[0].value = Some("777".into());
    let source = root.path().join("replacement");
    replacement.save_new(&source).unwrap();
    let save = |source: &str, flag: &str| {
        cli(&[
            "--store",
            root.path().to_str().unwrap(),
            "baseline",
            "save",
            "main",
            source,
            flag,
        ])
    };
    assert!(save("@missing", "--retain").status.success());
    assert_eq!(
        store.load_cases("main").unwrap()[&a.cases[0].id].observations[0]
            .value
            .as_deref(),
        Some("42")
    );
    let result = save(source.to_str().unwrap(), "--replace");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let cases = store.load_cases("main").unwrap();
    assert_eq!(cases.len(), 2);
    assert_eq!(
        cases[&a.cases[0].id].observations[0].value.as_deref(),
        Some("777")
    );
    assert_eq!(
        serde_json::to_value(&cases["other-case"]).unwrap(),
        serde_json::to_value(b).unwrap()
    );
    assert!(!root.path().join("baselines/main.json").exists());
    assert!(
        store
            .save("main", &source, airbug_bench::baseline::SaveMode::Replace)
            .is_err()
    );
}

#[test]
fn case_alias_reports_preserve_updates_and_match_cases_across_runs() {
    let root = tempfile::tempdir().unwrap();
    let mut a = simple_run(&root.path().join("original"));
    let store = airbug_bench::baseline::Store::new(root.path());
    store
        .save(
            "main",
            &root.path().join("original"),
            airbug_bench::baseline::SaveMode::Create,
        )
        .unwrap();
    a.observations[0].value = Some("999".into());
    store
        .save_cases("main", &a, airbug_bench::baseline::SaveMode::Replace)
        .unwrap();
    store
        .save_cases("comparison", &a, airbug_bench::baseline::SaveMode::Create)
        .unwrap();
    assert!(
        cli(&["--store", root.path().to_str().unwrap(), "list", "@main"])
            .status
            .success()
    );
    let mut b = a.clone();
    b.id = "different-run".into();
    b.cases[0].id = "second".into();
    b.observations[0].case = "second".into();
    b.environment
        .insert("machine".into(), "second-machine".into());
    store
        .save_cases("main", &b, airbug_bench::baseline::SaveMode::Replace)
        .unwrap();
    store
        .save_cases("comparison", &b, airbug_bench::baseline::SaveMode::Replace)
        .unwrap();
    let output = root.path().join("report.json");
    let result = cli(&[
        "--store",
        root.path().to_str().unwrap(),
        "report",
        "@main",
        "--baseline",
        "@comparison",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    let entries = document["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e["baseline_sha256"].is_string()));
    assert!(
        entries
            .iter()
            .all(|e| e["run"]["observations"][0]["value"] == "999")
    );
    assert!(
        entries
            .iter()
            .any(|e| e["run"]["environment"]["machine"] == "second-machine")
    );
    let result = cli(&["--store", root.path().to_str().unwrap(), "list", "@main"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("multiple source runs"));
}

#[test]
fn baseline_listing_understands_case_manifests_and_hides_legacy_duplicate() {
    let root = tempfile::tempdir().unwrap();
    let run = simple_run(&root.path().join("run"));
    let store = airbug_bench::baseline::Store::new(root.path());
    store
        .save(
            "main",
            &root.path().join("run"),
            airbug_bench::baseline::SaveMode::Create,
        )
        .unwrap();
    store
        .save_cases("main", &run, airbug_bench::baseline::SaveMode::Replace)
        .unwrap();
    let result = cli(&["--store", root.path().to_str().unwrap(), "baseline", "list"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8(result.stdout).unwrap();
    assert_eq!(text.matches("@main").count(), 1);
    assert!(text.contains("cases"), "{text}");
}

#[test]
fn baseline_save_modes_resolve_only_needed_sources() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    simple_run(&a);
    simple_run(&b);
    let original = fs::read(a.join("run.json")).unwrap();
    let store = dir.path().to_str().unwrap();
    let save = |source: &str, flags: &[&str]| {
        let mut args = vec!["--store", store, "baseline", "save", "main", source];
        args.extend_from_slice(flags);
        cli(&args)
    };
    assert!(save(a.to_str().unwrap(), &[]).status.success());
    let pointer = dir.path().join("baselines/main.json");
    let initial = fs::read(&pointer).unwrap();
    let retained = save("@missing", &["--retain"]);
    assert!(
        retained.status.success(),
        "{}",
        String::from_utf8_lossy(&retained.stderr)
    );
    assert!(String::from_utf8_lossy(&retained.stdout).contains("Retained baseline @main"));
    assert_eq!(fs::read(&pointer).unwrap(), initial);
    assert!(
        !save(b.to_str().unwrap(), &["--retain", "--replace"])
            .status
            .success()
    );
    assert_eq!(fs::read(&pointer).unwrap(), initial);
    assert!(!save("@missing", &["--replace"]).status.success());
    assert_eq!(fs::read(&pointer).unwrap(), initial);
    assert!(save(b.to_str().unwrap(), &["--replace"]).status.success());
    let reference = airbug_bench::baseline::Store::new(dir.path())
        .require("main")
        .unwrap();
    assert_eq!(reference.run, fs::canonicalize(b.join("run.json")).unwrap());
    assert_eq!(fs::read(a.join("run.json")).unwrap(), original);
    fs::write(b.join("run.json"), "{}").unwrap();
    assert!(!save(a.to_str().unwrap(), &["--retain"]).status.success());
    assert!(save(a.to_str().unwrap(), &["--replace"]).status.success());
}

#[test]
fn baseline_alias_checks_integrity_and_gate_exit_codes() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("run");
    let store = dir.path().join("store");
    let mut recorder = airbug_bench::Recorder::new();
    recorder
        .case(airbug_bench::Case {
            id: "fixture".into(),
            contract: Default::default(),
            metrics: vec![airbug_bench::Metric::duration("wall", "test", "total")],
        })
        .unwrap();
    recorder.observe("fixture", "wall", 42).unwrap();
    recorder.finish().unwrap().save_new(&run).unwrap();
    let args = [
        "--store",
        store.to_str().unwrap(),
        "baseline",
        "save",
        "main",
        run.to_str().unwrap(),
    ];
    assert!(cli(&args).status.success());
    assert!(!cli(&args).status.success());
    assert!(
        cli(&["--store", store.to_str().unwrap(), "report", "@main"])
            .status
            .success()
    );
    let config = dir.path().join("budget.json");
    fs::write(
        &config,
        r#"{"budgets":[{"case":"fixture","metric":"wall","unit":"ns","max":41}]}"#,
    )
    .unwrap();
    assert_eq!(
        cli(&[
            "--store",
            store.to_str().unwrap(),
            "gate",
            "@main",
            "--config",
            config.to_str().unwrap()
        ])
        .status
        .code(),
        Some(1)
    );
    let path = run.join("run.json");
    let mut content = fs::read_to_string(&path).unwrap();
    content.push('\n');
    fs::write(path, content).unwrap();
    assert!(
        !cli(&["--store", store.to_str().unwrap(), "report", "@main"])
            .status
            .success()
    );
    assert!(
        !cli(&["--store", store.to_str().unwrap(), "report", "@../run"])
            .status
            .success()
    );
}

fn simple_run(path: &Path) -> airbug_bench::Run {
    let mut rec = airbug_bench::Recorder::new();
    rec.case(airbug_bench::Case {
        id: "=unsafe,case".into(),
        contract: Default::default(),
        metrics: vec![airbug_bench::Metric::duration("wall", "test", "total")],
    })
    .unwrap();
    rec.observe("=unsafe,case", "wall", 42).unwrap();
    let run = rec.finish().unwrap();
    run.save_new(path).unwrap();
    run
}
#[test]
fn history_last_notes_export_and_bundle_roundtrip() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("a");
    simple_run(&run);
    let store = t.path().to_str().unwrap();
    assert!(cli(&["--store", store, "report", "last"]).status.success());
    let original = fs::read(run.join("run.json")).unwrap();
    assert!(
        cli(&["--store", store, "note", "last", "portable note"])
            .status
            .success()
    );
    assert_eq!(original, fs::read(run.join("run.json")).unwrap());
    let report = cli(&["--store", store, "report", "last"]);
    assert!(
        String::from_utf8(report.stdout)
            .unwrap()
            .contains("portable note")
    );
    let csv = t.path().join("out.csv");
    assert!(
        cli(&[
            "--store",
            store,
            "export",
            "last",
            "--format",
            "csv",
            "-o",
            csv.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(
        fs::read_to_string(csv)
            .unwrap()
            .contains("\"'=unsafe,case\"")
    );
    let jsonl = t.path().join("out.jsonl");
    assert!(
        cli(&[
            "--store",
            store,
            "export",
            "last",
            "--format",
            "jsonl",
            "-o",
            jsonl.to_str().unwrap()
        ])
        .status
        .success()
    );
    let value: serde_json::Value =
        serde_json::from_str(fs::read_to_string(jsonl).unwrap().trim()).unwrap();
    assert_eq!(value["observation"]["value"], "42");
    let bundle = t.path().join("run.bundle.json");
    assert!(
        cli(&[
            "--store",
            store,
            "bundle",
            "last",
            "-o",
            bundle.to_str().unwrap()
        ])
        .status
        .success()
    );
    let restored = t.path().join("restored");
    assert!(
        cli(&[
            "unpack",
            bundle.to_str().unwrap(),
            "-o",
            restored.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert_eq!(original, fs::read(restored.join("run.json")).unwrap());
    let other = t.path().join("empty-store");
    let o = cli(&[
        "--store",
        other.to_str().unwrap(),
        "notes",
        restored.to_str().unwrap(),
    ]);
    assert!(o.status.success());
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .contains("portable note")
    );
    let h = cli(&["--store", store, "history", "--json"]);
    let h: serde_json::Value = serde_json::from_slice(&h.stdout).unwrap();
    assert_eq!(h.as_array().unwrap().len(), 2);
}
#[test]
fn bundle_rejects_tampering_before_creating_output() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    simple_run(&run);
    let bundle = t.path().join("bundle.json");
    assert!(
        cli(&[
            "bundle",
            run.to_str().unwrap(),
            "-o",
            bundle.to_str().unwrap()
        ])
        .status
        .success()
    );
    let mut v: serde_json::Value = serde_json::from_slice(&fs::read(&bundle).unwrap()).unwrap();
    v["files"][0]["name"] = "../escaped".into();
    fs::write(&bundle, serde_json::to_vec(&v).unwrap()).unwrap();
    let out = t.path().join("out");
    assert!(
        !cli(&[
            "unpack",
            bundle.to_str().unwrap(),
            "-o",
            out.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(!out.exists());
}
#[cfg(unix)]
#[test]
fn dry_run_and_preflight_never_execute_or_create_output() {
    let t = tempfile::tempdir().unwrap();
    let output = t.path().join("new");
    let o = cli(&[
        "run",
        "--program",
        "/usr/bin/false",
        "--dry-run",
        "-o",
        output.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!output.exists());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v["hashes"].as_object().unwrap().len() == 1);
    let missing = cli(&[
        "run",
        "--program",
        "/not/a/real/binary",
        "--dry-run",
        "-o",
        output.to_str().unwrap(),
    ]);
    assert!(!missing.status.success());
    assert!(!output.exists());
}
#[test]
fn uncertainty_policy_does_not_allow_missing_capabilities() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    let mut r = simple_run(&run);
    for o in &mut r.observations {
        o.value = None;
        o.availability = airbug_bench::Availability::Unsupported("missing".into());
    }
    let bad = t.path().join("bad");
    r.save_new(&bad).unwrap();
    let o = cli(&[
        "compare",
        run.to_str().unwrap(),
        bad.to_str().unwrap(),
        "--check",
        "--uncertainty",
        "record",
    ]);
    assert_eq!(o.status.code(), Some(2));
    let o = cli(&[
        "compare",
        run.to_str().unwrap(),
        run.to_str().unwrap(),
        "--check",
        "--uncertainty",
        "warn",
    ]);
    assert!(o.status.success());
    assert!(
        String::from_utf8(o.stderr)
            .unwrap()
            .contains("inconclusive")
    );
}
#[test]
fn context_trend_and_ci_template() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    simple_run(&run);
    let o = cli(&["context", run.to_str().unwrap(), run.to_str().unwrap()]);
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .contains("No differences")
    );
    let out = t.path().join("trend.html");
    let o = cli(&[
        "--store",
        t.path().to_str().unwrap(),
        "trend",
        "--case",
        "=unsafe,case",
        "--metric",
        "wall",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(o.status.success());
    assert!(fs::read_to_string(out).unwrap().contains("<svg"));
    let ci = t.path().join("ci.yml");
    assert!(cli(&["ci", "-o", ci.to_str().unwrap()]).status.success());
    assert!(!cli(&["ci", "-o", ci.to_str().unwrap()]).status.success());
    let text = fs::read_to_string(ci).unwrap();
    assert!(text.contains("workflow_dispatch"));
    assert!(text.contains("include-hidden-files: true"));
}

#[cfg(unix)]
#[test]
fn privacy_multivariant_resume_and_retention() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let t = tempfile::tempdir().unwrap();
    let plan = t.path().join("plan.json");
    let run = t.path().join("secret-run");
    let secret = "BENCH-sensitive-\"split\\secret-194812";
    fs::write(&plan,serde_json::to_vec(&serde_json::json!({"candidate":{"path":"/bin/sh","args":["-c","printf '%s' \"$BENCH_TEST_SECRET\"; printf '%s' \"$BENCH_TEST_SECRET\" >&2; test -z \"$BENCH_UNLISTED\""],"cwd":null},"repetitions":1,"privacy":{"allow_env":[],"secret_env":["BENCH_TEST_SECRET"]}})).unwrap()).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_cargo-airbug-bench"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "-o",
            run.to_str().unwrap(),
        ])
        .env("BENCH_TEST_SECRET", secret)
        .env("BENCH_UNLISTED", "must-not-inherit")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    for p in [
        run.join("plan.json"),
        run.join("run.json"),
        run.join("logs/0-candidate.stdout"),
        run.join("logs/0-candidate.stderr"),
    ] {
        let data = fs::read_to_string(p).unwrap();
        assert!(!data.contains("sensitive"));
    }
    assert_eq!(
        fs::read_to_string(run.join("logs/0-candidate.stdout")).unwrap(),
        "[REDACTED]"
    );
    let bundle = t.path().join("export.json");
    assert!(
        cli(&[
            "bundle",
            run.to_str().unwrap(),
            "-o",
            bundle.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(!fs::read_to_string(bundle).unwrap().contains("sensitive"));
    let multi = t.path().join("multi");
    let p = serde_json::json!({"candidate":{"path":"/usr/bin/true","cwd":null},"baseline":{"path":"/usr/bin/true","cwd":null},"variants":{"third":{"path":"/usr/bin/true","cwd":null}},"repetitions":6});
    fs::write(&plan, serde_json::to_vec(&p).unwrap()).unwrap();
    let o = cli(&[
        "run",
        "--plan",
        plan.to_str().unwrap(),
        "-o",
        multi.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let schedule: serde_json::Value =
        serde_json::from_slice(&fs::read(multi.join("schedule.json")).unwrap()).unwrap();
    let mut positions = std::collections::BTreeMap::new();
    for (i, e) in schedule.as_array().unwrap().iter().enumerate() {
        *positions
            .entry((e["variant"].as_str().unwrap(), i % 3))
            .or_insert(0) += 1;
    }
    assert!(positions.values().all(|n| *n == 2));
    // An interrupted whole-process workload can continue after an external stop condition clears.
    let marker = t.path().join("continue");
    let partial = t.path().join("partial");
    let resumed = t.path().join("resumed");
    let p = serde_json::json!({"candidate":{"path":"/bin/sh","args":["-c","test -f \"$1\"","--",marker],"cwd":null},"repetitions":2,"privacy":{"allow_env":[],"secret_env":[]}});
    fs::write(&plan, serde_json::to_vec(&p).unwrap()).unwrap();
    assert!(
        !cli(&[
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "-o",
            partial.to_str().unwrap()
        ])
        .status
        .success()
    );
    let original = fs::read(partial.join("run.json")).unwrap();
    fs::write(&marker, "").unwrap();
    let o = cli(&[
        "resume",
        partial.to_str().unwrap(),
        "-o",
        resumed.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(original, fs::read(partial.join("run.json")).unwrap());
    let o = cli(&[
        "--store",
        t.path().to_str().unwrap(),
        "baseline",
        "save",
        "protected",
        multi.to_str().unwrap(),
    ]);
    assert!(o.status.success());
    let o = cli(&[
        "--store",
        t.path().to_str().unwrap(),
        "retention",
        "--keep",
        "0",
        "--apply",
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(multi.exists());
    assert!(partial.exists());
    assert!(resumed.exists());
    assert!(!run.exists());
}

#[test]
fn experiment_reports_preserve_failures_and_family_uncertainty() {
    use airbug_bench::*;
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("experiment");
    fs::create_dir(&root).unwrap();
    let base = simple_run(&root.join("single"));
    let mut paired = base.clone();
    paired.id = "paired-fixture".into();
    paired.observations.clear();
    for p in 0..12 {
        for (i, name, value) in [(0, "baseline", 100), (1, "candidate", 200)] {
            let mut o = base.observations[0].clone();
            o.process = p * 2 + i;
            o.pair = Some(p);
            o.variant = name.into();
            o.value = Some(value.to_string());
            paired.observations.push(o);
        }
    }
    paired.cases[0].contract.insert(
        "untrusted".into(),
        "</script><img src=x onerror=alert(1)>".into(),
    );
    paired.save_new(root.join("regression")).unwrap();
    let mut failed = base.clone();
    failed.status = Status::Failed;
    failed.notes.push("controlled failure".into());
    failed.save_new(root.join("failed")).unwrap();
    fs::create_dir(root.join("broken")).unwrap();
    fs::write(root.join("broken/run.json"), "invalid JSON").unwrap();
    fs::create_dir(root.join("unfinished")).unwrap();
    fs::write(root.join("unfinished/plan.json"), "{}").unwrap();
    fs::write(root.join("unfinished/status.json"), "{}").unwrap();
    let mut unsupported = base.clone();
    unsupported.observations[0].value = None;
    unsupported.observations[0].availability = Availability::Unsupported("no GPU".into());
    unsupported.save_new(root.join("unsupported")).unwrap();
    let mut diagnostic = paired.clone();
    diagnostic
        .provenance
        .insert("user.session.policy".into(), "profiler replay".into());
    diagnostic.save_new(root.join("profile")).unwrap();
    let json = t.path().join("report.json");
    let html = t.path().join("report.html");
    for out in [&json, &html] {
        let result = cli(&[
            "report",
            root.to_str().unwrap(),
            "--title",
            "Experiment <unsafe>",
            "-o",
            out.to_str().unwrap(),
        ]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let d: serde_json::Value = serde_json::from_slice(&fs::read(json).unwrap()).unwrap();
    assert_eq!(d["entries"].as_array().unwrap().len(), 7);
    assert_eq!(d["counts"]["regression"], 1);
    assert_eq!(d["counts"]["error"], 3);
    assert_eq!(d["counts"]["uncompared"], 1);
    assert_eq!(d["counts"]["diagnostic"], 1);
    assert_eq!(d["counts"]["unavailable"], 1);
    assert_eq!(d["entries"][0]["comparisons"][0]["independent_units"], 12);
    let page = fs::read_to_string(&html).unwrap();
    assert_eq!(page.matches("<!doctype html>").count(), 1);
    assert!(!page.contains("<img src=x"));
    assert!(page.contains("&lt;img src=x"));
    assert!(page.contains("Effect estimates and confidence intervals"));
    assert!(page.contains("Point estimates by metric"));
    assert!(page.contains("independent units"));
    assert!(page.contains("it is not a confidence interval"));
    assert!(page.matches("<figure class=\"viz\">").count() >= 4);
    assert!(page.contains("href=\"#run-0\""));
    assert_eq!(page.matches("class=\"run-card\"").count(), 7);
    assert!(
        !cli(&[
            "report",
            root.to_str().unwrap(),
            "-o",
            html.to_str().unwrap()
        ])
        .status
        .success()
    );
    // Explicit collection-level correction: 6 identical pairs suffice alone but not for 7 runs.
    paired.observations.retain(|o| o.pair.unwrap() < 6);
    fs::write(
        root.join("regression/run.json"),
        serde_json::to_vec_pretty(&paired).unwrap(),
    )
    .unwrap();
    let out = t.path().join("uncertain.json");
    assert!(
        cli(&[
            "report",
            root.to_str().unwrap(),
            "-o",
            out.to_str().unwrap()
        ])
        .status
        .success()
    );
    let d: serde_json::Value = serde_json::from_slice(&fs::read(out).unwrap()).unwrap();
    let e = d["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["label"] == "regression")
        .unwrap();
    assert_eq!(e["outcome"], "inconclusive");
}
#[test]
fn report_baselines_match_paths_and_expose_missing_targets() {
    let t = tempfile::tempdir().unwrap();
    let a = t.path().join("base");
    let b = t.path().join("head");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    for root in [&a, &b] {
        simple_run(&root.join("target-a"));
        simple_run(&root.join("target-b"));
    }
    simple_run(&a.join("removed"));
    simple_run(&b.join("new"));
    let p = t.path().join("result.json");
    let o = cli(&[
        "report",
        b.to_str().unwrap(),
        "--baseline",
        a.to_str().unwrap(),
        "-o",
        p.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let d: serde_json::Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
    assert_eq!(d["entries"].as_array().unwrap().len(), 4);
    assert_eq!(d["counts"]["error"], 2);
    let e = d["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["label"] == "target-a")
        .unwrap();
    assert!(e["baseline_sha256"].as_str().unwrap().len() == 64);
    assert_eq!(e["outcome"], "inconclusive");
    let o = cli(&[
        "report",
        b.to_str().unwrap(),
        "--baseline",
        a.join("target-a").to_str().unwrap(),
    ]);
    assert!(!o.status.success());
}

#[cfg(unix)]
#[test]
fn memory_requires_instrumentation_and_dry_run_is_read_only() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let t = tempfile::tempdir().unwrap();
    let out = t.path().join("memory");
    let path = out.to_str().unwrap();
    assert!(
        cli(&[
            "run",
            "--memory",
            "--no-ui",
            "--dry-run",
            "--program",
            "/bin/echo",
            "-o",
            path
        ])
        .status
        .success()
    );
    assert!(!out.exists());
    assert!(
        !cli(&[
            "run",
            "--memory",
            "--no-ui",
            "--program",
            "/bin/echo",
            "-o",
            path
        ])
        .status
        .success()
    );
    let run = airbug_bench::Run::load(&out).unwrap();
    assert_eq!(run.status, airbug_bench::Status::Failed);
    assert!(
        run.notes
            .iter()
            .any(|n| n.contains("memory profile missing"))
    );
}

#[test]
fn throughput_budget_selects_one_unit_and_rejects_missing_values() {
    let directory = tempfile::tempdir().unwrap();
    let run_path = directory.path().join("run");
    let mut rec = airbug_bench::Recorder::new();
    rec.case(airbug_bench::Case {
        id: "multi".into(),
        contract: std::collections::BTreeMap::from([
            ("work.counter.items".into(), "10".into()),
            ("work.counter.bytes".into(), "1048576".into()),
        ]),
        metrics: vec![airbug_bench::Metric::duration(
            "wall",
            "test",
            "batch_total",
        )],
    })
    .unwrap();
    rec.observe("multi", "wall", 1_000_000_000).unwrap();
    rec.finish().unwrap().save_new(&run_path).unwrap();
    let path = run_path.to_str().unwrap();
    assert_eq!(
        cli(&["throughput", path, "--min", "2"]).status.code(),
        Some(2)
    );
    assert!(
        cli(&["throughput", path, "--unit", "items", "--min", "2"])
            .status
            .success()
    );
    assert_eq!(
        cli(&["throughput", path, "--unit", "MiB", "--min", "2"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(
        cli(&["throughput", path, "--unit", "missing", "--min", "2"])
            .status
            .code(),
        Some(2)
    );
}

#[cfg(unix)]
#[test]
fn protocol_preserves_worker_allocations_across_processes_and_variants() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let _guard = RUNNER_TEST.lock().unwrap();
    let allocator = airbug_bench::alloc::TrackingAllocator::new(System);
    let mut suite = airbug_bench::Suite::new("worker-protocol");
    suite.bench_threads_allocated_with_local_input(
        "case",
        &allocator,
        2,
        || (),
        |_| unsafe {
            let layout = Layout::from_size_align(64, 8).unwrap();
            let pointer = allocator.alloc(layout);
            assert!(!pointer.is_null());
            allocator.dealloc(pointer, layout);
        },
        airbug_bench::DropPolicy::InsideTiming,
    );
    suite.summary_family("workers");
    suite.summary_scale(airbug_bench::viz::charts::AxisScale::Logarithmic);
    suite.config(airbug_bench::Config {
        samples: 1,
        warmup: std::time::Duration::ZERO,
        sample_time: std::time::Duration::from_nanos(1),
        max_iterations: 2,
    });
    suite.sampling(airbug_bench::Sampling {
        iterations: Some(2),
        ..Default::default()
    });
    suite.compensate_overhead(true);
    let mut fixture = suite.run("").unwrap();
    fixture.cases[0]
        .contract
        .insert("work.input.items".into(), "batch_total".into());
    for o in fixture
        .observations
        .iter_mut()
        .filter(|o| o.metric == "wall")
    {
        o.work_totals.insert("items".into(), "42".into());
    }
    let metric = fixture.cases[0]
        .metrics
        .iter()
        .find(|m| m.id == "wall")
        .unwrap()
        .clone();
    let rows: Vec<_> = fixture
        .observations
        .iter()
        .filter(|o| o.metric == "wall")
        .map(|o| airbug_bench::measurement::FormattedObservation {
            variant: o.variant.clone(),
            process: o.process,
            sequence: o.sequence,
            value: o
                .number()
                .unwrap()
                .map(|v| v / o.operations as f64 / 1000.0),
            unit: "custom-us".into(),
            unavailable_reason: None,
            display: None,
        })
        .collect();
    let snapshot = airbug_bench::measurement::FormattedMetric {
        case: fixture.cases[0].id.clone(),
        metric,
        human: rows.clone(),
        machine: rows.clone(),
        throughput: std::collections::BTreeMap::from([(
            "items".into(),
            airbug_bench::measurement::FormattedThroughput {
                work_per_operation: None,
                observations: rows,
            },
        )]),
    };
    airbug_bench::measurement::save_formatted(&mut fixture, &[snapshot]).unwrap();
    let t = tempfile::tempdir().unwrap();
    let payload = t.path().join("payload.txt");
    fs::write(
        &payload,
        format!(
            "BENCH_RESULT={}\n",
            serde_json::to_string(&fixture).unwrap()
        ),
    )
    .unwrap();
    let output = t.path().join("run");
    let result = cli(&[
        "run",
        "--program",
        "/bin/cat",
        "--baseline",
        "/bin/cat",
        "--repetitions",
        "2",
        "--protocol",
        "--no-ui",
        "--output",
        output.to_str().unwrap(),
        "--",
        payload.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let run = airbug_bench::Run::load(&output).unwrap();
    assert_eq!(
        airbug_bench::presentation::load_scales(&run).unwrap()["worker-protocol/case"],
        airbug_bench::viz::charts::AxisScale::Logarithmic
    );
    assert_eq!(
        airbug_bench::presentation::load_families(&run).unwrap()["worker-protocol/case"],
        "worker-protocol/workers"
    );
    let formatted = airbug_bench::measurement::load_formatted(&run).unwrap();
    assert_eq!(formatted.len(), 1);
    assert_eq!(formatted[0].human.len(), 4);
    assert_eq!(formatted[0].throughput["items"].observations.len(), 4);
    for row in &formatted[0].human {
        let raw = run
            .observations
            .iter()
            .find(|o| {
                o.metric == "wall"
                    && o.process == row.process
                    && o.variant == row.variant
                    && o.sequence == row.sequence
            })
            .unwrap();
        assert_eq!(
            row.value,
            raw.number()
                .unwrap()
                .map(|v| v / raw.operations as f64 / 1000.0)
        );
        assert_eq!(row.unit, "custom-us");
    }
    assert!(
        airbug_bench::report::html_run(&run)
            .unwrap()
            .contains("custom-us")
    );
    let mut changed = run.clone();
    changed
        .observations
        .iter_mut()
        .find(|o| o.metric == "wall")
        .unwrap()
        .work_totals
        .insert("items".into(), "43".into());
    assert!(airbug_bench::measurement::load_formatted(&changed).is_err());
    assert_eq!(run.worker_allocations.len(), 8);
    let identities: std::collections::BTreeSet<_> = run
        .worker_allocations
        .iter()
        .map(|w| (w.variant.as_str(), w.process))
        .collect();
    assert_eq!(identities.len(), 4);
    for worker in &run.worker_allocations {
        assert_eq!(worker.metrics["alloc.bytes"], "128");
        assert_eq!(worker.operations, 2);
        assert!(worker.adjusted_wall_ns.is_some());
        assert!(run.observations.iter().any(|o| o.case == worker.case
            && o.variant == worker.variant
            && o.process == worker.process
            && o.sequence == worker.sequence
            && o.metric == "wall.adjusted"));
        assert!(run.observations.iter().any(|o| o.case == worker.case
            && o.variant == worker.variant
            && o.process == worker.process
            && o.sequence == worker.sequence
            && o.metric == "wall"));
    }
    // Metric selection must retain matching worker records or remove them when
    // their enclosing wall sample is filtered out.
    for metric in ["wall", "wall.adjusted", "alloc.bytes"] {
        let selected = cli(&["compare", output.to_str().unwrap(), "--metric", metric]);
        assert!(
            selected.status.success(),
            "{}",
            String::from_utf8_lossy(&selected.stderr)
        );
    }
}

#[test]
fn compare_exports_seeded_null_draws_and_portable_html() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let mut run = simple_run(&source);
    let original = run.observations[0].clone();
    run.observations.clear();
    for process in 0..8 {
        let mut observation = original.clone();
        observation.process = process;
        observation.value = Some((40 + process).to_string());
        run.observations.push(observation);
    }
    let baseline = t.path().join("baseline");
    run.save_new(&baseline).unwrap();
    let path = baseline.to_str().unwrap();
    let args = [
        "compare",
        path,
        path,
        "--json",
        "--hypothesis-distribution",
        "--hypothesis-resamples",
        "128",
        "--hypothesis-seed",
        "7",
    ];
    let first = cli(&args);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, cli(&args).stdout);
    let rows: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(
        rows[0]["hypothesis"]["null_distribution"]
            .as_array()
            .unwrap()
            .len(),
        128
    );
    let output = t.path().join("comparison.html");
    let html = cli(&[
        "compare",
        path,
        path,
        "--html",
        "--hypothesis-resamples",
        "128",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(
        html.status.success(),
        "{}",
        String::from_utf8_lossy(&html.stderr)
    );
    let document = fs::read_to_string(output).unwrap();
    assert!(document.contains("<svg ") && document.contains("128 null draws"));
    assert!(!cli(&["compare", path, "--json", "--html"]).status.success());
    assert!(
        !cli(&["compare", path, "--hypothesis-resamples", "1"])
            .status
            .success()
    );
    assert!(
        !cli(&["compare", path, "--hypothesis-distribution"])
            .status
            .success()
    );
}

#[test]
fn report_summary_parameter_and_scale_reach_html() {
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    run.cases[0]
        .contract
        .insert("param.size".into(), "10".into());
    let source = t.path().join("numeric");
    run.save_new(&source).unwrap();
    let source = source.to_str().unwrap();
    let output = t.path().join("summary.html");
    let result = cli(&[
        "report",
        source,
        "--summary-parameter",
        "size",
        "--summary-scale",
        "logarithmic",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let html = fs::read_to_string(output).unwrap();
    assert!(html.contains("Input parameter summaries"));
    assert!(html.contains("X: size"));
    assert!(html.contains("0 series omitted"));
    assert!(html.contains("Violin summaries"));
    let mean_output = t.path().join("mean.html");
    assert!(
        cli(&[
            "report",
            source,
            "--summary-parameter",
            "size",
            "--summary-estimator",
            "mean",
            "-o",
            mean_output.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(
        fs::read_to_string(&mean_output)
            .unwrap()
            .contains("arithmetic mean of normalized observations")
    );
    assert!(
        !cli(&[
            "report",
            source,
            "--summary-estimator",
            "mean",
            "-o",
            t.path().join("invalid.html").to_str().unwrap()
        ])
        .status
        .success()
    );
    for observation in &mut run.observations {
        observation.value = Some("0".into());
    }
    let zero = t.path().join("zero");
    run.save_new(&zero).unwrap();
    let zero_html = t.path().join("zero.html");
    assert!(
        cli(&[
            "report",
            zero.to_str().unwrap(),
            "--summary-scale",
            "logarithmic",
            "--output",
            zero_html.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(
        fs::read_to_string(zero_html)
            .unwrap()
            .contains("population contains nonpositive values; no subset was plotted")
    );

    assert!(
        !cli(&["report", source, "--summary-scale", "logarithmic"])
            .status
            .success()
    );
    assert!(
        !cli(&["report", source, "--summary-parameter", "size"])
            .status
            .success()
    );
    let bad = t.path().join("summary.json");
    assert!(
        !cli(&[
            "report",
            source,
            "--summary-parameter",
            "size",
            "-o",
            bad.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(!bad.exists());
}

#[test]
fn html_compare_overlays_independent_process_regressions() {
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    run.cases[0].metrics[0].statistic = "batch_total".into();
    let original = run.observations[0].clone();
    run.observations.clear();
    for process in 0..2 {
        for count in 1..=3 {
            let mut o = original.clone();
            o.process = process;
            o.sequence = count;
            o.operations = count;
            o.value = Some((count * (10 + u64::from(process))).to_string());
            run.observations.push(o);
        }
    }
    let baseline = t.path().join("baseline");
    run.save_new(&baseline).unwrap();
    for o in &mut run.observations {
        o.value = Some((o.operations * 20).to_string());
    }
    let candidate = t.path().join("candidate");
    run.save_new(&candidate).unwrap();
    let output = cli(&[
        "compare",
        baseline.to_str().unwrap(),
        candidate.to_str().unwrap(),
        "--html",
        "--hypothesis-resamples",
        "128",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = String::from_utf8(output.stdout).unwrap();
    assert_eq!(html.matches("<polygon").count(), 8);
    assert!(html.contains("relative mean change"));
    assert!(html.contains("relative median change"));
    assert!(html.contains("Practical noise region"));
    for label in [
        "baseline process 0",
        "baseline process 1",
        "candidate process 0",
        "candidate process 1",
    ] {
        assert!(html.contains(label), "{label}");
    }
    assert!(html.contains("regression comparison"));
    assert!(html.contains("density comparison"));
    assert!(html.contains("Gaussian KDE bandwidth"));
    assert!(html.contains("point mass"));
    assert!(html.contains("Observation rug at zero"));
    let table_comparison = cli(&[
        "compare",
        baseline.to_str().unwrap(),
        candidate.to_str().unwrap(),
        "--html",
        "--no-plots",
    ]);
    assert!(table_comparison.status.success());
    let table_comparison = String::from_utf8(table_comparison.stdout).unwrap();
    assert!(table_comparison.contains("<table"));
    assert!(!table_comparison.contains("<svg"));
    assert!(
        !cli(&[
            "compare",
            baseline.to_str().unwrap(),
            candidate.to_str().unwrap(),
            "--no-plots"
        ])
        .status
        .success()
    );
    let report_path = t.path().join("experiment.html");
    let report = cli(&[
        "report",
        candidate.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--output",
        report_path.to_str().unwrap(),
    ]);
    assert!(
        report.status.success(),
        "{}",
        String::from_utf8_lossy(&report.stderr)
    );
    let report_html = fs::read_to_string(&report_path).unwrap();
    assert_eq!(report_html.matches("<polygon").count(), 8);
    assert!(report_html.contains("Iteration time comparison"));
    assert!(report_html.contains("Sequence indices do not imply paired measurements"));
    assert!(report_html.contains("Welch null distribution"));
    assert!(report_html.contains("Independent units: baseline 2, candidate 2"));
    assert_eq!(report_html.matches("<!doctype html>").count(), 1);
    assert!(report_html.contains("relative mean change"));
    assert!(report_html.contains("relative median change"));
    assert!(report_html.contains("Practical noise region"));
    assert!(report_html.contains("Violin summaries"));
    assert!(report_html.contains("candidate process 1"));
    assert!(report_html.contains("density comparison"));
    assert!(report_html.contains("Gaussian KDE bandwidth"));
    assert!(report_html.contains("point mass"));
    assert!(report_html.contains("No observations discarded"));
    let tables_path = t.path().join("tables.html");
    let tables = cli(&[
        "report",
        candidate.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--no-plots",
        "--output",
        tables_path.to_str().unwrap(),
    ]);
    assert!(
        tables.status.success(),
        "{}",
        String::from_utf8_lossy(&tables.stderr)
    );
    let tables = fs::read_to_string(tables_path).unwrap();
    assert!(tables.contains("<table"));
    assert!(!tables.contains("<svg"));
    assert!(!tables.contains("Violin summaries"));
    assert!(tables.contains("wall"));
    assert!(
        !cli(&[
            "report",
            candidate.to_str().unwrap(),
            "--no-plots",
            "--summary-parameter",
            "size",
            "--output",
            t.path().join("bad.html").to_str().unwrap()
        ])
        .status
        .success()
    );

    run.cases[0].metrics[0].scope = "incompatible".into();
    let incompatible = t.path().join("incompatible");
    run.save_new(&incompatible).unwrap();
    let failed_path = t.path().join("incompatible.html");
    let failed = cli(&[
        "report",
        incompatible.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--output",
        failed_path.to_str().unwrap(),
    ]);
    assert!(
        failed.status.success(),
        "{}",
        String::from_utf8_lossy(&failed.stderr)
    );
    let failed_html = fs::read_to_string(failed_path).unwrap();
    assert!(failed_html.contains("incompatible cases, metrics or workload contracts"));
    assert!(!failed_html.contains("regression comparison"));
    assert!(!failed_html.contains("density comparison"));
}

#[test]
fn analyze_saved_run_exports_distributions_without_mutating_source() {
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    run.cases[0].metrics[0].statistic = "batch_total".into();
    let original = run.observations[0].clone();
    run.observations = (1..=4)
        .map(|count| {
            let mut o = original.clone();
            o.operations = count;
            o.sequence = count;
            o.value = Some((count * 10 + count % 2).to_string());
            o
        })
        .collect();
    let path = t.path().join("run");
    run.save_new(&path).unwrap();
    let original_bytes = fs::read(path.join("run.json")).unwrap();
    let args = [
        "analyze",
        path.to_str().unwrap(),
        "--resamples",
        "128",
        "--analysis-seed",
        "7",
        "--bootstrap-distributions",
    ];
    let first = cli(&args);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, cli(&args).stdout);
    let report: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(
        report["rows"][0]["distributions"]["mean"]
            .as_array()
            .unwrap()
            .len(),
        128
    );
    assert_eq!(
        report["rows"][0]["regressions"][0]["slope_distribution"]
            .as_array()
            .unwrap()
            .len(),
        128
    );
    let mut tables_args = args.to_vec();
    tables_args.extend(["--format", "html", "--no-plots"]);
    let tables = cli(&tables_args);
    assert!(tables.status.success());
    let tables = String::from_utf8(tables.stdout).unwrap();
    assert!(tables.contains("<table"));
    assert!(!tables.contains("<svg"));
    assert!(tables.contains("wall"));
    let mut invalid = args.to_vec();
    invalid.push("--no-plots");
    assert!(!cli(&invalid).status.success());
    let html_path = t.path().join("analysis.html");
    let mut html_args = args.to_vec();
    html_args.extend(["--format", "html", "--output", html_path.to_str().unwrap()]);
    assert!(cli(&html_args).status.success());
    let html = fs::read(&html_path).unwrap();
    assert!(String::from_utf8_lossy(&html).contains("Violin summaries"));
    let log = cli(&[
        "analyze",
        path.to_str().unwrap(),
        "--resamples",
        "128",
        "--format",
        "html",
        "--summary-scale",
        "logarithmic",
    ]);
    assert!(log.status.success());
    assert!(String::from_utf8_lossy(&log.stdout).contains("violin summary"));
    assert!(
        !cli(&[
            "analyze",
            path.to_str().unwrap(),
            "--summary-scale",
            "logarithmic"
        ])
        .status
        .success()
    );
    assert_eq!(
        String::from_utf8_lossy(&html)
            .matches("Bootstrap confidence interval")
            .count(),
        5
    );
    assert!(!cli(&html_args).status.success());
    assert_eq!(fs::read(&html_path).unwrap(), html);
    assert_eq!(fs::read(path.join("run.json")).unwrap(), original_bytes);
    assert!(
        !cli(&["analyze", path.to_str().unwrap(), "--resamples", "1"])
            .status
            .success()
    );
    let ordinary = cli(&["analyze", path.to_str().unwrap(), "--resamples", "128"]);
    assert!(ordinary.status.success());
    assert!(!String::from_utf8_lossy(&ordinary.stdout).contains("slope_distribution"));
}

#[test]
fn compare_relative_export_preserves_process_units_and_check_decisions() {
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    run.cases[0].metrics[0].statistic = "batch_total".into();
    let original = run.observations[0].clone();
    run.observations.clear();
    for (process, count, value) in [(0, 5, 2), (1, 1, 4)] {
        for sequence in 0..count {
            let mut o = original.clone();
            o.process = process;
            o.sequence = sequence;
            o.operations = 10;
            o.value = Some((value * 10).to_string());
            run.observations.push(o);
        }
    }
    let baseline = t.path().join("baseline");
    run.save_new(&baseline).unwrap();
    for o in &mut run.observations {
        o.value = Some((o.value.as_ref().unwrap().parse::<u64>().unwrap() * 2).to_string());
    }
    let candidate = t.path().join("candidate");
    run.save_new(&candidate).unwrap();
    let before = fs::read(baseline.join("run.json")).unwrap();
    let args = [
        "compare",
        baseline.to_str().unwrap(),
        candidate.to_str().unwrap(),
        "--json",
        "--check",
        "--hypothesis-resamples",
        "128",
        "--hypothesis-seed",
        "7",
    ];
    let ordinary = cli(&args);
    let mut extended = args.to_vec();
    extended.push("--relative-distributions");
    let captured = cli(&extended);
    assert_eq!(ordinary.status.code(), captured.status.code());
    assert_eq!(captured.stdout, cli(&extended).stdout);
    let data: serde_json::Value = serde_json::from_slice(&captured.stdout).unwrap();
    let rows: serde_json::Value = serde_json::from_slice(&ordinary.stdout).unwrap();
    assert_eq!(data["comparisons"], rows);
    let relative = &data["relative"][0];
    assert_eq!(relative["baseline_units"], 2);
    assert_eq!(relative["candidate_units"], 2);
    let reference = airbug_bench::relative::bootstrap(
        &[2., 4.],
        &[4., 8.],
        &airbug_bench::bootstrap::Config {
            resamples: 128,
            seed: 7,
            confidence_level: 0.95,
        },
    )
    .unwrap();
    assert_eq!(relative["report"], serde_json::to_value(reference).unwrap());
    assert_eq!(before, fs::read(baseline.join("run.json")).unwrap());
    assert!(
        !cli(&[
            "compare",
            baseline.to_str().unwrap(),
            "--json",
            "--relative-distributions"
        ])
        .status
        .success()
    );
    assert!(
        !cli(&[
            "compare",
            baseline.to_str().unwrap(),
            candidate.to_str().unwrap(),
            "--relative-distributions"
        ])
        .status
        .success()
    );
}

#[test]
fn analyze_restores_saved_scale_and_allows_explicit_override() {
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    for o in &mut run.observations {
        o.value = Some("0".into());
    }
    let scales = run
        .cases
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                airbug_bench::viz::charts::AxisScale::Logarithmic,
            )
        })
        .collect();
    airbug_bench::presentation::save_scales(&mut run, &scales).unwrap();
    let path = t.path().join("saved");
    run.save_new(&path).unwrap();
    let source = fs::read(path.join("run.json")).unwrap();
    let args = [
        "analyze",
        path.to_str().unwrap(),
        "--format",
        "html",
        "--resamples",
        "128",
    ];
    let restored = cli(&args);
    assert!(
        restored.status.success(),
        "{}",
        String::from_utf8_lossy(&restored.stderr)
    );
    assert!(
        String::from_utf8_lossy(&restored.stdout)
            .contains("population contains nonpositive values")
    );
    let mut explicit = args.to_vec();
    explicit.extend(["--summary-scale", "linear"]);
    let overridden = cli(&explicit);
    assert!(overridden.status.success());
    assert!(
        !String::from_utf8_lossy(&overridden.stdout)
            .contains("population contains nonpositive values")
    );
    for (name, flags, suppressed) in [
        ("restored", vec![], true),
        ("override", vec!["--summary-scale", "linear"], false),
    ] {
        let destination = t.path().join(format!("{name}.html"));
        let mut args = vec![
            "report",
            path.to_str().unwrap(),
            "--output",
            destination.to_str().unwrap(),
        ];
        args.extend(flags);
        let result = cli(&args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            fs::read_to_string(destination)
                .unwrap()
                .contains("population contains nonpositive values"),
            suppressed
        );
    }
    assert_eq!(source, fs::read(path.join("run.json")).unwrap());
}

#[test]
fn report_restores_families_and_plots_dynamic_throughput_with_gaps() {
    use airbug_bench::{Status, presentation, report};
    use std::collections::BTreeMap;
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    let template_case = run.cases[0].clone();
    let template_observation = run.observations[0].clone();
    run.cases.clear();
    run.observations.clear();
    let mut families = BTreeMap::new();
    for size in [100u64, 1, 10] {
        let id = format!("input/{size}");
        let mut case = template_case.clone();
        case.id = id.clone();
        case.metrics[0].statistic = "batch_total".into();
        case.contract.insert("param.size".into(), size.to_string());
        for unit in ["bytes", "items"] {
            case.contract
                .insert(format!("work.input.{unit}"), "batch_total".into());
        }
        run.cases.push(case);
        families.insert(id.clone(), "dynamic".into());
        for sequence in 0..6 {
            let mut o = template_observation.clone();
            o.case = id.clone();
            o.process = if sequence < 5 { 0 } else { 1 };
            o.sequence = sequence;
            o.operations = 8;
            o.value = Some("1000000000".into());
            let total = size * if sequence < 5 { 1 } else { 3 };
            o.work_totals
                .insert("bytes".into(), (total * 1048576).to_string());
            o.work_totals.insert("items".into(), total.to_string());
            run.observations.push(o);
        }
    }
    presentation::save_families(&mut run, &families).unwrap();
    let medians = report::throughput_process_medians(&run).unwrap();
    let values: Vec<_> = medians
        .iter()
        .filter(|r| r.case == "input/10" && r.unit == "MiB")
        .map(|r| r.median)
        .collect();
    assert_eq!(values, [10., 30.]); // Work totals are per batch, not multiplied by eight operations.
    for gap in [false, true] {
        let mut fixture = run.clone();
        if gap {
            fixture.status = Status::Incomplete;
            fixture.observations.retain(|o| o.case != "input/10");
        }
        let source = t.path().join(if gap { "gap" } else { "complete" });
        fixture.save_new(&source).unwrap();
        let before = fs::read(source.join("run.json")).unwrap();
        let destination = source.join("report.html");
        let result = cli(&[
            "report",
            source.to_str().unwrap(),
            "--summary-parameter",
            "size",
            "--output",
            destination.to_str().unwrap(),
        ]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let html = fs::read_to_string(destination).unwrap();
        assert!(html.contains("MiB/s") && html.contains("items/s"));
        assert!(html.contains("family dynamic / candidate"));
        assert_eq!(html.contains("Incomplete throughput families"), gap);
        let throughput = html
            .split("<h2>Throughput input summaries</h2>")
            .nth(1)
            .unwrap()
            .split("</section>")
            .next()
            .unwrap();
        assert_eq!(
            throughput.matches("pathLength=\"100\"").count(),
            if gap { 0 } else { 2 }
        );
        assert_eq!(before, fs::read(source.join("run.json")).unwrap());
    }
}

#[test]
fn saved_report_restores_formatter_snapshot_and_rejects_stale_values() {
    use airbug_bench::measurement::{FormattedMetric, FormattedObservation, save_formatted};
    let t = tempfile::tempdir().unwrap();
    let mut run = simple_run(&t.path().join("source"));
    let case = &run.cases[0];
    let metric = case.metrics[0].clone();
    let rows: Vec<_> = run
        .observations
        .iter()
        .filter(|o| o.case == case.id && o.metric == metric.id)
        .map(|o| FormattedObservation {
            variant: o.variant.clone(),
            process: o.process,
            sequence: o.sequence,
            value: o.number().unwrap().map(|v| v / 1000.0),
            unit: "custom-display".into(),
            unavailable_reason: None,
            display: Some("custom text <script> | 1".into()),
        })
        .collect();
    let formatted = FormattedMetric {
        throughput: Default::default(),
        case: case.id.clone(),
        metric,
        human: rows.clone(),
        machine: rows,
    };
    save_formatted(&mut run, &[formatted]).unwrap();
    let source = t.path().join("formatted");
    run.save_new(&source).unwrap();
    let original = fs::read(source.join("run.json")).unwrap();
    let csv = t.path().join("formatted.csv");
    assert!(
        cli(&[
            "export",
            source.to_str().unwrap(),
            "--format",
            "formatted-csv",
            "-o",
            csv.to_str().unwrap()
        ])
        .status
        .success()
    );
    let csv_text = fs::read_to_string(csv).unwrap();
    assert!(csv_text.contains("custom-display"));
    assert!(csv_text.contains("normalized_per_operation"));
    assert!(!csv_text.contains("custom text"));
    let html = t.path().join("formatted.html");
    let result = cli(&[
        "report",
        source.to_str().unwrap(),
        "--no-plots",
        "-o",
        html.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let table_html = fs::read_to_string(html).unwrap();
    assert!(table_html.contains("custom-display"));
    assert!(table_html.contains("custom text &lt;script&gt;"));
    assert!(!table_html.contains("custom text <script>"));
    assert!(!table_html.contains("<svg"));
    let plotted = t.path().join("plotted.html");
    assert!(
        cli(&[
            "report",
            source.to_str().unwrap(),
            "-o",
            plotted.to_str().unwrap()
        ])
        .status
        .success()
    );
    let plot_html = fs::read_to_string(plotted).unwrap();
    assert!(plot_html.contains("formatted observations (custom-display)"));
    assert!(plot_html.contains("Saved formatter output"));
    assert_eq!(original, fs::read(source.join("run.json")).unwrap());
    run.observations[0].value = Some("99999".into());
    let stale = t.path().join("stale");
    run.save_new(&stale).unwrap();
    let stale_csv = t.path().join("stale.csv");
    let rejected = cli(&[
        "export",
        stale.to_str().unwrap(),
        "--format",
        "formatted-csv",
        "-o",
        stale_csv.to_str().unwrap(),
    ]);
    assert!(!rejected.status.success());
    assert!(!stale_csv.exists());
    let result = cli(&["report", stale.to_str().unwrap()]);
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("saved formatting is stale"));
    assert!(!text.contains("custom-display"));
    let stale_html = t.path().join("stale.html");
    assert!(
        cli(&[
            "report",
            stale.to_str().unwrap(),
            "-o",
            stale_html.to_str().unwrap()
        ])
        .status
        .success()
    );
    let stale_html = fs::read_to_string(stale_html).unwrap();
    assert!(stale_html.contains("Formatted plots unavailable"));
    assert!(!stale_html.contains("formatted observations (custom-display)"));
}

#[test]
fn init_preserves_renamed_dependency_and_enables_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("Cargo.toml");
    let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    fs::write(
        &manifest,
        format!(
            r#"[package]
name = "renamed-init"
version = "0.1.0"
[dev-dependencies]
ab = {{ package = "airbug-bench", path = {:?}, default-features = false, features = ["memory"] }}
"#,
            library
        ),
    )
    .unwrap();
    let o = cli(&["init", "--manifest-path", manifest.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let doc = fs::read_to_string(&manifest)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    assert!(doc["dev-dependencies"].get("airbug-bench").is_none());
    assert_eq!(
        doc["dev-dependencies"]["ab"]["default-features"].as_bool(),
        Some(false)
    );
    let features = doc["dev-dependencies"]["ab"]["features"]
        .as_array()
        .unwrap();
    assert!(features.iter().any(|v| v.as_str() == Some("memory")));
    assert!(features.iter().any(|v| v.as_str() == Some("macros")));
    assert!(
        fs::read_to_string(dir.path().join("benches/bench.rs"))
            .unwrap()
            .contains("#[ab::suite]")
    );
}

#[test]
fn analyze_restores_saved_bootstrap_settings_and_overrides_one_field() {
    use airbug_bench::bootstrap::{Config, save_settings};
    let temp = tempfile::tempdir().unwrap();
    let mut run = simple_run(&temp.path().join("source"));
    let original = run.observations[0].clone();
    run.observations = (0..4)
        .map(|sequence| {
            let mut value = original.clone();
            value.sequence = sequence;
            value.value = Some((sequence + 1).to_string());
            value
        })
        .collect();
    let case = run.cases[0].id.clone();
    save_settings(
        &mut run,
        &Config::default(),
        &std::collections::BTreeMap::from([(
            case.clone(),
            Config {
                resamples: 32,
                confidence_level: 0.9,
                seed: 7,
            },
        )]),
    )
    .unwrap();
    let path = temp.path().join("saved");
    run.save_new(&path).unwrap();
    let bytes = fs::read(path.join("run.json")).unwrap();
    for (extra, count) in [(vec![], 32), (vec!["--resamples", "64"], 64)] {
        let mut args = vec![
            "analyze",
            path.to_str().unwrap(),
            "--bootstrap-distributions",
        ];
        args.extend(extra);
        let output = cli(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: airbug_bench::bootstrap::Report =
            serde_json::from_slice(&output.stdout).unwrap();
        let settings = report.config_for_case(&case);
        assert_eq!(settings.resamples, count);
        assert_eq!(settings.confidence_level, 0.9);
        assert_eq!(settings.seed, 7);
        assert_eq!(
            report.rows[0].distributions.as_ref().unwrap().mean.len(),
            count
        );
    }
    assert_eq!(bytes, fs::read(path.join("run.json")).unwrap());
}
