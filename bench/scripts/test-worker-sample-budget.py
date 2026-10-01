#!/usr/bin/env python3
"""Check real Cargo forwarding of worker sample budgets and precedence."""
import json
import os
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
COMMAND = ["cargo", "bench", "--offline", "-p", "airbug-bench", "--bench", "attributed", "--"]
for unit, expected in [("workers", 2), ("batches", 5)]:
    for preview in [False, True]:
        env = dict(os.environ, AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0",
                   AIRBUG_BENCH_WORKER_START="invalid" if unit == "batches" else "local",
                   AIRBUG_BENCH_SAMPLE_COUNT_UNIT="invalid" if unit == "batches" else "workers")
        args = ["--filter", "collections/sum", "--threads", "3", "--samples", "5",
                "--iterations", "1", "--warmup-ms", "0", "--no-history"]
        if unit == "batches":
            args += ["--sample-count-unit", unit, "--worker-start", "shared"]
        args += ["--dry-run"] if preview else ["--json"]
        result = subprocess.run(COMMAND + args, cwd=ROOT, env=env, text=True, capture_output=True)
        assert result.returncode == 0, result.stdout + result.stderr
        try:
            payload = result.stdout.lstrip() if preview else next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT="))
            run, _ = json.JSONDecoder().raw_decode(payload)
        except ValueError as exc:
            raise AssertionError(result.stdout + result.stderr) from exc
        assert len(run["cases"]) == 1, run
        contract = run["cases"][0]["contract"]
        assert contract["sampling.count_unit"] == unit, contract
        assert contract["threads.timer_start"] == ("local" if unit == "workers" else "shared"), contract
        assert contract["sampling.requested_samples"] == "5", contract
        assert contract["samples"] == str(expected), contract
        if not preview:
            assert len(run["worker_timings"]) == expected * 3, run
            assert sum(w["operations"] for w in run["worker_timings"]) == expected * 3
print("cargo bench: worker budgets, full-batch rounding, preview and CLI/environment precedence passed")

# Dynamic counters travel with actual worker samples through Cargo and JSON.
env = dict(os.environ, AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0")
result = subprocess.run(COMMAND + ["--filter", "collections/sort_only/64", "--threads", "3",
    "--samples", "2", "--sample-count-unit", "batches", "--iterations", "65", "--warmup-ms", "0",
    "--worker-start", "local", "--no-history", "--json"], cwd=ROOT, env=env, text=True, capture_output=True)
assert result.returncode == 0, result.stdout + result.stderr
run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
assert len(run["cases"]) == 1, run
walls = [o for o in run["observations"] if o["metric"] == "wall"]
assert len(walls) == 2, run
for wall in walls:
    assert wall["work_totals"]["items"] == str(3 * 65 * 64), wall
    assert len(wall["worker_work_totals"]) == 3, wall
    assert all(totals["items"] == str(65 * 64) for totals in wall["worker_work_totals"].values()), wall
print("cargo bench: actual dynamic counters retained for each worker across waves")

for flag in ["--samples", "--iterations"]:
    for mode in ["--json", "--dry-run", "--test"]:
        env = dict(os.environ, AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0",
                   AIRBUG_TEST_NO_SETUP="1", AIRBUG_TEST_NO_EXECUTION="1")
        args = ["--filter", "collections/sum", flag, "0", "--include-ignored", "--no-history", mode]
        if mode == "--test":
            args += ["--json"]
        result = subprocess.run(COMMAND + args, cwd=ROOT, env=env, text=True, capture_output=True)
        assert result.returncode == 0, result.stdout + result.stderr
        payload = result.stdout.lstrip() if mode == "--dry-run" else next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT="))
        run, _ = json.JSONDecoder().raw_decode(payload)
        assert run["cases"] == [], run
        if mode != "--dry-run":
            assert run["observations"] == [], run
            assert json.loads(run["provenance"]["sampling.disabled_cases"]) == ["collections/sum"], run
print("cargo bench: zero sampling skips setup and execution in normal, smoke and preview modes")

for variable, flag in [("AIRBUG_BENCH_SAMPLES", "--samples"), ("AIRBUG_BENCH_ITERATIONS", "--iterations")]:
    env = dict(os.environ, AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0")
    env[variable] = "0"
    result = subprocess.run(COMMAND + ["--filter", "collections/sum", "--samples", "1", "--iterations", "1",
        "--warmup-ms", "0", "--no-history", "--json"], cwd=ROOT, env=env, text=True, capture_output=True)
    assert result.returncode == 0, result.stdout + result.stderr
    run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert len(run["observations"]) == 1 and run["observations"][0]["operations"] == 1, run
print("cargo bench: positive CLI values override disabled environment budgets")
