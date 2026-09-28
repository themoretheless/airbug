//! "Rerun failed" and "Run again" for test runs.
//!
//! The runner records how it was started (`runner`, `cargo_args`, `test_args` in
//! `manifest.json`), so the hub can start `airbug-hub test` again with the same arguments —
//! narrowed to the failed and not-run tests when asked. The new run is an ordinary run
//! directory; the hub only picks its id so the dashboard can open it right away.
use crate::{
    error::{HubError, Result},
    store,
    testrun::model::{Manifest, RUNNER_NEXTEST, TestStatus},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
};

/// More exact filters than this make command lines too long (Windows caps at 32k chars).
const MAX_FILTERS: usize = 300;
const RERUN_SUFFIX: &str = " · rerun failed";

/// libtest options whose next argument is their value (kept, not mistaken for a filter).
const LIBTEST_VALUE_FLAGS: [&str; 7] = [
    "--test-threads",
    "--skip",
    "--color",
    "--format",
    "--logfile",
    "--shuffle-seed",
    "-Z",
];
/// nextest options that select tests; a failed-only rerun replaces them.
const NEXTEST_FILTER_FLAGS: [&str; 3] = ["-E", "--filterset", "--filter-expr"];

#[derive(Debug, Default, Deserialize)]
struct LightReport {
    #[serde(default)]
    suites: Vec<LightSuite>,
    #[serde(default)]
    tests: Vec<LightTest>,
}

#[derive(Debug, Default, Deserialize)]
struct LightSuite {
    id: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct LightTest {
    suite: String,
    name: String,
    #[serde(default)]
    status: TestStatus,
}

/// What a rerun would run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Plan {
    pub title: String,
    pub nextest: bool,
    pub cargo_args: Vec<String>,
    pub test_args: Vec<String>,
    /// Tests selected (failed-only), 0 for a full rerun.
    pub tests: usize,
    /// More failures than [`MAX_FILTERS`]; only the first ones are selected.
    pub truncated: bool,
}

impl Plan {
    /// The same run from a terminal.
    pub fn command(&self) -> String {
        let mut parts = vec!["cargo".to_string(), "airbug".into(), "test".into()];
        if self.nextest {
            parts.push("--nextest".into());
        }
        parts.push("--title".into());
        parts.push(self.title.clone());
        parts.push("--".into());
        parts.extend(self.cargo_args.iter().cloned());
        if !self.test_args.is_empty() {
            parts.push("--".into());
            parts.extend(self.test_args.iter().cloned());
        }
        parts
            .iter()
            .map(|p| shell_quote(p))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Build the rerun of `manifest` (+ its report) — `failed_only` narrows to failed and
/// not-run tests.
fn plan_from(manifest: &Manifest, report: &LightReport, failed_only: bool) -> Result<Plan> {
    if manifest.runner.is_empty() {
        return Err(HubError::msg(
            "this run was recorded before reruns were supported; start it from the terminal",
        ));
    }
    let nextest = manifest.runner == RUNNER_NEXTEST;
    let base_title = manifest
        .title
        .strip_suffix(RERUN_SUFFIX)
        .unwrap_or(&manifest.title);
    if !failed_only {
        return Ok(Plan {
            title: base_title.to_string(),
            nextest,
            cargo_args: manifest.cargo_args.clone(),
            test_args: manifest.test_args.clone(),
            tests: 0,
            truncated: false,
        });
    }
    let mut picked: Vec<&LightTest> = report
        .tests
        .iter()
        .filter(|t| matches!(t.status, TestStatus::Failed | TestStatus::NotRun))
        .collect();
    if picked.is_empty() {
        return Err(HubError::msg("nothing failed in this run"));
    }
    let truncated = picked.len() > MAX_FILTERS;
    picked.truncate(MAX_FILTERS);
    let (cargo_args, test_args) = if nextest {
        let expr = picked
            .iter()
            .map(|t| {
                let binary = report
                    .suites
                    .iter()
                    .find(|s| s.id == t.suite)
                    .map(|s| s.name.as_str())
                    .filter(|n| !n.is_empty())
                    .unwrap_or(t.suite.as_str());
                format!("(binary_id(={binary}) & test(={}))", t.name)
            })
            .collect::<Vec<_>>()
            .join(" | ");
        let mut cargo_args = drop_flags(&manifest.cargo_args, &NEXTEST_FILTER_FLAGS);
        cargo_args.push("-E".into());
        cargo_args.push(expr);
        (cargo_args, flags_only(&manifest.test_args))
    } else {
        let mut test_args = flags_only(&manifest.test_args);
        test_args.retain(|a| a != "--exact");
        test_args.push("--exact".into());
        let mut names: Vec<&str> = picked.iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        test_args.extend(names.into_iter().map(str::to_string));
        (manifest.cargo_args.clone(), test_args)
    };
    Ok(Plan {
        title: format!("{base_title}{RERUN_SUFFIX}"),
        nextest,
        cargo_args,
        test_args,
        tests: picked.len(),
        truncated,
    })
}

/// Options of the test binaries, without positional filters (a failed-only rerun brings its
/// own exact names).
fn flags_only(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut take_value = false;
    for arg in args {
        if take_value {
            out.push(arg.clone());
            take_value = false;
        } else if arg.starts_with('-') {
            take_value = LIBTEST_VALUE_FLAGS.contains(&arg.as_str());
            out.push(arg.clone());
        }
    }
    out
}

/// `args` without `flags` (and their values, separate or `--flag=value`).
fn drop_flags(args: &[String], flags: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if flags.contains(&arg.as_str()) {
            skip = true;
            continue;
        }
        if flags
            .iter()
            .any(|f| arg.strip_prefix(f).is_some_and(|r| r.starts_with('=')))
        {
            continue;
        }
        out.push(arg.clone());
    }
    out
}

fn shell_quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=@,+%".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

fn load(root: &Path, id: &str) -> Result<(String, Manifest, LightReport)> {
    let id = store::resolve_test_run(root, id)?;
    let dir = store::runs_dir(root).join(&id);
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(dir.join("manifest.json")).map_err(|_| HubError::msg("not found"))?,
    )?;
    let report: LightReport = fs::read(dir.join("report.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    Ok((id, manifest, report))
}

/// Rerun info for the dashboard: counts and terminal commands (no process is started).
pub fn describe(manifest: &Value, report: &Value) -> Value {
    let Ok(manifest) = Manifest::deserialize(manifest) else {
        return json!({ "supported": false });
    };
    let report = LightReport::deserialize(report).unwrap_or_default();
    let all = plan_from(&manifest, &report, false);
    let failed = plan_from(&manifest, &report, true);
    match all {
        Err(err) => json!({ "supported": false, "reason": err.to_string() }),
        Ok(all) => json!({
            "supported": true,
            "nextest": all.nextest,
            "command_all": all.command(),
            "failed": failed.as_ref().map(|p| p.tests).unwrap_or(0),
            "truncated": failed.as_ref().is_ok_and(|p| p.truncated),
            "command_failed": failed.ok().map(|p| p.command()),
        }),
    }
}

/// Start the rerun in the background; returns the new run id and what was started.
pub fn start(root: &Path, port: u16, id: &str, failed_only: bool) -> Result<Value> {
    if let Some(live) = store::test_runs(root, 20)
        .into_iter()
        .find(|r| !r.stale && matches!(r.state.as_str(), "building" | "running"))
    {
        return Err(HubError::msg(format!(
            "busy: test run {} is still running",
            live.run_id
        )));
    }
    let (id, manifest, report) = load(root, id)?;
    let plan = plan_from(&manifest, &report, failed_only)?;
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut args = vec![
        "test".to_string(),
        "--root".into(),
        root.display().to_string(),
        "--run-id".into(),
        run_id.clone(),
        "--title".into(),
        plan.title.clone(),
        "--rerun-of".into(),
        id.clone(),
        "--port".into(),
        port.to_string(),
        "--quiet".into(),
    ];
    if plan.nextest {
        args.push("--nextest".into());
    }
    args.push("--".into());
    args.extend(plan.cargo_args.iter().cloned());
    if !plan.test_args.is_empty() {
        args.push("--".into());
        args.extend(plan.test_args.iter().cloned());
    }
    let exe = std::env::current_exe()?;
    let mut child = Command::new(exe)
        .args(&args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    // Reap it; the run directory is the only interface.
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(json!({
        "run_id": run_id,
        "rerun_of": id,
        "href": format!("#/tests/{run_id}"),
        "tests": plan.tests,
        "truncated": plan.truncated,
        "command": plan.command(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testrun::model::RUNNER_CARGO_TEST;

    fn manifest(runner: &str) -> Manifest {
        Manifest {
            title: "cargo test --workspace".into(),
            runner: runner.into(),
            cargo_args: vec!["--workspace".into(), "-E".into(), "all()".into()],
            test_args: vec![
                "--test-threads".into(),
                "2".into(),
                "slow".into(),
                "--include-ignored".into(),
            ],
            ..Manifest::default()
        }
    }

    fn report() -> LightReport {
        let test = |suite: &str, name: &str, status| LightTest {
            suite: suite.into(),
            name: name.into(),
            status,
        };
        LightReport {
            suites: vec![
                LightSuite {
                    id: "demo (src/lib.rs)".into(),
                    name: "demo".into(),
                },
                LightSuite {
                    id: "demo::it".into(),
                    name: "demo::it".into(),
                },
            ],
            tests: vec![
                test("demo (src/lib.rs)", "tests::ok", TestStatus::Passed),
                test("demo (src/lib.rs)", "tests::fails", TestStatus::Failed),
                test("demo::it", "crashed", TestStatus::NotRun),
                test("demo::it", "skipped", TestStatus::Ignored),
            ],
        }
    }

    #[test]
    fn libtest_rerun_keeps_flags_and_selects_exact_names() {
        let plan = plan_from(&manifest(RUNNER_CARGO_TEST), &report(), true).unwrap();
        assert_eq!(plan.title, "cargo test --workspace · rerun failed");
        assert_eq!(plan.cargo_args, ["--workspace", "-E", "all()"]);
        assert_eq!(
            plan.test_args,
            [
                "--test-threads",
                "2",
                "--include-ignored",
                "--exact",
                "crashed",
                "tests::fails"
            ]
        );
        assert_eq!(plan.tests, 2);
        let again = Manifest {
            title: plan.title.clone(),
            ..manifest(RUNNER_CARGO_TEST)
        };
        let replan = plan_from(&again, &report(), true).unwrap();
        assert_eq!(replan.title, plan.title, "suffix does not stack");
        assert_eq!(
            plan.command(),
            "cargo airbug test --title 'cargo test --workspace · rerun failed' -- --workspace -E \
             'all()' -- --test-threads 2 --include-ignored --exact crashed tests::fails"
        );
    }

    #[test]
    fn nextest_rerun_uses_a_filterset() {
        let plan = plan_from(&manifest(RUNNER_NEXTEST), &report(), true).unwrap();
        assert!(plan.nextest);
        assert_eq!(
            plan.cargo_args,
            [
                "--workspace",
                "-E",
                "(binary_id(=demo) & test(=tests::fails)) | (binary_id(=demo::it) & test(=crashed))"
            ]
        );
        assert_eq!(plan.test_args, ["--test-threads", "2", "--include-ignored"]);
    }

    #[test]
    fn full_rerun_and_refusals() {
        let plan = plan_from(&manifest(RUNNER_CARGO_TEST), &report(), false).unwrap();
        assert_eq!(plan.test_args, manifest(RUNNER_CARGO_TEST).test_args);
        assert_eq!(plan.tests, 0);
        assert!(
            plan_from(&manifest(""), &report(), false).is_err(),
            "old run"
        );
        let green = LightReport {
            tests: vec![LightTest {
                suite: "s".into(),
                name: "t".into(),
                status: TestStatus::Passed,
            }],
            ..LightReport::default()
        };
        assert!(plan_from(&manifest(RUNNER_CARGO_TEST), &green, true).is_err());
        let described = describe(
            &serde_json::to_value(manifest(RUNNER_CARGO_TEST)).unwrap(),
            &json!({ "tests": [{ "suite": "s", "name": "t", "status": "failed" }] }),
        );
        assert_eq!(described["supported"], true);
        assert_eq!(described["failed"], 1);
        assert!(
            described["command_failed"]
                .as_str()
                .unwrap()
                .ends_with("--exact t")
        );
    }

    #[test]
    fn quoting_and_flag_dropping() {
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
        assert_eq!(shell_quote("--workspace"), "--workspace");
        assert_eq!(
            drop_flags(
                &["-E".into(), "x".into(), "--filterset=y".into(), "-p".into()],
                &NEXTEST_FILTER_FLAGS
            ),
            ["-p"]
        );
    }
}
