#!/usr/bin/env python3
"""Exercise numeric harness option domains without running benchmark bodies."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
from decimal import Decimal, getcontext

getcontext().prec = 80

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-domains-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="option-domains"
version="0.1.0"
edition="2021"
[workspace]
[dev-dependencies]
airbug-bench={{path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="domains"
harness=false
''')
    (root / "benches/domains.rs").write_text('''#[airbug_bench::suite]
mod cases { #[bench] fn never() { panic!("domain preview executed workload"); } }
''')
    env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
    env.update(CARGO_TARGET_DIR=str(ROOT / "target/parity-external"), AIRBUG_DASHBOARD="1")
    built = subprocess.run(["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "domains", "--no-run", "--message-format=json"], cwd=root, env=env, capture_output=True, text=True, check=True)
    executable = next(item["executable"] for line in built.stdout.splitlines() if (item := json.loads(line)).get("executable"))
    integer_max = "18446744073709551615"
    integer_overflow = "18446744073709551616"
    cases = {
        "--samples": (["0", "1", "4294967296", integer_max], ["-1", integer_overflow, "NaN"]),
        "--iterations": (["0", "1", "4294967296", integer_max], ["-1", integer_overflow]),
        "--resamples": (["1", "1000001"], ["0", "-1", integer_overflow]),
        "--hypothesis-resamples": (["1", "1000001"], ["0", "-1", integer_overflow]),
        "--confidence-level": (["5e-324", "0.95", "0.9999999999999999"], ["0", "1", "NaN", "inf", "-1"]),
        "--significance-level": (["5e-324", "0.05", "0.9999999999999999"], ["0", "1", "NaN", "inf", "-1"]),
        "--noise-threshold-percent": (["0", "250", "1.7976931348623157e308"], ["-1", "NaN", "inf"]),
        "--measurement-ms": (["0.000001", "123456789.125"], ["0", "-1", "NaN", "inf", "0.0000001"]),
        "--warmup-ms": (["0", "0.000001", "123456789.125"], ["-1", "NaN", "inf", "0.0000001"]),
    }
    duration_max = "18446744073709551615999.999999"
    duration_overflow = "18446744073709551616000"
    for option in ("--measurement-ms", "--warmup-ms"):
        cases[option][0].append(duration_max)
        cases[option][1].append(duration_overflow)
    for option in ("--min-time-ms", "--max-time-ms"):
        cases[option] = (["0", "0.000001", duration_max], ["-1", "NaN", "inf", "0.0000001", duration_overflow])
    cases["--sample-ms"] = (["0.000001", duration_max], ["0", "-1", "NaN", "inf", "0.0000001", duration_overflow])
    for unit in ("bytes", "items", "chars", "cycles", "bits"):
        cases[f"--{unit}-count"] = (["0", "1", integer_max], ["-1", integer_overflow, "NaN"])
    checked = 0
    for option, (valid, invalid) in cases.items():
        for expected, values in ((True, valid), (False, invalid)):
            for value in values:
                result = subprocess.run([executable, "--bench", "--dry-run", "--json", option, value], cwd=root, env=env, capture_output=True, text=True, timeout=120)
                assert (result.returncode == 0) == expected, (option, value, expected, result.stdout, result.stderr)
                assert "domain preview executed workload" not in result.stdout + result.stderr
                if expected:
                    plan = json.loads(result.stdout)
                    if option == "--max-time-ms" and value == "0":
                        assert plan["cases"] == [], "zero maximum must disable selection"
                    if option == "--samples":
                        assert plan["samples"] == int(value)
                    for case in plan["cases"]:
                        contract = case["contract"]
                        if option == "--iterations":
                            assert contract["sampling.iterations"] == value
                        if option.endswith("-count"):
                            unit = option.removeprefix("--").removesuffix("-count")
                            assert contract[f"work.counter.{unit}"] == value
                        time_key = {"--warmup-ms": "warmup_ns", "--sample-ms": "sample_target_ns", "--measurement-ms": "sampling.measurement_target_ns", "--min-time-ms": "sampling.min_time_ns", "--max-time-ms": "sampling.max_time_ns"}.get(option)
                        if time_key:
                            assert int(contract[time_key]) == int(Decimal(value) * 1_000_000)
                checked += 1
    for key, option in [("AIRBUG_BENCH_SAMPLES", "--samples"), ("AIRBUG_BENCH_ITERATIONS", "--iterations"), ("AIRBUG_BENCH_WARMUP_MS", "--warmup-ms")]:
        invalid_env = dict(env, **{key: "invalid"})
        rejected = subprocess.run([executable, "--bench", "--dry-run"], cwd=root, env=invalid_env, capture_output=True, text=True)
        assert rejected.returncode != 0
        overridden = subprocess.run([executable, "--bench", "--dry-run", option, "1"], cwd=root, env=invalid_env, capture_output=True, text=True)
        assert overridden.returncode == 0, overridden.stderr
        checked += 2
    modes = {
        "--sampling": ["flat", "linear", "auto"],
        "--sample-count-unit": ["batches", "workers"],
        "--worker-start": ["shared", "local"],
        "--timer": ["os", "cpu"],
        "--overhead": ["raw", "subtract"],
        "--sort": ["registration", "lexical", "natural", "source", "kind"],
        "--output-format": ["table", "tree"],
        "--bytes-format": ["decimal", "binary"],
        "--color": ["auto", "always", "never"],
        "--profile": ["quick", "normal", "thorough"],
    }
    def preview(flags, expected, diagnostic=None):
        result = subprocess.run([executable, "--bench", "--dry-run", "--json", *flags], cwd=root, env=env, capture_output=True, text=True, timeout=120)
        assert (result.returncode == 0) == expected, (flags, result.stdout, result.stderr)
        assert "domain preview executed workload" not in result.stdout + result.stderr
        if diagnostic:
            assert diagnostic in result.stderr, (flags, result.stderr)
    for option, values in modes.items():
        for value in values:
            preview([option, value], True)
            checked += 1
        preview([option, "invalid"], False)
        preview([option], False)
        checked += 2
    output = str(root / "unused-output")
    for flags in [
        ["--quiet", "--verbose"], ["--plots", "--no-plots"],
        ["--exact", "--glob"], ["--exact", "--regex"], ["--glob", "--regex"],
        ["--sample-ms", "1", "--measurement-ms", "1"],
        ["--no-bootstrap", "--resamples", "1"],
        ["--format", "terse"], ["--list", "--format", "terse"],
        ["--summary-estimator", "mean"],
        ["--no-plots", "--output", output, "--summary-parameter", "size"],
        ["--discard", "--output", output],
    ]:
        preview(flags, False)
        checked += 1
    for estimator in ("mean", "process-median"):
        preview(["--output", output, "--summary-parameter", "size", "--summary-estimator", estimator], True)
        checked += 1
    for scale in ("linear", "logarithmic"):
        preview(["--output", output, "--summary-scale", scale], True)
        checked += 1
    for flags in [
        ["--no-html", "--plots"], ["--plots", "--no-html"],
        ["--no-html", "--output", output, "--summary-parameter", "size"],
        ["--no-html", "--output", output, "--summary-scale", "linear"],
    ]:
        preview(flags, False, "--no-html")
        checked += 1
    preview(["--no-html", "--no-plots"], True)
    checked += 1
    baseline_flags = ["--save-baseline", "--replace-baseline", "--retain-baseline", "--baseline", "--baseline-lenient", "--load-baseline"]
    store = str(root / "missing-store")
    for option in baseline_flags:
        for name in ("main", "release-2026_10"):
            preview([option, name, "--baseline-store", store], True)
            checked += 1
        for name in ("", "../escape", "a/b", "a\\b", "with space", "name.json", "Кейс"):
            preview([option, name, "--baseline-store", store], False, "baseline name")
            checked += 1
    for value in ("0.000001", "1", "1000"):
        preview(["--profile-time-ms", value], True)
        checked += 1
    for value in ("0", "-1", "NaN", "inf", duration_max):
        preview(["--profile-time-ms", value], False)
        checked += 1
    for extra in (["--test"], ["--resamples", "1"], ["--quick"], ["--overhead", "subtract"], ["--bytes-format", "binary"], ["--save-baseline", "main"], ["--load-baseline", "main"], ["--baseline", "main"], ["--discard"]):
        preview(["--profile-time-ms", "1", *extra], False)
        checked += 1
    for option in ("--baseline", "--load-baseline"):
        result = subprocess.run([executable, "--bench", option, "missing", "--baseline-store", store], cwd=root, env=env, capture_output=True, text=True, timeout=120)
        assert result.returncode != 0, result.stdout
        assert "domain preview executed workload" not in result.stdout + result.stderr
        checked += 1
    assert not Path(store).exists()
    assert not Path(output).exists()
    assert not (root / "target").exists()
    assert not (root / ".airbug").exists()
    assert not (root / ".bench").exists()
print(f"Option domains: {checked} boundary/precedence checks passed; no workload or history writes")
