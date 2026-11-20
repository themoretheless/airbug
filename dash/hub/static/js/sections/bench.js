// Runs › Bench: interactive launcher, all-runs hub table, A/B quick compare,
// full interactive Compare view, and the concise summary reused on Now.

import { enc, getJson, tryJson } from "../api.js";
import { renderLaunchPanel } from "../bench_view.js";
import { $, escapeAttr, escapeHtml } from "../dom.js";
import { fmtNs, fmtPct, median } from "../format.js";
import { fetchTimeline, isLive, runRow, targetAttrs, TR } from "../runs.js";

const preferredBaselines = new Map();
export function setPreferredBaseline(runId, baselineId) {
  if (runId && baselineId) preferredBaselines.set(runId, baselineId);
}
export function takePreferredBaseline(runId) {
  const val = preferredBaselines.get(runId) || "";
  if (val) preferredBaselines.delete(runId);
  return val;
}

const stripRunsPrefix = id => (id && id.startsWith("runs/") ? id.slice("runs/".length) : id);

/** Median wall time per case, grouped by case family, from a `run.json`. */
export async function renderCharts(url, el) {
  const run = await getJson(url).catch(() => null);
  if (!run) return;
  const caseById = {};
  for (const c of run.cases || []) {
    if (c && c.id) caseById[c.id] = c;
  }
  const byCase = {};
  for (const o of run.observations || []) {
    if (o.metric !== "wall") continue;
    const av = o.availability;
    const avState = typeof av === "string" ? av : (av && av.state);
    if (avState && avState !== "available") continue;
    if (o.value == null || !Number.isFinite(Number(o.value))) continue;
    const c = caseById[o.case];
    const m = c && (c.metrics || []).find(x => x.id === o.metric);
    const isBatch = (m && m.statistic === "batch_total")
      || (c && c.measurement_mode === "batch_total")
      || (c && c.contract && c.contract.measurement_mode === "batch_total");
    const ops = isBatch ? (Number(o.operations) || 1) : 1;
    (byCase[o.case] = byCase[o.case] || []).push(Number(o.value) / ops);
  }
  const families = {};
  for (const [caseId, values] of Object.entries(byCase)) {
    const parts = caseId.split("/");
    const engine = parts[parts.length - 1] || caseId;
    const family = parts.slice(0, -1).join("/") || caseId;
    (families[family] = families[family] || []).push({ engine, median: median(values) });
  }
  const cards = Object.keys(families).sort().map(family => {
    const rows = families[family].sort((a, b) => a.median - b.median);
    const max = Math.max(...rows.map(r => r.median), 1);
    const linRow = rows.find(r => r.engine === "lin");
    const linLags = Boolean(linRow && rows[0] && linRow.median > rows[0].median);
    const tr = rows.map(r => `<tr class="${linLags && r.engine === "lin" ? "bad" : ""}"><td>${escapeHtml(r.engine)}</td><td>${fmtNs(r.median)}<div class="bench-bar" style="width:${Math.max(2, (r.median / max) * 100)}%"></div></td></tr>`).join("");
    return `<div class="bench-chart-card"><h4 class="${linLags ? "bad" : ""}">${escapeHtml(family)}</h4>
      <table><thead><tr><th>engine</th><th>median wall</th></tr></thead><tbody>${tr}</tbody></table></div>`;
  }).join("");
  el.innerHTML = cards || `<p class="bench-live">No wall observations in run.json.</p>`;
}

// ---- interactive Runs › Bench hub (#/bench) ----

const hubState = {
  mounted: false,
  query: "",
  filter: "all",
  runs: [],
  hubId: "",
  cmpA: "",
  cmpB: "",
};

function filteredHubRuns() {
  const q = hubState.query.trim().toLowerCase();
  return hubState.runs.filter(r => {
    const st = (r.state || "").toLowerCase();
    if (hubState.filter === "live" && !isLive(r) && st !== "building") return false;
    if (hubState.filter === "done" && !["complete", "passed"].includes(st)) return false;
    if (hubState.filter === "fail" && !["failed", "error", "cancelled", "incomplete"].includes(st)) return false;
    if (q) {
      const hay = `${r.title || ""} ${r.run_id || ""} ${r.state || ""}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
}

function paintBenchHub() {
  const listEl = $("bench-list");
  if (!listEl) return;
  const rows = filteredHubRuns();
  const liveCount = hubState.runs.filter(r => isLive(r) || r.state === "building").length;
  $("bench-meta").textContent = `${hubState.runs.length} bench run(s)`
    + (liveCount ? ` · ${liveCount} live` : "")
    + (hubState.hubId ? " · hub " + hubState.hubId.slice(0, 8) : "");

  const doneRuns = hubState.runs.filter(r => ["complete", "passed"].includes((r.state || "").toLowerCase()));
  if (!hubState.cmpA && doneRuns[0]) hubState.cmpA = doneRuns[0].run_id;
  if (!hubState.cmpB && doneRuns[1]) hubState.cmpB = doneRuns[1].run_id;

  const cmpBar = $("bh-cmp-bar");
  if (cmpBar) {
    if (doneRuns.length >= 2) {
      cmpBar.hidden = false;
      const opt = (selId) => doneRuns.map(r =>
        `<option value="${escapeAttr(r.run_id)}"${r.run_id === selId ? " selected" : ""}>${escapeHtml((r.title || r.run_id).slice(0, 36))} · ${escapeHtml(r.run_id.slice(0, 12))}</option>`
      ).join("");
      const selA = $("bh-cmp-a"), selB = $("bh-cmp-b");
      const cmpSig = doneRuns.map(r => r.run_id).join(",");
      if (selA && selB && cmpBar.dataset.sig !== cmpSig) {
        cmpBar.dataset.sig = cmpSig;
        selA.innerHTML = opt(hubState.cmpA);
        selB.innerHTML = opt(hubState.cmpB);
      }
    } else {
      cmpBar.hidden = true;
    }
  }

  document.querySelectorAll("#bh-filters button[data-bf]").forEach(b => {
    b.classList.toggle("on", b.dataset.bf === hubState.filter);
  });

  listEl.innerHTML = rows.length
    ? rows.map(runRow).join("")
    : `<p class="bench-live">${hubState.runs.length ? "No bench runs match filter." : "No bench runs yet — launch one above."}</p>`;
}

function ensureBenchHubMounted() {
  const section = document.querySelector("#panel-bench .bench");
  if (!section) return;
  if (hubState.mounted && $("bh-launch")) return;
  hubState.mounted = true;

  let shell = $("bh-shell");
  if (!shell) {
    shell = document.createElement("div");
    shell.id = "bh-shell";
    shell.className = "bh-shell";
    shell.innerHTML = `
      <div class="bh-card" id="bh-launch"></div>
      <div class="bh-toolbar">
        <input type="search" id="bh-search" class="bx-search" placeholder="Filter bench runs by title, run_id, state…" autocomplete="off" />
        <div class="bx-chips" id="bh-filters">
          <button type="button" data-bf="all" class="on">All</button>
          <button type="button" data-bf="live">Live</button>
          <button type="button" data-bf="done">Complete</button>
          <button type="button" data-bf="fail">Failed</button>
        </div>
        <div class="bh-cmp" id="bh-cmp-bar" hidden>
          <select id="bh-cmp-a" aria-label="Candidate run"></select>
          <span>vs</span>
          <select id="bh-cmp-b" aria-label="Baseline run"></select>
          <button type="button" class="mini-btn primary" id="bh-cmp-go">Compare →</button>
        </div>
      </div>`;
    const listEl = $("bench-list");
    section.insertBefore(shell, listEl);
  }

  renderLaunchPanel($("bh-launch"), started => { location.hash = started.href; });
  $("bh-search").addEventListener("input", e => {
    hubState.query = e.target.value;
    paintBenchHub();
  });
  $("bh-filters").querySelectorAll("button[data-bf]").forEach(btn => {
    btn.addEventListener("click", () => {
      hubState.filter = btn.dataset.bf;
      paintBenchHub();
    });
  });
  $("bh-cmp-a").addEventListener("change", e => { hubState.cmpA = e.target.value; });
  $("bh-cmp-b").addEventListener("change", e => { hubState.cmpB = e.target.value; });
  $("bh-cmp-go").addEventListener("click", () => {
    const a = $("bh-cmp-a").value;
    const b = $("bh-cmp-b").value;
    if (a && b && a !== b) setPreferredBaseline(a, b);
    if (a) location.hash = `#/run/${enc(a)}/compare`;
  });
}

async function renderList() {
  ensureBenchHubMounted();
  const [timeline, meta] = await Promise.all([
    fetchTimeline("bench", 100),
    tryJson("bench/runs", null, {}),
  ]);
  hubState.runs = timeline.runs || [];
  hubState.hubId = meta.hub_id || "";
  paintBenchHub();
}

export default {
  cats: ["bench"],
  show: () => renderList().catch(err => { $("bench-meta").textContent = String(err); }),
  tick: n => (n % 4 === 0 ? renderList() : null),
};

// ---- latest bench run vs the previous comparable one (`GET /api/v1/bench/compare`) ----

const DECISION = {
  regression: ["slower", "bad"],
  improvement: ["faster", "good"],
  within_margin: ["same", ""],
  inconclusive: ["unclear", "warn"],
  unavailable: ["n/a", ""],
  neutral: ["info", ""],
};

const shortId = id => String(id).split("/").pop().slice(0, 12);
const link = (x, label) => `<a href="${escapeAttr(x.href)}"${targetAttrs(x.href)}>${escapeHtml(label)}</a>`;

export function benchCompareHtml(cmp) {
  if (!cmp) return "";
  if (!cmp.mode) return cmp.current && cmp.note ? `<p class="now-quiet">Latest run: ${escapeHtml(cmp.note)}.</p>` : "";
  const s = cmp.summary || {};
  const threshold = cmp.threshold != null ? cmp.threshold : 5;
  const head = [];
  if (s.regression) head.push(`<b class="bad">${s.regression}</b> slower`);
  if (s.improvement) head.push(`<b class="good">${s.improvement}</b> faster`);
  if (s.within_margin) head.push(`${s.within_margin} within ±${threshold}%`);
  if (s.inconclusive) head.push(`<span class="warn">${s.inconclusive}</span> unclear`);
  if (!head.length) head.push("nothing to compare");
  const cur = cmp.current;
  const against = cmp.mode === "ab"
    ? `baseline vs candidate in ${link(cur, shortId(cur.id))}`
    : `${link(cur, shortId(cur.id))} vs previous ${link(cmp.previous, shortId(cmp.previous.id))}`;
  const ago = TR && cur.when_ms ? ` · ${escapeHtml(TR.fmtAgo(cur.when_ms))}` : "";
  const rows = (cmp.rows || [])
    .filter(r => r.decision !== "neutral" && r.decision !== "unavailable")
    .slice(0, 5)
    .map(r => {
      const [label, tone] = DECISION[r.decision] || [r.decision, ""];
      const ci = r.interval_percent ? `${fmtPct(r.interval_percent[0])} … ${fmtPct(r.interval_percent[1])}` : "";
      return `<tr class="${escapeAttr(tone)}" title="${escapeAttr((r.note || "") + (ci ? " · interval " + ci : ""))}">
        <td><code>${escapeHtml(r.case)}</code> <small>${escapeHtml(r.metric)}</small></td>
        <td class="num">${escapeHtml(fmtPct(r.change_percent))}</td>
        <td>${escapeHtml(label)}</td></tr>`;
    }).join("");
  const more = (cmp.total || 0) > 5 ? `<p class="now-quiet">${cmp.total} comparisons in total.</p>` : "";
  return `<a class="now-headline" href="${escapeAttr(cur.href)}"${targetAttrs(cur.href)}>
      <span class="now-big">${head.join(" · ")}</span></a>
    <p class="now-quiet">${against}${ago}</p>
    ${rows ? `<table class="now-cmp">${rows}</table>` : ""}${more}`;
}

// ---- Full Interactive Compare Explorer (`#/run/<id>/compare`) ----

const THRESHOLDS = [1, 3, 5, 10, 15];
const when = ms => (TR && ms ? TR.fmtWhen(ms) : "");

export function createBenchCompareView(host, opts = {}) {
  host.innerHTML = `
    <div class="bc-head" data-part="head"></div>
    <div class="bc-controls">
      <div class="bx-ctrl-row">
        <label class="bx-lbl">Baseline <select id="run-baseline" class="run-pick" data-part="baseline"></select></label>
        <button type="button" class="mini-btn" data-part="swap" title="Swap candidate and baseline">⇄ Swap</button>
        <div class="bx-seg" data-part="thresh"></div>
      </div>
      <div class="bx-ctrl-row">
        <input type="search" class="bx-search" data-part="search" placeholder="Filter case, engine, note…" autocomplete="off" />
        <div class="bx-chips" data-part="filter"></div>
        <label class="bx-lbl">Sort <select data-part="sort">
          <option value="abs">|Δ%| largest</option>
          <option value="faster">Fastest improvements</option>
          <option value="slower">Worst slowdowns</option>
          <option value="current">Current fastest</option>
          <option value="baseline">Baseline fastest</option>
          <option value="name">Case name A→Z</option>
        </select></label>
      </div>
    </div>
    <div data-part="banner"></div>
    <div data-part="kpis" class="bx-kpis"></div>
    <div data-part="table"></div>`;

  const part = name => host.querySelector(`[data-part="${name}"]`);
  const st = {
    cmp: null,
    threshold: 5,
    filter: "all",
    sort: "abs",
    query: "",
    expanded: new Set(),
    onBaselineChange: opts.onBaselineChange || (() => {}),
    onThresholdChange: opts.onThresholdChange || (() => {}),
  };

  part("baseline").addEventListener("change", e => st.onBaselineChange(e.target.value));
  part("search").addEventListener("input", e => { st.query = e.target.value; paintRows(); });
  part("sort").addEventListener("change", e => { st.sort = e.target.value; paintRows(); });
  part("swap").addEventListener("click", () => {
    const prevId = st.cmp && st.cmp.previous && st.cmp.previous.id;
    const curId = st.cmp && st.cmp.current && st.cmp.current.id;
    if (!prevId || !curId) return;
    setPreferredBaseline(stripRunsPrefix(prevId), curId);
    location.hash = `#/run/${enc(stripRunsPrefix(prevId))}/compare`;
  });

  function renderThresh() {
    part("thresh").innerHTML = THRESHOLDS.map(t =>
      `<button type="button" class="${t === st.threshold ? "on" : ""}" data-th="${t}">±${t}%</button>`
    ).join("");
    part("thresh").querySelectorAll("button").forEach(b => {
      b.onclick = () => {
        st.threshold = Number(b.dataset.th);
        st.onThresholdChange(st.threshold);
      };
    });
  }

  function renderFilters(rows) {
    const faster = rows.filter(r => r.change_percent != null && r.change_percent <= -st.threshold).length;
    const slower = rows.filter(r => r.change_percent != null && r.change_percent >= st.threshold).length;
    const flat = rows.filter(r => r.change_percent != null && Math.abs(r.change_percent) < st.threshold).length;
    const regs = rows.filter(r => r.decision === "regression").length;
    const imps = rows.filter(r => r.decision === "improvement").length;
    const items = [
      ["all", `All (${rows.length})`],
      ["emp_fast", `Empirical faster (${faster})`],
      ["emp_slow", `Empirical slower (${slower})`],
      ["emp_flat", `Within ±${st.threshold}% (${flat})`],
      ...(imps ? [["improvement", `Confirmed faster (${imps})`]] : []),
      ...(regs ? [["regression", `Confirmed slower (${regs})`]] : []),
    ];
    part("filter").innerHTML = items.map(([id, lbl]) =>
      `<button type="button" class="${st.filter === id ? "on" : ""}" data-cf="${id}">${escapeHtml(lbl)}</button>`
    ).join("");
    part("filter").querySelectorAll("button").forEach(b => {
      b.onclick = () => { st.filter = b.dataset.cf; renderFilters(rows); paintRows(); };
    });
  }

  function filteredRows(rows) {
    const q = st.query.trim().toLowerCase();
    let list = rows.filter(r => {
      const cp = r.change_percent;
      if (st.filter === "emp_fast" && !(cp != null && cp <= -st.threshold)) return false;
      if (st.filter === "emp_slow" && !(cp != null && cp >= st.threshold)) return false;
      if (st.filter === "emp_flat" && !(cp != null && Math.abs(cp) < st.threshold)) return false;
      if (st.filter === "regression" && r.decision !== "regression") return false;
      if (st.filter === "improvement" && r.decision !== "improvement") return false;
      if (q) {
        const hay = `${r.case} ${r.metric} ${r.decision} ${r.note || ""}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
    list = list.slice().sort((a, b) => {
      if (st.sort === "abs") return Math.abs(b.change_percent ?? 0) - Math.abs(a.change_percent ?? 0);
      if (st.sort === "faster") return (a.change_percent ?? 9999) - (b.change_percent ?? 9999);
      if (st.sort === "slower") return (b.change_percent ?? -9999) - (a.change_percent ?? -9999);
      if (st.sort === "current") return (a.candidate ?? Infinity) - (b.candidate ?? Infinity);
      if (st.sort === "baseline") return (a.baseline ?? Infinity) - (b.baseline ?? Infinity);
      return a.case.localeCompare(b.case);
    });
    return list;
  }

  function paintRows() {
    const cmp = st.cmp;
    if (!cmp || !cmp.mode) {
      part("table").innerHTML = `<p class="now-quiet">${escapeHtml((cmp && cmp.note) || "Nothing to compare yet.")}</p>`;
      return;
    }
    const allRows = cmp.rows || [];
    const list = filteredRows(allRows);
    const maxAbs = Math.max(15, ...list.map(r => Math.min(200, Math.abs(r.change_percent ?? 0))));

    const htmlRows = list.map(r => {
      const cp = r.change_percent;
      const empTone = cp == null ? "" : cp <= -st.threshold ? "good" : (cp >= st.threshold || r.decision === "regression") ? "bad" : "";
      const speedup = r.baseline > 0 && r.candidate > 0 ? r.baseline / r.candidate : null;
      const spStr = speedup != null ? (speedup >= 1 ? `${speedup.toFixed(2)}× faster` : `${(1 / speedup).toFixed(2)}× slower`) : "—";
      const units = r.hypothesis && r.hypothesis.test ? Math.min(r.hypothesis.test.baseline_units || 0, r.hypothesis.test.candidate_units || 0) : r.independent_units;
      const statBadge = r.decision === "inconclusive" && units <= 1
        ? `<span class="bx-badge single" title="Only 1 process on at least one side — run with --repetitions 2+ for Welch bootstrap significance">1-proc</span>`
        : `<span class="bx-badge ${escapeAttr(DECISION[r.decision]?.[1] || "")}">${escapeHtml(DECISION[r.decision]?.[0] || r.decision)}</span>`;

      const pctWidth = cp != null ? Math.min(100, (Math.min(Math.abs(cp), 200) / maxAbs) * 100) : 0;
      const barHtml = cp != null
        ? `<div class="bc-divbar">
            <div class="bc-half bc-left">${cp < 0 ? `<div class="bc-fill good" style="width:${pctWidth.toFixed(1)}%"></div>` : ""}</div>
            <div class="bc-mid"></div>
            <div class="bc-half bc-right">${cp > 0 ? `<div class="bc-fill bad" style="width:${pctWidth.toFixed(1)}%"></div>` : ""}</div>
          </div>`
        : "";

      const open = st.expanded.has(r.case + "|" + r.metric);
      const ciText = r.interval_percent ? `${fmtPct(r.interval_percent[0])} … ${fmtPct(r.interval_percent[1])}` : "unavailable (needs >=2 independent processes/side)";
      const pVal = r.hypothesis && r.hypothesis.test && r.hypothesis.test.p_value != null
        ? Number(r.hypothesis.test.p_value).toExponential(2)
        : "n/a";

      return `<tr class="bc-row${open ? " open" : ""}" data-ck="${escapeAttr(r.case + "|" + r.metric)}">
          <td><code>${escapeHtml(r.case)}</code></td>
          <td class="num">${r.baseline != null ? fmtNs(r.baseline) : "—"}</td>
          <td class="num">${r.candidate != null ? fmtNs(r.candidate) : "—"}</td>
          <td class="num ${empTone}"><strong>${escapeHtml(fmtPct(cp))}</strong></td>
          <td class="num ${empTone}">${escapeHtml(spStr)}</td>
          <td class="bc-bar-col">${barHtml}</td>
          <td>${statBadge}</td>
        </tr>
        ${open ? `<tr class="bc-detail"><td colspan="7">
          <div class="bc-detail-box">
            <span><strong>Method:</strong> ${escapeHtml((r.hypothesis && r.hypothesis.method) || "median ratio")}</span>
            <span><strong>Independent units:</strong> ${escapeHtml(String(units ?? "—"))}</span>
            <span><strong>p-value:</strong> ${escapeHtml(pVal)}</span>
            <span><strong>95% CI:</strong> ${escapeHtml(ciText)}</span>
            ${r.note ? `<div class="traces-hint">${escapeHtml(r.note)}</div>` : ""}
          </div>
        </td></tr>` : ""}`;
    }).join("");

    part("table").innerHTML = `<div class="bx-table-tools"><span>Showing ${list.length} of ${cmp.total || allRows.length} comparisons · click any row for statistical details</span></div>
      <div class="table-scroll"><table class="traces-table bc-table">
        <thead><tr><th>Case</th><th class="num">Baseline</th><th class="num">Current</th><th class="num">Δ%</th><th class="num">Ratio</th><th>Delta vs ±${st.threshold}%</th><th>Significance</th></tr></thead>
        <tbody>${htmlRows || `<tr><td colspan="7">No comparisons match filter.</td></tr>`}</tbody>
      </table></div>`;

    part("table").querySelectorAll("tr.bc-row").forEach(tr => {
      tr.onclick = () => {
        const k = tr.dataset.ck;
        if (st.expanded.has(k)) st.expanded.delete(k);
        else st.expanded.add(k);
        paintRows();
      };
    });
  }

  return {
    update(cmp, chosenBaseline) {
      st.cmp = cmp;
      if (cmp && cmp.threshold != null) st.threshold = Number(cmp.threshold);
      renderThresh();

      const candidates = (cmp && cmp.candidates) || [];
      const optHtml = [["", "previous comparable run"], ...candidates.map(c =>
        [c.id, `${c.id} · ${when(c.when_ms)} · ${c.cases} cases`]
      )].map(([id, lbl]) =>
        `<option value="${escapeAttr(id)}"${id === (chosenBaseline || "") ? " selected" : ""}>${escapeHtml(lbl)}</option>`
      ).join("");
      part("baseline").innerHTML = optHtml;
      part("swap").disabled = !(cmp && cmp.previous && cmp.previous.id);

      const banner = part("banner");
      if (cmp && cmp.overlap) {
        banner.innerHTML = `<div class="bc-banner">Partial suite comparison: <strong>${cmp.overlap.shared}</strong> shared case(s) compared (${cmp.overlap.only_current} only in current, ${cmp.overlap.only_baseline} only in baseline).</div>`;
      } else {
        banner.innerHTML = "";
      }

      if (cmp && cmp.mode) {
        const rows = cmp.rows || [];
        const empFast = rows.filter(r => r.change_percent != null && r.change_percent <= -st.threshold).length;
        const empSlow = rows.filter(r => r.change_percent != null && r.change_percent >= st.threshold).length;
        const empFlat = rows.filter(r => r.change_percent != null && Math.abs(r.change_percent) < st.threshold).length;
        const medDelta = median(rows.map(r => r.change_percent).filter(Number.isFinite));
        part("kpis").hidden = false;
        part("kpis").innerHTML = `
          <button type="button" class="bx-kpi" data-ck="all"><small>Total compared</small><strong>${cmp.total || rows.length} cases</strong><span>median Δ ${escapeHtml(fmtPct(medDelta))}</span></button>
          <button type="button" class="bx-kpi" data-ck="emp_fast"><small>Empirical faster (≤−${st.threshold}%)</small><strong class="good">${empFast} faster</strong><span>Confirmed bootstrap: ${(cmp.summary && cmp.summary.improvement) || 0}</span></button>
          <button type="button" class="bx-kpi" data-ck="emp_slow"><small>Empirical slower (≥+${st.threshold}%)</small><strong class="bad">${empSlow} slower</strong><span>Confirmed bootstrap: ${(cmp.summary && cmp.summary.regression) || 0}</span></button>
          <button type="button" class="bx-kpi" data-ck="emp_flat"><small>Within ±${st.threshold}%</small><strong>${empFlat} stable</strong><span>Threshold ±${st.threshold}%</span></button>`;
        part("kpis").querySelectorAll(".bx-kpi").forEach(b => {
          b.onclick = () => { st.filter = b.dataset.ck; renderFilters(rows); paintRows(); };
        });
        part("head").innerHTML = cmp.mode === "ab"
          ? `<p class="now-quiet">Paired A/B within <code>${escapeHtml(cmp.current.id)}</code></p>`
          : `<p class="now-quiet">Candidate ${link(cmp.current, shortId(cmp.current.id))} vs Baseline ${link(cmp.previous, shortId(cmp.previous.id))}</p>`;
        renderFilters(rows);
      } else {
        part("head").innerHTML = "";
        part("kpis").hidden = true;
      }
      paintRows();
    },
  };
}
