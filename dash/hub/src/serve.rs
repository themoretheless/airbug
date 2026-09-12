//! Localhost-only dashboard server.
use crate::scan::{self, Snapshot};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
};

pub fn serve(root: PathBuf, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let root = Arc::new(root);
    eprintln!("airbug hub on http://127.0.0.1:{port}/  (root {})", root.display());
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let root = Arc::clone(&root);
        let _ = handle(&mut stream, &root);
    }
    Ok(())
}

fn handle(stream: &mut TcpStream, root: &Path) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    match path {
        "/" | "/index.html" => {
            respond(stream, "200 OK", "text/html; charset=utf-8", PAGE)
        }
        "/api/status" => {
            let snap = scan::scan(root);
            let body = serde_json::to_string_pretty(&snap).unwrap_or_else(|e| {
                format!("{{\"error\":{}}}", serde_json::to_string(&e.to_string()).unwrap())
            });
            respond(stream, "200 OK", "application/json; charset=utf-8", &body)
        }
        "/api/open" => {
            // ?path= encoded absolute path under root or known artifact — open via redirect to file URL is awkward;
            // instead return 404 for unknown and serve report HTML if under target/airbug-report.
            respond(
                stream,
                "404 Not Found",
                "text/plain; charset=utf-8",
                "use /report/ for unit HTML",
            )
        }
        p if p.starts_with("/report/") => serve_unit_report(stream, root, &p["/report/".len()..]),
        _ => respond(stream, "404 Not Found", "text/plain; charset=utf-8", "not found"),
    }
}

fn serve_unit_report(stream: &mut TcpStream, root: &Path, name: &str) -> std::io::Result<()> {
    let safe = name.replace("..", "");
    let path = root.join("target/airbug-report").join(safe);
    if !path.is_file() {
        return respond(stream, "404 Not Found", "text/plain; charset=utf-8", "missing");
    }
    let bytes = std::fs::read(&path)?;
    let mime = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        "application/json; charset=utf-8"
    } else {
        "text/html; charset=utf-8"
    };
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        bytes.len()
    )?;
    stream.write_all(&bytes)
}

fn respond(stream: &mut TcpStream, status: &str, mime: &str, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[allow(dead_code)]
pub fn status_json(root: &Path) -> String {
    let snap: Snapshot = scan::scan(root);
    serde_json::to_string_pretty(&snap).unwrap_or_default()
}

const PAGE: &str = include_str!("../static/dashboard.html");
