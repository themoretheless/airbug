# `airbug-otel`

Full-coverage OpenTelemetry 1.x / OTLP facade for Airbug covering **all 4 OpenTelemetry signals** (**Traces**, **Metrics**, **Logs**, and **Profiles `v1development`**), W3C distributed context propagation (`traceparent`, `tracestate`, `baggage`), a zero-boilerplate `tracing_subscriber::Layer` bridge (`TracingOtelLayer`), and automatic trace correlation with `airbug-err`.

## Feature Flags

| Feature | Default | Description |
| --- | :---: | --- |
| `otlp-http` | **Yes** | OTLP over HTTP (`http/protobuf` and `http/json`, plus `gzip` compression) on port `:4318`. |
| `tracing-bridge` | **Yes** | `TracingOtelLayer` (`airbug_otel::tracing_layer()`) bridging `#[tracing::instrument]` and `tracing::info!/warn!/error!` into OTel spans and structured OTel logs. |
| `otlp-grpc` | No | OTLP over Tonic gRPC (with optional `gzip` compression and metadata headers) on port `:4317`. Automatically manages an internal Tokio runtime if none is active. |
| `testing` | No | Deterministic, zero-network in-memory exporters (`install_test`) for spans, metrics, logs, and profiles. |

When both `otlp-http` and `otlp-grpc` are compiled in, **HTTP (`http/protobuf`) remains the default** unless overridden via `TelemetryConfig::protocol(...)` or `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` / `http/json`.

## Quick Start

```rust,no_run
use airbug_otel::{
    Compression, KeyValue, ObservableF64Gauge, Protocol, Severity, SpanKind, SpanOptions,
    TelemetryConfig, TraceSampler, emit_log_with_attrs, in_span_async_with_options, init,
    record_histogram, tracing_layer,
};
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guard = init(
        TelemetryConfig::new()
            .service_name("checkout-api")
            .service_version("1.4.0")
            .environment("prod")
            .protocol(Protocol::Http) // or Protocol::HttpJson / Protocol::Grpc
            .compression(Compression::Gzip)
            .sampler(TraceSampler::ParentBasedRatio(1.0))
            .histogram_buckets("http.server.duration_ms", vec![1.0, 5.0, 25.0, 100.0, 500.0]),
    )?;

    let subscriber = tracing_subscriber::registry().with(tracing_layer());
    let _sub = tracing::subscriber::set_default(subscriber);

    let _rss = ObservableF64Gauge::new("checkout", "process.rss_mib", |obs| {
        obs.observe(128.0, &[]);
    });

    in_span_async_with_options(
        "checkout",
        "POST /orders",
        SpanOptions::new().kind(SpanKind::Server),
        async {
            tracing::info!(user_id = 42, "creating order");
            record_histogram("checkout", "http.server.duration_ms", 11.4, &[]);
            emit_log_with_attrs(
                "checkout",
                Severity::Info,
                "order.created",
                "order persisted",
                &[KeyValue::new("order.id", "ord_123")],
            );
        },
    )
    .await;

    guard.shutdown()?;
    Ok(())
}
```

## Capabilities by Module

- **Lifecycle & Configuration (`airbug_otel`)**:
  - `init` / `install` returning RAII `TelemetryGuard` (`force_flush()`, `shutdown()`, safe `Drop` inside or outside Tokio runtimes, resetting global providers cleanly on shutdown).
  - Rich `TelemetryConfig`: `.service_name()`, `.service_version()`, `.environment()`, `.hub_id()`, `.run_id()`, `.resource_attribute()`, `.endpoint()`, `.protocol(Protocol::{Http, HttpJson, Grpc})`, `.compression(Compression::{None, Gzip})`, `.header(k, v)`, `.timeout(d)`, `.sampler(TraceSampler::*)`, `.histogram_buckets(name, boundaries)`.
- **Traces (`airbug_otel::traces`)**:
  - Sync & async context-preserving spans: `in_span`, `in_span_with_options`, `in_span_async`, `in_span_async_with_options`.
  - `SpanOptions` (`SpanKind::{Internal, Server, Client, Producer, Consumer}`, initial attributes, `SpanLink::from_traceparent` / `SpanLink::from_current`).
  - Active-span helpers: `set_attribute`, `set_attributes`, `add_span_event`, `set_span_status_ok`, `set_span_status_error`, `update_span_name`, `record_exception`, `record_exception_message`.
- **W3C Propagation (`airbug_otel::propagation`)**:
  - Composite W3C `TraceContextPropagator` + `BaggagePropagator` installed automatically.
  - `inject_map`, `inject_context_map`, `extract_map` (case-insensitive headers), `with_extracted_headers`, `with_extracted_headers_async`.
  - `current_trace_id`, `current_span_id`, `current_traceparent`, `with_baggage`, `with_baggage_async`, `baggage_value`.
- **Metrics (`airbug_otel::metrics`)**:
  - Instrument caching per `(scope, name, unit, description)` for zero-rebuild `add_counter`, `add_f64_counter`, `add_up_down_counter`, `record_histogram`, `record_u64_histogram`.
  - Wrappers for all 7 OTel instrument families: `U64Counter`, `F64Counter`, `I64UpDownCounter`, `F64UpDownCounter`, `F64Histogram`, `U64Histogram`, `F64Gauge`, `U64Gauge`, `I64Gauge`, `ObservableF64Gauge`, `ObservableU64Counter`, `ObservableI64UpDownCounter`.
- **Logs (`airbug_otel::logs`)**:
  - `emit_structured_log(LogRecordSpec)`, `emit_log_with_attrs`, `emit_log`, `log_trace`, `log_debug`, `log_info`, `log_warn`, `log_error`, `log_fatal`.
  - Automatic active-span correlation (`trace_id`, `span_id`, `trace_flags`) on every emitted log record.
- **`tracing` Bridge (`airbug_otel::tracing_bridge`)**:
  - `TracingOtelLayer` (`tracing_layer()`) attaches OTel `Context` on `on_enter`/`on_exit` so `current_trace_id()`, `emit_log`, and `airbug_err::capture_*` inside `#[tracing::instrument]` functions automatically share the span's `trace_id` and `span_id`.
- **OTLP Profiles (`airbug_otel::profiles`)**:
  - `ProfileFrame`, `ProfileSample` (with `.with_current_span()`), and `ProfileBatch::to_proto()` producing dictionary-deduplicated `opentelemetry_proto::tonic::profiles::v1::ProfilesData`.
  - `export_profiles_http(endpoint, &data, protocol, gzip)` posting `application/x-protobuf` or `application/json` to `/v1development/profiles` (and recording into `TestExporters.profiles` in tests).

## Environment Variables Honored

| Variable | Effect |
| --- | --- |
| `OTEL_SERVICE_NAME` | Fallback `service.name` resource attribute (default `"airbug"`). |
| `OTEL_RESOURCE_ATTRIBUTES` | Standard `k=v,k2=v2` resource attributes. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Collector base URL (`http://127.0.0.1:4318` for HTTP, `http://127.0.0.1:4317` for gRPC). |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` (default), `http/json`, or `grpc`. |
| `OTEL_EXPORTER_OTLP_COMPRESSION` | `gzip` or unset. |
| `OTEL_EXPORTER_OTLP_HEADERS` | Custom request headers / gRPC metadata (`k=v,k2=v2`). |
| `OTEL_EXPORTER_OTLP_TIMEOUT` | Exporter timeout in milliseconds. |
| `OTEL_TRACES_SAMPLER` / `OTEL_TRACES_SAMPLER_ARG` | `always_on`, `always_off`, `traceidratio`, `parentbased_always_on`, `parentbased_always_off`, `parentbased_traceidratio`. |
| `AIRBUG_HUB_ID` / `AIRBUG_RUN_ID` | Automatically attached as `airbug.hub_id` and `airbug.run_id` resource attributes. |

## Examples

```sh
cargo run -p airbug-otel --example span
cargo run -p airbug-otel --example metrics
cargo run -p airbug-otel --example logs
cargo run -p airbug-otel --example full_stack
```
