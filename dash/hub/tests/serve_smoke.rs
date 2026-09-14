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

    let event1 = r#"{
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
    let (st, body) = http(port, "POST", "/api/v1/errors", Some(event1));
    assert_eq!(st, 200, "{body}");
    assert!(
        body.contains("\"ok\": true") || body.contains("\"ok\":true"),
        "{body}"
    );
    let issue_id = body
        .split("\"issue_id\"")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .expect("issue_id")
        .to_string();

    let event2 = event1
        .replace("it-1", "it-2")
        .replace("2026-01-01T00:00:00.000Z", "2026-01-01T00:00:01.000Z");
    let (st, body) = http(port, "POST", "/api/v1/errors", Some(&event2));
    assert_eq!(st, 200, "{body}");

    let (st, body) = http(port, "GET", &format!("/api/v1/issues/{issue_id}"), None);
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("\"events\""), "{body}");
    assert!(body.contains("it-2"), "{body}");

    let (st, body) = http(
        port,
        "POST",
        &format!("/api/v1/issues/{issue_id}/resolve"),
        None,
    );
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("resolved"), "{body}");

    let (st, body) = http(
        port,
        "POST",
        &format!("/api/v1/issues/{issue_id}/ignore"),
        None,
    );
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("ignored"), "{body}");

    let (st, body) = http(
        port,
        "POST",
        &format!("/api/v1/issues/{issue_id}/reopen"),
        None,
    );
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("unresolved"), "{body}");

    let (st, _) = http(port, "POST", "/api/v1/issues/ISSUE-999/resolve", None);
    assert_eq!(st, 404);

    let (st, body) = http(
        port,
        "POST",
        &format!("/api/v1/issues/{issue_id}/nope"),
        None,
    );
    assert_eq!(st, 400, "{body}");

    let (st, body) = http(port, "GET", "/api/v1/issues", None);
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("ISSUE-"), "{body}");

    // Legacy alias still works.
    let (st, _) = http(port, "GET", "/api/status", None);
    assert_eq!(st, 200);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn serve_bench_runs_progress_and_run_id_filter() {
    let root: PathBuf = std::env::temp_dir().join(format!("airbug-hub-runs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("dash/hub/data")).unwrap();
    std::fs::create_dir_all(root.join("dash/collector/data")).unwrap();

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

    let (st, status_body) = http(port, "GET", "/api/v1/status", None);
    assert_eq!(st, 200, "{status_body}");
    let hub_id = status_body
        .split("\"hub_id\"")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .expect("hub_id")
        .to_string();
    assert!(!hub_id.is_empty());

    let (st, status2) = http(port, "GET", "/api/v1/status", None);
    assert_eq!(st, 200);
    assert!(status2.contains(&hub_id), "hub_id should be stable: {status2}");

    let (st, reg) = http(
        port,
        "POST",
        "/api/v1/bench/runs",
        Some(r#"{"title":"smoke","command":"test"}"#),
    );
    assert_eq!(st, 200, "{reg}");
    assert!(reg.contains(&hub_id), "{reg}");
    let run_id = reg
        .split("\"run_id\"")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .expect("run_id")
        .to_string();
    let out_dir = reg
        .split("\"out_dir\"")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .expect("out_dir")
        .to_string();

    let (st, list) = http(port, "GET", "/api/v1/bench/runs", None);
    assert_eq!(st, 200, "{list}");
    assert!(list.contains(&run_id), "{list}");

    std::fs::write(
        PathBuf::from(&out_dir).join("progress.json"),
        r#"{"state":"running","completed":2,"total":5,"variant":"quick"}"#,
    )
    .unwrap();

    let (st, detail) = http(port, "GET", &format!("/api/v1/bench/runs/{run_id}"), None);
    assert_eq!(st, 200, "{detail}");
    assert!(detail.contains("running") || detail.contains("\"completed\""), "{detail}");
    assert!(detail.contains("\"completed\":2") || detail.contains("\"completed\": 2"), "{detail}");

    std::fs::write(
        PathBuf::from(&out_dir).join("status-final.json"),
        r#"{"state":"complete"}"#,
    )
    .unwrap();
    let (st, detail) = http(port, "GET", &format!("/api/v1/bench/runs/{run_id}"), None);
    assert_eq!(st, 200, "{detail}");
    assert!(detail.contains("complete"), "{detail}");

    let log_line = format!(
        r#"{{"time":"2026-01-01T00:00:00Z","severity":"INFO","body":"bench log","service":"smoke","airbug.run_id":"{run_id}"}}"#
    );
    std::fs::write(root.join("dash/collector/data/logs.json"), format!("{log_line}\n")).unwrap();

    let (st, logs) = http(
        port,
        "GET",
        &format!("/api/v1/logs?limit=20&run_id={run_id}"),
        None,
    );
    assert_eq!(st, 200, "{logs}");
    assert!(logs.contains("bench log"), "{logs}");

    let (st, logs_other) = http(
        port,
        "GET",
        "/api/v1/logs?limit=20&run_id=00000000-0000-0000-0000-000000000000",
        None,
    );
    assert_eq!(st, 200, "{logs_other}");
    assert!(!logs_other.contains("bench log"), "{logs_other}");

    let event = format!(
        r#"{{
      "schema_version": 1,
      "event_id": "run-err-1",
      "timestamp": "2026-01-01T00:00:00.000Z",
      "level": "error",
      "message": "run correlated boom",
      "fingerprint": ["run:fp"],
      "breadcrumbs": [],
      "tags": {{"airbug.run_id": "{run_id}"}},
      "extra": {{}},
      "contexts": {{}}
    }}"#
    );
    let (st, body) = http(port, "POST", "/api/v1/errors", Some(&event));
    assert_eq!(st, 200, "{body}");

    let (st, issues) = http(
        port,
        "GET",
        &format!("/api/v1/issues?run_id={run_id}"),
        None,
    );
    assert_eq!(st, 200, "{issues}");
    assert!(issues.contains("run correlated boom") || issues.contains("ISSUE-"), "{issues}");

    let _ = std::fs::remove_dir_all(&root);
}
