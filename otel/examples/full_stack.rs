//! End-to-end showcase of all 4 OpenTelemetry signals + W3C propagation + `tracing` bridge
//! + `airbug-err` automatic trace correlation + `airbug-bench` memory profile conversion.
//!
//! Run against a running `airbug-hub` (built-in or Docker collector on `:4318`, hub on `:8790`):
//! ```sh
//! cargo run -p airbug-hub -- serve --root . --collector
//! cargo run -p airbug-otel --example full_stack
//! ```

use airbug_bench::memory::{Profile as MemoryProfile, Site as AllocationSite};
use airbug_otel::{
    F64Gauge, F64Histogram, I64Gauge, I64UpDownCounter, KeyValue, ObservableF64Gauge,
    ObservableI64UpDownCounter, ObservableU64Counter, ProfileBatch, ProfileFrame, ProfileSample,
    Protocol, Severity, SpanKind, SpanLink, SpanOptions, TelemetryConfig, TraceSampler, U64Counter,
    U64Gauge, U64Histogram, add_counter, add_span_event, baggage_value, current_trace_id,
    current_traceparent, emit_log_with_attrs, export_profiles_http, in_span, in_span_async,
    in_span_async_with_options, init, inject_map, record_histogram, set_attributes, tracing_layer,
    with_baggage, with_extracted_headers_async,
};
use std::{collections::HashMap, io, time::Duration};
use tracing_subscriber::layer::SubscriberExt;

#[tracing::instrument(fields(order.id = order_id, otel.kind = "server"))]
async fn process_checkout_request(order_id: u64) -> String {
    tracing::info!(step = "validate", "validating cart items");
    set_attributes(&[
        KeyValue::new("checkout.currency", "USD"),
        KeyValue::new("checkout.items", 3i64),
    ]);
    emit_log_with_attrs(
        "airbug.example.checkout",
        Severity::Info,
        "checkout.validated",
        "cart passed fraud and inventory checks",
        &[KeyValue::new("order.id", order_id as i64)],
    );
    current_traceparent().unwrap_or_default()
}

/// Convert an `airbug_bench::memory::Profile` into an OTLP `ProfileBatch` linked to the active span.
fn bench_memory_to_otlp_profile(service_name: &str, mem: &MemoryProfile) -> ProfileBatch {
    let mut batch = ProfileBatch::new(service_name, "allocations", "bytes")
        .period("space", "bytes", 1)
        .duration_nanos(12_500_000);
    for site in mem.sites() {
        let frames: Vec<ProfileFrame> = site
            .parsed_frames()
            .into_iter()
            .map(|(func, file, line)| {
                ProfileFrame::new(func, file.unwrap_or_default(), i64::from(line.unwrap_or(0)))
            })
            .collect();
        let sample = ProfileSample::new(frames, site.allocated_bytes as i64)
            .with_current_span()
            .with_attribute("allocations.count", site.allocations.to_string())
            .with_attribute("live.bytes", site.live_bytes.to_string());
        batch = batch.add_sample(sample);
    }
    batch
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:4318".into());
    let profiles_http_endpoint = std::env::var("AIRBUG_PROFILES_HTTP_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:4318".into());

    let guard = init(
        TelemetryConfig::new()
            .service_name("airbug-full-stack-demo")
            .service_version(env!("CARGO_PKG_VERSION"))
            .environment("dev")
            .endpoint(&otlp_endpoint)
            .sampler(TraceSampler::ParentBasedRatio(1.0))
            .histogram_buckets(
                "checkout.latency_ms",
                vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0],
            ),
    )?;

    let _err_guard = airbug_err::init(
        airbug_err::Options::new()
            .service("airbug-full-stack-demo")
            .release(env!("CARGO_PKG_VERSION"))
            .environment("dev"),
    )?;

    // Register all 7 synchronous & asynchronous metric instrument families.
    let req_counter = U64Counter::with_metadata(
        "airbug.example",
        "checkout.requests",
        "{request}",
        "Total checkout requests",
    );
    let inflight = I64UpDownCounter::new("airbug.example", "checkout.inflight");
    let latency_hist = F64Histogram::new("airbug.example", "checkout.latency_ms");
    let payload_hist = U64Histogram::with_boundaries(
        "airbug.example",
        "checkout.payload_bytes",
        vec![128.0, 1024.0, 8192.0],
    );
    let cpu_gauge = F64Gauge::with_description("airbug.example", "process.cpu_ratio", "CPU ratio");
    let queue_gauge = U64Gauge::new("airbug.example", "checkout.queue_depth");
    let skew_gauge = I64Gauge::new("airbug.example", "clock.skew_ms");
    let _obs_rss = ObservableF64Gauge::new("airbug.example", "process.rss_mib", |obs| {
        obs.observe(64.25, &[KeyValue::new("allocator", "airbug-bench")]);
    });
    let _obs_polls = ObservableU64Counter::new("airbug.example", "runtime.polls", |obs| {
        obs.observe(1024, &[]);
    });
    let _obs_workers =
        ObservableI64UpDownCounter::new("airbug.example", "runtime.active_workers", |obs| {
            obs.observe(4, &[]);
        });

    let subscriber = tracing_subscriber::registry().with(tracing_layer());
    let _sub_guard = tracing::subscriber::set_default(subscriber);

    inflight.add(1, &[]);
    req_counter.add(1, &[KeyValue::new("region", "eu-central")]);
    add_counter("airbug.example", "cache.hits", 5, &[]);
    record_histogram("airbug.example", "checkout.latency_ms", 14.2, &[]);
    latency_hist.record(18.7, &[KeyValue::new("route", "/checkout")]);
    payload_hist.record(1536, &[]);
    cpu_gauge.record(0.37, &[]);
    queue_gauge.record(2, &[]);
    skew_gauge.record(-1, &[]);

    // 1. Producer span with W3C Baggage + TraceContext header injection.
    let mut outbound_headers = HashMap::new();
    let upstream_tp = in_span_async("airbug.example", "api.gateway", async {
        let tp = process_checkout_request(42).await;
        with_baggage(&[KeyValue::new("tenant.id", "acme-corp")], || {
            inject_map(&mut outbound_headers);
        });
        tp
    })
    .await;

    // 2. Downstream async worker extracting W3C headers, linking batch parent, capturing error & profile.
    with_extracted_headers_async(&outbound_headers, async {
        let mut opts = SpanOptions::new()
            .kind(SpanKind::Consumer)
            .attribute("worker.queue", "payments");
        if let Some(link) =
            SpanLink::from_traceparent(&upstream_tp, vec![KeyValue::new("link.type", "enqueued")])
        {
            opts = opts.link(link);
        }

        in_span_async_with_options("airbug.example", "payment.worker", opts, async {
            let tenant = baggage_value("tenant.id").unwrap_or_else(|| "unknown".into());
            add_span_event(
                "baggage.resolved",
                &[KeyValue::new("tenant.id", tenant.clone())],
            );

            // Simulate a recoverable payment gateway error — airbug-err automatically stamps
            // trace_id + span_id on the error event AND records an exception event on this span!
            let gateway_err = io::Error::new(io::ErrorKind::TimedOut, "PSP upstream timeout");
            let _ = airbug_err::capture_error(&gateway_err);

            // Build & export an OTLP Profile correlated with this exact trace & span!
            let synthetic_bench_profile = MemoryProfile {
                schema: "bench.memory/1".into(),
                scope: "demo".into(),
                dropped_events: 0,
                events: 2,
                peak_tracked_bytes: 12_288,
                sites: vec![
                    AllocationSite {
                        frames: vec![
                            "full_stack::main (otel/examples/full_stack.rs:140)".into(),
                            "full_stack::serialize_receipt (otel/examples/full_stack.rs:85)".into(),
                        ],
                        allocated_bytes: 8192,
                        allocations: 16,
                        live_bytes: 0,
                        live_blocks: 0,
                    },
                    AllocationSite {
                        frames: vec![
                            "full_stack::main (otel/examples/full_stack.rs:140)".into(),
                            "full_stack::sign_payload (otel/examples/full_stack.rs:92)".into(),
                        ],
                        allocated_bytes: 4096,
                        allocations: 4,
                        live_bytes: 512,
                        live_blocks: 1,
                    },
                ],
            };
            let profile_proto =
                bench_memory_to_otlp_profile("airbug-full-stack-demo", &synthetic_bench_profile)
                    .to_proto();
            if let Err(err) = export_profiles_http(
                &profiles_http_endpoint,
                &profile_proto,
                Protocol::Http,
                true,
            ) {
                eprintln!("note: profiles export skipped ({err})");
            }

            if let Some(tid) = current_trace_id() {
                println!("Completed full-stack trace_id={tid} (tenant={tenant})");
            }
        })
        .await;
    })
    .await;

    in_span("airbug.example", "cleanup", || {
        inflight.add(-1, &[]);
    });

    std::thread::sleep(Duration::from_millis(150));
    guard.shutdown()?;
    Ok(())
}
