use std::path::Path;

use crate::scan::{Action, Artifact, DomainCard, Status, UnitReportV1, mtime_detail, read_json};

pub(crate) fn scan_unit(root: &Path) -> DomainCard {
    let report_dir = root.join("target/airbug-report");
    let report_json = report_dir.join("report.json");
    let index_html = report_dir.join("index.html");
    let mut artifacts = Vec::new();
    let mut status = Status::Missing;
    let mut summary = "No report yet. Generate with the airbug report tool.".into();

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

    DomainCard {
        id: "unit",
        title: "Unit",
        blurb: "Test helpers · airbug reports",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Generate report".into(),
                command: "python3 unit/tools/airbug_report.py --all-features --locked".into(),
            },
            Action {
                label: "Run unit tests".into(),
                command: "cargo test -p airbug --all-features".into(),
            },
        ],
    }
}
