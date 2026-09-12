# Airbug monorepo

Rust toolkit split by job. One naming rule:

**Folders = domains. Packages = `airbug` or `airbug-<role>`.**

| Domain folder | Packages |
|---------------|----------|
| `unit/` | `airbug`, `airbug-macros`, `airbug-containers`, `checkout-domain` |
| `bench/` | `airbug-bench`, `airbug-bench-macros`, `cargo-airbug-bench` |
| `mon/` | `airbug-mon` |
| `trace/` | `airbug-trace` |
| `dash/` | `airbug-hub` |

```text
unit/                      # package: airbug
  macros/                  # airbug-macros
  containers/              # airbug-containers
  checkout-domain/
bench/                     # package: airbug-bench
  macros/                  # airbug-bench-macros
  cli/                     # cargo-airbug-bench → cargo airbug-bench
  research/                # screening notes / corpus
  integrations/forma/      # excluded from workspace
mon/                       # package: airbug-mon (host monitor)
trace/                     # package: airbug-trace (OTLP traces+metrics)
dash/hub/                  # package: airbug-hub
```

## Quick start

```bash
cargo test -p airbug --all-features
cargo test -p airbug-containers
cargo test -p airbug-bench
cargo airbug-bench --help
cargo run -p airbug-mon --release
cargo test -p airbug-trace
cargo run -p airbug-hub -- serve --root .
cargo run -p airbug-hub -- serve --root . --collector   # + OTEL collector / Jaeger
```

## License

- `airbug*` unit/mon/trace/dash: MIT
- `airbug-bench*`: MIT OR Apache-2.0
