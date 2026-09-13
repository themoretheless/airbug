use std::path::Path;

use crate::config::RootPaths;
use crate::scan::{Action, Artifact, DomainCard, Status};

pub(crate) fn scan_collector(root: &Path) -> DomainCard {
    let paths = RootPaths::new(root);
    let compose = paths.compose_file();
    let config = paths.collector_config();
    let probe = crate::collector::probe();
    let mut artifacts = Vec::new();
    if compose.is_file() {
        artifacts.push(Artifact {
            label: "docker-compose.yml".into(),
            path: compose.display().to_string(),
            kind: "compose",
            detail: "otel-collector + jaeger".into(),
        });
    }
    if config.is_file() {
        artifacts.push(Artifact {
            label: "config.yaml".into(),
            path: config.display().to_string(),
            kind: "yaml",
            detail: "OTLP :4317 / :4318".into(),
        });
    }

    let (status, summary) = if probe.any_up() {
        let mut parts = Vec::new();
        if probe.otlp_http {
            parts.push("OTLP HTTP :4318");
        }
        if probe.otlp_grpc {
            parts.push("OTLP gRPC :4317");
        }
        if probe.jaeger_ui {
            parts.push("Jaeger :16686");
        }
        (Status::Ready, parts.join(" · "))
    } else if compose.is_file() {
        (
            Status::Missing,
            "Collector not running. Start hub with --collector (Docker or otelcol on PATH).".into(),
        )
    } else {
        (
            Status::Missing,
            "No dash/collector stack in this root.".into(),
        )
    };

    DomainCard {
        id: "collector",
        title: "Collector",
        blurb: "Local OTLP · Docker or otelcol binary",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Hub + collector".into(),
                command: "cargo run -p airbug-hub -- serve --root . --collector".into(),
            },
            Action {
                label: "Compose up (Docker)".into(),
                command: "docker compose -f dash/collector/docker-compose.yml up -d".into(),
            },
            Action {
                label: "Compose down".into(),
                command: "docker compose -f dash/collector/docker-compose.yml down".into(),
            },
            Action {
                label: "Open Jaeger".into(),
                command: "open http://127.0.0.1:16686/".into(),
            },
        ],
    }
}
