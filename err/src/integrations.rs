//! Ecosystem integrations for `tracing-subscriber`, framework-agnostic HTTP handlers
//! (Actix/Axum/custom sync & async handlers), and optional `tower` services.

use crate::event::{Exception, Severity, Stacktrace};
use crate::scope::{Scope, snapshot_scope, swap_scope};
use std::collections::BTreeMap;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Request metadata extracted from an incoming HTTP request for breadcrumbs,
/// request context attachment, and 5xx / panic tagging.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpRequestMeta {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub remote_addr: Option<String>,
    pub user_id: Option<String>,
}

impl HttpRequestMeta {
    /// Construct request metadata with method and path.
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            ..Self::default()
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        let q = query.into();
        if !q.is_empty() {
            self.query = Some(q);
        }
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .insert(name.into().to_ascii_lowercase(), value.into());
        self
    }

    pub fn remote_addr(mut self, addr: impl Into<String>) -> Self {
        self.remote_addr = Some(addr.into());
        self
    }

    pub fn user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Serialize into a structured `contexts["request"]` JSON object.
    pub fn to_context_value(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert(
            "method".into(),
            serde_json::Value::String(self.method.clone()),
        );
        let url = match &self.query {
            Some(q) if !q.is_empty() => format!("{}?{q}", self.path),
            _ => self.path.clone(),
        };
        obj.insert("url".into(), serde_json::Value::String(url));
        if let Some(q) = &self.query {
            obj.insert("query_string".into(), serde_json::Value::String(q.clone()));
        }
        if !self.headers.is_empty() {
            let hdrs = self
                .headers
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            obj.insert("headers".into(), serde_json::Value::Object(hdrs));
        }
        if let Some(addr) = &self.remote_addr {
            obj.insert(
                "remote_addr".into(),
                serde_json::Value::String(addr.clone()),
            );
        }
        serde_json::Value::Object(obj)
    }
}

/// Framework-agnostic HTTP middleware helper for capturing panics and HTTP 5xx
/// responses while attaching isolated request scope and breadcrumbs.
#[derive(Debug, Clone)]
pub struct HttpMiddleware {
    pub capture_5xx: bool,
    pub capture_panics: bool,
    pub record_breadcrumbs: bool,
    pub attach_request_context: bool,
}

impl Default for HttpMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMiddleware {
    /// Middleware with all captures and breadcrumbs enabled.
    pub fn new() -> Self {
        Self {
            capture_5xx: true,
            capture_panics: true,
            record_breadcrumbs: true,
            attach_request_context: true,
        }
    }

    pub fn capture_5xx(mut self, enabled: bool) -> Self {
        self.capture_5xx = enabled;
        self
    }

    pub fn capture_panics(mut self, enabled: bool) -> Self {
        self.capture_panics = enabled;
        self
    }

    pub fn record_breadcrumbs(mut self, enabled: bool) -> Self {
        self.record_breadcrumbs = enabled;
        self
    }

    pub fn attach_request_context(mut self, enabled: bool) -> Self {
        self.attach_request_context = enabled;
        self
    }

    /// Derive a per-request [`Scope`] overlaid onto `base`.
    pub fn apply_to_scope(&self, req: &HttpRequestMeta, mut scope: Scope) -> Scope {
        scope.set_tag("http.method", req.method.clone());
        scope.set_tag("http.route", req.path.clone());
        if let Some(uid) = &req.user_id {
            let mut u = scope.user.take().unwrap_or_default();
            if u.id.is_none() {
                u.id = Some(uid.clone());
            }
            scope.user = Some(u);
        }
        if self.attach_request_context {
            scope.set_context("request", req.to_context_value());
        }
        scope
    }

    /// Push an HTTP request breadcrumb onto the global breadcrumb ring.
    pub fn record_request_breadcrumb(&self, req: &HttpRequestMeta) {
        if !self.record_breadcrumbs {
            return;
        }
        let mut data = BTreeMap::new();
        data.insert(
            "method".into(),
            serde_json::Value::String(req.method.clone()),
        );
        data.insert("url".into(), serde_json::Value::String(req.path.clone()));
        if let Some(q) = &req.query {
            data.insert("query".into(), serde_json::Value::String(q.clone()));
        }
        crate::add_breadcrumb_with_data(
            "http",
            Some("http"),
            format!("{} {}", req.method, req.path),
            Severity::Info,
            data,
        );
    }

    /// Execute a synchronous handler within an isolated request scope, capturing
    /// panics and 5xx statuses. Returns `Err(panic_payload)` instead of aborting.
    pub fn run_sync<F, R>(
        &self,
        req: &HttpRequestMeta,
        status_of: impl FnOnce(&R) -> u16,
        f: F,
    ) -> std::thread::Result<R>
    where
        F: FnOnce() -> R + std::panic::UnwindSafe,
    {
        let isolated = self.apply_to_scope(req, snapshot_scope());
        let _guard = swap_scope(isolated);
        self.record_request_breadcrumb(req);

        let result = {
            let _catch_guard = crate::MiddlewareCatchGuard::enter();
            catch_unwind(f)
        };
        match result {
            Ok(res) => {
                let status = status_of(&res);
                if status >= 500 && self.capture_5xx {
                    capture_http_5xx(req, status);
                }
                Ok(res)
            }
            Err(payload) => {
                if self.capture_panics {
                    capture_http_panic(req, &payload);
                }
                Err(payload)
            }
        }
    }

    /// Like [`HttpMiddleware::run_sync`], resuming any caught panic after capture.
    pub fn call_sync<F, R>(
        &self,
        req: &HttpRequestMeta,
        status_of: impl FnOnce(&R) -> u16,
        f: F,
    ) -> R
    where
        F: FnOnce() -> R + std::panic::UnwindSafe,
    {
        match self.run_sync(req, status_of, f) {
            Ok(res) => res,
            Err(payload) => resume_unwind(payload),
        }
    }

    /// Wrap an asynchronous handler future in an isolated per-poll request scope,
    /// capturing panics (`Err(payload)`) and `status >= 500` responses.
    pub fn run_async<Fut, S>(
        &self,
        req: HttpRequestMeta,
        status_of: S,
        fut: Fut,
    ) -> CatchHttpFuture<Fut, S>
    where
        Fut: Future,
        S: FnMut(&Fut::Output) -> Option<u16>,
    {
        let request_scope = self.apply_to_scope(&req, snapshot_scope());
        self.record_request_breadcrumb(&req);
        CatchHttpFuture {
            fut: Box::pin(fut),
            req,
            middleware: self.clone(),
            request_scope,
            status_of: Box::new(status_of),
        }
    }
}

/// Future wrapper returned by [`HttpMiddleware::run_async`].
pub struct CatchHttpFuture<Fut, S> {
    fut: Pin<Box<Fut>>,
    req: HttpRequestMeta,
    middleware: HttpMiddleware,
    request_scope: Scope,
    status_of: Box<S>,
}

impl<Fut, S> Future for CatchHttpFuture<Fut, S>
where
    Fut: Future,
    S: FnMut(&Fut::Output) -> Option<u16>,
{
    type Output = std::thread::Result<Fut::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let _guard = swap_scope(this.request_scope.clone());
        let poll_result = {
            let _catch_guard = crate::MiddlewareCatchGuard::enter();
            catch_unwind(AssertUnwindSafe(|| this.fut.as_mut().poll(cx)))
        };
        match poll_result {
            Ok(Poll::Ready(out)) => {
                if let Some(status) = (this.status_of)(&out)
                    && status >= 500
                    && this.middleware.capture_5xx
                {
                    capture_http_5xx(&this.req, status);
                }
                Poll::Ready(Ok(out))
            }
            Ok(Poll::Pending) => {
                this.request_scope = snapshot_scope();
                Poll::Pending
            }
            Err(payload) => {
                if this.middleware.capture_panics {
                    capture_http_panic(&this.req, &payload);
                }
                Poll::Ready(Err(payload))
            }
        }
    }
}

fn capture_http_5xx(req: &HttpRequestMeta, status: u16) {
    let mut ev = crate::new_event(Severity::Error);
    ev.message = Some(format!(
        "HTTP {} {} returned {status}",
        req.method, req.path
    ));
    ev.tags
        .insert("http.status_code".into(), status.to_string());
    let _ = crate::capture_event(ev);
}

fn capture_http_panic(req: &HttpRequestMeta, payload: &Box<dyn std::any::Any + Send>) {
    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Box<dyn Any>".to_string()
    };
    let mut ev = crate::new_event(Severity::Fatal);
    ev.message = Some(format!(
        "panic handling HTTP {} {}: {msg}",
        req.method, req.path
    ));
    ev.exception = Some(Exception {
        ty: "panic".into(),
        value: msg,
        stacktrace: Some(Stacktrace {
            frames: crate::collect_frames(),
        }),
    });
    let _ = crate::capture_event(ev);
}

/// `tracing-subscriber` layer that forwards events to `airbug-err` and records
/// lower-severity records as breadcrumbs.
#[cfg(feature = "tracing-layer")]
#[derive(Debug, Clone)]
pub struct ErrTracingLayer {
    pub event_level: tracing::Level,
    pub breadcrumb_level: Option<tracing::Level>,
}

#[cfg(feature = "tracing-layer")]
impl Default for ErrTracingLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tracing-layer")]
impl ErrTracingLayer {
    /// Create a layer capturing `ERROR` as error events and `INFO`/`WARN` as breadcrumbs.
    pub fn new() -> Self {
        Self {
            event_level: tracing::Level::ERROR,
            breadcrumb_level: Some(tracing::Level::INFO),
        }
    }

    pub fn with_event_level(mut self, level: tracing::Level) -> Self {
        self.event_level = level;
        self
    }

    pub fn with_breadcrumb_level(mut self, level: Option<tracing::Level>) -> Self {
        self.breadcrumb_level = level;
        self
    }
}

#[cfg(feature = "tracing-layer")]
struct TracingFieldVisitor {
    message: Option<String>,
    fields: BTreeMap<String, serde_json::Value>,
}

#[cfg(feature = "tracing-layer")]
impl tracing::field::Visit for TracingFieldVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if let Some(num) = serde_json::Number::from_f64(value) {
            self.fields
                .insert(field.name().to_string(), serde_json::Value::Number(num));
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let formatted = format!("{value:?}");
        if field.name() == "message" {
            self.message = Some(formatted);
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(formatted),
            );
        }
    }
}

#[cfg(feature = "tracing-layer")]
fn map_tracing_level(level: tracing::Level) -> Severity {
    match level {
        tracing::Level::ERROR => Severity::Error,
        tracing::Level::WARN => Severity::Warning,
        tracing::Level::INFO | tracing::Level::DEBUG | tracing::Level::TRACE => Severity::Info,
    }
}

#[cfg(feature = "tracing-layer")]
impl<S> tracing_subscriber::Layer<S> for ErrTracingLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let target = meta.target();
        if target.starts_with("airbug_err")
            || target.starts_with("reqwest")
            || target.starts_with("hyper")
        {
            return;
        }

        let mut visitor = TracingFieldVisitor {
            message: None,
            fields: BTreeMap::new(),
        };
        event.record(&mut visitor);

        let severity = map_tracing_level(*meta.level());
        let msg = visitor.message.unwrap_or_else(|| meta.name().to_string());

        // `tracing::Level` orders ERROR < WARN < INFO < DEBUG < TRACE.
        if *meta.level() <= self.event_level {
            let mut ev = crate::new_event(severity);
            ev.message = Some(msg.clone());
            ev.tags.insert("tracing.target".into(), target.to_string());
            if let Some(file) = meta.file() {
                ev.tags.insert("tracing.file".into(), file.to_string());
            }
            ev.extra.extend(visitor.fields);
            if matches!(severity, Severity::Error | Severity::Fatal) {
                ev.exception = Some(Exception {
                    ty: format!("tracing::{target}"),
                    value: msg,
                    stacktrace: Some(Stacktrace {
                        frames: crate::collect_frames(),
                    }),
                });
            }
            let _ = crate::capture_event(ev);
        } else if let Some(bc_level) = self.breadcrumb_level
            && *meta.level() <= bc_level
        {
            visitor
                .fields
                .insert("target".into(), serde_json::Value::String(target.into()));
            crate::add_breadcrumb_with_data(
                "tracing",
                Some("default"),
                msg,
                severity,
                visitor.fields,
            );
        }
    }
}

/// Tower `Layer` attaching [`HttpMiddleware`] to any `http::Request` service.
#[cfg(feature = "tower")]
#[derive(Debug, Clone, Default)]
pub struct AirbugTowerLayer {
    pub middleware: HttpMiddleware,
}

#[cfg(feature = "tower")]
impl AirbugTowerLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_middleware(middleware: HttpMiddleware) -> Self {
        Self { middleware }
    }
}

#[cfg(feature = "tower")]
impl<S> tower_layer::Layer<S> for AirbugTowerLayer {
    type Service = AirbugTowerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AirbugTowerService {
            inner,
            middleware: self.middleware.clone(),
        }
    }
}

/// Tower `Service` wrapper produced by [`AirbugTowerLayer`].
#[cfg(feature = "tower")]
#[derive(Debug, Clone)]
pub struct AirbugTowerService<S> {
    inner: S,
    middleware: HttpMiddleware,
}

#[cfg(feature = "tower")]
impl<S, ReqBody, ResBody: 'static> tower_service::Service<http::Request<ReqBody>>
    for AirbugTowerService<S>
where
    S: tower_service::Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = http::Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let mut meta = HttpRequestMeta::new(req.method().as_str(), req.uri().path());
        if let Some(q) = req.uri().query() {
            meta = meta.query(q);
        }
        for (name, val) in req.headers() {
            if let Ok(s) = val.to_str() {
                meta = meta.header(name.as_str(), s);
            }
        }
        let middleware = self.middleware.clone();
        let fut = self.inner.call(req);
        let wrapped = middleware.run_async(
            meta,
            |res: &Result<http::Response<ResBody>, S::Error>| {
                res.as_ref().ok().map(|r| r.status().as_u16())
            },
            fut,
        );
        Box::pin(async move {
            match wrapped.await {
                Ok(result) => result,
                Err(payload) => resume_unwind(payload),
            }
        })
    }
}

#[cfg(all(test, any(feature = "tracing-layer", feature = "tower")))]
mod tests {
    #[cfg(feature = "tracing-layer")]
    #[test]
    fn tracing_layer_captures_errors_and_breadcrumbs() {
        use super::ErrTracingLayer;
        use crate::{MemoryTransport, Options, Severity, TEST_INIT_LOCK, init_with_transport};
        use std::sync::Arc;
        use tracing_subscriber::layer::SubscriberExt;

        let _lock = TEST_INIT_LOCK.lock().unwrap();
        let transport = Arc::new(MemoryTransport::new());
        let _guard = init_with_transport(
            Options::new().endpoint("http://127.0.0.1:9/api/v1/errors"),
            transport.clone(),
        )
        .unwrap();

        let subscriber = tracing_subscriber::registry().with(ErrTracingLayer::new());
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "checkout_svc", route = "/checkout", "step reached");
            tracing::error!(target: "checkout_svc", order_id = 42, "payment gateway failed");
        });

        let events = transport.events();
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev.level, Severity::Error);
        assert_eq!(ev.message.as_deref(), Some("payment gateway failed"));
        assert_eq!(ev.extra.get("order_id"), Some(&serde_json::json!(42)));
        assert!(
            ev.breadcrumbs
                .iter()
                .any(|b| b.category == "tracing" && b.message == "step reached")
        );
    }

    #[cfg(feature = "tower")]
    #[test]
    fn tower_layer_captures_5xx_and_attaches_request_context() {
        use super::AirbugTowerLayer;
        use crate::{MemoryTransport, Options, Severity, TEST_INIT_LOCK, init_with_transport};
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::Arc;
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        use tower_layer::Layer;
        use tower_service::Service;

        fn dummy_waker() -> Waker {
            unsafe fn clone(_: *const ()) -> RawWaker {
                RawWaker::new(std::ptr::null(), &VTABLE)
            }
            unsafe fn noop(_: *const ()) {}
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
            unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
        }

        #[derive(Clone)]
        struct EchoSvc(u16);
        impl Service<http::Request<()>> for EchoSvc {
            type Response = http::Response<()>;
            type Error = std::convert::Infallible;
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, _req: http::Request<()>) -> Self::Future {
                let status = self.0;
                Box::pin(
                    async move { Ok(http::Response::builder().status(status).body(()).unwrap()) },
                )
            }
        }

        let _lock = TEST_INIT_LOCK.lock().unwrap();
        let transport = Arc::new(MemoryTransport::new());
        let _guard = init_with_transport(
            Options::new().endpoint("http://127.0.0.1:9/api/v1/errors"),
            transport.clone(),
        )
        .unwrap();

        let mut svc = AirbugTowerLayer::new().layer(EchoSvc(503));
        let req = http::Request::builder()
            .method("POST")
            .uri("/api/orders?token=secret123")
            .header("x-request-id", "req-9")
            .body(())
            .unwrap();
        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(svc.poll_ready(&mut cx), Poll::Ready(Ok(()))));
        let mut fut = svc.call(req);
        let resp = match fut.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(r)) => r,
            _ => panic!("expected ready response"),
        };
        assert_eq!(resp.status().as_u16(), 503);

        let events = transport.events();
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev.level, Severity::Error);
        assert_eq!(ev.tags.get("http.method").map(String::as_str), Some("POST"));
        assert_eq!(
            ev.tags.get("http.status_code").map(String::as_str),
            Some("503")
        );
        assert!(ev.contexts.contains_key("request"));
    }
}
