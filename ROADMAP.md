# Airbug roadmap

The one plan for the whole monorepo. It lists only what is still open; finished work lives in
the changelogs and in the feature maps linked at the bottom. When an item ships, delete it here.

Last reviewed: 2026-11-12.

## 1. Run-centric rework (hub, CLI, mon)

Done so far: one run store (`airbug-run`, `.airbug/runs/<id>/`), one entry point
(`cargo airbug`), the hub regrouped as Now / Runs / Signals / Health. Remaining, in order:

1. **One run page in the hub** — tabs Results / Output / Signals / Compare for test and bench
   runs alike: signals scoped to the run, a baseline picker for Compare, a grid for matrix runs.
2. **Retire the bench web UI** — the live bench page moves into the hub; the per-tool report
   pages and the separate `cargo airbug-bench serve` server go away.
3. **`mon` as a headless sampler** — sampling without the egui window as the main mode, Linux
   support, and correct GPU memory on unified-memory machines (today `mem_used` is always 0
   and `mem_total` is the "GPU HW allocated memory" figure, `mon/src/gpu.rs`).

Follow-ups from the steps already merged:

- Announce `matrix`, `bench`, `git-compare` and `sessions` runs in the shared store; today only
  `cargo airbug-bench run` does, the rest is found by scanning `.airbug-bench`.
- Give bench sessions written outside `.airbug-bench` (`--output`) a report link in the hub.
- Move the test runner out of `airbug-hub` into the CLI.
- Publish `airbug-run` before the next `cargo-airbug-bench` release, which depends on it.

## 2. bench

### Criterion / Divan parity

Tracked in [bench/docs/parity/](bench/docs/parity/README.md) (`requirements.json`,
`completion.json`, `core.json`): 46 requirements verified, 6 partial. Parity is not reached.

Partial requirements:

- `measurement` — pluggable measurement values, aggregation and formatters.
- `plots` — enable/disable plots, linear/log scales, grouped comparisons.
- `output` — console verbosity/colour, tree/table, machine and report formats.
- `artifacts` — output directory, raw data and final summary.
- `platforms` — supported platforms/toolchains and runtime failure behaviour.
- `audit` — every upstream export, CLI option and feature flag mapped and verified.

Open acceptance items:

- `api_semantics` — check the behaviour of every public API already mapped.
- `report_matrix` — finish the report matrix.
- `console_policy` — review CLI output and usability.
- `visual_acceptance` — look through real rendered HTML reports, including interaction.
- `platform_acceptance` — confirm the current code on the remaining platforms (Windows
  runtime, native x86 timing).
- `final_gate` — final acceptance of a frozen change set, requirement by requirement, plus a
  repeat audit of upstream sources for missed features.

### Measurement and drivers

From [bench/docs/VALIDATION.md](bench/docs/VALIDATION.md) and
[bench/docs/ARCHITECTURE.md](bench/docs/ARCHITECTURE.md):

- Hardware performance counters: cache misses, branch mispredictions, SIMD utilisation.
  Linux/x86_64 first, then macOS ARM.
- Native window/present lifecycle and a GPU timestamp adapter for Forma scenarios.
- Browser/WASM scenario driver through an external host.
- Experiments: instrumentation effect, overhead against handwritten/Criterion/Divan loops,
  statistical stability under systematic drift.
- Crash-durable atomic publication of a run directory.
- CPU frequency-scaling and thermal-throttling correlation (needs model-specific registers).

### Idea backlog

[bench/FEATURES.md](bench/FEATURES.md) is an unprioritised catalogue of ideas (items 1–400),
not a commitment. Example suites it singles out and that do not exist yet: lock-free queues
(Michael–Scott), allocator comparison, post-quantum cryptography, hardware fingerprinting.

## 3. unit

TOP50 and NEXT50 are complete. What remains are deliberate stubs; each fails loudly today:

- `UpdateMode::InlineSource` returns `SnapshotError::Unsupported` — no source rewriting.
- `CompletionBarrier::await_done` — completion waits stay application-owned.
- `#[mock]` on generics / associated types is a `compile_error` pointing at manual `Mock` +
  `ExpectationBuilder::map_args` / `mock::borrowed`.
- `Prop::shrink_from` / `Shrink` are minimal, not a full property-testing framework.
- Adoption pilot on a real service; [unit/PILOT.md](unit/PILOT.md) covers only the local
  checkout demo.

## 4. err

Deferred on purpose ([err/README.md](err/README.md), "Skipped"):

- Release Health session protocol — release is only a tag on events.
- PII scrubbing rules engine — `before_send` is the only hook.
- Actix / Tower / Tracing integrations.

## 5. No open plans

`otel`, `run`, `cli`, `unit/containers`.

## Where finished work is recorded

- Changelogs: `bench/`, `unit/`, `unit/containers/`, `dash/hub/`, `err/`, `otel/`.
- unit feature maps: [TOP50](unit/TOP50.md), [NEXT50](unit/NEXT50.md).
- bench feature maps: [NEXT20](bench/docs/NEXT20.md), [FINAL20](bench/docs/FINAL20.md),
  [scenario coverage](bench/docs/FEATURES.md).
