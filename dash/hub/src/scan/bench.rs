use crate::scan::{
    Action, Artifact, DomainCard, Status, find_run_json, mtime_detail, rel_label, run_detail,
};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct BenchRuns {
    pub store: String,
    pub runs: Vec<BenchRunRow>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct BenchRunRow {
    pub id: String,
    pub rel: String,
    pub run_url: String,
    pub report_url: Option<String>,
    pub detail: String,
}

pub(crate) fn list_runs(root: &Path) -> BenchRuns {
    let store = root.join(".airbug-bench");
    if !store.is_dir() {
        return BenchRuns {
            store: store.display().to_string(),
            runs: Vec::new(),
            note: "No .airbug-bench store yet.".into(),
        };
    }
    let runs = find_run_json(&store, 24)
        .into_iter()
        .filter_map(|path| {
            let dir = path.parent()?;
            let rel_run = path.strip_prefix(root).ok()?.display().to_string();
            // strip ".airbug-bench/" for /bench/<rel>
            let under = path.strip_prefix(&store).ok()?.display().to_string();
            let report = dir.join("report.html");
            let report_url = report.is_file().then(|| {
                format!(
                    "/bench/{}",
                    report.strip_prefix(&store).unwrap().display()
                )
            });
            let id = dir
                .strip_prefix(&store)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| dir.display().to_string());
            Some(BenchRunRow {
                id,
                rel: rel_run,
                run_url: format!("/bench/{under}"),
                report_url,
                detail: run_detail(&path),
            })
        })
        .collect::<Vec<_>>();
    let note = if runs.is_empty() {
        "Store exists, but no run.json found.".into()
    } else {
        format!("{} recent run(s)", runs.len())
    };
    BenchRuns {
        store: store.display().to_string(),
        runs,
        note,
    }
}

pub(crate) fn scan_bench(root: &Path) -> DomainCard {
    let listed = list_runs(root);
    let mut artifacts = Vec::new();
    let mut status = Status::Missing;
    let mut summary = listed.note.clone();

    if Path::new(&listed.store).is_dir() {
        if listed.runs.is_empty() {
            status = Status::Stale;
        } else {
            status = Status::Ready;
            for row in &listed.runs {
                artifacts.push(Artifact {
                    label: row.rel.clone(),
                    path: root.join(&row.rel).display().to_string(),
                    kind: "json",
                    detail: row.detail.clone(),
                });
            }
        }
        artifacts.insert(
            0,
            Artifact {
                label: ".airbug-bench".into(),
                path: listed.store.clone(),
                kind: "dir",
                detail: mtime_detail(Path::new(&listed.store)),
            },
        );
    }

    DomainCard {
        id: "bench",
        title: "Bench",
        blurb: "Microbenchmarks · bench store",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Open charts".into(),
                command: "open http://127.0.0.1:8790/#/bench".into(),
            },
            Action {
                label: "Serve bench UI".into(),
                command: "cargo airbug-bench serve .airbug-bench --port 8787".into(),
            },
            Action {
                label: "Run workloads".into(),
                command: "cargo airbug-bench run -p airbug-bench --bench workloads -o .airbug-bench/session".into(),
            },
        ],
    }
}
