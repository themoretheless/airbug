// Runs › All: one timeline for tests and bench.

import { $, bindCopy, renderToggles } from "../dom.js";
import { emptyStart, fetchTimeline, runRow } from "../runs.js";

const KINDS = [["", "all"], ["test", "tests"], ["bench", "bench"]];
let kind = "";

async function refresh() {
  renderToggles($("runs-kinds"), KINDS, id => id === kind, id => { kind = id; refresh().catch(() => {}); }, "data-kind");
  const data = await fetchTimeline(kind, 100);
  const runs = data.runs || [];
  $("runs-meta").textContent = `${runs.length} run(s)` + (data.live ? ` · ${data.live} live` : "");
  const list = $("runs-list");
  list.innerHTML = runs.length ? runs.map(runRow).join("") : emptyStart("No runs yet.");
  bindCopy(list);
}

export default {
  cats: ["runs"],
  show: () => refresh().catch(err => { $("runs-meta").textContent = String(err); }),
  tick: n => (n % 2 === 0 ? refresh() : null),
};
