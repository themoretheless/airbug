use crate::experiment_report::{self, Options};
use airbug_bench::{Result, error};
use sha2::{Digest, Sha256};
use std::{
    collections::hash_map::RandomState,
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
    if root.join("progress.json").is_file() && !root.join("status-final.json").is_file() {
        return Err(error(
            "Benchmark is running; report becomes available after completion",
        ));
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
            let state = if root.join("status-final.json").is_file() {
                std::fs::read_to_string(root.join("status-final.json"))?
            } else if root.join("progress.json").is_file() {
                std::fs::read_to_string(root.join("progress.json"))?
            } else {
                serde_json::json!({"state": if root.is_dir() {"idle"} else {"preparing"}})
                    .to_string()
            };
            let value: serde_json::Value =
                serde_json::from_str(&state).unwrap_or(serde_json::Value::Null);
            let text = |key: &str| {
                value
                    .get(key)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string()
            };
            let number = |key: &str| value.get(key).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            live.lock().expect("live state lock").observe(
                &text("state"),
                &text("variant"),
                number("completed"),
                number("total"),
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
