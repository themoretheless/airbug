# Changelog

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
