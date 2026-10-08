// Pure formatting: numbers, durations, times, log severities.

export function fmtValue(v, unit) {
  const n = Number(v);
  let body;
  if (!Number.isFinite(n)) body = "—";
  else if (Math.abs(n) >= 1e9) body = (n / 1e9).toFixed(2) + "G";
  else if (Math.abs(n) >= 1e6) body = (n / 1e6).toFixed(2) + "M";
  else if (Math.abs(n) >= 1e3) body = (n / 1e3).toFixed(2) + "k";
  else if (Math.abs(n) >= 10) body = n.toFixed(2);
  else body = n.toPrecision(4);
  return unit ? body + " " + unit : body;
}

export function fmtPct(v) {
  if (v == null || !Number.isFinite(v)) return "—";
  return (v > 0 ? "+" : "") + v.toFixed(Math.abs(v) >= 10 ? 0 : 1) + "%";
}

export function fmtNs(ns) {
  if (!Number.isFinite(ns)) return "—";
  if (ns >= 1e9) return (ns / 1e9).toFixed(2) + " s";
  if (ns >= 1e6) return (ns / 1e6).toFixed(2) + " ms";
  if (ns >= 1e3) return (ns / 1e3).toFixed(1) + " µs";
  return ns.toFixed(0) + " ns";
}

export function median(nums) {
  if (!nums.length) return NaN;
  const s = nums.slice().sort((a, b) => a - b);
  const m = Math.floor(s.length / 2);
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
}

export function formatTime(entry) {
  if (entry.time_ms) {
    try {
      return new Date(entry.time_ms).toISOString().replace("T", " ").replace("Z", "");
    } catch {}
  }
  return entry.time || "—";
}

export const SEV_ORDER = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];

const SEV_MATCH = [
  ["FATAL", ["FATAL", "CRITICAL"]],
  ["ERROR", ["ERROR", "ERR"]],
  ["WARN", ["WARN"]],
  ["DEBUG", ["DEBUG"]],
  ["TRACE", ["TRACE"]],
];

/** Free-form severity text → one of SEV_ORDER. */
export function normalizeSev(sev) {
  const s = String(sev || "INFO").toUpperCase();
  const hit = SEV_MATCH.find(([, keys]) => keys.some(k => s.includes(k)));
  return hit ? hit[0] : "INFO";
}

export const sevClass = sev => normalizeSev(sev).toLowerCase();
