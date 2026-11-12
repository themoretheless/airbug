---
name: live-bench
argument-hint: <program or scenario to benchmark>
description: Run a cargo airbug-bench benchmark and hand over the live run page in the airbug hub while it is still measuring, so the user watches progress update every 1.5s. Use for "run the benchmark and give me a link", "show me the bench running", or any headless/agent run. Covers starting the hub, capturing the link, progress polling and results.
---

# Live bench run page

## Overview

`cargo airbug-bench` has no web server of its own. Every measuring command (`run`, `matrix`,
`bench`, `git-compare`, `resume`) announces its session in `.airbug/runs/`; the airbug hub shows
it live. When a hub for the current directory is up, the command prints on stdout:

```text
Live benchmark: http://127.0.0.1:8790/#/run/bench-<ms>-<pid>
```

## Steps

1. Make sure a hub serves this directory: `curl -s http://127.0.0.1:8790/api/v1/status` and read
   `.root`. None → `cargo airbug serve --root . &`. Another root → start yours with
   `--port 8791` and set `AIRBUG_HUB=http://127.0.0.1:8791` for the bench.
2. Start the benchmark in the background, e.g.
   `cargo airbug-bench run --program ./target/release/mybench --output .airbug-bench/session`.
3. Read the `Live benchmark: <url>` line and give it to the user before reading any results.
   Without that line there is no live view — stderr says why. Never guess or reuse a link.
4. For progress in chat read `<output>/progress.json` (`completed`, `total`, `variant`).
5. The command exits when `status-final.json`, `run.json` and `report.html|json|md` are saved;
   there is no server to stop. The hub keeps serving the page afterwards.

## Notes

- The page: progress with the current variant and an ETA, the matrix grid, captured output,
  signals received during the run, comparison against a baseline.
- `AIRBUG_DASHBOARD=0` keeps the session out of the run store for strict measurements; say that
  accuracy was chosen over the live page.
- `--dry-run` writes nothing. `--ui`, `--no-ui`, `--no-open` are accepted and ignored.
- The bench is not on crates.io: in another project add `airbug-bench` as a git/path
  dev-dependency with a `[[bench]]` target (`harness = false`) and install the CLI with
  `cargo install --path bench/cli` from the airbug repository.
