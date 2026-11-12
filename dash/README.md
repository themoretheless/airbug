# dash

Cross-cutting **hub** for the monorepo. Aggregates status from `unit`, `bench`, `mon`, `otel`, `err`, and the local OTEL collector without owning those products (SRP).

**Local-only:** binds `127.0.0.1` — a production-quality *developer tool*, not a public SaaS.

## Run

```bash
# Dashboard + built-in OTLP HTTP on :8790 (/v1/traces, /v1/metrics, /v1/logs, /v1development/profiles)
cargo run -p airbug-hub -- serve --root .
# http://127.0.0.1:8790/

# Dashboard + OpenTelemetry Collector (Docker → local otelcol → built-in Rust :4318 HTTP + :4317 gRPC)
cargo run -p airbug-hub -- serve --root . --collector
# Force the zero-dependency built-in Rust collector explicitly:
cargo run -p airbug-hub -- serve --root . --collector-mode builtin
# OTLP http://127.0.0.1:4318  grpc://127.0.0.1:4317
# Sentry DSN http://airbug@127.0.0.1:8790/1
# Jaeger http://127.0.0.1:16686/  (Docker mode only; optional external link alongside native Traces tab)

# .NET Aspire-style stack orchestrator: start hub + collector + .airbug/stack.json resources
cargo run -p airbug-hub -- up --root .

cargo run -p airbug-hub -- status --root .
cargo test -p airbug-hub
```

### Live test runs & TestOps (`cargo airbug test`)

```bash
cargo airbug test -- --workspace --exclude airbug-mon   # = cargo run -p airbug-hub -- test -- …
cargo airbug test --junit target/junit.xml -- --workspace
```

The runner wraps `cargo test --no-fail-fast`, lists tests first (real progress bar), and
writes `.airbug/runs/{run_id}/` as it goes: `manifest.json`, `progress.json`,
`report.json` (`airbug.test-report/1`), `junit.xml`, `output.log`, `events/` (from `airbug::report`,
via `AIRBUG_REPORT_DIR`) and a self-contained `index.html`. The hub only reads these
files, so the run is live at `#/tests/{run_id}` whether the hub started first or not.

**Allure / ReportPortal-style TestOps views (`#/tests`):**
- **Suites:** Grouped by test binary / suite with failure-first sorting and historical pass/fail strips.
- **Behaviors (`Epic › Feature › Story`):** Hierarchical grouping populated from `airbug::report::epic(...)`, `feature(...)`, and `story(...)`.
- **Defects:** Automated failure classification into `product_bug` (assertion/comparison diffs), `infrastructure` (connection refused, timeouts, suite crashes), `test_bug` (unclassified panics), `flaky` (passed on retry / marked flaky), and `quarantined` (`airbug::report::quarantined("ISSUE-123")`), plus custom regex/substring rules in `.airbug/defect-rules.json` (`GET/POST /api/v1/tests/rules`).
- **Slowest:** Top slowest tests in the run with duration bars.
- **Trace Correlation (`trace:<short> ↗`):** Tests that call `airbug::report::trace_context(trace_id, span_id)` (or auto-capture active OTLP spans) link directly to `#/traces/{trace_id}`.
- **JUnit XML Export:** Pass `--junit <FILE>` on the CLI or download `/api/v1/runs/{run_id}/junit.xml` directly from the UI header.

### .NET Aspire-style Resource Orchestrator (`#/resources` & `airbug-hub up`)

Define local processes and containers in `.airbug/stack.json` (or `airbug.stack.json`):

```json
{
  "schema": "airbug.stack/1",
  "resources": [
    {
      "name": "api",
      "kind": "process",
      "command": "cargo",
      "args": ["run", "-p", "my-api"],
      "ports": [{ "host": 3000, "scheme": "http" }],
      "depends_on": [],
      "healthcheck": { "kind": "http", "url": "http://127.0.0.1:3000/health" }
    }
  ]
}
```

- Always surfaces built-in `airbug-hub` and `otlp-collector` plus discovered Docker containers even without a `stack.json`.
- `airbug-hub up` starts the hub, OTLP collector, and all stack resources in topological `depends_on` order, and tears them down cleanly on Ctrl+C / exit.
- Automatically injects `OTEL_SERVICE_NAME`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_RESOURCE_ATTRIBUTES`, `AIRBUG_ERR_ENDPOINT`, `AIRBUG_SENTRY_DSN`, and .NET Aspire-compatible `SERVICES__<NAME>__HTTP` service discovery variables into every child process.
- Captures per-resource `stdout`/`stderr` ring buffers (`GET /api/v1/resources/:name/logs`) and masks secret environment variables (`PASSWORD`, `SECRET`, `TOKEN`, `KEY`, `DSN`, `AUTH`, `CREDENTIAL`).

### Sentry-grade Issue Intelligence & Envelope Receiver (`#/issues`)

- **Native Sentry SDK Compatibility:** Point any official Sentry SDK (`sentry-rust`, `@sentry/browser`, `sentry-python`, `sentry-go`) at `SENTRY_DSN=http://airbug@127.0.0.1:8790/1`. The hub accepts `POST /api/:project/envelope/` and `POST /api/:project/store/` (plain or `gzip`).
- **Inline Source Context (`±5` lines):** Stacktrace frames with workspace-local paths automatically resolve 5 lines of `pre_context`, `context_line`, and `post_context` (strictly confined inside `paths.root`).
- **Suspect Commit (`git blame`):** Automatically identifies the commit author, SHA, and subject that last touched the top in-app crashing frame (`git log -1 -L`).
- **Auto-Regression & Lifecycle:** Issues support `unresolved`, `resolved`, `resolved_in_next_release`, and `ignored`. If a new event arrives for a resolved issue (or from a newer release), it automatically transitions back to `unresolved` with a `REGRESSED ×N` badge.
- **Release Health (`POST /api/v1/sessions` & `GET /api/v1/releases`):** Tracks `Crash-Free Sessions %`, session counts, crashed sessions, and issue counts per release.

Options: `--title`, `--out DIR`, `--port N` (hub to link), `--no-list`, `--quiet`,
`--keep 50` (runs kept in `.airbug/runs`). Cargo args after `--`, test-binary args after
a second `--`.

`--nextest` switches to `cargo nextest run` (needs `cargo-nextest`): the list comes from
`cargo nextest list --message-format json`, results from nextest's status lines, and a test
that passes on retry is marked flaky. Suites are nextest binary ids (`demo`,
`demo::integration`); doctests are not run, as with plain nextest.

**Rerun failed** on a finished run starts `airbug-hub test` again with the failed and
not-run tests only (libtest: `--exact` names; nextest: a `-E` filterset), linked back via
`manifest.rerun_of`. **Run again** repeats the original command. Both are
`POST /api/v1/runs/:id/rerun` and refuse while another test run is live (409). The
equivalent shell command is shown for copying.

The **Now** page compares the newest finished bench run with the previous one that has the
same cases, contracts and environment (or baseline vs candidate inside one crossover run),
using the same statistics as `cargo airbug-bench compare` (±5 % threshold).

The dashboard: **Now** (live runs, latest test result, new failures, flaky, bench,
issues) · **Runs** (tests + bench timeline) · **Tests** (per-test view: failures first,
output, steps, diffs, attachments, history strip) · **Bench** · **Signals** (**Issues** ·
**Traces** — waterfall, Critical Path, Service Map, RED, zPages Tracez · **Logs** ·
**Metrics** — percentiles & exemplars · **Profiles** — interactive Flamechart) ·
**Health** (package cards, API catalog).

Collector config lives in `dash/collector/`. `--collector` (`--collector-mode auto|builtin|docker|binary`) tries, in order:

1. **Docker Compose** — `otel-collector` + Jaeger UI (`docker-compose.yml` + `config.yaml`, writing `traces.json`, `metrics.json`, and `logs.json` plus forwarding spans to Jaeger).
   Reached through the `docker compose` plugin, or a standalone `docker-compose` binary when the plugin is absent.
2. **Binary** — `otelcol-contrib` or `otelcol` on `PATH` with `config.standalone.yaml` (writes `traces.json`, `metrics.json`, `logs.json`; no Jaeger).
3. **Built-in Native Rust Receiver (`builtin`)** — starts inside `airbug-hub` on `127.0.0.1:4318` (OTLP HTTP `application/x-protobuf` + `application/json` + `gzip` for all 4 signals) and `127.0.0.1:4317` (Tonic OTLP gRPC `TraceService`, `MetricsService`, `LogsService` with `gzip`), rotating NDJSON files at 32 MiB (up to 3 backups). The main hub port (`127.0.0.1:8790`) also always accepts OTLP HTTP at `/v1/{traces,metrics,logs}` and `/v1development/profiles`.

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
  serve.rs         route table (/api/v1 + /v1/* OTLP HTTP + legacy /api aliases)
  collector.rs     Docker / otelcol / built-in Rust OTLP (:4318 HTTP + :4317 gRPC) lifecycle
  scan/            domain cards (unit, bench, mon, otel, err, collector)
  otlp/            receiver (HTTP+gRPC+rotation), traces (waterfall/CP/RED/topology/tracez),
                   metrics (histograms/quantiles/exemplars), logs (trace correlation), profiles (flamechart)
  issues.rs        IssueStore + SqliteIssueStore + http:// webhook (reqwest)
  runs.rs          GUID hub_id + bench run registry (runs.sqlite)
  assets.rs        embedded static files, one table (add a row per new file)
  static/          dashboard.html + dashboard.css + js/ (ES modules, no build step)
```

The client is one module per responsibility under `static/js/`:

```
main.js          registers sections, starts the background feeds
shell.js         routes, nav groups, badges; dispatches to registered sections
api.js           every request (no-store, throws on non-2xx)
dom.js · format.js · charts.js   shared helpers, pure where possible
runs.js          run timeline + row renderer shared by Now, Runs and Tests
sections/*.js    one per tab: { cats, show(route, changed), hide?(), tick?(n) }
```

A new tab is a new `sections/*.js` registered in `main.js`, a panel in `dashboard.html`, and a
row in `assets.rs`; the shell does not change. `assets.rs` tests fail when a module or one of its
imports is not served.

Paths are centralized in `config::RootPaths` (collector data, issues DB, unit report). OTLP JSON helpers live once under `otlp/` (DRY). Severity normalization is server-side in Rust (`otlp::normalize_severity`); the UI displays labels as returned.

## HTTP API

Prefer `/api/v1/...`. Legacy `/api/...` paths remain as aliases for one release.

| Method | Path | Body |
|--------|------|------|
| GET | `/` | Dashboard HTML |
| GET | `/static/dashboard.css` | Styles |
| GET | `/static/js/*.js` | Client UI (ES modules; entry `main.js`) |
| GET | `/api/v1/status` | Domain snapshot + Local APIs + `hub_id` |
| GET | `/api/v1` | API catalog only |
| GET | `/api/v1/logs?limit=&run_id=&trace_id=` | OTLP log tail (+ `by_severity`, `services`, `trace_id`, `span_id`, `event_name`, `attrs`) |
| GET | `/api/v1/metrics?limit=&run_id=` | OTLP metrics (`series`, `histogram`, `percentiles` p50/p90/p95/p99, `exemplars`, `latest`) |
| GET | `/api/v1/traces?limit=&service=&status=&min_ms=&run_id=&q=` | Distributed traces list + `services`, `topology` (Service Map), `red` summary |
| GET | `/api/v1/traces/:trace_id` | Full trace waterfall + Critical Path + correlated `logs` and `issues` |
| GET | `/api/v1/zpages/tracez` | OpenTelemetry zPages Tracez latency buckets + running counts + error samples |
| GET | `/api/v1/profiles?limit=&service=&type=&run_id=&trace_id=` | OTLP Profiles (`v1development`) summaries, Self/Total top functions & merged `flamechart` |
| POST | `/v1/{traces,metrics,logs}` · `/v1development/profiles` | Built-in OTLP HTTP receiver (`application/x-protobuf`, `application/json`, optional `gzip`; also `/api/v1/otlp/...`) |
| GET | `/api/v1/runs?kind=&limit=` | Timeline of test + bench runs (`live` count; stale runs flagged) |
| GET | `/api/v1/runs/:id\|latest?lite=` | Test run: report, `changes` vs previous run, `history`, `flaky`, `rerun` commands (`lite=1` drops per-test data) |
| GET | `/api/v1/runs/:id/junit.xml` | JUnit XML export (`application/xml; charset=utf-8`) for the test run |
| GET | `/api/v1/tests/analytics?window=` | Cross-run TestOps stability, slowest tests, behavior tree (`epic › feature › story`), and defect summary |
| GET / POST | `/api/v1/tests/rules` | List or update custom defect classification rules (`.airbug/defect-rules.json`) |
| GET | `/api/v1/resources` | .NET Aspire-style local resource inventory (built-in, discovered containers, `.airbug/stack.json`) |
| GET | `/api/v1/resources/:name/logs?tail=` | Captured `stdout`/`stderr` console ring buffer for a managed resource |
| POST | `/api/v1/resources/:name/{start,stop,restart}` | Start, stop, or restart a managed stack resource |
| GET | `/runs/:id/*` | Files of a test run (`index.html`, `output.log`, `events/…`) |
| POST | `/api/v1/runs/:id/rerun` | JSON `{"failed":true\|false}` → starts a rerun, `202 {run_id, href, command}`; 409 while a run is live |
| GET | `/api/v1/bench/compare?run=` | Newest (or given) bench run vs previous comparable run: `summary`, `rows` (regressions first) |
| POST | `/api/v1/bench/runs` | Register run → `{hub_id, run_id, out_dir, dash_url}` |
| GET | `/api/v1/bench/runs` | List runs (running first, then recent done) |
| GET | `/api/v1/bench/runs/:id` | Manifest + live `progress.json` / `status-final.json` |
| GET | `/api/v1/bench/runs/:id/report` | Report when present |
| GET | `/bench/runs/:id/*` | Bench artifacts under `.airbug-bench/runs/{id}/` |
| POST | `/api/v1/errors` | airbug-err event → issues (`schema_version`; tag `airbug.run_id`) |
| POST | `/api/:project/envelope/` · `/api/:project/store/` | Native Sentry SDK Envelope & Store receiver (`application/x-sentry-envelope`, `application/json`, `gzip`) |
| POST | `/api/v1/sessions` | Release health session update (`init`, `ok`, `errored`, `crashed`, `exited`) |
| GET | `/api/v1/releases` | Release health summary (`crash_free_rate`, session counts, issue counts) + local Sentry DSN |
| POST | `/api/v1/events` | Unified error/trace/metric/log envelope |
| GET | `/api/v1/issues?run_id=` | Issue list (optional filter by tag, includes `is_regression`, `regression_count`) |
| GET | `/api/v1/issues/:id` | Issue detail (`last_event`, `events` with `±5` line source context, `suspect_commit`, `tag_distributions`, `activity_buckets`) |
| POST | `/api/v1/issues/:id/{resolve,resolve_next_release,ignore,reopen}` | Status change |
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
| unit / tests | `.airbug/runs/*/{manifest,progress,report}.json` (legacy: `target/airbug-report/`) |
| bench | `.airbug-bench/**/run.json` + GUID registry `dash/hub/data/runs.sqlite` |
| mon | `~/.local/share/airbug-mon/airbug-mon.db` |
| otel | `otel/` crate (`airbug-otel` OTLP) |
| err | `err/` crate (`airbug-err`) + `dash/hub/data/issues.sqlite` |
| collector | localhost `:4317` (gRPC) / `:4318` (HTTP) / optional Jaeger `:16686` |
| traces | `dash/collector/data/traces.json` (+ rotated backups) via `/api/v1/traces` & `/api/v1/zpages/tracez` |
| logs (Explore-style) | `dash/collector/data/logs.json` (+ rotated backups) via `/api/v1/logs` |
| metrics (Grafana-like) | `dash/collector/data/metrics.json` (+ rotated backups) via `/api/v1/metrics` — Time series, Percentiles (p50/p95/p99), Exemplars, Gauge, Bar, Histogram, Heatmap, Pie, Table |
| profiles | `dash/collector/data/profiles.json` (+ rotated backups) via `/api/v1/profiles` — interactive SVG Flamechart + Self/Total table |
| issues | SQLite via `/api/v1/issues` + ingest `POST /api/v1/errors` (linked to `trace_id` / `span_id`) |

Distributed traces render natively inside **Signals › Traces** (`#/traces`, `#/traces/<trace_id>`). When the Docker collector is running, **Jaeger ↗** (`http://127.0.0.1:16686/`) also appears as an external link.

Optional new-issue webhook: `--webhook URL` or `AIRBUG_ISSUES_WEBHOOK` (**`http://` only** — HTTPS is refused).
