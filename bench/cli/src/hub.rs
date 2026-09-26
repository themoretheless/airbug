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
    stream.set_read_timeout(Some(RESPONSE_BUDGET))?;
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| error(format!("hub register: {host}:{port} gave no response: {e}")))?;
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

/// How long the hub gets to answer once the connection is up.
///
/// A connect deadline alone leaves the other half of the request unbounded: a hub whose handler
/// blocks accepts the socket and then says nothing, and `read_to_end` would wait for it forever.
const RESPONSE_BUDGET: Duration = Duration::from_secs(5);

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
    use std::{
        io::Read,
        net::{Shutdown, TcpListener},
        thread::spawn as spawn_thread,
    };

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

    /// Read one request the way the hub's own parser does: headers, then exactly the body the
    /// `Content-Length` promises.
    ///
    /// A stand-in hub cannot get away with one `read`: `register` writes its header and body as
    /// separate segments, so a single read returns a fragment. It cannot use `read_to_end`
    /// either, because the client waits for the hub to close, not the other way round.
    fn read_request(socket: &mut TcpStream) -> String {
        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 256];
        let head_end = loop {
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
            let read = socket.read(&mut chunk).expect("request headers");
            assert_ne!(read, 0, "the request ended before its headers");
            buf.extend_from_slice(&chunk[..read]);
        };
        let headers = String::from_utf8_lossy(&buf[..head_end]).to_string();
        let length: usize = headers
            .lines()
            .find(|line| line.to_ascii_lowercase().starts_with("content-length:"))
            .and_then(|line| line.split_once(':'))
            .and_then(|(_, value)| value.trim().parse().ok())
            .expect("a Content-Length");
        while buf.len() < head_end + length {
            let read = socket.read(&mut chunk).expect("request body");
            assert_ne!(read, 0, "the request ended before its body");
            buf.extend_from_slice(&chunk[..read]);
        }
        headers
    }

    #[test]
    fn register_reads_back_a_run_from_a_hub_that_answers() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a stand-in hub");
        let port = listener.local_addr().expect("stand-in hub addr").port();
        let server = spawn_thread(move || {
            let (mut socket, _) = listener.accept().expect("a registration");
            let head = read_request(&mut socket);
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
            // FIN, not a bare close: register reads until the peer hangs up, and closing a socket
            // that still holds unread bytes would answer with a reset instead.
            socket.shutdown(Shutdown::Write).expect("a clean goodbye");
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

    /// A hub that takes the connection and then says nothing must cost the response budget, not
    /// the run. A handler blocked inside the hub — the shape #9 fixed in the status route — is
    /// exactly this: the socket is established, so nothing arrives and nothing refuses.
    #[test]
    fn a_hub_that_never_answers_fails_within_the_response_budget() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a silent stand-in hub");
        let port = listener.local_addr().expect("stand-in hub addr").port();
        let (shutdown, held) = std::sync::mpsc::channel::<()>();
        let server = spawn_thread(move || {
            let (_socket, _) = listener.accept().expect("a registration");
            // Dropping `_socket` here would send a FIN and turn the test into an EOF check, so it
            // waits until the test asks for it.
            let _ = held.recv();
        });

        let start = Instant::now();
        let err = register(&format!("http://127.0.0.1:{port}"), "probe", None)
            .unwrap_err()
            .to_string();
        let elapsed = start.elapsed();
        let _ = shutdown.send(());
        assert!(err.contains("gave no response"), "{err}");
        assert!(
            elapsed >= Duration::from_secs(4) && elapsed < Duration::from_secs(10),
            "silent hub cost {elapsed:?}, expected the {RESPONSE_BUDGET:?} response budget"
        );
        server.join().expect("the silent stand-in hub");
    }
}
