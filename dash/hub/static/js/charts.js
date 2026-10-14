// Pure SVG renderers for metric boards. Input: series `{ name, points: [{ t, v }] }`; output: markup.

import { escapeHtml } from "./dom.js";
import { fmtValue } from "./format.js";

export function seriesLabel(s) {
  const short = s.attrs ? s.name + " · " + s.attrs : s.name;
  return short.length > 48 ? short.slice(0, 46) + "…" : short;
}

export function lastPoint(s) {
  return s.points && s.points.length ? s.points[s.points.length - 1].v : NaN;
}

export function sparkSvg(points, gradId) {
  const w = 320, h = 120, pad = 6;
  if (!points || points.length < 2) {
    return emptySvg(w, h, "need ≥2 samples");
  }
  const ts = points.map(p => p.t);
  const vs = points.map(p => p.v);
  const t0 = Math.min(...ts), t1 = Math.max(...ts);
  let v0 = Math.min(...vs), v1 = Math.max(...vs);
  if (v0 === v1) { v0 -= 1; v1 += 1; }
  const x = t => pad + ((t - t0) / (t1 - t0 || 1)) * (w - pad * 2);
  const y = v => h - pad - ((v - v0) / (v1 - v0 || 1)) * (h - pad * 2);
  const coords = points.map(p => `${x(p.t).toFixed(1)},${y(p.v).toFixed(1)}`);
  const line = coords.join(" ");
  const area = `${pad},${h - pad} ${line} ${w - pad},${h - pad}`;
  const midY = y((v0 + v1) / 2).toFixed(1);
  const gid = gradId || "metricFill";
  return `<svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="none" aria-hidden="true">
    <defs>
      <linearGradient id="${gid}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="#c8f54a"/>
        <stop offset="100%" stop-color="#c8f54a" stop-opacity="0"/>
      </linearGradient>
    </defs>
    <line class="grid-line" x1="${pad}" x2="${w - pad}" y1="${midY}" y2="${midY}"/>
    <polygon points="${area}" fill="url(#${gid})" style="opacity:0.35"/>
    <polyline class="line" points="${line}"/>
  </svg>`;
}

export function emptySvg(w, h, msg) {
  return `<svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="none" aria-hidden="true">
    <text x="12" y="${(h / 2 + 4).toFixed(0)}" fill="#7f9a8b" font-size="11">${escapeHtml(msg)}</text>
  </svg>`;
}

export function gaugeSvg(value, name, unit) {
  const w = 320, h = 120;
  const util = /utilization|ratio|percent/i.test(name) || unit === "1" || unit === "%";
  let max = util ? 1 : (Number.isFinite(value) && value > 0 ? value * 1.25 : 1);
  if (unit === "%") max = 100;
  const v = Number.isFinite(value) ? Math.max(0, Math.min(value, max)) : 0;
  const pct = max > 0 ? v / max : 0;
  const cx = 160, cy = 95, r = 70;
  const start = Math.PI;
  const end = Math.PI + Math.PI * pct;
  const arc = (a0, a1) => {
    const x0 = cx + r * Math.cos(a0), y0 = cy + r * Math.sin(a0);
    const x1 = cx + r * Math.cos(a1), y1 = cy + r * Math.sin(a1);
    const large = a1 - a0 > Math.PI ? 1 : 0;
    return `M ${x0} ${y0} A ${r} ${r} 0 ${large} 1 ${x1} ${y1}`;
  };
  return `<svg viewBox="0 0 ${w} ${h}" aria-hidden="true">
    <path d="${arc(Math.PI, 2 * Math.PI)}" fill="none" stroke="#ffffff18" stroke-width="12" stroke-linecap="round"/>
    <path d="${arc(start, end)}" fill="none" stroke="#c8f54a" stroke-width="12" stroke-linecap="round"/>
    <text x="${cx}" y="72" text-anchor="middle" fill="#c8f54a" font-size="18" font-family="IBM Plex Mono, monospace">${escapeHtml(fmtValue(value, unit || ""))}</text>
  </svg>`;
}

export function barSvg(seriesList) {
  const w = 640, h = 176, pad = 16;
  if (!seriesList.length) return emptySvg(w, h, "select series");
  const vals = seriesList.map(s => lastPoint(s));
  let v0 = 0, v1 = Math.max(...vals.filter(Number.isFinite), 1);
  if (v1 <= 0) v1 = 1;
  const bw = (w - pad * 2) / seriesList.length;
  const bars = seriesList.map((s, i) => {
    const v = lastPoint(s);
    const vh = Number.isFinite(v) ? ((v - v0) / (v1 - v0)) * (h - pad * 2) : 0;
    const x = pad + i * bw + bw * 0.15;
    const y = h - pad - vh;
    return `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${(bw * 0.7).toFixed(1)}" height="${Math.max(vh, 0).toFixed(1)}" fill="#c8f54a" opacity="0.85"/>
      <text x="${(x + bw * 0.35).toFixed(1)}" y="${h - 4}" text-anchor="middle" fill="#7f9a8b" font-size="8">${escapeHtml((s.name || "").split(".").pop() || s.name)}</text>`;
  }).join("");
  return `<svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="xMidYMid meet" aria-hidden="true">${bars}</svg>`;
}

export function histSvg(hist) {
  const w = 640, h = 176, pad = 16;
  const counts = (hist && hist.counts) || [];
  if (!counts.length) return emptySvg(w, h, "no histogram");
  const maxC = Math.max(...counts, 1);
  const bw = (w - pad * 2) / counts.length;
  const bars = counts.map((c, i) => {
    const vh = (c / maxC) * (h - pad * 2);
    const x = pad + i * bw + 1;
    const y = h - pad - vh;
    return `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${Math.max(bw - 2, 1).toFixed(1)}" height="${vh.toFixed(1)}" fill="#c8f54a" opacity="0.8"/>`;
  }).join("");
  const label = hist.min != null
    ? `${fmtValue(hist.min, "")} … ${fmtValue(hist.max, "")}`
    : "";
  return `<svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
    ${bars}
    <text x="${pad}" y="14" fill="#7f9a8b" font-size="10">${escapeHtml(label)}</text>
  </svg>`;
}

function heatColor(t) {
  const c = Math.max(0, Math.min(1, t));
  const r = Math.round(15 + c * 200);
  const g = Math.round(40 + c * 200);
  const b = Math.round(30 + (1 - c) * 40);
  return `rgb(${r},${g},${b})`;
}

export function heatmapSvg(seriesList) {
  const w = 640, h = 200, padL = 90, padR = 8, padT = 8, padB = 20;
  if (!seriesList.length) return emptySvg(w, h, "select series");
  const cols = 24;
  let tMin = Infinity, tMax = -Infinity, vMin = Infinity, vMax = -Infinity;
  for (const s of seriesList) {
    for (const p of s.points || []) {
      tMin = Math.min(tMin, p.t);
      tMax = Math.max(tMax, p.t);
      vMin = Math.min(vMin, p.v);
      vMax = Math.max(vMax, p.v);
    }
  }
  if (!Number.isFinite(tMin) || tMax <= tMin) return emptySvg(w, h, "need ≥2 samples");
  if (vMax === vMin) { vMin -= 1; vMax += 1; }
  const cellW = (w - padL - padR) / cols;
  const cellH = (h - padT - padB) / seriesList.length;
  const cells = [];
  seriesList.forEach((s, row) => {
    const buckets = new Array(cols).fill(null);
    for (const p of s.points || []) {
      let idx = Math.floor(((p.t - tMin) / (tMax - tMin)) * cols);
      if (idx >= cols) idx = cols - 1;
      if (idx < 0) idx = 0;
      buckets[idx] = p.v;
    }
    buckets.forEach((v, col) => {
      if (v == null) return;
      const t = (v - vMin) / (vMax - vMin);
      const x = padL + col * cellW;
      const y = padT + row * cellH;
      cells.push(`<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${Math.max(cellW - 0.5, 1).toFixed(1)}" height="${Math.max(cellH - 0.5, 1).toFixed(1)}" fill="${heatColor(t)}"/>`);
    });
    const label = (s.name || "").length > 14 ? s.name.slice(0, 12) + "…" : s.name;
    cells.push(`<text x="4" y="${(padT + row * cellH + cellH * 0.65).toFixed(1)}" fill="#9ab5a6" font-size="9">${escapeHtml(label)}</text>`);
  });
  return `<svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="xMidYMid meet" aria-hidden="true">${cells.join("")}</svg>`;
}

export function pieSvg(seriesList) {
  const w = 320, h = 160, cx = 100, cy = 80, r = 58;
  const items = seriesList.map(s => ({ s, v: Math.abs(lastPoint(s)) })).filter(x => Number.isFinite(x.v) && x.v > 0);
  if (!items.length) return emptySvg(w, h, "no positive values");
  const total = items.reduce((a, x) => a + x.v, 0) || 1;
  let angle = -Math.PI / 2;
  const colors = ["#c8f54a", "#8ecfad", "#e0a35a", "#7ab8e0", "#e07a7a", "#b8a0e0"];
  const slices = items.map((item, i) => {
    const slice = (item.v / total) * Math.PI * 2;
    const a0 = angle;
    const a1 = angle + slice;
    angle = a1;
    const x0 = cx + r * Math.cos(a0), y0 = cy + r * Math.sin(a0);
    const x1 = cx + r * Math.cos(a1), y1 = cy + r * Math.sin(a1);
    const large = slice > Math.PI ? 1 : 0;
    const d = `M ${cx} ${cy} L ${x0} ${y0} A ${r} ${r} 0 ${large} 1 ${x1} ${y1} Z`;
    return `<path d="${d}" fill="${colors[i % colors.length]}" opacity="0.9"/>`;
  }).join("");
  const legend = items.slice(0, 6).map((item, i) => {
    const y = 18 + i * 16;
    const pct = ((item.v / total) * 100).toFixed(0);
    const name = (item.s.name || "").split(".").pop() || item.s.name;
    return `<rect x="175" y="${y - 8}" width="8" height="8" fill="${colors[i % colors.length]}"/>
      <text x="188" y="${y}" fill="#d7e6dc" font-size="9">${escapeHtml(name)} ${pct}%</text>`;
  }).join("");
  return `<svg viewBox="0 0 ${w} ${h}" aria-hidden="true">${slices}${legend}</svg>`;
}

