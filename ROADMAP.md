# Airbug roadmap

The single plan for the whole monorepo: all in-repo engineering plans (`unit/ROADMAP.md`, `unit/TOP50.md`, `unit/NEXT50.md`, `bench/ROADMAP.md`, `bench/docs/ROADMAP.md`, `bench/docs/NEXT20.md`, `bench/docs/FINAL20.md`, `bench/docs/parity/*.json`, OpenTelemetry, TestOps, .NET Aspire, Sentry, physical Apple Silicon M4 Max counter acceptance, physical Metal GPU + native window Forma verification, and multi-OS GitHub Actions CI sign-off) have been completed and consolidated here. Only external credential-gated or bare-metal Linux PMU/RAPL hardware items remain open.

Last reviewed: 2026-11-20.

## Stage 1 — Release (`run`, `bench`, `unit`, `err`, `otel`)

1. **Manual Stage 1 crates.io publication (`cargo publish`)**: all publishable workspace crates (`airbug-macros`, `airbug`, `airbug-run`, `airbug-bench-macros`, `airbug-bench`, `cargo-airbug-bench`, `airbug-err`, `airbug-otel`) are at `0.11.0` and verified via `cargo publish --dry-run` / `cargo package --no-verify`; publishing to `crates.io` requires maintainer `CARGO_REGISTRY_TOKEN` credentials in dependency order.

## Stage 2 — Bare-Metal Linux PMU & RAPL Hardware Acceptance (`bench`)

2. **Bare-metal Linux PMU & MSR/RAPL acceptance**: `airbug_bench::perf` (`Instructions`, `Cycles`, `CacheReferences`, `CacheMisses`, `Branches`, `BranchMisses`, `ICacheMisses`, `DtlbMisses`, `ItlbMisses`, `PageFaults`) and `host.rs` sysfs RAPL (`energy_uj`) / MSR probing require validation on a physical Linux host with exposed PMU counters and `/dev/cpu/*/msr` privileges (Apple Silicon M4 Max hardware counters and Metal GPU + native window Forma acceptance are complete and recorded in `bench/CHANGELOG.md` and `bench/docs/VALIDATION.md`).

## Where finished work is recorded

- Changelogs: [bench/CHANGELOG.md](bench/CHANGELOG.md), [unit/CHANGELOG.md](unit/CHANGELOG.md), [unit/containers/CHANGELOG.md](unit/containers/CHANGELOG.md), [dash/hub/CHANGELOG.md](dash/hub/CHANGELOG.md), [err/CHANGELOG.md](err/CHANGELOG.md), [mon/CHANGELOG.md](mon/CHANGELOG.md), [otel/CHANGELOG.md](otel/CHANGELOG.md), [run/CHANGELOG.md](run/CHANGELOG.md).
- Scenario & validation docs: [bench/docs/FEATURES.md](bench/docs/FEATURES.md), [bench/docs/VALIDATION.md](bench/docs/VALIDATION.md), [unit/PILOT.md](unit/PILOT.md).
