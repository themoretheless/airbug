//! Shared OTLP JSON file helpers, rotated-backup tailing, ID decoding, and signal submodules.
pub mod logs;
pub mod metrics;
pub mod profiles;
pub mod receiver;
pub mod traces;

#[allow(unused_imports)]
pub use logs::{read_filtered as read_logs_filtered, read_logs_query};
pub use metrics::read_filtered as read_metrics_filtered;
pub use profiles::{ProfilesQuery, read_profiles};
pub use traces::{TracesQuery, read_trace_detail_with_correlations, read_traces, read_tracez};

use base64::Engine as _;
use serde_json::Value;
use std::{
    cmp::Reverse,
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Whether either `path` itself or a rotated `<stem>-*.<ext>` backup exists on disk.
pub fn has_signal_file(path: &Path) -> bool {
    path.is_file() || newest_rotated_backup(path).is_some()
}

/// Read the tail of `path` (up to `max_bytes`) and return a JSON-safe slice.
///
/// If the primary file was recently rotated and contains fewer than `max_bytes` (or does not
/// exist yet while a rotated backup `<stem>-*.json` exists), prepends tail bytes from the
/// newest rotated backup so file rotation never causes an empty UI window.
pub fn tail_text(path: &Path, max_bytes: u64) -> std::io::Result<String> {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => {
            let primary_len = meta.len();
            let primary = tail_single_file(path, max_bytes)?;
            if primary_len >= max_bytes {
                return Ok(primary);
            }
            let remaining = max_bytes.saturating_sub(primary_len);
            if remaining > 0
                && let Some(backup) = newest_rotated_backup(path)
                && let Ok(prefix) = tail_single_file(&backup, remaining)
                && !prefix.trim().is_empty()
            {
                if primary.trim().is_empty() {
                    return Ok(prefix);
                }
                let sep = if prefix.ends_with('\n') { "" } else { "\n" };
                return Ok(format!("{prefix}{sep}{primary}"));
            }
            Ok(primary)
        }
        Err(err) => {
            if let Some(backup) = newest_rotated_backup(path) {
                return tail_single_file(&backup, max_bytes);
            }
            Err(err)
        }
        Ok(_) => Err(std::io::Error::other("signal path is not a regular file")),
    }
}

fn tail_single_file(path: &Path, max_bytes: u64) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut raw = Vec::new();
    file.read_to_end(&mut raw)?;
    let slice = if start > 0 {
        if let Some(pos) = raw.iter().position(|&b| b == b'\n') {
            &raw[pos + 1..]
        } else {
            &raw[..]
        }
    } else {
        &raw[..]
    };
    let buf = String::from_utf8_lossy(slice).into_owned();
    if start > 0
        && let Some(i) = buf.find('{')
    {
        return Ok(buf[i..].to_string());
    }
    Ok(buf)
}

/// Find the newest rotated backup matching `<stem>-*.<ext>` in `path.parent()`.
pub fn newest_rotated_backup(path: &Path) -> Option<PathBuf> {
    let dir = path.parent()?;
    let stem = path.file_stem()?.to_str()?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("json");
    let prefix = format!("{stem}-");
    let suffix = format!(".{ext}");

    let entries = fs::read_dir(dir).ok()?;
    let mut candidates: Vec<(Reverse<usize>, SystemTime, String, PathBuf)> = Vec::new();

    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(rest) = name.strip_prefix(&prefix) else {
            continue;
        };
        let Some(mid) = rest.strip_suffix(&suffix) else {
            continue;
        };
        if mid.is_empty() {
            continue;
        }
        let mtime = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        // Numeric backup indices (`logs-1.json`, `logs-2.json`): index 1 is always newer than 2.
        // Timestamped otelcol backups (`logs-2026-11-12T...json`): rank by mtime & filename.
        let num_rank = mid.parse::<usize>().unwrap_or(0);
        let order_key = if num_rank > 0 {
            Reverse(num_rank)
        } else {
            Reverse(0)
        };
        candidates.push((order_key, mtime, name.to_string(), p));
    }

    candidates.sort_by(|a, b| {
        // If both are numeric (`<stem>-1.json` vs `<stem>-2.json`), smaller index is newer (`Reverse(1) > Reverse(2)`).
        match (a.0.0 > 0, b.0.0 > 0) {
            (true, true) => a.0.cmp(&b.0),
            _ => a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2)),
        }
    });
    candidates.pop().map(|(_, _, _, p)| p)
}

/// Decode an OTLP `traceId` (`expected_bytes == 16`) or `spanId` (`expected_bytes == 8`) from
/// hex string, base64 string, or `[u8]` JSON array into lowercase hex, rejecting all-zero IDs.
pub fn decode_id_hex(val: &Value, expected_bytes: usize) -> Option<String> {
    if let Some(s) = val.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        let hex_len = expected_bytes * 2;
        if trimmed.len() == hex_len && trimmed.bytes().all(|b| b.is_ascii_hexdigit()) {
            let bytes = hex::decode(trimmed).ok()?;
            if bytes.iter().all(|&b| b == 0) {
                return None;
            }
            return Some(hex::encode(bytes));
        }
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD
            .decode(trimmed)
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(trimmed))
            .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(trimmed))
            && bytes.len() == expected_bytes
            && !bytes.iter().all(|&b| b == 0)
        {
            return Some(hex::encode(bytes));
        }
        return None;
    }

    if let Some(arr) = val.as_array()
        && arr.len() == expected_bytes
    {
        let mut bytes = Vec::with_capacity(expected_bytes);
        for item in arr {
            let b = item.as_u64().and_then(|n| u8::try_from(n).ok())?;
            bytes.push(b);
        }
        if bytes.iter().all(|&b| b == 0) {
            return None;
        }
        return Some(hex::encode(bytes));
    }

    None
}

pub fn parse_u64(v: &Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_i64().and_then(|n| u64::try_from(n).ok()))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
}

pub fn parse_i64(v: &Value) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_u64().and_then(|n| i64::try_from(n).ok()))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
}

pub fn parse_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_i64().map(|n| n as f64))
        .or_else(|| v.as_u64().map(|n| n as f64))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
        .filter(|n| n.is_finite())
}

pub fn nano_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(v) = value.get(*key)
            && let Some(n) = parse_u64(v)
        {
            return Some(n);
        }
    }
    None
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
    resource_attrs(resource)
        .get(key)
        .cloned()
        .unwrap_or_default()
}

/// Flatten resource attributes into a string map.
pub fn resource_attrs(resource_container: &Value) -> HashMap<String, String> {
    attrs_to_map(
        resource_container
            .get("resource")
            .and_then(|r| r.get("attributes")),
    )
}

pub fn attrs_to_map(attrs_val: Option<&Value>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(attrs) = attrs_val.and_then(Value::as_array) {
        for attr in attrs {
            let k = attr.get("key").and_then(Value::as_str).unwrap_or("");
            if k.is_empty() {
                continue;
            }
            map.insert(
                k.to_string(),
                attr.get("value").map(any_value).unwrap_or_default(),
            );
        }
    }
    map
}

pub fn merge_record_attrs(
    mut map: HashMap<String, String>,
    record: &Value,
) -> HashMap<String, String> {
    for (k, v) in attrs_to_map(record.get("attributes")) {
        map.insert(k, v);
    }
    map
}

pub fn any_value(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    if let Some(b) = v.as_bool() {
        return b.to_string();
    }
    if let Some(obj) = v.as_object() {
        // Unwrap nested `{"value": {"StringValue": ...}}` or `{"stringValue": ...}`
        if let Some(inner) = obj.get("value")
            && obj.len() == 1
        {
            return any_value(inner);
        }
        if let Some(s) = obj
            .get("stringValue")
            .or_else(|| obj.get("string_value"))
            .or_else(|| obj.get("StringValue"))
            .and_then(Value::as_str)
        {
            return s.to_string();
        }
        if let Some(n) = obj
            .get("intValue")
            .or_else(|| obj.get("int_value"))
            .or_else(|| obj.get("IntValue"))
        {
            if let Some(s) = n.as_str() {
                return s.to_string();
            }
            return n.to_string();
        }
        if let Some(n) = obj
            .get("doubleValue")
            .or_else(|| obj.get("double_value"))
            .or_else(|| obj.get("DoubleValue"))
        {
            return n.to_string();
        }
        if let Some(b) = obj
            .get("boolValue")
            .or_else(|| obj.get("bool_value"))
            .or_else(|| obj.get("BoolValue"))
            .and_then(Value::as_bool)
        {
            return b.to_string();
        }
        if let Some(arr) = obj
            .get("arrayValue")
            .or_else(|| obj.get("array_value"))
            .and_then(|av| av.get("values"))
            .and_then(Value::as_array)
        {
            let items: Vec<String> = arr.iter().map(any_value).collect();
            return format!("[{}]", items.join(", "));
        }
        if let Some(kvs) = obj
            .get("kvlistValue")
            .or_else(|| obj.get("kvlist_value"))
            .and_then(|kv| kv.get("values"))
            .and_then(Value::as_array)
        {
            let pairs: Vec<String> = kvs
                .iter()
                .filter_map(|item| {
                    let k = item.get("key")?.as_str()?;
                    let val = item.get("value").map(any_value).unwrap_or_default();
                    Some(format!("{k}={val}"))
                })
                .collect();
            return format!("{{{}}}", pairs.join(", "));
        }
    }
    v.to_string()
}

/// Parse OTLP unix-nano field → (label, time_ms).
/// Falls back to later keys (such as `observedTimeUnixNano`) when an earlier key is `"0"` or `0`.
pub fn nano_time(value: &Value, keys: &[&str]) -> (String, Option<u64>) {
    let mut raw = None;
    let mut saw_zero = false;
    for key in keys {
        if let Some(candidate) = value.get(*key).and_then(|v| {
            v.as_str()
                .map(|s| s.trim().to_string())
                .or_else(|| v.as_u64().map(|n| n.to_string()))
        }) {
            if candidate == "0" || candidate.is_empty() {
                if candidate == "0" {
                    saw_zero = true;
                }
                continue;
            }
            raw = Some(candidate);
            break;
        }
    }
    if raw.is_none() && saw_zero {
        raw = Some("0".to_string());
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
    fn any_value_string_bool_and_array() {
        assert_eq!(any_value(&serde_json::json!({"stringValue": "svc"})), "svc");
        assert_eq!(any_value(&serde_json::json!({"boolValue": true})), "true");
        assert_eq!(
            any_value(&serde_json::json!({
                "arrayValue": {"values": [{"stringValue": "a"}, {"intValue": "2"}]}
            })),
            "[a, 2]"
        );
    }

    #[test]
    fn decode_id_supports_hex_base64_and_byte_arrays() {
        let raw_bytes: [u8; 16] = [
            0x5b, 0x8e, 0xff, 0xf7, 0x98, 0x03, 0x81, 0x03, 0xd2, 0x69, 0xb6, 0x33, 0x81, 0x3f,
            0xc6, 0x0c,
        ];
        let expected = "5b8efff798038103d269b633813fc60c";
        assert_eq!(
            decode_id_hex(&serde_json::json!(expected.to_ascii_uppercase()), 16).as_deref(),
            Some(expected)
        );
        let b64 = base64::engine::general_purpose::STANDARD.encode(raw_bytes);
        assert_eq!(
            decode_id_hex(&serde_json::json!(b64), 16).as_deref(),
            Some(expected)
        );
        let arr: Vec<u8> = raw_bytes.to_vec();
        assert_eq!(
            decode_id_hex(&serde_json::json!(arr), 16).as_deref(),
            Some(expected)
        );
        assert!(decode_id_hex(&serde_json::json!("0000000000000000"), 8).is_none());
    }

    #[test]
    fn tail_text_handles_multibyte_utf8_and_partial_lines() {
        let dir = std::env::temp_dir().join(format!("airbug-otlp-tail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("logs.json");
        let l1 = r#"{"body":"первая строка — 12.5 µs","severity":"INFO"}"#;
        let l2 = r#"{"body":"вторая строка — 99.0 µs","severity":"WARN"}"#;
        let content = format!("{l1}\n{l2}\n");
        std::fs::write(&file, &content).unwrap();

        let max_bytes = (l2.len() + 15) as u64;
        let tailed = tail_text(&file, max_bytes).unwrap();
        assert_eq!(tailed.trim(), l2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tail_text_prepends_newest_rotated_backup_when_primary_is_small() {
        let dir = std::env::temp_dir().join(format!("airbug-otlp-tail-rot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let older_bak = dir.join("logs-2.json");
        let newer_bak = dir.join("logs-1.json");
        let primary = dir.join("logs.json");

        std::fs::write(&older_bak, "{\"seq\":1}\n").unwrap();
        std::fs::write(&newer_bak, "{\"seq\":2}\n{\"seq\":3}\n").unwrap();
        std::fs::write(&primary, "{\"seq\":4}\n").unwrap();

        let tailed = tail_text(&primary, 4096).unwrap();
        let chunks = split_json_values(&tailed);
        assert_eq!(chunks, vec!["{\"seq\":2}", "{\"seq\":3}", "{\"seq\":4}"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
