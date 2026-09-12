use rbench::{
    model::{hash_file, write_new},
    *,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub path: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub cwd: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(default)]
    pub privacy: Option<crate::privacy::Policy>,
    /// Additional variants compared against baseline in a balanced crossover.
    #[serde(default)]
    pub variants: BTreeMap<String, Program>,
    #[serde(default)]
    pub start_pair: u32,
    pub candidate: Program,
    pub baseline: Option<Program>,
    #[serde(default = "repeats")]
    pub repetitions: u32,
    #[serde(default = "timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub protocol: bool,
    #[serde(default)]
    pub fixtures: Vec<PathBuf>,
    #[serde(default)]
    pub contract: BTreeMap<String, String>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}
fn repeats() -> u32 {
    12
}
fn timeout() -> u64 {
    60_000
}
#[derive(Clone, Debug, Serialize)]
struct Entry {
    process: u32,
    pair: Option<u32>,
    variant: String,
}
static CANCELLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(unix)]
extern "C" fn cancel(_: libc::c_int) {
    CANCELLED.store(true, std::sync::atomic::Ordering::Relaxed);
}
pub fn cancelled() -> bool {
    CANCELLED.load(std::sync::atomic::Ordering::Relaxed)
}
fn progress(out: &Path, completed: usize, total: usize, variant: &str) -> Result<()> {
    let temp = out.join("progress.next.json");
    fs::write(
        &temp,
        serde_json::to_vec(
            &serde_json::json!({"state":"running","completed":completed,"total":total,"variant":variant}),
        )?,
    )?;
    fs::rename(temp, out.join("progress.json"))?;
    Ok(())
}
pub fn install_cancel_handler() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, cancel as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, cancel as *const () as libc::sighandler_t);
    }
}
pub(crate) struct Lease(PathBuf);
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGKILL);
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn resolve(p: &mut Program) -> Result<()> {
    p.path = fs::canonicalize(&p.path)?;
    if let Some(cwd) = &p.cwd {
        p.cwd = Some(fs::canonicalize(cwd)?);
    }
    Ok(())
}
pub fn run(mut plan: Plan, out: &Path) -> Result<Run> {
    if plan.repetitions == 0
        || plan.repetitions > 10000
        || plan.timeout_ms == 0
        || plan.timeout_ms > 86_400_000
    {
        return Err(error(
            "repetitions 1..10000; timeout-ms 1..86400000 required",
        ));
    }
    plan.contract.insert(
        "runner.log_capture".into(),
        "bounded piped capture v1; reader threads outside worker".into(),
    );
    plan.contract.insert(
        "runner.environment_policy".into(),
        serde_json::to_string(&plan.privacy)?,
    );
    let privacy = plan
        .privacy
        .as_ref()
        .map(|p| p.prepare(&plan))
        .transpose()?;
    validate_variants(&plan)?;
    for p in plan.variants.values_mut() {
        resolve(p)?;
    }
    resolve(&mut plan.candidate)?;
    if let Some(p) = &mut plan.baseline {
        resolve(p)?;
    }
    let mut hashes = BTreeMap::new();
    for p in plan
        .fixtures
        .iter()
        .chain(std::iter::once(&plan.candidate.path))
        .chain(plan.baseline.iter().map(|p| &p.path))
        .chain(plan.variants.values().map(|p| &p.path))
    {
        let p = fs::canonicalize(p)?;
        hashes.insert(p.clone(), hash_file(&p)?);
    }
    let _lease = acquire_lease()?;
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(out)?;
    fs::create_dir(out.join("logs"))?;
    let mut result = Run::new();
    if let Some(policy) = &privacy {
        let env: BTreeMap<_, _> = policy
            .environment()
            .iter()
            .filter(|(k, _)| !plan.privacy.as_ref().unwrap().secret_env.contains(k))
            .collect();
        let environment = serde_json::to_string(&env)?;
        plan.contract
            .insert("runner.allowed_environment".into(), environment.clone());
        result
            .provenance
            .insert("allowed_environment".into(), environment);
    }
    result.provenance.extend(
        plan.provenance
            .iter()
            .map(|(k, v)| (format!("user.{k}"), v.clone())),
    );
    result
        .provenance
        .insert("runner".into(), env!("CARGO_PKG_VERSION").into());
    result.environment.insert(
        "host".into(),
        std::env::var("HOSTNAME").unwrap_or_else(|_| {
            Command::new("hostname")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().into())
                .unwrap_or("unknown".into())
        }),
    );
    if let Ok(o) = Command::new("uname").arg("-r").output() {
        result.environment.insert(
            "kernel".into(),
            String::from_utf8_lossy(&o.stdout).trim().into(),
        );
    }
    result.notes.push("Sequential processes; AB/BA order balanced. Whole-process wall includes startup and polling (1 ms). Global lease covers rbench only; thermals, frequency and other applications are uncontrolled. Inherited environment is not fully captured; use explicit plan.env for relevant settings.".into());
    result
        .provenance
        .insert("input_hashes".into(), serde_json::to_string(&hashes)?);
    if let Some(privacy) = &privacy {
        privacy.reject_literals(&result)?;
    }
    let schedule = schedule(&plan);
    if let Some(policy) = &plan.privacy {
        policy.prepare(&plan)?;
        result.notes.push("Explicit environment allowlist enabled. Secret values redacted literally and JSON-escaped before log persistence. Encoded or transformed disclosures are outside this policy.".into());
    }
    write_new(&out.join("plan.json"), &plan)?;
    result
        .provenance
        .insert("plan.sha256".into(), hash_file(&out.join("plan.json"))?);
    write_new(&out.join("schedule.json"), &schedule)?;
    write_new(
        &out.join("status.json"),
        &serde_json::json!({"state":"running"}),
    )?;
    let total_processes = schedule.len();
    progress(out, 0, total_processes, "")?;
    let experiment_start = Instant::now();
    let mut execution = (|| -> Result<()> {
        for e in schedule {
            if CANCELLED.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(error("run cancelled"));
            }
            let observation_start = result.observations.len();
            let p = if e.variant == "baseline" {
                plan.baseline.as_ref().unwrap()
            } else if e.variant == "candidate" {
                &plan.candidate
            } else {
                &plan.variants[&e.variant]
            };
            let stdout = out
                .join("logs")
                .join(format!("{}-{}.stdout", e.process, e.variant));
            let stderr = out
                .join("logs")
                .join(format!("{}-{}.stderr", e.process, e.variant));
            let mut command = Command::new(&p.path);
            if let Some(policy) = &privacy {
                policy.command(&mut command);
            }
            command
                .args(&p.args)
                .envs(&p.env)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::null());
            if let Some(c) = &p.cwd {
                command.current_dir(c);
            }
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
            let completed = e.process as usize;
            let eta = if completed > 0 {
                format!(
                    "~{:.0}s",
                    experiment_start.elapsed().as_secs_f64() / completed as f64
                        * (total_processes - completed) as f64
                )
            } else {
                "estimating".into()
            };
            eprintln!(
                "rbench: process {}/{} {} · ETA {}",
                completed + 1,
                total_processes,
                e.variant,
                eta
            );
            progress(out, completed, total_processes, &e.variant)?;
            let start = Instant::now();
            let mut heartbeat = Instant::now();
            // Declare capture before child: error unwinding kills the process group before joining pipes.
            let mut logs = crate::privacy::Logs::new();
            let mut child = ChildGuard(command.spawn()?);
            let needles = privacy.as_ref().map(|p| p.needles()).unwrap_or_default();
            logs.start(child.0.stdout.take().unwrap(), &stdout, needles.clone())?;
            logs.start(child.0.stderr.take().unwrap(), &stderr, needles)?;
            loop {
                if CANCELLED.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err(error("run cancelled"));
                }
                if fs::metadata(&stdout)?.len() > 64 * 1024 * 1024
                    || fs::metadata(&stderr)?.len() > 64 * 1024 * 1024
                {
                    return Err(error("worker log exceeds 64 MiB limit"));
                }
                if let Some(status) = child.0.try_wait()? {
                    if !status.success() {
                        return Err(error(format!(
                            "{} process {} exited {status}; see {}",
                            e.variant,
                            e.process,
                            stderr.display()
                        )));
                    }
                    break;
                }
                if start.elapsed() >= Duration::from_millis(plan.timeout_ms) {
                    return Err(error(format!(
                        "{} process {} timed out",
                        e.variant, e.process
                    )));
                }
                if heartbeat.elapsed() >= Duration::from_secs(1) {
                    use std::io::{Read, Seek, SeekFrom};
                    let mut log = File::open(&stderr)?;
                    let length = log.metadata()?.len();
                    log.seek(SeekFrom::Start(length.saturating_sub(4096)))?;
                    let mut tail = String::new();
                    let _ = log.read_to_string(&mut tail);
                    let phase = tail
                        .lines()
                        .rev()
                        .find(|s| s.starts_with("rbench:"))
                        .unwrap_or("worker executing");
                    let phase: String = phase
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(240)
                        .collect();
                    eprintln!(
                        "rbench: process {}/{} · {:.0}s elapsed · {}",
                        completed + 1,
                        total_processes,
                        start.elapsed().as_secs_f64(),
                        phase
                    );
                    heartbeat = Instant::now();
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            let elapsed = start.elapsed().as_nanos();
            drop(child);
            logs.finish()?;
            // Do not import results from changed inputs, including changes between pairs.
            for (p, expected) in &hashes {
                if hash_file(p)? != *expected {
                    return Err(error(format!("input/binary changed: {}", p.display())));
                }
            }
            if plan.protocol {
                if fs::metadata(&stdout)?.len() > 64 * 1024 * 1024 {
                    return Err(error("worker stdout exceeds 64 MiB protocol limit"));
                }
                let text = fs::read_to_string(&stdout)?;
                let messages: Vec<_> = text
                    .lines()
                    .filter_map(|l| l.strip_prefix("RBENCH_RESULT="))
                    .collect();
                if messages.len() != 1 {
                    return Err(error("expected exactly one RBENCH_RESULT message"));
                }
                let mut worker: Run = serde_json::from_str(messages[0])?;
                worker.validate()?;
                if let Some(descriptor) = plan.provenance.get("diagnostic.expected_descriptor") {
                    let expected: Case = serde_json::from_str(descriptor)?;
                    if worker.cases != vec![expected] {
                        return Err(error(
                            "profiler worker changed workload descriptor or contract",
                        ));
                    }
                }
                if let Some(case) = plan.provenance.get("diagnostic.expected_case") {
                    if worker.cases.len() != 1 || worker.cases[0].id != *case {
                        return Err(error("profiler worker changed requested case identity"));
                    }
                }
                if worker.status != Status::Complete {
                    return Err(error("worker returned incomplete run"));
                }
                if worker
                    .observations
                    .iter()
                    .any(|o| o.variant != "candidate" || o.process != 0 || o.pair.is_some())
                {
                    return Err(error("protocol worker must have one local process, candidate variant and no pair IDs"));
                }
                for note in &worker.notes {
                    if !result.notes.contains(note) {
                        result.notes.push(note.clone());
                    }
                }
                result.provenance.insert(
                    format!("worker.{}", e.process),
                    serde_json::to_string(&worker.provenance)?,
                );
                for c in &mut worker.cases {
                    c.contract.insert(
                        "worker_environment".into(),
                        serde_json::to_string(&worker.environment)?,
                    );
                    for (k, v) in &plan.contract {
                        c.contract.insert(format!("experiment.{k}"), v.clone());
                    }
                    c.contract.insert(
                        "fixture_hashes".into(),
                        serde_json::to_string(
                            &plan
                                .fixtures
                                .iter()
                                .map(|p| Ok((p.to_string_lossy().to_string(), hash_file(p)?)))
                                .collect::<Result<BTreeMap<_, _>>>()?,
                        )?,
                    );
                }
                if let Some(expected) = plan.provenance.get("session.expected_cases") {
                    let expected: Vec<Case> = serde_json::from_str(expected)?;
                    if worker.cases != expected {
                        return Err(error(
                            "resumed worker context/contracts differ from parent session",
                        ));
                    }
                }
                if result.cases.is_empty() {
                    result.cases = worker.cases;
                } else if result.cases != worker.cases {
                    return Err(error("worker cases/contracts changed across processes"));
                }
                for mut o in worker.observations {
                    o.process = e.process;
                    o.pair = e.pair;
                    o.variant = e.variant.clone();
                    result.observations.push(o);
                }
            } else {
                if result.cases.is_empty() {
                    let mut contract = plan.contract.clone();
                    contract.insert(
                        "completion".into(),
                        "process exit; launch and supervisor polling included".into(),
                    );
                    contract.insert(
                        "fixture_hashes".into(),
                        serde_json::to_string(
                            &plan
                                .fixtures
                                .iter()
                                .map(|p| Ok((p.to_string_lossy().to_string(), hash_file(p)?)))
                                .collect::<Result<BTreeMap<_, _>>>()?,
                        )?,
                    );
                    result.cases.push(Case {
                        id: "process".into(),
                        contract,
                        metrics: vec![Metric::duration(
                            "wall",
                            "whole process incl startup and polling",
                            "process_total",
                        )],
                    });
                }
                result.observations.push(Observation {
                    case: "process".into(),
                    metric: "wall".into(),
                    variant: e.variant.clone(),
                    process: e.process,
                    pair: e.pair,
                    sequence: 0,
                    value: Some(elapsed.to_string()),
                    operations: 1,
                    availability: Availability::Available,
                });
            }
            // Per-process journal survives interruption without pretending completion.
            write_new(
                &out.join(format!("partial-{}.json", e.process)),
                &serde_json::json!({"process": e.process,"observations": &result.observations[observation_start..]}),
            )?;
            progress(out, e.process as usize + 1, total_processes, &e.variant)?;
        }
        Ok(())
    })();
    if execution.is_ok() {
        if let Some(path) = plan.candidate.env.get("RBENCH_MEMORY_OUTPUT") {
            match rbench::memory::Profile::load(Path::new(path)) {
                Ok(_) => {
                    result
                        .provenance
                        .insert("memory.sha256".into(), hash_file(Path::new(path))?);
                }
                Err(e) => {
                    execution = Err(error(format!("memory profile missing or invalid: {e}")));
                }
            }
        }
    }
    result.status = if CANCELLED.load(std::sync::atomic::Ordering::Relaxed) {
        Status::Cancelled
    } else if execution.is_ok() {
        Status::Complete
    } else {
        Status::Failed
    };
    if execution.is_ok() {
        if let Err(e) = result.validate() {
            result.status = Status::Failed;
            execution = Err(e);
        }
    }
    if let Err(e) = &execution {
        result.notes.push(e.to_string());
    }
    if let Some(privacy) = &privacy {
        privacy.reject_literals(&result)?;
    }
    write_new(&out.join("run.json"), &result)?;
    let status = serde_json::json!({"state":result.status,"error":execution.as_ref().err().map(|e|e.to_string())});
    write_new(&out.join("status-final.json"), &status)?;
    execution?;
    result.validate()?;
    Ok(result)
}

/// Read-only preflight. Never invokes the workload or writes the output directory.
pub fn preflight(mut plan: Plan, out: &Path) -> Result<serde_json::Value> {
    if out.exists() {
        return Err(error("output already exists"));
    }
    if plan.repetitions == 0
        || plan.repetitions > 10000
        || plan.timeout_ms == 0
        || plan.timeout_ms > 86400000
    {
        return Err(error("invalid repetitions/timeout"));
    }
    if let Some(policy) = &plan.privacy {
        policy.prepare(&plan)?;
    }
    validate_variants(&plan)?;
    let mut hashes = BTreeMap::new();
    for p in std::iter::once(&mut plan.candidate)
        .chain(plan.baseline.iter_mut())
        .chain(plan.variants.values_mut())
    {
        resolve(p)?;
        if !p.path.is_file() {
            return Err(error("program must be a file"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&p.path)?.permissions().mode() & 0o111 == 0 {
                return Err(error("program is not executable"));
            }
        }
        if p.cwd.as_ref().is_some_and(|p| !p.is_dir()) {
            return Err(error("cwd is not a directory"));
        }
        hashes.insert(p.path.clone(), hash_file(&p.path)?);
        for v in p.env.values_mut() {
            *v = "<set; value omitted in preview>".into();
        }
    }
    for fixture in &plan.fixtures {
        let p = fs::canonicalize(fixture)?;
        hashes.insert(p.clone(), hash_file(&p)?);
    }
    let mut parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    while !parent.exists() {
        parent = parent
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
    }
    if !parent.is_dir() {
        return Err(error("output ancestor is not a directory"));
    }
    let space = Command::new("df")
        .arg("-k")
        .arg(parent)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
    let order = schedule(&plan);
    Ok(
        serde_json::json!({"plan":plan,"order":order,"hashes":hashes,"output":out,"disk_space_kib":space,"capabilities":{"clock":"Instant","unix_process_group":cfg!(unix),"gpu":"scenario-owned; not probed"},"note":"No workload executed. Actual worker cases/driver capabilities are not inferred from a binary. Worker --dry-run lists registered Suite cases."}),
    )
}

fn validate_variants(plan: &Plan) -> Result<()> {
    if plan
        .start_pair
        .checked_add(plan.repetitions)
        .is_none_or(|n| n > 10000)
    {
        return Err(error("pair budget exceeds 10000"));
    }
    if plan.variants.len() > 14 || (!plan.variants.is_empty() && plan.baseline.is_none()) {
        return Err(error(
            "additional variants require baseline; at most 16 total variants",
        ));
    }
    for name in plan.variants.keys() {
        if name.is_empty()
            || name == "baseline"
            || name == "candidate"
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(error("invalid or reserved variant name"));
        }
    }
    if !plan.variants.is_empty()
        && (plan.repetitions as usize) % (2 * (plan.variants.len() + 2)) != 0
    {
        return Err(error("multi-variant repetitions must be a multiple of twice the variant count (balanced reversed rotations)"));
    }
    Ok(())
}
fn schedule(plan: &Plan) -> Vec<Entry> {
    let mut schedule = vec![];
    for pair in plan.start_pair..plan.start_pair + plan.repetitions {
        let mut names = if plan.baseline.is_some() {
            vec!["baseline".to_string(), "candidate".to_string()]
        } else {
            vec!["candidate".to_string()]
        };
        names.extend(plan.variants.keys().cloned());
        if plan.variants.is_empty() {
            if pair % 2 == 1 {
                names.reverse();
            }
        } else {
            let n = names.len();
            names.rotate_left(pair as usize % n);
            if (pair as usize / n) % 2 == 1 {
                names.reverse();
            }
        }
        for variant in names {
            schedule.push(Entry {
                process: schedule.len() as u32,
                pair: plan.baseline.as_ref().map(|_| pair),
                variant,
            });
        }
    }
    schedule
}

pub(crate) fn acquire_lease() -> Result<Lease> {
    // Per-user lock coordinates local rbench processes across output directories.
    let uid = std::env::var("USER")
        .unwrap_or_else(|_| "default".into())
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    let lock = std::env::temp_dir().join(format!("rbench-{uid}.lock"));
    let mut lock_file=OpenOptions::new().write(true).create_new(true).open(&lock).map_err(|e|error(format!("cannot acquire {}: {e}; another runner may be active. Inspect stale lock before removing it",lock.display())))?;
    use std::io::Write;
    writeln!(lock_file, "pid={}", std::process::id())?;
    Ok(Lease(lock))
}
