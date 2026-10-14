// Entry point: registers sections with the shell and starts the background feeds.

import { $ } from "./dom.js";
import { every, register, showRoute, start } from "./shell.js";
import { fetchTimeline } from "./runs.js";
import bench from "./sections/bench.js";
import health, { refresh as refreshStatus } from "./sections/health.js";
import issues, { refresh as refreshIssues } from "./sections/issues.js";
import logs, { init as initLogs, refresh as refreshLogs } from "./sections/logs.js";
import metrics, { refresh as refreshMetrics } from "./sections/metrics.js";
import now from "./sections/now.js";
import runs from "./sections/runs.js";
import tests from "./sections/tests.js";

[now, runs, tests, bench, issues, logs, metrics, health].forEach(register);
initLogs();
start();

const FEEDS = [
  [15000, () => refreshStatus(reloadAll).catch(err => { $("meta").textContent = "failed to load /api/v1/status: " + err; })],
  [4000, refreshLogs],
  [4000, refreshMetrics],
  [5000, refreshIssues],
  [6000, () => fetchTimeline("", 40)],
];

function reloadAll() {
  FEEDS.forEach(([, feed]) => Promise.resolve(feed()).catch(() => {}));
  showRoute();
}

FEEDS.forEach(([ms, feed]) => every(ms, feed));
