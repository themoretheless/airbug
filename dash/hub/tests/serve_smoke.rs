//! Integration smoke: serve → status → ingest → issues.
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn hub_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_airbug-hub"));
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    cmd
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn wait_ready(port: u16, deadline: Instant) {
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("hub did not become ready on {port}");
}

fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let body_bytes = body.unwrap_or("").as_bytes();
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n");
    if body.is_some() {
        req.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body_bytes.len()
        ));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(body_bytes).unwrap();
    }
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();
    let text = String::from_utf8_lossy(&buf);
    let status = text
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = text.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status, body)
}

struct HubProc(Child);

impl Drop for HubProc {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn serve_status_ingest_issues() {
    let root: PathBuf = std::env::temp_dir().join(format!("airbug-hub-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("dash/hub/data")).unwrap();

    let port = free_port();
    let child = hub_bin()
        .args([
            "serve",
            "--root",
            root.to_str().unwrap(),
            "--port",
            &port.to_string(),
        ])
        .spawn()
        .expect("spawn hub");
    let _hub = HubProc(child);
    wait_ready(port, Instant::now() + Duration::from_secs(10));

    let (st, body) = http(port, "GET", "/api/v1/status", None);
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("\"domains\""), "{body}");

    let event = r#"{
      "schema_version": 1,
      "event_id": "it-1",
      "timestamp": "2026-01-01T00:00:00.000Z",
      "level": "error",
      "message": "integration boom",
      "fingerprint": ["it:fp"],
      "breadcrumbs": [],
      "tags": {},
      "extra": {},
      "contexts": {}
    }"#;
    let (st, body) = http(port, "POST", "/api/v1/errors", Some(event));
    assert_eq!(st, 200, "{body}");
    assert!(
        body.contains("\"ok\": true") || body.contains("\"ok\":true"),
        "{body}"
    );

    let (st, body) = http(port, "GET", "/api/v1/issues", None);
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("ISSUE-"), "{body}");

    // Legacy alias still works.
    let (st, _) = http(port, "GET", "/api/status", None);
    assert_eq!(st, 200);

    let _ = std::fs::remove_dir_all(&root);
}
