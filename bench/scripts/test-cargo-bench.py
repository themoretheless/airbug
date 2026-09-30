#!/usr/bin/env python3
"""Exercise Cargo's real argument forwarding and the attribute-generated harness."""
import json
import csv
import os
import tempfile
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
COMMAND = [
    "cargo", "bench", "-p", "airbug-bench", "--features", "macros",
    "--bench", "attributed", "--offline",
]


history = tempfile.TemporaryDirectory(prefix="airbug-bench-history-")
environment = dict(os.environ, AIRBUG_BENCH_HISTORY="0", AIRBUG_DASHBOARD_ROOT=history.name, AIRBUG_TEST_NO_SETUP="1", AIRBUG_TEST_NO_EXECUTION="1")


def run(*args, success=True):
    result = subprocess.run(
        [*COMMAND, *args], cwd=ROOT, capture_output=True, text=True, env=environment,
    )
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result.stdout + result.stderr


listed = run("--", "--list")
assert "collections/sort" in listed and "collections/sum" in listed, listed
assert "collections/disabled" not in listed, listed
for case in ["sort_only/64", "sort_only/1024", "parameterized_sum/8", "parameterized_sum/32", "clone_sorted"]:
    assert f"collections/{case}" in listed, listed

filtered = run("collections/sum", "--", "--exact", "--list")
assert "collections/sum" in filtered and "collections/sort" not in filtered, filtered

missing = run("no-such-case", "--", "--list")
assert "collections/" not in missing, missing

with tempfile.TemporaryDirectory(prefix="airbug-preview-") as directory:
    destination = pathlib.Path(directory) / "must-not-exist"
    preview = run("--", "--dry-run", "--output", str(destination))
    planned, _ = json.JSONDecoder().raw_decode(preview.lstrip())
    assert {case["id"] for case in planned["cases"]} == {
        line for line in listed.splitlines() if line.startswith("collections/")
    }, planned
    assert not destination.exists()
    run("--", "--list", "--output", str(destination))
    assert not destination.exists()
# Timer parsing and preview must remain lazy, even for unavailable hardware.
for clock in ["os", "cpu"]:
    preview = run("collections/sum", "--", "--exact", "--timer", clock, "--dry-run")
    planned, _ = json.JSONDecoder().raw_decode(preview.lstrip())
    assert planned["timer"] == clock, planned
    assert all(case["contract"]["timer.clock"] == clock for case in planned["cases"]), planned
    run("collections/sum", "--", "--exact", "--timer", clock, "--list")
for flags, message in [
    (["--timer"], "--timer requires os or cpu"),
    (["--timer", "invalid"], "timer must be os or cpu"),
    (["--timer", "os", "--timer", "cpu"], "timer specified twice"),
]:
    assert message in run("--", *flags, success=False)
environment.pop("AIRBUG_TEST_NO_EXECUTION")
assert not (pathlib.Path(history.name) / "target/airbug-report").exists()

measured = run("collections/sum", "--", "--exact", "--profile", "quick", "--json")
assert "BENCH_RESULT=" in measured and "collections/sum" in measured, measured
assert "collections/sort" not in measured, measured

duplicate = run("sort", "--", "--filter", "sum", success=False)
assert "filter specified twice" in duplicate, duplicate

unknown = run("--", "--unknown-option", success=False)
assert "unknown argument" in unknown, unknown

print("cargo bench: listing, filtering, measurement and argument errors passed")

reports = list(pathlib.Path(history.name).glob("target/airbug-report/runs/*/run.json"))
assert len(reports) == 1, reports
report = json.loads(reports[0].read_text())
assert report["state"] == "passed" and report["kind"] == "bench", report
assert len(report["tests"]) == 1 and report["tests"][0]["name"] == "collections/sum", report
assert report["tests"][0]["ns_per_op"] is not None, report
print("cargo bench: dashboard history and case metric passed")
run("collections/sum", "--", "--profile", "quick", "--output", history.name, success=False)
reports = sorted(pathlib.Path(history.name).glob("target/airbug-report/runs/*/run.json"))
assert len(reports) == 2, reports
assert json.loads(reports[-1].read_text())["state"] == "failed"
environment["AIRBUG_DASHBOARD"] = "0"
run("collections/sum", "--", "--profile", "quick")
assert len(list(pathlib.Path(history.name).glob("target/airbug-report/runs/*/run.json"))) == 2
print("cargo bench: artifact failure and dashboard opt-out passed")
with tempfile.TemporaryDirectory(prefix="airbug-previous-run-") as directory:
    history_root = pathlib.Path(directory) / "history"
    environment["AIRBUG_BENCH_HISTORY"] = "1"
    common = ["--exact", "--samples", "1", "--iterations", "2", "--warmup-ms", "0", "--history-dir", str(history_root), "--json"]
    for flag in ["--list", "--dry-run", "--test", "--no-history"]:
        value = run("collections/sum", "--", *common, flag)
        assert "BENCH_COMPARISON=" not in value, value
        assert not history_root.exists()
    def previous(case="collections/sum", *extra):
        value = run(case, "--", *common, *extra)
        result = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in value.splitlines() if line.startswith("BENCH_RESULT=")))
        comparison = json.loads(next(line.removeprefix("BENCH_COMPARISON=") for line in value.splitlines() if line.startswith("BENCH_COMPARISON=")))
        report_path = pathlib.Path(json.loads(next(line.removeprefix("BENCH_REPORT=") for line in value.splitlines() if line.startswith("BENCH_REPORT="))))
        assert report_path.is_file(), value
        html = report_path.read_text()
        assert "Previous run comparison" in html and case in html, html
        assert report_path.parent.parent == history_root / "reports"
        assert json.loads((report_path.parent / "run.json").read_text())["id"] == result["id"]
        return result, comparison
    first, comparison = previous()
    assert comparison["cases"][0]["status"] == "first_run", comparison
    previous("collections/sort")
    second, comparison = previous()
    assert comparison["cases"][0]["previous_run"] == first["id"], comparison
    assert comparison["cases"][0]["status"] == "compared", comparison
    assert comparison["cases"][0]["comparisons"][0]["decision"] == "inconclusive", comparison
    before = len(list(history_root.rglob("COMMITTED")))
    run("collections/sum", "--", *common, "--output", directory, success=False)
    assert len(list(history_root.rglob("COMMITTED"))) == before
    failure = run("collections/unselected", "--", *common, success=False)
    assert "unselected setup ran" in failure, failure
    assert len(list(history_root.rglob("COMMITTED"))) == before
    _, comparison = previous()
    assert comparison["cases"][0]["previous_run"] == second["id"], comparison
    environment["AIRBUG_BENCH_HISTORY"] = "0"
print("cargo bench: automatic previous-run comparison, filtered history and failed/smoke/preview exclusions passed")

with tempfile.TemporaryDirectory(prefix="airbug-discard-") as directory:
    saved_environment = environment.copy()
    environment.update(AIRBUG_BENCH_HISTORY="1", AIRBUG_DASHBOARD="1", AIRBUG_DASHBOARD_ROOT=directory)
    root = pathlib.Path(directory)
    common = ["--exact", "--samples", "1", "--iterations", "2", "--warmup-ms", "0", "--history-dir", str(root / "history"), "--discard", "--json"]
    result = run("collections/sum", "--", *common)
    assert "BENCH_RESULT=" in result and "BENCH_SUMMARY=" in result, result
    assert "BENCH_COMPARISON=" not in result and "Airbug run:" not in result, result
    assert list(root.iterdir()) == []
    for conflicting in [["--output", str(root / "out")], ["--profile-time-ms", "1"], ["--baseline", "main"]]:
        result = run("collections/unselected", "--", *common, *conflicting, success=False)
        assert "--discard cannot combine" in result, result
        assert "unselected setup ran" not in result, result
        assert list(root.iterdir()) == []
    environment.clear()
    environment.update(saved_environment)
print("cargo bench: discard executes measurements without artifacts or either history and rejects conflicting options")

named_dashboard_before = environment.get("AIRBUG_DASHBOARD")
environment["AIRBUG_DASHBOARD"] = "0"
with tempfile.TemporaryDirectory(prefix="airbug-named-baseline-") as directory:
    import hashlib
    store = pathlib.Path(directory)
    common = ["--exact", "--samples", "1", "--iterations", "2", "--warmup-ms", "0", "--json", "--baseline-store", directory]
    original = run("collections/sum", "--", *common)
    data = next(line.removeprefix("BENCH_RESULT=") for line in original.splitlines() if line.startswith("BENCH_RESULT="))
    source = store / "run.json"
    source.write_text(data)
    (store / "baselines").mkdir()
    pointer = store / "baselines/main.json"
    pointer.write_text(json.dumps({"run": str(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest()}))
    before = pointer.read_bytes()
    def named(case, name="main", mode="--baseline"):
        text = run(case, "--", *common, mode, name)
        return json.loads(next(line.removeprefix("BENCH_BASELINE=") for line in text.splitlines() if line.startswith("BENCH_BASELINE=")))
    result = named("collections/sum")
    assert result["name"] == "main", result
    assert result["report"]["cases"][0]["status"] == "compared", result
    for name in ["missing", "main"]:
        failure = run("collections/unselected", "--", *common, "--baseline", name, success=False)
        assert "unselected setup ran" not in failure, failure
        assert "baseline" in failure, failure
    assert named("collections/sort", mode="--baseline-lenient")["report"]["cases"][0]["status"] == "first_run"
    assert named("collections/sum", "missing", "--baseline-lenient")["report"]["cases"][0]["status"] == "first_run"
    assert pointer.read_bytes() == before
    saved_pointer = store / "baselines/saved.cases.json"
    for preview in ["--list", "--dry-run"]:
        run("collections/sum", "--", *common, "--save-baseline", "saved", preview)
        assert not saved_pointer.exists()
    run("collections/sum", "--", *common, "--save-baseline", "saved")
    first_pointer = saved_pointer.read_bytes()
    first_path = pathlib.Path(json.loads(first_pointer)["cases"]["collections/sum"]["run"])
    first_bytes = first_path.read_bytes()
    failure = run("collections/unselected", "--", *common, "--save-baseline", "saved", success=False)
    assert "already exists" in failure and "unselected setup ran" not in failure, failure
    retained = run("collections/sum", "--", *common, "--retain-baseline", "saved")
    saved_info = json.loads(next(line.removeprefix("BENCH_BASELINE_SAVED=") for line in retained.splitlines() if line.startswith("BENCH_BASELINE_SAVED=")))
    assert saved_info["retained"] is True
    assert saved_pointer.read_bytes() == first_pointer
    assert len(list((store / "baseline-runs").iterdir())) == 1
    run("collections/unselected", "--", *common, "--replace-baseline", "saved", success=False)
    assert saved_pointer.read_bytes() == first_pointer
    run("collections/sum", "--", *common, "--replace-baseline", "saved", "--output", directory, success=False)
    assert saved_pointer.read_bytes() == first_pointer
    replacement = run("collections/sum", "--", *common, "--baseline", "saved", "--replace-baseline", "saved")
    assert "BENCH_BASELINE=" in replacement and "BENCH_BASELINE_SAVED=" in replacement
    assert saved_pointer.read_bytes() != first_pointer
    assert first_path.read_bytes() == first_bytes
    for extra in [["--test"], ["--discard"], ["--profile-time-ms", "1"], ["--retain-baseline", "saved"]]:
        run("collections/unselected", "--", *common, "--save-baseline", "invalid", *extra, success=False)
        assert not (store / "baselines/invalid.cases.json").exists()
    sort_output = run("collections/sort", "--", *common)
    sort_data = next(line.removeprefix("BENCH_RESULT=") for line in sort_output.splitlines() if line.startswith("BENCH_RESULT="))
    sort_source = store / "sort.json"
    sort_source.write_text(sort_data)
    (store / "baselines/mixed.cases.json").write_text(json.dumps({"version": 1, "cases": {
        "collections/sum": json.loads(saved_pointer.read_bytes())["cases"]["collections/sum"],
        "collections/sort": {"run": str(sort_source), "sha256": hashlib.sha256(sort_source.read_bytes()).hexdigest()}
    }}))
    mixed = named("collections/sort", "mixed")
    assert mixed["report"]["cases"][0]["previous_run"] == json.loads(sort_data)["id"], mixed
    assert mixed["report"]["cases"][0]["status"] == "compared", mixed
    mixed = named("collections/sum", "mixed")
    assert mixed["report"]["cases"][0]["status"] == "compared", mixed
    both_args = [arg for arg in common if arg != "--exact"]
    run("^collections/(sum|sort)$", "--", *both_args, "--regex", "--save-baseline", "partial")
    partial_path = store / "baselines/partial.cases.json"
    old_cases = json.loads(partial_path.read_bytes())["cases"]
    old_sort_bytes = pathlib.Path(old_cases["collections/sort"]["run"]).read_bytes()
    run("collections/sum", "--", *common, "--replace-baseline", "partial")
    new_cases = json.loads(partial_path.read_bytes())["cases"]
    assert new_cases["collections/sort"] == old_cases["collections/sort"]
    assert new_cases["collections/sum"] != old_cases["collections/sum"]
    assert pathlib.Path(new_cases["collections/sort"]["run"]).read_bytes() == old_sort_bytes
    reloaded = run("^collections/(sum|sort)$", "--", *both_args, "--regex", "--load-baseline", "partial")
    reloaded_runs = [json.loads(line.removeprefix("BENCH_RESULT=")) for line in reloaded.splitlines() if line.startswith("BENCH_RESULT=")]
    assert {case["id"] for result in reloaded_runs for case in result["cases"]} == {"collections/sum", "collections/sort"}
    configured = run("collections/sum", "--", *common, "--baseline", "saved", "--significance-level", "0.01", "--noise-threshold-percent", "2.5", "--hypothesis-resamples", "128", "--hypothesis-seed", "73")
    configured_report = json.loads(next(line.removeprefix("BENCH_BASELINE=") for line in configured.splitlines() if line.startswith("BENCH_BASELINE=")))
    assert configured_report["report"]["config"] == {"significance_level": 0.01, "noise_threshold_percent": 2.5, "hypothesis": {"resamples": 128, "seed": 73}, "capture_distribution": False}
    hypothesis = configured_report["report"]["cases"][0]["comparisons"][0]["hypothesis"]["test"]
    assert hypothesis["resamples"] == 128 and hypothesis["seed"] == 73
    assert hypothesis["p_value"] is None and hypothesis["candidate_units"] == 1
    for flags in [["--summary-scale", "logarithmic"], ["--summary-parameter", "size"], ["--summary-scale", "invalid"], ["--hypothesis-resamples", "1"], ["--hypothesis-resamples", "1000001"], ["--hypothesis-seed", "1", "--hypothesis-seed", "2"], ["--hypothesis-seed"], ["--significance-level", "0"], ["--significance-level", "NaN"], ["--noise-threshold-percent", "inf"], ["--noise-threshold-percent", "-1"], ["--significance-level", "0.1", "--significance-level", "0.2"]]:
        invalid = run("collections/unselected", "--", *common, *flags, success=False)
        assert "unselected setup ran" not in invalid, invalid
    import copy
    for baseline_name, offset in [("hypothesis-old", 100), ("hypothesis-new", 300)]:
        independent = json.loads(data)
        independent["id"] = baseline_name
        for case in independent["cases"]:
            case["contract"]["param.size"] = "10"
        observations = []
        for process in range(12):
            for original in independent["observations"]:
                observation = copy.deepcopy(original)
                observation["process"] = process
                observation["value"] = str((offset + process) * observation["operations"])
                observations.append(observation)
        independent["observations"] = observations
        artifact = store / f"{baseline_name}.json"
        artifact.write_text(json.dumps(independent))
        (store / "baselines" / f"{baseline_name}.json").write_text(json.dumps({"run": str(artifact), "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest()}))
    hypotheses = []
    for _ in range(2):
        result = run("collections/sum", "--", *common, "--load-baseline", "hypothesis-new", "--baseline", "hypothesis-old", "--hypothesis-resamples", "128", "--hypothesis-seed", "73")
        report = json.loads(next(line.removeprefix("BENCH_BASELINE=") for line in result.splitlines() if line.startswith("BENCH_BASELINE=")))
        hypotheses.append(report["report"]["cases"][0]["comparisons"][0]["hypothesis"])
    assert hypotheses[0] == hypotheses[1]
    assert 0 < hypotheses[0]["test"]["p_value"] < 0.05
    assert hypotheses[0]["test"]["baseline_units"] == 12
    assert hypotheses[0]["test"]["resamples"] == 128 and hypotheses[0]["test"]["seed"] == 73
    distribution_output = store / "distribution-export"
    exported = run("collections/sum", "--", *common, "--load-baseline", "hypothesis-new", "--baseline", "hypothesis-old", "--hypothesis-resamples", "128", "--hypothesis-seed", "73", "--hypothesis-distribution", "--summary-parameter", "size", "--summary-scale", "logarithmic", "--output", str(distribution_output))
    exported_report = json.loads(next(line.removeprefix("BENCH_BASELINE=") for line in exported.splitlines() if line.startswith("BENCH_BASELINE=")))["report"]
    assert json.loads((distribution_output / "comparison.json").read_text()) == exported_report
    summary_html = (distribution_output / "report.html").read_text()
    assert "Input parameter summaries" in summary_html and "X: size" in summary_html
    assert "Iteration time comparison" in summary_html
    assert "baseline /" in summary_html and "candidate /" in summary_html
    plain_output = store / "plain-comparison-export"
    run("collections/sum", "--", *common, "--load-baseline", "hypothesis-new", "--baseline", "hypothesis-old", "--output", str(plain_output))
    assert "Iteration time comparison" in (plain_output / "report.html").read_text()
    assert not (plain_output / "estimates.json").exists()
    tables_output = store / "tables-comparison-export"
    run("collections/sum", "--", *common, "--load-baseline", "hypothesis-new", "--baseline", "hypothesis-old", "--hypothesis-resamples", "128", "--hypothesis-seed", "73", "--hypothesis-distribution", "--resamples", "128", "--bootstrap-distributions", "--no-plots", "--output", str(tables_output))
    assert json.loads((tables_output / "comparison.json").read_text()) == exported_report
    for html_file in tables_output.glob("*.html"):
        assert "<svg" not in html_file.read_text(), html_file
    assert (tables_output / "estimates.json").is_file()
    assert (tables_output / "relative-distributions.json").is_file()
    assert not (tables_output / "comparison.html").exists()


    assert "0 series omitted" in summary_html
    exported_html = (distribution_output / "comparison.html").read_text()
    assert "<svg " in exported_html and "128 null draws" in exported_html
    hypothesis_export = exported_report["cases"][0]["comparisons"][0]["hypothesis"]
    assert len(hypothesis_export["null_distribution"]) == 128
    extreme = sum(draw["kind"] != "finite" or abs(draw["value"]) >= abs(hypothesis_export["test"]["statistic"]) for draw in hypothesis_export["null_distribution"])
    assert hypothesis_export["test"]["p_value"] == (extreme + 1) / 129
    for name, slope in [("regression-old", 10), ("regression-new", 20)]:
        fixture = json.loads(data)
        fixture["id"] = name
        originals = {(o["case"], o["metric"], o["variant"]): o for o in fixture["observations"]}
        fixture["observations"] = []
        for original in originals.values():
            for process in range(2):
                for count in range(1, 4):
                    observation = copy.deepcopy(original)
                    observation.update(process=process, sequence=count, operations=count, value=str((slope + process) * count))
                    fixture["observations"].append(observation)
        artifact = store / f"{name}.json"
        artifact.write_text(json.dumps(fixture))
        (store / "baselines" / f"{name}.json").write_text(json.dumps({"run": str(artifact), "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest()}))
    regression_comparison_dir = store / "regression-comparison-export"
    run("collections/sum", "--", *common, "--load-baseline", "regression-new", "--baseline", "regression-old",
        "--resamples", "128", "--output", str(regression_comparison_dir))
    overlay = (regression_comparison_dir / "regression-comparison.html").read_text()
    assert overlay.count("<polygon") == 8, overlay
    assert "relative mean change" in overlay and "relative median change" in overlay
    assert "Practical noise region" in overlay
    relative = json.loads((regression_comparison_dir / "relative-distributions.json").read_text())
    assert len(relative) == 1
    assert relative[0]["baseline_units"] == 2 and relative[0]["candidate_units"] == 2
    assert len(relative[0]["report"]["mean"]["draws_percent"]) == 128
    assert abs(relative[0]["report"]["mean"]["point_percent"] - (20.5 / 10.5 - 1) * 100) < 1e-10
    assert "baseline process 1" in overlay and "candidate process 1" in overlay
    assert "density comparison" in overlay and "Gaussian KDE bandwidth" in overlay
    assert "Observation rug at zero" in overlay
    zero_fixture = json.loads(data)
    for observation in zero_fixture["observations"]:
        observation["value"] = "0"
    zero_artifact = store / "zero-violin.json"
    zero_artifact.write_text(json.dumps(zero_fixture))
    (store / "baselines" / "zero-violin.json").write_text(json.dumps({"run": str(zero_artifact), "sha256": hashlib.sha256(zero_artifact.read_bytes()).hexdigest()}))
    zero_output = store / "zero-violin-export"
    run("collections/sum", "--", *common, "--load-baseline", "zero-violin", "--summary-scale", "logarithmic", "--output", str(zero_output))
    assert "population contains nonpositive values; no subset was plotted" in (zero_output / "estimates.html").read_text()
    large_margin = run("collections/sum", "--", *common, "--load-baseline", "hypothesis-new", "--baseline", "hypothesis-old", "--noise-threshold-percent", "250", "--hypothesis-resamples", "128")
    large_margin_report = json.loads(next(line.removeprefix("BENCH_BASELINE=") for line in large_margin.splitlines() if line.startswith("BENCH_BASELINE=")))
    large_margin_case = large_margin_report["report"]["cases"][0]
    assert large_margin_case["config"]["noise_threshold_percent"] == 250.0
    assert large_margin_case["comparisons"][0]["decision"] == "within_margin", large_margin_case
    partial_before_repair = partial_path.read_bytes()
    pathlib.Path(new_cases["collections/sum"]["run"]).write_text("{}")
    blocked = run("collections/sort", "--", *common, "--replace-baseline", "partial", success=False)
    assert "artifact changed" in blocked, blocked
    assert partial_path.read_bytes() == partial_before_repair
    run("collections/sum", "--", *common, "--replace-baseline", "partial")
    repaired_cases = json.loads(partial_path.read_bytes())["cases"]
    assert repaired_cases["collections/sort"] == old_cases["collections/sort"]
    assert named("collections/sum", "partial")["report"]["cases"][0]["status"] == "compared"
    environment["AIRBUG_TEST_NO_EXECUTION"] = "1"
    environment["AIRBUG_TEST_NO_SETUP"] = "1"
    mixed_export = store / "mixed-export"
    mixed_loaded = run("^collections/(sum|sort)$", "--", *[arg for arg in common if arg != "--exact"], "--regex", "--load-baseline", "mixed", "--output", str(mixed_export))
    mixed_results = [json.loads(line.removeprefix("BENCH_RESULT=")) for line in mixed_loaded.splitlines() if line.startswith("BENCH_RESULT=")]
    assert len(mixed_results) == 2, mixed_loaded
    assert {case["id"] for result in mixed_results for case in result["cases"]} == {"collections/sum", "collections/sort"}
    exported = [json.loads(path.read_text()) for path in mixed_export.glob("*/run.json")]
    assert sorted(result["id"] for result in exported) == sorted(result["id"] for result in mixed_results)
    loaded = run("collections/sum", "--", *common, "--load-baseline", "saved", "--baseline", "main")
    assert "BENCH_BASELINE=" in loaded and "BENCH_RESULT=" in loaded, loaded
    loaded_result = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in loaded.splitlines() if line.startswith("BENCH_RESULT=")))
    saved_result = json.loads(pathlib.Path(json.loads(saved_pointer.read_bytes())["cases"]["collections/sum"]["run"]).read_text())
    assert loaded_result == saved_result
    export = store / "reanalyzed"
    run("collections/sum", "--", *common, "--load-baseline", "saved", "--output", str(export))
    assert json.loads((export / "run.json").read_text()) == saved_result
    for extra in [["--test"], ["--replace-baseline", "saved"], ["--profile-time-ms", "1"]]:
        run("collections/sum", "--", *common, "--load-baseline", "saved", *extra, success=False)
    missing = run("collections/unselected", "--", *common, "--load-baseline", "saved", success=False)
    assert "has no case" in missing and "unselected setup ran" not in missing, missing
    environment.pop("AIRBUG_TEST_NO_EXECUTION")
    source.write_text("{}")
    for mode in ["--baseline", "--baseline-lenient"]:
        failure = run("collections/unselected", "--", *common, mode, "main", success=False)
        assert "baseline artifact changed" in failure, failure
        assert "unselected setup ran" not in failure, failure
    run("collections/sum", "--", *common, "--baseline", "main", "--baseline-lenient", "main", success=False)
print("cargo bench: named strict/lenient baselines, preflight and corruption checks passed")

if named_dashboard_before is None:
    environment.pop("AIRBUG_DASHBOARD")
else:
    environment["AIRBUG_DASHBOARD"] = named_dashboard_before

for clock in ["os", "cpu"]:
    result = subprocess.run(
        [*COMMAND, "collections/sum", "--", "--exact", "--timer", clock,
         "--samples", "1", "--iterations", "2", "--warmup-ms", "0", "--json"],
        cwd=ROOT, capture_output=True, text=True, env=environment,
    )
    if clock == "cpu" and result.returncode != 0:
        assert any(reason in result.stderr for reason in [
            "CPU timer is unavailable on this platform", "CPU timer requires SSE2, TSC and RDTSCP",
            "CPU timer requires an invariant timestamp counter",
        ]), result.stdout + result.stderr
        assert "BENCH_RESULT=" not in result.stdout
        print("cargo bench: CPU clock explicitly unavailable on this host")
        continue
    assert result.returncode == 0, result.stdout + result.stderr
    payload = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert payload["provenance"]["timer.clock"] == clock, payload
    assert all(case["contract"]["timer.clock"] == clock for case in payload["cases"]), payload
    assert payload["provenance"][f"timer.{clock}.calibration.loop_iterations"] == "10000", payload
    assert payload["provenance"]["timer.overhead_policy"] == "raw; no subtraction", payload
    if clock == "cpu":
        assert int(payload["provenance"]["timer.frequency_hz"]) > 0, payload
print("cargo bench: timer selection, lazy previews and invalid arguments passed")
for flags in [["--list"], ["--dry-run"]]:
    lazy = run("collections/sum", "--", "--exact", "--overhead", "subtract", *flags)
    if "--dry-run" in flags:
        payload, _ = json.JSONDecoder().raw_decode(lazy.lstrip())
        assert payload["overhead"] == "subtract", payload
        assert any(m["id"] == "wall.adjusted" for m in payload["cases"][0]["metrics"]), payload
for flags, message in [
    (["--overhead"], "--overhead requires raw or subtract"),
    (["--overhead", "bad"], "overhead policy must be raw or subtract"),
    (["--overhead", "raw", "--overhead", "subtract"], "overhead policy specified twice"),
    (["--overhead", "subtract", "--profile-time-ms", "1"], "overhead compensation cannot combine"),
]:
    assert message in run("--", *flags, success=False)
for flags, compensated in [([], True), (["--test"], False)]:
    output = run("collections/sum", "--", "--exact", "--overhead", "subtract", "--iterations", "2", "--samples", "1", "--warmup-ms", "0", "--json", *flags)
    payload = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in output.splitlines() if line.startswith("BENCH_RESULT=")))
    values = {o["metric"]: int(o["value"]) for o in payload["observations"]}
    assert ("wall.adjusted" in values) == compensated, payload
    if compensated:
        assert values["wall.adjusted"] <= values["wall"], payload
        assert "timer.os.correction.alloc_batch_ns" in payload["provenance"], payload
print("cargo bench: compensated/raw metrics, lazy preview, smoke precedence and errors passed")


with tempfile.TemporaryDirectory(prefix="airbug-summary-") as directory:
    destination = pathlib.Path(directory) / "result"
    output = run("collections/sum", "--", "--exact", "--samples", "1", "--iterations", "100",
                 "--warmup-ms", "0", "--output", str(destination), "--json")
    summary = json.loads(next(line.removeprefix("BENCH_SUMMARY=") for line in output.splitlines() if line.startswith("BENCH_SUMMARY=")))
    assert len(summary) == 1 and summary[0]["normalized_per_operation"], summary
    stats = summary[0]["summary"]
    assert stats["count"] == 1 and stats["standard_deviation"] is None, summary
    assert stats["minimum"] == stats["maximum"] == stats["mean"] == stats["median"], stats
    assert summary[0]["operations"] == "100", summary
    assert summary[0]["operation_weighted_mean"] == stats["mean"], summary
    assert json.loads((destination / "summary.json").read_text()) == summary
    assert "BENCH_ESTIMATES=" not in output
text_summary = run("collections/sum", "--", "--exact", "--test")
assert "Descriptive statistics" in text_summary and "Scaled MAD" in text_summary, text_summary

# Profiling uses the same Cargo harness, but emits only profiling artifacts.
with tempfile.TemporaryDirectory(prefix="airbug-profile-cli-") as directory:
    destination = pathlib.Path(directory) / "session"
    for mode in ["--list", "--dry-run"]:
        preview = run("collections/sum", "--", "--exact", "--profile-time-ms", "3",
                      "--output", str(destination), mode)
        assert not destination.exists(), preview
        if mode == "--dry-run":
            value, _ = json.JSONDecoder().raw_decode(preview.lstrip())
            assert value["profile_time_ms"] == 3, value
    environment.pop("AIRBUG_DASHBOARD")
    output = run("collections/contended/threads=2", "--", "--exact",
                 "--profile-time-ms", "3", "--iterations", "2",
                 "--output", str(destination), "--json")
    profile = json.loads(next(line.removeprefix("PROFILE_RESULT=")
                              for line in output.splitlines() if line.startswith("PROFILE_RESULT=")))
    assert profile["complete"] and profile["requested_ns_per_case"] == "3000000", profile
    assert len(profile["cases"]) == 1 and "BENCH_RESULT=" not in output, output
    case = profile["cases"][0]
    assert case["id"] == "collections/contended/threads=2" and case["error"] is None, case
    assert int(case["stats"]["elapsed_ns"]) >= 3000000, case
    assert int(case["stats"]["operations"]) == case["stats"]["batches"] * 4, case
    assert json.loads((destination / "profile.json").read_text()) == profile
    assert json.loads((destination / case["directory"] / "summary.json").read_text()) == case
    assert not (destination / "report.json").exists()
    profile_history = sorted(pathlib.Path(history.name).glob("target/airbug-report/runs/*/run.json"))
    assert len(profile_history) == 3, profile_history
    recorded = json.loads(profile_history[-1].read_text())
    assert recorded["state"] == "passed" and len(recorded["tests"]) == 1, recorded
    assert recorded["tests"][0]["ns_per_op"] is None, recorded

    run("collections/sum", "--", "--exact", "--profile-time-ms", "3",
        "--output", str(destination), success=False)
    profile_history = sorted(pathlib.Path(history.name).glob("target/airbug-report/runs/*/run.json"))
    assert len(profile_history) == 4, profile_history
    assert json.loads(profile_history[-1].read_text())["state"] == "failed"
    environment["AIRBUG_DASHBOARD"] = "0"

for options in [("0",), ("1", "--test"), ("1", "--profile-time-ms", "2"),
                ("1", "--resamples", "100"), ("1", "--bytes-format", "binary")]:
    run("collections/sum", "--", "--exact", "--profile-time-ms", *options, success=False)
print("cargo bench: timed profiling, worker accounting, artifacts and CLI validation passed")

# Parameter names work with Cargo's positional filter; every setup is fresh.
for case in ["sort_only/64", "sort_only/1024", "parameterized_sum/8", "clone_sorted"]:
    output = run(f"collections/{case}", "--", "--exact", "--profile", "quick", "--json")
    payload = next(line.split("BENCH_RESULT=", 1)[1] for line in output.splitlines() if line.startswith("BENCH_RESULT="))
    result = json.loads(payload)
    assert [entry["id"] for entry in result["cases"]] == [f"collections/{case}"], result
    assert result["observations"], result
print("cargo bench: parameterized cases, fresh setup and output-drop policy passed")
# Extended features execute through the same Cargo target, with no runner dependency.
for case in ["generic_fill/type=u32/const=4", "generic_fill/type=u64/const=16",
             "async_sum/4", "async_sort/8", "contended/threads=2", "threaded_sort/8/threads=2", "manual/4", "allocate"]:
    assert f"collections/{case}" in listed, listed

def measured_run(*args):
    output = run(*args, "--json")
    payload = next(line.split("BENCH_RESULT=", 1)[1] for line in output.splitlines() if line.startswith("BENCH_RESULT="))
    return json.loads(payload)

sampling = measured_run("collections/async_sum/4", "--", "--exact")
assert len(sampling["observations"]) == 3, sampling
sampling_override = measured_run("collections/async_sum/4", "--", "--exact", "--samples", "2")
assert len(sampling_override["observations"]) == 2, sampling_override
preview = run("collections/async_sum/4", "--", "--exact", "--dry-run")
# Cargo writes build output to stderr, so parse the JSON from the stdout prefix.
preview_json, _ = json.JSONDecoder().raw_decode(preview.lstrip())
assert preview_json["cases"][0]["contract"]["samples"] == "3", preview_json

environment.pop("AIRBUG_TEST_NO_SETUP")
environment.pop("AIRBUG_DASHBOARD")
complete = measured_run("--", "--profile", "quick")
assert complete["status"] == "complete", complete
allocation_case = next(case for case in complete["cases"] if case["id"] == "collections/allocate")
assert allocation_case["contract"]["batch.policy"] == "LargeInput", allocation_case
assert allocation_case["contract"]["output.drop"] == "outside", allocation_case
ids = [case["id"] for case in complete["cases"]]
assert len(ids) == len(set(ids)), ids
assert set(ids) == set(line for line in listed.splitlines() if line.startswith("collections/")), ids
for case in complete["cases"]:
    observations = [o for o in complete["observations"] if o["case"] == case["id"]]
    assert len(observations) == 8, case
    if case["id"].endswith("threads=2"):
        assert all(o["operations"] % 2 == 0 for o in observations), observations
        assert case["contract"]["threads"] == "2", case
    if "/worker_local/" in case["id"]:
        assert case["contract"]["work.input.items"] == "batch_total", case
        assert all(int(o["work_totals"]["items"]) == o["operations"] for o in observations), observations
    if "async_" in case["id"]:
        assert "async.scope" in case["contract"], case
report_paths = sorted(pathlib.Path(history.name).glob("target/airbug-report/runs/*/run.json"))
full_history = json.loads(report_paths[-1].read_text())
assert full_history["state"] == "passed", full_history
assert {case["name"] for case in full_history["tests"]} == set(ids), full_history
print("cargo bench: types, consts, async, threads, manual timing, sampling overrides and full history passed")
assert "collections/nested/plain" in listed and "collections/nested/child/multiple" in listed, listed
assert "collections/nested/ignored" not in listed, listed
ignored = run("--", "--ignored", "--list")
assert "collections/nested/ignored" in ignored and "collections/nested/plain" not in ignored, ignored
included = run("--", "--include-ignored", "--list")
assert "collections/nested/ignored" in included and "collections/nested/plain" in included, included
ignored_run = measured_run("--", "--ignored", "--profile", "quick")
assert [case["id"] for case in ignored_run["cases"]] == ["collections/nested/ignored"], ignored_run
nested = measured_run("collections/nested/child/multiple", "--", "--exact")
assert len(nested["observations"]) == 3, nested
assert nested["cases"][0]["contract"]["work.counter.items"] == "2", nested
assert nested["cases"][0]["contract"]["work.counter.bytes"] == "128", nested
assert nested["cases"][0]["contract"]["work.counter.chars"] == "4", nested
print("cargo bench: nested group inheritance, ignored selection and multiple counters passed")
smoke = measured_run("collections/nested/ignored", "--", "--exact", "--ignored", "--test")
assert len(smoke["observations"]) == 1 and smoke["observations"][0]["operations"] == 1, smoke
assert smoke["cases"][0]["contract"]["execution.mode"] == "test_once", smoke
print("cargo bench: smoke mode runs a selected ignored case exactly once")
# Exact iteration counts cross the internal input-wave boundary for every lifecycle.
for case in ["sum", "sort_only/64", "owned_string", "async_sum/4", "async_sort/8",
             "async_owned_string", "threaded_sort/8/threads=2", "contended/threads=2"]:
    fixed = measured_run(f"collections/{case}", "--", "--exact", "--iterations", "130",
                         "--samples", "2", "--warmup-ms", "0")
    workers = 2 if case.endswith("threads=2") else 1
    assert len(fixed["observations"]) == 2, fixed
    assert all(o["operations"] == 130 * workers for o in fixed["observations"]), fixed
    assert fixed["cases"][0]["contract"]["sampling.iterations"] == "130", fixed
for case in ["threaded_sort/8/threads=2", "contended/threads=2"]:
    environment["AIRBUG_BENCH_THREADS"] = "invalid"
    overridden = measured_run(f"collections/{case.replace('threads=2', 'threads=3')}", "--", "--exact", "--threads", "3",
                              "--iterations", "130", "--samples", "1", "--warmup-ms", "0")
    assert overridden["cases"][0]["contract"]["threads"] == "3", overridden
    assert all(o["operations"] == 390 for o in overridden["observations"]), overridden
    environment["AIRBUG_BENCH_THREADS"] = "4"
    overridden = measured_run(f"collections/{case.replace('threads=2', 'threads=4')}", "--", "--exact",
                              "--iterations", "3", "--samples", "1", "--warmup-ms", "0")
    assert all(o["operations"] == 12 for o in overridden["observations"]), overridden
    del environment["AIRBUG_BENCH_THREADS"]
assert "workers must be 1..256" in run("--", "--threads", "257", "--list", success=False)
automatic = run("collections/contended/", "--", "--threads", "0", "--dry-run")
auto_case = json.JSONDecoder().raw_decode(automatic.lstrip())[0]["cases"][0]
assert 1 <= int(auto_case["contract"]["threads"]) <= 256, auto_case
matrix = measured_run("collections/contended/", "--", "--threads", "1,2", "--threads", "2,3", "--iterations", "3", "--samples", "1", "--warmup-ms", "0")
assert [c["id"] for c in matrix["cases"]] == [f"collections/contended/threads={n}" for n in [1, 2, 3]], matrix
assert [o["operations"] for o in matrix["observations"]] == [3, 6, 9], matrix
environment["AIRBUG_BENCH_THREADS"] = "2,1,2"
listed_matrix = run("collections/contended/", "--", "--list")
assert [line for line in listed_matrix.splitlines() if line.startswith("collections/")] == ["collections/contended/threads=2", "collections/contended/threads=1"], listed_matrix
del environment["AIRBUG_BENCH_THREADS"]
for invalid_threads in ["", "1,", "1,-2", "1,257"]:
    assert run("--", "--threads", invalid_threads, "--list", success=False)
print("cargo bench: runtime worker overrides, wave normalization and CLI/environment precedence passed")
for mode in ["linear", "auto"]:
    sampled = measured_run("collections/sum", "--", "--exact", "--samples", "4",
                           "--sample-ms", "1", "--warmup-ms", "0", "--sampling", mode)
    operations = [o["operations"] for o in sampled["observations"]]
    actual = sampled["cases"][0]["contract"]["sampling.mode"]
    assert actual in ("flat", "linear"), sampled
    if mode == "linear" or actual == "linear":
        assert operations == [operations[0] * i for i in range(1, 5)], sampled
invalid = run("collections/sum", "--", "--exact", "--iterations", "3", "--sampling", "linear", success=False)
assert "fixed iterations cannot be combined" in invalid, invalid
invalid = run("collections/sum", "--", "--exact", "--max-time-ms", "0", success=False)
assert "max_time exhausted" in invalid, invalid
for accounting, expected in [("--exclude-external-time", "measured"), ("--include-external-time", "workload_wall")]:
    preview = run("collections/sum", "--", "--exact", "--dry-run", "--iterations", "10",
                  "--min-time-ms", "20", "--max-time-ms", "50", accounting)
    contract = json.JSONDecoder().raw_decode(preview.lstrip())[0]["cases"][0]["contract"]
    assert contract["sampling.iterations"] == "10", contract
    assert contract["sampling.min_time_ns"] == "20000000", contract
    assert contract["sampling.max_time_ns"] == "50000000", contract
    assert contract["sampling.time_accounting"] == expected, contract
print("cargo bench: fixed iterations across lifecycle variants, schedules, limits and dry-run contracts passed")
regex_list = run(r"^collections/(sum|sort_only/64)$", "--", "--regex", "--list")
regex_ids = [line for line in regex_list.splitlines() if line.startswith("collections/")]
assert regex_ids == ["collections/sum", "collections/sort_only/64"], regex_list
excluded = run(r"^collections/(sum|sort_only/64)$", "--", "--regex", "--list", "--exclude-exact", "collections/sum")
assert [line for line in excluded.splitlines() if line.startswith("collections/")] == ["collections/sort_only/64"], excluded
excluded = run("--", "--list", "--exclude-regex", r"^collections/(sum|sort)")
assert "collections/sum\n" not in excluded and "collections/sort_only/64\n" not in excluded, excluded
for args in [("[", "--", "--regex", "--list"), ("sum", "--", "--regex", "--exact", "--list"),
             ("--", "--exclude-regex", "(", "--list")]:
    invalid = run(*args, success=False)
    assert "regex" in invalid, invalid
selected = measured_run(r"^collections/(sum|parameterized_sum/8)$", "--", "--regex", "--test")
assert {c["id"] for c in selected["cases"]} == {"collections/sum", "collections/parameterized_sum/8"}, selected
print("cargo bench: regex selection, exact/regex exclusions and invalid patterns passed")
for order, expected in [("registration", ["64", "1024"]), ("lexical", ["1024", "64"]), ("natural", ["64", "1024"]), ("source", ["64", "1024"]), ("kind", ["64", "1024"])]:
    for reverse in [False, True]:
        flags = ["--sort", order, *(["--reverse"] if reverse else [])]
        ids = [f"collections/sort_only/{n}" for n in (list(reversed(expected)) if reverse else expected)]
        listed_order = run("collections/sort_only/", "--", "--list", *flags)
        assert [line for line in listed_order.splitlines() if line.startswith("collections/")] == ids, listed_order
        preview = run("collections/sort_only/", "--", "--dry-run", *flags)
        assert [c["id"] for c in json.JSONDecoder().raw_decode(preview.lstrip())[0]["cases"]] == ids, preview
        execution = measured_run("collections/sort_only/", "--", "--test", *flags)
        assert [c["id"] for c in execution["cases"]] == ids, execution
assert "sort must be" in run("--", "--sort", "unknown", "--list", success=False)
print("cargo bench: registration/lexical/natural/reverse order agrees in list, dry-run and execution")
numeric = run("collections/numeric_order/", "--", "--sort", "natural", "--list")
assert [line for line in numeric.splitlines() if line.startswith("collections/")] == [
    f"collections/numeric_order/{value}" for value in ["-10.0", "-2.0", "0.0", "1.01", "1.1"]
], numeric
print("cargo bench: source locations and signed/decimal arguments order correctly")
external = measured_run("collections/external/", "--")
assert [c["id"] for c in external["cases"]] == ["collections/external/from_file/3", "collections/external/from_file/7", "collections/external/nested/leaf"], external
assert all(c["contract"]["samples"] == "2" for c in external["cases"]), external
assert len(external["observations"]) == 6, external
print("cargo bench: reusable groups from another file preserve nested cases and local defaults")
# A library exports groups; its benchmark target consumes them across a crate boundary.
with tempfile.TemporaryDirectory(prefix="airbug-external-crate-") as directory:
    fixture = pathlib.Path(directory)
    (fixture / "src").mkdir()
    (fixture / "benches").mkdir()
    (fixture / "Cargo.toml").write_text(
        '[package]\nname = "external-fixture"\nversion = "0.0.0"\nedition = "2024"\n'
        '[dependencies]\nairbug-bench = { path = ' + json.dumps(str(ROOT / "bench")) + ', features = ["macros"] }\n'
        '[[bench]]\nname = "imported"\nharness = false\n'
    )
    (fixture / "src/lib.rs").write_text(
        '#[airbug_bench::group(name = "shared", samples = 2, warmup_ms = 0, items = 2)]\n'
        'pub mod shared { #[bench(args = [2usize, 4])] fn work(n: usize) -> usize { std::hint::black_box(n) } }\n'
    )
    (fixture / "benches/imported.rs").write_text(
        '#[airbug_bench::suite(groups = [external_fixture::shared])] mod imported {}\n'
    )
    result = subprocess.run(
        ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", "--test", "--json"],
        cwd=ROOT, capture_output=True, text=True,
        env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
    )
    assert result.returncode == 0, result.stdout + result.stderr
    payload = next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT="))
    imported = json.loads(payload)
    assert [c["id"] for c in imported["cases"]] == ["imported/shared/work/2", "imported/shared/work/4"], imported
    assert len(imported["observations"]) == 2, imported
    (fixture / "benches/imported.rs").write_text(
        '#[airbug_bench::suite(groups = [external_fixture::shared], samples = 5, iterations = 3, sample_ms = 1, ignore = true, bytes = 128, items = 9)] mod imported {}\n'
    )
    for flags, samples, iterations in [([], 2, 3), (["--samples", "1", "--iterations", "2"], 1, 2)]:
        result = subprocess.run(
            ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", "--json", "--include-ignored", *flags],
            cwd=ROOT, capture_output=True, text=True,
            env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
        )
        assert result.returncode == 0, result.stdout + result.stderr
        inherited = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
        assert len(inherited["observations"]) == 2 * samples, inherited
        assert all(o["operations"] == iterations for o in inherited["observations"]), inherited
        assert all(c["contract"]["ignored"] == "true" and c["contract"]["work.counter.bytes"] == "128" and c["contract"]["work.counter.items"] == "2" for c in inherited["cases"]), inherited
    print("cargo bench: imported group sampling inherits across crate boundaries and CLI overrides win")
    comparison_library = (fixture / "src/lib.rs").read_text()
    comparison_bench = (fixture / "benches/imported.rs").read_text()
    (fixture / "src/lib.rs").write_text(comparison_library.replace('name = "shared",', 'name = "shared", significance_level = 0.02, hypothesis_seed = 0, hypothesis_resamples = 128,'))
    (fixture / "benches/imported.rs").write_text('#[airbug_bench::suite(significance_level = 0.01, noise_threshold_percent = 8.0, hypothesis_seed = 19, hypothesis_resamples = 256, samples = 1, iterations = 1, warmup_ms = 0, groups = [external_fixture::shared])] mod imported {}\n')
    for override in [[], ["--significance-level", "0.03", "--noise-threshold-percent", "1"]]:
        result = subprocess.run(
            ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", "--json", "--baseline-lenient", "absent", "--baseline-store", str(fixture / "baselines"), *override],
            cwd=ROOT, capture_output=True, text=True,
            env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
        )
        assert result.returncode == 0, result.stdout + result.stderr
        report = json.loads(next(line.removeprefix("BENCH_BASELINE=") for line in result.stdout.splitlines() if line.startswith("BENCH_BASELINE=")))
        for case in report["report"]["cases"]:
            assert case["config"]["significance_level"] == (0.03 if override else 0.02), case
            assert case["config"]["noise_threshold_percent"] == (1.0 if override else 8.0), case
            assert case["config"]["hypothesis"] == {"resamples": 128, "seed": 0}, case
    (fixture / "src/lib.rs").write_text(comparison_library)
    (fixture / "benches/imported.rs").write_text(comparison_bench)
    print("cargo bench: comparison attributes inherit per-field across crates and CLI overrides only specified fields")
    original_bench = (fixture / "benches/imported.rs").read_text()
    (fixture / "benches/imported.rs").write_text(
        '#[airbug_bench::suite(timer = "cpu", overhead = "subtract", samples = 1, iterations = 1, warmup_ms = 0, groups = [external_fixture::shared])] mod imported {}\n'
    )
    for flags in [["--dry-run"], ["--timer", "os", "--overhead", "raw", "--json"], ["--timer", "os", "--json"]]:
        result = subprocess.run(
            ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", *flags],
            cwd=ROOT, capture_output=True, text=True,
            env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
        )
        assert result.returncode == 0, result.stdout + result.stderr
        if "--dry-run" in flags:
            payload = json.loads(result.stdout)
            assert payload["timer"] == "cpu", payload
            assert payload["overhead"] == "subtract", payload
            assert all(any(m["id"] == "wall.adjusted" for m in c["metrics"]) for c in payload["cases"]), payload
            assert all(case["contract"]["timer.clock"] == "cpu" for case in payload["cases"]), payload
        else:
            payload = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
            assert all(case["contract"]["timer.clock"] == "os" for case in payload["cases"]), payload
            expected_policy = "raw" if "--overhead" in flags else "subtract-v1"
            assert all(c["contract"]["timer.overhead_policy"] == expected_policy for c in payload["cases"]), payload
            assert any(o["metric"] == "wall.adjusted" for o in payload["observations"]) == (expected_policy == "subtract-v1"), payload
            if expected_policy == "subtract-v1":
                assert payload["provenance"]["timer.overhead_policy"] == "subtract requested; see case policy", payload
    profile_destination = fixture / "profile-clock-defaults"
    for override in [[], ["--overhead", "raw"]]:
        profiled = subprocess.run(
            ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", "--timer", "os", "--profile-time-ms", "1", "--output", str(profile_destination), *override],
            cwd=ROOT, capture_output=True, text=True,
            env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
        )
        if not override:
            assert profiled.returncode != 0 and "overhead compensation cannot combine" in profiled.stderr, profiled.stdout + profiled.stderr
            assert not profile_destination.exists()
        else:
            assert profiled.returncode == 0, profiled.stdout + profiled.stderr
            assert json.loads((profile_destination / "profile.json").read_text())["complete"]
    (fixture / "benches/imported.rs").write_text(
        '#[airbug_bench::suite(timer = "invalid")] mod imported { #[bench] fn work() {} }\n'
    )
    invalid = subprocess.run(
        ["cargo", "check", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline"],
        cwd=ROOT, capture_output=True, text=True,
        env=dict(environment, CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
    )
    assert invalid.returncode != 0 and "timer must be os or cpu" in invalid.stderr, invalid.stdout + invalid.stderr
    (fixture / "benches/imported.rs").write_text(
        '#[airbug_bench::suite(overhead = "invalid")] mod imported { #[bench] fn work() {} }\n'
    )
    invalid = subprocess.run(
        ["cargo", "check", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline"],
        cwd=ROOT, capture_output=True, text=True,
        env=dict(environment, CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
    )
    assert invalid.returncode != 0 and "overhead must be raw or subtract" in invalid.stderr, invalid.stdout + invalid.stderr
    (fixture / "benches/imported.rs").write_text(original_bench)
    print("cargo bench: imported timer/overhead inheritance, lazy CPU preview, CLI precedence and invalid attributes passed")
    original_library = (fixture / "src/lib.rs").read_text()
    (fixture / "src/lib.rs").write_text(original_library.replace("[2usize, 4]", "[2usize, 2]"))
    result = subprocess.run(
        ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", "--dry-run"],
        cwd=ROOT, capture_output=True, text=True,
        env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
    )
    assert result.returncode != 0, result.stdout + result.stderr
    assert "duplicate benchmark ID: imported/shared/work/2" in result.stderr, result.stderr
    (fixture / "src/lib.rs").write_text(original_library)
    manifest = (fixture / "Cargo.toml").read_text().replace('airbug-bench = { path =', 'ab = { package = "airbug-bench", path =')
    manifest += '\n[[bench]]\nname = "standalone"\nharness = false\n'
    (fixture / "Cargo.toml").write_text(manifest)
    for explicit in [False, True]:
        override = "crate = crate::api, " if explicit else ""
        (fixture / "src/lib.rs").write_text(
            'pub use ab as api;\n#[ab::group(' + override + 'name = "shared")]\n'
            'pub mod shared { #[ab::bench(args = [2usize, 4])] fn work(n: usize) -> usize { n } }\n'
        )
        (fixture / "benches/imported.rs").write_text(
            'pub use ab as api;\n#[ab::suite(' + override + 'groups = [external_fixture::shared])] mod imported {\n'
            '#[ab::bench(types = [u32], consts = [2], drop_output = "outside")] async fn typed<T: Default + Copy, const N: usize>() -> [T; N] { [T::default(); N] }\n'
            '#[ab::group(name = "renamed")] mod nested { #[ab::bench] fn leaf() {} } }\n'
        )
        (fixture / "benches/standalone.rs").write_text(
            'pub use ab as api;\n#[ab::bench(' + override + 'types = [u16, u32])] fn independent<T: Default>() -> T { T::default() }\n'
            'fn main() -> ab::Result<()> { let mut suite = ab::Suite::new("standalone"); register_independent(&mut suite); suite.main() }\n'
        )
        for target, expected_count in [("imported", 4), ("standalone", 2)]:
            result = subprocess.run(
                ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", target, "--offline", "--", "--test", "--json"],
                cwd=ROOT, capture_output=True, text=True,
                env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
            )
            assert result.returncode == 0, result.stdout + result.stderr
            parsed = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
            assert len(parsed["cases"]) == expected_count and len(parsed["observations"]) == expected_count, parsed
            if target == "imported":
                assert any(c["id"] == "imported/renamed/leaf" for c in parsed["cases"]), parsed
    (fixture / "benches/standalone.rs").write_text(
        '#[global_allocator] static ALLOC: ab::alloc::TrackingAllocator<std::alloc::System> = ab::alloc::TrackingAllocator::new(std::alloc::System);\n'
        '#[ab::suite] mod allocated { #[bench(allocator = &crate::ALLOC)] fn vector() -> Vec<u8> { vec![std::hint::black_box(7); std::hint::black_box(64)] } }\n'
    )
    result = subprocess.run(
        ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "standalone", "--offline", "--", "--test", "--json"],
        cwd=ROOT, capture_output=True, text=True,
        env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
    )
    assert result.returncode == 0, result.stdout + result.stderr
    allocated = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert len(allocated["observations"]) == 14, allocated
    assert next(o["value"] for o in allocated["observations"] if o["metric"] == "alloc.bytes") == "64", allocated
    summaries = json.loads(next(line.removeprefix("BENCH_SUMMARY=") for line in result.stdout.splitlines() if line.startswith("BENCH_SUMMARY=")))
    wall_summary = next(row for row in summaries if row["metric"] == "wall")
    assert wall_summary["associated_allocations"]["alloc.bytes"]["median"] == 64, wall_summary

    (fixture / "benches/imported.rs").write_text(
        '#[ab::suite] mod quick { #[bench(custom = true, quick = false)] fn exact(n: u64) -> std::time::Duration { '
        'assert!(std::env::var_os("QUICK_MUST_NOT_RUN").is_none()); std::time::Duration::from_nanos(n * 100) } }\n'
    )
    def quick_run(*flags, lazy=False, env_overrides=None):
        env = dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
        env.update(env_overrides or {})
        if lazy:
            env["QUICK_MUST_NOT_RUN"] = "1"
        return subprocess.run(["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", *flags], cwd=ROOT, capture_output=True, text=True, env=env)
    preserved_source = (fixture / "benches/imported.rs").read_text()
    for option in ["threads = 2", "executor = missing_executor()", "allocator = &MISSING_ALLOCATOR", 'setup_thread = "worker"', "input_bytes = |v| v.len() as u64", "input_items = |v| 1", "input_chars = |v| 1", "input_cycles = |v| 1"]:
        (fixture / "benches/imported.rs").write_text(f'#[ab::suite(groups = [external_fixture::cases], {option})] mod rejected {{}}')
        rejected = quick_run("--list")
        assert rejected.returncode != 0, option
        assert "defaults cannot yet cross imported groups" in rejected.stderr, rejected.stderr
        assert "declare these options on the imported group" in rejected.stderr, rejected.stderr
    (fixture / "benches/imported.rs").write_text('fn main() -> ab::Result<()> { let mut suite = ab::Suite::new("manual"); suite.bench_threads("work", 2, || panic!("must not execute")); suite.main() }')
    repeated = quick_run("--threads", "1", "--threads", "2", "--list")
    assert repeated.returncode != 0 and "thread lists require main_registered" in repeated.stderr, repeated.stderr
    single = quick_run("--threads", "3", "--dry-run")
    assert single.returncode == 0, single.stderr
    assert json.JSONDecoder().raw_decode(single.stdout.lstrip())[0]["cases"][0]["contract"]["threads"] == "3"
    (fixture / "benches/imported.rs").write_text('fn main() -> ab::Result<()> { let mut suite = ab::Suite::new("manual"); suite.registration_threads(&[1, 2])?; suite.main() }')
    mismatch = quick_run("--threads", "4", "--list")
    assert mismatch.returncode != 0 and "CLI thread matrix differs" in mismatch.stderr, mismatch.stderr
    print("cargo bench: manual registration rejects unapplied thread matrices and retains single overrides")
    (fixture / "benches/imported.rs").write_text(preserved_source)
    print("cargo bench: unsupported imported defaults fail explicitly before benchmark execution")
    adaptive = quick_run("--quick", "--quick-min-ms", "0", "--quick-max-ms", "1000", "--json")
    assert adaptive.returncode == 0, adaptive.stdout + adaptive.stderr
    measured = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in adaptive.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert [o["operations"] for o in measured["observations"]] == [1, 2], measured
    assert measured["cases"][0]["contract"]["quick.stop"] == "stable", measured
    for flags in [("--list",), ("--dry-run",)]:
        lazy = quick_run("--quick", *flags, lazy=True)
        assert lazy.returncode == 0, lazy.stdout + lazy.stderr
    preview = json.loads(quick_run("--quick", "--dry-run", lazy=True).stdout)
    assert preview["adaptive_quick"]["max_time_ms"] == 5000, preview
    assert preview["samples"] is None and preview["target_measured_ms_per_case"] is None and preview["warmup_ms"] == 0, preview
    assert preview["cases"][0]["contract"]["sampling.mode"] == "quick_adaptive", preview
    for flags in [("--iterations", "2"), ("--sampling", "linear"), ("--quick-relative-deviation", "NaN"), ("--quick-max-ms", "0"), ("--quick-min-ms", "6000"), ("--profile-time-ms", "10")]:
        invalid = quick_run("--quick", *flags, lazy=True)
        assert invalid.returncode != 0 and "QUICK_MUST_NOT_RUN" not in invalid.stderr, invalid.stdout + invalid.stderr
    smoke = quick_run("--quick", "--test", "--iterations", "2", "--json")
    assert smoke.returncode == 0, smoke.stdout + smoke.stderr
    smoke_result = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in smoke.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert len(smoke_result["observations"]) == 1 and smoke_result["observations"][0]["operations"] == 1, smoke_result
    source = (fixture / "benches/imported.rs").read_text()
    (fixture / "benches/imported.rs").write_text(source.replace("quick = false", "quick = true, quick_config = ab::QuickConfig { min_time: std::time::Duration::ZERO, ..Default::default() }"))
    attributed = quick_run("--json")
    assert attributed.returncode == 0, attributed.stdout + attributed.stderr
    attributed_result = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in attributed.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert [o["operations"] for o in attributed_result["observations"]] == [1, 2], attributed_result
    preview = json.loads(quick_run("--dry-run", lazy=True).stdout)
    assert preview["samples"] is None and preview["cases"][0]["contract"]["quick.min_time_ns"] == "0", preview
    print("cargo bench: adaptive quick CLI, lazy previews, conflicts and smoke precedence passed")

    (fixture / "src/lib.rs").write_text(
        '#[ab::group] pub mod plots { '
        '#[bench(custom = true)] fn inherited(_: u64) -> std::time::Duration { std::time::Duration::ZERO } '
        '#[bench(custom = true, summary_scale = "linear")] fn explicit(_: u64) -> std::time::Duration { std::time::Duration::ZERO } }\n'
    )
    (fixture / "benches/imported.rs").write_text(
        '#[ab::suite(summary_scale = "logarithmic", groups = [external_fixture::plots])] mod plotting {}\n'
    )
    plot_flags = ["--iterations", "1", "--samples", "2", "--warmup-ms", "0", "--resamples", "128"]
    for label, extra, missing in [("inherited", [], 1), ("override", ["--summary-scale", "linear"], 0)]:
        destination = fixture / f"plot-{label}"
        result = quick_run(*plot_flags, *extra, "--output", str(destination))
        assert result.returncode == 0, result.stdout + result.stderr
        html = (destination / "estimates.html").read_text()
        assert html.count("population contains nonpositive values; no subset was plotted") == missing, html
        assert "plotting/plots/explicit" in html and "plotting/plots/inherited" in html
        saved = json.loads((destination / "run.json").read_text())
        scales = json.loads(saved["provenance"]["airbug.presentation.summary_scales.v1"])
        assert scales["plotting/plots/explicit"] == "linear"
        assert scales["plotting/plots/inherited"] == ("logarithmic" if missing else "linear")
    original = (fixture / "benches/imported.rs").read_text()
    (fixture / "benches/imported.rs").write_text(original.replace('summary_scale = "logarithmic"', 'summary_scale = "log"'))
    invalid = quick_run("--list", lazy=True)
    assert invalid.returncode != 0 and "summary_scale must be linear or logarithmic" in invalid.stderr, invalid.stdout + invalid.stderr
    print("cargo bench: cross-crate summary scale defaults, explicit linear, CLI override and invalid attribute passed")
    (fixture / "src/lib.rs").write_text(
        '#[ab::group] pub mod rates { #[bench(custom = true, args = [1024u64, 16, 128], bytes = |n| n)] '
        'fn sized(iterations: u64, size: u64) -> std::time::Duration { std::time::Duration::from_nanos(iterations * size * 2) } }\n'
    )
    (fixture / "benches/imported.rs").write_text(
        '#[ab::suite(summary_family = "sizes", groups = [external_fixture::rates])] mod family {}\n'
    )
    line_output = fixture / "input-lines"
    line_result = quick_run("--iterations", "2", "--samples", "2", "--warmup-ms", "0", "--summary-parameter", "arg", "--summary-estimator", "mean", "--summary-scale", "logarithmic", "--output", str(line_output))
    assert "arithmetic mean of normalized observations" in (line_output / "report.html").read_text()
    assert "ratio of means" in (line_output / "report.html").read_text()
    assert line_result.returncode == 0, line_result.stdout + line_result.stderr
    line_html = (line_output / "report.html").read_text()
    assert "Throughput input summaries" in line_html and "MiB/s" in line_html
    assert line_html.count("Lines connect observed estimates") == 2
    assert "family family/rates/sizes / candidate" in line_html
    line_run = json.loads((line_output / "run.json").read_text())
    families = json.loads(line_run["provenance"]["airbug.presentation.summary_families.v1"])
    assert len(families) == 3 and set(families.values()) == {"family/rates/sizes"}
    for observation in line_run["observations"]:
        assert int(observation["value"]) == observation["operations"] * int(observation["case"].rsplit("/", 1)[1]) * 2
    print("cargo bench: cross-crate family attributes and dynamic-byte time/throughput line exports passed")
    (fixture / "benches/imported.rs").write_text('fn main() -> ab::Result<()> { let mut suite = ab::Suite::new("plot_defaults"); suite.plots(false); suite.bench("value", || std::hint::black_box(1u64)); suite.main() }')
    for mode, flags, expect_svg in [("default", [], False), ("on", ["--plots"], True), ("off", ["--no-plots"], False)]:
        destination = fixture / ("plot-" + mode)
        result = quick_run("--iterations", "2", "--samples", "2", "--warmup-ms", "0", "--output", str(destination), *flags)
        assert result.returncode == 0, result.stdout + result.stderr
        assert ("<svg" in (destination / "report.html").read_text()) == expect_svg
    assert quick_run("--plots", "--no-plots", "--list").returncode != 0
    print("cargo bench: suite plot default and explicit CLI enable/disable passed")
    measurement_source = (ROOT / "bench/examples/custom_measurement.rs").read_text().replace("airbug_bench::", "ab::")
    (fixture / "benches/imported.rs").write_text(measurement_source)
    measurement_store = fixture / "measurement-baselines"
    measurement_flags = ["--iterations", "130", "--samples", "3", "--warmup-ms", "0", "--baseline-store", str(measurement_store), "--json"]
    first = quick_run(*measurement_flags, "--save-baseline", "counter")
    assert first.returncode == 0, first.stdout + first.stderr
    first_run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in first.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    counter_samples = [o for o in first_run["observations"] if o["metric"] == "work_units"]
    assert len(counter_samples) == 3 and all(o["operations"] == 130 and o["value"] == "390" for o in counter_samples)
    measurement_output = fixture / "measurement-report"
    loaded = quick_run(*measurement_flags, "--load-baseline", "counter", "--baseline", "counter", "--resamples", "128", "--output", str(measurement_output))
    assert loaded.returncode == 0, loaded.stdout + loaded.stderr
    assert json.loads((measurement_output / "run.json").read_text())["observations"] == first_run["observations"]
    formatted = json.loads((measurement_output / "formatted.json").read_text())
    with (measurement_output / "formatted.csv").open(newline="") as csv_file:
        csv_rows = list(csv.DictReader(csv_file))
    assert len(csv_rows) == len(formatted[0]["machine"])
    for csv_row, machine_row in zip(csv_rows, formatted[0]["machine"]):
        assert csv_row["case"] == formatted[0]["case"] and csv_row["metric"] == "work_units"
        assert float(csv_row["value"]) == machine_row["value"]
        assert csv_row["unit"] == machine_row["unit"]
        assert csv_row["normalized_per_operation"] == "true"
        assert int(csv_row["sequence"]) == machine_row["sequence"]

    assert formatted[0]["human"][0]["value"] == 1 and formatted[0]["human"][0]["unit"] == "groups"
    assert formatted[0]["machine"][0]["value"] == 3 and formatted[0]["machine"][0]["unit"] == "units/op"
    assert formatted[0]["throughput"]["items"]["work_per_operation"] == 6
    assert formatted[0]["throughput"]["items"]["observations"][0]["value"] == 2
    assert formatted[0]["throughput"]["items"]["observations"][0]["unit"] == "items/unit"
    assert "Formatted throughput:" in (measurement_output / "report.html").read_text()
    assert json.loads(next(line.removeprefix("BENCH_FORMATTED=") for line in loaded.stdout.splitlines() if line.startswith("BENCH_FORMATTED="))) == formatted
    assert "Formatted metric:" in (measurement_output / "report.html").read_text()
    assert "formatted comparison (groups)" in (measurement_output / "report.html").read_text()
    assert "formatted throughput comparison: items (items/unit)" in (measurement_output / "report.html").read_text()
    comparison = json.loads((measurement_output / "comparison.json").read_text())
    assert any(row["metric"] == "work_units" for case in comparison["cases"] for row in case["comparisons"])
    estimates = json.loads((measurement_output / "estimates.json").read_text())
    counter_estimate = next(row for row in estimates["rows"] if row["metric"] == "work_units")
    assert counter_estimate["unit"] == "units"
    assert counter_estimate["estimates"]["mean"]["point"] == 3
    display_estimates = json.loads((measurement_output / "formatted-estimates.json").read_text())
    assert len(display_estimates["rows"]) == 1
    assert display_estimates["rows"][0]["unit"] == "groups"
    assert display_estimates["rows"][0]["estimates"]["mean"]["point"] == 1
    assert "groups" in (measurement_output / "formatted-estimates.html").read_text()
    report_html = (measurement_output / "report.html").read_text()
    for artifact in ("estimates.html", "formatted-estimates.html", "regression-comparison.html", "formatted.csv", "run.json"):
        assert f'href="{artifact}"' in report_html
        assert (measurement_output / artifact).is_file()


    assert "work_units" in (measurement_output / "report.html").read_text()
    assert "work_units" in (measurement_output / "regression-comparison.html").read_text()
    print("cargo bench: custom measurement across crate boundary, baseline reload, normalized estimates and HTML passed")
    attributed_source = measurement_source.split("fn main() -> Result<()>")[0] + r"""
thread_local! { static TICKS: Rc<Cell<u64>> = Rc::new(Cell::new(0)); }
fn shared_counter() -> Counter { TICKS.with(|c| Counter(c.clone())) }
#[ab::suite]
mod measured {
    #[bench(measurement = super::shared_counter(), formatter = super::Groups, items = 6)]
    fn work() { super::TICKS.with(|c| c.set(c.get() + 3)); }
}
"""
    (fixture / "benches/imported.rs").write_text(attributed_source)
    attributed = quick_run(*measurement_flags, "--no-history")
    assert attributed.returncode == 0, attributed.stdout + attributed.stderr
    attributed_format = json.loads(next(line.removeprefix("BENCH_FORMATTED=") for line in attributed.stdout.splitlines() if line.startswith("BENCH_FORMATTED=")))
    assert attributed_format[0]["human"][0]["value"] == 1
    assert attributed_format[0]["throughput"]["items"]["observations"][0]["value"] == 2
    for replacement, diagnostic in [
        ("threads = 2, measurement = super::shared_counter()", "measurement cannot yet combine"),
        ("formatter = super::Groups", "formatter requires measurement"),
    ]:
        broken = attributed_source.replace("measurement = super::shared_counter(), formatter = super::Groups", replacement)
        (fixture / "benches/imported.rs").write_text(broken)
        rejected = quick_run("--list", lazy=True)
        assert rejected.returncode != 0 and diagnostic in rejected.stderr, rejected.stdout + rejected.stderr
    (fixture / "benches/imported.rs").write_text(attributed_source)
    print("cargo bench: measurement/formatter attributes with renamed dependency and invalid combinations passed")
    for async_keyword in ["", "async "]:
        manual_source = attributed_source.replace("measurement = super::shared_counter(),", "custom = true, measurement = super::shared_counter(),").replace("fn work() { super::TICKS.with(|c| c.set(c.get() + 3)); }", async_keyword + "fn work(n: u64) -> u64 { n * 3 }")
        (fixture / "benches/imported.rs").write_text(manual_source)
        manual = quick_run("--iterations", "7", "--samples", "1", "--warmup-ms", "0", "--json", "--no-history")
        assert manual.returncode == 0, manual.stdout + manual.stderr
        manual_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in manual.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
        assert next(o for o in manual_data["observations"] if o["metric"] == "work_units")["value"] == "21"
        manual_format = json.loads(next(line.removeprefix("BENCH_FORMATTED=") for line in manual.stdout.splitlines() if line.startswith("BENCH_FORMATTED=")))
        assert manual_format[0]["human"][0]["value"] == 1
    (fixture / "benches/imported.rs").write_text(attributed_source)
    print("cargo bench: sync/async caller-defined custom totals preserve formatter output")

    sampling_env = {"AIRBUG_BENCH_SAMPLES": "2", "AIRBUG_BENCH_ITERATIONS": "3", "AIRBUG_BENCH_WARMUP_MS": "0", "AIRBUG_BENCH_SAMPLE_MS": "1", "AIRBUG_BENCH_EXCLUDE_EXTERNAL_TIME": "true"}
    env_run = quick_run("--json", "--no-history", env_overrides=sampling_env)
    assert env_run.returncode == 0, env_run.stdout + env_run.stderr
    env_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in env_run.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    env_samples = [o for o in env_data["observations"] if o["metric"] == "work_units"]
    assert len(env_samples) == 2 and all(o["operations"] == 3 and o["value"] == "9" for o in env_samples)
    assert env_data["cases"][0]["contract"]["sampling.time_accounting"] == "measured"
    cli_run = quick_run("--json", "--no-history", "--samples", "1", "--iterations", "7", "--include-external-time", env_overrides=dict(sampling_env, AIRBUG_BENCH_SAMPLES="invalid", AIRBUG_BENCH_ITERATIONS="invalid"))
    assert cli_run.returncode == 0, cli_run.stdout + cli_run.stderr
    cli_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in cli_run.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    cli_samples = [o for o in cli_data["observations"] if o["metric"] == "work_units"]
    assert len(cli_samples) == 1 and cli_samples[0]["operations"] == 7 and cli_samples[0]["value"] == "21"
    assert cli_data["cases"][0]["contract"]["sampling.time_accounting"] == "workload_wall"
    invalid_env = quick_run("--list", env_overrides={"AIRBUG_BENCH_SAMPLES": "invalid"})
    assert invalid_env.returncode != 0 and "AIRBUG_BENCH_SAMPLES" in invalid_env.stderr
    print("cargo bench: environment sampling, per-field CLI precedence and named diagnostics passed")
    console_flags = ["--iterations", "3", "--samples", "2", "--warmup-ms", "0", "--no-history"]
    quiet = quick_run(*console_flags, "--quiet")
    assert quiet.returncode == 0, quiet.stdout + quiet.stderr
    assert "median 3 units/op" in quiet.stdout
    assert "bench: " not in quiet.stderr and "Benchmark contract" not in quiet.stderr
    assert "| Case" not in quiet.stdout and "Formatted metric:" not in quiet.stdout
    verbose = quick_run(*console_flags, "--verbose")
    assert verbose.returncode == 0, verbose.stdout + verbose.stderr
    assert "Benchmark environment:" in verbose.stderr and "Benchmark contract measured/work:" in verbose.stderr
    assert "Formatted metric:" in verbose.stdout
    machine = quick_run(*console_flags, "--quiet", "--json")
    assert machine.returncode == 0, machine.stdout + machine.stderr
    assert "bench: " not in machine.stderr
    assert "BENCH_RESULT=" in machine.stdout and "BENCH_FORMATTED=" in machine.stdout
    assert quick_run("--quiet", "--verbose", "--list").returncode != 0
    assert quick_run("-q", "--list").stdout.strip() == "measured/work"
    print("cargo bench: quiet/verbose modes retain results and JSON, hide progress and reject conflicts")
    color_output = fixture / "color-report"
    colored = quick_run(*console_flags, "--color", "always", "--output", str(color_output), env_overrides={"NO_COLOR": "1"})
    assert colored.returncode == 0, colored.stdout + colored.stderr
    assert "\x1b[1;36m" in colored.stdout
    assert "\x1b" not in (color_output / "report.html").read_text()
    assert "\x1b" not in (color_output / "run.json").read_text()
    for choice in ["auto", "never"]:
        plain = quick_run(*console_flags, "--color", choice)
        assert plain.returncode == 0 and "\x1b" not in plain.stdout, plain.stdout + plain.stderr
    colored_json = quick_run(*console_flags, "--json", "--color", "always")
    assert colored_json.returncode == 0 and "\x1b" not in colored_json.stdout
    assert "BENCH_RESULT=" in colored_json.stdout
    assert quick_run("--list", "--color", "always").stdout.strip() == "measured/work"
    assert quick_run("--color", "sometimes", "--list").returncode != 0
    assert quick_run("--color", "always", "--color", "never", "--list").returncode != 0
    print("cargo bench: forced/plain/auto color, machine and artifact isolation, listing and invalid policies passed")
    tree = quick_run(*console_flags, "--output-format", "tree")
    assert tree.returncode == 0, tree.stdout + tree.stderr
    assert "└── measured" in tree.stdout and "└── work" in tree.stdout
    assert "work_units [candidate process 0]: median 3 units/op" in tree.stdout
    assert "| Case" not in tree.stdout
    tree_list = quick_run("--list", "--output-format", "tree")
    assert tree_list.returncode == 0 and tree_list.stdout.strip() == "└── measured\n    └── work"
    quiet_tree = quick_run(*console_flags, "--output-format", "tree", "--quiet")
    assert quiet_tree.returncode == 0 and "└── measured" in quiet_tree.stdout and "bench: " not in quiet_tree.stderr
    tree_json = quick_run(*console_flags, "--output-format", "tree", "--json")
    assert tree_json.returncode == 0 and "BENCH_RESULT=" in tree_json.stdout and "└──" not in tree_json.stdout
    assert quick_run("--list", "--output-format", "unknown").returncode != 0
    print("cargo bench: hierarchical result/list output, quiet tree and unchanged JSON passed")
    (fixture / "benches/imported.rs").write_text('const SHARED: &[u64] = &[2, 7]; #[ab::suite] mod slices { #[bench(args = super::SHARED, custom = true)] fn owned(n: u64, value: u64) -> std::time::Duration { std::time::Duration::from_nanos(n * value) } }')
    sliced = quick_run("--iterations", "3", "--samples", "1", "--warmup-ms", "0", "--json", "--no-history")
    assert sliced.returncode == 0, sliced.stdout + sliced.stderr
    slice_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in sliced.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert [o["value"] for o in slice_data["observations"]] == ["6", "21"]
    print("cargo bench: shared argument slice supplies owned Copy values across crate boundary")
    (fixture / "benches/imported.rs").write_text('const SHARED: &[usize] = &[2, 7]; #[ab::suite] mod slices { #[bench(args = super::SHARED, setup = |n: usize| vec![1u8; n], input_bytes = |v: &Vec<u8>| v.len() as u64)] fn prepared(v: &mut [u8]) { assert_eq!(v[0], 1); v[0] = 9; } }')
    prepared = quick_run("--iterations", "3", "--samples", "1", "--warmup-ms", "0", "--json", "--no-history")
    assert prepared.returncode == 0, prepared.stdout + prepared.stderr
    prepared_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in prepared.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert [o["work_totals"]["bytes"] for o in prepared_data["observations"]] == ["6", "21"]
    print("cargo bench: shared argument slice supports typed setup values and fresh inputs")

    (fixture / "benches/imported.rs").write_text(r"""
#[derive(Clone, Copy, Default)] struct Label(u64);
impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "len-{}", self.0) }
}
#[ab::suite] mod labels {
    #[bench(args = [super::Label(2), super::Label(7)], custom = true)]
    fn owned(n: u64, value: super::Label) -> std::time::Duration { std::time::Duration::from_nanos(n * value.0) }
    #[bench(types = [super::Label], args = [T::default()], custom = true)]
    fn generic<T: Copy + Default + ToString>(n: u64, _: T) -> std::time::Duration { std::time::Duration::from_nanos(n) }
}
""")
    labelled = quick_run("--iterations", "3", "--samples", "1", "--warmup-ms", "0", "--json", "--no-history")
    assert labelled.returncode == 0, labelled.stdout + labelled.stderr
    labelled_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in labelled.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert [o["value"] for o in labelled_data["observations"]] == ["6", "21", "3"]
    assert [c["contract"]["param.arg"] for c in labelled_data["cases"]] == ["len-2", "len-7", "len-0"]
    print("cargo bench: Display-only value and generic ToString argument labels passed")
    (fixture / "benches/imported.rs").write_text('#[ab::suite] mod runtime { #[bench(args = [2u64, 7], bytes = |n| n * 1048576, custom = true)] fn value(n: u64, _: u64) -> std::time::Duration { std::time::Duration::from_secs(n) } }')
    runtime_env = {"AIRBUG_BENCH_TIMER": "os", "AIRBUG_BENCH_SORT": "natural", "AIRBUG_BENCH_REVERSE": "true", "AIRBUG_BENCH_BYTES_FORMAT": "binary"}
    runtime_run = quick_run("--iterations", "3", "--samples", "1", "--warmup-ms", "0", "--json", "--no-history", env_overrides=runtime_env)
    assert runtime_run.returncode == 0, runtime_run.stdout + runtime_run.stderr
    runtime_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in runtime_run.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    assert [c["id"] for c in runtime_data["cases"]] == ["runtime/value/7", "runtime/value/2"]
    runtime_rates = json.loads(next(line.removeprefix("BENCH_THROUGHPUT=") for line in runtime_run.stdout.splitlines() if line.startswith("BENCH_THROUGHPUT=")))
    assert [r["unit"] for r in runtime_rates] == ["MiB", "MiB"]
    assert [r["values"] for r in runtime_rates] == [[7.0], [2.0]]
    overridden = quick_run("--list", "--timer", "os", "--sort", "registration", "--forward", "--bytes-format", "decimal", env_overrides={k: "invalid" for k in runtime_env})
    assert overridden.returncode == 0, overridden.stdout + overridden.stderr
    assert overridden.stdout.splitlines() == ["runtime/value/2", "runtime/value/7"]
    print("cargo bench: environment timer/sort/direction/byte formatting and CLI overrides passed")
    (fixture / "benches/imported.rs").write_text('#[ab::suite] mod counts { #[bench(args = [2usize, 7], setup = |n: usize| vec![1u8; n], input_bytes = |v: &Vec<u8>| v.len() as u64, input_items = |_: &Vec<u8>| 1, input_bits = |v: &Vec<u8>| ab::counters::bits_from_bytes(v.len() as u64))] fn work(v: &mut [u8]) { std::hint::black_box(v); } }')
    counter_run = quick_run("--iterations", "3", "--samples", "1", "--warmup-ms", "0", "--json", "--no-history", "--bytes-count", "13", "--bits-count", "104", env_overrides={"AIRBUG_BENCH_BITS_COUNT": "invalid", "AIRBUG_BENCH_BYTES_COUNT": "invalid", "AIRBUG_BENCH_CHARS_COUNT": "0", "AIRBUG_BENCH_CYCLES_COUNT": "7"})
    assert counter_run.returncode == 0, counter_run.stdout + counter_run.stderr
    counter_data = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in counter_run.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
    for case in counter_data["cases"]:
        assert case["contract"]["work.counter.bytes"] == "13" and "work.input.bytes" not in case["contract"]
        assert case["contract"]["work.counter.chars"] == "0" and case["contract"]["work.counter.cycles"] == "7"
        assert case["contract"]["work.input.items"] == "batch_total"
    assert all(c["contract"]["work.counter.bits"] == "104" and "work.input.bits" not in c["contract"] for c in counter_data["cases"])
    for observation in counter_data["observations"]:
        assert observation["work_totals"] == {"items": "3"}
    env_bits = quick_run("--dry-run", env_overrides={"AIRBUG_BENCH_BITS_COUNT": "0"})
    assert env_bits.returncode == 0, env_bits.stderr
    assert all(c["contract"]["work.counter.bits"] == "0" for c in json.JSONDecoder().raw_decode(env_bits.stdout.lstrip())[0]["cases"])
    assert quick_run("--bytes-count", "-1", "--list").returncode != 0
    print("cargo bench: runtime counters override dynamic units, preserve others and honor CLI/environment precedence")














print("cargo bench: library-exported groups register and execute across a crate boundary")
print("cargo bench: renamed dependency and explicit crate paths work in suites, groups and standalone generic/async benches")
output_dir = pathlib.Path(history.name) / "bootstrap-result"
output = run("collections/sum", "--", "--exact", "--iterations", "50", "--samples", "6", "--warmup-ms", "0",
             "--resamples", "200", "--confidence-level", "0.9", "--analysis-seed", "123", "--bootstrap-distributions", "--output", str(output_dir), "--json")
estimates = json.loads(next(line.removeprefix("BENCH_ESTIMATES=") for line in output.splitlines() if line.startswith("BENCH_ESTIMATES=")))
assert estimates == json.loads((output_dir / "estimates.json").read_text()), estimates
for statistic in ["mean", "median", "standard_deviation", "median_absolute_deviation"]:
    draws = estimates["rows"][0]["distributions"][statistic]
    assert len(draws) == 200, statistic
    ordered = sorted(draws)
    def percentile(fraction):
        position = fraction * (len(ordered) - 1)
        lower = int(position)
        return ordered[lower] + (ordered[min(lower + 1, len(ordered) - 1)] - ordered[lower]) * (position - lower)
    import math
    point = estimates["rows"][0]["estimates"][statistic]
    assert math.isclose(percentile(0.05), point["lower"], rel_tol=1e-12, abs_tol=1e-12)
    assert math.isclose(percentile(0.95), point["upper"], rel_tol=1e-12, abs_tol=1e-12)

assert sum(estimates["rows"][0]["outliers"]["counts"]) == 6, estimates
assert len(estimates["rows"][0]["outliers"]["points"]) == 6, estimates
html = (output_dir / "estimates.html").read_text()
assert "<svg" in html and "Tukey fences" in html and "No observations discarded" in html, html
assert ("Gaussian KDE bandwidth" in html or "point mass" in html) and "Observation rug at zero" in html, html
assert "<!--CHARTS-->" not in html and "<!--DETAILS-->" not in html, html
assert html.count("Bootstrap confidence interval") == 4, html
assert html.count("Original point estimate") == 4, html
for statistic in ["mean", "median", "standard deviation", "MAD"]:
    assert f"bootstrap {statistic} (" in html, statistic
assert estimates["config"] == {"resamples": 200, "confidence_level": 0.9, "seed": 123}, estimates
assert estimates["rows"][0]["units"] == 6, estimates
assert estimates["rows"][0]["estimates"]["mean"]["lower"] <= estimates["rows"][0]["estimates"]["mean"]["upper"], estimates
text_report = run("collections/sum", "--", "--test", "--resamples", "20")
assert "Bootstrap estimates" in text_report and "unavailable" in text_report, text_report
for flags in [["--bootstrap-distributions"], ["--bootstrap-distributions", "--json", "--test"], ["--bootstrap-distributions", "--json", "--bootstrap-distributions"]]:
    invalid = run("collections/unselected", "--", *flags, success=False)
    assert "unselected setup ran" not in invalid
for option, value in [("--resamples", "1"), ("--confidence-level", "1"), ("--confidence-level", "NaN")]:
    run("--", "--list", option, value, success=False)
print("cargo bench: bootstrap CLI configuration, JSON artifact, text output and validation passed")
regression_dir = pathlib.Path(history.name) / "regression-export"
regression_output = run("collections/sum", "--", "--exact", "--sampling", "linear", "--samples", "5", "--sample-ms", "1", "--warmup-ms", "0", "--resamples", "200", "--json", "--bootstrap-distributions", "--output", str(regression_dir))
regression_report = json.loads(next(line.removeprefix("BENCH_ESTIMATES=") for line in regression_output.splitlines() if line.startswith("BENCH_ESTIMATES=")))
regressions = regression_report["rows"][0]["regressions"]
assert len(regressions) == 1 and regressions[0]["fit"]["samples"] == 5, regression_report
assert regressions[0]["fit"]["slope"]["point"] > 0, regression_report
assert len(regressions[0]["slope_distribution"]) == 200
assert regressions[0]["fit"]["slope"]["lower"] <= regressions[0]["fit"]["slope"]["upper"], regression_report
assert json.loads((regression_dir / "estimates.json").read_text()) == regression_report
recorded = json.loads((regression_dir / "run.json").read_text())
paired = regressions[0]["samples"]
assert len(paired) == 5
expected_pairs = [{"operations": o["operations"], "total": float(o["value"])}
                  for o in recorded["observations"]
                  if o["metric"] == regression_report["rows"][0]["metric"]
                  and o["case"] == regression_report["rows"][0]["case"]
                  and o["variant"] == regression_report["rows"][0]["variant"]
                  and o["process"] == regressions[0]["process"]]
assert paired == expected_pairs, (paired, expected_pairs)
regression_html = (regression_dir / "estimates.html").read_text()
assert "<polygon" in regression_html and "Slope confidence interval" in regression_html
assert "Violin summaries" in regression_html and "normalized_observation" in regression_html
assert "process 0 regression" in regression_html and "not a prediction interval" in regression_html
assert "chart unavailable" not in regression_html
assert "bootstrap slope" in regression_html and "ns/operation" in regression_html
assert regression_html.count("Bootstrap confidence interval") == 5
print("cargo bench: regression JSON retains exact paired batches and HTML includes slope confidence geometry")
unicode = measured_run("collections/unicode_text/", "--", "--test")
assert len(unicode["cases"]) == 2, unicode
assert [(c["contract"]["work.counter.bytes"], c["contract"]["work.counter.chars"]) for c in unicode["cases"]] == [("0", "0"), ("6", "2")], unicode
print("cargo bench: empty/Unicode arguments retain zero counters and distinguish bytes from characters")
for byte_format in ["decimal", "binary"]:
    formatted_dir = pathlib.Path(history.name) / f"rates-{byte_format}"
    # Throughput needs a positive interval; one empty-string operation can be
    # shorter than the host timer resolution in smoke mode.
    formatted = run("collections/unicode_text/", "--", "--iterations", "1000", "--samples", "1", "--warmup-ms", "0", "--bytes-format", byte_format, "--json", "--output", str(formatted_dir))
    rates = json.loads(next(line.removeprefix("BENCH_THROUGHPUT=") for line in formatted.splitlines() if line.startswith("BENCH_THROUGHPUT=")))
    assert rates == json.loads((formatted_dir / "throughput.json").read_text()), rates
    assert len(rates) == 4, rates
    zero_rates = [r for r in rates if r["case"].endswith('/""')]
    assert {r["unit"] for r in zero_rates} == {"B", "chars"}, zero_rates
    assert all(all(value == 0 for value in r["values"]) for r in zero_rates), zero_rates
    byte_units = {"B", "KB", "MB", "GB", "TB", "PB", "EB"} if byte_format == "decimal" else {"B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"}
    assert all(r["unit"] in byte_units | {"chars"} for r in rates), rates
assert "bytes format must be" in run("--", "--bytes-format", "invalid", "--list", success=False)
print("cargo bench: decimal/binary byte formatting and throughput.json preserve zero and character rates")
history.cleanup()


# Per-case statistical settings survive imported groups and per-field CLI overrides.
with tempfile.TemporaryDirectory(prefix="airbug-bootstrap-options-") as directory:
    fixture = pathlib.Path(directory)
    (fixture / "src").mkdir()
    (fixture / "benches").mkdir()
    (fixture / "Cargo.toml").write_text(
        '[package]\nname = "bootstrap-options-fixture"\nversion = "0.0.0"\nedition = "2024"\n'
        '[dependencies]\nairbug-bench = { path = ' + json.dumps(str(ROOT / "bench")) + ' }\n'
        '[[bench]]\nname = "cases"\nharness = false\n'
    )
    (fixture / "src/lib.rs").write_text(
        '#[airbug_bench::group(confidence_level = 0.9, analysis_seed = 7)] '
        'pub mod imported { #[bench(custom = true)] fn work(n: u64) -> std::time::Duration { std::time::Duration::from_nanos(n * 3) } }'
    )
    (fixture / "benches/cases.rs").write_text(
        '#[airbug_bench::suite(groups = [bootstrap_options_fixture::imported], resamples = 32, confidence_level = 0.8, samples = 4, iterations = 3, warmup_ms = 0)] '
        'mod cases { #[bench(custom = true, confidence_level = 0.99, analysis_seed = 0)] fn local(n: u64) -> std::time::Duration { std::time::Duration::from_nanos(n * 5) } }'
    )
    for flags, count, capture in [([], 32, False), ([], 32, True), (["--resamples", "64"], 64, True)]:
        completed = subprocess.run(
            ["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--offline", "--bench", "cases", "--", "--no-history", "--json", *(["--bootstrap-distributions"] if capture else []), *flags],
            cwd=ROOT, capture_output=True, text=True,
            env=dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external")),
        )
        assert completed.returncode == 0, completed.stdout + completed.stderr
        report = json.loads(next(line.removeprefix("BENCH_ESTIMATES=") for line in completed.stdout.splitlines() if line.startswith("BENCH_ESTIMATES=")))
        saved_run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in completed.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
        saved_settings = json.loads(saved_run["provenance"]["airbug.analysis.bootstrap.v1"])
        assert saved_settings["cases"] == report["case_configs"]
        assert report["case_configs"]["cases/local"] == {"resamples": count, "confidence_level": 0.99, "seed": 0}
        assert report["case_configs"]["cases/imported/work"] == {"resamples": count, "confidence_level": 0.9, "seed": 7}
        if capture:
            assert all(len(row["distributions"]["mean"]) == count for row in report["rows"])
        else:
            assert all("distributions" not in row for row in report["rows"])
    print("cargo bench: imported bootstrap defaults and per-field CLI overrides passed")
