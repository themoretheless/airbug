# trace

OpenTelemetry **traces and metrics** for the airbug monorepo.

Package: `airbug-trace` — install/shutdown wrapper over official
`opentelemetry` + OTLP exporters. Host UI metrics stay in `mon/`; microbenchmarks in `bench/`.

## Quick start

```bash
cargo test -p airbug-trace
cargo run -p airbug-trace --example span
cargo run -p airbug-trace --example metrics
```

Point at a collector (defaults: HTTP `http://localhost:4318`):

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
export OTEL_SERVICE_NAME=my-app
cargo run -p airbug-trace --example metrics
```

```rust
use airbug_trace::{add_counter, in_span, init, TelemetryConfig};
use opentelemetry::KeyValue;

let guard = init(TelemetryConfig::new().service_name("checkout"))?;
in_span("checkout", "place_order", || {
    add_counter("checkout", "orders.placed", 1, &[KeyValue::new("currency", "USD")]);
});
guard.shutdown()?;
```

## Features

| Feature | Transport | Default collector port |
|---------|-----------|------------------------|
| `otlp-http` (default) | OTLP HTTP/protobuf | 4318 |
| `otlp-grpc` | OTLP gRPC (tonic) | 4317 |

Enable only one for normal builds. With `--all-features`, gRPC wins.

## API

| Item | Role |
|------|------|
| `TelemetryConfig` / `TraceConfig` | service name + endpoint |
| `init` | global tracer **and** meter (OTLP) |
| `TelemetryGuard` | flush on `shutdown` / `Drop` |
| `in_span` / `set_attribute` | span helpers |
| `meter` / `add_counter` / `record_histogram` | metric helpers |

## Env

`OTEL_SERVICE_NAME`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_EXPORTER_OTLP_HEADERS`, …

Local collector (with hub):

```bash
cargo run -p airbug-hub -- serve --root . --collector
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
```
