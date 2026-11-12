# Airbug roadmap

The one plan for the whole monorepo: a single ordered list across all domains. Work goes top
to bottom; a later stage starts when the one before it is done unless it is marked as
independent. Only open work is listed — when an item ships, delete it here (changelogs and the
feature maps at the bottom are the record of what was done).

Last reviewed: 2026-11-12.

## Stage 1 — Every run in one place (`dash`, `bench`)

Goal: any test or bench run is opened, read and compared on one page of the hub.

1. **Announce every bench run in the shared store.** `matrix`, `bench`, `git-compare` and
   `sessions` runs write a manifest to `.airbug/runs/<id>/` the way `cargo airbug-bench run`
   already does; the hub stops scanning `.airbug-bench`.
2. **Report link for sessions outside `.airbug-bench`** (`--output` elsewhere) — today they
   are listed in the hub without one.
3. **One run page**: tabs Results / Output / Signals / Compare for test and bench runs alike.
   Signals are scoped to the run, Compare has a baseline picker, matrix runs get a grid.

## Stage 2 — One UI, one runner (`bench`, `dash`, `cli`)

Goal: the hub is the only web interface and `cargo airbug` the only launcher.

4. **Live bench page inside the hub**, fed by the same `progress.json`.
5. **Retire the bench web UI**: the `--ui` server, `cargo airbug-bench serve` and the per-tool
   report pages go away; static `report.html` stays as the offline/CI artifact.
6. **Move the test runner out of `airbug-hub` into the CLI**; the hub only reads runs.
7. **Release**: publish `airbug-run`, then `cargo-airbug-bench`, which depends on it.

## Stage 3 — Close Criterion / Divan parity (`bench`)

Goal: parity is declared, backed by the registry in
[bench/docs/parity/](bench/docs/parity/README.md) (46 requirements verified, 6 partial).
Placed after stage 2 on purpose: reports and console output are reviewed once, in their final
form.

8. **Finish the partial requirements**: `measurement` (pluggable values, aggregation,
   formatters), `plots` (on/off, linear/log scales, grouped comparisons), `output` (verbosity,
   colour, tree/table, machine formats), `artifacts` (output directory, raw data, summary).
9. **`api_semantics`** — check the behaviour of every public API already mapped.
10. **`report_matrix`, `console_policy`, `visual_acceptance`** — finish the report matrix,
    review CLI output, look through real rendered reports including interaction.
11. **`platforms` / `platform_acceptance`** — confirm the current code on Windows runtime and
    native x86 timing.
12. **`audit` / `final_gate`** — repeat the upstream source audit for missed features, then
    accept a frozen change set requirement by requirement.

## Stage 4 — Host and hardware signals (`mon`, `bench`)

Goal: the numbers behind a run come from the machine, on Linux as well as macOS. Independent
of stage 3; can run in parallel with it.

13. **`mon` as a headless sampler**: sampling without the egui window is the main mode, the
    window becomes optional.
14. **`mon` on Linux.**
15. **Correct GPU memory on unified-memory machines** — today `mem_used` is always 0 and
    `mem_total` is the "GPU HW allocated memory" figure (`mon/src/gpu.rs`).
16. **Hardware performance counters in bench**: cache misses, branch mispredictions, SIMD
    utilisation. Linux/x86_64 first, then macOS ARM.
17. **Frequency-scaling and thermal-throttling correlation** for bench runs (needs
    model-specific registers; attempt after 16).

## Stage 5 — Measurement depth (`bench`)

Goal: the scenarios Forma needs and evidence for the method itself.

18. **Native window/present lifecycle and a GPU timestamp adapter** for Forma scenarios.
19. **Browser/WASM scenario driver** through an external host.
20. **Method experiments**: instrumentation effect, overhead against
    handwritten/Criterion/Divan loops, stability under systematic drift.
21. **Crash-durable atomic publication** of a run directory.

## Unscheduled

Not in the sequence; each is picked up only when a real need appears.

- **unit — deliberate stubs** (each fails loudly today): `UpdateMode::InlineSource` (no source
  rewriting), `CompletionBarrier::await_done` (waits stay application-owned), `#[mock]` on
  generics / associated types, full shrinking for `Prop`.
- **unit — adoption pilot on a real service**; [unit/PILOT.md](unit/PILOT.md) covers only the
  local checkout demo.
- **err** ([err/README.md](err/README.md), "Skipped"): Release Health session protocol, PII
  scrubbing rules engine, Actix / Tower / Tracing integrations.
- **bench — idea catalogue**: [bench/FEATURES.md](bench/FEATURES.md), items 1–400,
  unprioritised. Example suites it singles out: lock-free queues (Michael–Scott), allocator
  comparison, post-quantum cryptography, hardware fingerprinting.

No open work: `otel`, `run`, `unit/containers`.

## Where finished work is recorded

- Changelogs: `bench/`, `unit/`, `unit/containers/`, `dash/hub/`, `err/`, `otel/`.
- unit feature maps: [TOP50](unit/TOP50.md), [NEXT50](unit/NEXT50.md).
- bench feature maps: [NEXT20](bench/docs/NEXT20.md), [FINAL20](bench/docs/FINAL20.md),
  [scenario coverage](bench/docs/FEATURES.md).
