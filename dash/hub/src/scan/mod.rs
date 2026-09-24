//! Cross-domain status snapshot for the monorepo hub.
mod bench;
mod collector;
mod err;
mod mon;
mod otel;
mod unit;
mod unit_report;

pub use unit_report::UnitReportV1;

use serde::Serialize;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use bench::scan_bench;
use collector::scan_collector;
use err::scan_err;
use mon::scan_mon;
use otel::scan_otel;
use unit::scan_unit;

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub hub_id: String,
    pub root: String,
    pub generated: String,
    pub domains: Domains,
    /// Local endpoints that are up right now (hub + optional collector/Jaeger/report).
    pub apis: Vec<ApiEndpoint>,
}

#[derive(Debug, Serialize)]
pub struct ApiEndpoint {
    pub name: &'static str,
    pub method: &'static str,
    pub href: String,
    pub detail: String,
    pub available: bool,
}

#[derive(Debug, Serialize)]
pub struct Domains {
    pub unit: DomainCard,
    pub bench: DomainCard,
    pub mon: DomainCard,
    pub otel: DomainCard,
    pub err: DomainCard,
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

pub fn scan_with_hub(root: &Path, port: u16, hub_id: &str) -> Snapshot {
    let domains = Domains {
        unit: scan_unit(root),
        bench: scan_bench(root),
        mon: scan_mon(),
        otel: scan_otel(root),
        err: scan_err(root),
        collector: scan_collector(root),
    };
    Snapshot {
        hub_id: hub_id.to_string(),
        root: root.display().to_string(),
        generated: iso_now(),
        apis: local_apis(root, port, &domains),
        domains,
    }
}

pub(crate) fn local_apis(root: &Path, port: u16, _domains: &Domains) -> Vec<ApiEndpoint> {
    let base = format!("http://127.0.0.1:{port}");
    let probe = crate::collector::probe();
    let paths = crate::config::RootPaths::new(root);
    let report = paths.unit_report_index();
    vec![
        ApiEndpoint {
            name: "Hub status",
            method: "GET",
            href: format!("{base}/api/v1/status"),
            detail: "Domain cards JSON".into(),
            available: true,
        },
        ApiEndpoint {
            name: "Hub logs",
            method: "GET",
            href: format!("{base}/api/v1/logs?limit=150"),
            detail: "OTLP log tail JSON".into(),
            available: true,
        },
        ApiEndpoint {
            name: "Hub metrics",
            method: "GET",
            href: format!("{base}/api/v1/metrics?limit=200"),
            detail: "OTLP metric points JSON".into(),
            available: true,
        },
        ApiEndpoint {
            name: "Hub errors ingest",
            method: "POST",
            href: format!("{base}/api/v1/errors"),
            detail: "airbug-err event JSON".into(),
            available: true,
        },
        ApiEndpoint {
            name: "Hub issues",
            method: "GET",
            href: format!("{base}/api/v1/issues"),
            detail: "Grouped error issues".into(),
            available: true,
        },
        ApiEndpoint {
            name: "Hub API catalog",
            method: "GET",
            href: format!("{base}/api/v1"),
            detail: "This list as JSON".into(),
            available: true,
        },
        ApiEndpoint {
            name: "Unit HTML report",
            method: "GET",
            href: format!("{base}/report/index.html"),
            detail: "Served when target/airbug-report exists".into(),
            available: report.is_file(),
        },
        ApiEndpoint {
            name: "OTLP HTTP",
            method: "POST",
            href: "http://127.0.0.1:4318".into(),
            detail: "Collector :4318 (traces / metrics / logs)".into(),
            available: probe.otlp_http,
        },
        ApiEndpoint {
            name: "OTLP gRPC",
            method: "gRPC",
            href: "http://127.0.0.1:4317".into(),
            detail: "Collector :4317".into(),
            available: probe.otlp_grpc,
        },
        ApiEndpoint {
            name: "Jaeger UI",
            method: "GET",
            href: "http://127.0.0.1:16686/".into(),
            detail: "Local trace browser".into(),
            available: probe.jaeger_ui,
        },
    ]
}

pub(crate) fn find_run_json(store: &Path, limit: usize) -> Vec<PathBuf> {
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

pub(crate) fn run_detail(path: &Path) -> String {
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

pub(crate) fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub(crate) fn mtime_detail(path: &Path) -> String {
    match fs::metadata(path).and_then(|m| m.modified()) {
        Ok(time) => match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => format!("mtime {}", d.as_secs()),
            Err(_) => "mtime unknown".into(),
        },
        Err(_) => "mtime unknown".into(),
    }
}

pub(crate) fn human_bytes(n: u64) -> String {
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

pub(crate) fn iso_now() -> String {
    // Keep dependency-free: unix seconds is enough for the hub stamp.
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}s unix", d.as_secs()),
        Err(_) => "unknown".into(),
    }
}
