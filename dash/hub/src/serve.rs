//! Localhost-only dashboard server (route table).
use crate::{
    config::{self, RootPaths},
    error::HubError,
    http::{self, Request},
    issues, otlp, scan,
};
use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Default)]
pub struct ServeOpts {
    pub webhook: Option<String>,
}

pub fn serve(root: PathBuf, port: u16, opts: ServeOpts) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let root = Arc::new(root);
    let opts = Arc::new(opts);
    eprintln!(
        "airbug hub on http://127.0.0.1:{port}/  (root {})",
        root.display()
    );
    if let Some(url) = opts.webhook.as_deref() {
        eprintln!("issues webhook: {url}");
    }
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let root = Arc::clone(&root);
        let opts = Arc::clone(&opts);
        let _ = handle(&mut stream, &root, port, &opts);
    }
    Ok(())
}

fn handle(
    stream: &mut std::net::TcpStream,
    root: &Path,
    port: u16,
    opts: &ServeOpts,
) -> std::io::Result<()> {
    let req = http::read_request(stream)?;
    let path_only = req.path.as_str();

    match (req.method.as_str(), path_only) {
        ("GET", "/") | ("GET", "/index.html") => {
            http::respond(stream, "200 OK", "text/html; charset=utf-8", PAGE)
        }
        ("GET", "/static/dashboard.css") => http::respond(
            stream,
            "200 OK",
            "text/css; charset=utf-8",
            DASHBOARD_CSS,
        ),
        ("GET", "/static/dashboard.js") => http::respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            DASHBOARD_JS,
        ),
        ("GET", "/api") | ("GET", "/api/") => {
            let snap = scan::scan_with_port(root, port);
            http::respond_json(stream, "200 OK", &snap.apis)
        }
        ("GET", "/api/status") => {
            let snap = scan::scan_with_port(root, port);
            http::respond_json(stream, "200 OK", &snap)
        }
        ("GET", "/api/logs") => {
            let limit = http::query_usize(&req.query, "limit", config::DEFAULT_LOGS_LIMIT)
                .clamp(1, 500);
            http::respond_json(stream, "200 OK", &otlp::read_logs(root, limit))
        }
        ("GET", "/api/metrics") => {
            let limit = http::query_usize(&req.query, "limit", config::DEFAULT_METRICS_LIMIT)
                .clamp(1, 2000);
            http::respond_json(stream, "200 OK", &otlp::read_metrics(root, limit))
        }
        ("POST", "/api/errors") => handle_errors_ingest(stream, root, &req, opts),
        ("GET", "/api/issues") => http::respond_json(stream, "200 OK", &issues::list(root)),
        ("GET", p) if p.starts_with("/api/issues/") => {
            let id = &p["/api/issues/".len()..];
            if id.is_empty() || id.contains('/') {
                return http::respond(stream, "404 Not Found", "text/plain; charset=utf-8", "not found");
            }
            match issues::get(root, id) {
                Some(issue) => http::respond_json(stream, "200 OK", &issue),
                None => http::respond_err(stream, "404 Not Found", &HubError::msg("not found")),
            }
        }
        ("POST", p) if p.starts_with("/api/issues/") => handle_issue_action(stream, root, p),
        ("GET", "/api/open") => http::respond(
            stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            "use /report/ for unit HTML",
        ),
        ("GET", p) if p.starts_with("/report/") => {
            serve_unit_report(stream, root, &p["/report/".len()..])
        }
        _ => http::respond(stream, "404 Not Found", "text/plain; charset=utf-8", "not found"),
    }
}

fn handle_errors_ingest(
    stream: &mut std::net::TcpStream,
    root: &Path,
    req: &Request,
    opts: &ServeOpts,
) -> std::io::Result<()> {
    let event: airbug_err::Event = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => {
            return http::respond_err(stream, "400 Bad Request", &HubError::from(e));
        }
    };
    match issues::ingest(root, event, opts.webhook.as_deref()) {
        Ok(resp) => http::respond_json(stream, "200 OK", &resp),
        Err(e) => http::respond_err(stream, "500 Internal Server Error", &e),
    }
}

fn handle_issue_action(
    stream: &mut std::net::TcpStream,
    root: &Path,
    path: &str,
) -> std::io::Result<()> {
    let rest = &path["/api/issues/".len()..];
    let Some((id, action)) = rest.split_once('/') else {
        return http::respond(stream, "404 Not Found", "text/plain; charset=utf-8", "not found");
    };
    let Some(status) = issues::status_from_action(action) else {
        return http::respond_err(
            stream,
            "400 Bad Request",
            &HubError::msg("action must be resolve|ignore|reopen"),
        );
    };
    match issues::set_status(root, id, status) {
        Ok(issue) => http::respond_json(stream, "200 OK", &issue),
        Err(e) => http::respond_err(stream, "404 Not Found", &e),
    }
}

fn serve_unit_report(
    stream: &mut std::net::TcpStream,
    root: &Path,
    name: &str,
) -> std::io::Result<()> {
    let paths = RootPaths::new(root);
    let Some(path) = paths.unit_report_file(name) else {
        return http::respond(stream, "404 Not Found", "text/plain; charset=utf-8", "missing");
    };
    if !path.is_file() {
        return http::respond(stream, "404 Not Found", "text/plain; charset=utf-8", "missing");
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
