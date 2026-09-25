//! Localhost-only dashboard server (route table).
use crate::{
    app::HubApp,
    config,
    error::HubError,
    event_model::{EVENT_MODEL_VERSION, EventEnvelope, EventPayload},
    http::{self, Request},
    issues, otlp, scan,
};
use std::{
    net::{TcpListener, TcpStream},
    path::Path,
    sync::Arc,
    thread,
};

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct ServeOpts {
    pub webhook: Option<String>,
}

pub fn serve(app: Arc<HubApp>) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", app.port))?;
    tracing::info!(
        port = app.port,
        root = %app.paths.root.display(),
        "airbug hub listening"
    );
    if let Some(url) = app.webhook.as_deref() {
        tracing::info!(%url, "issues webhook configured");
    }
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let app = Arc::clone(&app);
        thread::spawn(move || {
            if let Err(err) = handle(&mut stream, &app) {
                tracing::debug!(error = %err, "request handler error");
            }
        });
    }
    Ok(())
}

fn api_path(path: &str) -> Option<&str> {
    path.strip_prefix("/api/v1")
        .or_else(|| path.strip_prefix("/api"))
}

fn handle(stream: &mut TcpStream, app: &HubApp) -> std::io::Result<()> {
    let req = http::read_request(stream)?;
    let path_only = req.path.as_str();

    match (req.method.as_str(), path_only) {
        ("GET", "/") | ("GET", "/index.html") => {
            http::respond(stream, "200 OK", "text/html; charset=utf-8", PAGE)
        }
        ("GET", "/static/dashboard.css") => {
            http::respond(stream, "200 OK", "text/css; charset=utf-8", DASHBOARD_CSS)
        }
        ("GET", "/static/dashboard.js") => http::respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            DASHBOARD_JS,
        ),
        (method, path) if api_path(path).is_some() => {
            let rest = api_path(path).unwrap_or("");
            handle_api(stream, app, method, rest, &req)
        }
        ("GET", "/api/open") => http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "use /report/ for unit HTML",
        ),
        ("GET", p) if p.starts_with("/report/") => {
            serve_unit_report(stream, &app.paths.root, &p["/report/".len()..])
        }
        _ => http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found",
        ),
    }
}

fn handle_api(
    stream: &mut TcpStream,
    app: &HubApp,
    method: &str,
    rest: &str,
    req: &Request,
) -> std::io::Result<()> {
    let rest = if rest.is_empty() { "/" } else { rest };
    match (method, rest) {
        ("GET", "/") => {
            let snap = scan::scan_with_port(&app.paths.root, app.port);
            http::respond_json(stream, "200 OK", &snap.apis)
        }
        ("GET", "/status") => {
            let snap = scan::scan_with_port(&app.paths.root, app.port);
            http::respond_json(stream, "200 OK", &snap)
        }
        ("GET", "/logs") => {
            let limit =
                http::query_usize(&req.query, "limit", config::DEFAULT_LOGS_LIMIT).clamp(1, 500);
            http::respond_json(stream, "200 OK", &otlp::read_logs(&app.paths.root, limit))
        }
        ("GET", "/metrics") => {
            let limit = http::query_usize(&req.query, "limit", config::DEFAULT_METRICS_LIMIT)
                .clamp(1, 2000);
            http::respond_json(
                stream,
                "200 OK",
                &otlp::read_metrics(&app.paths.root, limit),
            )
        }
        ("POST", "/errors") => handle_errors_ingest(stream, app, req),
        ("POST", "/events") => handle_unified_event(stream, app, req),
        ("GET", "/events") => handle_event_search(stream, app, req),
        ("GET", "/issues") => http::respond_json(stream, "200 OK", &app.issues.list()),
        ("GET", p) if p.starts_with("/issues/") => {
            let id = &p["/issues/".len()..];
            if id.is_empty() || id.contains('/') {
                return http::respond(
                    stream,
                    "404 Not Found",
                    "text/plain; charset=utf-8",
                    "not found",
                );
            }

            match app.issues.get(id) {
                Some(issue) => http::respond_json(stream, "200 OK", &issue),
                None => http::respond_err(stream, "404 Not Found", &HubError::msg("not found")),
            }
        }

        ("POST", p) if p.starts_with("/issues/") => handle_issue_action(stream, app, p),
        _ => http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found",
        ),
    }
}

fn handle_unified_event(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
) -> std::io::Result<()> {
    let envelope: EventEnvelope = match serde_json::from_slice(&req.body) {
        Ok(value) => value,
        Err(error) => return http::respond_err(stream, "400 Bad Request", &HubError::from(error)),
    };
    if envelope.model_version != EVENT_MODEL_VERSION {
        return http::respond_err(
            stream,
            "400 Bad Request",
            &HubError::msg(format!(
                "unsupported event model_version {} (supported {})",
                envelope.model_version, EVENT_MODEL_VERSION
            )),
        );
    }
    let payload = serde_json::to_string(&envelope)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let event_id = envelope.event_id.clone();
    let signal = envelope.signal();
    let events_path = app.paths.events_file();
    if let Some(parent) = events_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path)?;
    writeln!(file, "{payload}")?;

    let issue = match envelope.payload {
        EventPayload::Error(event) => Some(
            app.issues
                .ingest(*event, app.webhook.as_deref())
                .map_err(|error| std::io::Error::other(error.to_string()))?,
        ),
        EventPayload::Trace { .. } | EventPayload::Metric { .. } | EventPayload::Log { .. } => None,
    };
    http::respond_json(
        stream,
        "202 Accepted",
        &serde_json::json!({
            "ok": true,
            "event_id": event_id,
            "signal": signal,
            "issue": issue,
        }),
    )
}

fn handle_event_search(stream: &mut TcpStream, app: &HubApp, req: &Request) -> std::io::Result<()> {
    let path = app.paths.events_file();
    let records = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
            return http::respond_json(
                stream,
                "200 OK",
                &serde_json::json!({"items": [], "count": 0}),
            );
        }
    };

    let mut items = Vec::new();
    for line in records.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(envelope): Result<EventEnvelope, _> = serde_json::from_str(trimmed) else {
            continue;
        };
        items.push(envelope);
    }

    let service = http::query_value(&req.query, "service");
    let env = http::query_value(&req.query, "environment");
    let release = http::query_value(&req.query, "release");
    let signal = http::query_value(&req.query, "signal");
    let tag_pairs: Vec<(String, String)> = req
        .query
        .split('&')
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            key.strip_prefix("tag_")
                .map(|tag_key| (tag_key.to_string(), value.to_string()))
        })
        .collect();

    let mut filtered = items;
    if let Some(service) = service {
        filtered.retain(|event| event.service.as_deref() == Some(service.as_str()));
    }
    if let Some(env) = env {
        filtered.retain(|event| event.environment.as_deref() == Some(env.as_str()));
    }
    if let Some(release) = release {
        filtered.retain(|event| event.release.as_deref() == Some(release.as_str()));
    }
    if let Some(signal) = signal {
        filtered.retain(|event| event.signal().as_str() == signal.as_str());
    }
    if !tag_pairs.is_empty() {
        filtered.retain(|event| event.match_tags(&tag_pairs));
    }

    let limit = http::query_usize(&req.query, "limit", 50).clamp(1, 500);
    let offset = http::query_usize(&req.query, "offset", 0);
    let total = filtered.len();
    let page = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    http::respond_json(
        stream,
        "200 OK",
        &serde_json::json!({
            "count": total,
            "limit": limit,
            "offset": offset,
            "items": page,
        }),
    )
}

fn handle_errors_ingest(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
) -> std::io::Result<()> {
    let event: airbug_err::Event = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => {
            return http::respond_err(stream, "400 Bad Request", &HubError::from(e));
        }
    };
    let envelope = EventEnvelope::from_error(event.clone());
    tracing::debug!(
        event_id = %envelope.event_id,
        signal = ?envelope.signal(),
        "normalized collector event"
    );
    match app.issues.ingest(event, app.webhook.as_deref()) {
        Ok(resp) => http::respond_json(stream, "200 OK", &resp),
        Err(e) => {
            let status = if e.to_string().contains("schema_version") {
                "400 Bad Request"
            } else {
                "500 Internal Server Error"
            };
            http::respond_err(stream, status, &e)
        }
    }
}

fn handle_issue_action(stream: &mut TcpStream, app: &HubApp, path: &str) -> std::io::Result<()> {
    let rest = &path["/issues/".len()..];
    let Some((id, action)) = rest.split_once('/') else {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found",
        );
    };
    let Some(status) = issues::status_from_action(action) else {
        return http::respond_err(
            stream,
            "400 Bad Request",
            &HubError::msg("action must be resolve|ignore|reopen"),
        );
    };
    match app.issues.set_status(id, status) {
        Ok(issue) => http::respond_json(stream, "200 OK", &issue),
        Err(e) => http::respond_err(stream, "404 Not Found", &e),
    }
}

fn serve_unit_report(stream: &mut TcpStream, root: &Path, name: &str) -> std::io::Result<()> {
    let paths = config::RootPaths::new(root);
    let Some(path) = paths.unit_report_file(name) else {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        );
    };
    if !path.is_file() {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        );
    }
    let bytes = std::fs::read(&path)?;
    http::respond_bytes(stream, "200 OK", http::mime_for_path(&path), &bytes)
}

pub fn status_json(root: &Path) -> String {
    let snap = scan::scan(root);
    serde_json::to_string_pretty(&snap).unwrap_or_default()
}

const PAGE: &str = include_str!("../static/dashboard.html");
const DASHBOARD_CSS: &str = include_str!("../static/dashboard.css");
const DASHBOARD_JS: &str = include_str!("../static/dashboard.js");
