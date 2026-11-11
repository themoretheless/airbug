// Signals › Traces: distributed trace list, interactive Gantt Waterfall + Critical Path,
// Service Topology SVG dependency graph, RED metrics, zPages Tracez latency matrix,
// and automatic Trace ↔ Logs ↔ Issues correlation.

import { enc, getJson, tryJson } from "../api.js";
import { $, copy, escapeAttr, escapeHtml, renderToggles } from "../dom.js";
import { formatTime, sevClass } from "../format.js";
import { currentRoute } from "../shell.js";

const SUBVIEWS = [
  ["spans", "Traces & Waterfall"],
  ["map", "Service Map"],
  ["red", "RED & zPages Tracez"],
];

let subview = "spans";
let lastList = null;
let lastTracez = null;
let lastDetail = null;
let detailTraceId = null;
let selectedSpanId = null;
let corrTab = "logs";

const fmtMs = ms => {
  const n = Number(ms);
  if (!Number.isFinite(n)) return "—";
  if (n >= 1000) return (n / 1000).toFixed(2) + " s";
  if (n >= 10) return n.toFixed(1) + " ms";
  if (n >= 1) return n.toFixed(2) + " ms";
  return (n * 1000).toFixed(0) + " µs";
};

function syncServiceSelect(services) {
  const el = $("traces-service");
  if (!el) return;
  const cur = el.value;
  el.innerHTML = [`<option value="">all services</option>`]
    .concat((services || []).map(s => `<option value="${escapeAttr(s)}">${escapeHtml(s)}</option>`))
    .join("");
  if (cur && (services || []).includes(cur)) el.value = cur;
}

function renderSubviewPicker() {
  const el = $("traces-views");
  if (!el) return;
  renderToggles(el, SUBVIEWS, id => id === subview, id => {
    subview = id;
    renderSubviewPicker();
    if (subview === "red" && !lastTracez) {
      tryJson("zpages/tracez", null, null).then(d => { lastTracez = d; paintList(); });
    }
    paintList();
  }, "data-tview");
}

function renderServiceTopologySvg(topology) {
  const nodes = (topology && topology.nodes) || [];
  const edges = (topology && topology.edges) || [];
  if (!nodes.length) {
    return `<div class="traces-card"><p class="traces-empty">No services observed in traces yet.</p></div>`;
  }

  // Assign layered ranks (callers on left → callees on right), handling cycles gracefully.
  const byName = new Map(nodes.map(n => [n.service, n]));
  const indeg = new Map(nodes.map(n => [n.service, 0]));
  const outAdj = new Map(nodes.map(n => [n.service, []]));
  for (const e of edges) {
    if (byName.has(e.caller) && byName.has(e.callee) && e.caller !== e.callee) {
      outAdj.get(e.caller).push(e.callee);
      indeg.set(e.callee, (indeg.get(e.callee) || 0) + 1);
    }
  }
  const rank = new Map();
  const queue = [];
  for (const [svc, d] of indeg.entries()) {
    if (d === 0) {
      rank.set(svc, 0);
      queue.push(svc);
    }
  }
  while (queue.length) {
    const u = queue.shift();
    const r = rank.get(u) || 0;
    for (const v of outAdj.get(u) || []) {
      if (!rank.has(v) || rank.get(v) < r + 1) {
        if (r + 1 < nodes.length) {
          rank.set(v, r + 1);
          queue.push(v);
        }
      }
    }
  }
  for (const n of nodes) {
    if (!rank.has(n.service)) rank.set(n.service, 0);
  }

  const layers = new Map();
  let maxRank = 0;
  for (const n of nodes) {
    const r = rank.get(n.service) || 0;
    if (r > maxRank) maxRank = r;
    if (!layers.has(r)) layers.set(r, []);
    layers.get(r).push(n);
  }

  const colCount = Math.max(1, maxRank + 1);
  const maxRows = Math.max(1, ...Array.from(layers.values(), arr => arr.length));
  const nodeW = 190;
  const nodeH = 66;
  const gapX = 130;
  const gapY = 34;
  const padX = 36;
  const padY = 32;
  const width = Math.max(680, padX * 2 + colCount * nodeW + (colCount - 1) * gapX);
  const height = Math.max(220, padY * 2 + maxRows * nodeH + (maxRows - 1) * gapY);

  const pos = new Map();
  for (let c = 0; c < colCount; c++) {
    const colNodes = layers.get(c) || [];
    const totalH = colNodes.length * nodeH + Math.max(0, colNodes.length - 1) * gapY;
    const startY = Math.max(padY, (height - totalH) / 2);
    colNodes.forEach((n, idx) => {
      const x = padX + c * (nodeW + gapX);
      const y = startY + idx * (nodeH + gapY);
      pos.set(n.service, { x, y, cx: x + nodeW / 2, cy: y + nodeH / 2 });
    });
  }

  const edgeSvg = edges.map(e => {
    const a = pos.get(e.caller);
    const b = pos.get(e.callee);
    if (!a || !b) return "";
    const forward = b.x > a.x;
    const x1 = forward ? a.x + nodeW : a.cx;
    const y1 = forward ? a.cy : a.y + nodeH;
    const x2 = forward ? b.x - 6 : b.cx;
    const y2 = forward ? b.cy : b.y - 6;
    const mx = (x1 + x2) / 2;
    const my = (y1 + y2) / 2 - (forward ? 0 : 18);
    const pathD = forward
      ? `M${x1},${y1} C${mx},${y1} ${mx},${y2} ${x2},${y2}`
      : `M${x1},${y1} Q${mx},${my - 28} ${x2},${y2}`;
    const hasErr = (e.errors || 0) > 0;
    const stroke = hasErr ? "var(--bad)" : "var(--glow)";
    const label = `${e.calls} call${e.calls === 1 ? "" : "s"} · p95 ${fmtMs(e.p95_ms)}${hasErr ? ` · ${e.error_rate_pct.toFixed(0)}% err` : ""}`;
    return `<g class="topo-edge">
      <path d="${pathD}" fill="none" stroke="${stroke}" stroke-width="${hasErr ? 2.2 : 1.7}" marker-end="url(#topo-arrow-${hasErr ? "err" : "ok"})" />
      <rect x="${(mx - 68).toFixed(1)}" y="${(my - 11).toFixed(1)}" width="136" height="19" rx="4" fill="var(--card)" stroke="var(--line)" />
      <text x="${mx.toFixed(1)}" y="${(my + 2).toFixed(1)}" text-anchor="middle" class="topo-edge-label">${escapeHtml(label)}</text>
    </g>`;
  }).join("");

  const nodeSvg = nodes.map(n => {
    const p = pos.get(n.service);
    const hasErr = (n.errors || 0) > 0;
    const border = hasErr ? "var(--bad)" : "var(--glow)";
    const sub1 = `${n.spans} span${n.spans === 1 ? "" : "s"}${hasErr ? ` · ${n.errors} err (${n.error_rate_pct.toFixed(1)}%)` : " · 0 err"}`;
    const sub2 = `p50 ${fmtMs(n.p50_ms)} · p95 ${fmtMs(n.p95_ms)}`;
    return `<g class="topo-node" data-svc="${escapeAttr(n.service)}" transform="translate(${p.x.toFixed(1)},${p.y.toFixed(1)})">
      <rect width="${nodeW}" height="${nodeH}" rx="8" fill="var(--card)" stroke="${border}" stroke-width="1.6" />
      <text x="12" y="22" class="topo-node-title">${escapeHtml(n.service)}</text>
      <text x="12" y="40" class="topo-node-sub${hasErr ? " err" : ""}">${escapeHtml(sub1)}</text>
      <text x="12" y="55" class="topo-node-sub">${escapeHtml(sub2)}</text>
    </g>`;
  }).join("");

  return `<div class="traces-card">
    <div class="traces-card-head">
      <h3>Service Dependency Map</h3>
      <span class="traces-hint">${nodes.length} service(s) · ${edges.length} cross-service edge(s) — click a service node to filter</span>
    </div>
    <div class="topo-scroll">
      <svg class="topo-svg" viewBox="0 0 ${width} ${height}" role="img" aria-label="Service dependency topology">
        <defs>
          <marker id="topo-arrow-ok" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 1 L 10 5 L 0 9 z" fill="var(--glow)" />
          </marker>
          <marker id="topo-arrow-err" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 1 L 10 5 L 0 9 z" fill="var(--bad)" />
          </marker>
        </defs>
        ${edgeSvg}
        ${nodeSvg}
      </svg>
    </div>
  </div>`;
}

function renderRedAndTracez(red, tracez) {
  const redRows = (red || []).map(r => {
    const errCls = r.errors > 0 ? " class=\"has-err\"" : "";
    return `<tr${errCls}>
      <td><code>${escapeHtml(r.service)}</code></td>
      <td><strong>${escapeHtml(r.operation)}</strong> <span class="span-kind-chip">${escapeHtml(r.kind || "")}</span></td>
      <td class="num">${r.count}</td>
      <td class="num">${Number(r.rate_per_sec || 0).toFixed(2)}/s</td>
      <td class="num">${r.errors ? `<span class="err-txt">${r.errors} (${r.error_rate_pct.toFixed(1)}%)</span>` : "0"}</td>
      <td class="num">${fmtMs(r.p50_ms)}</td>
      <td class="num">${fmtMs(r.p95_ms)}</td>
      <td class="num">${fmtMs(r.p99_ms)}</td>
      <td class="num">${fmtMs(r.max_ms)}</td>
    </tr>`;
  }).join("");

  const redCard = `<div class="traces-card">
    <div class="traces-card-head">
      <h3>RED Metrics (Rate · Errors · Duration)</h3>
      <span class="traces-hint">Root &amp; server operations</span>
    </div>
    ${redRows ? `<div class="table-scroll"><table class="traces-table">
      <thead><tr><th>service</th><th>operation</th><th class="num">spans</th><th class="num">rate</th><th class="num">errors</th><th class="num">p50</th><th class="num">p95</th><th class="num">p99</th><th class="num">max</th></tr></thead>
      <tbody>${redRows}</tbody>
    </table></div>` : `<p class="traces-empty">No root or server operations recorded yet.</p>`}
  </div>`;

  const labels = (tracez && tracez.bucket_labels) || ["<10µs", "<100µs", "<1ms", "<10ms", "<100ms", "<1s", "<10s", ">=10s"];
  const ops = (tracez && tracez.operations) || [];
  const zRows = ops.map(op => {
    const counts = op.bucket_counts || [];
    const samples = op.sample_trace_ids || {};
    const cells = labels.map((lbl, idx) => {
      const c = Number(counts[idx] != null ? counts[idx] : (op.buckets && op.buckets[lbl]) || 0);
      if (!c) return `<td class="num z-zero">·</td>`;
      const tid = samples[lbl];
      return `<td class="num z-hit">${tid ? `<a href="#/traces/${escapeAttr(tid)}" title="Open sample trace ${escapeAttr(tid.slice(0, 8))}">${c}</a>` : c}</td>`;
    }).join("");
    const errSample = (op.error_samples && op.error_samples[0]) || null;
    const errCell = op.error_count
      ? (errSample
        ? `<a class="err-txt" href="#/traces/${escapeAttr(errSample.trace_id)}" title="${escapeAttr(errSample.message || "error sample")}">${op.error_count}</a>`
        : `<span class="err-txt">${op.error_count}</span>`)
      : "0";
    return `<tr>
      <td><code>${escapeHtml(op.service)}</code></td>
      <td><strong>${escapeHtml(op.operation)}</strong></td>
      <td class="num">${op.running_count || 0}</td>
      ${cells}
      <td class="num">${errCell}</td>
    </tr>`;
  }).join("");

  const tracezCard = `<div class="traces-card">
    <div class="traces-card-head">
      <h3>OpenTelemetry zPages · Tracez</h3>
      <span class="traces-hint">Canonical latency buckets — click any non-zero cell to inspect a sample trace</span>
    </div>
    ${zRows ? `<div class="table-scroll"><table class="traces-table tracez-table">
      <thead><tr><th>service</th><th>span name</th><th class="num">running</th>${labels.map(l => `<th class="num">${escapeHtml(l)}</th>`).join("")}<th class="num">errors</th></tr></thead>
      <tbody>${zRows}</tbody>
    </table></div>` : `<p class="traces-empty">${escapeHtml((tracez && tracez.note) || "No zPages spans collected yet.")}</p>`}
  </div>`;

  return redCard + tracezCard;
}

function renderTraceRow(t) {
  const isErr = t.status === "ERROR" || t.error_count > 0;
  const svcBadges = (t.service_counts || []).map(sc =>
    `<span class="svc-pill${sc.errors ? " err" : ""}">${escapeHtml(sc.service)} ×${sc.count}</span>`
  ).join("");
  const when = t.time_ms ? formatTime(t) : "";
  return `<a class="trace-row${isErr ? " has-err" : ""}" href="#/traces/${escapeAttr(t.trace_id)}">
    <span class="trace-status ${isErr ? "error" : "ok"}">${isErr ? "ERROR" : "OK"}</span>
    <div class="trace-row-main">
      <div class="trace-row-title">
        <strong>${escapeHtml(t.root_service)}</strong>
        <span class="trace-op">${escapeHtml(t.root_name)}</span>
        <span class="span-kind-chip">${escapeHtml(t.root_kind || "INTERNAL")}</span>
      </div>
      <div class="trace-row-sub">
        <code>${escapeHtml(t.trace_id.slice(0, 16))}…</code>
        <span>${t.span_count} span${t.span_count === 1 ? "" : "s"}</span>
        ${t.error_count ? `<span class="err-txt">${t.error_count} error${t.error_count === 1 ? "" : "s"}</span>` : ""}
        ${svcBadges}
      </div>
    </div>
    <div class="trace-row-right">
      <strong class="trace-dur">${fmtMs(t.duration_ms)}</strong>
      <small>${escapeHtml(when)}</small>
    </div>
  </a>`;
}

function paintList() {
  if (!lastList) return;
  $("traces-meta").textContent = lastList.note || "";
  syncServiceSelect(lastList.services || []);
  renderSubviewPicker();

  const analytics = $("traces-analytics");
  if (subview === "map") {
    analytics.hidden = false;
    analytics.innerHTML = renderServiceTopologySvg(lastList.topology);
    analytics.querySelectorAll(".topo-node[data-svc]").forEach(node => {
      node.addEventListener("click", () => {
        const svc = node.getAttribute("data-svc") || "";
        $("traces-service").value = svc;
        refreshList();
      });
    });
  } else if (subview === "red") {
    analytics.hidden = false;
    analytics.innerHTML = renderRedAndTracez(lastList.red, lastTracez);
  } else {
    analytics.hidden = true;
    analytics.innerHTML = "";
  }

  const listEl = $("traces-list");
  const traces = lastList.traces || [];
  if (!traces.length) {
    listEl.innerHTML = `<p class="traces-empty">${escapeHtml(lastList.note || "No traces match current filters.")}</p>`;
    return;
  }
  listEl.innerHTML = traces.map(renderTraceRow).join("");
}

function kvTable(obj) {
  const entries = Object.entries(obj || {}).sort(([a], [b]) => a.localeCompare(b));
  if (!entries.length) return `<p class="span-empty">None</p>`;
  return `<table class="kv-table"><tbody>${entries.map(([k, v]) =>
    `<tr><th>${escapeHtml(k)}</th><td><code>${escapeHtml(v)}</code></td></tr>`
  ).join("")}</tbody></table>`;
}

function renderSpanInspector(trace, span) {
  const el = $("span-inspector");
  if (!span) {
    el.hidden = true;
    el.innerHTML = "";
    return;
  }
  el.hidden = false;
  const isErr = span.status === "ERROR";
  const eventsHtml = (span.events || []).length
    ? span.events.map(ev => {
      const attrs = ev.attrs || {};
      const stack = attrs["exception.stacktrace"];
      const rest = Object.fromEntries(Object.entries(attrs).filter(([k]) => k !== "exception.stacktrace"));
      return `<div class="span-event-card${ev.name === "exception" ? " exception" : ""}">
        <div class="span-event-head">
          <strong>${escapeHtml(ev.name)}</strong>
          <span>+${fmtMs(ev.offset_ms)}</span>
        </div>
        ${Object.keys(rest).length ? kvTable(rest) : ""}
        ${stack ? `<pre class="span-stack">${escapeHtml(stack)}</pre>` : ""}
      </div>`;
    }).join("")
    : `<p class="span-empty">No span events</p>`;

  const linksHtml = (span.links || []).length
    ? `<ul class="span-links-list">${span.links.map(lnk =>
      `<li><a class="trace-pill" href="#/traces/${escapeAttr(lnk.trace_id)}">trace:${escapeHtml(lnk.trace_id.slice(0, 12))}…</a> <code>span:${escapeHtml(lnk.span_id)}</code></li>`
    ).join("")}</ul>`
    : `<p class="span-empty">No span links</p>`;

  el.innerHTML = `<div class="span-insp-head">
      <div>
        <div class="span-insp-kicker">${escapeHtml(span.service)} · <span class="span-kind-chip">${escapeHtml(span.kind)}</span>${span.on_critical_path ? ` <span class="cp-badge">CRITICAL PATH</span>` : ""}${span.orphan ? ` <span class="orphan-badge">ORPHAN PARENT</span>` : ""}</div>
        <h3>${escapeHtml(span.name)}</h3>
      </div>
      <button type="button" class="span-insp-close" id="span-insp-close" title="Close inspector">×</button>
    </div>
    <div class="span-metrics-strip">
      <div><small>duration</small><strong>${fmtMs(span.duration_ms)}</strong></div>
      <div><small>self time</small><strong>${fmtMs(span.self_time_ms)}</strong></div>
      <div><small>start offset</small><strong>+${fmtMs(span.offset_ms)}</strong></div>
      <div><small>status</small><strong class="${isErr ? "err-txt" : ""}">${escapeHtml(span.status)}${span.status_message ? " · " + escapeHtml(span.status_message) : ""}</strong></div>
    </div>
    <div class="span-ids">
      <span>span <code>${escapeHtml(span.span_id)}</code></span>
      ${span.parent_span_id ? `<span>parent <code>${escapeHtml(span.parent_span_id)}</code></span>` : `<span>root span</span>`}
      ${span.scope ? `<span>scope <code>${escapeHtml(span.scope)}${span.scope_version ? "@" + escapeHtml(span.scope_version) : ""}</code></span>` : ""}
    </div>
    <h4>Span Attributes</h4>
    ${kvTable(span.attrs)}
    <h4>Events (${(span.events || []).length})</h4>
    ${eventsHtml}
    <h4>Links (${(span.links || []).length})</h4>
    ${linksHtml}
    <h4>Resource Attributes</h4>
    ${kvTable(span.resource_attrs)}`;

  const closeBtn = $("span-insp-close");
  if (closeBtn) {
    closeBtn.onclick = () => {
      selectedSpanId = null;
      paintDetail();
    };
  }
}

function renderCorrelatedSection(logs, issues) {
  const tabsEl = $("trace-corr-tabs");
  const bodyEl = $("trace-corr-body");
  renderToggles(tabsEl, [
    ["logs", `Correlated Logs (${logs.length})`],
    ["issues", `Correlated Issues (${issues.length})`],
  ], id => id === corrTab, id => {
    corrTab = id;
    renderCorrelatedSection(logs, issues);
  }, "data-ctab");

  if (corrTab === "issues") {
    if (!issues.length) {
      bodyEl.innerHTML = `<p class="traces-empty">No airbug-err issues recorded with this trace_id.</p>`;
      return;
    }
    bodyEl.innerHTML = issues.map(i => `<a class="corr-issue-row" href="#/issues">
      <span class="issue-status ${escapeAttr(i.status || "unresolved")}">${escapeHtml(i.status || "unresolved")}</span>
      <strong>${escapeHtml(i.id)}</strong>
      <span>${escapeHtml(i.title || "")}</span>
      <span class="corr-right">×${i.count || 1}</span>
    </a>`).join("");
    return;
  }

  if (!logs.length) {
    bodyEl.innerHTML = `<p class="traces-empty">No OTLP logs recorded with this trace_id.</p>`;
    return;
  }
  bodyEl.innerHTML = `<div class="corr-logs">${logs.map(e => `<div class="log-row">
    <div class="log-time">${escapeHtml(formatTime(e))}</div>
    <div class="log-sev ${sevClass(e.severity)}">${escapeHtml(e.severity || "INFO")}</div>
    <div class="log-body">${escapeHtml(e.body || "")}${e.span_id ? ` <code class="span-chip">span:${escapeHtml(e.span_id.slice(0, 8))}</code>` : ""}</div>
  </div>`).join("")}</div>`;
}

function paintDetail() {
  if (!lastDetail || !lastDetail.trace) return;
  const trace = lastDetail.trace;
  const spans = trace.spans || [];
  const totalMs = Math.max(trace.duration_ms || 0.001, 0.001);

  if (!selectedSpanId || !spans.some(s => s.span_id === selectedSpanId)) {
    const firstErr = spans.find(s => s.status === "ERROR");
    selectedSpanId = (firstErr || spans[0] || {}).span_id || null;
  }

  $("traces-title").textContent = `${trace.root_service} · ${trace.root_name}`;
  $("traces-meta").textContent = `${trace.span_count} span(s) · duration ${fmtMs(trace.duration_ms)} · critical path ${fmtMs(trace.critical_path_ms)}`;

  $("trace-detail-summary").innerHTML = `<code>${escapeHtml(trace.trace_id)}</code>
    <button type="button" class="mini-btn" id="copy-trace-id">copy trace_id</button>
    <a class="mini-btn" href="#/profiles" data-prof-trace="${escapeAttr(trace.trace_id)}" id="open-trace-profiles">Profiles for trace</a>
    ${lastDetail.jaeger_available && lastDetail.jaeger_url ? `<a class="mini-btn" href="${escapeAttr(lastDetail.jaeger_url)}" target="_blank" rel="noopener">Jaeger ↗</a>` : ""}`;

  const copyBtn = $("copy-trace-id");
  if (copyBtn) copyBtn.onclick = () => copy(trace.trace_id);

  // Ruler ticks: 0%, 25%, 50%, 75%, 100%.
  const ticks = [0, 0.25, 0.5, 0.75, 1].map(f =>
    `<span style="left:${(f * 100).toFixed(0)}%">${fmtMs(totalMs * f)}</span>`
  ).join("");

  const wfRows = spans.map(s => {
    const leftPct = Math.max(0, Math.min(99.2, (s.offset_ms / totalMs) * 100));
    const widthPct = Math.max(0.8, Math.min(100 - leftPct, (s.duration_ms / totalMs) * 100));
    const isErr = s.status === "ERROR";
    const isSel = s.span_id === selectedSpanId;
    const indentPx = Math.min(s.depth || 0, 12) * 14;
    const evDots = (s.events || []).map(ev => {
      const evLeft = totalMs > 0
        ? Math.max(0, Math.min(100, ((s.offset_ms + ev.offset_ms) / totalMs) * 100))
        : leftPct;
      return `<span class="wf-event-dot${ev.name === "exception" ? " exception" : ""}" style="left:${evLeft.toFixed(2)}%" title="${escapeAttr(ev.name + " (+" + fmtMs(ev.offset_ms) + ")")}"></span>`;
    }).join("");

    return `<div class="wf-row${isSel ? " selected" : ""}${isErr ? " is-err" : ""}${s.on_critical_path ? " is-cp" : ""}" data-span-id="${escapeAttr(s.span_id)}" role="button" tabindex="0">
      <div class="wf-label" style="padding-left:${indentPx + 8}px">
        <span class="wf-svc">${escapeHtml(s.service)}</span>
        <span class="wf-name">${escapeHtml(s.name)}</span>
        <span class="span-kind-chip">${escapeHtml(s.kind)}</span>
        ${s.on_critical_path ? `<span class="cp-dot" title="On Critical Path">CP</span>` : ""}
      </div>
      <div class="wf-track">
        <div class="wf-bar${isErr ? " err" : ""}${s.on_critical_path ? " cp" : ""}" style="left:${leftPct.toFixed(2)}%;width:${widthPct.toFixed(2)}%"></div>
        ${evDots}
        <span class="wf-dur">${fmtMs(s.duration_ms)}</span>
      </div>
    </div>`;
  }).join("");

  const wf = $("trace-waterfall");
  wf.innerHTML = `<div class="wf-header">
      <div>Service &amp; Operation</div>
      <div class="wf-ruler">${ticks}</div>
    </div>
    <div class="wf-body">${wfRows}</div>`;

  wf.querySelectorAll(".wf-row[data-span-id]").forEach(row => {
    const pick = () => {
      selectedSpanId = row.getAttribute("data-span-id");
      paintDetail();
    };
    row.addEventListener("click", pick);
    row.addEventListener("keydown", ev => {
      if (ev.key === "Enter" || ev.key === " ") {
        ev.preventDefault();
        pick();
      }
    });
  });

  const activeSpan = spans.find(s => s.span_id === selectedSpanId) || null;
  renderSpanInspector(trace, activeSpan);
  renderCorrelatedSection(lastDetail.logs || [], lastDetail.issues || []);
}

async function refreshList() {
  const service = ($("traces-service") && $("traces-service").value) || "";
  const status = ($("traces-status") && $("traces-status").value) || "";
  const minMs = ($("traces-min-ms") && $("traces-min-ms").value) || "";
  const q = ($("traces-filter") && $("traces-filter").value.trim()) || "";
  const tasks = [getJson("traces", { limit: 120, service, status, min_ms: minMs, q })];
  if (subview === "red") tasks.push(tryJson("zpages/tracez", null, null));
  const [listResp, tracezResp] = await Promise.all(tasks);
  lastList = listResp;
  if (tracezResp) lastTracez = tracezResp;
  if (!detailTraceId) paintList();
}

async function loadDetail(traceId) {
  detailTraceId = traceId;
  const data = await getJson("traces/" + enc(traceId)).catch(() => null);
  if (!data || !data.trace) {
    $("traces-meta").textContent = `Trace ${traceId} not found`;
    $("trace-waterfall").innerHTML = `<p class="traces-empty">Trace <code>${escapeHtml(traceId)}</code> was not found in recent OTLP storage.</p>`;
    $("span-inspector").hidden = true;
    return;
  }
  lastDetail = data;
  paintDetail();
}

export async function refresh() {
  const r = currentRoute();
  if (r.cat === "traces" && r.runId) {
    await loadDetail(r.runId);
    return;
  }
  await refreshList();
}

async function show(route) {
  if (route.runId) {
    $("traces-list-wrap").hidden = true;
    $("traces-detail").hidden = false;
    if (detailTraceId !== route.runId) selectedSpanId = null;
    await loadDetail(route.runId);
    return;
  }
  detailTraceId = null;
  $("traces-title").textContent = "Traces";
  $("traces-list-wrap").hidden = false;
  $("traces-detail").hidden = true;
  await refreshList();
}

export function init() {
  renderSubviewPicker();
  ["traces-service", "traces-status"].forEach(id => {
    const el = $(id);
    if (el) el.addEventListener("change", () => refreshList().catch(() => {}));
  });
  ["traces-min-ms", "traces-filter"].forEach(id => {
    const el = $(id);
    if (el) el.addEventListener("input", () => refreshList().catch(() => {}));
  });
}

export default {
  cats: ["traces"],
  show: route => show(route).catch(err => { $("traces-meta").textContent = "failed to load /api/v1/traces: " + err; }),
  tick: n => { if (n % 3 === 0) refresh().catch(() => {}); },
};
