# Changelog

## Unreleased

- `airbug-hub test` (`cargo airbug test`): wraps `cargo test`, lists tests first for a real
  progress bar, and writes a live run to `.airbug/runs/<run_id>/` (manifest, progress,
  `airbug.test-report/1` report, output log, `airbug::report` events, self-contained
  `index.html`). Crashed test binaries mark their unfinished tests `not_run`.
- `GET /api/v1/runs` — one timeline for test and bench runs; `GET /api/v1/runs/:id|latest`
  — per-test detail with new failures / fixed / removed vs the previous run, status history
  and flaky tests; `GET /runs/:id/*` serves run files.
- Dashboard reorganised around runs: Now · Runs · Tests · Bench · Issues · Logs · Metrics ·
  System. Old `#/overview`, `#/unit`, `#/mon`… routes redirect. No external fonts.

## 0.6.1

- Issue occurrence history (`issue_events`, capped); detail returns `events`
- Smoke coverage for resolve / ignore / reopen

## 0.6.0

- Initial family-aligned 0.6.0 release
