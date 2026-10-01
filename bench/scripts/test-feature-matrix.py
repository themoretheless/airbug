#!/usr/bin/env python3
"""Check benchmark runtimes independently, without Cargo's default features."""
import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--offline", action="store_true")
args = parser.parse_args()
root = Path(__file__).resolve().parents[2]
configurations = [
    (None, ["executors", "thread_allocations"]),
    ("async-futures", ["executors"]),
    ("async-smol", ["executors"]),
    ("async-tokio", ["executors"]),
    ("memory", ["memory", "thread_allocations"]),
]
for feature, targets in configurations:
    command = ["cargo", "test", "--locked", "-p", "airbug-bench", "--no-default-features"]
    if args.offline:
        command.append("--offline")
    if feature:
        command += ["--features", feature]
    for target in targets:
        command += ["--test", target]
    print(f"Feature isolation: {feature or 'none'}", flush=True)
    subprocess.run(command, cwd=root, check=True)
# Exercise exact serial oracles with the default statistical worker feature both
# disabled and enabled independently of macros and optional async runtimes.
for feature in [None, "parallel-analysis"]:
    command = ["cargo", "test", "--locked", "-p", "airbug-bench", "--no-default-features", "--lib"]
    if args.offline:
        command.append("--offline")
    if feature:
        command += ["--features", feature]
    command.append("parallel")
    print(f"Statistical feature isolation: {feature or 'serial'}", flush=True)
    subprocess.run(command, cwd=root, check=True)
print("Independent runtime, allocation and statistical features passed")
