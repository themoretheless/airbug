//! Minimal localhost HTTP helpers (no framework).
use crate::config::{MAX_BODY_BYTES, MAX_HEADER_BYTES};
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: Vec<u8>,
}

pub fn read_request(stream: &mut TcpStream) -> std::io::Result<Request> {
    let mut buf = Vec::with_capacity(8192);
    let mut chunk = [0u8; 4096];
    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buf.len() > MAX_HEADER_BYTES {
            break;
        }
    }

    let header_end = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(buf.len());
    let header = String::from_utf8_lossy(&buf[..header_end.min(buf.len())]);
    let mut lines = header.lines();
    let request_line = lines.next().unwrap_or("GET / HTTP/1.1");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let target = parts.next().unwrap_or("/");
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.to_string(), String::new()),
    };

    let content_length = header
        .lines()
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            (k.eq_ignore_ascii_case("content-length")).then(|| v.trim().parse::<usize>().ok())?
        })
        .unwrap_or(0)
        .min(MAX_BODY_BYTES);

    let mut body = buf[header_end.min(buf.len())..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(content_length);

    Ok(Request {
        method,
        path,
        query,
        body,
    })
}

pub fn query_value(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == key).then_some(v.to_string())
        })
        .next()
}

pub fn query_usize(query: &str, key: &str, default: usize) -> usize {
    query_value(query, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn respond(
    stream: &mut TcpStream,
    status: &str,
    mime: &str,
    body: &str,
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

pub fn respond_bytes(
    stream: &mut TcpStream,
    status: &str,
    mime: &str,
    body: &[u8],
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}

pub fn respond_json(
    stream: &mut TcpStream,
    status: &str,
    value: &impl serde::Serialize,
) -> std::io::Result<()> {
    let body = serde_json::to_string_pretty(value)
        .unwrap_or_else(|e| crate::error::HubError::from(e).json_body());
    respond(stream, status, "application/json; charset=utf-8", &body)
}

pub fn respond_err(
    stream: &mut TcpStream,
    status: &str,
    err: &crate::error::HubError,
) -> std::io::Result<()> {
    respond(
        stream,
        status,
        "application/json; charset=utf-8",
        &err.json_body(),
    )
}

pub fn mime_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => "application/json; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_default() {
        assert_eq!(query_usize("a=1&limit=40", "limit", 10), 40);
        assert_eq!(query_usize("a=1", "limit", 10), 10);
    }
}
