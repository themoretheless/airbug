// Signals › Profiles: OTLP Profiles (v1development) Flamechart / Icicle SVG viewer
// with click-to-zoom, hover tooltips, Self vs Total top functions, and linked traces.

import { getJson } from "../api.js";
import { $, escapeAttr, escapeHtml } from "../dom.js";
import { fmtValue } from "../format.js";

let last = null;
let zoomPath = [];
let renderedFrames = [];

function syncSelect(id, labelAll, values) {
  const el = $(id);
  if (!el) return;
  const cur = el.value;
  el.innerHTML = [`<option value="">${escapeHtml(labelAll)}</option>`]
    .concat((values || []).map(v => `<option value="${escapeAttr(v)}">${escapeHtml(v)}</option>`))
    .join("");
  if (cur && (values || []).includes(cur)) el.value = cur;
}

function frameColor(name, depth) {
  if (depth === 0) return "#334155";
  let h = 2166136261;
  const str = String(name || "fn");
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  const hue = 14 + (Math.abs(h) % 38); // warm terracotta-amber flame palette
  const sat = 66 + (Math.abs(h >> 8) % 18);
  const light = 46 + (depth % 3) * 4;
  return `hsl(${hue}, ${sat}%, ${light}%)`;
}

function resolveZoomNode(root) {
  let node = root;
  const validPath = [];
  for (const name of zoomPath) {
    const next = (node.children || []).find(c => c.name === name);
    if (!next) break;
    node = next;
    validPath.push(name);
  }
  zoomPath = validPath;
  return node;
}

function buildFlameSvg(root, unit) {
  if (!root || !root.value || !(root.children || []).length) {
    renderedFrames = [];
    return `<p class="profiles-empty">No profile stack frames match current filters.</p>`;
  }

  const focus = resolveZoomNode(root);
  const rootTotal = Math.max(1, Number(root.value) || 1);
  const width = 1080;
  const rowH = 24;
  const gap = 1.5;

  renderedFrames = [];
  let maxDepth = 0;

  function walk(node, depth, x0, spanW, pathFromFocus) {
    if (!node || !node.value || spanW < 1.2) return;
    if (depth > maxDepth) maxDepth = depth;
    const idx = renderedFrames.length;
    renderedFrames.push({
      node,
      depth,
      x: x0,
      w: spanW,
      path: pathFromFocus,
      pctRoot: (node.value / rootTotal) * 100,
      selfPctRoot: ((node.self_value || 0) / rootTotal) * 100,
    });
    let cursor = x0;
    for (const child of node.children || []) {
      const cw = (Number(child.value || 0) / Number(node.value || 1)) * spanW;
      walk(child, depth + 1, cursor, cw, pathFromFocus.concat([child.name]));
      cursor += cw;
    }
    return idx;
  }

  walk(focus, 0, 0, width, []);
  const height = Math.max(80, (maxDepth + 1) * (rowH + gap) + 12);

  const rects = renderedFrames.map((f, idx) => {
    const y = f.depth * (rowH + gap) + 4;
    const w = Math.max(1, f.w - 1);
    const fill = frameColor(f.node.name, f.depth);
    const maxChars = Math.floor((w - 10) / 6.6);
    const label = maxChars >= 4
      ? (f.node.name.length > maxChars ? f.node.name.slice(0, maxChars - 1) + "…" : f.node.name)
      : "";
    return `<g class="flame-frame" data-fidx="${idx}" transform="translate(${f.x.toFixed(2)},${y.toFixed(1)})">
      <rect width="${w.toFixed(2)}" height="${rowH}" rx="3" fill="${fill}" />
      ${label ? `<text x="6" y="16" class="flame-label">${escapeHtml(label)}</text>` : ""}
    </g>`;
  }).join("");

  const crumbs = zoomPath.length
    ? `<div class="flame-crumbs">Zoomed: <code>root</code> › ${zoomPath.map(p => `<code>${escapeHtml(p)}</code>`).join(" › ")} (${fmtValue(focus.value, unit)} · ${((focus.value / rootTotal) * 100).toFixed(1)}%)</div>`
    : "";

  return `${crumbs}<div class="flame-wrap">
    <svg class="flame-svg" viewBox="0 0 ${width} ${height}" preserveAspectRatio="none" role="img" aria-label="Profile icicle flamechart">
      ${rects}
    </svg>
  </div>`;
}

function renderTopTable(top, unit) {
  if (!top || !top.length) return "";
  const rows = top.slice(0, 30).map(fn => {
    const loc = fn.filename
      ? `${fn.filename}${fn.line > 0 ? ":" + fn.line : ""}`
      : "—";
    return `<tr>
      <td><code>${escapeHtml(fn.name)}</code></td>
      <td class="num">${escapeHtml(fmtValue(fn.self_value, unit))}</td>
      <td class="num">${Number(fn.self_pct || 0).toFixed(1)}%</td>
      <td class="num">${escapeHtml(fmtValue(fn.total_value, unit))}</td>
      <td class="num">${Number(fn.total_pct || 0).toFixed(1)}%</td>
      <td class="sub-loc"><code>${escapeHtml(loc)}</code></td>
    </tr>`;
  }).join("");

  return `<div class="traces-card">
    <div class="traces-card-head">
      <h3>Top Functions (Self vs Total)</h3>
      <span class="traces-hint">Sorted by self ${escapeHtml(unit || "value")}</span>
    </div>
    <div class="table-scroll">
      <table class="traces-table">
        <thead><tr><th>function</th><th class="num">self</th><th class="num">self %</th><th class="num">total</th><th class="num">total %</th><th>location</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </div>
  </div>`;
}

function bindFlameInteractions(unit) {
  const container = $("profiles-flame");
  const tip = $("profiles-tooltip");
  if (!container) return;

  container.querySelectorAll(".flame-frame[data-fidx]").forEach(el => {
    const f = renderedFrames[Number(el.getAttribute("data-fidx"))];
    if (!f) return;

    el.addEventListener("click", () => {
      if (f.depth === 0) {
        if (zoomPath.length) zoomPath.pop();
      } else {
        zoomPath = zoomPath.concat(f.path);
      }
      paint();
    });

    el.addEventListener("mousemove", ev => {
      if (!tip) return;
      const loc = f.node.file ? `${f.node.file}${f.node.line > 0 ? ":" + f.node.line : ""}` : "";
      tip.hidden = false;
      tip.innerHTML = `<strong>${escapeHtml(f.node.name)}</strong>
        ${loc ? `<div class="tip-loc">${escapeHtml(loc)}</div>` : ""}
        <div>Total: ${escapeHtml(fmtValue(f.node.value, unit))} (${f.pctRoot.toFixed(1)}%)</div>
        <div>Self: ${escapeHtml(fmtValue(f.node.self_value || 0, unit))} (${f.selfPctRoot.toFixed(1)}%)</div>`;
      const rect = container.getBoundingClientRect();
      tip.style.left = Math.min(rect.width - 220, Math.max(8, ev.clientX - rect.left + 12)) + "px";
      tip.style.top = Math.max(8, ev.clientY - rect.top + 12) + "px";
    });

    el.addEventListener("mouseleave", () => {
      if (tip) tip.hidden = true;
    });
  });
}

function paint() {
  if (!last) return;
  $("profiles-meta").textContent = last.note || "";
  syncSelect("profiles-service", "all services", last.services || []);
  syncSelect("profiles-type", "all sample types", last.sample_types || []);

  const resetBtn = $("profiles-zoom-reset");
  if (resetBtn) resetBtn.hidden = zoomPath.length === 0;

  const linksEl = $("profiles-links");
  const linked = last.linked_traces || [];
  linksEl.innerHTML = linked.length
    ? `<span class="traces-hint">Linked traces:</span> ` + linked.slice(0, 12).map(tid =>
      `<a class="trace-pill" href="#/traces/${escapeAttr(tid)}">trace:${escapeHtml(tid.slice(0, 8))}</a>`
    ).join(" ")
    : "";

  const unit = last.unit || "";
  const flameEl = $("profiles-flame");
  flameEl.innerHTML = buildFlameSvg(last.flamechart, unit);
  bindFlameInteractions(unit);

  $("profiles-top").innerHTML = renderTopTable(last.top_functions || [], unit);
}

export async function refresh() {
  const service = ($("profiles-service") && $("profiles-service").value) || "";
  const sampleType = ($("profiles-type") && $("profiles-type").value) || "";
  const traceId = ($("profiles-trace") && $("profiles-trace").value.trim()) || "";
  last = await getJson("profiles", {
    limit: 50,
    service,
    sample_type: sampleType,
    trace_id: traceId,
  });
  paint();
}

export function init() {
  ["profiles-service", "profiles-type"].forEach(id => {
    const el = $(id);
    if (el) el.addEventListener("change", () => { zoomPath = []; refresh().catch(() => {}); });
  });
  const trInput = $("profiles-trace");
  if (trInput) trInput.addEventListener("input", () => { zoomPath = []; refresh().catch(() => {}); });
  const resetBtn = $("profiles-zoom-reset");
  if (resetBtn) resetBtn.addEventListener("click", () => { zoomPath = []; paint(); });

  document.addEventListener("click", ev => {
    const link = ev.target && ev.target.closest && ev.target.closest("a[data-prof-trace]");
    if (!link) return;
    const tid = link.getAttribute("data-prof-trace") || "";
    if (trInput) trInput.value = tid;
    zoomPath = [];
  });
}

export default {
  cats: ["profiles"],
  show: () => refresh().catch(err => { $("profiles-meta").textContent = "failed to load /api/v1/profiles: " + err; }),
};
