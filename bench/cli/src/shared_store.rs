//! Announce a bench session in the shared run store (`.airbug/runs/`), so the hub lists it
//! next to test runs without scanning for it.
//!
//! Only a manifest and a progress file are written, before the first and after the last
//! measured process — nothing here runs on the measured path. The session's own files stay
//! where `--output` put them; the manifest points at that directory. Best effort: a store that
//! cannot be written never fails a benchmark. `AIRBUG_DASHBOARD=0` turns it off.
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
    pub fn finish(self, out: &Path) {
        let json = |name: &str| store::read_json::<serde_json::Value>(&out.join(name));
        let state = match json("status-final.json")
            .as_ref()
            .and_then(|v| v["state"].as_str())
        {
            Some("complete") => RunState::Passed,
            Some("failed") | Some("incomplete") => RunState::Failed,
            Some("cancelled") => RunState::Interrupted,
            _ => RunState::Error,
        };
        let count = |v: &serde_json::Value, key: &str| v[key].as_u64().unwrap_or(0) as usize;
        let totals = json("progress.json")
            .map(|p| Totals {
                total: count(&p, "total"),
                completed: count(&p, "completed"),
                ..Totals::default()
            })
            .unwrap_or_default();
        self.write(state, totals, Some(now_ms()));
    }
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
        announced.finish(&out);
        let progress = store::progress(dir).unwrap();
        assert_eq!(progress.state, RunState::Interrupted);
        assert_eq!((progress.totals.completed, progress.totals.total), (3, 8));
        assert!(progress.duration_s.is_some());
    }
}
