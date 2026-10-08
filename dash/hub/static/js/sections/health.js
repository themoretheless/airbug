// Health: the status snapshot. Feeds the header meta, the health line on every page, the
// Traces link, and the package cards + API catalog on its own tab.

import { getJson } from "../api.js";
import { $, bindCopy, escapeAttr, escapeHtml } from "../dom.js";
import { setBadge, setTraces } from "../shell.js";

const DOMAIN_ORDER = ["unit", "bench", "mon", "otel", "err", "collector"];

/** Signals whose absence leaves a panel silently empty: [API name, label, why]. */
const CHECKS = [
  ["OTLP HTTP", "OTLP :4318", "logs and metrics panels stay empty — start with --collector"],
  ["Hub errors ingest", "errors ingest", "airbug-err events are not received"],
  ["Jaeger UI", "Jaeger", "no trace UI (otelcol binary mode or no collector)"],
];

const listItems = (items, fn) => (items || []).map(fn).join("");

function card(d) {
  const arts = listItems(d.artifacts, a =>
    `<li><strong>${escapeHtml(a.label)}</strong> · ${escapeHtml(a.detail)}<br/><span>${escapeHtml(a.path)}</span></li>`);
  const acts = listItems(d.actions, a =>
    `<li><button type="button" data-cmd="${escapeAttr(a.command)}">${escapeHtml(a.label)}</button></li>`);
  const report = d.id === "unit" && (d.artifacts || []).some(a => a.label === "index.html")
    ? `<li><a href="/report/index.html" target="_blank" rel="noopener">Open unit HTML report</a></li>` : "";
  return `<article class="domain" data-id="${d.id}">
    <header>
      <h2>${escapeHtml(d.title)}</h2>
      <p class="blurb">${escapeHtml(d.blurb)}</p>
    </header>
    <div class="body">
      <span class="status ${d.status}">${escapeHtml(d.status)}</span>
      <p class="summary">${escapeHtml(d.summary)}</p>
      ${arts ? `<ul class="artifacts">${arts}</ul>` : ""}
      <ul class="actions">${acts}${report}</ul>
    </div>
  </article>`;
}

function renderApis(apis) {
  const up = (apis || []).filter(a => a.available);
  $("apis-meta").textContent = up.length + " available";
  $("api-list").innerHTML = up.length ? up.map(a => `<li>
      <div class="api-method">${escapeHtml(a.method || "GET")}</div>
      <div>
        <a href="${escapeAttr(a.href)}" target="_blank" rel="noopener">${escapeHtml(a.name)}</a>
        <div><code>${escapeHtml(a.href)}</code></div>
        <p class="api-detail">${escapeHtml(a.detail || "")}</p>
      </div>
    </li>`).join("")
    : `<li><p class="api-detail">No local APIs detected yet.</p></li>`;
}

/** One line that says out loud why a panel would otherwise stay empty. */
function renderHealthLine(apis) {
  const api = name => (apis || []).find(a => a.name === name);
  const results = CHECKS.map(([name, label, why]) => ({ label, why, ok: Boolean(api(name) && api(name).available) }));
  const bad = results.filter(r => !r.ok);
  const el = $("health");
  el.classList.toggle("warn", bad.length > 0);
  el.innerHTML = results.map(r =>
    `<span class="${r.ok ? "ok" : "bad"}" title="${escapeAttr(r.ok ? "ok" : r.why)}">${escapeHtml(r.label)} ${r.ok ? "✓" : "✗"}</span>`
  ).join("") + (bad.length ? `<span class="why">${escapeHtml(bad[0].why)}</span><a href="#/health">details</a>` : "");
  setBadge("health", bad.length ? "!" : "");
  const jaeger = api("Jaeger UI");
  setTraces(Boolean(jaeger && jaeger.available), jaeger ? jaeger.href : "");
}

function renderMeta(data, onReload) {
  const id = data.hub_id || "";
  $("meta").innerHTML = `<span title="hub_id ${escapeAttr(id)}">hub <code>${escapeHtml(id.slice(0, 8))}</code></span>`
    + `<span title="Project root this hub serves">root <code>${escapeHtml(data.root)}</code></span>`
    + `<span><button type="button" id="reload" title="Reload everything now">refresh</button></span>`;
  $("reload").onclick = onReload;
}

export async function refresh(onReload) {
  const data = await getJson("status");
  renderMeta(data, onReload);
  renderApis(data.apis);
  renderHealthLine(data.apis);
  const domains = data.domains || {};
  const el = $("domains");
  el.innerHTML = DOMAIN_ORDER.filter(k => domains[k]).map(k => card(domains[k])).join("");
  bindCopy(el);
}

export default {
  cats: ["system"],
  show: () => {},
};
