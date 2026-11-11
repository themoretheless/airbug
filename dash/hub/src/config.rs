//! Paths and limits for the local hub (DIP: leaves take RootPaths, not string literals).
use std::path::PathBuf;

/// Default hub listen port.
pub const DEFAULT_PORT: u16 = 8790;

pub const MAX_HEADER_BYTES: usize = 64 * 1024;
pub const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_OTLP_DECOMPRESSED_BYTES: usize = 16 * 1024 * 1024;
pub const OTLP_ROTATE_BYTES: u64 = 32 * 1024 * 1024;
pub const OTLP_MAX_BACKUPS: usize = 3;
pub const LOGS_TAIL_BYTES: u64 = 512 * 1024;
pub const METRICS_TAIL_BYTES: u64 = 2 * 1024 * 1024;
pub const TRACES_TAIL_BYTES: u64 = 2 * 1024 * 1024;
pub const PROFILES_TAIL_BYTES: u64 = 4 * 1024 * 1024;
pub const DEFAULT_LOGS_LIMIT: usize = 120;
pub const DEFAULT_METRICS_LIMIT: usize = 400;
pub const DEFAULT_TRACES_LIMIT: usize = 120;
pub const DEFAULT_PROFILES_LIMIT: usize = 50;

/// Absolute (or canonicalized) monorepo root and derived artifact paths.
#[derive(Debug, Clone)]
pub struct RootPaths {
    pub root: PathBuf,
}

impl RootPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn collector_dir(&self) -> PathBuf {
        self.root.join("dash/collector")
    }

    pub fn collector_data_dir(&self) -> PathBuf {
        self.collector_dir().join("data")
    }

    pub fn compose_file(&self) -> PathBuf {
        self.collector_dir().join("docker-compose.yml")
    }

    pub fn collector_config(&self) -> PathBuf {
        self.collector_dir().join("config.yaml")
    }

    pub fn standalone_template(&self) -> PathBuf {
        self.collector_dir().join("config.standalone.yaml")
    }

    pub fn logs_file(&self) -> PathBuf {
        self.collector_data_dir().join("logs.json")
    }

    pub fn metrics_file(&self) -> PathBuf {
        self.collector_data_dir().join("metrics.json")
    }

    pub fn traces_file(&self) -> PathBuf {
        self.collector_data_dir().join("traces.json")
    }

    pub fn profiles_file(&self) -> PathBuf {
        self.collector_data_dir().join("profiles.json")
    }

    pub fn otelcol_pid(&self) -> PathBuf {
        self.collector_data_dir().join("otelcol.pid")
    }

    pub fn runtime_config(&self) -> PathBuf {
        self.collector_data_dir().join("runtime-config.yaml")
    }

    pub fn issues_db(&self) -> PathBuf {
        self.hub_data_dir().join("issues.sqlite")
    }

    pub fn hub_data_dir(&self) -> PathBuf {
        self.root.join("dash/hub/data")
    }

    pub fn hub_id_file(&self) -> PathBuf {
        self.hub_data_dir().join("hub_id")
    }

    pub fn runs_db(&self) -> PathBuf {
        self.hub_data_dir().join("runs.sqlite")
    }

    /// Artifact root for registered bench runs: `<root>/.airbug-bench/runs`.
    pub fn bench_runs_dir(&self) -> PathBuf {
        self.root.join(".airbug-bench/runs")
    }

    pub fn bench_run_dir(&self, run_id: &str) -> PathBuf {
        self.bench_runs_dir().join(run_id)
    }

    pub fn events_file(&self) -> PathBuf {
        self.root.join("dash/hub/data/events.jsonl")
    }

    pub fn unit_report_dir(&self) -> PathBuf {
        self.root.join("target/airbug-report")
    }

    /// HTML report index when present (used by Local APIs catalog).
    pub fn unit_report_index(&self) -> PathBuf {
        self.unit_report_dir().join("index.html")
    }

    /// Resolve a file under `target/airbug-report` with `..` segments rejected.
    pub fn unit_report_file(&self, name: &str) -> Option<PathBuf> {
        if name.is_empty() || name.contains("..") || name.starts_with('/') || name.starts_with('\\')
        {
            return None;
        }
        let path = self.unit_report_dir().join(name);
        let report = self.unit_report_dir();
        if let Ok(canon_path) = path.canonicalize() {
            let canon_report = report.canonicalize().unwrap_or_else(|_| report.clone());
            return canon_path.starts_with(&canon_report).then_some(canon_path);
        }
        if path
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return None;
        }
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            None
        } else {
            Some(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_report_rejects_parent_dir() {
        let paths = RootPaths::new("/tmp/airbug-root");
        assert!(paths.unit_report_file("../secret").is_none());
        assert!(paths.unit_report_file("index.html").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn unit_report_rejects_escaping_symlink() {
        let dir = std::env::temp_dir().join(format!("airbug-cfg-sym-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let report_dir = dir.join("target/airbug-report");
        std::fs::create_dir_all(&report_dir).unwrap();
        let outside = dir.join("outside.txt");
        std::fs::write(&outside, "secret").unwrap();
        std::os::unix::fs::symlink(&outside, report_dir.join("escape.html")).unwrap();
        let paths = RootPaths::new(&dir);
        assert!(paths.unit_report_file("escape.html").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
