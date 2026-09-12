use rbench::{error, model::write_new, Result, Run, Status};
use serde::{Deserialize, Serialize};
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
                registered: p["metadata"]["rbench"]["targets"]
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
        "rbench: building {}/{} (outside measurement)",
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
    let mut doc = original.parse::<toml_edit::DocumentMut>()?;
    if doc.get("package").is_none() {
        return Err(error(
            "virtual workspace: select a member with --manifest-path",
        ));
    }
    let example = root.join("benches/rbench.rs");
    let config = root.join("rbench.json");
    if example.exists()
        || config.exists()
        || doc
            .get("bench")
            .and_then(|v| v.as_array_of_tables())
            .is_some_and(|a| {
                a.iter()
                    .any(|t| t.get("name").and_then(|v| v.as_str()) == Some("rbench"))
            })
    {
        return Err(error(
            "init would replace existing rbench files/target; nothing changed",
        ));
    }
    if doc
        .get("dev-dependencies")
        .and_then(|t| t.get("rbench"))
        .is_none()
    {
        let path = library
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rbench"));
        let path = fs::canonicalize(path)
            .map_err(|_| error("local rbench source missing; provide --library-path"))?;
        if !path.join("Cargo.toml").is_file() {
            return Err(error("--library-path must contain Cargo.toml"));
        }
        let mut dep = toml_edit::InlineTable::new();
        dep.insert(
            "path",
            toml_edit::Value::from(path.to_string_lossy().as_ref()),
        );
        doc["dev-dependencies"]["rbench"] = toml_edit::value(dep);
    }
    let mut bench = toml_edit::Table::new();
    bench["name"] = toml_edit::value("rbench");
    bench["harness"] = toml_edit::value(false);
    if doc.get("bench").is_none() {
        doc["bench"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["bench"]
        .as_array_of_tables_mut()
        .ok_or_else(|| error("invalid [[bench]] table"))?
        .push(bench);
    let slot = &mut doc["package"]["metadata"]["rbench"]["targets"];
    if slot.is_none() {
        *slot = toml_edit::value(toml_edit::Array::new());
    }
    slot.as_array_mut()
        .ok_or_else(|| error("metadata.rbench.targets must be an array"))?
        .push("rbench");
    fs::create_dir_all(example.parent().unwrap())?;
    let source = r#"use rbench::{DropPolicy, Suite};
fn main() -> rbench::Result<()> {
    let mut suite = Suite::new("example");
    suite.matrix("sort", &[("elements", &["32", "128", "512"])], |suite, id, params| {
        let size: u64 = params["elements"].parse().unwrap();
        suite.bench_checked(id, move || (0..size).rev().collect::<Vec<_>>(),
            |v| v.sort_unstable(),
            |v, _| if v.windows(2).all(|w| w[0] <= w[1]) { Ok(()) }
                else { Err(rbench::error("sort produced unordered output")) },
            DropPolicy::InsideTiming);
    })?;
    suite.main()
}
"#;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&example)?
        .write_all(source.as_bytes())?;
    if let Err(e) = write_new(
        &config,
        &serde_json::json!({"budgets":[{"case":"example/sort[elements=32]","metric":"wall","unit":"ns","max":1000000.0}]}),
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
    println!("Created benches/rbench.rs and rbench.json; registered Cargo target.\nRun: cargo rbench bench --manifest-path {} -o .rbench/first",manifest.display());
    Ok(())
}
#[derive(Serialize, Deserialize)]
struct Baseline {
    run: PathBuf,
    sha256: String,
}
fn name_path(store: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(error("baseline name: letters, digits, '-' or '_' only"));
    }
    Ok(store.join("baselines").join(format!("{name}.json")))
}
pub fn save_baseline(store: &Path, name: &str, run: &Path) -> Result<()> {
    let _lease = crate::runner::acquire_lease()?;
    let r = Run::load(run)?;
    if r.status != Status::Complete {
        return Err(error("baseline requires complete run"));
    }
    let run = fs::canonicalize(if run.is_dir() {
        run.join("run.json")
    } else {
        run.into()
    })?;
    let path = name_path(store, name)?;
    fs::create_dir_all(path.parent().unwrap())?;
    write_new(
        &path,
        &Baseline {
            sha256: rbench::model::hash_file(&run)?,
            run,
        },
    )?;
    println!("Saved baseline @{name}; existing names are never overwritten");
    Ok(())
}
pub fn resolve(store: &Path, value: &Path) -> Result<PathBuf> {
    if value == Path::new("last") && !value.exists() {
        return crate::artifacts::last(store);
    }
    if let Some(name) = value.to_str().and_then(|s| s.strip_prefix('@')) {
        let b: Baseline = serde_json::from_slice(&fs::read(name_path(store, name)?)?)?;
        if rbench::model::hash_file(&b.run)? != b.sha256 {
            return Err(error("baseline artifact changed since registration"));
        }
        Ok(b.run)
    } else {
        Ok(value.into())
    }
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
        if p.extension().is_some_and(|e| e == "json") {
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
