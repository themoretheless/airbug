// Runs › Tests: the newest test run, live; recent runs open on the run page.

import { getJson } from "../api.js";
import { $, bindCopy, escapeAttr, escapeHtml } from "../dom.js";
import { setTitle } from "../shell.js";
import { emptyStart, fetchTimeline, isLive, START_TESTS, TR } from "../runs.js";

let view = null;
let viewRun = null;
let live = false;
// A rerun started from this page: its directory appears a moment after the POST returns.
let pendingRerun = null;

const message = text => `<div class="now-empty"><p>${escapeHtml(text)}</p></div>`;

function onRerunStarted(started) {
  pendingRerun = started.run_id;
  refresh(true).catch(() => {});
}

function dotState(r) {
  if (isLive(r)) return "running";
  if (r.state === "passed") return "passed";
  return r.state === "failed" || r.state === "error" ? "failed" : "not_run";
}

function renderPicker(runs) {
  const el = $("tests-picker");
  if (!runs.length) {
    el.innerHTML = "";
    return;
  }
  const chips = runs.map(r => {
    const t = r.totals || {};
    const label = (r.git && r.git.short) || r.run_id.slice(0, 8);
    const count = isLive(r) ? `${t.completed || 0}/${t.total || "?"}` : (t.failed ? `${t.failed}✗` : `${t.passed || 0}✓`);
    return `<a class="pick" href="#/run/${escapeAttr(r.run_id)}" title="${escapeAttr(r.title + " · " + (TR ? TR.fmtWhen(r.started_at_ms) : ""))}">
      <span class="atr-dot ${dotState(r)}"></span>${escapeHtml(label)} <small>${escapeHtml(count)}</small></a>`;
  }).join("");
  el.innerHTML = `<a class="pick on" href="#/tests" title="always show the newest run">latest</a>${chips}`;
}

async function refresh(reset) {
  const el = $("tests-view");
  const id = pendingRerun || "latest";
  const [detail, list] = await Promise.all([
    getJson("runs/" + encodeURIComponent(id)).catch(err => ({ error: err })),
    fetchTimeline("test", 15).catch(() => ({ runs: [] })),
  ]);
  renderPicker(list.runs || []);
  if (detail.error) {
    view = null;
    viewRun = null;
    live = id === pendingRerun;
    el.innerHTML = live ? message("Starting the rerun…")
      : id === "latest" ? emptyStart("No test runs yet.", [START_TESTS])
      : message(detail.error.message || "run not found");
    bindCopy(el);
    return;
  }
  if (reset || !view || viewRun !== detail.run_id) {
    el.innerHTML = "";
    const host = document.createElement("div");
    el.appendChild(host);
    view = TR.createView(host, { rerun: onRerunStarted });
    viewRun = detail.run_id;
  }
  if (detail.run_id === pendingRerun) pendingRerun = null;
  view.update(detail);
  const item = detail.item || {};
  live = isLive(item);
  setTitle("Tests", item.state, String(detail.run_id).slice(0, 8));
}

export default {
  cats: ["tests"],
  show: (_route, changed) => refresh(changed).catch(err => { $("tests-view").innerHTML = message(String(err)); }),
  // Live runs refresh every tick, finished ones every fourth.
  tick: n => (live || n % 4 === 0 ? refresh(false) : null),
};
