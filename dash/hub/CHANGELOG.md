# Changelog

## Unreleased

- Add Test & bench runs: live case counts, launch history, filters, output, nested steps, comparisons and safe attachment downloads from existing Airbug diagnostics.

- `airbug-hub test` (`cargo airbug test`): wraps `cargo test`, lists tests first for a real
  progress bar, and writes a live run to `.airbug/runs/<run_id>/` (manifest, progress,
  `airbug.test-report/1` report, output log, `airbug::report` events, self-contained
  `index.html`). Crashed test binaries mark their unfinished tests `not_run`.
- `GET /api/v1/runs` — one timeline for test and bench runs; `GET /api/v1/runs/:id|latest`
  — per-test detail with new failures / fixed / removed vs the previous run, status history
  and flaky tests; `GET /runs/:id/*` serves run files.
- `airbug-hub test --nextest`: runs `cargo nextest run` and parses its list JSON and status
  lines into the same report (retries → flaky). `--run-id`, `--rerun-of` for linked runs.
- `POST /api/v1/runs/:id/rerun` and the "Rerun failed" / "Run again" buttons; test run
  detail gains `rerun` (supported, failed count, equivalent commands).
- `GET /api/v1/bench/compare` and a Now card: newest bench run vs the previous comparable
  run (or baseline vs candidate), via `airbug_bench::analysis::compare`.
- Dashboard reorganised around runs: Now · Runs · Tests · Bench · Issues · Logs · Metrics ·
  System. Old `#/overview`, `#/unit`, `#/mon`… routes redirect. No external fonts.

## 0.6.1

- Issue occurrence history (`issue_events`, capped); detail returns `events`
- Smoke coverage for resolve / ignore / reopen

## 0.6.0

- Initial family-aligned 0.6.0 release
