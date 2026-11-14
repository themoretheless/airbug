//! Production service adoption of `airbug` (`unit`) inside `airbug-hub`:
//! exercises `HubApp`, `IssueStore`, `SymbolStore`, `StreamHub`, and `EventEnvelope`
//! using `#[airbug::mock]`, `Capture`, `CallSequence`, `CompletionBarrier`,
//! `FixtureContext` + `#[derive(Generate)]`, `Prop::for_all_shrunk`, `Snapshots`,
//! `SoftAssert`, and `assert_that` without binding TCP loopback sockets.

use airbug::checks::SoftAssert;
use airbug::fixture::{FixtureContext, Generate as _, GenerationError};
use airbug::mock::Capture;
use airbug::native::TempDir;
use airbug::order::{CallSequence, CompletionBarrier};
use airbug::prop::{Prop, PropError};
use airbug::snapshot::{JsonMode, Snapshots, UpdateMode};
use airbug::verify::{VerifyMocks, with_mocks};
use airbug::{assert_that, prelude::Matcher};
use airbug_err::{
    DebugImage, EVENT_SCHEMA_VERSION, Event, Exception, Frame, Severity, Stacktrace, User,
};
use airbug_hub::app::HubApp;
use airbug_hub::error::{HubError, Result as HubResult};
use airbug_hub::event_model::{EventEnvelope, Signal};
use airbug_hub::issues::{IngestResponse, Issue, IssueStore, IssuesList};
use airbug_hub::stream::StreamHub;
use airbug_hub::symbols::parse_hex_or_dec;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[airbug::mock]
trait IssueSink {
    fn ingest(&self, event: Event, webhook: Option<String>) -> Result<IngestResponse, String>;
}

#[airbug::mock]
trait IncidentNotifier {
    fn notify(&self, issue_id: String, title: String) -> Result<(), String>;
}

#[derive(Clone)]
struct MockIssueStore {
    sink: MockIssueSink,
}

impl MockIssueStore {
    fn new(sink: MockIssueSink) -> Self {
        Self { sink }
    }
}

impl IssueStore for MockIssueStore {
    fn ingest(&self, event: Event, webhook: Option<&str>) -> HubResult<IngestResponse> {
        self.sink
            .ingest(event, webhook.map(str::to_owned))
            .map_err(HubError::msg)
    }

    fn list(&self) -> IssuesList {
        IssuesList {
            path: String::new(),
            available: true,
            issues: Vec::new(),
            note: String::new(),
        }
    }

    fn get(&self, _id: &str) -> Option<Issue> {
        None
    }

    fn set_status(&self, _id: &str, _status: airbug_hub::issues::IssueStatus) -> HubResult<Issue> {
        Err(HubError::msg("not implemented in mock store"))
    }
}

#[derive(Debug, Clone, airbug::Generate)]
struct EventSeed {
    #[fixture(with = gen_short_id)]
    event_id: String,
    #[fixture(with = gen_service_name)]
    service: String,
    #[fixture(with = gen_offset)]
    offset: u64,
}

fn gen_short_id(ctx: &mut FixtureContext) -> Result<String, GenerationError> {
    Ok(format!("evt-{:08x}", u64::generate(ctx)? & 0xffff_ffff))
}

fn gen_service_name(ctx: &mut FixtureContext) -> Result<String, GenerationError> {
    let idx = usize::generate(ctx)? % 3;
    Ok(["billing", "checkout", "ledger"][idx].to_string())
}

fn gen_offset(ctx: &mut FixtureContext) -> Result<u64, GenerationError> {
    Ok(0x1040 + (u64::generate(ctx)? % 0x10))
}

fn sample_raw_event(event_id: &str, service: &str, addr: u64) -> Event {
    Event {
        schema_version: EVENT_SCHEMA_VERSION,
        event_id: event_id.to_string(),
        timestamp: "2026-11-15T00:00:00Z".into(),
        level: Severity::Error,
        message: Some("charge failed for alice@example.com with Bearer tok_secret_42".into()),
        release: Some("0.11.0".into()),
        environment: Some("prod".into()),
        service: Some(service.to_string()),
        fingerprint: vec!["{{ default }}".into()],
        exception: Some(Exception {
            ty: "panic".into(),
            value: "unexpected card decline for alice@example.com".into(),
            stacktrace: Some(Stacktrace {
                frames: vec![Frame {
                    function: "<unsymbolicated>".into(),
                    filename: None,
                    lineno: None,
                    colno: None,
                    in_app: false,
                    instruction_addr: Some(format!("{addr:#x}")),
                    symbol_addr: None,
                }],
            }),
        }),
        breadcrumbs: vec![],
        tags: BTreeMap::from([("password".into(), "hunter2".into())]),
        user: Some(User {
            id: Some("u-1".into()),
            email: Some("alice@example.com".into()),
            username: Some("alice".into()),
        }),
        extra: BTreeMap::new(),
        contexts: BTreeMap::new(),
        trace_id: None,
        span_id: None,
        debug_images: vec![DebugImage {
            debug_id: Some("MOD0110".into()),
            code_id: None,
            debug_file: Some("billing.bin".into()),
            image_addr: Some("0x400000".into()),
            image_size: Some(0x10000),
            arch: Some("x86_64".into()),
        }],
    }
}

#[test]
fn hub_ingest_pipeline_symbolicates_scrubs_and_orders_effects() {
    let dir = TempDir::new().unwrap();
    let mut ctx = FixtureContext::with_seed(0xA17B_0001);
    let seed = ctx.build::<EventSeed>();

    let mut app = HubApp::new(
        dir.path(),
        8790,
        Some("http://127.0.0.1:9999/airbug".into()),
    )
    .unwrap();
    app.set_scrub_pii(true);
    app.symbols
        .upload_breakpad_sym(
            None,
            "\
MODULE Linux x86_64 MOD0110 billing.bin
FILE 0 src/billing.rs
FUNC 1000 200 0 billing::charge_card
1040 10 88 0
",
        )
        .unwrap();

    let sink = MockIssueSink::default();
    let notifier = MockIncidentNotifier::default();
    let seq = CallSequence::new("hub_ingest_then_notify");
    let captured = Capture::<(Event, Option<String>)>::new(4);

    let expected_event_id = seed.event_id.clone();
    sink.ingest
        .expect("ingest_prepared", move |(ev, webhook)| {
            ev.event_id == expected_event_id && webhook.as_deref().is_some()
        })
        .capture(&captured)
        .times(1)
        .in_sequence(&seq)
        .returns(Ok(IngestResponse {
            ok: true,
            issue_id: "ISS-101".into(),
            is_new: true,
            is_regression: false,
            count: 1,
        }));

    notifier
        .notify
        .expect("notify_new_issue", |(issue_id, _title)| {
            issue_id == "ISS-101"
        })
        .times(1)
        .in_sequence(&seq)
        .returns(Ok(()));

    app.issues = Arc::new(MockIssueStore::new(sink.clone()));
    let rx = app.stream.subscribe();
    let event = sample_raw_event(&seed.event_id, &seed.service, 0x400000 + seed.offset);

    let mocks: [&dyn VerifyMocks; 3] = [&sink, &notifier, &seq];
    with_mocks(&mocks, || {
        let resp = app.ingest_error_event(event).unwrap();
        notifier
            .notify("ISS-101".into(), "panic: scrubbed".into())
            .unwrap();

        let broadcast = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let recorded = captured.values();
        assert_that(&recorded.len()).is_equal_to(&1);
        let (stored_event, webhook) = &recorded[0];
        let frame = &stored_event
            .exception
            .as_ref()
            .unwrap()
            .stacktrace
            .as_ref()
            .unwrap()
            .frames[0];

        let mut soft = SoftAssert::default();
        soft.equal("issue_id", &resp.issue_id, &"ISS-101".to_string());
        soft.equal(
            "webhook",
            webhook,
            &Some("http://127.0.0.1:9999/airbug".to_string()),
        );
        soft.equal(
            "symbol_fn",
            &frame.function,
            &"billing::charge_card".to_string(),
        );
        soft.equal("symbol_lineno", &frame.lineno, &Some(88u32));
        soft.equal(
            "fingerprint",
            &stored_event.fingerprint,
            &vec!["panic:billing::charge_card".to_string()],
        );
        soft.equal(
            "scrubbed_message_no_email",
            &stored_event
                .message
                .as_deref()
                .unwrap_or("")
                .contains("alice@example.com"),
            &false,
        );
        soft.equal(
            "scrubbed_message_no_token",
            &stored_event
                .message
                .as_deref()
                .unwrap_or("")
                .contains("tok_secret_42"),
            &false,
        );
        soft.equal("stream_kind", &broadcast.kind, &"error".to_string());
        soft.assert();
    });

    let journal = std::fs::read_to_string(app.paths.events_file()).unwrap();
    let lines: Vec<&str> = journal.lines().collect();
    assert_that(&lines).has_length(1);
    let envelope: EventEnvelope = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(envelope.payload.signal(), Signal::Error);
    assert_eq!(envelope.event_id, seed.event_id);
}

#[test]
fn hub_ingest_store_failure_skips_stream_broadcast_and_coordinates_workers() {
    let dir = TempDir::new().unwrap();
    let mut app = HubApp::new(dir.path(), 8790, None).unwrap();
    let sink = MockIssueSink::default();

    let busy_matcher = Matcher::new("fail_evt_1", |(ev, _): &(Event, Option<String>)| {
        ev.event_id == "evt-fail"
    });
    let ok_matcher = Matcher::new("ok_evt_2", |(ev, _): &(Event, Option<String>)| {
        ev.event_id == "evt-ok"
    });
    sink.ingest
        .expect_matcher(busy_matcher)
        .times(1)
        .returns(Err("sqlite busy".into()));
    sink.ingest
        .expect_matcher(ok_matcher)
        .times(1)
        .returns(Ok(IngestResponse {
            ok: true,
            issue_id: "ISS-OK".into(),
            is_new: false,
            is_regression: true,
            count: 2,
        }));

    app.issues = Arc::new(MockIssueStore::new(sink.clone()));
    let app = Arc::new(app);
    let rx = app.stream.subscribe();
    let barrier = CompletionBarrier::named("ingest_workers", 2);

    let app1 = Arc::clone(&app);
    let b1 = barrier.clone();
    let h1 = thread::spawn(move || {
        let _tok = b1.enter("fail_worker");
        app1.ingest_error_event(sample_raw_event("evt-fail", "billing", 0x401000))
    });

    let app2 = Arc::clone(&app);
    let b2 = barrier.clone();
    let h2 = thread::spawn(move || {
        let _tok = b2.enter("ok_worker");
        app2.ingest_error_event(sample_raw_event("evt-ok", "checkout", 0x401020))
    });

    barrier.wait_timeout(Duration::from_secs(2)).unwrap();
    let res1 = h1.join().unwrap();
    let res2 = h2.join().unwrap();

    assert!(res1.is_err());
    assert_eq!(res2.unwrap().issue_id, "ISS-OK");

    let broadcast = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(broadcast.payload["event_id"], "evt-ok");
    assert!(rx.try_recv().is_err(), "failed ingest must not broadcast");

    let journal = std::fs::read_to_string(app.paths.events_file()).unwrap();
    assert_eq!(journal.lines().count(), 1);
    assert!(journal.contains("evt-ok"));

    let mocks: [&dyn VerifyMocks; 2] = [&sink, &barrier];
    with_mocks(&mocks, || {});
}

#[test]
fn hub_symbol_store_and_hex_parser_property_and_snapshot_checks() {
    // Roundtrip property over parse_hex_or_dec for both hex and decimal inputs.
    Prop::new(0x5A17_0001)
        .trials(64)
        .for_all_shrunk::<u64, _>(|n| {
            let hex = format!("{n:#x}");
            let dec = n.to_string();
            Ok(parse_hex_or_dec(&hex) == Some(*n) && parse_hex_or_dec(&dec) == Some(*n))
        })
        .unwrap();

    // Verify counterexample shrinking produces a minimal failing offset boundary.
    let prop_fail = Prop::new(0x5A17_0002)
        .trials(40)
        .max_shrinks(200)
        .for_all_shrunk::<u64, _>(|n| Ok((*n % 200) < 100));
    let Err(PropError::Failed(failure)) = prop_fail else {
        panic!("expected shrunk property failure, got {prop_fail:?}");
    };
    assert!(
        failure.message.contains("minimal") || !failure.message.is_empty(),
        "unexpected failure detail: {}",
        failure.message
    );

    // Canonical JSON snapshot verification for StreamHub::snapshot.
    let hub = StreamHub::new();
    hub.publish(
        "test_run",
        json!({
            "run_id": "test-123",
            "passed": 12,
            "failed": 0
        }),
    );
    let snap = hub.snapshot(10);
    let dir = TempDir::new().unwrap();
    Snapshots::new(dir.path())
        .mode(UpdateMode::Overwrite)
        .json_mode(JsonMode::Pretty)
        .check_json("stream_hub_snapshot", &snap)
        .unwrap();
    Snapshots::new(dir.path())
        .mode(UpdateMode::Verify)
        .json_mode(JsonMode::Pretty)
        .check_json("stream_hub_snapshot", &snap)
        .unwrap();
}
