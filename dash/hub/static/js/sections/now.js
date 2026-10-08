// Now: what is running, what just broke, and what changed since the last run.

import { tryJson } from "../api.js";
import { $, bindCopy, commandItem, escapeAttr, escapeHtml } from "../dom.js";
import { setBadge } from "../shell.js";
import { emptyStart, fetchTimeline, isLive, runRow, START_BENCH, START_TESTS, stateLabel, TR } from "../runs.js";
import { benchCompareHtml } from "./bench.js";

const card = (id, title, body) => {
  const el = $(id);
  el.innerHTML = `<h2>${escapeHtml(title)}</h2>` + body;
  bindCopy(el);
};

const quiet = text => `<p class="now-quiet">${escapeHtml(text)}</p>`;
const startHint = cmd => `<ul class="actions">${commandItem(cmd)}</ul>`;
const shortName = id => id.split("::").slice(1).join("::") || id;

function liveCard(runs) {
  const live = runs.filter(r => isLive(r) || (r.kind === "bench" && r.state === "running"));
  if (live.length) return `<div class="runs-list">${live.map(runRow).join("")}</div>`;
  return runs.length ? quiet("Nothing running. Start a run and it shows up here within a second.")
    : emptyStart("Nothing has run in this project yet.");
}

function testsCard(latest) {
  if (!latest || !latest.item) return quiet("No test runs yet.") + startHint(START_TESTS);
  const item = latest.item;
  const t = item.totals || {};
  const c = latest.changes;
  const since = c && `<code>${escapeHtml((c.previous_git && c.previous_git.short) || "previous run")}</code>`;
  const lines = [];
  if (c && c.new_failures_count) {
    lines.push(`<p class="bad">${c.new_failures_count} new failure(s) since ${since}</p>`);
    lines.push(`<ul class="now-list">${(c.new_failures || []).slice(0, 5).map(id => `<li><code>${escapeHtml(shortName(id))}</code></li>`).join("")}</ul>`);
  } else if (c && c.fixed_count) {
    lines.push(`<p class="good">${c.fixed_count} fixed since ${since}</p>`);
  }
  if ((latest.flaky || []).length) lines.push(`<p class="warn">${latest.flaky.length} flaky over the last runs</p>`);
  return `<a class="now-headline" href="#/tests/${escapeAttr(item.run_id)}">
      <span class="atr-state ${escapeAttr(stateLabel(item))}">${escapeHtml(stateLabel(item))}</span>
      <span class="now-big">${t.failed ? `<b class="bad">${t.failed}</b> failed · ` : ""}${t.passed || 0} passed</span>
    </a>
    ${TR ? TR.progressBar(t, isLive(item)) : ""}
    ${quiet([item.title, item.git && item.git.short, TR && TR.fmtAgo(item.started_at_ms)].filter(Boolean).join(" · "))}
    ${lines.join("")}`;
}

function benchCard(runs, cmp) {
  const bench = runs.filter(r => r.kind === "bench");
  const compared = cmp && cmp.mode;
  if (bench.length) return benchCompareHtml(cmp) + `<div class="runs-list compact">${bench.slice(0, compared ? 2 : 4).map(runRow).join("")}</div>`;
  return benchCompareHtml(cmp) + (cmp && cmp.current ? "" : quiet("No bench runs yet.") + startHint(START_BENCH));
}

function issuesCard(issues) {
  const open = (issues.issues || []).filter(i => (i.status || "unresolved") === "unresolved");
  setBadge("issues", open.length ? String(open.length) : "");
  if (!open.length) return quiet("No unresolved issues.");
  return `<a class="now-headline" href="#/issues"><span class="now-big"><b class="bad">${open.length}</b> unresolved</span></a>
    <ul class="now-list">${open.slice(0, 4).map(i => `<li>${escapeHtml(i.title || i.id)} <small>×${escapeHtml(String(i.count || 0))}</small></li>`).join("")}</ul>`;
}

async function refresh() {
  const [timeline, latest, issues, cmp] = await Promise.all([
    fetchTimeline("", 40),
    tryJson("runs/latest", { lite: 1 }),
    tryJson("issues", null, { issues: [] }),
    tryJson("bench/compare"),
  ]);
  const runs = timeline.runs || [];
  card("now-live", "Live", liveCard(runs));
  card("now-tests", "Tests", testsCard(latest));
  card("now-bench", "Bench", benchCard(runs, cmp));
  card("now-issues", "Issues", issuesCard(issues));
}

export default {
  cats: ["now"],
  show: () => refresh().catch(() => {}),
  tick: n => (n % 2 === 0 ? refresh() : null),
};
