# Agent instructions

## Benchmarks: always hand over the live dashboard link

`cargo airbug-bench run` can serve a progress page while it measures. Agents run
without a TTY, so the interface does **not** start by default — ask for it:

```sh
cargo airbug-bench run --program ./target/release/mybench \
  --output .airbug-bench/session --ui --no-open
cargo airbug-bench matrix --plan matrix.json \
  --output .airbug-bench/matrix --ui --no-open
```

- `--ui` forces the server; `--no-open` prints the URL instead of launching a browser.
- Run it in the background. After results are finalized the process keeps the server
  alive until SIGINT and never exits on its own
  (`bench/cli/src/cmd.rs:565`), so a foreground call hangs until timeout.
- Capture the line `Live benchmark: http://127.0.0.1:<port>/<key>/` from stdout and
  give it to the user as soon as it appears, before reading any results.
- Port and key are fresh per start. Never reuse a link from an earlier run; never
  guess one.
- The page refills itself every 1.5 s from `<output>/progress.json`
  (`bench/cli/src/web-ui.html:558`); `<output>/status-final.json` plus
  `report.html` / `report.json` / `report.md` land after completion. Reading
  `progress.json` directly is fine for reporting progress in chat.
- Send SIGINT only after the user is done looking; artifacts are already on disk.

Exceptions worth stating out loud rather than silently skipping the link:

- `--no-ui` for strict/timing-sensitive measurements — the browser and server perturb
  the host. Say that you chose accuracy over the dashboard.
- `--dry-run` starts no server and writes nothing.
- `matrix --ui` serves one page for the whole session (`bench/cli/src/cmd.rs:68`). It
  aggregates the per-cell `<output>/<n>/progress.json` files on the server
  (`bench/cli/src/web_ui.rs:187`), so the counter is processes over every combination
  and each lane is labelled with its axis values (`--cpu 2 · candidate`). The report
  stays closed until all cells finish, and a failed or interrupted cell surfaces as the
  aggregate state. Cells hold `run.json` only — no per-cell reports; `report` on the
  matrix directory afterwards collects the cells.
- `bench` (one run per registered target), `git-compare` and `sessions`-driven runs
  call the runner with no live UI attached; only `run` and `matrix` start a server.
  Offer `cargo airbug-bench serve <dir> --port 8787` afterwards instead of promising a
  link.

## Traces, metrics, logs, errors: one hub, fixed address

```sh
cargo run -p airbug-hub -- serve --root . --collector
```

`http://127.0.0.1:8790/` is the page that fills itself in while things run: status every
15 s, OTLP logs every 4 s, metrics every 4 s, issues every 5 s
(`dash/hub/static/dashboard.js:720`). Unlike the bench live UI the port is fixed and
there is no token, so the link stays valid across restarts. Hand it over anyway only
after the checks below — every one of them degrades silently to an empty page.

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
  by default; point it elsewhere with `AIRBUG_ERR_ENDPOINT` (`err/src/lib.rs:129`).
- Host metrics: `cargo run -p airbug-mon --release -- --otlp`, or
  `--headless --seconds N --otlp` without a GUI. mon itself is an egui window with no
  port — never link mon, link the hub.

Checks, in order:

1. `curl -s http://127.0.0.1:8790/api/v1/status` and read `.root`. One hub owns the
   port; a hub started for another project is a common state, and its error ingest then
   swallows this project's issues. Wrong root → start yours with `--port 8791` and give
   that URL; never assume the default port was free.
2. Is OTLP listening? `nc -z 127.0.0.1 4318`, or the `apis[]` entries in the status
   JSON. Dead collector means the logs and metrics panels stay empty forever.
   `--collector` tries the `docker compose` plugin, then a standalone `docker-compose`
   binary, then an `otelcol*` binary on `PATH` (`dash/hub/src/collector.rs`); with none
   of them the collector does not start and hub carries on serving. A container that is
   *up* is not enough — read the `collector:` lines on stderr, which say so when
   `:4318` never opened. Report that instead of claiming a live view.
3. Traces are not in the hub at all — they render only in Jaeger on
   `http://127.0.0.1:16686/`, which the Docker path brings up and the otelcol-binary
   path does not (`dash/README.md:95`). No Jaeger → no trace UI, say so rather than
   linking a 404.
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
write no history. The separate process runner's live dashboard rules above still apply.

## Tests are live in the hub: `cargo airbug test`

`cargo airbug test` (alias for `airbug-hub test`) wraps `cargo test`, lists tests first
so progress has a denominator, and writes a run directory the hub reads while it runs.

```sh
cargo airbug test -- --workspace --exclude airbug-mon   # anything after -- goes to cargo test
```

- stderr prints `Live tests: http://127.0.0.1:8790/#/tests/<run_id>` when a hub for this
  root is up (otherwise it says how to start one). Hand over that URL — it is live, per
  test, and keeps working after the run finishes.
- Files: `.airbug/runs/<run_id>/{manifest,progress,report}.json`, `output.log`, `events/`,
  and a self-contained `index.html`. `report.json` is `airbug.test-report/1`.
- The runner sets `AIRBUG_REPORT_DIR`, so `airbug::report` steps, comparisons and
  attachments land on the right test (`unit/src/report.rs`).
- Exit code: 0 all passed, cargo's code otherwise, 130 on Ctrl+C. `--out DIR` writes the
  run somewhere else (CI uses `target/airbug-report`); `--quiet` hides cargo's output.
- JSON for agents: `GET /api/v1/runs?kind=test` and `GET /api/v1/runs/latest` (per-test
  status, failure output, steps, new failures vs the previous run, flaky tests).
- `--nextest` runs `cargo nextest run` instead (list via `nextest list --message-format
  json`, per-test results parsed from its output, retries → `flaky`). Same files, same UI.
- Rerun what failed: `POST /api/v1/runs/<id>/rerun` with JSON `{"failed": true}` (the
  "Rerun failed" button); `false` repeats the whole run. The detail's `rerun.command_failed`
  is the equivalent shell command — prefer running that yourself when you have a terminal.
- Bench regressions at a glance: `GET /api/v1/bench/compare` compares the newest finished
  bench run with the previous one that has the same cases and environment (Now page).
- `airbug_report` also records live case history at `/#/runs` and final snapshots at `/report/`.

## Deeper docs

`bench/docs/REPORTS.md` (web interface, live interface),
`bench/scripts/test-live-ui.py` (the stdout URL contract).
