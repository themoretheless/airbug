//! Defect classification: built-in heuristics plus the custom triage rules in
//! `.airbug/defect-rules.json`.
use crate::model::{DefectClassification, Report, Step, SuiteState, TestCase, TestStatus};
use airbug_run::store::{read_json, write_json};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Comparison, Report, RunState, TestCase, TestMeta, TestStatus};

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
