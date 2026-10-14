// Signals › Metrics: series picker plus one renderer per visualization (table-driven: a new
// visualization is one entry in VIZ).

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

/**
 * `boards(shown, histogram)` → markup for the board area; `table` says whether the latest-values
 * table shows underneath; `combined` boards take the full width.
 */
const VIZ = [
  { id: "timeseries", label: "Time series", table: true,
    boards: shown => shown.map((s, i) => board(s.name, subOf(s), fmtValue(lastPoint(s), s.unit || ""), sparkSvg(s.points || [], "mf" + i))).join("") },
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
  pick.hidden = !series.length;
  pick.innerHTML = series.map(s =>
    `<button type="button" class="${selected.has(s.id) ? "on" : ""}" data-sid="${escapeAttr(s.id)}" title="${escapeAttr(s.id)}">${escapeHtml(seriesLabel(s))}</button>`
  ).join("");
  pick.querySelectorAll("button[data-sid]").forEach(btn => btn.addEventListener("click", () => {
    toggleKeepOne(selected, btn.getAttribute("data-sid"));
    paint();
  }));
}

function tableHtml(latest) {
  const rows = latest.map(m => {
    const when = m.time_ms ? formatTime(m) : (m.time || "—");
    const unit = m.unit ? ` ${escapeHtml(m.unit)}` : "";
    const sub = [m.service, m.scope, m.attrs].filter(Boolean).join(" · ");
    return `<tr>
      <td class="name">${escapeHtml(m.name)}<div class="sub">${escapeHtml(m.kind || "")}</div></td>
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
  const empty = `<p class="metrics-empty">${escapeHtml(last.note || "No metrics yet.")}</p>`;
  $("metrics-meta").textContent = last.note || "";
  syncSelection(series);
  renderPickers(series);
  const boards = $("metrics-boards");
  const panel = $("metrics-panel");
  boards.classList.toggle("single", Boolean(viz.combined));
  boards.innerHTML = series.length ? viz.boards(series.filter(s => selected.has(s.id)), last.histogram) : "";
  panel.hidden = !viz.table && Boolean(series.length || latest.length);
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
