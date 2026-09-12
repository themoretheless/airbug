//! Compare committed local Git snapshots without touching the active checkout.
use crate::{project, runner};
use rbench::{error, model::write_new, Result};
use std::{collections::BTreeMap, path::Path, process::Command};
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let o = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if !o.status.success() {
        return Err(error(String::from_utf8_lossy(&o.stderr)));
    }
    Ok(String::from_utf8(o.stdout)?.trim().into())
}
pub struct Request<'a> {
    pub repo: &'a Path,
    pub base: &'a str,
    pub head: &'a str,
    pub target: &'a str,
    pub manifest: &'a Path,
    pub output: &'a Path,
    pub repetitions: u32,
    pub offline: bool,
    pub args: Vec<String>,
}
pub fn run(r: Request<'_>) -> Result<()> {
    run_with_alpha(r, 0.05)
}
pub fn run_with_alpha(r: Request<'_>, alpha: f64) -> Result<()> {
    if r.output.exists() {
        return Err(error("output already exists"));
    }
    if r.manifest.is_absolute()
        || r.manifest
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(error("manifest must be a relative path within snapshot"));
    }
    if r.repetitions == 0 || r.repetitions > 10000 {
        return Err(error("repetitions 1..10000 required"));
    }
    let sha = |v: &str| {
        git(
            r.repo,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{v}^{{commit}}"),
            ],
        )
    };
    let base = sha(r.base)?;
    let head = sha(r.head)?;
    let status = git(r.repo, &["status", "--porcelain"])?;
    if let Some(p) = r.output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(p)?;
    }
    std::fs::create_dir(r.output)?;
    let out = std::fs::canonicalize(r.output)?;
    let mut executables = vec![];
    let mut dirs = vec![];
    for (name, revision) in [("baseline", &base), ("candidate", &head)] {
        let dir = out.join("checkouts").join(name);
        std::fs::create_dir_all(&dir)?;
        // Read tracked blobs directly. Reject symlinks/submodules; never extract path-bearing archives.
        let tree = Command::new("git")
            .arg("-C")
            .arg(r.repo)
            .args(["ls-tree", "-rz", revision])
            .output()?;
        if !tree.status.success() {
            return Err(error("git ls-tree failed"));
        }
        for entry in tree.stdout.split(|b| *b == 0).filter(|b| !b.is_empty()) {
            let entry = std::str::from_utf8(entry)?;
            let (meta, path) = entry
                .split_once('\t')
                .ok_or_else(|| error("invalid git tree entry"))?;
            let fields: Vec<_> = meta.split_whitespace().collect();
            if fields.len() != 3 || fields[1] != "blob" || !matches!(fields[0], "100644" | "100755")
            {
                return Err(error("snapshot contains symlink/submodule; use prepared binaries for this repository"));
            }
            let path = Path::new(path);
            if path.is_absolute()
                || path
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
            {
                return Err(error("unsafe snapshot path"));
            }
            let blob = Command::new("git")
                .arg("-C")
                .arg(r.repo)
                .args(["cat-file", "blob", fields[2]])
                .output()?;
            if !blob.status.success() {
                return Err(error("cannot read git blob"));
            }
            let dest = dir.join(path);
            std::fs::create_dir_all(dest.parent().unwrap())?;
            std::fs::write(&dest, blob.stdout)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(
                    dest,
                    std::fs::Permissions::from_mode(if fields[0] == "100755" {
                        0o755
                    } else {
                        0o644
                    }),
                )?;
            }
        }
        let targets = project::discover(Some(&dir.join(r.manifest)), r.offline)?;
        let t = targets
            .iter()
            .find(|t| t.registered && format!("{}/{}", t.package, t.name) == r.target)
            .ok_or_else(|| error("registered package/target absent from snapshot"))?;
        executables.push(project::build_at(t, r.offline, Some(&dir.join("target")))?);
        dirs.push(t.manifest.parent().unwrap().to_path_buf());
    }
    let mut args = r.args;
    if !args.iter().any(|a| a == "--json") {
        args.push("--json".into());
    }
    let program = |i: usize| runner::Program {
        path: executables[i].clone(),
        args: args.clone(),
        env: Default::default(),
        cwd: Some(dirs[i].clone()),
    };
    write_new(
        &out.join("git.json"),
        &serde_json::json!({"baseline":base,"candidate":head,"dirty_checkout_ignored":!status.is_empty(),"repository":r.repo}),
    )?;
    let run = runner::run(
        runner::Plan {
            privacy: None,
            variants: Default::default(),
            start_pair: 0,
            candidate: program(1),
            baseline: Some(program(0)),
            repetitions: r.repetitions,
            timeout_ms: 60000,
            protocol: true,
            fixtures: vec![],
            contract: Default::default(),
            provenance: BTreeMap::from([
                ("git.baseline".into(), base),
                ("git.candidate".into(), head),
            ]),
        },
        &out.join("run"),
    )?;
    println!(
        "{}\nSaved {}",
        rbench::report::comparison(&rbench::analysis::compare(&run, None, 5., alpha)?),
        out.join("run").display()
    );
    Ok(())
}
