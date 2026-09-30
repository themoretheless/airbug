# Changelog

## Unreleased

- Automatically compare completed direct Cargo runs per case, with immutable hash-checked history, filtered-run continuity, explicit incompatibility and history opt-out.

- Add compensation attributes with nested/imported inheritance and CLI precedence; correct run-level overhead-policy metadata and verify CPU clocks on ARM64 and x86_64 through Rosetta.

- Add optional Suite and CLI overhead compensation as `wall.adjusted`, retaining raw intervals and validating per-worker correction before wave aggregation.

- Add isolated calibration of allocation bookkeeping using the production event helpers, with separate grow/shrink estimates and checked fractional cost aggregation.

- Probe coarse timer resolution with bounded delayed retries; retain zero empty-read costs and report unresolved clocks explicitly instead of fabricating a resolution.

- Record cached per-clock empty-interval and loop calibration diagnostics in run provenance, preserving fractional loop costs and raw observations; omit diagnostics for cold, smoke and caller-only runs.

- Add lazy `timer` attributes with nested/imported group inheritance, per-case builder settings and CLI precedence; validate every selected clock before executing workloads.

- Add direct Cargo `--timer os|cpu`, lazy clock previews and explicit unsupported-CPU failures before execution.

- Add a low-level OS/CPU timer API with cached CPU frequency, diagnostic calibration and checked timestamp conversion; add `Suite::timer` selection across local, async, allocated and worker measurements, with worker cleanup on clock errors.

- Add allocation-counted borrowed/owned input batches with explicit drop and batch policies; exclude setup and harness buffers while aggregating measured counters.

- Associate allocation metrics with duration-ranked samples in text/JSON summaries, preserving peak units and detecting missing/mismatched sample pairs.

- Support allocator references in benchmark attributes and inherited group defaults for plain sync/async cases.

- Add async allocation-counted cases with lazy executor setup excluded and future creation/polling/output destruction included on the polling thread.

- Record twelve allocator metrics alongside each synchronous bench_allocated wall-time sample, excluding pilot/warmup records and report construction.

- Add allocator-specific thread-local phases with grow/shrink counters, signed byte balances, peak net growth and unwind cleanup.

- Split allocation accounting into allocation/free bytes and successful grow/shrink operations and byte deltas, including phase-local differences.

- Associate work counters with fastest, slowest and median-time samples; report per-operation counts and their weighted mean in descriptive JSON/text summaries.

- Report operation-weighted means separately from sample means for unequal batches, avoiding overflow from summing large raw totals.

- Emit per-process descriptive statistics in direct Cargo text/JSON output and summary.json without requiring bootstrap resampling.

- Add descriptive summaries without bootstrap resampling, with explicit single-observation deviation semantics and finite-range validation.

- Reject duplicate benchmark identities during dry-run before printing a plan; expose lazy registration validation with the conflicting ID.

- Preserve generic benchmark families as parent nodes in kind ordering, distinct from runtime argument cases.

- Add kind ordering with cases before nested groups, preserving case labels containing slashes.

- Allow explicit output batch policies without setup, including async functions borrowing arguments; verify destruction at batch boundaries.

- Add timed in-process profiling, panic-safe start/stop hooks, per-case artifacts and Cargo CLI validation.

- Accept automatic, scalar, range and slice worker selections; deduplicate effective counts and allow a local override of inherited parallelism.

- Add opt-in Tokio, Futures and Smol executor adapters, borrowed executor support, and tests of timer/task/I/O behavior without Send bounds.

- Support explicit input lifetimes and lifetime-dependent type bounds; instantiate lifetime-aware cases with concrete types to retain short borrows of captured inputs and higher-ranked function bounds.

- Compose async benchmarks with concurrent workers, local executors, borrowed/owned non-Send inputs and dynamic counters; preserve collective completion and panic-safe joins.

- Record dynamic per-input bytes/items/chars/cycles outside timing, including async and worker-local setup; persist actual batch totals and derive throughput without mixing in calibration/warmup.

- Add worker-local setup for threaded borrowed/owned inputs, supporting non-Send values and outputs with collective completion before destruction and panic-safe rendezvous.

- Accept external const arrays/slices with 1–20 entries, including tuple rows; validate size and row types at compile time without generating extra cases.

- Support multiple const parameters through tuple rows, including mixed usize/bool/char values and const expressions, crossed with type rows and runtime arguments.

- Support multiple generic type parameters through tuple rows in `types`, crossed with const and runtime arguments; reject mismatched tuple arity.

- Support explicit batch policies for async borrowed/owned inputs, preserving lazy executors and deferred destruction between batches.

- Add explicit synchronous input batch policies: per-iteration, fixed size, target batch count, small/large input modes, with attribute and builder APIs.

- Add async custom timing with lazy executors, caller-reported batch durations and borrowed benchmark arguments.

- Resolve renamed Cargo dependencies in generated benchmark code and support explicit `crate = path` overrides for suites, groups and standalone attributes.

- Add decimal/binary byte-rate formatting with automatic per-series prefixes, direct Cargo flags and `throughput.json`, preserving raw measurements and non-byte counters.

- Retain zero work counters in measurements and throughput reports; reject malformed counter metadata instead of silently omitting it.
- Add helpers for type/value/slice/iterator byte sizes, Unicode scalar counts and iterator item counts, with checked multiplication.

- Classify mild/severe outliers with Tukey fences, preserve every observation in estimates, and save an offline `estimates.html` report with colored SVG plots.

- Fit through-origin slopes for variable-size batches, with paired-bootstrap intervals, centered R² and per-process estimates in text and JSON analysis reports.

- Add seeded percentile bootstrap estimates for mean, median, standard deviation and scaled MAD, with configurable resample count/confidence, process-level aggregation and explicit within-process limitations.
- Expose bootstrap through direct Cargo CLI, text/JSON output, `estimates.json` artifacts and reusable offline analysis APIs.

- Make `#[group]` independently reusable and add `groups = [module::path]` to suites/groups for registration across source files and library crate boundaries.

- Sort numeric argument labels exactly, including signed decimal/exponent values and large integers; add source-location ordering with automatic attribute metadata.

- Add registration, lexical, natural and reverse ordering shared by listing, dry-run and execution without mutating registration order.

- Export the standard `black_box` barrier and add `black_box_drop` for explicit timed result consumption.
- Verify workload time limits across calibration, warmup and collection, including caller-excluded external work.

- Add compiled regex selection and exact/regex exclusions to `Selection` and direct Cargo CLI, including early invalid-pattern errors.

- Add fixed iterations and flat/linear/auto sampling through attributes, group defaults and direct Cargo CLI; record effective schedules and normalize variable sample sizes.
- Add cooperative minimum/maximum workload-time budgets with measured-only or setup/drop-inclusive accounting.
- Let concurrent fresh-input samples span bounded waves instead of imposing a 64-operation sample limit.

- Add `--test` / `Suite::test_selected` for one-operation smoke execution, including registered correctness checks. Results carry a separate execution-mode contract.

- Register nested inline groups with inherited sampling/counter/thread defaults; support ignored cases and include/only-ignored selection.
- Accept fresh inputs by ownership for sync, async and concurrent cases, preserving explicit setup and output-drop boundaries.
- Report simultaneous bytes/items/chars/cycles counters. Throughput budgets can select `--unit` and reject mixed-unit or empty measurements.
- Start a versioned Criterion/Divan feature audit with upstream archive hashes and an explicit incomplete requirements ledger.

- Extend benchmark attributes with type/const specializations, borrowed arguments, async functions with lazy executors, concurrent worker cases, bytes/items throughput, per-case sampling and caller-timed batches. Output drop can be excluded without setup.
- Concurrent batches wait for every worker before timing, normalize by total operations and retain prepared inputs/deferred outputs until all workers finish; worker errors propagate after joining.

- Support `#[bench(args = [...], setup = ...)]` in both suite and standalone registration: named parameter cases, fresh input outside timing, optional `name` and `drop_output` controls. Listing never invokes setup or workloads.

- Record direct `Suite::main()` launches in the hub test/benchmark history with live case status and median batch ns/op. `AIRBUG_DASHBOARD=0` disables recording; writes occur outside measured batches.

## 0.9.0

- `#[airbug_bench::suite]` generates a benchmark executable and registers its directly contained `#[bench]` functions; enable `macros` and set `harness = false` to run it with stable `cargo bench`.
- `Suite::main()` accepts Cargo's `--bench` flag and positional name filters, including `--exact` and `--list`; duplicate filters produce an explicit error.
- Add an attributed benchmark example and an end-to-end Cargo invocation regression script.
- Release tag `airbug-bench-v0.9.0`

- `bench::viz`: figures build themselves in — marks go out in staggered groups driven by `viz::REVEAL_CSS`, with no JavaScript, no SMIL and no second implementation of the scales in the browser. Print and `prefers-reduced-motion` render the finished frame, and a page without the CSS renders the same pixels
- `bench::viz::charts::heatmap` and `Palette::heat`: matrix diagrams with global, per-row and signed-around-zero ramps; missing values stay blank instead of being colored as zero, every cell keeps a full-label tooltip, and `max_cells` bounds the markup
- `viz::charts::ecdf` draws the cumulative share per lane on a shared value scale; `comparison_charts` adds it next to the strip for a crossover run with at least five independent units (a cumulative line over three units is the same dots with a second axis)
- Lines and bars draw themselves along their own length instead of fading in: marks carry `pathLength="100"` and a `dv` class, so `REVEAL_CSS` needs 28 bytes per drawn mark and no measured length, against 22 bytes per reveal group and 689 bytes of CSS per page
- `report::process_heat` renders the process matrix on the run and comparison pages; `comparison_charts` adds a case × metric change heat whenever effect intervals are available
- `Plot::svg` and `Plot::inline` stay static, so the live UI keeps repainting without restarting animations
- `cargo run -p airbug-bench --example viz_gallery` writes a playground page with every diagram rendered from synthetic data plus its measured markup size, and CSS-only motion controls; the renderer gained no playground-specific options

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

- Add async allocation-counted borrowed/owned input batches with lazy executor initialization and explicit output drop/batch policies.

- Allow allocator attributes with fresh inputs, input counters, batch policies and deferred output destruction for local sync/async cases.

- Add allocation collection for synchronous worker-local borrowed inputs, with per-wave aggregation and an explicit upper-bound worker peak metric.

- Add owned worker allocation accounting and sync threaded allocator attributes, including worker-local non-Send inputs/outputs and dynamic input counters.

- Support coordinator-prepared borrowed/owned inputs with threaded allocation accounting; preserve default and explicit coordinator setup in attributes.

- Add async worker allocation accounting for plain/borrowed/owned attribute cases; executor setup/drop excluded, non-Send futures supported, and panics release workers.

- Track peak live allocation block count in thread phases, batch samples and worker aggregates; expose it alongside peak bytes in reports.

- Classify successful same-size realloc as zero-byte growth consistently across thread and process allocation accounting.

- Add explicitly labelled per-operation allocation peak summaries in JSON/text alongside absolute count and byte peaks.

- Preserve individual worker-wave allocation and timing records in run JSON for sync/async allocated suites, with backward-compatible loading and runner identity remapping.

- Validate worker-wave completeness and exact consistency with aggregate timing, operations and allocation metrics; verify protocol process/variant remapping and metric filtering.

- Add timing-ranked worker allocation summaries to JSON/text, with selected worker identities and absolute/per-operation peak views.

- Support lazy async executor defaults in inline suites/groups, nested inheritance, and case/group overrides without affecting synchronous members.

- Propagate per-field sampling defaults into imported groups across module/crate boundaries, preserving child settings and CLI precedence.

- Inherit ignore and fixed counters across imported groups, preserving explicit false/zero and dynamic input counter precedence.

- Add builder-configured adaptive quick sampling with adjacent-batch residual stopping, cooperative deadlines, iteration caps and preservation of real observations.

- Expose adaptive quick sampling through cargo bench flags with configurable time/deviation limits, lazy previews and explicit option conflicts.

- Add per-case quick attributes, inline/imported group inheritance, explicit disabling and CLI precedence; validate selected-case conflicts before workload execution.

- Honor sampling minimum/maximum budgets during adaptive quick runs, clear stale quick metadata on subsequent ordinary runs, and verify repeated worker allocation accounting.
