//! Start/stop the local OpenTelemetry Collector (+ Jaeger UI) via Docker Compose.
use std::{
    io,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const COMPOSE_REL: &str = "dash/collector/docker-compose.yml";

pub struct CollectorHandle {
    pub compose_file: PathBuf,
    child: Option<Child>,
}

impl CollectorHandle {
    /// `docker compose up` for the bundled collector stack. Returns `None` if Docker
    /// is missing; prints a hint and leaves the hub running without a collector.
    pub fn start(root: &Path) -> io::Result<Option<Self>> {
        let compose_file = root.join(COMPOSE_REL);
        if !compose_file.is_file() {
            eprintln!(
                "collector: missing {} — skip (--collector needs the monorepo layout)",
                compose_file.display()
            );
            return Ok(None);
        }
        if which_docker().is_none() {
            eprintln!("collector: docker not found on PATH — install Docker Desktop / Engine");
            eprintln!("  or run: docker compose -f {COMPOSE_REL} up -d");
            return Ok(None);
        }

        eprintln!("collector: starting stack ({COMPOSE_REL}) …");
        let status = compose(&compose_file, &["up", "-d", "--remove-orphans"])?;
        if !status.success() {
            return Err(io::Error::other(
                "docker compose up failed — is Docker running?",
            ));
        }

        wait_port(4318, Duration::from_secs(45));
        eprintln!("collector: OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317");
        eprintln!("collector: Jaeger UI http://127.0.0.1:16686/");
        eprintln!("  export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318");

        Ok(Some(Self {
            compose_file,
            child: None,
        }))
    }

    pub fn stop(self) {
        eprintln!("collector: stopping stack …");
        let _ = compose(&self.compose_file, &["down"]);
        if let Some(mut child) = self.child {
            let _ = child.kill();
        }
    }
}

fn which_docker() -> Option<PathBuf> {
    Command::new("docker")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .filter(|s| s.success())
        .map(|_| PathBuf::from("docker"))
}

fn compose(file: &Path, args: &[&str]) -> io::Result<std::process::ExitStatus> {
    let mut cmd = Command::new("docker");
    cmd.arg("compose").arg("-f").arg(file).args(args);
    cmd.status()
}

fn wait_port(port: u16, budget: Duration) {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    eprintln!("collector: port {port} not open yet — check: docker compose -f {COMPOSE_REL} logs");
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
