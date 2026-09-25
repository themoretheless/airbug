//! Optional register-then-run against local airbug-hub.
use airbug_bench::{Result, error};
use serde::Deserialize;
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    time::{Duration, Instant},
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
    let mut stream = connect(&host, port)?;
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

/// Total time `register` spends reaching the hub before it gives up.
///
/// A loopback connect is sub-millisecond, so this is slack rather than an expected delay; the
/// point is the ceiling. See [`connect`].
const CONNECT_BUDGET: Duration = Duration::from_secs(2);

/// Connect with a deadline instead of the platform's own timeout.
///
/// `TcpStream::connect` has no deadline: on Windows a port reserved by Hyper-V/WinNAT drops the
/// SYN instead of refusing it, so `--hub` against such a port would hang for ~21 s before
/// reporting what is really a "nothing listening here".
fn connect(host: &str, port: u16) -> Result<TcpStream> {
    let deadline = Instant::now() + CONNECT_BUDGET;
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| error(format!("cannot resolve airbug hub at {host}:{port}: {e}")))?;
    let mut failure = format!("no addresses resolved for {host}:{port}");
    for addr in addrs {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        match TcpStream::connect_timeout(&addr, remaining) {
            Ok(stream) => return Ok(stream),
            Err(e) => failure = format!("{addr}: {e}"),
        }
    }
    Err(error(format!(
        "cannot reach airbug hub at {host}:{port}: {failure}"
    )))
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
            if user_out != Path::new(&reg.out_dir) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Read, net::TcpListener, thread::spawn as spawn_thread};

    /// A hub that answers nothing must cost the run the connect budget, not the platform's TCP
    /// timeout.
    ///
    /// 192.0.2.1 is RFC 5737 TEST-NET-1: nothing replies there, which is what a Windows port
    /// reserved by Hyper-V/WinNAT does to a loopback connect. A refusal would prove little —
    /// only silence reproduces the stall.
    #[test]
    fn an_unreachable_hub_fails_within_the_connect_budget() {
        let start = Instant::now();
        let err = register("http://192.0.2.1:9", "probe", None)
            .unwrap_err()
            .to_string();
        let elapsed = start.elapsed();
        assert!(err.contains("cannot reach airbug hub"), "{err}");
        assert!(
            elapsed < Duration::from_secs(5),
            "register spent {elapsed:?} on a hub that answers nothing"
        );
    }

    #[test]
    fn register_reads_back_a_run_from_a_hub_that_answers() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a stand-in hub");
        let port = listener.local_addr().expect("stand-in hub addr").port();
        let server = spawn_thread(move || {
            let (mut socket, _) = listener.accept().expect("a registration");
            // One fixed read: register never shuts its write side, so read_to_end here would
            // wait for a close that only ever comes from the hub.
            let mut request = [0u8; 1024];
            let read = socket.read(&mut request).expect("the request");
            let head = String::from_utf8_lossy(&request[..read.min(request.len())]);
            assert!(
                head.starts_with("POST /api/v1/bench/runs HTTP/1.1"),
                "unexpected request head: {head}"
            );
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\
                      {\"hub_id\":\"h1\",\"run_id\":\"r1\",\"out_dir\":\"/tmp/r1\",\"dash_url\":\"http://127.0.0.1:8790/\"}",
                )
                .expect("the response");
            drop(socket);
        });

        let reg = register(
            &format!("http://127.0.0.1:{port}"),
            "probe",
            Some("cargo airbug-bench run"),
        )
        .expect("a registration against a live stand-in hub");
        assert_eq!((reg.hub_id.as_str(), reg.run_id.as_str()), ("h1", "r1"));
        assert_eq!(reg.out_dir, "/tmp/r1");
        server.join().expect("the stand-in hub");
    }
}
