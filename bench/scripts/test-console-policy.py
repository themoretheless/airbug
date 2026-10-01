#!/usr/bin/env python3
"""Check native console policies with a Cargo-built deterministic harness."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-console-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="console-policy"
version="0.1.0"
edition="2021"
[workspace]
[dev-dependencies]
airbug-bench={{path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="console"
harness=false
''')
    (root / "benches/console.rs").write_text('''#[airbug_bench::suite]
mod cases {
    #[bench(custom = true)]
    fn fixed(iterations: u64) -> std::time::Duration {
        assert!(std::env::var_os("NO_WORK").is_none(), "workload executed before output validation");
        std::time::Duration::from_nanos(iterations * 3)
    }
}
''')
    env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
    env.update(CARGO_TARGET_DIR=str(ROOT / "target/parity-external"), AIRBUG_DASHBOARD="0", NO_COLOR="1")
    built = subprocess.run(["cargo", "bench", "--offline", "--no-run", "--message-format=json"], cwd=root, env=env, capture_output=True, text=True, check=True)
    executable = next(item["executable"] for line in built.stdout.splitlines() if (item := json.loads(line)).get("executable"))
    common = ["--bench", "--iterations", "3", "--samples", "2", "--warmup-ms", "0", "--no-history", "--resamples", "16"]
    def run(*args):
        result = subprocess.run([executable, *common, *args], cwd=root, env=env, capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
        return result
    ordinary = run()
    quiet = run("--quiet")
    verbose = run("--verbose")
    assert "bench: " in ordinary.stderr and "Benchmark environment:" not in ordinary.stderr
    assert "median 3 ns/op" in quiet.stdout and "bench: " not in quiet.stderr
    assert "| Case" not in quiet.stdout and "Benchmark contract" not in quiet.stderr
    assert "Benchmark environment:" in verbose.stderr and "Benchmark contract cases/fixed:" in verbose.stderr
    for mode in [[], ["--quiet"], ["--verbose"]]:
        machine = run(*mode, "--json", "--color", "always")
        assert "\x1b" not in machine.stdout
        lines = machine.stdout.splitlines()
        assert all(line.startswith("BENCH_") for line in lines), machine.stdout
        data = json.loads(next(line.split("=", 1)[1] for line in lines if line.startswith("BENCH_RESULT=")))
        assert len(data["observations"]) == 2
        assert all(o["value"] == "9" and o["operations"] == 3 for o in data["observations"])
    for choice in ["auto", "never"]:
        assert "\x1b" not in run("--color", choice).stdout
    destination = root / "report"
    colored = run("--color", "always", "--output", str(destination))
    assert "\x1b" in colored.stdout
    for path in destination.rglob("*"):
        if path.is_file():
            assert b"\x1b" not in path.read_bytes(), path
    assert run("--list", "--color", "always").stdout.strip() == "cases/fixed"
    tree = run("--quiet", "--output-format", "tree")
    assert "└── cases" in tree.stdout and "bench: " not in tree.stderr
    assert "95.00% interval" not in tree.stdout
    for kind in ("directory", "file"):
        occupied = root / ("occupied-" + kind)
        if kind == "directory":
            occupied.mkdir()
            sentinel = occupied / "previous.txt"
        else:
            sentinel = occupied
        sentinel.write_text("previous result")
        rejected = subprocess.run([executable, *common, "--output", str(occupied)], cwd=root,
                                  env=dict(env, NO_WORK="1"), capture_output=True, text=True, timeout=120)
        assert rejected.returncode != 0
        assert "output path already exists" in rejected.stderr, rejected.stderr
        assert "workload executed before output validation" not in rejected.stderr
        assert sentinel.read_text() == "previous result"
        preview = subprocess.run([executable, *common, "--output", str(occupied), "--dry-run"], cwd=root,
                                 env=dict(env, NO_WORK="1"), capture_output=True, text=True, timeout=120)
        assert preview.returncode == 0, preview.stderr
    assert not (root / ".airbug").exists()
    print("Console policy: default/quiet/verbose, machine output, color override, plain artifacts, listing and quiet tree passed")
