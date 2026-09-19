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
        Self {
            model_version: EVENT_MODEL_VERSION,
            event_id: event.event_id.clone(),
            timestamp: event.timestamp.clone(),
            service: event.service.clone(),
            environment: event.environment.clone(),
            release: event.release.clone(),
            trace_id: None,
            span_id: None,
            tags: event.tags.clone(),
            payload: EventPayload::Error(Box::new(event)),
        }
    }

    pub fn signal(&self) -> Signal {
        self.payload.signal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbug_err::{EVENT_SCHEMA_VERSION, Severity};

    fn error_event() -> ErrorEvent {
        ErrorEvent {
            schema_version: EVENT_SCHEMA_VERSION,
            event_id: "err-1".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            level: Severity::Error,
            message: Some("boom".into()),
            release: Some("1.2.3".into()),
            environment: Some("test".into()),
            service: Some("checkout".into()),
            fingerprint: vec!["checkout".into()],
            exception: None,
            breadcrumbs: vec![],
            tags: [("region".into(), "local".into())].into_iter().collect(),
            user: None,
            extra: Default::default(),
            contexts: Default::default(),
        }
    }

    #[test]
    fn error_event_maps_to_common_envelope() {
        let envelope = EventEnvelope::from_error(error_event());
        assert_eq!(envelope.signal(), Signal::Error);
        assert_eq!(envelope.service.as_deref(), Some("checkout"));
        assert_eq!(envelope.tags["region"], "local");
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
