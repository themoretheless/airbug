#!/usr/bin/env python3
"""Check total measurement targets, imported defaults, and CLI precedence."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="airbug-measurement-target-") as directory:
    root = Path(directory)
    (root / "benches").mkdir()
    (root / "src").mkdir()
    (root / "Cargo.toml").write_text(f'''[package]
name="measurement-target"
version="0.1.0"
edition="2021"
[workspace]
[dependencies]
ab={{package="airbug-bench",path={json.dumps(str(ROOT / 'bench'))}}}
[[bench]]
name="target"
harness=false
''')
    (root / "src/lib.rs").write_text('''
#[ab::group(samples=3, measurement_ms=12, warmup_ms=0)]
pub mod work {
    #[bench(custom=true)]
    fn inherited(n: u64) -> std::time::Duration {
        assert!(std::env::var_os("MUST_NOT_RUN").is_none());
        std::time::Duration::from_millis(n)
    }
    #[bench(custom=true, sample_ms=1)]
    fn child(n: u64) -> std::time::Duration {
        assert!(std::env::var_os("MUST_NOT_RUN").is_none());
        std::time::Duration::from_millis(n)
    }
}
''')
    (root / "benches/target.rs").write_text('#[ab::suite(groups=[measurement_target::work])] mod cases {}')
    def invoke(flags, env=None, success=True, lazy=True):
        environment = {k:v for k,v in os.environ.items() if not k.startswith("AIRBUG_BENCH_")}
        environment.update(AIRBUG_DASHBOARD="0", AIRBUG_BENCH_HISTORY="0", CARGO_TARGET_DIR=str(ROOT / "target/parity-external"))
        environment.update(env or {})
        if lazy:
            environment["MUST_NOT_RUN"]="1"
        output = subprocess.run(["cargo","bench","--offline","--manifest-path",str(root / "Cargo.toml"),"--bench","target","--",*flags],capture_output=True,text=True,env=environment)
        assert (output.returncode==0)==success, output.stdout+output.stderr
        return output.stdout+output.stderr
    def preview(flags=(), env=None):
        output=invoke([*flags,"--dry-run"],env)
        return json.JSONDecoder().raw_decode(output.lstrip())[0]
    original=preview()
    assert original["target_sample_ms"] is None
    by_name={row["id"].split("/")[-1]:row["contract"] for row in original["cases"]}
    assert by_name["inherited"]["sampling.measurement_target_ns"]=="12000000"
    assert by_name["inherited"]["sample_target_ns"]=="4000000"
    assert by_name["child"]["sample_target_ns"]=="1000000"
    assert "sampling.measurement_target_ns" not in by_name["child"]
    for flags,env in [(["--measurement-ms","6"],None),([],{"AIRBUG_BENCH_MEASUREMENT_MS":"6"}),(["--measurement-ms","6"],{"AIRBUG_BENCH_SAMPLE_MS":"invalid"})]:
        for case in preview(flags,env)["cases"]:
            assert case["contract"]["sample_target_ns"]=="2000000"
            assert case["contract"]["sampling.measurement_target_ns"]=="6000000"
    for case in preview(["--sample-ms","2"],{"AIRBUG_BENCH_MEASUREMENT_MS":"invalid"})["cases"]:
        assert case["contract"]["sample_target_ns"]=="2000000"
        assert "sampling.measurement_target_ns" not in case["contract"]
    assert "mutually exclusive" in invoke(["--sample-ms","1","--measurement-ms","6","--dry-run"],success=False)
    assert "must be positive" in invoke(["--measurement-ms","0","--dry-run"],success=False)
    assert "mutually exclusive" in invoke(["--dry-run"],{"AIRBUG_BENCH_SAMPLE_MS":"1","AIRBUG_BENCH_MEASUREMENT_MS":"6"},success=False)
    profiled=preview(["--profile","quick"],{"AIRBUG_BENCH_MEASUREMENT_MS":"invalid"})
    assert all("sampling.measurement_target_ns" not in c["contract"] for c in profiled["cases"])
    fractional = preview(["--measurement-ms", "0.000007", "--warmup-ms", "0.000001", "--min-time-ms", "0.000002", "--max-time-ms", "0.5"])
    assert fractional["warmup_ms"] == "0.000001"
    for case in fractional["cases"]:
        assert case["contract"]["sampling.measurement_target_ns"] == "7"
        assert case["contract"]["sample_target_ns"] == "3"
        assert case["contract"]["warmup_ns"] == "1"
        assert case["contract"]["sampling.min_time_ns"] == "2"
        assert case["contract"]["sampling.max_time_ns"] == "500000"
    for case in preview(["--sample-ms", "0.000001"], {"AIRBUG_BENCH_WARMUP_MS":".125"})["cases"]:
        assert case["contract"]["sample_target_ns"] == "1"
        assert case["contract"]["warmup_ns"] == "125000"
    tiny = preview(["--sample-ms", "0.000001", "--samples", "3"])
    assert tiny["target_sample_ms"] == "0.000001"
    assert tiny["target_measured_ms_per_case"] == "0.000003"
    assert preview(["--profile-time-ms", "0.25"])["profile_time_ms"] == "0.25"
    quick = preview(["--quick", "--quick-min-ms", "0.001", "--quick-max-ms", "0.002"])
    assert quick["adaptive_quick"]["min_time_ms"] == "0.001"
    assert quick["adaptive_quick"]["max_time_ms"] == "0.002"
    for invalid in ["-1", "NaN", "inf", "0.0000001", "18446744073709551616000"]:
        assert "invalid millisecond duration" in invoke(["--measurement-ms", invalid, "--dry-run"], success=False)
    for case in preview(["--measurement-ms", "18446744073709551615999.999999"])["cases"]:
        assert case["contract"]["sampling.measurement_target_ns"] == str((2**64-1)*1_000_000_000+999_999_999)
    maximum=(2**64-1)*1_000_000
    for case in preview(["--measurement-ms",str(2**64-1)])["cases"]:
        assert case["contract"]["sampling.measurement_target_ns"]==str(maximum)
        assert case["contract"]["sample_target_ns"]==str((maximum+2)//3)
    output=invoke(["--measurement-ms","6","--json"],lazy=False)
    run=json.loads(next(line.removeprefix("BENCH_RESULT=") for line in output.splitlines() if line.startswith("BENCH_RESULT=")))
    assert len(run["observations"])==6
    assert all(row["operations"]==2 and row["value"]=="2000000" for row in run["observations"])
    output=invoke(["--measurement-ms","9","--json"],lazy=False)
    run=json.loads(next(line.removeprefix("BENCH_RESULT=") for line in output.splitlines() if line.startswith("BENCH_RESULT=")))
    assert len(run["observations"])==6
    assert all(row["operations"]==3 and row["value"]=="3000000" for row in run["observations"])
    source = (root / "src/lib.rs").read_text()
    fractional_source = "const TARGET: f64 = 0.000007;\n" + source.replace(
        "measurement_ms=12, warmup_ms=0", 'measurement_ms=crate::TARGET, warmup_ms="0.000001", min_time_ms=0.000002, max_time_ms=0.5'
    ).replace("sample_ms=1", "sample_ms=0.125")
    (root / "src/lib.rs").write_text(fractional_source)
    values = {row["id"].split("/")[-1]: row["contract"] for row in preview()["cases"]}
    assert values["inherited"]["sampling.measurement_target_ns"] == "7"
    assert values["inherited"]["sample_target_ns"] == "3"
    assert values["child"]["sample_target_ns"] == "125000"
    for contract in values.values():
        assert contract["warmup_ns"] == "1"
        assert contract["sampling.min_time_ns"] == "2"
        assert contract["sampling.max_time_ns"] == "500000"
    for row in preview(["--warmup-ms", "0.25"])["cases"]:
        assert row["contract"]["warmup_ns"] == "250000"
    for value, expected in [('(1 << 40)', (1 << 40)*1_000_000), ('18446744073709551615', (2**64 - 1)*1_000_000), ('"18446744073709551615999.999999"', (2**64 - 1)*1_000_000_000+999_999_999)]:
        (root / "src/lib.rs").write_text(source.replace("warmup_ms=0", "warmup_ms="+value))
        assert all(row["contract"]["warmup_ns"] == str(expected) for row in preview()["cases"])
    for value in ["-0.5", "f64::NAN", "0.0000001"]:
        (root / "src/lib.rs").write_text(source.replace("warmup_ms=0", "warmup_ms="+value))
        assert "invalid millisecond duration" in invoke(["--dry-run"], success=False)
print("measurement target: imported inheritance, local override, CLI/environment precedence, conflicts, lazy preview and exact collection passed")
