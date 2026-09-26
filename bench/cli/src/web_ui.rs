use crate::experiment_report::{self, Options};
use airbug_bench::{Result, error};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, hash_map::RandomState},
    hash::BuildHasher,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    time::{Duration, SystemTime},
};

fn response(stream: &mut TcpStream, status: &str, mime: &str, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nX-Frame-Options: SAMEORIGIN\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
fn identity(label: &str) -> String {
    airbug_bench::model::hex(&Sha256::digest(label.as_bytes()))
}

/// One observation of the running experiment, taken when the interface polled the server.
#[derive(Clone, Debug)]
struct Sample {
    at: f64,
    state: String,
    variant: String,
    completed: usize,
    total: usize,
}

/// What the browser asks the server to draw in live mode: elapsed seconds per variant lane
/// and the completed-process trend. Sampling belongs to the server because only the server
/// sees every poll, and adding timings to `progress.json` would touch the measured path.
#[derive(Default)]
pub struct Live {
    started: Option<std::time::Instant>,
    samples: Vec<Sample>,
}

impl Live {
    fn observe(&mut self, state: &str, variant: &str, completed: usize, total: usize) {
        let started = *self.started.get_or_insert_with(std::time::Instant::now);
        let at = started.elapsed().as_secs_f64();
        let last = self.samples.last();
        let changed = match last {
            None => true,
            Some(s) => {
                s.state != state
                    || s.variant != variant
                    || s.completed != completed
                    || s.total != total
            }
        };
        if changed {
            self.samples.push(Sample {
                at,
                state: state.to_string(),
                variant: variant.to_string(),
                completed,
                total,
            });
        }
        if self.samples.len() > 4000 {
            self.samples.drain(0..self.samples.len() - 4000);
        }
    }

    /// Lane-and-trend figures over what was observed, or an empty string when nothing ran.
    fn figures(&self) -> String {
        use airbug_bench::viz::charts::{self, Segment, TimelineLane};
        if self.samples.is_empty() {
            return String::new();
        }
        let running = self
            .samples
            .iter()
            .filter(|s| matches!(s.state.as_str(), "running" | "preparing"))
            .count();
        if running == 0 {
            return String::new();
        }
        let last = self.samples.last().unwrap();
        let end = last.at + 1.5;
        let mut names: Vec<String> = vec![];
        for s in &self.samples {
            if !s.variant.is_empty() && !names.contains(&s.variant) {
                names.push(s.variant.clone());
            }
        }
        let mut lanes = vec![];
        for name in &names {
            let mut segments = vec![];
            let mut open: Option<(String, f64)> = None;
            for w in self.samples.windows(2) {
                let (from, to) = (&w[0], &w[1]);
                if from.variant == *name && open.is_none() {
                    open = Some((name.clone(), from.at));
                }
                if open.as_ref().is_some_and(|(v, _)| *v == *name) && to.variant != *name {
                    if let Some((_, start)) = open.take() {
                        segments.push(Segment {
                            start,
                            stop: to.at.max(start + 0.2),
                            detail: format!("{} of {} done", from.completed, from.total),
                        });
                    }
                }
            }
            if let Some((_, start)) = open.take() {
                segments.push(Segment {
                    start,
                    stop: end.max(start + 0.2),
                    detail: "in progress".into(),
                });
            }
            lanes.push(TimelineLane {
                label: name.clone(),
                segments,
            });
        }
        let completed: Vec<f64> = self.samples.iter().map(|s| s.completed as f64).collect();
        let notes = format!(
            "Sampled while the interface is open (about every 1.5 s), so lane edges are approximate. {} of {} processes finished.",
            last.completed, last.total
        );
        let mut out = charts::timeline("Processes by variant", &lanes, &notes);
        out.push_str(&charts::sparkline("Completed processes", &completed, None));
        out
    }
}

type Shared = std::sync::Arc<std::sync::Mutex<Live>>;

/// Parse a status file, treating a missing or malformed one as absent.
fn read_json(path: &Path) -> Result<serde_json::Value> {
    if !path.is_file() {
        return Ok(serde_json::Value::Null);
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?).unwrap_or(serde_json::Value::Null))
}

fn text(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn number(value: &serde_json::Value, key: &str) -> usize {
    value.get(key).and_then(|v| v.as_u64()).unwrap_or(0) as usize
}

/// Live status of the served directory: the run it holds, or the aggregate of a
/// matrix session.
fn live_state(root: &Path) -> Result<String> {
    if let Some(aggregate) = matrix_state(root)? {
        return Ok(aggregate);
    }
    if root.join("status-final.json").is_file() {
        return Ok(std::fs::read_to_string(root.join("status-final.json"))?);
    }
    if root.join("progress.json").is_file() {
        return Ok(std::fs::read_to_string(root.join("progress.json"))?);
    }
    Ok(serde_json::json!({"state": if root.is_dir() {"idle"} else {"preparing"}}).to_string())
}

/// One cell of a matrix session as the interface sees it.
struct Cell {
    label: String,
    progress: serde_json::Value,
    /// `state` of the cell's `status-final.json`, absent while it is unfinished.
    finished: Option<String>,
}

/// Aggregate progress of a matrix session: `matrix.json` names the combinations
/// and `<root>/<index>/` holds one run each. Cell runners already write their own
/// progress, so the interface reads what is on disk instead of the coordinator
/// publishing a second progress file beside the measured path.
///
/// Cells share one plan and differ only in worker arguments, so the process count
/// per cell is constant and `cells × that count` is the session total.
fn matrix_state(root: &Path) -> Result<Option<String>> {
    let manifest = root.join("matrix.json");
    if !manifest.is_file() {
        return Ok(None);
    }
    let combinations: Vec<BTreeMap<String, String>> =
        serde_json::from_str(&std::fs::read_to_string(&manifest)?)?;
    let cells = combinations
        .iter()
        .enumerate()
        .map(|(index, combination)| -> Result<Cell> {
            let cell = root.join(index.to_string());
            let final_state = text(&read_json(&cell.join("status-final.json"))?, "state");
            Ok(Cell {
                label: combination
                    .iter()
                    .map(|(flag, value)| format!("{flag} {value}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                progress: read_json(&cell.join("progress.json"))?,
                finished: (!final_state.is_empty()).then_some(final_state),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let per_cell = cells
        .iter()
        .map(|cell| number(&cell.progress, "total"))
        .max()
        .unwrap_or(0);
    let mut completed = 0;
    let mut running = None;
    let mut finished = 0;
    let mut interrupted = None;
    for cell in &cells {
        completed += number(&cell.progress, "completed");
        match &cell.finished {
            Some(state) => {
                finished += 1;
                if state != "complete" {
                    interrupted = Some(state.clone());
                }
            }
            None if cell.progress.is_object() && running.is_none() => running = Some(cell),
            None => (),
        }
    }
    let state = match &interrupted {
        Some(state) => state.clone(),
        None if running.is_some() => "running".to_string(),
        None if finished == cells.len() => "complete".to_string(),
        None => "preparing".to_string(),
    };
    let variant = running.map_or(String::new(), |cell| {
        let inner = text(&cell.progress, "variant");
        if inner.is_empty() {
            cell.label.clone()
        } else {
            format!("{} · {}", cell.label, inner)
        }
    });
    Ok(Some(
        serde_json::json!({
            "state": state,
            "completed": completed,
            "total": per_cell * cells.len(),
            "variant": variant,
        })
        .to_string(),
    ))
}

fn render(root: &Path, store: &Path, query: &str) -> Result<(String, String)> {
    let mut id = None;
    let mut baseline = None;
    let mut format = "html";
    let mut threshold = 5.0;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        let (key, value) = pair.split_once('=').ok_or_else(|| error("invalid query"))?;
        match key {
            "id" => id = Some(value),
            "baseline" if !value.is_empty() => baseline = Some(value),
            "baseline" => (),
            "format" => format = value,
            "threshold" => threshold = value.parse::<f64>()?,
            _ => return Err(error("unknown parameter")),
        }
    }
    if root.join("matrix.json").is_file() || root.join("progress.json").is_file() {
        let state = live_state(root)?;
        let value: serde_json::Value =
            serde_json::from_str(&state).unwrap_or(serde_json::Value::Null);
        if matches!(text(&value, "state").as_str(), "running" | "preparing") {
            return Err(error(
                "Benchmark is running; report becomes available after completion",
            ));
        }
    }
    let runs = experiment_report::files(root)?;
    let source = match id {
        Some(i) => runs
            .iter()
            .find(|r| identity(&r.0) == i)
            .ok_or_else(|| error("unknown run"))?
            .1
            .parent()
            .unwrap(),
        None => root,
    };
    let base = baseline
        .map(|i| {
            runs.iter()
                .find(|r| identity(&r.0) == i)
                .map(|r| r.1.parent().unwrap())
                .ok_or_else(|| error("unknown baseline"))
        })
        .transpose()?;
    let doc = experiment_report::build(Options {
        source,
        baseline: base,
        store,
        title: "Benchmark report",
        threshold,
        alpha: 0.05,
    })?;
    match format {
        "html" => Ok(("text/html".into(), doc.html()?)),
        "json" => Ok((
            "application/json".into(),
            serde_json::to_string_pretty(&doc)?,
        )),
        "md" => Ok(("text/markdown".into(), doc.markdown()?)),
        _ => Err(error("unknown format")),
    }
}
fn route(root: &Path, store: &Path, live: &Shared, path: &str) -> Result<(String, String)> {
    match path {
        "" | "index.html" => Ok(("text/html".into(), include_str!("web-ui.html").into())),
        "api/live" | "api/live-charts" => {
            let state = live_state(root)?;
            let value: serde_json::Value =
                serde_json::from_str(&state).unwrap_or(serde_json::Value::Null);
            live.lock().expect("live state lock").observe(
                &text(&value, "state"),
                &text(&value, "variant"),
                number(&value, "completed"),
                number(&value, "total"),
            );
            if path == "api/live" {
                return Ok(("application/json".into(), state));
            }
            Ok((
                "text/html".into(),
                live.lock().expect("live state lock").figures(),
            ))
        }
        "api/runs" => {
            let rows: Vec<_>=experiment_report::files(root)?.iter().map(|(label,path)|serde_json::json!({"id":identity(label),"label":label,"available":path.is_file(),"memory":path.parent().unwrap().join("memory.json").is_file()})).collect();
            Ok(("application/json".into(), serde_json::to_string(&rows)?))
        }
        _ if path.starts_with("memory?") => {
            let id = path
                .strip_prefix("memory?id=")
                .ok_or_else(|| error("memory requires a run id"))?;
            let runs = experiment_report::files(root)?;
            let source = &runs
                .iter()
                .find(|r| identity(&r.0) == id)
                .ok_or_else(|| error("unknown run"))?
                .1;
            let profile =
                airbug_bench::memory::Profile::load(&source.parent().unwrap().join("memory.json"))?;
            Ok(("text/html".into(), profile.html()?))
        }
        _ if path.starts_with("report?") => render(root, store, &path[7..]),
        _ => Err(error("unknown route")),
    }
}
pub fn serve(root: &Path, store: &Path, port: u16) -> Result<()> {
    // No worker processes need graceful artifact finalization in this read-only server.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
    }
    let root = root.canonicalize()?;
    if !root.is_dir() {
        return Err(error("serve requires an experiment directory"));
    }
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    listen(listener, &root, store, &shared())
}
fn shared() -> Shared {
    std::sync::Arc::new(std::sync::Mutex::new(Live::default()))
}
pub fn start_live(root: &Path, store: &Path) -> Result<String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let token = token();
    let url = format!("http://{}/{token}/", listener.local_addr()?);
    let root = root.to_path_buf();
    let store = store.to_path_buf();
    let live = shared();
    std::thread::spawn(move || {
        let _ = connections(listener, &root, &store, &token, &live);
    });
    Ok(url)
}
fn token() -> String {
    format!(
        "{:016x}{:016x}",
        RandomState::new().hash_one(SystemTime::now()),
        RandomState::new().hash_one(std::process::id())
    )
}
fn listen(listener: TcpListener, root: &Path, store: &Path, live: &Shared) -> Result<()> {
    let token = token();
    println!(
        "Report interface: http://{}/{token}/",
        listener.local_addr()?
    );
    connections(listener, root, store, &token, live)
}
fn connections(
    listener: TcpListener,
    root: &Path,
    store: &Path,
    token: &str,
    live: &Shared,
) -> Result<()> {
    let prefix = format!("/{token}/");
    for connection in listener.incoming() {
        let mut stream = connection?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        let mut request = Vec::new();
        let mut byte = [0; 1];
        while request.len() < 8192 && !request.ends_with(b"\r\n\r\n") {
            match stream.read(&mut byte) {
                Ok(1) => request.push(byte[0]),
                _ => break,
            }
        }
        let text = String::from_utf8_lossy(&request);
        let parts: Vec<_> = text
            .lines()
            .next()
            .unwrap_or("")
            .split_whitespace()
            .collect();
        if !request.ends_with(b"\r\n\r\n")
            || parts.len() != 3
            || parts[0] != "GET"
            || !parts[1].starts_with(&prefix)
        {
            let _ = response(&mut stream, "404 Not Found", "text/plain", "Not found");
            continue;
        }
        match route(root, store, live, &parts[1][prefix.len()..]) {
            Ok((mime, body)) => {
                let _ = response(&mut stream, "200 OK", &mime, &body);
            }
            Err(e) => {
                let _ = response(&mut stream, "400 Bad Request", "text/plain", &e.to_string());
            }
        }
    }
    Ok(())
}
pub fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).status();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .status();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let result = std::process::Command::new("xdg-open").arg(url).status();
    if !matches!(result, Ok(status) if status.success()) {
        eprintln!("Open the live interface manually: {url}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routes_reject_paths_and_invalid_selections() {
        let dir = tempfile::tempdir().unwrap();
        let live = shared();
        assert!(route(dir.path(), dir.path(), &live, "../../Cargo.toml").is_err());
        assert!(render(dir.path(), dir.path(), "id=../../secret").is_err());
        assert!(render(dir.path(), dir.path(), "path=/etc/passwd").is_err());
        let (_, page) = route(dir.path(), dir.path(), &live, "").unwrap();
        assert!(page.contains("iframe"));
    }

    #[test]
    fn live_charts_are_empty_before_any_run_and_html_after() {
        let dir = tempfile::tempdir().unwrap();
        let live = shared();
        let (mime, charts) = route(dir.path(), dir.path(), &live, "api/live-charts").unwrap();
        assert_eq!(mime, "text/html");
        assert!(!charts.contains("<svg"), "{charts}");
        std::fs::write(
            dir.path().join("progress.json"),
            r#"{"state":"running","completed":1,"total":4,"variant":"baseline"}"#,
        )
        .unwrap();
        let (_, first) = route(dir.path(), dir.path(), &live, "api/live").unwrap();
        assert!(first.contains("\"running\""));
        std::fs::write(
            dir.path().join("progress.json"),
            r#"{"state":"running","completed":2,"total":4,"variant":"candidate"}"#,
        )
        .unwrap();
        let (_, charts) = route(dir.path(), dir.path(), &live, "api/live-charts").unwrap();
        assert!(charts.contains("<svg"), "{charts}");
        assert!(charts.contains("baseline"));
        assert!(charts.contains("candidate"));
        assert!(charts.contains("of 4 done"));
    }

    /// A matrix session: one axis, and per cell its combination value, progress
    /// snapshot and final state.
    type Cell<'a> = (&'a str, Option<(&'a str, usize, usize)>, Option<&'a str>);

    fn matrix(dir: &Path, cells: &[Cell]) {
        let combinations: Vec<BTreeMap<String, String>> = cells
            .iter()
            .map(|(value, _, _)| BTreeMap::from([("--cpu".to_string(), value.to_string())]))
            .collect();
        std::fs::write(
            dir.join("matrix.json"),
            serde_json::to_vec(&combinations).unwrap(),
        )
        .unwrap();
        for (index, (_, progress, final_state)) in cells.iter().enumerate() {
            let cell = dir.join(index.to_string());
            std::fs::create_dir_all(&cell).unwrap();
            if let Some((variant, completed, total)) = progress {
                std::fs::write(
                    cell.join("progress.json"),
                    serde_json::json!({
                        "state": "running",
                        "completed": completed,
                        "total": total,
                        "variant": variant,
                    })
                    .to_string(),
                )
                .unwrap();
            }
            if let Some(state) = final_state {
                std::fs::write(
                    cell.join("status-final.json"),
                    serde_json::json!({"state": state}).to_string(),
                )
                .unwrap();
            }
        }
    }

    fn live_json(dir: &Path) -> serde_json::Value {
        let (_, body) = route(dir, dir, &shared(), "api/live").unwrap();
        serde_json::from_str(&body).unwrap()
    }

    #[test]
    fn matrix_root_aggregates_its_cells_into_one_progress() {
        let dir = tempfile::tempdir().unwrap();
        matrix(
            dir.path(),
            &[
                ("1", Some(("candidate", 4, 4)), Some("complete")),
                ("2", Some(("baseline", 2, 4)), None),
                ("4", None, None),
            ],
        );
        let state = live_json(dir.path());
        assert_eq!(state["state"], "running");
        // One shared plan means a constant process count per cell.
        assert_eq!(state["completed"], 6);
        assert_eq!(state["total"], 12);
        assert_eq!(state["variant"], "--cpu 2 · baseline");
    }

    #[test]
    fn matrix_aggregate_finishes_and_starts_empty() {
        let dir = tempfile::tempdir().unwrap();
        matrix(
            dir.path(),
            &[
                ("1", Some(("candidate", 4, 4)), Some("complete")),
                ("2", Some(("candidate", 4, 4)), Some("complete")),
                ("4", Some(("candidate", 4, 4)), Some("complete")),
            ],
        );
        let state = live_json(dir.path());
        assert_eq!(state["state"], "complete");
        assert_eq!(state["completed"], 12);
        assert_eq!(state["total"], 12);

        let fresh = tempfile::tempdir().unwrap();
        matrix(fresh.path(), &[("1", None, None), ("2", None, None)]);
        let state = live_json(fresh.path());
        assert_eq!(state["state"], "preparing");
        assert_eq!(state["total"], 0);
    }

    #[test]
    fn matrix_aggregate_surfaces_a_failed_cell() {
        let dir = tempfile::tempdir().unwrap();
        matrix(
            dir.path(),
            &[
                ("1", Some(("baseline", 4, 4)), Some("complete")),
                ("2", Some(("candidate", 1, 4)), Some("failed")),
            ],
        );
        let state = live_json(dir.path());
        assert_eq!(state["state"], "failed");
        assert_eq!(state["completed"], 5);
    }

    #[test]
    fn matrix_report_stays_closed_while_a_cell_runs() {
        let dir = tempfile::tempdir().unwrap();
        matrix(
            dir.path(),
            &[
                ("1", Some(("candidate", 4, 4)), Some("complete")),
                ("2", Some(("baseline", 2, 4)), None),
            ],
        );
        assert!(
            render(dir.path(), dir.path(), "").is_err(),
            "a running matrix must not serve a partial report"
        );
    }

    #[test]
    fn live_samples_merge_unchanged_polls() {
        let mut live = Live::default();
        live.observe("running", "baseline", 1, 3);
        for _ in 0..20 {
            live.observe("running", "baseline", 1, 3);
        }
        assert_eq!(live.samples.len(), 1);
        live.observe("running", "candidate", 2, 3);
        assert_eq!(live.samples.len(), 2);
    }
}
