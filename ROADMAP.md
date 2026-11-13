# Airbug roadmap

The one plan for the whole monorepo: a single ordered list across all domains. Work goes top
to bottom; a later stage starts when the one before it is done unless it is marked as
independent. Only open work is listed — when an item ships, delete it here (changelogs and the
feature maps at the bottom are the record of what was done).

Last reviewed: 2026-11-13.

## Stage 1 — Release (`run`, `bench`)

The hub is now the only web interface and `cargo airbug` the only launcher; what is left of
that work is shipping it.

1. **Release**: publish `airbug-run`, then `cargo-airbug-bench`, which depends on it. Needs
   the owner: the Release workflow is dispatched by hand and has no `airbug-run` family yet;
   packaging itself is checked on every PR (`publish-dry-run.yml`).

## Stage 2 — Close Criterion / Divan parity (`bench`)

The acceptance gate is passed on 7d8236d with current evidence for every item
([bench/docs/parity/completion.json](bench/docs/parity/completion.json)); 48 requirements are
verified. Full parity is not claimed until the last four are:

2. **Finish the partial requirements** — each has its leftovers named in `requirements.json`:
   `plots` (navigation/thumbnail equivalents, resource limits, custom-formatter integration),
   `output` (remaining formatter and entry-point scope), `artifacts` (plot entry-point audit),
   `audit` (the semantic review items its mappings list).

## Stage 3 — Hardware signals (`bench`)

Goal: the numbers behind a run come from the machine. `mon` already samples without a window
on Linux and macOS, with GPU load and memory (unified memory included). Independent of
stage 2; can run in parallel with it.

3. **Hardware counters, the rest**: `airbug_bench::perf` counts instructions, cycles, cache
   and branch events on Linux. Still owed: acceptance on a machine with a PMU (neither the
   development VM nor CI exposes one), macOS ARM, SIMD utilisation.
4. **Frequency and thermal data beyond sysfs**: the runner already records CPU frequency,
   temperature and throttle counts between processes on Linux (`host.json`) and notes
   throttling; `diagnose` correlates it with
   per-process timings. Still owed: model-specific registers and macOS.

## Stage 4 — Measurement depth (`bench`)

Goal: the scenarios Forma needs and evidence for the method itself.

5. **Native window/present lifecycle and a GPU timestamp adapter** for Forma scenarios.

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
