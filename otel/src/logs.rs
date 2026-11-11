//! Structured OpenTelemetry Logs with automatic active-span `trace_id` / `span_id` correlation.

use opentelemetry::{
    Array, Context, KeyValue, Value,
    logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _},
    trace::{SpanContext, TraceContextExt},
};
use std::borrow::Cow;

pub use opentelemetry::logs::Severity;

/// Rich structured log specification for [`emit_structured_log`].
#[derive(Debug, Clone)]
pub struct LogRecordSpec<'a> {
    pub scope: &'static str,
    pub severity: Severity,
    pub event_name: Option<&'static str>,
    pub target: Option<Cow<'static, str>>,
    pub body: Cow<'a, str>,
    pub attributes: &'a [KeyValue],
    pub explicit_span_context: Option<SpanContext>,
}

impl<'a> LogRecordSpec<'a> {
    pub fn new(scope: &'static str, severity: Severity, body: &'a str) -> Self {
        Self {
            scope,
            severity,
            event_name: None,
            target: None,
            body: Cow::Borrowed(body),
            attributes: &[],
            explicit_span_context: None,
        }
    }

    pub fn event_name(mut self, name: &'static str) -> Self {
        self.event_name = Some(name);
        self
    }

    pub fn target(mut self, target: impl Into<Cow<'static, str>>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn attributes(mut self, attributes: &'a [KeyValue]) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn span_context(mut self, sc: SpanContext) -> Self {
        self.explicit_span_context = Some(sc);
        self
    }
}

pub(crate) fn value_to_any_value(value: &Value) -> AnyValue {
    match value {
        Value::Bool(v) => AnyValue::Boolean(*v),
        Value::I64(v) => AnyValue::Int(*v),
        Value::F64(v) => AnyValue::Double(*v),
        Value::String(v) => AnyValue::String(v.clone()),
        Value::Array(arr) => match arr {
            Array::Bool(items) => AnyValue::ListAny(Box::new(
                items.iter().copied().map(AnyValue::Boolean).collect(),
            )),
            Array::I64(items) => {
                AnyValue::ListAny(Box::new(items.iter().copied().map(AnyValue::Int).collect()))
            }
            Array::F64(items) => AnyValue::ListAny(Box::new(
                items.iter().copied().map(AnyValue::Double).collect(),
            )),
            Array::String(items) => AnyValue::ListAny(Box::new(
                items.iter().cloned().map(AnyValue::String).collect(),
            )),
            _ => AnyValue::String(value.to_string().into()),
        },
        _ => AnyValue::String(value.to_string().into()),
    }
}

/// Emit a structured OpenTelemetry [`LogRecordSpec`] after [`crate::init`].
///
/// Automatically correlates with the active OpenTelemetry span in [`Context::current`]
/// whenever `explicit_span_context` is `None`.
pub fn emit_structured_log(spec: LogRecordSpec<'_>) {
    let Ok(slot) = crate::logs_slot().lock() else {
        return;
    };
    let Some((_, provider)) = slot.as_ref() else {
        return;
    };
    let logger = provider.logger(spec.scope);
    let mut record = logger.create_log_record();
    let now = std::time::SystemTime::now();
    record.set_timestamp(now);
    record.set_observed_timestamp(now);
    record.set_severity_text(spec.severity.name());
    record.set_severity_number(spec.severity);
    if let Some(event_name) = spec.event_name
        && !event_name.is_empty()
    {
        record.set_event_name(event_name);
    }
    if let Some(target) = spec.target {
        record.set_target(target);
    }
    record.set_body(spec.body.into_owned().into());
    for kv in spec.attributes {
        record.add_attribute(kv.key.clone(), value_to_any_value(&kv.value));
    }

    let resolved_sc = spec.explicit_span_context.or_else(|| {
        Context::map_current(|cx| {
            let sc = cx.span().span_context().clone();
            sc.is_valid().then_some(sc)
        })
    });
    if let Some(sc) = resolved_sc
        && sc.is_valid()
    {
        record.set_trace_context(sc.trace_id(), sc.span_id(), Some(sc.trace_flags()));
    }

    logger.emit(record);
}

/// Emit a log record with structured attributes and optional `event_name` (`"order.placed"` or `None`).
pub fn emit_log_with_attrs(
    scope: &'static str,
    severity: Severity,
    event_name: impl Into<Option<&'static str>>,
    body: impl AsRef<str>,
    attrs: &[KeyValue],
) {
    emit_structured_log(LogRecordSpec {
        scope,
        severity,
        event_name: event_name.into(),
        target: None,
        body: Cow::Borrowed(body.as_ref()),
        attributes: attrs,
        explicit_span_context: None,
    });
}

/// Emit a log record after [`crate::init`] (no-op if telemetry was never installed).
/// Automatically attaches the active span's `trace_id` and `span_id` if inside a span.
pub fn emit_log(scope: &'static str, severity: Severity, body: impl AsRef<str>) {
    emit_structured_log(LogRecordSpec::new(scope, severity, body.as_ref()));
}

/// Convenience: TRACE log.
pub fn log_trace(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Trace, body);
}

/// Convenience: DEBUG log.
pub fn log_debug(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Debug, body);
}

/// Convenience: INFO log.
pub fn log_info(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Info, body);
}

/// Convenience: WARN log.
pub fn log_warn(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Warn, body);
}

/// Convenience: ERROR log.
pub fn log_error(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Error, body);
}

/// Convenience: FATAL log.
pub fn log_fatal(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Fatal, body);
}
