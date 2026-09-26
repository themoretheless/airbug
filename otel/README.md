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
| `testing` | in-memory exporters + `install_test` | — |

Enable only one OTLP transport for normal builds. With `--all-features` both
compile in and HTTP stays the default, so `OTEL_EXPORTER_OTLP_ENDPOINT` pointing at
`:4318` keeps working; opt into gRPC with `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` and
switch the endpoint to `:4317`. A requested transport that is not compiled in is
ignored in favour of the one that is.

```bash
cargo test -p airbug-otel --features testing
```

## API

| Item | Role |
|------|------|
| `TelemetryConfig` / `TelemetryError` / `TelemetryGuard` | service name + endpoint; errors; keep-alive guard |
| `TelemetryHandle` | alias for `TelemetryGuard` (`install` / `init`) |
| `TraceConfig` / `TraceError` / `TracerGuard` | deprecated aliases |
| `init` / `install` | global tracer, meter, **and** logger (OTLP) |
| `install_test` | in-memory providers (`feature = "testing"`) |
| `TelemetryGuard::force_flush` / `shutdown` | flush without / with teardown |
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
