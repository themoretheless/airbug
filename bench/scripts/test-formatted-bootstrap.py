#!/usr/bin/env python3
"""Custom throughput confidence intervals through Cargo and saved baselines."""
import csv
import json
import os
import re
import xml.etree.ElementTree as ET
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
with tempfile.TemporaryDirectory(prefix="airbug-formatted-bootstrap-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="formatted-bootstrap-fixture"
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
    source = source.replace('unit: "groups".into()', 'unit: "groups<&>".into()')
    source = source.replace("    fn scale_for_machines(", '''    fn format_throughput(&self, value: f64, _: &str) -> Result<Option<String>> {
        Ok(Some(format!("<rate {value:.2}>")))
    }
    fn scale_for_machines(''')
    source = source.replace("        counter.set(counter.get() + 3);", '''        assert!(std::env::var_os("NO_WORK").is_none());
        counter.set(counter.get() + std::env::var("WORK_STEP").ok().map(|value| value.parse::<u64>().unwrap()).unwrap_or(3));''')
    source = source.replace('    suite.work_units("items", 6);', '    suite.parameter("size", 1);\n    suite.work_units("items", 6);')
    (root / "benches/custom.rs").write_text(source)
    base = ["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "custom", "--", "--samples", "3", "--iterations", "2", "--warmup-ms", "0", "--resamples", "32", "--baseline-store", str(root / "baselines"), "--no-plots"]
    def run(name, flags, lazy=False, plots=False, step=None):
        invocation = [arg for arg in base if not (plots and arg == "--no-plots")]
        if plots and "--no-html" not in flags:
            invocation += ["--summary-parameter", "size", "--summary-estimator", "mean"]
        result = subprocess.run([*invocation, "--output", str(root / name), *flags], cwd=ROOT, env=dict(env, **({"NO_WORK": "1"} if lazy else {}), **({"WORK_STEP": str(step)} if step else {})), capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        return root / name
    first = run("first", ["--save-baseline", "saved"])
    original = (first / "run.json").read_bytes()
    # New registration overrides must not rewrite the saved measurement's work contract.
    for name, flags, plots in [("first", None, False), ("loaded", ["--load-baseline", "saved", "--items-count", "12"], False), ("plots", ["--load-baseline", "saved"], True)]:
        output = first if flags is None else run(name, flags, lazy=True, plots=plots)
        saved = json.loads((output / "run.json").read_text())
        rows = list(csv.DictReader((output / "formatted.csv").open()))
        assert rows and all(row["operations"] == "2" for row in rows), rows
        for row in rows:
            raw_observation = next(o for o in saved["observations"] if o["case"] == row["case"] and o["metric"] == row["metric"] and str(o["process"]) == row["process"] and str(o["sequence"]) == row["sequence"] and o["variant"] == row["variant"])
            assert row["raw_value"] == raw_observation["value"]
            case = next(c for c in saved["cases"] if c["id"] == row["case"])
            assert json.loads(row["case_contract"]) == case["contract"]
            assert row["normalized_per_operation"] == "true"
            assert json.loads(row["case_contract"])["work.counter.items"] == "6"
        formatted = json.loads((output / "formatted-estimates.json").read_text())["rows"][0]
        raw = next(row for row in json.loads((output / "estimates.json").read_text())["rows"] if row["metric"] == "work_units")
        assert raw["estimates"]["mean"]["point"] == 3
        assert formatted["estimates"]["mean"]["point"] == 1
        assert raw["work_counters"] == {"items": 6}
        assert len(formatted["throughput"]) == 2
        for rate in formatted["throughput"]:
            assert rate["unit"] == "items/unit"
            assert rate["values"] == [2, 2, 2]
            assert rate["display"] == ["<rate 2.00>"] * 3
        html = (output / "formatted-estimates.html").read_text()
        assert "&lt;rate 2.00&gt;" in html and "<rate 2.00>" not in html
        assert ("<svg" in html) == plots
        if plots:
            summary_html = (output / "report.html").read_text()
            assert "size vs work_units (groups&lt;&amp;&gt;" in summary_html
            assert "x = 1, y = 1" in summary_html
            assert "work_units / items throughput (items/unit)" in summary_html
            assert "x = 1, y = 2" in summary_html
            restored_path = output / "restored-summary.html"
            restored = subprocess.run([
                "cargo", "run", "--offline", "--locked", "-p", "cargo-airbug-bench", "--",
                "report", str(output / "run.json"), "--summary-parameter", "size",
                "--summary-estimator", "mean", "--output", str(restored_path),
            ], cwd=ROOT, env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target")), capture_output=True, text=True)
            assert restored.returncode == 0, restored.stdout + restored.stderr
            restored_html = restored_path.read_text()
            assert "size vs work_units (groups&lt;&amp;&gt;" in restored_html
            assert "x = 1, y = 1" in restored_html
            assert "work_units / items throughput (items/unit)" in restored_html
            assert "x = 1, y = 2" in restored_html

    compared = run("compared", ["--baseline", "saved"], step=6)
    relative = json.loads((compared / "relative-distributions.json").read_text())
    custom = next(row for row in relative if row["metric"] == "work_units")
    assert custom["resampling_unit"] == "normalized_observation"
    assert len(custom["throughput"]) == 2
    for rate in custom["throughput"]:
        assert rate["point_percent"] == -50 and rate["interval_percent"] == [-50, -50], rate
        assert rate["unit"] == "items/unit"
        assert "formatter applied" in rate["method"]
    # Restore custom summaries alongside a baseline, on both axis scales.
    # Neither standalone report invocation may execute the measured workload.
    plotted_comparison = run("compared-plots", ["--baseline", "saved"], plots=True, step=6)
    for scale in ("linear", "logarithmic"):
        report_path = plotted_comparison / f"baseline-{scale}.html"
        result = subprocess.run([
            "cargo", "run", "--offline", "--locked", "-p", "cargo-airbug-bench", "--",
            "report", str(plotted_comparison / "run.json"), "--baseline", str(first / "run.json"),
            "--summary-parameter", "size", "--summary-estimator", "mean",
            "--summary-scale", scale, "--output", str(report_path),
        ], cwd=ROOT, env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target"), NO_WORK="1"), capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        rendered = report_path.read_text()
        assert "size vs work_units (groups&lt;&amp;&gt;" in rendered
        assert "x = 1, y = 2" in rendered
        assert "work_units / items throughput (items/unit)" in rendered
        assert "x = 1, y = 1" in rendered
        assert "throughput mean (items/unit)" in rendered and "-50.000000" in rendered
        assert "different bootstrap settings" not in rendered
    for extension, flags in (("md", []), ("html", ["--no-plots"])):
        report_path = plotted_comparison / f"numerical.{extension}"
        result = subprocess.run([
            "cargo", "run", "--offline", "--locked", "-p", "cargo-airbug-bench", "--",
            "report", str(plotted_comparison / "run.json"), "--baseline", str(first / "run.json"),
            "--output", str(report_path), *flags,
        ], cwd=ROOT, env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target"), NO_WORK="1"), capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        numerical = report_path.read_text()
        assert "throughput mean (items/unit)" in numerical and "-50.000000" in numerical
        assert "<svg" not in numerical

    # Invalid saved summary metadata must not replace an existing report.
    stale = json.loads((plotted_comparison / "run.json").read_text())
    snapshot_key = "airbug.presentation.parameter-summary.v1"
    snapshot = json.loads(stale["provenance"][snapshot_key])
    snapshot["source"] = "0" * 64
    stale["provenance"][snapshot_key] = json.dumps(snapshot)
    stale_path = plotted_comparison / "stale.json"
    stale_path.write_text(json.dumps(stale))
    preserved_path = plotted_comparison / "preserved.html"
    preserved_path.write_text("previous report")
    result = subprocess.run([
        "cargo", "run", "--offline", "--locked", "-p", "cargo-airbug-bench", "--",
        "report", str(stale_path), "--summary-parameter", "size",
        "--summary-estimator", "mean", "--output", str(preserved_path),
    ], cwd=ROOT, env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target"), NO_WORK="1"), capture_output=True, text=True)
    assert result.returncode != 0, result.stdout + result.stderr
    assert "stale parameter-summary" in result.stderr, result.stderr
    assert preserved_path.read_text() == "previous report"
    # Saved-run filtering preserves the custom raw metric comparison. These
    # CLI paths cannot execute the formatter code registered in the bench binary.
    compare_base = [
        "cargo", "run", "--offline", "--locked", "-p", "cargo-airbug-bench", "--",
        "compare", str(first / "run.json"), str(compared / "run.json"),
        "--metric", "work_units", "--hypothesis-resamples", "32",
    ]
    for mode, flags in (("json", ["--json", "--relative-distributions"]),
                        ("html", ["--html"]), ("plain-html", ["--html", "--no-plots"])):
        result = subprocess.run([*compare_base, *flags], cwd=ROOT,
            env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target"), NO_WORK="1"), capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        if mode == "json":
            data = json.loads(result.stdout)
            assert len(data["relative"]) == 1
            relative_row = data["relative"][0]
            assert relative_row["metric"] == "work_units"
            assert relative_row["report"]["mean"]["point_percent"] == 100
            assert len(relative_row["throughput"]) == 2
            assert all(rate["point_percent"] == -50 for rate in relative_row["throughput"])
        else:
            assert "work_units" in result.stdout
            assert ("<svg" in result.stdout) == (mode == "html")
            (compared / f"standalone-{mode}.html").write_text(result.stdout)
    default_comparison = subprocess.run([*compare_base[:-2], "--json", "--relative-distributions"], cwd=ROOT,
        env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target")), capture_output=True, text=True)
    assert default_comparison.returncode == 0, default_comparison.stdout + default_comparison.stderr
    restored_default = json.loads(default_comparison.stdout)["relative"][0]
    assert restored_default["report"]["config"]["resamples"] == 32
    assert all(rate["point_percent"] == -50 for rate in restored_default["throughput"])
    changed_settings = subprocess.run([*compare_base[:-1], "33", "--json", "--relative-distributions"], cwd=ROOT,
        env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target")), capture_output=True, text=True)
    assert changed_settings.returncode == 0, changed_settings.stdout + changed_settings.stderr
    unavailable = json.loads(changed_settings.stdout)["relative"][0]["throughput"]
    assert len(unavailable) == 2
    assert all(rate["point_percent"] is None and "different bootstrap settings" in rate["unavailable_reason"] for rate in unavailable)
    def saved_compare(before, after, extra=()):
        return subprocess.run([
            *compare_base[:compare_base.index("compare") + 1], str(before), str(after),
            "--metric", "work_units", "--hypothesis-resamples", "32",
            "--json", "--relative-distributions", *extra,
        ], cwd=ROOT, env=dict(env, CARGO_TARGET_DIR=str(ROOT / "target"), NO_WORK="1"), capture_output=True, text=True)

    # Changing baseline data must not reuse rates computed against the old one.
    alternate = json.loads((first / "run.json").read_text())
    for observation in alternate["observations"]:
        if observation["metric"] == "work_units":
            observation["value"] = str(int(observation["value"]) * 3)
    alternate_path = root / "alternate-baseline.json"
    alternate_path.write_text(json.dumps(alternate))
    result = saved_compare(alternate_path, compared / "run.json")
    assert result.returncode == 0, result.stdout + result.stderr
    rates = json.loads(result.stdout)["relative"][0]["throughput"]
    assert len(rates) == 2
    assert all(rate["point_percent"] is None and rate["interval_percent"] is None
               and "another baseline" in rate["unavailable_reason"] for rate in rates)

    # A modified candidate is stale, and errors must not replace an existing artifact.
    changed = json.loads((compared / "run.json").read_text())
    for observation in changed["observations"]:
        if observation["metric"] == "work_units":
            observation["value"] = str(int(observation["value"]) + 1)
    changed_path = root / "changed-candidate.json"
    changed_path.write_text(json.dumps(changed))
    preserved_compare = root / "previous-comparison.json"
    preserved_compare.write_text("previous comparison")
    result = saved_compare(first / "run.json", changed_path, ("--output", str(preserved_compare)))
    assert result.returncode != 0, result.stdout + result.stderr
    assert "stale custom throughput comparison snapshot" in result.stderr, result.stderr
    assert preserved_compare.read_text() == "previous comparison"
    comparison_html = (compared / "regression-comparison.html").read_text()
    assert "throughput mean (items/unit)" in comparison_html and "-50.000000" in comparison_html
    assert "formatter applied" in comparison_html
    assert "<svg" not in comparison_html
    env["AIRBUG_BENCH_HISTORY"] = "1"
    automatic = ["--history-dir", str(root / "history"), "--hypothesis-resamples", "32"]
    run("history-first", automatic)
    automatic_output = run("history-second", automatic, step=6)
    history = json.loads((automatic_output / "comparison.json").read_text())
    custom = next(row for row in history["cases"][0]["relative"] if row["metric"] == "work_units")
    assert all(rate["point_percent"] == -50 for rate in custom["throughput"])
    assert len(custom["throughput"]) == 2
    assert "formatter applied" in (automatic_output / "report.html").read_text()
    committed = [json.loads(path.read_text()) for path in (root / "history").glob("*/*/comparison.json")]
    assert history in committed
    committed_pair = []
    for directory in (root / "history").glob("*/*"):
        if (directory / "comparison.json").is_file():
            value = json.loads((directory / "comparison.json").read_text())
            committed_pair.append((value, directory / "run.json"))
    current_path = next(path for value, path in committed_pair if value == history)
    previous_path = next(path for value, path in committed_pair if value != history)
    result = saved_compare(previous_path, current_path)
    assert result.returncode == 0, result.stdout + result.stderr
    restored_rates = json.loads(result.stdout)["relative"][0]["throughput"]
    assert len(restored_rates) == 2 and all(rate["point_percent"] == -50 for rate in restored_rates)
    # The usual output file must carry exactly the same enriched run as history.
    assert json.loads(current_path.read_text()) == json.loads((automatic_output / "run.json").read_text())
    result = saved_compare(previous_path, automatic_output / "run.json")
    assert result.returncode == 0, result.stdout + result.stderr
    assert all(rate["point_percent"] == -50 for rate in json.loads(result.stdout)["relative"][0]["throughput"])


    for name, flags, lazy in [
        ("raw-named", ["--baseline", "saved", "--hypothesis-distribution"], False),
        ("raw-loaded", ["--load-baseline", "saved"], True),
        ("raw-history", [*automatic, "--hypothesis-distribution"], False),
    ]:
        output = run(name, [*flags, "--no-html", "--json", "--bootstrap-distributions"], lazy=lazy, plots=True)
        assert not list(output.rglob("*.html")), list(output.iterdir())
        for artifact in ("run.json", "summary.json", "estimates.json", "formatted.json", "formatted.csv", "formatted-estimates.json"):
            assert (output / artifact).is_file(), (name, artifact)
        if name != "raw-loaded":
            assert (output / "comparison.json").is_file()
        if name == "raw-named":
            assert (output / "relative-distributions.json").is_file()
    result = subprocess.run([*base, "--no-html", "--json", "--hypothesis-distribution", *automatic], cwd=ROOT,
                            env=env, capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "BENCH_REPORT=" not in result.stdout, result.stdout
    latest = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    raw_output = root / "history" / "reports" / latest["id"]
    assert (raw_output / "run.json").is_file(), raw_output
    assert (raw_output / "estimates.json").is_file()
    assert not list(raw_output.rglob("*.html"))
    assert (first / "run.json").read_bytes() == original
    parsed_charts = 0
    custom_axis = False
    for path in root.rglob("*.html"):
        for svg in re.findall(r"<svg\b.*?</svg>", path.read_text(), flags=re.DOTALL):
            chart = ET.fromstring(svg)
            parsed_charts += 1
            # The XML text must recover the literal user unit, with no invented
            # child element from the angle brackets or malformed ampersand.
            labels = ["".join(node.itertext()) for node in chart.iter()
                      if node.tag.rsplit("}", 1)[-1] == "text"]
            custom_axis |= any("groups<&>" in label for label in labels)
    assert parsed_charts > 0
    assert custom_axis, "custom unit missing from exported SVG text"

print("Cargo custom bootstrap: source counters, custom scale/text, saved-baseline reanalysis, raw preservation, relative formatter throughput, escaped HTML with/without plots valid SVG XML, baseline summary axis scales, stale-output preservation, filtered standalone raw comparisons and JSON/CSV-only artifacts passed")
