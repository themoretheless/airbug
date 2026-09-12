//! Wait strategies (ready checks after `docker start`).
use std::{
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
                Err(ContainerError::Timeout("container did not reach running state".into()))
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
        }
    }
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
