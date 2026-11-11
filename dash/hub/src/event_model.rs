//! Shared event envelope for the Airbug collector.
//!
//! Signal-specific payloads keep their native shape, while the envelope gives
//! errors, traces, metrics, and logs common correlation and service fields.
use airbug_err::Event as ErrorEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const EVENT_MODEL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    Error,
    Trace,
    Metric,
    Log,
}

impl Signal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Trace => "trace",
            Self::Metric => "metric",
            Self::Log => "log",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "signal", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    Error(Box<ErrorEvent>),
    Trace {
        trace_id: String,
        span_id: String,
        parent_span_id: Option<String>,
        name: String,
        attributes: Value,
    },
    Metric {
        name: String,
        value: f64,
        kind: String,
        attributes: Value,
    },
    Log {
        body: String,
        severity: String,
        attributes: Value,
    },
}

impl EventPayload {
    pub fn signal(&self) -> Signal {
        match self {
            Self::Error(_) => Signal::Error,
            Self::Trace { .. } => Signal::Trace,
            Self::Metric { .. } => Signal::Metric,
            Self::Log { .. } => Signal::Log,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub model_version: u32,
    pub event_id: String,
    pub timestamp: String,
    pub service: Option<String>,
    pub environment: Option<String>,
    pub release: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub tags: std::collections::BTreeMap<String, String>,
    pub payload: EventPayload,
}

impl EventEnvelope {
    pub fn from_error(event: ErrorEvent) -> Self {
        let direct_trace = non_zero_id(event.trace_id.as_deref());
        let direct_span = non_zero_id(event.span_id.as_deref());
        let (trace_id, span_id) = if direct_trace.is_some() {
            (direct_trace, direct_span)
        } else {
            let (fb_t, fb_s) = serde_json::to_value(&event)
                .ok()
                .map(|v| extract_trace_context_from_event_json(&v))
                .unwrap_or((None, None));
            (fb_t, direct_span.or(fb_s))
        };
        Self {
            model_version: EVENT_MODEL_VERSION,
            event_id: event.event_id.clone(),
            timestamp: event.timestamp.clone(),
            service: event.service.clone(),
            environment: event.environment.clone(),
            release: event.release.clone(),
            trace_id,
            span_id,
            tags: event.tags.clone(),
            payload: EventPayload::Error(Box::new(event)),
        }
    }

    pub fn signal(&self) -> Signal {
        self.payload.signal()
    }

    pub fn match_tags(&self, tag_filters: &[(String, String)]) -> bool {
        tag_filters
            .iter()
            .all(|(key, expected)| self.tags.get(key).is_some_and(|value| value == expected))
    }
}

fn non_zero_id(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() || trimmed.chars().all(|c| c == '0') {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

/// Extract `(trace_id, span_id)` from a serialized `airbug-err::Event` JSON object,
/// checking top-level `trace_id`/`span_id`, `contexts.trace.*`, `tags.*`, and `extra.*`.
pub fn extract_trace_context_from_event_json(val: &Value) -> (Option<String>, Option<String>) {
    let trace_id = non_zero_id(val.get("trace_id").and_then(Value::as_str))
        .or_else(|| {
            non_zero_id(
                val.pointer("/contexts/trace/trace_id")
                    .and_then(Value::as_str),
            )
        })
        .or_else(|| non_zero_id(val.pointer("/tags/trace_id").and_then(Value::as_str)))
        .or_else(|| non_zero_id(val.pointer("/tags/otel.trace_id").and_then(Value::as_str)))
        .or_else(|| non_zero_id(val.pointer("/extra/trace_id").and_then(Value::as_str)));

    let span_id = non_zero_id(val.get("span_id").and_then(Value::as_str))
        .or_else(|| {
            non_zero_id(
                val.pointer("/contexts/trace/span_id")
                    .and_then(Value::as_str),
            )
        })
        .or_else(|| non_zero_id(val.pointer("/tags/span_id").and_then(Value::as_str)))
        .or_else(|| non_zero_id(val.pointer("/tags/otel.span_id").and_then(Value::as_str)))
        .or_else(|| non_zero_id(val.pointer("/extra/span_id").and_then(Value::as_str)));

    (trace_id, span_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbug_err::EVENT_SCHEMA_VERSION;

    fn error_event() -> ErrorEvent {
        ErrorEvent {
            schema_version: EVENT_SCHEMA_VERSION,
            event_id: "err-1".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            level: airbug_err::Severity::Error,
            message: Some("boom".into()),
            release: Some("1.2.3".into()),
            environment: Some("test".into()),
            service: Some("checkout".into()),
            fingerprint: vec!["checkout".into()],
            exception: None,
            breadcrumbs: vec![],
            tags: std::collections::BTreeMap::from([("region".into(), "local".into())]),
            user: None,
            extra: Default::default(),
            contexts: Default::default(),
            trace_id: Some("5b8efff798038103d269b633813fc60c".into()),
            span_id: Some("eee19b7ec3c1b174".into()),
        }
    }

    #[test]
    fn error_event_maps_to_common_envelope() {
        let envelope = EventEnvelope::from_error(error_event());
        assert_eq!(envelope.signal(), Signal::Error);
        assert_eq!(envelope.service.as_deref(), Some("checkout"));
        assert_eq!(envelope.tags["region"], "local");
        assert_eq!(
            envelope.trace_id.as_deref(),
            Some("5b8efff798038103d269b633813fc60c")
        );
        assert_eq!(envelope.span_id.as_deref(), Some("eee19b7ec3c1b174"));
    }

    #[test]
    fn extract_trace_context_checks_top_level_contexts_and_tags() {
        let v1 = serde_json::json!({
            "trace_id": "5B8EFFF798038103D269B633813FC60C",
            "span_id": "EEE19B7EC3C1B174"
        });
        assert_eq!(
            extract_trace_context_from_event_json(&v1),
            (
                Some("5b8efff798038103d269b633813fc60c".into()),
                Some("eee19b7ec3c1b174".into())
            )
        );

        let v2 = serde_json::json!({
            "trace_id": "00000000000000000000000000000000",
            "contexts": {
                "trace": {
                    "trace_id": "11112222333344445555666677778888",
                    "span_id": "9999aaaabbbbcccc"
                }
            }
        });
        assert_eq!(
            extract_trace_context_from_event_json(&v2),
            (
                Some("11112222333344445555666677778888".into()),
                Some("9999aaaabbbbcccc".into())
            )
        );

        let v3 = serde_json::json!({
            "tags": {
                "trace_id": "aaaabbbbccccddddeeeeffff00001111",
                "span_id": "1234567890abcdef"
            }
        });
        assert_eq!(
            extract_trace_context_from_event_json(&v3),
            (
                Some("aaaabbbbccccddddeeeeffff00001111".into()),
                Some("1234567890abcdef".into())
            )
        );
    }

    #[test]
    fn envelope_round_trips_all_signal_shapes() {
        let envelope = EventEnvelope {
            model_version: EVENT_MODEL_VERSION,
            event_id: "trace-1".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            service: Some("checkout".into()),
            environment: None,
            release: None,
            trace_id: Some("abc".into()),
            span_id: Some("def".into()),
            tags: Default::default(),
            payload: EventPayload::Trace {
                trace_id: "abc".into(),
                span_id: "def".into(),
                parent_span_id: None,
                name: "checkout".into(),
                attributes: serde_json::json!({"http.method": "GET"}),
            },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.event_id, envelope.event_id);
        assert_eq!(decoded.trace_id, envelope.trace_id);
        assert_eq!(decoded.signal(), envelope.signal());
        assert_eq!(decoded.signal(), Signal::Trace);
    }
}
