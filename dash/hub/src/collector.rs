//! Start/stop the local OpenTelemetry Collector.
//!
//! Order in `auto` mode: Docker Compose (collector + Jaeger) → `otelcol-contrib` / `otelcol`
//! binary → built-in Rust OTLP HTTP (`127.0.0.1:4318`) + Tonic gRPC (`127.0.0.1:4317`) receiver.
use crate::{app::HubApp, config::RootPaths, otlp::receiver::BuiltinCollector};
use std::{
    fs, io,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

pub const DEFAULT_OTLP_HTTP_PORT: u16 = 4318;
pub const DEFAULT_OTLP_GRPC_PORT: u16 = 4317;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorMode {
    Auto,
    Builtin,
    Docker,
    Binary,
}

impl CollectorMode {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "builtin" | "native" => Some(Self::Builtin),
            "docker" | "compose" => Some(Self::Docker),
            "binary" | "otelcol" => Some(Self::Binary),
            _ => None,
        }
    }
}

pub struct CollectorHandle {
    /// Present when Docker Compose backend is used (Ctrl+C / stop → compose down).
    pub compose_file: Option<PathBuf>,
    child: Option<Child>,
    pid_file: Option<PathBuf>,
    builtin: Option<BuiltinCollector>,
}

impl CollectorHandle {
    /// Start OTLP collector according to `mode`.
    ///
    /// `Auto` prefers Docker Compose (+ Jaeger), then `otelcol-contrib`/`otelcol` on `PATH`,
    /// and finally falls back to the built-in Rust OTLP HTTP + gRPC receiver so `--collector`
    /// always brings up `:4318` and `:4317` on every machine.
    pub fn start(
        app: Arc<HubApp>,
        mode: CollectorMode,
        http_port: u16,
        grpc_port: u16,
    ) -> io::Result<Option<Self>> {
        let paths = &app.paths;
        fs::create_dir_all(paths.collector_data_dir())?;

        match mode {
            CollectorMode::Auto => {
                if http_port == DEFAULT_OTLP_HTTP_PORT && grpc_port == DEFAULT_OTLP_GRPC_PORT {
                    if let Some(handle) = try_docker(paths)? {
                        return Ok(Some(handle));
                    }
                    if let Some(handle) = try_binary(paths)? {
                        return Ok(Some(handle));
                    }
                    if port_open(http_port) && port_open(grpc_port) {
                        eprintln!(
                            "collector: reusing existing OTLP listener on http://127.0.0.1:{http_port}  grpc://127.0.0.1:{grpc_port}"
                        );
                        return Ok(None);
                    }
                }
                start_builtin(app, http_port, grpc_port).map(Some)
            }
            CollectorMode::Builtin => start_builtin(app, http_port, grpc_port).map(Some),
            CollectorMode::Docker => try_docker(paths)?.ok_or_else(|| {
                io::Error::other(
                    "Docker Compose collector unavailable (check `docker compose version` and dash/collector/docker-compose.yml)",
                )
            }).map(Some),
            CollectorMode::Binary => try_binary(paths)?.ok_or_else(|| {
                io::Error::other(
                    "otelcol-contrib / otelcol binary unavailable on PATH (or use --collector-mode builtin)",
                )
            }).map(Some),
        }
    }

    pub fn stop(mut self) {
        eprintln!("collector: stopping …");
        if let Some(mut builtin) = self.builtin.take() {
            builtin.stop();
        }
        if let Some(compose) = &self.compose_file {
            let _ = compose_cmd(compose, &["down"]);
        }
        if let Some(mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(pid_file) = &self.pid_file {
            if let Ok(pid_s) = fs::read_to_string(pid_file)
                && let Ok(pid) = pid_s.trim().parse::<i32>()
            {
                let _ = Command::new("kill").arg(pid.to_string()).status();
            }
            let _ = fs::remove_file(pid_file);
        }
    }
}

fn start_builtin(app: Arc<HubApp>, http_port: u16, grpc_port: u16) -> io::Result<CollectorHandle> {
    let builtin = BuiltinCollector::start(app, http_port, grpc_port)?;
    let http_addr = builtin.http_addr();
    let grpc_addr = builtin.grpc_addr();
    eprintln!(
        "collector: built-in Rust OTLP receiver listening on http://{http_addr}  grpc://{grpc_addr} (traces/metrics/logs/profiles; no external Docker/otelcol required)"
    );
    eprintln!("  export OTEL_EXPORTER_OTLP_ENDPOINT=http://{http_addr}");
    Ok(CollectorHandle {
        compose_file: None,
        child: None,
        pid_file: None,
        builtin: Some(builtin),
    })
}

fn try_docker(paths: &RootPaths) -> io::Result<Option<CollectorHandle>> {
    let compose_file = paths.compose_file();
    if !compose_file.is_file() {
        return Ok(None);
    }
    if !command_ok("docker", &["version"]) {
        eprintln!("collector: Docker not available — trying otelcol binary / built-in …");
        return Ok(None);
    }
    let Some((program, lead)) = compose_program() else {
        eprintln!(
            "collector: Docker is up but neither `docker compose` nor `docker-compose` works — \
             trying otelcol binary / built-in …"
        );
        return Ok(None);
    };

    let front_end = if lead.is_empty() {
        program.to_string()
    } else {
        format!("{program} {}", lead.join(" "))
    };
    eprintln!(
        "collector: starting Docker stack ({front_end} -f {}) …",
        compose_file.display()
    );
    let status = compose_cmd(&compose_file, &["up", "-d", "--remove-orphans"])?;
    if !status.success() {
        eprintln!("collector: docker compose up failed — trying otelcol binary / built-in …");
        return Ok(None);
    }

    wait_port(4318, Duration::from_secs(45));
    if probe().otlp_http {
        eprintln!("collector: OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317");
        eprintln!("collector: Jaeger UI http://127.0.0.1:16686/");
        eprintln!("  export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318");
    } else {
        eprintln!(
            "collector: Docker stack is up but :4318 never opened — logs and metrics panels \
             stay empty; check `docker-compose -f {} logs otel-collector`",
            compose_file.display()
        );
    }

    Ok(Some(CollectorHandle {
        compose_file: Some(compose_file),
        child: None,
        pid_file: None,
        builtin: None,
    }))
}

fn render_standalone_config(template: &str, paths: &RootPaths) -> String {
    let logs_yaml = format!(
        "\"{}\"",
        paths.logs_file().display().to_string().replace('\\', "/")
    );
    let metrics_yaml = format!(
        "\"{}\"",
        paths
            .metrics_file()
            .display()
            .to_string()
            .replace('\\', "/")
    );
    let traces_yaml = format!(
        "\"{}\"",
        paths.traces_file().display().to_string().replace('\\', "/")
    );
    template
        .replace("__LOGS_PATH__", &logs_yaml)
        .replace("__METRICS_PATH__", &metrics_yaml)
        .replace("__TRACES_PATH__", &traces_yaml)
}

fn try_binary(paths: &RootPaths) -> io::Result<Option<CollectorHandle>> {
    let Some(bin) = find_otelcol() else {
        return Ok(None);
    };
    let template = paths.standalone_template();
    if !template.is_file() {
        eprintln!(
            "collector: missing {} — cannot start binary mode",
            template.display()
        );
        return Ok(None);
    }

    let runtime_cfg = paths.runtime_config();
    let rendered = render_standalone_config(&fs::read_to_string(&template)?, paths);
    fs::write(&runtime_cfg, rendered)?;

    eprintln!(
        "collector: starting {} ({}) …",
        bin.display(),
        runtime_cfg.display()
    );
    let mut child = Command::new(&bin)
        .arg("--config")
        .arg(&runtime_cfg)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    let pid_file = paths.otelcol_pid();
    fs::write(&pid_file, child.id().to_string())?;

    wait_port(4318, Duration::from_secs(20));
    if !probe().otlp_http {
        let _ = child.kill();
        let _ = fs::remove_file(&pid_file);
        return Err(io::Error::other(format!(
            "{} started but :4318 never opened — check config {}",
            bin.display(),
            runtime_cfg.display()
        )));
    }

    eprintln!(
        "collector: OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317 (traces in hub; no Jaeger in binary mode)"
    );
    eprintln!("  export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318");

    Ok(Some(CollectorHandle {
        compose_file: None,
        child: Some(child),
        pid_file: Some(pid_file),
        builtin: None,
    }))
}

fn find_otelcol() -> Option<PathBuf> {
    for name in ["otelcol-contrib", "otelcol"] {
        if command_ok(name, &["--version"]) || command_ok(name, &["version"]) {
            return Some(PathBuf::from(name));
        }
        if which(name).is_some() {
            return Some(PathBuf::from(name));
        }
    }
    None
}

fn which(name: &str) -> Option<PathBuf> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn command_ok(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// `(program, leading args)` for whichever Compose front-end this host has.
fn compose_program() -> Option<(&'static str, &'static [&'static str])> {
    if command_ok("docker", &["compose", "version"]) {
        return Some(("docker", &["compose"]));
    }
    if command_ok("docker-compose", &["version"]) {
        return Some(("docker-compose", &[]));
    }
    None
}

fn compose_cmd(file: &Path, args: &[&str]) -> io::Result<std::process::ExitStatus> {
    let (program, lead) = compose_program().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "no Compose front-end: install the docker compose plugin or docker-compose",
        )
    })?;
    let mut command = Command::new(program);
    command.args(lead).arg("-f").arg(file).args(args);
    command.status()
}

fn wait_port(port: u16, budget: Duration) {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if port_open(port) {
            return;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    eprintln!("collector: port {port} not open yet");
}

/// Probe localhost OTLP / Jaeger for the hub status card.
pub fn probe() -> CollectorProbe {
    CollectorProbe {
        otlp_http: port_open(DEFAULT_OTLP_HTTP_PORT),
        otlp_grpc: port_open(DEFAULT_OTLP_GRPC_PORT),
        jaeger_ui: port_open(16686),
    }
}

/// Whether a loopback port accepts, giving up after [`PROBE_BUDGET`].
fn port_open(port: u16) -> bool {
    port_open_at(SocketAddr::from(([127, 0, 0, 1], port)), PROBE_BUDGET)
}

fn port_open_at(addr: SocketAddr, budget: Duration) -> bool {
    TcpStream::connect_timeout(&addr, budget).is_ok()
}

/// How long a port may take to answer before the probe calls it down.
///
/// A port the host reserves for its own virtualisation is silently dropped rather than refused,
/// so an untimed connect waits out the whole TCP timeout — long enough to stall the status route
/// this feeds, which the dashboard polls every few seconds.
const PROBE_BUDGET: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy)]
pub struct CollectorProbe {
    pub otlp_http: bool,
    pub otlp_grpc: bool,
    pub jaeger_ui: bool,
}

impl CollectorProbe {
    pub fn any_up(self) -> bool {
        self.otlp_http || self.otlp_grpc || self.jaeger_ui
    }
}

/// Best-effort stop from a Ctrl+C handler (Docker and/or pid file).
pub fn emergency_stop(root: &Path) {
    let paths = RootPaths::new(root);
    let compose = paths.compose_file();
    if compose.is_file() && compose_program().is_some() {
        let _ = compose_cmd(&compose, &["down"]);
    }
    let pid_file = paths.otelcol_pid();
    if let Ok(pid_s) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_s.trim().parse::<i32>() {
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
        let _ = fs::remove_file(&pid_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dropped_syn_is_down_rather_than_hanging() {
        let black_hole = SocketAddr::from(([192, 0, 2, 1], 4318));
        let started = Instant::now();
        assert!(!port_open_at(black_hole, PROBE_BUDGET));
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn an_open_port_is_reported_up() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        assert!(port_open(port));
    }

    #[test]
    fn standalone_template_renders_traces_metrics_and_logs_paths() {
        let paths = RootPaths::new("/tmp/airbug-ws");
        let tpl = "logs: __LOGS_PATH__\nmetrics: __METRICS_PATH__\ntraces: __TRACES_PATH__\n";
        let out = render_standalone_config(tpl, &paths);
        assert!(out.contains("dash/collector/data/logs.json"));
        assert!(out.contains("dash/collector/data/metrics.json"));
        assert!(out.contains("dash/collector/data/traces.json"));
    }

    #[test]
    fn collector_mode_parses_all_variants() {
        assert_eq!(CollectorMode::parse("auto"), Some(CollectorMode::Auto));
        assert_eq!(
            CollectorMode::parse("builtin"),
            Some(CollectorMode::Builtin)
        );
        assert_eq!(CollectorMode::parse("docker"), Some(CollectorMode::Docker));
        assert_eq!(CollectorMode::parse("binary"), Some(CollectorMode::Binary));
        assert_eq!(CollectorMode::parse("bogus"), None);
    }
}
