#!/usr/bin/env python3
"""Exercise Cargo's real argument forwarding and the attribute-generated harness."""
import json
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
    def quick_run(*flags, lazy=False):
        env = dict(environment, AIRBUG_DASHBOARD="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
        if lazy:
            env["QUICK_MUST_NOT_RUN"] = "1"
        return subprocess.run(["cargo", "bench", "--manifest-path", str(fixture / "Cargo.toml"), "--bench", "imported", "--offline", "--", *flags], cwd=ROOT, capture_output=True, text=True, env=env)
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
    assert formatted[0]["human"][0]["value"] == 1 and formatted[0]["human"][0]["unit"] == "groups"
    assert formatted[0]["machine"][0]["value"] == 3 and formatted[0]["machine"][0]["unit"] == "units/op"
    assert json.loads(next(line.removeprefix("BENCH_FORMATTED=") for line in loaded.stdout.splitlines() if line.startswith("BENCH_FORMATTED="))) == formatted
    assert "Formatted metric:" in (measurement_output / "report.html").read_text()
    comparison = json.loads((measurement_output / "comparison.json").read_text())
    assert any(row["metric"] == "work_units" for case in comparison["cases"] for row in case["comparisons"])
    estimates = json.loads((measurement_output / "estimates.json").read_text())
    counter_estimate = next(row for row in estimates["rows"] if row["metric"] == "work_units")
    assert counter_estimate["unit"] == "units"
    assert counter_estimate["estimates"]["mean"]["point"] == 3
    assert "work_units" in (measurement_output / "report.html").read_text()
    assert "work_units" in (measurement_output / "regression-comparison.html").read_text()
    print("cargo bench: custom measurement across crate boundary, baseline reload, normalized estimates and HTML passed")





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
