# dash

Cross-cutting **hub** for the monorepo. Aggregates status from `unit`, `bench`, `mon`, `otel`, `err`, and the local OTEL collector without owning those products (SRP).

**Local-only:** binds `127.0.0.1` — a production-quality *developer tool*, not a public SaaS.

## Run

```bash
# Dashboard only
cargo run -p airbug-hub -- serve --root .
# http://127.0.0.1:8790/

# Dashboard + OpenTelemetry Collector (Docker **or** local otelcol)
cargo run -p airbug-hub -- serve --root . --collector
# OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317
# Jaeger http://127.0.0.1:16686/  (Docker mode only)

cargo run -p airbug-hub -- status --root .
cargo test -p airbug-hub
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

## Architecture (`airbug-hub`)

```text
dash/hub/src/
  main.rs          CLI + tracing
  app.rs           HubApp composition root
  config.rs        RootPaths + limits (DIP for artifact paths)
  error.rs         HubError (thiserror); JSON `{ok:false,error}`
  http.rs          request parse / respond helpers
  serve.rs         route table (/api/v1 + legacy /api aliases)
  collector.rs     Docker / otelcol lifecycle
  scan/            domain cards (unit, bench, mon, otel, err, collector)
  otlp/            shared file tail + logs/metrics parsers
  issues.rs        IssueStore + SqliteIssueStore + http:// webhook (reqwest)
  static/          dashboard.html + dashboard.css + dashboard.js
```

Paths are centralized in `config::RootPaths` (collector data, issues DB, unit report). OTLP JSON helpers live once under `otlp/` (DRY). Severity normalization is server-side in Rust (`otlp::normalize_severity`); the UI displays labels as returned.

## HTTP API

Prefer `/api/v1/...`. Legacy `/api/...` paths remain as aliases for one release.

| Method | Path | Body |
|--------|------|------|
| GET | `/` | Dashboard HTML |
| GET | `/static/dashboard.css` | Styles |
| GET | `/static/dashboard.js` | Client UI |
| GET | `/api/v1/status` | Domain snapshot + Local APIs |
| GET | `/api/v1` | API catalog only |
| GET | `/api/v1/logs?limit=` | OTLP log tail (+ `by_severity`, `services`) |
| GET | `/api/v1/metrics?limit=` | OTLP metrics (`series`, `histogram`, `latest`) |
| POST | `/api/v1/errors` | airbug-err event → issues (`schema_version`) |
| GET | `/api/v1/issues` | Issue list |
| GET | `/api/v1/issues/:id` | Issue detail (`last_event` + recent `events`) |
| POST | `/api/v1/issues/:id/{resolve,ignore,reopen}` | Status change |
| GET | `/report/*` | Unit HTML report files |

Error JSON shape: `{ "ok": false, "error": "…" }`.

Threat model: [SECURITY.md](../SECURITY.md).

## Unified collector event model

Airbug is a Sentry alternative with its own collector contract rather than a
Sentry protocol implementation. The hub normalizes error events and OTLP
signals around one envelope:

| Common field | Purpose |
|--------------|---------|
| `event_id`, `timestamp` | Event identity and ordering |
| `service`, `environment`, `release` | Deployment context |
| `trace_id`, `span_id` | Error/log/metric correlation |
| `tags` | Searchable dimensions |
| `payload.signal` | `error`, `trace`, `metric`, or `log` |

Error payloads retain breadcrumbs, user, exception, stacktrace, contexts, and
extras. Trace, metric, and log payloads retain their signal-native data while
sharing the same correlation fields.

## What it reads

| Domain | Sources |
|--------|---------|
| unit | local Rust test results from the workspace |
| bench | `.airbug-bench/**/run.json` |
| mon | `~/.local/share/airbug-mon/airbug-mon.db` |
| otel | `otel/` crate (`airbug-otel` OTLP) |
| err | `err/` crate (`airbug-err`) + `dash/hub/data/issues.sqlite` |
| collector | localhost `:4317` / `:4318` / Jaeger `:16686` |
| logs (Explore-style) | `dash/collector/data/logs.json` via `/api/logs` |
| metrics (Grafana-like) | `dash/collector/data/metrics.json` via `/api/metrics` — Time series, Gauge, Bar, Histogram, Heatmap, Pie, Table |
| issues | SQLite via `/api/issues` + ingest `POST /api/errors` |

Traces are **not** embedded next to logs; open **Jaeger UI** from Local APIs when the Docker collector is up (`http://127.0.0.1:16686/`).

Optional new-issue webhook: `--webhook URL` or `AIRBUG_ISSUES_WEBHOOK` (**`http://` only** — HTTPS is refused).
