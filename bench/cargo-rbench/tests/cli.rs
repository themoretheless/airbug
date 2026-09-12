static RUNNER_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
use std::{fs, path::Path, process::Command};
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-rbench"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn help_and_cargo_invocation() {
    assert!(cli(&["--help"]).status.success());
    assert!(cli(&["rbench", "doctor"]).status.success());
}
#[test]
fn check_filter_narrows_cases() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    {
        let mut rec = rbench::Recorder::new();
        for (id, val) in [("fast", 10u128), ("slow", 100u128)] {
            rec.case(rbench::Case {
                id: id.into(),
                contract: Default::default(),
                metrics: vec![rbench::Metric::duration("wall", "test", "total")],
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
    assert!(cli(&["--store", store, "note", p, "hello note"]).status.success());
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
    let o = cli(&["context", store.join("a").to_str().unwrap(), store.join("a").to_str().unwrap(), "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
    // A differing environment key surfaces exactly one diff.
    let mut b = a.clone();
    b.environment.insert("os".into(), "other-os".into());
    b.save_new(store.join("b")).unwrap();
    let o = cli(&["context", store.join("a").to_str().unwrap(), store.join("b").to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let diffs = v.as_array().unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0]["key"], "environment.os");
    assert_eq!(diffs[0]["b"], "other-os");
    // trend --json: one complete run with case "=unsafe,case"/metric "wall" value 42.
    let o = cli(&["--store", store.to_str().unwrap(), "trend", "--case", "=unsafe,case", "--metric", "wall", "--json"]);
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
    failed.status = rbench::Status::Failed;
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
        let mut rec = rbench::Recorder::new();
        rec.case(rbench::Case {
            id: "scan".into(),
            contract: std::collections::BTreeMap::from([
                ("work.unit".to_string(), "elements".to_string()),
                ("work.count".to_string(), "10".to_string()),
            ]),
            metrics: vec![rbench::Metric::duration("wall", "test", "batch_total")],
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
    assert_eq!(cli(&["throughput", path, "--min", "20"]).status.code(), Some(1));
    assert_eq!(cli(&["throughput", path, "--min", "5"]).status.code(), Some(0));
    assert_eq!(cli(&["throughput", path, "--max", "5"]).status.code(), Some(1));
    // Misuse: inverted range.
    assert_eq!(
        cli(&["throughput", path, "--min", "100", "--max", "10"]).status.code(),
        Some(2)
    );
}
#[test]
fn doctor_text_and_json_report_capabilities() {
    let o = cli(&["doctor"]);
    assert!(o.status.success());
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(text.contains("rbench "));
    assert!(text.contains("Statistics: independent process units required"));
    let o = cli(&["doctor", "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["rbench"], env!("CARGO_PKG_VERSION"));
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
        assert!(o.status.success(), "{shell}: {}", String::from_utf8_lossy(&o.stderr));
        let script = String::from_utf8(o.stdout).unwrap();
        assert!(!script.is_empty(), "{shell}: empty script");
        // Every generator embeds the completed binary name and the real subcommands.
        assert!(script.contains("cargo-rbench"), "{shell}: missing binary name");
        assert!(script.contains("doctor") && script.contains("compare"), "{shell}: missing subcommands");
    }
    // The cargo-subcommand invocation form produces the same output.
    let o = cli(&["rbench", "completions", "zsh"]);
    assert!(o.status.success());
    assert!(String::from_utf8(o.stdout).unwrap().starts_with("#compdef cargo-rbench"));
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
    let r = rbench::Run::load(p).unwrap();
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
    assert!(!cli(&["run", "--program", "/bin/echo", "--output", p])
        .status
        .success());
    let bad = t.path().join("bad");
    assert!(!cli(&[
        "run",
        "--program",
        "/usr/bin/false",
        "--output",
        bad.to_str().unwrap()
    ])
    .status
    .success());
    assert_eq!(
        rbench::Run::load(&bad).unwrap().status,
        rbench::Status::Failed
    );
    let timeout = t.path().join("timeout");
    assert!(!cli(&[
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
    .success());
    assert!(fs::read_to_string(timeout.join("run.json"))
        .unwrap()
        .contains("timed out"));
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
        "RBENCH_RESULT={}",
    ]);
    assert!(!o.status.success());
    assert_eq!(
        rbench::Run::load(out).unwrap().status,
        rbench::Status::Failed
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
    let run = rbench::Run::load(out).unwrap();
    assert_eq!(run.cases.len(), 11);
    assert!(run.cases.iter().all(|c| c
        .metrics
        .iter()
        .any(|m| m.id == "gpu.pass.mean" && m.phase == "gpu_diagnostic")));
    assert!(run
        .observations
        .iter()
        .filter(|o| o.metric == "gpu.pass.mean")
        .all(|o| o.value.is_none()));
    let p = src.join("results.json");
    let mut rows: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    rows.pop();
    fs::write(p, serde_json::to_vec(&rows).unwrap()).unwrap();
    assert!(!cli(&[
        "import-forma",
        src.to_str().unwrap(),
        "-o",
        t.path().join("bad").to_str().unwrap()
    ])
    .status
    .success());
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
fn baseline_alias_checks_integrity_and_gate_exit_codes() {
    let _guard = RUNNER_TEST.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("run");
    let store = dir.path().join("store");
    let mut recorder = rbench::Recorder::new();
    recorder
        .case(rbench::Case {
            id: "fixture".into(),
            contract: Default::default(),
            metrics: vec![rbench::Metric::duration("wall", "test", "total")],
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

fn simple_run(path: &Path) -> rbench::Run {
    let mut rec = rbench::Recorder::new();
    rec.case(rbench::Case {
        id: "=unsafe,case".into(),
        contract: Default::default(),
        metrics: vec![rbench::Metric::duration("wall", "test", "total")],
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
    assert!(cli(&["--store", store, "note", "last", "portable note"])
        .status
        .success());
    assert_eq!(original, fs::read(run.join("run.json")).unwrap());
    let report = cli(&["--store", store, "report", "last"]);
    assert!(String::from_utf8(report.stdout)
        .unwrap()
        .contains("portable note"));
    let csv = t.path().join("out.csv");
    assert!(cli(&[
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
    .success());
    assert!(fs::read_to_string(csv)
        .unwrap()
        .contains("\"'=unsafe,case\""));
    let jsonl = t.path().join("out.jsonl");
    assert!(cli(&[
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
    .success());
    let value: serde_json::Value =
        serde_json::from_str(fs::read_to_string(jsonl).unwrap().trim()).unwrap();
    assert_eq!(value["observation"]["value"], "42");
    let bundle = t.path().join("run.bundle.json");
    assert!(cli(&[
        "--store",
        store,
        "bundle",
        "last",
        "-o",
        bundle.to_str().unwrap()
    ])
    .status
    .success());
    let restored = t.path().join("restored");
    assert!(cli(&[
        "unpack",
        bundle.to_str().unwrap(),
        "-o",
        restored.to_str().unwrap()
    ])
    .status
    .success());
    assert_eq!(original, fs::read(restored.join("run.json")).unwrap());
    let other = t.path().join("empty-store");
    let o = cli(&[
        "--store",
        other.to_str().unwrap(),
        "notes",
        restored.to_str().unwrap(),
    ]);
    assert!(o.status.success());
    assert!(String::from_utf8(o.stdout)
        .unwrap()
        .contains("portable note"));
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
    assert!(cli(&[
        "bundle",
        run.to_str().unwrap(),
        "-o",
        bundle.to_str().unwrap()
    ])
    .status
    .success());
    let mut v: serde_json::Value = serde_json::from_slice(&fs::read(&bundle).unwrap()).unwrap();
    v["files"][0]["name"] = "../escaped".into();
    fs::write(&bundle, serde_json::to_vec(&v).unwrap()).unwrap();
    let out = t.path().join("out");
    assert!(!cli(&[
        "unpack",
        bundle.to_str().unwrap(),
        "-o",
        out.to_str().unwrap()
    ])
    .status
    .success());
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
        o.availability = rbench::Availability::Unsupported("missing".into());
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
    assert!(String::from_utf8(o.stderr)
        .unwrap()
        .contains("inconclusive"));
}
#[test]
fn context_trend_and_ci_template() {
    let t = tempfile::tempdir().unwrap();
    let run = t.path().join("run");
    simple_run(&run);
    let o = cli(&["context", run.to_str().unwrap(), run.to_str().unwrap()]);
    assert!(String::from_utf8(o.stdout)
        .unwrap()
        .contains("No differences"));
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
    let secret = "RBENCH-sensitive-\"split\\secret-194812";
    fs::write(&plan,serde_json::to_vec(&serde_json::json!({"candidate":{"path":"/bin/sh","args":["-c","printf '%s' \"$RBENCH_TEST_SECRET\"; printf '%s' \"$RBENCH_TEST_SECRET\" >&2; test -z \"$RBENCH_UNLISTED\""],"cwd":null},"repetitions":1,"privacy":{"allow_env":[],"secret_env":["RBENCH_TEST_SECRET"]}})).unwrap()).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_cargo-rbench"))
        .args([
            "run",
            "--plan",
            plan.to_str().unwrap(),
            "-o",
            run.to_str().unwrap(),
        ])
        .env("RBENCH_TEST_SECRET", secret)
        .env("RBENCH_UNLISTED", "must-not-inherit")
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
    assert!(cli(&[
        "bundle",
        run.to_str().unwrap(),
        "-o",
        bundle.to_str().unwrap()
    ])
    .status
    .success());
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
    assert!(!cli(&[
        "run",
        "--plan",
        plan.to_str().unwrap(),
        "-o",
        partial.to_str().unwrap()
    ])
    .status
    .success());
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
    use rbench::*;
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
    assert!(page.contains("href=\"#run-0\""));
    assert_eq!(page.matches("class=\"run-card\"").count(), 7);
    assert!(!cli(&[
        "report",
        root.to_str().unwrap(),
        "-o",
        html.to_str().unwrap()
    ])
    .status
    .success());
    // Explicit collection-level correction: 6 identical pairs suffice alone but not for 7 runs.
    paired.observations.retain(|o| o.pair.unwrap() < 6);
    fs::write(
        root.join("regression/run.json"),
        serde_json::to_vec_pretty(&paired).unwrap(),
    )
    .unwrap();
    let out = t.path().join("uncertain.json");
    assert!(cli(&[
        "report",
        root.to_str().unwrap(),
        "-o",
        out.to_str().unwrap()
    ])
    .status
    .success());
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
    assert!(cli(&[
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
    .success());
    assert!(!out.exists());
    assert!(!cli(&[
        "run",
        "--memory",
        "--no-ui",
        "--program",
        "/bin/echo",
        "-o",
        path
    ])
    .status
    .success());
    let run = rbench::Run::load(&out).unwrap();
    assert_eq!(run.status, rbench::Status::Failed);
    assert!(run
        .notes
        .iter()
        .any(|n| n.contains("memory profile missing")));
}
