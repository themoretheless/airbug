#!/usr/bin/env python3
"""Exercise allocated worker setup through real Cargo and imported groups."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-allocated-threads-") as directory:
    root = Path(directory)
    (root / "src").mkdir()
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="allocated-threads"
version="0.1.0"
edition="2021"
[workspace]
[dependencies]
ab={{package="airbug-bench",path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="cases"
harness=false
''')
    (root / "src/lib.rs").write_text('''
#[global_allocator]
static ALLOCATOR: ab::alloc::TrackingAllocator<std::alloc::System> = ab::alloc::TrackingAllocator::new(std::alloc::System);
#[ab::group(allocator=&crate::ALLOCATOR, samples=1, iterations=2, warmup_ms=0)]
pub mod work {
    fn input() -> std::rc::Rc<Vec<u8>> {
        assert!(std::env::var_os("MUST_NOT_RUN").is_none());
        std::rc::Rc::new(vec![7;128])
    }
    #[bench(setup=input, setup_thread="worker", input_bytes=|i: &std::rc::Rc<Vec<u8>>| i.len() as u64)]
    fn sync(input: &mut std::rc::Rc<Vec<u8>>) -> Vec<u8> { vec![input[0];16] }
    #[bench(setup=input)]
    async fn asynchronous(input: std::rc::Rc<Vec<u8>>) -> Vec<u8> {
        std::future::ready(()).await;
        vec![input[0];16]
    }
}
''')
    (root / "benches/cases.rs").write_text('#[ab::suite(groups=[allocated_threads::work], threads=[1,2])] mod cases {}')
    def invoke(flags, extra=None, lazy=False):
        env = {k: v for k, v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
        env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
        env.update(extra or {})
        if lazy:
            env["MUST_NOT_RUN"] = "1"
        result = subprocess.run(["cargo", "bench", "--offline", "--manifest-path", str(root / "Cargo.toml"), "--bench", "cases", "--", *flags], env=env, capture_output=True, text=True)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout
    for flags, env, expected in [([], {}, {1,2}), (["--threads", "3,1"], {"AIRBUG_BENCH_THREADS":"invalid"}, {1,3}), ([], {"AIRBUG_BENCH_THREADS":"2"}, {2})]:
        preview = json.JSONDecoder().raw_decode(invoke([*flags,"--dry-run"], env, lazy=True).lstrip())[0]
        assert len(preview["cases"]) == len(expected)*2
        assert {int(c["contract"]["threads"]) for c in preview["cases"]} == expected
        output = invoke([*flags,"--json"], env)
        run = json.loads(next(line.removeprefix("BENCH_RESULT=") for line in output.splitlines() if line.startswith("BENCH_RESULT=")))
        for case in run["cases"]:
            workers = int(case["contract"]["threads"])
            rows = {r["metric"]: r for r in run["observations"] if r["case"] == case["id"]}
            assert rows["wall"]["operations"] == workers*2
            is_async = "/asynchronous/" in case["id"]
            assert int(rows["alloc.count"]["value"]) == workers*2*(2 if is_async else 1)
            if not is_async:
                assert int(rows["alloc.bytes"]["value"]) == workers*2*16
                assert int(rows["wall"]["work_totals"]["bytes"]) == workers*2*128
    tree = invoke(["--threads","1","--output-format","tree","--bytes-format","binary"])
    assert "throughput bytes" in tree and "paired with time:" in tree
    assert "alloc.count" in tree and "count/op" in tree
    assert "(sample peak)" in tree and "(peak per operation)" in tree
    assert "worker-slot wall" in tree and "all waves combined" in tree
    assert "worker-slot throughput bytes" in tree
    assert "worker-wave alloc.count" in tree and "worker-wave records" in tree
    assert "not independent process repetitions" in tree
    assert "(wave peak)" in tree
print("allocated thread inheritance: imported defaults, worker-local Rc setup, async setup, CLI/environment precedence, lazy preview and exact counts passed")
