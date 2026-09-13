# Airbug

Helpers for ordinary Rust tests: keep `#[test]`, `cargo test`, `assert!` and
`assert_eq!`. Add data generation, trait mocks, named cases, collection checks,
and reusable validation where they reduce test boilerplate. Rust 1.96 or newer.
No unsafe code. `airbug-checkout-domain` is a **demo** crate (`publish = false`) used by pilot examples only.
 The default build has no external dependencies; optional macros
use a separate procedural-macro crate.

## Cross-method order (0.5)

```rust
use airbug::{CallSequence, Mock, with_mocks};
let order = CallSequence::new("checkout");
let charge = Mock::<(), ()>::new("payments.charge");
let save = Mock::<(), ()>::new("orders.save");
charge.expect("charge", |_| true).in_sequence(&order).returns(());
save.expect("save", |_| true).in_sequence(&order).returns(());
with_mocks(&[&charge, &save, &order], || {
    charge.call(());
    save.call(());
});
```

Terminal answer registrations append steps in order. Calls are admitted atomically
across method mocks. Each step needs a positive exact count; zero and variable
count ranges are rejected before reserving a step. A repeated step completes
only after all its calls. `.returns_sequence` sets that count to response length.

Wrong-order calls return MockError from try_call (or panic from call), do not run
capture/answer callbacks, and do not consume counts or advance the sequence.
The violation remains visible in both the method mock and sequence verification,
even after later correct calls. Excess calls are also retained. Always verify the
mocks: a sequence cannot detect unrelated unexpected arguments by itself.

Order means **admission**, not completion. An answer may still be running when
the next step is admitted; wait for operation completion in the service when that
is the requirement. Sequence locking does not run user callbacks. Calls without
a sequence can interleave. Finish registering the sequence before the first call
to any mock with a registered step, including a call with nonmatching arguments.
Ordered mocks cannot reset counts; create a fresh sequence for a new scenario.

First-matching expectation semantics remain unchanged. For the same method with
identical matchers, use one repeated step, not separate overlapping expectations.
Different matchers may be placed at different steps. A panic in an admitted answer
still counts as an invocation. Sequences do not verify in Drop and have no reset.

The [checkout pilot](PILOT.md) compares a handwritten baseline with generated mocks,
using the same demonstration service. Run it with:

```text
cargo run --features macros --example checkout_pilot -- success
cargo run --features macros --example checkout_pilot -- decline
cargo run --features macros --example checkout_pilot -- store-failure
```

## Top 50 additions (0.4)

[TOP50.md](TOP50.md) maps all 50 selected ideas to their API and regression test.
The `prelude` is optional; direct module imports remain supported.

```rust
use airbug::prelude::*;
let mut fixture = FixtureContext::with_seed(42);
fixture.reuse(10u64);
let temporary = fixture.scoped(|ctx| { ctx.reuse(20u64); ctx.build::<u64>() });
assert_eq!(temporary, 20);
assert_eq!(fixture.build::<u64>(), 10);
```

Fixture supports child contexts with independent seeds, checkpoint/restore of
built-in RNG state and configuration, checked sequential u64 IDs, uniform choice
from slices, ordered numeric pairs, and lazy fallible streams. `max_nodes` caps
all generation requests in one root graph, including the root. Captured factory
state remains shared through Rc: checkpoint and child contexts cannot rewind
external side effects. `scoped` restores registrations and limits after success
or unwind but retains RNG progress. Root generation errors include the seed.
Boundary sets live in `fixture::boundaries` for ordinary table-driven tests.

```rust
use airbug::prelude::*;
let capture = Capture::new(4);
let fetch = Mock::<u64, Option<u64>>::new("fetch");
fetch.journal_capacity(4);
fetch.expect("id", |id| *id == 7).capture(&capture)
    .returns_sequence([None, Some(42)]);
assert_eq!(fetch.call(7), None);
assert_eq!(fetch.call(7), Some(42));
assert_eq!(capture.values(), [7, 7]);
fetch.assert_verified();
```

`returns_sequence` forces the exact sequence length; `returning_once` forces one
call and accepts non-Clone owned responses. They replace any earlier count setting.
Concurrent sequences are consumed in answer-lock order. `times_between`,
`at_least`, and `at_most` configure reusable answers. First matching rule still
wins; an exhausted rule does not fall through. `Matcher` supports named reusable
predicates with short-circuit and/or/negate. Captures are bounded owned copies;
cloning user arguments happens outside locks. Journaling is opt-in and bounded,
and stores Debug representations of accepted/rejected calls. Debug data may be
sensitive. `reset_counts` clears counts, failures and journal for reusable answers
only; it rejects active calls, ordered expectations and one-shot/response-sequence expectations. It does not
clear separate captures or reopen configuration.

```rust
use airbug::prelude::*;
let mut report = CheckReport::default();
report.equal("order.quantity", &2, &2);
report.assert();
assert_subset(&[1, 1], &[1, 2, 1]);
assert_sorted(&[1, 1, 2]);
assert_relative_eq(100.0, 100.01, 0.001);
```

The checks module adds field projections, unique/sorted/all/count checks,
multiset subset/superset, relative float comparison, newline normalization,
error-source chain checks, panic-message checks and structured failure records.
`CheckReport` borrows values only while evaluating and stores owned diagnostic
strings; call assert explicitly to fail the test. Uniqueness and multiset checks
are O(n²). Error chains are inspected up to 64 links. Relative comparison requires
finite values and nonnegative finite tolerance. Panic checks require UnwindSafe
closures and unwind builds; non-string panic payloads do not match text.

```rust
use airbug::Validator;
let validator = Validator::<(u8, u8)>::new()
    .check("end", "range_order", "end must follow start", |(start, end)| start <= end)
    .max_errors(2);
let errors = validator.validate(&(5, 1)).unwrap_err()
    .map(|error| (error.field, error.code));
assert_eq!(errors, [("end".into(), "range_order".into())]);
```

Every validation error now has a code independent of its message. Built-in codes
are `not_empty`, `length`, `greater_than`, `inclusive_between`, `not_none`, and
`unique`; must defaults to `custom`. `with_code` replaces the last check's code.
Object-level check compares multiple fields; group_when runs a conditional
validator; optional skips None and validates Some; unique_by reports repeated
keys at element paths. A positive max_errors budget propagates through nested
rules and skips later callbacks once full; global stop_on_first_failure sets it
to one. Error maps convert owned records into application-specific formats.

```rust
use airbug::prelude::*;
let snapshots = Snapshots::new("snapshots").redact("request-123", "<request>");
snapshots.inline("id=<request>", "id=request-123").unwrap();
```

Snapshots default to read-only Verify. `check` compares a named text file;
`check_case` uses a separate path per named case. Names accept 1..=100 ASCII
letters, digits, underscores or hyphens. CreateMissing explicitly creates absent
files but refuses changes; Overwrite explicitly replaces them. Inline comparison
never edits source. Literal redactions run on actual text in declaration order
before comparison/storage. Text is exact, including trailing newline. File size
and redaction expansion default to a 1 MiB limit. The diff reports at most 32
differing lines. Updates use exclusive lock files and same-directory rename;
concurrent writers may receive an I/O error rather than overwrite each other's
partial output. A process crash may leave .lock/.tmp files for manual cleanup.
Snapshot directories are caller-owned trusted locations, not a sandbox boundary.

```rust
use airbug::prelude::*;
use std::time::{Duration, SystemTime};
let clock = ManualClock::new(SystemTime::UNIX_EPOCH);
let mut attempts = 0;
let value = Eventually::new(Duration::from_secs(5), Duration::from_secs(1))
    .check(&clock, || { attempts += 1; attempts }, |n| *n == 3).unwrap();
assert_eq!(value, 3);
assert_eq!(clock.elapsed(), Duration::from_secs(2));
```

ManualClock shares state across clones. Advancing moves calendar and monotonic
time atomically; set_wall_time changes only calendar time. Manual sleep advances
without real waiting. RealClock uses Instant and thread sleep. Eventually polls
synchronously with a deadline and bounded diagnostic history, checks once even
with zero timeout, rejects zero intervals and nonadvancing clocks, and never
preempts a blocked callback. Probe/predicate panics propagate; join external
workers yourself. This is not an async scheduling simulator.

Version 0.4 adds public code/seed fields to ValidationError/GenerationError and a
NodeLimit error variant. Update struct literals and exhaustive matches when
migrating. Custom Generate implementations from 0.3 remain unchanged.

## Local installation

```toml
[dev-dependencies]
airbug = { path = "../airbug", features = ["macros"] }
```

Omit `features` for the dependency-free core. Validator may be used as a normal
application dependency; test helpers belong in dev-dependencies. No custom test
runner, global state, or mandatory TestContext is installed.

## Importing

There is one canonical path per name, so examples stay comparable:

- The main types come from the crate root: `airbug::{Mock, Validator,
  FixtureContext, CallSequence, assert_that, with_mocks}`.
- Supporting types stay in their module: `airbug::mock::{Capture, Matcher}`,
  `airbug::time::{Clock, ManualClock}`, `airbug::snapshot::Snapshots`,
  `airbug::checks::*`.
- `airbug::prelude::*` pulls in both sets at once, for test files that would
  otherwise open with a long import list.

Prefer the root path over the module path for anything the root re-exports:
`airbug::Mock`, not `airbug::mock::Mock`.

## Derived generation and field builders

```rust
# #[cfg(feature = "macros")] {
use airbug::{FixtureContext, Generate};
#[derive(Generate)]
struct Order {
    customer: String,
    note: String,
    quantity: u32,
    #[fixture(default)]
    tags: Vec<String>,
}
let mut fixture = FixtureContext::with_seed(42);
let order = fixture.builder::<Order>()
    .with_customer("Alice".into())
    .with_quantity(2)
    .build();
assert_eq!(order.customer, "Alice");
assert_eq!(order.quantity, 2);
assert!(order.tags.is_empty());
# }
```

Derive supports named, tuple and unit structs, type/const generics, and where
clauses. Generated setters are `with_field` (or `with_0` for tuple fields), with
the original field's visibility. `#[fixture(default)]` uses Default;
`#[fixture(with = function)]` calls a function taking `&mut FixtureContext` and
returning `Result<FieldType, GenerationError>`. An override skips that field's
generator. Builder generation bypasses the root factory for that one call,
retains nested factories, and still participates in cycle/depth checks.
Lifetime-parameterized structs, enums and unions require manual implementations.
The generated builder is named `TypeFixtureBuilder`; reserve that type name.

## Named test cases

```rust
# #[cfg(feature = "macros")]
#[airbug::cases(empty(0, 0), positive(2, 4), negative(-3, -6))]
fn doubles(input: i32, expected: i32) {
    assert_eq!(input * 2, expected);
}
```

Cases become separate native tests such as `doubles::positive`: filter, ignore,
and run them using cargo test. The attribute emits #[test] itself; do not add
another #[test]. Synchronous functions returning () or Result are supported,
along with should_panic and ignore. Case expressions run independently inside
each test. Async parameterized cases are not supported; use your runtime's test
attribute directly for async tests.

## Extra native assertions

```rust
use airbug::{assert_contains, assert_same_items, check_all};
assert_contains!([1, 2, 3], 2);
assert_same_items!([1, 2, 1], [2, 1, 1]);
let quantity = 2;
let customer = "Alice";
check_all! { quantity > 0; !customer.is_empty(); }
```

`assert_same_items!` ignores order but preserves duplicate counts (O(n²), no
Ord/Hash requirement). `check_all!` evaluates all boolean expressions once and
reports every false expression; user panics still propagate immediately. These
macros supplement native assertions and do not require fluent assertion syntax.

## Generated trait mocks

```rust
# #[cfg(feature = "macros")] {
use airbug::with_mocks;
#[airbug::mock]
trait Repository: Send + Sync {
    fn lookup(&self, customer: &str) -> Option<u64>;
}
let repository = MockRepository::default();
repository.lookup.expect("Alice", |(name,)| name == "Alice")
    .returns(Some(42));
with_mocks(&[&repository], || {
    assert_eq!(repository.lookup("Alice"), Some(42));
});
# }
```

The macro preserves the trait and adds `MockTrait`, constructed with Default.
Every method has a public Mock field of the same name. Arguments are tuples,
including `(value,)` for one argument and `()` for zero. Shared-reference inputs
are copied via ToOwned (str becomes String, slices become Vec); mutable-reference
arguments and borrowed returns require a manual adapter. Input values must be
Debug and their stored representations must be 'static. Default trait methods
are also mocked strictly, rather than executing their original bodies.

Safe nongeneric traits with optional Send/Sync supertraits are supported. Methods
may use &self or &mut self and may be async; generated async methods return their
configured value when polled, with no simulated scheduling delay. Generic
methods, associated types/constants, Self in argument/return types, opaque
returns, and method attributes other than doc need manual adapters. Generated
names are `MockTrait`; reserve that name in the trait's module.

`VerifyMocks::verify_mocks()` aggregates all method failures. Optional
`with_mocks` verifies after a synchronous body; `with_mocks_async` awaits a future
on your executor before verifying. Neither masks an original panic. Join/await
workers before the body completes. Dropping an unfinished async wrapper does
not verify. Return values (including Result) are passed through, but verification
still runs on normal completion, even when the body returns Err.

## Fixture

```rust
use airbug::{FixtureContext, Generate, GenerationError};
struct Order { id: u64, customer: String }
impl Generate for Order {
    fn generate(ctx: &mut FixtureContext) -> Result<Self, GenerationError> {
        Ok(Self { id: ctx.try_build()?, customer: ctx.try_build()? })
    }
}
let mut fixture = FixtureContext::with_seed(42);
fixture.reuse(String::from("Alice"));
let order = fixture.build::<Order>();
assert_eq!(order.customer, "Alice");
```

Each request traverses generation rules and creates a fresh value. `register`
overrides nested generation, `register_fallible` supports failures, and `remove`
restores the default rule. `reuse` clones a value; Rc/Arc preserve shared identity.
External types without Generate use `try_build_registered` or `build_registered`.

Use `try_build()?` **inside** Generate and fallible factories. It returns typed
cycle, depth, node-budget, missing-factory and custom errors without using panic for control
flow. `build` is a convenience that panics on errors at a test boundary. User
panics propagate; unwinding restores the graph stack. No recovery is possible
from an aborting panic. Random state and factory side effects are not rolled back.

Default depth is 64, graph budget is 100,000 nodes per root request, and vector length is 3; setters permit zero. An Option is
Some by default; register a None factory to terminate an optional recursive edge.
Recursive types are rejected on re-entry even if a user factory could eventually
terminate them. Contexts are per-test and not Send/Sync. Seeds reproduce values
for an identical traversal and architecture within this release; this is not a
cryptographic or property-testing generator. Floats are finite in [0, 1).

## Assertions

```rust
use airbug::assert_that;
assert_that(&Some(5)).value().is_between(&1, &10);
assert_that(&vec![1, 2]).has_length(2).contains(&1);
assert_that("Alice").because("customer was fixed").starts_with("Al");
assert_that(&0.1f64).is_close_to(0.10001, 0.001);
```

Assertions borrow values, chain, and panic at the caller on failure. `because`
applies to subsequent checks, including projected Option/Result values. Supported
checks cover equality, inequality, ordering, inclusive ranges, booleans, string
prefix/suffix/containment, slices/vectors, Option and Result. `is_close_to` uses
absolute tolerance and rejects nonfinite inputs or negative tolerance. Checks
stop on first failure; there is no soft-assertion scope.

## Validator

```rust
use airbug::Validator;
struct User { name: String, age: u8 }
let validator = Validator::<User>::new()
    .rule_for("name", |u| &u.name)
        .not_empty().with_message("Name is required")
        .length(2, 80).stop_on_first_failure().done()
    .rule_for("age", |u| &u.age).inclusive_between(18, 120).done();
let user = User { name: "Alice".into(), age: 25 };
assert!(validator.validate(&user).is_ok());
```

`validate` returns Result<(), ValidationErrors>. Errors include field paths and
messages in declaration order. By default every check runs; field-level
`stop_on_first_failure` changes that field only. `when` conditions all checks on
its field, and the last condition replaces earlier ones. `child` prefixes nested
paths; `for_each` adds element indexes, e.g. `items[2].name`. Field names are literal
path segments: use nonempty simple names for unambiguous output.

`must` adds arbitrary predicates, `not_none` checks presence. String `not_empty`
rejects whitespace, and length counts Unicode scalar values, not graphemes.
Reversed or unordered range bounds panic at construction, as does with_message
without a preceding check. Predicates must be Send + Sync; validators can be
shared across threads. Callbacks run synchronously and their panics propagate.

## Mock

```rust
use airbug::Mock;
let lookup = Mock::new("Repository::lookup");
lookup.expect("known id", |id: &u64| *id == 7)
    .times(2).returns(Some(String::from("Alice")));
assert_eq!(lookup.call(7).as_deref(), Some("Alice"));
assert_eq!(lookup.call(7).as_deref(), Some("Alice"));
lookup.assert_verified();
```

One Mock represents one method. Implement a trait by forwarding to call; use a
tuple for multiple arguments and () for no arguments. `returning` supplies a
computed response, including owned non-Clone results. `returns` clones a fixed
response. Expectation count defaults to one; times(0) forbids matching calls.
The **first matching rule wins**, even when exhausted: overlapping matchers do
not form a response sequence. Opt into cross-method order with CallSequence.

Configure expectations before dispatch; registering after the first call panics.
Clones share state. Counts are protected
by a mutex, while user matchers, Debug formatting, and answers execute outside
locks. Call counts mean invocation, even if an answer panics. User callback
panics propagate. `try_call` reports unexpected or excessive calls as MockError;
`call` panics. Both retain interaction failures for subsequent verification.
Call `verify` or `assert_verified` explicitly after workers join. Drop never
verifies; forgetting verification leaves missing calls undetected. Do not capture
strong clones of a mock in its own answer permanently: that creates an Arc cycle.

The low-level mock API uses owned 'static argument/result types. Borrowed-return
lifetime modeling and async validators remain outside this release's surface.

## Verification and release

```text
cargo test --workspace --all-features
cargo test -p airbug --no-default-features
cargo test --workspace --all-features --release
cargo run --features macros --example native_service
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
python3 unit/tools/verify_packages.py --toolchain stable
```

The package verifier requires Python 3.9+ and tests both extracted .crate archives
in an external consumer. It uses a local patch for the unpublished macro crate,
working around Cargo's temporary-registry checksum error. It does not publish.

The service example exercises all four modules and ensures invalid orders do
not reach the repository. CI runs tests on Linux, macOS, and Windows with Rust
1.96 and stable; local success does not substitute for the remote matrix.

Version 0.3 adds opt-in macros and native helpers. Version 0.2 changes Generate::generate to return Result; migrate child build calls
to try_build()?. Validator callbacks now require Send + Sync. This is a release
candidate for the documented surface, not compatibility with every .NET API.
Publishing and a stable 1.0 contract require a release review and successful CI.

## Web test reports

Generate a standalone browser report with suite summaries, searchable test cases,
status filters, failure logs and the last 20 runs:

```sh
python3 unit/tools/airbug_report.py --all-features --locked --doc-tests
```

Open `target/airbug-report/index.html`. The optional Python 3.9+ tool executes real
native tests one process per case and preserves failure exit codes. Doctests are
an opt-in aggregate entry. It needs no frontend dependencies or report server.
See [WEB_REPORT.md](WEB_REPORT.md) for execution differences, options and CI usage.

Structured diagnostics are opt-in via `airbug::report::step`, `try_step`,
`attach_text`, `attach_bytes`, `assert_equal` and `assert_text_equal`.
Equality failures from fluent assertions and `CheckReport` are recorded too.
Step/comparison statuses remain distinct from the native test outcome.
