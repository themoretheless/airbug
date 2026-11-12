//! TestOps defect classification, custom triage rules, and cross-run stability & duration analytics.
use super::model::{DefectClassification, Report, Step, SuiteState, TestCase, TestStatus};
use airbug_run::store::{read_json, runs_dir, write_json};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefectRule {
    pub name: String,
    /// `"product_bug"`, `"infrastructure"`, `"test_bug"`, `"flaky"`, or `"quarantined"`.
    pub category: String,
    /// Substring (case-insensitive) matched against test output or test name.
    pub pattern: String,
}

pub fn rules_path(root: &Path) -> PathBuf {
    root.join(".airbug").join("defect-rules.json")
}

pub fn load_rules(root: &Path) -> Vec<DefectRule> {
    read_json(&rules_path(root)).unwrap_or_default()
}

pub fn save_rules(root: &Path, rules: &[DefectRule]) -> std::io::Result<()> {
    write_json(&rules_path(root), &rules, true)
}

fn has_failed_comparison(test: &TestCase) -> bool {
    if test.comparisons.iter().any(|c| !c.passed) {
        return true;
    }
    fn step_has_failed_cmp(steps: &[Step]) -> bool {
        steps
            .iter()
            .any(|s| s.comparisons.iter().any(|c| !c.passed) || step_has_failed_cmp(&s.children))
    }
    step_has_failed_cmp(&test.steps)
}

fn is_infrastructure_output(output: &str) -> Option<&'static str> {
    const INDICATORS: &[(&str, &str)] = &[
        ("Connection refused", "connection refused"),
        ("connection refused", "connection refused"),
        ("TimedOut", "operation timed out"),
        ("timed out", "operation timed out"),
        ("os error", "OS / environment error"),
        ("docker", "container / docker failure"),
        ("container", "container / docker failure"),
        ("broken pipe", "broken pipe"),
        ("Address already in use", "port / address conflict"),
        ("address already in use", "port / address conflict"),
    ];
    for (needle, reason) in INDICATORS {
        if output.contains(needle) {
            return Some(reason);
        }
    }
    None
}

fn is_assertion_output(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    lower.contains("assertion") || lower.contains("expected") || lower.contains("assert_eq")
}

/// Classify failed, ignored, or quarantined tests in `report`.
pub fn classify_defects(report: &mut Report, custom_rules: &[DefectRule]) {
    let crashed_suites: HashSet<String> = report
        .suites
        .iter()
        .filter(|s| s.state == SuiteState::Crashed)
        .map(|s| s.id.clone())
        .collect();

    for test in &mut report.tests {
        let is_quarantined = test.meta.quarantined.is_some();
        let is_failed = matches!(test.status, TestStatus::Failed | TestStatus::NotRun);
        let is_flaky_marked = !test.flaky.is_empty();

        if is_quarantined {
            let reason = test
                .meta
                .quarantined
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("quarantined test");
            test.defect = Some(DefectClassification {
                category: "quarantined".into(),
                rule: reason.to_string(),
            });
            continue;
        }

        if !is_failed && !is_flaky_marked {
            continue;
        }

        let output = test.output.as_deref().unwrap_or("");
        let out_lower = output.to_ascii_lowercase();
        let name_lower = test.name.to_ascii_lowercase();

        let mut matched_custom = None;
        for rule in custom_rules {
            let pat = rule.pattern.trim().to_ascii_lowercase();
            if !pat.is_empty() && (out_lower.contains(&pat) || name_lower.contains(&pat)) {
                matched_custom = Some(DefectClassification {
                    category: rule.category.clone(),
                    rule: rule.name.clone(),
                });
                break;
            }
        }
        if let Some(custom) = matched_custom {
            test.defect = Some(custom);
            continue;
        }

        if is_flaky_marked {
            test.defect = Some(DefectClassification {
                category: "flaky".into(),
                rule: "marked flaky or retried".into(),
            });
            continue;
        }

        if crashed_suites.contains(&test.suite) || test.status == TestStatus::NotRun {
            test.defect = Some(DefectClassification {
                category: "infrastructure".into(),
                rule: "suite process crashed before completion".into(),
            });
            continue;
        }

        if let Some(reason) = is_infrastructure_output(output) {
            test.defect = Some(DefectClassification {
                category: "infrastructure".into(),
                rule: reason.into(),
            });
            continue;
        }

        if !test.comparisons.is_empty()
            || has_failed_comparison(test)
            || is_assertion_output(output)
        {
            test.defect = Some(DefectClassification {
                category: "product_bug".into(),
                rule: "assertion or comparison mismatch".into(),
            });
            continue;
        }

        test.defect = Some(DefectClassification {
            category: "test_bug".into(),
            rule: "unclassified panic or failure".into(),
        });
    }
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    values.retain(|v| v.is_finite() && *v >= 0.0);
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[mid])
    } else {
        Some((values[mid - 1] + values[mid]) / 2.0)
    }
}

/// Compute cross-run TestOps analytics across the newest `window` test runs.
pub fn compute_analytics(root: &Path, window: usize) -> Value {
    let window = window.clamp(1, 100);
    let custom_rules = load_rules(root);
    let items = crate::store::test_runs(root, window);
    let mut reports: Vec<Report> = Vec::new();
    for item in &items {
        let path = runs_dir(root).join(&item.run_id).join("report.json");
        if let Some(mut rep) = read_json::<Report>(&path) {
            classify_defects(&mut rep, &custom_rules);
            reports.push(rep);
        }
    }

    #[derive(Default)]
    struct TestStat {
        id: String,
        suite: String,
        name: String,
        total_runs: usize,
        passes: usize,
        fails: usize,
        flaky_count: usize,
        quarantined: Option<String>,
        durations: Vec<f64>,
        latest_duration_s: Option<f64>,
    }

    let mut stats: BTreeMap<String, TestStat> = BTreeMap::new();
    // reports is newest first; iterate in reverse so latest_duration_s is from newest run
    for (idx, rep) in reports.iter().enumerate() {
        for t in &rep.tests {
            if matches!(t.status, TestStatus::Pending | TestStatus::Ignored) {
                continue;
            }
            let entry = stats.entry(t.id.clone()).or_insert_with(|| TestStat {
                id: t.id.clone(),
                suite: t.suite.clone(),
                name: t.name.clone(),
                ..TestStat::default()
            });
            entry.total_runs += 1;
            match t.status {
                TestStatus::Passed => entry.passes += 1,
                TestStatus::Failed | TestStatus::NotRun => entry.fails += 1,
                _ => {}
            }
            if !t.flaky.is_empty() {
                entry.flaky_count += 1;
            }
            if idx == 0 && t.meta.quarantined.is_some() {
                entry.quarantined = t.meta.quarantined.clone();
            }
            let dur = t.duration_s.or(t.finished_s);
            if let Some(d) = dur {
                if idx == 0 {
                    entry.latest_duration_s = Some(d);
                } else {
                    entry.durations.push(d);
                }
            }
        }
    }

    let mut stability: Vec<Value> = stats
        .values()
        .map(|s| {
            let pass_rate = if s.total_runs > 0 {
                (s.passes as f64 / s.total_runs as f64) * 100.0
            } else {
                100.0
            };
            let is_flaky = s.flaky_count > 0 || (s.passes > 0 && s.fails > 0);
            json!({
                "id": s.id,
                "suite": s.suite,
                "name": s.name,
                "pass_rate": (pass_rate * 10.0).round() / 10.0,
                "total_runs": s.total_runs,
                "passes": s.passes,
                "fails": s.fails,
                "flaky_count": if is_flaky { s.flaky_count.max(1) } else { 0 },
                "quarantined": s.quarantined,
            })
        })
        .collect();
    stability.sort_by(|a, b| {
        let ra = a["pass_rate"].as_f64().unwrap_or(100.0);
        let rb = b["pass_rate"].as_f64().unwrap_or(100.0);
        ra.partial_cmp(&rb)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });

    let mut slowest: Vec<Value> = stats
        .values()
        .filter_map(|s| {
            let cur = s
                .latest_duration_s
                .or_else(|| s.durations.first().copied())?;
            let med = median(s.durations.clone()).unwrap_or(cur);
            let regressed = s.durations.len() >= 2 && med > 0.005 && cur > med * 2.0;
            Some(json!({
                "id": s.id,
                "suite": s.suite,
                "name": s.name,
                "duration_s": cur,
                "median_s": med,
                "regressed": regressed,
            }))
        })
        .collect();
    slowest.sort_by(|a, b| {
        let da = a["duration_s"].as_f64().unwrap_or(0.0);
        let db = b["duration_s"].as_f64().unwrap_or(0.0);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
    slowest.truncate(25);

    // Behaviors tree & Defects summary from the newest report (if any).
    let mut defects_summary: BTreeMap<String, usize> = BTreeMap::from([
        ("product_bug".into(), 0),
        ("infrastructure".into(), 0),
        ("test_bug".into(), 0),
        ("flaky".into(), 0),
        ("quarantined".into(), 0),
    ]);

    #[derive(Default)]
    struct StoryBucket {
        passed: usize,
        failed: usize,
        total: usize,
    }
    #[derive(Default)]
    struct FeatureBucket {
        passed: usize,
        failed: usize,
        total: usize,
        stories: BTreeMap<String, StoryBucket>,
    }
    #[derive(Default)]
    struct EpicBucket {
        passed: usize,
        failed: usize,
        total: usize,
        features: BTreeMap<String, FeatureBucket>,
    }
    let mut epics: BTreeMap<String, EpicBucket> = BTreeMap::new();

    if let Some(latest) = reports.first() {
        for t in &latest.tests {
            if let Some(def) = &t.defect {
                *defects_summary.entry(def.category.clone()).or_insert(0) += 1;
            }
            let epic_name = t
                .meta
                .epic
                .clone()
                .unwrap_or_else(|| "Unassigned Epic".into());
            let feature_name = t
                .meta
                .feature
                .clone()
                .unwrap_or_else(|| "General Feature".into());
            let story_name = t
                .meta
                .story
                .clone()
                .unwrap_or_else(|| "Default Story".into());
            let passed = usize::from(t.status == TestStatus::Passed);
            let failed = usize::from(matches!(t.status, TestStatus::Failed | TestStatus::NotRun));

            let ep = epics.entry(epic_name).or_default();
            ep.total += 1;
            ep.passed += passed;
            ep.failed += failed;

            let ft = ep.features.entry(feature_name).or_default();
            ft.total += 1;
            ft.passed += passed;
            ft.failed += failed;

            let st = ft.stories.entry(story_name).or_default();
            st.total += 1;
            st.passed += passed;
            st.failed += failed;
        }
    }

    let behaviors: Vec<Value> = epics
        .into_iter()
        .map(|(epic, ep)| {
            let features: Vec<Value> = ep
                .features
                .into_iter()
                .map(|(feature, ft)| {
                    let stories: Vec<Value> = ft
                        .stories
                        .into_iter()
                        .map(|(story, st)| {
                            json!({
                                "story": story,
                                "passed": st.passed,
                                "failed": st.failed,
                                "total": st.total,
                            })
                        })
                        .collect();
                    json!({
                        "feature": feature,
                        "passed": ft.passed,
                        "failed": ft.failed,
                        "total": ft.total,
                        "stories": stories,
                    })
                })
                .collect();
            json!({
                "epic": epic,
                "passed": ep.passed,
                "failed": ep.failed,
                "total": ep.total,
                "features": features,
            })
        })
        .collect();

    json!({
        "window_runs": reports.len(),
        "stability": stability,
        "slowest": slowest,
        "behaviors": behaviors,
        "defects_summary": defects_summary,
        "rules": custom_rules,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testrun::model::{Comparison, Report, RunState, TestCase, TestMeta, TestStatus};

    fn make_test(name: &str, status: TestStatus, output: Option<&str>) -> TestCase {
        TestCase {
            id: format!("suite::{name}"),
            suite: "suite".into(),
            name: name.into(),
            status,
            finished_s: Some(0.1),
            duration_s: Some(0.1),
            output: output.map(Into::into),
            steps: Vec::new(),
            comparisons: Vec::new(),
            attachments: Vec::new(),
            flaky: Vec::new(),
            ignore_reason: None,
            meta: TestMeta::default(),
            defect: None,
        }
    }

    #[test]
    fn classify_defects_covers_all_categories() {
        let mut t_quarantined = make_test("t_quarantined", TestStatus::Failed, Some("boom"));
        t_quarantined.meta.quarantined = Some("ISSUE-99".into());

        let t_custom = make_test("t_custom", TestStatus::Failed, Some("redis timeout 999"));
        let mut t_flaky = make_test("t_flaky", TestStatus::Passed, None);
        t_flaky.flaky.push("retry 1".into());

        let t_infra = make_test(
            "t_infra",
            TestStatus::Failed,
            Some("Connection refused (os error 61)"),
        );
        let mut t_product = make_test(
            "t_product",
            TestStatus::Failed,
            Some("assertion `left == right` failed"),
        );
        t_product.comparisons.push(Comparison {
            name: "eq".into(),
            passed: false,
            expected: "1".into(),
            actual: "2".into(),
            truncated: false,
        });
        let t_bug = make_test("t_bug", TestStatus::Failed, Some("index out of bounds"));

        let mut report = Report {
            run_id: "r1".into(),
            title: "t".into(),
            state: RunState::Failed,
            tests: vec![t_quarantined, t_custom, t_flaky, t_infra, t_product, t_bug],
            ..Report::default()
        };

        let rules = vec![DefectRule {
            name: "Redis outage".into(),
            category: "infrastructure".into(),
            pattern: "redis timeout".into(),
        }];
        classify_defects(&mut report, &rules);

        assert_eq!(
            report.tests[0].defect.as_ref().unwrap().category,
            "quarantined"
        );
        assert_eq!(
            report.tests[1].defect.as_ref().unwrap().category,
            "infrastructure"
        );
        assert_eq!(
            report.tests[1].defect.as_ref().unwrap().rule,
            "Redis outage"
        );
        assert_eq!(report.tests[2].defect.as_ref().unwrap().category, "flaky");
        assert_eq!(
            report.tests[3].defect.as_ref().unwrap().category,
            "infrastructure"
        );
        assert_eq!(
            report.tests[4].defect.as_ref().unwrap().category,
            "product_bug"
        );
        assert_eq!(
            report.tests[5].defect.as_ref().unwrap().category,
            "test_bug"
        );
    }
}
