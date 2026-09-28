//! `cargo nextest` support: seed the report from `cargo nextest list --message-format json`
//! (a stable, documented format) and follow the status lines of `cargo nextest run`.
//!
//! ```text
//!     Starting 5 tests across 2 binaries (1 test skipped)
//!         PASS [   0.004s] demo tests::adds
//!         FAIL [   0.005s] (2/5) demo tests::fails
//!   stdout ───                                   (or `──── STDOUT: …` / `--- STDOUT: … ---`)
//!     …captured output, until the next status line…
//!   TRY 2 PASS [   0.004s] demo::it flaky_one
//!      SIGSEGV [   0.010s] demo tests::crashes
//!         SKIP [         ] demo tests::skipped
//!      Summary [   0.020s] 5 tests run: 3 passed, 2 failed, 1 skipped
//! ```
//!
//! Only the shape `<STATUS WORDS> [<duration>] [(<n>/<total>)] <binary-id> <test name>` is
//! relied on. The runner sets [`RUN_ENV`] so the lines we need are printed whatever the user's
//! nextest config says (command-line flags still win).
use super::{
    libtest::Collector,
    model::{Suite, SuiteKind, SuiteState, TestStatus},
};
use serde_json::Value;
use std::path::Path;

/// Environment for `cargo nextest run`: every result on its own line (skips included),
/// failure output right after its status line, no final re-listing, no progress bar.
pub const RUN_ENV: [(&str, &str); 6] = [
    ("NEXTEST_STATUS_LEVEL", "skip"),
    ("NEXTEST_FINAL_STATUS_LEVEL", "none"),
    ("NEXTEST_FAILURE_OUTPUT", "immediate"),
    ("NEXTEST_SUCCESS_OUTPUT", "never"),
    ("NEXTEST_HIDE_PROGRESS_BAR", "1"),
    ("NEXTEST_NO_INPUT_HANDLER", "1"),
];

/// `cargo nextest run` options that `cargo nextest list` rejects, with whether they take a
/// value. They are dropped from the listing pass only.
const RUN_ONLY: [(&str, bool); 15] = [
    ("--no-fail-fast", false),
    ("--fail-fast", false),
    ("--no-capture", false),
    ("--nocapture", false),
    ("--retries", true),
    ("--max-fail", true),
    ("-j", true),
    ("--test-threads", true),
    ("--status-level", true),
    ("--final-status-level", true),
    ("--failure-output", true),
    ("--success-output", true),
    ("--no-tests", true),
    ("--hide-progress-bar", false),
    ("--flaky-result", true),
];

#[derive(Debug, Default)]
pub struct State {
    /// Binary ids from the list, longest first (ids are prefixes of each other: `demo`,
    /// `demo::it`).
    binaries: Vec<String>,
    /// Test that owns the lines after its failure line.
    capture: Option<(usize, String)>,
    /// Tests that failed an attempt earlier in this run (for flaky detection).
    failed_attempts: Vec<usize>,
    listed: bool,
}

/// The listing pass: `cargo nextest list --message-format json [args] [-- filters]`.
pub fn list_args(cargo_args: &[String], test_args: &[String]) -> Vec<String> {
    let mut args = vec![
        "nextest".to_string(),
        "list".into(),
        "--message-format".into(),
        "json".into(),
    ];
    let mut skip_value = false;
    for arg in cargo_args {
        if skip_value {
            skip_value = false;
            continue;
        }
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, _)) if flag.starts_with('-') => (flag, true),
            _ => (arg.as_str(), false),
        };
        if let Some((_, takes_value)) = RUN_ONLY.iter().find(|(name, _)| *name == flag) {
            skip_value = *takes_value && !inline;
            continue;
        }
        args.push(arg.clone());
    }
    if !test_args.is_empty() {
        args.push("--".into());
        args.extend(test_args.iter().cloned());
    }
    args
}

/// The real run: `cargo nextest run --no-fail-fast [args] [-- filters]`.
pub fn run_args(cargo_args: &[String], test_args: &[String]) -> Vec<String> {
    let mut args = vec!["nextest".to_string(), "run".into()];
    if !cargo_args
        .iter()
        .any(|a| a == "--no-fail-fast" || a == "--fail-fast" || a.starts_with("--max-fail"))
    {
        args.push("--no-fail-fast".into());
    }
    args.extend(cargo_args.iter().cloned());
    if !test_args.is_empty() {
        args.push("--".into());
        args.extend(test_args.iter().cloned());
    }
    args
}

/// A listing pass that failed because `list` does not know one of the run's options — not
/// a build failure; the run goes ahead without a known total.
pub fn list_rejected_args(output_tail: &str) -> bool {
    output_tail.contains("unexpected argument") || output_tail.contains("wasn't expected")
}

impl Collector {
    /// Feed one line of the listing pass. Returns true once the JSON document was read.
    pub fn nextest_list_line(&mut self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.starts_with('{')
            && let Ok(doc) = serde_json::from_str::<Value>(trimmed)
            && doc.get("rust-suites").is_some()
        {
            self.seed_nextest(&doc);
            return true;
        }
        self.nextest_activity(trimmed);
        false
    }

    /// Suites and tests from `cargo nextest list --message-format json`.
    fn seed_nextest(&mut self, doc: &Value) {
        let Some(suites) = doc.get("rust-suites").and_then(Value::as_object) else {
            return;
        };
        for (binary_id, suite) in suites {
            if suite.get("status").and_then(Value::as_str) == Some("skipped") {
                continue;
            }
            let kind = suite.get("kind").and_then(Value::as_str).unwrap_or("");
            let binary = suite
                .get("binary-path")
                .and_then(Value::as_str)
                .and_then(|p| Path::new(p).file_stem())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let index = self.nextest_suite(binary_id, kind, binary);
            let Some(cases) = suite.get("testcases").and_then(Value::as_object) else {
                continue;
            };
            for (name, case) in cases {
                let filter = case.get("filter-match");
                let status = filter.and_then(|f| f.get("status")).and_then(Value::as_str);
                let reason = filter.and_then(|f| f.get("reason")).and_then(Value::as_str);
                match (status, reason) {
                    (Some("mismatch"), Some("ignored")) => {
                        let test = self.test_index(index, name);
                        self.report.tests[test].status = TestStatus::Ignored;
                    }
                    (Some("mismatch"), _) => {}
                    _ => {
                        self.test_index(index, name);
                    }
                }
            }
        }
        self.nextest.listed = true;
        self.report.recount();
    }

    fn nextest_suite(&mut self, binary_id: &str, kind: &str, binary: String) -> usize {
        if let Some(index) = self
            .report
            .suites
            .iter()
            .position(|s| s.name == binary_id && s.target == kind_label(kind))
        {
            return index;
        }
        let index = self.add_suite(Suite {
            id: binary_id.to_string(),
            name: binary_id.to_string(),
            target: kind_label(kind).to_string(),
            binary,
            kind: match kind {
                "test" => SuiteKind::Integration,
                _ => SuiteKind::Unit,
            },
            ..Suite::default()
        });
        if !self.nextest.binaries.iter().any(|b| b == binary_id) {
            self.nextest.binaries.push(binary_id.to_string());
            self.nextest
                .binaries
                .sort_by_key(|b| std::cmp::Reverse(b.len()));
        }
        index
    }

    /// Feed one line of `cargo nextest run`. `now_s` is seconds since the run started.
    pub fn nextest_run_line(&mut self, line: &str, now_s: f64) {
        let line = line.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();
        if let Some(status) = parse_status(trimmed) {
            self.nextest_end_capture();
            self.nextest_status(status, now_s);
            return;
        }
        let ends_capture = trimmed.starts_with("Summary [")
            || trimmed.starts_with("Cancelling")
            || trimmed.starts_with("error:")
            || (trimmed.len() >= 8 && trimmed.chars().all(|c| c == '─' || c == '-'));
        if ends_capture {
            self.nextest_end_capture();
        } else if let Some((_, text)) = self.nextest.capture.as_mut() {
            text.push_str(line);
            text.push('\n');
            return;
        }
        if let Some(rest) = trimmed.strip_prefix("Starting ") {
            self.activity = format!("nextest: starting {rest}");
            return;
        }
        self.nextest_activity(trimmed);
    }

    fn nextest_status(&mut self, status: StatusLine<'_>, now_s: f64) {
        let (binary_id, name) = self.split_binary(status.rest);
        let outcome = outcome(&status.words);
        self.activity = format!("{} {binary_id} {name}", status.words.join(" "));
        let Some(outcome) = outcome else {
            return;
        };
        let suite = match self.report.suites.iter().position(|s| s.name == binary_id) {
            Some(index) => index,
            None => self.nextest_suite(binary_id, "", String::new()),
        };
        if self.report.suites[suite].state == SuiteState::Pending {
            self.report.suites[suite].state = SuiteState::Running;
        }
        let index = self.test_index(suite, name);
        let retried = status.attempt.is_some_and(|a| a > 1) || status.words.contains(&"FLAKY");
        let test = &mut self.report.tests[index];
        test.finished_s = Some((now_s * 1000.0).round() / 1000.0);
        match outcome {
            TestStatus::Failed => {
                test.status = TestStatus::Failed;
                self.nextest.failed_attempts.push(index);
                self.nextest.capture = Some((index, String::new()));
            }
            TestStatus::Passed => {
                test.status = TestStatus::Passed;
                if retried || self.nextest.failed_attempts.contains(&index) {
                    let attempt = status.attempt.map(|a| format!(" (attempt {a})"));
                    test.flaky
                        .push(format!("passed on retry{}", attempt.unwrap_or_default()));
                }
            }
            other => test.status = other,
        }
    }

    /// `demo::it some::test` → (`demo::it`, `some::test`), preferring known binary ids.
    fn split_binary<'a>(&self, rest: &'a str) -> (&'a str, &'a str) {
        for binary in &self.nextest.binaries {
            if let Some(name) = rest
                .strip_prefix(binary.as_str())
                .and_then(|r| r.strip_prefix(' '))
            {
                return (&rest[..binary.len()], name.trim());
            }
        }
        match rest.split_once(' ') {
            Some((binary, name)) => (binary, name.trim()),
            None => ("unknown", rest),
        }
    }

    fn nextest_end_capture(&mut self) {
        if let Some((index, text)) = self.nextest.capture.take() {
            let text = dedent(text.trim_matches('\n'));
            if text.trim().is_empty() {
                return;
            }
            let test = &mut self.report.tests[index];
            test.output = Some(match test.output.take() {
                Some(previous) => format!("{previous}\n{text}"),
                None => text,
            });
        }
    }

    /// Close a nextest run: every binary ran to the end (a crash is per test), and listed
    /// tests without a result did not run.
    pub fn nextest_finish(&mut self) {
        self.nextest_end_capture();
        for suite in &mut self.report.suites {
            if suite.state == SuiteState::Running {
                suite.state = SuiteState::Done;
            }
        }
        for test in &mut self.report.tests {
            if test.status == TestStatus::Pending {
                test.status = TestStatus::NotRun;
            }
        }
        self.report.recount();
    }

    pub fn nextest_listed(&self) -> bool {
        self.nextest.listed
    }

    fn nextest_activity(&mut self, trimmed: &str) {
        for prefix in ["Compiling ", "Checking ", "Building ", "Finished ", "error"] {
            if trimmed.starts_with(prefix) {
                self.activity = trimmed.chars().take(200).collect();
                return;
            }
        }
    }
}

fn kind_label(kind: &str) -> &str {
    if kind.is_empty() { "nextest" } else { kind }
}

#[derive(Debug, PartialEq)]
struct StatusLine<'a> {
    /// Words before the bracket: `PASS`, `TRY 2 FAIL`, `SIGSEGV`, `LEAK-FAIL`.
    words: Vec<&'a str>,
    attempt: Option<u32>,
    /// `<binary-id> <test name>`.
    rest: &'a str,
}

/// `TRY 2 FAIL [   0.004s] (3/9) demo tests::fails` → words, attempt and the rest.
fn parse_status(trimmed: &str) -> Option<StatusLine<'_>> {
    let open = trimmed.find('[')?;
    let head = trimmed[..open].trim();
    let words: Vec<&str> = head.split_whitespace().collect();
    let status_word = |w: &&str| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '/')
    };
    if words.is_empty()
        || !words.iter().all(status_word)
        || !words
            .iter()
            .any(|w| w.chars().any(|c| c.is_ascii_uppercase()))
    {
        return None;
    }
    let close = open + trimmed[open..].find(']')?;
    // `[   0.004s]`, `[> 60.000s]`, `[         ]` — anything else is not a status line
    // (`ERROR [main] …` in captured output must stay output).
    if !trimmed[open + 1..close]
        .chars()
        .all(|c| c.is_ascii_digit() || " .smh>".contains(c))
    {
        return None;
    }
    let mut rest = trimmed[close + 1..].trim_start();
    if let Some(inner) = rest.strip_prefix('(')
        && let Some((counter, after)) = inner.split_once(')')
        && counter
            .chars()
            .all(|c| c.is_ascii_digit() || c == '/' || c == ' ')
    {
        rest = after.trim_start();
    }
    if rest.is_empty() {
        return None;
    }
    let attempt = match words.as_slice() {
        ["TRY", n, ..] => n.parse().ok(),
        _ => None,
    };
    Some(StatusLine {
        words,
        attempt,
        rest,
    })
}

/// Final outcome of a status line, or `None` for progress lines (`START`, `SLOW`, `RETRY`…).
fn outcome(words: &[&str]) -> Option<TestStatus> {
    let last = words
        .iter()
        .rev()
        .find(|w| w.chars().any(|c| c.is_ascii_uppercase()))?;
    match *last {
        "PASS" | "FLAKY" | "LEAK" => Some(TestStatus::Passed),
        "SKIP" => Some(TestStatus::Ignored),
        "FAIL" | "LEAK-FAIL" | "TIMEOUT" | "ABORT" | "CRASH" | "EXC" => Some(TestStatus::Failed),
        w if w.starts_with("SIG") => Some(TestStatus::Failed),
        _ => None,
    }
}

/// Newer nextest indents captured output by four spaces; drop the common indent.
fn dedent(text: &str) -> String {
    let indent = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);
    text.lines()
        .map(|l| l.get(indent..).unwrap_or_else(|| l.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testrun::model::Report;

    const LIST: &str = r#"{"rust-build-meta":{},"test-count":5,"rust-suites":{"demo":{"package-name":"demo","binary-id":"demo","binary-name":"demo","kind":"lib","binary-path":"/t/target/debug/deps/demo-0123456789abcdef","status":"listed","testcases":{"tests::adds":{"ignored":false,"filter-match":{"status":"matches"}},"tests::fails":{"ignored":false,"filter-match":{"status":"matches"}},"tests::skipped":{"ignored":true,"filter-match":{"status":"mismatch","reason":"ignored"}},"tests::filtered":{"ignored":false,"filter-match":{"status":"mismatch","reason":"string"}}}},"demo::it":{"binary-id":"demo::it","kind":"test","binary-path":"/t/target/debug/deps/it-fedcba9876543210","status":"listed","testcases":{"flaky_one":{"ignored":false,"filter-match":{"status":"matches"}},"crashes":{"ignored":false,"filter-match":{"status":"matches"}},"never_ran":{"ignored":false,"filter-match":{"status":"matches"}}}}}}"#;

    const RUN_NEW: &str = "    Starting 5 tests across 2 binaries (1 test skipped)
        PASS [   0.004s] (1/5) demo tests::adds
        FAIL [   0.005s] (2/5) demo tests::fails
  stdout ───

    running 1 test
    test tests::fails ... FAILED

  stderr ───

    thread 'tests::fails' panicked at src/lib.rs:17:9:
    math is broken

  TRY 1 FAIL [   0.004s] (3/5) demo::it flaky_one
  TRY 2 PASS [   0.004s] (3/5) demo::it flaky_one
     SIGSEGV [   0.010s] (4/5) demo::it crashes
        SKIP [         ] demo tests::skipped
────────────
     Summary [   0.020s] 5 tests run: 3 passed, 2 failed, 1 skipped
error: test run failed";

    fn status(c: &Collector, suite: &str, name: &str) -> TestStatus {
        let i = c.find_test(suite, name).expect("test exists");
        c.report.tests[i].status
    }

    #[test]
    fn list_seeds_suites_and_skips_filtered_tests() {
        let mut c = Collector::new(Report::default());
        assert!(!c.nextest_list_line("   Compiling demo v0.1.0"));
        assert!(c.nextest_list_line(LIST));
        assert_eq!(c.report.suites.len(), 2);
        assert_eq!(c.report.suites[0].id, "demo");
        assert_eq!(c.report.suites[0].binary, "demo-0123456789abcdef");
        assert_eq!(c.suite_for_binary("it-fedcba9876543210"), Some("demo::it"));
        assert_eq!(
            c.report.totals.total, 6,
            "filtered test is not part of the run"
        );
        assert_eq!(status(&c, "demo", "tests::skipped"), TestStatus::Ignored);
        assert!(c.find_test("demo", "tests::filtered").is_none());
    }

    #[test]
    fn run_lines_give_outcomes_output_and_flaky_retries() {
        let mut c = Collector::new(Report::default());
        c.nextest_list_line(LIST);
        for (i, line) in RUN_NEW.lines().enumerate() {
            c.nextest_run_line(line, i as f64 / 10.0);
        }
        c.nextest_finish();
        assert_eq!(status(&c, "demo", "tests::adds"), TestStatus::Passed);
        assert_eq!(status(&c, "demo", "tests::fails"), TestStatus::Failed);
        assert_eq!(status(&c, "demo", "tests::skipped"), TestStatus::Ignored);
        assert_eq!(status(&c, "demo::it", "flaky_one"), TestStatus::Passed);
        assert_eq!(status(&c, "demo::it", "crashes"), TestStatus::Failed);
        assert_eq!(status(&c, "demo::it", "never_ran"), TestStatus::NotRun);
        let fails = &c.report.tests[c.find_test("demo", "tests::fails").unwrap()];
        let output = fails.output.as_deref().unwrap();
        assert!(output.contains("math is broken"), "{output}");
        assert!(output.starts_with("stdout ───"), "dedented: {output}");
        assert!(!output.contains("Summary"), "{output}");
        let flaky = &c.report.tests[c.find_test("demo::it", "flaky_one").unwrap()];
        assert_eq!(flaky.flaky, ["passed on retry (attempt 2)"]);
        assert!(flaky.output.is_none());
        assert_eq!(c.report.totals.failed, 2);
        assert_eq!(c.report.totals.passed, 2);
        assert!(c.report.suites.iter().all(|s| s.state == SuiteState::Done));
    }

    #[test]
    fn older_formats_and_unlisted_runs_still_parse() {
        let mut c = Collector::new(Report::default());
        let old = "        PASS [   0.003s] demo tests::adds
        FAIL [   0.004s] demo tests::fails
--- STDOUT:              demo tests::fails ---
running 1 test
--- STDERR:              demo tests::fails ---
thread 'tests::fails' panicked at 'boom'
------------
     Summary [   0.005s] 2 tests run: 1 passed, 1 failed, 0 skipped";
        for line in old.lines() {
            c.nextest_run_line(line, 0.0);
        }
        c.nextest_finish();
        assert_eq!(status(&c, "demo", "tests::adds"), TestStatus::Passed);
        let fails = &c.report.tests[c.find_test("demo", "tests::fails").unwrap()];
        assert!(fails.output.as_deref().unwrap().contains("boom"));
        assert!(!fails.output.as_deref().unwrap().contains("Summary"));
    }

    #[test]
    fn status_lines_are_recognised_strictly() {
        let s = parse_status("TRY 2 FAIL [   0.004s] (3/9) demo tests::fails").unwrap();
        assert_eq!(s.words, ["TRY", "2", "FAIL"]);
        assert_eq!(s.attempt, Some(2));
        assert_eq!(s.rest, "demo tests::fails");
        assert!(parse_status("Summary [   0.020s] 5 tests run").is_none());
        assert!(parse_status("thread 'x' panicked at [src]").is_none());
        assert!(parse_status("assert_eq!(v[0], 1)").is_none());
        assert!(parse_status("ERROR [main] connection refused").is_none());
        assert_eq!(outcome(&["SLOW"]), None);
        assert_eq!(outcome(&["LEAK-FAIL"]), Some(TestStatus::Failed));
        assert_eq!(outcome(&["SIGABRT"]), Some(TestStatus::Failed));
    }

    #[test]
    fn list_drops_run_only_options() {
        let args: Vec<String> = [
            "--workspace",
            "--retries",
            "2",
            "--no-fail-fast",
            "-j=4",
            "-p",
            "x",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(
            list_args(&args, &["foo".into()]),
            [
                "nextest",
                "list",
                "--message-format",
                "json",
                "--workspace",
                "-p",
                "x",
                "--",
                "foo"
            ]
        );
        assert_eq!(
            run_args(&args, &[]),
            [
                "nextest",
                "run",
                "--workspace",
                "--retries",
                "2",
                "--no-fail-fast",
                "-j=4",
                "-p",
                "x"
            ]
        );
    }
}
