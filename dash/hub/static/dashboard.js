    const VIZ_TYPES = [
      { id: "timeseries", label: "Time series" },
      { id: "gauge", label: "Gauge" },
      { id: "bar", label: "Bar" },
      { id: "histogram", label: "Histogram" },
      { id: "heatmap", label: "Heatmap" },
      { id: "pie", label: "Pie" },
      { id: "table", label: "Table" },
    ];
    const SEV_ORDER = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
    const CATS = [
      { id: "now", label: "Now" },
      { id: "runs", label: "Runs" },
      { id: "tests", label: "Tests" },
      { id: "bench", label: "Bench" },
      { id: "issues", label: "Issues" },
      { id: "logs", label: "Logs" },
      { id: "metrics", label: "Metrics" },
      { id: "system", label: "System" },
    ];
    /** Old routes keep working: package tabs folded into System, overview became Now. */
    const ALIASES = {
      overview: "now", unit: "tests", mon: "system", otel: "system", err: "system",
      collector: "system", apis: "system",
    };
    const CAT_LEDE = {
      now: "What is running, what just broke, and what changed since the last run.",
      runs: "Every test and bench run, newest first — live ones on top.",
      tests: "Per-test results from cargo airbug test: failures first, steps, diffs, history.",
      bench: "Live and finished bench runs (GUID sessions).",
      issues: "Issue inbox from airbug-err events.",
      logs: "OTLP log tail and filters.",
      metrics: "OTLP metrics visualizations.",
      system: "Package status, collector, and the local API catalog.",
    };
    const START_TESTS = "cargo airbug test -- --workspace --exclude airbug-mon";
    const START_BENCH = "AIRBUG_HUB=http://127.0.0.1:8790 cargo airbug-bench run -p airbug-bench --bench workloads";

    const domainsEl = document.getElementById("domains");
    const catsEl = document.getElementById("cats");
    const ledeEl = document.getElementById("lede");
    const metaEl = document.getElementById("meta");
    const toastEl = document.getElementById("toast");
    const logsMetaEl = document.getElementById("logs-meta");
    const logsPanelEl = document.getElementById("logs-panel");
    const logsFilterEl = document.getElementById("logs-filter");
    const logsServiceEl = document.getElementById("logs-service");
    const sevChipsEl = document.getElementById("sev-chips");
    const sevBarEl = document.getElementById("sev-bar");
    const metricsMetaEl = document.getElementById("metrics-meta");
    const metricsPanelEl = document.getElementById("metrics-panel");
    const metricsBoardsEl = document.getElementById("metrics-boards");
    const metricsPickEl = document.getElementById("metrics-pick");
    const vizPickerEl = document.getElementById("viz-picker");
    const apisEl = document.getElementById("apis");
    const apisMetaEl = document.getElementById("apis-meta");
    const apiListEl = document.getElementById("api-list");
    const issuesMetaEl = document.getElementById("issues-meta");
    const issuesPanelEl = document.getElementById("issues-panel");
    const issueDetailEl = document.getElementById("issue-detail");
    const issueDetailTitleEl = document.getElementById("issue-detail-title");
    const issueDetailBodyEl = document.getElementById("issue-detail-body");
    const issueActionsEl = document.getElementById("issue-actions");
    let selectedIssueId = null;
    /** @type {Record<string, any>} */
    let lastDomains = {};
    let currentCat = "now";
    let currentRunId = null;
    let benchPollTimer = null;
    let lastHubId = "";

    /** @type {Set<string>} */
    let selectedSeries = new Set();
    let lastSeriesIds = [];
    let currentViz = "timeseries";
    /** @type {any} */
    let lastMetrics = null;
    /** @type {any} */
    let lastLogs = null;
    /** @type {Set<string>} */
    let selectedSeverities = new Set(SEV_ORDER);

    function parseRoute() {
      const raw = (location.hash || "").replace(/^#\/?/, "").trim();
      const parts = raw.split("/").filter(Boolean);
      let id = parts[0] || "now";
      if (ALIASES[id]) id = ALIASES[id];
      const cat = CATS.some(c => c.id === id) ? id : "now";
      const runId = (cat === "bench" || cat === "tests") && parts[1] ? decodeURIComponent(parts[1]) : null;
      return { cat, runId };
    }

    function showRoute() {
      const { cat, runId } = parseRoute();
      const changedRun = cat !== currentCat || runId !== currentRunId;
      currentCat = cat;
      currentRunId = runId;
      ledeEl.textContent = CAT_LEDE[cat] || CAT_LEDE.now;
      catsEl.querySelectorAll("a").forEach(a => {
        a.classList.toggle("active", a.getAttribute("data-cat") === cat);
      });
      document.querySelectorAll(".panel").forEach(p => {
        p.hidden = p.getAttribute("data-cat") !== cat;
      });
      if (cat === "bench") {
        refreshBench().catch(err => {
          document.getElementById("bench-meta").textContent = String(err);
        });
      } else if (benchPollTimer) {
        clearInterval(benchPollTimer);
        benchPollTimer = null;
      }
      if (cat === "tests") refreshTests(changedRun).catch(err => showTestsError(err));
      if (cat === "now") refreshNow().catch(() => {});
      if (cat === "runs") refreshRuns().catch(err => {
        document.getElementById("runs-meta").textContent = String(err);
      });
      document.title = "airbug hub · " + (CATS.find(c => c.id === cat)?.label || cat)
        + (runId ? " · " + runId.slice(0, 8) : "");
    }

    function renderCats() {
      catsEl.innerHTML = CATS.map(c =>
        `<a href="#/${c.id}" data-cat="${c.id}">${escapeHtml(c.label)}<span class="cat-badge" data-badge="${c.id}" hidden></span></a>`
      ).join("");
    }

    function setBadge(cat, text) {
      const el = catsEl.querySelector(`[data-badge="${cat}"]`);
      if (!el) return;
      el.hidden = !text;
      el.textContent = text || "";
    }

    // ---- runs: one timeline for tests + bench ------------------------------------------

    const TR = window.AirbugTestRun;
    let runsKind = "";
    let lastTimeline = null;

    async function fetchTimeline(kind, limit) {
      const q = new URLSearchParams({ limit: String(limit || 60) });
      if (kind) q.set("kind", kind);
      const res = await fetch("/api/v1/runs?" + q.toString(), { cache: "no-store" });
      if (!res.ok) throw new Error("GET /api/v1/runs → " + res.status);
      const data = await res.json();
      if (!kind) {
        lastTimeline = data;
        setBadge("runs", data.live ? String(data.live) + " live" : "");
      }
      return data;
    }

    function isLive(r) {
      return !r.stale && (r.state === "running" || r.state === "building");
    }

    function stateLabel(r) {
      return r.stale ? "stale" : r.state;
    }

    function runSummary(r) {
      if (r.kind === "test") {
        const t = r.totals || {};
        if (isLive(r)) {
          return `${t.completed || 0}/${t.total || "?"} · ${t.failed || 0} failed` +
            (r.activity ? ` · ${r.activity}` : "");
        }
        const parts = [`${t.passed || 0} passed`];
        if (t.failed) parts.push(`${t.failed} failed`);
        if (t.not_run) parts.push(`${t.not_run} not run`);
        if (t.ignored) parts.push(`${t.ignored} ignored`);
        return parts.join(" · ");
      }
      const p = r.progress || {};
      if (p.total) return `${p.completed || 0}/${p.total} processes${p.variant ? " · " + p.variant : ""}`;
      return r.href && r.href.startsWith("#") ? "bench session" : "bench artifacts";
    }

    function runLink(r, inner, cls) {
      const external = r.href && !r.href.startsWith("#");
      return `<a class="${cls || ""}" href="${escapeAttr(r.href || "#")}"${external ? ` target="_blank" rel="noopener"` : ""}>${inner}</a>`;
    }

    function runRow(r) {
      const live = isLive(r);
      const t = r.totals || {};
      const bar = r.kind === "test" && live && TR ? TR.progressBar(t, true) : "";
      const commit = r.git && r.git.short
        ? `<code>${escapeHtml(r.git.short)}</code>${r.git.dirty ? "<em>+dirty</em>" : ""}` : "";
      const when = TR ? TR.fmtAgo(r.started_at_ms || r.updated_at_ms) : "";
      const dur = r.duration_s != null && TR ? TR.fmtDuration(r.duration_s) : "";
      return runLink(r, `
        <span class="run-kind ${escapeAttr(r.kind)}">${escapeHtml(r.kind)}</span>
        <span class="atr-state ${escapeAttr(stateLabel(r))}">${escapeHtml(stateLabel(r))}</span>
        <span class="run-main">
          <strong>${escapeHtml(r.title || r.run_id)}</strong>
          <span class="run-sub">${commit} ${escapeHtml(runSummary(r))}</span>
          ${bar}
        </span>
        <span class="run-when">${escapeHtml(when)}${dur ? `<br/>${escapeHtml(dur)}` : ""}</span>`, "run-row" + (live ? " live" : ""));
    }

    async function refreshRuns() {
      const kinds = [["", "all"], ["test", "tests"], ["bench", "bench"]];
      const kindsEl = document.getElementById("runs-kinds");
      kindsEl.innerHTML = kinds.map(([id, label]) =>
        `<button type="button" class="${runsKind === id ? "on" : ""}" data-kind="${id}">${label}</button>`
      ).join("");
      kindsEl.querySelectorAll("button").forEach(btn => {
        btn.onclick = () => { runsKind = btn.getAttribute("data-kind"); refreshRuns().catch(() => {}); };
      });
      const data = await fetchTimeline(runsKind, 100);
      const runs = data.runs || [];
      document.getElementById("runs-meta").textContent =
        `${runs.length} run(s)` + (data.live ? ` · ${data.live} live` : "");
      const listEl = document.getElementById("runs-list");
      listEl.innerHTML = runs.length
        ? runs.map(runRow).join("")
        : emptyStart("No runs yet.");
      bindCopy(listEl);
    }

    function emptyStart(title) {
      return `<div class="now-empty">
        <p>${escapeHtml(title)} Start one — it appears here while it runs:</p>
        <ul class="actions">
          <li><button type="button" data-cmd="${escapeAttr(START_TESTS)}">copy</button> <code>${escapeHtml(START_TESTS)}</code></li>
          <li><button type="button" data-cmd="${escapeAttr(START_BENCH)}">copy</button> <code>${escapeHtml(START_BENCH)}</code></li>
        </ul>
      </div>`;
    }

    function bindCopy(el) {
      el.querySelectorAll("button[data-cmd]").forEach(btn => {
        btn.onclick = ev => { ev.preventDefault(); copy(btn.getAttribute("data-cmd")); };
      });
    }

    // ---- tests ---------------------------------------------------------------------------

    let testView = null;
    let testViewRun = null;
    let testsLive = false;
    // A rerun started from this page: its directory appears a moment after the POST returns.
    let pendingRerun = null;

    function onRerunStarted(started) {
      pendingRerun = started.run_id;
      location.hash = started.href || ("#/tests/" + started.run_id);
    }

    function showTestsError(err) {
      document.getElementById("tests-view").innerHTML =
        `<div class="now-empty"><p>${escapeHtml(String(err))}</p></div>`;
    }

    async function refreshTests(reset) {
      const viewEl = document.getElementById("tests-view");
      const id = currentRunId || "latest";
      const [detailRes, list] = await Promise.all([
        fetch("/api/v1/runs/" + encodeURIComponent(id), { cache: "no-store" }),
        fetchTimeline("test", 15).catch(() => ({ runs: [] })),
      ]);
      renderTestsPicker(list.runs || []);
      if (!detailRes.ok) {
        testView = null;
        testViewRun = null;
        testsLive = false;
        if (id === pendingRerun) {
          testsLive = true;
          viewEl.innerHTML = `<div class="now-empty"><p>Starting the rerun…</p></div>`;
          return;
        }
        const body = await detailRes.json().catch(() => ({}));
        viewEl.innerHTML = id === "latest"
          ? emptyStart("No test runs yet.")
          : `<div class="now-empty"><p>${escapeHtml(body.error || "run not found")}</p></div>`;
        bindCopy(viewEl);
        return;
      }
      const data = await detailRes.json();
      if (reset || !testView || testViewRun !== data.run_id) {
        viewEl.innerHTML = "";
        const host = document.createElement("div");
        viewEl.appendChild(host);
        testView = TR.createView(host, { rerun: onRerunStarted });
        testViewRun = data.run_id;
      }
      if (data.run_id === pendingRerun) pendingRerun = null;
      testView.update(data);
      const item = data.item || {};
      testsLive = isLive(item);
      document.title = "airbug hub · Tests · " + (item.state || "") + " · " + String(data.run_id).slice(0, 8);
    }

    function renderTestsPicker(runs) {
      const el = document.getElementById("tests-picker");
      if (!runs.length) {
        el.innerHTML = "";
        return;
      }
      const following = !currentRunId;
      const chips = runs.map(r => {
        const active = r.run_id === testViewRun && !following ? " on" : "";
        const t = r.totals || {};
        const label = (r.git && r.git.short) || r.run_id.slice(0, 8);
        const count = isLive(r) ? `${t.completed || 0}/${t.total || "?"}` : (t.failed ? `${t.failed}✗` : `${t.passed || 0}✓`);
        return `<a class="pick${active}" href="#/tests/${escapeAttr(r.run_id)}" title="${escapeAttr(r.title + " · " + (TR ? TR.fmtWhen(r.started_at_ms) : ""))}">
          <span class="atr-dot ${escapeAttr(isLive(r) ? "running" : r.state === "passed" ? "passed" : r.state === "failed" || r.state === "error" ? "failed" : "not_run")}"></span>
          ${escapeHtml(label)} <small>${escapeHtml(count)}</small></a>`;
      }).join("");
      el.innerHTML = `<a class="pick${following ? " on" : ""}" href="#/tests" title="always show the newest run">latest</a>${chips}`;
    }

    // ---- now -----------------------------------------------------------------------------

    async function refreshNow() {
      const [timeline, latest, issues, benchCmp] = await Promise.all([
        fetchTimeline("", 40),
        fetch("/api/v1/runs/latest?lite=1", { cache: "no-store" }).then(r => (r.ok ? r.json() : null)).catch(() => null),
        fetch("/api/v1/issues", { cache: "no-store" }).then(r => r.json()).catch(() => ({ issues: [] })),
        fetch("/api/v1/bench/compare", { cache: "no-store" }).then(r => (r.ok ? r.json() : null)).catch(() => null),
      ]);
      const runs = timeline.runs || [];
      const live = runs.filter(r => isLive(r) || (r.kind === "bench" && r.state === "running"));
      const liveEl = document.getElementById("now-live");
      liveEl.innerHTML = `<h2>Live</h2>` + (live.length
        ? `<div class="runs-list">${live.map(runRow).join("")}</div>`
        : (runs.length
          ? `<p class="now-quiet">Nothing running. Start a run and it shows up here within a second.</p>`
          : emptyStart("Nothing has run in this project yet.")));
      bindCopy(liveEl);

      const testsEl = document.getElementById("now-tests");
      if (latest && latest.item) {
        const item = latest.item;
        const t = item.totals || {};
        const c = latest.changes;
        const lines = [];
        if (c && c.new_failures_count) {
          lines.push(`<p class="bad">${c.new_failures_count} new failure(s) since <code>${escapeHtml((c.previous_git && c.previous_git.short) || "previous run")}</code></p>`);
          lines.push(`<ul class="now-list">${(c.new_failures || []).slice(0, 5).map(id =>
            `<li><code>${escapeHtml(id.split("::").slice(1).join("::") || id)}</code></li>`).join("")}</ul>`);
        } else if (c && c.fixed_count) {
          lines.push(`<p class="good">${c.fixed_count} fixed since <code>${escapeHtml((c.previous_git && c.previous_git.short) || "previous run")}</code></p>`);
        }
        if ((latest.flaky || []).length) lines.push(`<p class="warn">${latest.flaky.length} flaky over the last runs</p>`);
        testsEl.innerHTML = `<h2>Tests</h2>
          <a class="now-headline" href="#/tests/${escapeAttr(item.run_id)}">
            <span class="atr-state ${escapeAttr(stateLabel(item))}">${escapeHtml(stateLabel(item))}</span>
            <span class="now-big">${t.failed ? `<b class="bad">${t.failed}</b> failed · ` : ""}${t.passed || 0} passed</span>
          </a>
          ${TR ? TR.progressBar(t, isLive(item)) : ""}
          <p class="now-quiet">${escapeHtml(item.title)} · ${escapeHtml(item.git && item.git.short ? item.git.short : "")} · ${escapeHtml(TR ? TR.fmtAgo(item.started_at_ms) : "")}</p>
          ${lines.join("")}`;
      } else {
        testsEl.innerHTML = `<h2>Tests</h2><p class="now-quiet">No test runs yet.</p>
          <ul class="actions"><li><button type="button" data-cmd="${escapeAttr(START_TESTS)}">copy</button> <code>${escapeHtml(START_TESTS)}</code></li></ul>`;
        bindCopy(testsEl);
      }

      const benchEl = document.getElementById("now-bench");
      const bench = runs.filter(r => r.kind === "bench").slice(0, 4);
      benchEl.innerHTML = `<h2>Bench</h2>` + benchCompareHtml(benchCmp) + (bench.length
        ? `<div class="runs-list compact">${bench.slice(0, benchCmp && benchCmp.mode ? 2 : 4).map(runRow).join("")}</div>`
        : benchCmp && benchCmp.current ? "" : `<p class="now-quiet">No bench runs yet.</p>
           <ul class="actions"><li><button type="button" data-cmd="${escapeAttr(START_BENCH)}">copy</button> <code>${escapeHtml(START_BENCH)}</code></li></ul>`);
      bindCopy(benchEl);

      const issuesEl = document.getElementById("now-issues");
      const open = (issues.issues || []).filter(i => (i.status || "unresolved") === "unresolved");
      setBadge("issues", open.length ? String(open.length) : "");
      issuesEl.innerHTML = `<h2>Issues</h2>` + (open.length
        ? `<a class="now-headline" href="#/issues"><span class="now-big"><b class="bad">${open.length}</b> unresolved</span></a>
           <ul class="now-list">${open.slice(0, 4).map(i =>
             `<li>${escapeHtml(i.title || i.id)} <small>×${escapeHtml(String(i.count || 0))}</small></li>`).join("")}</ul>`
        : `<p class="now-quiet">No unresolved issues.</p>`);
    }

    const DECISION = {
      regression: ["slower", "bad"],
      improvement: ["faster", "good"],
      within_margin: ["same", ""],
      inconclusive: ["unclear", "warn"],
      unavailable: ["n/a", ""],
      neutral: ["info", ""],
    };

    function fmtPct(v) {
      if (v == null || !Number.isFinite(v)) return "—";
      return (v > 0 ? "+" : "") + v.toFixed(Math.abs(v) >= 10 ? 0 : 1) + "%";
    }

    // Latest bench run vs the previous comparable one (`GET /api/v1/bench/compare`).
    function benchCompareHtml(cmp) {
      if (!cmp) return "";
      if (!cmp.mode) {
        return cmp.current && cmp.note ? `<p class="now-quiet">Latest run: ${escapeHtml(cmp.note)}.</p>` : "";
      }
      const s = cmp.summary || {};
      const threshold = cmp.threshold != null ? cmp.threshold : 5;
      const head = [];
      if (s.regression) head.push(`<b class="bad">${s.regression}</b> slower`);
      if (s.improvement) head.push(`<b class="good">${s.improvement}</b> faster`);
      if (s.within_margin) head.push(`${s.within_margin} within ±${threshold}%`);
      if (s.inconclusive) head.push(`<span class="warn">${s.inconclusive}</span> unclear`);
      if (!head.length) head.push("nothing to compare");
      const link = (x, label) => `<a href="${escapeAttr(x.href)}"${x.href.startsWith("#") ? "" : ` target="_blank" rel="noopener"`}>${escapeHtml(label)}</a>`;
      const short = id => { const parts = String(id).split("/"); return parts[parts.length - 1].slice(0, 12); };
      const cur = cmp.current;
      const against = cmp.mode === "ab"
        ? `baseline vs candidate in ${link(cur, short(cur.id))}`
        : `${link(cur, short(cur.id))} vs previous ${link(cmp.previous, short(cmp.previous.id))}`;
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
      return `<a class="now-headline" href="${escapeAttr(cur.href)}"${cur.href.startsWith("#") ? "" : ` target="_blank" rel="noopener"`}>
          <span class="now-big">${head.join(" · ")}</span></a>
        <p class="now-quiet">${against}${ago}</p>
        ${rows ? `<table class="now-cmp">${rows}</table>` : ""}${more}`;
    }

    function median(nums) {
      if (!nums.length) return NaN;
      const s = nums.slice().sort((a, b) => a - b);
      const m = Math.floor(s.length / 2);
      return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
    }

    function fmtNs(ns) {
      if (!Number.isFinite(ns)) return "—";
      if (ns >= 1e9) return (ns / 1e9).toFixed(2) + " s";
      if (ns >= 1e6) return (ns / 1e6).toFixed(2) + " ms";
      if (ns >= 1e3) return (ns / 1e3).toFixed(1) + " µs";
      return ns.toFixed(0) + " ns";
    }

    async function refreshBench() {
      const listEl = document.getElementById("bench-list");
      const detailEl = document.getElementById("bench-detail");
      const metaElB = document.getElementById("bench-meta");
      const titleEl = document.getElementById("bench-title");
      if (currentRunId) {
        listEl.hidden = true;
        detailEl.hidden = false;
        titleEl.textContent = "Bench run";
        await refreshBenchDetail(currentRunId);
        if (!benchPollTimer) {
          benchPollTimer = setInterval(() => {
            if (currentCat === "bench" && currentRunId) {
              refreshBenchDetail(currentRunId).catch(() => {});
            }
          }, 1500);
        }
        return;
      }
      if (benchPollTimer) {
        clearInterval(benchPollTimer);
        benchPollTimer = null;
      }
      listEl.hidden = false;
      detailEl.hidden = true;
      titleEl.textContent = "Bench runs";
      const res = await fetch("/api/v1/bench/runs", { cache: "no-store" });
      const data = await res.json();
      metaElB.textContent = (data.note || "") + (data.hub_id ? " · hub " + data.hub_id.slice(0, 8) : "");
      const runs = data.runs || [];
      if (!runs.length) {
        listEl.innerHTML = `<p class="bench-live">${escapeHtml(data.note || "No runs.")}</p>`;
        return;
      }
      listEl.innerHTML = runs.map(r => {
        const st = escapeAttr(r.state || "registered");
        return `<a class="bench-row" href="#/bench/${escapeAttr(r.run_id)}">
          <span class="bench-state ${st}">${escapeHtml(r.state)}</span>
          <div>
            <strong>${escapeHtml(r.title || r.run_id)}</strong>
            <span>${escapeHtml(r.run_id)}</span>
          </div>
          <span>${escapeHtml(r.updated_at || "")}</span>
        </a>`;
      }).join("");
    }

    async function refreshBenchDetail(runId) {
      const res = await fetch("/api/v1/bench/runs/" + encodeURIComponent(runId), { cache: "no-store" });
      if (!res.ok) {
        document.getElementById("bench-live").textContent = "run not found";
        return;
      }
      const d = await res.json();
      document.getElementById("bench-meta").textContent =
        `${d.state} · hub ${String(d.hub_id || "").slice(0, 8)}`;
      const live = d.live || {};
      const completed = Number(live.completed || 0);
      const total = Number(live.total || 0);
      const prog = document.getElementById("bench-progress");
      if (total > 0) {
        prog.max = total;
        prog.value = completed;
      } else {
        prog.max = 100;
        prog.value = d.state === "complete" ? 100 : (d.state === "running" ? 50 : 0);
      }
      document.getElementById("bench-live").textContent = [
        live.state || d.state,
        total ? `${completed}/${total}` : "",
        live.variant || "",
      ].filter(Boolean).join(" · ");
      const links = [];
      if (d.report_url) links.push(`<a href="${escapeAttr(d.report_url)}" target="_blank" rel="noopener">report.html</a>`);
      if (d.run_json_url) links.push(`<a href="${escapeAttr(d.run_json_url)}" target="_blank" rel="noopener">run.json</a>`);
      if (d.jaeger_url) links.push(`<a href="${escapeAttr(d.jaeger_url)}" target="_blank" rel="noopener">Jaeger traces</a>`);
      document.getElementById("bench-links").innerHTML = links.join(" · ") || "";
      const frame = document.getElementById("bench-report-frame");
      if (d.has_report_html && d.report_url) {
        frame.hidden = false;
        if (frame.getAttribute("data-src") !== d.report_url) {
          frame.src = d.report_url;
          frame.setAttribute("data-src", d.report_url);
        }
      } else {
        frame.hidden = true;
      }
      if (d.has_run_json && d.run_json_url) {
        await renderBenchCharts(d.run_json_url);
      } else {
        document.getElementById("bench-charts").innerHTML =
          `<p class="bench-live">Waiting for run.json (charts appear when the suite finishes or writes partial results).</p>`;
      }
      await Promise.all([
        refreshBenchLogs(runId),
        refreshBenchMetrics(runId),
        refreshBenchIssues(runId),
      ]);
    }

    async function renderBenchCharts(url) {
      const res = await fetch(url, { cache: "no-store" });
      if (!res.ok) return;
      const run = await res.json();
      const byCase = {};
      for (const o of run.observations || []) {
        if (o.metric !== "wall") continue;
        if (!byCase[o.case]) byCase[o.case] = [];
        byCase[o.case].push(Number(o.value));
      }
      const families = {};
      for (const [caseId, values] of Object.entries(byCase)) {
        const parts = caseId.split("/");
        const engine = parts[parts.length - 1] || caseId;
        const family = parts.slice(0, -1).join("/") || caseId;
        if (!families[family]) families[family] = [];
        families[family].push({ engine, median: median(values) });
      }
      const cards = Object.keys(families).sort().map(family => {
        const rows = families[family].sort((a, b) => a.median - b.median);
        const max = Math.max(...rows.map(r => r.median), 1);
        const tr = rows.map(r => {
          const pct = Math.max(2, (r.median / max) * 100);
          return `<tr><td>${escapeHtml(r.engine)}</td><td>${fmtNs(r.median)}<div class="bench-bar" style="width:${pct}%"></div></td></tr>`;
        }).join("");
        return `<div class="bench-chart-card"><h4>${escapeHtml(family)}</h4>
          <table><thead><tr><th>engine</th><th>median wall</th></tr></thead><tbody>${tr}</tbody></table></div>`;
      }).join("");
      document.getElementById("bench-charts").innerHTML = cards
        || `<p class="bench-live">No wall observations in run.json.</p>`;
    }

    async function refreshBenchLogs(runId) {
      const el = document.getElementById("bench-logs");
      const res = await fetch("/api/v1/logs?limit=80&run_id=" + encodeURIComponent(runId), { cache: "no-store" });
      const data = await res.json();
      const entries = data.entries || [];
      el.textContent = entries.length
        ? entries.map(e => `[${e.severity}] ${e.service} ${e.body}`).join("\n")
        : (data.note || "No logs.");
    }

    async function refreshBenchMetrics(runId) {
      const el = document.getElementById("bench-metrics");
      const res = await fetch("/api/v1/metrics?limit=80&run_id=" + encodeURIComponent(runId), { cache: "no-store" });
      const data = await res.json();
      const points = data.points || [];
      el.textContent = points.length
        ? points.slice(-40).map(p => `${p.name}=${p.value} ${p.unit || ""}`.trim()).join("\n")
        : (data.note || "No metrics.");
    }

    async function refreshBenchIssues(runId) {
      const el = document.getElementById("bench-issues");
      const res = await fetch("/api/v1/issues?run_id=" + encodeURIComponent(runId), { cache: "no-store" });
      const data = await res.json();
      const issues = data.issues || [];
      el.textContent = issues.length
        ? issues.map(i => `${i.id} · ${i.title} · ${i.status}`).join("\n")
        : (data.note || "No issues.");
    }

    function toast(text) {
      toastEl.textContent = text;
      toastEl.classList.add("show");
      clearTimeout(toastEl._t);
      toastEl._t = setTimeout(() => toastEl.classList.remove("show"), 2200);
    }

    async function copy(text) {
      try {
        await navigator.clipboard.writeText(text);
        toast("copied: " + text);
      } catch {
        toast(text);
      }
    }

    function card(d) {
      const arts = (d.artifacts || []).map(a =>
        `<li><strong>${escapeHtml(a.label)}</strong> · ${escapeHtml(a.detail)}<br/><span>${escapeHtml(a.path)}</span></li>`
      ).join("");
      const acts = (d.actions || []).map(a =>
        `<li><button type="button" data-cmd="${escapeAttr(a.command)}">${escapeHtml(a.label)}</button></li>`
      ).join("");
      const extra = d.id === "unit" && (d.artifacts || []).some(a => a.label === "index.html")
        ? `<li><a href="/report/index.html" target="_blank" rel="noopener">Open unit HTML report</a></li>`
        : "";
      return `<article class="domain" data-id="${d.id}">
        <header>
          <h2>${escapeHtml(d.title)}</h2>
          <p class="blurb">${escapeHtml(d.blurb)}</p>
        </header>
        <div class="body">
          <span class="status ${d.status}">${escapeHtml(d.status)}</span>
          <p class="summary">${escapeHtml(d.summary)}</p>
          ${arts ? `<ul class="artifacts">${arts}</ul>` : ""}
          <ul class="actions">${acts}${extra}</ul>
        </div>
      </article>`;
    }

    function renderApis(apis) {
      const up = (apis || []).filter(a => a.available);
      if (!up.length) {
        apiListEl.innerHTML = `<li><p class="api-detail">No local APIs detected yet.</p></li>`;
        apisMetaEl.textContent = "0 available";
        return;
      }
      apisMetaEl.textContent = up.length + " available";
      apiListEl.innerHTML = up.map(a => {
        const href = escapeAttr(a.href);
        return `<li>
          <div class="api-method">${escapeHtml(a.method || "GET")}</div>
          <div>
            <a href="${href}" target="_blank" rel="noopener">${escapeHtml(a.name)}</a>
            <div><code>${escapeHtml(a.href)}</code></div>
            <p class="api-detail">${escapeHtml(a.detail || "")}</p>
          </div>
        </li>`;
      }).join("");
    }

    function seriesLabel(s) {
      const short = s.attrs ? s.name + " · " + s.attrs : s.name;
      return short.length > 48 ? short.slice(0, 46) + "…" : short;
    }

    function fmtValue(v, unit) {
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

    function lastPoint(s) {
      return s.points && s.points.length ? s.points[s.points.length - 1].v : NaN;
    }

    function sparkSvg(points, gradId) {
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

    function emptySvg(w, h, msg) {
      return `<svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="none" aria-hidden="true">
        <text x="12" y="${(h / 2 + 4).toFixed(0)}" fill="#7f9a8b" font-size="11">${escapeHtml(msg)}</text>
      </svg>`;
    }

    function gaugeSvg(value, name, unit) {
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

    function barSvg(seriesList) {
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

    function histSvg(hist) {
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

    function heatmapSvg(seriesList) {
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

    function pieSvg(seriesList) {
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

    function ensureSelection(series) {
      const ids = series.map(s => s.id);
      const same = ids.length === lastSeriesIds.length && ids.every((id, i) => id === lastSeriesIds[i]);
      if (!same) {
        lastSeriesIds = ids;
        selectedSeries = new Set(ids.slice(0, 4));
      } else {
        for (const id of [...selectedSeries]) {
          if (!ids.includes(id)) selectedSeries.delete(id);
        }
        if (!selectedSeries.size && ids.length) {
          selectedSeries = new Set(ids.slice(0, 4));
        }
      }
    }

    function renderVizPicker() {
      vizPickerEl.innerHTML = VIZ_TYPES.map(v =>
        `<button type="button" class="${currentViz === v.id ? "on" : ""}" data-viz="${v.id}">${escapeHtml(v.label)}</button>`
      ).join("");
      vizPickerEl.querySelectorAll("button[data-viz]").forEach(btn => {
        btn.addEventListener("click", () => {
          currentViz = btn.getAttribute("data-viz");
          if (lastMetrics) renderMetrics(lastMetrics);
          else renderVizPicker();
        });
      });
    }

    function renderSeriesChips(series) {
      if (!series.length) {
        metricsPickEl.hidden = true;
        metricsPickEl.innerHTML = "";
        return;
      }
      metricsPickEl.hidden = false;
      metricsPickEl.innerHTML = series.map(s => {
        const on = selectedSeries.has(s.id) ? "on" : "";
        return `<button type="button" class="${on}" data-sid="${escapeAttr(s.id)}" title="${escapeAttr(s.id)}">${escapeHtml(seriesLabel(s))}</button>`;
      }).join("");
      metricsPickEl.querySelectorAll("button[data-sid]").forEach(btn => {
        btn.addEventListener("click", () => {
          const id = btn.getAttribute("data-sid");
          if (selectedSeries.has(id)) {
            if (selectedSeries.size > 1) selectedSeries.delete(id);
          } else {
            selectedSeries.add(id);
          }
          if (lastMetrics) renderMetrics(lastMetrics);
        });
      });
    }

    function boardShell(title, sub, now, svg, wide) {
      return `<article class="metric-board${wide ? " wide" : ""}">
        <header>
          <div>
            <h3>${escapeHtml(title)}</h3>
            ${sub ? `<div class="sub">${escapeHtml(sub)}</div>` : ""}
          </div>
          ${now != null ? `<div class="now">${escapeHtml(now)}</div>` : ""}
        </header>
        ${svg}
      </article>`;
    }

    function renderMetricBoards(series, histogram) {
      ensureSelection(series);
      renderSeriesChips(series);
      const shown = series.filter(s => selectedSeries.has(s.id));
      const combined = ["bar", "histogram", "heatmap", "pie"].includes(currentViz);
      metricsBoardsEl.classList.toggle("single", combined || currentViz === "table");

      if (!series.length) {
        metricsBoardsEl.innerHTML = "";
        return;
      }

      if (currentViz === "table") {
        metricsBoardsEl.innerHTML = "";
        return;
      }

      if (currentViz === "timeseries") {
        metricsBoardsEl.innerHTML = shown.map((s, i) => {
          const sub = [s.service, s.attrs].filter(Boolean).join(" · ");
          return boardShell(s.name, sub, fmtValue(lastPoint(s), s.unit || ""), sparkSvg(s.points || [], "mf" + i), false);
        }).join("");
        return;
      }

      if (currentViz === "gauge") {
        metricsBoardsEl.innerHTML = shown.map(s => {
          const sub = [s.service, s.attrs].filter(Boolean).join(" · ");
          return boardShell(s.name, sub, null, gaugeSvg(lastPoint(s), s.name, s.unit || ""), false);
        }).join("");
        return;
      }

      if (currentViz === "bar") {
        metricsBoardsEl.innerHTML = boardShell("Bar · selected series", shown.length + " series", null, barSvg(shown), true);
        return;
      }

      if (currentViz === "histogram") {
        metricsBoardsEl.innerHTML = boardShell(
          "Histogram · all recent points",
          (histogram && histogram.bucket_count ? histogram.bucket_count + " buckets" : ""),
          null,
          histSvg(histogram),
          true
        );
        return;
      }

      if (currentViz === "heatmap") {
        metricsBoardsEl.innerHTML = boardShell("Heatmap · time × series", shown.length + " series", null, heatmapSvg(shown), true);
        return;
      }

      if (currentViz === "pie") {
        metricsBoardsEl.innerHTML = boardShell("Pie · latest values", shown.length + " series", null, pieSvg(shown), true);
        return;
      }

      metricsBoardsEl.innerHTML = "";
    }

    function renderLatestTable(latest, note) {
      if (currentViz !== "table" && currentViz !== "timeseries" && currentViz !== "gauge") {
        // Keep table visible for explore-style overview except when another combined viz owns the board.
        if (["bar", "histogram", "heatmap", "pie"].includes(currentViz)) {
          metricsPanelEl.hidden = true;
          return;
        }
      }
      metricsPanelEl.hidden = false;
      if (!latest.length) {
        metricsPanelEl.innerHTML = `<p class="metrics-empty">${escapeHtml(note || "No metrics yet.")}</p>`;
        return;
      }
      const rows = latest.map(m => {
        const when = m.time_ms ? formatTime(m) : (m.time || "—");
        const unit = m.unit ? ` ${escapeHtml(m.unit)}` : "";
        const sub = [m.service, m.scope, m.attrs].filter(Boolean).join(" · ");
        return `<tr>
          <td class="name">${escapeHtml(m.name)}<div class="sub">${escapeHtml(m.kind || "")}</div></td>
          <td class="val">${Number(m.value).toPrecision(6)}${unit}<div class="sub">${escapeHtml(when)}</div></td>
          <td>${sub ? `<div class="sub">${escapeHtml(sub)}</div>` : "—"}</td>
        </tr>`;
      }).join("");
      metricsPanelEl.innerHTML = `<table class="metrics-table">
        <thead><tr><th>metric</th><th>value</th><th>labels</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>`;
    }

    function renderMetrics(data) {
      lastMetrics = data;
      metricsMetaEl.textContent = data.note || "";
      renderVizPicker();
      const series = data.series || [];
      const latest = data.latest || [];
      renderMetricBoards(series, data.histogram);
      if (currentViz === "table") {
        metricsPanelEl.hidden = false;
        renderLatestTable(latest, data.note);
      } else if (currentViz === "timeseries" || currentViz === "gauge") {
        renderLatestTable(latest, data.note);
      } else {
        metricsPanelEl.hidden = true;
      }
      if (!series.length && !latest.length) {
        metricsBoardsEl.innerHTML = "";
        metricsPanelEl.hidden = false;
        metricsPanelEl.innerHTML = `<p class="metrics-empty">${escapeHtml(data.note || "No metrics yet.")}</p>`;
      }
    }

    function formatTime(entry) {
      if (entry.time_ms) {
        try {
          return new Date(entry.time_ms).toISOString().replace("T", " ").replace("Z", "");
        } catch {}
      }
      return entry.time || "—";
    }

    function sevClass(sev) {
      const s = String(sev || "info").toLowerCase();
      if (s.includes("fatal")) return "fatal";
      if (s.includes("error") || s.includes("err")) return "error";
      if (s.includes("warn")) return "warn";
      if (s.includes("debug")) return "debug";
      if (s.includes("trace")) return "trace";
      return "info";
    }

    function normalizeSev(sev) {
      const s = String(sev || "INFO").toUpperCase();
      if (s.includes("FATAL") || s.includes("CRITICAL")) return "FATAL";
      if (s.includes("ERROR") || s.includes("ERR")) return "ERROR";
      if (s.includes("WARN")) return "WARN";
      if (s.includes("DEBUG")) return "DEBUG";
      if (s.includes("TRACE")) return "TRACE";
      return "INFO";
    }

    function renderSevChips() {
      sevChipsEl.innerHTML = SEV_ORDER.map(sev => {
        const on = selectedSeverities.has(sev) ? "on" : "";
        return `<button type="button" class="${on}" data-sev="${sev}">${sev}</button>`;
      }).join("");
      sevChipsEl.querySelectorAll("button[data-sev]").forEach(btn => {
        btn.addEventListener("click", () => {
          const sev = btn.getAttribute("data-sev");
          if (selectedSeverities.has(sev)) {
            if (selectedSeverities.size > 1) selectedSeverities.delete(sev);
          } else {
            selectedSeverities.add(sev);
          }
          renderSevChips();
          if (lastLogs) paintLogs();
        });
      });
    }

    function renderSevBar(bySeverity) {
      const items = bySeverity || [];
      const total = items.reduce((a, x) => a + Number(x.count || 0), 0);
      if (!total) {
        sevBarEl.hidden = true;
        sevBarEl.innerHTML = "";
        return;
      }
      sevBarEl.hidden = false;
      sevBarEl.innerHTML = items.map(x => {
        const pct = (Number(x.count) / total) * 100;
        return `<span class="${escapeAttr(x.severity)}" style="width:${pct.toFixed(2)}%" title="${escapeAttr(x.severity + ": " + x.count)}"></span>`;
      }).join("");
    }

    function syncServiceSelect(services) {
      const cur = logsServiceEl.value;
      const opts = [`<option value="">all services</option>`]
        .concat((services || []).map(s => `<option value="${escapeAttr(s)}">${escapeHtml(s)}</option>`));
      logsServiceEl.innerHTML = opts.join("");
      if (cur && (services || []).includes(cur)) logsServiceEl.value = cur;
    }

    function paintLogs() {
      const data = lastLogs || { entries: [], note: "" };
      const q = (logsFilterEl.value || "").trim().toLowerCase();
      const svc = logsServiceEl.value || "";
      const entries = (data.entries || []).filter(e => {
        if (!selectedSeverities.has(normalizeSev(e.severity))) return false;
        if (svc && e.service !== svc) return false;
        if (q) {
          const hay = `${e.body || ""} ${e.service || ""} ${e.scope || ""}`.toLowerCase();
          if (!hay.includes(q)) return false;
        }
        return true;
      });
      logsMetaEl.textContent = [
        data.note || "",
        entries.length !== (data.entries || []).length
          ? `showing ${entries.length}/${(data.entries || []).length}`
          : "",
      ].filter(Boolean).join(" · ");

      if (!entries.length) {
        logsPanelEl.innerHTML = `<p class="logs-empty">${escapeHtml(data.note || "No logs match filters.")}</p>`;
        return;
      }
      logsPanelEl.innerHTML = entries.map(e => {
        const sev = escapeHtml(e.severity || "INFO");
        const svcLabel = [e.service, e.scope].filter(Boolean).join(" · ");
        return `<div class="log-row">
          <div class="log-time">${escapeHtml(formatTime(e))}</div>
          <div class="log-sev ${sevClass(e.severity)}">${sev}</div>
          <div class="log-body">${escapeHtml(e.body || "")}${svcLabel ? `<div class="log-svc">${escapeHtml(svcLabel)}</div>` : ""}</div>
        </div>`;
      }).join("");
      logsPanelEl.scrollTop = logsPanelEl.scrollHeight;
    }

    function renderLogs(data) {
      lastLogs = data;
      renderSevBar(data.by_severity);
      syncServiceSelect(data.services);
      paintLogs();
    }

    function escapeHtml(s) {
      return String(s).replace(/[&<>"']/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;","\"":"&quot;","'":"&#39;"}[c]));
    }
    function escapeAttr(s) {
      return escapeHtml(s).replace(/\n/g, " ");
    }

    async function refreshIssues() {
      const res = await fetch("/api/v1/issues", { cache: "no-store" });
      const data = await res.json();
      issuesMetaEl.textContent = data.note || "";
      const issues = data.issues || [];
      if (!issues.length) {
        issuesPanelEl.innerHTML = `<p class="issues-empty">${escapeHtml(data.note || "No issues.")}</p>`;
        return;
      }
      issuesPanelEl.innerHTML = issues.map(i => {
        const active = i.id === selectedIssueId ? " active" : "";
        return `<div class="issue-row${active}" data-id="${escapeAttr(i.id)}" role="button" tabindex="0">
          <div class="issue-id">${escapeHtml(i.id)}</div>
          <div>
            <div class="issue-title">${escapeHtml(i.title || "")}
              <span class="issue-status ${escapeAttr(i.status || "unresolved")}">${escapeHtml(i.status || "unresolved")}</span>
            </div>
            <div class="issue-meta-line">${escapeHtml([i.level, i.service, i.environment, i.release].filter(Boolean).join(" · "))}</div>
            <div class="issue-meta-line">first ${escapeHtml(i.first_seen || "")} · last ${escapeHtml(i.last_seen || "")}</div>
          </div>
          <div class="issue-count">×${escapeHtml(String(i.count || 0))}</div>
        </div>`;
      }).join("");
      issuesPanelEl.querySelectorAll(".issue-row").forEach(row => {
        row.addEventListener("click", () => openIssue(row.getAttribute("data-id")));
        row.addEventListener("keydown", ev => {
          if (ev.key === "Enter" || ev.key === " ") {
            ev.preventDefault();
            openIssue(row.getAttribute("data-id"));
          }
        });
      });
    }

    async function openIssue(id) {
      if (!id) return;
      selectedIssueId = id;
      const res = await fetch("/api/v1/issues/" + encodeURIComponent(id), { cache: "no-store" });
      if (!res.ok) {
        toast("issue not found");
        return;
      }
      const issue = await res.json();
      issueDetailEl.hidden = false;
      const stored = Array.isArray(issue.events) ? issue.events.length : 0;
      issueDetailTitleEl.textContent =
        issue.id + " · " + (issue.title || "") +
        ` · ${stored} stored / ${issue.count || 0} lifetime`;
      const detailPayload = (issue.events && issue.events.length)
        ? { events: issue.events, last_event: issue.last_event }
        : (issue.last_event || issue);
      issueDetailBodyEl.textContent = JSON.stringify(detailPayload, null, 2);
      issueActionsEl.innerHTML = ["resolve", "ignore", "reopen"].map(a =>
        `<button type="button" data-action="${a}">${a}</button>`
      ).join("");
      issueActionsEl.querySelectorAll("button[data-action]").forEach(btn => {
        btn.addEventListener("click", async () => {
          const action = btn.getAttribute("data-action");
          const r = await fetch(`/api/v1/issues/${encodeURIComponent(id)}/${action}`, { method: "POST" });
          if (!r.ok) {
            toast("action failed");
            return;
          }
          toast(action + " · " + id);
          await refreshIssues();
          await openIssue(id);
        });
      });
      await refreshIssues();
    }

    async function refreshLogs() {
      const res = await fetch("/api/v1/logs?limit=150", { cache: "no-store" });
      const data = await res.json();
      renderLogs(data);
    }

    async function refreshMetrics() {
      const res = await fetch("/api/v1/metrics?limit=800", { cache: "no-store" });
      const data = await res.json();
      renderMetrics(data);
    }

    async function refresh() {
      const res = await fetch("/api/v1/status", { cache: "no-store" });
      const data = await res.json();
      lastHubId = data.hub_id || "";
      metaEl.innerHTML = `<span>hub <code>${escapeHtml(lastHubId)}</code></span><span>root <code>${escapeHtml(data.root)}</code></span><span>generated <code>${escapeHtml(data.generated)}</code></span><span><button type="button" id="reload">refresh</button></span>`;
      document.getElementById("reload").onclick = () => {
        refresh(); refreshLogs(); refreshMetrics(); refreshIssues();
        showRoute();
      };
      renderApis(data.apis);
      lastDomains = data.domains || {};
      const order = ["unit", "bench", "mon", "otel", "err", "collector"];
      domainsEl.innerHTML = order.filter(k => lastDomains[k]).map(k => card(lastDomains[k])).join("");
      bindCopy(domainsEl);
    }

    renderCats();
    renderVizPicker();
    renderSevChips();
    logsFilterEl.addEventListener("input", () => paintLogs());
    logsServiceEl.addEventListener("change", () => paintLogs());
    window.addEventListener("hashchange", () => showRoute());
    if (!location.hash) location.hash = "#/now";
    else showRoute();

    // One clock for the run views: live tests refresh every 1.5 s, the rest every few ticks.
    let tick = 0;
    setInterval(() => {
      tick++;
      if (document.hidden) return;
      if (currentCat === "tests" && (testsLive || tick % 4 === 0)) {
        refreshTests(false).catch(() => {});
      } else if (currentCat === "now" && tick % 2 === 0) {
        refreshNow().catch(() => {});
      } else if (currentCat === "runs" && tick % 2 === 0) {
        refreshRuns().catch(() => {});
      } else if (tick % 4 === 0 && currentCat !== "now" && currentCat !== "runs") {
        fetchTimeline("", 40).catch(() => {});
      }
    }, 1500);

    refresh().catch(err => {
      metaEl.textContent = "failed to load /api/v1/status: " + err;
    });
    refreshLogs().catch(err => {
      logsMetaEl.textContent = "failed to load /api/v1/logs: " + err;
    });
    refreshMetrics().catch(err => {
      metricsMetaEl.textContent = "failed to load /api/v1/metrics: " + err;
    });
    refreshIssues().catch(err => {
      issuesMetaEl.textContent = "failed to load /api/v1/issues: " + err;
    });
    setInterval(() => { refresh().catch(() => {}); }, 15000);
    setInterval(() => { refreshLogs().catch(() => {}); }, 4000);
    setInterval(() => { refreshMetrics().catch(() => {}); }, 4000);
    setInterval(() => { refreshIssues().catch(() => {}); }, 5000);
