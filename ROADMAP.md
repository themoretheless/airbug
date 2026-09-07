# Remaining development areas

Version 0.4 implements the selected 50 ideas; see TOP50.md for the exact mapping.

- Property-based testing with shrinking, persisted counterexamples and replay.
- Generic trait/method mocks, associated types and complex borrowed returns.
- Partial-order dependencies and runtime-specific pending/cancelled async interaction checks.
- Strict cross-method admission ordering is implemented in 0.5.
- Async validation with cancellation and controlled concurrency.
- Structured JSON snapshots, inline-source updates and obsolete-snapshot detection.
- Optional HTTP/database/runtime integrations driven by real application tests.

The standard Rust test runner remains the foundation. No external services are
started, snapshot baselines changed, or failed tests retried implicitly.
