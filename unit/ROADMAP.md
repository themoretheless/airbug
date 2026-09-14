# Remaining development areas

Version 0.4 implements the selected 50 ideas; see TOP50.md for the exact mapping.
NEXT50 (in progress): see [NEXT50.md](NEXT50.md). Slice 1 ships JSON snapshots
(feature `json`), obsolete detection, seeded `Prop`, and `Eventually::check_async`.

- Property-based testing with shrinking, persisted counterexamples and replay.
  Seeded `Prop::for_all` / `replay` / seed files are in NEXT50 slice 1; shrinking remains.
- Generic trait/method mocks, associated types and complex borrowed returns.
- Partial-order dependencies and runtime-specific pending/cancelled async interaction checks.
- Strict cross-method admission ordering is implemented in 0.5.
- Async validation with cancellation and controlled concurrency.
- Structured JSON snapshots, inline-source updates and obsolete-snapshot detection.
  `check_json` / `list_obsolete` / `assert_no_obsolete` are in NEXT50 slice 1
  (`json` feature for JSON); inline-source updates remain.
- Optional HTTP/database/runtime integrations driven by real application tests.
  See `unit/containers` (`airbug-containers`) for Testcontainers.NET-style Docker helpers.

The standard Rust test runner remains the foundation. No external services are
started, snapshot baselines changed, or failed tests retried implicitly.
