#!/usr/bin/env python3
"""Exercise Cargo's real argument forwarding and the attribute-generated harness."""
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
COMMAND = [
    "cargo", "bench", "-p", "airbug-bench", "--features", "macros",
    "--bench", "attributed", "--offline",
]


def run(*args, success=True):
    result = subprocess.run(
        [*COMMAND, *args], cwd=ROOT, capture_output=True, text=True,
    )
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result.stdout + result.stderr


listed = run("--", "--list")
assert "collections/sort" in listed and "collections/sum" in listed, listed
assert "collections/disabled" not in listed, listed

filtered = run("collections/sum", "--", "--exact", "--list")
assert "collections/sum" in filtered and "collections/sort" not in filtered, filtered

missing = run("no-such-case", "--", "--list")
assert "collections/" not in missing, missing

measured = run("collections/sum", "--", "--exact", "--profile", "quick", "--json")
assert "BENCH_RESULT=" in measured and "collections/sum" in measured, measured
assert "collections/sort" not in measured, measured

duplicate = run("sort", "--", "--filter", "sum", success=False)
assert "filter specified twice" in duplicate, duplicate

unknown = run("--", "--unknown-option", success=False)
assert "unknown argument" in unknown, unknown

print("cargo bench: listing, filtering, measurement and argument errors passed")
