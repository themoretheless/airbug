//! Named, hash-checked references to completed benchmark artifacts.
use crate::{Result, Run, Status, error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineRef {
    pub run: PathBuf,
    pub sha256: String,
}
#[derive(Clone, Copy, Debug, Default)]
pub enum SaveMode {
    #[default]
    Create,
    Replace,
    Retain,
}
#[derive(Clone, Debug)]
pub struct Saved {
    pub reference: BaselineRef,
    pub retained: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct SavedCases {
    pub manifest: PathBuf,
    pub retained: bool,
    pub updated_cases: Vec<String>,
}
pub struct Store {
    root: PathBuf,
}
impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    /// Serialize case-baseline publication with storage retention. Keep the
    /// returned file alive until every protected-artifact operation completes.
    pub fn lock_storage(&self) -> Result<fs::File> {
        let directory = self.root.join("baselines");
        fs::create_dir_all(&directory)?;
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join(".store.lock"))?;
        lock.lock()?;
        Ok(lock)
    }
    fn path(&self, name: &str) -> Result<PathBuf> {
        if name.is_empty()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(error("baseline name: letters, digits, '-' or '_' only"));
        }
        Ok(self.root.join("baselines").join(format!("{name}.json")))
    }
    pub fn has_case_manifest(&self, name: &str) -> Result<bool> {
        match fs::symlink_metadata(self.path(name)?.with_extension("cases.json")) {
            Ok(_) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err.into()),
        }
    }
    /// Missing names return None. A missing, changed or invalid referenced run
    /// is an error, not a missing baseline that may silently be replaced.
    pub fn load(&self, name: &str) -> Result<Option<BaselineRef>> {
        Ok(self.load_run(name)?.map(|(reference, _)| reference))
    }
    /// Load and verify one snapshot. The returned run is decoded from the exact
    /// bytes whose digest was checked, without reopening the source artifact.
    pub fn load_run(&self, name: &str) -> Result<Option<(BaselineRef, Run)>> {
        let path = self.path(name)?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let reference: BaselineRef = serde_json::from_slice(&bytes)?;
        let bytes = fs::read(&reference.run)?;
        if crate::model::hex(&Sha256::digest(&bytes)) != reference.sha256 {
            return Err(error("baseline artifact changed since registration"));
        }
        let run = completed(&bytes)?;
        Ok(Some((reference, run)))
    }
    pub fn require(&self, name: &str) -> Result<BaselineRef> {
        self.load(name)?
            .ok_or_else(|| error(format!("baseline @{name} does not exist")))
    }
    /// Load a named baseline before executing selected cases. Strict mode
    /// requires both the name and every selected case. Lenient mode allows
    /// missing data, but never suppresses corruption or an invalid artifact.
    pub fn prepare_comparison(
        &self,
        name: &str,
        selected: &[&str],
        strict: bool,
    ) -> Result<Option<Run>> {
        let loaded = self.load_run(name)?;
        let Some((_, run)) = loaded else {
            return if strict {
                Err(error(format!("baseline @{name} does not exist")))
            } else {
                Ok(None)
            };
        };
        if strict {
            for id in selected {
                if !run.cases.iter().any(|case| case.id == *id) {
                    return Err(error(format!("baseline @{name} has no case {id:?}")));
                }
            }
        }
        Ok(Some(run))
    }
    /// Load case snapshots independently, preserving the original environment
    /// and provenance for each case, including cases saved in different runs.
    pub fn load_cases(&self, name: &str) -> Result<BTreeMap<String, Run>> {
        self.case_references(name)?
            .into_iter()
            .map(|(id, reference)| {
                let run = checked_case(&id, &reference)?;
                Ok((id, run))
            })
            .collect()
    }
    fn case_references(&self, name: &str) -> Result<BTreeMap<String, BaselineRef>> {
        let path = self.path(name)?.with_extension("cases.json");
        match fs::read(path) {
            Ok(bytes) => {
                let manifest: CaseManifest = serde_json::from_slice(&bytes)?;
                if manifest.version != 1 {
                    return Err(error("unsupported case baseline version"));
                }
                Ok(manifest.cases)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(self
                .load_run(name)?
                .map(|(reference, run)| {
                    run.cases
                        .into_iter()
                        .map(|case| (case.id, reference.clone()))
                        .collect()
                })
                .unwrap_or_default()),
            Err(err) => Err(err.into()),
        }
    }
    /// Resolve the selected case snapshots before measurement.
    pub fn prepare_case_comparison(
        &self,
        name: &str,
        selected: &[&str],
        strict: bool,
    ) -> Result<BTreeMap<String, Run>> {
        let mut cases = self.load_cases(name)?;
        if strict {
            for id in selected {
                if !cases.contains_key(*id) {
                    return Err(error(format!("baseline @{name} has no case {id:?}")));
                }
            }
        }
        cases.retain(|id, _| selected.contains(&id.as_str()));
        Ok(cases)
    }
    /// Load selected cases grouped by their actual immutable artifact, never
    /// by run ID alone. Independently produced snapshots may reuse a run ID.
    pub fn load_selected_runs(&self, name: &str, selected: &[&str]) -> Result<Vec<Run>> {
        if selected.is_empty() {
            return Err(error("no cases selected for baseline loading"));
        }
        let path = self.path(name)?.with_extension("cases.json");
        let references = match fs::read(path) {
            Ok(bytes) => {
                let manifest: CaseManifest = serde_json::from_slice(&bytes)?;
                if manifest.version != 1 {
                    return Err(error("unsupported case baseline version"));
                }
                manifest.cases
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let (reference, run) = self
                    .load_run(name)?
                    .ok_or_else(|| error(format!("baseline @{name} does not exist")))?;
                run.cases
                    .into_iter()
                    .map(|case| (case.id, reference.clone()))
                    .collect()
            }
            Err(err) => return Err(err.into()),
        };
        let mut groups: BTreeMap<(PathBuf, String), Vec<&str>> = BTreeMap::new();
        for id in selected {
            let reference = references
                .get(*id)
                .ok_or_else(|| error(format!("baseline @{name} has no case {id:?}")))?;
            groups
                .entry((reference.run.clone(), reference.sha256.clone()))
                .or_default()
                .push(id);
        }
        let mut runs = Vec::new();
        for ((path, hash), ids) in groups {
            let bytes = fs::read(path)?;
            if crate::model::hex(&Sha256::digest(&bytes)) != hash {
                return Err(error("case baseline artifact changed since registration"));
            }
            let mut run = completed(&bytes)?;
            for id in &ids {
                if !run.cases.iter().any(|case| case.id == *id) {
                    return Err(error(format!("case baseline artifact has no case {id:?}")));
                }
            }
            run.cases.retain(|case| ids.contains(&case.id.as_str()));
            run.observations.retain(|o| ids.contains(&o.case.as_str()));
            run.worker_allocations
                .retain(|w| ids.contains(&w.case.as_str()));
            run.validate()?;
            runs.push(run);
        }
        Ok(runs)
    }
    pub fn prepare_case_save(&self, name: &str, mode: SaveMode) -> Result<()> {
        self.prepare_case_save_for(name, mode, &[])
    }
    /// Explicit replacement may repair selected corrupt sources, but every
    /// retained case must remain readable and pass its original digest check.
    pub fn prepare_case_save_for(
        &self,
        name: &str,
        mode: SaveMode,
        selected: &[&str],
    ) -> Result<()> {
        let references = self.case_references(name)?;
        validate_update(name, &references, mode, selected)
    }
    /// Merge a measured subset into a case baseline. An OS file lock serializes
    /// read/modify/publish so disjoint concurrent updates cannot lose cases.
    pub fn save_cases(&self, name: &str, run: &Run, mode: SaveMode) -> Result<SavedCases> {
        let path = self.path(name)?.with_extension("cases.json");
        completed(&serde_json::to_vec(run)?)?;
        let _storage_lock = self.lock_storage()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))?;
        lock.lock()?;
        let mut references = self.case_references(name)?;
        let ids: Vec<_> = run.cases.iter().map(|case| case.id.as_str()).collect();
        validate_update(name, &references, mode, &ids)?;
        let selected: Vec<_> = run
            .cases
            .iter()
            .filter(|case| !matches!(mode, SaveMode::Retain) || !references.contains_key(&case.id))
            .map(|case| case.id.clone())
            .collect();
        if selected.is_empty() {
            return Ok(SavedCases {
                manifest: path,
                retained: true,
                updated_cases: selected,
            });
        }
        let directory = self.root.join("baseline-runs");
        fs::create_dir_all(&directory)?;
        static NEXT_CASES: AtomicU64 = AtomicU64::new(0);
        let generation = format!(
            "{}-{:016x}",
            Run::new().id,
            NEXT_CASES.fetch_add(1, Ordering::Relaxed)
        );
        let artifact = directory.join(format!("{generation}.json"));
        crate::model::write_new(&artifact, run)?;
        let reference = BaselineRef {
            sha256: crate::model::hash_file(&artifact)?,
            run: fs::canonicalize(artifact)?,
        };
        for id in &selected {
            references.insert(id.clone(), reference.clone());
        }
        let temporary = path.with_extension(format!("{generation}.tmp"));
        crate::model::write_new(
            &temporary,
            &CaseManifest {
                version: 1,
                cases: references,
            },
        )?;
        let _cleanup = Pending(temporary.clone());
        fs::rename(temporary, &path)?;
        Ok(SavedCases {
            manifest: path,
            retained: false,
            updated_cases: selected,
        })
    }
    /// Check a save request before running workloads. Publication still enforces
    /// create/retain atomically if another process changes the name afterwards.
    pub fn prepare_save(&self, name: &str, mode: SaveMode) -> Result<()> {
        let path = self.path(name)?;
        match mode {
            SaveMode::Create => match fs::symlink_metadata(path) {
                Ok(_) => return Err(error(format!("baseline @{name} already exists"))),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err.into()),
            },
            SaveMode::Retain => {
                self.load(name)?;
            }
            SaveMode::Replace => {}
        }
        Ok(())
    }
    /// Persist an owned snapshot then publish its named reference. Source paths
    /// are generated internally; caller-supplied run IDs never become paths.
    /// Failed publication leaves the completed snapshot available for recovery.
    pub fn save_run(&self, name: &str, run: &Run, mode: SaveMode) -> Result<Saved> {
        self.prepare_save(name, mode)?;
        if matches!(mode, SaveMode::Retain) {
            if let Some(reference) = self.load(name)? {
                return Ok(Saved {
                    reference,
                    retained: true,
                });
            }
        }
        let bytes = serde_json::to_vec(run)?;
        completed(&bytes)?;
        let directory = self.root.join("baseline-runs");
        fs::create_dir_all(&directory)?;
        static NEXT_RUN: AtomicU64 = AtomicU64::new(0);
        let path = directory.join(format!(
            "{}-{:016x}.json",
            Run::new().id,
            NEXT_RUN.fetch_add(1, Ordering::Relaxed)
        ));
        crate::model::write_new(&path, run)?;
        self.save(name, &path, mode)
    }
    /// Publish a new pointer atomically, without modifying or deleting source
    /// artifacts. Retain checks an existing reference before reading the new run.
    pub fn save(&self, name: &str, run: &Path, mode: SaveMode) -> Result<Saved> {
        let destination = self.path(name)?;
        let _storage_lock = self.lock_storage()?;
        if self.has_case_manifest(name)? {
            return Err(error(
                "case baseline exists; update it with save_cases instead of replacing its legacy reference",
            ));
        }
        if matches!(mode, SaveMode::Retain) {
            if let Some(reference) = self.load(name)? {
                return Ok(Saved {
                    reference,
                    retained: true,
                });
            }
        }
        let run = fs::canonicalize(if run.is_dir() {
            run.join("run.json")
        } else {
            run.to_owned()
        })?;
        let bytes = fs::read(&run)?;
        completed(&bytes)?;
        let reference = BaselineRef {
            run,
            sha256: crate::model::hex(&Sha256::digest(&bytes)),
        };
        let directory = destination.parent().unwrap();
        fs::create_dir_all(directory)?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let temporary = directory.join(format!(
            ".{name}.{}.{:016x}.tmp",
            Run::new().id,
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let _cleanup = Pending(temporary.clone());
        serde_json::to_writer_pretty(&mut file, &reference)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        if matches!(mode, SaveMode::Replace) {
            fs::rename(&temporary, &destination)?;
        } else {
            match fs::hard_link(&temporary, &destination) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    if matches!(mode, SaveMode::Retain) {
                        return Ok(Saved {
                            reference: self.require(name)?,
                            retained: true,
                        });
                    }
                    return Err(error(format!("baseline @{name} already exists")));
                }
                Err(err) => return Err(err.into()),
            }
        }
        Ok(Saved {
            reference,
            retained: false,
        })
    }
}
fn checked_case(id: &str, reference: &BaselineRef) -> Result<Run> {
    let bytes = fs::read(&reference.run)?;
    if crate::model::hex(&Sha256::digest(&bytes)) != reference.sha256 {
        return Err(error("case baseline artifact changed since registration"));
    }
    split_cases(&completed(&bytes)?)
        .remove(id)
        .ok_or_else(|| error(format!("case baseline artifact has no case {id:?}")))
}
fn validate_update(
    name: &str,
    references: &BTreeMap<String, BaselineRef>,
    mode: SaveMode,
    selected: &[&str],
) -> Result<()> {
    if matches!(mode, SaveMode::Create) && !references.is_empty() {
        return Err(error(format!("baseline @{name} already exists")));
    }
    for (id, reference) in references {
        if matches!(mode, SaveMode::Replace) && selected.contains(&id.as_str()) {
            continue;
        }
        checked_case(id, reference)?;
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
struct CaseManifest {
    version: u32,
    cases: BTreeMap<String, BaselineRef>,
}
fn split_cases(run: &Run) -> BTreeMap<String, Run> {
    run.cases
        .iter()
        .map(|case| {
            let mut snapshot = run.clone();
            snapshot.cases.retain(|c| c.id == case.id);
            snapshot.observations.retain(|o| o.case == case.id);
            snapshot.worker_allocations.retain(|w| w.case == case.id);
            (case.id.clone(), snapshot)
        })
        .collect()
}
fn completed(bytes: &[u8]) -> Result<Run> {
    let run: Run = serde_json::from_slice(bytes)?;
    run.validate()?;
    if run.status != Status::Complete
        || run.cases.iter().any(|case| {
            case.contract
                .get("execution.mode")
                .is_some_and(|mode| mode == "test_once")
        })
    {
        return Err(error(
            "baseline requires a complete benchmark run, not a smoke test",
        ));
    }
    Ok(run)
}
struct Pending(PathBuf);
impl Drop for Pending {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn artifact(root: &Path, name: &str) -> PathBuf {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: name.into(),
                contract: Default::default(),
                metrics: vec![crate::Metric::duration("wall", "test", "batch_total")],
            })
            .unwrap();
        recorder.observe(name, "wall", 10).unwrap();
        let path = root.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_vec(&recorder.finish().unwrap()).unwrap(),
        )
        .unwrap();
        path
    }
    #[test]
    fn create_replace_retain_preserve_source_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(root.path());
        let a = artifact(root.path(), "a");
        let b = artifact(root.path(), "b");
        let old = fs::read(&a).unwrap();
        assert!(store.load("main").unwrap().is_none());
        assert!(store.require("main").is_err());
        store.save("main", &a, SaveMode::Create).unwrap();
        assert!(store.save("main", &b, SaveMode::Create).is_err());
        assert_eq!(
            store.require("main").unwrap().run,
            fs::canonicalize(&a).unwrap()
        );
        assert!(
            store
                .save("main", &root.path().join("missing"), SaveMode::Retain)
                .unwrap()
                .retained
        );
        store.save("main", &b, SaveMode::Replace).unwrap();
        assert_eq!(
            store.require("main").unwrap().run,
            fs::canonicalize(&b).unwrap()
        );
        assert_eq!(fs::read(a).unwrap(), old);
        assert!(!store.save("new", &b, SaveMode::Retain).unwrap().retained);
        assert_eq!(
            fs::read_dir(root.path().join("baselines"))
                .unwrap()
                .filter(|entry| entry.as_ref().unwrap().file_name() != ".store.lock")
                .count(),
            2
        );
    }
    #[test]
    fn corrupt_references_and_invalid_replacements_are_errors() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(root.path());
        let a = artifact(root.path(), "a");
        store.save("main", &a, SaveMode::Create).unwrap();
        let pointer = fs::read(store.path("main").unwrap()).unwrap();
        let b = artifact(root.path(), "b");
        let mut run: Run = serde_json::from_slice(&fs::read(&b).unwrap()).unwrap();
        run.cases[0]
            .contract
            .insert("execution.mode".into(), "test_once".into());
        fs::write(&b, serde_json::to_vec(&run).unwrap()).unwrap();
        assert!(store.save("main", &b, SaveMode::Replace).is_err());
        assert_eq!(fs::read(store.path("main").unwrap()).unwrap(), pointer);
        for name in ["", "../escape", "a/b", "@main", "a b"] {
            assert!(store.save(name, &a, SaveMode::Replace).is_err());
        }
        fs::write(&a, b"{}").unwrap();
        assert!(store.load("main").is_err());
        assert!(store.save("main", &b, SaveMode::Retain).is_err());
        fs::remove_file(a).unwrap();
        assert!(store.load("main").is_err());
    }
    #[test]
    fn strict_and_lenient_preflight_return_verified_snapshots_without_writes() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(root.path());
        assert!(
            store
                .prepare_comparison("missing", &["a"], false)
                .unwrap()
                .is_none()
        );
        assert!(store.prepare_comparison("missing", &["a"], true).is_err());
        assert!(!root.path().join("baselines").exists());
        let source = artifact(root.path(), "a");
        store.save("main", &source, SaveMode::Create).unwrap();
        let snapshot = store
            .prepare_comparison("main", &["a"], true)
            .unwrap()
            .unwrap();
        assert!(store.prepare_comparison("main", &["a", "b"], true).is_err());
        assert!(
            store
                .prepare_comparison("main", &["b"], false)
                .unwrap()
                .is_some()
        );
        fs::write(&source, b"{}").unwrap();
        assert_eq!(snapshot.cases[0].id, "a");
        assert_eq!(snapshot.observations[0].value.as_deref(), Some("10"));
        assert!(store.prepare_comparison("main", &["a"], false).is_err());
        assert!(store.prepare_comparison("main", &["a"], true).is_err());
    }

    #[test]
    fn owned_snapshots_are_immutable_and_do_not_trust_run_ids() {
        let root = tempfile::tempdir().unwrap();
        let source = artifact(root.path(), "a");
        let mut run: Run = serde_json::from_slice(&fs::read(source).unwrap()).unwrap();
        run.id = "../../escape".into();
        let store = Store::new(root.path());
        let first = store.save_run("main", &run, SaveMode::Create).unwrap();
        let bytes = fs::read(&first.reference.run).unwrap();
        assert!(
            first
                .reference
                .run
                .starts_with(fs::canonicalize(root.path().join("baseline-runs")).unwrap())
        );
        assert!(store.prepare_save("main", SaveMode::Create).is_err());
        run.observations[0].value = Some("20".into());
        assert!(
            store
                .save_run("main", &run, SaveMode::Retain)
                .unwrap()
                .retained
        );
        assert_eq!(
            fs::read_dir(root.path().join("baseline-runs"))
                .unwrap()
                .count(),
            1
        );
        let second = store.save_run("main", &run, SaveMode::Replace).unwrap();
        assert_ne!(first.reference.run, second.reference.run);
        assert_eq!(fs::read(&first.reference.run).unwrap(), bytes);
        assert_eq!(
            store.load_run("main").unwrap().unwrap().1.observations[0]
                .value
                .as_deref(),
            Some("20")
        );
        run.cases[0]
            .contract
            .insert("execution.mode".into(), "test_once".into());
        assert!(store.save_run("main", &run, SaveMode::Replace).is_err());
        assert_eq!(
            fs::read_dir(root.path().join("baseline-runs"))
                .unwrap()
                .count(),
            2
        );
    }

    #[test]
    fn case_updates_preserve_other_runs_provenance_and_concurrent_changes() {
        let root = tempfile::tempdir().unwrap();
        let a = artifact(root.path(), "a");
        let b = artifact(root.path(), "b");
        let mut a: Run = serde_json::from_slice(&fs::read(a).unwrap()).unwrap();
        let mut b: Run = serde_json::from_slice(&fs::read(b).unwrap()).unwrap();
        a.environment.insert("machine".into(), "first".into());
        b.environment.insert("machine".into(), "second".into());
        let store = Store::new(root.path());
        store.save_cases("main", &a, SaveMode::Create).unwrap();
        store.save_cases("main", &b, SaveMode::Replace).unwrap();
        let loaded = store.load_cases("main").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded["a"].environment["machine"], "first");
        assert_eq!(loaded["b"].environment["machine"], "second");
        a.observations[0].value = Some("20".into());
        let manifest_before =
            fs::read(store.path("main").unwrap().with_extension("cases.json")).unwrap();
        let files_before = fs::read_dir(root.path().join("baseline-runs"))
            .unwrap()
            .count();
        store.save_cases("main", &a, SaveMode::Retain).unwrap();
        assert_eq!(
            fs::read(store.path("main").unwrap().with_extension("cases.json")).unwrap(),
            manifest_before
        );
        assert_eq!(
            fs::read_dir(root.path().join("baseline-runs"))
                .unwrap()
                .count(),
            files_before
        );
        assert_eq!(
            store.load_cases("main").unwrap()["a"].observations[0]
                .value
                .as_deref(),
            Some("10")
        );
        std::thread::scope(|scope| {
            scope.spawn(|| store.save_cases("race", &a, SaveMode::Replace).unwrap());
            scope.spawn(|| store.save_cases("race", &b, SaveMode::Replace).unwrap());
        });
        assert_eq!(store.load_cases("race").unwrap().len(), 2);
        assert!(store.save_cases("main", &a, SaveMode::Create).is_err());
        // A rejected update releases its lock and leaves both cases readable.
        store.save_cases("main", &a, SaveMode::Replace).unwrap();
        assert_eq!(store.load_cases("main").unwrap().len(), 2);
        let manifest: CaseManifest = serde_json::from_slice(
            &fs::read(store.path("main").unwrap().with_extension("cases.json")).unwrap(),
        )
        .unwrap();
        fs::write(&manifest.cases["b"].run, b"{}").unwrap();
        assert!(store.load_cases("main").is_err());
        assert!(store.save_cases("main", &a, SaveMode::Retain).is_err());
        let before = fs::read(store.path("main").unwrap().with_extension("cases.json")).unwrap();
        assert!(
            store
                .prepare_case_save_for("main", SaveMode::Replace, &["a"])
                .is_err()
        );
        assert!(store.save_cases("main", &a, SaveMode::Replace).is_err());
        assert_eq!(
            fs::read(store.path("main").unwrap().with_extension("cases.json")).unwrap(),
            before
        );
        store
            .prepare_case_save_for("main", SaveMode::Replace, &["b"])
            .unwrap();
        store.save_cases("main", &b, SaveMode::Replace).unwrap();
        assert_eq!(store.load_cases("main").unwrap().len(), 2);
    }

    #[test]
    fn reanalysis_groups_by_artifact_and_preserves_identity_collisions() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(root.path());
        let a = artifact(root.path(), "a");
        let b = artifact(root.path(), "b");
        let a: Run = serde_json::from_slice(&fs::read(a).unwrap()).unwrap();
        let mut b: Run = serde_json::from_slice(&fs::read(b).unwrap()).unwrap();
        b.id = a.id.clone();
        b.environment.insert("machine".into(), "second".into());
        store.save_cases("main", &a, SaveMode::Create).unwrap();
        store.save_cases("main", &b, SaveMode::Replace).unwrap();
        let loaded = store.load_selected_runs("main", &["a", "b"]).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(
            loaded
                .iter()
                .any(|run| serde_json::to_value(run).unwrap() == serde_json::to_value(&a).unwrap())
        );
        assert!(
            loaded
                .iter()
                .any(|run| serde_json::to_value(run).unwrap() == serde_json::to_value(&b).unwrap())
        );
        assert_eq!(store.load_selected_runs("main", &["b"]).unwrap().len(), 1);
        assert!(store.load_selected_runs("main", &["missing"]).is_err());
        assert!(store.load_selected_runs("main", &[]).is_err());
        let mut both = a.clone();
        both.cases.extend(b.cases.clone());
        both.observations.extend(b.observations.clone());
        store
            .save_cases("together", &both, SaveMode::Create)
            .unwrap();
        let loaded = store.load_selected_runs("together", &["a", "b"]).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(
            serde_json::to_value(&loaded[0]).unwrap(),
            serde_json::to_value(both).unwrap()
        );
    }

    #[test]
    fn concurrent_retention_publishes_one_complete_pointer() {
        let root = tempfile::tempdir().unwrap();
        let paths: Vec<_> = (0..8)
            .map(|i| artifact(root.path(), &format!("run{i}")))
            .collect();
        let barrier = std::sync::Barrier::new(paths.len());
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = paths
                .iter()
                .map(|path| {
                    let barrier = &barrier;
                    let root = root.path();
                    scope.spawn(move || {
                        barrier.wait();
                        Store::new(root)
                            .save("race", path, SaveMode::Retain)
                            .unwrap()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|r| !r.retained).count(), 1);
        let winner = Store::new(root.path()).require("race").unwrap();
        assert!(results.iter().all(|r| r.reference.sha256 == winner.sha256));
        assert_eq!(
            fs::read_dir(root.path().join("baselines"))
                .unwrap()
                .filter(|entry| entry.as_ref().unwrap().file_name() != ".store.lock")
                .count(),
            1
        );
    }
}
