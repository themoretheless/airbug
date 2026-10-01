#!/usr/bin/env python3
"""Verify manual worker families through Cargo, including environment overrides."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-manual-matrix-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name = "manual-matrix"
version = "0.1.0"
edition = "2021"
[workspace]
[dev-dependencies]
ab = {{ package = "airbug-bench", path = {json.dumps(str(ROOT / 'bench'))} }}
[[bench]]
name = "manual"
harness = false
''')
    (root / "benches/manual.rs").write_text('''
fn main() -> ab::Result<()> {
    ab::Suite::new("manual").main_registered(|suite| {
        suite.thread_matrix("work", [1usize, 2, 1], |suite, id, workers| {
            suite.bench_threads(id, workers, || {
                assert!(std::env::var_os("MUST_NOT_RUN").is_none());
                std::hint::black_box(42)
            });
        }).unwrap();
    })
}
''')
    def run(flags, counts, lazy=False, extra_env=None, iterations=1, waves=None):
        env = dict(os.environ, AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0",
                   CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
        env.update(extra_env or {})
        if lazy:
            env["MUST_NOT_RUN"] = "1"
        result = subprocess.run(["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"),
                                 "--bench", "manual", "--", *flags], capture_output=True, text=True, env=env)
        assert result.returncode == 0, result.stdout + result.stderr
        if lazy:
            cases = json.JSONDecoder().raw_decode(result.stdout.lstrip())[0]["cases"]
        else:
            report = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
            cases = report["cases"]
            assert [row["operations"] for row in report["observations"] if row["metric"] == "wall"] == [n * iterations for n in counts]
            if waves is not None:
                for case, workers in zip(cases, counts):
                    records = [row for row in report["worker_timings"] if row["case"] == case["id"]]
                    assert len(records) == workers * waves, records
                    assert {row["worker"] for row in records} == set(range(workers)), records
                    for worker in range(workers):
                        slots = [row for row in records if row["worker"] == worker]
                        assert {row["wave"] for row in slots} == set(range(waves)), slots
                        assert sum(row["operations"] for row in slots) == iterations, slots
        assert [case["id"] for case in cases] == [f"manual/work/threads={n}" for n in counts]
        assert [case["contract"]["threads"] for case in cases] == list(map(str, counts))
    run(["--dry-run"], [1, 2], lazy=True)
    run(["--threads", "3,1,3", "--dry-run"], [3, 1], lazy=True)
    run(["--threads", "2", "--threads", "1", "--test", "--json"], [2, 1])
    run(["--test", "--json"], [3, 1], extra_env={"AIRBUG_BENCH_THREADS": "3,1"})
    run(["--threads", "2", "--test", "--json"], [2], extra_env={"AIRBUG_BENCH_THREADS": "3,1"})
    (root / "benches/manual.rs").write_text('''
use std::sync::atomic::{AtomicUsize, Ordering};
static SETUPS: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
fn main() -> ab::Result<()> {
    ab::Suite::new("manual").main_registered(|suite| {
        suite.thread_matrix("work", [1usize, 2], |suite, id, workers| {
            suite.bench_threads_with_input(id, workers, move || {
                assert!(std::env::var_os("MUST_NOT_RUN").is_none());
                assert_ne!(std::env::var("FORBIDDEN_WORKERS").ok(), Some(workers.to_string()));
                SETUPS.fetch_add(1, Ordering::SeqCst);
                vec![3usize, 1, 2]
            }, |input| {
                assert_eq!(*input, [3, 1, 2]);
                input.sort_unstable();
                CALLS.fetch_add(1, Ordering::SeqCst);
            }, ab::DropPolicy::OutsideTiming);
        }).unwrap();
    })?;
    let expected: usize = std::env::var("EXPECTED_CALLS").unwrap().parse().unwrap();
    assert_eq!(SETUPS.load(Ordering::SeqCst), expected);
    assert_eq!(CALLS.load(Ordering::SeqCst), expected);
    Ok(())
}
''')
    run(["--dry-run"], [1, 2], lazy=True, extra_env={"EXPECTED_CALLS": "0"})
    run(["manual/work/threads=1", "--exact", "--test", "--json"], [1],
        extra_env={"EXPECTED_CALLS": "1", "FORBIDDEN_WORKERS": "2"})
    run(["--threads", "2,1", "--iterations", "65", "--samples", "1", "--warmup-ms", "0", "--json"],
        [2, 1], iterations=65, waves=2, extra_env={"EXPECTED_CALLS": "195"})
print("manual worker matrix: defaults, lazy previews, repeated CLI flags, environment, CLI precedence, filtered fresh inputs and multi-wave execution passed")
