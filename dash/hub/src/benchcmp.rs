//! The latest bench run against the previous comparable one (same cases, contracts and
//! environment), with the statistics of `cargo airbug-bench compare` (`airbug_bench::analysis`).
//!
//! A crossover run (`baseline` + `candidate` in one `run.json`) is compared within itself.
//! Results are cached until a `run.json` under `.airbug-bench` changes, since the Now page
//! polls and run files can be large.
use crate::scan::find_run_json;
use airbug_bench::{
    Run, Status,
    analysis::{self, Comparison, Decision},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

/// Newest `run.json` files considered.
const SCAN: usize = 40;
/// Percent change that counts as a regression / improvement (the CLI's default).
pub const THRESHOLD: f64 = 5.0;
const ALPHA: f64 = 0.05;
const MAX_ROWS: usize = 60;
const MAX_RUN_BYTES: u64 = 64 * 1024 * 1024;

type CacheKey = (Vec<(PathBuf, u64)>, Option<String>);
static CACHE: Mutex<Option<(CacheKey, Value)>> = Mutex::new(None);

struct Entry {
    id: String,
    href: String,
    when_ms: u64,
    run: Run,
}

fn mtime_ms(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// `GET /api/v1/bench/compare[?run=<id>]`.
pub fn compare(root: &Path, id: Option<&str>) -> Value {
    let store = root.join(".airbug-bench");
    let files: Vec<(PathBuf, u64)> = find_run_json(&store, SCAN)
        .into_iter()
        .map(|p| {
            let when = mtime_ms(&p);
            (p, when)
        })
        .collect();
    let key = (files.clone(), id.map(str::to_string));
    if let Ok(cache) = CACHE.lock()
        && let Some((cached_key, value)) = cache.as_ref()
        && *cached_key == key
    {
        return value.clone();
    }
    let entries: Vec<Entry> = files
        .iter()
        .filter_map(|(path, when)| load(&store, path, *when))
        .collect();
    let value = compare_entries(&entries, id);
    if let Ok(mut cache) = CACHE.lock() {
        *cache = Some((key, value.clone()));
    }
    value
}

fn load(store: &Path, path: &Path, when_ms: u64) -> Option<Entry> {
    if fs::metadata(path).ok()?.len() > MAX_RUN_BYTES {
        return None;
    }
    let run: Run = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    let dir = path.parent()?;
    let rel = dir
        .strip_prefix(store)
        .ok()?
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let href = match rel.strip_prefix("runs/") {
        Some(uuid) if uuid::Uuid::parse_str(uuid).is_ok() => format!("#/bench/{uuid}"),
        _ if dir.join("report.html").is_file() => format!("/bench/{rel}/report.html"),
        _ => format!("/bench/{rel}/run.json"),
    };
    Some(Entry {
        id: rel,
        href,
        when_ms,
        run,
    })
}

fn variants(run: &Run) -> BTreeSet<&str> {
    run.observations
        .iter()
        .map(|o| o.variant.as_str())
        .collect()
}

fn describe(entry: &Entry) -> Value {
    json!({
        "id": entry.id,
        "href": entry.href,
        "when_ms": entry.when_ms,
        "cases": entry.run.cases.len(),
    })
}

fn decision_rank(d: &Decision) -> u8 {
    match d {
        Decision::Regression => 0,
        Decision::Improvement => 1,
        Decision::Inconclusive => 2,
        Decision::WithinMargin => 3,
        Decision::Unavailable => 4,
        Decision::Neutral => 5,
    }
}

fn result(
    mode: &str,
    current: &Entry,
    previous: Option<&Entry>,
    mut rows: Vec<Comparison>,
) -> Value {
    let count = |d: Decision| rows.iter().filter(|r| r.decision == d).count();
    let summary = json!({
        "regression": count(Decision::Regression),
        "improvement": count(Decision::Improvement),
        "within_margin": count(Decision::WithinMargin),
        "inconclusive": count(Decision::Inconclusive),
        "unavailable": count(Decision::Unavailable),
        "neutral": count(Decision::Neutral),
    });
    rows.sort_by(|a, b| {
        decision_rank(&a.decision)
            .cmp(&decision_rank(&b.decision))
            .then_with(|| {
                let size = |r: &Comparison| r.change_percent.map(f64::abs).unwrap_or(0.0);
                size(b).total_cmp(&size(a))
            })
    });
    let total = rows.len();
    rows.truncate(MAX_ROWS);
    json!({
        "mode": mode,
        "threshold": THRESHOLD,
        "current": describe(current),
        "previous": previous.map(describe),
        "summary": summary,
        "total": total,
        "rows": rows,
    })
}

fn compare_entries(entries: &[Entry], id: Option<&str>) -> Value {
    let complete = |e: &&Entry| e.run.status == Status::Complete;
    let current = match id {
        Some(id) => entries
            .iter()
            .find(|e| e.id == id || e.id == format!("runs/{id}")),
        None => entries.iter().find(complete),
    };
    let Some(current) = current else {
        return json!({ "mode": null, "note": "no finished bench run yet" });
    };
    if current.run.status != Status::Complete {
        return json!({
            "mode": null,
            "current": describe(current),
            "note": "this bench run has not completed",
        });
    }
    let own = variants(&current.run);
    if own == BTreeSet::from(["baseline", "candidate"]) {
        return match analysis::compare(&current.run, None, THRESHOLD, ALPHA) {
            Ok(rows) => result("ab", current, None, rows),
            Err(err) => json!({
                "mode": null,
                "current": describe(current),
                "note": format!("could not compare baseline and candidate: {err}"),
            }),
        };
    }
    if own != BTreeSet::from(["candidate"]) {
        return json!({
            "mode": null,
            "current": describe(current),
            "note": "unsupported variant set",
        });
    }
    let position = entries
        .iter()
        .position(|e| std::ptr::eq(e, current))
        .unwrap_or(0);
    let mut older_complete = 0usize;
    for previous in entries[position + 1..].iter().filter(complete) {
        older_complete += 1;
        let comparable = variants(&previous.run) == own
            && previous.run.environment == current.run.environment
            && previous.run.cases == current.run.cases;
        if !comparable {
            continue;
        }
        if let Ok(rows) = analysis::compare(&previous.run, Some(&current.run), THRESHOLD, ALPHA) {
            return result("previous", current, Some(previous), rows);
        }
    }
    json!({
        "mode": null,
        "current": describe(current),
        "note": if older_complete == 0 {
            "no earlier finished run to compare with".to_string()
        } else {
            format!(
                "{older_complete} earlier run(s), none with the same cases, contracts and environment"
            )
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbug_bench::{Availability, Case, Metric, Observation};
    use std::collections::BTreeMap;

    fn run(id: &str, variant_values: &[(&str, &[f64])]) -> Run {
        let mut run = Run::new();
        run.id = id.into();
        run.status = Status::Complete;
        run.cases = vec![Case {
            id: "sum".into(),
            contract: BTreeMap::new(),
            metrics: vec![Metric::duration("wall", "operation", "median")],
        }];
        let mut sequence = 0;
        for (variant, values) in variant_values {
            for (process, value) in values.iter().enumerate() {
                sequence += 1;
                run.observations.push(Observation {
                    case: "sum".into(),
                    metric: "wall".into(),
                    variant: variant.to_string(),
                    process: process as u32,
                    pair: Some(process as u32),
                    sequence,
                    value: Some(value.to_string()),
                    operations: 1,
                    worker_work_totals: Default::default(),
                    work_totals: BTreeMap::new(),
                    availability: Availability::Available,
                });
            }
        }
        run
    }

    fn entry(id: &str, when_ms: u64, run: Run) -> Entry {
        Entry {
            id: id.into(),
            href: format!("#/bench/{id}"),
            when_ms,
            run,
        }
    }

    const SLOW: [f64; 12] = [
        200.0, 201.0, 199.0, 202.0, 198.0, 200.0, 203.0, 197.0, 200.0, 201.0, 199.0, 200.0,
    ];
    const FAST: [f64; 12] = [
        100.0, 101.0, 99.0, 102.0, 98.0, 100.0, 103.0, 97.0, 100.0, 101.0, 99.0, 100.0,
    ];

    #[test]
    fn latest_run_is_compared_with_the_previous_comparable_one() {
        let mut other_env = run("other", &[("candidate", &FAST)]);
        other_env
            .environment
            .insert("host".into(), "elsewhere".into());
        let entries = vec![
            entry("new", 3, run("new", &[("candidate", &SLOW)])),
            entry("other", 2, other_env),
            entry("old", 1, run("old", &[("candidate", &FAST)])),
        ];
        let value = compare_entries(&entries, None);
        assert_eq!(value["mode"], "previous", "{value}");
        assert_eq!(value["current"]["id"], "new");
        assert_eq!(
            value["previous"]["id"], "old",
            "different environment skipped"
        );
        assert_eq!(value["summary"]["regression"], 1, "{value}");
        let change = value["rows"][0]["change_percent"].as_f64().unwrap();
        assert!((change - 100.0).abs() < 1.0, "{change}");
    }

    #[test]
    fn crossover_runs_compare_within_themselves_and_lonely_runs_say_so() {
        let ab = run("ab", &[("baseline", &SLOW), ("candidate", &FAST)]);
        let value = compare_entries(&[entry("ab", 1, ab)], None);
        assert_eq!(value["mode"], "ab", "{value}");
        assert_eq!(value["summary"]["improvement"], 1, "{value}");

        let alone = compare_entries(&[entry("x", 1, run("x", &[("candidate", &FAST)]))], None);
        assert!(alone["mode"].is_null());
        assert_eq!(alone["note"], "no earlier finished run to compare with");
        assert_eq!(
            compare_entries(&[], None)["note"],
            "no finished bench run yet"
        );
    }
}
