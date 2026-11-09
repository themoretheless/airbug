//! Run directories under `<root>/.airbug/runs/<run_id>/`.
//!
//! Writers own their directory; readers poll it. Every JSON document is replaced by rename,
//! so a reader never sees a half-written file.
use crate::model::{MANIFEST_SCHEMA, Manifest, Progress};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs, io,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

/// Directory of runs under the project root.
pub const RUNS_DIR: &str = ".airbug/runs";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const PROGRESS_FILE: &str = "progress.json";
pub const REPORT_FILE: &str = "report.json";

/// Documents larger than this are not parsed.
pub const MAX_DOCUMENT_BYTES: u64 = 64 * 1024 * 1024;

pub fn runs_dir(root: &Path) -> PathBuf {
    root.join(RUNS_DIR)
}

pub fn run_dir(root: &Path, run_id: &str) -> PathBuf {
    runs_dir(root).join(run_id)
}

/// Write `value` to `path` atomically (temp file + rename).
pub fn write_json(path: &Path, value: &impl Serialize, pretty: bool) -> io::Result<()> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(value)
    } else {
        serde_json::to_vec(value)
    }
    .map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    // On Windows a reader holding the file open makes the rename fail for a moment.
    let mut last = Ok(());
    for _ in 0..5 {
        last = fs::rename(&tmp, path);
        if last.is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    last
}

/// Parse a JSON document; `None` when it is missing, oversized or not valid yet.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    if fs::metadata(path).ok()?.len() > MAX_DOCUMENT_BYTES {
        return None;
    }
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

/// The manifest of a run directory, if it is one this version understands.
pub fn manifest(dir: &Path) -> Option<Manifest> {
    let manifest: Manifest = read_json(&dir.join(MANIFEST_FILE))?;
    (manifest.schema == MANIFEST_SCHEMA).then_some(manifest)
}

pub fn progress(dir: &Path) -> Option<Progress> {
    read_json(&dir.join(PROGRESS_FILE))
}

/// Every run directory in `runs_dir` with a readable manifest of `kind`, in no fixed order.
pub fn list(runs_dir: &Path, kind: &str) -> Vec<(PathBuf, Manifest)> {
    let Ok(entries) = fs::read_dir(runs_dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|dir| {
            let manifest = manifest(&dir)?;
            (manifest.kind == kind).then_some((dir, manifest))
        })
        .collect()
}

/// Keep the newest `keep` finished runs of `kind`; never touch `current` or runs still going.
pub fn prune(runs_dir: &Path, kind: &str, keep: usize, current: &str) {
    let mut finished: Vec<(u64, PathBuf)> = list(runs_dir, kind)
        .into_iter()
        .filter(|(dir, _)| dir.file_name().and_then(|n| n.to_str()) != Some(current))
        .filter(|(dir, _)| progress(dir).is_some_and(|p| p.state.is_finished()))
        .map(|(dir, manifest)| (manifest.started_at_ms, dir))
        .collect();
    finished.sort_by_key(|a| std::cmp::Reverse(a.0));
    // `current` is one of the kept runs.
    for (_, dir) in finished.into_iter().skip(keep.saturating_sub(1)) {
        let _ = fs::remove_dir_all(dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RunState, now_ms};
    use std::env;

    fn scratch(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "airbug-run-{name}-{}-{}",
            std::process::id(),
            now_ms()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn put(runs: &Path, id: &str, kind: &str, started: u64, state: RunState) {
        let dir = runs.join(id);
        fs::create_dir_all(&dir).unwrap();
        let manifest = Manifest {
            schema: MANIFEST_SCHEMA.into(),
            kind: kind.into(),
            run_id: id.into(),
            started_at_ms: started,
            ..Manifest::default()
        };
        write_json(&dir.join(MANIFEST_FILE), &manifest, true).unwrap();
        let progress = Progress {
            run_id: id.into(),
            kind: kind.into(),
            state,
            ..Progress::default()
        };
        write_json(&dir.join(PROGRESS_FILE), &progress, false).unwrap();
    }

    #[test]
    fn write_then_read_round_trips_and_leaves_no_temp_file() {
        let dir = scratch("roundtrip");
        put(&dir, "a", "test", 7, RunState::Passed);
        let run = dir.join("a");
        assert_eq!(manifest(&run).unwrap().started_at_ms, 7);
        assert_eq!(progress(&run).unwrap().state, RunState::Passed);
        assert!(!run.join("manifest.json.tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_filters_by_kind_and_skips_foreign_directories() {
        let dir = scratch("list");
        put(&dir, "t", "test", 1, RunState::Passed);
        put(&dir, "b", "bench", 2, RunState::Passed);
        fs::create_dir_all(dir.join("junk")).unwrap();
        fs::write(
            dir.join("junk").join(MANIFEST_FILE),
            r#"{"schema":"other/9","kind":"test","run_id":"x","title":""}"#,
        )
        .unwrap();
        let ids: Vec<String> = list(&dir, "test")
            .into_iter()
            .map(|(_, m)| m.run_id)
            .collect();
        assert_eq!(ids, ["t"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_keeps_newest_finished_runs() {
        let dir = scratch("prune");
        let states = [
            RunState::Passed,
            RunState::Failed,
            RunState::Running,
            RunState::Passed,
        ];
        for (i, state) in states.into_iter().enumerate() {
            put(&dir, &format!("run-{i}"), "test", i as u64, state);
        }
        put(&dir, "bench-old", "bench", 0, RunState::Passed);
        // keep 2 = current (run-3) + newest other finished (run-1); run-2 is still running.
        prune(&dir, "test", 2, "run-3");
        assert!(dir.join("run-3").is_dir());
        assert!(dir.join("run-2").is_dir());
        assert!(dir.join("run-1").is_dir());
        assert!(!dir.join("run-0").exists());
        assert!(
            dir.join("bench-old").is_dir(),
            "other kinds are not this caller's to prune"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
