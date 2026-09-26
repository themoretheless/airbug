//! Composition root for the localhost hub.
use crate::{
    config::RootPaths,
    error::Result,
    issues::{IssueStore, SqliteIssueStore},
    runs::{RunStore, load_or_create_hub_id},
};
use std::sync::Arc;

/// Injected dependencies for request handlers.
pub struct HubApp {
    pub paths: RootPaths,
    pub hub_id: String,
    pub issues: Arc<dyn IssueStore>,
    pub runs: Arc<RunStore>,
    pub webhook: Option<String>,
    pub port: u16,
}

impl HubApp {
    pub fn new(
        root: impl Into<std::path::PathBuf>,
        port: u16,
        webhook: Option<String>,
    ) -> Result<Self> {
        let paths = RootPaths::new(root);
        let hub_id = load_or_create_hub_id(&paths.hub_id_file())?;
        let issues = Arc::new(SqliteIssueStore::open(paths.issues_db())?) as Arc<dyn IssueStore>;
        let runs = Arc::new(RunStore::open(
            paths.root.clone(),
            &paths.runs_db(),
            hub_id.clone(),
            port,
        )?);
        if let Some(ref url) = webhook {
            crate::issues::validate_webhook_url(url)?;
        }
        Ok(Self {
            paths,
            hub_id,
            issues,
            runs,
            webhook,
            port,
        })
    }
}
