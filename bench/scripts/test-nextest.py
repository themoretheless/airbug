#!/usr/bin/env python3
"""Check libtest listing protocol and optionally the actual nextest runner."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-nextest-fixture-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="nextest-fixture"
version="0.1.0"
edition="2021"
[workspace]
[dev-dependencies]
ab={{package="airbug-bench",path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="cases"
harness=false
''', encoding="utf-8")
    (root / "benches/cases.rs").write_text('''
#[ab::suite(samples=2, iterations=3, warmup_ms=0)]
mod cases {
    fn record(value: &str) {
        use std::io::Write;
        assert!(std::env::var_os("NO_WORK").is_none());
        let mut file = std::fs::OpenOptions::new().append(true).create(true)
            .open(std::env::var("CALL_LOG").unwrap()).unwrap();
        // Format once: parallel nextest processes must not interleave value/newline writes.
        file.write_all(format!("{value}\\n").as_bytes()).unwrap();
    }
    #[bench(args=[1usize,2])]
    fn value(n: usize) { record(&n.to_string()); }
    #[bench(ignore=true)]
    fn ignored() { record("ignored"); }
    #[bench]
    fn fails() { record("fails"); panic!("intentional fixture failure"); }
}
''', encoding="utf-8")
    env = {k:v for k,v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
    env.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"), CALL_LOG=str(root/"calls"))
    def call(command, extra=None, success=True):
        result = subprocess.run(command, cwd=root, env=dict(env, **(extra or {})), capture_output=True, text=True, encoding="utf-8")
        assert (result.returncode==0)==success, result.stdout+result.stderr
        return result
    cargo = ["cargo","test","--offline","--manifest-path",str(root/"Cargo.toml"),"--bench","cases","--"]
    listed = call([*cargo,"--list","--format","terse","--color","always"], {"NO_WORK":"1"})
    assert set(listed.stdout.splitlines()) == {"cases/value/1: benchmark","cases/value/2: benchmark","cases/fails: benchmark"}
    ignored = call([*cargo,"--list","--format","terse","--ignored"], {"NO_WORK":"1"})
    assert ignored.stdout.strip() == "cases/ignored: benchmark"
    call([*cargo,"--format","terse"], {"NO_WORK":"1"}, success=False)
    call([*cargo,"--list","--format","terse","--output-format","tree"], {"NO_WORK":"1"}, success=False)
    call([*cargo,"cases/value/2","--exact","--nocapture"], {"NEXTEST":"1"})
    assert (root/"calls").read_text(encoding="utf-8").splitlines() == ["2"]
    (root/"calls").unlink()
    nextest = os.environ.get("AIRBUG_NEXTEST") or shutil.which("cargo-nextest")
    if nextest:
        base = [nextest,"nextest"]
        options = ["--offline","--manifest-path",str(root/"Cargo.toml"),"--bench","cases"]
        result = call([*base,"list",*options,"--message-format","json"], {"NO_WORK":"1"})
        listing = json.loads(result.stdout)
        tests = {name:case for suite in listing["rust-suites"].values() for name,case in suite["testcases"].items()}
        assert set(tests) == {"cases/value/1","cases/value/2","cases/fails","cases/ignored"}, tests
        assert tests["cases/ignored"]["ignored"]
        # Exercise parallel child processes repeatedly: the call log must preserve
        # exactly one complete record per case, regardless of scheduling.
        for _ in range(8):
            call([*base,"run",*options,"--test-threads","2","-E","test(value)"])
            observed = (root/"calls").read_text(encoding="utf-8").splitlines()
            assert sorted(observed) == ["1","2"], observed
            (root/"calls").unlink()
        call([*base,"run",*options,"--run-ignored","only","-E","test(ignored)"])
        assert (root/"calls").read_text(encoding="utf-8").splitlines() == ["ignored"]
        (root/"calls").unlink()
        result = call([*base,"run",*options,"-E","test(fails)"], success=False)
        assert "intentional fixture failure" in result.stdout+result.stderr
        assert (root/"calls").read_text(encoding="utf-8").splitlines() == ["fails"]
        print("nextest: actual discovery, parameter cases, ignored selection, once-only execution and failure propagation passed")
    else:
        print("nextest: listing/launch protocol passed; real runner unavailable (set AIRBUG_NEXTEST)")
    assert not (root/"target/airbug-bench").exists()
