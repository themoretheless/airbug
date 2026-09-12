//! Cross-domain status snapshot for the monorepo hub.
use serde::Serialize;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub root: String,
    pub generated: String,
    pub domains: Domains,
}

#[derive(Debug, Serialize)]
pub struct Domains {
    pub unit: DomainCard,
    pub bench: DomainCard,
    pub mon: DomainCard,
    pub trace: DomainCard,
    pub collector: DomainCard,
}

#[derive(Debug, Serialize)]
pub struct DomainCard {
    pub id: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub status: Status,
    pub summary: String,
    pub artifacts: Vec<Artifact>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ready,
    Missing,
    Stale,
    Reserved,
    Error,
}

#[derive(Debug, Serialize)]
pub struct Artifact {
    pub label: String,
    pub path: String,
    pub kind: &'static str,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct Action {
    pub label: String,
    pub command: String,
}

pub fn scan(root: &Path) -> Snapshot {
    Snapshot {
        root: root.display().to_string(),
        generated: iso_now(),
        domains: Domains {
            unit: scan_unit(root),
            bench: scan_bench(root),
            mon: scan_mon(),
            trace: scan_trace(root),
            collector: scan_collector(root),
        },
    }
}

fn scan_unit(root: &Path) -> DomainCard {
    let report_dir = root.join("target/airbug-report");
    let report_json = report_dir.join("report.json");
    let index_html = report_dir.join("index.html");
    let mut artifacts = Vec::new();
    let mut status = Status::Missing;
    let mut summary = "No report yet. Generate with the airbug report tool.".into();

    if report_json.is_file() {
        match read_json(&report_json) {
            Ok(value) => {
                let tests = value.get("tests").and_then(|t| t.as_array());
                let (passed, failed, broken) = count_tests(tests);
                let title = value
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Airbug report");
                let commit = value
                    .get("commit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
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
                command: "python3 unit/tools/airbug_report.py --all-features --locked"
                    .into(),
            },
            Action {
                label: "Run unit tests".into(),
                command: "cargo test -p airbug --all-features".into(),
            },
        ],
    }
}

fn scan_bench(root: &Path) -> DomainCard {
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

fn scan_mon() -> DomainCard {
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
        blurb: "Host metrics · airbug-mon",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Open airbug-mon".into(),
                command: "cargo run -p airbug-mon --release".into(),
            },
            Action {
                label: "Headless sample".into(),
                command: "cargo run -p airbug-mon --release -- --headless --seconds 15".into(),
            },
        ],
    }
}

fn scan_trace(root: &Path) -> DomainCard {
    let manifest = root.join("trace/Cargo.toml");
    let readme = root.join("trace/README.md");
    let mut artifacts = Vec::new();
    if manifest.is_file() {
        artifacts.push(Artifact {
            label: "Cargo.toml".into(),
            path: manifest.display().to_string(),
            kind: "crate",
            detail: "airbug-trace · OTLP traces+metrics".into(),
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
        id: "trace",
        title: "Trace",
        blurb: "OpenTelemetry traces+metrics · airbug-trace",
        status: if ready {
            Status::Ready
        } else {
            Status::Reserved
        },
        summary: if ready {
            "OTLP trace and metric helper crate present.".into()
        } else {
            "No tracer crate yet — domain reserved for causal paths.".into()
        },
        artifacts,
        actions: if ready {
            vec![
                Action {
                    label: "Test airbug-trace".into(),
                    command: "cargo test -p airbug-trace".into(),
                },
                Action {
                    label: "Run span example".into(),
                    command: "cargo run -p airbug-trace --example span".into(),
                },
                Action {
                    label: "Run metrics example".into(),
                    command: "cargo run -p airbug-trace --example metrics".into(),
                },
            ]
        } else {
            vec![]
        },
    }
}

fn scan_collector(root: &Path) -> DomainCard {
    let compose = root.join("dash/collector/docker-compose.yml");
    let config = root.join("dash/collector/config.yaml");
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
            "Collector not running. Start hub with --collector or compose up.".into(),
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
        blurb: "Local OTLP collector + Jaeger UI",
        status,
        summary,
        artifacts,
        actions: vec![
            Action {
                label: "Hub + collector".into(),
                command: "cargo run -p airbug-hub -- serve --root . --collector".into(),
            },
            Action {
                label: "Compose up".into(),
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

fn count_tests(tests: Option<&Vec<Value>>) -> (usize, usize, usize) {
    let Some(tests) = tests else {
        return (0, 0, 0);
    };
    let mut passed = 0;
    let mut failed = 0;
    let mut broken = 0;
    for test in tests {
        match test.get("status").and_then(|s| s.as_str()).unwrap_or("") {
            "passed" | "ok" => passed += 1,
            "failed" => failed += 1,
            "broken" => broken += 1,
            _ => {}
        }
    }
    (passed, failed, broken)
}

fn find_run_json(store: &Path, limit: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![store.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip heavy trees.
                if path.file_name().and_then(|n| n.to_str()) == Some("target") {
                    continue;
                }
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some("run.json") {
                found.push(path);
            }
        }
    }
    found.sort_by_key(|p| {
        fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    found.reverse();
    found.truncate(limit);
    found
}

fn run_detail(path: &Path) -> String {
    match read_json(path) {
        Ok(value) => {
            let status = value
                .pointer("/status")
                .or_else(|| value.get("status"))
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".into());
            format!("{status} · {}", mtime_detail(path))
        }
        Err(_) => mtime_detail(path),
    }
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn mtime_detail(path: &Path) -> String {
    match fs::metadata(path).and_then(|m| m.modified()) {
        Ok(time) => match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => format!("mtime {}", d.as_secs()),
            Err(_) => "mtime unknown".into(),
        },
        Err(_) => "mtime unknown".into(),
    }
}

fn rel_label(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{n} {}", UNITS[idx])
    } else {
        format!("{size:.1} {}", UNITS[idx])
    }
}

fn iso_now() -> String {
    // Keep dependency-free: unix seconds is enough for the hub stamp.
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}s unix", d.as_secs()),
        Err(_) => "unknown".into(),
    }
}
