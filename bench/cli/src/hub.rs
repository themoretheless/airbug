//! Optional register-then-run against local airbug-hub.
use airbug_bench::{Result, error};
use serde::Deserialize;
use std::{
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
};

#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub hub_id: String,
    pub run_id: String,
    pub out_dir: String,
    pub dash_url: String,
}

pub fn register(hub_base: &str, title: &str, command: Option<&str>) -> Result<RegisterResponse> {
    let base = hub_base.trim_end_matches('/');
    let url = format!("{base}/api/v1/bench/runs");
    let body = serde_json::json!({
        "title": title,
        "command": command,
    });
    let body = serde_json::to_vec(&body)?;
    let (host, port, path) = parse_http_url(&url)?;
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|e| error(format!("cannot reach airbug hub at {host}:{port}: {e}")))?;
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);
    let Some(idx) = text.find("\r\n\r\n") else {
        return Err(error("hub register: malformed HTTP response"));
    };
    let (headers, payload) = text.split_at(idx + 4);
    if !headers.contains("200") {
        return Err(error(format!(
            "hub register failed: {}",
            payload.trim().chars().take(200).collect::<String>()
        )));
    }
    serde_json::from_str(payload.trim()).map_err(|e| error(format!("hub register JSON: {e}")))
}

pub fn apply_otel_resource(hub_id: &str, run_id: &str) {
    let extra = format!("airbug.hub_id={hub_id},airbug.run_id={run_id}");
    let merged = match std::env::var("OTEL_RESOURCE_ATTRIBUTES") {
        Ok(existing) if !existing.is_empty() => format!("{existing},{extra}"),
        _ => extra,
    };
    // SAFETY: single-threaded CLI before spawning workers; sets correlation for children.
    unsafe {
        std::env::set_var("OTEL_RESOURCE_ATTRIBUTES", merged);
    }
}

fn parse_http_url(url: &str) -> Result<(String, u16, String)> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| error("AIRBUG_HUB / --hub must be http://127.0.0.1:…"))?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let path = if path.is_empty() {
        "/".into()
    } else {
        format!("/{path}")
    };
    let (host, port) = if let Some((h, p)) = authority.split_once(':') {
        (
            h.to_string(),
            p.parse().map_err(|_| error("invalid hub port"))?,
        )
    } else {
        (authority.to_string(), 80)
    };
    Ok((host, port, path))
}

pub fn resolve_output(
    hub: Option<&str>,
    output: Option<PathBuf>,
    title: &str,
    command: Option<&str>,
) -> Result<(PathBuf, Option<RegisterResponse>)> {
    if let Some(hub) = hub {
        let reg = register(hub, title, command)?;
        eprintln!("airbug dash: {}", reg.dash_url);
        apply_otel_resource(&reg.hub_id, &reg.run_id);
        if let Some(user_out) = output {
            // Prefer hub-issued directory; warn if user passed a different -o.
            if user_out != PathBuf::from(&reg.out_dir) {
                eprintln!(
                    "bench: ignoring --output {} in favor of hub out_dir {}",
                    user_out.display(),
                    reg.out_dir
                );
            }
        }
        return Ok((PathBuf::from(reg.out_dir.clone()), Some(reg)));
    }
    let out = output.ok_or_else(|| error("provide --output or set AIRBUG_HUB / --hub"))?;
    Ok((out, None))
}
