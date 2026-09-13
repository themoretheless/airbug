//! Error event payload shared with airbug-hub `/api/v1/errors`.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current major schema for [`Event`]. Hub rejects unsupported majors.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    EVENT_SCHEMA_VERSION
}

/// Severity levels aligned with Bugsnag / Rollbar / Sentry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Fatal,
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fatal => "fatal",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// One stack frame (newest first, matching typical backtrace order).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Frame {
    pub function: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineno: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub in_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stacktrace {
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exception {
    #[serde(rename = "type")]
    pub ty: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stacktrace: Option<Stacktrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Breadcrumb {
    pub timestamp: String,
    pub category: String,
    pub message: String,
    pub level: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Full event posted to the hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Contract major. Defaults to [`EVENT_SCHEMA_VERSION`] when omitted.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub event_id: String,
    pub timestamp: String,
    pub level: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    pub fingerprint: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception: Option<Exception>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumbs: Vec<Breadcrumb>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contexts: BTreeMap<String, serde_json::Value>,
}

impl Event {
    /// Short title for issue lists.
    pub fn title(&self) -> String {
        if let Some(ex) = &self.exception {
            if ex.value.is_empty() {
                ex.ty.clone()
            } else {
                format!("{}: {}", ex.ty, truncate(&ex.value, 120))
            }
        } else if let Some(msg) = &self.message {
            truncate(msg, 140)
        } else {
            "untitled event".into()
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
