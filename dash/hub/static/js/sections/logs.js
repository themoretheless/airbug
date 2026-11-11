// Signals › Logs: OTLP log tail with severity, service, text/trace filters,
// clickable trace_id correlation pills, and expandable structured attributes.

import { getJson } from "../api.js";
import { $, escapeAttr, escapeHtml, renderToggles, toggleKeepOne } from "../dom.js";
import { formatTime, normalizeSev, sevClass, SEV_ORDER } from "../format.js";

const severities = new Set(SEV_ORDER);
const expanded = new Set();
let last = null;
let initializedScroll = false;

function rowKey(e, idx) {
  return `${e.time || idx}|${e.service || ""}|${e.trace_id || ""}|${(e.body || "").slice(0, 40)}`;
}

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
  if (!q) return true;
  const hay = `${e.body || ""} ${e.service || ""} ${e.scope || ""} ${e.trace_id || ""} ${e.span_id || ""} ${e.event_name || ""}`.toLowerCase();
  return hay.includes(q);
}

function renderAttrsDrawer(e) {
  const pairs = Object.entries(e.attrs || {}).sort(([a], [b]) => a.localeCompare(b));
  if (e.trace_id) pairs.unshift(["trace_id", e.trace_id]);
  if (e.span_id) pairs.splice(e.trace_id ? 1 : 0, 0, ["span_id", e.span_id]);
  if (e.event_name) pairs.unshift(["event.name", e.event_name]);
  if (!pairs.length) return "";
  return `<div class="log-attrs">${pairs.map(([k, v]) =>
    `<span class="log-attr-pair"><em>${escapeHtml(k)}</em>=<code>${escapeHtml(v)}</code></span>`
  ).join(" ")}</div>`;
}

function paint() {
  if (!last) return;
  const all = last.entries || [];
  const q = ($("logs-filter").value || "").trim().toLowerCase();
  const entries = all.filter(e => matches(e, q, $("logs-service").value || ""));
  $("logs-meta").textContent = [last.note || "", entries.length !== all.length ? `showing ${entries.length}/${all.length}` : ""]
    .filter(Boolean).join(" · ");
  const panel = $("logs-panel");
  const sel = typeof window.getSelection === "function" ? window.getSelection() : null;
  if (sel && !sel.isCollapsed && panel.contains(sel.anchorNode)) {
    return;
  }
  const nearBottom = !initializedScroll || (panel.scrollHeight - panel.scrollTop - panel.clientHeight <= 32);
  if (!entries.length) {
    panel.innerHTML = `<p class="logs-empty">${escapeHtml(last.note || "No logs match filters.")}</p>`;
    return;
  }
  panel.innerHTML = entries.map((e, idx) => {
    const key = rowKey(e, idx);
    const isOpen = expanded.has(key);
    const svc = [e.service, e.scope].filter(Boolean).join(" · ");
    const traceBadge = e.trace_id
      ? ` <a class="trace-pill" href="#/traces/${escapeAttr(e.trace_id)}" title="Open trace ${escapeAttr(e.trace_id)}">trace:${escapeHtml(e.trace_id.slice(0, 8))}</a>`
      : "";
    const evBadge = e.event_name ? ` <span class="span-kind-chip">${escapeHtml(e.event_name)}</span>` : "";
    const hasAttrs = Boolean(e.trace_id || e.span_id || e.event_name || Object.keys(e.attrs || {}).length);
    return `<div class="log-row${hasAttrs ? " expandable" : ""}${isOpen ? " open" : ""}" data-lkey="${escapeAttr(key)}">
      <div class="log-time">${escapeHtml(formatTime(e))}</div>
      <div class="log-sev ${sevClass(e.severity)}">${escapeHtml(e.severity || "INFO")}</div>
      <div class="log-body">${escapeHtml(e.body || "")}${evBadge}${traceBadge}${svc ? `<div class="log-svc">${escapeHtml(svc)}</div>` : ""}${isOpen ? renderAttrsDrawer(e) : ""}</div>
    </div>`;
  }).join("");

  panel.querySelectorAll(".log-row.expandable").forEach(row => {
    row.addEventListener("click", ev => {
      if (ev.target && ev.target.closest && ev.target.closest("a")) return;
      const k = row.getAttribute("data-lkey");
      if (expanded.has(k)) expanded.delete(k);
      else expanded.add(k);
      paint();
    });
  });

  if (nearBottom) {
    panel.scrollTop = panel.scrollHeight;
    initializedScroll = true;
  }
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
