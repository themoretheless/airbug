# Airbug monorepo

Rust toolkit split by job, not by historical repo name.

## Domains

| Domain | Role | Contents |
|--------|------|----------|
| `unit/` | Unit/integration test helpers | `airbug`, `airbug-macros`, `checkout-domain`, `containers` |
| `bench/` | Microbenchmarks & scenario runner | `rbench`, `rbench-macros`, `cargo-rbench` |
| `otel/` | Runtime observability (metrics → OTLP later) | `monik` |
| `trace/` | Spans / call traces (reserved) | placeholder |
| `dash/` | Cross-domain hub dashboard | `airbug-hub` |

**Why `otel` not “monitor”:** monik already collects host metrics and alerts; this domain is the home for future OpenTelemetry export. It is not an OTLP SDK yet.

**Why `trace` is separate from `bench`:** benches measure throughput/latency; traces explain causal paths. Keep them apart (SRP).

**Why `dash` is separate:** the hub only aggregates artifacts and links; it must not become a fourth product runtime.

## Layout

```text
unit/
  airbug/
  airbug-macros/
  checkout-domain/
  containers/          # Testcontainers-style Docker helpers
bench/
  rbench/
  rbench-macros/
  cargo-rbench/
  docs/
  scripts/
otel/
  monik/
trace/
  README.md
dash/
  hub/                 # airbug-hub binary
integrations/forma/    # excluded nested example
research/              # local notes (partially gitignored)
```

## Quick start

```bash
cargo test -p airbug --all-features
cargo test -p airbug-containers
cargo test -p rbench
cargo run -p cargo-rbench -- --help
cargo run -p monik --release
cargo run -p airbug-hub -- serve --root .
# open http://127.0.0.1:8790/
```

## License

- `unit/*` + `otel/monik` + `dash/hub`: MIT
- `bench/*`: MIT OR Apache-2.0
