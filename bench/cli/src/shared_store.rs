//! Announce a bench session in the shared run store (`.airbug/runs/`), so the hub lists it
//! next to test runs without scanning for it.
//!
//! Only a manifest and a progress file are written, before the first and after the last
//! measured process — nothing here runs on the measured path. The session's own files stay
//! where `--output` put them; the manifest points at that directory. Best effort: a store that
//! cannot be written never fails a benchmark. `AIRBUG_DASHBOARD=0` turns it off.
use airbug_bench::Result;
use airbug_run::{
    model::{KIND_BENCH, MANIFEST_SCHEMA, Manifest, Progress, RunState, Totals, now_ms},
    store::{self, MANIFEST_FILE, PROGRESS_FILE},
};
use std::path::{Path, PathBuf};

pub struct Announced {
    dir: PathBuf,
    run_id: String,
    started_at_ms: u64,
}

fn enabled() -> bool {
    std::env::var("AIRBUG_DASHBOARD").map_or(true, |v| v != "0")
}

pub fn announce(out: &Path) -> Option<Announced> {
    if !enabled() {
        return None;
    }
    let root = std::env::current_dir().ok()?;
    announce_in(&root, out, std::env::args().collect())
}

fn announce_in(root: &Path, out: &Path, command: Vec<String>) -> Option<Announced> {
    let started_at_ms = now_ms();
    let run_id = format!("bench-{started_at_ms}-{}", std::process::id());
    let dir = store::run_dir(root, &run_id);
    std::fs::create_dir_all(&dir).ok()?;
    let artifacts = std::path::absolute(out).ok()?;
    let manifest = Manifest {
        schema: MANIFEST_SCHEMA.into(),
        kind: KIND_BENCH.into(),
        run_id: run_id.clone(),
        title: out
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "bench".into()),
        command,
        cwd: root.display().to_string(),
        started_at_ms,
        artifacts: Some(artifacts.display().to_string()),
        ..Manifest::default()
    };
    store::write_json(&dir.join(MANIFEST_FILE), &manifest, true).ok()?;
    let announced = Announced {
        dir,
        run_id,
        started_at_ms,
    };
    announced.write(RunState::Running, Totals::default(), None);
    Some(announced)
}

impl Announced {
    fn write(&self, state: RunState, totals: Totals, finished_at_ms: Option<u64>) {
        let progress = Progress {
            run_id: self.run_id.clone(),
            kind: KIND_BENCH.into(),
            state,
            totals,
            updated_at_ms: finished_at_ms.unwrap_or(self.started_at_ms),
            finished_at_ms,
            duration_s: finished_at_ms
                .map(|end| end.saturating_sub(self.started_at_ms) as f64 / 1000.0),
            pid: std::process::id(),
            ..Progress::default()
        };
        let _ = store::write_json(&self.dir.join(PROGRESS_FILE), &progress, false);
    }

    /// Record how the session ended, from the files the runner left in `out`.
    ///
    /// A single run keeps `status-final.json` and `progress.json` in `out`; a matrix or a
    /// multi-target session keeps them one level down, one directory per cell. `ok` is the
    /// command's own result and decides when the files say nothing.
    pub fn finish(self, out: &Path, ok: bool) {
        let (states, totals) = outcome(out);
        if states.is_empty() && totals.total == 0 && !ok {
            // Rejected before anything was measured: not a run worth listing.
            let _ = std::fs::remove_dir_all(&self.dir);
            return;
        }
        let any = |name: &str| states.iter().any(|s| s == name);
        let state = if any("cancelled") {
            RunState::Interrupted
        } else if any("failed") || any("incomplete") || !ok {
            if states.is_empty() {
                RunState::Error
            } else {
                RunState::Failed
            }
        } else {
            RunState::Passed
        };
        self.write(state, totals, Some(now_ms()));
    }
}

/// Final states and summed process counts of a session directory and its direct children.
fn outcome(out: &Path) -> (Vec<String>, Totals) {
    let mut dirs = vec![out.to_path_buf()];
    if let Ok(entries) = std::fs::read_dir(out) {
        dirs.extend(entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()));
    }
    let mut states = vec![];
    let mut totals = Totals::default();
    for dir in dirs {
        let json = |name: &str| store::read_json::<serde_json::Value>(&dir.join(name));
        if let Some(state) = json("status-final.json")
            .as_ref()
            .and_then(|v| v["state"].as_str())
        {
            states.push(state.to_string());
        }
        if let Some(p) = json("progress.json") {
            totals.total += p["total"].as_u64().unwrap_or(0) as usize;
            totals.completed += p["completed"].as_u64().unwrap_or(0) as usize;
        }
    }
    (states, totals)
}

/// Run `body` as one announced session writing into `out`.
pub fn session<T>(out: &Path, body: impl FnOnce() -> Result<T>) -> Result<T> {
    let announced = announce(out);
    let result = body();
    if let Some(announced) = announced {
        announced.finish(out, result.is_ok());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_is_listed_as_a_bench_run_and_ends_with_the_runner_state() {
        let root = tempfile::tempdir().unwrap();
        let out = root.path().join("session");
        std::fs::create_dir_all(&out).unwrap();
        let announced = announce_in(root.path(), &out, vec!["cargo-airbug-bench".into()]).unwrap();
        let runs = store::list(&store::runs_dir(root.path()), KIND_BENCH);
        assert_eq!(runs.len(), 1);
        let (dir, manifest) = &runs[0];
        assert_eq!(manifest.title, "session");
        assert_eq!(manifest.artifacts.as_deref(), Some(out.to_str().unwrap()));
        assert_eq!(store::progress(dir).unwrap().state, RunState::Running);

        std::fs::write(
            out.join("status-final.json"),
            r#"{"state":"cancelled","error":null}"#,
        )
        .unwrap();
        std::fs::write(
            out.join("progress.json"),
            r#"{"state":"running","completed":3,"total":8,"variant":"candidate"}"#,
        )
        .unwrap();
        announced.finish(&out, false);
        let progress = store::progress(dir).unwrap();
        assert_eq!(progress.state, RunState::Interrupted);
        assert_eq!((progress.totals.completed, progress.totals.total), (3, 8));
        assert!(progress.duration_s.is_some());
    }

    #[test]
    fn a_matrix_sums_its_cells_and_a_rejected_call_leaves_no_run() {
        let root = tempfile::tempdir().unwrap();
        let out = root.path().join("matrix");
        for (cell, done) in [("0", 4), ("1", 2)] {
            let dir = out.join(cell);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("status-final.json"), r#"{"state":"complete"}"#).unwrap();
            std::fs::write(
                dir.join("progress.json"),
                format!(r#"{{"completed":{done},"total":4}}"#),
            )
            .unwrap();
        }
        let announced = announce_in(root.path(), &out, vec![]).unwrap();
        announced.finish(&out, true);
        let runs = store::list(&store::runs_dir(root.path()), KIND_BENCH);
        let progress = store::progress(&runs[0].0).unwrap();
        assert_eq!(progress.state, RunState::Passed);
        assert_eq!((progress.totals.completed, progress.totals.total), (6, 8));

        let rejected = announce_in(root.path(), &root.path().join("absent"), vec![]).unwrap();
        let dir = rejected.dir.clone();
        rejected.finish(&root.path().join("absent"), false);
        assert!(!dir.exists());
    }
}
