//! Integration smoke: serve → status → ingest → issues.
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
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

/// Read a top-level string field out of a JSON response body.
///
/// Every id and path here goes through the parser rather than a `split('"')`: a Windows
/// `out_dir` is a verbatim path (`\\?\C:\…`), which the wire encodes with escaped
/// backslashes and a hand-rolled split would hand back still escaped.
fn json_str(body: &str, key: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(body).expect("response should be JSON");
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} should be a string in {body}"))
        .to_string()
}

/// Spawn a hub serving `root` and hand back the port it really owns.
///
/// `free_port` gives the port away before the child binds it, so two tests in this binary can
/// be handed the same one and the loser's hub then exits on a bind error. Talking to the winner
/// would keep the test green until that hub is killed, which resets this test's in-flight
/// request, so a start counts only once the child itself names this root, and a lost port is
/// retried with a fresh one.
fn start_hub(root: &Path) -> (u16, HubProc) {
    let marker = root.file_name().unwrap().to_string_lossy().into_owned();
    for _ in 0..10 {
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
        let mut hub = HubProc(child);
        if serves_root(&mut hub, port, &marker) {
            return (port, hub);
        }
    }
    panic!("no hub serves {marker}")
}

/// Poll `/api/v1/status` until this exact child reports our root, giving up if it dies first.
fn serves_root(hub: &mut HubProc, port: u16, marker: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if hub.0.try_wait().ok().flatten().is_some() {
            return false;
        }
        if try_get(port, "/api/v1/status").is_some_and(|body| body.contains(marker)) {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

/// A GET that reports failure instead of panicking, for probing a hub that may not be ours.
fn try_get(port: u16, path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .ok()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
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

    let (port, _hub) = start_hub(&root);

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
    let issue_id = json_str(&body, "issue_id");

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
    let root: PathBuf =
        std::env::temp_dir().join(format!("airbug-hub-runs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("dash/hub/data")).unwrap();
    std::fs::create_dir_all(root.join("dash/collector/data")).unwrap();

    let (port, _hub) = start_hub(&root);

    let (st, status_body) = http(port, "GET", "/api/v1/status", None);
    assert_eq!(st, 200, "{status_body}");
    let hub_id = json_str(&status_body, "hub_id");
    assert!(!hub_id.is_empty());

    let (st, status2) = http(port, "GET", "/api/v1/status", None);
    assert_eq!(st, 200);
    assert!(
        status2.contains(&hub_id),
        "hub_id should be stable: {status2}"
    );

    let (st, reg) = http(
        port,
        "POST",
        "/api/v1/bench/runs",
        Some(r#"{"title":"smoke","command":"test"}"#),
    );
    assert_eq!(st, 200, "{reg}");
    assert!(reg.contains(&hub_id), "{reg}");
    let run_id = json_str(&reg, "run_id");
    let out_dir = json_str(&reg, "out_dir");

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
    assert!(
        detail.contains("running") || detail.contains("\"completed\""),
        "{detail}"
    );
    assert!(
        detail.contains("\"completed\":2") || detail.contains("\"completed\": 2"),
        "{detail}"
    );

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
    std::fs::write(
        root.join("dash/collector/data/logs.json"),
        format!("{log_line}\n"),
    )
    .unwrap();

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
    assert!(
        issues.contains("run correlated boom") || issues.contains("ISSUE-"),
        "{issues}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn serve_unit_report_route_and_traversal_guard() {
    let root: PathBuf =
        std::env::temp_dir().join(format!("airbug-hub-report-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("dash/hub/data")).unwrap();
    std::fs::create_dir_all(root.join("target/airbug-report")).unwrap();
    std::fs::write(
        root.join("target/airbug-report/index.html"),
        "<html><body>airbug report</body></html>",
    )
    .unwrap();

    let (port, _hub) = start_hub(&root);

    let (st, body) = http(port, "GET", "/api/v1/status", None);
    assert_eq!(st, 200, "{body}");
    let status: serde_json::Value = serde_json::from_str(&body).expect("status JSON");
    let report_api = status["apis"]
        .as_array()
        .and_then(|apis| {
            apis.iter()
                .find(|api| api["href"] == format!("http://127.0.0.1:{port}/report/index.html"))
        })
        .expect("unit report endpoint in status");
    assert_eq!(report_api["method"], "GET");
    assert_eq!(report_api["available"], true);

    let (st, body) = http(port, "GET", "/report/index.html", None);
    assert_eq!(st, 200, "{body}");
    assert!(body.contains("airbug report"), "{body}");

    let (st, body) = http(port, "GET", "/report/../secret", None);
    assert_eq!(st, 404, "{body}");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn serve_unified_event_archive_accepts_trace() {
    let root: PathBuf =
        std::env::temp_dir().join(format!("airbug-hub-events-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("dash/hub/data")).unwrap();

    let (port, _hub) = start_hub(&root);

    let event = r#"{
      "model_version": 1,
      "event_id": "trace-1",
      "timestamp": "2026-01-01T00:00:00Z",
      "service": "checkout",
      "environment": "test",
      "release": null,
      "trace_id": "abc",
      "span_id": "def",
      "tags": {},
      "payload": {
        "signal": "trace",
        "data": {
          "trace_id": "abc",
          "span_id": "def",
          "parent_span_id": null,
          "name": "checkout",
          "attributes": {"http.method": "GET"}
        }
      }
    }"#;
    let (st, body) = http(port, "POST", "/api/v1/events", Some(event));
    assert_eq!(st, 202, "{body}");
    assert!(body.contains("\"trace\""), "{body}");

    let archived = std::fs::read_to_string(root.join("dash/hub/data/events.jsonl")).unwrap();
    assert!(archived.contains("\"event_id\":\"trace-1\""), "{archived}");
    assert!(archived.contains("\"signal\":\"trace\""), "{archived}");

    let _ = std::fs::remove_dir_all(&root);
}
