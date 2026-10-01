#!/usr/bin/env python3
"""Check threaded batch attributes, inherited defaults and runtime worker overrides."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-thread-batches-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="thread-batches"
version="0.1.0"
edition="2021"
[workspace]
[dependencies]
ab={{package="airbug-bench",path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="batches"
harness=false
''')
    (root / "benches/batches.rs").write_text('''
#[ab::suite(threads=2, batch=ab::BatchPolicy::Iterations(2.try_into().unwrap()), samples=1, iterations=5, warmup_ms=0)]
mod cases {
    #[bench]
    fn inherited() -> Vec<u8> { assert!(std::env::var_os("MUST_NOT_RUN").is_none()); vec![1; 8] }
    #[bench(batch=ab::BatchPolicy::Batches(1.try_into().unwrap()), drop_output="inside")]
    fn overridden() -> Vec<u8> { vec![1; 8] }
    #[bench(setup=|| std::rc::Rc::new(1))]
    async fn asynchronous(v: &mut std::rc::Rc<i32>) -> std::rc::Rc<i32> { v.clone() }
}
''')
    env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
    env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
    command = ["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "batches", "--"]
    for workers in [2, 3]:
        result = subprocess.run(command + ["--threads", str(workers), "--no-bootstrap", "--json"],
                                env=env, capture_output=True, text=True, timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
        assert len(run["cases"]) == 3, run
        for case in run["cases"]:
            whole = "/overridden/" in case["id"]
            assert case["contract"]["threads.batch_policy"] == ("Batches(1)" if whole else "Iterations(2)"), case
            records = [r for r in run["worker_timings"] if r["case"] == case["id"]]
            assert len(records) == workers * (1 if whole else 3), records
            for worker in range(workers):
                assert [r["operations"] for r in records if r["worker"] == worker] == ([5] if whole else [2, 2, 1]), records
    result = subprocess.run(command + ["--dry-run"], env=dict(env, MUST_NOT_RUN="1"), capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    preview = json.JSONDecoder().raw_decode(result.stdout.lstrip())[0]
    assert all("threads.batch_policy" in c["contract"] for c in preview["cases"])
    (root / "src").mkdir()
    (root / "src/lib.rs").write_text('''
#[ab::group]
pub mod shared {
    #[bench]
    fn inherited() -> Vec<u8> { vec![0; 8] }
    #[bench(batch=ab::BatchPolicy::Batches(1.try_into().unwrap()))]
    fn overridden() -> Vec<u8> { vec![0; 8] }
    #[bench(setup=|| std::rc::Rc::new(1))]
    async fn asynchronous(v: &mut std::rc::Rc<i32>) -> std::rc::Rc<i32> { v.clone() }
}
#[ab::group]
pub mod local {
    use std::{rc::Rc, cell::Cell};
    pub struct Output(Rc<Cell<usize>>);
    impl Drop for Output { fn drop(&mut self) { self.0.set(self.0.get()-1); } }
    #[bench(args=[Rc::new(Cell::new(0))], drop_output="outside")]
    fn retained(state: &Rc<Cell<usize>>) -> Output {
        let n=state.get()+1; assert!(n<=2); state.set(n); Output(state.clone())
    }
}
''')
    for group, workers in [("shared", 3), ("local", 1)]:
        (root / "benches/batches.rs").write_text(f'''
#[ab::suite(groups=[thread_batches::{group}], threads={workers},
    batch=ab::BatchPolicy::Iterations(2.try_into().unwrap()), samples=1, iterations=5, warmup_ms=0)]
mod cases {{}}
''')
        result = subprocess.run(command + ["--no-bootstrap", "--json"], env=env, capture_output=True, text=True, timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in result.stdout.splitlines() if line.startswith("BENCH_RESULT=")))
        assert len(run["cases"]) == (3 if group == "shared" else 1), run
        for case in run["cases"]:
            whole = "/overridden/" in case["id"]
            assert case["contract"]["threads.batch_policy"] == ("Batches(1)" if whole else "Iterations(2)"), case
            if group == "shared":
                records = [r for r in run["worker_timings"] if r["case"] == case["id"]]
                assert len(records) == workers * (1 if whole else 3), records
print("cargo bench: lexical/imported batch inheritance, local override, async Rc inputs, non-Send local output lifetimes, runtime worker counts and lazy preview passed")
