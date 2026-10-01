#!/usr/bin/env python3
"""Ordinary single-process Cargo runs expose exploratory baseline intervals."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0")
command = ["cargo", "bench", "--offline", "-p", "airbug-bench", "--bench", "attributed", "--", "collections/sum", "--exact", "--samples", "3", "--iterations", "2", "--warmup-ms", "0", "--items-count", "2", "--resamples", "32", "--no-plots"]
with tempfile.TemporaryDirectory(prefix="airbug-relative-cargo-") as directory:
    root = Path(directory)
    def run(name, *flags, lazy=False):
        current = dict(env)
        if lazy:
            current.update(AIRBUG_TEST_NO_SETUP="1", AIRBUG_TEST_NO_EXECUTION="1")
        output = root / name
        result = subprocess.run([*command, "--baseline-store", str(root / "baselines"), "--output", str(output), *flags], cwd=ROOT, env=current, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        return output, result.stdout
    run("first", "--save-baseline", "reference", "--json")
    output, text = run("second", "--baseline", "reference", "--json")
    assert "Relative bootstrap estimates" not in text
    relative = json.loads((output / "relative-distributions.json").read_text())[0]
    assert relative["resampling_unit"] == "normalized_observation"
    assert (relative["baseline_units"], relative["candidate_units"]) == (3, 3)
    assert relative["report"]["mean"]["interval_percent"] is not None
    assert len(relative["throughput"]) == 2
    document = (output / "regression-comparison.html").read_text()
    assert "batches may be autocorrelated" in document
    assert "<svg" not in document
    for layout in ["table", "tree"]:
        for quiet in [False, True]:
            output, text = run(f"{layout}-{'quiet' if quiet else 'text'}", "--load-baseline", "reference", "--baseline", "reference", "--output-format", layout, *(["--quiet"] if quiet else []), lazy=True)
            assert ("Relative bootstrap estimates" in text) == (not quiet), text
            assert ("└── collections" in text) == (layout == "tree"), text
            assert "batches may be autocorrelated" in (output / "regression-comparison.html").read_text()
    env["AIRBUG_BENCH_HISTORY"] = "1"
    automatic = ["--history-dir", str(root / "history"), "--hypothesis-resamples", "32", "--hypothesis-seed", "7"]
    run("history-first", *automatic, "--json")
    for quiet in [False, True]:
        output, text = run("history-quiet" if quiet else "history-text", *automatic, *(["--quiet"] if quiet else []))
        assert ("Relative bootstrap estimates" in text) == (not quiet)
        comparison = json.loads((output / "comparison.json").read_text())
        relative = comparison["cases"][0]["relative"][0]
        assert relative["resampling_unit"] == "normalized_observation"
        assert relative["report"]["config"]["resamples"] == 32
        assert "draws_percent" not in relative["report"]["mean"]
        assert "Relative bootstrap estimates" in (output / "report.html").read_text()
    output, text = run("history-quiet-tree", *automatic, "--quiet", "--output-format", "tree")
    assert "└── collections" in text
    assert "Relative bootstrap estimates" not in text
    assert "Previous run comparison" in text
    assert "Relative bootstrap estimates" in (output / "report.html").read_text()
    output, text = run("history-json", *automatic, "--json", "--output-format", "tree", "--color", "always")
    assert "└──" not in text and "\x1b" not in text
    for line in text.splitlines():
        assert line.startswith("BENCH_"), line
        json.loads(line.split("=", 1)[1])
    assert "Relative bootstrap estimates" not in text
    comparison = json.loads(next(line.removeprefix("BENCH_COMPARISON=") for line in text.splitlines() if line.startswith("BENCH_COMPARISON=")))
    assert comparison["cases"][0]["relative"][0]["report"]["mean"]["interval_percent"] is not None
print("Cargo named/automatic baselines: normalized batches, intervals, rates, no-plots HTML, quiet/JSON isolation, compact history and lazy reanalysis passed; functional checks only")
