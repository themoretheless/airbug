//! Start/stop the local OpenTelemetry Collector.
//!
//! Order: Docker Compose (collector + Jaeger) → `otelcol-contrib` / `otelcol` binary.
//! Compose is reached through `docker compose` when the CLI plugin is installed,
//! otherwise through a standalone `docker-compose` binary.
use crate::config::RootPaths;
use std::{
    fs, io,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct CollectorHandle {
    /// Present when Docker Compose backend is used (Ctrl+C / stop → compose down).
    pub compose_file: Option<PathBuf>,
    child: Option<Child>,
    pid_file: Option<PathBuf>,
}

impl CollectorHandle {
    /// Prefer Docker stack; otherwise spawn a local otelcol binary.
    pub fn start(root: &Path) -> io::Result<Option<Self>> {
        let paths = RootPaths::new(root);
        fs::create_dir_all(paths.collector_data_dir())?;

        if let Some(handle) = try_docker(&paths)? {
            return Ok(Some(handle));
        }
        if let Some(handle) = try_binary(&paths)? {
            return Ok(Some(handle));
        }

        eprintln!("collector: neither Docker nor otelcol found");
        eprintln!(
            "  Docker:  docker compose -f {} up -d   (plugin, or a standalone docker-compose)",
            paths.compose_file().display()
        );
        eprintln!("  Binary:  install otelcol-contrib (or otelcol) on PATH");
        eprintln!("           https://github.com/open-telemetry/opentelemetry-collector-releases");
        Ok(None)
    }

    pub fn stop(self) {
        eprintln!("collector: stopping …");
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

fn try_docker(paths: &RootPaths) -> io::Result<Option<CollectorHandle>> {
    let compose_file = paths.compose_file();
    if !compose_file.is_file() {
        return Ok(None);
    }
    if !command_ok("docker", &["version"]) {
        eprintln!("collector: Docker not available — trying otelcol binary …");
        return Ok(None);
    }
    let Some((program, lead)) = compose_program() else {
        eprintln!(
            "collector: Docker is up but neither `docker compose` nor `docker-compose` works — \
             trying otelcol binary …"
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
        eprintln!("collector: docker compose up failed — trying otelcol binary …");
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
    }))
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

    let data_dir = paths.collector_data_dir();
    let logs_path = paths.logs_file();
    let metrics_path = paths.metrics_file();
    let runtime_cfg = paths.runtime_config();
    let logs_yaml = format!("\"{}\"", logs_path.display().to_string().replace('\\', "/"));
    let metrics_yaml = format!(
        "\"{}\"",
        metrics_path.display().to_string().replace('\\', "/")
    );
    let rendered = fs::read_to_string(&template)?
        .replace("__LOGS_PATH__", &logs_yaml)
        .replace("__METRICS_PATH__", &metrics_yaml);
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
        "collector: OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317 (no Jaeger in binary mode)"
    );
    eprintln!("  export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318");
    let _ = data_dir;

    Ok(Some(CollectorHandle {
        compose_file: None,
        child: Some(child),
        pid_file: Some(pid_file),
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
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    eprintln!("collector: port {port} not open yet");
}

/// Probe localhost OTLP / Jaeger for the hub status card.
pub fn probe() -> CollectorProbe {
    CollectorProbe {
        otlp_http: TcpStream::connect(("127.0.0.1", 4318)).is_ok(),
        otlp_grpc: TcpStream::connect(("127.0.0.1", 4317)).is_ok(),
        jaeger_ui: TcpStream::connect(("127.0.0.1", 16686)).is_ok(),
    }
}

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
