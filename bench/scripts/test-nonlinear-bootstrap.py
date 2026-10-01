#!/usr/bin/env python3
"""Nonlinear formatter statistics through ordinary Cargo and saved baselines."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0",
           CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
with tempfile.TemporaryDirectory(prefix="airbug-nonlinear-bootstrap-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="nonlinear-bootstrap-fixture"
version="0.1.0"
edition="2021"
[workspace]
[dev-dependencies]
airbug-bench={{path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="custom"
harness=false
''')
    source = (ROOT / "bench/examples/custom_measurement.rs").read_text()
    source = source.replace("v / 3.0", 'if std::env::var_os("FOLD_DISPLAY").is_some() { (v - 4.0).powi(2) } else if std::env::var_os("NEGATIVE_DISPLAY").is_some() { -v } else if std::env::var_os("REVERSE_DISPLAY").is_some() { 100.0 - v } else if std::env::var_os("OFFSET_DISPLAY").is_some() { v - 100.0 } else { v * v }')
    source = source.replace('unit: "groups".into()', 'unit: if std::env::var_os("FOLD_DISPLAY").is_some() { "folded".into() } else if std::env::var_os("NEGATIVE_DISPLAY").is_some() { "negative<&>".into() } else if std::env::var_os("REVERSE_DISPLAY").is_some() { "remaining<&>".into() } else if std::env::var_os("OFFSET_DISPLAY").is_some() { "shifted<&>".into() } else { "squared<&>".into() }')
    source = source.replace("counter.set(counter.get() + 3);", '''assert!(std::env::var_os("NO_WORK").is_none());
        counter.set(counter.get() + std::env::var("WORK_STEP").ok().map(|v| v.parse::<u64>().unwrap()).unwrap_or(3));''')
    source = source.replace("    fn scale_throughputs(", """    fn scale_intervals(&self, _: f64, intervals: &[[f64; 2]]) -> Result<Option<airbug_bench::measurement::FormattedIntervals>> {
        if std::env::var_os("FOLD_DISPLAY").is_none() { return Ok(None); }
        Ok(Some(airbug_bench::measurement::FormattedIntervals {
            bounds: intervals.iter().map(|&[a, b]| {
                let left = (a - 4.).powi(2);
                let right = (b - 4.).powi(2);
                [if a <= 4. && b >= 4. { 0. } else { left.min(right) }, left.max(right)]
            }).collect(),
            unit: "folded".into(),
        }))
    }
    fn scale_throughputs(""")
    (root / "benches/custom.rs").write_text(source)
    base = ["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"),
            "--bench", "custom", "--", "--samples", "3", "--sample-ms", "1",
            "--warmup-ms", "0", "--sampling", "linear", "--resamples", "32", "--bootstrap-distributions",
            "--baseline-store", str(root / "baselines")]
    for name, flags, lazy in [
        ("first", ["--save-baseline", "saved"], False),
        ("loaded", ["--load-baseline", "saved", "--no-plots"], True),
        ("json-only", ["--load-baseline", "saved", "--no-html"], True),
        ("negative", ["--load-baseline", "saved"], True),
        ("reverse", ["--load-baseline", "saved"], True),
        ("offset", ["--load-baseline", "saved"], True),
        ("compared", ["--baseline", "saved"], False),
    ]:
        output = root / name
        result = subprocess.run([*base, "--output", str(output), *flags], cwd=ROOT,
                                env=dict(env, **({"NEGATIVE_DISPLAY": "1"} if name == "negative" else {}), **({"REVERSE_DISPLAY": "1"} if name == "reverse" else {}), **({"NO_WORK": "1"} if lazy else {}), **({"OFFSET_DISPLAY": "1"} if name == "offset" else {}), **({"WORK_STEP": "6"} if name == "compared" else {})),
                                capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        raw = json.loads((output / "estimates.json").read_text())
        shown = json.loads((output / "formatted-estimates.json").read_text())
        raw_row = next(r for r in raw["rows"] if r["metric"] == "work_units")
        row = shown["rows"][0]
        assert row["estimates"] == raw_row["estimates"]
        assert row["unit"] == raw_row["unit"] == "units"
        mean = next(p for p in shown["presentation"] if p["statistic"] == "mean")
        point = -3 if name == "negative" else 97 if name == "reverse" else -97 if name == "offset" else (36 if name == "compared" else 9)
        unit = "negative<&>" if name == "negative" else "remaining<&>" if name == "reverse" else "shifted<&>" if name == "offset" else "squared<&>"
        assert mean["estimate"]["point"] == point
        assert mean["unit"] == unit
        assert mean["estimate"]["standard_error"] == 0
        assert mean["draws"] == [point] * 32
        assert row["throughput"][0]["values"] == ([1, 1, 1] if name == "compared" else [2, 2, 2])
        assert not raw.get("presentation")
        if name == "json-only":
            assert not (output / "formatted-estimates.html").exists()
        else:
            html = (output / "formatted-estimates.html").read_text()
            assert "formatter(mean)" in html
            assert unit.replace("<", "&lt;").replace("&>", "&amp;&gt;") in html
            assert "statistical display unavailable" not in html
            assert ("<svg" in html) == (name in ("first", "offset", "reverse", "negative", "compared"))
            for svg in re.findall(r"<svg\b.*?</svg>", html, re.S):
                ET.fromstring(svg)
        if name == "compared":
            comparison = (output / "regression-comparison.html").read_text()
            assert "formatted regression comparison" in comparison
            assert "formatter(mean) comparison" in comparison
            assert "formatter(median) comparison" in comparison
            assert "baseline process 0" in comparison and "candidate process 0" in comparison
            assert row["regressions"] and row["regressions"][0]["samples"]
            assert "transformed total (squared&lt;&amp;&gt;)" in comparison
            # Saved candidate coordinates transform actual raw totals, independent of calibration.
            for regression in row["regressions"]:
                source = regression["samples"]
                displayed = regression["presentation"]["coordinates"]["samples"]
                assert displayed == [[s["operations"], s["total"] ** 2] for s in source]
            for svg in re.findall(r"<svg\b.*?</svg>", comparison, re.S):
                ET.fromstring(svg)
    single = list(base)
    single[single.index("--samples") + 1] = "1"
    for label, flags in [("single-old", ["--save-baseline", "single"]),
                         ("single-new", ["--baseline", "single"])]:
        result = subprocess.run([*single, *flags, "--output", str(root / label)],
                                cwd=ROOT, env=env, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
    single_html = (root / "single-new/regression-comparison.html").read_text()
    assert "transformed statistical comparison unavailable: insufficient bootstrap samples" in single_html
    # The CLI has no access to the fixture's formatter implementation.
    cli = ["cargo", "run", "--offline", "--locked", "-p", "cargo-airbug-bench", "--"]
    cli_env = dict(env, CARGO_TARGET_DIR=str(ROOT / "target"), NO_WORK="1")
    before = root / "first/run.json"
    after = root / "compared/run.json"
    for command in [
        ["compare", str(before), str(after), "--metric", "work_units", "--html"],
        ["report", str(after), "--baseline", str(before)],
    ]:
        target = root / (command[0] + "-restored.html")
        result = subprocess.run([*cli, *command, "--output", str(target)], cwd=ROOT, env=cli_env, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        html = target.read_text()
        assert "Saved formatter comparison" in html
        assert "formatted regression comparison" in html
        assert "transformed total (squared&lt;&amp;&gt;)" in html
        assert "formatter(mean) comparison" in html
        assert "formatter(median) comparison" in html
        assert "baseline process 0" in html and "candidate process 0" in html
        for svg in re.findall(r"<svg\b.*?</svg>", html, re.S):
            ET.fromstring(svg)
    mismatch = root / "different-settings.html"
    result = subprocess.run([*cli, "compare", str(before), str(after), "--html", "--hypothesis-resamples", "33", "--output", str(mismatch)], cwd=ROOT, env=cli_env, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "saved regression comparison unavailable: different bootstrap settings" in mismatch.read_text()
    assert "transformed total (squared&lt;&amp;&gt;)" not in mismatch.read_text()
    stale = json.loads(after.read_text())
    key = "airbug.presentation.regression-comparison.v1"
    snapshots = json.loads(stale["provenance"][key])
    assert snapshots
    snapshots[0]["candidate"] = "0" * 64
    stale["provenance"][key] = json.dumps(snapshots)
    stale_path = root / "stale.json"
    stale_path.write_text(json.dumps(stale))
    preserved = root / "preserved.html"
    preserved.write_text("previous report")
    result = subprocess.run([*cli, "compare", str(before), str(stale_path), "--html", "--output", str(preserved)], cwd=ROOT, env=cli_env, capture_output=True, text=True)
    assert result.returncode != 0
    assert "stale saved regression comparison snapshot" in result.stdout + result.stderr
    assert preserved.read_text() == "previous report"

    folded_output = root / "folded-comparison"
    result = subprocess.run([*base, "--baseline", "saved", "--output", str(folded_output)],
        cwd=ROOT, env=dict(env, FOLD_DISPLAY="1", WORK_STEP="6"), capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    folded = json.loads((folded_output / "formatted-estimates.json").read_text())
    fold_row = folded["rows"][0]
    assert next(p for p in folded["presentation"] if p["statistic"] == "mean")["estimate"]["point"] == 4
    assert fold_row["regressions"]
    for regression in fold_row["regressions"]:
        coordinates = regression["presentation"]["coordinates"]
        assert coordinates["curve"][0] == [0, 16]
        n = len(coordinates["curve"])
        bounds = regression["fit"]["slope"]
        for i, (x, _) in enumerate(coordinates["curve"]):
            a, b = x * bounds["lower"], x * bounds["upper"]
            endpoints = [(a - 4) ** 2, (b - 4) ** 2]
            low = 0 if a <= 4 <= b else min(endpoints)
            assert coordinates["band"][i] == [x, low]
            assert coordinates["band"][2 * n - i - 1] == [x, max(endpoints)]
    folded_restore = root / "folded-restored.html"
    result = subprocess.run([*cli, "compare", str(before), str(folded_output / "run.json"), "--metric", "work_units", "--html", "--output", str(folded_restore)],
        cwd=ROOT, env=cli_env, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    html = folded_restore.read_text()
    assert "transformed total (folded)" in html and "formatter(mean) comparison" in html
    for svg in re.findall(r"<svg\b.*?</svg>", html, re.S):
        ET.fromstring(svg)

    # Automatic history must publish the same enriched run to output and archive.
    history = root / "history"
    for name, step in [("history-first", "3"), ("history-second", "6")]:
        result = subprocess.run([*base, "--history-dir", str(history), "--hypothesis-resamples", "32", "--output", str(root / name)],
            cwd=ROOT, env=dict(env, AIRBUG_BENCH_HISTORY="1", WORK_STEP=step, FOLD_DISPLAY="1"), capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
    automatic_before = root / "history-first/run.json"
    automatic_after = root / "history-second/run.json"
    automatic = json.loads(automatic_after.read_text())
    assert json.loads(automatic["provenance"][key])
    archives = [json.loads(p.read_text()) for p in history.glob("*/*/run.json")]
    assert next(r for r in archives if r["id"] == automatic["id"]) == automatic
    assert "Saved formatter comparison" in (root / "history-second/report.html").read_text()
    auto_html = (root / "history-second/report.html").read_text()
    assert "formatter(mean) comparison" in auto_html
    assert "formatter(median) comparison" in auto_html
    restored_auto = root / "restored-history.html"
    result = subprocess.run([*cli, "compare", str(automatic_before), str(automatic_after), "--metric", "work_units", "--html", "--output", str(restored_auto)],
        cwd=ROOT, env=cli_env, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "Saved formatter comparison" in restored_auto.read_text()
    assert "formatter(mean) comparison" in restored_auto.read_text()
    assert "transformed total (folded)" in restored_auto.read_text()

print("Cargo nonlinear bootstrap: raw statistics preserved, transformed point/draws/SE, custom throughput, escaped HTML/SVG, lazy baseline, named-baseline and restored standalone regression comparison, nonmonotone interval bounds, folded automatic-history archive/output equivalence, stale/config guards, no-plots and no-html passed")
