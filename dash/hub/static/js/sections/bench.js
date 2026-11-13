// Runs › Bench: hub-registered sessions, run.json charts, and the "vs previous run" summary
// that Now and the run page reuse.

import { enc, getJson } from "../api.js";
import { $, escapeAttr, escapeHtml } from "../dom.js";
import { fmtNs, fmtPct, median } from "../format.js";
import { targetAttrs, TR } from "../runs.js";

async function renderList() {
  const data = await getJson("bench/runs");
  $("bench-meta").textContent = (data.note || "") + (data.hub_id ? " · hub " + data.hub_id.slice(0, 8) : "");
  const runs = data.runs || [];
  $("bench-list").innerHTML = runs.length
    ? runs.map(r => `<a class="bench-row" href="#/run/${escapeAttr(enc(r.run_id))}">
        <span class="bench-state ${escapeAttr(r.state || "registered")}">${escapeHtml(r.state)}</span>
        <div><strong>${escapeHtml(r.title || r.run_id)}</strong><span>${escapeHtml(r.run_id)}</span></div>
        <span>${escapeHtml(r.updated_at || "")}</span>
      </a>`).join("")
    : `<p class="bench-live">${escapeHtml(data.note || "No runs.")}</p>`;
}

/** Median wall time per case, grouped by case family, from a `run.json`. */
export async function renderCharts(url, el) {
  const run = await getJson(url).catch(() => null);
  if (!run) return;
  const caseById = {};
  for (const c of run.cases || []) {
    if (c && c.id) caseById[c.id] = c;
  }
  const byCase = {};
  for (const o of run.observations || []) {
    if (o.metric !== "wall") continue;
    if (o.availability && o.availability !== "available") continue;
    if (o.value == null || !Number.isFinite(Number(o.value))) continue;
    const c = caseById[o.case];
    const mode = (c && c.measurement_mode) || (c && c.contract && c.contract.measurement_mode);
    const ops = mode === "batch_total" ? (Number(o.operations) || 1) : 1;
    (byCase[o.case] = byCase[o.case] || []).push(Number(o.value) / ops);
  }
  const families = {};
  for (const [caseId, values] of Object.entries(byCase)) {
    const parts = caseId.split("/");
    const engine = parts[parts.length - 1] || caseId;
    const family = parts.slice(0, -1).join("/") || caseId;
    (families[family] = families[family] || []).push({ engine, median: median(values) });
  }
  const cards = Object.keys(families).sort().map(family => {
    const rows = families[family].sort((a, b) => a.median - b.median);
    const max = Math.max(...rows.map(r => r.median), 1);
    const tr = rows.map(r => `<tr><td>${escapeHtml(r.engine)}</td><td>${fmtNs(r.median)}<div class="bench-bar" style="width:${Math.max(2, (r.median / max) * 100)}%"></div></td></tr>`).join("");
    return `<div class="bench-chart-card"><h4>${escapeHtml(family)}</h4>
      <table><thead><tr><th>engine</th><th>median wall</th></tr></thead><tbody>${tr}</tbody></table></div>`;
  }).join("");
  el.innerHTML = cards || `<p class="bench-live">No wall observations in run.json.</p>`;
}

export default {
  cats: ["bench"],
  show: () => renderList().catch(err => { $("bench-meta").textContent = String(err); }),
  tick: n => (n % 4 === 0 ? renderList() : null),
};

// ---- latest bench run vs the previous comparable one (`GET /api/v1/bench/compare`) ----

const DECISION = {
  regression: ["slower", "bad"],
  improvement: ["faster", "good"],
  within_margin: ["same", ""],
  inconclusive: ["unclear", "warn"],
  unavailable: ["n/a", ""],
  neutral: ["info", ""],
};

const shortId = id => String(id).split("/").pop().slice(0, 12);
const link = (x, label) => `<a href="${escapeAttr(x.href)}"${targetAttrs(x.href)}>${escapeHtml(label)}</a>`;

export function benchCompareHtml(cmp) {
  if (!cmp) return "";
  if (!cmp.mode) return cmp.current && cmp.note ? `<p class="now-quiet">Latest run: ${escapeHtml(cmp.note)}.</p>` : "";
  const s = cmp.summary || {};
  const threshold = cmp.threshold != null ? cmp.threshold : 5;
  const head = [];
  if (s.regression) head.push(`<b class="bad">${s.regression}</b> slower`);
  if (s.improvement) head.push(`<b class="good">${s.improvement}</b> faster`);
  if (s.within_margin) head.push(`${s.within_margin} within ±${threshold}%`);
  if (s.inconclusive) head.push(`<span class="warn">${s.inconclusive}</span> unclear`);
  if (!head.length) head.push("nothing to compare");
  const cur = cmp.current;
  const against = cmp.mode === "ab"
    ? `baseline vs candidate in ${link(cur, shortId(cur.id))}`
    : `${link(cur, shortId(cur.id))} vs previous ${link(cmp.previous, shortId(cmp.previous.id))}`;
  const ago = TR && cur.when_ms ? ` · ${escapeHtml(TR.fmtAgo(cur.when_ms))}` : "";
  const rows = (cmp.rows || [])
    .filter(r => r.decision !== "neutral" && r.decision !== "unavailable")
    .slice(0, 5)
    .map(r => {
      const [label, tone] = DECISION[r.decision] || [r.decision, ""];
      const ci = r.interval_percent ? `${fmtPct(r.interval_percent[0])} … ${fmtPct(r.interval_percent[1])}` : "";
      return `<tr class="${escapeAttr(tone)}" title="${escapeAttr((r.note || "") + (ci ? " · interval " + ci : ""))}">
        <td><code>${escapeHtml(r.case)}</code> <small>${escapeHtml(r.metric)}</small></td>
        <td class="num">${escapeHtml(fmtPct(r.change_percent))}</td>
        <td>${escapeHtml(label)}</td></tr>`;
    }).join("");
  const more = (cmp.total || 0) > 5 ? `<p class="now-quiet">${cmp.total} comparisons in total.</p>` : "";
  return `<a class="now-headline" href="${escapeAttr(cur.href)}"${targetAttrs(cur.href)}>
      <span class="now-big">${head.join(" · ")}</span></a>
    <p class="now-quiet">${against}${ago}</p>
    ${rows ? `<table class="now-cmp">${rows}</table>` : ""}${more}`;
}
