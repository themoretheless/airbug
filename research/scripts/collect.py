import urllib.request,urllib.parse,json,pathlib,concurrent.futures,time,hashlib
ROOT=pathlib.Path(__file__).resolve().parents[1]; (ROOT/'sources').mkdir(exist_ok=True)
def get(url):
 req=urllib.request.Request(url,headers={'User-Agent':'rbench-research','Accept':'application/vnd.github+json'})
 with urllib.request.urlopen(req,timeout=25) as r:return r.read()
queries=['benchmark language:Rust','benchmark framework','microbenchmark','profiling language:Rust','performance testing','rendering language:Rust','database language:Rust','benchmark language:C++']
repos={}
for i,q in enumerate(queries):
 p=ROOT/f'search-{i}.json'
 try:
  if not p.exists():p.write_bytes(get('https://api.github.com/search/repositories?'+urllib.parse.urlencode({'q':q,'per_page':100,'sort':'stars'})))
  data=json.loads(p.read_text())
  for x in data.get('items',[]):
   if x['fork']:continue
   if x['full_name'] not in repos:repos[x['full_name']]={'name':x['full_name'],'branch':x['default_branch'],'description':x['description'],'stars':x['stargazers_count'],'url':x['html_url'],'queries':[]}
   repos[x['full_name']]['queries'].append(q)
  print(q,len(repos),flush=True)
 except Exception as e: print(q,str(e),flush=True)
 time.sleep(7)
for name in ['criterion-rs/criterion.rs','nvzqz/divan','gungraun/gungraun','dotnet/BenchmarkDotNet','bazhenov/tango','imazen/zenbench','envidera/zench','google/benchmark','sharkdp/hyperfine','bencherdev/bencher','bheisler/iai','rust-lang/measureme','tokio-rs/loom','HdrHistogram/HdrHistogram_rust']:
 repos.setdefault(name,{'name':name,'branch':'HEAD','url':'https://github.com/'+name,'queries':['curated'],'description':None,'stars':None})
def read(r):
 folder=ROOT/'sources'/r['name'].replace('/','__');folder.mkdir(exist_ok=True)
 for filename in ['README.md','README.rst','README','readme.md']:
  url=f"https://raw.githubusercontent.com/{r['name']}/{r['branch']}/{filename}"
  try:
   path=folder/filename
   if not path.exists():path.write_bytes(get(url))
   data=path.read_bytes();r.update(readme=str(path.relative_to(ROOT)),readme_url=url,sha256=hashlib.sha256(data).hexdigest(),bytes=len(data));return r
  except Exception:pass
 r['error']='README unavailable';return r
with concurrent.futures.ThreadPoolExecutor(max_workers=12) as pool:
 results=[]
 for r in pool.map(read,repos.values()):
  results.append(r)
  if len(results)%50==0: print('read',len(results),flush=True)
(ROOT/'repositories.json').write_text(json.dumps(results,indent=2,ensure_ascii=False))
print('DONE',len(results),'read',sum('readme' in r for r in results),flush=True)
