//! Optional structured diagnostics for the web report. Ordinary `cargo test`
//! needs no setup: steps still execute and assertions still assert, but no files
//! are written unless the reporter configures `AIRBUG_REPORT_DIR` for the process.
use std::{
    cell::RefCell,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

const MAX_TEXT: usize = 64 * 1024;
const MAX_ATTACHMENT: usize = 1024 * 1024;
const MAX_ATTACHMENTS: usize = 4 * MAX_ATTACHMENT;
const MAX_EVENTS: u64 = 8 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(1);
static WRITER: Mutex<()> = Mutex::new(());
static ATTACHED_BYTES: Mutex<usize> = Mutex::new(0);
thread_local! { static PARENTS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) }; }

fn directory() -> Option<PathBuf> {
    std::env::var_os("AIRBUG_REPORT_DIR").map(PathBuf::from)
}
fn id() -> u64 {
    NEXT.fetch_add(1, Ordering::Relaxed)
}
fn parent() -> String {
    PARENTS.with(|parents| {
        parents
            .borrow()
            .last()
            .map(u64::to_string)
            .unwrap_or_else(|| "null".into())
    })
}
fn clipped(value: &str) -> &str {
    let mut end = value.len().min(MAX_TEXT);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}
fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch < '\u{20}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
fn emit(event: &str) -> io::Result<()> {
    let Some(dir) = directory() else {
        return Ok(());
    };
    let _guard = WRITER.lock().unwrap_or_else(|e| e.into_inner());
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("events.jsonl"))?;
    if file.metadata()?.len() + event.len() as u64 + 1 > MAX_EVENTS {
        return Err(io::Error::other("Airbug report event limit exceeded"));
    }
    writeln!(file, "{event}")
}
fn record(event: &str) {
    if let Err(error) = emit(event) {
        let message = format!("Airbug diagnostics could not be recorded: {error}");
        let _ = writeln!(io::stderr(), "{message}");
        if let Some(dir) = directory() {
            // A fixed marker lets the collector flag incomplete diagnostics.
            let _ = fs::write(dir.join("report-error.txt"), message);
        }
    }
}

struct Step {
    id: u64,
    started: Instant,
}
impl Step {
    fn new(name: &str) -> Self {
        let id = id();
        record(&format!(
            "{{\"type\":\"step_start\",\"id\":{id},\"parent\":{},\"name\":{}}}",
            parent(),
            quote(clipped(name))
        ));
        PARENTS.with(|parents| parents.borrow_mut().push(id));
        Self {
            id,
            started: Instant::now(),
        }
    }
    fn finish(self, status: &str) {
        PARENTS.with(|parents| {
            parents.borrow_mut().pop();
        });
        record(&format!(
            "{{\"type\":\"step_end\",\"id\":{},\"status\":{},\"duration\":{}}}",
            self.id,
            quote(status),
            self.started.elapsed().as_secs_f64()
        ));
    }
}

/// Record a synchronous step, including nested steps on the same thread.
/// A panic marks the step failed and is resumed with its original payload.
/// To classify `Result::Err` as failed use [`try_step`]. Futures are not polled.
pub fn step<T>(name: &str, action: impl FnOnce() -> T) -> T {
    if directory().is_none() {
        return action();
    }
    let step = Step::new(name);
    match catch_unwind(AssertUnwindSafe(action)) {
        Ok(value) => {
            step.finish("passed");
            value
        }
        Err(payload) => {
            step.finish("failed");
            resume_unwind(payload)
        }
    }
}

/// Like [`step`], but marks a returned error as failed without consuming it.
pub fn try_step<T, E>(name: &str, action: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    if directory().is_none() {
        return action();
    }
    let step = Step::new(name);
    match catch_unwind(AssertUnwindSafe(action)) {
        Ok(value) => {
            step.finish(if value.is_ok() { "passed" } else { "failed" });
            value
        }
        Err(payload) => {
            step.finish("failed");
            resume_unwind(payload)
        }
    }
}

/// Attach UTF-8 text to the current step, or to the test when outside a step.
/// At most 1 MiB per attachment and 4 MiB total per reporting process.
/// Returns file/size errors to the caller; a disabled reporter is a no-op.
pub fn attach_text(name: &str, text: &str) -> io::Result<()> {
    attach_bytes(name, "text/plain; charset=utf-8", text.as_bytes())
}

/// Attach bytes under a display name and media type. Names never become paths.
/// Binary files are download-only; the UI does not execute HTML or SVG content.
pub fn attach_bytes(name: &str, media_type: &str, bytes: &[u8]) -> io::Result<()> {
    let Some(dir) = directory() else {
        return Ok(());
    };
    let mut total = ATTACHED_BYTES.lock().unwrap_or_else(|e| e.into_inner());
    if bytes.len() > MAX_ATTACHMENT || *total + bytes.len() > MAX_ATTACHMENTS {
        return Err(io::Error::other("Airbug attachment size limit exceeded"));
    }
    let id = id();
    let file = format!("attachment-{}-{id}.bin", std::process::id());
    let path = dir.join(&file);
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    output.write_all(bytes)?;
    *total += bytes.len();
    drop(total);
    emit(&format!(
        "{{\"type\":\"attachment\",\"parent\":{},\"name\":{},\"mediaType\":{},\"file\":{},\"size\":{}}}",
        parent(),
        quote(clipped(name)),
        quote(clipped(media_type)),
        quote(&file),
        bytes.len()
    ))
}

pub(crate) fn comparison(name: &str, expected: &str, actual: &str, passed: bool) {
    if directory().is_none() {
        return;
    }
    record(&format!(
        "{{\"type\":\"comparison\",\"parent\":{},\"name\":{},\"expected\":{},\"actual\":{},\"passed\":{passed},\"truncated\":{}}}",
        parent(),
        quote(clipped(name)),
        quote(clipped(expected)),
        quote(clipped(actual)),
        expected.len() > MAX_TEXT || actual.len() > MAX_TEXT
    ));
}

/// Assert text equality and record raw expected/actual text for a line diff.
#[track_caller]
pub fn assert_text_equal(name: &str, expected: &str, actual: &str) {
    let passed = expected == actual;
    comparison(name, expected, actual, passed);
    assert!(passed, "{name}: expected {expected:?}, actual {actual:?}");
}

/// Assert equality and record pretty Debug representations for the report.
#[track_caller]
pub fn assert_equal<T: Debug + PartialEq + ?Sized>(name: &str, expected: &T, actual: &T) {
    let passed = expected == actual;
    if directory().is_some() {
        comparison(
            name,
            &format!("{expected:#?}"),
            &format!("{actual:#?}"),
            passed,
        );
    }
    assert!(passed, "{name}: expected {expected:?}, actual {actual:?}");
}

/// Whether this process was configured to emit structured diagnostics.
pub fn is_enabled() -> bool {
    directory().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_strings_escape_controls_and_preserve_unicode() {
        assert_eq!(quote("\"\\\n\r\t\0雪"), "\"\\\"\\\\\\n\\r\\t\\u0000雪\"");
    }
    #[test]
    fn clipping_never_splits_utf8() {
        let value = "雪".repeat(MAX_TEXT);
        assert!(clipped(&value).len() <= MAX_TEXT);
        assert_eq!(clipped(&value).chars().last(), Some('雪'));
    }
}
