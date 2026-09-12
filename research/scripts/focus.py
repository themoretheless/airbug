import urllib.request,pathlib,concurrent.futures,tarfile,io,json,hashlib
root=pathlib.Path(__file__).resolve().parents[1]/'focused';root.mkdir(exist_ok=True)
names=['criterion-rs/criterion.rs','nvzqz/divan','gungraun/gungraun','dotnet/BenchmarkDotNet','bazhenov/tango','imazen/zenbench','envidera/zench','google/benchmark','sharkdp/hyperfine','bencherdev/bencher','bheisler/iai','rust-lang/measureme','HdrHistogram/HdrHistogram_rust','tokio-rs/tracing','wolfpld/tracy','asv-runner/asv_runner']
def run(name):
 try:
  req=urllib.request.Request('https://codeload.github.com/'+name+'/tar.gz/HEAD',headers={'User-Agent':'rbench-research'})
  data=urllib.request.urlopen(req,timeout=90).read();d=root/name.replace('/','__');d.mkdir(exist_ok=True);n=0
  with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as t:
   for m in t:
    parts=pathlib.PurePosixPath(m.name).parts[1:]
    if not m.isfile() or not parts or '..' in parts or m.size>500000:continue
    if pathlib.Path(m.name).suffix not in ['.rs','.cs','.md','.toml','.py','.cc','.h','.yml','.json']:continue
    p=d.joinpath(*parts);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read());n+=1
  return {'repo':name,'url':'https://codeload.github.com/'+name+'/tar.gz/HEAD','archive_sha256':hashlib.sha256(data).hexdigest(),'files':n}
 except Exception as e:return {'repo':name,'error':str(e)}
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
 results=list(pool.map(run,names))
(root/'manifest.json').write_text(json.dumps(results,indent=2));print(json.dumps(results,indent=2))
