//! Minimal localhost HTTP helpers (no framework).
use crate::config::{MAX_BODY_BYTES, MAX_HEADER_BYTES, MAX_OTLP_DECOMPRESSED_BYTES};
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    /// Header names lowercased.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub body_too_large: bool,
}

fn is_loopback_authority(authority: &str) -> bool {
    let authority = authority.trim();
    if let Some(rest) = authority.strip_prefix("[::1]") {
        return rest.is_empty()
            || rest
                .strip_prefix(':')
                .is_some_and(|p| p.parse::<u16>().is_ok());
    }
    let (host, port_ok) = match authority.split_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().is_ok()),
        None => (authority, true),
    };
    port_ok && (host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1")
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Reject DNS-rebinding requests whose `Host` header is not a loopback authority.
    pub fn is_local_host(&self) -> bool {
        match self.header("host") {
            None => true,
            Some(host) => is_loopback_authority(host),
        }
    }

    /// Guard for browser-initiated mutating `POST` requests (while still allowing CLI/SDK
    /// clients that send neither `Origin` nor `Sec-Fetch-Site`).
    pub fn allows_browser_post(&self) -> bool {
        let origin_ok = match self.header("origin") {
            None => true,
            Some(origin) => origin
                .split_once("://")
                .is_some_and(|(_, rest)| is_loopback_authority(rest.trim_end_matches('/'))),
        };
        let sec_fetch_ok = match self.header("sec-fetch-site") {
            None => true,
            Some(site) => {
                let s = site.trim();
                s.eq_ignore_ascii_case("same-origin") || s.eq_ignore_ascii_case("none")
            }
        };
        origin_ok && sec_fetch_ok
    }

    /// Guard for endpoints with side effects beyond the hub's own data (starting
    /// processes): the request must be JSON — which a cross-site form or `no-cors` fetch
    /// cannot send without a CORS preflight the hub never approves — and, when the browser
    /// names an origin, it must be the page the request is addressed to.
    pub fn same_origin_json(&self) -> bool {
        let json = self
            .header("content-type")
            .is_some_and(|v| v.to_ascii_lowercase().starts_with("application/json"));
        let origin_ok = match (self.header("origin"), self.header("host")) {
            (None, _) => self.is_local_host(),
            (Some(origin), Some(host)) => {
                is_loopback_authority(host)
                    && origin.split_once("://").is_some_and(|(_, rest)| {
                        let origin_host = rest.trim_end_matches('/');
                        is_loopback_authority(origin_host)
                            && origin_host.eq_ignore_ascii_case(host.trim())
                    })
            }
            (Some(_), None) => false,
        };
        json && origin_ok && self.allows_browser_post()
    }
}

pub fn read_request(stream: &mut TcpStream) -> std::io::Result<Request> {
    let mut buf = Vec::with_capacity(8192);
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            if buf.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "empty request",
                ));
            }
            break buf.len();
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
        if buf.len() > MAX_HEADER_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "request headers exceed MAX_HEADER_BYTES",
            ));
        }
    };

    let header = String::from_utf8_lossy(&buf[..header_end.min(buf.len())]);
    let mut lines = header.lines();
    let request_line = lines.next().unwrap_or("GET / HTTP/1.1");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let target = parts.next().unwrap_or("/");
    let (raw_path, query) = match target.split_once('?') {
        Some((p, q)) => (p, q.to_string()),
        None => (target, String::new()),
    };
    let path = percent_decode_path(raw_path).unwrap_or_else(|| raw_path.to_string());

    let content_length = header
        .lines()
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            (k.trim().eq_ignore_ascii_case("content-length"))
                .then(|| v.trim().parse::<usize>().ok())?
        })
        .unwrap_or(0);

    let headers = header
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (k, v) = line.split_once(':')?;
            Some((k.trim().to_ascii_lowercase(), v.trim().to_string()))
        })
        .collect();

    let max_body = if is_otlp_wire_path(&path) {
        MAX_OTLP_DECOMPRESSED_BYTES
    } else {
        MAX_BODY_BYTES
    };
    if content_length > max_body {
        return Ok(Request {
            method,
            path,
            query,
            headers,
            body: Vec::new(),
            body_too_large: true,
        });
    }

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
        headers,
        body,
        body_too_large: false,
    })
}

fn is_otlp_wire_path(path: &str) -> bool {
    matches!(
        path,
        "/v1/traces"
            | "/v1/metrics"
            | "/v1/logs"
            | "/v1/profiles"
            | "/v1development/profiles"
            | "/api/v1/otlp/v1/traces"
            | "/api/v1/otlp/v1/metrics"
            | "/api/v1/otlp/v1/logs"
            | "/api/v1/otlp/v1/profiles"
            | "/api/v1/otlp/v1development/profiles"
            | "/api/otlp/v1/traces"
            | "/api/otlp/v1/metrics"
            | "/api/otlp/v1/logs"
            | "/api/otlp/v1/profiles"
            | "/api/otlp/v1development/profiles"
    )
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

pub fn query_f64(query: &str, key: &str) -> Option<f64> {
    query_value(query, key)
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite())
}

pub fn query_str(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key && !v.is_empty()).then(|| percent_decode(v).unwrap_or_else(|| v.to_string()))
    })
}

fn percent_decode(s: &str) -> Option<String> {
    percent_decode_inner(s, true)
}

fn percent_decode_path(s: &str) -> Option<String> {
    percent_decode_inner(s, false)
}

fn percent_decode_inner(s: &str, plus_as_space: bool) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' if plus_as_space => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let h =
                    u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?, 16).ok()?;
                out.push(h);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
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
        Some("log") | Some("txt") | Some("md") => "text/plain; charset=utf-8",
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
