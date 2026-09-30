#![cfg(feature = "json")]
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn reporter_preserves_outcomes_diagnostics_history_and_exit_codes() {
    let root = Temp(std::env::temp_dir().join(format!(
            "airbug-reporter-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir_all(root.0.join("src")).unwrap();
    let airbug_path = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
    fs::write(root.0.join("Cargo.toml"), format!("[package]\nname='reporter-fixture'\nversion='0.0.0'\nedition='2024'\n[workspace]\n[dev-dependencies]\nairbug={{path={airbug_path:?}}}\n")).unwrap();
    fs::write(root.0.join("src/lib.rs"), r#"
#[test] fn passes_with_steps() { airbug::report::step("prepare <value>", || { airbug::report::attach_text("payload.txt", "hello").unwrap(); airbug::report::assert_equal("value", &1, &1); }); }
#[test] fn fails() { panic!("expected fixture failure"); }
#[test] #[ignore] fn ignored() { panic!("must not run"); }
#[test] #[should_panic(expected="expected panic")] fn expected_panic() { panic!("expected panic"); }
#[test] fn timeout() { std::thread::sleep(std::time::Duration::from_secs(3)); }
"#).unwrap();
    let invoke = |filter: Option<&str>| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_airbug_report"));
        command.args(["--root", root.0.to_str().unwrap(), "--timeout-seconds", "1"]);
        if let Some(filter) = filter {
            command.args(["--filter", filter]);
        }
        command.args(["--", "--offline", "--lib"]).output().unwrap()
    };
    let first = invoke(None);
    assert!(
        !first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let output = root.0.join("target/airbug-report");
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["state"], "failed", "{report}");
    let cases = report["tests"].as_array().unwrap();
    assert_eq!(cases.len(), 5);
    for (name, status) in [
        ("passes_with_steps", "passed"),
        ("fails", "failed"),
        ("ignored", "ignored"),
        ("expected_panic", "passed"),
        ("timeout", "broken"),
    ] {
        assert_eq!(
            cases.iter().find(|c| c["name"] == name).unwrap()["status"],
            status,
            "{report}"
        );
    }
    let index = cases
        .iter()
        .position(|c| c["name"] == "passes_with_steps")
        .unwrap();
    let events = output
        .join("runs")
        .join(report["id"].as_str().unwrap())
        .join(index.to_string())
        .join("events.jsonl");
    let events = fs::read_to_string(events).unwrap();
    assert!(
        events.contains("step_start")
            && events.contains("attachment")
            && events.contains("comparison")
    );
    let second = invoke(Some("passes_with_steps"));
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(fs::read_dir(output.join("runs")).unwrap().count(), 2);
    // Build errors must still produce a terminal, inspectable record and nonzero exit.
    fs::write(root.0.join("src/lib.rs"), "not valid rust").unwrap();
    let third = invoke(None);
    assert!(!third.status.success());
    let broken: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(broken["state"], "failed");
    assert!(
        broken["message"]
            .as_str()
            .unwrap()
            .contains("Compilation failed")
    );
}
