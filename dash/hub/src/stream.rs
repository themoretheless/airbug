//! Live streaming transports (`#201–206`):
//! - Bounded flow-controlled pub/sub (`StreamHub`, `#205`)
//! - RFC 6455 WebSocket (`GET /api/v1/ws`, `#201`)
//! - Server-Sent Events (`GET /api/v1/stream`, `#202`)
//! - HTML result fragments with HTTP/2-friendly `Link` preload headers (`GET /api/v1/fragments/*`, `#203`)
//! - Read-only GraphQL endpoint (`POST /api/v1/graphql`, `#204`)
//! - JSON Lines progress feed (`GET /api/v1/runs/<id>/progress.jsonl`, `#206`)

use crate::app::HubApp;
use crate::http::{self, Request};
use crate::{scan, store};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, mpsc};
use std::time::Duration;

const DEFAULT_SUBSCRIBER_CAPACITY: usize = 128;
const RECENT_HISTORY_LIMIT: usize = 200;
const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub seq: u64,
    pub kind: String,
    pub payload: Value,
}

/// Flow-controlled bounded broadcast broker for live hub updates.
pub struct StreamHub {
    seq: AtomicU64,
    dropped_total: AtomicU64,
    recent: Mutex<VecDeque<StreamEvent>>,
    subscribers: Mutex<Vec<mpsc::SyncSender<StreamEvent>>>,
}

impl Default for StreamHub {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamHub {
    pub fn new() -> Self {
        Self {
            seq: AtomicU64::new(1),
            dropped_total: AtomicU64::new(0),
            recent: Mutex::new(VecDeque::with_capacity(RECENT_HISTORY_LIMIT)),
            subscribers: Mutex::new(Vec::new()),
        }
    }

    pub fn publish(&self, kind: &str, payload: Value) {
        let ev = StreamEvent {
            seq: self.seq.fetch_add(1, Ordering::Relaxed),
            kind: kind.to_string(),
            payload,
        };
        if let Ok(mut hist) = self.recent.lock() {
            if hist.len() >= RECENT_HISTORY_LIMIT {
                hist.pop_front();
            }
            hist.push_back(ev.clone());
        }
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.retain(|tx| match tx.try_send(ev.clone()) {
                Ok(()) => true,
                Err(mpsc::TrySendError::Full(_)) => {
                    self.dropped_total.fetch_add(1, Ordering::Relaxed);
                    true
                }
                Err(mpsc::TrySendError::Disconnected(_)) => false,
            });
        }
    }

    pub fn subscribe(&self) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::sync_channel(DEFAULT_SUBSCRIBER_CAPACITY);
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        }
        rx
    }

    pub fn snapshot(&self, limit: usize) -> Value {
        let recent: Vec<StreamEvent> = self
            .recent
            .lock()
            .map(|h| {
                h.iter()
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect()
            })
            .unwrap_or_default();
        let active_subscribers = self.subscribers.lock().map(|s| s.len()).unwrap_or(0);
        json!({
            "next_seq": self.seq.load(Ordering::Relaxed),
            "dropped_total": self.dropped_total.load(Ordering::Relaxed),
            "active_subscribers": active_subscribers,
            "events": recent,
        })
    }
}

/// Pure-Rust RFC 3174 SHA-1 digest used solely for RFC 6455 `Sec-WebSocket-Accept`.
pub fn sha1_digest(input: &[u8]) -> [u8; 20] {
    let mut msg = input.to_vec();
    let bit_len = (input.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    for chunk in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 80];
        for (i, &word) in chunk.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(word);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

pub fn websocket_accept_key(client_key: &str) -> String {
    let combined = format!("{}{WS_GUID}", client_key.trim());
    let digest = sha1_digest(combined.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(digest)
}

fn write_ws_frame(stream: &mut TcpStream, opcode: u8, payload: &[u8]) -> std::io::Result<()> {
    let mut header = Vec::with_capacity(10);
    header.push(0x80 | (opcode & 0x0F));
    let len = payload.len();
    if len <= 125 {
        header.push(len as u8);
    } else if len <= 65535 {
        header.push(126);
        header.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        header.push(127);
        header.extend_from_slice(&(len as u64).to_be_bytes());
    }
    stream.write_all(&header)?;
    stream.write_all(payload)?;
    stream.flush()
}

/// `#201` WebSocket progress/event live stream endpoint (`GET /api/v1/ws`).
pub fn handle_ws(stream: &mut TcpStream, app: &HubApp, req: &Request) -> std::io::Result<()> {
    let Some(key) = req.header("sec-websocket-key") else {
        return http::respond(
            stream,
            "400 Bad Request",
            "text/plain; charset=utf-8",
            "missing Sec-WebSocket-Key header",
        );
    };
    let accept = websocket_accept_key(key);
    write!(
        stream,
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
    )?;
    stream.flush()?;

    let once_limit = http::query_usize(&req.query, "once", 0);
    let rx = app.stream.subscribe();
    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));

    let hello = StreamEvent {
        seq: 0,
        kind: "hello".into(),
        payload: json!({
            "hub_id": app.hub_id,
            "dropped_total": app.stream.dropped_total.load(Ordering::Relaxed),
        }),
    };
    let hello_json = serde_json::to_vec(&hello).unwrap_or_default();
    write_ws_frame(stream, 0x1, &hello_json)?;

    let mut sent = 1usize;
    if once_limit > 0 && sent >= once_limit {
        let _ = write_ws_frame(stream, 0x8, &[]);
        return Ok(());
    }

    loop {
        match rx.recv_timeout(Duration::from_millis(150)) {
            Ok(ev) => {
                let frame_bytes = serde_json::to_vec(&ev).unwrap_or_default();
                if write_ws_frame(stream, 0x1, &frame_bytes).is_err() {
                    break;
                }
                sent += 1;
                if once_limit > 0 && sent >= once_limit {
                    let _ = write_ws_frame(stream, 0x8, &[]);
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let mut peek = [0u8; 2];
                match stream.read(&mut peek) {
                    Ok(0) => break,
                    Ok(n) => {
                        let opcode = peek[0] & 0x0F;
                        if opcode == 0x8 {
                            let _ = write_ws_frame(stream, 0x8, &[]);
                            break;
                        }
                        if opcode == 0x9 {
                            let _ = write_ws_frame(stream, 0xA, &[]);
                        }
                        let _ = n;
                    }
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => break,
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

/// `#202` Server-Sent Events progressive updates (`GET /api/v1/stream`).
pub fn handle_sse(stream: &mut TcpStream, app: &HubApp, req: &Request) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nCache-Control: no-cache\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n"
    )?;
    let hello = json!({
        "hub_id": app.hub_id,
        "dropped_total": app.stream.dropped_total.load(Ordering::Relaxed),
    });
    write!(stream, "id: 0\nevent: hello\ndata: {hello}\n\n")?;
    stream.flush()?;

    let once_limit = http::query_usize(&req.query, "once", 0);
    if once_limit == 1 {
        return Ok(());
    }

    let rx = app.stream.subscribe();
    let mut sent = 1usize;
    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(ev) => {
                let data = serde_json::to_string(&ev.payload).unwrap_or_default();
                if write!(
                    stream,
                    "id: {}\nevent: {}\ndata: {data}\n\n",
                    ev.seq, ev.kind
                )
                .is_err()
                    || stream.flush().is_err()
                {
                    break;
                }
                sent += 1;
                if once_limit > 0 && sent >= once_limit {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if write!(stream, ": keepalive\n\n").is_err() || stream.flush().is_err() {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

/// `#203` HTML result fragments with preload `Link` hints (`GET /api/v1/fragments/issues`).
pub fn handle_fragment_issues(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
) -> std::io::Result<()> {
    let run_id = http::query_str(&req.query, "run_id");
    let list = app.issues.list_filtered(run_id.as_deref());
    let mut html = String::from(
        "<section class=\"airbug-fragment\" data-fragment=\"issues\"><table><thead><tr><th>Title</th><th>Level</th><th>Count</th><th>Status</th></tr></thead><tbody>",
    );
    for issue in list.issues.iter().take(50) {
        html.push_str(&format!(
            "<tr data-issue-id=\"{}\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&issue.id),
            escape_html(&issue.title),
            escape_html(&issue.level),
            issue.count,
            escape_html(issue.status.as_str())
        ));
    }
    html.push_str("</tbody></table></section>");
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nLink: </api/v1/issues>; rel=preload; as=fetch, </api/v1/stream>; rel=preload; as=fetch\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{html}",
        html.len()
    )
}

/// `#203` HTML result fragments for recent runs (`GET /api/v1/fragments/runs`).
pub fn handle_fragment_runs(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
) -> std::io::Result<()> {
    let limit = http::query_usize(&req.query, "limit", 25).clamp(1, 100);
    let kind = http::query_str(&req.query, "kind");
    let runs = store::timeline(&app.paths.root, &app.runs, kind.as_deref(), limit);
    let mut html = String::from(
        "<section class=\"airbug-fragment\" data-fragment=\"runs\"><table><thead><tr><th>ID</th><th>Kind</th><th>Status</th></tr></thead><tbody>",
    );
    if let Some(arr) = runs.get("runs").and_then(Value::as_array) {
        for run in arr {
            let id = run.get("run_id").and_then(Value::as_str).unwrap_or("");
            let rkind = run.get("kind").and_then(Value::as_str).unwrap_or("");
            let status = run
                .get("state")
                .or_else(|| run.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("");
            html.push_str(&format!(
                "<tr data-run-id=\"{}\"><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(id),
                escape_html(id),
                escape_html(rkind),
                escape_html(status),
            ));
        }
    }
    html.push_str("</tbody></table></section>");
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nLink: </api/v1/runs>; rel=preload; as=fetch, </api/v1/ws>; rel=preload; as=fetch\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{html}",
        html.len()
    )
}

/// `#204` Read-only GraphQL query/subscription metadata endpoint (`POST /api/v1/graphql`).
pub fn handle_graphql(stream: &mut TcpStream, app: &HubApp, req: &Request) -> std::io::Result<()> {
    #[derive(Deserialize)]
    struct GraphQlReq {
        query: String,
    }

    let parsed: GraphQlReq = match serde_json::from_slice(&req.body) {
        Ok(q) => q,
        Err(e) => {
            return http::respond_json(
                stream,
                "400 Bad Request",
                &json!({ "errors": [{ "message": format!("invalid GraphQL JSON body: {e}") }] }),
            );
        }
    };

    let q = parsed.query.as_str();
    let mut data = serde_json::Map::new();
    let mut matched = false;

    if q.contains("status") {
        matched = true;
        data.insert(
            "status".into(),
            serde_json::to_value(scan::scan_with_hub(&app.paths.root, app.port, &app.hub_id))
                .unwrap_or(Value::Null),
        );
    }
    if q.contains("issues") {
        matched = true;
        data.insert(
            "issues".into(),
            serde_json::to_value(app.issues.list()).unwrap_or(Value::Null),
        );
    }
    if q.contains("runs") {
        matched = true;
        data.insert(
            "runs".into(),
            store::timeline(&app.paths.root, &app.runs, None, 50),
        );
    }
    if q.contains("streamSnapshot") || q.trim_start().starts_with("subscription") {
        matched = true;
        data.insert("streamSnapshot".into(), app.stream.snapshot(50));
        data.insert(
            "transport".into(),
            json!({ "ws": "/api/v1/ws", "sse": "/api/v1/stream" }),
        );
    }

    if !matched {
        return http::respond_json(
            stream,
            "400 Bad Request",
            &json!({ "errors": [{ "message": "unknown GraphQL selection (supported: status, issues, runs, streamSnapshot)" }] }),
        );
    }

    http::respond_json(stream, "200 OK", &json!({ "data": data }))
}

/// `#206` Batched JSON Lines progress feed (`GET /api/v1/runs/<id>/progress.jsonl`).
pub fn handle_run_progress_jsonl(
    stream: &mut TcpStream,
    app: &HubApp,
    id: &str,
) -> std::io::Result<()> {
    let Some(detail) = crate::rundetail::describe(&app.paths.root, &app.runs, id) else {
        return http::respond_err(
            stream,
            "404 Not Found",
            &crate::error::HubError::msg("run not found"),
        );
    };

    let mut lines = String::new();
    let progress_candidates = [
        app.paths
            .root
            .join(".airbug/runs")
            .join(id)
            .join("progress.json"),
        app.paths.bench_run_dir(id).join("progress.json"),
        app.paths.bench_run_dir(id).join("progress.jsonl"),
    ];
    for candidate in &progress_candidates {
        if !candidate.is_file() {
            continue;
        }
        if candidate.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            if let Ok(raw) = std::fs::read_to_string(candidate) {
                for line in raw.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        lines.push_str(trimmed);
                        lines.push('\n');
                    }
                }
            }
        } else if let Ok(raw) = std::fs::read_to_string(candidate)
            && let Ok(val) = serde_json::from_str::<Value>(&raw)
        {
            lines.push_str(&serde_json::to_string(&val).unwrap_or_default());
            lines.push('\n');
        }
    }

    if lines.is_empty() {
        lines.push_str(&serde_json::to_string(&detail).unwrap_or_default());
        lines.push('\n');
    }

    http::respond(
        stream,
        "200 OK",
        "application/x-ndjson; charset=utf-8",
        &lines,
    )
}

fn escape_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_and_websocket_accept_key_match_rfc_vectors() {
        let digest = sha1_digest(b"abc");
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, "a9993e364706816aba3e25717850c26c9cd0d89d");

        // RFC 6455 section 4.2.2 example key & expected Sec-WebSocket-Accept
        assert_eq!(
            websocket_accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn stream_hub_backpressure_drops_overflow_without_blocking() {
        let hub = StreamHub::new();
        let _rx = hub.subscribe();
        let total = DEFAULT_SUBSCRIBER_CAPACITY + 15;
        for i in 0..total {
            hub.publish("tick", json!({ "i": i }));
        }
        let snap = hub.snapshot(10);
        assert_eq!(snap["dropped_total"], 15);
        assert_eq!(snap["active_subscribers"], 1);
        assert_eq!(snap["events"].as_array().unwrap().len(), 10);
    }
}
