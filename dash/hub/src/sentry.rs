//! Native Sentry SDK Envelope (`/api/<project_id>/envelope/`) and Store (`/api/<project_id>/store/`) receiver.
//!
//! Any standard Sentry SDK (`sentry` in Rust, `@sentry/browser`, `@sentry/node`, Python `sentry-sdk`,
//! Go `sentry-go`) can point its `DSN` at `http://airbug@127.0.0.1:8790/1` without adapters.
use crate::{
    app::HubApp,
    config::MAX_OTLP_DECOMPRESSED_BYTES,
    error::{HubError, Result},
    event_model::EventEnvelope,
    http::{self, Request},
    issues::now_iso,
};
use airbug_err::{
    Breadcrumb, EVENT_SCHEMA_VERSION, Event, Exception, Frame, SessionStatus, SessionUpdate,
    Severity, Stacktrace, User,
};
use flate2::read::GzDecoder;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Read, net::TcpStream};

/// Returns `Some((project_id, kind))` when `path` is `/api/<id>/envelope[/]` or `/api/<id>/store[/]`.
pub fn match_sentry_path(path: &str) -> Option<(&str, &'static str)> {
    let rest = path.strip_prefix("/api/")?;
    let rest = rest.trim_end_matches('/');
    let (project_id, tail) = rest.split_once('/')?;
    if project_id.is_empty() || project_id == "v1" || project_id == "otlp" {
        return None;
    }
    match tail {
        "envelope" => Some((project_id, "envelope")),
        "store" => Some((project_id, "store")),
        _ => None,
    }
}

pub fn handle_sentry_http(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
    kind: &str,
) -> std::io::Result<()> {
    let raw = match decode_body(req) {
        Ok(bytes) => bytes,
        Err(e) => return http::respond_err(stream, "400 Bad Request", &e),
    };

    match kind {
        "store" => match parse_sentry_event_json(&raw) {
            Ok(event) => {
                let id = event.event_id.clone();
                let env = EventEnvelope::from_error(event.clone());
                if let Err(e) = app.issues.ingest(event, app.webhook.as_deref()) {
                    return http::respond_err(stream, "400 Bad Request", &e);
                }
                let _ = app.append_event_envelope(&env);
                http::respond_json(stream, "200 OK", &json!({ "id": id }))
            }
            Err(e) => http::respond_err(stream, "400 Bad Request", &e),
        },
        "envelope" => match ingest_envelope(app, &raw) {
            Ok(id) => http::respond_json(stream, "200 OK", &json!({ "id": id })),
            Err(e) => http::respond_err(stream, "400 Bad Request", &e),
        },
        _ => http::respond_err(stream, "404 Not Found", &HubError::msg("not found")),
    }
}

fn decode_body(req: &Request) -> Result<Vec<u8>> {
    let is_gzip = req
        .header("content-encoding")
        .is_some_and(|v| v.to_ascii_lowercase().contains("gzip"))
        || req.body.starts_with(&[0x1f, 0x8b]);
    if !is_gzip {
        return Ok(req.body.clone());
    }
    let mut decoder = GzDecoder::new(req.body.as_slice()).take(MAX_OTLP_DECOMPRESSED_BYTES as u64);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| HubError::msg(format!("invalid gzip payload: {e}")))?;
    Ok(out)
}

pub fn ingest_envelope(app: &HubApp, raw: &[u8]) -> Result<String> {
    let (events, sessions, header_event_id) = parse_envelope_bytes(raw)?;
    let mut last_id = header_event_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    for event in events {
        last_id = event.event_id.clone();
        let env = EventEnvelope::from_error(event.clone());
        app.issues.ingest(event, app.webhook.as_deref())?;
        let _ = app.append_event_envelope(&env);
    }
    for session in sessions {
        let _ = app.issues.record_session(session);
    }
    Ok(last_id)
}

/// Parse a Sentry Envelope byte stream into `(events, sessions, envelope_event_id)`.
pub fn parse_envelope_bytes(
    raw: &[u8],
) -> Result<(Vec<Event>, Vec<SessionUpdate>, Option<String>)> {
    let mut pos = 0usize;
    let next_line = |buf: &[u8], start: &mut usize| -> Option<Vec<u8>> {
        if *start >= buf.len() {
            return None;
        }
        let rel_end = buf[*start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(buf.len() - *start);
        let line = buf[*start..*start + rel_end]
            .strip_suffix(b"\r")
            .unwrap_or(&buf[*start..*start + rel_end])
            .to_vec();
        *start = (*start + rel_end + 1).min(buf.len());
        Some(line)
    };

    let Some(header_line) = next_line(raw, &mut pos) else {
        return Err(HubError::msg("empty sentry envelope"));
    };
    let header_val: Value = serde_json::from_slice(&header_line)
        .map_err(|e| HubError::msg(format!("invalid sentry envelope header: {e}")))?;
    let header_event_id = header_val
        .get("event_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let header_trace = header_val.get("trace").cloned();

    let mut events = Vec::new();
    let mut sessions = Vec::new();

    while pos < raw.len() {
        let Some(item_hdr_bytes) = next_line(raw, &mut pos) else {
            break;
        };
        if item_hdr_bytes.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }
        let Ok(item_hdr) = serde_json::from_slice::<Value>(&item_hdr_bytes) else {
            break;
        };
        let item_type = item_hdr
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("event");
        let payload_bytes = if let Some(len) = item_hdr.get("length").and_then(Value::as_u64) {
            let len = len as usize;
            let end = (pos + len).min(raw.len());
            let slice = raw[pos..end].to_vec();
            pos = end;
            if pos < raw.len() && raw[pos] == b'\r' {
                pos += 1;
            }
            if pos < raw.len() && raw[pos] == b'\n' {
                pos += 1;
            }
            slice
        } else {
            next_line(raw, &mut pos).unwrap_or_default()
        };

        match item_type {
            "event" | "transaction" => {
                if let Ok(mut ev) = parse_sentry_event_json(&payload_bytes) {
                    if ev.trace_id.is_none()
                        && let Some(ref tr) = header_trace
                        && let Some(tid) = tr.get("trace_id").and_then(Value::as_str)
                    {
                        ev.trace_id = Some(tid.to_string());
                    }
                    if item_type == "event" {
                        events.push(ev);
                    }
                }
            }
            "session" => {
                if let Ok(sess) = parse_sentry_session_json(&payload_bytes) {
                    sessions.push(sess);
                }
            }
            "sessions" => {
                if let Ok(v) = serde_json::from_slice::<Value>(&payload_bytes)
                    && let Some(arr) = v.get("sessions").and_then(Value::as_array)
                {
                    for s in arr {
                        if let Ok(bytes) = serde_json::to_vec(s)
                            && let Ok(sess) = parse_sentry_session_json(&bytes)
                        {
                            sessions.push(sess);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok((events, sessions, header_event_id))
}

pub fn parse_sentry_event_json(raw: &[u8]) -> Result<Event> {
    let v: Value =
        serde_json::from_slice(raw).map_err(|e| HubError::msg(format!("invalid JSON: {e}")))?;
    // If it's already a native `airbug_err::Event` with `schema_version`, accept directly
    if v.get("schema_version").is_some()
        && let Ok(ev) = serde_json::from_value::<Event>(v.clone())
    {
        return Ok(ev);
    }

    let event_id = v
        .get("event_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let timestamp = match v.get("timestamp") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        Some(Value::Number(n)) => {
            if let Some(secs) = n.as_f64() {
                let s = secs.max(0.0) as u64;
                format!("{s}s unix")
            } else {
                now_iso()
            }
        }
        _ => now_iso(),
    };

    let level = match v
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("error")
        .to_ascii_lowercase()
        .as_str()
    {
        "fatal" | "critical" => Severity::Fatal,
        "warning" | "warn" => Severity::Warning,
        "info" | "debug" | "log" => Severity::Info,
        _ => Severity::Error,
    };

    let message = v
        .get("message")
        .and_then(|m| {
            m.as_str().map(str::to_string).or_else(|| {
                m.get("formatted")
                    .or_else(|| m.get("message"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
        })
        .or_else(|| {
            v.get("logentry").and_then(|le| {
                le.get("formatted")
                    .or_else(|| le.get("message"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
        });

    let release = v
        .get("release")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let environment = v
        .get("environment")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let service = v
        .get("service")
        .or_else(|| v.get("server_name"))
        .or_else(|| v.pointer("/tags/service"))
        .or_else(|| v.pointer("/tags/service.name"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let exception = parse_sentry_exception(&v);
    let breadcrumbs = parse_sentry_breadcrumbs(&v);
    let tags = parse_sentry_tags(&v);

    let user = v.get("user").and_then(Value::as_object).map(|u| User {
        id: u.get("id").and_then(|x| {
            x.as_str()
                .map(str::to_string)
                .or_else(|| Some(x.to_string()))
        }),
        email: u.get("email").and_then(Value::as_str).map(str::to_string),
        username: u
            .get("username")
            .and_then(Value::as_str)
            .map(str::to_string),
    });

    let extra: BTreeMap<String, Value> = v
        .get("extra")
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, val)| (k.clone(), val.clone())).collect())
        .unwrap_or_default();

    let contexts: BTreeMap<String, Value> = v
        .get("contexts")
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, val)| (k.clone(), val.clone())).collect())
        .unwrap_or_default();

    let trace_id = v
        .get("trace_id")
        .or_else(|| v.pointer("/contexts/trace/trace_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let span_id = v
        .get("span_id")
        .or_else(|| v.pointer("/contexts/trace/span_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let mut fingerprint: Vec<String> = v
        .get("fingerprint")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    if fingerprint.is_empty() || (fingerprint.len() == 1 && fingerprint[0] == "{{ default }}") {
        fingerprint = vec![compute_sentry_fingerprint(&exception, message.as_deref())];
    }

    Ok(Event {
        schema_version: EVENT_SCHEMA_VERSION,
        event_id,
        timestamp,
        level,
        message,
        release,
        environment,
        service,
        fingerprint,
        exception,
        breadcrumbs,
        tags,
        user,
        extra,
        contexts,
        trace_id,
        span_id,
    })
}

fn compute_sentry_fingerprint(exception: &Option<Exception>, message: Option<&str>) -> String {
    if let Some(ex) = exception {
        if let Some(st) = &ex.stacktrace
            && let Some(top) = st
                .frames
                .iter()
                .find(|f| f.in_app)
                .or_else(|| st.frames.first())
        {
            return format!(
                "sentry:{}:{}:{}",
                ex.ty,
                top.function,
                top.lineno.unwrap_or(0)
            );
        }
        return format!("sentry:{}:{}", ex.ty, ex.value);
    }
    format!("sentry:msg:{}", message.unwrap_or("unknown"))
}

fn parse_sentry_exception(v: &Value) -> Option<Exception> {
    let ex_obj = v.get("exception")?;
    let val = if let Some(values) = ex_obj.get("values").and_then(Value::as_array) {
        values.last()?
    } else if ex_obj.is_object() && ex_obj.get("type").is_some() {
        ex_obj
    } else {
        ex_obj.as_array()?.last()?
    };

    let ty = val
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("Error")
        .to_string();
    let value = val
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let stacktrace = val
        .get("stacktrace")
        .and_then(|st| st.get("frames"))
        .and_then(Value::as_array)
        .map(|raw_frames| {
            // Sentry stacktrace.frames are ordered oldest-to-newest (innermost frame last);
            // reverse so `airbug-err` newest-first convention holds!
            let mut frames: Vec<Frame> = raw_frames
                .iter()
                .rev()
                .map(|f| {
                    let function = f
                        .get("function")
                        .and_then(Value::as_str)
                        .unwrap_or("<anonymous>")
                        .to_string();
                    let filename = f
                        .get("abs_path")
                        .or_else(|| f.get("filename"))
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let lineno = f.get("lineno").and_then(Value::as_u64).map(|n| n as u32);
                    let colno = f.get("colno").and_then(Value::as_u64).map(|n| n as u32);
                    let in_app = f.get("in_app").and_then(Value::as_bool).unwrap_or(true);
                    Frame {
                        function,
                        filename,
                        lineno,
                        colno,
                        in_app,
                    }
                })
                .collect();
            if frames.is_empty() {
                frames.push(Frame {
                    function: "<unknown>".into(),
                    filename: None,
                    lineno: None,
                    colno: None,
                    in_app: false,
                });
            }
            Stacktrace { frames }
        });

    Some(Exception {
        ty,
        value,
        stacktrace,
    })
}

fn parse_sentry_breadcrumbs(v: &Value) -> Vec<Breadcrumb> {
    let Some(bc_node) = v.get("breadcrumbs") else {
        return Vec::new();
    };
    let arr = bc_node
        .as_array()
        .or_else(|| bc_node.get("values").and_then(Value::as_array));
    let Some(arr) = arr else {
        return Vec::new();
    };
    arr.iter()
        .map(|b| {
            let timestamp = match b.get("timestamp") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n
                    .as_f64()
                    .map(|f| format!("{:.3}s", f))
                    .unwrap_or_else(now_iso),
                _ => now_iso(),
            };
            let ty = b.get("type").and_then(Value::as_str).map(str::to_string);
            let category = b
                .get("category")
                .and_then(Value::as_str)
                .unwrap_or("default")
                .to_string();
            let message = b
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let level = match b
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("info")
                .to_ascii_lowercase()
                .as_str()
            {
                "fatal" | "critical" => Severity::Fatal,
                "error" => Severity::Error,
                "warning" | "warn" => Severity::Warning,
                _ => Severity::Info,
            };
            let data = b
                .get("data")
                .and_then(Value::as_object)
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();
            Breadcrumb {
                timestamp,
                ty,
                category,
                message,
                level,
                data,
            }
        })
        .collect()
}

fn parse_sentry_tags(v: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(tags) = v.get("tags") else {
        return out;
    };
    if let Some(obj) = tags.as_object() {
        for (k, val) in obj {
            if let Some(s) = val.as_str() {
                out.insert(k.clone(), s.to_string());
            } else {
                out.insert(k.clone(), val.to_string());
            }
        }
    } else if let Some(arr) = tags.as_array() {
        for pair in arr {
            if let Some(tuple) = pair.as_array()
                && tuple.len() == 2
                && let (Some(k), Some(val)) = (tuple[0].as_str(), tuple[1].as_str())
            {
                out.insert(k.to_string(), val.to_string());
            }
        }
    }
    out
}

pub fn parse_sentry_session_json(raw: &[u8]) -> Result<SessionUpdate> {
    let v: Value = serde_json::from_slice(raw)
        .map_err(|e| HubError::msg(format!("invalid session JSON: {e}")))?;
    if v.get("session_id").is_some()
        && let Ok(direct) = serde_json::from_value::<SessionUpdate>(v.clone())
    {
        return Ok(direct);
    }
    let session_id = v
        .get("sid")
        .or_else(|| v.get("session_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let distinct_id = v.get("did").or_else(|| v.get("distinct_id")).and_then(|d| {
        d.as_str()
            .map(str::to_string)
            .or_else(|| Some(d.to_string()))
    });
    let release = v
        .pointer("/attrs/release")
        .or_else(|| v.get("release"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let environment = v
        .pointer("/attrs/environment")
        .or_else(|| v.get("environment"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let service = v
        .pointer("/attrs/service")
        .or_else(|| v.get("service"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let status = match v
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("ok")
        .to_ascii_lowercase()
        .as_str()
    {
        "exited" => SessionStatus::Exited,
        "crashed" => SessionStatus::Crashed,
        "abnormal" => SessionStatus::Abnormal,
        _ => SessionStatus::Ok,
    };
    let errors = v.get("errors").and_then(Value::as_u64).unwrap_or(0) as u32;
    let started_at = v
        .get("started")
        .or_else(|| v.get("started_at"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(now_iso);
    let duration_ms = v
        .get("duration")
        .and_then(Value::as_f64)
        .map(|secs| (secs.max(0.0) * 1000.0) as u64)
        .or_else(|| v.get("duration_ms").and_then(Value::as_u64));

    Ok(SessionUpdate {
        session_id,
        distinct_id,
        release,
        environment,
        service,
        status,
        errors,
        started_at,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;

    #[test]
    fn parses_sentry_envelope_with_event_and_session_and_reverses_frames() {
        let envelope = concat!(
            r#"{"event_id":"9ec79c33ec9942ab8353589fcb2e04dc","sent_at":"2026-11-12T00:00:00Z"}"#,
            "\n",
            r#"{"type":"session"}"#,
            "\n",
            r#"{"sid":"sess-1","did":"user-42","status":"crashed","errors":1,"started":"2026-11-12T00:00:00Z","duration":1.5,"attrs":{"release":"checkout@1.2.0","environment":"staging"}}"#,
            "\n",
            r#"{"type":"event"}"#,
            "\n",
            r#"{"event_id":"9ec79c33ec9942ab8353589fcb2e04dc","level":"error","release":"checkout@1.2.0","environment":"staging","server_name":"checkout-svc","exception":{"values":[{"type":"ValueError","value":"invalid coupon","stacktrace":{"frames":[{"function":"main","filename":"src/main.rs","lineno":10,"in_app":true},{"function":"apply_coupon","filename":"src/coupon.rs","lineno":42,"in_app":true}]}}]},"breadcrumbs":{"values":[{"timestamp":"2026-11-12T00:00:00Z","type":"http","category":"fetch","message":"POST /redeem","level":"info","data":{"status_code":400}}]},"tags":[["tenant","acme"]],"contexts":{"trace":{"trace_id":"4bf92f3577b34da6a3ce929d0e0e4736","span_id":"00f067aa0ba902b7"}}}"#,
            "\n"
        );

        let (events, sessions, hdr_id) = parse_envelope_bytes(envelope.as_bytes()).unwrap();
        assert_eq!(hdr_id.as_deref(), Some("9ec79c33ec9942ab8353589fcb2e04dc"));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].release, "checkout@1.2.0");
        assert_eq!(sessions[0].status, SessionStatus::Crashed);
        assert_eq!(sessions[0].duration_ms, Some(1500));

        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev.release.as_deref(), Some("checkout@1.2.0"));
        assert_eq!(ev.service.as_deref(), Some("checkout-svc"));
        assert_eq!(
            ev.trace_id.as_deref(),
            Some("4bf92f3577b34da6a3ce929d0e0e4736")
        );
        assert_eq!(ev.tags.get("tenant").map(String::as_str), Some("acme"));
        assert_eq!(ev.breadcrumbs.len(), 1);
        assert_eq!(ev.breadcrumbs[0].category, "fetch");

        // Innermost frame (`apply_coupon`) must be first after reversal
        let frames = &ev
            .exception
            .as_ref()
            .unwrap()
            .stacktrace
            .as_ref()
            .unwrap()
            .frames;
        assert_eq!(frames[0].function, "apply_coupon");
        assert_eq!(frames[1].function, "main");
    }

    #[test]
    fn gzip_decode_roundtrip_for_sentry_request() {
        let body_str = r#"{"event_id":"e-gzip","message":"compressed hello","level":"warning"}"#;
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(body_str.as_bytes()).unwrap();
        let compressed = enc.finish().unwrap();

        let req = Request {
            method: "POST".into(),
            path: "/api/1/store/".into(),
            query: String::new(),
            headers: vec![("content-encoding".into(), "gzip".into())],
            body: compressed,
            body_too_large: false,
        };
        let decoded = decode_body(&req).unwrap();
        let ev = parse_sentry_event_json(&decoded).unwrap();
        assert_eq!(ev.event_id, "e-gzip");
        assert_eq!(ev.message.as_deref(), Some("compressed hello"));
    }
}
