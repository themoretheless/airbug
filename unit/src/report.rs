//! Optional structured diagnostics for the web report. Ordinary `cargo test`
//! needs no setup: steps still execute and assertions still assert, but no files
//! are written unless the reporter configures `AIRBUG_REPORT_DIR` for the process.
use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    fs::{self, OpenOptions},
    future::Future,
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    path::PathBuf,
    pin::Pin,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
    time::Instant,
};

// Shared implementation for the reporter binary and the benchmark harness.
#[doc(hidden)]
pub mod live;

const MAX_TEXT: usize = 64 * 1024;
const MAX_ATTACHMENT: usize = 1024 * 1024;
const MAX_ATTACHMENTS: usize = 4 * MAX_ATTACHMENT;
const MAX_EVENTS: u64 = 8 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(1);
static WRITER: Mutex<()> = Mutex::new(());
static ATTACHED_BYTES: Mutex<usize> = Mutex::new(0);
thread_local! {
    static PARENTS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static TRACE_EMITTED: Cell<bool> = const { Cell::new(false) };
}

fn emit_env_trace_once() {
    let already = TRACE_EMITTED.with(|c| c.get());
    if already {
        return;
    }
    if let Ok(trace_id) = std::env::var("AIRBUG_TRACE_ID") {
        let trace_id = trace_id.trim();
        if !trace_id.is_empty() {
            TRACE_EMITTED.with(|c| c.set(true));
            let span_id = std::env::var("AIRBUG_SPAN_ID")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            trace_context(trace_id, span_id.as_deref());
        }
    }
}

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
/// Who wrote an event: `"pid":…,"bin":…,"test":…`.
///
/// Step ids are only unique inside one process, so the collector keys them by `pid`. `bin`
/// is the test executable's file stem (`mock-1a2b…`), which cargo prints in its
/// `Running … (target/debug/deps/mock-1a2b…)` line. libtest runs every test on a thread
/// named after the test, so the thread name attributes the event to its test; `main` and
/// unnamed threads (doctests, `harness = false`, helper threads) stay unattributed.
/// Under cargo-nextest every test is its own process and `NEXTEST_TEST_NAME` names it, so
/// helper threads count too.
fn origin() -> String {
    static PROCESS: OnceLock<(String, Option<String>)> = OnceLock::new();
    let (process, nextest) = PROCESS.get_or_init(|| {
        let exe = std::env::current_exe().unwrap_or_default();
        let bin = exe.file_stem().unwrap_or_default().to_string_lossy();
        let process = format!("\"pid\":{},\"bin\":{}", std::process::id(), quote(&bin));
        let nextest = std::env::var("NEXTEST_TEST_NAME")
            .ok()
            .filter(|name| !name.is_empty())
            .map(|name| quote(clipped(&name)));
        (process, nextest)
    });
    if let Some(test) = nextest {
        return format!("{process},\"test\":{test}");
    }
    let thread = std::thread::current();
    let test = match thread.name() {
        Some(name) if name != "main" => quote(clipped(name)),
        _ => "null".into(),
    };
    format!("{process},\"test\":{test}")
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
        emit_env_trace_once();
        let id = id();
        record(&format!(
            "{{\"type\":\"step_start\",\"id\":{id},\"parent\":{},\"name\":{},{}}}",
            parent(),
            quote(clipped(name)),
            origin()
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
            "{{\"type\":\"step_end\",\"id\":{},\"status\":{},\"duration\":{},{}}}",
            self.id,
            quote(status),
            self.started.elapsed().as_secs_f64(),
            origin()
        ));
    }
}

struct AsyncStepState {
    id: u64,
    started: Instant,
    finished: bool,
}

impl AsyncStepState {
    fn new(name: &str) -> Option<Self> {
        directory()?;
        emit_env_trace_once();
        let id = id();
        record(&format!(
            "{{\"type\":\"step_start\",\"id\":{id},\"parent\":{},\"name\":{},{}}}",
            parent(),
            quote(clipped(name)),
            origin()
        ));
        Some(Self {
            id,
            started: Instant::now(),
            finished: false,
        })
    }

    fn finish(&mut self, status: &str) {
        if !self.finished {
            self.finished = true;
            record(&format!(
                "{{\"type\":\"step_end\",\"id\":{},\"status\":{},\"duration\":{},{}}}",
                self.id,
                quote(status),
                self.started.elapsed().as_secs_f64(),
                origin()
            ));
        }
    }
}

impl Drop for AsyncStepState {
    fn drop(&mut self) {
        self.finish("cancelled");
    }
}

struct StepFuture<F: Future> {
    fut: Pin<Box<F>>,
    state: Option<AsyncStepState>,
    classify: fn(&F::Output) -> &'static str,
}

impl<F: Future> Future for StepFuture<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let step_id = this.state.as_ref().map(|s| s.id);
        if let Some(id) = step_id {
            PARENTS.with(|parents| parents.borrow_mut().push(id));
        }
        let poll_res = catch_unwind(AssertUnwindSafe(|| this.fut.as_mut().poll(cx)));
        if step_id.is_some() {
            PARENTS.with(|parents| {
                parents.borrow_mut().pop();
            });
        }
        match poll_res {
            Ok(Poll::Ready(output)) => {
                if let Some(state) = &mut this.state {
                    let status = (this.classify)(&output);
                    state.finish(status);
                }
                Poll::Ready(output)
            }
            Ok(Poll::Pending) => Poll::Pending,
            Err(payload) => {
                if let Some(state) = &mut this.state {
                    state.finish("failed");
                }
                resume_unwind(payload)
            }
        }
    }
}

/// Record an asynchronous step across `.await` points, keeping nested steps and
/// attachments parented during each poll without leaving stale parent IDs across yields.
pub fn step_async<F: Future>(name: &str, fut: F) -> impl Future<Output = F::Output> {
    StepFuture {
        fut: Box::pin(fut),
        state: AsyncStepState::new(name),
        classify: |_| "passed",
    }
}

/// Like [`step_async`], but marks a returned `Err` as `"failed"` without consuming it.
pub fn try_step_async<F, T, E>(name: &str, fut: F) -> impl Future<Output = Result<T, E>>
where
    F: Future<Output = Result<T, E>>,
{
    StepFuture {
        fut: Box::pin(fut),
        state: AsyncStepState::new(name),
        classify: |res| if res.is_ok() { "passed" } else { "failed" },
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

/// Attach a JSON document (`application/json`) from a UTF-8 string.
pub fn attach_json(name: &str, json: &str) -> io::Result<()> {
    attach_bytes(name, "application/json", json.as_bytes())
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
    let event = format!(
        "{{\"type\":\"attachment\",\"parent\":{},\"name\":{},\"mediaType\":{},\"file\":{},\"size\":{},{}}}",
        parent(),
        quote(clipped(name)),
        quote(clipped(media_type)),
        quote(&file),
        bytes.len(),
        origin()
    );
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    if let Err(err) = output.write_all(bytes) {
        drop(output);
        let _ = fs::remove_file(&path);
        return Err(err);
    }
    drop(output);
    match emit(&event) {
        Ok(()) => {
            *total += bytes.len();
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&path);
            drop(total);
            let message = format!("Airbug diagnostics could not be recorded: {error}");
            let _ = writeln!(io::stderr(), "{message}");
            let _ = fs::write(dir.join("report-error.txt"), message);
            Err(error)
        }
    }
}

pub(crate) fn comparison(name: &str, expected: &str, actual: &str, passed: bool) {
    if directory().is_none() {
        return;
    }
    record(&format!(
        "{{\"type\":\"comparison\",\"parent\":{},\"name\":{},\"expected\":{},\"actual\":{},\"passed\":{passed},\"truncated\":{},{}}}",
        parent(),
        quote(clipped(name)),
        quote(clipped(expected)),
        quote(clipped(actual)),
        expected.len() > MAX_TEXT || actual.len() > MAX_TEXT,
        origin()
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

/// Record a flaky-label event when reporting is enabled; otherwise a no-op.
pub fn flaky(reason: &str) {
    if directory().is_none() {
        return;
    }
    record(&format!(
        "{{\"type\":\"flaky\",\"parent\":{},\"reason\":{},{}}}",
        parent(),
        quote(clipped(reason)),
        origin()
    ));
}

fn record_meta(field: &str, value: &str) {
    if directory().is_none() {
        return;
    }
    emit_env_trace_once();
    record(&format!(
        "{{\"type\":\"meta\",\"field\":{},\"value\":{},{}}}",
        quote(field),
        quote(clipped(value)),
        origin()
    ));
}

/// Record the TestOps `epic` behavior label for the current test.
pub fn epic(value: &str) {
    record_meta("epic", value);
}

/// Record the TestOps `feature` behavior label for the current test.
pub fn feature(value: &str) {
    record_meta("feature", value);
}

/// Record the TestOps `story` behavior label for the current test.
pub fn story(value: &str) {
    record_meta("story", value);
}

/// Record the TestOps `severity` (`blocker`, `critical`, `normal`, `minor`, `trivial`) for the current test.
pub fn severity(value: &str) {
    record_meta("severity", value);
}

/// Record the TestOps `owner` for the current test.
pub fn owner(value: &str) {
    record_meta("owner", value);
}

/// Record a TestOps `tag` for the current test (may be called multiple times).
pub fn tag(value: &str) {
    record_meta("tag", value);
}

/// Mark the current test as `quarantined` with a reason.
pub fn quarantine(reason: &str) {
    record_meta("quarantined", reason);
}

/// Record an external link (`tms`, `issue`, etc.) for the current test.
pub fn link(kind: &str, id: &str, url: Option<&str>) {
    if directory().is_none() {
        return;
    }
    emit_env_trace_once();
    let url_json = match url {
        Some(u) => quote(clipped(u)),
        None => "null".into(),
    };
    record(&format!(
        "{{\"type\":\"link\",\"kind\":{},\"linkId\":{},\"url\":{url_json},{}}}",
        quote(clipped(kind)),
        quote(clipped(id)),
        origin()
    ));
}

/// Record a TMS (Test Management System) link ID for the current test.
pub fn tms_link(id: &str) {
    link("tms", id, None);
}

/// Record an issue tracker link ID for the current test.
pub fn issue_link(id: &str) {
    link("issue", id, None);
}

/// Record a test parameter `(key, value)` for the current test.
pub fn parameter(key: &str, value: &str) {
    if directory().is_none() {
        return;
    }
    emit_env_trace_once();
    record(&format!(
        "{{\"type\":\"parameter\",\"name\":{},\"value\":{},{}}}",
        quote(clipped(key)),
        quote(clipped(value)),
        origin()
    ));
}

/// Associate an OpenTelemetry `trace_id` (and optional `span_id`) with the current test.
pub fn trace_context(trace_id: &str, span_id: Option<&str>) {
    if directory().is_none() {
        return;
    }
    let span_json = match span_id {
        Some(s) => quote(clipped(s)),
        None => "null".into(),
    };
    record(&format!(
        "{{\"type\":\"trace\",\"traceId\":{},\"spanId\":{span_json},{}}}",
        quote(clipped(trace_id)),
        origin()
    ));
}

/// Literal redaction pairs for [`crate::snapshot::Snapshots::redact`].
pub mod redact {
    /// Common secret patterns as `(from, to)` replacements.
    pub fn secrets() -> Vec<(String, String)> {
        vec![
            ("Bearer ".into(), "Bearer <redacted>".into()),
            ("bearer ".into(), "bearer <redacted>".into()),
            ("Authorization: ".into(), "Authorization: <redacted>".into()),
            ("api_key=".into(), "api_key=<redacted>".into()),
            ("api-key=".into(), "api-key=<redacted>".into()),
            ("token=".into(), "token=<redacted>".into()),
            ("password=".into(), "password=<redacted>".into()),
            ("secret=".into(), "secret=<redacted>".into()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_strings_escape_controls_and_preserve_unicode() {
        assert_eq!(quote("\"\\\n\r\t\0雪"), "\"\\\"\\\\\\n\\r\\t\\u0000雪\"");
    }
    #[test]
    fn origin_names_the_process_and_the_test_thread() {
        let here = origin();
        if std::env::var_os("NEXTEST_TEST_NAME").is_some() {
            // nextest: the process is the test, whatever the thread.
            assert!(
                here.contains("origin_names_the_process_and_the_test_thread"),
                "{here}"
            );
            return;
        }
        assert!(here.starts_with(&format!("\"pid\":{},", std::process::id())));
        // libtest names the test thread after the test.
        assert!(
            here.ends_with(
                "\"test\":\"report::tests::origin_names_the_process_and_the_test_thread\""
            ),
            "{here}"
        );
        let helper = std::thread::spawn(origin).join().unwrap();
        assert!(helper.ends_with("\"test\":null"), "{helper}");
    }
    #[test]
    fn clipping_never_splits_utf8() {
        let value = "雪".repeat(MAX_TEXT);
        assert!(clipped(&value).len() <= MAX_TEXT);
        assert_eq!(clipped(&value).chars().last(), Some('雪'));
    }
    #[test]
    fn attach_bytes_rolls_back_file_and_quota_when_emit_fails() {
        let dir = crate::native::TempDir::new().unwrap();
        let _env = crate::native::EnvScope::set("AIRBUG_REPORT_DIR", dir.path());
        // Pre-fill events.jsonl past MAX_EVENTS so emit() fails.
        let events_path = dir.path().join("events.jsonl");
        let f = fs::File::create(&events_path).unwrap();
        f.set_len(MAX_EVENTS).unwrap();

        let before = *ATTACHED_BYTES.lock().unwrap_or_else(|e| e.into_inner());
        let err = attach_bytes("blob.bin", "application/octet-stream", b"hello").unwrap_err();
        assert!(err.to_string().contains("event limit exceeded"), "{err}");
        let after = *ATTACHED_BYTES.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(before, after);
        assert!(dir.path().join("report-error.txt").is_file());
        let attachments: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("attachment-"))
            .collect();
        assert!(attachments.is_empty(), "{attachments:?}");
    }

    #[test]
    fn step_async_and_try_step_async_record_nested_steps_and_cancellation() {
        use std::task::Waker;

        let dir = crate::native::TempDir::new().unwrap();
        let _env = crate::native::EnvScope::set("AIRBUG_REPORT_DIR", dir.path());

        struct YieldOnce(bool);
        impl Future for YieldOnce {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.0 {
                    Poll::Ready(())
                } else {
                    self.0 = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        let mut fut = std::pin::pin!(step_async("Outer async", async {
            step("Sync child before yield", || {});
            YieldOnce(false).await;
            let res: Result<u32, &str> = try_step_async("Fallible child", async {
                YieldOnce(false).await;
                Err("boom")
            })
            .await;
            assert_eq!(res, Err("boom"));
            42u32
        }));

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        // First poll yields; PARENTS must be empty outside poll!
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(parent(), "null");

        // Second poll yields inside Fallible child; PARENTS must still be empty outside poll!
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(parent(), "null");

        // Third poll completes.
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Ready(42));
        assert_eq!(parent(), "null");

        // Also test dropping an incomplete StepFuture emits "cancelled".
        {
            let mut cancelled = std::pin::pin!(step_async("Cancelled step", YieldOnce(false)));
            assert_eq!(cancelled.as_mut().poll(&mut cx), Poll::Pending);
        }

        let content = fs::read_to_string(dir.path().join("events.jsonl")).unwrap();
        assert!(content.contains("\"name\":\"Outer async\""), "{content}");
        assert!(
            content.contains("\"name\":\"Sync child before yield\""),
            "{content}"
        );
        assert!(content.contains("\"name\":\"Fallible child\""), "{content}");
        assert!(content.contains("\"status\":\"failed\""), "{content}");
        assert!(content.contains("\"status\":\"cancelled\""), "{content}");
    }

    #[test]
    fn testops_metadata_and_trace_context_write_expected_events() {
        let dir = crate::native::TempDir::new().unwrap();
        let _env = crate::native::EnvScope::set("AIRBUG_REPORT_DIR", dir.path());

        epic("Checkout");
        feature("Payments");
        story("Card authorization");
        severity("critical");
        owner("payments-team");
        tag("smoke");
        tms_link("TMS-101");
        issue_link("BUG-202");
        link("docs", "DOC-1", Some("https://example.test/doc"));
        parameter("currency", "USD");
        quarantine("flaky gateway");
        trace_context("4bf92f3577b34da6a3ce929d0e0e4736", Some("00f067aa0ba902b7"));

        let content = fs::read_to_string(dir.path().join("events.jsonl")).unwrap();
        assert!(
            content.contains("\"type\":\"meta\",\"field\":\"epic\",\"value\":\"Checkout\""),
            "{content}"
        );
        assert!(
            content.contains("\"type\":\"meta\",\"field\":\"feature\",\"value\":\"Payments\""),
            "{content}"
        );
        assert!(
            content
                .contains("\"type\":\"meta\",\"field\":\"story\",\"value\":\"Card authorization\""),
            "{content}"
        );
        assert!(
            content.contains("\"type\":\"meta\",\"field\":\"severity\",\"value\":\"critical\""),
            "{content}"
        );
        assert!(
            content.contains("\"type\":\"meta\",\"field\":\"owner\",\"value\":\"payments-team\""),
            "{content}"
        );
        assert!(
            content.contains("\"type\":\"meta\",\"field\":\"tag\",\"value\":\"smoke\""),
            "{content}"
        );
        assert!(
            content.contains(
                "\"type\":\"meta\",\"field\":\"quarantined\",\"value\":\"flaky gateway\""
            ),
            "{content}"
        );
        assert!(
            content
                .contains("\"type\":\"link\",\"kind\":\"tms\",\"linkId\":\"TMS-101\",\"url\":null"),
            "{content}"
        );
        assert!(
            content.contains(
                "\"type\":\"link\",\"kind\":\"issue\",\"linkId\":\"BUG-202\",\"url\":null"
            ),
            "{content}"
        );
        assert!(
            content.contains(
                "\"type\":\"link\",\"kind\":\"docs\",\"linkId\":\"DOC-1\",\"url\":\"https://example.test/doc\""
            ),
            "{content}"
        );
        assert!(
            content.contains("\"type\":\"parameter\",\"name\":\"currency\",\"value\":\"USD\""),
            "{content}"
        );
        assert!(
            content.contains(
                "\"type\":\"trace\",\"traceId\":\"4bf92f3577b34da6a3ce929d0e0e4736\",\"spanId\":\"00f067aa0ba902b7\""
            ),
            "{content}"
        );
    }
}
