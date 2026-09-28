//! `airbug-hub test` (`cargo airbug test`) — run `cargo test` as a live, per-test run.
//!
//! The runner lists tests first (`cargo test … -- --list`) so the hub can show a real
//! progress bar, then runs them with `AIRBUG_REPORT_DIR` pointing into the run directory so
//! `airbug::report` steps, comparisons and attachments land beside the results. Everything is
//! plain files (see [`model`]); the hub only reads them, so a run is live in the dashboard
//! whether or not the hub was started first.
pub mod events;
pub mod html;
pub mod libtest;
pub mod model;

use libtest::Collector;
use model::{
    Git, MANIFEST_SCHEMA, Manifest, Progress, REPORT_SCHEMA, Report, RunState, SuiteState,
    TestStatus, now_ms,
};
use std::{
    env, fs,
    io::{self, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

/// Directory of test runs under the project root.
pub const RUNS_DIR: &str = ".airbug/runs";
const DEFAULT_KEEP: usize = 50;
const WRITE_EVERY: Duration = Duration::from_millis(300);
/// How long to keep reading after cargo exited (a grandchild may hold the pipe open).
const DRAIN_AFTER_EXIT: Duration = Duration::from_secs(2);
const BUILD_LOG_LINES: usize = 200;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub root: PathBuf,
    pub out: Option<PathBuf>,
    pub title: Option<String>,
    pub port: u16,
    pub list: bool,
    pub quiet: bool,
    pub keep: usize,
    pub cargo_args: Vec<String>,
    pub test_args: Vec<String>,
}

pub const USAGE: &str = "\
airbug-hub test [--root PATH] [--out DIR] [--title TEXT] [--port N] [--no-list] [--quiet]
                [--keep N] [-- <cargo test args> [-- <test binary args>]]
    # runs cargo test, writes .airbug/runs/<id>/ (or --out DIR), live in the hub at
    # http://127.0.0.1:<port>/#/tests/<id>; exit code follows the tests
    # --no-list skips the listing pass (needed for harness = false test targets)";

pub fn parse_args<I, S>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut options = Options {
        root: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        out: None,
        title: None,
        port: crate::config::DEFAULT_PORT,
        list: true,
        quiet: false,
        keep: DEFAULT_KEEP,
        cargo_args: Vec::new(),
        test_args: Vec::new(),
    };
    let mut args = args.into_iter().map(Into::into);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "--root" => options.root = PathBuf::from(value("--root")?),
            "--out" | "-o" => options.out = Some(PathBuf::from(value("--out")?)),
            "--title" => options.title = Some(value("--title")?),
            "--port" => {
                options.port = value("--port")?
                    .parse()
                    .map_err(|_| "invalid --port".to_string())?
            }
            "--keep" => {
                options.keep = value("--keep")?
                    .parse()
                    .map_err(|_| "invalid --keep".to_string())?
            }
            "--no-list" => options.list = false,
            "--quiet" | "-q" => options.quiet = true,
            "--" => {
                let rest: Vec<String> = args.by_ref().collect();
                match rest.iter().position(|a| a == "--") {
                    Some(split) => {
                        options.cargo_args = rest[..split].to_vec();
                        options.test_args = rest[split + 1..].to_vec();
                    }
                    None => options.cargo_args = rest,
                }
                break;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(options)
}

/// Entry point for `airbug-hub test`; returns the process exit code.
pub fn main(args: Vec<String>) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            return 2;
        }
    };
    match run(options) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("airbug test: {err}");
            1
        }
    }
}

fn run(options: Options) -> io::Result<i32> {
    let root = options
        .root
        .canonicalize()
        .unwrap_or_else(|_| options.root.clone());
    let run_id = uuid::Uuid::new_v4().to_string();
    let in_store = options.out.is_none();
    let dir = match &options.out {
        Some(out) if out.is_absolute() => out.clone(),
        Some(out) => root.join(out),
        None => root.join(RUNS_DIR).join(&run_id),
    };
    fs::create_dir_all(&dir)?;
    let dir = dir.canonicalize().unwrap_or(dir);
    let events_dir = dir.join("events");
    if events_dir.is_dir() {
        fs::remove_dir_all(&events_dir)?;
    }
    fs::create_dir_all(&events_dir)?;

    let run_args = run_command(&options, false);
    let title = options.title.clone().unwrap_or_else(|| {
        let args = options.cargo_args.join(" ");
        format!("cargo test {args}").trim().to_string()
    });
    let git = git_info(&root);
    let started_at_ms = now_ms();
    let manifest = Manifest {
        schema: MANIFEST_SCHEMA.into(),
        kind: "test".into(),
        run_id: run_id.clone(),
        title: title.clone(),
        command: std::iter::once("cargo".to_string())
            .chain(run_args.iter().cloned())
            .collect(),
        cwd: root.display().to_string(),
        git: git.clone(),
        host: env::var("HOSTNAME")
            .or_else(|_| env::var("COMPUTERNAME"))
            .unwrap_or_default(),
        started_at_ms,
    };
    write_json(&dir.join("manifest.json"), &manifest, true)?;
    // A stale report from an earlier run in the same --out folder must not survive.
    for stale in ["report.json", "progress.json", "index.html", "output.log"] {
        let _ = fs::remove_file(dir.join(stale));
    }

    let report = Report {
        schema: REPORT_SCHEMA.into(),
        run_id: run_id.clone(),
        title,
        state: RunState::Building,
        commit: git.commit.clone(),
        git,
        started_at_ms,
        ..Report::default()
    };
    let mut collector = Collector::new(report);
    let mut store = Store {
        dir: dir.clone(),
        run_id: run_id.clone(),
        started: Instant::now(),
        last: None,
    };
    store.write(&mut collector, true);
    fs::write(dir.join("index.html"), html::render(&collector.report))?;

    let live_url = format!("http://127.0.0.1:{}/#/tests/{run_id}", options.port);
    eprintln!("airbug test: run {run_id}");
    eprintln!("airbug test: files {}", dir.display());
    if in_store {
        match probe_hub(options.port, &root) {
            HubProbe::Ours => eprintln!("Live tests: {live_url}"),
            HubProbe::Other(other) => eprintln!(
                "airbug test: hub on :{} serves {other}, not this root — start one with \
                 `cargo airbug serve --port <N>` to watch live",
                options.port
            ),
            HubProbe::Absent => eprintln!(
                "airbug test: no hub on :{} — `cargo airbug serve` shows this run live at {live_url}",
                options.port
            ),
        }
    }

    let _ = ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst));
    let mut log = fs::File::create(dir.join("output.log"))?;
    let resource = format!("airbug.run_id={run_id}");
    let otel_resource = match env::var("OTEL_RESOURCE_ATTRIBUTES") {
        Ok(existing) if !existing.is_empty() => format!("{existing},{resource}"),
        _ => resource,
    };
    let env_vars = [
        (
            "AIRBUG_REPORT_DIR".to_string(),
            events_dir.display().to_string(),
        ),
        ("AIRBUG_RUN_ID".to_string(), run_id.clone()),
        ("CARGO_TERM_COLOR".to_string(), "never".to_string()),
        ("OTEL_RESOURCE_ATTRIBUTES".to_string(), otel_resource),
    ];

    let mut exit_code = None;
    if options.list {
        let list_args = run_command(&options, true);
        writeln!(log, "$ cargo {}", list_args.join(" "))?;
        let mut tail: Vec<String> = Vec::new();
        let status = stream(&root, &list_args, &env_vars, &mut |line| {
            let Some(line) = line else {
                store.write(&mut collector, false);
                return;
            };
            let _ = writeln!(log, "{line}");
            collector.list_line(line);
            tail.push(line.to_string());
            if tail.len() > BUILD_LOG_LINES {
                tail.remove(0);
            }
            if !options.quiet && is_build_line(line) {
                eprintln!("{line}");
            }
        })?;
        if !status.success() {
            collector.report.state = if INTERRUPTED.load(Ordering::SeqCst) {
                RunState::Interrupted
            } else {
                RunState::Error
            };
            collector.report.build_log = Some(tail.join("\n"));
            exit_code = Some(status.code().unwrap_or(1));
        }
    }

    if exit_code.is_none() {
        collector.report.state = RunState::Running;
        store.write(&mut collector, true);
        writeln!(log, "$ cargo {}", run_args.join(" "))?;
        let started = store.started;
        let status = stream(&root, &run_args, &env_vars, &mut |line| {
            let Some(line) = line else {
                store.write(&mut collector, false);
                return;
            };
            let _ = writeln!(log, "{line}");
            collector.run_line(line, started.elapsed().as_secs_f64());
            if !options.quiet {
                println!("{line}");
            }
        })?;
        collector.finish();
        events::fold(&mut collector, &events_dir);
        let crashed = collector
            .report
            .suites
            .iter()
            .any(|s| s.state == SuiteState::Crashed);
        collector.report.state = if INTERRUPTED.load(Ordering::SeqCst) {
            RunState::Interrupted
        } else if collector.report.totals.failed > 0 || crashed {
            RunState::Failed
        } else if !status.success() {
            RunState::Error
        } else {
            RunState::Passed
        };
        if collector.report.state == RunState::Error && collector.report.tests.is_empty() {
            collector.report.build_log = Some(tail_of(&dir.join("output.log"), BUILD_LOG_LINES));
        }
        exit_code = Some(status.code().unwrap_or(1));
    }

    let finished = now_ms();
    collector.report.finished_at_ms = Some(finished);
    collector.report.duration_s = Some(store.started.elapsed().as_secs_f64());
    collector.report.exit_code = exit_code;
    store.write(&mut collector, true);
    fs::write(dir.join("index.html"), html::render(&collector.report))?;
    print_summary(
        &collector.report,
        &dir,
        in_store.then_some(live_url.as_str()),
    );
    if in_store {
        prune(&root.join(RUNS_DIR), options.keep, &run_id);
    }
    Ok(match collector.report.state {
        RunState::Passed => 0,
        RunState::Interrupted => 130,
        _ => exit_code.filter(|&c| c != 0).unwrap_or(1),
    })
}

/// `cargo` arguments for the run (or the `--list` pass).
pub fn run_command(options: &Options, list: bool) -> Vec<String> {
    let mut args = vec!["test".to_string()];
    if !options.cargo_args.iter().any(|a| a == "--no-fail-fast") {
        args.push("--no-fail-fast".into());
    }
    args.extend(options.cargo_args.iter().cloned());
    if list || !options.test_args.is_empty() {
        args.push("--".into());
        args.extend(options.test_args.iter().cloned());
    }
    if list {
        args.push("--list".into());
    }
    args
}

struct Store {
    dir: PathBuf,
    run_id: String,
    started: Instant,
    last: Option<Instant>,
}

impl Store {
    /// Write `report.json` + `progress.json`; throttled unless `force`.
    fn write(&mut self, collector: &mut Collector, force: bool) {
        if !force && self.last.is_some_and(|t| t.elapsed() < WRITE_EVERY) {
            return;
        }
        self.last = Some(Instant::now());
        collector.report.recount();
        let report = &collector.report;
        let progress = Progress {
            run_id: self.run_id.clone(),
            kind: "test".into(),
            state: report.state,
            totals: report.totals.clone(),
            activity: collector.activity.clone(),
            updated_at_ms: now_ms(),
            finished_at_ms: report.finished_at_ms,
            duration_s: Some(
                report
                    .duration_s
                    .unwrap_or_else(|| self.started.elapsed().as_secs_f64()),
            ),
            pid: std::process::id(),
        };
        let force_note = |r: io::Result<()>| {
            if force && let Err(err) = r {
                eprintln!("airbug test: could not write run files: {err}");
            }
        };
        force_note(write_json(&self.dir.join("report.json"), report, false));
        force_note(write_json(&self.dir.join("progress.json"), &progress, true));
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize, pretty: bool) -> io::Result<()> {
    let bytes = if pretty {
        serde_json::to_vec_pretty(value)
    } else {
        serde_json::to_vec(value)
    }
    .map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    // Readers poll these files; a rename never exposes a half-written document.
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

/// Run cargo with stdout+stderr merged into one pipe (so `Running …` precedes its tests) and
/// feed every line to `on_line(Some(line))`; `on_line(None)` is a periodic tick.
fn stream(
    root: &Path,
    args: &[String],
    env_vars: &[(String, String)],
    on_line: &mut dyn FnMut(Option<&str>),
) -> io::Result<ExitStatus> {
    let (mut child, reader) = spawn(root, args, env_vars)?;
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    let line = String::from_utf8_lossy(&buf);
                    let line = line.trim_end_matches(['\n', '\r']).to_string();
                    if tx.send(line).is_err() {
                        break;
                    }
                }
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });
    let mut exited: Option<(ExitStatus, Instant)> = None;
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => on_line(Some(line.as_str())),
            Err(mpsc::RecvTimeoutError::Timeout) => on_line(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if exited.is_none()
            && let Some(status) = child.try_wait()?
        {
            exited = Some((status, Instant::now()));
        }
        if let Some((_, at)) = exited
            && at.elapsed() > DRAIN_AFTER_EXIT
        {
            break;
        }
    }
    match exited {
        Some((status, _)) => Ok(status),
        None => child.wait(),
    }
}

fn spawn(
    root: &Path,
    args: &[String],
    env_vars: &[(String, String)],
) -> io::Result<(Child, io::PipeReader)> {
    let (reader, writer) = io::pipe()?;
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command
        .current_dir(root)
        .args(args)
        .stdin(Stdio::null())
        .stdout(writer.try_clone()?)
        .stderr(writer);
    for (key, value) in env_vars {
        command.env(key, value);
    }
    // `command` (holding the write ends) drops on return, so the reader sees EOF once
    // cargo and its children exit.
    let child = command.spawn()?;
    Ok((child, reader))
}

/// Lines worth echoing during the listing pass (build progress, errors — not test names).
fn is_build_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    [
        "Compiling",
        "Finished",
        "error",
        "warning",
        "Blocking",
        "Updating",
    ]
    .iter()
    .any(|p| trimmed.starts_with(p))
}

fn tail_of(path: &Path, lines: usize) -> String {
    let text = fs::read_to_string(path).unwrap_or_default();
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

fn git_info(root: &Path) -> Git {
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (out.status.success() && !text.is_empty()).then_some(text)
    };
    let commit = git(&["rev-parse", "HEAD"]);
    Git {
        short: commit.as_ref().map(|c| c.chars().take(8).collect()),
        commit,
        branch: git(&["rev-parse", "--abbrev-ref", "HEAD"]),
        subject: git(&["log", "-1", "--format=%s"]),
        dirty: git(&["status", "--porcelain", "--untracked-files=no"]).is_some(),
    }
}

enum HubProbe {
    Ours,
    Other(String),
    Absent,
}

/// Is a hub for this root listening on `port`?
fn probe_hub(port: u16, root: &Path) -> HubProbe {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(300)) else {
        return HubProbe::Absent;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let request = "GET /api/v1/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request.as_bytes()).is_err() {
        return HubProbe::Absent;
    }
    let mut text = String::new();
    if stream.read_to_string(&mut text).is_err() {
        return HubProbe::Absent;
    }
    let body = text.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");
    let hub_root = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("root").and_then(|r| r.as_str()).map(str::to_string));
    match hub_root {
        Some(hub_root) if Path::new(&hub_root) == root => HubProbe::Ours,
        Some(hub_root) => HubProbe::Other(hub_root),
        None => HubProbe::Absent,
    }
}

fn print_summary(report: &Report, dir: &Path, live_url: Option<&str>) {
    let t = &report.totals;
    let crashed = report
        .suites
        .iter()
        .filter(|s| s.state == SuiteState::Crashed)
        .count();
    let state = serde_json::to_value(report.state)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    let mut line = format!(
        "airbug test: {state} — {} passed, {} failed, {} ignored, {} not run",
        t.passed, t.failed, t.ignored, t.not_run
    );
    if crashed > 0 {
        line.push_str(&format!(", {crashed} crashed suite(s)"));
    }
    if let Some(duration) = report.duration_s {
        line.push_str(&format!(" in {duration:.1}s"));
    }
    eprintln!("{line}");
    for test in report
        .tests
        .iter()
        .filter(|t| t.status == TestStatus::Failed)
        .take(20)
    {
        eprintln!("  FAILED {} › {}", test.suite, test.name);
    }
    for suite in report
        .suites
        .iter()
        .filter(|s| s.state == SuiteState::Crashed)
    {
        eprintln!("  CRASHED {}", suite.id);
    }
    eprintln!("airbug test: report {}", dir.join("index.html").display());
    if let Some(url) = live_url {
        eprintln!("airbug test: hub {url}");
    }
}

/// Keep the newest `keep` finished test runs in the store; never touch `current` or runs that
/// are still going.
fn prune(runs_dir: &Path, keep: usize, current: &str) {
    let Ok(entries) = fs::read_dir(runs_dir) else {
        return;
    };
    let mut finished: Vec<(u64, PathBuf)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()) != Some(current))
        .filter_map(|path| {
            let manifest: Manifest =
                serde_json::from_slice(&fs::read(path.join("manifest.json")).ok()?).ok()?;
            if manifest.schema != MANIFEST_SCHEMA || manifest.kind != "test" {
                return None;
            }
            let progress: Progress =
                serde_json::from_slice(&fs::read(path.join("progress.json")).ok()?).ok()?;
            progress
                .state
                .is_finished()
                .then_some((manifest.started_at_ms, path))
        })
        .collect();
    finished.sort_by_key(|a| std::cmp::Reverse(a.0));
    // `current` is one of the kept runs.
    for (_, path) in finished.into_iter().skip(keep.saturating_sub(1)) {
        let _ = fs::remove_dir_all(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_split_into_cargo_and_test_binary_parts() {
        let options = parse_args([
            "--title",
            "ci",
            "--no-list",
            "--",
            "--workspace",
            "--exclude",
            "airbug-mon",
            "--",
            "--test-threads",
            "2",
        ])
        .unwrap();
        assert_eq!(options.title.as_deref(), Some("ci"));
        assert!(!options.list);
        assert_eq!(
            options.cargo_args,
            ["--workspace", "--exclude", "airbug-mon"]
        );
        assert_eq!(options.test_args, ["--test-threads", "2"]);
        assert_eq!(
            run_command(&options, false),
            [
                "test",
                "--no-fail-fast",
                "--workspace",
                "--exclude",
                "airbug-mon",
                "--",
                "--test-threads",
                "2"
            ]
        );
        assert_eq!(run_command(&options, true).last().unwrap(), "--list");
    }

    #[test]
    fn list_pass_always_separates_binary_args() {
        let options = parse_args(["--", "--no-fail-fast", "-p", "demo"]).unwrap();
        assert_eq!(
            run_command(&options, true),
            ["test", "--no-fail-fast", "-p", "demo", "--", "--list"]
        );
        assert_eq!(
            run_command(&options, false),
            ["test", "--no-fail-fast", "-p", "demo"]
        );
    }

    #[test]
    fn unknown_flags_are_rejected() {
        assert!(parse_args(["--bogus"]).is_err());
        assert!(parse_args(["--out"]).is_err());
    }

    #[test]
    fn prune_keeps_newest_finished_runs() {
        let dir = env::temp_dir().join(format!("airbug-prune-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (i, state) in ["passed", "failed", "running", "passed"].iter().enumerate() {
            let run = dir.join(format!("run-{i}"));
            fs::create_dir_all(&run).unwrap();
            let manifest = Manifest {
                schema: MANIFEST_SCHEMA.into(),
                kind: "test".into(),
                run_id: format!("run-{i}"),
                started_at_ms: i as u64,
                ..Manifest::default()
            };
            write_json(&run.join("manifest.json"), &manifest, true).unwrap();
            fs::write(
                run.join("progress.json"),
                format!(r#"{{"run_id":"run-{i}","kind":"test","state":"{state}","totals":{{"total":0,"completed":0,"passed":0,"failed":0,"ignored":0,"not_run":0}}}}"#),
            )
            .unwrap();
        }
        // keep 2 = current (run-3) + newest other finished (run-1); run-2 is still running.
        prune(&dir, 2, "run-3");
        assert!(dir.join("run-3").is_dir());
        assert!(dir.join("run-2").is_dir());
        assert!(dir.join("run-1").is_dir());
        assert!(!dir.join("run-0").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
