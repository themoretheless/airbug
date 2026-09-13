# Airbug monorepo

Rust toolkit split by job. One naming rule:

**Folders = domains. Packages = `airbug` or `airbug-<role>`.**

| Domain folder | Packages |
|---------------|----------|
| `unit/` | `airbug`, `airbug-macros`, `airbug-containers`, `checkout-domain` |
| `bench/` | `airbug-bench`, `airbug-bench-macros`, `cargo-airbug-bench` |
| `mon/` | `airbug-mon` |
| `otel/` | `airbug-otel` |
| `err/` | `airbug-err` |
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
otel/                      # package: airbug-otel (OTLP traces+metrics+logs)
err/                       # package: airbug-err (panic/error → hub issues)
dash/hub/                  # package: airbug-hub (+ collector / logs / issues)
```

## Quick start

```bash
cargo test -p airbug --all-features
cargo test -p airbug-containers
cargo test -p airbug-bench
cargo airbug-bench --help
cargo run -p airbug-mon --release
cargo test -p airbug-otel
cargo test -p airbug-err
cargo run -p airbug-hub -- serve --root .
cargo run -p airbug-hub -- serve --root . --collector   # Docker or otelcol on PATH
```

## License

- `airbug*` unit/mon/otel/dash: MIT
- `airbug-bench*`: MIT OR Apache-2.0
