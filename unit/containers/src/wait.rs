//! Wait strategies (ready checks after `docker start`).
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    thread,
    time::{Duration, Instant},
};

use crate::docker::{ContainerError, Runtime};

/// When a container is considered ready for tests.
#[derive(Clone, Debug)]
pub enum Wait {
    /// Container process is running (default).
    Running,
    /// Substring appears in combined stdout/stderr logs.
    Message {
        /// Text that must appear in logs.
        text: String,
        /// Look only at stdout when true; otherwise both streams.
        stdout_only: bool,
    },
    /// TCP connect to a mapped container port succeeds.
    Tcp {
        /// Container port (not host).
        container_port: u16,
    },
    /// HTTP GET to an absolute URL returns a 2xx status.
    Http {
        /// Full URL, for example `http://127.0.0.1:8080/health`.
        url: String,
    },
    /// HTTP GET returns 2xx and the body contains `expect` (simple ready probe).
    HttpJson {
        /// Full URL.
        url: String,
        /// Reserved JSON-path hint (matched as a substring today).
        path: String,
        /// Required body substring.
        expect: String,
    },
}

impl Wait {
    /// Wait until logs contain `text` on stdout.
    pub fn message_on_stdout(text: impl Into<String>) -> Self {
        Self::Message {
            text: text.into(),
            stdout_only: true,
        }
    }

    /// Wait until logs contain `text` on stdout or stderr.
    pub fn message(text: impl Into<String>) -> Self {
        Self::Message {
            text: text.into(),
            stdout_only: false,
        }
    }

    /// Wait until the mapped public port accepts TCP connections.
    pub fn tcp_port(container_port: u16) -> Self {
        Self::Tcp { container_port }
    }

    /// Wait until `GET url` returns HTTP 2xx.
    pub fn http(url: impl Into<String>) -> Self {
        Self::Http { url: url.into() }
    }

    /// Wait until `GET url` returns HTTP 2xx and the body contains `expect`.
    ///
    /// `path` is accepted for API symmetry with JSON-path probes; it is also
    /// required as a body substring when non-empty.
    pub fn http_json(
        url: impl Into<String>,
        path: impl Into<String>,
        expect: impl Into<String>,
    ) -> Self {
        Self::HttpJson {
            url: url.into(),
            path: path.into(),
            expect: expect.into(),
        }
    }

    /// Poll an HTTP wait condition without a container (for [`Wait::http`] /
    /// [`Wait::http_json`]).
    pub fn poll(&self, timeout: Duration) -> Result<(), ContainerError> {
        match self {
            Self::Http { url } => poll_until(timeout, || http_ready(url, None, None)),
            Self::HttpJson { url, path, expect } => {
                let path = if path.is_empty() {
                    None
                } else {
                    Some(path.as_str())
                };
                poll_until(timeout, || http_ready(url, path, Some(expect.as_str())))
            }
            _ => Err(ContainerError::Failed(
                "Wait::poll supports only http / http_json variants".into(),
            )),
        }
    }

    pub(crate) fn block_until(
        &self,
        runtime: &Runtime,
        id: &str,
        host: &str,
        mapped: &dyn Fn(u16) -> Result<u16, ContainerError>,
        timeout: Duration,
    ) -> Result<(), ContainerError> {
        let deadline = Instant::now() + timeout;
        match self {
            Self::Running => {
                while Instant::now() < deadline {
                    if runtime.is_running(id)? {
                        return Ok(());
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(ContainerError::Timeout(
                    "container did not reach running state".into(),
                ))
            }
            Self::Message { text, stdout_only } => {
                while Instant::now() < deadline {
                    let logs = runtime.logs(id, *stdout_only)?;
                    if logs.contains(text) {
                        return Ok(());
                    }
                    if !runtime.is_running(id)? {
                        return Err(ContainerError::Failed(format!(
                            "container exited before wait message `{text}` appeared"
                        )));
                    }
                    thread::sleep(Duration::from_millis(150));
                }
                Err(ContainerError::Timeout(format!(
                    "timed out waiting for log message `{text}`"
                )))
            }
            Self::Tcp { container_port } => {
                let port = mapped(*container_port)?;
                let addr = format!("{host}:{port}");
                while Instant::now() < deadline {
                    if try_connect(&addr) {
                        return Ok(());
                    }
                    if !runtime.is_running(id)? {
                        return Err(ContainerError::Failed(
                            "container exited before TCP port became ready".into(),
                        ));
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(ContainerError::Timeout(format!(
                    "timed out waiting for TCP {addr}"
                )))
            }
            Self::Http { url } => {
                while Instant::now() < deadline {
                    if http_ready(url, None, None) {
                        return Ok(());
                    }
                    if !runtime.is_running(id)? {
                        return Err(ContainerError::Failed(
                            "container exited before HTTP became ready".into(),
                        ));
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(ContainerError::Timeout(format!(
                    "timed out waiting for HTTP 2xx from {url}"
                )))
            }
            Self::HttpJson { url, path, expect } => {
                let path = if path.is_empty() {
                    None
                } else {
                    Some(path.as_str())
                };
                while Instant::now() < deadline {
                    if http_ready(url, path, Some(expect.as_str())) {
                        return Ok(());
                    }
                    if !runtime.is_running(id)? {
                        return Err(ContainerError::Failed(
                            "container exited before HTTP JSON probe succeeded".into(),
                        ));
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(ContainerError::Timeout(format!(
                    "timed out waiting for HTTP JSON probe on {url}"
                )))
            }
        }
    }
}

fn poll_until(timeout: Duration, ready: impl Fn() -> bool) -> Result<(), ContainerError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if ready() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(ContainerError::Timeout(
        "timed out waiting for HTTP ready probe".into(),
    ))
}

fn try_connect(addr: &str) -> bool {
    let Ok(mut addrs) = addr.to_socket_addrs() else {
        return false;
    };
    let Some(sock) = addrs.next() else {
        return false;
    };
    TcpStream::connect_timeout(&sock, Duration::from_millis(200)).is_ok()
}

fn http_ready(url: &str, path_needle: Option<&str>, expect: Option<&str>) -> bool {
    match http_get(url) {
        Some((status, body)) if (200..300).contains(&status) => {
            if let Some(expect) = expect
                && !body.contains(expect)
            {
                return false;
            }
            if let Some(path) = path_needle
                && !body.contains(path)
            {
                return false;
            }
            true
        }
        _ => false,
    }
}

/// Minimal HTTP/1.1 GET over TCP. Supports `http://host[:port]/path`.
fn http_get(url: &str) -> Option<(u16, String)> {
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = match rest.split_once('/') {
        Some((hostport, path)) => (hostport, format!("/{path}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse().ok()?),
        None => (authority, 80u16),
    };
    let addr = format!("{host}:{port}");
    let mut addrs = addr.to_socket_addrs().ok()?;
    let sock = addrs.next()?;
    let mut stream = TcpStream::connect_timeout(&sock, Duration::from_millis(400)).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(400)));
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).ok()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    let status_line = text.lines().next()?;
    let mut parts = status_line.split_whitespace();
    let _ = parts.next()?; // HTTP/1.x
    let status: u16 = parts.next()?.parse().ok()?;
    let body = match text.split_once("\r\n\r\n") {
        Some((_, body)) => body.to_string(),
        None => match text.split_once("\n\n") {
            Some((_, body)) => body.to_string(),
            None => String::new(),
        },
    };
    Some((status, body))
}
