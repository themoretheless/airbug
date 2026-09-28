/* airbug test run view — shared by the hub (#/tests) and the standalone index.html.
 *
 * AirbugTestRun.createView(element, { files }) → { update(data) }
 *   data = { report, changes?, history?: { runs, tests }, flaky?, files? }
 * `update` can be called on every poll: filters, search, open tests and scroll survive.
 */
(function (global) {
  "use strict";

  const ORDER = { failed: 0, not_run: 1, pending: 2, passed: 3, ignored: 4 };
  const LABEL = { failed: "failed", passed: "passed", ignored: "ignored", not_run: "not run", pending: "pending" };
  const FILTERS = [
    { id: "problems", label: "problems", test: t => t.status === "failed" || t.status === "not_run" },
    { id: "failed", label: "failed", test: t => t.status === "failed" },
    { id: "not_run", label: "not run", test: t => t.status === "not_run" },
    { id: "passed", label: "passed", test: t => t.status === "passed" },
    { id: "ignored", label: "ignored", test: t => t.status === "ignored" },
    { id: "all", label: "all", test: () => true },
  ];
  const MAX_ROWS = 1500;

  function esc(value) {
    return String(value == null ? "" : value).replace(/[&<>"']/g, c => (
      { "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c]
    ));
  }

  function fmtDuration(seconds) {
    const s = Number(seconds);
    if (!Number.isFinite(s)) return "";
    if (s < 1) return Math.round(s * 1000) + " ms";
    if (s < 60) return s.toFixed(s < 10 ? 2 : 1) + " s";
    const m = Math.floor(s / 60);
    return m + " m " + Math.round(s - m * 60) + " s";
  }

  function fmtWhen(ms) {
    const n = Number(ms);
    if (!n) return "";
    try {
      return new Date(n).toLocaleString(undefined, {
        month: "short", day: "numeric", hour: "2-digit", minute: "2-digit", second: "2-digit",
      });
    } catch (_) {
      return new Date(n).toISOString();
    }
  }

  function fmtAgo(ms) {
    const n = Number(ms);
    if (!n) return "";
    const s = Math.max(0, (Date.now() - n) / 1000);
    if (s < 60) return Math.round(s) + "s ago";
    if (s < 3600) return Math.round(s / 60) + "m ago";
    if (s < 86400) return Math.round(s / 3600) + "h ago";
    return Math.round(s / 86400) + "d ago";
  }

  function splitName(name) {
    const i = String(name).lastIndexOf("::");
    if (i < 0) return { pre: "", leaf: String(name) };
    return { pre: name.slice(0, i + 2), leaf: name.slice(i + 2) };
  }

  /** Line diff (LCS). Returns [{op: " "|"-"|"+", text}] or null when too large. */
  function lineDiff(a, b) {
    const x = String(a).split("\n");
    const y = String(b).split("\n");
    const n = x.length, m = y.length;
    if (n * m > 400000) return null;
    const dp = [];
    for (let i = 0; i <= n; i++) dp.push(new Int32Array(m + 1));
    for (let i = n - 1; i >= 0; i--) {
      for (let j = m - 1; j >= 0; j--) {
        dp[i][j] = x[i] === y[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
      }
    }
    const out = [];
    let i = 0, j = 0;
    while (i < n && j < m) {
      if (x[i] === y[j]) { out.push({ op: " ", text: x[i] }); i++; j++; }
      else if (dp[i + 1][j] >= dp[i][j + 1]) { out.push({ op: "-", text: x[i] }); i++; }
      else { out.push({ op: "+", text: y[j] }); j++; }
    }
    while (i < n) out.push({ op: "-", text: x[i++] });
    while (j < m) out.push({ op: "+", text: y[j++] });
    return out;
  }

  /** Summary numbers shared by the hub "Now" page and the view header. */
  function summarize(report) {
    const t = (report && report.totals) || {};
    return {
      total: t.total || 0,
      completed: t.completed || 0,
      passed: t.passed || 0,
      failed: t.failed || 0,
      ignored: t.ignored || 0,
      not_run: t.not_run || 0,
    };
  }

  function progressBar(totals, live) {
    const total = Math.max(totals.total || 0, totals.completed || 0, 1);
    const pct = v => ((v || 0) / total * 100).toFixed(2) + "%";
    return `<div class="atr-bar${live ? " live" : ""}" role="progressbar" aria-valuemin="0"
        aria-valuemax="${total}" aria-valuenow="${totals.completed || 0}">
      <span class="p" style="width:${pct(totals.passed)}"></span>
      <span class="f" style="width:${pct(totals.failed)}"></span>
      <span class="n" style="width:${pct(totals.not_run)}"></span>
      <span class="i" style="width:${pct(totals.ignored)}"></span>
    </div>`;
  }

  function createView(root, options) {
    const opts = options || {};
    const state = {
      data: null,
      filter: null,
      filterTouched: false,
      query: "",
      open: new Set(),
      suiteToggle: new Map(),
      attachmentText: new Map(),
    };

    root.classList.add("atr");
    root.innerHTML = `
      <div data-part="head"></div>
      <div data-part="changes"></div>
      <div data-part="blocks"></div>
      <div class="atr-tools">
        <input type="search" data-part="search" placeholder="filter tests by name or suite…" autocomplete="off" />
        <span class="atr-count" data-part="count"></span>
      </div>
      <div data-part="list"></div>
      <div data-part="extra"></div>`;
    const part = name => root.querySelector(`[data-part="${name}"]`);
    const searchEl = part("search");

    function files() {
      const base = opts.files != null ? opts.files : (state.data && state.data.files) || "";
      return base;
    }

    function report() {
      return (state.data && state.data.report) || null;
    }

    function changeSets() {
      const c = state.data && state.data.changes;
      return {
        newFailures: new Set((c && c.new_failures) || []),
        fixed: new Set((c && c.fixed) || []),
      };
    }

    function flakySet() {
      const ids = new Set(((state.data && state.data.flaky) || []).map(f => f.id));
      for (const t of (report() && report().tests) || []) {
        if (t.flaky && t.flaky.length) ids.add(t.id);
      }
      return ids;
    }

    function allFilters() {
      const extra = [];
      const sets = changeSets();
      const flaky = flakySet();
      if (flaky.size) extra.push({ id: "flaky", label: "flaky", test: t => flaky.has(t.id) });
      if (sets.newFailures.size || sets.fixed.size) {
        extra.push({
          id: "changed", label: "changed",
          test: t => sets.newFailures.has(t.id) || sets.fixed.has(t.id),
        });
      }
      return FILTERS.concat(extra);
    }

    function defaultFilter(r) {
      const t = summarize(r);
      return t.failed + t.not_run > 0 ? "problems" : "all";
    }

    function update(data) {
      state.data = data || {};
      const r = report();
      if (!state.filterTouched || !allFilters().some(f => f.id === state.filter)) {
        state.filter = defaultFilter(r);
      }
      paint();
    }

    function paint() {
      paintHead();
      paintChanges();
      paintBlocks();
      paintList();
      paintUnattributed();
    }

    function paintHead() {
      const r = report();
      const el = part("head");
      if (!r) {
        el.innerHTML = `<div class="atr-empty">No test run data yet.</div>`;
        return;
      }
      const item = (state.data && state.data.item) || {};
      const stateName = item.stale ? "stale" : (r.state || "running");
      const live = !item.stale && (r.state === "running" || r.state === "building");
      const t = summarize(r);
      const git = r.git || {};
      const meta = [];
      if (git.short) {
        meta.push(`<span title="${esc(git.commit || "")}">commit <code>${esc(git.short)}</code>${git.dirty ? ` <span class="dirty">+dirty</span>` : ""}</span>`);
      }
      if (git.branch && git.branch !== "HEAD") meta.push(`<span>${esc(git.branch)}</span>`);
      if (git.subject) meta.push(`<span>${esc(git.subject)}</span>`);
      if (r.started_at_ms) meta.push(`<span title="${esc(fmtWhen(r.started_at_ms))}">started ${esc(fmtAgo(r.started_at_ms))}</span>`);
      const duration = r.duration_s != null ? r.duration_s : (state.data.progress && state.data.progress.duration_s);
      if (duration != null) meta.push(`<span>${live ? "running for" : "took"} ${esc(fmtDuration(duration))}</span>`);
      if (r.exit_code != null && r.exit_code !== 0) meta.push(`<span>exit ${esc(r.exit_code)}</span>`);
      if (files()) meta.push(`<a href="${esc(files())}output.log" target="_blank" rel="noopener">output.log</a>`);
      if (files() && opts.standaloneLink !== false) meta.push(`<a href="${esc(files())}index.html" target="_blank" rel="noopener">standalone report</a>`);
      const activity = live && state.data.progress && state.data.progress.activity
        ? `<div class="atr-activity">${esc(state.data.progress.activity)}</div>` : "";
      const counter = t.total
        ? `${t.completed}/${t.total} tests`
        : (r.state === "building" ? "building…" : "no tests");

      const filters = allFilters();
      const all = (r.tests || []);
      const chips = filters.map(f => {
        const count = all.filter(f.test).length;
        const on = state.filter === f.id ? " on" : "";
        const disabled = count === 0 && f.id !== "all" && state.filter !== f.id ? " disabled" : "";
        const cls = f.id === "changed" ? "new" : f.id;
        return `<button type="button" class="atr-chip ${esc(cls)}${on}" data-filter="${esc(f.id)}"${disabled}>
          ${esc(f.label)} <b>${count}</b></button>`;
      }).join("");

      el.innerHTML = `<div class="atr-head">
        <div class="atr-title">
          <span class="atr-state ${esc(stateName)}">${esc(stateName)}</span>
          <h2>${esc(r.title || "cargo test")}</h2>
          <span class="atr-count">${esc(counter)}</span>
        </div>
        <div class="atr-meta">${meta.join("")}</div>
        ${progressBar(t, live)}
        ${activity}
        <div class="atr-chips">${chips}</div>
      </div>`;
    }

    function testLabel(id) {
      const r = report();
      const t = r && (r.tests || []).find(x => x.id === id);
      return t ? `${t.name}` : id;
    }

    function paintChanges() {
      const el = part("changes");
      const c = state.data && state.data.changes;
      if (!c) {
        el.innerHTML = "";
        return;
      }
      const prev = c.previous_git && c.previous_git.short ? `<code>${esc(c.previous_git.short)}</code>` : "previous run";
      const bits = [];
      if (c.new_failures_count) bits.push(`<b class="bad">${c.new_failures_count} new failure${c.new_failures_count === 1 ? "" : "s"}</b>`);
      if (c.fixed_count) bits.push(`${c.fixed_count} fixed`);
      if (c.added_count) bits.push(`${c.added_count} new test${c.added_count === 1 ? "" : "s"}`);
      if (c.removed_count) bits.push(`${c.removed_count} removed`);
      const tone = c.new_failures_count ? "bad" : (c.fixed_count ? "good" : "");
      const list = (c.new_failures || []).slice(0, 8).map(id =>
        `<li><button type="button" class="link" data-goto="${esc(id)}">${esc(testLabel(id))}</button></li>`
      ).join("");
      el.innerHTML = `<div class="atr-changes ${tone}">
        vs ${prev} <span class="atr-count">(${esc(fmtAgo(c.previous_started_at_ms))})</span>:
        ${bits.length ? bits.join(" · ") : "no status changes"}
        ${list ? `<ul>${list}</ul>` : ""}
      </div>`;
    }

    function paintBlocks() {
      const r = report();
      const el = part("blocks");
      if (!r) {
        el.innerHTML = "";
        return;
      }
      const blocks = [];
      if (r.build_log) {
        blocks.push(`<div class="atr-block"><h3>build / listing output</h3><pre>${esc(r.build_log)}</pre></div>`);
      }
      if (r.diagnostics && r.diagnostics.length) {
        blocks.push(`<div class="atr-block"><h3>diagnostics</h3><pre>${esc(r.diagnostics.join("\n"))}</pre></div>`);
      }
      el.innerHTML = blocks.join("");
    }

    function matches(test, suite) {
      const q = state.query.trim().toLowerCase();
      if (!q) return true;
      return String(test.name).toLowerCase().includes(q) || String(suite).toLowerCase().includes(q);
    }

    function paintList() {
      const r = report();
      const el = part("list");
      const countEl = part("count");
      if (!r) {
        el.innerHTML = "";
        countEl.textContent = "";
        return;
      }
      const filter = allFilters().find(f => f.id === state.filter) || FILTERS[FILTERS.length - 1];
      const sets = changeSets();
      const flaky = flakySet();
      const strips = (state.data.history && state.data.history.tests) || {};
      const runs = (state.data.history && state.data.history.runs) || [];

      const bySuite = new Map();
      for (const s of r.suites || []) bySuite.set(s.id, { suite: s, tests: [] });
      let shown = 0, rendered = 0;
      for (const t of r.tests || []) {
        if (!filter.test(t) || !matches(t, t.suite)) continue;
        if (!bySuite.has(t.suite)) bySuite.set(t.suite, { suite: { id: t.suite, state: "done", totals: {} }, tests: [] });
        bySuite.get(t.suite).tests.push(t);
        shown++;
      }
      const groups = Array.from(bySuite.values()).filter(g =>
        g.tests.length || (g.suite.state === "crashed" && (filter.id === "problems" || filter.id === "all"))
      );
      groups.sort((a, b) => rank(a) - rank(b));
      function rank(g) {
        const s = g.suite;
        if (s.state === "crashed") return 0;
        if ((s.totals && s.totals.failed) || g.tests.some(t => t.status === "failed")) return 1;
        if (s.state === "running") return 2;
        return 3;
      }

      const total = (r.tests || []).length;
      countEl.textContent = shown === total ? `${total} tests` : `${shown} of ${total} tests`;
      if (!groups.length) {
        const live = r.state === "running" || r.state === "building";
        el.innerHTML = `<div class="atr-empty">${
          live && !total ? "Waiting for tests…" :
          state.filter === "problems" ? "No failures." : "No tests match."
        }</div>`;
        return;
      }

      const autoOpen = state.filter !== "all" || state.query.trim() !== "" || total <= 300;
      el.innerHTML = groups.map(g => {
        const s = g.suite;
        const toggled = state.suiteToggle.get(s.id);
        const hasProblem = s.state === "crashed" || g.tests.some(t => t.status === "failed" || t.status === "not_run");
        const open = toggled != null ? toggled : (autoOpen || hasProblem);
        const tt = s.totals || {};
        const stats = [
          tt.failed ? `<span class="f">${tt.failed} failed</span>` : "",
          `${tt.passed || 0}/${tt.total || g.tests.length} passed`,
          tt.ignored ? `${tt.ignored} ignored` : "",
          s.duration_s != null ? fmtDuration(s.duration_s) : "",
        ].filter(Boolean).join(" · ");
        let body = "";
        if (open) {
          const crash = s.state === "crashed"
            ? `<div class="atr-crash"><div class="atr-block"><h3>process ended without a result — last output</h3><pre>${esc(s.crash_log || "(no output)")}</pre></div></div>`
            : "";
          const tests = g.tests.slice().sort((a, b) =>
            (ORDER[a.status] ?? 9) - (ORDER[b.status] ?? 9) || String(a.name).localeCompare(String(b.name))
          );
          const rows = [];
          for (const t of tests) {
            if (rendered >= MAX_ROWS) break;
            rows.push(testRow(t, sets, flaky, strips[t.id], runs));
            rendered++;
          }
          const more = tests.length > rows.length
            ? `<div class="atr-more">${tests.length - rows.length} more — narrow the filter to see them</div>` : "";
          body = `<div class="atr-suite-body">${crash}${rows.join("")}${more}</div>`;
        }
        const suiteState = s.state || "done";
        return `<section class="atr-suite" data-suite="${esc(s.id)}">
          <button type="button" class="atr-suite-head" data-toggle-suite="${esc(s.id)}" aria-expanded="${open}">
            <span class="atr-dot ${esc(suiteState === "done" && tt.failed ? "failed" : suiteState)}"></span>
            <span class="atr-suite-name">${esc(s.id)}${s.kind ? `<small>${esc(s.kind)}</small>` : ""}${suiteState === "crashed" ? `<span class="atr-badge new">crashed</span>` : ""}</span>
            <span class="atr-suite-stats">${stats}</span>
          </button>
          ${body}
        </section>`;
      }).join("");
    }

    function strip(chars, runs) {
      if (!chars) return "";
      return `<span class="atr-strip" title="history, oldest → newest">${Array.from(chars).map((c, i) => {
        const run = runs[i] || {};
        const label = `${run.short || ""} ${fmtWhen(run.started_at_ms)}`.trim();
        return `<i class="${esc(c)}" title="${esc(label)}"></i>`;
      }).join("")}</span>`;
    }

    function testRow(t, sets, flaky, history, runs) {
      const name = splitName(t.name);
      const badges = [];
      if (sets.newFailures.has(t.id)) badges.push(`<span class="atr-badge new">new failure</span>`);
      if (sets.fixed.has(t.id)) badges.push(`<span class="atr-badge fixed">fixed</span>`);
      if (flaky.has(t.id)) badges.push(`<span class="atr-badge flaky">flaky</span>`);
      const stepCount = countSteps(t.steps || []);
      if (stepCount) badges.push(`<span class="atr-badge info">${stepCount} step${stepCount === 1 ? "" : "s"}</span>`);
      const atts = countAttachments(t);
      if (atts) badges.push(`<span class="atr-badge info">${atts} file${atts === 1 ? "" : "s"}</span>`);
      const open = state.open.has(t.id);
      const side = [
        strip(history, runs),
        t.finished_s != null ? `<span title="finished at, seconds into the run">+${esc(Number(t.finished_s).toFixed(2))}s</span>` : "",
        `<span>${esc(LABEL[t.status] || t.status)}</span>`,
      ].join("");
      return `<div class="atr-test${open ? " open" : ""}" data-test="${esc(t.id)}">
        <button type="button" class="atr-test-head" data-toggle-test="${esc(t.id)}" aria-expanded="${open}">
          <span class="atr-dot ${esc(t.status)}"></span>
          <span class="atr-name"><span class="pre">${esc(name.pre)}</span>${esc(name.leaf)}${badges.join("")}</span>
          <span class="atr-side">${side}</span>
        </button>
        ${open ? testDetail(t) : ""}
      </div>`;
    }

    function countSteps(steps) {
      return steps.reduce((n, s) => n + 1 + countSteps(s.children || []), 0);
    }

    function countAttachments(node) {
      let n = (node.attachments || []).length;
      for (const s of node.steps || node.children || []) n += countAttachments(s);
      return n;
    }

    function testDetail(t) {
      const parts = [];
      if (t.ignore_reason) parts.push(`<div class="muted">ignored: ${esc(t.ignore_reason)}</div>`);
      if (t.flaky && t.flaky.length) parts.push(`<div class="muted">marked flaky: ${esc(t.flaky.join("; "))}</div>`);
      if (t.output) parts.push(`<div class="atr-block"><h3>output</h3><pre>${esc(t.output)}</pre></div>`);
      if (t.comparisons && t.comparisons.length) parts.push(t.comparisons.map(comparison).join(""));
      if (t.steps && t.steps.length) parts.push(`<div class="atr-block"><h3>steps</h3><ul class="atr-steps">${t.steps.map(step).join("")}</ul></div>`);
      if (t.attachments && t.attachments.length) parts.push(attachments(t.attachments));
      if (!parts.length) {
        parts.push(`<div class="muted">${
          t.status === "failed" ? "No captured output." :
          t.status === "not_run" ? "The test process ended before this test reported a result." :
          "Nothing recorded. Use airbug::report::step / assert_equal / attach_* to add detail."
        }</div>`);
      }
      return `<div class="atr-detail">${parts.join("")}</div>`;
    }

    function step(s) {
      const status = s.status === "unfinished" ? "not_run" : s.status;
      const inner = [];
      if (s.comparisons && s.comparisons.length) inner.push(s.comparisons.map(comparison).join(""));
      if (s.attachments && s.attachments.length) inner.push(attachments(s.attachments));
      if (s.flaky && s.flaky.length) inner.push(`<div class="muted">flaky: ${esc(s.flaky.join("; "))}</div>`);
      const children = s.children && s.children.length ? `<ul>${s.children.map(step).join("")}</ul>` : "";
      return `<li class="atr-step">
        <div class="atr-step-line"><span class="atr-dot ${esc(status)}"></span>${esc(s.name)}
          ${s.duration_s != null ? `<span class="dur">${esc(fmtDuration(s.duration_s))}</span>` : ""}
          ${s.status === "unfinished" ? `<span class="atr-badge flaky">unfinished</span>` : ""}</div>
        ${inner.join("")}${children}
      </li>`;
    }

    function comparison(c) {
      const diff = c.passed ? null : lineDiff(c.expected, c.actual);
      let body;
      if (c.passed) {
        body = `<div class="ctx">${esc(c.actual)}</div>`;
      } else if (diff) {
        body = diff.map(d => {
          const cls = d.op === "-" ? "del" : d.op === "+" ? "add" : "ctx";
          return `<div class="${cls}">${esc(d.op)} ${esc(d.text)}</div>`;
        }).join("");
      } else {
        body = `<div class="del">- ${esc(c.expected)}</div><div class="add">+ ${esc(c.actual)}</div>`;
      }
      return `<div class="atr-diff">
        <div class="atr-diff-head"><span>${esc(c.name)}</span>
          <span class="${c.passed ? "ok" : "bad"}">${c.passed ? "equal" : "− expected  + actual"}${c.truncated ? " · truncated" : ""}</span></div>
        <div class="atr-diff-body">${body}</div>
      </div>`;
    }

    function isText(mediaType) {
      return /^text\/|json|xml|yaml|csv/.test(String(mediaType || ""));
    }

    function attachments(list) {
      return `<div class="atr-atts">${list.map(a => {
        const href = files() + "events/" + encodeURIComponent(a.file);
        const key = a.file;
        const loaded = state.attachmentText.get(key);
        const view = isText(a.media_type)
          ? `<button type="button" data-view="${esc(key)}">${loaded != null ? "hide" : "view"}</button>` : "";
        return `<div class="atr-att">📎 <a href="${esc(href)}" download="${esc(a.name)}">${esc(a.name)}</a>
          <span class="atr-count">${esc(a.media_type)} · ${esc(a.size)} B</span>${view}
          ${loaded != null ? `<pre>${esc(loaded)}</pre>` : ""}</div>`;
      }).join("")}</div>`;
    }

    function paintUnattributed() {
      const r = report();
      const el = part("extra");
      const items = (r && r.unattributed) || [];
      if (!items.length) {
        el.innerHTML = "";
        return;
      }
      el.innerHTML = `<div class="atr-block"><h3>steps outside a test thread</h3>${items.map(u => `
        <div class="atr-suite"><div class="atr-detail">
          <div class="muted">${esc([u.suite, u.thread].filter(Boolean).join(" · ") || "unknown origin")}</div>
          ${u.comparisons && u.comparisons.length ? u.comparisons.map(comparison).join("") : ""}
          ${u.steps && u.steps.length ? `<ul class="atr-steps">${u.steps.map(step).join("")}</ul>` : ""}
          ${u.attachments && u.attachments.length ? attachments(u.attachments) : ""}
        </div></div>`).join("")}</div>`;
    }

    function focusTest(id) {
      state.open.add(id);
      const t = report() && (report().tests || []).find(x => x.id === id);
      const filter = allFilters().find(f => f.id === state.filter);
      if (t && filter && !filter.test(t)) {
        state.filter = "all";
        state.filterTouched = true;
      }
      if (t) state.suiteToggle.set(t.suite, true);
      paint();
      const row = Array.from(root.querySelectorAll("[data-test]")).find(n => n.getAttribute("data-test") === id);
      if (row && row.scrollIntoView) row.scrollIntoView({ block: "center" });
    }

    root.addEventListener("click", ev => {
      const target = ev.target.closest("button, a");
      if (!target || !root.contains(target)) return;
      if (target.hasAttribute("data-filter")) {
        state.filter = target.getAttribute("data-filter");
        state.filterTouched = true;
        paintHead();
        paintList();
      } else if (target.hasAttribute("data-toggle-test")) {
        const id = target.getAttribute("data-toggle-test");
        if (state.open.has(id)) state.open.delete(id); else state.open.add(id);
        paintList();
      } else if (target.hasAttribute("data-toggle-suite")) {
        const id = target.getAttribute("data-toggle-suite");
        const expanded = target.getAttribute("aria-expanded") === "true";
        state.suiteToggle.set(id, !expanded);
        paintList();
      } else if (target.hasAttribute("data-goto")) {
        focusTest(target.getAttribute("data-goto"));
      } else if (target.hasAttribute("data-view")) {
        const key = target.getAttribute("data-view");
        if (state.attachmentText.has(key)) {
          state.attachmentText.delete(key);
          paint();
          return;
        }
        const href = files() + "events/" + encodeURIComponent(key);
        fetch(href, { cache: "no-store" })
          .then(res => (res.ok ? res.text() : Promise.reject(new Error(res.status))))
          .then(text => {
            state.attachmentText.set(key, text.length > 200000 ? text.slice(0, 200000) + "\n… (truncated)" : text);
            paint();
          })
          .catch(() => {
            state.attachmentText.set(key, "(could not load — open the file link instead)");
            paint();
          });
      }
    });
    searchEl.addEventListener("input", () => {
      state.query = searchEl.value;
      paintList();
    });

    return { update, focusTest };
  }

  global.AirbugTestRun = { createView, summarize, lineDiff, fmtDuration, fmtAgo, fmtWhen, progressBar, esc };
})(typeof window !== "undefined" ? window : globalThis);
