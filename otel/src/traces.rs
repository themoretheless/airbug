//! Full OpenTelemetry Tracing helpers: sync & async spans, `SpanOptions`, `SpanLink`,
//! attributes, events, statuses, and semantic `exception` recording.

use opentelemetry::{
    Context, KeyValue, Value, global,
    propagation::{Extractor, TextMapPropagator as _},
    trace::{FutureExt, Link, SpanContext, Status, TraceContextExt, Tracer as _},
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::{borrow::Cow, future::Future};

pub use opentelemetry::trace::SpanKind;

/// Link from a newly created span to an external or causally-related [`SpanContext`].
#[derive(Debug, Clone, PartialEq)]
pub struct SpanLink {
    pub span_context: SpanContext,
    pub attributes: Vec<KeyValue>,
}

struct SingleTraceparentExtractor<'a>(&'a str);

impl Extractor for SingleTraceparentExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        key.eq_ignore_ascii_case("traceparent").then_some(self.0)
    }

    fn keys(&self) -> Vec<&str> {
        vec!["traceparent"]
    }
}

impl SpanLink {
    /// Construct a link to an explicit [`SpanContext`] with attributes.
    pub fn new(span_context: SpanContext, attributes: Vec<KeyValue>) -> Self {
        Self {
            span_context,
            attributes,
        }
    }

    /// Parse a W3C `traceparent` header (`00-<trace_id>-<span_id>-<flags>`) into a [`SpanLink`].
    pub fn from_traceparent(traceparent: &str, attributes: Vec<KeyValue>) -> Option<Self> {
        let prop = TraceContextPropagator::new();
        let cx = prop.extract(&SingleTraceparentExtractor(traceparent.trim()));
        let sc = cx.span().span_context().clone();
        sc.is_valid().then_some(Self {
            span_context: sc,
            attributes,
        })
    }

    /// Capture a [`SpanLink`] pointing to the currently active span, if any.
    pub fn from_current() -> Option<Self> {
        Context::map_current(|cx| {
            let sc = cx.span().span_context().clone();
            sc.is_valid().then_some(Self {
                span_context: sc,
                attributes: Vec::new(),
            })
        })
    }

    /// Attach attributes to this [`SpanLink`].
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }
}

/// Options when starting a span via [`in_span_with_options`] or [`in_span_async_with_options`].
#[derive(Debug, Clone)]
pub struct SpanOptions {
    pub kind: SpanKind,
    pub attributes: Vec<KeyValue>,
    pub links: Vec<SpanLink>,
}

impl Default for SpanOptions {
    fn default() -> Self {
        Self {
            kind: SpanKind::Internal,
            attributes: Vec::new(),
            links: Vec::new(),
        }
    }
}

impl SpanOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn attribute(
        mut self,
        key: impl Into<opentelemetry::Key>,
        value: impl Into<Value>,
    ) -> Self {
        self.attributes.push(KeyValue::new(key, value));
        self
    }

    pub fn attributes(mut self, attrs: impl IntoIterator<Item = KeyValue>) -> Self {
        self.attributes.extend(attrs);
        self
    }

    pub fn link(mut self, link: SpanLink) -> Self {
        self.links.push(link);
        self
    }

    pub fn links(mut self, links: impl IntoIterator<Item = SpanLink>) -> Self {
        self.links.extend(links);
        self
    }
}

fn build_span_context(
    scope: &'static str,
    name: Cow<'static, str>,
    options: SpanOptions,
) -> Context {
    let tracer = global::tracer(scope);
    let mut builder = tracer.span_builder(name).with_kind(options.kind);
    if !options.attributes.is_empty() {
        builder = builder.with_attributes(options.attributes);
    }
    if !options.links.is_empty() {
        let links = options
            .links
            .into_iter()
            .map(|l| Link::new(l.span_context, l.attributes, 0))
            .collect();
        builder = builder.with_links(links);
    }
    Context::map_current(|parent_cx| {
        let span = builder.start_with_context(&tracer, parent_cx);
        parent_cx.with_span(span)
    })
}

struct SpanEndGuard(Context);

impl Drop for SpanEndGuard {
    fn drop(&mut self) {
        self.0.span().end();
    }
}

/// Run `f` inside a named span on the global tracer `scope`.
pub fn in_span<T>(
    scope: &'static str,
    name: impl Into<Cow<'static, str>>,
    f: impl FnOnce() -> T,
) -> T {
    let tracer = global::tracer(scope);
    tracer.in_span(name, |_cx| f())
}

/// Run `f` inside a span configured with [`SpanOptions`].
pub fn in_span_with_options<T>(
    scope: &'static str,
    name: impl Into<Cow<'static, str>>,
    options: SpanOptions,
    f: impl FnOnce() -> T,
) -> T {
    let cx = build_span_context(scope, name.into(), options);
    let _end = SpanEndGuard(cx.clone());
    let _attach = cx.attach();
    f()
}

/// Run `future` inside a named span on `scope`, preserving the active span across `.await` points.
pub async fn in_span_async<F: Future>(
    scope: &'static str,
    name: impl Into<Cow<'static, str>>,
    future: F,
) -> F::Output {
    in_span_async_with_options(scope, name, SpanOptions::default(), future).await
}

/// Run `future` inside a span configured with [`SpanOptions`], preserving the active span across `.await` points.
pub async fn in_span_async_with_options<F: Future>(
    scope: &'static str,
    name: impl Into<Cow<'static, str>>,
    options: SpanOptions,
    future: F,
) -> F::Output {
    let cx = build_span_context(scope, name.into(), options);
    let _end = SpanEndGuard(cx.clone());
    future.with_context(cx).await
}

/// Attach a string attribute to the currently active span (no-op if none).
pub fn set_attribute(key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) {
    Context::map_current(|cx| {
        cx.span()
            .set_attribute(KeyValue::new(key.into(), value.into()));
    });
}

/// Attach multiple typed attributes (`bool`, `i64`, `f64`, `String`, `Value::Array`) to the active span.
pub fn set_attributes(attrs: &[KeyValue]) {
    Context::map_current(|cx| {
        cx.span().set_attributes(attrs.iter().cloned());
    });
}

/// Record a timestamped event on the currently active span.
pub fn add_span_event(name: impl Into<Cow<'static, str>>, attrs: &[KeyValue]) {
    let name = name.into();
    let owned: Vec<KeyValue> = attrs.to_vec();
    Context::map_current(|cx| {
        cx.span().add_event(name, owned);
    });
}

/// Mark the currently active span status as `Ok`.
pub fn set_span_status_ok() {
    Context::map_current(|cx| {
        cx.span().set_status(Status::Ok);
    });
}

/// Mark the currently active span status as `Error` with a description message.
pub fn set_span_status_error(description: impl Into<Cow<'static, str>>) {
    let desc = description.into();
    Context::map_current(|cx| {
        cx.span().set_status(Status::error(desc));
    });
}

/// Dynamically update the name of the currently active span.
pub fn update_span_name(name: impl Into<Cow<'static, str>>) {
    let name = name.into();
    Context::map_current(|cx| {
        cx.span().update_name(name);
    });
}

/// Record a `std::error::Error` (with full `.source()` chain) as a semantic `"exception"` event
/// on the currently active span and mark its status as `Error`.
pub fn record_exception(err: &(dyn std::error::Error + 'static)) {
    let mut chain = Vec::new();
    let mut cur: Option<&dyn std::error::Error> = Some(err);
    while let Some(e) = cur {
        chain.push(e.to_string());
        cur = e.source();
    }
    let msg = chain.join(": ");
    record_exception_message("std::error::Error", msg, None);
}

/// Record an explicit exception (`exception.type`, `exception.message`, optional `exception.stacktrace`)
/// as an `"exception"` event on the currently active span and mark its status as `Error`.
pub fn record_exception_message(
    ty: impl Into<String>,
    message: impl Into<String>,
    stacktrace: Option<&str>,
) {
    let ty = ty.into();
    let message = message.into();
    let mut attrs = vec![
        KeyValue::new("exception.type", ty.clone()),
        KeyValue::new("exception.message", message.clone()),
    ];
    if let Some(st) = stacktrace
        && !st.is_empty()
    {
        attrs.push(KeyValue::new("exception.stacktrace", st.to_string()));
    }
    let summary = if message.is_empty() {
        ty
    } else {
        format!("{ty}: {message}")
    };
    Context::map_current(|cx| {
        let span = cx.span();
        span.add_event("exception", attrs);
        span.set_status(Status::error(summary));
    });
}
