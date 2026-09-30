//! Fold `airbug::report` events (`events/events.jsonl`) into the tests of a report.
//!
//! Every event carries `pid`, `bin` and `test` (see `unit/src/report.rs`). Step ids are unique
//! per process, so nodes are keyed by `(pid, id)`; `bin` maps to the suite through cargo's
//! `Running` line and `test` is the libtest thread name.
use super::{
    libtest::Collector,
    model::{Attachment, Comparison, Step, Unattributed},
};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

/// Upper bound mirrors the writer's `MAX_EVENTS`.
const MAX_EVENTS_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Default, Deserialize)]
struct Raw {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    parent: Option<u64>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default, rename = "mediaType")]
    media_type: String,
    #[serde(default)]
    file: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    expected: String,
    #[serde(default)]
    actual: String,
    #[serde(default)]
    passed: bool,
    #[serde(default)]
    truncated: bool,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    pid: u32,
    #[serde(default)]
    bin: String,
    #[serde(default)]
    test: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Owner {
    bin: String,
    test: Option<String>,
}

#[derive(Debug, Default)]
struct Node {
    step: Step,
    children: Vec<usize>,
}

#[derive(Debug, Default)]
struct Loose {
    roots: Vec<usize>,
    comparisons: Vec<Comparison>,
    attachments: Vec<Attachment>,
    flaky: Vec<String>,
}

/// Read `events.jsonl` + `report-error.txt` from `dir` and attach them to `collector.report`.
pub fn fold(collector: &mut Collector, dir: &Path) {
    let marker = dir.join("report-error.txt");
    if let Ok(text) = fs::read_to_string(&marker) {
        collector.report.diagnostics.push(text.trim().to_string());
    }
    let path = dir.join("events.jsonl");
    let Ok(meta) = fs::metadata(&path) else {
        return;
    };
    if meta.len() > MAX_EVENTS_BYTES {
        collector.report.diagnostics.push(format!(
            "events.jsonl is {} bytes (limit {MAX_EVENTS_BYTES}); steps skipped",
            meta.len()
        ));
        return;
    }
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    fold_text(collector, &text);
}

/// Case-isolated reporter directories already identify the owning case.
/// Normalize the owner so the same event parser serves both run formats.
pub fn fold_case(test: &mut super::model::TestCase, events: &[serde_json::Value]) {
    let mut collector = Collector::default();
    collector.list_line("Running unittests src/lib.rs (target/debug/deps/case)");
    collector.list_line("case: test");
    let text = events
        .iter()
        .map(|event| {
            let mut event = event.clone();
            if let Some(raw) = event.as_object_mut() {
                raw.insert("bin".into(), serde_json::json!("case"));
                raw.insert("test".into(), serde_json::json!("case"));
            }
            event.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    fold_text(&mut collector, &text);
    if let Some(parsed) = collector.report.tests.into_iter().next() {
        test.steps = parsed.steps;
        test.comparisons = parsed.comparisons;
        test.attachments = parsed.attachments;
        test.flaky = parsed.flaky;
    }
}

fn fold_text(collector: &mut Collector, text: &str) {
    let mut nodes: Vec<Node> = Vec::new();
    let mut by_id: HashMap<(u32, u64), usize> = HashMap::new();
    let mut owners: Vec<(Owner, Loose)> = Vec::new();
    let mut skipped = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(raw) = serde_json::from_str::<Raw>(line) else {
            skipped += 1;
            continue;
        };
        let owner = Owner {
            bin: raw.bin.clone(),
            test: raw.test.clone(),
        };
        let parent = raw.parent.and_then(|p| by_id.get(&(raw.pid, p)).copied());
        match raw.kind.as_str() {
            "step_start" => {
                let Some(id) = raw.id else { continue };
                nodes.push(Node {
                    step: Step {
                        name: raw.name,
                        status: "unfinished".into(),
                        ..Step::default()
                    },
                    children: Vec::new(),
                });
                let index = nodes.len() - 1;
                by_id.insert((raw.pid, id), index);
                match parent {
                    Some(p) => nodes[p].children.push(index),
                    None => loose(&mut owners, owner).roots.push(index),
                }
            }
            "step_end" => {
                let Some(&index) = raw.id.and_then(|id| by_id.get(&(raw.pid, id))) else {
                    continue;
                };
                nodes[index].step.status = raw.status;
                nodes[index].step.duration_s = raw.duration;
            }
            "attachment" => {
                if !is_plain_file_name(&raw.file) {
                    skipped += 1;
                    continue;
                }
                let item = Attachment {
                    name: raw.name,
                    media_type: raw.media_type,
                    file: raw.file,
                    size: raw.size,
                };
                match parent {
                    Some(p) => nodes[p].step.attachments.push(item),
                    None => loose(&mut owners, owner).attachments.push(item),
                }
            }
            "comparison" => {
                let item = Comparison {
                    name: raw.name,
                    expected: raw.expected,
                    actual: raw.actual,
                    passed: raw.passed,
                    truncated: raw.truncated,
                };
                match parent {
                    Some(p) => nodes[p].step.comparisons.push(item),
                    None => loose(&mut owners, owner).comparisons.push(item),
                }
            }
            "flaky" => match parent {
                Some(p) => nodes[p].step.flaky.push(raw.reason),
                None => loose(&mut owners, owner).flaky.push(raw.reason),
            },
            _ => skipped += 1,
        }
    }

    for (owner, items) in owners {
        let steps: Vec<Step> = items.roots.iter().map(|&i| build(&nodes, i)).collect();
        let suite = collector.suite_for_binary(&owner.bin).map(str::to_string);
        let test = match (&suite, &owner.test) {
            (Some(suite), Some(name)) => collector.find_test(suite, name),
            _ => None,
        };
        match test {
            Some(index) => {
                let test = &mut collector.report.tests[index];
                test.steps.extend(steps);
                test.comparisons.extend(items.comparisons);
                test.attachments.extend(items.attachments);
                test.flaky.extend(items.flaky);
            }
            None => collector.report.unattributed.push(Unattributed {
                suite: suite.or_else(|| (!owner.bin.is_empty()).then(|| owner.bin.clone())),
                thread: owner.test,
                steps,
                comparisons: items.comparisons,
                attachments: items.attachments,
            }),
        }
    }
    if skipped > 0 {
        collector
            .report
            .diagnostics
            .push(format!("{skipped} report event(s) could not be read"));
    }
}

fn loose(owners: &mut Vec<(Owner, Loose)>, owner: Owner) -> &mut Loose {
    let index = match owners.iter().position(|(o, _)| *o == owner) {
        Some(index) => index,
        None => {
            owners.push((owner, Loose::default()));
            owners.len() - 1
        }
    };
    &mut owners[index].1
}

fn build(nodes: &[Node], index: usize) -> Step {
    let node = &nodes[index];
    let mut step = node.step.clone();
    step.children = node.children.iter().map(|&i| build(nodes, i)).collect();
    step
}

/// Attachment names come from test processes; they must stay inside `events/`.
fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains(['/', '\\'])
        && name != "."
        && name != ".."
        && !name.contains(':')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testrun::model::Report;

    #[test]
    fn steps_nest_per_process_and_attach_to_their_test() {
        let mut c = Collector::new(Report::default());
        c.list_line(
            "     Running tests/reporting.rs (target/debug/deps/reporting-0123456789abcdef)",
        );
        c.list_line("checkout: test");
        c.list_line("other: test");
        let events = r#"
{"type":"step_start","id":1,"parent":null,"name":"Checkout","pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"step_start","id":1,"parent":null,"name":"Other","pid":8,"bin":"reporting-0123456789abcdef","test":"other"}
{"type":"step_start","id":2,"parent":1,"name":"Pay","pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"comparison","parent":2,"name":"amount","expected":"1","actual":"2","passed":false,"truncated":false,"pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"attachment","parent":2,"name":"log","mediaType":"text/plain","file":"attachment-7-3.bin","size":3,"pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"attachment","parent":null,"name":"evil","mediaType":"text/plain","file":"../../etc/passwd","size":3,"pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"step_end","id":2,"status":"failed","duration":0.5,"pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"flaky","parent":null,"reason":"network","pid":7,"bin":"reporting-0123456789abcdef","test":"checkout"}
{"type":"step_start","id":9,"parent":null,"name":"Helper","pid":7,"bin":"reporting-0123456789abcdef","test":null}
not json
"#;
        fold_text(&mut c, events);
        let checkout = &c.report.tests[c
            .find_test("reporting (tests/reporting.rs)", "checkout")
            .unwrap()];
        assert_eq!(checkout.steps.len(), 1);
        let root = &checkout.steps[0];
        assert_eq!(root.name, "Checkout");
        assert_eq!(root.status, "unfinished");
        assert_eq!(root.children[0].name, "Pay");
        assert_eq!(root.children[0].status, "failed");
        assert_eq!(root.children[0].comparisons[0].actual, "2");
        assert_eq!(root.children[0].attachments[0].file, "attachment-7-3.bin");
        assert_eq!(checkout.flaky, vec!["network".to_string()]);
        assert!(
            checkout.attachments.is_empty(),
            "path-like names are dropped"
        );

        let other = &c.report.tests[c
            .find_test("reporting (tests/reporting.rs)", "other")
            .unwrap()];
        assert_eq!(other.steps[0].name, "Other");
        assert!(other.steps[0].children.is_empty(), "same id, other pid");

        assert_eq!(c.report.unattributed.len(), 1);
        assert_eq!(c.report.unattributed[0].steps[0].name, "Helper");
        assert_eq!(c.report.diagnostics.len(), 1);
    }
}
