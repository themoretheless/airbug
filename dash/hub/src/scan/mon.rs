use std::{fs, path::PathBuf};

use crate::scan::{Action, Artifact, DomainCard, Status, human_bytes, mtime_detail};

pub(crate) fn scan_mon() -> DomainCard {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let db = PathBuf::from(home).join(".local/share/airbug-mon/airbug-mon.db");
    let mut artifacts = Vec::new();
    let (status, summary) = if db.is_file() {
        let meta = fs::metadata(&db).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        artifacts.push(Artifact {
            label: "airbug-mon.db".into(),
            path: db.display().to_string(),
            kind: "sqlite",
            detail: format!("{} · {}", human_bytes(size), mtime_detail(&db)),
        });
        (
            Status::Ready,
            format!("Host metrics database present ({})", human_bytes(size)),
        )
    } else {
        (
            Status::Missing,
            "No airbug-mon database yet. Launch the desktop monitor.".into(),
        )
    };

    DomainCard {
        id: "mon",
        title: "Mon",
        blurb: "Host metrics · airbug-mon (+ optional OTLP)",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Open airbug-mon".into(),
                command: "cargo run -p airbug-mon --release".into(),
            },
            Action {
                label: "Open + OTLP".into(),
                command: "cargo run -p airbug-mon --release -- --otlp".into(),
            },
            Action {
                label: "Headless sample".into(),
                command: "cargo run -p airbug-mon --release -- --headless --seconds 15".into(),
            },
            Action {
                label: "Headless + OTLP".into(),
                command: "cargo run -p airbug-mon --release -- --headless --seconds 15 --otlp"
                    .into(),
            },
        ],
    }
}
