//! Explicit environment policy; literal secret redaction happens before log bytes reach disk.
use rbench::{error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    process::Command,
};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// Inherit only these nonsecret environment variables when a policy is present.
    #[serde(default)]
    pub allow_env: Vec<String>,
    /// Names only: values are obtained from the parent environment, never serialized.
    #[serde(default)]
    pub secret_env: Vec<String>,
}
pub struct Prepared {
    env: BTreeMap<String, String>,
    needles: Vec<Vec<u8>>,
}
impl Policy {
    pub fn prepare(&self, plan: &impl Serialize) -> Result<Prepared> {
        let mut env = BTreeMap::new();
        let mut needles = vec![];
        for name in self.allow_env.iter().chain(&self.secret_env) {
            if name.is_empty()
                || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || env.contains_key(name)
            {
                return Err(error("invalid or duplicate privacy environment name"));
            }
            let value = std::env::var(name).map_err(|_| {
                error(format!(
                    "required environment variable {name} missing or not UTF-8"
                ))
            })?;
            if self.secret_env.contains(name) {
                if !(8..=4096).contains(&value.len()) {
                    return Err(error("secret values must contain 8..4096 bytes"));
                }
                needles.push(value.as_bytes().to_vec());
                let quoted = serde_json::to_string(&value)?;
                needles.push(quoted.as_bytes()[1..quoted.len() - 1].to_vec());
            }
            env.insert(name.clone(), value);
        }
        needles.sort_by_key(|n| std::cmp::Reverse(n.len()));
        needles.dedup();
        let serialized = serde_json::to_vec(plan)?;
        if needles
            .iter()
            .any(|n| serialized.windows(n.len()).any(|w| w == n))
        {
            return Err(error(
                "secret literal found in plan; use secret_env names only",
            ));
        }
        Ok(Prepared { env, needles })
    }
}
impl Prepared {
    pub fn command(&self, c: &mut Command) {
        c.env_clear().envs(&self.env);
    }
    pub fn reject_literals(&self, value: &impl Serialize) -> Result<()> {
        let bytes = serde_json::to_vec(value)?;
        if self
            .needles
            .iter()
            .any(|n| bytes.windows(n.len()).any(|w| w == n))
        {
            return Err(error("secret literal found in persisted metadata"));
        }
        Ok(())
    }
    pub fn environment(&self) -> &BTreeMap<String, String> {
        &self.env
    }
    pub fn needles(&self) -> Vec<Vec<u8>> {
        self.needles.clone()
    }
}
/// Bounded streaming literal replacement, including secrets split across read boundaries.
/// Encoded/transformed secrets are outside this policy; workloads must not emit them.
fn copy_redacted(
    mut input: impl Read,
    mut output: impl Write,
    needles: &[Vec<u8>],
) -> std::io::Result<()> {
    let keep = needles.iter().map(Vec::len).max().unwrap_or(1) - 1;
    let mut pending = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut total = 0usize;
    loop {
        let n = input.read(&mut buffer)?;
        total += n;
        if total > 64 * 1024 * 1024 {
            return Err(std::io::Error::other("worker log exceeds 64 MiB limit"));
        }
        pending.extend_from_slice(&buffer[..n]);
        let end = if n == 0 {
            pending.len()
        } else {
            pending.len().saturating_sub(keep)
        };
        let mut i = 0;
        while i < end {
            if let Some(secret) = needles.iter().find(|s| pending[i..].starts_with(s)) {
                output.write_all(b"[REDACTED]")?;
                i += secret.len();
            } else {
                output.write_all(&pending[i..i + 1])?;
                i += 1;
            }
        }
        pending.drain(..i);
        output.flush()?;
        if n == 0 {
            return Ok(());
        }
    }
}
pub trait LogInput: Read + Send + 'static {
    fn nonblocking(&self) -> std::io::Result<()>;
}
#[cfg(unix)]
impl<T: Read + Send + 'static + std::os::fd::AsRawFd> LogInput for T {
    fn nonblocking(&self) -> std::io::Result<()> {
        // SAFETY: borrowed valid pipe descriptor; only its nonblocking status is changed.
        unsafe {
            let flags = libc::fcntl(self.as_raw_fd(), libc::F_GETFL);
            if flags < 0
                || libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
            {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
}
#[cfg(not(unix))]
impl<T: Read + Send + 'static> LogInput for T {
    fn nonblocking(&self) -> std::io::Result<()> {
        Ok(())
    }
}
struct Stoppable<R> {
    input: R,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stopping: Option<std::time::Instant>,
}
impl<R: Read> Read for Stoppable<R> {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.input.read(b) {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if self.stop.load(std::sync::atomic::Ordering::Acquire)
                        && self
                            .stopping
                            .get_or_insert_with(std::time::Instant::now)
                            .elapsed()
                            > std::time::Duration::from_millis(250)
                    {
                        return Err(std::io::Error::other("worker left inherited log pipe open"));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                result => return result,
            }
        }
    }
}
pub struct Logs(
    Vec<std::thread::JoinHandle<std::io::Result<()>>>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
);
impl Logs {
    pub fn new() -> Self {
        Self(vec![], Default::default())
    }
    pub fn start(
        &mut self,
        input: impl LogInput,
        path: &std::path::Path,
        needles: Vec<Vec<u8>>,
    ) -> Result<()> {
        input.nonblocking()?;
        let input = Stoppable {
            input,
            stop: self.1.clone(),
            stopping: None,
        };
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        self.0.push(
            std::thread::Builder::new()
                .name("rbench-log".into())
                .spawn(move || copy_redacted(input, std::io::BufWriter::new(file), &needles))?,
        );
        Ok(())
    }
    pub fn finish(&mut self) -> Result<()> {
        self.1.store(true, std::sync::atomic::Ordering::Release);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut failed = false;
        for h in self.0.drain(..) {
            while !h.is_finished() && std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            // A blocked non-Unix capture cannot stall the CLI forever; main exits on this error.
            if !h.is_finished() {
                failed = true;
                continue;
            }
            if !matches!(h.join(), Ok(Ok(()))) {
                failed = true;
            }
        }
        if failed {
            return Err(error("worker log capture failed or exceeded limit"));
        }
        Ok(())
    }
}
impl Drop for Logs {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn inherited_open_pipe_does_not_hang_capture() {
        let (reader, _writer) = std::os::unix::net::UnixStream::pair().unwrap();
        let t = tempfile::tempdir().unwrap();
        let mut logs = Logs::new();
        logs.start(reader, &t.path().join("log"), vec![]).unwrap();
        let start = std::time::Instant::now();
        assert!(logs.finish().is_err());
        assert!(start.elapsed() < std::time::Duration::from_secs(3));
    }
    #[test]
    fn split_and_overlapping_literals() {
        struct One(std::io::Cursor<Vec<u8>>);
        impl Read for One {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                self.0.read(&mut b[..1])
            }
        }
        let mut out = vec![];
        copy_redacted(
            One(std::io::Cursor::new(
                b"abSECRET12SECRET1234tailSECRET".to_vec(),
            )),
            &mut out,
            &[b"SECRET1234".to_vec(), b"SECRET12".to_vec()],
        )
        .unwrap();
        assert_eq!(out, b"ab[REDACTED][REDACTED]tailSECRET");
    }
}
