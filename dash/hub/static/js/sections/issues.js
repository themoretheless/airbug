// Signals › Issues: grouped airbug-err events with resolve / ignore / reopen.

import { enc, getJson, post } from "../api.js";
import { $, escapeAttr, escapeHtml, toast } from "../dom.js";

const ACTIONS = ["resolve", "ignore", "reopen"];
let selected = null;

function row(i) {
  const status = i.status || "unresolved";
  return `<div class="issue-row${i.id === selected ? " active" : ""}" data-id="${escapeAttr(i.id)}" role="button" tabindex="0">
    <div class="issue-id">${escapeHtml(i.id)}</div>
    <div>
      <div class="issue-title">${escapeHtml(i.title || "")}
        <span class="issue-status ${escapeAttr(status)}">${escapeHtml(status)}</span>
      </div>
      <div class="issue-meta-line">${escapeHtml([i.level, i.service, i.environment, i.release].filter(Boolean).join(" · "))}</div>
      <div class="issue-meta-line">first ${escapeHtml(i.first_seen || "")} · last ${escapeHtml(i.last_seen || "")}</div>
    </div>
    <div class="issue-count">×${escapeHtml(String(i.count || 0))}</div>
  </div>`;
}

export async function refresh() {
  const data = await getJson("issues");
  $("issues-meta").textContent = data.note || "";
  const issues = data.issues || [];
  const panel = $("issues-panel");
  if (!issues.length) {
    panel.innerHTML = `<p class="issues-empty">${escapeHtml(data.note || "No issues.")}</p>`;
    return;
  }
  panel.innerHTML = issues.map(row).join("");
  panel.querySelectorAll(".issue-row").forEach(el => {
    const open = () => openIssue(el.getAttribute("data-id"));
    el.addEventListener("click", open);
    el.addEventListener("keydown", ev => {
      if (ev.key === "Enter" || ev.key === " ") {
        ev.preventDefault();
        open();
      }
    });
  });
}

async function act(id, action) {
  try {
    await post(`issues/${enc(id)}/${action}`);
  } catch {
    toast("action failed");
    return;
  }
  toast(action + " · " + id);
  await openIssue(id);
}

async function openIssue(id) {
  if (!id) return;
  selected = id;
  const issue = await getJson("issues/" + enc(id)).catch(() => null);
  if (!issue) {
    toast("issue not found");
    return;
  }
  $("issue-detail").hidden = false;
  const stored = Array.isArray(issue.events) ? issue.events.length : 0;
  $("issue-detail-title").textContent = `${issue.id} · ${issue.title || ""} · ${stored} stored / ${issue.count || 0} lifetime`;
  const payload = stored ? { events: issue.events, last_event: issue.last_event } : (issue.last_event || issue);
  $("issue-detail-body").textContent = JSON.stringify(payload, null, 2);
  const actions = $("issue-actions");
  actions.innerHTML = ACTIONS.map(a => `<button type="button" data-action="${a}">${a}</button>`).join("");
  actions.querySelectorAll("button[data-action]").forEach(btn => {
    btn.addEventListener("click", () => act(id, btn.getAttribute("data-action")));
  });
  await refresh();
}

export default {
  cats: ["issues"],
  show: () => refresh().catch(err => { $("issues-meta").textContent = String(err); }),
};
