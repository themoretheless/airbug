//! Error / panic reporting for airbug (Rust subset of Sentry ∪ Bugsnag ∪ Rollbar).
//!
//! One [`init`] (or [`init_with_transport`]) per process. Prefer injectable transports in tests.
//!
//! ```ignore
//! let _guard = airbug_err::init(airbug_err::Options::new()
//!     .endpoint("http://127.0.0.1:8790/api/v1/errors")
//!     .release(env!("CARGO_PKG_VERSION"))
//!     .environment("dev"))?;
//! airbug_err::capture_message("something odd");
//! ```
mod event;
mod fingerprint;
mod scope;
mod transport;

pub use event::{
    Breadcrumb, EVENT_SCHEMA_VERSION, Event, Exception, Frame, Severity, Stacktrace, User,
};
pub use scope::{Scope, add_breadcrumb, configure_scope};
pub use transport::{EventTransport, HttpTransport, MemoryTransport, TransportError};

use event::Exception as Ex;
use fingerprint::resolve as resolve_fingerprint;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use uuid::Uuid;

static CLIENT: OnceLock<Mutex<Option<Client>>> = OnceLock::new();
static SEND_FAILURES: AtomicU64 = AtomicU64::new(0);
static LAST_SEND_ERROR: Mutex<Option<String>> = Mutex::new(None);
static TRACE_CONTEXT_HOOK: OnceLock<Mutex<Option<TraceContextHook>>> = OnceLock::new();

/// Hook signature returning `(trace_id, span_id)` for an outgoing [`Event`].
pub type TraceContextHook = fn(&Event) -> Option<(String, String)>;

fn trace_hook_slot() -> &'static Mutex<Option<TraceContextHook>> {
    TRACE_CONTEXT_HOOK.get_or_init(|| Mutex::new(None))
}

/// Register a global trace-context provider hook invoked when capturing events.
///
/// The hook receives a reference to the [`Event`] being captured and may return
/// `(trace_id, span_id)` (as well as record the exception on an active span).
pub fn register_trace_context_hook(hook: TraceContextHook) {
    if let Ok(mut slot) = trace_hook_slot().lock() {
        *slot = Some(hook);
    }
}

/// Alias for [`register_trace_context_hook`].
pub fn set_trace_context_provider(hook: TraceContextHook) {
    register_trace_context_hook(hook);
}

fn client_slot() -> &'static Mutex<Option<Client>> {
    CLIENT.get_or_init(|| Mutex::new(None))
}

/// Client configuration.
#[derive(Clone)]
pub struct Options {
    pub endpoint: String,
    pub hub_id: Option<String>,
    pub release: Option<String>,
    pub environment: Option<String>,
    pub service: Option<String>,
    pub sample_rate: f64,
    pub max_breadcrumbs: usize,
    pub before_send: Option<BeforeSend>,
}

/// Callback that may drop or mutate an event before send.
pub type BeforeSend = std::sync::Arc<dyn Fn(Event) -> Option<Event> + Send + Sync>;

impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options")
            .field("endpoint", &self.endpoint)
            .field("hub_id", &self.hub_id)
            .field("release", &self.release)
            .field("environment", &self.environment)
            .field("service", &self.service)
            .field("sample_rate", &self.sample_rate)
            .field("max_breadcrumbs", &self.max_breadcrumbs)
            .field("before_send", &self.before_send.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint(),
            hub_id: default_hub_id(),
            release: None,
            environment: None,
            service: None,
            sample_rate: 1.0,
            max_breadcrumbs: 100,
            before_send: None,
        }
    }
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn hub_id(mut self, hub_id: impl Into<String>) -> Self {
        self.hub_id = Some(hub_id.into());
        self
    }

    pub fn release(mut self, release: impl Into<String>) -> Self {
        self.release = Some(release.into());
        self
    }

    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn sample_rate(mut self, rate: f64) -> Self {
        self.sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn max_breadcrumbs(mut self, max: usize) -> Self {
        self.max_breadcrumbs = max.max(1);
        self
    }

    pub fn before_send<F>(mut self, f: F) -> Self
    where
        F: Fn(Event) -> Option<Event> + Send + Sync + 'static,
    {
        self.before_send = Some(std::sync::Arc::new(f));
        self
    }
}

fn default_endpoint() -> String {
    std::env::var("AIRBUG_ERR_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:8790/api/v1/errors".into())
}

fn default_hub_id() -> Option<String> {
    std::env::var("AIRBUG_HUB_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| parse_otel_resource_attr("airbug.hub_id"))
}

fn default_run_id() -> Option<String> {
    std::env::var("AIRBUG_RUN_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| parse_otel_resource_attr("airbug.run_id"))
}

fn parse_otel_resource_attr(key: &str) -> Option<String> {
    let raw = std::env::var("OTEL_RESOURCE_ATTRIBUTES").ok()?;
    for pair in raw.split(',') {
        let pair = pair.trim();
        if let Some((k, v)) = pair.split_once('=')
            && k.trim() == key
        {
            let val = v.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

struct Client {
    options: Options,
    transport: Arc<dyn EventTransport>,
}

/// Keeps the client alive; dropping clears the global client.
pub struct Guard {
    _private: (),
}

impl Guard {
    /// Ensure pending work is delivered.
    ///
    /// Delivery is currently synchronous on [`EventTransport::send`], so this
    /// is a documented no-op success. Kept so call sites can flush on shutdown
    /// without caring about the transport implementation.
    pub fn flush(&self) -> Result<(), TransportError> {
        Ok(())
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.flush();
        if let Ok(mut slot) = client_slot().lock() {
            *slot = None;
        }
    }
}

/// Install the global client with the default HTTP transport (and optional panic hook).
///
/// One init per process is the supported model.
pub fn init(options: Options) -> Result<Guard, TransportError> {
    let transport = if let Some(hub_id) = &options.hub_id {
        HttpTransport::with_hub_id(&options.endpoint, Some(hub_id.clone()))?
    } else {
        HttpTransport::new(&options.endpoint)?
    };
    init_with_transport(options, Arc::new(transport))
}

/// Install with an injectable [`EventTransport`] (tests / custom backends).
pub fn init_with_transport(
    options: Options,
    transport: Arc<dyn EventTransport>,
) -> Result<Guard, TransportError> {
    scope::configure_max_breadcrumbs(options.max_breadcrumbs);
    let client = Client { options, transport };
    if let Ok(mut slot) = client_slot().lock() {
        *slot = Some(client);
    }
    #[cfg(feature = "panic")]
    install_panic_hook();
    Ok(Guard { _private: () })
}

/// How many send failures since process start.
pub fn send_failure_count() -> u64 {
    SEND_FAILURES.load(Ordering::Relaxed)
}

/// Last transport error message, if any.
pub fn last_send_error() -> Option<String> {
    LAST_SEND_ERROR.lock().ok().and_then(|g| g.clone())
}

/// Capture a plain message.
pub fn capture_message(message: impl AsRef<str>) -> Option<String> {
    capture_message_with_level(message, Severity::Info)
}

/// Capture a plain message with severity.
pub fn capture_message_with_level(message: impl AsRef<str>, level: Severity) -> Option<String> {
    let mut event = base_event(level);
    event.message = Some(message.as_ref().to_string());
    send_event(event)
}

/// Capture any `std::error::Error` (with backtrace of the capture site).
///
/// The exception type is the concrete Rust type name (via [`std::any::type_name_of_val`]),
/// not a generic `"Error"` stub.
pub fn capture_error<E: std::error::Error>(err: &E) -> Option<String> {
    let mut chain = Vec::new();
    let mut cur: Option<&dyn std::error::Error> = Some(err);
    while let Some(e) = cur {
        chain.push(e.to_string());
        cur = e.source();
    }
    let value = chain.join(": ");
    let mut event = base_event(Severity::Error);
    event.exception = Some(Ex {
        ty: std::any::type_name_of_val(err).to_string(),
        value,
        stacktrace: Some(Stacktrace {
            frames: collect_frames(),
        }),
    });
    event.message = Some(err.to_string());
    send_event(event)
}

/// Capture an `anyhow::Error` (feature `anyhow`).
#[cfg(feature = "anyhow")]
pub fn capture_anyhow(err: &anyhow::Error) -> Option<String> {
    let mut chain = Vec::new();
    for e in err.chain() {
        chain.push(e.to_string());
    }
    let value = chain.join(": ");
    let mut event = base_event(Severity::Error);
    event.exception = Some(Ex {
        ty: "anyhow::Error".into(),
        value,
        stacktrace: Some(Stacktrace {
            frames: collect_frames(),
        }),
    });
    event.message = Some(err.to_string());
    send_event(event)
}

fn base_event(level: Severity) -> Event {
    let (release, environment, service) = with_client(|c| {
        (
            c.options.release.clone(),
            c.options.environment.clone(),
            c.options.service.clone(),
        )
    })
    .unwrap_or((None, None, None));

    let mut event = Event {
        schema_version: EVENT_SCHEMA_VERSION,
        event_id: Uuid::new_v4().to_string(),
        timestamp: iso_now(),
        level,
        message: None,
        release,
        environment,
        service,
        fingerprint: vec!["{{ default }}".into()],
        exception: None,
        breadcrumbs: scope::take_breadcrumbs(),
        tags: BTreeMap::new(),
        user: None,
        extra: BTreeMap::new(),
        contexts: collect_contexts(),
        trace_id: None,
        span_id: None,
    };

    scope::with_scope(|s| {
        event.tags.extend(s.tags.clone());
        event.extra.extend(s.extra.clone());
        if let Some(user) = &s.user {
            event.user = Some(user.clone());
        }
        if let Some(fp) = &s.fingerprint {
            event.fingerprint = fp.clone();
        }
        if let Some(lvl) = s.level {
            event.level = lvl;
        }
        if let Some(tid) = &s.trace_id {
            event.trace_id = Some(tid.clone());
        }
        if let Some(sid) = &s.span_id {
            event.span_id = Some(sid.clone());
        }
    });

    if !event.tags.contains_key("airbug.run_id")
        && let Some(run_id) = default_run_id()
    {
        event.tags.insert("airbug.run_id".into(), run_id);
    }

    event
}

thread_local! {
    static IN_SEND_OR_PANIC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

struct ReentrancyGuard(bool);

impl ReentrancyGuard {
    fn try_enter() -> Option<Self> {
        let already = IN_SEND_OR_PANIC
            .try_with(|c| c.replace(true))
            .unwrap_or(true);
        if already { None } else { Some(Self(true)) }
    }
}

impl Drop for ReentrancyGuard {
    fn drop(&mut self) {
        if self.0 {
            let _ = IN_SEND_OR_PANIC.try_with(|c| c.set(false));
        }
    }
}

fn apply_trace_correlation(event: &mut Event) {
    let hook = trace_hook_slot().lock().ok().and_then(|slot| *slot);
    if let Some(hook) = hook
        && let Some((tid, sid)) = hook(event)
    {
        if event.trace_id.is_none() && !tid.trim().is_empty() {
            event.trace_id = Some(tid);
        }
        if event.span_id.is_none() && !sid.trim().is_empty() {
            event.span_id = Some(sid);
        }
    }

    #[cfg(feature = "otel")]
    {
        use opentelemetry::KeyValue;
        use opentelemetry::trace::{Status, TraceContextExt};

        let cx = opentelemetry::Context::current();
        let span = cx.span();
        let sc = span.span_context();
        if sc.is_valid() {
            if event.trace_id.is_none() {
                event.trace_id = Some(sc.trace_id().to_string());
            }
            if event.span_id.is_none() {
                event.span_id = Some(sc.span_id().to_string());
            }
            if event.exception.is_some() || matches!(event.level, Severity::Error | Severity::Fatal)
            {
                let mut attrs = Vec::with_capacity(3);
                if let Some(ex) = &event.exception {
                    attrs.push(KeyValue::new("exception.type", ex.ty.clone()));
                    attrs.push(KeyValue::new("exception.message", ex.value.clone()));
                    if let Some(st) = &ex.stacktrace
                        && !st.frames.is_empty()
                    {
                        let rendered = st
                            .frames
                            .iter()
                            .take(32)
                            .map(|f| match (&f.filename, f.lineno) {
                                (Some(file), Some(line)) => {
                                    format!("{} ({file}:{line})", f.function)
                                }
                                (Some(file), None) => format!("{} ({file})", f.function),
                                _ => f.function.clone(),
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        attrs.push(KeyValue::new("exception.stacktrace", rendered));
                    }
                } else {
                    attrs.push(KeyValue::new(
                        "exception.type",
                        event.level.as_str().to_string(),
                    ));
                    if let Some(msg) = &event.message {
                        attrs.push(KeyValue::new("exception.message", msg.clone()));
                    }
                }
                span.add_event("exception", attrs);
                span.set_status(Status::error(event.title()));
            }
        }
    }

    if let Some(tid) = &event.trace_id
        && !event.contexts.contains_key("trace")
    {
        let mut trace_obj = serde_json::Map::new();
        trace_obj.insert("trace_id".into(), serde_json::Value::String(tid.clone()));
        if let Some(sid) = &event.span_id {
            trace_obj.insert("span_id".into(), serde_json::Value::String(sid.clone()));
        }
        event
            .contexts
            .insert("trace".into(), serde_json::Value::Object(trace_obj));
    }
}

fn send_event(mut event: Event) -> Option<String> {
    let _reentrancy = ReentrancyGuard::try_enter()?;
    if !should_sample() {
        return None;
    }

    apply_trace_correlation(&mut event);
    event.fingerprint = resolve_fingerprint(&event);

    let before = with_client(|c| c.options.before_send.clone()).flatten();
    if let Some(cb) = before {
        event = cb(event)?;
    }

    let id = event.event_id.clone();
    let transport = with_client(|c| Arc::clone(&c.transport))?;
    match transport.send(&event) {
        Ok(()) => Some(id),
        Err(err) => {
            SEND_FAILURES.fetch_add(1, Ordering::Relaxed);
            if let Ok(mut slot) = LAST_SEND_ERROR.lock() {
                *slot = Some(err.to_string());
            }
            tracing::warn!(error = %err, event_id = %id, "airbug-err failed to send event");
            None
        }
    }
}

fn should_sample() -> bool {
    let rate = with_client(|c| c.options.sample_rate).unwrap_or(1.0);
    if rate >= 1.0 {
        return true;
    }
    if rate <= 0.0 {
        return false;
    }
    let n = (Uuid::new_v4().as_u128() % 10_000) as f64 / 10_000.0;
    n < rate
}

fn with_client<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Client) -> R,
{
    let slot = client_slot().lock().ok()?;
    slot.as_ref().map(f)
}

pub(crate) fn iso_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let (y, mo, d, h, mi, s) = civil_from_days(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{millis:03}Z")
}

fn civil_from_days(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let tod = (secs % 86_400) as u32;
    let h = tod / 3600;
    let mi = (tod % 3600) / 60;
    let s = tod % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mo = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if mo <= 2 { y + 1 } else { y };
    (y, mo, d, h, mi, s)
}

fn collect_contexts() -> BTreeMap<String, serde_json::Value> {
    let mut ctx = BTreeMap::new();
    ctx.insert(
        "os".into(),
        serde_json::json!({
            "family": std::env::consts::FAMILY,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }),
    );
    ctx.insert(
        "rust".into(),
        serde_json::json!({
            "channel": option_env!("RUSTC_CHANNEL").unwrap_or("unknown"),
        }),
    );
    if let Ok(host) = hostname() {
        ctx.insert("device".into(), serde_json::json!({ "hostname": host }));
    }
    ctx
}

fn hostname() -> Result<String, ()> {
    if let Ok(h) = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")) {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    static CACHED_HOST: OnceLock<Option<String>> = OnceLock::new();
    CACHED_HOST
        .get_or_init(|| {
            if let Ok(s) = std::fs::read_to_string("/etc/hostname") {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            let out = std::process::Command::new("hostname").output().ok()?;
            if !out.status.success() {
                return None;
            }
            let s = String::from_utf8(out.stdout).ok()?;
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .clone()
        .ok_or(())
}

fn collect_frames() -> Vec<Frame> {
    let bt = backtrace::Backtrace::new();
    let mut frames = Vec::new();
    for frame in bt.frames() {
        for sym in frame.symbols() {
            let function = sym
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "<unknown>".into());
            let filename = sym.filename().map(|p| p.display().to_string());
            let lineno = sym.lineno();
            let in_app = filename
                .as_deref()
                .map(|f| {
                    !f.contains("/rustc/")
                        && !f.contains("/.cargo/registry/")
                        && !f.contains("library/std")
                        && !f.contains("library/core")
                })
                .unwrap_or(false);
            if function.contains("airbug_err::") || function.contains("backtrace::") {
                continue;
            }
            frames.push(Frame {
                function,
                filename,
                lineno,
                in_app,
            });
        }
    }
    frames
}

#[cfg(feature = "panic")]
fn install_panic_hook() {
    use std::panic;
    use std::sync::atomic::{AtomicBool, Ordering};
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let already_in_send = IN_SEND_OR_PANIC.try_with(|c| c.get()).unwrap_or(true);
        if !already_in_send && with_client(|_| ()).is_some() {
            let payload = panic_payload(info);
            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
            let mut event = base_event(Severity::Fatal);
            event.message = Some(payload.clone());
            event.exception = Some(Ex {
                ty: "panic".into(),
                value: match location {
                    Some(loc) => format!("{payload} at {loc}"),
                    None => payload,
                },
                stacktrace: Some(Stacktrace {
                    frames: collect_frames(),
                }),
            });
            let _ = send_event(event);
        }
        prev(info);
    }));
}

#[cfg(feature = "panic")]
fn panic_payload(info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Box<dyn Any>".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    /// Global client is process-wide; serialize tests that call `init*`.
    static INIT_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn before_send_can_drop() {
        let _lock = INIT_LOCK.lock().unwrap();
        let seen = Arc::new(Mutex::new(0u32));
        let seen2 = Arc::clone(&seen);
        let _guard = init_with_transport(
            Options::new()
                .endpoint("http://127.0.0.1:9/api/v1/errors")
                .before_send(move |_| {
                    *seen2.lock().unwrap() += 1;
                    None
                }),
            Arc::new(MemoryTransport::new()),
        )
        .unwrap();
        assert!(capture_message("nope").is_none());
        assert_eq!(*seen.lock().unwrap(), 1);
    }

    #[test]
    fn injectable_transport_receives_event() {
        let _lock = INIT_LOCK.lock().unwrap();
        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(transport.clone()),
        )
        .unwrap();
        assert!(capture_message("hello").is_some());
        assert_eq!(
            transport.last().and_then(|e| e.message),
            Some("hello".into())
        );
    }

    #[test]
    fn capture_error_uses_concrete_type_name() {
        let _lock = INIT_LOCK.lock().unwrap();
        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(transport.clone()),
        )
        .unwrap();
        let err = io::Error::other("disk full");
        assert!(capture_error(&err).is_some());
        let ty = transport
            .last()
            .and_then(|e| e.exception.map(|ex| ex.ty))
            .unwrap_or_default();
        assert!(
            ty.contains("io") && ty.ends_with("Error"),
            "expected std::io::Error type name, got {ty}"
        );
        assert_ne!(ty, "Error");
    }

    #[test]
    fn guard_flush_succeeds() {
        let _lock = INIT_LOCK.lock().unwrap();
        let guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(MemoryTransport::new()),
        )
        .unwrap();
        assert!(guard.flush().is_ok());
    }

    #[test]
    fn civil_from_days_epoch() {
        let (y, mo, d, h, mi, s) = civil_from_days(0);
        assert_eq!((y, mo, d, h, mi, s), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn http_transport_dedupes_hub_id_in_endpoint() {
        let t = HttpTransport::with_hub_id(
            "http://127.0.0.1:8790/api/v1/errors/abc-123",
            Some("abc-123".into()),
        )
        .unwrap();
        assert_eq!(
            t.resolved_url(),
            "http://127.0.0.1:8790/api/v1/errors/abc-123"
        );

        let t2 = HttpTransport::with_hub_id(
            "http://127.0.0.1:8790/api/v1/errors",
            Some("abc-123".into()),
        )
        .unwrap();
        assert_eq!(
            t2.resolved_url(),
            "http://127.0.0.1:8790/api/v1/errors/abc-123"
        );
    }

    #[test]
    fn panic_in_configure_scope_does_not_double_panic() {
        let _lock = INIT_LOCK.lock().unwrap();
        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(transport.clone()),
        )
        .unwrap();
        let res = std::panic::catch_unwind(|| {
            configure_scope(|_s| {
                panic!("boom inside configure_scope");
            });
        });
        assert!(res.is_err());
        assert_eq!(transport.len(), usize::from(cfg!(feature = "panic")));
    }

    #[test]
    fn panic_in_before_send_does_not_loop() {
        let _lock = INIT_LOCK.lock().unwrap();
        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new()
                .endpoint("unused")
                .before_send(|_| panic!("boom inside before_send")),
            Arc::new(transport.clone()),
        )
        .unwrap();
        let res = std::panic::catch_unwind(|| {
            let _ = capture_message("trigger");
        });
        assert!(res.is_err());
        assert_eq!(transport.len(), 0);
    }

    #[test]
    fn scope_trace_context_populates_event_and_omits_when_none() {
        let _lock = INIT_LOCK.lock().unwrap();
        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(transport.clone()),
        )
        .unwrap();

        configure_scope(|s| s.clear());
        assert!(capture_message("without trace").is_some());
        let plain = transport.last().unwrap();
        assert_eq!(plain.trace_id, None);
        assert_eq!(plain.span_id, None);
        let json = serde_json::to_value(&plain).unwrap();
        assert!(json.get("trace_id").is_none());
        assert!(json.get("span_id").is_none());

        configure_scope(|s| {
            s.set_trace_context(
                "4bf92f3577b34da6a3ce929d0e0e4736",
                Some("00f067aa0ba902b7".into()),
            );
        });
        assert!(capture_message("with scope trace").is_some());
        configure_scope(|s| s.clear());
        let traced = transport.last().unwrap();
        assert_eq!(
            traced.trace_id.as_deref(),
            Some("4bf92f3577b34da6a3ce929d0e0e4736")
        );
        assert_eq!(traced.span_id.as_deref(), Some("00f067aa0ba902b7"));
        assert_eq!(
            traced.contexts["trace"]["trace_id"],
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
    }

    #[test]
    fn custom_trace_context_hook_populates_event() {
        let _lock = INIT_LOCK.lock().unwrap();
        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(transport.clone()),
        )
        .unwrap();

        fn test_hook(_ev: &Event) -> Option<(String, String)> {
            Some((
                "11111111111111111111111111111111".into(),
                "2222222222222222".into(),
            ))
        }
        register_trace_context_hook(test_hook);
        assert!(capture_message("hooked").is_some());
        if let Ok(mut slot) = trace_hook_slot().lock() {
            *slot = None;
        }
        let captured = transport.last().unwrap();
        assert_eq!(
            captured.trace_id.as_deref(),
            Some("11111111111111111111111111111111")
        );
        assert_eq!(captured.span_id.as_deref(), Some("2222222222222222"));
    }

    #[cfg(feature = "otel")]
    #[test]
    fn active_otel_span_correlates_event_and_records_exception() {
        use opentelemetry::trace::{Status, TraceContextExt, Tracer as _, TracerProvider as _};

        let _lock = INIT_LOCK.lock().unwrap();
        let span_exporter = opentelemetry_sdk::trace::InMemorySpanExporter::default();
        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_simple_exporter(span_exporter.clone())
            .build();
        let tracer = provider.tracer("airbug.err.test");

        let transport = MemoryTransport::new();
        let _guard = init_with_transport(
            Options::new().endpoint("unused"),
            Arc::new(transport.clone()),
        )
        .unwrap();

        let (expected_tid, expected_sid) = tracer.in_span("checkout-op", |cx| {
            let sc = cx.span().span_context().clone();
            let err = io::Error::other("card declined");
            assert!(capture_error(&err).is_some());
            (sc.trace_id().to_string(), sc.span_id().to_string())
        });
        provider.force_flush().unwrap();

        let captured = transport.last().unwrap();
        assert_eq!(captured.trace_id.as_deref(), Some(expected_tid.as_str()));
        assert_eq!(captured.span_id.as_deref(), Some(expected_sid.as_str()));

        let spans = span_exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), 1);
        let span = &spans[0];
        assert!(matches!(span.status, Status::Error { .. }));
        assert!(
            span.events.events.iter().any(|ev| {
                ev.name == "exception"
                    && ev
                        .attributes
                        .iter()
                        .any(|kv| kv.key.as_str() == "exception.message")
            }),
            "expected exception event on OTel span, got {:?}",
            span.events
        );
    }
}
