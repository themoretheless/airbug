//! Read side of the run store: one timeline of test runs (`.airbug/runs`) and bench runs
//! (GUID registry + `.airbug-bench` scan), plus per-test history across test runs.
//!
//! The hub never writes here — runners own their directories; see `testrun::model`.
use crate::{
    error::{HubError, Result},
    runs::RunStore,
    scan,
    testrun::model::{
        Git, KIND_BENCH, MANIFEST_SCHEMA, Manifest, Progress, RunState, TestStatus, Totals,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// A running test run whose progress has not moved for this long is shown as stale.
const STALE_AFTER_MS: u64 = 10 * 60 * 1000;
/// Test runs (current included) that feed history strips and flaky detection.
const HISTORY_RUNS: usize = 12;
const MAX_LISTED_CHANGES: usize = 200;

#[derive(Debug, Clone, Serialize)]
pub struct RunItem {
    pub run_id: String,
    /// `test` or `bench`.
    pub kind: &'static str,
    pub title: String,
    pub state: String,
    pub started_at_ms: u64,
    pub updated_at_ms: u64,
    /// `#/…` routes stay in the dashboard; anything else opens as a file.
    pub href: String,
    pub stale: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<Git>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totals: Option<Totals>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    /// Bench live progress (`completed` / `total` / `variant`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<Value>,
}

use airbug_run::store::read_json as read;
pub use airbug_run::store::runs_dir;

fn state_str<T: Serialize>(state: &T) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

fn now_ms() -> u64 {
    crate::testrun::model::now_ms()
}

fn mtime_ms(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Test runs, newest first.
pub fn test_runs(root: &Path, limit: usize) -> Vec<RunItem> {
    let Ok(entries) = fs::read_dir(runs_dir(root)) else {
        return Vec::new();
    };
    let mut items: Vec<RunItem> = entries
        .flatten()
        .filter_map(|entry| test_run_item(&entry.path()))
        .collect();
    items.sort_by_key(|a| std::cmp::Reverse(a.started_at_ms));
    items.truncate(limit);
    items
}

fn test_run_item(dir: &Path) -> Option<RunItem> {
    let manifest: Manifest = read(&dir.join("manifest.json"))?;
    if manifest.schema != MANIFEST_SCHEMA || manifest.kind != "test" {
        return None;
    }
    let progress: Option<Progress> = read(&dir.join("progress.json"));
    let state = progress.as_ref().map(|p| p.state).unwrap_or_default();
    let updated = progress
        .as_ref()
        .map(|p| p.updated_at_ms)
        .unwrap_or(manifest.started_at_ms);
    let stale = !state.is_finished() && now_ms().saturating_sub(updated) > STALE_AFTER_MS;
    Some(RunItem {
        href: crate::rundetail::href(&manifest.run_id),
        run_id: manifest.run_id,
        kind: "test",
        title: manifest.title,
        state: state_str(&state),
        started_at_ms: manifest.started_at_ms,
        updated_at_ms: updated,
        stale,
        git: Some(manifest.git),
        totals: progress.as_ref().map(|p| p.totals.clone()),
        duration_s: progress.as_ref().and_then(|p| p.duration_s),
        activity: progress.map(|p| p.activity).filter(|a| !a.is_empty()),
        progress: None,
    })
}

/// Bench runs: registered GUID sessions first, then any other `run.json` under `.airbug-bench`.
pub fn bench_runs(root: &Path, registry: &RunStore) -> Vec<RunItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    if let Ok(list) = registry.list() {
        for run in list.runs {
            seen.insert(format!("runs/{}", run.run_id));
            items.push(RunItem {
                href: crate::rundetail::href(&run.run_id),
                run_id: run.run_id,
                kind: "bench",
                title: run.title,
                state: state_str(&run.state),
                started_at_ms: unix_stamp_ms(&run.created_at),
                updated_at_ms: unix_stamp_ms(&run.updated_at),
                stale: false,
                git: None,
                totals: None,
                duration_s: None,
                activity: None,
                progress: run.progress,
            });
        }
    }
    items.extend(announced_bench_runs(root, &mut seen));
    for row in scan::list_bench_dirs(root).runs {
        let id = row.id.replace('\\', "/");
        // Announced sessions are listed once, cells of a matrix included.
        if seen
            .iter()
            .any(|known| id == *known || id.starts_with(&format!("{known}/")))
        {
            continue;
        }
        let run_json = root.join(&row.rel);
        let dir = run_json.parent().map(Path::to_path_buf).unwrap_or_default();
        let state = read::<Value>(&dir.join("status-final.json"))
            .and_then(|v| v.get("state").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "complete".into());
        let when = mtime_ms(&run_json);
        items.push(RunItem {
            href: crate::rundetail::href(&id),
            title: id.clone(),
            run_id: id,
            kind: "bench",
            state,
            started_at_ms: when,
            updated_at_ms: when,
            stale: false,
            git: None,
            totals: None,
            duration_s: None,
            activity: None,
            progress: None,
        });
    }
    items
}

/// Bench sessions announced in the shared run store by `cargo airbug-bench`.
///
/// The session's files stay in its own output directory; while it runs, live counts come from
/// the `progress.json` files the runner keeps there. Directories under `.airbug-bench` are added to
/// `seen` so the scan below does not list them twice.
fn announced_bench_runs(root: &Path, seen: &mut HashSet<String>) -> Vec<RunItem> {
    let bench_store = root.join(".airbug-bench");
    airbug_run::store::list(&runs_dir(root), KIND_BENCH)
        .into_iter()
        .filter_map(|(dir, manifest)| {
            let artifacts = manifest.artifacts.as_deref().map(PathBuf::from);
            let under = artifacts
                .as_deref()
                .and_then(|a| a.strip_prefix(&bench_store).ok())
                .map(|p| p.display().to_string().replace('\\', "/"));
            if let Some(under) = &under
                && !seen.insert(under.clone())
            {
                return None;
            }
            let progress = airbug_run::store::progress(&dir);
            let state = progress.as_ref().map(|p| p.state).unwrap_or_default();
            let live = (!state.is_finished())
                .then(|| artifacts.as_deref().and_then(live_counts))
                .flatten();
            let updated = match &live {
                Some((_, at)) => *at,
                None => progress
                    .as_ref()
                    .map(|p| p.updated_at_ms)
                    .unwrap_or(manifest.started_at_ms),
            };
            let live = live.map(|(counts, _)| counts);
            let counts = live.or_else(|| {
                progress
                    .as_ref()
                    .filter(|p| p.totals.total > 0)
                    .map(|p| json!({"completed": p.totals.completed, "total": p.totals.total}))
            });
            let href = crate::rundetail::href(&manifest.run_id);
            Some(RunItem {
                href,
                run_id: manifest.run_id,
                kind: "bench",
                title: manifest.title,
                state: state_str(&state),
                started_at_ms: manifest.started_at_ms,
                updated_at_ms: updated,
                stale: !state.is_finished() && now_ms().saturating_sub(updated) > STALE_AFTER_MS,
                git: None,
                totals: None,
                duration_s: progress.as_ref().and_then(|p| p.duration_s),
                activity: None,
                progress: counts,
            })
        })
        .collect()
}

/// Process counts of a running session and when they last moved: its own `progress.json`, or
/// the sum over its cells (a matrix, one directory per combination).
fn live_counts(artifacts: &Path) -> Option<(Value, u64)> {
    let own = artifacts.join("progress.json");
    if let Some(v) = read::<Value>(&own) {
        return Some((v, mtime_ms(&own)));
    }
    let (mut completed, mut total, mut at) = (0, 0, 0);
    for cell in std::fs::read_dir(artifacts).ok()?.flatten() {
        let file = cell.path().join("progress.json");
        if let Some(v) = read::<Value>(&file) {
            completed += v["completed"].as_u64().unwrap_or(0);
            total += v["total"].as_u64().unwrap_or(0);
            at = at.max(mtime_ms(&file));
        }
    }
    (total > 0).then(|| (json!({"completed": completed, "total": total}), at))
}

/// Resolve `/runs/<id>/files/<rel>` to a file of an announced bench session, wherever its
/// `--output` directory is.
pub fn artifact_file(root: &Path, id: &str, rel: &str) -> Option<PathBuf> {
    let plain = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    };
    if !plain(id) || id.contains("..") || rel.split('/').any(|seg| !plain(seg) || seg == "..") {
        return None;
    }
    let manifest = airbug_run::store::manifest(&runs_dir(root).join(id))?;
    if manifest.kind != KIND_BENCH {
        return None;
    }
    let dir = PathBuf::from(manifest.artifacts?).canonicalize().ok()?;
    let path = dir.join(rel).canonicalize().ok()?;
    (path.starts_with(&dir) && path.is_file()).then_some(path)
}

/// `"1712345678s unix"` or ISO-8601 (`YYYY-MM-DDTHH:MM:SS(.mmm)Z`) → milliseconds.
fn unix_stamp_ms(stamp: &str) -> u64 {
    let s = stamp.trim();
    if let Some(secs_str) = s.strip_suffix("s unix") {
        return secs_str.parse::<u64>().map(|n| n * 1000).unwrap_or(0);
    }
    parse_iso_ms(s).unwrap_or(0)
}

fn parse_iso_ms(s: &str) -> Option<u64> {
    let s = s.strip_suffix('Z')?;
    let (date, time) = s.split_once('T')?;
    let mut d_parts = date.split('-');
    let y: i64 = d_parts.next()?.parse().ok()?;
    let mo: i64 = d_parts.next()?.parse().ok()?;
    let d: i64 = d_parts.next()?.parse().ok()?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    let mut t_parts = time.split(':');
    let h: u64 = t_parts.next()?.parse().ok()?;
    let mi: u64 = t_parts.next()?.parse().ok()?;
    let sec_part = t_parts.next()?;
    let (sec_s, ms) = match sec_part.split_once('.') {
        Some((sec_s, frac)) => {
            let mut frac_ms = 0u64;
            for (i, ch) in frac.chars().take(3).enumerate() {
                let digit = ch.to_digit(10)? as u64;
                frac_ms += digit * 10u64.pow(2 - i as u32);
            }
            (sec_s, frac_ms)
        }
        None => (sec_part, 0),
    };
    let sec: u64 = sec_s.parse().ok()?;
    let y_adj = if mo <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = (y_adj - era * 400) as u64;
    let mp = (if mo > 2 { mo - 3 } else { mo + 9 }) as u64;
    let doy = (153 * mp + 2) / 5 + (d as u64) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + (doe as i64) - 719_468;
    if days < 0 {
        return None;
    }
    Some((days as u64) * 86_400_000 + (h * 3600 + mi * 60 + sec) * 1000 + ms)
}

/// Shared `airbug_report` and direct `cargo bench` case history.
fn case_runs(root: &Path) -> Vec<RunItem> {
    let Ok(history) = crate::test_runs::list(root) else {
        return Vec::new();
    };
    history["runs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|run| {
            let id = run["id"].as_str()?;
            let kind = match run["kind"].as_str()? {
                "test" => "test",
                "bench" => "bench",
                _ => return None,
            };
            let count = |key: &str| run["counts"][key].as_u64().unwrap_or(0) as usize;
            let passed = count("passed");
            let failed = count("failed") + count("broken");
            let ignored = count("ignored");
            let not_run = count("cancelled");
            let completed = passed + failed + ignored + not_run;
            let total = completed + count("queued") + count("running");
            let state = run["state"].as_str().unwrap_or("unknown").to_owned();
            let updated_at_ms = run["updated_ms"].as_u64().unwrap_or(0);
            Some(RunItem {
                run_id: id.into(),
                kind,
                title: run["title"].as_str().unwrap_or(id).into(),
                stale: state == "running"
                    && now_ms().saturating_sub(updated_at_ms) > STALE_AFTER_MS,
                state,
                started_at_ms: run["created_ms"].as_u64().unwrap_or(0),
                updated_at_ms,
                href: format!("#/run/{id}"),
                git: None,
                totals: Some(Totals {
                    total,
                    completed,
                    passed,
                    failed,
                    ignored,
                    not_run,
                }),
                duration_s: run["duration"].as_f64(),
                activity: None,
                progress: (kind == "bench")
                    .then(|| json!({"completed": completed, "total": total, "unit": "cases"})),
            })
        })
        .collect()
}

/// One timeline across kinds, newest first. `kind` filters (`test` / `bench`).
pub fn timeline(root: &Path, registry: &RunStore, kind: Option<&str>, limit: usize) -> Value {
    let mut runs = Vec::new();
    if kind.is_none_or(|k| k == "test") {
        runs.extend(test_runs(root, limit));
    }
    if kind.is_none_or(|k| k == "bench") {
        runs.extend(bench_runs(root, registry));
    }
    runs.extend(
        case_runs(root)
            .into_iter()
            .filter(|run| kind.is_none_or(|k| k == run.kind)),
    );
    // Live first, then newest.
    runs.sort_by(|a, b| {
        let live = |r: &RunItem| {
            !r.stale && matches!(r.state.as_str(), "building" | "running" | "registered")
        };
        live(b).cmp(&live(a)).then_with(|| {
            b.started_at_ms
                .max(b.updated_at_ms)
                .cmp(&a.started_at_ms.max(a.updated_at_ms))
        })
    });
    let live = runs
        .iter()
        .filter(|r| !r.stale && matches!(r.state.as_str(), "building" | "running"))
        .count();
    runs.truncate(limit);
    json!({
        "runs": runs,
        "live": live,
        "store": runs_dir(root).display().to_string(),
    })
}

#[derive(Debug, Default, Deserialize)]
struct LightReport {
    #[serde(default)]
    tests: Vec<LightTest>,
}

#[derive(Debug, Deserialize)]
struct LightTest {
    id: String,
    #[serde(default)]
    status: TestStatus,
}

fn status_char(status: Option<TestStatus>) -> char {
    match status {
        Some(TestStatus::Passed) => 'p',
        Some(TestStatus::Failed) => 'f',
        Some(TestStatus::Ignored) => 'i',
        Some(TestStatus::NotRun) => 'n',
        Some(TestStatus::Pending) | None => '-',
    }
}

/// `id` is a run UUID or `latest`.
pub fn resolve_test_run(root: &Path, id: &str) -> Result<String> {
    if id == "latest" {
        return test_runs(root, 1)
            .into_iter()
            .next()
            .map(|r| r.run_id)
            .ok_or_else(|| HubError::msg("not found: no test runs yet"));
    }
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(HubError::msg("invalid run_id"));
    }
    Ok(id.to_string())
}

/// Full test run for the dashboard: report + change set vs the previous run of the same
/// command + status history of unstable tests. `lite` drops the per-test payload (for the
/// "Now" page, which only needs totals, changes and the flaky list).
pub fn test_run_detail(root: &Path, id: &str, lite: bool) -> Result<Value> {
    let case_items = case_runs(root);
    let requested = if id == "latest" {
        test_runs(root, 1)
            .into_iter()
            .chain(case_items.iter().filter(|r| r.kind == "test").cloned())
            .max_by_key(|r| r.started_at_ms)
            .map(|r| r.run_id)
            .unwrap_or_else(|| id.into())
    } else {
        id.into()
    };
    if let Some(item) = case_items.iter().find(|r| r.run_id == requested) {
        return crate::test_runs::dashboard_detail(root, &requested, item, lite)
            .map_err(Into::into);
    }
    let id = resolve_test_run(root, &requested)?;
    let dir = runs_dir(root).join(&id);
    let item = test_run_item(&dir).ok_or_else(|| HubError::msg("not found"))?;
    let manifest: Value = read(&dir.join("manifest.json")).unwrap_or(Value::Null);
    let progress: Value = read(&dir.join("progress.json")).unwrap_or(Value::Null);
    let mut report: Value = read(&dir.join("report.json")).unwrap_or(Value::Null);
    if let Ok(mut parsed_report) =
        serde_json::from_value::<crate::testrun::model::Report>(report.clone())
    {
        crate::testrun::analytics::classify_defects(
            &mut parsed_report,
            &crate::testrun::analytics::load_rules(root),
        );
        if let Ok(val) = serde_json::to_value(&parsed_report) {
            report = val;
        }
    }
    let current: LightReport = serde_json::from_value(report.clone()).unwrap_or_default();

    let older: Vec<RunItem> = test_runs(root, 500)
        .into_iter()
        .filter(|r| {
            r.run_id != item.run_id
                && r.title == item.title
                && r.started_at_ms < item.started_at_ms
                && matches!(r.state.as_str(), "passed" | "failed")
        })
        .take(HISTORY_RUNS - 1)
        .collect();
    // Oldest → newest, current last.
    let mut window: Vec<(RunItem, HashMap<String, TestStatus>)> = older
        .into_iter()
        .rev()
        .map(|run| {
            let light: LightReport =
                read(&runs_dir(root).join(&run.run_id).join("report.json")).unwrap_or_default();
            let map = light.tests.into_iter().map(|t| (t.id, t.status)).collect();
            (run, map)
        })
        .collect();
    let current_map: HashMap<String, TestStatus> = current
        .tests
        .iter()
        .map(|t| (t.id.clone(), t.status))
        .collect();

    let previous = window.last().map(|(run, map)| (run.clone(), map.clone()));
    let finished = RunState::is_finished(
        serde_json::from_value::<RunState>(json!(item.state)).unwrap_or_default(),
    );
    let changes = previous.as_ref().map(|(run, prev)| {
        let mut new_failures = Vec::new();
        let mut fixed = Vec::new();
        let mut added = 0usize;
        for test in &current.tests {
            let before = prev.get(&test.id).copied();
            if before.is_none() && test.status != TestStatus::Pending {
                added += 1;
            }
            match (test.status, before) {
                (TestStatus::Failed, Some(TestStatus::Failed)) => {}
                (TestStatus::Failed, _) => new_failures.push(test.id.clone()),
                (TestStatus::Passed, Some(TestStatus::Failed)) => fixed.push(test.id.clone()),
                _ => {}
            }
        }
        let removed: Vec<&String> = if finished {
            let mut removed: Vec<&String> = prev
                .keys()
                .filter(|id| !current_map.contains_key(*id))
                .collect();
            removed.sort();
            removed
        } else {
            Vec::new()
        };
        json!({
            "previous_run_id": run.run_id,
            "previous_git": run.git,
            "previous_started_at_ms": run.started_at_ms,
            "new_failures": new_failures.iter().take(MAX_LISTED_CHANGES).collect::<Vec<_>>(),
            "new_failures_count": new_failures.len(),
            "fixed": fixed.iter().take(MAX_LISTED_CHANGES).collect::<Vec<_>>(),
            "fixed_count": fixed.len(),
            "added_count": added,
            "removed": removed.iter().take(MAX_LISTED_CHANGES).collect::<Vec<_>>(),
            "removed_count": removed.len(),
        })
    });

    window.push((item.clone(), current_map));
    let mut strips = serde_json::Map::new();
    let mut flaky = Vec::new();
    for test in &current.tests {
        let strip: String = window
            .iter()
            .map(|(_, map)| status_char(map.get(&test.id).copied()))
            .collect();
        let fails = strip.matches('f').count();
        let passes = strip.matches('p').count();
        if fails > 0 && passes > 0 {
            flaky.push(json!({ "id": test.id, "fails": fails, "passes": passes }));
        }
        if fails > 0 || strip.contains('n') {
            strips.insert(test.id.clone(), Value::String(strip));
        }
    }
    let runs: Vec<Value> = window
        .iter()
        .map(|(run, _)| {
            json!({
                "run_id": run.run_id,
                "state": run.state,
                "started_at_ms": run.started_at_ms,
                "short": run.git.as_ref().and_then(|g| g.short.clone()),
            })
        })
        .collect();

    let rerun = crate::rerun::describe(&manifest, &report);
    let (report, strips) = if lite {
        (Value::Null, serde_json::Map::new())
    } else {
        (report, strips)
    };
    Ok(json!({
        "run_id": item.run_id,
        "item": item,
        "manifest": manifest,
        "progress": progress,
        "report": report,
        "changes": changes,
        "history": { "runs": runs, "tests": strips },
        "flaky": flaky,
        "rerun": rerun,
        "files": format!("/runs/{id}/"),
    }))
}

/// Resolve `/runs/<id>/<rel>` to a file inside that run directory.
pub fn run_file(root: &Path, id: &str, rel: &str) -> Option<PathBuf> {
    uuid::Uuid::parse_str(id).ok()?;
    if rel.is_empty()
        || rel.starts_with('/')
        || rel.contains('\\')
        || rel.split('/').any(|seg| seg == ".." || seg.is_empty())
    {
        return None;
    }
    let dir = runs_dir(root).join(id).canonicalize().ok()?;
    let path = dir.join(rel).canonicalize().ok()?;
    (path.starts_with(&dir) && path.is_file()).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testrun::model::{REPORT_SCHEMA, Report, TestCase};

    fn write_run(
        root: &Path,
        id: &str,
        started: u64,
        state: RunState,
        tests: &[(&str, TestStatus)],
    ) {
        let dir = runs_dir(root).join(id);
        fs::create_dir_all(&dir).unwrap();
        let manifest = Manifest {
            schema: MANIFEST_SCHEMA.into(),
            kind: "test".into(),
            run_id: id.into(),
            title: "cargo test".into(),
            started_at_ms: started,
            ..Manifest::default()
        };
        fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let mut report = Report {
            schema: REPORT_SCHEMA.into(),
            run_id: id.into(),
            state,
            tests: tests
                .iter()
                .map(|(name, status)| TestCase {
                    id: format!("s::{name}"),
                    suite: "s".into(),
                    name: name.to_string(),
                    status: *status,
                    ..TestCase::default()
                })
                .collect(),
            ..Report::default()
        };
        report.recount();
        fs::write(
            dir.join("report.json"),
            serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
        let progress = Progress {
            run_id: id.into(),
            kind: "test".into(),
            state,
            totals: report.totals.clone(),
            updated_at_ms: now_ms(),
            ..Progress::default()
        };
        fs::write(
            dir.join("progress.json"),
            serde_json::to_vec(&progress).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn detail_reports_changes_and_flaky_tests() {
        let root = std::env::temp_dir().join(format!("airbug-store-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let a = "00000000-0000-4000-8000-000000000001";
        let b = "00000000-0000-4000-8000-000000000002";
        let c = "00000000-0000-4000-8000-000000000003";
        use TestStatus::{Failed, Passed};
        write_run(
            &root,
            a,
            1,
            RunState::Failed,
            &[("x", Failed), ("y", Passed), ("gone", Passed)],
        );
        write_run(
            &root,
            b,
            2,
            RunState::Passed,
            &[("x", Passed), ("y", Passed), ("gone", Passed)],
        );
        write_run(
            &root,
            c,
            3,
            RunState::Failed,
            &[("x", Passed), ("y", Failed), ("new", Passed)],
        );

        let detail = test_run_detail(&root, "latest", false).unwrap();
        assert_eq!(detail["run_id"], c);
        let changes = &detail["changes"];
        assert_eq!(changes["previous_run_id"], b);
        assert_eq!(changes["new_failures"], json!(["s::y"]));
        assert_eq!(changes["fixed"], json!([]));
        assert_eq!(changes["removed"], json!(["s::gone"]));
        assert_eq!(changes["added_count"], 1);
        assert_eq!(detail["history"]["tests"]["s::x"], "fpp");
        assert_eq!(detail["history"]["tests"]["s::y"], "ppf");
        assert!(detail["history"]["tests"].get("s::new").is_none());
        let flaky: Vec<&str> = detail["flaky"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["id"].as_str().unwrap())
            .collect();
        assert_eq!(flaky, ["s::x", "s::y"]);

        let lite = test_run_detail(&root, c, true).unwrap();
        assert!(lite["report"].is_null());
        assert_eq!(lite["changes"]["new_failures_count"], 1);
        assert!(test_run_detail(&root, "../etc", false).is_err());
        assert!(run_file(&root, c, "report.json").is_some());
        assert!(run_file(&root, c, "../x").is_none());
        assert!(run_file(&root, "nope", "report.json").is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_bench_session_outside_the_bench_store_is_linked_and_served() {
        let root = std::env::temp_dir().join(format!("airbug-artifacts-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let out = &root.join("elsewhere/session");
        fs::create_dir_all(out.join("0")).unwrap();
        fs::write(out.join("report.html"), "<html>").unwrap();
        fs::write(out.join("0/progress.json"), r#"{"completed":1,"total":4}"#).unwrap();
        let dir = runs_dir(&root).join("bench-1-2");
        fs::create_dir_all(&dir).unwrap();
        let manifest = Manifest {
            schema: MANIFEST_SCHEMA.into(),
            kind: KIND_BENCH.into(),
            run_id: "bench-1-2".into(),
            title: "session".into(),
            artifacts: Some(out.display().to_string()),
            ..Manifest::default()
        };
        fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        let items = announced_bench_runs(&root, &mut HashSet::new());
        assert_eq!(items[0].href, "#/run/bench-1-2");
        assert_eq!(items[0].progress.as_ref().unwrap()["total"], 4);

        let served = |rel| artifact_file(&root, "bench-1-2", rel);
        assert!(served("report.html").is_some());
        assert!(served("0/progress.json").is_some());
        assert!(served("../../.airbug/runs/bench-1-2/manifest.json").is_none());
        assert!(artifact_file(&root, "..", "report.html").is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unix_stamps_parse() {
        assert_eq!(unix_stamp_ms("12s unix"), 12_000);
        assert_eq!(unix_stamp_ms("1970-01-01T00:00:12.345Z"), 12_345);
        assert_eq!(unix_stamp_ms("garbage"), 0);
    }
}
