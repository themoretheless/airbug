//! On-disk contract of a test run: `.airbug/runs/<id>/`.
//!
//! ```text
//! manifest.json   who/what/where — written once at start (kind, command, git, host)
//! progress.json   small live snapshot for pollers (state, totals, activity)
//! report.json     full result: suites, tests, output, steps, comparisons, attachments
//! output.log      raw merged cargo output
//! events/         AIRBUG_REPORT_DIR of the test processes (events.jsonl + attachments)
//! index.html      self-contained report (same renderer as the hub)
//! ```
//!
//! `report.json` keeps a top-level `tests[].status` so older readers of
//! `target/airbug-report/report.json` (`UnitReportV1`) still count it.
use serde::{Deserialize, Serialize};

pub const MANIFEST_SCHEMA: &str = "airbug.run/1";
pub const REPORT_SCHEMA: &str = "airbug.test-report/1";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: String,
    pub kind: String,
    pub run_id: String,
    pub title: String,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Git {
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub short: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    #[default]
    Building,
    Running,
    Passed,
    Failed,
    Error,
    Interrupted,
}

impl RunState {
    pub fn is_finished(self) -> bool {
        !matches!(self, Self::Building | Self::Running)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    #[default]
    Pending,
    Passed,
    Failed,
    Ignored,
    NotRun,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuiteKind {
    #[default]
    Unit,
    Integration,
    Doc,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuiteState {
    #[default]
    Pending,
    Running,
    Done,
    Crashed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Totals {
    pub total: usize,
    pub completed: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub not_run: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Suite {
    /// Stable across runs: `<binary name> (<target>)` or `doc-tests <crate>`.
    pub id: String,
    /// Binary name without cargo's metadata hash (`mock`, `airbug_hub`).
    pub name: String,
    /// Source target cargo printed (`tests/mock.rs`, `src/lib.rs`).
    pub target: String,
    /// Executable file stem including the hash; matches `bin` in report events.
    pub binary: String,
    pub kind: SuiteKind,
    pub state: SuiteState,
    pub totals: Totals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    /// Last lines before the process died without a `test result:` line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crash_log: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestCase {
    /// `<suite id>::<test name>` — the key for history across runs.
    pub id: String,
    pub suite: String,
    pub name: String,
    pub status: TestStatus,
    /// Seconds since the run started when libtest reported the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_reason: Option<String>,
    /// Captured stdout/panic message (failures, or every test with `--show-output`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparisons: Vec<Comparison>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flaky: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    /// `passed`, `failed`, or `unfinished` when the process died inside the step.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparisons: Vec<Comparison>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flaky: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comparison {
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    pub media_type: String,
    /// File name inside `events/` (never a path).
    pub file: String,
    pub size: u64,
}

/// Report events that could not be tied to a test (helper threads, doctests).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Unattributed {
    pub suite: Option<String>,
    pub thread: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparisons: Vec<Comparison>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Report {
    pub schema: String,
    pub run_id: String,
    pub title: String,
    pub state: RunState,
    #[serde(default)]
    pub git: Git,
    /// Kept for `UnitReportV1` readers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default)]
    pub started_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub totals: Totals,
    #[serde(default)]
    pub suites: Vec<Suite>,
    #[serde(default)]
    pub tests: Vec<TestCase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unattributed: Vec<Unattributed>,
    /// Tail of the output when the build (or listing) failed before tests ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_log: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Progress {
    pub run_id: String,
    pub kind: String,
    pub state: RunState,
    pub totals: Totals,
    /// Last interesting line: `Compiling serde v1…`, current suite, …
    #[serde(default)]
    pub activity: String,
    #[serde(default)]
    pub updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    /// Runner process id, so a reader can tell a crashed runner from a slow one.
    #[serde(default)]
    pub pid: u32,
}

impl Report {
    /// Recompute run and per-suite totals from test statuses.
    pub fn recount(&mut self) {
        let mut run = Totals::default();
        for suite in &mut self.suites {
            suite.totals = Totals::default();
        }
        for test in &self.tests {
            let suite = self.suites.iter_mut().find(|s| s.id == test.suite);
            for totals in [Some(&mut run), suite.map(|s| &mut s.totals)]
                .into_iter()
                .flatten()
            {
                totals.total += 1;
                match test.status {
                    TestStatus::Pending => continue,
                    TestStatus::Passed => totals.passed += 1,
                    TestStatus::Failed => totals.failed += 1,
                    TestStatus::Ignored => totals.ignored += 1,
                    TestStatus::NotRun => totals.not_run += 1,
                }
                totals.completed += 1;
            }
        }
        self.totals = run;
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
