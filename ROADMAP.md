# Airbug roadmap

The one plan for the whole monorepo: a single ordered list across all domains. Work goes top
to bottom; a later stage starts when the one before it is done unless it is marked as
independent. Only open work is listed — when an item ships, delete it here (changelogs and the
feature maps at the bottom are the record of what was done).

Last reviewed: 2026-11-12.

## Stage 1 — One UI, one runner (`bench`, `dash`, `cli`)

Goal: the hub is the only web interface and `cargo airbug` the only launcher. Every run
already opens on one hub page (`#/run/<id>`: Results / Output / Signals / Compare).

1. **Live bench page inside the hub**, fed by the same `progress.json`.
2. **Retire the bench web UI**: the `--ui` server, `cargo airbug-bench serve` and the per-tool
   report pages go away; static `report.html` stays as the offline/CI artifact.
3. **Move the test runner out of `airbug-hub` into the CLI**; the hub only reads runs.
4. **Release**: publish `airbug-run`, then `cargo-airbug-bench`, which depends on it.

## Stage 2 — Close Criterion / Divan parity (`bench`)

Goal: parity is declared, backed by the registry in
[bench/docs/parity/](bench/docs/parity/README.md) (46 requirements verified, 6 partial).
Placed after stage 1 on purpose: reports and console output are reviewed once, in their final
form.

5. **Finish the partial requirements**: `measurement` (pluggable values, aggregation,
   formatters), `plots` (on/off, linear/log scales, grouped comparisons), `output` (verbosity,
   colour, tree/table, machine formats), `artifacts` (output directory, raw data, summary).
6. **`api_semantics`** — check the behaviour of every public API already mapped.
7. **`report_matrix`, `console_policy`, `visual_acceptance`** — finish the report matrix,
    review CLI output, look through real rendered reports including interaction.
8. **`platforms` / `platform_acceptance`** — confirm the current code on Windows runtime and
    native x86 timing.
9. **`audit` / `final_gate`** — repeat the upstream source audit for missed features, then
    accept a frozen change set requirement by requirement.

## Stage 3 — Host and hardware signals (`mon`, `bench`)

Goal: the numbers behind a run come from the machine, on Linux as well as macOS. Independent
of stage 2; can run in parallel with it.

10. **`mon` as a headless sampler**: sampling without the egui window is the main mode, the
    window becomes optional.
11. **`mon` on Linux.**
12. **Correct GPU memory on unified-memory machines** — today `mem_used` is always 0 and
    `mem_total` is the "GPU HW allocated memory" figure (`mon/src/gpu.rs`).
13. **Hardware performance counters in bench**: cache misses, branch mispredictions, SIMD
    utilisation. Linux/x86_64 first, then macOS ARM.
14. **Frequency-scaling and thermal-throttling correlation** for bench runs (needs
    model-specific registers; attempt after the counters).

## Stage 4 — Measurement depth (`bench`)

Goal: the scenarios Forma needs and evidence for the method itself.

15. **Native window/present lifecycle and a GPU timestamp adapter** for Forma scenarios.
16. **Browser/WASM scenario driver** through an external host.
17. **Method experiments**: instrumentation effect, overhead against
    handwritten/Criterion/Divan loops, stability under systematic drift.
18. **Crash-durable atomic publication** of a run directory.

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
