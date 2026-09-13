//! Paths and limits for the local hub (DIP: leaves take RootPaths, not string literals).
use std::path::PathBuf;

/// Default hub listen port.
pub const DEFAULT_PORT: u16 = 8790;

pub const MAX_HEADER_BYTES: usize = 64 * 1024;
pub const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
pub const LOGS_TAIL_BYTES: u64 = 512 * 1024;
pub const METRICS_TAIL_BYTES: u64 = 2 * 1024 * 1024;
pub const DEFAULT_LOGS_LIMIT: usize = 120;
pub const DEFAULT_METRICS_LIMIT: usize = 400;

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

    pub fn otelcol_pid(&self) -> PathBuf {
        self.collector_data_dir().join("otelcol.pid")
    }

    pub fn runtime_config(&self) -> PathBuf {
        self.collector_data_dir().join("runtime-config.yaml")
    }

    pub fn issues_db(&self) -> PathBuf {
        self.root.join("dash/hub/data/issues.sqlite")
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
        // Best-effort containment without requiring the file to exist yet.
        path.canonicalize()
            .ok()
            .filter(|p| p.starts_with(report.canonicalize().unwrap_or_else(|_| report.clone())))
            .or_else(|| {
                // File may not exist; still reject obvious escapes via components.
                if path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    None
                } else {
                    Some(path)
                }
            })
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
}
