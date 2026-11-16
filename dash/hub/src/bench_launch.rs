//! Live benchmark target discovery, subprocess launcher, and cancellation for the hub.
//!
//! Launches `cargo bench --bench <target> -- ... --output <out_dir>` directly (no shell),
//! tracks active child processes by `run_id`, and streams stdout/stderr to `<out_dir>/logs/`.
use crate::{
    app::HubApp,
    config::RootPaths,
    error::{HubError, Result},
    runs::{CreateRunRequest, is_uuidish},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, OnceLock},
    thread,
};

#[derive(Debug, Default, Deserialize)]
pub struct LaunchRequest {
    pub bench: Option<String>,
    pub title: Option<String>,
    pub filter: Option<String>,
    #[serde(default)]
    pub exact: bool,
    pub profile: Option<String>,
    pub samples: Option<u32>,
    pub warmup_ms: Option<u64>,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub env_mixed_case: bool,
}

static ACTIVE_RUNS: OnceLock<Mutex<HashMap<String, Arc<Mutex<Child>>>>> = OnceLock::new();

fn active_runs() -> &'static Mutex<HashMap<String, Arc<Mutex<Child>>>> {
    ACTIVE_RUNS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn valid_target_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

fn valid_arg_value(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && !s.contains('\0')
}

fn unquote_toml_str(raw: &str) -> Option<String> {
    let s = raw.trim();
    if let Some(rest) = s.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    if let Some(rest) = s.strip_prefix('\'') {
        let end = rest.find('\'')?;
        return Some(rest[..end].to_string());
    }
    None
}

/// Parse `[package].name` and `[[bench]].name` entries from `<root>/Cargo.toml`.
pub fn parse_cargo_targets(cargo_toml: &str) -> (Option<String>, Vec<String>) {
    let mut pkg_name = None;
    let mut benches = Vec::new();
    let mut section = String::new();

    for raw_line in cargo_toml.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("[[") && line.ends_with("]]") {
            section = line[2..line.len() - 2].trim().to_string();
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            if key == "name"
                && let Some(val) = unquote_toml_str(v)
            {
                if section == "package" && pkg_name.is_none() {
                    pkg_name = Some(val);
                } else if section == "bench" && valid_target_name(&val) && !benches.contains(&val)
                {
                    benches.push(val);
                }
            }
        }
    }
    (pkg_name, benches)
}

/// `GET /api/v1/bench/targets` — discover runnable benchmark targets in `root`.
pub fn targets(root: &Path) -> Value {
    let path = root.join("Cargo.toml");
    let text = fs::read_to_string(&path).unwrap_or_default();
    let (package, targets) = parse_cargo_targets(&text);
    let package = package.or_else(|| {
        root.file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
    });
    json!({
        "package": package,
        "targets": targets,
    })
}

fn write_json_atomic(path: &Path, val: &Value) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_vec_pretty(val)?;
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Return true if a hub-launched benchmark process for `run_id` is still alive.
pub fn is_running(run_id: &str) -> bool {
    let Ok(mut map) = active_runs().lock() else {
        return false;
    };
    let Some(handle) = map.get(run_id).cloned() else {
        return false;
    };
    let alive = match handle.lock() {
        Ok(mut child) => child.try_wait().ok().flatten().is_none(),
        Err(_) => false,
    };
    if !alive {
        map.remove(run_id);
    }
    alive
}

fn active_run_id() -> Option<String> {
    let mut map = active_runs().lock().ok()?;
    let mut finished = Vec::new();
    let mut live = None;
    for (id, handle) in map.iter() {
        let alive = match handle.lock() {
            Ok(mut child) => child.try_wait().ok().flatten().is_none(),
            Err(_) => false,
        };
        if alive {
            live = Some(id.clone());
            break;
        } else {
            finished.push(id.clone());
        }
    }
    for id in finished {
        map.remove(&id);
    }
    live
}

/// `POST /api/v1/bench/launch` — allocate a run session and spawn `cargo bench`.
pub fn launch(app: &HubApp, req: LaunchRequest) -> Result<Value> {
    if let Some(running_id) = active_run_id() {
        return Err(HubError::msg(format!(
            "busy: bench run {running_id} is already running"
        )));
    }

    let root = &app.paths.root;
    let discovered = targets(root);
    let default_target = discovered["targets"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("compare")
        .to_string();

    let bench = req
        .bench
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or(default_target);
    if !valid_target_name(&bench) {
        return Err(HubError::msg("invalid bench target name"));
    }

    let filter = req
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Some(ref f) = filter
        && !valid_arg_value(f)
    {
        return Err(HubError::msg("invalid filter pattern"));
    }

    if req.excludes.len() > 32 || req.excludes.iter().any(|e| !valid_arg_value(e)) {
        return Err(HubError::msg("invalid exclude pattern"));
    }

    if let Some(samples) = req.samples
        && !(1..=1000).contains(&samples)
    {
        return Err(HubError::msg("samples must be between 1 and 1000"));
    }

    if let Some(warmup_ms) = req.warmup_ms
        && warmup_ms > 60_000
    {
        return Err(HubError::msg("warmup_ms must be <= 60000"));
    }

    let profile = match req
        .profile
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => None,
        Some(p @ ("quick" | "normal" | "thorough")) => Some(p.to_string()),
        Some(_) => {
            return Err(HubError::msg(
                "profile must be quick, normal, or thorough",
            ));
        }
    };

    let pkg = discovered["package"].as_str().unwrap_or("bench");
    let auto_title = match &filter {
        Some(f) => format!("{pkg} {bench} · {f}"),
        None => format!("{pkg} {bench}"),
    };
    let title = req
        .title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or(auto_title);

    let mut bench_args: Vec<String> = Vec::new();
    if let Some(ref p) = profile {
        bench_args.push("--profile".into());
        bench_args.push(p.clone());
    }
    if let Some(samples) = req.samples {
        bench_args.push("--samples".into());
        bench_args.push(samples.to_string());
    }
    if let Some(warmup_ms) = req.warmup_ms {
        bench_args.push("--warmup-ms".into());
        bench_args.push(warmup_ms.to_string());
    }
    if let Some(ref f) = filter {
        bench_args.push("--filter".into());
        bench_args.push(f.clone());
        if req.exact {
            bench_args.push("--exact".into());
        }
    }
    for ex in &req.excludes {
        bench_args.push("--exclude".into());
        bench_args.push(ex.clone());
    }

    let mut display_parts = vec![
        "cargo".to_string(),
        "bench".into(),
        "--bench".into(),
        bench.clone(),
        "--".into(),
    ];
    display_parts.extend(bench_args.iter().cloned());
    let display_cmd = display_parts.join(" ");

    let created = app.runs.create(CreateRunRequest {
        title: Some(title),
        command: Some(display_cmd.clone()),
    })?;
    let out_dir = Path::new(&created.out_dir);
    let logs_dir = out_dir.join("logs");
    fs::create_dir_all(&logs_dir)?;

    let variant_label = match &filter {
        Some(f) => format!("compiling & starting ({f})…"),
        None => "compiling benchmark…".to_string(),
    };
    write_json_atomic(
        &out_dir.join("progress.json"),
        &json!({
            "state": "building",
            "completed": 0,
            "total": 0,
            "variant": variant_label,
        }),
    )?;

    let stdout_file = File::create(logs_dir.join("stdout.log"))?;
    let stderr_file = File::create(logs_dir.join("stderr.log"))?;

    let mut cmd = Command::new("cargo");
    cmd.current_dir(root)
        .arg("bench")
        .arg("--bench")
        .arg(&bench)
        .arg("--")
        .args(&bench_args)
        .arg("--output")
        .arg(&created.out_dir)
        .env("AIRBUG_HUB", format!("http://127.0.0.1:{}", app.port))
        .env("AIRBUG_DASH_URL", &created.dash_url)
        .env(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            format!("http://127.0.0.1:{}", app.port),
        )
        .env(
            "OTEL_RESOURCE_ATTRIBUTES",
            format!("airbug.hub_id={},airbug.run_id={}", app.hub_id, created.run_id),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));
    if req.env_mixed_case {
        cmd.env("LIN_BENCH_MIXED_CASE", "1");
    }

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(err) => {
            let msg = format!("failed to spawn cargo bench: {err}");
            let _ = write_json_atomic(
                &out_dir.join("status-final.json"),
                &json!({ "state": "failed", "error": msg }),
            );
            return Err(HubError::msg(msg));
        }
    };

    let run_id = created.run_id.clone();
    let out_dir_owned = out_dir.to_path_buf();
    let handle = Arc::new(Mutex::new(child));
    if let Ok(mut map) = active_runs().lock() {
        map.insert(run_id.clone(), Arc::clone(&handle));
    }

    thread::spawn(move || {
        let mut exited_ok = false;
        let mut note: Option<String> = None;
        loop {
            let done = match handle.lock() {
                Ok(mut child) => match child.try_wait() {
                    Ok(Some(status)) => {
                        exited_ok = status.success();
                        if !exited_ok {
                            note = Some(format!("process exited with {status}"));
                        }
                        true
                    }
                    Ok(None) => false,
                    Err(e) => {
                        note = Some(format!("wait failed: {e}"));
                        true
                    }
                },
                Err(_) => true,
            };
            if done {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(200));
        }
        if let Ok(mut map) = active_runs().lock() {
            map.remove(&run_id);
        }
        let final_path = out_dir_owned.join("status-final.json");
        if !final_path.is_file() {
            let state = if exited_ok { "complete" } else { "failed" };
            let _ = write_json_atomic(
                &final_path,
                &json!({
                    "state": state,
                    "error": note,
                }),
            );
        }
    });

    Ok(json!({
        "run_id": created.run_id,
        "href": format!("#/run/{}", created.run_id),
        "dash_url": created.dash_url,
        "out_dir": created.out_dir,
        "command": display_cmd,
    }))
}

/// `POST /api/v1/bench/runs/<id>/cancel` — terminate an active hub-launched bench run.
pub fn cancel(root: &Path, run_id: &str) -> Result<Value> {
    if !is_uuidish(run_id) {
        return Err(HubError::msg("invalid run_id"));
    }
    let handle = active_runs()
        .lock()
        .ok()
        .and_then(|map| map.get(run_id).cloned())
        .ok_or_else(|| HubError::msg("bench run is not running under the hub"))?;
    if let Ok(mut child) = handle.lock() {
        let _ = child.kill();
    }
    let out_dir = RootPaths::new(root).bench_run_dir(run_id);
    if out_dir.is_dir() {
        let _ = write_json_atomic(
            &out_dir.join("status-final.json"),
            &json!({ "state": "cancelled" }),
        );
    }
    Ok(json!({ "ok": true, "run_id": run_id, "state": "cancelled" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_package_and_bench_targets_from_cargo_toml() {
        let toml = r#"
[workspace]
members = ["."]

[package]
name = "lin" # database
version = "0.7.1"

[[bench]]
name = "compare"
harness = false

[[bench]]
name = 'micro-suite'
harness = false
"#;
        let (pkg, benches) = parse_cargo_targets(toml);
        assert_eq!(pkg.as_deref(), Some("lin"));
        assert_eq!(benches, vec!["compare", "micro-suite"]);
    }
}
