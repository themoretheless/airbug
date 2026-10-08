// The run model shared by Now, Runs and Tests: the timeline feed and one row renderer.

import { getJson } from "./api.js";
import { commandItem, escapeAttr, escapeHtml } from "./dom.js";
import { setBadge } from "./shell.js";

export const TR = window.AirbugTestRun;

export const START_TESTS = "cargo airbug test -- --workspace --exclude airbug-mon";
export const START_BENCH = "AIRBUG_HUB=http://127.0.0.1:8790 cargo airbug-bench run -p airbug-bench --bench workloads";

export async function fetchTimeline(kind, limit = 60) {
  const data = await getJson("runs", { limit, kind });
  if (!kind) setBadge("runs", data.live ? `${data.live} live` : "");
  return data;
}

export const isLive = r => !r.stale && (r.state === "running" || r.state === "building");
export const stateLabel = r => (r.stale ? "stale" : r.state);
export const isExternal = href => Boolean(href) && !href.startsWith("#");
export const targetAttrs = href => (isExternal(href) ? ` target="_blank" rel="noopener"` : "");

function runSummary(r) {
  if (r.kind === "test") {
    const t = r.totals || {};
    if (isLive(r)) {
      return `${t.completed || 0}/${t.total || "?"} · ${t.failed || 0} failed` + (r.activity ? ` · ${r.activity}` : "");
    }
    const parts = [`${t.passed || 0} passed`];
    if (t.failed) parts.push(`${t.failed} failed`);
    if (t.not_run) parts.push(`${t.not_run} not run`);
    if (t.ignored) parts.push(`${t.ignored} ignored`);
    return parts.join(" · ");
  }
  const p = r.progress || {};
  if (p.total) return `${p.completed || 0}/${p.total} ${p.unit || "processes"}${p.variant ? " · " + p.variant : ""}`;
  return isExternal(r.href) ? "bench artifacts" : "bench session";
}

export function runRow(r) {
  const live = isLive(r);
  const bar = r.kind === "test" && live && TR ? TR.progressBar(r.totals || {}, true) : "";
  const commit = r.git && r.git.short ? `<code>${escapeHtml(r.git.short)}</code>${r.git.dirty ? "<em>+dirty</em>" : ""}` : "";
  const when = TR ? TR.fmtAgo(r.started_at_ms || r.updated_at_ms) : "";
  const dur = r.duration_s != null && TR ? TR.fmtDuration(r.duration_s) : "";
  return `<a class="run-row${live ? " live" : ""}" href="${escapeAttr(r.href || "#")}"${targetAttrs(r.href)}>
    <span class="run-kind ${escapeAttr(r.kind)}">${escapeHtml(r.kind)}</span>
    <span class="atr-state ${escapeAttr(stateLabel(r))}">${escapeHtml(stateLabel(r))}</span>
    <span class="run-main">
      <strong>${escapeHtml(r.title || r.run_id)}</strong>
      <span class="run-sub">${commit} ${escapeHtml(runSummary(r))}</span>
      ${bar}
    </span>
    <span class="run-when">${escapeHtml(when)}${dur ? `<br/>${escapeHtml(dur)}` : ""}</span></a>`;
}

/** Empty state that says how to start something. */
export function emptyStart(title, commands = [START_TESTS, START_BENCH]) {
  return `<div class="now-empty">
    <p>${escapeHtml(title)} Start one — it appears here while it runs:</p>
    <ul class="actions">${commands.map(c => commandItem(c)).join("")}</ul>
  </div>`;
}
