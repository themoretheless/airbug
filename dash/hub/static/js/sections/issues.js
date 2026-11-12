// Signals › Issues: Sentry-grade grouped error inbox with Release Health,
// local Sentry DSN, auto-regression badges, Suspect Commit (git blame),
// ±5 line Source Context stacktraces, Breadcrumbs timeline, Tag distributions,
// Occurrence switcher, and one-click correlation to OpenTelemetry traces.

import { enc, getJson, post, tryJson } from "../api.js";
import { $, bindCopy, escapeAttr, escapeHtml, toast } from "../dom.js";
import { setBadge } from "../shell.js";

const ACTIONS = [
  ["resolve", "Resolve"],
  ["resolve_next_release", "Resolve next release"],
  ["ignore", "Ignore"],
  ["reopen", "Reopen"],
];

let selected = null;
let currentIssue = null;
let eventIndex = 0;
let showAllFrames = false;
let showRawJson = false;

function cleanTraceId(raw) {
  if (!raw || typeof raw !== "string") return "";
  const t = raw.trim();
  if (!t || /^0+$/.test(t)) return "";
  return t.toLowerCase();
}

function traceIdFromEvent(ev) {
  if (!ev || typeof ev !== "object") return "";
  return cleanTraceId(ev.trace_id)
    || cleanTraceId(ev.contexts && ev.contexts.trace && ev.contexts.trace.trace_id)
    || cleanTraceId(ev.tags && (ev.tags.trace_id || ev.tags["otel.trace_id"]))
    || cleanTraceId(ev.extra && ev.extra.trace_id);
}

function extractIssueTraceId(issue, activeEv) {
  if (activeEv) {
    const evTid = traceIdFromEvent(activeEv);
    if (evTid) return evTid;
  }
  if (!issue) return "";
  const direct = cleanTraceId(issue.trace_id) || traceIdFromEvent(issue.last_event);
  if (direct) return direct;
  for (const ev of issue.events || []) {
    const found = traceIdFromEvent(ev);
    if (found) return found;
  }
  return "";
}

function statusLabel(status) {
  if (status === "resolved_in_next_release") return "resolved next release";
  return status || "unresolved";
}

function row(i) {
  const status = i.status || "unresolved";
  const tid = cleanTraceId(i.trace_id);
  const tracePill = tid
    ? ` <a class="trace-pill" href="#/traces/${escapeAttr(tid)}" title="Open correlated trace">trace:${escapeHtml(tid.slice(0, 8))}</a>`
    : "";
  const regBadge = i.is_regression
    ? ` <span class="issue-status regressed" title="Regressed after being resolved">REGRESSED${i.regression_count > 1 ? ` ×${i.regression_count}` : ""}</span>`
    : "";
  return `<div class="issue-row${i.id === selected ? " active" : ""}" data-id="${escapeAttr(i.id)}" role="button" tabindex="0">
    <div class="issue-id">${escapeHtml(i.id)}</div>
    <div>
      <div class="issue-title">${escapeHtml(i.title || "")}
        <span class="issue-status ${escapeAttr(status)}">${escapeHtml(statusLabel(status))}</span>${regBadge}${tracePill}
      </div>
      <div class="issue-meta-line">${escapeHtml([i.level, i.service, i.environment, i.release, i.culprit].filter(Boolean).join(" · "))}</div>
      <div class="issue-meta-line">first ${escapeHtml(i.first_seen || "")} · last ${escapeHtml(i.last_seen || "")}</div>
    </div>
    <div class="issue-count">×${escapeHtml(String(i.count || 0))}</div>
  </div>`;
}

function renderReleasesBar(relData) {
  const el = $("issues-releases");
  if (!el) return;
  const dsn = (relData && relData.dsn) || `http://airbug@${location.hostname || "127.0.0.1"}:${location.port || 8790}/1`;
  const releases = (relData && relData.releases) || [];
  const cards = releases.slice(0, 6).map(r => {
    const rate = Number(r.crash_free_rate ?? 100).toFixed(1);
    const tone = rate >= 99 ? "good" : rate >= 95 ? "warn" : "bad";
    return `<div class="rel-card">
      <div class="rel-head">
        <strong>${escapeHtml(r.release)}</strong>
        <span class="rel-rate ${tone}">${escapeHtml(rate)}% crash-free</span>
      </div>
      <div class="rel-meta">
        ${escapeHtml(String(r.total_sessions || 0))} sessions ·
        ${escapeHtml(String(r.crashed_sessions || 0))} crashed ·
        ${escapeHtml(String(r.issue_count || 0))} issues
      </div>
    </div>`;
  }).join("");

  el.innerHTML = `<div class="dsn-bar">
    <span><strong>Sentry DSN:</strong> <code>${escapeHtml(dsn)}</code>
      <button type="button" class="mini-btn" data-cmd="${escapeAttr(`SENTRY_DSN=${dsn}`)}">copy DSN</button>
    </span>
    <span class="muted">Accepts native Sentry SDK envelopes (<code>POST /api/1/envelope/</code>) &amp; <code>airbug-err</code> events</span>
  </div>
  ${cards ? `<div class="rel-grid">${cards}</div>` : ""}`;
  bindCopy(el);
}

export async function refresh() {
  const [data, relData] = await Promise.all([
    getJson("issues"),
    tryJson("releases", null, { releases: [] }),
  ]);
  renderReleasesBar(relData);
  $("issues-meta").textContent = data.note || "";
  const issues = data.issues || [];
  const openCount = issues.filter(i => (i.status || "unresolved") === "unresolved").length;
  setBadge("issues", openCount ? String(openCount) : "", openCount ? "bad" : "");
  const panel = $("issues-panel");
  if (!issues.length) {
    panel.innerHTML = `<p class="issues-empty">${escapeHtml(data.note || "No issues.")}</p>`;
    return;
  }
  panel.innerHTML = issues.map(row).join("");
  panel.querySelectorAll(".issue-row").forEach(el => {
    const open = ev => {
      if (ev && ev.target && ev.target.closest && ev.target.closest("a")) return;
      openIssue(el.getAttribute("data-id"));
    };
    el.addEventListener("click", open);
    el.addEventListener("keydown", ev => {
      if (ev.key === "Enter" || ev.key === " ") {
        ev.preventDefault();
        openIssue(el.getAttribute("data-id"));
      }
    });
  });
}

async function act(id, action) {
  try {
    await post(`issues/${enc(id)}/${action}`);
  } catch {
    toast("action failed");
    return;
  }
  toast(action + " · " + id);
  await openIssue(id);
}

function renderSuspectCommit(sc) {
  if (!sc || !sc.commit) return "";
  return `<div class="issue-suspect">
    <div class="issue-sec-label">Suspect Commit (git blame)</div>
    <div class="issue-suspect-body">
      <code>${escapeHtml(sc.short || sc.commit.slice(0, 8))}</code>
      <strong>${escapeHtml(sc.subject || "")}</strong>
      <span class="muted">by ${escapeHtml(sc.author || "unknown")} · ${escapeHtml(sc.file || "")}${sc.lineno ? `:${escapeHtml(String(sc.lineno))}` : ""}</span>
    </div>
  </div>`;
}

function renderFrameSource(f) {
  if (!f.context_line && (!f.pre_context || !f.pre_context.length)) return "";
  const pre = f.pre_context || [];
  const postLines = f.post_context || [];
  const baseLine = (f.lineno || 1) - pre.length;
  const rows = [];
  pre.forEach((line, idx) => {
    rows.push(`<div class="src-line"><span class="src-ln">${baseLine + idx}</span><span class="src-code">${escapeHtml(line)}</span></div>`);
  });
  if (f.context_line != null) {
    rows.push(`<div class="src-line active"><span class="src-ln">${f.lineno || ""}</span><span class="src-code">${escapeHtml(f.context_line)}</span></div>`);
  }
  postLines.forEach((line, idx) => {
    rows.push(`<div class="src-line"><span class="src-ln">${(f.lineno || 1) + 1 + idx}</span><span class="src-code">${escapeHtml(line)}</span></div>`);
  });
  return `<div class="frame-src">${rows.join("")}</div>`;
}

function renderStacktrace(ev) {
  const exList = (ev && ev.exceptions) || [];
  if (!exList.length) return `<div class="muted">No exception stacktrace attached to this event.</div>`;
  return exList.map(ex => {
    const allFrames = ex.frames || [];
    const hasInApp = allFrames.some(f => f.in_app);
    const frames = (!showAllFrames && hasInApp) ? allFrames.filter(f => f.in_app) : allFrames;
    const frameHtml = frames.map(f => {
      const loc = [f.file, f.lineno, f.colno].filter(x => x != null && x !== "").join(":");
      return `<div class="stack-frame${f.in_app ? " in-app" : ""}">
        <div class="stack-frame-head">
          <span class="fn"><code>${escapeHtml(f.function || "<unknown>")}</code></span>
          ${loc ? `<span class="loc">${escapeHtml(loc)}</span>` : ""}
          <span class="badge ${f.in_app ? "in-app" : "sys"}">${f.in_app ? "in-app" : "system"}</span>
        </div>
        ${renderFrameSource(f)}
      </div>`;
    }).join("");
    return `<div class="issue-sec">
      <div class="issue-sec-head">
        <span><strong>${escapeHtml(ex.ty || "Error")}</strong>: ${escapeHtml(ex.value || "")}</span>
        ${hasInApp ? `<button type="button" class="mini-btn" data-toggle-frames="1">${showAllFrames ? "In-App only" : `All frames (${allFrames.length})`}</button>` : ""}
      </div>
      <div class="stack-list">${frameHtml}</div>
    </div>`;
  }).join("");
}

function renderBreadcrumbs(ev) {
  const crumbs = (ev && ev.breadcrumbs) || [];
  if (!crumbs.length) return "";
  const rows = crumbs.map(b => {
    const dataPairs = Object.entries(b.data || {}).map(([k, v]) => `${k}=${typeof v === "string" ? v : JSON.stringify(v)}`).join(" ");
    return `<div class="crumb-row">
      <span class="crumb-cat">${escapeHtml(b.category || b.type || "default")}</span>
      <span class="crumb-lvl ${escapeAttr(b.level || "info")}">${escapeHtml(b.level || "info")}</span>
      <span class="crumb-msg">${escapeHtml(b.message || "")} ${dataPairs ? `<code>${escapeHtml(dataPairs)}</code>` : ""}</span>
      <span class="crumb-ts">${escapeHtml(b.timestamp || "")}</span>
    </div>`;
  }).join("");
  return `<div class="issue-sec">
    <div class="issue-sec-head"><strong>Breadcrumbs (${crumbs.length})</strong></div>
    <div class="crumb-list">${rows}</div>
  </div>`;
}

function renderTagsAndSparkline(issue) {
  const dists = issue.tag_distributions || [];
  const buckets = issue.activity_buckets || [];
  const maxBucket = Math.max(1, ...buckets.map(b => b.count || 0));
  const spark = buckets.length
    ? `<div class="issue-spark" title="24h event activity">${buckets.map(b => {
        const h = Math.max(2, Math.round(((b.count || 0) / maxBucket) * 28));
        return `<span class="spark-bar${b.count ? " on" : ""}" style="height:${h}px" title="${escapeAttr(`${b.count} events`)}"></span>`;
      }).join("")}</div>`
    : "";

  const tagsHtml = dists.map(d => {
    const top = (d.values || []).slice(0, 3).map(v => `${escapeHtml(v.value)} (${v.percent}%)`).join(", ");
    return `<div class="tag-dist-item">
      <span class="tag-key">${escapeHtml(d.key)}</span>
      <span class="tag-vals">${top}</span>
    </div>`;
  }).join("");

  if (!spark && !tagsHtml) return "";
  return `<div class="issue-sec">
    <div class="issue-sec-head"><strong>Activity &amp; Tag Distributions</strong></div>
    ${spark}
    ${tagsHtml ? `<div class="tag-dist-grid">${tagsHtml}</div>` : ""}
  </div>`;
}

function renderIssueDetail() {
  const issue = currentIssue;
  if (!issue) return;
  $("issue-detail").hidden = false;
  const events = Array.isArray(issue.events) && issue.events.length
    ? issue.events
    : (issue.last_event ? [issue.last_event] : []);
  if (eventIndex >= events.length) eventIndex = Math.max(0, events.length - 1);
  if (eventIndex < 0) eventIndex = 0;
  const activeEv = events[eventIndex] || issue.last_event || {};

  const stored = events.length;
  $("issue-detail-title").textContent = `${issue.id} · ${issue.title || ""} · ${stored} stored / ${issue.count || 0} lifetime`;

  const traceId = extractIssueTraceId(issue, activeEv);
  const actions = $("issue-actions");
  const openTraceBtn = traceId
    ? `<a class="issue-trace-btn" href="#/traces/${escapeAttr(traceId)}">Open Trace →</a>`
    : "";
  actions.innerHTML = openTraceBtn + ACTIONS.map(([a, label]) =>
    `<button type="button" data-action="${escapeAttr(a)}">${escapeHtml(label)}</button>`
  ).join("");
  actions.querySelectorAll("button[data-action]").forEach(btn => {
    btn.addEventListener("click", () => act(issue.id, btn.getAttribute("data-action")));
  });

  const navBar = `<div class="issue-occ-bar">
    <div class="issue-occ-nav">
      <button type="button" class="mini-btn" data-occ-nav="-1"${eventIndex <= 0 ? " disabled" : ""}>← Prev</button>
      <span>Occurrence <strong>${stored ? eventIndex + 1 : 0}</strong> of <strong>${stored}</strong>
        ${activeEv.timestamp ? ` · ${escapeHtml(activeEv.timestamp)}` : ""}
        ${activeEv.release ? ` · release <code>${escapeHtml(activeEv.release)}</code>` : ""}
      </span>
      <button type="button" class="mini-btn" data-occ-nav="1"${eventIndex >= stored - 1 ? " disabled" : ""}>Next →</button>
    </div>
    <button type="button" class="mini-btn" data-toggle-raw="1">${showRawJson ? "Structured View" : "Raw JSON"}</button>
  </div>`;

  const bodyEl = $("issue-detail-body");
  if (showRawJson) {
    const payload = stored ? { events: issue.events, last_event: activeEv } : (issue.last_event || issue);
    bodyEl.innerHTML = `${navBar}<pre class="issue-raw-pre">${escapeHtml(JSON.stringify(payload, null, 2))}</pre>`;
  } else {
    bodyEl.innerHTML = `${navBar}
      ${renderSuspectCommit(issue.suspect_commit)}
      ${renderStacktrace(activeEv)}
      ${renderBreadcrumbs(activeEv)}
      ${renderTagsAndSparkline(issue)}`;
  }

  bodyEl.querySelectorAll("button[data-occ-nav]").forEach(btn => {
    btn.addEventListener("click", () => {
      eventIndex += Number(btn.getAttribute("data-occ-nav") || 0);
      renderIssueDetail();
    });
  });
  const rawBtn = bodyEl.querySelector("button[data-toggle-raw]");
  if (rawBtn) {
    rawBtn.addEventListener("click", () => {
      showRawJson = !showRawJson;
      renderIssueDetail();
    });
  }
  const framesBtn = bodyEl.querySelector("button[data-toggle-frames]");
  if (framesBtn) {
    framesBtn.addEventListener("click", () => {
      showAllFrames = !showAllFrames;
      renderIssueDetail();
    });
  }
}

async function openIssue(id) {
  if (!id) return;
  selected = id;
  const issue = await getJson("issues/" + enc(id)).catch(() => null);
  if (!issue) {
    toast("issue not found");
    return;
  }
  currentIssue = issue;
  eventIndex = 0;
  renderIssueDetail();
  await refresh();
}

export default {
  cats: ["issues"],
  show: () => refresh().catch(err => { $("issues-meta").textContent = String(err); }),
};
