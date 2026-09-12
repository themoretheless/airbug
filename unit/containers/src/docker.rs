//! Docker/Podman CLI runtime and C#-style container builder.
use crate::wait::Wait;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt,
    process::{Command, Output, Stdio},
    time::Duration,
};

/// Errors from building, starting, or inspecting containers.
#[derive(Debug)]
pub enum ContainerError {
    /// Docker/Podman CLI is missing or not usable.
    RuntimeUnavailable(String),
    /// CLI command failed.
    Command {
        /// Invoked program.
        program: String,
        /// Arguments.
        args: Vec<String>,
        /// Captured stderr/stdout.
        message: String,
    },
    /// Wait strategy timed out.
    Timeout(String),
    /// Container stopped unexpectedly.
    Failed(String),
    /// JSON inspect/parse problems.
    Inspect(String),
    /// Requested container port is not published.
    PortNotMapped(u16),
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeUnavailable(m) => write!(f, "container runtime unavailable: {m}"),
            Self::Command {
                program,
                args,
                message,
            } => write!(f, "`{program} {}` failed: {message}", args.join(" ")),
            Self::Timeout(m) | Self::Failed(m) | Self::Inspect(m) => write!(f, "{m}"),
            Self::PortNotMapped(p) => write!(f, "container port {p} is not published"),
        }
    }
}

impl std::error::Error for ContainerError {}

/// Detected container CLI (`docker` or `podman`).
#[derive(Clone, Debug)]
pub struct Runtime {
    program: String,
}

impl Runtime {
    /// Prefer `docker`, then `podman`.
    pub fn detect() -> Result<Self, ContainerError> {
        for program in ["docker", "podman"] {
            let output = Command::new(program)
                .arg("version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if matches!(output, Ok(status) if status.success()) {
                return Ok(Self {
                    program: program.into(),
                });
            }
        }
        Err(ContainerError::RuntimeUnavailable(
            "neither `docker` nor `podman` succeeded on `version`".into(),
        ))
    }

    fn run(&self, args: &[&str]) -> Result<Output, ContainerError> {
        let output = Command::new(&self.program)
            .args(args)
            .output()
            .map_err(|e| ContainerError::RuntimeUnavailable(e.to_string()))?;
        if output.status.success() {
            Ok(output)
        } else {
            let message = String::from_utf8_lossy(&output.stderr);
            let fallback = String::from_utf8_lossy(&output.stdout);
            Err(ContainerError::Command {
                program: self.program.clone(),
                args: args.iter().map(|s| (*s).to_string()).collect(),
                message: if message.trim().is_empty() {
                    fallback.trim().to_string()
                } else {
                    message.trim().to_string()
                },
            })
        }
    }

    fn run_text(&self, args: &[&str]) -> Result<String, ContainerError> {
        let output = self.run(args)?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub(crate) fn is_running(&self, id: &str) -> Result<bool, ContainerError> {
        let state = self.run_text(&["inspect", "-f", "{{.State.Running}}", id])?;
        Ok(state.eq_ignore_ascii_case("true"))
    }

    pub(crate) fn logs(&self, id: &str, stdout_only: bool) -> Result<String, ContainerError> {
        let mut args = vec!["logs"];
        if stdout_only {
            args.push("--stdout");
        }
        args.push(id);
        let output = Command::new(&self.program)
            .args(&args)
            .output()
            .map_err(|e| ContainerError::RuntimeUnavailable(e.to_string()))?;
        let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
        if !stdout_only {
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        Ok(combined)
    }

    fn inspect_json(&self, id: &str) -> Result<Value, ContainerError> {
        let text = self.run_text(&["inspect", id])?;
        let value: Value =
            serde_json::from_str(&text).map_err(|e| ContainerError::Inspect(e.to_string()))?;
        value
            .as_array()
            .and_then(|a| a.first())
            .cloned()
            .ok_or_else(|| ContainerError::Inspect("inspect returned empty array".into()))
    }

    fn mapped_port(&self, id: &str, container_port: u16) -> Result<u16, ContainerError> {
        let inspect = self.inspect_json(id)?;
        let ports = inspect
            .pointer("/NetworkSettings/Ports")
            .ok_or_else(|| ContainerError::Inspect("missing NetworkSettings.Ports".into()))?;
        for key in [
            format!("{container_port}/tcp"),
            format!("{container_port}/udp"),
        ] {
            if let Some(bindings) = ports.get(&key).and_then(|v| v.as_array()) {
                for binding in bindings {
                    if let Some(host) = binding.get("HostPort").and_then(|v| v.as_str()) {
                        if let Ok(port) = host.parse::<u16>() {
                            return Ok(port);
                        }
                    }
                }
            }
        }
        Err(ContainerError::PortNotMapped(container_port))
    }

    fn stop_rm(&self, id: &str) {
        let _ = self.run(&["rm", "-f", id]);
    }
}

/// Immutable builder matching Testcontainers.NET `ContainerBuilder` naming.
#[derive(Clone, Debug)]
pub struct ContainerBuilder {
    image: String,
    name: Option<String>,
    env: BTreeMap<String, String>,
    /// Random host port bindings: container port only (`-p 6379`).
    ports: Vec<u16>,
    /// Fixed host:container bindings.
    fixed_ports: Vec<(u16, u16)>,
    command: Vec<String>,
    entrypoint: Option<String>,
    wait: Wait,
    startup_timeout: Duration,
    auto_remove: bool,
    labels: BTreeMap<String, String>,
    privileged: bool,
    network: Option<String>,
    working_dir: Option<String>,
}

impl ContainerBuilder {
    /// Start from an image reference (`repo:tag`), like `new ContainerBuilder("redis:7")`.
    pub fn new(image: impl Into<String>) -> Self {
        let mut labels = BTreeMap::new();
        labels.insert("airbug.containers".into(), "true".into());
        Self {
            image: image.into(),
            name: None,
            env: BTreeMap::new(),
            ports: Vec::new(),
            fixed_ports: Vec::new(),
            command: Vec::new(),
            entrypoint: None,
            wait: Wait::Running,
            startup_timeout: Duration::from_secs(60),
            auto_remove: true,
            labels,
            privileged: false,
            network: None,
            working_dir: None,
        }
    }

    /// `WithImage`.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    /// `WithName`.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `WithEnvironment`.
    pub fn with_environment(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// `WithPortBinding(port, assignRandomHostPort)`.
    ///
    /// When `assign_random_host_port` is true, publishes `container_port` to an ephemeral host port.
    /// When false, binds the same port on the host (`host:container` with equal ports).
    pub fn with_port_binding(mut self, container_port: u16, assign_random_host_port: bool) -> Self {
        if assign_random_host_port {
            self.ports.push(container_port);
        } else {
            self.fixed_ports.push((container_port, container_port));
        }
        self
    }

    /// Publish `container_port` onto a specific `host_port`.
    pub fn with_fixed_port_binding(mut self, host_port: u16, container_port: u16) -> Self {
        self.fixed_ports.push((host_port, container_port));
        self
    }

    /// `WithCommand`.
    pub fn with_command<I, S>(mut self, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command = command.into_iter().map(Into::into).collect();
        self
    }

    /// `WithEntrypoint` — single executable override.
    pub fn with_entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    /// `WithWaitStrategy`.
    pub fn with_wait_strategy(mut self, wait: Wait) -> Self {
        self.wait = wait;
        self
    }

    /// Startup timeout for wait strategies (default 60s).
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// `WithAutoRemove` (default true → `docker rm -f` on drop).
    pub fn with_auto_remove(mut self, enabled: bool) -> Self {
        self.auto_remove = enabled;
        self
    }

    /// `WithLabel`.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// `WithPrivileged`.
    pub fn with_privileged(mut self, enabled: bool) -> Self {
        self.privileged = enabled;
        self
    }

    /// `WithNetwork`.
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }

    /// `WithWorkingDirectory`.
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// `Build` — materializes a not-yet-started container handle.
    pub fn build(self) -> Result<Container, ContainerError> {
        Ok(Container {
            runtime: Runtime::detect()?,
            id: None,
            image: self.image,
            name: self.name,
            env: self.env,
            ports: self.ports,
            fixed_ports: self.fixed_ports,
            command: self.command,
            entrypoint: self.entrypoint,
            wait: self.wait,
            startup_timeout: self.startup_timeout,
            auto_remove: self.auto_remove,
            labels: self.labels,
            privileged: self.privileged,
            network: self.network,
            working_dir: self.working_dir,
        })
    }
}

/// Running (or built) container. Stops/removes on [`Drop`] when auto-remove is on.
#[derive(Debug)]
pub struct Container {
    runtime: Runtime,
    id: Option<String>,
    image: String,
    name: Option<String>,
    env: BTreeMap<String, String>,
    ports: Vec<u16>,
    fixed_ports: Vec<(u16, u16)>,
    command: Vec<String>,
    entrypoint: Option<String>,
    wait: Wait,
    startup_timeout: Duration,
    auto_remove: bool,
    labels: BTreeMap<String, String>,
    privileged: bool,
    network: Option<String>,
    working_dir: Option<String>,
}

impl Container {
    /// `StartAsync` equivalent (sync for `#[test]`).
    pub fn start(mut self) -> Result<Self, ContainerError> {
        let mut owned: Vec<String> = vec!["run".into(), "-d".into()];
        for (key, value) in &self.labels {
            owned.push("--label".into());
            owned.push(format!("{key}={value}"));
        }
        if let Some(name) = &self.name {
            owned.push("--name".into());
            owned.push(name.clone());
        }
        for (key, value) in &self.env {
            owned.push("-e".into());
            owned.push(format!("{key}={value}"));
        }
        for container_port in &self.ports {
            owned.push("-p".into());
            owned.push(format!("{container_port}"));
        }
        for (host, container) in &self.fixed_ports {
            owned.push("-p".into());
            owned.push(format!("{host}:{container}"));
        }
        if self.privileged {
            owned.push("--privileged".into());
        }
        if let Some(network) = &self.network {
            owned.push("--network".into());
            owned.push(network.clone());
        }
        if let Some(dir) = &self.working_dir {
            owned.push("-w".into());
            owned.push(dir.clone());
        }
        if let Some(entrypoint) = &self.entrypoint {
            owned.push("--entrypoint".into());
            owned.push(entrypoint.clone());
        }
        owned.push(self.image.clone());
        owned.extend(self.command.iter().cloned());

        let arg_refs: Vec<&str> = owned.iter().map(String::as_str).collect();
        let id = self.runtime.run_text(&arg_refs)?;
        self.id = Some(id.clone());

        let runtime = self.runtime.clone();
        let host = hostname();
        let wait = self.wait.clone();
        let timeout = self.startup_timeout;
        let mapped = |container_port: u16| runtime.mapped_port(&id, container_port);
        if let Err(err) = wait.block_until(&runtime, &id, &host, &mapped, timeout) {
            runtime.stop_rm(&id);
            self.id = None;
            return Err(err);
        }
        Ok(self)
    }

    /// Docker/Podman container id.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Hostname used for client connections (usually `127.0.0.1`).
    pub fn hostname(&self) -> String {
        hostname()
    }

    /// `GetMappedPublicPort`.
    pub fn get_mapped_public_port(&self, container_port: u16) -> Result<u16, ContainerError> {
        let id = self
            .id
            .as_deref()
            .ok_or_else(|| ContainerError::Failed("container is not started".into()))?;
        self.runtime.mapped_port(id, container_port)
    }

    /// Combined logs since start.
    pub fn get_logs(&self) -> Result<String, ContainerError> {
        let id = self
            .id
            .as_deref()
            .ok_or_else(|| ContainerError::Failed("container is not started".into()))?;
        self.runtime.logs(id, false)
    }

    /// Force-stop and remove now (also done on drop when auto-remove is enabled).
    pub fn dispose(&mut self) {
        if let Some(id) = self.id.take() {
            self.runtime.stop_rm(&id);
        }
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        if self.auto_remove {
            self.dispose();
        }
    }
}

fn hostname() -> String {
    std::env::var("AIRBUG_CONTAINERS_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}
