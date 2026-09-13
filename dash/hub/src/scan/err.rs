use std::path::Path;

use crate::config::RootPaths;
use crate::scan::{Action, Artifact, DomainCard, Status};

pub(crate) fn scan_err(root: &Path) -> DomainCard {
    let paths = RootPaths::new(root);
    let manifest = root.join("err/Cargo.toml");
    let readme = root.join("err/README.md");
    let db = paths.issues_db();
    let mut artifacts = Vec::new();
    if manifest.is_file() {
        artifacts.push(Artifact {
            label: "Cargo.toml".into(),
            path: manifest.display().to_string(),
            kind: "crate",
            detail: "airbug-err · panic/error → hub issues".into(),
        });
    }
    if readme.is_file() {
        artifacts.push(Artifact {
            label: "README.md".into(),
            path: readme.display().to_string(),
            kind: "md",
            detail: "domain docs".into(),
        });
    }
    if db.is_file() {
        artifacts.push(Artifact {
            label: "issues.sqlite".into(),
            path: db.display().to_string(),
            kind: "db",
            detail: "grouped issues store".into(),
        });
    }
    let ready = manifest.is_file();
    DomainCard {
        id: "err",
        title: "Err",
        blurb: "Error monitoring · airbug-err",
        status: if ready {
            Status::Ready
        } else {
            Status::Reserved
        },
        summary: if ready {
            "Panic/error SDK present; ingest via POST /api/errors.".into()
        } else {
            "No error-monitoring crate yet.".into()
        },
        artifacts,
        actions: if ready {
            vec![
                Action {
                    label: "Test airbug-err".into(),
                    command: "cargo test -p airbug-err".into(),
                },
                Action {
                    label: "Capture example".into(),
                    command: "cargo run -p airbug-err --example capture".into(),
                },
                Action {
                    label: "Panic example".into(),
                    command: "cargo run -p airbug-err --example panic".into(),
                },
            ]
        } else {
            vec![]
        },
    }
}
