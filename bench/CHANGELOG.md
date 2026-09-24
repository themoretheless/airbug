# Changelog

## Unreleased

- `bench::viz`: figures build themselves in — marks go out in staggered groups driven by `viz::REVEAL_CSS`, with no JavaScript, no SMIL and no second implementation of the scales in the browser. Print and `prefers-reduced-motion` render the finished frame, and a page without the CSS renders the same pixels
- `bench::viz::charts::heatmap` and `Palette::heat`: matrix diagrams with global, per-row and signed-around-zero ramps; missing values stay blank instead of being colored as zero, every cell keeps a full-label tooltip, and `max_cells` bounds the markup
- `viz::charts::ecdf` draws the cumulative share per lane on a shared value scale; `comparison_charts` adds it next to the strip for a crossover run with at least five independent units (a cumulative line over three units is the same dots with a second axis)
- Lines and bars draw themselves along their own length instead of fading in: marks carry `pathLength="100"` and a `dv` class, so `REVEAL_CSS` needs 28 bytes per drawn mark and no measured length, against 22 bytes per reveal group and 689 bytes of CSS per page
- `report::process_heat` renders the process matrix on the run and comparison pages; `comparison_charts` adds a case × metric change heat whenever effect intervals are available
- `Plot::svg` and `Plot::inline` stay static, so the live UI keeps repainting without restarting animations
- `cargo run -p airbug-bench --example viz_gallery` writes a playground page with every diagram rendered from synthetic data plus its measured markup size, and CSS-only motion controls; the renderer gained no playground-specific options

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
