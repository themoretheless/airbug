//! Typed shapes for scanned unit report artifacts (contract with report tool).
use serde::Deserialize;

/// Expected shape of `target/airbug-report/report.json` (v1).
#[derive(Debug, Deserialize)]
pub struct UnitReportV1 {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub tests: Vec<UnitTestRowV1>,
}

#[derive(Debug, Deserialize)]
pub struct UnitTestRowV1 {
    #[serde(default)]
    pub status: Option<String>,
}

impl UnitReportV1 {
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut broken = 0;
        for test in &self.tests {
            match test.status.as_deref().unwrap_or("") {
                "passed" | "ok" => passed += 1,
                "failed" => failed += 1,
                "broken" => broken += 1,
                _ => {}
            }
        }
        (passed, failed, broken)
    }
}
