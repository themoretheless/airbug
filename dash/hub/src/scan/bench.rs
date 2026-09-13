use std::path::Path;

use crate::scan::{
    find_run_json, mtime_detail, rel_label, run_detail, Action, Artifact, DomainCard, Status,
};

pub(crate) fn scan_bench(root: &Path) -> DomainCard {
    let store = root.join(".airbug-bench");
    let mut artifacts = Vec::new();
    let mut status = Status::Missing;
    let mut summary = "No .bench store yet.".into();

    if store.is_dir() {
        let runs = find_run_json(&store, 12);
        if runs.is_empty() {
            status = Status::Stale;
            summary = "Store exists, but no run.json found.".into();
        } else {
            status = Status::Ready;
            summary = format!("{} recent run artifact(s) under .bench", runs.len());
            for path in runs {
                let detail = run_detail(&path);
                artifacts.push(Artifact {
                    label: rel_label(root, &path),
                    path: path.display().to_string(),
                    kind: "json",
                    detail,
                });
            }
        }
        artifacts.insert(
            0,
            Artifact {
                label: ".airbug-bench".into(),
                path: store.display().to_string(),
                kind: "dir",
                detail: mtime_detail(&store),
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
                label: "Serve bench UI".into(),
                command: "cargo airbug-bench serve .bench --port 8787".into(),
            },
            Action {
                label: "Run workloads".into(),
                command: "cargo airbug-bench run -p airbug-bench --bench workloads -o .airbug-bench/session".into(),
            },
        ],
    }
}
