// The page frame: routes, top nav, sub-nav, badges. Knows sections only through the registry,
// so adding a section means registering it — nothing here changes.
//
// A section is `{ cats: string[], show(route, changed), hide?(), tick?(n) }`.

import { $, escapeAttr, escapeHtml } from "./dom.js";

/** Four questions, not eight packages: what is happening, which run, which signal, is the hub healthy. */
const GROUPS = [
  { id: "now", label: "Now", cats: ["now"] },
  { id: "runs", label: "Runs", cats: ["runs", "tests", "bench"],
    sub: [["runs", "All"], ["tests", "Tests"], ["bench", "Bench"]] },
  { id: "signals", label: "Signals", cats: ["issues", "logs", "metrics"],
    sub: [["issues", "Issues"], ["logs", "Logs"], ["metrics", "Metrics"]] },
  { id: "health", label: "Health", cats: ["system"] },
];

/** Old routes keep working: package tabs folded into Health, overview became Now. */
const ALIASES = {
  launches: "tests", overview: "now", unit: "tests", mon: "system", otel: "system", err: "system",
  collector: "system", apis: "system", health: "system", signals: "issues",
};

const LEDE = {
  now: "What is running, what just broke, and what changed since the last run.",
  runs: "Every test and bench run, newest first — live ones on top.",
  tests: "Per-test results from cargo airbug test: failures first, steps, diffs, history.",
  bench: "Live and finished bench runs (GUID sessions).",
  issues: "Issue inbox from airbug-err events.",
  logs: "OTLP log tail and filters.",
  metrics: "OTLP metrics visualizations.",
  system: "Is the hub wired up: collector, packages, and the local API catalog.",
};

const LABEL = Object.fromEntries(GROUPS.flatMap(g => (g.sub || [[g.cats[0], g.label]])));
const CATS = GROUPS.flatMap(g => g.cats);
const groupOf = cat => GROUPS.find(g => g.id === cat || g.cats.includes(cat)) || GROUPS[0];

const sections = new Map();
let current = { cat: "", runId: null };
let traces = { up: false, href: "" };

export function register(section) {
  for (const cat of section.cats) sections.set(cat, section);
}

export const currentRoute = () => current;

export function parseRoute() {
  const parts = (location.hash || "").replace(/^#\/?/, "").trim().split("/").filter(Boolean);
  let id = parts[0] || "now";
  if (ALIASES[id]) id = ALIASES[id];
  const cat = CATS.includes(id) ? id : "now";
  const runId = parts[1] ? decodeURIComponent(parts[1]) : null;
  return { cat, runId };
}

function renderNav() {
  $("cats").innerHTML = GROUPS.map(g =>
    `<a href="#/${g.cats[0]}" data-group="${g.id}">${escapeHtml(g.label)}<span class="cat-badge" data-badge="${g.id}" hidden></span></a>`
  ).join("");
}

function renderSubnav(cat) {
  const el = $("subnav");
  const group = groupOf(cat);
  el.hidden = !group.sub;
  if (!group.sub) {
    el.innerHTML = "";
    return;
  }
  const items = group.sub.map(([id, label]) =>
    `<a href="#/${id}" class="${id === cat ? "on" : ""}">${escapeHtml(label)}</a>`);
  if (group.id === "signals") {
    items.push(traces.up
      ? `<a href="${escapeAttr(traces.href)}" target="_blank" rel="noopener">Traces ↗</a>`
      : `<span class="off" title="Traces render only in Jaeger, which the Docker collector brings up">Traces — Jaeger not running</span>`);
  }
  el.innerHTML = items.join("");
}

/** A badge on the top-nav group that owns `catOrGroup`; empty text hides it. */
export function setBadge(catOrGroup, text) {
  const el = $("cats").querySelector(`[data-badge="${groupOf(catOrGroup).id}"]`);
  if (!el) return;
  el.hidden = !text;
  el.textContent = text || "";
}

export function setTraces(up, href) {
  traces = { up, href };
  renderSubnav(current.cat);
}

export function setTitle(...parts) {
  document.title = ["airbug hub", ...parts.filter(Boolean)].join(" · ");
}

export function showRoute() {
  const next = parseRoute();
  const changed = next.cat !== current.cat || next.runId !== current.runId;
  const prev = sections.get(current.cat);
  const section = sections.get(next.cat);
  if (prev && prev !== section && prev.hide) prev.hide();
  current = next;

  $("lede").textContent = LEDE[next.cat] || LEDE.now;
  const group = groupOf(next.cat).id;
  $("cats").querySelectorAll("a").forEach(a => a.classList.toggle("active", a.getAttribute("data-group") === group));
  renderSubnav(next.cat);
  document.querySelectorAll(".panel").forEach(p => { p.hidden = p.getAttribute("data-cat") !== next.cat; });
  setTitle(LABEL[next.cat], next.runId && next.runId.slice(0, 8));

  if (section) Promise.resolve(section.show(next, changed)).catch(err => console.error(err));
}

/** Ticks the visible section every 1.5 s; hidden tabs skip. */
export function start() {
  renderNav();
  window.addEventListener("hashchange", showRoute);
  if (!location.hash) location.hash = "#/now";
  else showRoute();
  let n = 0;
  setInterval(() => {
    n++;
    if (document.hidden) return;
    const section = sections.get(current.cat);
    if (section && section.tick) Promise.resolve(section.tick(n)).catch(() => {});
  }, 1500);
}

/** Background feeds keep badges and caches fresh regardless of the visible tab. */
export function every(ms, fn) {
  const run = () => Promise.resolve(fn()).catch(() => {});
  run();
  setInterval(() => { if (!document.hidden) run(); }, ms);
}
