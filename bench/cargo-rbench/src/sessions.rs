use crate::{artifacts, runner};
use rbench::{
    model::{hash_file, write_new},
    *,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
fn artifact(path: &Path) -> Result<PathBuf> {
    Ok(fs::canonicalize(if path.is_dir() {
        path.join("run.json")
    } else {
        path.into()
    })?)
}
fn checked_plan(path: &Path) -> Result<(PathBuf, Run, runner::Plan)> {
    let path = artifact(path)?;
    let r = Run::load(&path)?;
    if r.provenance.get("plan.sha256")
        != Some(&hash_file(&path.parent().unwrap().join("plan.json"))?)
    {
        return Err(error(
            "plan integrity absent or changed; start a fresh experiment",
        ));
    }
    let plan: runner::Plan =
        serde_json::from_slice(&fs::read(path.parent().unwrap().join("plan.json"))?)?;
    let hashes: BTreeMap<PathBuf, String> = serde_json::from_str(
        r.provenance
            .get("input_hashes")
            .ok_or_else(|| error("runner input identities missing"))?,
    )?;
    for (p, h) in hashes {
        if hash_file(&p)? != h {
            return Err(error("recorded binary or fixture changed"));
        }
    }
    Ok((path, r, plan))
}
pub fn resume(source: &Path, out: &Path) -> Result<()> {
    let (source, old, mut plan) = checked_plan(source)?;
    if old.status == Status::Complete {
        return Err(error("complete experiment cannot be resumed"));
    }
    let policy = plan
        .privacy
        .as_ref()
        .ok_or_else(|| error("resume requires an original explicit environment allowlist"))?;
    if !policy.secret_env.is_empty() {
        return Err(error(
            "resume cannot verify secret environment identity; declare a fresh experiment",
        ));
    }
    let prepared = policy.prepare(&plan)?;
    let current = serde_json::to_string(prepared.environment())?;
    if old.provenance.get("allowed_environment") != Some(&current) {
        return Err(error("allowed environment changed"));
    }
    let expected_os = old.environment.get("os").map(String::as_str);
    let expected_arch = old.environment.get("arch").map(String::as_str);
    if expected_os != Some(std::env::consts::OS) || expected_arch != Some(std::env::consts::ARCH) {
        return Err(error("host platform changed"));
    }
    for (key, command, args) in [
        ("host", "hostname", vec![]),
        ("kernel", "uname", vec!["-r"]),
    ] {
        let o = std::process::Command::new(command).args(args).output()?;
        let value = String::from_utf8(o.stdout)?.trim().to_string();
        if !o.status.success() || old.environment.get(key) != Some(&value) {
            return Err(error("host/kernel changed"));
        }
    }
    let mut complete = BTreeMap::<u32, BTreeSet<String>>::new();
    for o in &old.observations {
        complete
            .entry(o.pair.unwrap_or(plan.start_pair + o.process))
            .or_default()
            .insert(o.variant.clone());
    }
    let variants = if plan.baseline.is_some() {
        2 + plan.variants.len()
    } else {
        1
    };
    let mut prefix = 0;
    for pair in plan.start_pair..plan.start_pair + plan.repetitions {
        if complete.get(&pair).is_some_and(|v| v.len() == variants) {
            prefix += 1;
        } else {
            break;
        }
    }
    if prefix == plan.repetitions {
        return Err(error("all scheduled processes present; do not reinterpret failed validation as resumable measurement"));
    }
    // Multi-variant balance must remain intact, so restart at its last complete rotation block.
    if !plan.variants.is_empty() {
        let block = 2 * (plan.variants.len() + 2) as u32;
        prefix = prefix / block * block;
    }
    if !old.cases.is_empty() {
        plan.provenance.insert(
            "session.expected_cases".into(),
            serde_json::to_string(&old.cases)?,
        );
    }
    plan.repetitions -= prefix;
    plan.start_pair += prefix;
    plan.provenance.insert(
        "session.parent.path".into(),
        source.to_string_lossy().into(),
    );
    plan.provenance
        .insert("session.parent.sha256".into(), hash_file(&source)?);
    plan.provenance.insert("session.policy".into(),"new independent linked session; original immutable; no automatic pooling; incomplete pair/block replayed".into());
    runner::run(plan, out)?;
    Ok(())
}
pub fn profile(
    source: &Path,
    case: &str,
    profiler: &Path,
    args: Vec<String>,
    out: &Path,
) -> Result<()> {
    let (source, old, mut plan) = checked_plan(source)?;
    if old.status != Status::Complete || !old.cases.iter().any(|c| c.id == case) {
        return Err(error("complete source and exact recorded case required"));
    }
    if !plan.protocol {
        return Err(error(
            "profiler case replay requires a protocol worker with filter support",
        ));
    }
    let mut descriptor = old.cases.iter().find(|c| c.id == case).unwrap().clone();
    descriptor.contract.retain(|k, _| {
        k != "worker_environment" && k != "fixture_hashes" && !k.starts_with("experiment.")
    });
    plan.provenance.insert(
        "diagnostic.expected_descriptor".into(),
        serde_json::to_string(&descriptor)?,
    );
    if case.starts_with("forma/") && case.ends_with("/gpu") {
        return Err(error("GPU-only profiler replay needs a dedicated single-case GPU worker; normal Forma cases are supported"));
    }
    let original = plan.candidate.path.clone();
    let mut workload = vec![];
    let mut iter = plan.candidate.args.into_iter();
    while let Some(a) = iter.next() {
        if matches!(a.as_str(), "--filter" | "--exclude" | "--tag") {
            iter.next();
        } else if !matches!(a.as_str(), "--exact" | "--glob")
            && !(case.starts_with("forma/") && a == "--gpu-timestamps")
        {
            workload.push(a);
        }
    }
    // Suite IDs are exact; Forma's integration accepts its scenario name as a filter.
    workload.extend([
        "--filter".into(),
        case.strip_prefix("forma/")
            .map(|name| name.split('/').next().unwrap())
            .unwrap_or(case)
            .into(),
    ]);
    if !case.starts_with("forma/") {
        workload.push("--exact".into());
    }
    plan.candidate.path = fs::canonicalize(profiler)?;
    plan.candidate.args = args;
    plan.candidate.args.push(original.to_string_lossy().into());
    plan.candidate.args.extend(workload);
    plan.baseline = None;
    plan.variants.clear();
    plan.repetitions = 1;
    plan.start_pair = 0;
    plan.fixtures.push(original);
    plan.contract.insert(
        "diagnostic.profiler".into(),
        plan.candidate.path.to_string_lossy().into(),
    );
    plan.provenance.insert(
        "session.parent.path".into(),
        source.to_string_lossy().into(),
    );
    plan.provenance
        .insert("session.parent.sha256".into(), hash_file(&source)?);
    plan.provenance.insert(
        "session.policy".into(),
        "profiler replay; perturbed diagnostic data; never pool or compare as ordinary benchmark"
            .into(),
    );
    plan.provenance
        .insert("diagnostic.expected_case".into(), case.into());
    let run = runner::run(plan, out)?;
    if run.cases.len() != 1 || run.cases[0].id != case {
        return Err(error(
            "profiler did not replay exactly the requested case; diagnostic output retained",
        ));
    }
    Ok(())
}
/// Conservative retention: only direct owned runner directories; quarantine is reversible.
pub fn retention(store: &Path, keep: usize, apply: bool) -> Result<serde_json::Value> {
    let _lease = runner::acquire_lease()?;
    let root = fs::canonicalize(store)?;
    let history = artifacts::history(&root)?;
    let mut protected = BTreeSet::new();
    let refs = root.join("baselines");
    if refs.exists() {
        for e in fs::read_dir(refs)? {
            let e = e?;
            if !e.file_type()?.is_file() {
                return Err(error("unexpected baseline entry"));
            }
            let v: serde_json::Value = serde_json::from_slice(&fs::read(e.path())?)?;
            let p = PathBuf::from(
                v["run"]
                    .as_str()
                    .ok_or_else(|| error("invalid baseline reference"))?,
            );
            let hash = v["sha256"]
                .as_str()
                .ok_or_else(|| error("missing baseline hash"))?;
            if hash_file(&p)? != hash {
                return Err(error("baseline integrity failed; retention refused"));
            }
            protected.insert(artifact(&p)?);
        }
    }
    for row in &history {
        if row.error.is_some() {
            return Err(error("invalid run in store; retention refused"));
        }
        let r = Run::load(&row.path)?;
        if let Some(p) = r.provenance.get("user.session.parent.path") {
            let p = artifact(Path::new(p))?;
            if Some(&hash_file(&p)?) != r.provenance.get("user.session.parent.sha256") {
                return Err(error("linked session integrity failed"));
            }
            protected.insert(p);
            protected.insert(row.path.clone());
        }
    }
    for row in history.iter().rev().take(keep) {
        protected.insert(row.path.clone());
    }
    fn size(p: &Path, visited: &mut usize) -> Result<u64> {
        *visited += 1;
        if *visited > 100000 {
            return Err(error("retention inventory limit"));
        }
        let m = fs::symlink_metadata(p)?;
        if m.file_type().is_symlink() {
            return Err(error("symlink in run; retention refused"));
        }
        if m.is_file() {
            return Ok(m.len());
        }
        let mut n = 0;
        for e in fs::read_dir(p)? {
            n += size(&e?.path(), visited)?;
        }
        Ok(n)
    }
    let mut rows = vec![];
    let mut selected = vec![];
    for row in history {
        let dir = row.path.parent().unwrap();
        let owned = dir.parent() == Some(root.as_path())
            && dir.join("plan.json").is_file()
            && dir.join("schedule.json").is_file()
            && dir.join("status-final.json").is_file();
        let eligible = owned && row.status == "Complete" && !protected.contains(&row.path);
        rows.push(serde_json::json!({"path":dir,"bytes":size(dir,&mut 0)?,"eligible":eligible,"protected":protected.contains(&row.path)}));
        if eligible {
            selected.push(dir.to_path_buf());
        }
    }
    if apply && !selected.is_empty() {
        let quarantine = root.join("quarantine");
        if quarantine.is_symlink() {
            return Err(error("quarantine symlink refused"));
        }
        fs::create_dir_all(&quarantine)?;
        let batch = quarantine.join(Run::new().id);
        fs::create_dir(&batch)?;
        write_new(&batch.join("manifest.json"), &rows)?;
        for p in selected {
            fs::rename(&p, batch.join(p.file_name().unwrap()))?;
        }
    }
    Ok(
        serde_json::json!({"apply":apply,"policy":"quarantine only; no permanent deletion; only direct completed runner directories; newest N, baselines and linked sessions protected","runs":rows}),
    )
}
