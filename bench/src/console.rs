//! Terminal styling, kept separate from report and protocol serialization.
use crate::{Result, error};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsoleColor {
    #[default]
    Auto,
    Always,
    Never,
}
impl ConsoleColor {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(error("color must be auto, always or never")),
        }
    }
    pub(crate) fn enabled(self, terminal: bool, no_color: bool) -> bool {
        match self {
            Self::Auto => terminal && !no_color,
            Self::Always => true,
            Self::Never => false,
        }
    }
}
pub(crate) fn render(text: &str, enabled: bool) -> String {
    if !enabled {
        return text.to_owned();
    }
    text.split_inclusive('\n')
        .map(|line| {
            let content = line.strip_suffix('\n').unwrap_or(line);
            let suffix = if line.ends_with('\n') { "\n" } else { "" };
            if content.starts_with('#')
                || content.starts_with("Report:")
                || content.starts_with("Named baseline")
            {
                format!("\x1b[1;36m{content}\x1b[0m{suffix}")
            } else if content.contains(": median ") {
                format!("\x1b[1m{content}\x1b[0m{suffix}")
            } else {
                line.to_owned()
            }
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_policy_and_styling_preserve_plain_text() {
        assert!(ConsoleColor::Auto.enabled(true, false));
        assert!(!ConsoleColor::Auto.enabled(false, false));
        assert!(!ConsoleColor::Auto.enabled(true, true));
        assert!(ConsoleColor::Always.enabled(false, true));
        assert!(!ConsoleColor::Never.enabled(true, false));
        let text = "# Results\n| value | unit |\ncase: median 3 ns/op\n";
        let rendered = render(text, true);
        assert_eq!(
            rendered
                .replace("\x1b[1;36m", "")
                .replace("\x1b[1m", "")
                .replace("\x1b[0m", ""),
            text
        );
        assert!(rendered.contains("\x1b[1;36m# Results\x1b[0m\n"));
        assert_eq!(render(text, false), text);
        assert!(ConsoleColor::parse("sometimes").is_err());
    }
}

/// Layout of human results; protocol and report formats are independent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsoleFormat {
    #[default]
    Table,
    Tree,
}
impl ConsoleFormat {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "table" => Ok(Self::Table),
            "tree" => Ok(Self::Tree),
            _ => Err(error("output format must be table or tree")),
        }
    }
}
// Keep user-provided labels on one terminal line without changing stored identity.
fn tree_text(value: &str) -> String {
    value.chars().fold(String::new(), |mut text, ch| {
        if ch.is_control() {
            text.extend(ch.escape_default());
        } else {
            text.push(ch);
        }
        text
    })
}

#[derive(Default)]
struct Node {
    children: Vec<(String, Node)>,
    values: Vec<String>,
}
impl Node {
    fn insert(&mut self, path: &str) -> &mut Self {
        let mut node = self;
        for name in path.split('/') {
            let index = node
                .children
                .iter()
                .position(|(label, _)| label == name)
                .unwrap_or_else(|| {
                    node.children.push((name.into(), Node::default()));
                    node.children.len() - 1
                });
            node = &mut node.children[index].1;
        }
        node
    }
    fn render(&self, prefix: &str, output: &mut String) {
        let count = self.children.len() + self.values.len();
        for (index, value) in self.values.iter().enumerate() {
            let value = tree_text(value);
            output.push_str(&format!(
                "{prefix}{} {value}\n",
                if index + 1 == count {
                    "└──"
                } else {
                    "├──"
                }
            ));
        }
        for (index, (name, node)) in self.children.iter().enumerate() {
            let last = self.values.len() + index + 1 == count;
            let name = tree_text(name);
            output.push_str(&format!(
                "{prefix}{} {name}\n",
                if last { "└──" } else { "├──" }
            ));
            node.render(
                &format!("{prefix}{}", if last { "    " } else { "│   " }),
                output,
            );
        }
    }
}
pub(crate) fn tree_names<'a>(names: impl IntoIterator<Item = &'a str>) -> String {
    let mut root = Node::default();
    for name in names {
        root.insert(name);
    }
    let mut result = String::new();
    root.render("", &mut result);
    result
}
/// Keep deep nesting from consuming a terminal's entire width. Pipes retain
/// the full tree for scripts and logs; names and result text are never shortened.
pub(crate) fn terminal_tree(text: &str) -> String {
    use std::io::IsTerminal;
    let width = std::io::stdout()
        .is_terminal()
        .then(|| terminal_size::terminal_size().map(|(width, _)| usize::from(width.0)))
        .flatten();
    compact_tree_indent(text, width)
}

fn compact_tree_indent(text: &str, width: Option<usize>) -> String {
    let Some(width) = width else {
        return text.to_owned();
    };
    let maximum_depth = (width / 3 / 4).max(1);
    let mut changed = false;
    let mut output = String::new();
    for line in text.split_inclusive('\n') {
        let mut remaining = line;
        let mut depth = 0;
        while let Some(rest) = remaining
            .strip_prefix("    ")
            .or_else(|| remaining.strip_prefix("│   "))
        {
            remaining = rest;
            depth += 1;
        }
        if depth > maximum_depth && (remaining.starts_with("├── ") || remaining.starts_with("└── "))
        {
            changed = true;
            output.push_str(&format!(
                "{}[{depth}] {remaining}",
                "    ".repeat(maximum_depth)
            ));
        } else {
            output.push_str(line);
        }
    }
    if changed {
        output.insert_str(0, "Deep tree: [N] denotes nesting depth.\n");
    }
    output
}

fn rate(count: f64, nanos: f64, unit: &str, bytes: Option<crate::report::BytesFormat>) -> String {
    let value = count / nanos * 1e9;
    if nanos <= 0.0 || !value.is_finite() {
        return "unavailable".into();
    }
    let (mut value, mut label) = crate::report::display_throughput(value, unit);
    if matches!(unit, "bytes" | "bits") {
        let binary = unit == "bytes" && bytes != Some(crate::report::BytesFormat::Decimal);
        let (base, units) = if unit == "bits" {
            (
                1000.0,
                ["bits", "kbit", "Mbit", "Gbit", "Tbit", "Pbit", "Ebit"],
            )
        } else if binary {
            (1024.0, ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"])
        } else {
            (1000.0, ["B", "kB", "MB", "GB", "TB", "PB", "EB"])
        };
        let mut index = 0;
        while index + 1 < units.len() && value >= base {
            value /= base;
            index += 1;
        }
        label = format!("{}/s", units[index]);
    }
    format!("{value:.4} {label}")
}

pub(crate) fn tree(
    rows: &[crate::report::DescriptiveRow],
    bytes: Option<crate::report::BytesFormat>,
) -> String {
    let mut root = Node::default();
    for row in rows {
        let value = row
            .summary
            .as_ref()
            .map(|s| s.median.to_string())
            .unwrap_or_else(|| "unavailable".into());
        let mut line = format!(
            "{} [{} process {}]: median {} {}{}; missing {}",
            row.metric,
            row.variant,
            row.process,
            value,
            row.unit,
            if row.normalized_per_operation {
                "/op"
            } else {
                ""
            },
            row.unavailable
        );
        if let Some(summary) = &row.summary {
            line.push_str(&format!(
                "; min {}; max {}; mean {}; samples {}; operations {}",
                summary.minimum, summary.maximum, summary.mean, summary.count, row.operations
            ));
        } else {
            line.push_str(&format!("; samples 0; operations {}", row.operations));
        }
        let node = root.insert(&row.case);
        node.values.push(line);
        if let Some(summary) = row.summary.as_ref().filter(|_| row.unit == "ns") {
            for (unit, counts) in &row.associated_counters {
                node.values.push(format!(
                    "throughput {unit} [{} process {}], paired with time: fastest {}; slowest {}; median {}; aggregate {}",
                    row.variant, row.process,
                    rate(counts.fastest, summary.minimum, unit, bytes),
                    rate(counts.slowest, summary.maximum, unit, bytes),
                    rate(counts.median, summary.median, unit, bytes),
                    row.operation_weighted_mean.map(|time| rate(counts.operation_mean, time, unit, bytes)).unwrap_or_else(|| "unavailable".into()),
                ));
            }
        }
        if let Some(summary) = &row.worker_wall_per_operation {
            node.values.push(format!(
                "worker-slot wall [{} process {}]: median {} ns/op; min {}; max {}; mean {}; samples {} (all waves combined)",
                row.variant, row.process, summary.median, summary.minimum, summary.maximum, summary.mean, summary.count
            ));
            for (unit, counts) in &row.worker_associated_counters {
                node.values.push(format!(
                    "worker-slot throughput {unit} [{} process {}], paired with worker time: fastest {}; slowest {}; median {}; aggregate {}",
                    row.variant, row.process,
                    rate(counts.fastest, summary.minimum, unit, bytes),
                    rate(counts.slowest, summary.maximum, unit, bytes),
                    rate(counts.median, summary.median, unit, bytes),
                    row.worker_operation_weighted_mean.map(|time| rate(counts.operation_mean, time, unit, bytes)).unwrap_or_else(|| "unavailable".into()),
                ));
            }
        }
        if let Some(workers) = &row.worker_allocations {
            node.values.push(format!(
                "worker-wave allocations [{} process {}]: {} worker-wave records; fastest sample {} wave {} worker {}; slowest sample {} wave {} worker {}",
                row.variant, row.process, workers.samples,
                workers.fastest.sequence, workers.fastest.wave, workers.fastest.worker,
                workers.slowest.sequence, workers.slowest.wave, workers.slowest.worker,
            ));
        }
        for (population, allocations) in std::iter::once(("", &row.associated_allocations)).chain(
            row.worker_allocations
                .as_ref()
                .map(|workers| ("worker-wave ", &workers.allocations)),
        ) {
            for (metric, allocation) in allocations {
                let label = format!(
                    "{population}{metric} [{} process {}], paired with time",
                    row.variant, row.process
                );
                if let Some(value) = allocation {
                    node.values.push(format!(
                        "{label}: fastest {}; slowest {}; median {}; mean {} {}{}",
                        value.fastest,
                        value.slowest,
                        value.median,
                        value.mean,
                        value.unit,
                        if value.normalized_per_operation {
                            "/op"
                        } else {
                            if population.is_empty() {
                                " (sample peak)"
                            } else {
                                " (wave peak)"
                            }
                        }
                    ));
                    if let Some(per_op) = &value.per_operation {
                        node.values.push(format!("{label} (peak per operation): fastest {}; slowest {}; median {}; mean {} {}/op",
                        per_op.fastest, per_op.slowest, per_op.median, per_op.operation_mean, value.unit));
                    }
                } else {
                    node.values
                        .push(format!("{label}: unavailable (incomplete sample pairing)"));
                }
            }
        }
    }
    let mut result = String::from("# Results\n");
    if rows
        .iter()
        .any(|row| row.worker_wall_per_operation.is_some() || row.worker_allocations.is_some())
    {
        result.push_str("Worker slots describe normalized batches, not independent process repetitions or individual-operation latency.\n");
    }
    root.render("", &mut result);
    result
}
#[cfg(test)]
mod tree_tests {
    use super::*;
    #[test]
    fn worker_tree_preserves_population_and_weighted_throughput() {
        let mut suite = crate::Suite::new("workers");
        suite.bench_threads_with_local_input(
            "case",
            2,
            || (),
            |_| (),
            crate::DropPolicy::InsideTiming,
        );
        suite.work_units("items", 2);
        suite.config(crate::Config {
            samples: 2,
            warmup: std::time::Duration::ZERO,
            ..Default::default()
        });
        suite.sampling(crate::Sampling {
            iterations: Some(2),
            ..Default::default()
        });
        let mut run = suite.run("").unwrap();
        for worker in &mut run.worker_timings {
            worker.operations = if worker.sequence == 0 { 1 } else { 3 };
            worker.wall_ns =
                (worker.operations * (10 + worker.sequence * 20 + worker.worker * 10)).to_string();
        }
        for observation in &mut run.observations {
            observation.operations = if observation.sequence == 0 { 2 } else { 6 };
            observation.value = Some(
                if observation.sequence == 0 {
                    "20"
                } else {
                    "120"
                }
                .into(),
            );
        }
        run.validate().unwrap();
        let original = serde_json::to_vec(&run).unwrap();
        let rows = crate::report::descriptive(&run).unwrap();
        assert_eq!(rows[0].worker_operation_weighted_mean, Some(30.));
        let text = tree(&rows, None);
        assert!(text.contains("worker-slot wall [candidate process 0]: median 25 ns/op; min 10; max 40; mean 25; samples 4"), "{text}");
        assert!(text.contains("worker-slot throughput items [candidate process 0], paired with worker time: fastest 200000000.0000 items/s; slowest 50000000.0000 items/s; median 80000000.0000 items/s; aggregate 66666666.6667 items/s"), "{text}");
        assert!(text.contains("not independent process repetitions"));
        assert_eq!(original, serde_json::to_vec(&run).unwrap());
        // Legacy summary JSON has no worker-weighted mean; do not substitute the
        // aggregate elapsed time or the unweighted worker mean for missing data.
        let mut old = serde_json::to_value(&rows).unwrap();
        old[0]
            .as_object_mut()
            .unwrap()
            .remove("worker_operation_weighted_mean");
        let old: Vec<crate::report::DescriptiveRow> = serde_json::from_value(old).unwrap();
        let text = tree(&old, None);
        assert!(text.contains("median 80000000.0000 items/s; aggregate unavailable"));
    }

    #[test]
    fn result_tree_keeps_processes_and_unavailable_values_separate() {
        let mut rec = crate::Recorder::new();
        rec.case(crate::Case {
            id: "group/case".into(),
            contract: Default::default(),
            metrics: vec![crate::Metric::duration("wall", "fixture", "batch_total")],
        })
        .unwrap();
        rec.observe("group/case", "wall", 20).unwrap();
        let mut run = rec.finish().unwrap();
        run.observations[0].operations = 2;
        let mut missing = run.observations[0].clone();
        missing.process = 1;
        missing.value = None;
        missing.availability = crate::Availability::Unsupported("unavailable timer".into());
        run.observations.push(missing);
        let text = tree(&crate::report::descriptive(&run).unwrap(), None);
        assert!(text.contains("wall [candidate process 0]: median 10 ns/op; missing 0"));
        assert!(text.contains("wall [candidate process 1]: median unavailable ns/op; missing 1"));
        assert_eq!(text.matches("└── case").count(), 1);
    }
    #[test]
    fn result_tree_reports_normalized_distribution_and_exact_operation_count() {
        let mut rec = crate::Recorder::new();
        rec.case(crate::Case {
            id: "case".into(),
            contract: Default::default(),
            metrics: vec![crate::Metric::duration("wall", "fixture", "batch_total")],
        })
        .unwrap();
        rec.observe("case", "wall", 20).unwrap();
        rec.observe("case", "wall", 80).unwrap();
        let mut run = rec.finish().unwrap();
        run.observations[0].operations = 2;
        run.observations[1].operations = 4;
        let rows = crate::report::descriptive(&run).unwrap();
        let output = tree(&rows, None);
        assert!(output.contains("median 15 ns/op"), "{output}");
        assert!(
            output.contains("min 10; max 20; mean 15; samples 2; operations 6"),
            "{output}"
        );
        let mut large = rows;
        large[0].operations = "18446744073709551616".into();
        assert!(tree(&large, None).contains("operations 18446744073709551616"));
    }
    #[test]
    fn tree_preserves_time_pairing_rates_peaks_and_missing_allocations() {
        let mut rec = crate::Recorder::new();
        let mut count = crate::Metric::duration("alloc.count", "fixture", "batch_total");
        count.unit = "allocations".into();
        let mut peak = crate::Metric::duration("alloc.peak", "fixture", "sample_peak");
        peak.unit = "bytes".into();
        rec.case(crate::Case {
            id: "group/case".into(),
            contract: [
                ("work.counter.cycles".into(), "2500000000".into()),
                ("work.counter.chars".into(), "0".into()),
            ]
            .into(),
            metrics: vec![
                crate::Metric::duration("wall", "fixture", "batch_total"),
                count,
                peak,
            ],
        })
        .unwrap();
        for (wall, allocations, peak) in [(1_000_000_000, 9, 200), (2_000_000_000, 1, 50)] {
            rec.observe("group/case", "wall", wall).unwrap();
            rec.observe("group/case", "alloc.count", allocations)
                .unwrap();
            rec.observe("group/case", "alloc.peak", peak).unwrap();
        }
        let mut run = rec.finish().unwrap();
        run.cases[0]
            .contract
            .insert("work.input.items".into(), "batch_total".into());
        run.cases[0]
            .contract
            .insert("work.input.bytes".into(), "batch_total".into());
        for o in run.observations.iter_mut().filter(|o| o.metric == "wall") {
            o.work_totals.insert(
                "items".into(),
                if o.sequence == 0 { "100" } else { "3" }.into(),
            );
            o.work_totals.insert(
                "bytes".into(),
                if o.sequence == 0 { "1024" } else { "4096" }.into(),
            );
        }
        let original = serde_json::to_vec(&run).unwrap();
        let rows = crate::report::descriptive(&run).unwrap();
        let binary = tree(&rows, Some(crate::report::BytesFormat::Binary));
        assert!(
            binary.contains("fastest 100.0000 items/s; slowest 1.5000 items/s"),
            "{binary}"
        );
        assert!(
            binary.contains("fastest 1.0000 KiB/s; slowest 2.0000 KiB/s"),
            "{binary}"
        );
        assert!(
            binary.contains("fastest 2.5000 GHz; slowest 1.2500 GHz"),
            "{binary}"
        );
        assert!(binary.contains("fastest 0.0000 chars/s"), "{binary}");
        assert!(
            binary.contains(
                "paired with time: fastest 9; slowest 1; median 5; mean 5 allocations/op"
            ),
            "{binary}"
        );
        assert!(binary.contains("bytes (sample peak)"));
        assert!(binary.contains("(peak per operation)"));
        assert!(
            tree(&rows, Some(crate::report::BytesFormat::Decimal)).contains("fastest 1.0240 kB/s")
        );
        assert_eq!(serde_json::to_vec(&run).unwrap(), original);
        run.observations
            .retain(|o| o.metric != "alloc.count" || o.sequence == 0);
        assert!(
            tree(&crate::report::descriptive(&run).unwrap(), None)
                .contains("unavailable (incomplete sample pairing)")
        );
        assert_eq!(rate(0.0, 0.0, "items", None), "unavailable");
    }

    #[test]
    fn tree_labels_keep_unicode_and_escape_control_characters() {
        let name = "группа/🦀\ncase\t\x1b[2J";
        assert_eq!(
            tree_names([name]),
            "└── группа\n    └── 🦀\\ncase\\t\\u{1b}[2J\n"
        );
        let mut root = Node::default();
        root.insert("case")
            .values
            .push("custom\rmetric\x08: 3".into());
        let mut text = String::new();
        root.render("", &mut text);
        assert_eq!(text, "└── case\n    └── custom\\rmetric\\u{8}: 3\n");
        assert_eq!(root.children[0].1.values[0], "custom\rmetric\x08: 3");
        let long = "длинное имя 🦀".repeat(40);
        assert_eq!(tree_names([long.as_str()]), format!("└── {long}\n"));
    }

    #[test]
    fn narrow_terminal_caps_indentation_without_losing_identity_or_values() {
        let label = "длинное имя 🦀".repeat(40);
        let input = format!(
            "└── root\n{}├── {label}\n{}└── median 3 ns/op\n",
            "│   ".repeat(64),
            "    ".repeat(65)
        );
        assert_eq!(compact_tree_indent(&input, None), input);
        for width in [40, 80, 120] {
            let output = compact_tree_indent(&input, Some(width));
            assert!(output.starts_with("Deep tree: [N] denotes nesting depth.\n└── root\n"));
            assert!(output.contains(&format!("[64] ├── {label}\n")));
            assert!(output.contains("[65] └── median 3 ns/op\n"));
            for line in output.lines().skip(2) {
                assert!(line.len() - line.trim_start().len() <= width / 3);
            }
        }
        let ordinary = "└── root\n    └── child\n";
        assert_eq!(compact_tree_indent(ordinary, Some(80)), ordinary);
    }

    #[test]
    fn deep_tree_preserves_long_labels_and_every_leaf() {
        let levels: Vec<_> = (0..64).map(|i| format!("level-{i:02}")).collect();
        let prefix = levels.join("/");
        let label = "длинное имя 🦀".repeat(40);
        let names: Vec<_> = (0..256)
            .map(|i| format!("{prefix}/{i:03}-{label}"))
            .collect();
        let output = tree_names(names.iter().map(String::as_str));
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), 64 + 256);
        for (depth, name) in levels.iter().enumerate() {
            assert_eq!(lines[depth], format!("{}└── {name}", "    ".repeat(depth)));
        }
        for i in 0..256 {
            let branch = if i == 255 { "└──" } else { "├──" };
            assert_eq!(
                lines[64 + i],
                format!("{}{branch} {i:03}-{label}", "    ".repeat(64))
            );
        }
    }

    #[test]
    fn hierarchy_preserves_selection_order_and_prefix_cases() {
        assert_eq!(
            tree_names(["suite/z/10", "suite/z/2", "suite/a", "suite/a/nested"]),
            "└── suite\n    ├── z\n    │   ├── 10\n    │   └── 2\n    └── a\n        └── nested\n"
        );
        let mut node = Node::default();
        node.insert("parent").values.push("parent value".into());
        node.insert("parent/child")
            .values
            .push("child value".into());
        let mut text = String::new();
        node.render("", &mut text);
        assert_eq!(
            text,
            "└── parent\n    ├── parent value\n    └── child\n        └── child value\n"
        );
        assert_eq!(tree_names([]), "");
        assert!(ConsoleFormat::parse("wide").is_err());
    }
}
