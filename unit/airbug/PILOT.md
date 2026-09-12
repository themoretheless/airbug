# Checkout pilot and call ordering

## Scope

This is a local demonstration service, not an integration into an existing user
application. The Airbug repository contained library code and examples only; a
repository/module path is still needed for a real-service adoption pilot. No
payment provider, database or event broker is contacted.

The service in [examples/checkout/domain.rs](examples/checkout/domain.rs) uses
ordinary traits, structs and Result. Optional cfg_attr annotations generate test
mocks with the macros feature; the same service compiles without it.

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
- No performance or production-readiness claim follows from this local pilot.
  Remote Linux/macOS/Windows CI remains to be run on a connected repository.

## Run

```text
cargo test --features macros --test checkout_pilot --test checkout_native --test order
cargo run --features macros --example checkout_pilot -- success
cargo run --features macros --example checkout_pilot -- decline
cargo run --features macros --example checkout_pilot -- store-failure
```
