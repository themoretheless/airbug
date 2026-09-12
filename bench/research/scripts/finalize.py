import pathlib,json,hashlib,collections,re,datetime
root=pathlib.Path(__file__).resolve().parents[1];rows=json.loads((root/'screening.json').read_text())
counts=collections.Counter(x['disposition'] for x in rows)
errors=[]
for x in rows:
 if 'readme' in x:
  p=root/x['readme']
  if not p.exists() or hashlib.sha256(p.read_bytes()).hexdigest()!=x['sha256']:errors.append('README hash mismatch: '+x['name'])
  lines=p.read_text(errors='replace').splitlines()
  for hits in x['signals'].values():
   for h in hits:
    if lines[h['line']-1].strip()[:260]!=h['text']:errors.append('Evidence mismatch: '+x['name'])
assert counts['retained']>=503,counts
assert len({x['name'] for x in rows})==len(rows)
# Add final disposition to generated catalog, idempotently.
p=root/'REPOSITORIES.md';s=p.read_text()
for x in rows:
 key='## '+x['name']+'\n\n'
 marker=key+'- Итог: '+x['disposition']+' — '+x['reason']+'\n'
 s=re.sub(re.escape(key)+r'(?:- Итог: [^\n]*\n)*', lambda _: marker, s)
p.write_text(s)
# Validate authored local Markdown links (exclude copied upstream docs).
authored=[root.parent/'README.md',root/'REPORT.md',root/'FOCUSED.md',*list((root.parent/'docs').glob('*.md'))]
for p in authored:
 for link in re.findall(r'\]\(([^)]+)\)',p.read_text()):
  if '://' in link or link.startswith('#'):continue
  link=re.sub(r':\d+$','',link.split('#')[0]);target=(p.parent/link).resolve()
  if target.name=='audit.json':continue
  if not target.exists():errors.append('Missing link '+str(p.relative_to(root.parent))+': '+link)
source_projects=['criterion-rs/criterion.rs','nvzqz/divan','dotnet/BenchmarkDotNet','bazhenov/tango','imazen/zenbench','gungraun/gungraun','sharkdp/hyperfine']
audit={'checked_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'records':len(rows),'readmes':sum('readme'in x for x in rows),'dispositions':dict(counts),'retained_distinct_readme_hashes':len({x['sha256'] for x in rows if x['disposition']=='retained'}),'focused_engine_source_review':source_projects,'forma_commit':'e2ef5f7792af363fe6e625febd44ced0c874cb9f','errors':errors,'checks':['README SHA-256','evidence line correspondence','unique repository names','minimum retained count','authored local links'],'not_tested':['upstream benchmarks','new Rust library','Forma GPU/window runtime','statistical engine implementation']}
(root/'audit.json').write_text(json.dumps(audit,indent=2,ensure_ascii=False));print(json.dumps(audit,indent=2,ensure_ascii=False));assert not errors
