// Browser / WASM scenario driver: measures the cases a page exposes and reports them in the
// worker protocol, so the process runner treats a browser like any other program.
//
//   cargo airbug-bench run --protocol --program node -o .airbug-bench/browser \
//     -- bench/scripts/browser-driver.cjs bench/examples/browser/index.html
//
// The page sets `globalThis.airbugBench = { "case id": () => { one operation } }` (or a
// promise of it). Each sample is a batch calibrated to ~--target-ms; the value is the batch
// total in ns and the operation count, exactly what a native worker reports.
//
// Needs Playwright with Chromium (`require('playwright')`). Options: --samples N (default 12),
// --target-ms M (default 50), --warmup-ms W (default 200), --filter SUBSTRING.
const path = require('path');

function options(argv) {
  const out = { samples: 12, targetMs: 50, warmupMs: 200, filter: '', page: null };
  for (let i = 0; i < argv.length; i++) {
    const value = () => {
      const n = Number(argv[++i]);
      if (!Number.isFinite(n) || n <= 0) throw new Error(`${argv[i - 1]} needs a positive number`);
      return n;
    };
    if (argv[i] === '--samples') out.samples = Math.floor(value());
    else if (argv[i] === '--target-ms') out.targetMs = value();
    else if (argv[i] === '--warmup-ms') out.warmupMs = value();
    else if (argv[i] === '--filter') out.filter = argv[++i] || '';
    else if (argv[i] === '--json') continue; // accepted: the native harness flag
    else if (!out.page) out.page = argv[i];
    else throw new Error(`unexpected argument ${argv[i]}`);
  }
  if (!out.page) throw new Error('usage: browser-driver.cjs <page.html | URL> [--samples N] [--target-ms M]');
  return out;
}

// Runs inside the page.
async function measure({ samples, targetMs, warmupMs, filter }) {
  const cases = await globalThis.airbugBench;
  if (!cases || typeof cases !== 'object') throw new Error('the page did not set globalThis.airbugBench');
  // What the clock can resolve: the smallest step performance.now() takes.
  let resolution = Infinity;
  for (let i = 0; i < 50; i++) {
    const a = performance.now();
    let b = a;
    while (b === a) b = performance.now();
    resolution = Math.min(resolution, b - a);
  }
  const batch = (fn, n) => {
    const start = performance.now();
    for (let i = 0; i < n; i++) fn();
    return performance.now() - start;
  };
  const results = [];
  for (const [id, fn] of Object.entries(cases)) {
    if (typeof fn !== 'function' || (filter && !id.includes(filter))) continue;
    for (const end = performance.now() + warmupMs; performance.now() < end;) fn();
    // Grow the batch until it lasts long enough for the clock's resolution not to matter.
    let operations = 1;
    for (;;) {
      const ms = batch(fn, operations);
      if (ms >= targetMs || operations >= 2 ** 30) break;
      operations = Math.max(operations * 2, Math.ceil((operations * targetMs) / Math.max(ms, resolution)));
    }
    const totals = [];
    for (let s = 0; s < samples; s++) totals.push(batch(fn, operations));
    results.push({ id, operations, totals });
  }
  return { results, resolution, agent: navigator.userAgent, isolated: globalThis.crossOriginIsolated === true };
}

(async () => {
  const opts = options(process.argv.slice(2));
  const { chromium } = require('playwright');
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(String(e)));
    const url = /^[a-z]+:\/\//i.test(opts.page) ? opts.page : 'file://' + path.resolve(opts.page);
    await page.goto(url, { waitUntil: 'load' });
    const measured = await page.evaluate(measure, opts);
    if (errors.length) throw new Error('page error: ' + errors[0]);
    if (!measured.results.length) throw new Error('no cases matched');
    const scope = `browser main thread; performance.now() resolution ${measured.resolution.toFixed(3)} ms`
      + `${measured.isolated ? '' : ' (not cross-origin isolated)'}; calls included`;
    const run = {
      schema: 1,
      id: `${Date.now()}-${process.pid}`,
      status: 'complete',
      environment: { runtime: 'browser', agent: measured.agent },
      provenance: { 'driver': 'browser-driver v1', 'driver.page': url },
      cases: measured.results.map(r => ({
        id: r.id,
        contract: {
          'completion': 'synchronous function return',
          'execution.mode': 'benchmark',
          'samples': String(opts.samples),
          'sample_target_ns': String(Math.round(opts.targetMs * 1e6)),
          'sampling.iterations': 'calibrated',
          'timer.clock': 'performance.now',
          'timer.resolution_ns': String(Math.round(measured.resolution * 1e6)),
        },
        metrics: [{ id: 'wall', unit: 'ns', scope, phase: 'measurement', statistic: 'batch_total', direction: 'lower' }],
      })),
      observations: measured.results.flatMap(r => r.totals.map((ms, sequence) => ({
        case: r.id, metric: 'wall', variant: 'candidate', process: 0, pair: null, sequence,
        value: String(Math.round(ms * 1e6)), operations: r.operations, availability: { state: 'available' },
      }))),
      notes: ['Browser batches: a JIT, a garbage collector and a coarsened clock sit between the code and the number. Batch averages are not individual-operation latency.'],
    };
    process.stdout.write('BENCH_RESULT=' + JSON.stringify(run) + '\n');
  } finally {
    await browser.close();
  }
})().catch(err => {
  process.stderr.write(`browser-driver: ${err.message || err}\n`);
  process.exit(1);
});
