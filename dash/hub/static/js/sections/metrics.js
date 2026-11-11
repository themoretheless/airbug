// Signals › Metrics: series picker, native OTLP Histogram/ExponentialHistogram quantiles
// (p50/p90/p95/p99), trace-linked Exemplars, plus one renderer per visualization.

import { getJson } from "../api.js";
import { $, escapeAttr, escapeHtml, renderToggles, toggleKeepOne } from "../dom.js";
import { barSvg, gaugeSvg, heatmapSvg, histSvg, lastPoint, pieSvg, seriesLabel, sparkSvg } from "../charts.js";
import { fmtValue, formatTime } from "../format.js";

const subOf = s => [s.service, s.attrs].filter(Boolean).join(" · ");

function board(title, sub, now, svg, wide) {
  return `<article class="metric-board${wide ? " wide" : ""}">
    <header>
      <div>
        <h3>${escapeHtml(title)}</h3>
        ${sub ? `<div class="sub">${escapeHtml(sub)}</div>` : ""}
      </div>
      ${now != null ? `<div class="now">${escapeHtml(now)}</div>` : ""}
    </header>
    ${svg}
  </article>`;
}

function bucketBarsSvg(buckets) {
  const items = buckets || [];
  if (!items.length) return "";
  const w = 440;
  const h = 92;
  const padX = 8;
  const padY = 10;
  const maxC = Math.max(1, ...items.map(b => Number(b.count || 0)));
  const slot = (w - padX * 2) / items.length;
  const bw = Math.max(2, slot - 2);
  const bars = items.map((b, idx) => {
    const c = Number(b.count || 0);
    const bh = c > 0 ? Math.max(2, ((h - padY * 2 - 14) * c) / maxC) : 0;
    const x = padX + idx * slot;
    const y = h - padY - 12 - bh;
    return `<g>
      <title>${escapeHtml(`${b.label}: ${c}`)}</title>
      <rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${bw.toFixed(1)}" height="${bh.toFixed(1)}" rx="2" fill="#5b8def" opacity="0.9" />
    </g>`;
  }).join("");
  const firstLbl = items[0] ? items[0].label : "";
  const lastLbl = items[items.length - 1] ? items[items.length - 1].label : "";
  return `<svg class="pct-svg" viewBox="0 0 ${w} ${h}" role="img" aria-label="Histogram bucket distribution">
    ${bars}
    <text x="${padX}" y="${h - 4}" class="pct-axis">${escapeHtml(firstLbl)}</text>
    <text x="${w - padX}" y="${h - 4}" text-anchor="end" class="pct-axis">${escapeHtml(lastLbl)}</text>
  </svg>`;
}

function percentilesBoards(percentiles) {
  const list = percentiles || [];
  if (!list.length) {
    return `<article class="metric-board wide"><p class="metrics-empty">No OTLP Histogram, ExponentialHistogram, or Summary instruments recorded yet.</p></article>`;
  }
  return list.map(h => {
    const u = h.unit || "";
    const meta = [h.service, h.kind, h.temporality, h.attrs].filter(Boolean).join(" · ");
    const stats = `<div class="pct-strip">
      <div><small>p50</small><strong>${escapeHtml(fmtValue(h.p50, u))}</strong></div>
      <div><small>p90</small><strong>${escapeHtml(fmtValue(h.p90, u))}</strong></div>
      <div><small>p95</small><strong>${escapeHtml(fmtValue(h.p95, u))}</strong></div>
      <div><small>p99</small><strong>${escapeHtml(fmtValue(h.p99, u))}</strong></div>
      <div><small>count</small><strong>${escapeHtml(String(h.count || 0))}</strong></div>
      ${h.max != null ? `<div><small>max</small><strong>${escapeHtml(fmtValue(h.max, u))}</strong></div>` : ""}
    </div>`;
    return board(h.name, meta, `p95 ${fmtValue(h.p95, u)}`, stats + bucketBarsSvg(h.buckets), false);
  }).join("");
}

/**
 * `boards(shown, histogram, percentiles)` → markup for the board area; `table` says whether the
 * latest-values table shows underneath; `combined` boards take the full width.
 */
const VIZ = [
  { id: "timeseries", label: "Time series", table: true,
    boards: shown => shown.map((s, i) => board(s.name, subOf(s), fmtValue(lastPoint(s), s.unit || ""), sparkSvg(s.points || [], "mf" + i))).join("") },
  { id: "percentiles", label: "Percentiles (p50/p95/p99)", table: true, usesPercentiles: true,
    boards: (_shown, _h, pcts) => percentilesBoards(pcts) },
  { id: "gauge", label: "Gauge", table: true,
    boards: shown => shown.map(s => board(s.name, subOf(s), null, gaugeSvg(lastPoint(s), s.name, s.unit || ""))).join("") },
  { id: "bar", label: "Bar", combined: true,
    boards: shown => board("Bar · selected series", shown.length + " series", null, barSvg(shown), true) },
  { id: "histogram", label: "Histogram", combined: true,
    boards: (_shown, h) => board("Histogram · all recent points", h && h.bucket_count ? h.bucket_count + " buckets" : "", null, histSvg(h), true) },
  { id: "heatmap", label: "Heatmap", combined: true,
    boards: shown => board("Heatmap · time × series", shown.length + " series", null, heatmapSvg(shown), true) },
  { id: "pie", label: "Pie", combined: true,
    boards: shown => board("Pie · latest values", shown.length + " series", null, pieSvg(shown), true) },
  { id: "table", label: "Table", table: true, combined: true, boards: () => "" },
];

let viz = VIZ[0];
let selected = new Set();
let seriesIds = [];
let last = null;

/** Keeps the selection across refreshes; a different series set resets it to the first four. */
function syncSelection(series) {
  const ids = series.map(s => s.id);
  const same = ids.length === seriesIds.length && ids.every((id, i) => id === seriesIds[i]);
  if (!same) {
    seriesIds = ids;
    selected = new Set(ids.slice(0, 4));
    return;
  }
  for (const id of [...selected]) if (!ids.includes(id)) selected.delete(id);
  if (!selected.size && ids.length) selected = new Set(ids.slice(0, 4));
}

function renderPickers(series) {
  renderToggles($("viz-picker"), VIZ.map(v => [v.id, v.label]), id => id === viz.id, id => {
    viz = VIZ.find(v => v.id === id) || VIZ[0];
    paint();
  }, "data-viz");
  const pick = $("metrics-pick");
  pick.hidden = Boolean(viz.usesPercentiles) || !series.length;
  pick.innerHTML = series.map(s =>
    `<button type="button" class="${selected.has(s.id) ? "on" : ""}" data-sid="${escapeAttr(s.id)}" title="${escapeAttr(s.id)}">${escapeHtml(seriesLabel(s))}</button>`
  ).join("");
  pick.querySelectorAll("button[data-sid]").forEach(btn => btn.addEventListener("click", () => {
    toggleKeepOne(selected, btn.getAttribute("data-sid"));
    paint();
  }));
}

function renderExemplars(exemplars) {
  const el = $("metrics-exemplars");
  if (!el) return;
  const items = (exemplars || []).slice(-16).reverse();
  el.hidden = !items.length;
  if (!items.length) {
    el.innerHTML = "";
    return;
  }
  el.innerHTML = `<div class="exemplars-head"><strong>Trace Exemplars</strong> <span>${items.length} sample(s) linking metric spikes to spans</span></div>
    <div class="exemplars-list">${items.map(ex => {
      const val = fmtValue(ex.value, ex.unit || "");
      const link = ex.trace_id
        ? `<a class="trace-pill" href="#/traces/${escapeAttr(ex.trace_id)}">trace:${escapeHtml(ex.trace_id.slice(0, 8))} ↗</a>`
        : "";
      return `<span class="exemplar-chip"><code>${escapeHtml(ex.metric)}</code> = <strong>${escapeHtml(val)}</strong> ${link}</span>`;
    }).join("")}</div>`;
}

function tableHtml(latest) {
  const rows = latest.map(m => {
    const when = m.time_ms ? formatTime(m) : (m.time || "—");
    const unit = m.unit ? ` ${escapeHtml(m.unit)}` : "";
    const kindMeta = [m.kind || "", m.temporality || "", m.monotonic === true ? "monotonic" : ""].filter(Boolean).join(" · ");
    const sub = [m.service, m.scope, m.attrs].filter(Boolean).join(" · ");
    return `<tr>
      <td class="name">${escapeHtml(m.name)}<div class="sub">${escapeHtml(kindMeta)}</div></td>
      <td class="val">${Number(m.value).toPrecision(6)}${unit}<div class="sub">${escapeHtml(when)}</div></td>
      <td>${sub ? `<div class="sub">${escapeHtml(sub)}</div>` : "—"}</td>
    </tr>`;
  }).join("");
  return `<table class="metrics-table">
    <thead><tr><th>metric</th><th>value</th><th>labels</th></tr></thead>
    <tbody>${rows}</tbody>
  </table>`;
}

function paint() {
  if (!last) return;
  const series = last.series || [];
  const latest = last.latest || [];
  const percentiles = last.percentiles || [];
  const empty = `<p class="metrics-empty">${escapeHtml(last.note || "No metrics yet.")}</p>`;
  $("metrics-meta").textContent = last.note || "";
  syncSelection(series);
  renderPickers(series);
  renderExemplars(last.exemplars || []);
  const boards = $("metrics-boards");
  const panel = $("metrics-panel");
  boards.classList.toggle("single", Boolean(viz.combined) || (viz.usesPercentiles && !percentiles.length));
  boards.innerHTML = (series.length || viz.usesPercentiles)
    ? viz.boards(series.filter(s => selected.has(s.id)), last.histogram, percentiles)
    : "";
  panel.hidden = !viz.table && Boolean(series.length || latest.length || percentiles.length);
  panel.innerHTML = latest.length ? tableHtml(latest) : empty;
}

export async function refresh() {
  last = await getJson("metrics", { limit: 800 });
  paint();
}

export default {
  cats: ["metrics"],
  show: () => refresh().catch(err => { $("metrics-meta").textContent = "failed to load /api/v1/metrics: " + err; }),
};
