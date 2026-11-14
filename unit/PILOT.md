# Checkout pilot and call ordering

## Scope

This pilot exercises a separate domain crate (`unit/checkout-domain`, package
`airbug-checkout-domain`) end-to-end against `airbug`. No external payment provider,
database, or broker is contacted.

`airbug-checkout-domain` uses ordinary traits, structs, and `Result`. Optional
`cfg_attr(feature = "macros", airbug::mock)` attributes generate thread-safe mocks
when the `macros` feature is enabled while allowing dependency-free builds without it.

`tests/service_pilot.rs` extends the pilot beyond basic mocks to cover generic and
associated-type traits (`#[airbug::mock]`), `CallSequence` admission order,
`CompletionBarrier` concurrent worker coordination, `FixtureContext` + `#[derive(Generate)]`,
`Prop::for_all_shrunk`, `Snapshots` (`UpdateMode::InlineSource` and JSON), and `SoftAssert`.

Inside the monorepo, [dash/hub/tests/unit_service_adoption.rs](../dash/hub/tests/unit_service_adoption.rs)
applies `airbug` (`macros` + `json`) to the real `airbug-hub` service (`HubApp`,
`IssueStore`, `SymbolStore`, `StreamHub`, `EventEnvelope`).

## Scenarios checked

| Scenario | Required interactions |
|---|---|
| Success | find → charge → save → publish, with exact arguments |
| Invalid id/customer/amount (3 cases) | none |
| Identical request already exists | find only; return original receipt |
| Same id with changed payload | find only; reject conflict |
| Lookup failure | find only |
| Charge rejected | find → charge; no save/event/refund |
| Save failed | find → charge → save → refund exact charge |
| Refund also failed | same order; retain both failures |
| Event publication failed | report saved/paid receipt; do not refund |
| Deliberately out-of-order save | immediate diagnostic; no answer execution |

The pilot contains 12 named tests in [tests/checkout_pilot.rs](tests/checkout_pilot.rs).
The order engine has separate tests for missing/excessive calls, repeats, ranges,
late registration, cross-thread admission, re-entry, reset and panic behavior.

## Native baseline versus Airbug

[tests/checkout_native.rs](tests/checkout_native.rs) tests the same service's happy
path and lookup failure using standard Rust alone: five handwritten trait methods,
a shared call log and an enum describing the recorded interactions.

The Airbug pilot replaces those five handwritten adapters with three mock
attributes. Each scenario declares answers, argument matchers and ordered steps.
`with_mocks` checks all supplied mocks plus the sequence after the test body,
including when the service normally returns an Err. Assertions remain assert_eq.

This does not establish a universal reduction in line count: Airbug expectations
still take space, and a reusable handwritten fake can be concise. The observed
benefit here is removing adapter/logging code and reporting wrong order at the
violating call rather than after a final whole-log comparison. The native baseline
covers fewer scenarios; comparing their whole-file line counts would be misleading.

The three-field input is built with a literal helper. A Fixture builder would add
setup here without meaningful savings. Fixture remains useful for larger graphs;
it is not mandatory in the service pilot.

## Order contract

CallSequence orders admission into answers, not completion of them. Register
positive exact-count steps before any call to a participating method mock.
Unordered calls may interleave. Repeated identical calls use one step with times(n)
or returns_sequence; first-matching rules do not fall through when exhausted.

Verify method mocks as well as the sequence. An unrelated unexpected argument can
fail its method mock without being recognized as an ordered step. A rejected
ordered call preserves its count and position but latches a verification failure.
Use a new sequence for each test/phase; ordered counts cannot be reset.

## Findings and limits

- The service distinguishes failed persistence from failed event publication.
  Refunding after a successful save would be a different business policy.
- A failed event is surfaced with its committed receipt. Durable delivery/outbox
  and retry handling are not implemented in this demonstration service.
- Idempotency is checked sequentially; concurrent deduplication and atomic storage
  are not modeled. Those need tests against the real persistence layer.
- Service calls are synchronous. Async completion ordering needs explicit awaits
  or a future runtime-specific adapter, not just admission sequencing.
- No performance claim follows from this pilot; `.github/workflows/ci.yml` (`pilot-matrix`)
  executes the pilot suite across `ubuntu-latest`, `macos-latest`, and `windows-latest`.

## Run

```text
cargo test -p airbug --all-features --test service_pilot --test checkout_pilot --test checkout_native --test order
cargo run -p airbug --features macros --example checkout_pilot -- success
cargo run -p airbug --features macros --example checkout_pilot -- decline
cargo run -p airbug --features macros --example checkout_pilot -- store-failure
```
