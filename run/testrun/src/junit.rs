//! JUnit XML exporter (`render_junit_xml(&Report) -> String`).
use super::model::{Report, Step, Suite, TestCase, TestStatus};
use std::collections::BTreeMap;
use std::fmt::Write;

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // Strip illegal XML 1.0 control characters except \t, \n, \r
            c if c < '\u{20}' && c != '\t' && c != '\n' && c != '\r' => {}
            c => out.push(c),
        }
    }
    out
}

fn step_summary(steps: &[Step], depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    for s in steps {
        let dur = s
            .duration_s
            .map(|d| format!(" ({d:.3}s)"))
            .unwrap_or_default();
        let _ = writeln!(out, "{indent}[{}] {}{dur}", s.status, s.name);
        step_summary(&s.children, depth + 1, out);
    }
}

/// Render a [`Report`] as a valid JUnit XML document.
pub fn render_junit_xml(report: &Report) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let total_tests = report.totals.total;
    let total_failures = report.totals.failed + report.totals.not_run;
    let total_skipped = report.totals.ignored;
    let total_time = report.duration_s.unwrap_or(0.0);

    let _ = writeln!(
        xml,
        "<testsuites name=\"{}\" tests=\"{total_tests}\" failures=\"{total_failures}\" skipped=\"{total_skipped}\" time=\"{total_time:.4}\">",
        escape_xml(&report.title)
    );

    let mut by_suite: BTreeMap<String, Vec<&TestCase>> = BTreeMap::new();
    for s in &report.suites {
        by_suite.entry(s.id.clone()).or_default();
    }
    for t in &report.tests {
        by_suite.entry(t.suite.clone()).or_default().push(t);
    }

    let suite_lookup: BTreeMap<&str, &Suite> =
        report.suites.iter().map(|s| (s.id.as_str(), s)).collect();

    for (suite_id, tests) in by_suite {
        let suite_meta = suite_lookup.get(suite_id.as_str()).copied();
        let s_tests = tests.len();
        let s_failures = tests
            .iter()
            .filter(|t| matches!(t.status, TestStatus::Failed | TestStatus::NotRun))
            .count();
        let s_skipped = tests
            .iter()
            .filter(|t| t.status == TestStatus::Ignored)
            .count();
        let s_time = suite_meta
            .and_then(|s| s.duration_s)
            .unwrap_or_else(|| tests.iter().filter_map(|t| t.duration_s).sum());

        let _ = writeln!(
            xml,
            "  <testsuite name=\"{}\" tests=\"{s_tests}\" failures=\"{s_failures}\" skipped=\"{s_skipped}\" time=\"{s_time:.4}\">",
            escape_xml(&suite_id)
        );

        for t in tests {
            let time = t.duration_s.or(t.finished_s).unwrap_or(0.0);
            let _ = writeln!(
                xml,
                "    <testcase name=\"{}\" classname=\"{}\" time=\"{time:.4}\">",
                escape_xml(&t.name),
                escape_xml(&t.suite)
            );

            let mut props: Vec<(&str, String)> = Vec::new();
            if let Some(v) = &t.meta.epic {
                props.push(("epic", v.clone()));
            }
            if let Some(v) = &t.meta.feature {
                props.push(("feature", v.clone()));
            }
            if let Some(v) = &t.meta.story {
                props.push(("story", v.clone()));
            }
            if let Some(v) = &t.meta.severity {
                props.push(("severity", v.clone()));
            }
            if let Some(v) = &t.meta.owner {
                props.push(("owner", v.clone()));
            }
            if let Some(v) = &t.meta.quarantined {
                props.push(("quarantined", v.clone()));
            }
            if let Some(v) = &t.meta.trace_id {
                props.push(("trace_id", v.clone()));
            }
            if let Some(v) = &t.meta.span_id {
                props.push(("span_id", v.clone()));
            }
            for tag in &t.meta.tags {
                props.push(("tag", tag.clone()));
            }
            for link in &t.meta.links {
                props.push((
                    "link",
                    match &link.url {
                        Some(u) => format!("{}:{} ({u})", link.kind, link.id),
                        None => format!("{}:{}", link.kind, link.id),
                    },
                ));
            }
            for (k, v) in &t.meta.parameters {
                props.push(("param", format!("{k}={v}")));
            }
            if let Some(def) = &t.defect {
                props.push(("defect.category", def.category.clone()));
                props.push(("defect.rule", def.rule.clone()));
            }

            if !props.is_empty() {
                xml.push_str("      <properties>\n");
                for (k, v) in props {
                    let _ = writeln!(
                        xml,
                        "        <property name=\"{}\" value=\"{}\"/>",
                        escape_xml(k),
                        escape_xml(&v)
                    );
                }
                xml.push_str("      </properties>\n");
            }

            match t.status {
                TestStatus::Failed | TestStatus::NotRun => {
                    let fail_type = t
                        .defect
                        .as_ref()
                        .map(|d| d.category.as_str())
                        .unwrap_or("failure");
                    let first_line = t
                        .output
                        .as_deref()
                        .and_then(|o| o.lines().find(|l| !l.trim().is_empty()))
                        .or_else(|| t.defect.as_ref().map(|d| d.rule.as_str()))
                        .unwrap_or("test failed");
                    let mut body = t.output.clone().unwrap_or_default();
                    if !t.steps.is_empty() {
                        if !body.is_empty() && !body.ends_with('\n') {
                            body.push('\n');
                        }
                        body.push_str("Steps:\n");
                        step_summary(&t.steps, 1, &mut body);
                    }
                    let _ = writeln!(
                        xml,
                        "      <failure type=\"{}\" message=\"{}\">{}</failure>",
                        escape_xml(fail_type),
                        escape_xml(first_line),
                        escape_xml(&body)
                    );
                }
                TestStatus::Ignored => {
                    if let Some(reason) = &t.ignore_reason {
                        let _ =
                            writeln!(xml, "      <skipped message=\"{}\"/>", escape_xml(reason));
                    } else {
                        xml.push_str("      <skipped/>\n");
                    }
                }
                _ => {}
            }

            if let Some(out) = &t.output
                && t.status == TestStatus::Passed
                && !out.is_empty()
            {
                let _ = writeln!(xml, "      <system-out>{}</system-out>", escape_xml(out));
            }

            xml.push_str("    </testcase>\n");
        }

        xml.push_str("  </testsuite>\n");
    }

    xml.push_str("</testsuites>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DefectClassification, RunState, TestMeta, Totals};

    #[test]
    fn renders_valid_escaped_junit_xml() {
        let meta = TestMeta {
            epic: Some("Billing & Payments".into()),
            feature: Some("Refunds <v2>".into()),
            severity: Some("critical".into()),
            trace_id: Some("0123456789abcdef0123456789abcdef".into()),
            ..TestMeta::default()
        };

        let mut report = Report {
            run_id: "run-1".into(),
            title: "cargo test \"all\"".into(),
            started_at_ms: 1000,
            finished_at_ms: Some(2000),
            duration_s: Some(1.25),
            state: RunState::Failed,
            exit_code: Some(101),
            totals: Totals {
                total: 2,
                completed: 2,
                passed: 1,
                failed: 1,
                ignored: 0,
                not_run: 0,
            },
            ..Report::default()
        };
        report.tests = vec![
            TestCase {
                id: "billing::refund_ok".into(),
                suite: "billing".into(),
                name: "refund_ok".into(),
                status: TestStatus::Passed,
                finished_s: Some(0.5),
                duration_s: Some(0.5),
                output: None,
                steps: vec![],
                comparisons: vec![],
                attachments: vec![],
                flaky: vec![],
                ignore_reason: None,
                meta: meta.clone(),
                defect: None,
            },
            TestCase {
                id: "billing::refund_err".into(),
                suite: "billing".into(),
                name: "refund_err".into(),
                status: TestStatus::Failed,
                finished_s: Some(1.25),
                duration_s: Some(0.75),
                output: Some("assertion `a < b && c > d` failed".into()),
                steps: vec![],
                comparisons: vec![],
                attachments: vec![],
                flaky: vec![],
                ignore_reason: None,
                meta,
                defect: Some(DefectClassification {
                    category: "product_bug".into(),
                    rule: "assertion failure".into(),
                }),
            },
        ];

        let xml = render_junit_xml(&report);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("Billing &amp; Payments"));
        assert!(xml.contains("Refunds &lt;v2&gt;"));
        assert!(xml.contains("<failure type=\"product_bug\""));
        assert!(xml.contains("name=\"trace_id\" value=\"0123456789abcdef0123456789abcdef\""));
    }
}
