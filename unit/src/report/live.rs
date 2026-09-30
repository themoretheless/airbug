//! File-backed run history shared by the test reporter and benchmark harness.
//! Writes occur at case boundaries, outside benchmark measurements.
use super::quote;
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Case {
    pub suite: String,
    pub name: String,
    pub status: String,
    pub duration: Option<f64>,
    pub started_ms: Option<u128>,
    pub output: String,
    /// Benchmark batch average, in nanoseconds per operation.
    pub ns_per_op: Option<f64>,
}

/// One launch, stored independently so concurrent launches never overwrite each other.
pub struct Run {
    pub id: String,
    pub directory: PathBuf,
    pub cases: Vec<Case>,
    kind: String,
    title: String,
    state: String,
    created: u128,
    started: Instant,
    current: Option<(usize, Instant)>,
    message: String,
}

pub fn project_root() -> io::Result<PathBuf> {
    if let Some(root) = std::env::var_os("AIRBUG_DASHBOARD_ROOT") {
        return PathBuf::from(root).canonicalize();
    }
    let cwd = std::env::current_dir()?;
    let mut closest = None;
    for dir in cwd.ancestors() {
        if let Ok(manifest) = fs::read_to_string(dir.join("Cargo.toml")) {
            closest.get_or_insert_with(|| dir.to_path_buf());
            if manifest.lines().any(|line| line.trim() == "[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }
    }
    Ok(closest.unwrap_or(cwd))
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

impl Run {
    pub fn new(output: &Path, kind: &str, title: &str) -> io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let id = format!("{stamp}-{}", std::process::id());
        let directory = output.join("runs").join(&id);
        fs::create_dir_all(&directory)?;
        let run = Self {
            id,
            directory,
            cases: Vec::new(),
            kind: kind.into(),
            title: title.into(),
            state: "running".into(),
            created: now(),
            started: Instant::now(),
            current: None,
            message: "Preparing run".into(),
        };
        run.save()?;
        Ok(run)
    }

    pub fn add_case(&mut self, suite: &str, name: &str, ignored: bool) -> usize {
        let index = self.cases.len();
        self.cases.push(Case {
            suite: suite.into(),
            name: name.into(),
            status: if ignored { "ignored" } else { "queued" }.into(),
            duration: None,
            started_ms: None,
            output: String::new(),
            ns_per_op: None,
        });
        index
    }

    pub fn message(&mut self, text: &str) -> io::Result<()> {
        self.message = clip(text, 64 * 1024);
        self.save()
    }

    pub fn start_case(&mut self, index: usize) -> io::Result<PathBuf> {
        self.cases[index].status = "running".into();
        self.cases[index].started_ms = Some(now());
        self.current = Some((index, Instant::now()));
        let dir = self.directory.join(index.to_string());
        fs::create_dir_all(&dir)?;
        self.message = format!("Running {}", self.cases[index].name);
        self.save()?;
        Ok(dir)
    }

    pub fn finish_case(
        &mut self,
        index: usize,
        status: &str,
        output: &str,
        ns_per_op: Option<f64>,
    ) -> io::Result<()> {
        let case = &mut self.cases[index];
        case.status = status.into();
        case.duration = self
            .current
            .take()
            .filter(|(i, _)| *i == index)
            .map(|(_, t)| t.elapsed().as_secs_f64());
        case.output = clip(output, 32 * 1024);
        case.ns_per_op = ns_per_op.filter(|v| v.is_finite());
        self.save()
    }

    pub fn finish(&mut self, success: bool, message: &str) -> io::Result<()> {
        self.state = if success { "passed" } else { "failed" }.into();
        self.message = clip(message, 64 * 1024);
        if let Some((index, time)) = self.current.take() {
            self.cases[index].status = "broken".into();
            self.cases[index].duration = Some(time.elapsed().as_secs_f64());
            self.cases[index].output = clip(message, 32 * 1024);
        }
        for case in &mut self.cases {
            if case.status == "queued" {
                case.status = "cancelled".into();
            }
        }
        self.save()
    }

    pub fn save(&self) -> io::Result<()> {
        let tests = self.cases.iter().map(|c| format!(
            "{{\"suite\":{},\"name\":{},\"status\":{},\"duration\":{},\"output\":{},\"ns_per_op\":{},\"started_ms\":{}}}",
            quote(&c.suite), quote(&c.name), quote(&c.status),
            c.duration.map_or("null".into(), |n| n.to_string()), quote(&c.output),
            c.ns_per_op.map_or("null".into(), |n| n.to_string()),
            c.started_ms.map_or("null".into(), |n| n.to_string())
        )).collect::<Vec<_>>().join(",");
        let json = format!(
            "{{\"version\":1,\"id\":{},\"kind\":{},\"title\":{},\"state\":{},\"pid\":{},\"created_ms\":{},\"updated_ms\":{},\"duration\":{},\"message\":{},\"tests\":[{}]}}",
            quote(&self.id),
            quote(&self.kind),
            quote(&self.title),
            quote(&self.state),
            std::process::id(),
            self.created,
            now(),
            self.started.elapsed().as_secs_f64(),
            quote(&self.message),
            tests
        );
        let temporary = self.directory.join("run.tmp");
        fs::write(&temporary, json)?;
        fs::rename(temporary, self.directory.join("run.json"))
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        if self.state == "running" {
            let _ = self.finish(
                false,
                if std::thread::panicking() {
                    "Runner panicked"
                } else {
                    "Run interrupted before completion"
                },
            );
        }
    }
}

fn clip(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.into();
    }
    let mut end = max;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[output truncated]", &value[..end])
}
