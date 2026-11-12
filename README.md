# Airbug monorepo

Rust toolkit split by job. One naming rule:

**Folders = domains. Packages = `airbug` or `airbug-<role>`.**

| Domain folder | Packages |
|---------------|----------|
| `unit/` | `airbug`, `airbug-macros`, `airbug-containers`, `airbug-checkout-domain` |
| `bench/` | `airbug-bench`, `airbug-bench-macros`, `cargo-airbug-bench` |
| `mon/` | `airbug-mon` |
| `otel/` | `airbug-otel` |
| `err/` | `airbug-err` |
| `dash/` | `airbug-hub` (+ `dash/collector` ops assets) |
| `cli/` | `cargo-airbug` (`cargo airbug test \| bench \| serve \| up \| status`) |
| `run/` | `airbug-run` (run format + store shared by runners and the hub), `airbug-testrun` (the test runner behind `cargo airbug test`) |

```text
unit/                      # package: airbug (Allure/ReportPortal-style TestOps, #[airbug::test], async steps)
  macros/                  # airbug-macros (#[airbug::test(...)], #[test_case], #[test_matrix])
  containers/              # airbug-containers
  checkout-domain/         # airbug-checkout-domain
bench/                     # package: airbug-bench
  macros/                  # airbug-bench-macros
  cli/                     # cargo-airbug-bench → cargo airbug-bench
  research/                # screening notes / corpus
  integrations/forma/      # excluded from workspace
mon/                       # package: airbug-mon (host monitor)
otel/                      # package: airbug-otel (OTLP traces+metrics+logs+profiles, W3C propagation, tracing bridge)
err/                       # package: airbug-err (panic/error, breadcrumbs, Release Health sessions → hub + Sentry DSN)
dash/hub/                  # package: airbug-hub (TestOps + .NET Aspire Resources + Sentry Issues + OTLP collector & UI)
run/                       # package: airbug-run (one on-disk contract for runs + TestOps metadata & defect models)
```

## Architecture (local toolkit)

```text
airbug-mon / apps ──► airbug-otel ──► OTLP (:4318 HTTP / :4317 gRPC) ──► dash/collector or built-in hub receiver ──► files ──► airbug-hub
airbug-err / Sentry SDKs ──► POST /api/v1/errors | /api/1/envelope/ | /api/v1/sessions ──► airbug-hub (SQLite issues + source context + release health)
cargo airbug up ──► .airbug/stack.json + Docker discovery ──► airbug-hub (#/resources Aspire orchestrator + live stdout/stderr)
cargo airbug test (--junit) ──► .airbug/runs ──► airbug-hub (#/tests TestOps behaviors, defect classifier, JUnit XML)
bench artifacts ──► airbug-hub runs timeline
```

Hub binds **`127.0.0.1` only**. Issue webhooks accept **`http://` only**. This is a developer monorepo tool, not a hosted SaaS. Details: [dash/README.md](dash/README.md).

## Maturity

| Area | Status | Notes |
|------|--------|-------|
| `unit` / macros | release / publishable | `#[airbug::test]` taxonomy, sync + `step_async` steps, OTel trace correlation, defect classification, JUnit XML |
| `bench` / cli / macros | release / publishable | Typed `BenchError`; docs synced to in-crate modules |
| `otel` | full OTel stack | OTLP HTTP (`protobuf`/`json`, `gzip`) + gRPC, traces/metrics/logs/profiles, W3C propagation, `TracingOtelLayer`, `InMemoryTelemetry` |
| `err` | full Sentry stack | `EventTransport` + `in_app` frames + structured breadcrumbs + Release Health sessions + OTel `trace_id`/`span_id` correlation |
| `hub` | full local dev platform | TestOps (`Behaviors`/`Defects`/`Slowest`/`junit.xml`), `.NET Aspire` Resources (`#/resources`), Sentry Issue Inspector (`±5` source context, `git blame` suspect commit, auto-regressions, `/api/1/envelope/`), and built-in OTLP collector |
| `mon` | usable (macOS) | `MonApp`; CI on macOS |
| `containers` / checkout-domain | internal | `publish = false` |

See [ARCHITECTURE.md](ARCHITECTURE.md), [SECURITY.md](SECURITY.md) and [ROADMAP.md](ROADMAP.md) (what is still open).

Known debt (not blocking): none on `syn` majors (unit + bench macros both on syn 2.x).

## Quick start

```bash
cargo test --workspace --locked --exclude airbug-mon   # cross-platform
cargo test -p airbug-mon                               # macOS host monitor
cargo test -p airbug --all-features
cargo test -p airbug-otel --all-features
cargo test -p airbug-err --all-features
cargo test -p airbug-hub
cargo airbug serve                                       # hub on http://127.0.0.1:8790/
cargo airbug up                                          # start hub + collector + .airbug/stack.json resources (#/resources)
cargo airbug bench run --program ./target/release/mybench --output .airbug-bench/session
cargo airbug test --junit junit.xml -- --workspace --exclude airbug-mon  # live TestOps run → hub #/tests + JUnit XML
cargo run -p airbug-hub -- serve --root . --collector   # Docker, otelcol on PATH, or zero-dependency built-in OTLP collector
cargo run -p airbug-otel --example full_stack           # emit traces + metrics + logs + profiles + correlated error
./dash/scripts/smoke_local.sh                           # optional local smoke
```

## License

- Root + `airbug*` unit/mon/otel/err/dash: MIT (see [LICENSE](LICENSE))
- `airbug-bench*`: MIT OR Apache-2.0
