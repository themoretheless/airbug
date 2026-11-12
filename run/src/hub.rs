//! Where a run can be watched: is the hub for this root up, and what is the run's address.
//!
//! Whatever writes a run prints the link itself, so nobody has to guess a port or a path.
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::Path,
    time::Duration,
};

/// The hub's fixed port; `AIRBUG_HUB` (`http://127.0.0.1:<port>`) points elsewhere.
pub const DEFAULT_PORT: u16 = 8790;

pub enum Probe {
    /// A hub serving this root is listening.
    Ours,
    /// A hub is listening, but for another root (its path).
    Other(String),
    Absent,
}

/// The port named by `AIRBUG_HUB`, or the default.
pub fn port() -> u16 {
    std::env::var("AIRBUG_HUB")
        .ok()
        .and_then(|url| {
            let tail = url.trim_end_matches('/').rsplit(':').next()?.to_string();
            tail.parse().ok()
        })
        .unwrap_or(DEFAULT_PORT)
}

/// The run page of `run_id` on a local hub.
pub fn run_url(port: u16, run_id: &str) -> String {
    format!("http://127.0.0.1:{port}/#/run/{run_id}")
}

/// Is a hub for `root` listening on `port`?
pub fn probe(port: u16, root: &Path) -> Probe {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(300)) else {
        return Probe::Absent;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let request = "GET /api/v1/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    let mut text = String::new();
    if stream.write_all(request.as_bytes()).is_err() || stream.read_to_string(&mut text).is_err() {
        return Probe::Absent;
    }
    let body = text.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");
    let hub_root = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("root").and_then(|r| r.as_str()).map(str::to_string));
    let same = |a: &Path, b: &Path| a == b || a.canonicalize().ok() == b.canonicalize().ok();
    match hub_root {
        Some(hub_root) if same(Path::new(&hub_root), root) => Probe::Ours,
        Some(hub_root) => Probe::Other(hub_root),
        None => Probe::Absent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closed_port_is_absent_and_the_address_is_the_run_page() {
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = free.local_addr().unwrap().port();
        drop(free);
        assert!(matches!(probe(port, Path::new(".")), Probe::Absent));
        assert_eq!(run_url(8790, "abc"), "http://127.0.0.1:8790/#/run/abc");
    }
}
