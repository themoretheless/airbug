#!/usr/bin/env python3
"""Multiple positive filters, exclusions, lazy validation, and exact execution."""
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
base = ["cargo", "bench", "--offline", "-p", "airbug-bench", "--bench", "attributed", "--"]
env = {k:v for k,v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0")
def call(flags, lazy=True, success=True):
    current = dict(env)
    if lazy:
        current.update(AIRBUG_TEST_NO_SETUP="1", AIRBUG_TEST_NO_EXECUTION="1")
    result = subprocess.run([*base, *flags], cwd=ROOT, env=current, capture_output=True, text=True)
    assert (result.returncode == 0) == success, result.stdout+result.stderr
    return result.stdout+result.stderr

expected = {"collections/sort", "collections/sum"}
for filters in [
    ["collections/sort", "collections/sum", "--exact"],
    ["--filter", "collections/sort", "--filter", "collections/sum", "--exact"],
    ["collections/sort", "--filter", "collections/sum", "--exact"],
    ["^collections/sort$", "^collections/sum$", "--regex"],
    ["collections/sor?", "collections/su?", "--glob"],
]:
    listed = call([*filters,"--list"])
    assert {s for s in listed.splitlines() if s.startswith("collections/")} == expected
    preview = json.JSONDecoder().raw_decode(call([*filters,"--dry-run"]).lstrip())[0]
    assert {c["id"] for c in preview["cases"]} == expected
    excluded = call([*filters,"--exclude-exact","collections/sort","--list"])
    assert {s for s in excluded.splitlines() if s.startswith("collections/")} == {"collections/sum"}
assert "invalid benchmark regex" in call(["valid", "[", "--regex", "--list"], success=False)
for mode, operations in [(["--test"],1), (["--samples","1","--iterations","2","--warmup-ms","0"],2)]:
    text = call(["collections/sort","collections/sum","collections/sort","--exact",*mode,"--json"],lazy=False)
    result = json.loads(next(s.removeprefix("BENCH_RESULT=") for s in text.splitlines() if s.startswith("BENCH_RESULT=")))
    assert {c["id"] for c in result["cases"]} == expected
    assert len(result["observations"]) == 2
    assert all(o["operations"] == operations for o in result["observations"])
print("Selection: positional/repeated/mixed filters, exact/glob/regex union, exclusions, lazy validation and single execution passed")

# A typo must explain how to recover without invoking setup or producing artifacts.
from tempfile import TemporaryDirectory
with TemporaryDirectory() as directory:
    output = Path(directory) / "not-created"
    message = call(["definitely_missing_case", "--output", str(output)], success=False)
    assert "no benchmarks match this selection" in message
    assert "\nRegistered cases (including ignored):\n  " in message
    assert 'Message("' not in message
    assert "cargo bench -- --list" in message
    assert "collections/sort" in message
    assert not output.exists()
assert "collections/" not in call(["definitely_missing_case", "--list"])
assert json.JSONDecoder().raw_decode(call(["definitely_missing_case", "--dry-run"]).lstrip())[0]["cases"] == []
print("Empty selection: actionable error, no setup or artifacts, list/dry-run remain valid")

# Help stays concise by default, with a discoverable complete reference.
for option in ["-h", "--help"]:
    text = call([option])
    assert "--help-all" in text and "--quick" in text
    assert "All options:" not in text
full = call(["--help-all", "--threads"])
assert "All options:" in full and "--quick-min-ms" in full
print("Help: concise defaults and complete reference, without setup or execution")

with TemporaryDirectory() as directory:
    root = Path(directory)
    flags = ["collections/sum", "--exact", "--samples", "3", "--iterations", "2", "--warmup-ms", "0", "--no-plots"]
    call([*flags, "--output", str(root / "default")], lazy=False)
    report = json.loads((root / "default/estimates.json").read_text())
    assert report["config"] == {"resamples": 10000, "confidence_level": 0.95, "seed": 0}
    assert report["rows"] and report["rows"][0]["estimates"]["mean"] is not None
    assert (root / "default/estimates.html").exists()
    assert "estimates.html" in (root / "default/report.html").read_text()
    call([*flags, "--no-bootstrap", "--output", str(root / "disabled")], lazy=False)
    assert (root / "disabled/run.json").exists()
    assert not (root / "disabled/estimates.json").exists()
    assert "estimates.html" not in (root / "disabled/report.html").read_text()
    for options in [["--no-bootstrap", "--resamples", "4"], ["--resamples", "4", "--no-bootstrap"], ["--no-bootstrap", "--bootstrap-distributions"]]:
        assert "--no-bootstrap cannot combine" in call([*flags, *options, "--json"], success=False)
print("Default statistics: confidence intervals and linked artifacts; explicit opt-out and conflicts before work")
