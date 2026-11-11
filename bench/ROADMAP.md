# AirBench Development Roadmap

This document tracks implementation progress of optimization ideas from FEATURES.md (~1000 items).

## Priority Tier System

### 🔴 Critical (Immediate Implementation)
These are foundational features that enable the core value proposition.

### 🟡 High (Next Sprint)
Important for usability and competitiveness but not blocking MVP.

### 🟢 Medium (Backlog)
Nice-to-have enhancements that improve completeness.

### ⚪ Low (Future Consideration)
Low priority or experimental features.

---

## Current Status: Planning Phase (Version 0.9.x)

| ID | Feature | Tier | Status | Notes |
|----|---------|------|--------|-------|
| 1 | Case-level wall clock precision to nanosecond granularity | 🔴 | ✅ Done | Core functionality exists |
| 2 | Per-sample variance tracking across iterations | 🔴 | ✅ Done | Implemented in Suite |
| 3 | Memory allocation profiling per benchmark case | 🔴 | ⏳ Planned | Need hardware perf events |
| 4 | CPU frequency scaling correlation logging | 🔴 | ⏳ Planned | Requires MSR access |
| 5 | Cache miss rate instrumentation via hardware counters | 🔴 | 📝 In Progress | Example created, needs PFC integration |
| 6 | Branch misprediction counting in hot paths | 🔴 | 📝 In Progress | Same as #5 |
| 7 | SIMD utilization metrics extraction | 🔴 | 📝 Backlog | Needs PMU access |
| 8 | Instruction cache pressure measurement | 🔴 | 📝 Backlog | Intel-specific PMU |
| ... | ... | ... | ... | ... |

---

## Implementation Categories

### Performance Instrumentation (Features 1-100)

#### Completed
- [x] Basic timing infrastructure with nanosecond precision
- [x] Statistical analysis (mean, median, confidence intervals)
- [x] Deterministic RNG seeding for reproducibility

#### In Progress
- [🚧] Hardware performance counter integration
  - File: `bench/src/perf_counters.rs` (TBD)
  - Dependencies: `perf-event-open-family`, `libpfm4`
  - Platforms: Linux/x86_64 first, then macOS ARM

#### Blocked
- [ ] Power consumption monitoring (requires external hardware)
- [ ] Thermal throttling detection (requires CPU model specific registers)

### Benchmark Infrastructure (Features 101-200)

#### Completed
- [x] JSON protocol for inter-process communication
- [x] Live dashboard server with WebSocket support
- [x] Protocol buffer exports

#### In Progress
- [🚧] Container performance isolation tests
- [ ] Docker sandbox overhead measurement

### Data Structures & Algorithms (Features 401-500)

#### Examples Created
- [x] `examples/cache-benchmark.rs` - Cache patterns and hash functions
- [ ] `examples/concurrency-benchmark.rs` - Lock-free structures (TBD)
- [ ] `examples/crypto-benchmark.rs` - Post-quantum algorithms (TBD)

#### Pending
- [ ] Bloom filter variants benchmarking
- [ ] Roaring bitmap operations
- [ ] Skip list vs red-black tree comparison

---

## Milestones

### v0.10.0 (Q1 2027)
- [ ] Hardware perf counter support (Linux + macOS)
- [ ] Memory allocator comparison suite
- [ ] Lock-free queue benchmarks (Michael-Scott)
- [ ] Post-quantum cryptography implementations

### v0.11.0 (Q2 2027)
- [ ] WebAssembly execution mode benchmarking
- [ ] Cross-platform CI pipeline integration
- [ ] Automated regression detection
- [ ] Custom dashboard plugins API

### v1.0.0 (Q3 2027)
- [ ] Full feature parity with documented ~1000 ideas
- [ ] Production-ready documentation
- [ ] Community benchmark library
- [ ] Plugin ecosystem for third-party extensions

---

## Contribution Guidelines

To add a new benchmark case:
1. Create `bench/examples/my-new-feature.rs`
2. Use `Suite::bench_fixture()` pattern from `cache-benchmark.rs`
3. Add tags: `.tag("category").tag("subcategory")`
4. Run `cargo check --example my-new-feature`
5. Submit PR with benchmark output samples

### Tagging Convention
```rust
.tag("performance").tag("memory")
.parameter("allocator", "jemalloc")
.parameter("size", "4096")
```

---

## Tracking Issues

| Feature Range | GitHub Issue | Status |
|---------------|--------------|--------|
| Hardware counters | #XXX | Open |
| Lock-free data structures | #XXX | Planned |
| Post-quantum crypto | #XXX | Backlog |
| WebAssembly support | #XXX | Future |

---

*Last updated: 2026-10-08*
*Source: FEATURES.md catalog (~1000 optimization ideas)*
