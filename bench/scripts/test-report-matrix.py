#!/usr/bin/env python3
"""The report matrix as one executable check: every entry point crossed with the report options.

Direct harness, standalone `report` / `compare`, and history — raw and custom metrics, baseline,
filters, --no-html / --no-plots, linear and logarithmic summaries. Each row states the behaviour
it expects; a row that fails names itself.

    cargo build --release -p cargo-airbug-bench -p airbug-bench --examples
    python3 bench/scripts/test-report-matrix.py
"""
import json, os, re, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXE = ".exe" if os.name == "nt" else ""
SORT = ROOT / f"target/release/examples/sort{EXE}"
CUSTOM = ROOT / f"target/release/examples/custom_measurement{EXE}"
CLI = ROOT / f"target/release/cargo-airbug-bench{EXE}"
rows = []

def run(*args, cwd):
    done = subprocess.run([str(a) for a in args], cwd=cwd, capture_output=True, text=True,
                          encoding="utf-8", env=dict(os.environ, AIRBUG_DASHBOARD="0"), timeout=600)
    assert done.returncode == 0, f"{args[0]} {args[1:]}\n{done.stdout[-800:]}\n{done.stderr[-800:]}"
    return done.stdout

def row(entry, option, behaviour, check):
    assert check, f"{entry} · {option}: expected {behaviour}"
    rows.append((entry, option, behaviour))

def figures(path):
    return len(re.findall("<svg", Path(path).read_text(encoding="utf-8")))

with tempfile.TemporaryDirectory(prefix="airbug-report-matrix-") as directory:
    w = Path(directory)
    quick = ["--samples", "3", "--warmup-ms", "0", "--resamples", "64", "--quiet",
             "--history-dir", w / "history", "--baseline-store", w / "baselines"]
    def direct(name, *options, exe=SORT):
        run(exe, *quick, *options, "--output", w / name, cwd=w)
        return w / name
    files = lambda d: {p.name for p in d.iterdir()}

    a = direct("plain")
    row("direct", "default", "run.json, report.html, estimates.html/json, summary.json, comparison.json",
        {"run.json", "report.html", "estimates.html", "estimates.json", "summary.json", "comparison.json"} <= files(a))
    row("direct", "default", "one raw-observation figure per case", figures(a / "report.html") == 6)
    b = direct("no-html", "--no-html")
    row("direct", "--no-html", "machine files kept, no HTML at all",
        {"run.json", "estimates.json", "summary.json"} <= files(b) and not any(n.endswith(".html") for n in files(b)))
    c = direct("no-plots", "--no-plots")
    row("direct", "--no-plots", "report.html without figures", figures(c / "report.html") == 0)
    f = direct("filtered", "sort/stable")
    cases = [c["id"] for c in json.loads((f / "run.json").read_text(encoding="utf-8"))["cases"]]
    row("direct", "filter", "only matching cases measured and reported",
        len(cases) == 3 and all(c.startswith("sort/stable") for c in cases))
    g = direct("log-summary", "--summary-parameter", "elements", "--summary-scale", "logarithmic")
    row("direct", "--summary-parameter/--summary-scale logarithmic", "one extra summary figure over the parameter",
        figures(g / "report.html") == 7 and "elements vs wall" in (g / "report.html").read_text(encoding="utf-8"))
    h = direct("linear-summary", "--summary-parameter", "elements", "--summary-scale", "linear")
    row("direct", "--summary-scale linear", "the same figure on a linear axis", figures(h / "report.html") == 7)
    direct("saved", "--save-baseline", "main")
    row("direct", "--save-baseline", "named baseline stored", (w / "baselines/baselines").is_dir())
    e = direct("against", "--baseline", "main")
    row("direct", "--baseline", "comparison artifacts next to the report",
        {"comparison.json", "regression-comparison.html", "relative-distributions.json"} <= files(e))
    cm = direct("custom", exe=CUSTOM)
    text = (cm / "report.html").read_text(encoding="utf-8")
    row("direct", "custom measurement + formatter", "custom metric and its formatted unit in the report",
        "work_units" in text and "groups" in text)
    before = sorted(p.name for p in (w / "history").rglob("*"))
    direct("no-history", "--no-history")
    row("history", "--no-history", "history directory untouched",
        before == sorted(p.name for p in (w / "history").rglob("*")))
    row("history", "default", "each run appended to the history", len(before) > 0)

    for kind in ("html", "md", "json"):
        run(CLI, "report", a, "-o", w / f"standalone.{kind}", cwd=w)
        row("report", f"-o .{kind}", "standalone document from a saved run", (w / f"standalone.{kind}").stat().st_size > 0)
    run(CLI, "report", e, "--baseline", a, "-o", w / "baseline.html", cwd=w)
    row("report", "--baseline", "comparison document", "baseline" in (w / "baseline.html").read_text(encoding="utf-8").lower())
    run(CLI, "report", cm, "-o", w / "custom.md", cwd=w)
    row("report", "custom measurement", "custom metric survives the saved run", "work_units" in (w / "custom.md").read_text(encoding="utf-8"))
    row("compare", "text", "comparison summary on stdout", "bench comparison" in run(CLI, "compare", a, e, cwd=w))
    run(CLI, "compare", a, e, "--html", "-o", w / "welch.html", cwd=w)
    row("compare", "--html", "null-distribution figures", figures(w / "welch.html") > 0)
    run(CLI, "compare", a, e, "--html", "--no-plots", "-o", w / "tables.html", cwd=w)
    row("compare", "--html --no-plots", "tables only", figures(w / "tables.html") == 0)

width = max(len(r[1]) for r in rows)
for entry, option, behaviour in rows:
    print(f"{entry:8} {option:{width}}  {behaviour}")
print(f"Report matrix: {len(rows)} combinations passed")
