// Signals › Logs: OTLP log tail with severity, service and text filters (filtered client-side).

import { getJson } from "../api.js";
import { $, escapeAttr, escapeHtml, renderToggles, toggleKeepOne } from "../dom.js";
import { formatTime, normalizeSev, sevClass, SEV_ORDER } from "../format.js";

const severities = new Set(SEV_ORDER);
let last = null;

function renderSevChips() {
  renderToggles($("sev-chips"), SEV_ORDER.map(s => [s, s]), s => severities.has(s), s => {
    toggleKeepOne(severities, s);
    renderSevChips();
    paint();
  }, "data-sev");
}

function renderSevBar(bySeverity) {
  const el = $("sev-bar");
  const items = bySeverity || [];
  const total = items.reduce((a, x) => a + Number(x.count || 0), 0);
  el.hidden = !total;
  el.innerHTML = total ? items.map(x => {
    const pct = (Number(x.count) / total) * 100;
    return `<span class="${escapeAttr(x.severity)}" style="width:${pct.toFixed(2)}%" title="${escapeAttr(x.severity + ": " + x.count)}"></span>`;
  }).join("") : "";
}

function syncServices(services) {
  const el = $("logs-service");
  const cur = el.value;
  el.innerHTML = [`<option value="">all services</option>`]
    .concat((services || []).map(s => `<option value="${escapeAttr(s)}">${escapeHtml(s)}</option>`)).join("");
  if (cur && (services || []).includes(cur)) el.value = cur;
}

function matches(e, q, svc) {
  if (!severities.has(normalizeSev(e.severity))) return false;
  if (svc && e.service !== svc) return false;
  return !q || `${e.body || ""} ${e.service || ""} ${e.scope || ""}`.toLowerCase().includes(q);
}

function paint() {
  if (!last) return;
  const all = last.entries || [];
  const q = ($("logs-filter").value || "").trim().toLowerCase();
  const entries = all.filter(e => matches(e, q, $("logs-service").value || ""));
  $("logs-meta").textContent = [last.note || "", entries.length !== all.length ? `showing ${entries.length}/${all.length}` : ""]
    .filter(Boolean).join(" · ");
  const panel = $("logs-panel");
  if (!entries.length) {
    panel.innerHTML = `<p class="logs-empty">${escapeHtml(last.note || "No logs match filters.")}</p>`;
    return;
  }
  panel.innerHTML = entries.map(e => {
    const svc = [e.service, e.scope].filter(Boolean).join(" · ");
    return `<div class="log-row">
      <div class="log-time">${escapeHtml(formatTime(e))}</div>
      <div class="log-sev ${sevClass(e.severity)}">${escapeHtml(e.severity || "INFO")}</div>
      <div class="log-body">${escapeHtml(e.body || "")}${svc ? `<div class="log-svc">${escapeHtml(svc)}</div>` : ""}</div>
    </div>`;
  }).join("");
  panel.scrollTop = panel.scrollHeight;
}

export async function refresh() {
  last = await getJson("logs", { limit: 150 });
  renderSevBar(last.by_severity);
  syncServices(last.services);
  paint();
}

export function init() {
  renderSevChips();
  $("logs-filter").addEventListener("input", paint);
  $("logs-service").addEventListener("change", paint);
}

export default {
  cats: ["logs"],
  show: () => refresh().catch(err => { $("logs-meta").textContent = "failed to load /api/v1/logs: " + err; }),
};
