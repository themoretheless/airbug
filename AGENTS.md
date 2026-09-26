# Agent instructions

## Benchmarks: always hand over the live dashboard link

`cargo airbug-bench run` can serve a progress page while it measures. Agents run
without a TTY, so the interface does **not** start by default — ask for it:

```sh
cargo airbug-bench run --program ./target/release/mybench \
  --output .airbug-bench/session --ui --no-open
```

- `--ui` forces the server; `--no-open` prints the URL instead of launching a browser.
- Run it in the background. After results are finalized the process keeps the server
  alive until SIGINT and never exits on its own
  (`bench/cli/src/cmd.rs:534`), so a foreground call hangs until timeout.
- Capture the line `Live benchmark: http://127.0.0.1:<port>/<key>/` from stdout and
  give it to the user as soon as it appears, before reading any results.
- Port and key are fresh per start. Never reuse a link from an earlier run; never
  guess one.
- The page refills itself every 1.5 s from `<output>/progress.json`
  (`bench/cli/src/web_ui.rs:197`); `<output>/status-final.json` plus
  `report.html` / `report.json` / `report.md` land after completion. Reading
  `progress.json` directly is fine for reporting progress in chat.
- Send SIGINT only after the user is done looking; artifacts are already on disk.

Exceptions worth stating out loud rather than silently skipping the link:

- `--no-ui` for strict/timing-sensitive measurements — the browser and server perturb
  the host. Say that you chose accuracy over the dashboard.
- `--dry-run` starts no server and writes nothing.
- `matrix`, `git-compare` and `sessions`-driven runs call the runner repeatedly with no
  live UI attached (`bench/cli/src/cmd.rs:510` is the only caller). Offer
  `cargo airbug-bench serve <dir> --port 8787` afterwards instead of promising a link.

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

## `airbug` (unit) has no live dashboard

`airbug` is a test-support library. Its one binary, `airbug_report`, blocks on
`cargo test --workspace` and writes the report only afterwards, so there is nothing to
attach a URL to while tests run. Do not invent one.

```sh
cargo run -p airbug --bin airbug_report -- --all-features --locked --doc-tests
```

- It writes `target/airbug-report/{report.json,index.html}` at the end
  (`unit/src/bin/airbug_report.rs:85`); CI does exactly this and uploads the folder.
- Hand over `http://127.0.0.1:8790/report/` — the hub serves that folder — and say it is
  a snapshot that appears only when the run finishes, not a live view.
- Per-test machine-readable events are written only when something exports
  `AIRBUG_REPORT_DIR` (`unit/src/report.rs:28`); the report binary does not set it.

## Deeper docs

`bench/docs/REPORTS.md` (web interface, live interface),
`bench/scripts/test-live-ui.py` (the stdout URL contract).
