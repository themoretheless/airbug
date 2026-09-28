# Changelog

## 0.8.0

- `cargo airbug-bench matrix` accepts `--ui`, `--no-ui` and `--no-open`: one live page for the whole session, whose counter sums processes over every combination and whose chart lanes are labelled with the cell's axis values (`--cpu 2 · candidate`). Aggregation reads the cells' own `progress.json` / `status-final.json` server-side, so the runner and its artifacts are unchanged.
- `cargo airbug-bench run --hub <base-url>` (or `AIRBUG_HUB`) registers the run with a local airbug-hub and measures into the hub-issued directory, so the dashboard URL is printed before the first sample and `--output` yields to it with a warning. Both halves of the request are bounded — a hub that refuses, drops the SYN or never answers costs seconds rather than the platform's TCP timeout.
- The CLI runs on a 16 MiB worker thread. Windows reserves 1 MiB for a main thread and a debug build overflowed it inside argument parsing, which killed every `cargo test` on that platform.
- Release tag `airbug-bench-v0.8.0`

## 0.7.0

- `bench::viz`: offline, dependency-free SVG charts (`Plot` builder, forest, strip, diverging bars, timeline lanes, sparkline, dot plot) shared by reports and the live UI
- `report::comparison_charts` renders effect intervals, per-metric distributions and per-process deltas; `analysis::pairs` and `analysis::summarize` expose comparison math per independent unit
- Live UI redesign with `api/live-charts`: diagrams are rendered server-side from the UI's own poll history, so the measurement path stays untouched
- Public serde contracts, `report::plot` signature and report caps unchanged; release tag `airbug-bench-v0.7.0`

## 0.6.0

- Align bench family semver with unit at `0.6.0`
- Release tag `airbug-bench-v0.6.0` (independent of `airbug-v*`)

## 0.1.3

- Restore independent package-family versioning and tag `airbug-bench-v0.1.3`
- Drop monorepo-wide `v*` tag scheme

## 0.1.2

- Shared GitHub Actions release workflows (`themoretheless/.github`)
- Install path: pin via `branch = "release"` or tag `airbug-bench-v*`

## 0.1.1

- Typed `BenchError` public error API
- CLI split into `args` / `cmd` modules
- Align `airbug-bench-macros` on syn 2.x
- Docs: ARCHITECTURE reflects in-crate module layout

## 0.1.0

- Initial release of airbug-bench library, macros, and cargo subcommand
