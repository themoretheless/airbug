(() => {
  const $ = id => document.getElementById(id);
  const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const badge = status => `<span class="run-badge status-${['passed','failed','broken','running','queued','ignored','cancelled'].includes(status) ? status : 'unknown'}">${esc(status)}</span>`;
  const htmlCache = new Map();
  function setHtml(id, value) { if (htmlCache.get(id) !== value) { $(id).innerHTML = value; htmlCache.set(id, value); } }
  const caseDuration = test => duration(test.status === 'running' && test.started_ms ? Math.max(0,(Date.now()-test.started_ms)/1000) : test.duration);
  const duration = value => value == null ? '—' : `${Number(value).toFixed(3)} s`;
  let activeId = null, selectedCase = null, data = null, busy = false;
  let selectionVersion = 0;
  async function get(url) {
    const response = await fetch(url, {cache:'no-store'});
    if (!response.ok) throw new Error(`Could not load run (${response.status})`);
    return response.json();
  }
  function route() { return location.hash.replace(/^#\/?/, '').split('/'); }
  function renderCases() {
    if (!data) return;
    const query = $('runs-search').value.toLowerCase(), status = $('runs-status').value;
    const visible = data.tests.map((test,index) => ({...test,index})).filter(test =>
      (!status || test.status === status) && `${test.suite} ${test.name}`.toLowerCase().includes(query));
    const caseHtml = visible.length ? `<table><thead><tr><th>Case</th><th>Status</th><th>Duration</th>${data.kind === 'bench' ? '<th>Median ns/op</th>' : ''}</tr></thead><tbody>${visible.map(test =>
      `<tr class="${selectedCase === test.index ? 'selected' : ''}"><td><button data-case="${test.index}">${esc(test.name)}</button><small>${esc(test.suite)}</small></td><td>${badge(test.status)}</td><td>${caseDuration(test)}</td>${data.kind === 'bench' ? `<td>${test.ns_per_op == null ? '—' : Number(test.ns_per_op).toFixed(2)}</td>` : ''}</tr>`).join('')}</tbody></table>` : '<p class="runs-empty">No cases match this filter.</p>';
    setHtml('runs-cases', caseHtml);
    $('runs-cases').querySelectorAll('[data-case]').forEach(button => button.onclick = () => {
      selectedCase = Number(button.dataset.case); location.hash = '#/launches/' + activeId; selectionVersion++; renderCases(); refreshCase().catch(showError);
    });
  }
  async function refreshCase() {
    if (selectedCase == null || !activeId) { setHtml('runs-case-detail', '<p>Select a case to inspect its output, steps and attachments.</p>'); return; }
    const id = activeId, index = selectedCase, version = selectionVersion;
    const value = await get(`/api/v1/launches/${encodeURIComponent(id)}/cases/${index}`);
    if (activeId !== id || selectedCase !== index || version !== selectionVersion) return;
    const test = value.case, events = value.events || [];
    const ends = new Map(events.filter(e => e.type === 'step_end').map(e => [e.id,e]));
    const base = `/api/v1/launches/${encodeURIComponent(id)}/cases/${index}/attachments/`;
    function eventHtml(event) {
      if (event.type === 'comparison') return `<div class="run-comparison">${badge(event.passed ? 'passed' : 'failed')} <strong>${esc(event.name)}</strong><div><section><small>Expected</small><pre>${esc(event.expected)}</pre></section><section><small>Actual</small><pre>${esc(event.actual)}</pre></section></div>${event.truncated ? '<small>Comparison truncated by reporter</small>' : ''}</div>`;
      if (event.type === 'attachment') return `<p><a download="${esc(event.name)}" href="${base}${encodeURIComponent(event.file)}">↓ ${esc(event.name)}</a> <small>${esc(event.mediaType)} · ${esc(event.size)} bytes</small></p>`;
      if (event.type === 'flaky') return `<p>Flaky: ${esc(event.reason)}</p>`;
      return '';
    }
    const children = new Map();
    events.forEach(event => { const parent = event.parent ?? null; if (!children.has(parent)) children.set(parent, []); children.get(parent).push(event); });
    function tree(parent, depth=0) {
      if (depth > 64) return '<p>Step depth limit reached</p>';
      return (children.get(parent) || []).map(event => {
        if (event.type !== 'step_start') return eventHtml(event);
        const end = ends.get(event.id);
        return `<section class="run-step"><h4>${badge(end?.status || (test.status === 'running' ? 'running' : 'broken'))} ${esc(event.name)} <small>${duration(end?.duration)}</small></h4>${tree(event.id,depth+1)}</section>`;
      }).join('');
    }
    const content = `<h3>${esc(test.name)}</h3><p>${badge(test.status)} · ${caseDuration(test)} ${data.kind === 'test' ? 'including process startup' : 'including calibration and warmup'}</p>
      ${(value.warnings || []).map(w => `<p class="runs-error">${esc(w)}</p>`).join('')}
      <div>${tree(null) || '<p class="runs-empty">No structured steps recorded for this case.</p>'}</div>
      <h4>Output</h4><pre>${esc(test.output || (test.status === 'running' ? 'Output will appear when the process finishes.' : 'No captured output.'))}</pre>`;
    setHtml('runs-case-detail', content);
  }
  function showError(error) { $('runs-connection').textContent = String(error); $('runs-connection').className = 'runs-error'; }
  async function refresh() {
    if (route()[0] !== 'launches' || busy) return;
    busy = true;
    try {
      const history = await get('/api/v1/launches');
      if (route()[0] !== 'launches') return;
      const kind = $('runs-kind').value;
      const runs = history.runs.filter(run => !kind || run.kind === kind);
      const requested = route()[1];
      const id = requested || runs[0]?.id || null;
      if (id !== activeId) { activeId = id; selectedCase = null; data = null; selectionVersion++; }
      $('runs-history').innerHTML = runs.length ? runs.map(run => `<a class="run-history-card ${run.id === id ? 'active' : ''}" href="#/launches/${encodeURIComponent(run.id)}"><small>${esc(run.kind)} · ${new Date(run.created_ms).toLocaleString()}</small><strong>${esc(run.title)}</strong>${badge(run.state)}<small>${Object.entries(run.counts || {}).map(([key,n]) => `${n} ${esc(key)}`).join(' · ') || 'Preparing'}</small></a>`).join('') : '<p class="runs-empty">No runs yet. Start airbug_report or an Airbug cargo bench target.</p>';
      if (!id) { $('runs-summary').innerHTML = '<h3>Ready for your first run</h3><p>Launch tests with airbug_report. Airbug benchmarks record their progress automatically.</p>'; setHtml('runs-cases', ''); setHtml('runs-case-detail', ''); return; }
      const loaded = await get(`/api/v1/launches/${encodeURIComponent(id)}`);
      if (route()[0] !== 'launches' || (route()[1] && route()[1] !== id)) return;
      data = loaded;
      const counts = {};
      data.tests.forEach(test => counts[test.status] = (counts[test.status] || 0) + 1);
      const done = data.tests.filter(test => !['queued','running'].includes(test.status)).length;
      const age = (Date.now() - data.updated_ms) / 1000;
      $('runs-summary').innerHTML = `<h2>${esc(data.title)} ${badge(data.state)}</h2><div class="run-counts">${Object.entries(counts).map(([key,n]) => `<span><strong>${n}</strong> ${esc(key)}</span>`).join('')}</div><progress max="${Math.max(1,data.tests.length)}" value="${done}" aria-label="Completed cases"></progress><p>${done} / ${data.tests.length} cases · ${duration(data.state === 'running' ? Math.max(0,(Date.now()-data.created_ms)/1000) : data.duration)}</p><pre class="run-message">${esc(data.message)}</pre>${data.state === 'running' && age > 60 ? '<p>Still awaiting a case result. If the runner was forcibly terminated, this record may be incomplete.</p>' : ''}`;
      renderCases();
      await refreshCase();
      $('runs-connection').className = history.errors.length ? 'runs-error' : '';
      $('runs-connection').textContent = history.errors.length ? history.errors.join(' · ') : `Updated ${new Date().toLocaleTimeString()} · latest 100 launches`;
    } catch (error) { showError(error); } finally { busy = false; }
  }
  $('runs-search').addEventListener('input', renderCases);
  $('runs-status').addEventListener('change', renderCases);
  $('runs-kind').addEventListener('change', () => { location.hash = '#/launches'; refresh(); });
  window.addEventListener('hashchange', refresh);
  setInterval(refresh, 1000);
  refresh();
})();
