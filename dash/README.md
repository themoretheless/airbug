# dash

Cross-cutting **hub** for the monorepo. Aggregates status from `unit`, `bench`, `mon`, `trace`, and the local OTEL collector without owning those products (SRP).

## Run

```bash
# Dashboard only
cargo run -p airbug-hub -- serve --root .
# http://127.0.0.1:8790/

# Dashboard + OpenTelemetry Collector + Jaeger UI (Docker required)
cargo run -p airbug-hub -- serve --root . --collector
# OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317
# Jaeger http://127.0.0.1:16686/

cargo run -p airbug-hub -- status --root .
```

Collector stack lives in `dash/collector/` (`docker-compose.yml` + `config.yaml`).  
`--collector` runs `docker compose up -d` on start and `down` when the hub exits.

Send app telemetry:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
cargo run -p airbug-trace --example span
cargo run -p airbug-trace --example metrics
```

## What it reads

| Domain | Sources |
|--------|---------|
| unit | `target/airbug-report/{report.json,index.html}` |
| bench | `.airbug-bench/**/run.json` |
| mon | `~/.local/share/airbug-mon/airbug-mon.db` |
| trace | `trace/` crate (`airbug-trace` OTLP traces+metrics) |
| collector | localhost `:4317` / `:4318` / Jaeger `:16686` |

It copies suggested commands into the clipboard; it does not start airbug-mon or bench for you.
