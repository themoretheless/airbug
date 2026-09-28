use std::path::Path;

use crate::scan::{Action, Artifact, DomainCard, Status, UnitReportV1, mtime_detail, read_json};

/// Latest `cargo airbug test` run wins; the legacy `target/airbug-report` folder is the fallback.
pub(crate) fn scan_unit(root: &Path) -> DomainCard {
    let (status, summary, artifacts) = match crate::store::test_runs(root, 1).into_iter().next() {
        Some(run) => {
            let totals = run.totals.clone().unwrap_or_default();
            let commit = run
                .git
                .as_ref()
                .and_then(|g| g.short.clone())
                .map(|c| format!(" · {c}"))
                .unwrap_or_default();
            let summary = match run.state.as_str() {
                "building" | "running" if !run.stale => format!(
                    "{}: {}/{} done, {} failed so far{commit}",
                    run.state, totals.completed, totals.total, totals.failed
                ),
                _ => format!(
                    "{}{}: {} passed, {} failed, {} ignored, {} not run{commit}",
                    run.state,
                    if run.stale { " (stale)" } else { "" },
                    totals.passed,
                    totals.failed,
                    totals.ignored,
                    totals.not_run
                ),
            };
            let status = match run.state.as_str() {
                _ if run.stale => Status::Stale,
                "failed" | "error" => Status::Error,
                "interrupted" => Status::Stale,
                _ => Status::Ready,
            };
            let dir = crate::store::runs_dir(root).join(&run.run_id);
            let artifacts = vec![Artifact {
                label: format!("test run {}", &run.run_id[..8.min(run.run_id.len())]),
                path: dir.display().to_string(),
                kind: "dir",
                detail: mtime_detail(&dir.join("progress.json")),
            }];
            (status, summary, artifacts)
        }
        None => legacy_report(root),
    };

    DomainCard {
        id: "unit",
        title: "Unit",
        blurb: "cargo test · live per-test runs",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Run tests (live)".into(),
                command: "cargo airbug test -- --workspace --exclude airbug-mon".into(),
            },
            Action {
                label: "Open tests".into(),
                command: "open http://127.0.0.1:8790/#/tests".into(),
            },
        ],
    }
}

fn legacy_report(root: &Path) -> (Status, String, Vec<Artifact>) {
    let report_dir = root.join("target/airbug-report");
    let report_json = report_dir.join("report.json");
    let index_html = report_dir.join("index.html");
    let mut artifacts = Vec::new();
    let mut status = Status::Missing;
    let mut summary = "No test runs yet. Start one with `cargo airbug test`.".to_string();

    if report_json.is_file() {
        match read_json(&report_json).and_then(|value| {
            serde_json::from_value::<UnitReportV1>(value).map_err(|e| e.to_string())
        }) {
            Ok(report) => {
                let (passed, failed, broken) = report.counts();
                let title = report.title.as_deref().unwrap_or("Airbug report");
                let commit = report.commit.as_deref().unwrap_or("");
                summary = format!(
                    "{title}: {passed} passed, {failed} failed, {broken} broken{}",
                    if commit.is_empty() {
                        String::new()
                    } else {
                        format!(" · {commit}")
                    }
                );
                status = if failed + broken > 0 {
                    Status::Error
                } else {
                    Status::Ready
                };
                artifacts.push(Artifact {
                    label: "report.json".into(),
                    path: report_json.display().to_string(),
                    kind: "json",
                    detail: mtime_detail(&report_json),
                });
            }
            Err(err) => {
                status = Status::Error;
                summary = format!("Could not read report.json: {err}");
            }
        }
    }

    if index_html.is_file() {
        artifacts.push(Artifact {
            label: "index.html".into(),
            path: index_html.display().to_string(),
            kind: "html",
            detail: mtime_detail(&index_html),
        });
        if matches!(status, Status::Missing) {
            status = Status::Ready;
            summary = "HTML report present.".into();
        }
    }
    (status, summary, artifacts)
}
