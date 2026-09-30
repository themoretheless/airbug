---
name: live-bench
argument-hint: <program or scenario to benchmark>
description: Run a cargo airbug-bench benchmark and hand over the live dashboard URL while it is still measuring, so the user watches progress update every 1.5s. Use for "run the benchmark and give me a link", "show me the bench running", live progress page, or any headless/agent run where a TTY is absent and the interface would otherwise not start. Covers progress polling, result finalization and closing the leftover server.
---

# Live bench dashboard

## Overview

`cargo airbug-bench run` only starts its progress server when stdout is a terminal. Agents have no
TTY, so a plain run measures in silence. Two flags turn it on: `--ui` forces the interface, and
`--no-open` prints the URL instead of launching a browser. The bench then writes `<output>.live.json`
beside the run directory with `{"url": "...", "pid": 12345}` as soon as the port is bound.

No wrapper is needed — start the run in the background, read `<output>.live.json`, and poll the
server.

## If `cargo airbug-bench` is missing

This skill is global, but the bench lives in the airbug repo and `airbug-bench` is not on crates.io.
In a project that does not depend on it, `cargo airbug-bench` fails with `no such command`, and no
worker binary exists yet. Declare the bench in that project's `Cargo.toml`:

```toml
[[bench]]
name = "bench"
harness = false

[dev-dependencies]
airbug-bench = { git = "https://github.com/themoretheless/airbug", tag = "airbug-bench-v0.7.0" }

[package.metadata.airbug_bench]
targets = ["bench"]
```

Pin by `tag` or `rev`. `branch = "release"` moves: it carries 0.8.0 now, while the newest
`airbug-bench-v*` tag names 0.7.0. `cargo airbug-bench init` produces the same three tables plus a
starting `benches/bench.rs`, but points the dependency at a local path (the checkout the CLI was
built from), so run it first and then replace that line with the pinned form above.

The subcommand resolves from `PATH`, one time per machine:

```sh
cargo install --git https://github.com/themoretheless/airbug --branch main cargo-airbug-bench
```

Install the **runner** from `main`. The `release` branch and the `airbug-bench-v*` tags carry the
library version, and the runner built from those refs predates `<output>.live.json` — from it you get
the `Live benchmark:` line on stdout and nothing else. The package name goes **positionally**:
`--bin cargo-airbug-bench` stops with `multiple packages with binaries found: airbug, airbug-hub,
airbug-mon, bench-forma-example, cargo-airbug-bench`, and `--package` is not accepted as a flag for a
git source. Inside the airbug repo itself the command is wired up by `.cargo/config.toml`, so neither
step is needed there.

## Start a run and give the link

```sh
EXE=$(cargo bench --no-run --message-format=json 2>/dev/null | jq -r \
  'select(.reason=="compiler-artifact" and ((.target.kind|index("bench"))!=null) and (.executable!=null)) | .executable')
cargo airbug-bench run --program "$EXE" --repetitions 20 \
  --output .airbug-bench/myrun --ui --no-open -- <worker args>
```

Start it as a background task; the interface keeps its process alive after the results are
finalized, so a foreground call hangs until it is interrupted.

- `--ui` forces the server without a TTY; `--no-open` prints the URL instead of launching a
  browser. Both are bench flags, so they go **before** the bare `--` — after it they reach the
  measured program, which exits on an unknown argument and never publishes a link.
- `--output` names a directory that must not exist yet. Its sibling `<output>.live.json` appears
  as soon as the port is bound.
- As soon as `Live benchmark: http://127.0.0.1:<port>/<token>/` appears — from the task's log or
  from `<output>.live.json` — **send that URL to the user**, before reading any results. The page
  refills itself every 1.5 s and needs no refresh.
- Port and token are fresh per start, and a stale `<output>.live.json` from an earlier run points
  at a dead port. Never reuse a link from an earlier run and never build one by hand; take it from
  the current run only.
- The first start compiles, which can take minutes without printing a URL. An installed
  `cargo-airbug-bench` (or a prebuilt `./target/release/cargo-airbug-bench` inside the airbug
  checkout) removes that wait.
- A worker binary is not at `target/release/<name>`. `cargo bench` puts it under
  `target/release/build/<package>/<hash>/out/bench-<hash>`, so take the path from the build output as
  above rather than assembling it. `cargo airbug-bench build` builds without printing a path, and
  `bench` — the registered-target list — starts no live server at all; only `run` and `matrix` do.
- With `--hub` the run registers with airbug-hub first and prints `airbug dash: <url>` too; the
  hub link is the run view, the `Live benchmark:` line is the live progress page.

## Poll progress without blocking

```sh
curl -s "<url without its trailing slash>/api/live"    # {"state","completed","total","variant"}
cat .airbug-bench/myrun/progress.json                   # the same document, no server needed
```

- The recorded URL ends in `/`; appending `/api/live` to it makes a double slash the server
  answers with 400. Strip the trailing slash first.
- Progress in chat comes from these, not from the browser. `status-final.json` in the run
  directory means the measurement is over; `report.html`, `report.json`, `report.md` and
  `run.json` land with it.

## Close

```sh
kill -INT $(sed -n 's/.*"pid": \([0-9]*\).*/\1/p' .airbug-bench/myrun.live.json)
```

Artifacts are already on disk once `status-final.json` exists, so interrupting loses nothing.
Stop only after the user says they are done looking, and hand over `<output>/report.html` for the
finalized report.

## When not to promise a link

- `--no-ui` for timing-sensitive measurement — the browser and the server perturb the host. Say
  that accuracy was chosen over the dashboard.
- `--dry-run` starts no server and writes nothing.
- `git-compare` and `sessions`-driven flows call the runner repeatedly with no live UI attached.
  Offer `cargo airbug-bench serve <dir> --port 8787` afterwards instead of a link.
- `cargo airbug test` and `airbug_report` publish live test history through the hub; see
  `AGENTS.md` for the correct routes and root checks.
