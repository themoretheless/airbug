// DOM helpers shared by every section: escaping, element lookup, toast, copy-to-clipboard.

export function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c]));
}

export function escapeAttr(s) {
  return escapeHtml(s).replace(/\n/g, " ");
}

export const $ = id => document.getElementById(id);

let toastTimer = null;

export function toast(text) {
  const el = $("toast");
  el.textContent = text;
  el.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.classList.remove("show"), 2200);
}

export async function copy(text) {
  try {
    await navigator.clipboard.writeText(text);
    toast("copied: " + text);
  } catch {
    toast(text);
  }
}

/** Wires every `button[data-cmd]` under `el` to copy its command. */
export function bindCopy(el) {
  el.querySelectorAll("button[data-cmd]").forEach(btn => {
    btn.onclick = ev => { ev.preventDefault(); copy(btn.getAttribute("data-cmd")); };
  });
}

/** A copyable shell command as a list item. */
export function commandItem(cmd, label = "copy") {
  return `<li><button type="button" data-cmd="${escapeAttr(cmd)}">${escapeHtml(label)}</button> <code>${escapeHtml(cmd)}</code></li>`;
}

/** Toggle buttons rendered from `[id, label]` pairs; `onPick(id)` runs on click. */
export function renderToggles(el, items, isOn, onPick, attr = "data-id") {
  el.innerHTML = items.map(([id, label]) =>
    `<button type="button" class="${isOn(id) ? "on" : ""}" ${attr}="${escapeAttr(id)}">${escapeHtml(label)}</button>`
  ).join("");
  el.querySelectorAll(`button[${attr}]`).forEach(btn => {
    btn.addEventListener("click", () => onPick(btn.getAttribute(attr)));
  });
}

/** Multi-select toggle that never empties the set. */
export function toggleKeepOne(set, id) {
  if (set.has(id)) {
    if (set.size > 1) set.delete(id);
  } else {
    set.add(id);
  }
}
