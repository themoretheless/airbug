import json,pathlib,re,collections,csv
root=pathlib.Path(__file__).resolve().parents[1]
rows=json.loads((root/'repositories.json').read_text())
patterns={'statistics':r'bootstrap|confidence interval|statistical|variance|outlier','comparison':r'baseline|regression|paired|interleav','lifecycle':r'warm.?up|calibrat|setup|teardown','memory':r'allocat|memory usage|heap|RSS','gpu':r'\bGPU\b|wgpu|Vulkan|Metal','async':r'\basync\b|concurren|latency|throughput','report':r'JSON|CSV|report|dashboard','correctness':r'correctness|validat|checksum|reproducib','isolation':r'affinity|subprocess|isolation|separate process'}
for r in rows:
 text=(root/r['readme']).read_text(errors='replace') if 'readme' in r else ''
 lines=text.splitlines();r['signals']={}
 for k,p in patterns.items():
  hits=[{'line':i+1,'text':l.strip()[:260]} for i,l in enumerate(lines) if re.search(p,l,re.I)]
  if hits:r['signals'][k]=hits[:3]
 r['screening_level']='automated README evidence extraction' if text else 'metadata only'
 r['category']='workload: graphics' if 'rendering language:Rust' in r['queries'] else 'workload: storage' if 'database language:Rust' in r['queries'] else 'profiling' if 'profiling language:Rust' in r['queries'] else 'benchmark/testing candidate'
(root/'screening.json').write_text(json.dumps(rows,indent=2,ensure_ascii=False))
with (root/'REPOSITORIES.md').open('w') as f:
 f.write('# Корпус исследования\n\nАвтоматический скрининг README; сигналы означают упоминание, а не проверенную возможность. Подробные решения по ключевым проектам находятся в FOCUSED.md.\n\n')
 for r in rows:
  f.write(f"## {r['name']}\n\n- Источник: {r['url']}\n- Категория: {r['category']}\n- Описание: {r.get('description')}\n- Уровень: {r['screening_level']}\n")
  if 'readme' in r:
   f.write(f"- Снимок: [{r['readme']}]({r['readme']}); SHA-256: `{r['sha256']}`\n")
   for k,hits in r['signals'].items():
    h=hits[0];f.write(f"- {k}: строка {h['line']}: {h['text']}\n")
  else:f.write('- README не получен; не засчитывается в анализ содержимого.\n')
  f.write('\n')
counts=collections.Counter(k for r in rows for k in r['signals'])
(root/'counts.json').write_text(json.dumps({'unique_repositories':len(rows),'readmes':sum('readme'in r for r in rows),'signals':counts},indent=2))
for start in range(0,len(rows),150):
 with (root/f'review-{start//150}.txt').open('w') as f:
  for r in rows[start:start+150]:
   f.write(r['name']+' | '+str(r.get('description'))[:130]+' | '+','.join(r['signals'])+'\n')
print((root/'counts.json').read_text())
