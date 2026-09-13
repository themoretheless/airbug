use std::path::Path;

use crate::scan::{Action, Artifact, DomainCard, Status};

pub(crate) fn scan_otel(root: &Path) -> DomainCard {
    let manifest = root.join("otel/Cargo.toml");
    let readme = root.join("otel/README.md");
    let mut artifacts = Vec::new();
    if manifest.is_file() {
        artifacts.push(Artifact {
            label: "Cargo.toml".into(),
            path: manifest.display().to_string(),
            kind: "crate",
            detail: "airbug-otel · OTLP traces+metrics+logs".into(),
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
    let ready = manifest.is_file();
    DomainCard {
        id: "otel",
        title: "Otel",
        blurb: "OpenTelemetry traces+metrics+logs · airbug-otel",
        status: if ready {
            Status::Ready
        } else {
            Status::Reserved
        },
        summary: if ready {
            "OTLP traces, metrics, and logs helper crate present.".into()
        } else {
            "No OpenTelemetry crate yet.".into()
        },
        artifacts,
        actions: if ready {
            vec![
                Action {
                    label: "Test airbug-otel".into(),
                    command: "cargo test -p airbug-otel".into(),
                },
                Action {
                    label: "Run span example".into(),
                    command: "cargo run -p airbug-otel --example span".into(),
                },
                Action {
                    label: "Run metrics example".into(),
                    command: "cargo run -p airbug-otel --example metrics".into(),
                },
                Action {
                    label: "Run logs example".into(),
                    command: "cargo run -p airbug-otel --example logs".into(),
                },
            ]
        } else {
            vec![]
        },
    }
}
