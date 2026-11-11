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
| `cli/` | `cargo-airbug` (`cargo airbug test \| bench \| serve \| status`) |
| `run/` | `airbug-run` (run format + store shared by runners and the hub) |

```text
unit/                      # package: airbug
  macros/                  # airbug-macros
  containers/              # airbug-containers
  checkout-domain/         # airbug-checkout-domain
bench/                     # package: airbug-bench
  macros/                  # airbug-bench-macros
  cli/                     # cargo-airbug-bench → cargo airbug-bench
  research/                # screening notes / corpus
  integrations/forma/      # excluded from workspace
mon/                       # package: airbug-mon (host monitor)
otel/                      # package: airbug-otel (OTLP traces+metrics+logs+profiles, W3C propagation, tracing bridge)
err/                       # package: airbug-err (panic/error → hub issues + OTel trace correlation)
dash/hub/                  # package: airbug-hub (built-in OTLP HTTP/gRPC collector + traces/profiles/metrics/logs/issues UI)
run/                       # package: airbug-run (one on-disk contract for runs)
```

## Architecture (local toolkit)

```text
airbug-mon / apps ──► airbug-otel ──► OTLP (:4318 HTTP / :4317 gRPC) ──► dash/collector or built-in hub receiver ──► files ──► airbug-hub
airbug-err (+ OTel trace_id/span_id) ──► POST /api/v1/errors ──► airbug-hub (SQLite issues)
cargo airbug test ──► .airbug/runs ──► airbug-hub (live tests)
bench artifacts ──► airbug-hub runs timeline
```

Hub binds **`127.0.0.1` only**. Issue webhooks accept **`http://` only**. This is a developer monorepo tool, not a hosted SaaS. Details: [dash/README.md](dash/README.md).

## Maturity

| Area | Status | Notes |
|------|--------|-------|
| `unit` / macros | release / publishable | Strong tests + CI matrix; crates.io metadata |
| `bench` / cli / macros | release / publishable | Typed `BenchError`; docs synced to in-crate modules |
| `otel` | full OTel stack | OTLP HTTP (`protobuf`/`json`, `gzip`) + gRPC, traces/metrics/logs/profiles, W3C propagation, `TracingOtelLayer`, `InMemoryTelemetry` |
| `err` | usable | `EventTransport` + `schema_version` + automatic OTel `trace_id`/`span_id` correlation → hub |
| `hub` | usable | Built-in OTLP HTTP/gRPC receiver, native Traces (waterfall/topology/RED/zPages) + Profiles flamechart + Metrics quantiles/exemplars + Logs + Issues |
| `mon` | usable (macOS) | `MonApp`; CI on macOS |
| `containers` / checkout-domain | internal | `publish = false` |

See [ARCHITECTURE.md](ARCHITECTURE.md) and [SECURITY.md](SECURITY.md).

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
cargo airbug bench run --program ./target/release/mybench --output .airbug-bench/session
cargo airbug test -- --workspace --exclude airbug-mon    # live per-test run → hub #/tests
cargo run -p airbug-hub -- serve --root . --collector   # Docker, otelcol on PATH, or zero-dependency built-in OTLP collector
cargo run -p airbug-otel --example full_stack           # emit traces + metrics + logs + profiles + correlated error
./dash/scripts/smoke_local.sh                           # optional local smoke
```

## License

- Root + `airbug*` unit/mon/otel/err/dash: MIT (see [LICENSE](LICENSE))
- `airbug-bench*`: MIT OR Apache-2.0
