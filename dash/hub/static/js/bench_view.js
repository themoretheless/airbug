// Interactive Benchmark Explorer shared by `#/run/<id>` Results and Matrix views.
// Stateful view: `createBenchExplorer(host, callbacks)` -> `{ update(payload) }`.

import { enc, getJson, post, tryJson } from "./api.js";
import { escapeAttr, escapeHtml } from "./dom.js";
import { fmtNs, median } from "./format.js";

const ENGINE_PALETTE = {
  lin: "#1f4fd1",
  lin_batch: "#0f766e",
  lin_cursor: "#4338ca",
  lin_cursor_projected: "#2563eb",
  lin_grouped_full: "#0d9488",
  lin_sequential_full: "#64748b",
  sqlite: "#b45309",
  sqlite_fullfsync: "#c2410c",
  duckdb: "#7c3aed",
  duckdb_appender: "#9333ea",
  hashmap: "#2f7d4f",
  hash: "#2f7d4f",
};
const FALLBACK_COLORS = ["#1f4fd1", "#b45309", "#7c3aed", "#2f7d4f", "#c2410c", "#0f766e", "#be185d"];

export function engineColor(engine, idx = 0) {
  return ENGINE_PALETTE[engine] || FALLBACK_COLORS[idx % FALLBACK_COLORS.length];
}

export function fmtRate(rate, unit) {
  if (!Number.isFinite(rate) || rate <= 0) return "";
  const u = unit || "ops/s";
  if (rate >= 1e9) return (rate / 1e9).toFixed(2) + "B " + u;
  if (rate >= 1e6) return (rate / 1e6).toFixed(2) + "M " + u;
  if (rate >= 1e3) return (rate / 1e3).toFixed(1) + "k " + u;
  return rate.toFixed(1) + " " + u;
}

export function fmtMult(r) {
  if (!Number.isFinite(r) || r <= 0) return "—";
  if (r >= 100) return r.toFixed(0) + "×";
  if (r >= 10) return r.toFixed(1) + "×";
  if (r >= 1) return r.toFixed(2) + "×";
  return "−" + (1 / r).toFixed(2) + "×";
}

function quantile(sorted, q) {
  if (!sorted.length) return NaN;
  const pos = (sorted.length - 1) * q;
  const lo = Math.floor(pos);
  const hi = Math.ceil(pos);
  if (lo === hi) return sorted[lo];
  return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo);
}

function computeStats(values) {
  const s = values.slice().sort((a, b) => a - b);
  const n = s.length;
  const mean = s.reduce((a, b) => a + b, 0) / n;
  const variance = n > 1 ? s.reduce((a, v) => a + (v - mean) ** 2, 0) / (n - 1) : 0;
  const stddev = Math.sqrt(variance);
  const p25 = quantile(s, 0.25);
  const p75 = quantile(s, 0.75);
  return {
    count: n,
    min: s[0],
    p25,
    median: median(s),
    p75,
    p95: quantile(s, 0.95),
    max: s[n - 1],
    mean,
    stddev,
    iqr: p75 - p25,
    cvPct: mean > 0 ? (stddev / mean) * 100 : 0,
  };
}

function extractWorkSpec(contract) {
  if (!contract) return null;
  for (const [k, v] of Object.entries(contract)) {
    if (k.startsWith("work.counter.")) {
      const unit = k.slice("work.counter.".length);
      const count = Number(v);
      if (unit && Number.isFinite(count) && count > 0) return { unit, count, fixed: true };
    }
  }
  if (contract["work.count"] && contract["work.unit"]) {
    const count = Number(contract["work.count"]);
    if (Number.isFinite(count) && count > 0) return { unit: contract["work.unit"], count, fixed: true };
  }
  for (const [k, v] of Object.entries(contract)) {
    if (k.startsWith("work.input.") && v === "batch_total") {
      const unit = k.slice("work.input.".length);
      if (unit) return { unit, count: null, fixed: false };
    }
  }
  return null;
}

function inferTags(shortFamily, engine, contract) {
  const tags = new Set();
  for (const [k, v] of Object.entries(contract || {})) {
    if (k.startsWith("tag.") && v === "true") {
      const t = k.slice(4);
      if (t !== engine && t !== "phase") tags.add(t);
    }
  }
  const f = shortFamily.toLowerCase();
  if (f.startsWith("cold_")) tags.add("cold");
  if (f.includes("1row")) tags.add("1row");
  if (f.includes("join")) tags.add("join");
  if (f.includes("window")) tags.add("window");
  if (f.includes("agg")) tags.add("agg");
  if (f.includes("durable") || f.includes("group_commit") || f.includes("insert") || f.includes("append")) tags.add("write");
  if (f.includes("fts") || f.includes("text")) tags.add("text");
  return [...tags];
}

function extractParams(contract) {
  const out = [];
  for (const [k, v] of Object.entries(contract || {})) {
    if (k.startsWith("param.")) out.push(`${k.slice(6)}=${v}`);
  }
  return out;
}

export function parseRunJson(runData, suiteLabel = "") {
  if (!runData || !Array.isArray(runData.cases)) return null;
  const caseById = new Map();
  for (const c of runData.cases) {
    if (c && c.id) caseById.set(c.id, c);
  }
  const obsByCase = new Map();
  let totalObs = 0;
  for (const o of runData.observations || []) {
    if (!o || o.metric !== "wall") continue;
    const av = o.availability;
    const avState = typeof av === "string" ? av : (av && av.state);
    if (avState && avState !== "available") continue;
    const val = Number(o.value);
    if (!Number.isFinite(val)) continue;
    totalObs++;
    const list = obsByCase.get(o.case) || [];
    list.push(o);
    obsByCase.set(o.case, list);
  }

  const cases = [];
  const enginesSet = new Set();
  const processesSet = new Set();

  for (const c of runData.cases) {
    const rawList = obsByCase.get(c.id) || [];
    if (!rawList.length) continue;
    const parts = c.id.split("/");
    const engine = parts[parts.length - 1] || c.id;
    const familyId = parts.slice(0, -1).join("/") || c.id;
    const shortFamily = parts.length >= 2 ? parts[parts.length - 2] : c.id;
    const wallMetric = (c.metrics || []).find(m => m.id === "wall") || (c.metrics && c.metrics[0]);
    const isBatch = (wallMetric && wallMetric.statistic === "batch_total")
      || c.measurement_mode === "batch_total"
      || (c.contract && c.contract["measurement_mode"] === "batch_total");
    const work = extractWorkSpec(c.contract);

    const samples = [];
    const nsVals = [];
    const rateVals = [];
    let rateUnit = work ? (work.unit === "bytes" ? "MiB/s" : work.unit + "/s") : "";

    for (let idx = 0; idx < rawList.length; idx++) {
      const o = rawList[idx];
      const rawNs = Number(o.value);
      const ops = isBatch ? Math.max(1, Number(o.operations) || 1) : 1;
      const nsPerOp = rawNs / ops;
      const proc = Number(o.process || 0);
      processesSet.add(proc);
      let rate = null;
      if (work && rawNs > 0) {
        const totalWork = work.fixed ? work.count * ops : Number((o.work_totals || {})[work.unit]);
        if (Number.isFinite(totalWork) && totalWork > 0) {
          rate = (totalWork * 1e9) / rawNs;
          if (work.unit === "bytes") rate /= 1048576;
          if (Number.isFinite(rate)) rateVals.push(rate);
        }
      }
      nsVals.push(nsPerOp);
      samples.push({
        seq: Number(o.sequence != null ? o.sequence : idx),
        proc,
        nsPerOp,
        rawNs,
        ops,
        rate,
      });
    }
    if (!nsVals.length) continue;
    enginesSet.add(engine);
    const stats = computeStats(nsVals);
    const throughput = rateVals.length ? median(rateVals) : null;
    cases.push({
      id: c.id,
      suiteLabel,
      familyId: suiteLabel ? `${suiteLabel} · ${familyId}` : familyId,
      rawFamilyId: familyId,
      shortFamily,
      engine,
      stats,
      throughput,
      rateUnit,
      throughputText: throughput ? fmtRate(throughput, rateUnit) : "",
      processes: new Set(samples.map(s => s.proc)).size,
      samples,
      tags: inferTags(shortFamily, engine, c.contract),
      params: extractParams(c.contract),
      contract: c.contract || {},
    });
  }

  const familyMap = new Map();
  for (const item of cases) {
    let fam = familyMap.get(item.familyId);
    if (!fam) {
      fam = {
        id: item.familyId,
        rawFamilyId: item.rawFamilyId,
        shortFamily: item.shortFamily,
        suiteLabel: item.suiteLabel,
        items: [],
        byEngine: {},
        tags: new Set(),
        params: item.params,
      };
      familyMap.set(item.familyId, fam);
    }
    fam.items.push(item);
    fam.byEngine[item.engine] = item;
    for (const t of item.tags) fam.tags.add(t);
  }

  const families = [...familyMap.values()].map(f => ({
    ...f,
    tags: [...f.tags],
    engines: f.items.map(x => x.engine),
  }));

  return {
    status: runData.status || "",
    environment: runData.environment || {},
    cases,
    families,
    engines: [...enginesSet],
    totalObs,
    processCount: processesSet.size || 1,
    notes: runData.notes || [],
  };
}

export const LAUNCH_PRESETS = [
  { id: "ci", label: "CI Core (quick)", filter: "", excludes: ["compare/durable*", "compare/cold*", "compare/wal*", "compare/hot_reopen*", "compare/*1row*", "compare/*phase*"], profile: "quick" },
  { id: "all", label: "All Suites", filter: "", excludes: [], profile: "quick" },
  { id: "1row", label: "1-Row DML & TX", filter: "1row", excludes: [], profile: "quick" },
  { id: "cold", label: "Cold Queries", filter: "cold_", excludes: [], profile: "quick" },
  { id: "durable", label: "Durable Full Sync", filter: "durable_insert_", excludes: [], profile: "quick" },
  { id: "window", label: "Window & Join", filter: "window", excludes: [], profile: "quick" },
  { id: "mixed", label: "Mixed-Case Substr", filter: "cold_text_substr", excludes: [], profile: "quick", env_mixed_case: true },
];

export function renderLaunchPanel(host, onLaunched) {
  host.innerHTML = `<div class="bench-launcher-head">
      <strong>Launch live benchmark</strong>
      <span class="bench-live" id="bl-target-meta">loading targets…</span>
    </div>
    <div class="bench-presets" id="bl-presets"></div>
    <form class="bench-launch-form" id="bl-form">
      <label>Target <select id="bl-target"></select></label>
      <label>Filter <input type="text" id="bl-filter" placeholder="e.g. join_inner or 1row" /></label>
      <label>Profile <select id="bl-profile">
        <option value="quick">quick</option>
        <option value="normal">normal</option>
        <option value="thorough">thorough</option>
      </select></label>
      <label>Samples <input type="number" id="bl-samples" min="1" max="1000" placeholder="default" /></label>
      <label class="bench-chk"><input type="checkbox" id="bl-exact" /> exact</label>
      <label class="bench-chk"><input type="checkbox" id="bl-mixed" /> MIXED_CASE</label>
      <button type="submit" class="atr-btn primary" id="bl-submit">▶ Run live</button>
    </form>
    <div class="bench-live" id="bl-status"></div>`;

  let excludes = [];
  const presetsEl = host.querySelector("#bl-presets");
  presetsEl.innerHTML = LAUNCH_PRESETS.map(p =>
    `<button type="button" class="mini-btn" data-preset="${escapeAttr(p.id)}">${escapeHtml(p.label)}</button>`
  ).join("");

  presetsEl.querySelectorAll("button[data-preset]").forEach(btn => {
    btn.addEventListener("click", () => {
      const p = LAUNCH_PRESETS.find(x => x.id === btn.dataset.preset);
      if (!p) return;
      host.querySelector("#bl-filter").value = p.filter;
      host.querySelector("#bl-profile").value = p.profile;
      host.querySelector("#bl-mixed").checked = Boolean(p.env_mixed_case);
      host.querySelector("#bl-exact").checked = false;
      excludes = p.excludes.slice();
      presetsEl.querySelectorAll("button").forEach(b => b.classList.toggle("on", b === btn));
    });
  });

  tryJson("bench/targets", null, { targets: ["compare"] }).then(info => {
    const targets = info.targets && info.targets.length ? info.targets : ["compare"];
    host.querySelector("#bl-target").innerHTML = targets.map(t =>
      `<option value="${escapeAttr(t)}">${escapeHtml(t)}</option>`
    ).join("");
    host.querySelector("#bl-target-meta").textContent = [info.package, targets.length + " target(s)"].filter(Boolean).join(" · ");
  });

  host.querySelector("#bl-form").addEventListener("submit", async ev => {
    ev.preventDefault();
    const btn = host.querySelector("#bl-submit");
    const status = host.querySelector("#bl-status");
    btn.disabled = true;
    status.textContent = "Starting cargo bench…";
    const samplesRaw = host.querySelector("#bl-samples").value.trim();
    const payload = {
      bench: host.querySelector("#bl-target").value || "compare",
      filter: host.querySelector("#bl-filter").value.trim() || null,
      exact: host.querySelector("#bl-exact").checked,
      profile: host.querySelector("#bl-profile").value,
      samples: samplesRaw ? Number(samplesRaw) : null,
      excludes,
      env_mixed_case: host.querySelector("#bl-mixed").checked,
    };
    try {
      const started = await post("bench/launch", payload);
      status.textContent = `Started ${started.run_id}`;
      if (onLaunched) onLaunched(started);
      else location.hash = started.href;
    } catch (err) {
      status.textContent = err.message || String(err);
    } finally {
      btn.disabled = false;
    }
  });
}

function choosePrimaryItem(family, primary, bestLinBatch) {
  if (primary === "lin" && bestLinBatch && family.byEngine.lin && family.byEngine.lin_batch) {
    return family.byEngine.lin_batch.stats.median < family.byEngine.lin.stats.median
      ? family.byEngine.lin_batch
      : family.byEngine.lin;
  }
  return family.byEngine[primary] || null;
}

function choosePeerItem(family, primaryItem, peerMode) {
  const candidates = family.items.filter(x => !primaryItem || x.engine !== primaryItem.engine);
  if (!candidates.length) return null;
  if (peerMode === "fastest") return candidates.slice().sort((a, b) => a.stats.median - b.stats.median)[0];
  if (peerMode === "slowest") return candidates.slice().sort((a, b) => b.stats.median - a.stats.median)[0];
  if (family.byEngine[peerMode] && (!primaryItem || peerMode !== primaryItem.engine)) return family.byEngine[peerMode];
  return null;
}

function familyOutcome(ratio) {
  if (ratio == null) return "single";
  if (ratio < 1.0) return "loss";
  if (ratio >= 1.15) return "win";
  return "lag";
}

function logTicks(minV, maxV, count = 4) {
  const lo = Math.log10(Math.max(1, minV));
  const hi = Math.log10(Math.max(minV * 1.1, maxV));
  const out = [];
  for (let i = 0; i <= count; i++) {
    const val = 10 ** (lo + ((hi - lo) * i) / count);
    out.push(val);
  }
  return out;
}

function renderBoxSamplesSvg(fam, visibleEngines, scale) {
  const items = fam.items.filter(x => visibleEngines.has(x.engine));
  if (!items.length) return `<p class="now-quiet">No visible engines in this family.</p>`;
  const allVals = items.flatMap(it => it.samples.map(s => s.nsPerOp));
  const minVal = Math.max(0.5, Math.min(...allVals) * 0.85);
  const maxVal = Math.max(...allVals, minVal * 1.2) * 1.08;
  const maxNameLen = Math.max(8, ...items.map(it => it.engine.length));
  const w = 520, padL = Math.min(168, Math.max(116, Math.ceil(maxNameLen * 6.8) + 14)), padR = 30, padT = 18, padB = 26, rowH = 32;
  const h = padT + padB + items.length * rowH;
  const plotW = w - padL - padR;

  const toX = v => {
    const norm = scale === "log"
      ? (Math.log10(Math.max(0.5, v)) - Math.log10(minVal)) / Math.max(0.1, Math.log10(maxVal) - Math.log10(minVal))
      : v / Math.max(1, maxVal);
    return padL + Math.max(0, Math.min(1, norm)) * plotW;
  };

  const ticks = scale === "log"
    ? logTicks(minVal, maxVal, 4)
    : [0, 0.25, 0.5, 0.75, 1].map(f => maxVal * f);

  const grid = ticks.map((t, idx) => {
    const x = toX(t);
    const isLast = idx === ticks.length - 1;
    const tx = isLast ? Math.min(w - 4, x + 8) : x;
    const anchor = isLast ? "end" : "middle";
    return `<line x1="${x.toFixed(1)}" y1="${padT - 4}" x2="${x.toFixed(1)}" y2="${h - padB}" stroke="#d9d9d4" stroke-dasharray="2 3" />
      <text x="${tx.toFixed(1)}" y="${h - 8}" text-anchor="${anchor}" font-size="9.5" fill="#5c5c58">${escapeHtml(fmtNs(t))}</text>`;
  }).join("");

  const rows = items.map((it, i) => {
    const col = engineColor(it.engine, i);
    const cy = padT + i * rowH + rowH / 2;
    const s = it.stats;
    const xMin = toX(s.min), xP25 = toX(s.p25), xMed = toX(s.median), xP75 = toX(s.p75), xMax = toX(s.max);
    const boxTip = `${it.engine}: median ${fmtNs(s.median)}, p25 ${fmtNs(s.p25)}, p75 ${fmtNs(s.p75)}, CV ${s.cvPct.toFixed(1)}% (${s.count} obs)`;
    const dots = it.samples.map((smp, j) => {
      const cx = toX(smp.nsPerOp);
      const jy = cy + (((j % 5) - 2) * 2.6);
      const delta = s.median > 0 ? ((smp.nsPerOp - s.median) / s.median) * 100 : 0;
      const tip = `${it.engine} · p${smp.proc} · #${smp.seq}\n${fmtNs(smp.nsPerOp)} (${delta >= 0 ? "+" : ""}${delta.toFixed(1)}% vs med)\nops/batch: ${smp.ops}${smp.rate ? " · " + fmtRate(smp.rate, it.rateUnit) : ""}`;
      return `<circle cx="${cx.toFixed(1)}" cy="${jy.toFixed(1)}" r="3.4" fill="${col}" stroke="#fff" stroke-width="1" class="bx-pt" data-tip="${escapeAttr(tip)}" />`;
    }).join("");
    return `<g>
      <text x="${padL - 8}" y="${cy + 3.5}" text-anchor="end" font-size="11" fill="#1b1b1a" font-family="IBM Plex Mono, monospace">${escapeHtml(it.engine)}</text>
      <line x1="${xMin.toFixed(1)}" y1="${cy}" x2="${xMax.toFixed(1)}" y2="${cy}" stroke="${col}" stroke-width="1.5" stroke-opacity="0.6" />
      <rect x="${xP25.toFixed(1)}" y="${cy - 7}" width="${Math.max(3, xP75 - xP25).toFixed(1)}" height="14" rx="2" fill="${col}" fill-opacity="0.18" stroke="${col}" stroke-width="1.4" class="bx-pt" data-tip="${escapeAttr(boxTip)}" />
      <line x1="${xMed.toFixed(1)}" y1="${cy - 9}" x2="${xMed.toFixed(1)}" y2="${cy + 9}" stroke="${col}" stroke-width="2.6" />
      ${dots}
    </g>`;
  }).join("");

  return `<svg class="bench-svg" viewBox="0 0 ${w} ${h}" role="img" aria-label="Distribution per engine">${grid}${rows}</svg>`;
}

function renderSequenceSvg(fam, visibleEngines, scale) {
  const items = fam.items.filter(x => visibleEngines.has(x.engine));
  if (!items.length) return "";
  const allVals = items.flatMap(it => it.samples.map(s => s.nsPerOp));
  const minVal = Math.max(0.5, Math.min(...allVals) * 0.85);
  const maxVal = Math.max(...allVals, minVal * 1.2) * 1.08;
  const maxSeq = Math.max(1, ...items.flatMap(it => it.samples.map((s, idx) => idx)));
  const w = 520, h = 195, padL = 64, padR = 16, padT = 16, padB = 26;
  const plotW = w - padL - padR, plotH = h - padT - padB;

  const toX = idx => padL + (idx / Math.max(1, maxSeq)) * plotW;
  const toY = v => {
    const norm = scale === "log"
      ? (Math.log10(Math.max(0.5, v)) - Math.log10(minVal)) / Math.max(0.1, Math.log10(maxVal) - Math.log10(minVal))
      : v / Math.max(1, maxVal);
    return padT + plotH - Math.max(0, Math.min(1, norm)) * plotH;
  };

  const yTicks = scale === "log" ? logTicks(minVal, maxVal, 3) : [0, 0.33, 0.66, 1].map(f => maxVal * f);
  const grid = yTicks.map(t => {
    const y = toY(t);
    return `<line x1="${padL}" y1="${y.toFixed(1)}" x2="${w - padR}" y2="${y.toFixed(1)}" stroke="#d9d9d4" stroke-dasharray="2 3" />
      <text x="${padL - 6}" y="${(y + 3).toFixed(1)}" text-anchor="end" font-size="9.5" fill="#5c5c58">${escapeHtml(fmtNs(t))}</text>`;
  }).join("");

  const lines = items.map((it, i) => {
    const col = engineColor(it.engine, i);
    // Group by process so lines do not jump awkwardly across repetitions.
    const byProc = new Map();
    it.samples.forEach((s, idx) => {
      const arr = byProc.get(s.proc) || [];
      arr.push({ ...s, idx });
      byProc.set(s.proc, arr);
    });
    const procPaths = [...byProc.entries()].map(([proc, arr]) => {
      const pts = arr.map((s, k) => `${toX(k).toFixed(1)},${toY(s.nsPerOp).toFixed(1)}`).join(" ");
      const dots = arr.map((s, k) => {
        const tip = `${it.engine} · p${proc} · obs #${s.seq}\n${fmtNs(s.nsPerOp)} (ops=${s.ops})`;
        return `<circle cx="${toX(k).toFixed(1)}" cy="${toY(s.nsPerOp).toFixed(1)}" r="3.2" fill="${col}" stroke="#fff" stroke-width="1" class="bx-pt" data-tip="${escapeAttr(tip)}" />`;
      }).join("");
      return `<polyline fill="none" stroke="${col}" stroke-width="1.6" stroke-opacity="${proc > 0 ? "0.6" : "0.95"}" ${proc > 0 ? 'stroke-dasharray="4 2"' : ""} points="${pts}" />${dots}`;
    }).join("");
    return procPaths;
  }).join("");

  const legend = items.map((it, i) =>
    `<span class="bx-leg"><i style="background:${engineColor(it.engine, i)}"></i>${escapeHtml(it.engine)}</span>`
  ).join("");

  return `<svg class="bench-svg" viewBox="0 0 ${w} ${h}" role="img" aria-label="Sample sequence trace">${grid}${lines}
    <text x="${padL + plotW / 2}" y="${h - 6}" text-anchor="middle" font-size="9.5" fill="#5c5c58">sample sequence</text></svg>
    <div class="bx-legend">${legend}</div>`;
}

export function createBenchExplorer(host, callbacks = {}) {
  host.innerHTML = `
    <div class="bx-top" data-part="top"></div>
    <div class="bx-drawer" data-part="drawer" hidden></div>
    <div class="bx-kpis" data-part="kpis"></div>
    <div class="bx-subtabs" data-part="subtabs"></div>
    <div class="bx-controls" data-part="controls">
      <div class="bx-ctrl-row">
        <input type="search" class="bx-search" data-part="search" placeholder="Filter case, engine, tag, param… (press /)" autocomplete="off" />
        <label class="bx-lbl">Primary <select data-part="primary"></select></label>
        <label class="bx-lbl">vs Peer <select data-part="peer"></select></label>
        <div class="bx-seg" data-part="mode"></div>
        <div class="bx-seg" data-part="scale"></div>
        <label class="bx-lbl">Sort <select data-part="sort"></select></label>
      </div>
      <div class="bx-ctrl-row">
        <div class="bx-chips" data-part="outcomes"></div>
        <div class="bx-chips" data-part="tags"></div>
        <label class="bench-chk"><input type="checkbox" data-part="bestbatch" checked /> best lin_batch</label>
      </div>
      <div class="bx-engines" data-part="engines"></div>
    </div>
    <div class="bx-stage" data-part="stage"></div>
    <div class="bx-tip" data-part="tip" hidden></div>`;

  const part = name => host.querySelector(`[data-part="${name}"]`);
  const state = {
    runMeta: null,
    parsed: null,
    matrixParsed: null,
    matrixMeta: null,
    matrixLoading: false,
    live: false,
    drawerOpen: false,
    subview: "explorer", // explorer | distributions | table | matrix | report
    plotTab: "box",      // box | trace
    query: "",
    primary: "lin",
    peer: "sqlite",
    bestBatch: true,
    mode: "speedup",     // speedup | latency | throughput
    scale: "log",        // log | lin
    sort: "default",     // default | win | lag | fastest | slowest | throughput | cv
    outcome: "all",      // all | win | lag | single
    tag: "all",
    visibleEngines: null,
    selectedFamilyId: null,
    tableSort: { col: "median", asc: true },
    sig: "",
  };

  function bindTooltip(root) {
    const tip = part("tip");
    root.querySelectorAll(".bx-pt[data-tip]").forEach(el => {
      el.addEventListener("mouseenter", ev => {
        tip.textContent = el.getAttribute("data-tip") || "";
        tip.hidden = false;
        positionTip(ev);
      });
      el.addEventListener("mousemove", positionTip);
      el.addEventListener("mouseleave", () => { tip.hidden = true; });
    });
    function positionTip(ev) {
      const pad = 12;
      const x = Math.min(window.innerWidth - 260, ev.clientX + pad);
      const y = Math.min(window.innerHeight - 90, ev.clientY + pad);
      tip.style.left = Math.max(8, x) + "px";
      tip.style.top = Math.max(8, y) + "px";
    }
  }

  function activeData() {
    return state.subview === "matrix" && state.matrixParsed ? state.matrixParsed : state.parsed;
  }

  function ensureSelections(parsed) {
    if (!parsed) return;
    const engines = parsed.engines;
    if (!engines.includes(state.primary)) {
      state.primary = engines.includes("lin") ? "lin" : (engines[0] || "lin");
    }
    const validPeers = ["sqlite", "duckdb", "sqlite_fullfsync", "duckdb_appender", "fastest", "slowest", ...engines];
    if (!validPeers.includes(state.peer)) {
      state.peer = engines.includes("sqlite") ? "sqlite" : (engines.find(e => e !== state.primary) || "fastest");
    }
    if (!state.visibleEngines) {
      state.visibleEngines = new Set(engines);
    } else {
      for (const e of engines) {
        if (!state.visibleEngines.has(e) && state.visibleEngines.size === 0) state.visibleEngines.add(e);
      }
      // Auto-include any newly arrived engine from a live run.
      for (const e of engines) {
        if (!state._seenEngines || !state._seenEngines.has(e)) state.visibleEngines.add(e);
      }
    }
    state._seenEngines = new Set(engines);
  }

  function decorateFamilies(parsed) {
    if (!parsed) return [];
    return parsed.families.map(f => {
      const pItem = choosePrimaryItem(f, state.primary, state.bestBatch);
      const peerItem = choosePeerItem(f, pItem, state.peer);
      const ratio = pItem && peerItem && pItem.stats.median > 0
        ? peerItem.stats.median / pItem.stats.median
        : null;
      const bestRate = Math.max(0, ...f.items.map(x => x.throughput || 0));
      const maxCv = Math.max(0, ...f.items.map(x => x.stats.cvPct || 0));
      const fastestMed = Math.min(...f.items.map(x => x.stats.median));
      return {
        ...f,
        primaryItem: pItem,
        peerItem,
        ratio,
        outcome: familyOutcome(ratio),
        bestRate,
        maxCv,
        fastestMed,
      };
    });
  }

  function filterAndSort(families) {
    const q = state.query.trim().toLowerCase();
    let list = families.filter(f => {
      if (state.outcome === "lag") {
        if (f.outcome !== "loss" && f.outcome !== "lag") return false;
      } else if (state.outcome !== "all" && f.outcome !== state.outcome) {
        return false;
      }
      if (state.tag !== "all" && !f.tags.includes(state.tag)) return false;
      if (state.visibleEngines && !f.items.some(x => state.visibleEngines.has(x.engine))) return false;
      if (q) {
        const hay = `${f.id} ${f.engines.join(" ")} ${f.tags.join(" ")} ${f.params.join(" ")}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
    if (state.sort === "win") {
      list = list.slice().sort((a, b) => (b.ratio ?? -1) - (a.ratio ?? -1));
    } else if (state.sort === "lag") {
      list = list.slice().sort((a, b) => (a.ratio ?? 999999) - (b.ratio ?? 999999));
    } else if (state.sort === "fastest") {
      list = list.slice().sort((a, b) => (a.primaryItem ? a.primaryItem.stats.median : a.fastestMed) - (b.primaryItem ? b.primaryItem.stats.median : b.fastestMed));
    } else if (state.sort === "slowest") {
      list = list.slice().sort((a, b) => (b.primaryItem ? b.primaryItem.stats.median : b.fastestMed) - (a.primaryItem ? a.primaryItem.stats.median : a.fastestMed));
    } else if (state.sort === "throughput") {
      list = list.slice().sort((a, b) => b.bestRate - a.bestRate);
    } else if (state.sort === "cv") {
      list = list.slice().sort((a, b) => b.maxCv - a.maxCv);
    }
    return list;
  }

  async function launchFilter(filterPat) {
    const cmdText = Array.isArray(state.runMeta && state.runMeta.command)
      ? state.runMeta.command.join(" ")
      : String((state.runMeta && state.runMeta.command) || "");
    const bMatch = cmdText.match(/--bench\s+([A-Za-z0-9_-]+)/);
    const payload = { bench: (bMatch && bMatch[1]) || null, filter: filterPat || null, profile: "quick" };
    if (cmdText.includes("LIN_BENCH_MIXED_CASE=1")) payload.env_mixed_case = true;
    if (!filterPat && cmdText.includes("ci-bench.sh")) {
      payload.excludes = LAUNCH_PRESETS[0].excludes.slice();
    }
    try {
      const res = await post("bench/launch", payload);
      if (callbacks.onNavigate) callbacks.onNavigate(res.href);
      else location.hash = res.href;
    } catch (err) {
      part("stage").insertAdjacentHTML("afterbegin", `<p class="now-quiet err-txt">${escapeHtml(err.message || String(err))}</p>`);
    }
  }

  function renderTopBar() {
    const meta = state.runMeta || {};
    const cmdArr = Array.isArray(meta.command) ? meta.command.join(" ") : (meta.command || "");
    const fileLinks = [
      meta.has_run_json && `<a href="${escapeAttr(meta.files + "run.json")}" target="_blank" rel="noopener">run.json ↗</a>`,
      meta.has_report_html && `<a href="${escapeAttr(meta.files + "report.html")}" target="_blank" rel="noopener">report.html ↗</a>`,
    ].filter(Boolean).join(" · ");

    part("top").innerHTML = `<div class="bx-actions">
      <button type="button" class="atr-btn primary" id="bx-rerun" ${state.live ? "disabled" : ""}>▶ Rerun suite</button>
      <button type="button" class="atr-btn" id="bx-toggle-launch">${state.drawerOpen ? "Hide launcher" : "▶ Quick run…"}</button>
      ${meta.can_cancel ? `<button type="button" class="atr-btn bad" id="bx-stop">■ Stop run</button>` : ""}
      ${cmdArr ? `<code class="atr-cmd" title="Recorded command">${escapeHtml(cmdArr)}</code>` : ""}
      <span class="bx-file-links">${fileLinks}</span>
    </div>`;

    const rerunBtn = part("top").querySelector("#bx-rerun");
    if (rerunBtn) {
      rerunBtn.onclick = () => {
        const fMatch = cmdArr.match(/--filter\s+(\S+)/);
        launchFilter(fMatch ? fMatch[1] : "");
      };
    }
    part("top").querySelector("#bx-toggle-launch").onclick = () => {
      state.drawerOpen = !state.drawerOpen;
      const drawer = part("drawer");
      drawer.hidden = !state.drawerOpen;
      if (state.drawerOpen && !drawer.dataset.bound) {
        drawer.dataset.bound = "1";
        renderLaunchPanel(drawer, started => {
          state.drawerOpen = false;
          drawer.hidden = true;
          if (callbacks.onNavigate) callbacks.onNavigate(started.href);
          else location.hash = started.href;
        });
      }
      renderTopBar();
    };
    const stopBtn = part("top").querySelector("#bx-stop");
    if (stopBtn && meta.run_id) {
      stopBtn.onclick = async () => {
        stopBtn.disabled = true;
        await post(`bench/runs/${enc(meta.run_id)}/cancel`, {}).catch(() => {});
      };
    }
  }

  function renderSubtabs() {
    const meta = state.runMeta || {};
    const tabs = [
      ["explorer", "Explorer"],
      ["distributions", "Distributions"],
      ["table", "Sortable Table"],
      ["matrix", state.matrixLoading ? "All Suites Matrix…" : "All Suites Matrix"],
    ];
    if (meta.has_report_html) tabs.push(["report", "Raw Report HTML"]);
    part("subtabs").innerHTML = tabs.map(([id, label]) =>
      `<button type="button" class="traces-view-btn${state.subview === id ? " on" : ""}" data-sv="${id}">${escapeHtml(label)}</button>`
    ).join("");
    part("subtabs").querySelectorAll("button").forEach(b => {
      b.onclick = () => {
        state.subview = b.dataset.sv;
        if (state.subview === "matrix" && !state.matrixParsed && !state.matrixLoading) loadMatrix();
        paint();
      };
    });
  }

  async function loadMatrix() {
    state.matrixLoading = true;
    renderSubtabs();
    try {
      const timeline = await getJson("runs", { kind: "bench", limit: 40 });
      const runs = (timeline.runs || []).filter(r => ["complete", "passed"].includes(r.state));
      const picked = [];
      const seenTitles = new Set();
      for (const r of runs) {
        const key = (r.title || r.run_id).toLowerCase();
        if (seenTitles.has(key)) continue;
        seenTitles.add(key);
        picked.push(r);
        if (picked.length >= 10) break;
      }
      const loaded = await Promise.all(picked.map(async r => {
        const desc = await tryJson("run", { id: r.run_id });
        if (!desc || !desc.has_run_json) return null;
        const json = await getJson(desc.files + "run.json").catch(() => null);
        if (!json) return null;
        const label = r.title || r.run_id.slice(0, 8);
        return parseRunJson(json, label);
      }));
      const merged = {
        status: "complete",
        environment: {},
        cases: [],
        families: [],
        engines: [],
        totalObs: 0,
        processCount: 0,
        notes: [],
      };
      const engSet = new Set();
      for (const p of loaded.filter(Boolean)) {
        merged.cases.push(...p.cases);
        merged.families.push(...p.families);
        merged.totalObs += p.totalObs;
        merged.processCount = Math.max(merged.processCount, p.processCount);
        for (const e of p.engines) engSet.add(e);
      }
      merged.engines = [...engSet];
      state.matrixParsed = merged;
    } finally {
      state.matrixLoading = false;
      paint();
    }
  }

  function renderKpis(families, parsed) {
    const cmp = families.filter(f => f.ratio != null);
    const wins = cmp.filter(f => f.ratio >= 1.15).length;
    const parity = cmp.filter(f => f.ratio >= 1.0 && f.ratio < 1.15).length;
    const lags = cmp.filter(f => f.ratio < 1.0).length;
    const bestWin = cmp.slice().sort((a, b) => b.ratio - a.ratio)[0];
    const fastest = parsed.cases.slice().sort((a, b) => a.stats.median - b.stats.median)[0];
    const peakTput = parsed.cases.filter(c => c.throughput).sort((a, b) => b.throughput - a.throughput)[0];

    part("kpis").innerHTML = `
      <button type="button" class="bx-kpi" data-kpi="reset">
        <small>Cases &amp; Samples</small>
        <strong>${parsed.cases.length} cases · ${families.length} families</strong>
        <span>${parsed.totalObs} obs · ${parsed.processCount} proc</span>
      </button>
      <button type="button" class="bx-kpi" data-kpi="score">
        <small>${escapeHtml(state.primary)} vs ${escapeHtml(state.peer)}</small>
        <strong>${cmp.length ? `${wins} / ${cmp.length} wins` : `${parsed.cases.length} single-engine`}</strong>
        <span>${cmp.length ? `${parity} narrow (1–1.15×) · ${lags ? `<b class="bad">${lags} behind (&lt;1×)</b>` : "0 behind"} (click filter)` : "no shared peer in selection"}</span>
      </button>
      <button type="button" class="bx-kpi" data-kpi="best" ${bestWin ? `data-fam="${escapeAttr(bestWin.id)}"` : ""}>
        <small>Peak speedup</small>
        <strong>${bestWin ? fmtMult(bestWin.ratio) : "—"}</strong>
        <span>${bestWin ? `${escapeHtml(bestWin.shortFamily)} vs ${escapeHtml(bestWin.peerItem.engine)}` : "select a peer"}</span>
      </button>
      <button type="button" class="bx-kpi" data-kpi="fast" ${peakTput ? `data-fam="${escapeAttr(peakTput.familyId)}"` : fastest ? `data-fam="${escapeAttr(fastest.familyId)}"` : ""}>
        <small>${peakTput ? "Peak throughput" : "Fastest median"}</small>
        <strong>${peakTput ? escapeHtml(peakTput.throughputText) : fastest ? fmtNs(fastest.stats.median) : "—"}</strong>
        <span>${fastest ? `fastest: ${escapeHtml(fastest.shortFamily)} (${fmtNs(fastest.stats.median)})` : ""}</span>
      </button>`;

    part("kpis").querySelectorAll(".bx-kpi").forEach(btn => {
      btn.onclick = () => {
        const k = btn.dataset.kpi;
        if (k === "reset") {
          state.outcome = "all";
          state.tag = "all";
          state.query = "";
          part("search").value = "";
        } else if (k === "score") {
          state.outcome = state.outcome === "loss"
            ? "lag"
            : state.outcome === "lag"
              ? "win"
              : state.outcome === "win"
                ? "all"
                : "loss";
        } else if (btn.dataset.fam) {
          state.selectedFamilyId = btn.dataset.fam;
          state.subview = "explorer";
        }
        paint();
      };
    });
  }

  function renderControls(parsed, families) {
    ensureSelections(parsed);
    const engines = parsed.engines;
    const engSig = engines.join(",");
    const primarySel = part("primary");
    if (primarySel.dataset.engSig !== engSig) {
      primarySel.dataset.engSig = engSig;
      primarySel.innerHTML = engines.map(e =>
        `<option value="${escapeAttr(e)}"${e === state.primary ? " selected" : ""}>${escapeHtml(e)}</option>`
      ).join("");
    } else {
      primarySel.value = state.primary;
    }
    primarySel.onchange = () => { state.primary = primarySel.value; paint(); };

    const peerOpts = ["sqlite", "duckdb", "sqlite_fullfsync", "duckdb_appender", ...engines, "fastest", "slowest"]
      .filter((v, i, a) => a.indexOf(v) === i);
    const peerSel = part("peer");
    const peerSig = peerOpts.join(",");
    if (peerSel.dataset.peerSig !== peerSig) {
      peerSel.dataset.peerSig = peerSig;
      peerSel.innerHTML = peerOpts.map(p =>
        `<option value="${escapeAttr(p)}"${p === state.peer ? " selected" : ""}>${escapeHtml(p)}</option>`
      ).join("");
    } else {
      peerSel.value = state.peer;
    }
    peerSel.onchange = () => { state.peer = peerSel.value; paint(); };

    const modes = [["speedup", "Speedup ×"], ["latency", "Latency"], ["throughput", "Throughput"]];
    part("mode").innerHTML = modes.map(([id, lbl]) =>
      `<button type="button" class="${state.mode === id ? "on" : ""}" data-m="${id}">${lbl}</button>`
    ).join("");
    part("mode").querySelectorAll("button").forEach(b => b.onclick = () => { state.mode = b.dataset.m; paint(); });

    const scales = [["log", "Log10"], ["lin", "Linear"]];
    part("scale").innerHTML = scales.map(([id, lbl]) =>
      `<button type="button" class="${state.scale === id ? "on" : ""}" data-sc="${id}">${lbl}</button>`
    ).join("");
    part("scale").querySelectorAll("button").forEach(b => b.onclick = () => { state.scale = b.dataset.sc; paint(); });

    const sorts = [
      ["default", "Suite order"],
      ["win", "Biggest speedup"],
      ["lag", "Lag / Parity first"],
      ["fastest", "Fastest latency"],
      ["slowest", "Slowest latency"],
      ["throughput", "Highest throughput"],
      ["cv", "Highest CV% jitter"],
    ];
    const sortSel = part("sort");
    if (!sortSel.options.length) {
      sortSel.innerHTML = sorts.map(([id, lbl]) =>
        `<option value="${id}"${id === state.sort ? " selected" : ""}>${lbl}</option>`
      ).join("");
    }
    sortSel.value = state.sort;
    sortSel.onchange = () => { state.sort = sortSel.value; paint(); };

    const lagsCount = families.filter(f => f.ratio != null && f.ratio < 1.0).length;
    const outcomes = [
      ["all", "All"],
      ["loss", lagsCount ? `Behind (${lagsCount})` : "Behind (<1×)"],
      ["win", "Wins (>1.15×)"],
      ["lag", "Parity / Lag"],
      ["single", "Single-engine"],
    ];
    part("outcomes").innerHTML = outcomes.map(([id, lbl]) => {
      const cls = [state.outcome === id ? "on" : "", id === "loss" && lagsCount ? "bad-chip" : ""].filter(Boolean).join(" ");
      return `<button type="button" class="${cls}" data-out="${id}">${lbl}</button>`;
    }).join("");
    part("outcomes").querySelectorAll("button").forEach(b => b.onclick = () => { state.outcome = b.dataset.out; paint(); });

    const tagCounts = new Map();
    for (const f of families) {
      for (const t of f.tags) tagCounts.set(t, (tagCounts.get(t) || 0) + 1);
    }
    const topTags = [...tagCounts.entries()].sort((a, b) => b[1] - a[1]).slice(0, 10);
    part("tags").innerHTML = [["all", families.length], ...topTags].map(([t, c]) =>
      `<button type="button" class="${state.tag === t ? "on" : ""}" data-tag="${escapeAttr(t)}">${escapeHtml(t)} <small>${c}</small></button>`
    ).join("");
    part("tags").querySelectorAll("button").forEach(b => b.onclick = () => { state.tag = b.dataset.tag; paint(); });

    const bb = part("bestbatch");
    bb.checked = state.bestBatch;
    bb.onchange = () => { state.bestBatch = bb.checked; paint(); };

    part("engines").innerHTML = engines.map((e, i) => `
      <label class="bx-eng">
        <input type="checkbox" data-eng="${escapeAttr(e)}" ${state.visibleEngines.has(e) ? "checked" : ""} />
        <i style="background:${engineColor(e, i)}"></i>
        <span>${escapeHtml(e)}</span>
      </label>`).join("");
    part("engines").querySelectorAll("input[data-eng]").forEach(chk => {
      chk.onchange = () => {
        const e = chk.dataset.eng;
        if (chk.checked) state.visibleEngines.add(e);
        else if (state.visibleEngines.size > 1) state.visibleEngines.delete(e);
        else chk.checked = true;
        paint();
      };
    });
  }

  function barWidth(val, minV, maxV, invert = false) {
    if (!Number.isFinite(val) || val <= 0) return 4;
    let frac;
    if (state.scale === "log") {
      const lo = Math.log10(Math.max(0.5, minV));
      const hi = Math.log10(Math.max(minV * 1.2, maxV));
      frac = (Math.log10(Math.max(0.5, val)) - lo) / Math.max(0.4, hi - lo);
    } else {
      frac = val / Math.max(1, maxV);
    }
    return Math.max(4, Math.min(100, (invert ? (1 - frac) : frac) * 100));
  }

  function renderExplorerStage(list, parsed) {
    if (!list.length) {
      part("stage").innerHTML = `<p class="now-quiet">No benchmarks match current filters.</p>`;
      return;
    }
    if (!list.some(f => f.id === state.selectedFamilyId)) state.selectedFamilyId = list[0].id;

    const allVis = list.flatMap(f => f.items.filter(x => state.visibleEngines.has(x.engine)));
    const minNs = Math.min(...allVis.map(x => x.stats.median), 10);
    const maxNs = Math.max(...allVis.map(x => x.stats.median), 100);
    const maxRate = Math.max(1, ...allVis.map(x => x.throughput || 0));
    const maxLogRatio = Math.max(0.6, ...list.filter(f => f.ratio).map(f => Math.log10(Math.max(f.ratio, 1 / f.ratio))));

    const cardsHtml = list.map(f => {
      const active = f.id === state.selectedFamilyId;
      const items = f.items.filter(x => state.visibleEngines.has(x.engine)).slice().sort((a, b) => a.stats.median - b.stats.median);
      const isLoss = f.ratio != null && f.ratio < 1.0;
      const badgeTone = f.ratio == null ? "single" : isLoss ? "bad" : f.ratio < 1.15 ? "warn" : "good";
      const ratioBadge = f.ratio != null
        ? `<span class="bx-badge ${badgeTone}">${escapeHtml(fmtMult(f.ratio))} vs ${escapeHtml(f.peerItem.engine)}</span>`
        : `<span class="bx-badge single">${items.length} eng</span>`;

      let barsHtml = "";
      if (state.mode === "speedup" && f.ratio != null) {
        const mag = f.ratio >= 1 ? f.ratio : 1 / f.ratio;
        const w = Math.max(8, Math.min(100, (Math.log10(Math.max(1.05, mag)) / maxLogRatio) * 100));
        barsHtml = `
          <div class="bx-speed-bar">
            <span class="bx-lbl-sm${isLoss ? " bad" : ""}">${escapeHtml(f.primaryItem.engine)}: ${fmtNs(f.primaryItem.stats.median)}</span>
            <div class="bx-track"><div class="bx-fill ${badgeTone}" style="width:${w.toFixed(1)}%"></div></div>
            <span class="bx-lbl-sm">${escapeHtml(f.peerItem.engine)}: ${fmtNs(f.peerItem.stats.median)}</span>
          </div>`;
      } else {
        barsHtml = items.map((it, idx) => {
          const val = state.mode === "throughput" ? (it.throughput || 0) : it.stats.median;
          const w = state.mode === "throughput"
            ? (it.throughput ? barWidth(it.throughput, 1, maxRate) : 0)
            : barWidth(it.stats.median, minNs, maxNs);
          const col = engineColor(it.engine, idx);
          const text = state.mode === "throughput" ? (it.throughputText || fmtNs(it.stats.median)) : fmtNs(it.stats.median);
          return `<div class="bx-erow">
            <span class="bx-ename">${escapeHtml(it.engine)}</span>
            <div class="bx-track"><div class="bx-fill" style="width:${w.toFixed(1)}%;background:${col}"></div></div>
            <span class="bx-eval">${escapeHtml(text)}</span>
          </div>`;
        }).join("");
      }

      return `<div class="bx-card${active ? " on" : ""}${isLoss ? " is-loss" : ""}" data-fid="${escapeAttr(f.id)}">
        <div class="bx-card-head">
          <div>
            <strong class="${isLoss ? "bad" : ""}">${escapeHtml(f.suiteLabel ? f.id : f.shortFamily)}</strong>
            ${f.params.length ? `<small class="bx-params">${escapeHtml(f.params.join(" · "))}</small>` : ""}
          </div>
          <div class="bx-card-meta">
            ${f.bestRate ? `<span class="bx-tput">${escapeHtml(f.items.find(x => x.throughput === f.bestRate)?.throughputText || "")}</span>` : ""}
            ${ratioBadge}
            <button type="button" class="mini-btn bx-run-one" data-runfam="${escapeAttr(f.shortFamily)}" title="Run this family live">▶</button>
          </div>
        </div>
        <div class="bx-bars">${barsHtml}</div>
      </div>`;
    }).join("");

    const sel = list.find(f => f.id === state.selectedFamilyId) || list[0];
    const inspHtml = renderInspector(sel);

    part("stage").innerHTML = `<div class="bx-layout">
      <div class="bx-list">
        <div class="bx-list-meta">${list.length} / ${parsed.families.length} families · ↑↓ to navigate</div>
        ${cardsHtml}
      </div>
      <aside class="bx-insp">${inspHtml}</aside>
    </div>`;

    part("stage").querySelectorAll(".bx-card[data-fid]").forEach(el => {
      el.addEventListener("click", ev => {
        if (ev.target && ev.target.closest && ev.target.closest(".bx-run-one")) return;
        state.selectedFamilyId = el.dataset.fid;
        paint();
      });
    });
    part("stage").querySelectorAll(".bx-run-one[data-runfam]").forEach(btn => {
      btn.addEventListener("click", ev => {
        ev.stopPropagation();
        launchFilter(btn.dataset.runfam);
      });
    });
    part("stage").querySelectorAll("[data-ptab]").forEach(b => {
      b.onclick = () => { state.plotTab = b.dataset.ptab; paint(); };
    });
    const jumpOut = part("stage").querySelector("#bx-jump-out");
    if (jumpOut && callbacks.onOpenOutput) jumpOut.onclick = () => callbacks.onOpenOutput(sel.shortFamily);
    const jumpSig = part("stage").querySelector("#bx-jump-sig");
    if (jumpSig && callbacks.onOpenSignals) jumpSig.onclick = () => callbacks.onOpenSignals(sel.shortFamily);
    const runFamBtn = part("stage").querySelector("#bx-insp-run");
    if (runFamBtn) runFamBtn.onclick = () => launchFilter(sel.shortFamily);

    bindTooltip(part("stage"));
  }

  function renderInspector(fam) {
    const items = fam.items.slice().sort((a, b) => a.stats.median - b.stats.median);
    const baseMed = fam.primaryItem ? fam.primaryItem.stats.median : items[0].stats.median;
    const fastestOther = items.find(x => fam.primaryItem && x !== fam.primaryItem && x.stats.median < baseMed);
    const isFamLoss = (fam.ratio != null && fam.ratio < 1.0) || Boolean(fastestOther);
    const rows = items.map((it, i) => {
      const ratio = baseMed > 0 ? it.stats.median / baseMed : 1;
      const isPrimaryLossRow = it === fam.primaryItem && Boolean(fastestOther);
      const ratioStr = it === fam.primaryItem
        ? (fastestOther ? `base (${(baseMed / fastestOther.stats.median).toFixed(2)}× slower vs ${fastestOther.engine})` : "base")
        : (ratio >= 1 ? ratio.toFixed(2) + "× slower" : (1 / ratio).toFixed(2) + "× faster");
      return `<tr class="${isPrimaryLossRow ? "bad" : ""}">
        <td><span class="bx-dot" style="background:${engineColor(it.engine, i)}"></span><code>${escapeHtml(it.engine)}</code></td>
        <td class="num"><strong>${fmtNs(it.stats.median)}</strong></td>
        <td class="num">${fmtNs(it.stats.iqr)}</td>
        <td class="num">${it.stats.cvPct.toFixed(1)}%</td>
        <td class="num">${escapeHtml(it.throughputText || "—")}</td>
        <td class="num${isPrimaryLossRow ? " bad" : ""}">${escapeHtml(ratioStr)}</td>
      </tr>`;
    }).join("");

    const contractPairs = Object.entries(items[0].contract || {})
      .filter(([k]) => !k.startsWith("tag.") && k !== "worker_environment")
      .sort(([a], [b]) => a.localeCompare(b));

    return `<div class="bx-insp-head">
        <div>
          <div class="traces-hint">${escapeHtml(fam.rawFamilyId)}</div>
          <h3 class="${isFamLoss ? "bad" : ""}">${escapeHtml(fam.shortFamily)}</h3>
        </div>
        <div class="bx-insp-actions">
          <button type="button" class="mini-btn primary" id="bx-insp-run">▶ Run</button>
          ${callbacks.onOpenSignals ? `<button type="button" class="mini-btn" id="bx-jump-sig">OTLP</button>` : ""}
          ${callbacks.onOpenOutput ? `<button type="button" class="mini-btn" id="bx-jump-out">Log</button>` : ""}
        </div>
      </div>
      <div class="viz-picker">
        <button type="button" class="${state.plotTab === "box" ? "on" : ""}" data-ptab="box">Box &amp; Samples</button>
        <button type="button" class="${state.plotTab === "trace" ? "on" : ""}" data-ptab="trace">Sequence Trace</button>
      </div>
      <div class="bx-plot-wrap">
        ${state.plotTab === "box"
          ? renderBoxSamplesSvg(fam, state.visibleEngines, state.scale)
          : renderSequenceSvg(fam, state.visibleEngines, state.scale)}
      </div>
      <div class="bx-table-wrap"><table class="bx-table">
        <thead><tr><th>Engine</th><th class="num">Median</th><th class="num">IQR</th><th class="num">CV%</th><th class="num">Rate</th><th class="num">Rel</th></tr></thead>
        <tbody>${rows}</tbody>
      </table></div>
      ${contractPairs.length ? `<details class="bx-contract"><summary>Contract (${contractPairs.length} fields)</summary>
        <dl>${contractPairs.map(([k, v]) => `<dt>${escapeHtml(k)}</dt><dd><code>${escapeHtml(v)}</code></dd>`).join("")}</dl></details>` : ""}`;
  }

  function renderDistributionsStage(list) {
    if (!list.length) {
      part("stage").innerHTML = `<p class="now-quiet">No families match current filters.</p>`;
      return;
    }
    const cards = list.slice(0, 60).map(f => `
      <div class="bench-chart-card">
        <div class="bx-card-head">
          <h4 class="${f.ratio != null && f.ratio < 1.0 ? "bad" : ""}">${escapeHtml(f.id)}</h4>
          ${f.ratio != null ? `<span class="bx-badge ${f.ratio < 1.0 ? "bad" : f.ratio < 1.15 ? "warn" : "good"}">${escapeHtml(fmtMult(f.ratio))}</span>` : ""}
        </div>
        ${renderBoxSamplesSvg(f, state.visibleEngines, state.scale)}
      </div>`).join("");
    part("stage").innerHTML = `<div class="bx-dist-grid">${cards}</div>`;
    bindTooltip(part("stage"));
  }

  function renderTableStage(list) {
    const flat = list.flatMap(f => f.items
      .filter(x => state.visibleEngines.has(x.engine))
      .map(it => ({
        fam: f,
        it,
        ratio: f.peerItem && it.stats.median > 0 ? f.peerItem.stats.median / it.stats.median : null,
      })));
    const { col, asc } = state.tableSort;
    flat.sort((a, b) => {
      const val = r => {
        if (col === "case") return r.it.id;
        if (col === "engine") return r.it.engine;
        if (col === "median") return r.it.stats.median;
        if (col === "p95") return r.it.stats.p95;
        if (col === "cv") return r.it.stats.cvPct;
        if (col === "tput") return r.it.throughput || 0;
        if (col === "ratio") return r.ratio ?? -1;
        return r.it.stats.median;
      };
      const av = val(a), bv = val(b);
      const cmp = typeof av === "string" ? av.localeCompare(bv) : av - bv;
      return asc ? cmp : -cmp;
    });

    const cols = [
      ["case", "Case"],
      ["engine", "Engine"],
      ["median", "Median"],
      ["p95", "P95"],
      ["cv", "CV%"],
      ["tput", "Throughput"],
      ["ratio", "vs Peer"],
    ];
    const ths = cols.map(([id, lbl]) =>
      `<th class="${id === "case" || id === "engine" ? "" : "num"}"><button type="button" class="bx-th-sort" data-col="${id}">${lbl}${col === id ? (asc ? " ▲" : " ▼") : ""}</button></th>`
    ).join("");
    const rows = flat.map(({ fam, it, ratio }) => {
      const isLossRow = ratio != null && ratio < 1.0 && it === fam.primaryItem;
      const ratioTone = ratio == null ? "" : ratio < 1.0 ? "bad" : ratio >= 1.15 ? "good" : "warn";
      return `<tr class="${isLossRow ? "bad" : ""}">
      <td><code>${escapeHtml(it.id)}</code></td>
      <td><span class="bx-dot" style="background:${engineColor(it.engine)}"></span>${escapeHtml(it.engine)}</td>
      <td class="num">${fmtNs(it.stats.median)}</td>
      <td class="num">${fmtNs(it.stats.p95)}</td>
      <td class="num">${it.stats.cvPct.toFixed(1)}%</td>
      <td class="num">${escapeHtml(it.throughputText || "—")}</td>
      <td class="num ${ratioTone}">${ratio != null ? escapeHtml(fmtMult(ratio)) : "—"}</td>
    </tr>`;
    }).join("");

    part("stage").innerHTML = `<div class="bx-table-tools">
        <span>${flat.length} rows</span>
        <button type="button" class="mini-btn" id="bx-csv">⬇ Export CSV</button>
      </div>
      <div class="table-scroll"><table class="traces-table"><thead><tr>${ths}<th>Samples</th></tr></thead><tbody>
        ${rows || `<tr><td colspan="8">No rows match filter.</td></tr>`}
      </tbody></table></div>`;

    part("stage").querySelectorAll(".bx-th-sort").forEach(b => {
      b.onclick = () => {
        if (state.tableSort.col === b.dataset.col) state.tableSort.asc = !state.tableSort.asc;
        else state.tableSort = { col: b.dataset.col, asc: true };
        renderTableStage(list);
      };
    });
    const csvBtn = part("stage").querySelector("#bx-csv");
    if (csvBtn) {
      csvBtn.onclick = () => {
        const hdr = "case,engine,median_ns,mean_ns,p95_ns,cv_pct,throughput,samples,procs\n";
        const body = flat.map(({ it }) =>
          `"${it.id.replace(/"/g, '""')}",${it.engine},${it.stats.median},${it.stats.mean},${it.stats.p95},${it.stats.cvPct.toFixed(2)},${it.throughput || ""},${it.stats.count},${it.processes}`
        ).join("\n");
        const blob = new Blob([hdr + body], { type: "text/csv" });
        const a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = `bench-${(state.runMeta && state.runMeta.run_id) || "export"}.csv`;
        a.click();
      };
    }
  }

  function renderReportStage() {
    const meta = state.runMeta || {};
    if (!meta.has_report_html) {
      part("stage").innerHTML = `<p class="now-quiet">Report HTML will appear once the run finishes.</p>`;
      return;
    }
    part("stage").innerHTML = `<iframe class="bx-report-frame" src="${escapeAttr(meta.files + "report.html")}" title="Offline bench report"></iframe>`;
    const frame = part("stage").querySelector("iframe");
    frame.addEventListener("load", () => {
      try {
        const h = frame.contentWindow.document.documentElement.scrollHeight;
        if (h > 400) frame.style.height = (h + 32) + "px";
      } catch {}
    });
  }

  function paint() {
    renderTopBar();
    renderSubtabs();
    if (state.subview === "report") {
      part("kpis").hidden = true;
      part("controls").hidden = true;
      renderReportStage();
      return;
    }
    const parsed = activeData();
    if (!parsed || !parsed.cases.length) {
      part("kpis").hidden = true;
      part("controls").hidden = true;
      part("stage").innerHTML = `<p class="now-quiet">${state.matrixLoading ? "Loading all finished bench suites…" : "Waiting for first completed benchmark case…"}</p>`;
      return;
    }
    part("kpis").hidden = false;
    part("controls").hidden = false;
    renderControls(parsed, parsed.families);
    const decorated = decorateFamilies(parsed);
    renderKpis(decorated, parsed);
    const filtered = filterAndSort(decorated);

    if (state.subview === "distributions") renderDistributionsStage(filtered);
    else if (state.subview === "table") renderTableStage(filtered);
    else renderExplorerStage(filtered, parsed);
  }

  const searchInput = part("search");
  searchInput.addEventListener("input", () => {
    state.query = searchInput.value;
    const parsed = activeData();
    if (!parsed) return;
    const decorated = decorateFamilies(parsed);
    const filtered = filterAndSort(decorated);
    if (state.subview === "distributions") renderDistributionsStage(filtered);
    else if (state.subview === "table") renderTableStage(filtered);
    else renderExplorerStage(filtered, parsed);
  });

  function onKey(ev) {
    if (!host.isConnected || host.closest(".panel[hidden]")) return;
    const active = document.activeElement;
    const typing = active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA" || active.tagName === "SELECT");
    if (ev.key === "/" && !typing) {
      ev.preventDefault();
      searchInput.focus();
      return;
    }
    if (typing || state.subview !== "explorer") return;
    const parsed = activeData();
    if (!parsed) return;
    const list = filterAndSort(decorateFamilies(parsed));
    if (!list.length) return;
    const idx = list.findIndex(f => f.id === state.selectedFamilyId);
    if (ev.key === "ArrowDown" || ev.key === "j") {
      ev.preventDefault();
      state.selectedFamilyId = list[(idx + 1) % list.length].id;
      renderExplorerStage(list, parsed);
    } else if (ev.key === "ArrowUp" || ev.key === "k") {
      ev.preventDefault();
      state.selectedFamilyId = list[(idx - 1 + list.length) % list.length].id;
      renderExplorerStage(list, parsed);
    }
  }
  window.addEventListener("keydown", onKey);

  return {
    destroy() { window.removeEventListener("keydown", onKey); },
    focusCase(term) {
      state.query = term || "";
      searchInput.value = state.query;
      state.subview = "explorer";
      paint();
    },
    update({ runMeta, runData, live }) {
      state.runMeta = runMeta;
      state.live = Boolean(live);
      const sig = `${runMeta && runMeta.run_id}|${runMeta && runMeta.state}|${runMeta && runMeta.updated_at_ms}|${runData ? (runData.observations || []).length : 0}`;
      if (sig === state.sig) return;
      state.sig = sig;
      if (runData) state.parsed = parseRunJson(runData);
      paint();
    },
  };
}
