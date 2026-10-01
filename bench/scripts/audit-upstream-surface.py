#!/usr/bin/env python3
"""Inventory declarations from authenticated, extracted upstream source archives.

This textual inventory is audit input, not a semantic completeness verifier.
It does not resolve reexports, cfg reachability or macro-generated APIs.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import tarfile
import tempfile
import os

PINNED = {
    "criterion": ("0.8.2", "950046b2aa2492f9a536f5f4f9a3de7b9e2476e575e05bd6c333371add4d98f3"),
    "divan": ("0.1.21", "a405457ec78b8fe08b0e32b4a3570ab5dff6dd16eb9e76a5ee0a9d9cbd898933"),
}


def inventory(root):
    manifest = {}
    for crate, (version, expected_hash) in PINNED.items():
        prefix = f"{crate}-{version}"
        base = root / prefix
        archive = root / f"{prefix}.crate"
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        if digest != expected_hash:
            raise ValueError(f"{archive}: archive checksum differs from pinned release")
        with tarfile.open(archive, "r:gz") as tar:
            expected = {}
            for member in tar.getmembers():
                name = member.name.removeprefix(prefix + "/")
                if name == member.name:
                    continue
                if name == "Cargo.toml" or (name.startswith("src/") and name.endswith(".rs")):
                    if not member.isfile() or name in expected:
                        raise ValueError(f"{archive}: duplicate/nonregular source {name}")
                    expected[name] = tar.extractfile(member).read()
        sources = {str(path.relative_to(base)) for path in (base / "src").rglob("*.rs")}
        archived_sources = set(expected) - {"Cargo.toml"}
        if not archived_sources or sources != archived_sources:
            raise ValueError(f"{base}: source set differs from archive; missing={sorted(archived_sources - sources)}, extra={sorted(sources - archived_sources)}")
        for name, content in expected.items():
            if (base / name).read_bytes() != content:
                raise ValueError(f"{base / name}: contents differ from pinned archive")
        records = []
        for name in sorted(sources):
            for line, text in enumerate(expected[name].decode("utf-8").splitlines(), 1):
                if re.search(r"^\s*pub (?:unsafe |const |async )*(?:fn|trait|enum|struct|type|mod|use)\b", text):
                    records.append({"file": name, "line": line, "declaration": text.strip()})
        if not records:
            raise ValueError(f"{base}: no public declarations found")
        manifest[crate] = {"version": version, "archive": f"https://static.crates.io/crates/{crate}/{prefix}.crate", "sha256": digest, "surface": records}
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sources", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "docs/parity/upstream-surface.json")
    args = parser.parse_args()
    try:
        result = inventory(args.sources)
    except (OSError, ValueError, tarfile.TarError) as failure:
        parser.error(str(failure))
    # Publish only after validating both complete inputs. A failed audit must
    # never replace the last known inventory with partial or empty evidence.
    args.output.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=args.output.parent, delete=False) as file:
            temporary = Path(file.name)
            file.write(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
        os.replace(temporary, args.output)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
