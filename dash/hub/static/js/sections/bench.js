// Runs › Bench: run list, one run's live detail, and the "vs previous run" summary Now reuses.

import { enc, getJson, tryJson } from "../api.js";
import { $, escapeAttr, escapeHtml } from "../dom.js";
import { fmtNs, fmtPct, median } from "../format.js";
import { currentRoute } from "../shell.js";
import { targetAttrs, TR } from "../runs.js";

let pollTimer = null;

function stopPolling() {
  clearInterval(pollTimer);
  pollTimer = null;
}

async function renderList() {
  stopPolling();
  $("bench-list").hidden = false;
  $("bench-detail").hidden = true;
  $("bench-title").textContent = "Bench runs";
  const data = await getJson("bench/runs");
  $("bench-meta").textContent = (data.note || "") + (data.hub_id ? " · hub " + data.hub_id.slice(0, 8) : "");
  const runs = data.runs || [];
  $("bench-list").innerHTML = runs.length
    ? runs.map(r => `<a class="bench-row" href="#/bench/${escapeAttr(r.run_id)}">
        <span class="bench-state ${escapeAttr(r.state || "registered")}">${escapeHtml(r.state)}</span>
        <div><strong>${escapeHtml(r.title || r.run_id)}</strong><span>${escapeHtml(r.run_id)}</span></div>
        <span>${escapeHtml(r.updated_at || "")}</span>
      </a>`).join("")
    : `<p class="bench-live">${escapeHtml(data.note || "No runs.")}</p>`;
}

function renderProgress(d) {
  const live = d.live || {};
  const completed = Number(live.completed || 0);
  const total = Number(live.total || 0);
  const prog = $("bench-progress");
  prog.max = total > 0 ? total : 100;
  prog.value = total > 0 ? completed : (d.state === "complete" ? 100 : d.state === "running" ? 50 : 0);
  $("bench-live").textContent = [live.state || d.state, total ? `${completed}/${total}` : "", live.variant || ""]
    .filter(Boolean).join(" · ");
}

function renderLinks(d) {
  const links = [[d.report_url, "report.html"], [d.run_json_url, "run.json"], [d.jaeger_url, "Jaeger traces"]]
    .filter(([href]) => href)
    .map(([href, label]) => `<a href="${escapeAttr(href)}" target="_blank" rel="noopener">${label}</a>`);
  $("bench-links").innerHTML = links.join(" · ");
  const frame = $("bench-report-frame");
  frame.hidden = !(d.has_report_html && d.report_url);
  if (!frame.hidden && frame.getAttribute("data-src") !== d.report_url) {
    frame.src = d.report_url;
    frame.setAttribute("data-src", d.report_url);
  }
}

async function renderCharts(url) {
  const run = await getJson(url).catch(() => null);
  if (!run) return;
  const byCase = {};
  for (const o of run.observations || []) {
    if (o.metric !== "wall") continue;
    (byCase[o.case] = byCase[o.case] || []).push(Number(o.value));
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
  $("bench-charts").innerHTML = cards || `<p class="bench-live">No wall observations in run.json.</p>`;
}

/** Run-scoped signal feeds, one line per item. */
const FEEDS = [
  { el: "bench-logs", path: "logs", params: { limit: 80 }, items: d => d.entries, line: e => `[${e.severity}] ${e.service} ${e.body}`, empty: "No logs." },
  { el: "bench-metrics", path: "metrics", params: { limit: 80 }, items: d => (d.points || []).slice(-40), line: p => `${p.name}=${p.value} ${p.unit || ""}`.trim(), empty: "No metrics." },
  { el: "bench-issues", path: "issues", params: {}, items: d => d.issues, line: i => `${i.id} · ${i.title} · ${i.status}`, empty: "No issues." },
];

async function renderFeed(feed, runId) {
  const data = await tryJson(feed.path, { ...feed.params, run_id: runId }, {});
  const items = feed.items(data) || [];
  $(feed.el).textContent = items.length ? items.map(feed.line).join("\n") : (data.note || feed.empty);
}

async function renderDetail(runId) {
  const d = await getJson("bench/runs/" + enc(runId)).catch(() => null);
  if (!d) {
    $("bench-live").textContent = "run not found";
    return;
  }
  $("bench-meta").textContent = `${d.state} · hub ${String(d.hub_id || "").slice(0, 8)}`;
  renderProgress(d);
  renderLinks(d);
  if (d.has_run_json && d.run_json_url) await renderCharts(d.run_json_url);
  else $("bench-charts").innerHTML = `<p class="bench-live">Waiting for run.json (charts appear when the suite finishes or writes partial results).</p>`;
  await Promise.all(FEEDS.map(f => renderFeed(f, runId)));
}

async function show(route) {
  if (!route.runId) return renderList();
  $("bench-list").hidden = true;
  $("bench-detail").hidden = false;
  $("bench-title").textContent = "Bench run";
  await renderDetail(route.runId);
  if (!pollTimer) {
    pollTimer = setInterval(() => {
      const r = currentRoute();
      if (r.cat === "bench" && r.runId) renderDetail(r.runId).catch(() => {});
    }, 1500);
  }
}

export default {
  cats: ["bench"],
  show: route => show(route).catch(err => { $("bench-meta").textContent = String(err); }),
  hide: stopPolling,
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
