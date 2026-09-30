#!/usr/bin/env python3
"""Inventory public declarations from locally extracted, versioned upstream crates.

This is audit input, not a semantic completeness verifier. Download the archives
listed in bench/docs/parity/upstream-surface.json and extract them under --sources.
"""
import pathlib,re,json,hashlib,argparse
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--sources', type=pathlib.Path, required=True)
parser.add_argument('--output', type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[1] / 'docs/parity/upstream-surface.json')
args=parser.parse_args()
root=args.sources
args.output.parent.mkdir(parents=True,exist_ok=True)
versions={'criterion':'0.8.2','divan':'0.1.21'}
manifest={}
for crate,version in versions.items():
 base=root/f'{crate}-{version}'; records=[]
 for p in sorted((base/'src').rglob('*.rs')):
  for i,line in enumerate(p.read_text().splitlines(),1):
   if re.search(r'^\s*pub (?:unsafe |const |async )*(?:fn|trait|enum|struct|type|mod|use)\b',line):
    records.append({'file':str(p.relative_to(base)),'line':i,'declaration':line.strip()})
 archive=root/f'{crate}-{version}.crate'
 manifest[crate]={'version':version,'archive':f'https://static.crates.io/crates/{crate}/{crate}-{version}.crate','sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'surface':records}
args.output.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
