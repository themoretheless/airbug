//! Incremental parser for stable `cargo test` output (cargo + libtest "pretty" format).
//!
//! Only stable, long-lived lines are recognised:
//!
//! ```text
//!      Running unittests src/lib.rs (target/debug/deps/airbug-1a2b3c4d5e6f7a8b)
//!      Running tests/mock.rs (target/debug/deps/mock-1a2b3c4d5e6f7a8b)
//!    Doc-tests airbug
//! running 3 tests
//! test path::to::name ... ok | FAILED | ignored | ignored, reason
//! ---- path::to::name stdout ----
//! test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
//! name: test                         (with `-- --list`)
//! ```
//!
//! cargo writes `Running` to stderr and the test binary writes to stdout; the runner merges both
//! into one pipe so the order is the order they were written in.
use super::model::{Report, Suite, SuiteKind, SuiteState, TestCase, TestStatus};
use std::collections::{HashMap, VecDeque};

const CRASH_TAIL_LINES: usize = 40;

#[derive(Debug, Default)]
pub struct Collector {
    pub report: Report,
    tests: HashMap<String, usize>,
    by_binary: HashMap<String, usize>,
    current: Option<usize>,
    capture: Option<(usize, String)>,
    /// `test name ... ` printed without an outcome (single thread + `--nocapture`).
    dangling: Option<String>,
    recent: VecDeque<String>,
    /// Last line worth showing as activity (`Compiling …`, `Running …`).
    pub activity: String,
}

impl Collector {
    pub fn new(report: Report) -> Self {
        Self {
            report,
            ..Self::default()
        }
    }

    /// Feed one line of `cargo test -- --list` output.
    pub fn list_line(&mut self, line: &str) {
        let line = line.trim_end_matches(['\r', '\n']);
        if self.suite_header(line) {
            return;
        }
        self.note_activity(line);
        let Some(suite) = self.current else {
            return;
        };
        let Some((name, kind)) = line.rsplit_once(": ") else {
            return;
        };
        if kind == "bench" || name.is_empty() || name.starts_with(' ') {
            return;
        }
        if !matches!(kind, "test" | "compile fail" | "compile") {
            return;
        }
        self.test_index(suite, &normalize_name(name));
    }

    /// Feed one line of the real run. `now_s` is seconds since the run started.
    pub fn run_line(&mut self, line: &str, now_s: f64) {
        let line = line.trim_end_matches(['\r', '\n']);
        self.recent.push_back(line.to_string());
        if self.recent.len() > CRASH_TAIL_LINES {
            self.recent.pop_front();
        }

        if let Some(name) = header_name(line) {
            self.end_capture();
            let suite = self.current_or_unknown();
            let index = self.test_index(suite, name);
            self.capture = Some((index, String::new()));
            return;
        }
        if self.capture.is_some() {
            let ends = line == "failures:"
                || line == "successes:"
                || line.starts_with("test result: ")
                || parse_suite_header(line).is_some();
            if ends {
                self.end_capture();
            } else {
                if let Some((_, text)) = self.capture.as_mut() {
                    text.push_str(line);
                    text.push('\n');
                }
                return;
            }
        }

        if self.suite_header(line) {
            return;
        }
        self.note_activity(line);

        if is_running_count(line) {
            if let Some(suite) = self.current {
                self.report.suites[suite].state = SuiteState::Running;
            }
            return;
        }
        if let Some(rest) = line.strip_prefix("test result: ") {
            self.suite_result(rest);
            return;
        }
        if let Some((name, outcome)) = result_line(line) {
            self.dangling = None;
            self.outcome(name, outcome, now_s);
            return;
        }
        if let Some(name) = line
            .strip_prefix("test ")
            .and_then(|rest| rest.trim_end().strip_suffix(" ..."))
        {
            self.dangling = Some(name.to_string());
            return;
        }
        if let Some(name) = self.dangling.clone() {
            let trimmed = line.trim();
            if trimmed == "ok" || trimmed.starts_with("FAILED") || trimmed.starts_with("ignored") {
                self.dangling = None;
                self.outcome(&name, trimmed, now_s);
            }
        }
    }

    /// Close the run: suites that never printed `test result:` crashed, and tests that never
    /// reported did not run.
    pub fn finish(&mut self) {
        self.end_capture();
        if let Some(current) = self.current.take() {
            self.close_suite(current);
        }
        for test in &mut self.report.tests {
            if test.status == TestStatus::Pending {
                test.status = TestStatus::NotRun;
            }
        }
        for suite in &mut self.report.suites {
            if suite.state == SuiteState::Running {
                suite.state = SuiteState::Crashed;
            }
        }
        self.report.recount();
    }

    /// Recognise `Running …` / `Doc-tests …` and switch the current suite.
    fn suite_header(&mut self, line: &str) -> bool {
        let Some(header) = parse_suite_header(line) else {
            return false;
        };
        self.end_capture();
        if let Some(previous) = self.current.take() {
            self.close_suite(previous);
        }
        let index = match self.by_binary.get(&header.binary) {
            Some(&index) => index,
            None => {
                let mut id = header.id();
                let mut n = 2;
                while self.report.suites.iter().any(|s| s.id == id) {
                    id = format!("{} #{n}", header.id());
                    n += 1;
                }
                self.report.suites.push(Suite {
                    id,
                    name: header.name.clone(),
                    target: header.target.clone(),
                    binary: header.binary.clone(),
                    kind: header.kind,
                    ..Suite::default()
                });
                let index = self.report.suites.len() - 1;
                self.by_binary.insert(header.binary.clone(), index);
                index
            }
        };
        self.current = Some(index);
        self.recent.clear();
        self.activity = format!("running {}", self.report.suites[index].id);
        true
    }

    /// A suite ended (next header or end of output) — without `test result:` it crashed.
    fn close_suite(&mut self, index: usize) {
        let suite = &mut self.report.suites[index];
        if suite.state != SuiteState::Running {
            return;
        }
        suite.state = SuiteState::Crashed;
        let tail: Vec<&str> = self.recent.iter().map(String::as_str).collect();
        suite.crash_log = Some(tail.join("\n"));
        let id = suite.id.clone();
        for test in &mut self.report.tests {
            if test.suite == id && test.status == TestStatus::Pending {
                test.status = TestStatus::NotRun;
            }
        }
    }

    fn suite_result(&mut self, rest: &str) {
        let Some(index) = self.current else {
            return;
        };
        let suite = &mut self.report.suites[index];
        suite.state = SuiteState::Done;
        suite.duration_s = rest
            .rsplit_once("finished in ")
            .and_then(|(_, t)| t.trim().trim_end_matches('s').parse::<f64>().ok());
        let id = suite.id.clone();
        // Listed but never reported (e.g. filtered in the run but not in the list).
        for test in &mut self.report.tests {
            if test.suite == id && test.status == TestStatus::Pending {
                test.status = TestStatus::NotRun;
            }
        }
    }

    fn outcome(&mut self, name: &str, outcome: &str, now_s: f64) {
        let (status, reason) = if outcome == "ok" {
            (TestStatus::Passed, None)
        } else if outcome.starts_with("FAILED") {
            (TestStatus::Failed, None)
        } else if let Some(rest) = outcome.strip_prefix("ignored") {
            let reason = rest.strip_prefix(", ").map(str::to_string);
            (TestStatus::Ignored, reason)
        } else {
            // `bench: …` lines and anything unknown.
            return;
        };
        let suite = self.current_or_unknown();
        if self.report.suites[suite].state == SuiteState::Pending {
            self.report.suites[suite].state = SuiteState::Running;
        }
        let index = self.test_index(suite, &normalize_name(name));
        let test = &mut self.report.tests[index];
        test.status = status;
        test.ignore_reason = reason;
        test.finished_s = Some((now_s * 1000.0).round() / 1000.0);
    }

    fn end_capture(&mut self) {
        if let Some((index, text)) = self.capture.take() {
            let text = text.trim_matches('\n').to_string();
            if text.is_empty() {
                return;
            }
            let test = &mut self.report.tests[index];
            test.output = Some(match test.output.take() {
                Some(previous) => format!("{previous}\n{text}"),
                None => text,
            });
        }
    }

    fn current_or_unknown(&mut self) -> usize {
        if let Some(index) = self.current {
            return index;
        }
        let binary = "unknown".to_string();
        let index = match self.by_binary.get(&binary) {
            Some(&index) => index,
            None => {
                self.report.suites.push(Suite {
                    id: "unknown".into(),
                    name: "unknown".into(),
                    binary: binary.clone(),
                    ..Suite::default()
                });
                let index = self.report.suites.len() - 1;
                self.by_binary.insert(binary, index);
                index
            }
        };
        self.current = Some(index);
        index
    }

    fn test_index(&mut self, suite: usize, name: &str) -> usize {
        let suite_id = self.report.suites[suite].id.clone();
        let id = format!("{suite_id}::{name}");
        if let Some(&index) = self.tests.get(&id) {
            return index;
        }
        self.report.tests.push(TestCase {
            id: id.clone(),
            suite: suite_id,
            name: name.to_string(),
            ..TestCase::default()
        });
        let index = self.report.tests.len() - 1;
        self.tests.insert(id, index);
        index
    }

    /// Map a report-event `bin` (executable stem) to its suite id.
    pub fn suite_for_binary(&self, binary: &str) -> Option<&str> {
        self.by_binary
            .get(binary)
            .map(|&i| self.report.suites[i].id.as_str())
    }

    /// Index of a test by suite id and libtest name.
    pub fn find_test(&self, suite: &str, name: &str) -> Option<usize> {
        self.tests.get(&format!("{suite}::{name}")).copied()
    }

    fn note_activity(&mut self, line: &str) {
        let trimmed = line.trim();
        for prefix in ["Compiling ", "Checking ", "Building ", "Finished ", "Fresh "] {
            if trimmed.starts_with(prefix) {
                self.activity = trimmed.to_string();
                return;
            }
        }
        if trimmed.starts_with("error") {
            self.activity = trimmed.chars().take(200).collect();
        }
    }
}

#[derive(Debug, PartialEq)]
struct SuiteHeader {
    name: String,
    target: String,
    binary: String,
    kind: SuiteKind,
}

impl SuiteHeader {
    fn id(&self) -> String {
        match self.kind {
            SuiteKind::Doc => format!("doc-tests {}", self.name),
            _ => format!("{} ({})", self.name, self.target),
        }
    }
}

fn parse_suite_header(line: &str) -> Option<SuiteHeader> {
    let trimmed = line.trim();
    if let Some(krate) = trimmed.strip_prefix("Doc-tests ") {
        let krate = krate.trim();
        return Some(SuiteHeader {
            name: krate.to_string(),
            target: "doc".into(),
            binary: format!("doc:{krate}"),
            kind: SuiteKind::Doc,
        });
    }
    let rest = trimmed.strip_prefix("Running ")?;
    let (kind, rest) = match rest.strip_prefix("unittests ") {
        Some(rest) => (SuiteKind::Unit, rest),
        None => (SuiteKind::Integration, rest),
    };
    let open = rest.rfind(" (")?;
    let target = rest[..open].trim().to_string();
    let path = rest[open + 2..].strip_suffix(')')?;
    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let binary = file.strip_suffix(".exe").unwrap_or(file).to_string();
    Some(SuiteHeader {
        name: strip_hash(&binary).to_string(),
        target,
        binary,
        kind,
    })
}

/// `mock-1a2b3c4d5e6f7a8b` → `mock`.
fn strip_hash(binary: &str) -> &str {
    match binary.rsplit_once('-') {
        Some((name, hash))
            if hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            name
        }
        _ => binary,
    }
}

/// `running 3 tests` / `running 1 test`.
fn is_running_count(line: &str) -> bool {
    line.strip_prefix("running ")
        .and_then(|rest| {
            rest.strip_suffix(" tests")
                .or_else(|| rest.strip_suffix(" test"))
        })
        .is_some_and(|n| n.parse::<usize>().is_ok())
}

/// `---- name stdout ----` → `name`.
fn header_name(line: &str) -> Option<&str> {
    line.strip_prefix("---- ")?.strip_suffix(" stdout ----")
}

/// `test name ... outcome` → `(name, outcome)`.
fn result_line(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("test ")?;
    let (name, outcome) = rest.rsplit_once(" ... ")?;
    let outcome = outcome.trim();
    if outcome.is_empty() {
        return None;
    }
    Some((name, outcome))
}

/// Drop libtest decorations that differ between `--list` and the run.
fn normalize_name(name: &str) -> String {
    let mut name = name.trim();
    for suffix in [" - should panic", " - compile fail", " - compile"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            name = stripped;
        }
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_list(c: &mut Collector, text: &str) {
        for line in text.lines() {
            c.list_line(line);
        }
    }

    fn feed_run(c: &mut Collector, text: &str) {
        for (i, line) in text.lines().enumerate() {
            c.run_line(line, i as f64 / 10.0);
        }
    }

    fn status(c: &Collector, suite: &str, name: &str) -> TestStatus {
        let index = c.find_test(suite, name).expect("test exists");
        c.report.tests[index].status
    }

    const LIST: &str = "\
   Compiling demo v0.1.0 (/tmp/demo)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.52s
     Running unittests src/lib.rs (target/debug/deps/demo-0123456789abcdef)
tests::adds: test
tests::fails: test
tests::skipped: test
tests::panics: test

4 tests, 0 benchmarks
     Running tests/api.rs (target/debug/deps/api-fedcba9876543210)
crashes: test
never_runs: test

2 tests, 0 benchmarks
   Doc-tests demo
src/lib.rs - add (line 3): test

1 test, 0 benchmarks
";

    const RUN: &str = "\
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/demo-0123456789abcdef)

running 4 tests
test tests::skipped ... ignored, needs network
test tests::adds ... ok
test tests::panics - should panic ... ok
test tests::fails ... FAILED

failures:

---- tests::fails stdout ----

thread 'tests::fails' panicked at src/lib.rs:12:9:
assertion `left == right` failed
  left: 4
 right: 5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::fails

test result: FAILED. 2 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: test failed, to rerun pass `--lib`
     Running tests/api.rs (target/debug/deps/api-fedcba9876543210)

running 2 tests
fatal runtime error: stack overflow
error: test failed, to rerun pass `--test api`

Caused by:
  process didn't exit successfully: `/tmp/demo/target/debug/deps/api-fedcba9876543210` (signal: 6, SIGABRT: process abort signal)
   Doc-tests demo

running 1 test
test src/lib.rs - add (line 3) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
";

    #[test]
    fn list_then_run_attributes_every_outcome() {
        let mut c = Collector::new(Report::default());
        feed_list(&mut c, LIST);
        c.report.recount();
        assert_eq!(c.report.totals.total, 7);
        assert_eq!(c.report.suites.len(), 3);
        assert_eq!(c.report.suites[0].id, "demo (src/lib.rs)");
        assert_eq!(c.report.suites[1].id, "api (tests/api.rs)");
        assert_eq!(c.report.suites[2].id, "doc-tests demo");

        feed_run(&mut c, RUN);
        c.finish();
        let lib = "demo (src/lib.rs)";
        assert_eq!(status(&c, lib, "tests::adds"), TestStatus::Passed);
        assert_eq!(status(&c, lib, "tests::fails"), TestStatus::Failed);
        assert_eq!(status(&c, lib, "tests::panics"), TestStatus::Passed);
        assert_eq!(status(&c, lib, "tests::skipped"), TestStatus::Ignored);
        let skipped = &c.report.tests[c.find_test(lib, "tests::skipped").unwrap()];
        assert_eq!(skipped.ignore_reason.as_deref(), Some("needs network"));

        let failed = &c.report.tests[c.find_test(lib, "tests::fails").unwrap()];
        let output = failed.output.as_deref().unwrap();
        assert!(output.starts_with("thread 'tests::fails' panicked"), "{output}");
        assert!(output.contains("right: 5"), "{output}");
        assert!(!output.contains("failures:"), "{output}");

        assert_eq!(c.report.suites[0].state, SuiteState::Done);
        assert_eq!(c.report.suites[0].duration_s, Some(0.03));

        let api = "api (tests/api.rs)";
        assert_eq!(c.report.suites[1].state, SuiteState::Crashed);
        let crash = c.report.suites[1].crash_log.as_deref().unwrap();
        assert!(crash.contains("stack overflow"), "{crash}");
        assert_eq!(status(&c, api, "crashes"), TestStatus::NotRun);
        assert_eq!(status(&c, api, "never_runs"), TestStatus::NotRun);

        assert_eq!(
            status(&c, "doc-tests demo", "src/lib.rs - add (line 3)"),
            TestStatus::Passed
        );
        assert_eq!(
            c.report.totals,
            crate::testrun::model::Totals {
                total: 7,
                completed: 7,
                passed: 3,
                failed: 1,
                ignored: 1,
                not_run: 2,
            }
        );
        assert_eq!(c.suite_for_binary("api-fedcba9876543210"), Some(api));
    }

    #[test]
    fn run_without_list_discovers_tests() {
        let mut c = Collector::new(Report::default());
        feed_run(&mut c, RUN);
        c.finish();
        assert_eq!(c.report.totals.passed, 3);
        assert_eq!(c.report.totals.failed, 1);
        assert_eq!(c.report.suites[1].state, SuiteState::Crashed);
    }

    #[test]
    fn dangling_outcome_on_next_line_is_attributed() {
        let mut c = Collector::new(Report::default());
        feed_run(
            &mut c,
            "     Running tests/io.rs (target/debug/deps/io-0123456789abcdef)\n\
             running 1 test\n\
             test prints ... \n\
             hello from the test\n\
             ok\n\
             test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
        );
        c.finish();
        assert_eq!(status(&c, "io (tests/io.rs)", "prints"), TestStatus::Passed);
    }

    #[test]
    fn windows_paths_and_duplicate_names_get_distinct_suites() {
        let a = parse_suite_header(
            r"     Running tests\smoke.rs (target\debug\deps\smoke-0123456789abcdef.exe)",
        )
        .unwrap();
        assert_eq!(a.binary, "smoke-0123456789abcdef");
        assert_eq!(a.name, "smoke");
        assert_eq!(a.target, r"tests\smoke.rs");

        let mut c = Collector::new(Report::default());
        c.list_line("     Running tests/smoke.rs (target/debug/deps/smoke-0123456789abcdef)");
        c.list_line("     Running tests/smoke.rs (target/debug/deps/smoke-fedcba9876543210)");
        assert_eq!(c.report.suites[0].id, "smoke (tests/smoke.rs)");
        assert_eq!(c.report.suites[1].id, "smoke (tests/smoke.rs) #2");
    }

    #[test]
    fn hashless_binaries_keep_their_name() {
        assert_eq!(strip_hash("airbug_hub"), "airbug_hub");
        assert_eq!(strip_hash("my-tool-0123456789abcdef"), "my-tool");
        assert_eq!(strip_hash("my-tool-short"), "my-tool-short");
    }
}
