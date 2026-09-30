use airbug_bench::{Result, error, model::write_new};
use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize)]
pub struct Target {
    pub package: String,
    pub name: String,
    pub manifest: PathBuf,
    pub registered: bool,
}
pub fn discover(manifest: Option<&Path>, offline: bool) -> Result<Vec<Target>> {
    let mut cmd = Command::new("cargo");
    cmd.args(["metadata", "--no-deps", "--format-version", "1"]);
    if let Some(p) = manifest {
        cmd.arg("--manifest-path").arg(p);
    }
    if offline {
        cmd.arg("--offline");
    }
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(error(String::from_utf8_lossy(&out.stderr)));
    }
    let data: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let members = data["workspace_members"]
        .as_array()
        .ok_or_else(|| error("Cargo metadata missing workspace members"))?;
    let mut targets = vec![];
    for p in data["packages"]
        .as_array()
        .ok_or_else(|| error("Cargo metadata missing packages"))?
    {
        if !members.contains(&p["id"]) {
            continue;
        }
        for t in p["targets"]
            .as_array()
            .ok_or_else(|| error("missing targets"))?
        {
            if !t["kind"]
                .as_array()
                .is_some_and(|k| k.iter().any(|v| v == "bench"))
            {
                continue;
            }
            let name = t["name"]
                .as_str()
                .ok_or_else(|| error("missing target name"))?;
            targets.push(Target {
                package: p["name"].as_str().unwrap_or_default().into(),
                name: name.into(),
                manifest: PathBuf::from(
                    p["manifest_path"]
                        .as_str()
                        .ok_or_else(|| error("missing manifest"))?,
                ),
                registered: p["metadata"]["airbug_bench"]["targets"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|v| v == name)),
            });
        }
    }
    targets.sort_by(|a, b| (&a.package, &a.name).cmp(&(&b.package, &b.name)));
    Ok(targets)
}
pub fn build(target: &Target, offline: bool) -> Result<PathBuf> {
    build_at(target, offline, None)
}
pub fn build_at(target: &Target, offline: bool, target_dir: Option<&Path>) -> Result<PathBuf> {
    eprintln!(
        "bench: building {}/{} (outside measurement)",
        target.package, target.name
    );
    let mut c = Command::new("cargo");
    c.args([
        "bench",
        "--no-run",
        "--message-format=json",
        "--manifest-path",
    ])
    .arg(&target.manifest)
    .args(["--bench", &target.name]);
    if offline {
        c.arg("--offline");
    }
    if let Some(dir) = target_dir {
        c.env("CARGO_TARGET_DIR", dir);
    }
    let out = c.output()?;
    eprint!("{}", String::from_utf8_lossy(&out.stderr));
    let mut executable = None;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if v["reason"] == "compiler-message" {
                if let Some(m) = v["message"]["rendered"].as_str() {
                    eprint!("{m}");
                }
            }
            if v["reason"] == "compiler-artifact"
                && v["target"]["name"] == target.name
                && v["target"]["kind"]
                    .as_array()
                    .is_some_and(|k| k.iter().any(|v| v == "bench"))
            {
                if let Some(p) = v["executable"].as_str() {
                    executable = Some(PathBuf::from(p));
                }
            }
        }
    }
    if !out.status.success() {
        return Err(error("benchmark build failed"));
    }
    executable.ok_or_else(|| error("Cargo returned no benchmark executable"))
}
pub fn init(manifest: &Path, library: Option<&Path>) -> Result<()> {
    let manifest = fs::canonicalize(manifest)?;
    let root = manifest.parent().unwrap();
    let original = fs::read_to_string(&manifest)?;
    let mut doc = original
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| error(e.to_string()))?;
    if doc.get("package").is_none() {
        return Err(error(
            "virtual workspace: select a member with --manifest-path",
        ));
    }
    let example = root.join("benches/bench.rs");
    let config = root.join("bench.json");
    if example.exists()
        || config.exists()
        || doc
            .get("bench")
            .and_then(|v| v.as_array_of_tables())
            .is_some_and(|a| {
                a.iter()
                    .any(|t| t.get("name").and_then(|v| v.as_str()) == Some("bench"))
            })
    {
        return Err(error(
            "init would replace existing bench files/target; nothing changed",
        ));
    }
    // Honor renamed dependencies and explicitly enable attributes even when defaults are off.
    let dependency = doc
        .get("dev-dependencies")
        .and_then(|t| t.as_table_like())
        .and_then(|t| {
            t.iter().find_map(|(name, value)| {
                (name == "airbug-bench"
                    || value.get("package").and_then(|v| v.as_str()) == Some("airbug-bench"))
                .then(|| name.to_owned())
            })
        })
        .unwrap_or_else(|| "airbug-bench".into());
    if doc
        .get("dev-dependencies")
        .and_then(|t| t.get(&dependency))
        .is_none()
    {
        let path = library
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
        let path = fs::canonicalize(path)
            .map_err(|_| error("local bench source missing; provide --library-path"))?;
        if !path.join("Cargo.toml").is_file() {
            return Err(error("--library-path must contain Cargo.toml"));
        }
        let mut dep = toml_edit::InlineTable::new();
        dep.insert("package", toml_edit::Value::from("airbug-bench"));
        dep.insert(
            "path",
            toml_edit::Value::from(path.to_string_lossy().as_ref()),
        );
        doc["dev-dependencies"][&dependency] = toml_edit::value(dep);
    }
    let dep = &mut doc["dev-dependencies"][&dependency];
    if let Some(version) = dep.as_str() {
        let mut table = toml_edit::InlineTable::new();
        table.insert("version", toml_edit::Value::from(version));
        *dep = toml_edit::value(table);
    }
    if dep.get("features").is_none() {
        dep["features"] = toml_edit::value(toml_edit::Array::new());
    }
    let features = dep["features"]
        .as_array_mut()
        .ok_or_else(|| error("benchmark dependency features must be an array"))?;
    if !features.iter().any(|v| v.as_str() == Some("macros")) {
        features.push("macros");
    }
    let mut bench = toml_edit::Table::new();
    bench["name"] = toml_edit::value("bench");
    bench["harness"] = toml_edit::value(false);
    if doc.get("bench").is_none() {
        doc["bench"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["bench"]
        .as_array_of_tables_mut()
        .ok_or_else(|| error("invalid [[bench]] table"))?
        .push(bench);
    let slot = &mut doc["package"]["metadata"]["airbug_bench"]["targets"];
    if slot.is_none() {
        *slot = toml_edit::value(toml_edit::Array::new());
    }
    slot.as_array_mut()
        .ok_or_else(|| error("metadata.bench.targets must be an array"))?
        .push("bench");
    fs::create_dir_all(example.parent().unwrap())?;
    let source = r#"#[AIRBUG::suite]
mod example {
    #[bench(args = [32usize, 128, 512], setup = |n| (0..n).rev().collect::<Vec<_>>())]
    fn sort(values: &mut [usize]) {
        values.sort_unstable();
    }
}
"#
    .replace("AIRBUG", &dependency.replace('-', "_"));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&example)?
        .write_all(source.as_bytes())?;
    if let Err(e) = write_new(
        &config,
        &serde_json::json!({"budgets":[{"case":"example/sort/32","metric":"wall","unit":"ns","max":1000000.0}]}),
    ) {
        let _ = fs::remove_file(&example);
        return Err(e);
    }
    // Check concurrent edits before committing the manifest; preserve all existing TOML comments.
    if fs::read_to_string(&manifest)? != original {
        let _ = fs::remove_file(&example);
        let _ = fs::remove_file(&config);
        return Err(error("manifest changed during init"));
    }
    if let Err(e) = fs::write(&manifest, doc.to_string()) {
        let _ = fs::remove_file(example);
        let _ = fs::remove_file(config);
        return Err(e.into());
    }
    println!(
        "Created benches/bench.rs and bench.json; registered Cargo target.\nRun: cargo bench --manifest-path \"{}\" --bench bench",
        manifest.display()
    );
    Ok(())
}
use airbug_bench::baseline::BaselineRef as Baseline;
pub fn save_baseline(
    store: &Path,
    name: &str,
    run: &Path,
    mode: airbug_bench::baseline::SaveMode,
) -> Result<()> {
    let _lease = crate::runner::acquire_lease()?;
    let baselines = airbug_bench::baseline::Store::new(store);
    if baselines.has_case_manifest(name)? {
        if matches!(mode, airbug_bench::baseline::SaveMode::Retain) {
            baselines.prepare_case_save(name, mode)?;
            println!("Retained baseline @{name}");
            return Ok(());
        }
        let source = resolve(store, run)?;
        let measured = airbug_bench::Run::load(&source)?;
        baselines.save_cases(name, &measured, mode)?;
        println!("Saved baseline @{name}");
        return Ok(());
    }
    // Retaining a valid reference must not depend on resolving the unused source.
    let source = if matches!(mode, airbug_bench::baseline::SaveMode::Retain)
        && baselines.load(name)?.is_some()
    {
        run.to_owned()
    } else {
        resolve(store, run)?
    };
    let saved = baselines.save(name, &source, mode)?;
    println!(
        "{} baseline @{name}",
        if saved.retained { "Retained" } else { "Saved" }
    );
    Ok(())
}
pub fn resolve(store: &Path, value: &Path) -> Result<PathBuf> {
    if value == Path::new("last") && !value.exists() {
        return crate::artifacts::last(store);
    }
    if let Some(name) = value.to_str().and_then(|s| s.strip_prefix('@')) {
        if store
            .join("baselines")
            .join(format!("{name}.cases.json"))
            .exists()
        {
            baseline_view(store, name, false)
        } else {
            Ok(airbug_bench::baseline::Store::new(store).require(name)?.run)
        }
    } else {
        Ok(value.into())
    }
}
/// Reports compare stable case labels even when their source runs differ.
pub fn resolve_report(store: &Path, value: &Path) -> Result<PathBuf> {
    if let Some(name) = value.to_str().and_then(|s| s.strip_prefix('@')) {
        baseline_view(store, name, true)
    } else {
        resolve(store, value)
    }
}
fn baseline_view(store: &Path, name: &str, per_case: bool) -> Result<PathBuf> {
    use sha2::{Digest, Sha256};
    let baselines = airbug_bench::baseline::Store::new(store);
    let _lock = baselines.lock_storage()?;
    let cases = baselines.load_cases(name)?;
    if cases.is_empty() {
        return Err(error(format!("baseline @{name} has no cases")));
    }
    let rows: Vec<(String, airbug_bench::Run)> = if per_case {
        cases
            .into_iter()
            .map(|(id, run)| {
                (
                    airbug_bench::model::hex(&Sha256::digest(id.as_bytes())),
                    run,
                )
            })
            .collect()
    } else {
        let ids: Vec<_> = cases.keys().map(String::as_str).collect();
        let mut runs = baselines.load_selected_runs(name, &ids)?;
        if runs.len() != 1 {
            return Err(error(format!(
                "baseline @{name} contains multiple source runs; use report or cargo bench --load-baseline for per-source analysis"
            )));
        }
        vec![(String::new(), runs.remove(0))]
    };
    let digest = airbug_bench::model::hex(&Sha256::digest(serde_json::to_vec(&rows)?));
    let root = store.join("baseline-views");
    let destination = root.join(digest);
    if destination.exists() {
        for (label, run) in &rows {
            let saved = airbug_bench::Run::load(destination.join(label))?;
            if serde_json::to_value(saved)? != serde_json::to_value(run)? {
                return Err(error("resolved baseline view changed; refusing stale data"));
            }
        }
        return Ok(destination);
    }
    fs::create_dir_all(&root)?;
    let temporary = root.join(format!(".{}.tmp", airbug_bench::Run::new().id));
    if per_case {
        fs::create_dir(&temporary)?;
    }
    for (label, run) in &rows {
        run.save_new(temporary.join(label))?;
    }
    fs::rename(&temporary, &destination)?;
    Ok(destination)
}
pub fn baselines(store: &Path) -> Result<()> {
    let dir = store.join("baselines");
    if !dir.exists() {
        return Ok(());
    }
    let mut files = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.sort();
    for p in files {
        if let Some(name) = p
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".cases.json"))
        {
            let cases = airbug_bench::baseline::Store::new(store).load_cases(name)?;
            println!("@{name} → {} cases ({})", cases.len(), p.display());
        } else if p.extension().is_some_and(|e| e == "json") {
            if p.with_extension("cases.json").exists() {
                continue;
            }
            let b: Baseline = serde_json::from_slice(&fs::read(&p)?)?;
            println!(
                "@{} → {}",
                p.file_stem().unwrap().to_string_lossy(),
                b.run.display()
            );
        }
    }
    Ok(())
}
