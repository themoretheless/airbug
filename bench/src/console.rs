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
pub(crate) fn tree(rows: &[crate::report::DescriptiveRow]) -> String {
    let mut root = Node::default();
    for row in rows {
        let value = row
            .summary
            .as_ref()
            .map(|s| s.median.to_string())
            .unwrap_or_else(|| "unavailable".into());
        root.insert(&row.case).values.push(format!(
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
        ));
    }
    let mut result = String::from("# Results\n");
    root.render("", &mut result);
    result
}
#[cfg(test)]
mod tree_tests {
    use super::*;
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
        let text = tree(&crate::report::descriptive(&run).unwrap());
        assert!(text.contains("wall [candidate process 0]: median 10 ns/op; missing 0"));
        assert!(text.contains("wall [candidate process 1]: median unavailable ns/op; missing 1"));
        assert_eq!(text.matches("└── case").count(), 1);
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
