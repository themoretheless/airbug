import pathlib,json,re,collections
r=pathlib.Path(__file__).resolve().parents[1];rows=json.loads((r/'screening.json').read_text())
manual_review=json.loads((r/'manual-exclusions.json').read_text())
exclude_words=r'LLM|coding agent|AI agent|language model|reinforcement|robot learning|robot manipulation|semantic segmentation|object detection|neural radiance|NeRF|metagenom|taxonomic|GitHub profile|profile manager|profiles|profile switch|profile generations|lens profile|color profile|password|dotfiles|aliases|portfolio|language standard|security guide|security benchmark|security standards|geofencing|skill.*framework|gpt|imitation learning|visual question|agricultur|speech to text|speech-to-text'
manual=set('us/crw reyamira/models abundant-ai/swe-marathon raphaelmansuy/edgeparse goobolabs/somali-language-standard clouedoc/hzfind ultralytics/template-rust ruvnet/rupixel openai/evals mikeroyal/Open-Source-Security-Guide amazon-archives/aws-security-benchmark android-bench/android-bench NVIDIA/SkillEvaluator MILVLG/openvqa cavalab/srbench numbbo/coco neurosim/MLP_NeuroSim_V3.0 openml/automlbenchmark codefuse-ai/OpAgent edison7009/EchoBird serokell/deploy-rs westpoint-io/lazyrsync supermemoryai/smfs ZakisM/bl3_save_edit TheYkk/git-switcher siketyan/ghr hw0lff/shikane jzbor/nix-sweep aralroca/aralroca MarlinDiary/worklouder-input-cli Th0rgal/shard kguardian-dev/kguardian naftulikay/aws-env gleich/profile_stack roniel-rhack/envi inceptyon-labs/TARS vobst/btf2json loocor/mcpmate ImShyMike/hackatime-heatmap ElNiak/cupp-rs Blazity/next-enterprise marmelab/awesome-rest albinotonnina/albinotonnina.com avoidwork/filesize.js d3ward/toolz dmaicher/doctrine-test-bundle DoneDeal0/superdiff janreges/siteone-crawler dwyl/book AirportR/fulltclash swc-project/jest themidnightgospel/Imposter quii/mockingjay-server jacksonh/manos xullexer/PYDNS-Scanner techinz/browsers-benchmark ZoranPandovski/awesome-testing-tools zszszszsz/.config linhay/harmony-next.skills bartoszlenar/Validot chrisneagu/FTC-Skystone-Dark-Angels-Romania-2020 testinggospels/camouflage faisalkindi/DLSS5oneclick nikomatsakis/skill-tree m13253/FaithType rcedgar/muscle MCG-NKU/SalBenchmark mattiaspaul/deedsBCV'.split())
# Explicit exclusions and transparent conservative relevance rule, not a quality score.
for x in rows:
 desc=x.get('description') or ''
 excluded=x['name'] in manual or bool(re.search(exclude_words,desc,re.I))
 if x['name'] in ['huggingface/inference-benchmarker','vllm-project/vllm-bench','alibaba/MNN','microsoft/DiskANN']:excluded=False
 if x['name']=='bheisler/criterion.rs':x['disposition']='alias';x['reason']='Historical Criterion location; canonical criterion-rs/criterion.rs counted separately.'
 elif 'readme' not in x:x['disposition']='unavailable';x['reason']='No README body; excluded from content-review count.'
 elif x['name'] in manual_review:x['disposition']='excluded';x['reason']=manual_review[x['name']]
 elif excluded:x['disposition']='excluded';x['reason']='Search false positive or domain outside Rust performance harness requirements.'
 else:
  x['disposition']='retained';x['reason']='Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.'
(r/'screening.json').write_text(json.dumps(rows,indent=2,ensure_ascii=False))
counts=collections.Counter(x['disposition'] for x in rows)
(r/'curation.json').write_text(json.dumps(dict(counts),indent=2));print(counts)
with (r/'DECISIONS.tsv').open('w') as f:
 f.write('repository\tdisposition\tcategory\treason\n')
 for x in rows:f.write('\t'.join([x['name'],x['disposition'],x['category'],x['reason']])+'\n')
