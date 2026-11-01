#!/usr/bin/env python3
"""Real Cargo verification that persistent Airbug workers leave Rayon isolated."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-worker-runtime-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name = "worker-runtime"
version = "0.1.0"
edition = "2021"
[workspace]
[dev-dependencies]
ab = {{ package = "airbug-bench", path = {json.dumps(str(ROOT / 'bench'))} }}
rayon-core = "1.13"
[[bench]]
name = "runtime"
harness = false
''')
    (root / "benches/runtime.rs").write_text('''
use std::{cell::Cell, collections::HashSet, rc::Rc, sync::{Arc, Mutex}};
static EXITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
struct WorkerExit;
impl Drop for WorkerExit {
    fn drop(&mut self) { EXITS.fetch_add(1, std::sync::atomic::Ordering::SeqCst); }
}
thread_local! { static EXIT: WorkerExit = const { WorkerExit }; }
thread_local! { static CALLS: Cell<usize> = const { Cell::new(0) }; }
fn main() -> ab::Result<()> {
    let workers = Arc::new(Mutex::new(HashSet::new()));
    let captured = workers.clone();
    let mut suite = ab::Suite::new("runtime");
    suite.bench_threads_with_local_input("nested", 2, || Rc::new(()), move |_| {
        EXIT.with(|_| ());
        assert_eq!(rayon_core::current_thread_index(), None);
        assert_eq!(rayon_core::current_num_threads(), 3);
        let nested = rayon_core::join(|| rayon_core::current_num_threads(), || 42);
        assert_eq!(nested, (3, 42));
        captured.lock().unwrap().insert(std::thread::current().id());
        CALLS.with(|calls| calls.set(calls.get() + 1));
    }, ab::DropPolicy::OutsideTiming);
    suite.main()?;
    assert_eq!(workers.lock().unwrap().len(), 2);
    assert_eq!(EXITS.load(std::sync::atomic::Ordering::SeqCst), 2, "workers must exit before main returns");
    Ok(())
}
''')
    env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
    env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", RAYON_NUM_THREADS="3",
               CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
    output = subprocess.run(["cargo", "bench", "--manifest-path", str(root / "Cargo.toml"),
        "--bench", "runtime", "--", "--samples", "2", "--iterations", "65", "--warmup-ms", "5",
        "--no-bootstrap", "--json"], env=env, capture_output=True, text=True, encoding="utf-8", timeout=180)
    assert output.returncode == 0, output.stdout + output.stderr
    run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in output.stdout.splitlines()
                          if line.startswith("BENCH_RESULT=")))
    assert run["cases"][0]["contract"]["threads.executor"] == "os-pool-v1", run
    assert len(run["worker_timings"]) == 8, run
    assert sum(row["operations"] for row in run["worker_timings"]) == 260, run
    profile = subprocess.run(["cargo", "bench", "--manifest-path", str(root / "Cargo.toml"),
        "--bench", "runtime", "--", "--profile-time-ms", "3", "--iterations", "2",
        "--output", str(root / "profile"), "--json"], env=env, capture_output=True,
        text=True, encoding="utf-8", timeout=180)
    assert profile.returncode == 0, profile.stdout + profile.stderr
    report = json.loads(next(line.removeprefix("PROFILE_RESULT=") for line in profile.stdout.splitlines()
                             if line.startswith("PROFILE_RESULT=")))
    assert report["complete"] and len(report["cases"]) == 1, report
print("cargo bench: stable OS workers across warmup/waves; nested Rayon keeps its separate three-thread global pool")
