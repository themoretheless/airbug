//! Read OTLP log records written by the local collector file exporter.
use super::{
    any_value, merge_record_attrs, nano_time, normalize_severity, resource_attr, resource_attrs,
    split_json_values, tail_text,
};
use crate::config::{self, RootPaths};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub path: String,
    pub available: bool,
    pub entries: Vec<LogEntry>,
    pub by_severity: Vec<SeverityCount>,
    pub services: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeverityCount {
    pub severity: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub time: String,
    pub time_ms: Option<u64>,
    pub severity: String,
    pub body: String,
    pub service: String,
    pub scope: String,
    #[serde(default)]
    pub attrs: std::collections::HashMap<String, String>,
}

pub fn read_recent(root: &Path, limit: usize) -> LogsResponse {
    read_filtered(root, limit, None)
}

pub fn read_filtered(root: &Path, limit: usize, run_id: Option<&str>) -> LogsResponse {
    let paths = RootPaths::new(root);
    let path = paths.logs_file();
    let path_s = path.display().to_string();
    if !path.is_file() {
        return LogsResponse {
            path: path_s,
            available: false,
            entries: vec![],
            by_severity: vec![],
            services: vec![],
            note: "No log file yet. Start hub with --collector and emit OTLP logs.".into(),
        };
    }

    match tail_parse(&path, limit.saturating_mul(4).max(limit)) {
        Ok(mut entries) => {
            if let Some(rid) = run_id {
                entries.retain(|e| e.attrs.get("airbug.run_id").map(|s| s.as_str()) == Some(rid));
            }
            if entries.len() > limit {
                entries = entries.split_off(entries.len() - limit);
            }
            let by_severity = count_by_severity(&entries);
            let services = distinct_services(&entries);
            LogsResponse {
                path: path_s,
                available: true,
                note: if entries.is_empty() {
                    if run_id.is_some() {
                        "No logs for this airbug.run_id yet.".into()
                    } else {
                        "Log file is empty or still buffering.".into()
                    }
                } else {
                    format!("{} recent record(s)", entries.len())
                },
                entries,
                by_severity,
                services,
            }
        }
        Err(err) => LogsResponse {
            path: path_s,
            available: true,
            entries: vec![],
            by_severity: vec![],
            services: vec![],
            note: format!("Failed to read logs: {err}"),
        },
    }
}

fn count_by_severity(entries: &[LogEntry]) -> Vec<SeverityCount> {
    let order = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
    let mut map: HashMap<&str, u64> = HashMap::new();
    for e in entries {
        *map.entry(normalize_severity(&e.severity)).or_insert(0) += 1;
    }
    order
        .iter()
        .filter_map(|sev| {
            let count = *map.get(sev)?;
            (count > 0).then_some(SeverityCount {
                severity: (*sev).into(),
                count,
            })
        })
        .collect()
}

fn distinct_services(entries: &[LogEntry]) -> Vec<String> {
    let mut services: Vec<String> = entries
        .iter()
        .map(|e| e.service.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    services.sort();
    services.dedup();
    services
}

fn tail_parse(path: &Path, limit: usize) -> std::io::Result<Vec<LogEntry>> {
    let buf = tail_text(path, config::LOGS_TAIL_BYTES)?;
    let mut entries = Vec::new();
    for chunk in split_json_values(&buf) {
        if let Ok(value) = serde_json::from_str::<Value>(chunk) {
            extract_entries(&value, &mut entries);
        }
    }
    if entries.len() > limit {
        entries = entries.split_off(entries.len() - limit);
    }
    Ok(entries)
}

fn extract_entries(value: &Value, out: &mut Vec<LogEntry>) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            extract_entries(item, out);
        }
        return;
    }

    if let Some(resource_logs) = value.get("resourceLogs").and_then(|v| v.as_array()) {
        for rl in resource_logs {
            let service = resource_attr(rl, "service.name");
            let base_attrs = resource_attrs(rl);
            let scopes = rl
                .get("scopeLogs")
                .or_else(|| rl.get("scope_logs"))
                .and_then(|v| v.as_array());
            let Some(scopes) = scopes else { continue };
            for sl in scopes {
                let scope = sl
                    .get("scope")
                    .and_then(|s| s.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let records = sl
                    .get("logRecords")
                    .or_else(|| sl.get("log_records"))
                    .and_then(|v| v.as_array());
                let Some(records) = records else { continue };
                for rec in records {
                    let (time, time_ms) = nano_time(
                        rec,
                        &[
                            "timeUnixNano",
                            "time_unix_nano",
                            "observedTimeUnixNano",
                            "observed_time_unix_nano",
                        ],
                    );
                    out.push(LogEntry {
                        time,
                        time_ms,
                        severity: record_severity(rec),
                        body: record_body(rec),
                        service: service.clone(),
                        scope: scope.clone(),
                        attrs: merge_record_attrs(base_attrs.clone(), rec),
                    });
                }
            }
        }
        return;
    }

    if value.is_object() {
        let mut attrs = std::collections::HashMap::new();
        if let Some(rid) = value.get("airbug.run_id").and_then(|v| v.as_str()) {
            attrs.insert("airbug.run_id".into(), rid.into());
        }
        out.push(LogEntry {
            time: value
                .get("time")
                .or_else(|| value.get("timestamp"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            time_ms: None,
            severity: value
                .get("severity")
                .or_else(|| value.get("level"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| "INFO".into()),
            body: value
                .get("body")
                .map(stringify_body)
                .or_else(|| {
                    value
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| value.to_string()),
            service: value
                .get("service")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            scope: String::new(),
            attrs,
        });
    }
}

fn record_severity(rec: &Value) -> String {
    if let Some(s) = rec
        .get("severityText")
        .or_else(|| rec.get("severity_text"))
        .and_then(|v| v.as_str())
    {
        return s.to_string();
    }
    if let Some(n) = rec
        .get("severityNumber")
        .or_else(|| rec.get("severity_number"))
        .and_then(|v| v.as_u64())
    {
        return match n {
            1..=4 => "TRACE",
            5..=8 => "DEBUG",
            9..=12 => "INFO",
            13..=16 => "WARN",
            17..=20 => "ERROR",
            21..=24 => "FATAL",
            _ => "INFO",
        }
        .into();
    }
    "INFO".into()
}

fn record_body(rec: &Value) -> String {
    rec.get("body").map(stringify_body).unwrap_or_default()
}

fn stringify_body(v: &Value) -> String {
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
    any_value(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    #[test]
    fn reads_flat_log_object() {
        let dir = env::temp_dir().join(format!("airbug-logs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let data = dir.join("dash/collector/data");
        fs::create_dir_all(&data).unwrap();
        fs::write(
            data.join("logs.json"),
            r#"{"body":"hello","severity":"WARN","service":"demo"}"#,
        )
        .unwrap();
        let resp = read_recent(&dir, 50);
        assert!(resp.available);
        assert_eq!(resp.entries.len(), 1);
        assert_eq!(resp.entries[0].body, "hello");
        assert_eq!(resp.by_severity[0].severity, "WARN");
        assert_eq!(resp.services, vec!["demo".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }
}
