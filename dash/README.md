# dash

Cross-cutting **hub** for the monorepo. Aggregates status from `unit`, `bench`, `otel`, and `trace` without owning those products (SRP).

## Run

```bash
cargo run -p airbug-hub -- serve --root .
# http://127.0.0.1:8790/

cargo run -p airbug-hub -- status --root .
```

## What it reads

| Domain | Sources |
|--------|---------|
| unit | `target/airbug-report/{report.json,index.html}` |
| bench | `.rbench/**/run.json` |
| otel | `~/.local/share/monik/monik.db` |
| trace | placeholder only |

It copies suggested commands into the clipboard; it does not start monik or rbench for you.
