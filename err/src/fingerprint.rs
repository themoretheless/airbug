//! Default issue grouping (simplified Sentry-style fingerprint).
use crate::event::{Event, Frame, Severity};
use sha2::{Digest, Sha256};
use std::path::Path;

const TOP_FRAMES: usize = 5;

/// Resolve fingerprint parts for an event.
///
/// If the client set a custom fingerprint (anything other than a single
/// `"{{ default }}"`), it is used as-is. Otherwise we hash exception **type** +
/// top in-app frames (falling back to any frames). Exception **values** /
/// messages are not hashed, so unique ids in text do not create new issues.
pub fn resolve(event: &Event) -> Vec<String> {
    if !is_default(&event.fingerprint) {
        return event.fingerprint.clone();
    }
    vec![default_key(event)]
}

fn is_default(fp: &[String]) -> bool {
    fp.is_empty() || (fp.len() == 1 && (fp[0] == "{{ default }}" || fp[0].is_empty()))
}

fn default_key(event: &Event) -> String {
    let mut hasher = Sha256::new();
    if let Some(ex) = &event.exception {
        hasher.update(ex.ty.as_bytes());
        hasher.update(b"\0");
        let frames = frames_for_hash(ex.stacktrace.as_ref().map(|s| s.frames.as_slice()));
        if frames.is_empty() {
            // No frames: fall back to a normalized value so distinct error
            // *kinds* without stacks still group (digits/uuids stripped).
            hasher.update(normalize_volatile(ex.value.as_str()).as_bytes());
            hasher.update(b"\0");
        } else {
            for f in frames {
                hasher.update(f.function.as_bytes());
                hasher.update(b"|");
                if let Some(file) = &f.filename {
                    hasher.update(basename(file).as_bytes());
                }
                hasher.update(b"|");
                if let Some(line) = f.lineno {
                    hasher.update(line.to_string().as_bytes());
                }
                hasher.update(b"\n");
            }
        }
    } else if let Some(msg) = &event.message {
        hasher.update(b"msg\0");
        hasher.update(severity_tag(event.level).as_bytes());
        hasher.update(b"\0");
        hasher.update(normalize_volatile(msg).as_bytes());
    } else {
        hasher.update(b"empty");
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        hex.push_str(&format!("{b:02x}"));
    }
    format!("fp:{hex}")
}

fn severity_tag(level: Severity) -> &'static str {
    match level {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
        Severity::Fatal => "fatal",
    }
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Strip volatile tokens (uuids, hex ids, long digit runs) so message-only
/// fingerprints stay stable across unique request/user ids.
pub(crate) fn normalize_volatile(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // UUID: 8-4-4-4-12 hex with dashes
        if let Some(end) = match_uuid(bytes, i) {
            out.push_str("<id>");
            i = end;
            continue;
        }
        // Hex run of length >= 8
        if let Some(end) = match_hex_run(bytes, i, 8) {
            out.push_str("<hex>");
            i = end;
            continue;
        }
        // Digit run of length >= 3
        if let Some(end) = match_digit_run(bytes, i, 3) {
            out.push_str("<n>");
            i = end;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn match_uuid(bytes: &[u8], start: usize) -> Option<usize> {
    const PATTERN: &[usize] = &[8, 4, 4, 4, 12];
    let mut i = start;
    for (idx, &len) in PATTERN.iter().enumerate() {
        if i + len > bytes.len() || !bytes[i..i + len].iter().all(u8::is_ascii_hexdigit) {
            return None;
        }
        i += len;
        if idx + 1 < PATTERN.len() {
            if i >= bytes.len() || bytes[i] != b'-' {
                return None;
            }
            i += 1;
        }
    }
    Some(i)
}

fn match_hex_run(bytes: &[u8], start: usize, min: usize) -> Option<usize> {
    let mut i = start;
    while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
        i += 1;
    }
    if i - start >= min { Some(i) } else { None }
}

fn match_digit_run(bytes: &[u8], start: usize, min: usize) -> Option<usize> {
    let mut i = start;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i - start >= min { Some(i) } else { None }
}

fn frames_for_hash(frames: Option<&[Frame]>) -> Vec<&Frame> {
    let Some(frames) = frames else {
        return Vec::new();
    };
    let in_app: Vec<&Frame> = frames
        .iter()
        .filter(|f| f.in_app)
        .take(TOP_FRAMES)
        .collect();
    if !in_app.is_empty() {
        return in_app;
    }
    frames.iter().take(TOP_FRAMES).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Exception, Stacktrace};

    fn sample_event(ty: &str, value: &str, frames: Vec<Frame>) -> Event {
        Event {
            schema_version: crate::EVENT_SCHEMA_VERSION,
            event_id: "e1".into(),
            timestamp: "t".into(),
            level: Severity::Error,
            message: None,
            release: None,
            environment: None,
            service: None,
            fingerprint: vec!["{{ default }}".into()],
            exception: Some(Exception {
                ty: ty.into(),
                value: value.into(),
                stacktrace: Some(Stacktrace { frames }),
            }),
            breadcrumbs: vec![],
            tags: Default::default(),
            user: None,
            extra: Default::default(),
            contexts: Default::default(),
        }
    }

    fn frames() -> Vec<Frame> {
        vec![
            Frame {
                function: "foo".into(),
                filename: Some("/abs/path/a.rs".into()),
                lineno: Some(10),
                in_app: true,
            },
            Frame {
                function: "bar".into(),
                filename: Some("b.rs".into()),
                lineno: Some(20),
                in_app: true,
            },
        ]
    }

    #[test]
    fn same_stack_same_fingerprint() {
        let a = resolve(&sample_event("panic", "boom", frames()));
        let b = resolve(&sample_event("panic", "boom", frames()));
        assert_eq!(a, b);
        assert!(a[0].starts_with("fp:"));
    }

    #[test]
    fn different_values_same_stack_same_fingerprint() {
        let a = resolve(&sample_event("panic", "user 42 failed", frames()));
        let b = resolve(&sample_event("panic", "user 99 failed", frames()));
        assert_eq!(a, b);
    }

    #[test]
    fn basename_not_absolute_path() {
        let mut alt = frames();
        alt[0].filename = Some("/other/root/a.rs".into());
        let a = resolve(&sample_event("panic", "x", frames()));
        let b = resolve(&sample_event("panic", "x", alt));
        assert_eq!(a, b);
    }

    #[test]
    fn custom_fingerprint_preserved() {
        let mut e = sample_event("panic", "boom", vec![]);
        e.fingerprint = vec!["checkout".into(), "timeout".into()];
        assert_eq!(
            resolve(&e),
            vec!["checkout".to_string(), "timeout".to_string()]
        );
    }

    #[test]
    fn different_type_different_key() {
        let a = resolve(&sample_event("panic", "boom", frames()));
        let b = resolve(&sample_event("Error", "boom", frames()));
        assert_ne!(a, b);
    }

    #[test]
    fn message_only_strips_volatile_ids() {
        let mut a = sample_event("unused", "unused", vec![]);
        a.exception = None;
        a.message = Some("order 550e8400-e29b-41d4-a716-446655440000 failed".into());
        let mut b = sample_event("unused", "unused", vec![]);
        b.exception = None;
        b.message = Some("order 11111111-2222-3333-4444-555555555555 failed".into());
        assert_eq!(resolve(&a), resolve(&b));
    }

    #[test]
    fn normalize_volatile_examples() {
        assert_eq!(
            normalize_volatile("user 12345 deadbeefcafe"),
            "user <n> <hex>"
        );
    }
}
