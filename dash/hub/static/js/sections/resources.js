// Resources (.NET Aspire-style local service & container orchestrator).
// Lists built-in services, discovered Docker containers, and managed `.airbug/stack.json`
// resources with Start / Stop / Restart actions, live stdout/stderr console, and
// injected OpenTelemetry + SERVICES__* service discovery environment inspection.

import { enc, getJson, post } from "../api.js";
import { $, bindCopy, escapeAttr, escapeHtml, toast } from "../dom.js";
import { formatTime } from "../format.js";
import { setBadge } from "../shell.js";

let cached = null;
let selectedName = null;
let inspectorTab = "console";

function stateBadge(state) {
  const s = String(state || "Unknown");
  const cls = s === "Running" ? "running"
    : (s === "Failed" || s === "Exited") ? "failed"
    : s === "Starting" ? "starting" : "stopped";
  return `<span class="res-badge state-${escapeAttr(cls)}">${escapeHtml(s)}</span>`;
}

function healthBadge(health) {
  const h = String(health || "Unknown");
  const cls = h === "Healthy" ? "healthy" : h === "Unhealthy" ? "unhealthy" : "unknown";
  return `<span class="res-badge health-${escapeAttr(cls)}">${escapeHtml(h)}</span>`;
}

function renderEndpoints(endpoints) {
  const list = endpoints || [];
  if (!list.length) return `<span class="muted">—</span>`;
  return list.map(u => {
    if (/^https?:\/\//.test(u)) {
      return `<a href="${escapeAttr(u)}" target="_blank" rel="noopener" class="res-ep">${escapeHtml(u)} ↗</a>`;
    }
    return `<code class="res-ep">${escapeHtml(u)}</code>`;
  }).join(" ");
}

function renderRow(r) {
  const active = r.name === selectedName ? " active" : "";
  const procInfo = r.pid
    ? `PID ${escapeHtml(String(r.pid))}`
    : (r.image ? escapeHtml(r.image) : (r.command ? `<code>${escapeHtml(r.command)}</code>` : "—"));
  const deps = (r.depends_on && r.depends_on.length)
    ? `<div class="res-sub">depends on: ${escapeHtml(r.depends_on.join(", "))}</div>`
    : "";
  const exitNote = r.exit_code != null
    ? `<div class="res-sub bad">exit ${escapeHtml(String(r.exit_code))}</div>`
    : "";

  const actions = [];
  if (r.managed) {
    if (r.state === "Running") {
      actions.push(`<button type="button" class="mini-btn" data-res-act="restart" data-res="${escapeAttr(r.name)}">Restart</button>`);
      actions.push(`<button type="button" class="mini-btn" data-res-act="stop" data-res="${escapeAttr(r.name)}">Stop</button>`);
    } else {
      actions.push(`<button type="button" class="mini-btn primary" data-res-act="start" data-res="${escapeAttr(r.name)}">Start</button>`);
    }
  }
  actions.push(`<button type="button" class="mini-btn" data-res-inspect="${escapeAttr(r.name)}">Inspect</button>`);

  const tele = `<div class="res-tele">
    <a href="#/traces" title="View traces">Traces</a> ·
    <a href="#/logs" title="View logs">Logs</a> ·
    <a href="#/metrics" title="View metrics">Metrics</a> ·
    <a href="#/issues" title="View issues">Issues</a>
  </div>`;

  return `<tr class="res-row${active}" data-res-row="${escapeAttr(r.name)}">
    <td>
      <div class="res-name"><strong>${escapeHtml(r.name)}</strong>
        <span class="res-source">${escapeHtml(r.source || (r.managed ? "stack.json" : "built-in"))}</span>
      </div>
      ${deps}${exitNote}
    </td>
    <td><code>${escapeHtml(r.kind || "process")}</code></td>
    <td>${stateBadge(r.state)} ${healthBadge(r.health)}</td>
    <td>${renderEndpoints(r.endpoints)}</td>
    <td class="res-proc">${procInfo}</td>
    <td>${tele}</td>
    <td class="res-actions">${actions.join(" ")}</td>
  </tr>`;
}

export async function refresh() {
  const data = await getJson("resources");
  cached = data;
  const list = data.resources || [];
  const unhealthy = list.filter(r => r.state === "Failed" || r.health === "Unhealthy").length;
  const running = list.filter(r => r.state === "Running").length;
  setBadge("resources", unhealthy ? `${unhealthy}!` : (running ? String(running) : ""));

  const metaEl = $("resources-meta");
  if (metaEl) {
    const stackInfo = data.stack_exists
      ? `stack: ${data.stack_file}`
      : `no .airbug/stack.json (showing built-in & discovered resources)`;
    metaEl.textContent = `${running}/${list.length} running · ${stackInfo}`;
  }

  const wrap = $("resources-table-wrap");
  if (!wrap) return;
  if (!list.length) {
    wrap.innerHTML = `<p class="issues-empty">No resources discovered.</p>`;
    return;
  }

  wrap.innerHTML = `<table class="res-table">
    <thead>
      <tr>
        <th>Resource</th>
        <th>Kind</th>
        <th>State &amp; Health</th>
        <th>Endpoints</th>
        <th>PID / Image</th>
        <th>Telemetry</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      ${list.map(renderRow).join("")}
    </tbody>
  </table>`;

  wrap.querySelectorAll("button[data-res-act]").forEach(btn => {
    btn.addEventListener("click", ev => {
      ev.stopPropagation();
      runAction(btn.getAttribute("data-res"), btn.getAttribute("data-res-act"));
    });
  });

  wrap.querySelectorAll("button[data-res-inspect]").forEach(btn => {
    btn.addEventListener("click", ev => {
      ev.stopPropagation();
      openInspector(btn.getAttribute("data-res"));
    });
  });

  wrap.querySelectorAll("tr[data-res-row]").forEach(tr => {
    tr.addEventListener("click", ev => {
      if (ev.target.closest("a, button")) return;
      openInspector(tr.getAttribute("data-res-row"));
    });
  });

  if (selectedName) {
    await renderInspector();
  }
}

async function runAction(name, act) {
  if (!name || !act) return;
  try {
    toast(`${act} ${name}…`);
    await post(`resources/${enc(name)}/${act}`);
    selectedName = name;
    await refresh();
  } catch (err) {
    toast(`${act} failed: ${err.message || err}`);
  }
}

async function openInspector(name) {
  selectedName = name;
  await refresh();
}

async function renderInspector() {
  const el = $("resource-inspector");
  if (!el) return;
  const list = (cached && cached.resources) || [];
  const res = list.find(r => r.name === selectedName);
  if (!res) {
    el.hidden = true;
    return;
  }
  el.hidden = false;

  let bodyHtml = "";
  if (inspectorTab === "console") {
    const logsData = await getJson(`resources/${enc(res.name)}/logs`, { tail: 400 }).catch(() => ({ lines: [] }));
    const lines = logsData.lines || [];
    if (!lines.length) {
      bodyHtml = `<div class="res-console-empty">No captured stdout/stderr lines for <code>${escapeHtml(res.name)}</code> yet.</div>`;
    } else {
      bodyHtml = `<div class="res-console">${lines.map(l =>
        `<div class="res-line ${l.stream === "stderr" ? "stderr" : "stdout"}">
          <span class="res-ts">${escapeHtml(formatTime({ time_ms: l.time_ms }))}</span>
          <span class="res-stream">${escapeHtml(l.stream)}</span>
          <span class="res-txt">${escapeHtml(l.line)}</span>
        </div>`
      ).join("")}</div>`;
    }
  } else {
    const envEntries = Object.entries(res.env || {});
    bodyHtml = `<table class="res-env-table">
      <thead><tr><th>Variable</th><th>Value (secrets masked)</th><th></th></tr></thead>
      <tbody>
        ${envEntries.map(([k, v]) => `<tr>
          <td><code>${escapeHtml(k)}</code></td>
          <td><code>${escapeHtml(String(v))}</code></td>
          <td><button type="button" class="mini-btn" data-cmd="${escapeAttr(`${k}=${v}`)}">copy</button></td>
        </tr>`).join("")}
      </tbody>
    </table>`;
  }

  el.innerHTML = `<div class="res-insp-head">
    <div>
      <strong>${escapeHtml(res.name)}</strong>
      ${stateBadge(res.state)} ${healthBadge(res.health)}
      <span class="muted">(${escapeHtml(String(res.log_lines || 0))} console lines)</span>
    </div>
    <div class="viz-picker">
      <button type="button" class="${inspectorTab === "console" ? "on" : ""}" data-insp-tab="console">Console (stdout/stderr)</button>
      <button type="button" class="${inspectorTab === "env" ? "on" : ""}" data-insp-tab="env">Environment &amp; Service Discovery (${Object.keys(res.env || {}).length})</button>
      <button type="button" data-insp-close="1">Close ×</button>
    </div>
  </div>
  <div class="res-insp-body">${bodyHtml}</div>`;

  bindCopy(el);
  el.querySelectorAll("button[data-insp-tab]").forEach(btn => {
    btn.addEventListener("click", () => {
      inspectorTab = btn.getAttribute("data-insp-tab") || "console";
      renderInspector();
    });
  });
  const closeBtn = el.querySelector("button[data-insp-close]");
  if (closeBtn) {
    closeBtn.addEventListener("click", () => {
      selectedName = null;
      el.hidden = true;
      refresh();
    });
  }
}

export default {
  cats: ["resources"],
  show: () => refresh().catch(err => {
    const m = $("resources-meta");
    if (m) m.textContent = String(err);
  }),
  tick: () => {
    if (selectedName && inspectorTab === "console") {
      renderInspector().catch(() => {});
    }
  },
};
