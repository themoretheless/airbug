# Airbug roadmap

The one plan for the whole monorepo: a single ordered list across all domains. Work goes top
to bottom; a later stage starts when the one before it is done unless it is marked as
independent. Only open work is listed — when an item ships, delete it here (changelogs and the
feature maps at the bottom are the record of what was done).

Last reviewed: 2026-11-13.

## Stage 1 — Release (`run`, `bench`)

The hub is now the only web interface and `cargo airbug` the only launcher; what is left of
that work is shipping it.

1. **Release**: dispatch the Release workflow for `airbug-run` (0.1.0), publish it, then
   release `airbug-bench` — `cargo-airbug-bench` depends on `airbug-run`. Everything is in
   place (the workflow knows the `airbug-run` family, packaging is dry-run on every PR); it
   needs the owner's hand and crates.io token.

## Stage 2 — Hardware signals (`bench`)

Goal: the numbers behind a run come from the machine. `mon` already samples without a window
on Linux and macOS, with GPU load and memory (unified memory included). 

2. **Hardware counters, the rest**: `airbug_bench::perf` counts instructions, cycles, cache
   and branch events on Linux. Still owed: acceptance on a machine with a PMU (neither the
   development VM nor CI exposes one), macOS ARM, SIMD utilisation.
3. **Frequency and thermal data beyond sysfs**: the runner already records CPU frequency,
   temperature and throttle counts between processes on Linux (`host.json`) and notes
   throttling; `diagnose` correlates it with
   per-process timings. Still owed: model-specific registers and macOS.

## Stage 3 — Measurement depth (`bench`)

Goal: the scenarios Forma needs and evidence for the method itself.

4. **Forma on real devices**: `perf_bench` and `window_bench` hand their raw series to
   the process runner (themoretheless/forma#22, checked with Forma's CPU backend). Still owed:
   a run on a machine with a GPU and a display, and feeding GPU timestamp ticks through
   `airbug_bench::frames` instead of Forma's already resolved durations.

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
