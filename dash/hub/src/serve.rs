//! Localhost-only dashboard server (route table).
use crate::{
    app::HubApp,
    assets, config,
    error::HubError,
    event_model::{EVENT_MODEL_VERSION, EventEnvelope, EventPayload},
    http::{self, Request},
    issues, otlp, runs, scan, store,
};
use std::{
    net::{TcpListener, TcpStream},
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
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
        hub_id = %app.hub_id,
        root = %app.paths.root.display(),
        "airbug hub listening"
    );
    if let Some(url) = app.webhook.as_deref() {
        tracing::info!(%url, "issues webhook configured");
    }
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
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
    if let Some(rest) = path.strip_prefix("/api/v1")
        && (rest.is_empty() || rest.starts_with('/'))
    {
        return Some(rest);
    }
    if let Some(rest) = path.strip_prefix("/api")
        && (rest.is_empty() || rest.starts_with('/'))
    {
        return Some(rest);
    }
    None
}

fn handle(stream: &mut TcpStream, app: &HubApp) -> std::io::Result<()> {
    let req = http::read_request(stream)?;
    if !req.is_local_host() {
        return http::respond_err(
            stream,
            "403 Forbidden",
            &HubError::msg("non-local Host header rejected"),
        );
    }
    if req.body_too_large {
        return http::respond_err(
            stream,
            "413 Payload Too Large",
            &HubError::msg("request body exceeds MAX_BODY_BYTES"),
        );
    }
    if req.method == "POST" && !req.allows_browser_post() {
        return http::respond_err(
            stream,
            "403 Forbidden",
            &HubError::msg("cross-origin browser POST rejected"),
        );
    }
    let path_only = req.path.as_str();
    if req.method == "POST"
        && let Some(sig) = otlp::receiver::OtlpSignal::from_http_path(path_only)
    {
        return otlp::receiver::handle_http_otlp(stream, app, &req, sig);
    }
    if req.method == "POST"
        && let Some((_project_id, kind)) = crate::sentry::match_sentry_path(path_only)
    {
        return crate::sentry::handle_sentry_http(stream, app, &req, kind);
    }
    if req.method == "GET"
        && let Some((mime, body)) = assets::lookup(path_only)
    {
        return http::respond(stream, "200 OK", mime, body);
    }

    match (req.method.as_str(), path_only) {
        ("GET", p) if p.starts_with("/runs/") => serve_run_file(stream, app, &p["/runs/".len()..]),
        ("GET", "/api/open") => http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "use /report/ for unit HTML",
        ),
        (method, path) if api_path(path).is_some() => {
            let rest = api_path(path).unwrap_or("");
            handle_api(stream, app, method, rest, &req)
        }
        ("GET", "/report") => serve_unit_report(stream, &app.paths.root, "index.html"),
        ("GET", p) if p.starts_with("/report/") => {
            let tail = &p["/report/".len()..];
            let name = if tail.is_empty() { "index.html" } else { tail };
            serve_unit_report(stream, &app.paths.root, name)
        }
        ("GET", p) if p.starts_with("/bench/") => {
            serve_bench_artifact(stream, &app.paths.root, &p["/bench/".len()..])
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
        ("GET", "/launches") => match crate::test_runs::list(&app.paths.root) {
            Ok(value) => http::respond_json(stream, "200 OK", &value),
            Err(error) => http::respond_err(stream, "500 Internal Server Error", &error.into()),
        },
        ("GET", p) if p.starts_with("/launches/") => {
            let parts: Vec<_> = p["/launches/".len()..].split('/').collect();
            let result = match parts.as_slice() {
                [id] => crate::test_runs::detail(&app.paths.root, id),
                [id, "cases", index] => index
                    .parse::<usize>()
                    .map_err(std::io::Error::other)
                    .and_then(|index| crate::test_runs::case(&app.paths.root, id, index)),
                [id, "cases", index, "attachments", file] => {
                    let result = index
                        .parse::<usize>()
                        .map_err(std::io::Error::other)
                        .and_then(|index| {
                            crate::test_runs::attachment(&app.paths.root, id, index, file)
                        });
                    return match result {
                        Ok(bytes) => http::respond_bytes(
                            stream,
                            "200 OK",
                            "application/octet-stream",
                            &bytes,
                        ),
                        Err(error) => http::respond_err(stream, "404 Not Found", &error.into()),
                    };
                }
                _ => Err(std::io::ErrorKind::NotFound.into()),
            };
            match result {
                Ok(value) => http::respond_json(stream, "200 OK", &value),
                Err(error) => http::respond_err(stream, "404 Not Found", &error.into()),
            }
        }
        ("GET", "/") => {
            let snap = scan::scan_with_hub(&app.paths.root, app.port, &app.hub_id);
            http::respond_json(stream, "200 OK", &snap.apis)
        }
        ("GET", "/status") => {
            let snap = scan::scan_with_hub(&app.paths.root, app.port, &app.hub_id);
            http::respond_json(stream, "200 OK", &snap)
        }
        ("GET", "/logs") => {
            let limit =
                http::query_usize(&req.query, "limit", config::DEFAULT_LOGS_LIMIT).clamp(1, 500);
            let run_id = http::query_str(&req.query, "run_id");
            let trace_id = http::query_str(&req.query, "trace_id");
            http::respond_json(
                stream,
                "200 OK",
                &otlp::read_logs_query(
                    &app.paths.root,
                    limit,
                    run_id.as_deref(),
                    trace_id.as_deref(),
                ),
            )
        }
        ("GET", "/metrics") => {
            let limit = http::query_usize(&req.query, "limit", config::DEFAULT_METRICS_LIMIT)
                .clamp(1, 2000);
            let run_id = http::query_str(&req.query, "run_id");
            http::respond_json(
                stream,
                "200 OK",
                &otlp::read_metrics_filtered(&app.paths.root, limit, run_id.as_deref()),
            )
        }
        ("GET", "/traces") => {
            let limit =
                http::query_usize(&req.query, "limit", config::DEFAULT_TRACES_LIMIT).clamp(1, 500);
            let query = otlp::TracesQuery {
                limit,
                service: http::query_str(&req.query, "service"),
                status: http::query_str(&req.query, "status"),
                min_ms: http::query_f64(&req.query, "min_ms"),
                run_id: http::query_str(&req.query, "run_id"),
                search: http::query_str(&req.query, "q"),
            };
            http::respond_json(
                stream,
                "200 OK",
                &otlp::read_traces(&app.paths.root, &query),
            )
        }
        ("GET", p) if p.starts_with("/traces/") => {
            let id = &p["/traces/".len()..];
            if id.is_empty() || id.contains('/') {
                return http::respond(
                    stream,
                    "404 Not Found",
                    "text/plain; charset=utf-8",
                    "not found",
                );
            }
            match otlp::read_trace_detail_with_correlations(app, id) {
                Some(resp) => http::respond_json(stream, "200 OK", &resp),
                None => {
                    http::respond_err(stream, "404 Not Found", &HubError::msg("trace not found"))
                }
            }
        }
        ("GET", "/profiles") => {
            let limit = http::query_usize(&req.query, "limit", config::DEFAULT_PROFILES_LIMIT)
                .clamp(1, 200);
            let query = otlp::ProfilesQuery {
                limit,
                service: http::query_str(&req.query, "service"),
                sample_type: http::query_str(&req.query, "sample_type")
                    .or_else(|| http::query_str(&req.query, "type")),
                run_id: http::query_str(&req.query, "run_id"),
                trace_id: http::query_str(&req.query, "trace_id"),
            };
            http::respond_json(
                stream,
                "200 OK",
                &otlp::read_profiles(&app.paths.root, &query),
            )
        }
        ("GET", "/zpages/tracez") => {
            let service = http::query_str(&req.query, "service");
            http::respond_json(
                stream,
                "200 OK",
                &otlp::read_tracez(&app.paths.root, service.as_deref()),
            )
        }
        ("POST", "/errors") => handle_errors_ingest(stream, app, req),
        ("POST", p) if p.starts_with("/errors/") => {
            let Some(hub_uuid) = p.strip_prefix("/errors/") else {
                return http::respond(
                    stream,
                    "404 Not Found",
                    "text/plain; charset=utf-8",
                    "not found",
                );
            };
            handle_errors_for_hub(stream, app, req, hub_uuid)
        }
        ("POST", "/events") => handle_unified_event(stream, app, req),
        ("GET", "/events") => handle_event_search(stream, app, req),
        ("GET", "/issues") => {
            let run_id = http::query_str(&req.query, "run_id");
            http::respond_json(
                stream,
                "200 OK",
                &app.issues.list_filtered(run_id.as_deref()),
            )
        }
        ("GET", "/run") => {
            let id = http::query_str(&req.query, "id").unwrap_or_default();
            match crate::rundetail::describe(&app.paths.root, &app.runs, &id) {
                Some(value) => http::respond_json(stream, "200 OK", &value),
                None => http::respond_err(stream, "404 Not Found", &HubError::msg("run not found")),
            }
        }
        ("GET", "/runs") => {
            let limit = http::query_usize(&req.query, "limit", 60).clamp(1, 500);
            let kind = http::query_str(&req.query, "kind");
            http::respond_json(
                stream,
                "200 OK",
                &store::timeline(&app.paths.root, &app.runs, kind.as_deref(), limit),
            )
        }
        ("GET", p) if p.starts_with("/runs/") && p.ends_with("/junit.xml") => {
            let id = &p["/runs/".len()..p.len() - "/junit.xml".len()];
            handle_run_junit(stream, app, id)
        }
        ("GET", p) if p.starts_with("/runs/") => {
            let id = &p["/runs/".len()..];
            let lite = http::query_value(&req.query, "lite").is_some_and(|v| v == "1");
            match store::test_run_detail(&app.paths.root, id, lite) {
                Ok(detail) => http::respond_json(stream, "200 OK", &detail),
                Err(e) => {
                    let status = if e.to_string().contains("not found") {
                        "404 Not Found"
                    } else {
                        "400 Bad Request"
                    };
                    http::respond_err(stream, status, &e)
                }
            }
        }
        ("POST", p) if p.starts_with("/runs/") && p.ends_with("/rerun") => handle_rerun(
            stream,
            app,
            req,
            &p["/runs/".len()..p.len() - "/rerun".len()],
        ),
        ("GET", "/tests/analytics") => {
            let window = http::query_usize(&req.query, "window", 20).clamp(1, 100);
            http::respond_json(
                stream,
                "200 OK",
                &crate::testrun::analytics::compute_analytics(&app.paths.root, window),
            )
        }
        ("GET", "/tests/rules") => http::respond_json(
            stream,
            "200 OK",
            &serde_json::json!({
                "rules": crate::testrun::analytics::load_rules(&app.paths.root),
            }),
        ),
        ("POST", "/tests/rules") => handle_post_test_rules(stream, app, req),
        ("GET", "/resources") => {
            http::respond_json(stream, "200 OK", &app.resources.list_resources())
        }
        ("GET", p) if p.starts_with("/resources/") && p.ends_with("/logs") => {
            let name = &p["/resources/".len()..p.len() - "/logs".len()];
            let tail = http::query_usize(&req.query, "tail", 500).clamp(1, 2000);
            match app.resources.logs(name, tail) {
                Ok(val) => http::respond_json(stream, "200 OK", &val),
                Err(e) => http::respond_err(stream, "404 Not Found", &e),
            }
        }
        ("POST", p) if p.starts_with("/resources/") => handle_resource_action(stream, app, p),
        ("POST", "/sessions") => handle_session_ingest(stream, app, req),
        ("GET", "/releases") => http::respond_json(
            stream,
            "200 OK",
            &serde_json::json!({
                "releases": app.issues.list_releases(),
                "dsn": format!("http://airbug@127.0.0.1:{}/1", app.port),
            }),
        ),
        ("POST", "/bench/runs") => handle_create_run(stream, app, req),
        ("GET", "/bench/compare") => {
            let run = http::query_str(&req.query, "run");
            let baseline = http::query_str(&req.query, "baseline");
            http::respond_json(
                stream,
                "200 OK",
                &crate::benchcmp::compare(&app.paths.root, run.as_deref(), baseline.as_deref()),
            )
        }
        ("GET", "/bench") | ("GET", "/bench/runs") => match app.runs.list() {
            Ok(list) => http::respond_json(stream, "200 OK", &list),
            Err(e) => http::respond_err(stream, "500 Internal Server Error", &e),
        },
        ("GET", p) if p.starts_with("/bench/runs/") => handle_bench_run_get(stream, app, p),
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

#[derive(serde::Deserialize)]
struct RerunRequest {
    #[serde(default = "yes")]
    failed: bool,
}

fn yes() -> bool {
    true
}

/// `POST /api/v1/runs/<id>/rerun` `{"failed": true}` — start `airbug-hub test` again.
fn handle_rerun(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
    id: &str,
) -> std::io::Result<()> {
    if !req.same_origin_json() {
        return http::respond_err(
            stream,
            "403 Forbidden",
            &HubError::msg("send JSON (Content-Type: application/json) from the hub page"),
        );
    }
    let body: RerunRequest = if req.body.is_empty() {
        RerunRequest { failed: true }
    } else {
        match serde_json::from_slice(&req.body) {
            Ok(v) => v,
            Err(e) => return http::respond_err(stream, "400 Bad Request", &HubError::from(e)),
        }
    };
    match crate::rerun::start(&app.paths.root, app.port, id, body.failed) {
        Ok(started) => http::respond_json(stream, "202 Accepted", &started),
        Err(e) => {
            let text = e.to_string();
            let status = if text.starts_with("busy") {
                "409 Conflict"
            } else if text.contains("not found") {
                "404 Not Found"
            } else {
                "400 Bad Request"
            };
            http::respond_err(stream, status, &e)
        }
    }
}

fn handle_create_run(stream: &mut TcpStream, app: &HubApp, req: &Request) -> std::io::Result<()> {
    let body = if req.body.is_empty() {
        runs::CreateRunRequest {
            title: None,
            command: None,
        }
    } else {
        match serde_json::from_slice(&req.body) {
            Ok(v) => v,
            Err(e) => return http::respond_err(stream, "400 Bad Request", &HubError::from(e)),
        }
    };
    match app.runs.create(body) {
        Ok(resp) => http::respond_json(stream, "200 OK", &resp),
        Err(e) => http::respond_err(stream, "500 Internal Server Error", &e),
    }
}

fn handle_bench_run_get(stream: &mut TcpStream, app: &HubApp, path: &str) -> std::io::Result<()> {
    let rest = &path["/bench/runs/".len()..];
    if rest.is_empty() {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found",
        );
    }
    if let Some((id, tail)) = rest.split_once('/') {
        if !runs::is_uuidish(id) {
            return http::respond_err(stream, "400 Bad Request", &HubError::msg("invalid run_id"));
        }
        if tail == "report" || tail == "report.html" {
            return serve_file(stream, &app.paths.bench_run_dir(id).join("report.html"));
        }
        if tail == "run.json" {
            return serve_file(stream, &app.paths.bench_run_dir(id).join("run.json"));
        }
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found",
        );
    }
    match app.runs.get(rest) {
        Ok(detail) => http::respond_json(stream, "200 OK", &detail),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                "404 Not Found"
            } else {
                "400 Bad Request"
            };
            http::respond_err(stream, status, &e)
        }
    }
}

fn serve_file(stream: &mut TcpStream, path: &Path) -> std::io::Result<()> {
    if !path.is_file() {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        );
    }
    let bytes = std::fs::read(path)?;
    http::respond_bytes(stream, "200 OK", http::mime_for_path(path), &bytes)
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
    let event_id = envelope.event_id.clone();
    let signal = envelope.signal();
    let issue = match &envelope.payload {
        EventPayload::Error(event) => {
            match app.issues.ingest((**event).clone(), app.webhook.as_deref()) {
                Ok(resp) => Some(resp),
                Err(e) => {
                    let status = if e.to_string().contains("schema_version") {
                        "400 Bad Request"
                    } else {
                        "500 Internal Server Error"
                    };
                    return http::respond_err(stream, status, &e);
                }
            }
        }
        EventPayload::Trace { .. } | EventPayload::Metric { .. } | EventPayload::Log { .. } => None,
    };
    if let Err(err) = app.append_event_envelope(&envelope) {
        return http::respond_err(stream, "500 Internal Server Error", &HubError::from(err));
    }

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

fn handle_errors_for_hub(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
    hub_uuid: &str,
) -> std::io::Result<()> {
    if app.hub_id != hub_uuid {
        return http::respond_err(stream, "404 Not Found", &HubError::msg("hub not found"));
    }
    handle_errors_ingest(stream, app, req)
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

    let service = http::query_str(&req.query, "service");
    let env = http::query_str(&req.query, "environment");
    let release = http::query_str(&req.query, "release");
    let signal = http::query_str(&req.query, "signal");
    let trace_id = http::query_str(&req.query, "trace_id");
    let tag_pairs: Vec<(String, String)> = req
        .query
        .split('&')
        .filter_map(|pair| {
            let (key, _) = pair.split_once('=')?;
            let tag_key = key.strip_prefix("tag_")?;
            let val = http::query_str(pair, key)?;
            Some((tag_key.to_string(), val))
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
    if let Some(tid) = trace_id {
        filtered.retain(|event| {
            event
                .trace_id
                .as_deref()
                .is_some_and(|t| t.eq_ignore_ascii_case(&tid))
        });
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
        Ok(resp) => {
            let _ = app.append_event_envelope(&envelope);
            http::respond_json(stream, "200 OK", &resp)
        }
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
            &HubError::msg("action must be resolve|resolve_next_release|ignore|reopen"),
        );
    };
    match app.issues.set_status(id, status) {
        Ok(issue) => http::respond_json(stream, "200 OK", &issue),
        Err(e) => http::respond_err(stream, "404 Not Found", &e),
    }
}

fn handle_run_junit(stream: &mut TcpStream, app: &HubApp, id: &str) -> std::io::Result<()> {
    let resolved_id = match store::resolve_test_run(&app.paths.root, id) {
        Ok(id) => id,
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                "404 Not Found"
            } else {
                "400 Bad Request"
            };
            return http::respond_err(stream, status, &e);
        }
    };
    let junit_file = store::runs_dir(&app.paths.root)
        .join(&resolved_id)
        .join("junit.xml");
    if junit_file.is_file()
        && let Ok(xml) = std::fs::read_to_string(&junit_file)
    {
        return http::respond(stream, "200 OK", "application/xml; charset=utf-8", &xml);
    }
    let detail = match store::test_run_detail(&app.paths.root, &resolved_id, false) {
        Ok(detail) => detail,
        Err(e) => return http::respond_err(stream, "404 Not Found", &e),
    };
    let Some(report_val) = detail.get("report") else {
        return http::respond_err(
            stream,
            "404 Not Found",
            &HubError::msg("test run report not available yet"),
        );
    };
    let report: airbug_run::model::Report = match serde_json::from_value(report_val.clone()) {
        Ok(r) => r,
        Err(e) => return http::respond_err(stream, "500 Internal Server Error", &e.into()),
    };
    let xml = crate::testrun::junit::render_junit_xml(&report);
    http::respond(stream, "200 OK", "application/xml; charset=utf-8", &xml)
}

fn handle_post_test_rules(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
) -> std::io::Result<()> {
    use crate::testrun::analytics::{DefectRule, load_rules, save_rules};
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum RulesPayload {
        Wrapped { rules: Vec<DefectRule> },
        List(Vec<DefectRule>),
        Single(DefectRule),
    }
    let parsed: RulesPayload = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return http::respond_err(stream, "400 Bad Request", &HubError::from(e)),
    };
    let rules = match parsed {
        RulesPayload::Wrapped { rules } | RulesPayload::List(rules) => rules,
        RulesPayload::Single(rule) => {
            let mut existing = load_rules(&app.paths.root);
            existing.retain(|r| r.name != rule.name);
            existing.push(rule);
            existing
        }
    };
    if let Err(e) = save_rules(&app.paths.root, &rules) {
        return http::respond_err(stream, "500 Internal Server Error", &HubError::from(e));
    }
    http::respond_json(
        stream,
        "200 OK",
        &serde_json::json!({ "ok": true, "rules": rules }),
    )
}

fn handle_resource_action(stream: &mut TcpStream, app: &HubApp, path: &str) -> std::io::Result<()> {
    let rest = &path["/resources/".len()..];
    let Some((name, action)) = rest.split_once('/') else {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found",
        );
    };
    let result = match action {
        "start" => app.resources.start_resource(name),
        "stop" => app.resources.stop_resource(name),
        "restart" => app.resources.restart_resource(name),
        _ => {
            return http::respond_err(
                stream,
                "400 Bad Request",
                &HubError::msg("action must be start|stop|restart"),
            );
        }
    };
    match result {
        Ok(val) => http::respond_json(stream, "200 OK", &val),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                "404 Not Found"
            } else {
                "400 Bad Request"
            };
            http::respond_err(stream, status, &e)
        }
    }
}

fn handle_session_ingest(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
) -> std::io::Result<()> {
    let session = match serde_json::from_slice::<airbug_err::SessionUpdate>(&req.body) {
        Ok(s) => s,
        Err(_) => match crate::sentry::parse_sentry_session_json(&req.body) {
            Ok(s) => s,
            Err(e) => return http::respond_err(stream, "400 Bad Request", &e),
        },
    };
    match app.issues.record_session(session) {
        Ok(()) => http::respond_json(stream, "200 OK", &serde_json::json!({ "ok": true })),
        Err(e) => http::respond_err(stream, "500 Internal Server Error", &e),
    }
}

/// `/runs/<id>/<file>` — files of a test run (`index.html`, `output.log`, `events/…`);
/// `/runs/<id>/files/<file>` — files of a bench session's output directory.
fn serve_run_file(stream: &mut TcpStream, app: &HubApp, rest: &str) -> std::io::Result<()> {
    let found = rest.split_once('/').and_then(|(id, rel)| {
        if let Some(artifact) = rel.strip_prefix("files/") {
            return store::artifact_file(&app.paths.root, id, artifact);
        }
        let rel = if rel.is_empty() { "index.html" } else { rel };
        store::run_file(&app.paths.root, id, rel)
    });
    match found {
        Some(path) => serve_file(stream, &path),
        None => http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        ),
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
    serve_file(stream, &path)
}

fn serve_bench_artifact(stream: &mut TcpStream, root: &Path, rel: &str) -> std::io::Result<()> {
    if rel.is_empty() || rel.contains("..") || rel.starts_with('/') || Path::new(rel).is_absolute()
    {
        return http::respond(
            stream,
            "400 Bad Request",
            "text/plain; charset=utf-8",
            "bad path",
        );
    }
    let store = root.join(".airbug-bench");
    let path = store.join(rel);
    let Ok(canon_store) = store.canonicalize() else {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        );
    };
    let Ok(canon) = path.canonicalize() else {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        );
    };
    if !canon.starts_with(&canon_store) || !canon.is_file() {
        return http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "missing",
        );
    }
    let bytes = std::fs::read(&canon)?;
    http::respond_bytes(stream, "200 OK", http::mime_for_path(&canon), &bytes)
}

#[allow(dead_code)]
pub fn status_json(root: &Path) -> String {
    status_json_with_port(root, config::DEFAULT_PORT)
}

pub fn status_json_with_port(root: &Path, port: u16) -> String {
    let paths = config::RootPaths::new(root);
    let hub_id =
        runs::load_or_create_hub_id(&paths.hub_id_file()).unwrap_or_else(|_| "unknown".into());
    let snap = scan::scan_with_hub(root, port, &hub_id);
    serde_json::to_string_pretty(&snap).unwrap_or_default()
}
