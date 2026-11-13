// One run, any kind: Results / Output / Signals / Compare.
//
// `GET /api/v1/run?id=` says what the run is and where its files are served; each tab reads
// only what it shows. Old links (`#/tests/‹id›`, `#/bench/‹id›`) land here too.

import { enc, getJson, tryJson } from "../api.js";
import { $, bindCopy, escapeAttr, escapeHtml } from "../dom.js";
import { currentRoute, setTitle } from "../shell.js";
import { fetchTimeline, TR } from "../runs.js";
import { benchCompareHtml, renderCharts } from "./bench.js";

const TABS = [["results", "Results"], ["output", "Output"], ["signals", "Signals"], ["compare", "Compare"]];
const MAX_TEXT = 400000;
const DONE = ["passed", "failed", "error", "interrupted", "complete", "cancelled", "incomplete"];

let run = null;        // description from /api/v1/run, or a stand-in for runs it does not know
let detail = null;     // test runs: /api/v1/runs/<id>
let view = null;       // test results view, kept across ticks
let viewKey = "";
let chartsKey = "";
let logPath = "";
let baseline = {};     // run id → chosen baseline id
let pendingRerun = null;
let pace = { id: "", at: 0, done: 0 };  // first sighting of a live bench run, for the ETA

const note = text => `<p class="now-quiet">${escapeHtml(text)}</p>`;
const isTest = () => run && run.kind === "test";
const state = () => (isTest() ? (detail && detail.item && detail.item.state) : run && run.state) || "";
const live = () => Boolean(run) && !DONE.includes(state()) && !(detail && detail.item && detail.item.stale);
const tabOf = route => (TABS.some(([id]) => id === route.tab) ? route.tab : "results");
const here = (id, tab) => `#/run/${enc(id)}${tab && tab !== "results" ? "/" + tab : ""}`;

async function fetchText(url) {
  const res = await fetch(url);
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
  // Test runs keep their results in the test API; live case history is known only there.
  detail = await getJson("runs/" + enc(id));
  const item = detail.item || {};
  run = described || { run_id: detail.run_id, kind: "test", title: item.title, started_at_ms: item.started_at_ms, files: detail.files };
}

function renderHead(route) {
  const tab = tabOf(route);
  const item = (detail && detail.item) || {};
  const started = run.started_at_ms || item.started_at_ms;
  const p = run.progress || {};
  const facts = [run.kind, started && TR ? TR.fmtWhen(started) : "", p.total ? `${p.completed}/${p.total} processes` : ""];
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

/** Time left, from how fast processes finished since this page first saw the run. */
function eta(p) {
  if (pace.id !== run.run_id) pace = { id: run.run_id, at: Date.now(), done: p.completed || 0 };
  const gained = (p.completed || 0) - pace.done;
  if (!p.total || gained <= 0) return "";
  const left = ((Date.now() - pace.at) / gained) * (p.total - p.completed) / 1000;
  return left < 1 ? "" : `about ${TR ? TR.fmtDuration(left) : Math.round(left) + " s"} left`;
}

async function benchResults(body, fresh) {
  if (fresh) {
    body.innerHTML = `<div class="bench-progress-wrap"><progress id="run-progress" max="100" value="0"></progress>
        <div class="bench-live" id="run-now"></div></div>
      <div class="bench-links" id="run-links"></div><div id="run-cells"></div>
      <div id="run-charts" class="bench-charts"></div>
      <iframe id="run-report" class="bench-report-frame" title="Bench report" hidden></iframe>`;
    chartsKey = "";
  }
  const p = run.progress || {};
  const bar = $("run-progress");
  bar.max = p.total || 100;
  bar.value = p.total ? p.completed : (live() ? 0 : 100);
  $("run-now").textContent = live() ? [p.variant, eta(p)].filter(Boolean).join(" · ") : "";
  const files = [[run.has_report_html, "report.html"], [run.has_memory_html, "memory.html"], [run.has_run_json, "run.json"]]
    .filter(([has]) => has)
    .map(([, name]) => `<a href="${escapeAttr(run.files + name)}" target="_blank" rel="noopener">${name}</a>`);
  $("run-links").innerHTML = files.join(" · ");
  $("run-cells").innerHTML = cellGrid();
  const frame = $("run-report");
  frame.hidden = !run.has_report_html;
  if (run.has_report_html && frame.getAttribute("data-src") !== run.files) {
    frame.src = run.files + "report.html";
    frame.setAttribute("data-src", run.files);
  }
  const charts = $("run-charts");
  if (!run.has_run_json) {
    charts.innerHTML = (run.cells || []).length || run.has_report_html ? "" : note("Results appear when the first process finishes.");
  } else if (!run.has_report_html && chartsKey !== run.run_id + state()) {
    // The report page already draws these; without it, draw from run.json.
    await renderCharts(run.files + "run.json", charts);
    if (!live()) chartsKey = run.run_id + state();
  }
}

function testResults(body, fresh) {
  if (fresh || !view || viewKey !== run.run_id) {
    body.innerHTML = "";
    const host = document.createElement("div");
    body.appendChild(host);
    view = TR.createView(host, { rerun: started => { pendingRerun = started.run_id; location.hash = here(started.run_id); } });
    viewKey = run.run_id;
  }
  view.update(detail);
}

// ---- Output ----

async function output(body, fresh) {
  let url = isTest() ? run.files + "output.log" : "";
  if (!isTest()) {
    const logs = run.logs || [];
    if (!logs.length) {
      body.innerHTML = note("No captured output yet.");
      return;
    }
    if (!logs.some(l => l.path === logPath)) logPath = logs[0].path;
    if (fresh || !$("run-log-pick") || $("run-log-pick").options.length !== logs.length) {
      body.innerHTML = `<select id="run-log-pick" class="run-pick">${logs.map(l =>
        `<option value="${escapeAttr(l.path)}"${l.path === logPath ? " selected" : ""}>${escapeHtml(l.path)} · ${l.bytes} B</option>`).join("")}</select>
        <pre class="run-output" id="run-output"></pre>`;
      $("run-log-pick").addEventListener("change", e => { logPath = e.target.value; output(body, false).catch(() => {}); });
    }
    url = run.files + logPath;
  } else if (fresh || !$("run-output")) {
    body.innerHTML = `<pre class="run-output" id="run-output"></pre>`;
  }
  const pre = $("run-output");
  const stick = pre.scrollTop + pre.clientHeight >= pre.scrollHeight - 8;
  pre.textContent = await fetchText(url).catch(() => "No output recorded.");
  if (stick) pre.scrollTop = pre.scrollHeight;
}

// ---- Signals ----

const SIGNALS = [
  { title: "Issues", path: "issues", params: {}, items: d => d.issues, at: i => Date.parse(i.last_seen),
    html: i => `<a href="#/issues/${escapeAttr(i.id)}">${escapeHtml(i.title)} · ${escapeHtml(i.status)}</a>` },
  { title: "Traces", path: "traces", params: { limit: 200 }, items: d => d.traces, at: t => t.time_ms,
    html: t => `<a href="#/traces/${escapeAttr(t.trace_id)}">[${escapeHtml(t.status || "OK")}] ${escapeHtml(t.root_service)} · ${escapeHtml(t.root_name)} (${t.span_count} spans · ${Number(t.duration_ms || 0).toFixed(2)} ms)</a>` },
  { title: "Logs", path: "logs", params: { limit: 500 }, items: d => d.entries, at: e => e.time_ms,
    html: e => escapeHtml(`[${e.severity}] ${e.service} ${e.body}`) },
  { title: "Metrics", path: "metrics", params: { limit: 200 }, items: d => (d.points || []).slice(-60), at: p => p.time_ms,
    html: p => escapeHtml(`${p.name}=${p.value} ${p.unit || ""}`.trim()) },
];

async function signals(body) {
  const tagged = run.signals_run_id;
  const from = (run.started_at_ms || 0) - 2000;
  const to = (run.finished_at_ms || Date.now()) + 2000;
  const scope = tagged ? "tagged with this run's id" : "received while the run was going";
  const blocks = await Promise.all(SIGNALS.map(async s => {
    const data = await tryJson(s.path, tagged ? { ...s.params, run_id: tagged } : s.params, {});
    let items = s.items(data) || [];
    // Without a run id on the signal, the run's time window is the scope.
    if (!tagged) items = items.filter(x => { const at = Number(s.at(x)); return !Number.isFinite(at) || (at >= from && at <= to); });
    const lines = items.slice(0, 80).map(x => `<div>${s.html(x)}</div>`).join("");
    return `<h3>${s.title} <small>${items.length}</small></h3><div class="bench-tele-panel">${lines || escapeHtml(data.note || "Nothing.")}</div>`;
  }));
  body.innerHTML = `${note("Signals " + scope + ".")}<div class="bench-tele">${blocks.join("")}</div>`;
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
  const chosen = baseline[run.run_id] || "";
  const cmp = await getJson("bench/compare", { run: run.compare_id, baseline: chosen });
  const options = [["", "previous comparable run"], ...(cmp.candidates || []).map(c => [c.id, `${c.id} · ${when(c.when_ms)} · ${c.cases} cases`])];
  body.innerHTML = picker(options, chosen) + (benchCompareHtml(cmp) || note("Nothing to compare yet."));
  bindPicker(body);
}

function list(title, ids, tone) {
  if (!ids.length) return "";
  const shown = ids.slice(0, 200).map(id => `<li><code>${escapeHtml(id)}</code></li>`).join("");
  return `<h3 class="${tone}">${escapeHtml(title)} <small>${ids.length}</small></h3><ul class="run-diff">${shown}</ul>`;
}

async function compareTests(body) {
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
  else if (tab === "signals") await signals(body);
  else if (fresh || !$("run-baseline")) await compare(body);
  bindCopy(body);
}

export default {
  cats: ["run"],
  show: (route, changed) => {
    if (!route.runId) {
      location.hash = "#/runs";
      return null;
    }
    return refresh(route, changed).catch(err => { $("run-body").innerHTML = note(String(err)); });
  },
  // Live runs refresh every tick, finished ones every fourth.
  tick: n => {
    const route = currentRoute();
    if (!route.runId || (pendingRerun !== route.runId && !live() && n % 4)) return null;
    return refresh(route, false);
  },
};
