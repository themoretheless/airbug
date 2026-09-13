//! Shared OTLP JSON file helpers (DRY for logs + metrics parsers).
mod logs;
mod metrics;

pub use logs::read_recent as read_logs;
pub use metrics::read_recent as read_metrics;

use serde_json::Value;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

/// Read the tail of a file (up to `max_bytes`) and return a JSON-safe slice.
pub fn tail_text(path: &Path, max_bytes: u64) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if start > 0 {
        if let Some(i) = buf.find('{') {
            return Ok(buf[i..].to_string());
        }
    }
    Ok(buf)
}

/// Split NDJSON or a single JSON document into parseable chunks.
pub fn split_json_values(input: &str) -> Vec<&str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return vec![];
    }
    if trimmed
        .lines()
        .filter(|l| l.trim().starts_with('{'))
        .count()
        > 1
    {
        return trimmed
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with('{'))
            .collect();
    }
    vec![trimmed]
}

pub fn resource_attr(resource: &Value, key: &str) -> String {
    let attrs = resource
        .get("resource")
        .and_then(|r| r.get("attributes"))
        .and_then(|a| a.as_array());
    let Some(attrs) = attrs else {
        return String::new();
    };
    for attr in attrs {
        let k = attr.get("key").and_then(|v| v.as_str()).unwrap_or("");
        if k == key {
            return attr.get("value").map(any_value).unwrap_or_default();
        }
    }
    String::new()
}

pub fn any_value(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    if let Some(obj) = v.as_object() {
        if let Some(s) = obj
            .get("stringValue")
            .or_else(|| obj.get("string_value"))
            .and_then(|x| x.as_str())
        {
            return s.to_string();
        }
        if let Some(n) = obj.get("intValue").or_else(|| obj.get("int_value")) {
            return n.to_string();
        }
        if let Some(n) = obj.get("doubleValue").or_else(|| obj.get("double_value")) {
            return n.to_string();
        }
    }
    v.to_string()
}

/// Parse OTLP unix-nano field → (label, time_ms).
pub fn nano_time(value: &Value, keys: &[&str]) -> (String, Option<u64>) {
    let mut raw = None;
    for key in keys {
        raw = value.get(*key).and_then(|v| {
            v.as_str()
                .map(str::to_string)
                .or_else(|| v.as_u64().map(|n| n.to_string()))
        });
        if raw.is_some() {
            break;
        }
    }
    match raw {
        Some(nanos) => {
            let ms = nanos.parse::<u128>().ok().map(|n| (n / 1_000_000) as u64);
            (ms.map(|m| m.to_string()).unwrap_or(nanos), ms)
        }
        None => (String::new(), None),
    }
}

pub fn normalize_severity(sev: &str) -> &'static str {
    let s = sev.to_ascii_uppercase();
    if s.contains("FATAL") || s.contains("CRITICAL") {
        "FATAL"
    } else if s.contains("ERROR") || s.contains("ERR") {
        "ERROR"
    } else if s.contains("WARN") {
        "WARN"
    } else if s.contains("DEBUG") {
        "DEBUG"
    } else if s.contains("TRACE") {
        "TRACE"
    } else {
        "INFO"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ndjson() {
        let chunks = split_json_values("{\"a\":1}\n{\"b\":2}\n");
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn severity_normalize() {
        assert_eq!(normalize_severity("error"), "ERROR");
        assert_eq!(normalize_severity("Warning"), "WARN");
    }

    #[test]
    fn any_value_string() {
        let v = serde_json::json!({"stringValue": "svc"});
        assert_eq!(any_value(&v), "svc");
    }
}
