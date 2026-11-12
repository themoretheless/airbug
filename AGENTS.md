# Agent instructions

## Benchmarks: hand over the hub link the run prints

The bench has no web server of its own. Every measuring command (`run`, `matrix`, `bench`,
`git-compare`, `resume`) announces its session in `.airbug/runs/` and, when a hub for this
directory is up, prints the run page on stdout:

```sh
cargo airbug serve --root . &          # once; see the checks in the next section
cargo airbug-bench run --program ./target/release/mybench --output .airbug-bench/session
# Live benchmark: http://127.0.0.1:8790/#/run/bench-<ms>-<pid>
```

- Capture the `Live benchmark: <url>` line and give it to the user as soon as it appears,
  before reading any results. The id is fresh per run: never reuse or guess one.
- No hub → no such line; stderr says `bench: no hub on :8790 — …` with the address the run
  will have. Start the hub (it reads the run store, so it can start after the run) or say that
  there is no live view. A hub for another root is reported the same way — do not link it.
- The command exits when results are saved; nothing has to be stopped afterwards.
- The page shows progress, the current variant and an ETA, the matrix grid
  (`--cpu 2 · candidate`), captured output, signals received during the run, and a comparison
  against a baseline. It polls the hub every 1.5 s; the hub reads `<output>/progress.json`
  and `status-final.json`. Reading `progress.json` directly is fine for progress in chat.
- `report.html` / `report.json` / `report.md` land in `<output>` after completion and work
  without a hub.
- `AIRBUG_DASHBOARD=0` keeps the session out of the run store — use it for strict,
  timing-sensitive measurements and say that you chose accuracy over the live page.
- `--dry-run` writes nothing. `--ui`, `--no-ui`, `--no-open` are accepted and ignored;
  `cargo airbug-bench serve` no longer exists.

## Traces, metrics, logs, errors: one hub, fixed address

```sh
cargo run -p airbug-hub -- serve --root . --collector
```

`http://127.0.0.1:8790/` is the page that fills itself in while things run: status every
15 s, OTLP traces every 4 s, OTLP logs every 4 s, metrics every 4 s, profiles every 8 s,
issues every 5 s (`FEEDS` in `dash/hub/static/js/main.js`). The port is fixed and there
is no token, so the link stays valid across restarts. Hand it over
anyway only after the checks below — every one of them degrades silently to an empty page.

**Several hubs at once:** each hub has its own `hub_id`, printed by `/api/v1/status`. A shared
port is not enough then — a hub started for another project swallows this project's errors. Point
`airbug-err` at the intended hub with `Options::new().endpoint("…/api/v1/errors").hub_id(uuid)`,
which posts to `/api/v1/errors/{uuid}` and makes a mismatched hub answer 404. For logs and
metrics, correlate instead of rerouting: `OTEL_RESOURCE_ATTRIBUTES="airbug.hub_id=<uuid>"`.

- Route the signals to it before starting what you measure:
  `export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318` (`otel/src/lib.rs:49`).
  That endpoint means HTTP, and HTTP is what `airbug-otel` uses even in an
  `--all-features` build; `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` is the only way to get
  gRPC, and then the endpoint must move to `:4317`. Mismatched pairs fail at
  shutdown with a bare `transport error`.
- Errors need no setup: `airbug-err` posts to `http://127.0.0.1:8790/api/v1/errors`
  by default (and Release Health sessions to `/api/v1/sessions`); point it elsewhere with
  `AIRBUG_ERR_ENDPOINT` (`err/src/lib.rs:129`). Standard Sentry SDKs (`sentry-rust`,
  `@sentry/browser`, `sentry-sdk`) can also use DSN `http://airbug@127.0.0.1:8790/1`
  (`POST /api/1/envelope/` and `/api/1/store/`).
- Local Aspire stack: `cargo airbug up` starts the hub + collector and launches `.airbug/stack.json`
  resources with automatic `OTEL_*`, `AIRBUG_*`, `AIRBUG_SENTRY_DSN`, and `SERVICES__<NAME>__HTTP`
  injection (`http://127.0.0.1:8790/#/resources`).
- Host metrics: `cargo run -p airbug-mon --release -- --otlp`, or
  `--headless --seconds N --otlp` without a GUI. mon itself is an egui window with no
  port — never link mon, link the hub.

Checks, in order:

1. `curl -s http://127.0.0.1:8790/api/v1/status` and read `.root`. One hub owns the
   port; a hub started for another project is a common state, and its error ingest then
   swallows this project's issues. Wrong root → start yours with `--port 8791` and give
   that URL; never assume the default port was free.
2. Is OTLP listening? `nc -z 127.0.0.1 4318`, or the `apis[]` entries in the status
   JSON. Dead collector means the traces, logs, metrics, and profiles panels stay empty
   forever. `--collector` tries the `docker compose` plugin, then a standalone
   `docker-compose` binary, then an `otelcol*` binary on `PATH`, and finally falls back
   to the built-in Rust OTLP HTTP (`:4318` + `:8790/v1/*`) & gRPC (`:4317`) receiver
   (`--collector-mode builtin` forces the built-in receiver directly;
   `dash/hub/src/collector.rs`). Read the `collector:` lines on stderr and verify `:4318`
   is open before claiming a live view.
3. Traces and OTLP Profiles render natively in the hub at `http://127.0.0.1:8790/#/traces`
   (Critical Path Waterfall, Service Map, RED metrics, zPages `/api/v1/zpages/tracez`) and
   `http://127.0.0.1:8790/#/profiles` (interactive Flamechart). Jaeger on
   `http://127.0.0.1:16686/` is optional and only brought up by the Docker path
   (`dash/README.md`). No Jaeger → link `#/traces` in the hub rather than `:16686`.
4. For progress in chat without a browser: `cargo run -p airbug-hub -- status --root .`
   prints the same snapshot as JSON.
5. `serve --collector` runs until SIGINT and tears the stack down on exit; start it in
   the background. Without `--collector` no handler is installed, and a background job
   started from a non-interactive shell inherits SIGINT as ignored — stop it with
   SIGTERM.

## Live tests and direct cargo bench

`airbug_report` records individual standard Rust tests in
`target/airbug-report/runs/<id>/run.json` while they execute. Run it with:

```sh
cargo run -p airbug --features json --bin airbug_report -- --all-features --locked --doc-tests
```

The hub's `/#/runs` tab displays test and benchmark launch history, case progress,
output, steps, comparisons and attachments. Verify the hub's root before handing
out its URL, as above. Test execution is sequential with one process per case;
duration includes process startup. Doctests are an aggregate suite.
`report.json` and `index.html` at the report root remain final CI snapshots.

Direct `cargo bench` targets using `Suite::main()` write the same live history at
case boundaries. `AIRBUG_DASHBOARD=0` disables those writes for strict measurements.
`AIRBUG_DASHBOARD_ROOT` overrides workspace discovery. `--list` and `--dry-run`
write no history.

## Tests are live in the hub: `cargo airbug test`

`cargo airbug` is the one entry point (`cli/`, `cargo-airbug`): it runs `test` itself
(`run/testrun`), routes `serve`, `up` and `status` to `airbug-hub` and `bench …` to
`cargo-airbug-bench` (`cargo airbug bench run …` is
`cargo airbug-bench run …`). `cargo airbug test` wraps `cargo test`, lists tests first
so progress has a denominator, and writes a run directory the hub reads while it runs.

```sh
cargo airbug test --junit junit.xml -- --workspace --exclude airbug-mon   # anything after -- goes to cargo test
```

- stderr prints `Live tests: http://127.0.0.1:8790/#/run/<run_id>` when a hub for this
  root is up (otherwise it says how to start one). Hand over that URL — it is live, per
  test, and keeps working after the run finishes.
- Files: `.airbug/runs/<run_id>/{manifest,progress,report}.json`, `junit.xml`, `output.log`, `events/`,
  and a self-contained `index.html`. `report.json` is `airbug.test-report/1`.
- The runner sets `AIRBUG_REPORT_DIR`, so `airbug::report` steps (`step`, `step_async`),
  TestOps taxonomy (`#[airbug::test(epic = ..., feature = ..., story = ..., severity = ..., owner = ...)]`),
  comparisons and attachments land on the right test (`unit/src/report.rs`), and defects are
  automatically classified (`product_bug`, `test_bug`, `infrastructure`, `flaky`, `quarantined`).
- Exit code: 0 all passed, cargo's code otherwise, 130 on Ctrl+C. `--out DIR` writes the
  run somewhere else (CI uses `target/airbug-report`); `--junit FILE` exports JUnit XML;
  `--quiet` hides cargo's output.
- JSON for agents: `GET /api/v1/runs?kind=test`, `GET /api/v1/runs/latest`,
  `GET /api/v1/runs/<id>/junit.xml`, and `GET /api/v1/tests/analytics` (per-test
  status, failure output, steps, defect classification, stability, slowest tests, behaviors).
- `--nextest` runs `cargo nextest run` instead (list via `nextest list --message-format
  json`, per-test results parsed from its output, retries → `flaky`). Same files, same UI.
- Rerun what failed: `POST /api/v1/runs/<id>/rerun` with JSON `{"failed": true}` (the
  "Rerun failed" button); `false` repeats the whole run. The detail's `rerun.command_failed`
  is the equivalent shell command — prefer running that yourself when you have a terminal.
- Bench regressions at a glance: `GET /api/v1/bench/compare` compares the newest finished
  bench run with the previous one that has the same cases and environment (Now page);
  `?run=<id>&baseline=<id>` compares a chosen pair (the run page's Compare tab).
- `airbug_report` also records live case history at `/#/runs` and final snapshots at `/report/`.

## Deeper docs

`bench/docs/REPORTS.md` (reports and the run page).
