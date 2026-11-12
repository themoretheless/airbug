//! One description for any run, whatever wrote it: `GET /api/v1/run?id=<id>`.
//!
//! The run page asks this first and then knows which files and feeds to read. A test run is
//! described by its window only — its results come from `/api/v1/runs/<id>`. A bench run may be
//! announced in the run store, registered with the hub as a GUID session, or just a directory
//! under `.airbug-bench`; all three end up as a directory plus the URL prefix that serves it.
use crate::{config::RootPaths, runs::RunStore};
use airbug_run::{
    model::{KIND_BENCH, Manifest},
    store::{manifest, progress, read_json, runs_dir},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

const MAX_CELLS: usize = 256;
const MAX_LOGS: usize = 400;

/// `#/run/<id>` with the id kept as one path segment.
pub fn href(id: &str) -> String {
    let mut out = String::from("#/run/");
    for b in id.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn ms(time: std::io::Result<SystemTime>) -> u64 {
    time.ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn mtime(path: &Path) -> u64 {
    ms(fs::metadata(path).and_then(|m| m.modified()))
}

fn safe_rel(id: &str) -> bool {
    !id.is_empty()
        && !id.contains('\\')
        && id
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

struct Bench {
    dir: PathBuf,
    /// URL prefix that serves `dir`.
    files: String,
    title: String,
    started_at_ms: u64,
    /// Set when OTLP signals carry `airbug.run_id` for this run.
    signals_run_id: Option<String>,
    /// State recorded outside the directory (run store), if any.
    recorded: Option<(String, Option<u64>)>,
}

fn locate(root: &Path, registry: &RunStore, id: &str) -> Option<(Option<Manifest>, Option<Bench>)> {
    if !safe_rel(id) {
        return None;
    }
    if !id.contains('/')
        && let Some(m) = manifest(&runs_dir(root).join(id))
    {
        if m.kind != KIND_BENCH {
            return Some((Some(m), None));
        }
        let state = progress(&runs_dir(root).join(id)).map(|p| {
            let name = serde_json::to_value(p.state)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            (name, p.finished_at_ms)
        });
        let bench = Bench {
            dir: PathBuf::from(m.artifacts.clone()?),
            files: format!("/runs/{id}/files/"),
            title: m.title.clone(),
            started_at_ms: m.started_at_ms,
            signals_run_id: None,
            recorded: state,
        };
        return Some((Some(m), Some(bench)));
    }
    let (dir, rel, title, signals) = match registry.get(id) {
        Ok(detail) => (
            RootPaths::new(root).bench_run_dir(id),
            format!("runs/{id}"),
            detail.title,
            Some(id.to_string()),
        ),
        Err(_) => (
            root.join(".airbug-bench").join(id),
            id.to_string(),
            id.to_string(),
            None,
        ),
    };
    dir.is_dir().then(|| {
        let started = ["plan.json", "matrix.json", "targets.json", "git.json"]
            .iter()
            .map(|f| mtime(&dir.join(f)))
            .find(|t| *t > 0)
            .unwrap_or_else(|| ms(fs::metadata(&dir).and_then(|m| m.created())));
        (
            None,
            Some(Bench {
                files: format!("/bench/{rel}/"),
                dir,
                title,
                started_at_ms: started,
                signals_run_id: signals,
                recorded: None,
            }),
        )
    })
}

/// The variant a running directory is measuring right now.
fn variant(dir: &Path) -> Option<String> {
    if dir.join("status-final.json").is_file() {
        return None;
    }
    read_json::<Value>(&dir.join("progress.json"))?["variant"]
        .as_str()
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// State, counts and last change of one runner directory.
fn observe(dir: &Path) -> (Option<String>, u64, u64, u64) {
    let read = |name: &str| read_json::<Value>(&dir.join(name));
    let last = read("status-final.json");
    let live = read("progress.json");
    let count = |key: &str| live.as_ref().and_then(|p| p[key].as_u64()).unwrap_or(0);
    let state = match (&last, &live) {
        (Some(v), _) => v["state"].as_str().map(str::to_string),
        (None, Some(_)) => Some("running".into()),
        (None, None) => dir.join("run.json").is_file().then(|| "complete".into()),
    };
    let at = ["status-final.json", "progress.json", "run.json"]
        .iter()
        .map(|f| mtime(&dir.join(f)))
        .max()
        .unwrap_or(0);
    (state, count("completed"), count("total"), at)
}

fn log_files(dir: &Path, prefix: &str, out: &mut Vec<Value>) {
    let Ok(entries) = fs::read_dir(dir.join("logs")) else {
        return;
    };
    let mut names: Vec<(String, u64)> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| {
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            (e.file_name().to_string_lossy().into_owned(), size)
        })
        .collect();
    names.sort();
    for (name, size) in names {
        if out.len() >= MAX_LOGS {
            return;
        }
        out.push(json!({ "path": format!("{prefix}logs/{name}"), "bytes": size }));
    }
}

fn cells(dir: &Path) -> Vec<Value> {
    let axes: Vec<Value> = read_json(&dir.join("matrix.json")).unwrap_or_default();
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir() && e.file_name() != "logs")
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    // Matrix cells are numbered; keep them in plan order.
    names.sort_by_key(|n| (n.parse::<usize>().unwrap_or(usize::MAX), n.clone()));
    names
        .into_iter()
        .take(MAX_CELLS)
        .filter_map(|name| {
            let cell = dir.join(&name);
            let (state, completed, total, _) = observe(&cell);
            let state = state?;
            let axis = name
                .parse::<usize>()
                .ok()
                .and_then(|i| axes.get(i))
                .cloned();
            Some(json!({
                "name": name,
                "axes": axis,
                "state": state,
                "completed": completed,
                "total": total,
                "has_run_json": cell.join("run.json").is_file(),
            }))
        })
        .collect()
}

fn aggregate(cells: &[Value]) -> Option<String> {
    let has = |s: &str| cells.iter().any(|c| c["state"] == s);
    if cells.is_empty() {
        None
    } else if has("running") {
        Some("running".into())
    } else if has("cancelled") {
        Some("cancelled".into())
    } else if has("failed") || has("incomplete") {
        Some("failed".into())
    } else {
        Some("complete".into())
    }
}

pub fn describe(root: &Path, registry: &RunStore, id: &str) -> Option<Value> {
    let (manifest, bench) = locate(root, registry, id)?;
    let Some(bench) = bench else {
        let m = manifest?;
        let p = progress(&runs_dir(root).join(id));
        return Some(json!({
            "run_id": id,
            "kind": m.kind,
            "title": m.title,
            "command": m.command,
            "started_at_ms": m.started_at_ms,
            "finished_at_ms": p.as_ref().and_then(|p| p.finished_at_ms),
            "files": format!("/runs/{id}/"),
        }));
    };
    let (own_state, mut completed, mut total, mut at) = observe(&bench.dir);
    let cells = cells(&bench.dir);
    if total == 0 {
        for c in &cells {
            completed += c["completed"].as_u64().unwrap_or(0);
            total += c["total"].as_u64().unwrap_or(0);
        }
    }
    for c in &cells {
        at = at.max(mtime(
            &bench
                .dir
                .join(c["name"].as_str().unwrap_or(""))
                .join("progress.json"),
        ));
    }
    // The run store knows when a command ended; the directory knows how the runner ended.
    let finished = |s: &str| !matches!(s, "running" | "building");
    let recorded_end = bench
        .recorded
        .as_ref()
        .filter(|(s, _)| finished(s))
        .map(|(s, end)| (s.clone(), *end));
    let state = own_state
        .or_else(|| aggregate(&cells).filter(|s| s != "complete" || recorded_end.is_some()))
        .or_else(|| recorded_end.as_ref().map(|(s, _)| s.clone()))
        .unwrap_or_else(|| "running".into());
    let done = finished(&state);
    let mut logs = vec![];
    log_files(&bench.dir, "", &mut logs);
    for c in &cells {
        let name = c["name"].as_str().unwrap_or("");
        log_files(&bench.dir.join(name), &format!("{name}/"), &mut logs);
    }
    let has = |f: &str| bench.dir.join(f).is_file();
    // What is being measured now: the session itself, or its running cell.
    let now = variant(&bench.dir).or_else(|| {
        cells
            .iter()
            .filter(|c| c["state"] == "running")
            .find_map(|c| {
                let name = c["name"].as_str()?;
                let axes = c["axes"].as_object().map(|a| {
                    a.iter()
                        .map(|(k, v)| format!("{k} {}", v.as_str().unwrap_or("")))
                        .collect::<Vec<_>>()
                        .join(" ")
                });
                let what = variant(&bench.dir.join(name)).unwrap_or_default();
                Some(format!(
                    "{} · {what}",
                    axes.unwrap_or_else(|| name.to_string())
                ))
            })
    });
    Some(json!({
        "run_id": id,
        "kind": KIND_BENCH,
        "title": bench.title,
        "command": manifest.as_ref().map(|m| m.command.clone()),
        "state": state,
        "started_at_ms": bench.started_at_ms,
        "finished_at_ms": done.then(|| recorded_end.and_then(|(_, end)| end).unwrap_or(at)),
        "updated_at_ms": at,
        "progress": { "completed": completed, "total": total, "variant": now },
        "files": bench.files,
        "has_report_html": has("report.html"),
        "has_run_json": has("run.json"),
        "matrix": has("matrix.json"),
        "cells": cells,
        "logs": logs,
        "signals_run_id": bench.signals_run_id,
        "compare_id": bench.dir.strip_prefix(root.join(".airbug-bench")).ok()
            .map(|p| p.display().to_string().replace('\\', "/")),
    }))
}
