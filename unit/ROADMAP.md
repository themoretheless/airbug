# Remaining development areas

Version 0.4 implements the selected 50 ideas; see TOP50.md for the exact mapping.
**NEXT50 is complete** — see [NEXT50.md](NEXT50.md). Listed stubs remain intentional:

- `UpdateMode::InlineSource` → `SnapshotError::Unsupported` (no source rewrite)
- `CompletionBarrier::await_done` → explicit Err / panic (`use application join/await`)
- `#[mock]` generics / associated types → compile_error pointing at manual
  `Mock` + `ExpectationBuilder::map_args` / `mock::borrowed`
- `Prop::shrink_from` / `Shrink` are minimal, not a full property framework

- Property-based testing: seeded `Prop::for_all` / `replay` / seed files / minimal shrink.
- Generic trait/method mocks: escape hatches (`map_args`, `borrowed`); macro still
  rejects unsupported shapes with a clear message.
- Partial-order `CallDag` / `happened_before`; pending async probe via
  `Mock::pending` / `pending_response`. Completion waits stay application-owned.
- Async validation with cancellation and `validate_parallel`.
- Structured JSON snapshots (`json` feature), obsolete detection, hex bytes,
  schema stamp; inline-source remains unsupported by design in this cut.
- Optional HTTP/database/runtime integrations driven by real application tests.
  See `unit/containers` (`airbug-containers`) for Testcontainers.NET-style Docker helpers
  (`Wait::http` / `Wait::http_json`, CI soft-skip docs).

The standard Rust test runner remains the foundation. No external services are
started, snapshot baselines changed, or failed tests retried implicitly.
