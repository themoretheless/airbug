# otel

OpenTelemetry **traces, metrics, and logs** for the airbug monorepo.

Package: `airbug-otel` — install/shutdown wrapper over official
`opentelemetry` + OTLP exporters. Host UI metrics stay in `mon/`; microbenchmarks in `bench/`.

## Quick start

```bash
cargo test -p airbug-otel
cargo run -p airbug-otel --example span
cargo run -p airbug-otel --example metrics
cargo run -p airbug-otel --example logs
```

Point at a collector (defaults: HTTP `http://localhost:4318`):

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
export OTEL_SERVICE_NAME=my-app
cargo run -p airbug-otel --example logs
```

```rust
use airbug_otel::{init, log_info, in_span, TelemetryConfig};

let guard = init(TelemetryConfig::new().service_name("checkout"))?;
in_span("checkout", "place_order", || {
    log_info("checkout", "order accepted");
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
| `init` | global tracer, meter, **and** logger (OTLP) |
| `TelemetryGuard` | flush on `shutdown` / `Drop` |
| `in_span` / `set_attribute` | span helpers |
| `meter` / `add_counter` / `record_histogram` | metric helpers |
| `emit_log` / `log_info` / `log_warn` / `log_error` | log helpers |

## Env

`OTEL_SERVICE_NAME`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_EXPORTER_OTLP_HEADERS`, …

Local collector (with hub):

```bash
cargo run -p airbug-hub -- serve --root . --collector
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
cargo run -p airbug-otel --example logs
# watch the OTLP logs panel on http://127.0.0.1:8790/
```
