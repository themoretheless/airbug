// Browser acceptance of real HTML reports: loads each page at two window sizes, exercises
// search, filters, details, sorting, in-page navigation and hover, and reports script errors,
// page overflow and marks clipped outside a scroller. Needs Playwright with Chromium.
//   node bench/scripts/visual-acceptance.cjs <dir with reports> <dir for screenshots> <file>...
const { chromium } = require('playwright');
const path = require('path');
const [dir, outDir, ...files] = process.argv.slice(2);
(async () => {
  const b = await chromium.launch(); const rows = [];
  for (const f of files) for (const [tag, w, h] of [['wide', 1280, 900], ['narrow', 390, 844]]) {
    const p = await b.newPage({ viewport: { width: w, height: h } });
    const errs = []; p.on('pageerror', e => errs.push(String(e).slice(0, 120))); p.on('console', m => { if (m.type() === 'error') errs.push(m.text().slice(0, 120)); });
    await p.goto('file://' + path.resolve(dir, f)); await p.waitForTimeout(400);
    const info = await p.evaluate(() => {
      const de = document.documentElement;
      const over = [...document.querySelectorAll('body *')].filter(e => { const r = e.getBoundingClientRect(); return r.width > 0 && r.right > innerWidth + 2; });
      const scrollers = e => { for (let n = e; n && n !== document.body; n = n.parentElement) { const s = getComputedStyle(n); if (/auto|scroll/.test(s.overflowX)) return true; } return false; };
      const bad = over.filter(e => !scrollers(e));
      return { title: document.title, svgs: document.querySelectorAll('svg').length, tables: document.querySelectorAll('table').length,
        pageOverflow: de.scrollWidth - innerWidth, clipped: bad.length, clippedSample: bad.slice(0, 3).map(e => e.tagName + '.' + (e.className.baseVal ?? e.className) + ':' + Math.round(e.getBoundingClientRect().right)),
        titles: document.querySelectorAll('svg title, [title]').length, inputs: [...document.querySelectorAll('input,select,button,summary,a[href^="#"]')].length,
        emptySvgText: [...document.querySelectorAll('svg text')].filter(t => !t.textContent.trim()).length, textLen: document.body.innerText.length };
    });
    // Interaction: search, outcome filter, expand/collapse, details, in-page navigation, hover.
    const acts = [];
    const search = p.locator('#search:visible, input[type=search]:visible').first();
    if (await search.count()) { const before = await p.locator('tbody tr:visible, details:visible, section:visible').count(); await search.fill('zzzz-no-match'); await p.waitForTimeout(150); const none = await p.locator('tbody tr:visible, details:visible, section:visible').count(); await search.fill(''); await p.waitForTimeout(150); const back = await p.locator('tbody tr:visible, details:visible, section:visible').count(); acts.push(`search ${before}->${none}->${back}`); }
    const sel = p.locator('select:visible').first();
    if (await sel.count()) { const n = await sel.locator('option').count(); if (n > 1) { await sel.selectOption({ index: n - 1 }); await p.waitForTimeout(100); await sel.selectOption({ index: 0 }); acts.push(`select ${n}`); } }
    for (const id of ['#expand', '#collapse']) { const el = p.locator(id); if (await el.count() && await el.isVisible()) { await el.click(); await p.waitForTimeout(100); acts.push(id + ' open=' + await p.locator('details[open]').count()); } }
    const det = p.locator('details:visible > summary').first();
    if (await det.count()) { await det.click(); acts.push('details toggled'); }
    const th = p.locator('th[data-sort], th.sortable, th button').first();
    if (await th.count() && await th.isVisible()) { await th.click(); acts.push('sort clicked'); }
    const nav = p.locator('a[href^="#"]:visible').first();
    if (await nav.count()) { const href = await nav.getAttribute('href'); await nav.click().catch(() => {}); await p.waitForTimeout(150); acts.push('nav ' + href + ' y=' + await p.evaluate(() => Math.round(scrollY))); }
    const mark = p.locator('svg circle:visible, svg rect:visible, svg path:visible').first();
    if (await mark.count()) { await mark.hover({ force: true, timeout: 2000 }).catch(() => {}); acts.push('hover ok'); }
    await p.evaluate(() => scrollTo(0, 0));
    await p.screenshot({ path: path.join(outDir, f.replace(/[\/.]/g, '_') + '-' + tag + '.png') });
    await p.screenshot({ path: path.join(outDir, f.replace(/[\/.]/g, '_') + '-' + tag + '-full.png'), fullPage: true });
    rows.push({ file: f, view: tag, ...info, acts: acts.join('; '), errors: errs });
    await p.close();
  }
  console.log(JSON.stringify(rows, null, 1)); await b.close();
})();
