# Unreleased

- Renamed the crates: `runit` is now `airbug`, `runit-macros` is now
  `airbug-macros`. Breaking for every user; update the dependency name and any
  `runit::` paths.
- Documented the whole public API and enabled `missing_docs` plus
  `rustdoc::broken_intra_doc_links`; CI now fails on either.
- `docs.rs` builds with all features, so the macro API appears in the published
  documentation.
- `Assertion::check` and `Assertion::subject` are public, so downstream crates
  can add assertions in an extension trait with the same failure formatting.
- Split `mock.rs` into submodules and moved `MockError`, `VerificationErrors`
  and `VerifyMocks` to a new `airbug::verify`, removing the `mock` <-> `order`
  module cycle. `airbug::mock::MockError` still resolves.
- `prelude` lists its `checks` imports explicitly instead of glob-importing.
- Tests renamed by subject rather than by origin; the shared checkout domain is
  a `airbug-checkout-domain` dev-dependency crate instead of a file the tests reached
  into with `#[path]`.

# 0.5.0

## Unreleased

- Add synchronous nested report steps, text/binary attachments, and structured expected/actual diffs; preserve native assertions and panic outcomes.

- Add an execution timeline with measured start/end offsets, zoom, filters and build/discovery phases.

- Add an optional native-test report CLI and standalone Allure-inspired web UI:
  suite/status/search filters, logs, durations, JSON export and bounded run history.
- Distinguish test failures, ignored cases, process timeouts and build/discovery errors.
- Add end-to-end reporter tests and a CI report artifact job.

- Shared CallSequence enforces strict admission order across mock methods/objects.
- Out-of-order calls retain diagnostic errors without consuming counts or running answers.
- Positive exact repeated steps, cross-thread admission and optional grouped verification.
- Ordered expectations reject count resets; ordinary mock behavior remains unchanged.
- Snapshot test directories now include a unique counter, preventing timestamp collisions.
- Checkout demonstration pilot covers idempotency, compensation and event failures,
  with a handwritten native baseline for the same service.

# 0.4.0

- Implements the 50 selected additions, mapped to API/tests in TOP50.md.
- Fixture scopes, children, replay, seed diagnostics, graph budget, generators.
- Mock response sequences, one-shot answers, count ranges, capture and bounded journals.
- Reusable matchers and safe reset of reusable expectation counts.
- Native structured checks, collection checks, numeric/text/error diagnostics.
- Validator codes, cross-field and conditional rules, optional children and error budgets.
- Explicit text/inline snapshots, bounded diffs, literal redaction and named-case paths.
- Manual calendar/monotonic clock and cooperative eventual checks with history.
- Breaking: GenerationError gains seed; ValidationError gains code; NodeLimit variant.

# 0.3.0

- Optional macros feature: Generate derive and field builders, mock trait adapters,
  named cases emitted as ordinary #[test] functions.
- Native collection assertions and grouped boolean checks.
- Optional sync/async verification wrappers; no custom runtime or test runner.
- Consumer compile-fail diagnostics and renamed-dependency tests.
- Core remains dependency-free with default features.

# 0.2.0

- Breaking: Generate returns Result; nested generation uses try_build.
- Typed generation errors, fallible factories, removable overrides, TypeId cycle detection.
- Thread-safe reusable validators, conditional rules, field cascade, collection validation.
- Strict thread-safe method mocks with matchers, answers, exact counts, explicit verification.
- Assertion value projections, vector checks, ranges, string boundaries, float tolerance.
- Stable Rust checks, platform CI matrix, integration tests, runnable service example.

# 0.1.0

Initial experimental implementation.
