# Airbug web reports

Generate an Allure-inspired, self-contained report from real native Rust tests:

```sh
python3 tools/airbug_report.py --workspace --all-features --locked --doc-tests
```

Open `target/airbug-report/index.html` in a browser. No server, Node.js, npm,
external assets or network connection is needed to view the report. The CLI needs
Python 3.9+ and Cargo/Rust on PATH. Add `--offline` when dependencies are cached.
The tool also works with ordinary Rust projects that do not depend on Airbug:

```sh
python3 /path/to/airbug/tools/airbug_report.py \
  --manifest-path /path/to/project/Cargo.toml \
  --output /path/to/reports/project --workspace --all-features
```

## Interface

- Overview: status counts, pass rate excluding skipped entries, wall time,
  suite summaries, failures and infrastructure problems.
- Test cases: search names, suites and output; filter status/suite; sort by name,
  duration or failures first; paginate large runs.
- Details: stdout/stderr, exit code, timeout/truncation indicators, recent
  outcomes for the same test. `#[airbug::cases]` cases are individual entries.
- Timeline: measured monotonic start/end offsets for each case, chronological
  bars, exact duration, suite/status/search filters, zoom up to 64× and an option
  to fit the time range to filtered entries. Optional build/discovery intervals
  explain runner overhead. Click a test label or bar to open its log. Older
  reports without timestamps show an unavailable state; times are not inferred.
- History: the last 20 runs in the same output directory, including commit,
  status counts and duration. A filtered run has its own counts; changes in
  outcomes are not labelled flaky automatically.
- JSON export, responsive layout, keyboard-operable controls and dialogs,
  automatic light/dark appearance.

## Execution contract

The adapter uses Cargo JSON build artifacts and the standard libtest harness.
It builds once, discovers tests with `--list --format terse`, then executes each
selected test with `--exact --nocapture` in a separate process, serially. Package
working directories, package metadata variables, build-script environment
outputs, and generated dynamic-library paths are restored for those processes.
This follows the documented [Cargo test working directory rules](https://doc.rust-lang.org/cargo/commands/cargo-test.html#working-directory-of-tests)
and [runtime environment](https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths).

This is an optional diagnostic run, with different process isolation and timing
from one ordinary parallel `cargo test` invocation. Durations include process
startup; they are not benchmarks. Keep normal `cargo test` in CI as well.
Custom harnesses, custom Cargo target runners, cross-compilation and reproducing
Cargo configuration-provided runtime environment (`[env]`) are not supported.
Export any required environment variables before invoking this tool. Native
benchmarks are excluded. Libtest's text discovery/summary format is validated;
unrecognized output becomes `broken`, never an assumed pass.

`#[ignore]` is respected unless `--include-ignored` is explicitly supplied.
`#[should_panic]` is evaluated by libtest. A process that exits without completing
one test is `broken`, including a test calling `exit(0)`. Tests are never retried.

Doctests are opt-in with `--doc-tests` and run through Cargo as **one aggregate
entry**, with individual results in its log. They are not included in the native
case count. A documentation-only run with no executed examples is skipped.

Options include `--package NAME` (repeatable), `--features`, `--all-features`,
`--no-default-features`, `--workspace`, `--release`, `--locked`, `--offline`,
`--filter SUBSTRING`, `--title`, `--timeout SECONDS` (default 120 per process),
and `--build-timeout SECONDS` (default 600, also for the aggregate doctest run).
Output paths are relative to the CLI's initial working directory.

Exit codes: `0` for completed results without failures/broken entries (ignored
entries remain skipped); `1` for failed/broken results, including build failures;
`2` for no matching entries or report/configuration errors. Build failures still
produce an HTML report. Ctrl-C terminates the active process group; an interrupted
run does not replace the previous report. Timeout termination also targets child
processes (POSIX process group / Windows `taskkill /T`).

## Files and CI

The output directory contains `index.html`, `report.json`, and `history.json`.
The HTML embeds its data, so it can be copied alone. JSON uses `schema: 1`.
Persist `history.json` in the same output directory to retain trends across jobs.
Different projects do not share history, and invalid history is ignored with a
warning. The repository CI uploads the HTML/JSON as an artifact, including failed
runs; history persistence across CI machines is not configured automatically.

Report files are replaced atomically, individually. An exclusive `.report.lock`
prevents concurrent report writers from corrupting history. If the writer is
forcibly killed during export, remove this lock only after confirming it is no
longer running. The three files are not a transactional database; the HTML always
contains the matching data even if export is interrupted between file replacements.

Test logs retain the last 128 KiB, with explicit truncation. Build/discovery
output is limited to 16 MiB; exceeding that limit is an infrastructure error.
Process output is spooled to temporary disk files, so exceptionally noisy tests
can consume disk space until termination. Logs are embedded as inert text, and
the report blocks network requests. Logs may contain application data; there is
no automatic secret redaction.

Run reporter regression tests with:

```sh
python3 -m unittest discover -s tools/tests -v
```

This is not an Allure results adapter or an Allure server. Issue-tracker links
and a multi-user report service are not yet implemented.

## Structured steps, attachments and comparisons

```rust
use airbug::report;

#[test]
fn checkout() {
    report::step("Checkout", || {
        let total = report::step("Calculate total", || 2 * 1250);
        report::attach_text("payment.log", "status: 201").unwrap();
        report::attach_bytes("request.json", "application/json", br#"{"amount":2500}"#).unwrap();
        report::assert_equal("Charged amount", &2500, &total);
        report::assert_text_equal("Response", "paid\n", "paid\n");
    });
}
```

Steps are synchronous closures. Nested steps on the same thread retain their
parent; separate threads start separate root steps. `step` marks normal returns
passed and panics failed, preserving the original panic payload. Use `try_step`
for closures returning `Result`: `Err` marks that step failed and returns the same
error. Neither function polls futures or propagates context into spawned tasks.
A timeout/abort can leave an incomplete step; it is displayed as broken.

Step/comparison statuses describe individual operations. A `#[should_panic]` test
or a test that catches a failure can pass while containing a failed step. The
native harness still determines the test outcome. Malformed/incomplete diagnostic
files turn an otherwise passing entry into broken, with an explicit diagnostic
error; they never hide an existing native test failure.

`attach_text` and `attach_bytes` attach to the current step (or the test root),
returning I/O/size errors to the caller. Display names never become filesystem
paths. Text/JSON/XML/HTML is previewed as inert text. PNG/JPEG/GIF/WebP can be
previewed as images; other binary formats, including SVG, are download-only.
The original bytes are embedded in the standalone HTML and JSON for download.
No secret redaction is performed.

`assert_equal(name, expected, actual)` records pretty Debug values;
`assert_text_equal` records raw text for a line diff. Existing
`assert_that(...).is_equal_to(...)` and `CheckReport::assert()` also record their
equality failures. Plain Rust `assert_eq!` remains unchanged: its panic output
is available in the log, but no expected/actual fields are guessed from it.

Limits per reporting test process: 1 MiB per attachment, 4 MiB total attachments,
8 MiB event file, 4096 events, 64 nested step levels. Text comparison fields retain
at most 64 KiB on UTF-8 boundaries, with an explicit truncation flag. Diff
calculation uses at most 2048 lines per side and retains at most 256 KiB. Text
attachment previews retain 64 KiB; downloads retain complete accepted files.
Raw sidecar files live in a temporary per-test directory and are removed after
collection. `AIRBUG_REPORT_DIR` is set only by the runner; no files are emitted by
ordinary `cargo test`. Subprocesses should remove this variable before starting
another independently instrumented test process; the sidecar belongs to one
process, with thread-safe writes inside it. Doctests are still aggregate entries
and are not configured for structured diagnostics.

Diagnostic recording errors do not replace the action's result or panic. Step
recording logs a warning and writes an error marker for the collector; attachment
errors are returned to the caller. If the directory itself becomes inaccessible,
only the stderr warning may be available. Treat report output as diagnostic data,
not an independent proof that a test passed.
