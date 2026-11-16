// One run, any kind: Results / Output / Signals / Compare.
//
// `GET /api/v1/run?id=` says what the run is and where its files are served; each tab reads
// only what it shows and preserves interactive state across live polls.

import { enc, getJson, tryJson } from "../api.js";
import { createBenchExplorer } from "../bench_view.js";
import { sparkSvg } from "../charts.js";
import { $, bindCopy, copy, escapeAttr, escapeHtml } from "../dom.js";
import { fmtNs, fmtValue, formatTime, normalizeSev, SEV_ORDER, sevClass } from "../format.js";
import { currentRoute, setTitle } from "../shell.js";
import { fetchTimeline, TR } from "../runs.js";
import { createBenchCompareView, takePreferredBaseline } from "./bench.js";

const TABS = [["results", "Results"], ["output", "Output"], ["signals", "Signals"], ["compare", "Compare"]];
const MAX_TEXT = 400000;
const DONE = ["passed", "failed", "error", "interrupted", "complete", "cancelled", "incomplete"];

let run = null;          // description from /api/v1/run, or a stand-in for runs it does not know
let detail = null;       // test runs: /api/v1/runs/<id>
let view = null;         // test results view, kept across ticks
let viewKey = "";
let benchExplorer = null;
let benchHostRunId = "";
let runJsonKey = "";
let lastRunJson = null;
let pendingRerun = null;
let pace = { id: "", at: 0, done: 0 };

const baseline = {};     // run id -> chosen baseline id
const cmpThreshold = {}; // run id -> chosen % threshold
let cmpView = null;
let cmpViewRunId = "";

const outState = {
  runId: "",
  path: "",
  sig: "",
  rawText: "",
  query: "",
  level: "all",   // all | done | running | warn
  follow: true,
  wrap: true,
  pendingFocusCase: "",
};

const sigState = {
  runId: "",
  sig: "",
  tab: "traces",  // overview | traces | metrics | logs | issues
  query: "",
  sort: "seq",    // seq | dur_desc | name
  scale: "log",   // log | lin
  traces: [],
  logs: [],
  metricsData: null,
  issues: [],
  expandedTraceId: "",
  traceDetails: new Map(),
  expandedLogs: new Set(),
  sevs: new Set(SEV_ORDER),
  scopeText: "",
};

const note = text => `<p class="now-quiet">${escapeHtml(text)}</p>`;
const isTest = () => run && run.kind === "test";
const state = () => (isTest() ? (detail && detail.item && detail.item.state) : run && run.state) || "";
const live = () => Boolean(run) && !DONE.includes(state()) && !(detail && detail.item && detail.item.stale);
const tabOf = route => (TABS.some(([id]) => id === route.tab) ? route.tab : "results");
const here = (id, tab) => `#/run/${enc(id)}${tab && tab !== "results" ? "/" + tab : ""}`;

function fmtBytes(n) {
  const b = Number(n || 0);
  if (b >= 1048576) return (b / 1048576).toFixed(1) + " MB";
  if (b >= 1024) return (b / 1024).toFixed(1) + " KB";
  return b + " B";
}

async function fetchText(url) {
  const res = await fetch(url, { cache: "no-store" });
  if (!res.ok) throw new Error(`${res.status} ${url}`);
  const text = await res.text();
  return text.length > MAX_TEXT ? "… earlier output cut …\n" + text.slice(-MAX_TEXT) : text;
}

async function load(id) {
  const described = await tryJson("run", { id });
  if (described && described.kind !== "test") {
    run = described;
    detail = null;
    return;
  }
  detail = await getJson("runs/" + enc(id));
  const item = detail.item || {};
  run = described || { run_id: detail.run_id, kind: "test", title: item.title, started_at_ms: item.started_at_ms, files: detail.files };
}

function renderHead(route) {
  const tab = tabOf(route);
  const item = (detail && detail.item) || {};
  const started = run.started_at_ms || item.started_at_ms;
  const p = run.progress || {};
  const facts = [run.kind, started && TR ? TR.fmtWhen(started) : "", p.total ? `${p.completed}/${p.total} cases` : ""];
  if (run.matrix) facts.push(`${(run.cells || []).length} combinations`);
  $("run-head").innerHTML = `<a class="bench-back" href="#/runs">← all runs</a>
    <h2><span class="atr-state ${escapeAttr(state())}">${escapeHtml(state() || "…")}</span> ${escapeHtml(run.title || run.run_id)}</h2>
    <div class="runs-meta">${escapeHtml(facts.filter(Boolean).join(" · "))}</div>`;
  $("run-tabs").innerHTML = TABS.map(([id, label]) =>
    `<a href="${escapeAttr(here(route.runId, id))}" class="${id === tab ? "on" : ""}">${label}</a>`).join("");
  setTitle(run.title || "Run", state());
}

// ---- Results ----

function cellGrid() {
  const cells = run.cells || [];
  if (!cells.length) return "";
  const axes = [...new Set(cells.flatMap(c => Object.keys(c.axes || {})))];
  const head = (axes.length ? axes : ["part"]).map(a => `<th>${escapeHtml(a)}</th>`).join("");
  const rows = cells.map(c => {
    const names = axes.length ? axes.map(a => `<td><code>${escapeHtml((c.axes || {})[a] ?? "")}</code></td>`).join("") : `<td><code>${escapeHtml(c.name)}</code></td>`;
    const file = c.has_run_json ? `<a href="${escapeAttr(run.files + c.name + "/run.json")}" target="_blank" rel="noopener">run.json</a>` : "";
    return `<tr>${names}<td><span class="bench-state ${escapeAttr(c.state)}">${escapeHtml(c.state)}</span></td>
      <td class="num">${c.total ? `${c.completed}/${c.total}` : ""}</td><td>${file}</td></tr>`;
  }).join("");
  return `<table class="run-grid"><thead><tr>${head}<th>state</th><th>processes</th><th></th></tr></thead><tbody>${rows}</tbody></table>`;
}

function eta(p) {
  if (pace.id !== run.run_id) pace = { id: run.run_id, at: Date.now(), done: p.completed || 0 };
  const gained = (p.completed || 0) - pace.done;
  if (!p.total || gained <= 0) return "";
  const left = ((Date.now() - pace.at) / gained) * (p.total - p.completed) / 1000;
  return left < 1 ? "" : `about ${TR ? TR.fmtDuration(left) : Math.round(left) + " s"} left`;
}

async function benchResults(body, fresh) {
  const needMount = fresh || !$("run-explorer") || benchHostRunId !== run.run_id;
  if (needMount) {
    if (benchExplorer) benchExplorer.destroy();
    body.innerHTML = `<div class="bench-progress-wrap" id="run-progress-wrap">
        <progress id="run-progress" max="100" value="0"></progress>
        <div class="bench-live" id="run-now"></div>
      </div>
      <div id="run-cells"></div>
      <div id="run-explorer"></div>`;
    benchHostRunId = run.run_id;
    runJsonKey = "";
    lastRunJson = null;
    benchExplorer = createBenchExplorer($("run-explorer"), {
      onNavigate: href => { location.hash = href; },
      onOpenOutput: term => {
        if (outState.runId !== run.run_id) { outState.runId = run.run_id; outState.sig = ""; outState.path = ""; }
        outState.query = term || "";
        location.hash = here(run.run_id, "output");
      },
      onOpenSignals: term => {
        if (sigState.runId !== run.run_id) { sigState.runId = run.run_id; sigState.sig = ""; sigState.traceDetails.clear(); }
        sigState.query = term || "";
        sigState.tab = "traces";
        location.hash = here(run.run_id, "signals");
      },
    });
  }

  const p = run.progress || {};
  const isLiveRun = live();
  $("run-progress-wrap").hidden = !isLiveRun && !p.total;
  const bar = $("run-progress");
  bar.max = p.total || 100;
  bar.value = p.total ? p.completed : (isLiveRun ? 0 : 100);
  const pctText = p.total ? `${Math.round(((p.completed || 0) / p.total) * 100)}% (${p.completed}/${p.total})` : "";
  $("run-now").textContent = isLiveRun
    ? [pctText, p.variant, eta(p)].filter(Boolean).join(" · ")
    : pctText;
  $("run-cells").innerHTML = cellGrid();

  const nextKey = `${run.run_id}|${state()}|${run.updated_at_ms || 0}|${run.has_run_json}`;
  if (run.has_run_json && runJsonKey !== nextKey) {
    const data = await getJson(run.files + "run.json").catch(() => null);
    if (data) {
      lastRunJson = data;
      runJsonKey = nextKey;
    }
  }
  benchExplorer.update({ runMeta: run, runData: lastRunJson, live: isLiveRun });
  if (outState.pendingFocusCase && lastRunJson) {
    const target = outState.pendingFocusCase;
    outState.pendingFocusCase = "";
    benchExplorer.focusCase(target);
  }
}

function testResults(body, fresh) {
  if (fresh || !view || viewKey !== run.run_id) {
    if (benchExplorer) { benchExplorer.destroy(); benchExplorer = null; benchHostRunId = ""; }
    body.innerHTML = "";
    const host = document.createElement("div");
    body.appendChild(host);
    view = TR.createView(host, { rerun: started => { pendingRerun = started.run_id; location.hash = here(started.run_id); } });
    viewKey = run.run_id;
  }
  view.update(detail);
}

// ---- Output ----

function classifyLogLine(line) {
  const low = line.toLowerCase();
  if (low.includes("error:") || low.includes("failed") || low.includes("panicked") || low.includes("warn")) return "warn";
  if (line.includes(" done ") || line.includes("median ") || line.includes("win ")) return "done";
  if (line.includes(" running ") || line.includes("compiling")) return "running";
  return "info";
}

function formatLogLineHtml(line, idx) {
  const kind = classifyLogLine(line);
  const highlighted = escapeHtml(line).replace(/\b(compare\/[A-Za-z0-9_/-]+)\b/g,
    `<button type="button" class="out-case-link" data-case="$1" title="Inspect $1 in Results">$1</button>`);
  return `<div class="out-line out-${kind}">
    <span class="out-ln">${idx + 1}</span>
    <span class="out-txt">${highlighted}</span>
  </div>`;
}

function paintBenchOutput(body) {
  const pre = $("run-output");
  if (!pre) return;
  const sel = typeof window.getSelection === "function" ? window.getSelection() : null;
  if (sel && !sel.isCollapsed && pre.contains(sel.anchorNode)) return;

  const lines = (outState.rawText || "").split("\n");
  const q = outState.query.trim().toLowerCase();
  const matched = [];
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i];
    if (i === lines.length - 1 && l === "") continue;
    if (outState.level !== "all" && classifyLogLine(l) !== outState.level) continue;
    if (q && !l.toLowerCase().includes(q)) continue;
    matched.push([i, l]);
  }
  $("out-count").textContent = `${matched.length} / ${Math.max(0, lines.length - 1)} lines`;
  pre.classList.toggle("nowrap", !outState.wrap);

  const stick = outState.follow || (pre.scrollTop + pre.clientHeight >= pre.scrollHeight - 12);
  pre.innerHTML = matched.length
    ? matched.slice(-2500).map(([i, l]) => formatLogLineHtml(l, i)).join("")
    : `<p class="logs-empty">No lines match filter.</p>`;

  pre.querySelectorAll(".out-case-link[data-case]").forEach(btn => {
    btn.addEventListener("click", () => {
      const c = btn.dataset.case || "";
      const parts = c.split("/");
      outState.pendingFocusCase = parts.length >= 2 ? parts[parts.length - 2] : c;
      location.hash = here(run.run_id, "results");
    });
  });

  if (stick) pre.scrollTop = pre.scrollHeight;
}

async function output(body, fresh) {
  if (isTest()) {
    if (fresh || !$("run-output")) {
      body.innerHTML = `<pre class="run-output" id="run-output"></pre>`;
    }
    const pre = $("run-output");
    const stick = pre.scrollTop + pre.clientHeight >= pre.scrollHeight - 8;
    pre.textContent = await fetchText(run.files + "output.log").catch(() => "No output recorded.");
    if (stick) pre.scrollTop = pre.scrollHeight;
    return;
  }

  const logs = run.logs || [];
  if (!logs.length) {
    body.innerHTML = note("No captured output yet.");
    return;
  }
  if (outState.runId !== run.run_id) {
    outState.runId = run.run_id;
    outState.sig = "";
    outState.path = logs[0].path;
  }
  if (!logs.some(l => l.path === outState.path)) outState.path = logs[0].path;

  const needMount = fresh || !$("out-shell");
  if (needMount) {
    body.innerHTML = `<div id="out-shell" class="out-shell">
      <div class="out-bar">
        <div class="bx-chips" id="out-files"></div>
        <div class="out-actions">
          <a id="out-raw" class="mini-btn" target="_blank" rel="noopener">Raw ↗</a>
          <button type="button" class="mini-btn" id="out-copy">Copy visible</button>
          <button type="button" class="mini-btn" id="out-follow">Follow tail</button>
          <button type="button" class="mini-btn" id="out-wrap">Wrap</button>
        </div>
      </div>
      <div class="bx-ctrl-row">
        <input type="search" id="out-search" class="bx-search" placeholder="Filter log lines, case names, medians…" autocomplete="off" />
        <div class="bx-chips" id="out-levels">
          <button type="button" data-lv="all">All</button>
          <button type="button" data-lv="done">Done (medians)</button>
          <button type="button" data-lv="running">Running</button>
          <button type="button" data-lv="warn">Warn / Error</button>
        </div>
        <span class="traces-hint" id="out-count"></span>
      </div>
      <div class="run-output out-pre" id="run-output"></div>
    </div>`;

    $("out-search").value = outState.query;
    $("out-search").addEventListener("input", e => { outState.query = e.target.value; paintBenchOutput(body); });
    $("out-levels").querySelectorAll("button[data-lv]").forEach(btn => {
      btn.addEventListener("click", () => {
        outState.level = btn.dataset.lv;
        $("out-levels").querySelectorAll("button").forEach(b => b.classList.toggle("on", b === btn));
        paintBenchOutput(body);
      });
    });
    $("out-follow").addEventListener("click", () => {
      outState.follow = !outState.follow;
      $("out-follow").classList.toggle("on", outState.follow);
      if (outState.follow) {
        const pre = $("run-output");
        if (pre) pre.scrollTop = pre.scrollHeight;
      }
    });
    $("out-wrap").addEventListener("click", () => {
      outState.wrap = !outState.wrap;
      $("out-wrap").classList.toggle("on", outState.wrap);
      paintBenchOutput(body);
    });
    $("out-copy").addEventListener("click", () => {
      const text = Array.from(document.querySelectorAll("#run-output .out-txt")).map(el => el.textContent).join("\n");
      copy(text || outState.rawText);
    });
  }

  $("out-files").innerHTML = logs.map(l =>
    `<button type="button" class="${l.path === outState.path ? "on" : ""}" data-lpath="${escapeAttr(l.path)}">${escapeHtml(l.path)} <small>${fmtBytes(l.bytes)}</small></button>`
  ).join("");
  $("out-files").querySelectorAll("button[data-lpath]").forEach(btn => {
    btn.addEventListener("click", () => {
      outState.path = btn.dataset.lpath;
      outState.sig = "";
      output(body, false).catch(() => {});
    });
  });

  $("out-follow").classList.toggle("on", outState.follow);
  $("out-wrap").classList.toggle("on", outState.wrap);
  $("out-levels").querySelectorAll("button[data-lv]").forEach(b => b.classList.toggle("on", b.dataset.lv === outState.level));
  $("out-raw").href = run.files + outState.path;

  const meta = logs.find(l => l.path === outState.path) || logs[0];
  const nextSig = `${run.run_id}|${outState.path}|${meta.bytes}`;
  if (outState.sig !== nextSig) {
    outState.rawText = await fetchText(run.files + outState.path).catch(() => "No output recorded.");
    outState.sig = nextSig;
    paintBenchOutput(body);
  }
}

// ---- Signals ----

function kvHtml(obj) {
  const entries = Object.entries(obj || {}).sort(([a], [b]) => a.localeCompare(b));
  if (!entries.length) return `<p class="now-quiet">No attributes.</p>`;
  return `<table class="kv-table"><tbody>${entries.map(([k, v]) =>
    `<tr><th>${escapeHtml(k)}</th><td><code>${escapeHtml(v)}</code></td></tr>`
  ).join("")}</tbody></table>`;
}

function paintSignals() {
  const stage = $("sig-stage");
  if (!stage) return;

  $("sig-scope").textContent = sigState.scopeText;
  const counts = {
    traces: sigState.traces.length,
    metrics: ((sigState.metricsData && sigState.metricsData.series) || []).length,
    logs: sigState.logs.length,
    issues: sigState.issues.length,
  };

  const tabs = [
    ["traces", `Traces (${counts.traces})`],
    ["metrics", `Metrics (${counts.metrics})`],
    ["logs", `Logs (${counts.logs})`],
    ["issues", `Issues (${counts.issues})`],
    ["overview", "Overview"],
  ];
  $("sig-tabs").innerHTML = tabs.map(([id, lbl]) =>
    `<button type="button" class="${sigState.tab === id ? "on" : ""}" data-stab="${id}">${escapeHtml(lbl)}</button>`
  ).join("");
  $("sig-tabs").querySelectorAll("button").forEach(b => {
    b.onclick = () => { sigState.tab = b.dataset.stab; paintSignals(); };
  });

  const q = sigState.query.trim().toLowerCase();

  if (sigState.tab === "traces") {
    let list = sigState.traces.filter(t => !q || `${t.root_name} ${t.root_service} ${t.trace_id} ${t.status}`.toLowerCase().includes(q));
    if (sigState.sort === "dur_desc") list = list.slice().sort((a, b) => (b.duration_ms || 0) - (a.duration_ms || 0));
    else if (sigState.sort === "name") list = list.slice().sort((a, b) => (a.root_name || "").localeCompare(b.root_name || ""));
    const maxDur = Math.max(0.01, ...list.map(t => Number(t.duration_ms || 0)));

    stage.innerHTML = `<div class="traces-toolbar">
        <select id="sig-sort">
          <option value="seq"${sigState.sort === "seq" ? " selected" : ""}>Sequence order</option>
          <option value="dur_desc"${sigState.sort === "dur_desc" ? " selected" : ""}>Longest span first</option>
          <option value="name"${sigState.sort === "name" ? " selected" : ""}>Operation A→Z</option>
        </select>
        <span class="traces-hint">${list.length} trace(s) · click any span to inspect inline</span>
      </div>
      <div class="sig-list">${list.length ? list.map(t => {
        const dur = Number(t.duration_ms || 0);
        const w = Math.max(2, Math.min(100, (dur / maxDur) * 100));
        const open = sigState.expandedTraceId === t.trace_id;
        const det = sigState.traceDetails.get(t.trace_id);
        let drawer = "";
        if (open) {
          const spans = (det && det.trace && det.trace.spans) || [];
          drawer = `<div class="sig-drawer">
            ${det === undefined ? `<p class="now-quiet">Loading trace spans…</p>` : spans.length ? spans.map(sp => `
              <div class="sig-span-box">
                <div class="bx-card-head">
                  <strong>${escapeHtml(sp.name)}</strong>
                  <span><a href="#/traces/${escapeAttr(t.trace_id)}" class="trace-pill">Full Waterfall ↗</a></span>
                </div>
                <div class="traces-hint">${escapeHtml(sp.service)} · ${escapeHtml(sp.kind)} · ${Number(sp.duration_ms || 0).toFixed(3)} ms · status ${escapeHtml(sp.status)}</div>
                ${kvHtml(sp.attrs)}
              </div>`).join("") : `<p class="now-quiet">No spans returned for trace <code>${escapeHtml(t.trace_id)}</code>.</p>`}
          </div>`;
        }
        return `<div class="sig-trace-item${open ? " open" : ""}">
          <button type="button" class="sig-trace-row" data-tid="${escapeAttr(t.trace_id)}">
            <span class="trace-status ${t.status === "ERROR" ? "error" : "ok"}">${escapeHtml(t.status || "OK")}</span>
            <span class="sig-op"><code>${escapeHtml(t.root_name)}</code> <small>${escapeHtml(t.root_service)} · ${t.span_count} span(s)</small></span>
            <span class="sig-bar-wrap"><span class="sig-bar" style="width:${w.toFixed(1)}%"></span></span>
            <span class="num sig-dur">${dur.toFixed(2)} ms</span>
          </button>
          ${drawer}
        </div>`;
      }).join("") : `<p class="now-quiet">No traces match filter.</p>`}</div>`;

    const sortEl = $("sig-sort");
    if (sortEl) sortEl.onchange = () => { sigState.sort = sortEl.value; paintSignals(); };
    stage.querySelectorAll(".sig-trace-row[data-tid]").forEach(btn => {
      btn.onclick = async () => {
        const tid = btn.dataset.tid;
        if (sigState.expandedTraceId === tid) {
          sigState.expandedTraceId = "";
          paintSignals();
          return;
        }
        sigState.expandedTraceId = tid;
        paintSignals();
        if (!sigState.traceDetails.has(tid)) {
          const det = await tryJson("traces/" + enc(tid), null, null);
          sigState.traceDetails.set(tid, det || null);
          if (sigState.expandedTraceId === tid) paintSignals();
        }
      };
    });
    return;
  }

  if (sigState.tab === "metrics") {
    const m = sigState.metricsData || {};
    const points = (m.points || []).filter(p => !q || `${p.name} ${p.attrs} ${p.service}`.toLowerCase().includes(q));
    const series = (m.series || []).filter(s => !q || `${s.name} ${s.attrs} ${s.service}`.toLowerCase().includes(q));
    const groups = new Map();
    for (const p of points) {
      const arr = groups.get(p.name) || [];
      arr.push(p);
      groups.set(p.name, arr);
    }

    const boards = [...groups.entries()].map(([name, pts]) => {
      const sorted = pts.slice().sort((a, b) => Number(b.value) - Number(a.value));
      const minV = Math.max(0.1, Math.min(...sorted.map(x => Number(x.value) || 0)));
      const maxV = Math.max(1, ...sorted.map(x => Number(x.value) || 0));
      const rows = sorted.slice(0, 80).map(p => {
        const val = Number(p.value || 0);
        const w = sigState.scale === "log"
          ? Math.max(4, ((Math.log10(Math.max(0.1, val)) - Math.log10(minV)) / Math.max(0.4, Math.log10(maxV) - Math.log10(minV))) * 100)
          : Math.max(4, (val / maxV) * 100);
        const caseLbl = (p.attr_map && (p.attr_map["bench.case"] || p.attr_map["bench.variant"])) || p.attrs || p.name;
        const unit = p.unit === "ns" ? fmtNs(val) : fmtValue(val, p.unit || "");
        return `<div class="bx-erow">
          <span class="bx-ename" title="${escapeAttr(caseLbl)}">${escapeHtml(caseLbl)}</span>
          <div class="bx-track"><div class="bx-fill" style="width:${w.toFixed(1)}%;background:var(--glow)"></div></div>
          <span class="bx-eval">${escapeHtml(unit)}</span>
        </div>`;
      }).join("");
      return `<div class="bench-chart-card">
        <div class="bx-card-head"><h4>${escapeHtml(name)}</h4><span class="traces-hint">${pts.length} series</span></div>
        <div class="bx-bars">${rows}</div>
      </div>`;
    }).join("");

    const multiSeries = series.filter(s => (s.points || []).length >= 2);
    const sparks = multiSeries.slice(0, 12).map((s, i) => `<article class="metric-board">
      <header><div><h3>${escapeHtml(s.name)}</h3><div class="sub">${escapeHtml(s.attrs || s.service || "")}</div></div></header>
      ${sparkSvg(s.points, "smf" + i)}
    </article>`).join("");

    stage.innerHTML = `<div class="bx-ctrl-row">
        <div class="bx-seg">
          <button type="button" class="${sigState.scale === "log" ? "on" : ""}" id="sig-sc-log">Log10</button>
          <button type="button" class="${sigState.scale === "lin" ? "on" : ""}" id="sig-sc-lin">Linear</button>
        </div>
        <span class="traces-hint">${points.length} metric point(s) across ${groups.size} instrument(s)</span>
      </div>
      ${sparks ? `<div class="metrics-boards">${sparks}</div>` : ""}
      <div class="bx-dist-grid">${boards || `<p class="now-quiet">No OTLP metrics match filter.</p>`}</div>`;
    $("sig-sc-log").onclick = () => { sigState.scale = "log"; paintSignals(); };
    $("sig-sc-lin").onclick = () => { sigState.scale = "lin"; paintSignals(); };
    return;
  }

  if (sigState.tab === "logs") {
    const entries = sigState.logs.filter(e => {
      if (!sigState.sevs.has(normalizeSev(e.severity))) return false;
      if (!q) return true;
      return `${e.body || ""} ${e.service || ""} ${e.trace_id || ""} ${JSON.stringify(e.attrs || {})}`.toLowerCase().includes(q);
    });
    const sevHtml = SEV_ORDER.map(s =>
      `<button type="button" class="${sigState.sevs.has(s) ? "on" : ""}" data-ssev="${s}">${s}</button>`
    ).join("");
    const rows = entries.slice(0, 200).map((e, i) => {
      const k = `${e.time || i}|${(e.body || "").slice(0, 32)}`;
      const open = sigState.expandedLogs.has(k);
      const tracePill = e.trace_id
        ? ` <a class="trace-pill" href="#/traces/${escapeAttr(e.trace_id)}">trace:${escapeHtml(e.trace_id.slice(0, 8))}</a>`
        : "";
      return `<div class="log-row expandable${open ? " open" : ""}" data-sk="${escapeAttr(k)}">
        <div class="log-time">${escapeHtml(formatTime(e))}</div>
        <div class="log-sev ${sevClass(e.severity)}">${escapeHtml(e.severity || "INFO")}</div>
        <div class="log-body">${escapeHtml(e.body || "")}${tracePill}${open ? kvHtml({ ...(e.attrs || {}), trace_id: e.trace_id || "", span_id: e.span_id || "" }) : ""}</div>
      </div>`;
    }).join("");
    stage.innerHTML = `<div class="bx-ctrl-row"><div class="sev-chips" id="sig-sevs">${sevHtml}</div><span class="traces-hint">${entries.length} record(s) · click any log to inspect attributes</span></div>
      <div class="logs-panel">${rows || `<p class="logs-empty">No logs match filter.</p>`}</div>`;
    $("sig-sevs").querySelectorAll("button").forEach(b => {
      b.onclick = () => {
        const s = b.dataset.ssev;
        if (sigState.sevs.has(s) && sigState.sevs.size > 1) sigState.sevs.delete(s);
        else sigState.sevs.add(s);
        paintSignals();
      };
    });
    stage.querySelectorAll(".log-row.expandable").forEach(row => {
      row.onclick = ev => {
        if (ev.target && ev.target.closest && ev.target.closest("a")) return;
        const k = row.dataset.sk;
        if (sigState.expandedLogs.has(k)) sigState.expandedLogs.delete(k);
        else sigState.expandedLogs.add(k);
        paintSignals();
      };
    });
    return;
  }

  if (sigState.tab === "issues") {
    const items = sigState.issues.filter(i => !q || `${i.id} ${i.title} ${i.status}`.toLowerCase().includes(q));
    stage.innerHTML = `<div class="bench-tele-panel">${items.length
      ? items.map(i => `<div><a href="#/issues/${escapeAttr(i.id)}">${escapeHtml(i.title)} · ${escapeHtml(i.status)}</a></div>`).join("")
      : "No issues correlated with this run."}</div>`;
    return;
  }

  // Overview
  stage.innerHTML = `<div class="bx-kpis">
    <button type="button" class="bx-kpi" data-go="traces"><small>Traces</small><strong>${counts.traces}</strong><span>Open trace explorer →</span></button>
    <button type="button" class="bx-kpi" data-go="metrics"><small>Metric series</small><strong>${counts.metrics}</strong><span>Open metric charts →</span></button>
    <button type="button" class="bx-kpi" data-go="logs"><small>OTLP Logs</small><strong>${counts.logs}</strong><span>Open log inspector →</span></button>
    <button type="button" class="bx-kpi" data-go="issues"><small>Issues</small><strong>${counts.issues}</strong><span>Open issues →</span></button>
  </div>`;
  stage.querySelectorAll(".bx-kpi[data-go]").forEach(b => {
    b.onclick = () => { sigState.tab = b.dataset.go; paintSignals(); };
  });
}

async function signals(body, fresh, force = false) {
  if (sigState.runId !== run.run_id) {
    sigState.runId = run.run_id;
    sigState.sig = "";
    sigState.expandedTraceId = "";
    sigState.traceDetails.clear();
  }
  const needMount = fresh || !$("sig-shell");
  if (needMount) {
    body.innerHTML = `<div id="sig-shell" class="sig-shell">
      <div class="bx-ctrl-row">
        <div class="bx-chips" id="sig-tabs"></div>
        <input type="search" id="sig-search" class="bx-search" placeholder="Filter spans, metrics, log attrs…" autocomplete="off" />
        <button type="button" class="mini-btn" id="sig-refresh">↻ Refresh</button>
      </div>
      <p class="now-quiet" id="sig-scope"></p>
      <div id="sig-stage"></div>
    </div>`;
    $("sig-search").value = sigState.query;
    $("sig-search").addEventListener("input", e => { sigState.query = e.target.value; paintSignals(); });
    $("sig-refresh").addEventListener("click", () => { signals(body, false, true).catch(() => {}); });
  }

  const nextSig = `${run.run_id}|${state()}|${run.updated_at_ms || 0}`;
  if (!force && !live() && sigState.sig === nextSig) {
    paintSignals();
    return;
  }

  const tagged = run.signals_run_id;
  const from = (run.started_at_ms || 0) - 2000;
  const to = (run.finished_at_ms || Date.now()) + 2000;
  sigState.scopeText = tagged
    ? `Signals tagged with airbug.run_id=${tagged}.`
    : "Signals received while this run executed (time-window correlation).";

  const scope = (items, atFn) => tagged ? (items || []) : (items || []).filter(x => {
    const at = Number(atFn(x));
    return !Number.isFinite(at) || (at >= from && at <= to);
  });

  const qParams = extra => (tagged ? { ...extra, run_id: tagged } : extra);
  const [trResp, lgResp, mtResp, isResp] = await Promise.all([
    tryJson("traces", qParams({ limit: 300 }), {}),
    tryJson("logs", qParams({ limit: 500 }), {}),
    tryJson("metrics", qParams({ limit: 600 }), {}),
    tryJson("issues", tagged ? { run_id: tagged } : {}, {}),
  ]);

  sigState.traces = scope(trResp.traces, t => t.time_ms);
  sigState.logs = scope(lgResp.entries, e => e.time_ms);
  sigState.metricsData = {
    ...mtResp,
    points: scope(mtResp.points, p => p.time_ms),
    series: mtResp.series || [],
  };
  sigState.issues = scope(isResp.issues, i => Date.parse(i.last_seen));
  sigState.sig = nextSig;
  paintSignals();
}

// ---- Compare ----

const picker = (options, chosen) => `<label class="run-pick-label">Baseline <select id="run-baseline" class="run-pick">
  ${options.map(([id, label]) => `<option value="${escapeAttr(id)}"${id === chosen ? " selected" : ""}>${escapeHtml(label)}</option>`).join("")}</select></label>`;

function bindPicker(body) {
  const el = $("run-baseline");
  if (el) el.addEventListener("change", () => { baseline[run.run_id] = el.value; compare(body).catch(() => {}); });
}

const when = ms => (TR && ms ? TR.fmtWhen(ms) : "");

async function compareBench(body) {
  if (!run.compare_id) {
    body.innerHTML = note("This session was written outside .airbug-bench, so there are no other runs to compare it with.");
    return;
  }
  const pref = takePreferredBaseline(run.run_id);
  if (pref) baseline[run.run_id] = pref;

  if (!cmpView || cmpViewRunId !== run.run_id || !body.querySelector("#run-baseline")) {
    body.innerHTML = `<div id="cmp-host"></div>`;
    cmpViewRunId = run.run_id;
    cmpView = createBenchCompareView($("cmp-host"), {
      onBaselineChange: val => { baseline[run.run_id] = val; compareBench(body).catch(() => {}); },
      onThresholdChange: th => { cmpThreshold[run.run_id] = th; compareBench(body).catch(() => {}); },
    });
  }
  const chosen = baseline[run.run_id] || "";
  const threshold = cmpThreshold[run.run_id] != null ? cmpThreshold[run.run_id] : 5;
  const cmp = await getJson("bench/compare", {
    run: run.compare_id,
    baseline: chosen,
    threshold,
    limit: 500,
  });
  cmpView.update(cmp, chosen);
}

function list(title, ids, tone) {
  if (!ids.length) return "";
  const shown = ids.slice(0, 200).map(id => `<li><code>${escapeHtml(id)}</code></li>`).join("");
  return `<h3 class="${tone}">${escapeHtml(title)} <small>${ids.length}</small></h3><ul class="run-diff">${shown}</ul>`;
}

async function compareTests(body) {
  cmpView = null;
  const timeline = await fetchTimeline("test", 40);
  const others = (timeline.runs || []).filter(r => r.run_id !== run.run_id);
  if (!others.length) {
    body.innerHTML = note("No other test run to compare with.");
    return;
  }
  const previous = detail && detail.changes && detail.changes.previous_run_id;
  const chosen = baseline[run.run_id] || (others.some(r => r.run_id === previous) ? previous : others[0].run_id);
  const base = await getJson("runs/" + enc(chosen));
  const statuses = d => new Map(((d.report && d.report.tests) || []).map(t => [t.id, t.status]));
  const [now, before] = [statuses(detail), statuses(base)];
  const failed = s => s === "failed" || s === "timeout";
  const ids = fn => [...now.keys()].filter(fn).sort();
  const broke = ids(id => failed(now.get(id)) && before.has(id) && !failed(before.get(id)));
  const fixed = ids(id => now.get(id) === "passed" && failed(before.get(id)));
  const still = ids(id => failed(now.get(id)) && failed(before.get(id)));
  const added = ids(id => !before.has(id));
  const removed = [...before.keys()].filter(id => !now.has(id)).sort();
  const options = others.map(r => [r.run_id, `${(r.git && r.git.short) || r.run_id.slice(0, 8)} · ${when(r.started_at_ms)} · ${r.state}`]);
  const same = !(broke.length + fixed.length + added.length + removed.length);
  body.innerHTML = picker(options, chosen)
    + `<p class="now-big">${broke.length ? `<b class="bad">${broke.length}</b> newly failing` : "no new failures"} · <b class="good">${fixed.length}</b> fixed</p>`
    + (same ? note(`Same ${now.size} tests, same outcome${still.length ? ` (${still.length} failing in both)` : ""}.`) : "")
    + list("Newly failing", broke, "bad") + list("Fixed", fixed, "good") + list("Failing in both", still, "")
    + list("Added", added, "") + list("Removed", removed, "");
  bindPicker(body);
}

const compare = body => (isTest() ? compareTests(body) : compareBench(body));

// ---- section ----

async function refresh(route, fresh) {
  const body = $("run-body");
  try {
    await load(route.runId);
  } catch (err) {
    run = null;
    view = null;
    if (benchExplorer) { benchExplorer.destroy(); benchExplorer = null; benchHostRunId = ""; }
    const starting = route.runId === pendingRerun;
    $("run-head").innerHTML = `<a class="bench-back" href="#/runs">← all runs</a>`;
    $("run-tabs").innerHTML = "";
    body.innerHTML = note(starting ? "Starting the rerun…" : err.message || "run not found");
    return;
  }
  if (run.run_id === pendingRerun) pendingRerun = null;
  renderHead(route);
  const tab = tabOf(route);
  if (tab === "results") await (isTest() ? testResults(body, fresh) : benchResults(body, fresh));
  else if (tab === "output") await output(body, fresh);
  else if (tab === "signals") await signals(body, fresh);
  else if (fresh || !$("run-baseline")) await compare(body);
  bindCopy(body);
}

export default {
  cats: ["run"],
  hide: () => {
    if (benchExplorer) { benchExplorer.destroy(); benchExplorer = null; benchHostRunId = ""; }
  },
  show: (route, changed) => {
    if (!route.runId) {
      location.hash = "#/runs";
      return null;
    }
    return refresh(route, changed).catch(err => { $("run-body").innerHTML = note(String(err)); });
  },
  tick: n => {
    const route = currentRoute();
    if (!route.runId || (pendingRerun !== route.runId && !live() && n % 4)) return null;
    return refresh(route, false);
  },
};
