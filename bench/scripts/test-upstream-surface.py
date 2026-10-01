#!/usr/bin/env python3
"""Verify pinned inventories and ensure failed audits preserve previous output."""
import argparse
import json
import hashlib
import tomllib
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--sources", type=Path, required=True)
args = parser.parse_args()
ROOT = Path(__file__).resolve().parents[2]
auditor = ROOT / "bench/scripts/audit-upstream-surface.py"
with tempfile.TemporaryDirectory(prefix="airbug-audit-") as directory:
    root = Path(directory)
    output = root / "inventory.json"
    subprocess.run([sys.executable, str(auditor), "--sources", str(args.sources), "--output", str(output)], check=True)
    current = json.loads(output.read_text())
    previous = json.loads((ROOT / "bench/docs/parity/upstream-surface.json").read_text())
    assert current == previous, "pinned public inventory changed"
    for crate in ("criterion", "divan"):
        mapping = json.loads((ROOT / f"bench/docs/parity/{crate}-runner.json").read_text())
        rows = [row for row in current[crate]["surface"] if row["file"] == ("src/lib.rs" if crate == "criterion" else "src/divan.rs")]
        if crate == "criterion":
            rows = [row for row in rows if 457 <= row["line"] <= 1243]
        methods = [match.group(1) for row in rows if (match := re.search(r"\bfn\s+(\w+)", row["declaration"]))]
        mapped = [row["upstream"] for row in mapping["methods"]]
        assert len(set(mapped)) == len(mapped)
        assert set(methods) == set(mapped), (crate, set(methods) ^ set(mapped))
        assert mapping["method_count"] == len(methods)
    bencher = json.loads((ROOT / "bench/docs/parity/criterion-bencher.json").read_text())
    declarations = {(row["line"], re.search(r"\bfn\s+(\w+)", row["declaration"]).group(1))
                    for row in current["criterion"]["surface"]
                    if row["file"] == "src/bencher.rs" and re.search(r"\bfn\s+(\w+)", row["declaration"])}
    mapped = [(row["line"], row["upstream"]) for row in bencher["methods"]]
    assert len(mapped) == len(set(mapped))
    assert set(mapped) == declarations, "Criterion Bencher method mapping changed"
    divan_bencher = json.loads((ROOT / "bench/docs/parity/divan-bencher.json").read_text())
    declarations = {(row["line"], re.search(r"\bfn\s+(\w+)", row["declaration"]).group(1))
                    for row in current["divan"]["surface"]
                    if row["file"] == "src/benchmark/mod.rs" and re.search(r"\bfn\s+(\w+)", row["declaration"])}
    public = [(row["line"], row["upstream"]) for row in divan_bencher["methods"]]
    internal = [(row["line"], row["method"]) for row in divan_bencher["excluded_internal_methods"]]
    assert len(public + internal) == len(set(public + internal))
    assert set(public + internal) == declarations, "Divan benchmark method inventory changed"
    assert all(line < 470 for line, _ in public)
    assert all(line >= 470 for line, _ in internal)
    groups = json.loads((ROOT / "bench/docs/parity/criterion-groups.json").read_text())
    declarations = {(row["line"], re.search(r"\bfn\s+(\w+)", row["declaration"]).group(1))
                    for row in current["criterion"]["surface"]
                    if row["file"] == "src/benchmark_group.rs" and re.search(r"\bfn\s+(\w+)", row["declaration"])}
    mapped = [(row["line"], row["upstream"]) for row in groups["methods"]]
    assert len(mapped) == len(set(mapped))
    assert set(mapped) == declarations, "Criterion group/identity method inventory changed"
    extensions = json.loads((ROOT / "bench/docs/parity/criterion-extensions.json").read_text())
    for trait in extensions["traits"]:
        source = (args.sources / "criterion-0.8.2" / trait["file"]).read_text().splitlines()
        assert source[trait["start"] - 1] == f'pub trait {trait["trait"]} {{'
        assert source[trait["end"] - 1] == "}"
        actual = []
        for line in range(trait["start"], trait["end"] + 1):
            if match := re.match(r"\s*(fn|type)\s+(\w+)", source[line - 1]):
                actual.append((line, match[1], match[2]))
        mapped = [(row["line"], row["kind"], row["upstream"]) for row in trait["members"]]
        assert actual == mapped, f'{trait["trait"]} member inventory changed'
    features = json.loads((ROOT / "bench/docs/parity/features.json").read_text())
    for crate, version, source_file, pattern in [
        ("criterion", "0.8.2", "src/lib.rs", r'Arg::new\("([^"]+)"\)'),
        ("divan", "0.1.21", "src/cli.rs", r'(?:Arg::new|option|flag|ignored_flag)\("([^"]+)"\)'),
    ]:
        source_root = args.sources / f"{crate}-{version}"
        source_bytes = (source_root / source_file).read_bytes()
        cli = json.loads((ROOT / f"bench/docs/parity/{crate}-cli.json").read_text())
        actual = re.findall(pattern, source_bytes.decode())
        mapped = [row["upstream"] for row in cli["arguments"]]
        assert len(actual) == len(set(actual)) == cli["argument_count"]
        assert len(mapped) == len(set(mapped)) and set(actual) == set(mapped)
        assert hashlib.sha256(source_bytes).hexdigest() == cli["source_sha256"]
        manifest = (source_root / "Cargo.toml").read_bytes()
        feature_map = features["crates"][crate]
        assert hashlib.sha256(manifest).hexdigest() == feature_map["manifest_sha256"]
        actual_features = tomllib.loads(manifest.decode())["features"]
        mapped_features = {name: row["enables"] for name, row in feature_map["features"].items()}
        assert actual_features == mapped_features, f"{crate} feature dependencies changed"
    counters = json.loads((ROOT / "bench/docs/parity/divan-counters.json").read_text())
    source = (args.sources / "divan-0.1.21/src/counter/mod.rs").read_text()
    actual = [(line, match[1]) for line, text in enumerate(source.splitlines(), 1)
              if (match := re.search(r"pub (?:const )?fn (\w+)", text))]
    assert actual == [(row["line"], row["upstream"]) for row in counters["methods"]]
    generated = re.findall(r"^\s*type_bytes!\((\w+)\);", source, re.M)
    assert generated == list(counters["macro_generated_helpers"])
    allocation = json.loads((ROOT / "bench/docs/parity/allocations.json").read_text())["public_api_audit"]
    source = (args.sources / "divan-0.1.21/src/alloc.rs").read_text()
    actual = [(line, match[1]) for line, text in enumerate(source.splitlines(), 1)
              if (match := re.search(r"pub (?:const )?fn (\w+)", text))]
    mapped = [(row["line"], row["method"]) for row in allocation["constructors"] + allocation["internal_methods"]]
    assert actual == mapped, "Divan allocation declarations changed"
    for name in ["ThreadAllocInfo", "AllocTally", "AllocOpMap"]:
        assert f"pub(crate) struct {name}" in source
    assert "pub(crate) enum AllocOp" in source
    assert "pub(crate) static IGNORE_ALLOC" in source
    mutations = ["missing_directory", "missing_file", "extra_file", "changed_file", "changed_manifest", "bad_archive", "missing_archive"]
    for mutation in mutations:
        sources = root / mutation
        shutil.copytree(args.sources, sources)
        crate = sources / "divan-0.1.21"
        if mutation == "missing_directory":
            shutil.rmtree(crate / "src")
        elif mutation == "missing_file":
            (crate / "src/lib.rs").unlink()
        elif mutation == "extra_file":
            (crate / "src/extra.rs").write_text("pub fn fabricated() {}\n")
        elif mutation == "changed_file":
            with (crate / "src/lib.rs").open("a") as file:
                file.write("\n// altered source\n")
        elif mutation == "changed_manifest":
            with (crate / "Cargo.toml").open("a") as file:
                file.write("\n# altered manifest\n")
        elif mutation == "bad_archive":
            with (sources / "divan-0.1.21.crate").open("ab") as file:
                file.write(b"altered archive")
        else:
            (sources / "divan-0.1.21.crate").unlink()
        output.write_bytes(b"keep prior audit\n")
        result = subprocess.run([sys.executable, str(auditor), "--sources", str(sources), "--output", str(output)], capture_output=True, text=True)
        assert result.returncode != 0, mutation
        assert output.read_bytes() == b"keep prior audit\n", mutation
        shutil.rmtree(sources)
print("Pinned inventories match: 253 Criterion + 244 Divan declarations; 24/23 manager methods and 14 Criterion and 10 Divan Bencher methods plus 15 group/identity methods and four extension traits mapped; 57 CLI arguments and 17 Cargo features match; counter helpers including macro declarations match; seven invalid inputs rejected without replacing output")
