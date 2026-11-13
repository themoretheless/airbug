//! .NET Aspire-style AppHost & Resource Orchestrator (`.airbug/stack.json` + zero-config discovery).
use crate::{
    collector,
    config::RootPaths,
    error::{HubError, Result},
};
use airbug_run::model::now_ms;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fs,
    io::{BufRead, BufReader},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub const STACK_SCHEMA: &str = "airbug.stack/1";
pub const MAX_CONSOLE_LINES: usize = 2000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackManifest {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub resources: Vec<ResourceSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub name: String,
    /// `"process"` (default) or `"container"`.
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub ports: Vec<PortBinding>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub healthcheck: Option<HealthcheckSpec>,
}

fn default_kind() -> String {
    "process".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortBinding {
    pub host: u16,
    #[serde(default)]
    pub container: Option<u16>,
    #[serde(default = "default_scheme")]
    pub scheme: String,
}

fn default_scheme() -> String {
    "http".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthcheckSpec {
    /// `"tcp"` or `"http"`.
    #[serde(default = "default_hc_kind")]
    pub kind: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub url: Option<String>,
}

fn default_hc_kind() -> String {
    "tcp".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLine {
    pub time_ms: u64,
    pub stream: String,
    pub line: String,
}

struct ManagedRuntime {
    spec: ResourceSpec,
    child: Option<Child>,
    container_id: Option<String>,
    pid: Option<u32>,
    state: String,
    health: String,
    started_at_ms: Option<u64>,
    exit_code: Option<i32>,
    effective_env: BTreeMap<String, String>,
    logs: Arc<Mutex<VecDeque<ConsoleLine>>>,
}

pub struct ResourceManager {
    paths: RootPaths,
    hub_id: String,
    hub_port: u16,
    started_at_ms: u64,
    stack_path: Mutex<Option<PathBuf>>,
    managed: Mutex<BTreeMap<String, ManagedRuntime>>,
    docker_cache: Mutex<Option<(Instant, Vec<Value>)>>,
}

const SECRET_PATTERNS: &[&str] = &[
    "PASSWORD",
    "SECRET",
    "TOKEN",
    "KEY",
    "DSN",
    "AUTH",
    "CREDENTIAL",
];

pub fn is_secret_env_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    // Safe airbug/OTEL service keys that contain DSN or KEY-like substrings?
    // The prompt states: Masks secret env vars (`PASSWORD`, `SECRET`, `TOKEN`, `KEY`, `DSN`, `AUTH`, `CREDENTIAL`) in `masked_env`
    SECRET_PATTERNS.iter().any(|pat| upper.contains(pat))
}

pub fn mask_env(env: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    env.iter()
        .map(|(k, v)| {
            let masked = if is_secret_env_key(k) {
                "••••••••".into()
            } else {
                v.clone()
            };
            (k.clone(), masked)
        })
        .collect()
}

impl ResourceManager {
    pub fn new(paths: RootPaths, hub_id: String, hub_port: u16) -> Arc<Self> {
        let mgr = Arc::new(Self {
            paths,
            hub_id,
            hub_port,
            started_at_ms: now_ms(),
            stack_path: Mutex::new(None),
            managed: Mutex::new(BTreeMap::new()),
            docker_cache: Mutex::new(None),
        });
        mgr.reload_stack(None);
        mgr
    }

    pub fn set_stack_path(&self, custom: Option<PathBuf>) {
        self.reload_stack(custom);
    }

    pub fn reload_stack(&self, custom: Option<PathBuf>) {
        let candidate = custom.or_else(|| {
            let dot = self.paths.root.join(".airbug/stack.json");
            if dot.is_file() {
                Some(dot)
            } else {
                let top = self.paths.root.join("airbug.stack.json");
                top.is_file().then_some(top)
            }
        });
        if let Ok(mut sp) = self.stack_path.lock() {
            *sp = candidate.clone();
        }
        let Some(path) = candidate else {
            return;
        };
        let Ok(raw) = fs::read_to_string(&path) else {
            return;
        };
        let Ok(manifest) = serde_json::from_str::<StackManifest>(&raw) else {
            return;
        };
        let Ok(mut guard) = self.managed.lock() else {
            return;
        };
        for spec in manifest.resources {
            if spec.name.trim().is_empty() {
                continue;
            }
            let name = spec.name.clone();
            let effective_env = self.build_env_for(&spec, &guard);
            guard
                .entry(name)
                .and_modify(|rt| {
                    rt.spec = spec.clone();
                    rt.effective_env = effective_env.clone();
                })
                .or_insert_with(|| ManagedRuntime {
                    spec,
                    child: None,
                    container_id: None,
                    pid: None,
                    state: "stopped".into(),
                    health: "unknown".into(),
                    started_at_ms: None,
                    exit_code: None,
                    effective_env,
                    logs: Arc::new(Mutex::new(VecDeque::new())),
                });
        }
        // Second pass so every resource sees `SERVICES__*` of all stack peers.
        let specs: Vec<ResourceSpec> = guard.values().map(|r| r.spec.clone()).collect();
        for spec in specs {
            let env_map = self.build_env_for(&spec, &guard);
            if let Some(rt) = guard.get_mut(&spec.name) {
                rt.effective_env = env_map;
            }
        }
    }

    fn build_env_for(
        &self,
        spec: &ResourceSpec,
        all: &BTreeMap<String, ManagedRuntime>,
    ) -> BTreeMap<String, String> {
        let mut env = spec.env.clone();
        env.insert("OTEL_SERVICE_NAME".into(), spec.name.clone());
        env.entry("OTEL_EXPORTER_OTLP_ENDPOINT".into())
            .or_insert_with(|| format!("http://127.0.0.1:{}", collector::DEFAULT_OTLP_HTTP_PORT));
        let attrs = format!(
            "service.name={},airbug.hub_id={},airbug.resource={}",
            spec.name, self.hub_id, spec.name
        );
        env.insert("OTEL_RESOURCE_ATTRIBUTES".into(), attrs);
        env.insert(
            "AIRBUG_ERR_ENDPOINT".into(),
            format!(
                "http://127.0.0.1:{}/api/v1/errors/{}",
                self.hub_port, self.hub_id
            ),
        );
        env.insert("AIRBUG_HUB_ID".into(), self.hub_id.clone());
        env.insert(
            "AIRBUG_SENTRY_DSN".into(),
            format!("http://airbug@127.0.0.1:{}/1", self.hub_port),
        );

        // Service discovery env vars across all stack resources (including self)
        for peer in all.values().map(|r| &r.spec).chain(std::iter::once(spec)) {
            let upper: String = peer
                .name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_uppercase()
                    } else {
                        '_'
                    }
                })
                .collect();
            for p in &peer.ports {
                let scheme_upper = p.scheme.to_ascii_uppercase();
                let val = if p.scheme.eq_ignore_ascii_case("tcp") {
                    format!("127.0.0.1:{}", p.host)
                } else {
                    format!("{}://127.0.0.1:{}", p.scheme, p.host)
                };
                env.insert(format!("SERVICES__{upper}__{scheme_upper}"), val);
            }
        }
        env
    }

    /// Start all resources in topological `depends_on` order.
    pub fn start_all(&self) -> Result<Vec<String>> {
        self.reload_stack(None);
        let order = {
            let guard = self
                .managed
                .lock()
                .map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
            topo_sort(&guard)
        };
        let mut started = Vec::new();
        for name in order {
            self.start_resource(&name)?;
            started.push(name);
        }
        Ok(started)
    }

    pub fn start_resource(&self, name: &str) -> Result<Value> {
        self.reload_stack(None);
        let mut guard = self
            .managed
            .lock()
            .map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
        let env_map = {
            let Some(rt) = guard.get(name) else {
                return Err(HubError::msg(format!("resource not found: {name}")));
            };
            self.build_env_for(&rt.spec, &guard)
        };
        let rt = guard
            .get_mut(name)
            .ok_or_else(|| HubError::msg(format!("resource not found: {name}")))?;
        rt.effective_env = env_map;

        // If already running, check if still alive
        refresh_runtime_locked(rt);
        if rt.state == "running" {
            return Ok(serialize_managed(rt));
        }

        if rt.spec.kind.eq_ignore_ascii_case("container") {
            let image = rt
                .spec
                .image
                .as_deref()
                .ok_or_else(|| HubError::msg("container resource requires `image`"))?;
            let container_name = format!(
                "airbug-{}-{}",
                self.hub_id.chars().take(8).collect::<String>(),
                rt.spec.name
            );
            let _ = Command::new("docker")
                .args(["rm", "-f", &container_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let mut cmd = Command::new("docker");
            cmd.args(["run", "-d", "--name", &container_name]);
            cmd.arg("--label")
                .arg(format!("airbug.root={}", self.paths.root.display()));
            cmd.arg("--label")
                .arg(format!("airbug.resource={}", rt.spec.name));
            for p in &rt.spec.ports {
                let cport = p.container.unwrap_or(p.host);
                cmd.arg("-p").arg(format!("127.0.0.1:{}:{cport}", p.host));
            }
            for (k, v) in &rt.effective_env {
                cmd.arg("-e").arg(format!("{k}={v}"));
            }
            cmd.arg(image);
            cmd.args(&rt.spec.args);
            let out = cmd
                .output()
                .map_err(|e| HubError::msg(format!("failed to run docker: {e}")))?;
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(HubError::msg(format!(
                    "docker run failed: {}",
                    stderr.trim()
                )));
            }
            let cid = String::from_utf8_lossy(&out.stdout).trim().to_string();
            rt.container_id = Some(cid);
            rt.state = "running".into();
            rt.started_at_ms = Some(now_ms());
            rt.exit_code = None;
            refresh_runtime_locked(rt);
            return Ok(serialize_managed(rt));
        }

        let program = rt
            .spec
            .command
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| HubError::msg(format!("resource '{name}' has no command")))?;

        let cwd = match &rt.spec.cwd {
            Some(rel) => {
                let candidate = self.paths.root.join(rel);
                let canon_root = self
                    .paths
                    .root
                    .canonicalize()
                    .unwrap_or_else(|_| self.paths.root.clone());
                let canon_cwd = candidate
                    .canonicalize()
                    .map_err(|e| HubError::msg(format!("invalid cwd: {e}")))?;
                if !canon_cwd.starts_with(&canon_root) {
                    return Err(HubError::msg(
                        "resource cwd must stay inside workspace root",
                    ));
                }
                canon_cwd
            }
            None => self.paths.root.clone(),
        };

        let mut cmd = Command::new(program);
        cmd.current_dir(cwd)
            .args(&rt.spec.args)
            .envs(&rt.effective_env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| HubError::msg(format!("failed to spawn '{program}': {e}")))?;
        let pid = child.id();
        let logs = Arc::clone(&rt.logs);

        if let Some(stdout) = child.stdout.take() {
            let logs_out = Arc::clone(&logs);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(std::result::Result::ok) {
                    push_log(&logs_out, "stdout", line);
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let logs_err = Arc::clone(&logs);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(std::result::Result::ok) {
                    push_log(&logs_err, "stderr", line);
                }
            });
        }

        rt.pid = Some(pid);
        rt.child = Some(child);
        rt.state = "running".into();
        rt.started_at_ms = Some(now_ms());
        rt.exit_code = None;
        refresh_runtime_locked(rt);
        Ok(serialize_managed(rt))
    }

    pub fn stop_resource(&self, name: &str) -> Result<Value> {
        let mut guard = self
            .managed
            .lock()
            .map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
        let rt = guard
            .get_mut(name)
            .ok_or_else(|| HubError::msg(format!("resource not found: {name}")))?;
        stop_runtime_locked(rt);
        Ok(serialize_managed(rt))
    }

    pub fn restart_resource(&self, name: &str) -> Result<Value> {
        let _ = self.stop_resource(name)?;
        self.start_resource(name)
    }

    pub fn stop_all(&self) {
        if let Ok(mut guard) = self.managed.lock() {
            for rt in guard.values_mut() {
                stop_runtime_locked(rt);
            }
        }
    }

    pub fn logs(&self, name: &str, tail: usize) -> Result<Value> {
        let tail = tail.clamp(1, MAX_CONSOLE_LINES);
        if name == "airbug-hub" || name == "otlp-collector" {
            return Ok(json!({
                "resource": name,
                "lines": [{
                    "time_ms": self.started_at_ms,
                    "stream": "stdout",
                    "line": format!("{name} is running inside airbug-hub (PID {})", std::process::id()),
                }],
            }));
        }
        let guard = self
            .managed
            .lock()
            .map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
        let Some(rt) = guard.get(name) else {
            return Err(HubError::msg(format!("resource not found: {name}")));
        };
        let buf = rt
            .logs
            .lock()
            .map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
        let skip = buf.len().saturating_sub(tail);
        let lines: Vec<ConsoleLine> = buf.iter().skip(skip).cloned().collect();
        Ok(json!({
            "resource": name,
            "lines": lines,
        }))
    }

    pub fn list_resources(&self) -> Value {
        self.reload_stack(None);
        let mut resources: Vec<Value> = Vec::new();

        // 1. Built-in airbug-hub resource
        let hub_env = BTreeMap::from([
            ("AIRBUG_HUB_ID".into(), self.hub_id.clone()),
            ("AIRBUG_ROOT".into(), self.paths.root.display().to_string()),
            (
                "AIRBUG_SENTRY_DSN".into(),
                format!("http://airbug@127.0.0.1:{}/1", self.hub_port),
            ),
        ]);
        resources.push(json!({
            "name": "airbug-hub",
            "kind": "process",
            "builtin": true,
            "controllable": false,
            "state": "running",
            "health": "healthy",
            "pid": std::process::id(),
            "image": Value::Null,
            "started_at_ms": self.started_at_ms,
            "endpoints": [format!("http://127.0.0.1:{}", self.hub_port)],
            "ports": [{ "host": self.hub_port, "container": Value::Null, "scheme": "http" }],
            "masked_env": mask_env(&hub_env),
            "env_keys": hub_env.keys().collect::<Vec<_>>(),
            "depends_on": [],
            "log_count": 1,
        }));

        // 2. Built-in otlp-collector resource
        let probe = collector::probe();
        let otlp_up = probe.otlp_http || probe.otlp_grpc;
        let mut otlp_endpoints = vec![format!("http://127.0.0.1:{}/v1/*", self.hub_port)];
        if probe.otlp_http {
            otlp_endpoints.insert(
                0,
                format!("http://127.0.0.1:{}", collector::DEFAULT_OTLP_HTTP_PORT),
            );
        }
        if probe.otlp_grpc {
            otlp_endpoints.push(format!(
                "grpc://127.0.0.1:{}",
                collector::DEFAULT_OTLP_GRPC_PORT
            ));
        }
        let otlp_env = BTreeMap::from([
            (
                "OTEL_EXPORTER_OTLP_ENDPOINT".into(),
                format!("http://127.0.0.1:{}", collector::DEFAULT_OTLP_HTTP_PORT),
            ),
            (
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT".into(),
                format!("http://127.0.0.1:{}/v1/traces", self.hub_port),
            ),
        ]);
        resources.push(json!({
            "name": "otlp-collector",
            "kind": "collector",
            "builtin": true,
            "controllable": false,
            "state": if otlp_up { "running" } else { "standby" },
            "health": if probe.otlp_http { "healthy" } else { "standby" },
            "pid": std::process::id(),
            "image": Value::Null,
            "started_at_ms": self.started_at_ms,
            "endpoints": otlp_endpoints,
            "ports": [
                { "host": collector::DEFAULT_OTLP_HTTP_PORT, "container": Value::Null, "scheme": "http" },
                { "host": collector::DEFAULT_OTLP_GRPC_PORT, "container": Value::Null, "scheme": "grpc" }
            ],
            "masked_env": mask_env(&otlp_env),
            "env_keys": otlp_env.keys().collect::<Vec<_>>(),
            "depends_on": [],
            "log_count": 1,
        }));

        // 3. Managed stack.json resources
        let mut known_names =
            HashSet::from(["airbug-hub".to_string(), "otlp-collector".to_string()]);
        if let Ok(mut guard) = self.managed.lock() {
            for rt in guard.values_mut() {
                refresh_runtime_locked(rt);
                known_names.insert(rt.spec.name.clone());
                resources.push(serialize_managed(rt));
            }
        }

        // 4. Discovered Docker containers (cached for 5s so polling never stalls)
        for c in self.discover_containers() {
            let name = c["name"].as_str().unwrap_or("").to_string();
            if !name.is_empty() && !known_names.contains(&name) {
                known_names.insert(name);
                resources.push(c);
            }
        }

        let stack_file = self
            .stack_path
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .map(|p| p.display().to_string());

        json!({
            "schema": STACK_SCHEMA,
            "stack_file": stack_file,
            "resources": resources,
        })
    }

    fn discover_containers(&self) -> Vec<Value> {
        if std::env::var_os("AIRBUG_DISABLE_DOCKER_DISCOVERY").is_some() {
            return Vec::new();
        }
        if let Ok(guard) = self.docker_cache.lock()
            && let Some((at, ref cached)) = *guard
            && at.elapsed() < Duration::from_secs(5)
        {
            return cached.clone();
        }
        let discovered = run_docker_ps(&self.paths.root);
        if let Ok(mut guard) = self.docker_cache.lock() {
            *guard = Some((Instant::now(), discovered.clone()));
        }
        discovered
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

fn push_log(buf: &Arc<Mutex<VecDeque<ConsoleLine>>>, stream: &str, line: String) {
    if let Ok(mut guard) = buf.lock() {
        if guard.len() >= MAX_CONSOLE_LINES {
            guard.pop_front();
        }
        guard.push_back(ConsoleLine {
            time_ms: now_ms(),
            stream: stream.to_string(),
            line,
        });
    }
}

fn stop_runtime_locked(rt: &mut ManagedRuntime) {
    if let Some(mut child) = rt.child.take() {
        let _ = child.kill();
        let status = child.wait().ok();
        rt.exit_code = status.and_then(|s| s.code());
    }
    if let Some(cid) = rt.container_id.take() {
        let _ = Command::new("docker")
            .args(["rm", "-f", &cid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    rt.pid = None;
    rt.state = "stopped".into();
    rt.health = "stopped".into();
}

fn refresh_runtime_locked(rt: &mut ManagedRuntime) {
    if let Some(ref mut child) = rt.child
        && let Ok(Some(status)) = child.try_wait()
    {
        let code = status.code().unwrap_or(-1);
        rt.exit_code = Some(code);
        rt.pid = None;
        rt.child = None;
        rt.state = if code == 0 {
            "exited".into()
        } else {
            "failed".into()
        };
        rt.health = "unhealthy".into();
    }

    if rt.state == "running" {
        rt.health = check_health(&rt.spec);
    } else if rt.state == "stopped" {
        rt.health = "stopped".into();
    }
}

fn check_health(spec: &ResourceSpec) -> String {
    let Some(ref hc) = spec.healthcheck else {
        if let Some(first_port) = spec.ports.first() {
            return if tcp_port_open(first_port.host) {
                "healthy".into()
            } else {
                "starting".into()
            };
        }
        return "healthy".into();
    };

    if hc.kind.eq_ignore_ascii_case("http")
        && let Some(ref url) = hc.url
        && let Some(port) = extract_loopback_port(url)
    {
        return if tcp_port_open(port) {
            "healthy".into()
        } else {
            "unhealthy".into()
        };
    }
    if let Some(port) = hc.port.or_else(|| spec.ports.first().map(|p| p.host)) {
        return if tcp_port_open(port) {
            "healthy".into()
        } else {
            "unhealthy".into()
        };
    }
    "healthy".into()
}

fn extract_loopback_port(url: &str) -> Option<u16> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let authority = rest.split('/').next()?;
    let (_, port_s) = authority.split_once(':')?;
    port_s.parse().ok()
}

fn tcp_port_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(120)).is_ok()
}

fn serialize_managed(rt: &ManagedRuntime) -> Value {
    let endpoints: Vec<String> = rt
        .spec
        .ports
        .iter()
        .map(|p| {
            if p.scheme.eq_ignore_ascii_case("tcp") {
                format!("tcp://127.0.0.1:{}", p.host)
            } else {
                format!("{}://127.0.0.1:{}", p.scheme, p.host)
            }
        })
        .collect();
    let log_count = rt.logs.lock().map(|b| b.len()).unwrap_or(0);
    json!({
        "name": rt.spec.name,
        "kind": rt.spec.kind,
        "builtin": false,
        "controllable": true,
        "command": rt.spec.command,
        "args": rt.spec.args,
        "image": rt.spec.image,
        "state": rt.state,
        "health": rt.health,
        "pid": rt.pid,
        "container_id": rt.container_id,
        "started_at_ms": rt.started_at_ms,
        "exit_code": rt.exit_code,
        "endpoints": endpoints,
        "ports": rt.spec.ports,
        "masked_env": mask_env(&rt.effective_env),
        "env_keys": rt.effective_env.keys().collect::<Vec<_>>(),
        "depends_on": rt.spec.depends_on,
        "log_count": log_count,
    })
}

fn topo_sort(map: &BTreeMap<String, ManagedRuntime>) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut visiting = HashSet::new();
    let mut out = Vec::new();

    fn visit(
        name: &str,
        map: &BTreeMap<String, ManagedRuntime>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        out: &mut Vec<String>,
    ) {
        if visited.contains(name) || visiting.contains(name) {
            return;
        }
        visiting.insert(name.to_string());
        if let Some(rt) = map.get(name) {
            for dep in &rt.spec.depends_on {
                if map.contains_key(dep) {
                    visit(dep, map, visited, visiting, out);
                }
            }
        }
        visiting.remove(name);
        visited.insert(name.to_string());
        out.push(name.to_string());
    }

    for k in map.keys() {
        visit(k, map, &mut visited, &mut visiting, &mut out);
    }
    out
}

fn run_docker_ps(root: &Path) -> Vec<Value> {
    let Ok(mut child) = Command::new("docker")
        .args(["ps", "--format", "{{json .}}"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    let deadline = Instant::now() + Duration::from_millis(450);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Vec::new();
                }
                break;
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Vec::new();
                }
                thread::sleep(Duration::from_millis(25));
            }
            Err(_) => return Vec::new(),
        }
    }
    let Some(mut stdout) = child.stdout.take() else {
        return Vec::new();
    };
    let mut text = String::new();
    use std::io::Read;
    if stdout.read_to_string(&mut text).is_err() {
        return Vec::new();
    }
    let root_str = root.display().to_string();
    let mut list = Vec::new();
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<HashMap<String, Value>>(line) else {
            continue;
        };
        let names = v
            .get("Names")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let image = v
            .get("Image")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let labels = v.get("Labels").and_then(Value::as_str).unwrap_or("");
        let relevant = labels.contains(&root_str)
            || names.contains("airbug")
            || names.contains("otel")
            || names.contains("jaeger")
            || image.contains("opentelemetry")
            || image.contains("jaeger");
        if !relevant {
            continue;
        }
        let cid = v
            .get("ID")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        list.push(json!({
            "name": names,
            "kind": "container",
            "builtin": false,
            "controllable": false,
            "image": image,
            "container_id": cid,
            "state": "running",
            "health": "healthy",
            "pid": Value::Null,
            "endpoints": [],
            "ports": [],
            "masked_env": {},
            "env_keys": [],
            "depends_on": [],
            "log_count": 0,
        }));
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_masking_hides_sensitive_keys_and_preserves_normal_keys() {
        let env = BTreeMap::from([
            ("POSTGRES_PASSWORD".into(), "supersecret".into()),
            ("API_SECRET_TOKEN".into(), "tok_123".into()),
            (
                "AIRBUG_SENTRY_DSN".into(),
                "http://airbug@127.0.0.1:8790/1".into(),
            ),
            ("OTEL_SERVICE_NAME".into(), "checkout-api".into()),
            ("SERVICES__POSTGRES__TCP".into(), "127.0.0.1:5432".into()),
        ]);
        let masked = mask_env(&env);
        assert_eq!(masked["POSTGRES_PASSWORD"], "••••••••");
        assert_eq!(masked["API_SECRET_TOKEN"], "••••••••");
        assert_eq!(masked["AIRBUG_SENTRY_DSN"], "••••••••");
        assert_eq!(masked["OTEL_SERVICE_NAME"], "checkout-api");
        assert_eq!(masked["SERVICES__POSTGRES__TCP"], "127.0.0.1:5432");
    }

    #[test]
    fn resource_manager_spawns_process_captures_console_and_injects_otel_env() {
        let root =
            std::env::temp_dir().join(format!("airbug-resources-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".airbug")).unwrap();

        #[cfg(unix)]
        let (cmd, args) = (
            "sh",
            vec![
                "-c",
                "echo \"SVC=$OTEL_SERVICE_NAME DB=$SERVICES__DB__TCP\"; sleep 5",
            ],
        );
        #[cfg(windows)]
        let (cmd, args) = (
            "cmd",
            vec!["/C", "echo SVC=%OTEL_SERVICE_NAME% DB=%SERVICES__DB__TCP%"],
        );

        let stack = json!({
            "schema": "airbug.stack/1",
            "resources": [
                {
                    "name": "db",
                    "kind": "process",
                    "command": cmd,
                    "args": args,
                    "ports": [{ "host": 15432, "scheme": "tcp" }],
                    "env": { "DB_PASSWORD": "my_secret_password" }
                },
                {
                    "name": "api",
                    "kind": "process",
                    "command": cmd,
                    "args": args,
                    "ports": [{ "host": 13000, "scheme": "http" }],
                    "depends_on": ["db"]
                }
            ]
        });
        fs::write(
            root.join(".airbug/stack.json"),
            serde_json::to_vec_pretty(&stack).unwrap(),
        )
        .unwrap();

        let mgr = ResourceManager::new(RootPaths::new(&root), "hub-test-uuid".into(), 8790);
        let started = mgr.start_all().unwrap();
        assert_eq!(started, vec!["db".to_string(), "api".to_string()]);

        // Wait briefly for stdout line
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut saw_output = false;
        while Instant::now() < deadline {
            let logs = mgr.logs("api", 50).unwrap();
            if let Some(lines) = logs["lines"].as_array()
                && lines.iter().any(|l| {
                    l["line"]
                        .as_str()
                        .unwrap_or("")
                        .contains("SVC=api DB=127.0.0.1:15432")
                })
            {
                saw_output = true;
                break;
            }
            thread::sleep(Duration::from_millis(40));
        }
        assert!(
            saw_output,
            "expected captured stdout with injected OTEL & SERVICES env"
        );

        let list = mgr.list_resources();
        let resources = list["resources"].as_array().unwrap();
        let db_res = resources.iter().find(|r| r["name"] == "db").unwrap();
        assert_eq!(db_res["masked_env"]["DB_PASSWORD"], "••••••••");
        assert_eq!(db_res["masked_env"]["OTEL_SERVICE_NAME"], "db");

        mgr.stop_resource("api").unwrap();
        mgr.stop_resource("db").unwrap();
        let list_after = mgr.list_resources();
        let api_after = list_after["resources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == "api")
            .unwrap();
        assert_eq!(api_after["state"], "stopped");

        let _ = fs::remove_dir_all(&root);
    }
}
