# dash

Cross-cutting **hub** for the monorepo. Aggregates status from `unit`, `bench`, `mon`, `otel`, and the local OTEL collector without owning those products (SRP).

## Run

```bash
# Dashboard only
cargo run -p airbug-hub -- serve --root .
# http://127.0.0.1:8790/

# Dashboard + OpenTelemetry Collector (Docker **or** local otelcol)
cargo run -p airbug-hub -- serve --root . --collector
# OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317
# Jaeger http://127.0.0.1:16686/  (Docker mode only)
# Logs appear on the hub page (from dash/collector/data/logs.json)

cargo run -p airbug-hub -- status --root .
```

Collector lives in `dash/collector/`. `--collector` tries, in order:

1. **Docker Compose** — `otel-collector` + Jaeger UI (`docker-compose.yml` + `config.yaml`)
2. **Binary** — `otelcol-contrib` or `otelcol` on `PATH` with `config.standalone.yaml` (no Jaeger)

Ctrl+C stops whichever backend was started.

Send app telemetry:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
cargo run -p airbug-otel --example span
cargo run -p airbug-otel --example metrics
cargo run -p airbug-otel --example logs
```

## What it reads

| Domain | Sources |
|--------|---------|
| unit | `target/airbug-report/{report.json,index.html}` |
| bench | `.airbug-bench/**/run.json` |
| mon | `~/.local/share/airbug-mon/airbug-mon.db` |
| otel | `otel/` crate (`airbug-otel` OTLP) |
| err | `err/` crate (`airbug-err`) + `dash/hub/data/issues.sqlite` |
| collector | localhost `:4317` / `:4318` / Jaeger `:16686` |
| logs (Explore-style) | `dash/collector/data/logs.json` via `/api/logs` — severity bar, text/service filters |
| metrics (Grafana-like) | `dash/collector/data/metrics.json` via `/api/metrics` — viz: Time series, Gauge, Bar, Histogram, Heatmap, Pie, Table |
| issues panel | SQLite via `/api/issues` + ingest `POST /api/errors` |

Traces are **not** embedded next to logs; open **Jaeger UI** from Local APIs when the Docker collector is up (`http://127.0.0.1:16686/`).

It copies suggested commands into the clipboard; it does not start airbug-mon or bench for you.

While the hub is running, the **Local APIs** block lists only endpoints that are up
(`/api/status`, `/api/logs`, `/api/metrics`, `/api/errors`, `/api/issues`, `/api`, unit report if present, OTLP/Jaeger when `--collector` is live).

Issues come from `airbug-err` via `POST /api/errors` and are stored in `dash/hub/data/issues.sqlite`.
Optional new-issue webhook: `--webhook URL` or `AIRBUG_ISSUES_WEBHOOK`.
