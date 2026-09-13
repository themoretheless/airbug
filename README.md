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
otel/                      # package: airbug-otel (OTLP traces+metrics+logs)
err/                       # package: airbug-err (panic/error → hub issues)
dash/hub/                  # package: airbug-hub (+ collector / logs / issues)
```

## Architecture (local toolkit)

```text
airbug-mon ──► airbug-otel ──► OTLP ──► dash/collector ──► files
airbug-err ──► POST /api/v1/errors ──► airbug-hub (SQLite issues)
unit / bench artifacts ──► airbug-hub status cards
```

Hub binds **`127.0.0.1` only**. Issue webhooks accept **`http://` only**. This is a developer monorepo tool, not a hosted SaaS. Details: [dash/README.md](dash/README.md).

## Maturity

| Area | Status | Notes |
|------|--------|-------|
| `unit` / macros | release / publishable | Strong tests + CI matrix; crates.io metadata |
| `bench` / cli / macros | release / publishable | Typed `BenchError`; docs synced to in-crate modules |
| `otel` | usable | OTLP façade + `TelemetryHandle` |
| `err` | usable | `EventTransport` + `schema_version` → hub |
| `hub` | usable | `HubApp` / `IssueStore`, `/api/v1`, concurrency |
| `mon` | usable (macOS) | `MonApp`; CI on macOS |
| `containers` / checkout-domain | internal | `publish = false` |

See [ARCHITECTURE.md](ARCHITECTURE.md) and [SECURITY.md](SECURITY.md).

Known debt (not blocking): none on `syn` majors (unit + bench macros both on syn 2.x).

## Quick start

```bash
cargo test --workspace --locked --exclude airbug-mon   # cross-platform
cargo test -p airbug-mon                               # macOS host monitor
cargo test -p airbug --all-features
cargo test -p airbug-otel
cargo test -p airbug-err
cargo test -p airbug-hub
cargo run -p airbug-hub -- serve --root .
cargo run -p airbug-hub -- serve --root . --collector   # Docker or otelcol on PATH
./dash/scripts/smoke_local.sh                           # optional local smoke
```

## License

- Root + `airbug*` unit/mon/otel/err/dash: MIT (see [LICENSE](LICENSE))
- `airbug-bench*`: MIT OR Apache-2.0
