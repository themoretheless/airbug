//! Composition root for the localhost hub.
use crate::{
    config::RootPaths,
    issues::{IssueStore, SqliteIssueStore},
};
use std::sync::Arc;

/// Injected dependencies for request handlers.
pub struct HubApp {
    pub paths: RootPaths,
    pub issues: Arc<dyn IssueStore>,
    pub webhook: Option<String>,
    pub port: u16,
}

impl HubApp {
    pub fn new(
        root: impl Into<std::path::PathBuf>,
        port: u16,
        webhook: Option<String>,
    ) -> crate::error::Result<Self> {
        let paths = RootPaths::new(root);
        let issues = Arc::new(SqliteIssueStore::open(paths.issues_db())?) as Arc<dyn IssueStore>;
        if let Some(ref url) = webhook {
            crate::issues::validate_webhook_url(url)?;
        }
        Ok(Self {
            paths,
            issues,
            webhook,
            port,
        })
    }
}
