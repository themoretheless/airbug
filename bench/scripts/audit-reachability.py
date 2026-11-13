#!/usr/bin/env python3
"""Check the textual upstream inventory against what is really reachable.

The inventory (bench/docs/parity/upstream-surface.json) lists `pub` declarations by reading
source text, so it cannot tell a public item nobody can name from one re-exported under another
path. rustdoc knows: its JSON output is walked from the crate root through modules, re-exports,
enum variants, and the methods of every reachable type and trait. Every reachable function,
method, type, trait and macro defined in the crate must sit on a line the inventory lists.

    cd <extracted criterion-0.8.2> && RUSTC_BOOTSTRAP=1 cargo rustdoc --all-features -- \
        -Z unstable-options --output-format json          # same for divan-0.1.21
    python3 bench/scripts/audit-reachability.py --criterion target/doc/criterion.json \
        --divan target/doc/divan.json [--output bench/docs/parity/reachability.json]
"""
import argparse, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
KINDS = {"function", "struct", "enum", "trait", "macro", "union", "type_alias", "proc_macro"}

# Reachable items the textual scan cannot see as `pub` rows, and where each is accounted for.
MACROS = {
    "criterion_group": "native registration: Suite builder or #[airbug_bench::bench] (criterion-groups.json::registration_semantic_audit)",
    "criterion_main": "native entry point: Suite::main() / main_registered (criterion-runner.json)",
}

def reachable(doc):
    index, seen, found, role = doc["index"], set(), {}, {}
    def visit(item_id, via, context="item"):
        key = str(item_id)
        if key in seen or key not in index:
            return
        seen.add(key)
        item = index[key]
        if item.get("crate_id", 0) != 0:
            return
        inner = item["inner"]
        kind = next(iter(inner))
        body = inner[kind]
        name = item.get("name") or ""
        path = f"{via}::{name}" if name else via
        if kind in KINDS and item.get("span"):
            found[key] = (kind, path, item["span"]["filename"], item["span"]["begin"][0])
            role[key] = context
        if kind == "module":
            for child in body["items"]:
                visit(child, path)
        elif kind == "use":
            if body.get("id") is not None:
                visit(body["id"], via)
        elif kind in ("struct", "enum", "union"):
            for impl in body.get("impls", []):
                visit(impl, path)
        elif kind == "trait":
            for child in body["items"]:
                visit(child, path, "trait_method")
        elif kind == "impl":
            # Blanket and auto-trait impls carry no source of their own.
            if not body.get("is_synthetic") and body.get("blanket_impl") is None:
                for child in body["items"]:
                    visit(child, via, "trait_impl" if body.get("trait") else "item")
    root = index[str(doc["root"])]
    visit(doc["root"], "")
    return found, role

parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
parser.add_argument("--criterion", type=Path, required=True)
parser.add_argument("--divan", type=Path, required=True)
parser.add_argument("--output", type=Path)
args = parser.parse_args()
inventory = json.loads((ROOT / "bench/docs/parity/upstream-surface.json").read_text())
report, failed = {}, False
for crate, path in (("criterion", args.criterion), ("divan", args.divan)):
    doc = json.loads(path.read_text())
    assert doc["crate_version"] == inventory[crate]["version"], (crate, doc["crate_version"])
    found, role = reachable(doc)
    listed = {(row["file"], row["line"]) for row in inventory[crate]["surface"]}
    hit = {(f, line) for _, _, f, line in found.values()}
    # Attributes and doc comments sit above a declaration; rustdoc points at the first of them.
    def covered(file, line):
        return any((file, line + k) in listed for k in range(0, 40)) or (file, line) in listed
    explained = {"trait_methods_of_listed_traits": 0, "methods_of_trait_impls": 0, "macros": {}}
    missing = set()
    for key, (kind, name, f, line) in found.items():
        if covered(f, line):
            continue
        short = name.rsplit("::", 1)[-1]
        if role[key] == "trait_impl":
            # `impl Measurement for WallTime`: the trait's own method is the public contract.
            explained["methods_of_trait_impls"] += 1
        elif role[key] == "trait_method":
            # No `pub` keyword on trait methods; the trait row above them is listed.
            explained["trait_methods_of_listed_traits"] += 1
        elif kind == "macro" and short in MACROS:
            explained["macros"][short] = MACROS[short]
        else:
            missing.add((kind, name, f, line))
    missing = sorted(missing)
    # Rows nobody outside the crate can name: `pub` inside private modules.
    unreachable = sorted((f, line) for f, line in listed
                         if not any(f == hf and 0 <= line - hl < 40 for hf, hl in hit))
    report[crate] = {
        "version": doc["crate_version"],
        "reachable_items": len(found),
        "inventory_rows": len(listed),
        "reachable_outside_the_textual_inventory": explained,
        "reachable_and_unaccounted": [dict(kind=k, path=n, file=f, line=l) for k, n, f, l in missing],
        "inventory_rows_not_reachable": len(unreachable),
    }
    print(f"{crate} {doc['crate_version']}: {len(found)} reachable items, {len(listed)} inventory rows, "
          f"{len(missing)} reachable and unaccounted, {len(unreachable)} inventory rows not reachable (internal `pub`)")
    failed |= bool(missing)
if args.output:
    args.output.write_text(json.dumps(report, indent=2) + "\n")
for crate in report:
    for row in report[crate]["reachable_and_unaccounted"][:40]:
        print("  UNLISTED", crate, row)
sys.exit(1 if failed else 0)
