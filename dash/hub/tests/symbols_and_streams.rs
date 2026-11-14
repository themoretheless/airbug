use airbug_err::{
    DebugImage, EVENT_SCHEMA_VERSION, Event, Exception, Frame, Severity, Stacktrace, User,
};
use airbug_hub::app::HubApp;
use airbug_hub::serve::serve_listener;
use airbug_hub::stream::websocket_accept_key;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "airbug-hub-test-{}-{}-{:?}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn spawn_hub(dir: &std::path::Path) -> (Arc<HubApp>, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = Arc::new(HubApp::new(dir, port, None).unwrap());
    let app_srv = Arc::clone(&app);
    thread::spawn(move || {
        let _ = serve_listener(listener, app_srv);
    });
    (app, port)
}

fn sample_unsymbolicated_event(email: Option<&str>) -> Event {
    Event {
        schema_version: EVENT_SCHEMA_VERSION,
        event_id: format!(
            "evt-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ),
        timestamp: "2026-11-14T12:00:00Z".into(),
        level: Severity::Error,
        message: Some("charge failed for alice@example.com with Bearer secret_tok_999".into()),
        release: Some("0.11.0".into()),
        environment: Some("prod".into()),
        service: Some("billing".into()),
        fingerprint: vec!["{{ default }}".into()],
        exception: Some(Exception {
            ty: "panic".into(),
            value: "panic at checkout".into(),
            stacktrace: Some(Stacktrace {
                frames: vec![Frame {
                    function: "<unsymbolicated>".into(),
                    filename: None,
                    lineno: None,
                    colno: None,
                    in_app: false,
                    instruction_addr: Some("0x401040".into()),
                    symbol_addr: None,
                }],
            }),
        }),
        breadcrumbs: vec![],
        tags: BTreeMap::from([("password".into(), "hunter2".into())]),
        user: email.map(|em| User {
            id: Some("u1".into()),
            email: Some(em.into()),
            username: Some("alice".into()),
        }),
        extra: BTreeMap::new(),
        contexts: BTreeMap::new(),
        trace_id: None,
        span_id: None,
        debug_images: vec![DebugImage {
            debug_id: Some("ABC123DEF456".into()),
            code_id: None,
            debug_file: Some("billing.bin".into()),
            image_addr: Some("0x400000".into()),
            image_size: Some(0x10000),
            arch: Some("x86_64".into()),
        }],
    }
}

fn read_ws_frame(stream: &mut TcpStream) -> (u8, Vec<u8>) {
    let mut hdr = [0u8; 2];
    stream.read_exact(&mut hdr).unwrap();
    let opcode = hdr[0] & 0x0F;
    let len_byte = hdr[1] & 0x7F;
    let len = if len_byte <= 125 {
        len_byte as usize
    } else if len_byte == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).unwrap();
        u16::from_be_bytes(ext) as usize
    } else {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).unwrap();
        u64::from_be_bytes(ext) as usize
    };
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).unwrap();
    (opcode, payload)
}

#[test]
fn symbol_server_pii_scrub_and_upstream_relay() {
    let tmp = TempDir::new("symbols-relay");

    // Start mock upstream relay server.
    let upstream_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let upstream_port = upstream_listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        if let Ok((mut s, _)) = upstream_listener.accept() {
            let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
            let mut buf = [0u8; 8192];
            let mut acc = Vec::new();
            while let Ok(n) = s.read(&mut buf) {
                if n == 0 {
                    break;
                }
                acc.extend_from_slice(&buf[..n]);
                if let Some(pos) = acc.windows(4).position(|w| w == b"\r\n\r\n") {
                    let body = String::from_utf8_lossy(&acc[pos + 4..]).to_string();
                    if !body.is_empty() {
                        let _ = tx.send(body);
                        let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
                        break;
                    }
                }
            }
        }
    });

    let (app, port) = spawn_hub(&tmp.0);
    app.set_scrub_pii(true);
    app.set_upstream(Some(format!(
        "http://127.0.0.1:{upstream_port}/api/v1/errors"
    )))
    .unwrap();

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let base = format!("http://127.0.0.1:{port}");

    // Upload Breakpad .sym module containing 0x1040 -> billing::process_payment at line 42.
    let breakpad = "\
MODULE Linux x86_64 ABC123DEF456 billing.bin
FILE 0 src/billing.rs
FUNC 1000 100 0 billing::process_payment
1040 10 42 0
";
    let up_resp = client
        .post(format!("{base}/api/v1/debug-images"))
        .header("content-type", "text/plain")
        .body(breakpad)
        .send()
        .unwrap();
    assert_eq!(up_resp.status().as_u16(), 200);
    let up_json: Value = up_resp.json().unwrap();
    assert_eq!(up_json["debug_id"], "ABC123DEF456");
    assert_eq!(up_json["symbol_count"], 1);

    let list_json: Value = client
        .get(format!("{base}/api/v1/debug-images"))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(list_json["images"].as_array().unwrap().len(), 1);

    // Direct symbolication endpoint test.
    let ev = sample_unsymbolicated_event(Some("alice@example.com"));
    let sym_resp: Value = client
        .post(format!("{base}/api/v1/symbolicate"))
        .json(&ev)
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(sym_resp["resolved_frames"], 1);
    let sym_frame = &sym_resp["event"]["exception"]["stacktrace"]["frames"][0];
    assert_eq!(sym_frame["function"], "billing::process_payment");
    assert_eq!(sym_frame["filename"], "src/billing.rs");
    assert_eq!(sym_frame["lineno"], 42);
    assert_eq!(sym_frame["symbol_addr"], "0x401000");

    // Ingest with PII scrubbing + symbolication + upstream relay.
    let ing_resp = client
        .post(format!("{base}/api/v1/errors"))
        .json(&ev)
        .send()
        .unwrap();
    assert_eq!(ing_resp.status().as_u16(), 200);
    let ing_json: Value = ing_resp.json().unwrap();
    let issue_id = ing_json["issue_id"].as_str().unwrap().to_string();

    let issue_detail: Value = client
        .get(format!("{base}/api/v1/issues/{issue_id}"))
        .send()
        .unwrap()
        .json()
        .unwrap();
    let stored_frame = &issue_detail["last_event"]["exception"]["stacktrace"]["frames"][0];
    assert_eq!(stored_frame["function"], "billing::process_payment");
    let stored_msg = issue_detail["last_event"]["message"].as_str().unwrap();
    assert!(!stored_msg.contains("alice@example.com"), "{stored_msg}");
    assert!(!stored_msg.contains("secret_tok_999"), "{stored_msg}");
    assert_eq!(issue_detail["last_event"]["tags"]["password"], "[REDACTED]");
    assert_eq!(issue_detail["last_event"]["user"]["email"], "[REDACTED]");

    let relayed_raw = rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let relayed: Value = serde_json::from_str(&relayed_raw).unwrap();
    assert_eq!(
        relayed["exception"]["stacktrace"]["frames"][0]["function"],
        "billing::process_payment"
    );
    assert!(
        !relayed["message"]
            .as_str()
            .unwrap()
            .contains("alice@example.com")
    );
}

#[test]
fn streaming_endpoints_ws_sse_fragments_graphql_and_jsonl() {
    let tmp = TempDir::new("streams");
    let (_app, port) = spawn_hub(&tmp.0);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let base = format!("http://127.0.0.1:{port}");

    // Create a registered bench run and progress snapshot so progress.jsonl & fragments/runs have data.
    let create_resp: Value = client
        .post(format!("{base}/api/v1/bench/runs"))
        .json(&json!({ "title": "Demo" }))
        .send()
        .unwrap()
        .json()
        .unwrap();
    let created_run_id = create_resp["run_id"].as_str().unwrap().to_string();
    let run_dir = tmp.0.join(".airbug-bench/runs").join(&created_run_id);
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::write(
        run_dir.join("progress.json"),
        serde_json::to_vec(&json!({ "done": 3, "total": 10 })).unwrap(),
    )
    .unwrap();

    // 1. WebSocket handshake + hello + live error event frame (`?once=2`).
    let mut ws = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(3),
    )
    .unwrap();
    ws.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    write!(
        ws,
        "GET /api/v1/ws?once=2 HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    )
    .unwrap();
    ws.flush().unwrap();

    let mut hdr_buf = Vec::new();
    let mut byte = [0u8; 1];
    while !hdr_buf.windows(4).any(|w| w == b"\r\n\r\n") {
        ws.read_exact(&mut byte).unwrap();
        hdr_buf.push(byte[0]);
    }
    let hdr_str = String::from_utf8_lossy(&hdr_buf);
    assert!(hdr_str.starts_with("HTTP/1.1 101"), "{hdr_str}");
    let expected_accept = websocket_accept_key(key);
    assert!(hdr_str.contains(&expected_accept), "{hdr_str}");

    let (op1, f1) = read_ws_frame(&mut ws);
    assert_eq!(op1, 0x1);
    let v1: Value = serde_json::from_slice(&f1).unwrap();
    assert_eq!(v1["kind"], "hello");

    // Trigger an error ingest so the open WebSocket receives the live second frame.
    let ev = sample_unsymbolicated_event(None);
    assert_eq!(
        client
            .post(format!("{base}/api/v1/errors"))
            .json(&ev)
            .send()
            .unwrap()
            .status()
            .as_u16(),
        200
    );

    let (op2, f2) = read_ws_frame(&mut ws);
    assert_eq!(op2, 0x1);
    let v2: Value = serde_json::from_slice(&f2).unwrap();
    assert_eq!(v2["kind"], "error");

    // 2. SSE progressive endpoint (`?once=1`).
    let sse_resp = client
        .get(format!("{base}/api/v1/stream?once=1"))
        .send()
        .unwrap();
    assert_eq!(sse_resp.status().as_u16(), 200);
    assert!(
        sse_resp.headers()["content-type"]
            .to_str()
            .unwrap()
            .contains("text/event-stream")
    );
    let sse_body = sse_resp.text().unwrap();
    assert!(sse_body.contains("event: hello"), "{sse_body}");

    // 3. HTML result fragments with preload Link header.
    let frag_issues = client
        .get(format!("{base}/api/v1/fragments/issues"))
        .send()
        .unwrap();
    assert_eq!(frag_issues.status().as_u16(), 200);
    assert!(frag_issues.headers().contains_key("link"));
    let html_issues = frag_issues.text().unwrap();
    assert!(
        html_issues.contains("data-fragment=\"issues\""),
        "{html_issues}"
    );
    assert!(html_issues.contains("<tr data-issue-id="), "{html_issues}");

    let frag_runs = client
        .get(format!("{base}/api/v1/fragments/runs"))
        .send()
        .unwrap();
    assert_eq!(frag_runs.status().as_u16(), 200);
    let html_runs = frag_runs.text().unwrap();
    assert!(html_runs.contains("data-fragment=\"runs\""), "{html_runs}");

    // 4. GraphQL query endpoint.
    let gql_resp: Value = client
        .post(format!("{base}/api/v1/graphql"))
        .json(&json!({ "query": "query { status issues streamSnapshot }" }))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert!(gql_resp["data"]["status"]["hub_id"].is_string());
    assert!(
        !gql_resp["data"]["issues"]["issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(gql_resp["data"]["streamSnapshot"]["events"].is_array());

    // 5. JSON Lines batched progress feed.
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let jsonl = client
            .get(format!(
                "{base}/api/v1/runs/{created_run_id}/progress.jsonl"
            ))
            .send()
            .unwrap();
        if jsonl.status().as_u16() == 200 {
            let ct = jsonl.headers()["content-type"]
                .to_str()
                .unwrap()
                .to_string();
            assert!(ct.contains("application/x-ndjson"), "{ct}");
            let body = jsonl.text().unwrap();
            let first: Value = serde_json::from_str(body.lines().next().unwrap()).unwrap();
            assert!(first.is_object());
            break;
        }
        assert!(Instant::now() < deadline, "progress.jsonl timed out");
        thread::sleep(Duration::from_millis(50));
    }
}
