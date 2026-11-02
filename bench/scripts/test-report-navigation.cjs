// Execute the shipped report script with a small DOM test double. This checks
// navigation state transitions, not browser rendering or visual acceptance.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

function element(extra = {}) {
    return Object.assign({value: '', textContent: '', hidden: false, children: [],
        addEventListener() {}}, extra);
}
const ids = Object.fromEntries(['search', 'outcome', 'count', 'collection-tools',
    'expand', 'collapse', 'section-search', 'section-links', 'section-count',
    'report-index', 'print'].map(id => [id, element()]));
const article = element({dataset: {outcome: 'regression'}, textContent: 'case result'});
const outer = element({tagName: 'DETAILS', open: false, parentElement: article});
const inner = element({tagName: 'DETAILS', open: false, parentElement: outer});
let focused = 0, scrolled = 0;
ids['chart / λ'] = element({parentElement: inner,
    focus(options) { assert.equal(options.preventScroll, true); focused++; },
    scrollIntoView() { scrolled++; }});
const events = {};
const location = {hash: '#chart%20%2F%20%CE%BB'};
const document = {
    getElementById(id) { return ids[id] || null; },
    querySelector() { return article; },
    querySelectorAll(selector) {
        return selector === 'article[data-outcome]' ? [article] : [];
    }
};
const html = fs.readFileSync(path.join(__dirname, '../src/report-template.html'), 'utf8');
const script = html.match(/<script type="text\/javascript">([\s\S]*?)<\/script>/)[1];
ids.search.value = 'does not match';
ids.outcome.value = 'error';
vm.runInNewContext(script, {document, location,
    window: {addEventListener(name, handler) { events[name] = handler; }}});
function assertRevealed() {
    assert.equal(ids.search.value, '');
    assert.equal(ids.outcome.value, '');
    assert.equal(article.hidden, false);
    assert.equal(outer.open, true);
    assert.equal(inner.open, true);
    assert.equal(ids['chart / λ'].tabIndex, -1);
}
assertRevealed();
assert.equal(focused, 1);
assert.equal(scrolled, 1);
assert.equal(typeof events.hashchange, 'function');

// Back/forward after filtering and collapsing must reveal the target again.
inner.open = outer.open = false;
article.hidden = true;
ids.search.value = 'other case';
ids.outcome.value = 'error';
events.hashchange();
assertRevealed();
assert.equal(focused, 2);
assert.equal(scrolled, 2);

// Invalid, missing and empty fragments must preserve the user's current view.
for (const hash of ['#%E0%A4', '#missing', '', '#']) {
    location.hash = hash;
    ids.search.value = 'keep filter';
    ids.outcome.value = 'error';
    inner.open = false;
    events.hashchange();
    assert.equal(ids.search.value, 'keep filter');
    assert.equal(ids.outcome.value, 'error');
    assert.equal(inner.open, false);
    assert.equal(focused, 2);
    assert.equal(scrolled, 2);
}
console.log('Report navigation: bookmark and history reveal, nested details, filters, Unicode and invalid fragments passed');
