#!/usr/bin/env python3
"""Cargo smoke mode in both profiles, explicit measurements, and protocol runner."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-cargo-test-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="cargo-test-fixture"
version="0.1.0"
edition="2021"
[workspace]
[dev-dependencies]
ab={{package="airbug-bench",path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="cases"
harness=false
''')
    (root / "benches/cases.rs").write_text('''
#[ab::suite(samples=2, iterations=3, warmup_ms=0)]
mod cases {
    #[bench(setup=|| {
        assert!(std::env::var_os("NO_WORK").is_none());
        vec![3,1,2]
    })]
    fn sort(input: &mut [i32]) {
        assert!(std::env::var_os("NO_WORK").is_none());
        input.sort_unstable();
    }
    #[bench]
    fn other() { assert!(std::env::var_os("ONLY_SORT").is_none()); }
}
''')
    env = {k:v for k,v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
    env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
    def call(command, extra=None):
        result = subprocess.run(command, cwd=root, env=dict(env, **(extra or {})), capture_output=True, text=True)
        assert result.returncode == 0, result.stdout+result.stderr
        return result.stdout
    def run(output):
        return json.loads(next(line.removeprefix("BENCH_RESULT=") for line in output.splitlines() if line.startswith("BENCH_RESULT=")))
    for profile in [[], ["--release"]]:
        command = ["cargo", "test", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--benches", *profile, "--"]
        default_output = call(command, {"AIRBUG_BENCH_HISTORY":"1"})
        assert "Report:" not in default_output
        assert not (root / "target/airbug-bench").exists()
        result = run(call([*command, "--json"]))
        assert len(result["cases"]) == 2
        assert all(o["operations"] == 1 for o in result["observations"])
        selected = run(call([*command, "cases/sort", "--exact", "--json"], {"ONLY_SORT":"1"}))
        assert len(selected["cases"]) == 1
        assert selected["cases"][0]["id"] == "cases/sort"
        listed = call([*command, "--list"], {"NO_WORK":"1"})
        assert "cases/sort" in listed
        call([*command, "--version"], {"NO_WORK":"1"})
    bench = ["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "cases", "--"]
    measured = run(call([*bench, "--json"]))
    assert len(measured["observations"]) == 4
    assert all(o["operations"] == 3 for o in measured["observations"])
    smoke = run(call([*bench, "--test", "--json"]))
    assert all(o["operations"] == 1 for o in smoke["observations"])
    builds = call(["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "cases", "--no-run", "--message-format=json"])
    binary = next(json.loads(line)["executable"] for line in builds.splitlines() if json.loads(line).get("executable"))
    assert all(o["operations"] == 1 for o in run(call([binary,"--json"]))["observations"])
    assert all(o["operations"] == 3 for o in run(call([binary,"--bench","--json"]))["observations"])
    assert all(o["operations"] == 1 for o in run(call([binary,"--test","--json"], {"AIRBUG_BENCH_RUNNER":"1"}))["observations"])
    output = root / "runner"
    call(["cargo", "run", "--offline", "--manifest-path", str(ROOT / "Cargo.toml"), "-p", "cargo-airbug-bench", "--", "run", "--program", binary, "--protocol", "--repetitions", "1", "--no-ui", "--output", str(output), "--", "--json"])
    saved = json.loads((output / "run.json").read_text())
    assert len(saved["observations"]) == 4
    assert all(o["operations"] == 3 for o in saved["observations"])
    assert not (root / "target/airbug-bench/reports").exists()
print("Cargo test benches: debug/release smoke, filters, lazy list/version, cargo bench, direct executable and protocol runner passed")
