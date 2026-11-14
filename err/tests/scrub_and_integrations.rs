use airbug_err::{
    DebugImage, HttpMiddleware, HttpRequestMeta, MemoryTransport, Options, ScrubAction,
    ScrubTarget, Scrubber, Severity, User, capture_event, configure_scope, init_with_transport,
    new_event,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

static CLIENT_LOCK: Mutex<()> = Mutex::new(());

fn block_on_ready<F: Future>(fut: F) -> F::Output {
    fn dummy_waker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        unsafe fn noop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }

    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    let mut pinned = Box::pin(fut);
    loop {
        if let Poll::Ready(out) = Pin::as_mut(&mut pinned).poll(&mut cx) {
            return out;
        }
        std::hint::spin_loop();
    }
}

#[test]
fn scrubber_default_pii_redacts_common_patterns_and_urls() {
    let scrubber = Scrubber::default_pii();
    let text = concat!(
        "user alice@example.com ",
        "Bearer my_secret_token_999 ",
        "ghp_1234567890abcdef ",
        "card 4111-1111-1111-1111 ",
        "ip 192.168.10.55 ",
        "ipv6 2001:db8::1 ",
        "ssn 123-45-6789 ",
        "https://admin:pass123@api.example.com/pay?secret=xyz&ok=1 ",
        "password=supersecret",
    );
    let scrubbed = scrubber.scrub_str(text);
    assert!(!scrubbed.contains("alice@example.com"), "{scrubbed}");
    assert!(!scrubbed.contains("my_secret_token_999"), "{scrubbed}");
    assert!(!scrubbed.contains("ghp_1234567890abcdef"), "{scrubbed}");
    assert!(!scrubbed.contains("4111-1111-1111-1111"), "{scrubbed}");
    assert!(!scrubbed.contains("192.168.10.55"), "{scrubbed}");
    assert!(!scrubbed.contains("2001:db8::1"), "{scrubbed}");
    assert!(!scrubbed.contains("123-45-6789"), "{scrubbed}");
    assert!(!scrubbed.contains("pass123"), "{scrubbed}");
    assert!(!scrubbed.contains("secret=xyz"), "{scrubbed}");
    assert!(!scrubbed.contains("supersecret"), "{scrubbed}");
    assert!(scrubbed.contains("Bearer [REDACTED]"), "{scrubbed}");
    let hashed = Scrubber::new()
        .rule(ScrubTarget::Email, ScrubAction::Hash)
        .scrub_str("contact alice@example.com now");
    assert!(hashed.starts_with("contact [sha256:"), "{hashed}");
    assert!(!hashed.contains("alice@example.com"), "{hashed}");
}

#[test]
fn scrubber_rules_hash_and_drop_event_fields() {
    let _lock = CLIENT_LOCK.lock().unwrap();
    let transport = Arc::new(MemoryTransport::new());
    let scrubber = Scrubber::default_pii()
        .hash_field("user.id")
        .drop_field("tags.transient")
        .drop_field("extra.raw_payload")
        .redact_literal("INTERNAL-*");

    let image = DebugImage {
        debug_id: Some("dbg-1".into()),
        code_id: None,
        debug_file: Some("app.sym".into()),
        image_addr: Some("0x400000".into()),
        image_size: Some(0x10000),
        arch: Some("x86_64".into()),
    };

    let _guard = init_with_transport(
        Options::new()
            .endpoint("http://127.0.0.1:9/api/v1/errors")
            .scrubber(scrubber)
            .add_debug_image(image.clone()),
        transport.clone(),
    )
    .unwrap();

    configure_scope(|s| {
        s.set_user(User {
            id: Some("user-42".into()),
            email: Some("bob@corp.example".into()),
            username: Some("bob".into()),
        });
        s.set_tag("transient", "drop-me");
        s.set_tag("keep", "INTERNAL-secret-code");
        s.set_extra("raw_payload", serde_json::json!({"a": 1}));
        s.set_extra("api_key", serde_json::json!("topsecret"));
    });

    let mut ev = new_event(Severity::Error);
    ev.message = Some("contact bob@corp.example for INTERNAL-42".into());
    capture_event(ev);

    configure_scope(|s| s.clear());

    let events = transport.events();
    assert_eq!(events.len(), 1);
    let sent = &events[0];
    assert_eq!(sent.debug_images, vec![image]);
    let msg = sent.message.as_deref().unwrap();
    assert!(!msg.contains("bob@corp.example"), "{msg}");
    assert!(!msg.contains("INTERNAL-42"), "{msg}");
    assert!(msg.contains("[REDACTED]"), "{msg}");

    let u = sent.user.as_ref().unwrap();
    assert!(u.id.as_deref().unwrap().starts_with("[sha256:"), "{u:?}");
    assert_eq!(u.email.as_deref(), Some("[REDACTED]"));
    assert!(!sent.tags.contains_key("transient"));
    assert_eq!(
        sent.tags.get("keep").map(String::as_str),
        Some("[REDACTED]")
    );
    assert!(!sent.extra.contains_key("raw_payload"));
    assert_eq!(
        sent.extra.get("api_key"),
        Some(&serde_json::json!("[REDACTED]"))
    );
}

#[test]
fn http_middleware_captures_5xx_panics_and_isolates_scope() {
    let _lock = CLIENT_LOCK.lock().unwrap();
    let transport = Arc::new(MemoryTransport::new());
    let _guard = init_with_transport(
        Options::new().endpoint("http://127.0.0.1:9/api/v1/errors"),
        transport.clone(),
    )
    .unwrap();

    configure_scope(|s| s.set_tag("outer", "stays"));

    let mw = HttpMiddleware::new();
    let meta_ok = HttpRequestMeta::new("GET", "/health");
    let res = mw.call_sync(&meta_ok, |code| *code, || 200u16);
    assert_eq!(res, 200);
    assert!(transport.events().is_empty());

    let meta_500 = HttpRequestMeta::new("POST", "/checkout")
        .query("coupon=ABC")
        .header("x-trace", "1")
        .user_id("usr-7");
    let status = mw.call_sync(&meta_500, |code| *code, || 502u16);
    assert_eq!(status, 502);

    let meta_panic = HttpRequestMeta::new("DELETE", "/item/9");
    let panicked = mw.run_sync(&meta_panic, |_: &()| 200, || panic!("db vanished"));
    assert!(panicked.is_err());

    let async_panic = block_on_ready(mw.run_async(
        HttpRequestMeta::new("PUT", "/async"),
        |code| Some(*code),
        async {
            if true {
                panic!("async boom");
            }
            200u16
        },
    ));
    assert!(async_panic.is_err());

    // Thread scope outside middleware calls must not retain per-request tags.
    let mut after_tags = Vec::new();
    configure_scope(|s| {
        after_tags = s.tags.keys().cloned().collect();
        s.clear();
    });
    assert_eq!(after_tags, vec!["outer".to_string()]);

    let events = transport.events();
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[0].tags.get("http.status_code").map(String::as_str),
        Some("502")
    );
    assert!(events[0].contexts.contains_key("request"));
    assert_eq!(events[1].level, Severity::Fatal);
    assert_eq!(
        events[1].tags.get("http.route").map(String::as_str),
        Some("/item/9")
    );
    assert_eq!(
        events[2].tags.get("http.route").map(String::as_str),
        Some("/async")
    );
}
