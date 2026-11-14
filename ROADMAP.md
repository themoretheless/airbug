# Airbug roadmap

The single plan for the whole monorepo: only external, credential-gated, or physical-hardware validation items remain open. Implemented engineering items are recorded in the crate changelogs below.

Last reviewed: 2026-11-14.

## Stage 1 — Release (`run`, `bench`)

1. **Manual Stage 1 crates.io publication (`cargo publish`)**: all workspace crates are at `0.11.0` and ready to package offline; publishing requires maintainer crates.io credentials in dependency order — `airbug-macros` and `airbug` first, then `airbug-run`, `airbug-bench-macros`, `airbug-bench`, `cargo-airbug-bench`, `airbug-err`, `airbug-otel`.

## Stage 2 — Physical Hardware Acceptance (`bench`)

2. **Bare-metal Linux PMU & MSR/RAPL acceptance**: `airbug_bench::perf` (`Instructions`, `Cycles`, `CacheReferences`, `CacheMisses`, `BranchInstructions`, `BranchMisses`, `ICacheMisses`, `DtlbMisses`, `ItlbMisses`, `PageFaults`) and `host.rs` sysfs RAPL (`energy_uj`) / MSR probing require validation on a physical Linux host with exposed PMU counters and `/dev/cpu/*/msr` privileges.
3. **Physical Apple Silicon (`macOS` ARM64) counter acceptance**: verify `airbug-bench` host probes (`sysctl` CPU frequency) and optional macOS performance counters (`kpc`) on physical Apple Silicon hardware.

## Stage 3 — Physical GPU & External Forma Checkout (`bench`)

4. **Forma on physical GPU and display hardware**: `bench/integrations/forma` integrates `airbug_bench::frames::Frames` lifecycle tracking, GPU timestamp validation (`GpuSpan`), and `TrackingAllocator<memory::Allocator<System>>` (`--alloc-stacks DIR`). Running `bench/integrations/forma` end-to-end requires the external `research/private/forma/vector-ui` checkout on a host with a physical GPU and display.

## Stage 4 — Multi-OS Remote CI Execution

5. **Remote GitHub Actions multi-OS runner sign-off**: `.github/workflows/ci.yml` defines `ubuntu-latest`, `macos-latest`, and `windows-latest` jobs; remote workflow execution and `unit` production service adoption remain for CI/environment execution.

## Where finished work is recorded

- Changelogs: [bench/CHANGELOG.md](bench/CHANGELOG.md), [unit/CHANGELOG.md](unit/CHANGELOG.md), [unit/containers/CHANGELOG.md](unit/containers/CHANGELOG.md), [dash/hub/CHANGELOG.md](dash/hub/CHANGELOG.md), [err/CHANGELOG.md](err/CHANGELOG.md), [mon/CHANGELOG.md](mon/CHANGELOG.md), [otel/CHANGELOG.md](otel/CHANGELOG.md), [run/CHANGELOG.md](run/CHANGELOG.md).
- Scenario & validation docs: [bench/docs/FEATURES.md](bench/docs/FEATURES.md), [bench/docs/VALIDATION.md](bench/docs/VALIDATION.md), [unit/PILOT.md](unit/PILOT.md).
