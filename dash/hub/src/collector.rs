//! Start/stop the local OpenTelemetry Collector.
//!
//! Order: Docker Compose (collector + Jaeger) → `otelcol-contrib` / `otelcol` binary.
use std::{
    fs, io,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const COMPOSE_REL: &str = "dash/collector/docker-compose.yml";
const STANDALONE_TEMPLATE: &str = "dash/collector/config.standalone.yaml";

pub struct CollectorHandle {
    /// Present when Docker Compose backend is used (Ctrl+C / stop → compose down).
    pub compose_file: Option<PathBuf>,
    child: Option<Child>,
    pid_file: Option<PathBuf>,
}

impl CollectorHandle {
    /// Prefer Docker stack; otherwise spawn a local otelcol binary.
    pub fn start(root: &Path) -> io::Result<Option<Self>> {
        let data_dir = root.join("dash/collector/data");
        fs::create_dir_all(&data_dir)?;

        if let Some(handle) = try_docker(root)? {
            return Ok(Some(handle));
        }
        if let Some(handle) = try_binary(root, &data_dir)? {
            return Ok(Some(handle));
        }

        eprintln!("collector: neither Docker nor otelcol found");
        eprintln!("  Docker:  docker compose -f {COMPOSE_REL} up -d");
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

fn try_docker(root: &Path) -> io::Result<Option<CollectorHandle>> {
    let compose_file = root.join(COMPOSE_REL);
    if !compose_file.is_file() {
        return Ok(None);
    }
    if !command_ok("docker", &["version"]) {
        eprintln!("collector: Docker not available — trying otelcol binary …");
        return Ok(None);
    }

    eprintln!("collector: starting Docker stack ({COMPOSE_REL}) …");
    let status = compose_cmd(&compose_file, &["up", "-d", "--remove-orphans"])?;
    if !status.success() {
        eprintln!("collector: docker compose up failed — trying otelcol binary …");
        return Ok(None);
    }

    wait_port(4318, Duration::from_secs(45));
    eprintln!("collector: OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317");
    eprintln!("collector: Jaeger UI http://127.0.0.1:16686/");
    eprintln!("  export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318");

    Ok(Some(CollectorHandle {
        compose_file: Some(compose_file),
        child: None,
        pid_file: None,
    }))
}

fn try_binary(root: &Path, data_dir: &Path) -> io::Result<Option<CollectorHandle>> {
    let Some(bin) = find_otelcol() else {
        return Ok(None);
    };
    let template = root.join(STANDALONE_TEMPLATE);
    if !template.is_file() {
        eprintln!(
            "collector: missing {} — cannot start binary mode",
            template.display()
        );
        return Ok(None);
    }

    let logs_path = data_dir.join("logs.json");
    let metrics_path = data_dir.join("metrics.json");
    let runtime_cfg = data_dir.join("runtime-config.yaml");
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

    let pid_file = data_dir.join("otelcol.pid");
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
        // Some builds only print version to stdout with -v / no args — try which via `command -v` style.
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

fn compose_cmd(file: &Path, args: &[&str]) -> io::Result<std::process::ExitStatus> {
    Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(file)
        .args(args)
        .status()
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
    let compose = root.join(COMPOSE_REL);
    if compose.is_file() && command_ok("docker", &["version"]) {
        let _ = compose_cmd(&compose, &["down"]);
    }
    let pid_file = root.join("dash/collector/data/otelcol.pid");
    if let Ok(pid_s) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_s.trim().parse::<i32>() {
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
        let _ = fs::remove_file(&pid_file);
    }
}
