//! Default issue grouping (simplified Sentry-style fingerprint).
use crate::event::{Event, Frame};
use sha2::{Digest, Sha256};

const TOP_FRAMES: usize = 5;

/// Resolve fingerprint parts for an event.
///
/// If the client set a custom fingerprint (anything other than a single
/// `"{{ default }}"`), it is used as-is. Otherwise we hash exception type +
/// top in-app frames (falling back to any frames).
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
        hasher.update(ex.value.as_bytes());
        hasher.update(b"\0");
        let frames = frames_for_hash(ex.stacktrace.as_ref().map(|s| s.frames.as_slice()));
        for f in frames {
            hasher.update(f.function.as_bytes());
            hasher.update(b"|");
            if let Some(file) = &f.filename {
                hasher.update(file.as_bytes());
            }
            hasher.update(b"|");
            if let Some(line) = f.lineno {
                hasher.update(line.to_string().as_bytes());
            }
            hasher.update(b"\n");
        }
    } else if let Some(msg) = &event.message {
        hasher.update(b"msg\0");
        hasher.update(msg.as_bytes());
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
    use crate::event::{Exception, Severity, Stacktrace};

    fn sample_event(ty: &str, frames: Vec<Frame>) -> Event {
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
                value: "boom".into(),
                stacktrace: Some(Stacktrace { frames }),
            }),
            breadcrumbs: vec![],
            tags: Default::default(),
            user: None,
            extra: Default::default(),
            contexts: Default::default(),
        }
    }

    #[test]
    fn same_stack_same_fingerprint() {
        let frames = vec![
            Frame {
                function: "foo".into(),
                filename: Some("a.rs".into()),
                lineno: Some(10),
                in_app: true,
            },
            Frame {
                function: "bar".into(),
                filename: Some("b.rs".into()),
                lineno: Some(20),
                in_app: true,
            },
        ];
        let a = resolve(&sample_event("panic", frames.clone()));
        let b = resolve(&sample_event("panic", frames));
        assert_eq!(a, b);
        assert!(a[0].starts_with("fp:"));
    }

    #[test]
    fn custom_fingerprint_preserved() {
        let mut e = sample_event("panic", vec![]);
        e.fingerprint = vec!["checkout".into(), "timeout".into()];
        assert_eq!(
            resolve(&e),
            vec!["checkout".to_string(), "timeout".to_string()]
        );
    }

    #[test]
    fn different_type_different_key() {
        let frames = vec![Frame {
            function: "foo".into(),
            filename: Some("a.rs".into()),
            lineno: Some(1),
            in_app: true,
        }];
        let a = resolve(&sample_event("panic", frames.clone()));
        let b = resolve(&sample_event("Error", frames));
        assert_ne!(a, b);
    }
}
