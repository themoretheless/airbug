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

1. **Docker Compose** — `otel-collector` + Jaeger UI (`docker-compose.yml` + `config.yaml`).
   Reached through the `docker compose` plugin, or a standalone `docker-compose` binary when the plugin is absent.
2. **Binary** — `otelcol-contrib` or `otelcol` on `PATH` with `config.standalone.yaml` (no Jaeger)

Ctrl+C stops whichever backend was started. Without `--collector` the hub installs no
signal handler, and a background job started from a non-interactive shell inherits
SIGINT as ignore — stop it with SIGTERM.

Send app telemetry:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
# Correlate with a hub bench run (from POST /api/v1/bench/runs):
# export OTEL_RESOURCE_ATTRIBUTES=airbug.hub_id=…,airbug.run_id=…
cargo run -p airbug-otel --example span
cargo run -p airbug-otel --example metrics
cargo run -p airbug-otel --example logs
```

### Live bench sessions (register-then-run)

Hub does **not** spawn benches. Client registers, writes artifacts under `.airbug-bench/runs/{run_id}/`, and sends OTLP with correlation attrs.

1. `POST /api/v1/bench/runs` → `{hub_id, run_id, out_dir, dash_url}`
2. Write `progress.json` while running; `status-final.json` + `run.json` / `report.html` when done
3. Open `http://127.0.0.1:8790/#/bench/{run_id}` (printed before measure)

Correlation keys (OTEL resource / err tags): `airbug.hub_id`, `airbug.run_id`.  
Hub instance UUID lives in `dash/hub/data/hub_id` and is exposed on `GET /api/v1/status`.

```bash
export AIRBUG_HUB=http://127.0.0.1:8790
cargo run -p cargo-airbug-bench -- run --plan …   # registers, sets -o to out_dir
# or lin: scripts/ci-bench.sh
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
  runs.rs          GUID hub_id + bench run registry (runs.sqlite)
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
| GET | `/api/v1/status` | Domain snapshot + Local APIs + `hub_id` |
| GET | `/api/v1` | API catalog only |
| GET | `/api/v1/logs?limit=&run_id=` | OTLP log tail (+ `by_severity`, `services`); filter by `airbug.run_id` |
| GET | `/api/v1/metrics?limit=&run_id=` | OTLP metrics (`series`, `histogram`, `latest`); optional `run_id` filter |
| POST | `/api/v1/bench/runs` | Register run → `{hub_id, run_id, out_dir, dash_url}` |
| GET | `/api/v1/bench/runs` | List runs (running first, then recent done) |
| GET | `/api/v1/bench/runs/:id` | Manifest + live `progress.json` / `status-final.json` |
| GET | `/api/v1/bench/runs/:id/report` | Report when present |
| GET | `/bench/runs/:id/*` | Bench artifacts under `.airbug-bench/runs/{id}/` |
| POST | `/api/v1/errors` | airbug-err event → issues (`schema_version`; tag `airbug.run_id`) |
| POST | `/api/v1/events` | Unified error/trace/metric/log envelope |
| GET | `/api/v1/issues?run_id=` | Issue list (optional filter by tag) |
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
| unit | `target/airbug-report/{report.json,index.html}` |
| bench | `.airbug-bench/**/run.json` + GUID registry `dash/hub/data/runs.sqlite` |
| mon | `~/.local/share/airbug-mon/airbug-mon.db` |
| otel | `otel/` crate (`airbug-otel` OTLP) |
| err | `err/` crate (`airbug-err`) + `dash/hub/data/issues.sqlite` |
| collector | localhost `:4317` / `:4318` / Jaeger `:16686` |
| logs (Explore-style) | `dash/collector/data/logs.json` via `/api/logs` |
| metrics (Grafana-like) | `dash/collector/data/metrics.json` via `/api/metrics` — Time series, Gauge, Bar, Histogram, Heatmap, Pie, Table |
| issues | SQLite via `/api/issues` + ingest `POST /api/errors` |

Traces are **not** embedded next to logs; open **Jaeger UI** from Local APIs when the Docker collector is up (`http://127.0.0.1:16686/`).

Optional new-issue webhook: `--webhook URL` or `AIRBUG_ISSUES_WEBHOOK` (**`http://` only** — HTTPS is refused).
