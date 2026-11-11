//! `tracing_subscriber::Layer` bridging `tracing` spans (`#[tracing::instrument]`, `info_span!`)
//! and events (`tracing::info!`, `warn!`, `error!`) into OpenTelemetry spans and structured logs.

use crate::logs::{LogRecordSpec, emit_structured_log};
use opentelemetry::{
    Context, ContextGuard, KeyValue, global,
    logs::Severity,
    trace::{SpanKind, Status, TraceContextExt, Tracer as _},
};
use std::{borrow::Cow, cell::RefCell, fmt};
use tracing::{
    Event, Id, Level, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Record},
};
use tracing_subscriber::{Layer, layer::Context as LayerContext, registry::LookupSpan};

struct OtelSpanState {
    cx: Context,
}

thread_local! {
    static ENTERED_GUARDS: RefCell<Vec<(Id, ContextGuard)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Default)]
struct FieldExtractor {
    attributes: Vec<KeyValue>,
    message: Option<String>,
    otel_name: Option<String>,
    otel_kind: Option<SpanKind>,
    otel_status_error: Option<String>,
    has_error_field: bool,
}

fn parse_span_kind(raw: &str) -> Option<SpanKind> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "client" => Some(SpanKind::Client),
        "server" => Some(SpanKind::Server),
        "producer" => Some(SpanKind::Producer),
        "consumer" => Some(SpanKind::Consumer),
        "internal" => Some(SpanKind::Internal),
        _ => None,
    }
}

impl Visit for FieldExtractor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.attributes
            .push(KeyValue::new(field.name().to_string(), value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if let Ok(signed) = i64::try_from(value) {
            self.attributes
                .push(KeyValue::new(field.name().to_string(), signed));
        } else {
            self.attributes
                .push(KeyValue::new(field.name().to_string(), value.to_string()));
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.attributes
            .push(KeyValue::new(field.name().to_string(), value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.attributes
            .push(KeyValue::new(field.name().to_string(), value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" => self.message = Some(value.to_string()),
            "otel.name" => self.otel_name = Some(value.to_string()),
            "otel.kind" => {
                if let Some(kind) = parse_span_kind(value) {
                    self.otel_kind = Some(kind);
                } else {
                    self.attributes
                        .push(KeyValue::new("otel.kind", value.to_string()));
                }
            }
            "otel.status_code" if value.eq_ignore_ascii_case("error") => {
                self.otel_status_error
                    .get_or_insert_with(|| "ERROR".to_string());
            }
            name => {
                if name == "error" || name == "err" {
                    self.has_error_field = true;
                }
                self.attributes
                    .push(KeyValue::new(name.to_string(), value.to_string()));
            }
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.has_error_field = true;
        let rendered = value.to_string();
        if field.name() == "error" || field.name() == "err" {
            self.otel_status_error = Some(rendered.clone());
        }
        self.attributes
            .push(KeyValue::new(field.name().to_string(), rendered));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let rendered = format!("{value:?}");
        if field.name() == "message" {
            self.message = Some(rendered);
        } else {
            if field.name() == "error" || field.name() == "err" {
                self.has_error_field = true;
            }
            self.attributes
                .push(KeyValue::new(field.name().to_string(), rendered));
        }
    }
}

fn map_level_to_severity(level: Level) -> Severity {
    match level {
        Level::TRACE => Severity::Trace,
        Level::DEBUG => Severity::Debug,
        Level::INFO => Severity::Info,
        Level::WARN => Severity::Warn,
        Level::ERROR => Severity::Error,
    }
}

/// A `tracing_subscriber::Layer` that translates `tracing` spans and events into
/// OpenTelemetry spans, span events, and structured OTel `LogRecord`s.
#[derive(Debug, Clone)]
pub struct TracingOtelLayer {
    scope: &'static str,
}

impl Default for TracingOtelLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TracingOtelLayer {
    pub fn new() -> Self {
        Self {
            scope: "airbug.tracing",
        }
    }

    pub fn with_scope(mut self, scope: &'static str) -> Self {
        self.scope = scope;
        self
    }
}

/// Construct a [`TracingOtelLayer`] ready to attach via `tracing_subscriber::registry().with(airbug_otel::tracing_layer())`.
pub fn tracing_layer() -> TracingOtelLayer {
    TracingOtelLayer::new()
}

impl<S> Layer<S> for TracingOtelLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: LayerContext<'_, S>) {
        if Context::is_current_telemetry_suppressed() {
            return;
        }
        let Some(span_ref) = ctx.span(id) else {
            return;
        };

        let parent_cx = if attrs.is_root() {
            Context::new()
        } else if let Some(parent_id) = attrs.parent() {
            ctx.span(parent_id)
                .and_then(|p| p.extensions().get::<OtelSpanState>().map(|s| s.cx.clone()))
                .unwrap_or_else(Context::current)
        } else if attrs.is_contextual() {
            ctx.lookup_current()
                .and_then(|p| p.extensions().get::<OtelSpanState>().map(|s| s.cx.clone()))
                .unwrap_or_else(Context::current)
        } else {
            Context::current()
        };

        let meta = attrs.metadata();
        let mut extractor = FieldExtractor::default();
        attrs.record(&mut extractor);

        extractor
            .attributes
            .push(KeyValue::new("code.namespace", meta.target()));
        if let Some(file) = meta.file() {
            extractor
                .attributes
                .push(KeyValue::new("code.filepath", file));
        }
        if let Some(line) = meta.line() {
            extractor
                .attributes
                .push(KeyValue::new("code.lineno", i64::from(line)));
        }

        let span_name: Cow<'static, str> = match extractor.otel_name {
            Some(custom) => Cow::Owned(custom),
            None => Cow::Borrowed(meta.name()),
        };

        let tracer = global::tracer(self.scope);
        let mut builder = tracer
            .span_builder(span_name)
            .with_kind(extractor.otel_kind.unwrap_or(SpanKind::Internal))
            .with_attributes(extractor.attributes);
        if let Some(err_msg) = extractor.otel_status_error {
            builder = builder.with_status(Status::error(err_msg));
        }
        let otel_span = builder.start_with_context(&tracer, &parent_cx);
        let cx = parent_cx.with_span(otel_span);

        span_ref.extensions_mut().replace(OtelSpanState { cx });
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: LayerContext<'_, S>) {
        let Some(span_ref) = ctx.span(id) else {
            return;
        };
        let mut extractor = FieldExtractor::default();
        values.record(&mut extractor);

        let extensions = span_ref.extensions();
        let Some(state) = extensions.get::<OtelSpanState>() else {
            return;
        };
        let span = state.cx.span();
        if let Some(name) = extractor.otel_name {
            span.update_name(name);
        }
        if !extractor.attributes.is_empty() {
            span.set_attributes(extractor.attributes);
        }
        if let Some(err_msg) = extractor.otel_status_error {
            span.set_status(Status::error(err_msg));
        }
    }

    fn on_enter(&self, id: &Id, ctx: LayerContext<'_, S>) {
        let Some(span_ref) = ctx.span(id) else {
            return;
        };
        let maybe_cx = span_ref
            .extensions()
            .get::<OtelSpanState>()
            .map(|s| s.cx.clone());
        if let Some(cx) = maybe_cx {
            let guard = cx.attach();
            let _ = ENTERED_GUARDS.try_with(|stack| {
                stack.borrow_mut().push((id.clone(), guard));
            });
        }
    }

    fn on_exit(&self, id: &Id, _ctx: LayerContext<'_, S>) {
        let _ = ENTERED_GUARDS.try_with(|stack| {
            let mut stack = stack.borrow_mut();
            if let Some(pos) = stack.iter().rposition(|(entered_id, _)| entered_id == id) {
                let (_removed_id, guard) = stack.remove(pos);
                drop(guard);
            }
        });
    }

    fn on_event(&self, event: &Event<'_>, ctx: LayerContext<'_, S>) {
        if Context::is_current_telemetry_suppressed() {
            return;
        }
        let meta = event.metadata();
        // Prevent feedback loops from airbug-err transport warnings or internal otel logs.
        if meta.target().starts_with("opentelemetry")
            || meta.target().starts_with("tonic")
            || meta.target().starts_with("hyper")
            || meta.target().starts_with("h2")
            || meta.target().starts_with("reqwest")
        {
            return;
        }

        let mut extractor = FieldExtractor::default();
        event.record(&mut extractor);

        let body = extractor.message.unwrap_or_else(|| meta.name().to_string());

        let active_cx = ctx
            .event_span(event)
            .and_then(|s| {
                s.extensions()
                    .get::<OtelSpanState>()
                    .map(|st| st.cx.clone())
            })
            .unwrap_or_else(Context::current);

        let span = active_cx.span();
        let sc = span.span_context().clone();
        if sc.is_valid() {
            let mut span_attrs = extractor.attributes.clone();
            span_attrs.push(KeyValue::new("level", meta.level().as_str()));
            span_attrs.push(KeyValue::new("target", meta.target()));
            span_attrs.push(KeyValue::new("message", body.clone()));
            span.add_event(meta.name(), span_attrs);
            if *meta.level() == Level::ERROR || extractor.has_error_field {
                span.set_status(Status::error(body.clone()));
            }
        }

        let mut log_attrs = extractor.attributes;
        if let Some(file) = meta.file() {
            log_attrs.push(KeyValue::new("code.filepath", file));
        }
        if let Some(line) = meta.line() {
            log_attrs.push(KeyValue::new("code.lineno", i64::from(line)));
        }

        emit_structured_log(LogRecordSpec {
            scope: self.scope,
            severity: map_level_to_severity(*meta.level()),
            event_name: Some(meta.name()),
            target: Some(Cow::Borrowed(meta.target())),
            body: Cow::Owned(body),
            attributes: &log_attrs,
            explicit_span_context: sc.is_valid().then_some(sc),
        });
    }

    fn on_close(&self, id: Id, ctx: LayerContext<'_, S>) {
        if let Some(span_ref) = ctx.span(&id)
            && let Some(state) = span_ref.extensions_mut().remove::<OtelSpanState>()
        {
            state.cx.span().end();
        }
    }
}
