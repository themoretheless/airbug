//! Composition root for the localhost hub.
use crate::{
    config::RootPaths,
    error::Result,
    event_model::EventEnvelope,
    issues::{IssueStore, SqliteIssueStore},
    otlp::receiver::OtlpStore,
    resources::ResourceManager,
    runs::{RunStore, load_or_create_hub_id},
};
use std::sync::{Arc, Mutex};

/// Injected dependencies for request handlers.
pub struct HubApp {
    pub paths: RootPaths,
    pub hub_id: String,
    pub issues: Arc<dyn IssueStore>,
    pub runs: Arc<RunStore>,
    pub resources: Arc<ResourceManager>,
    pub webhook: Option<String>,
    pub port: u16,
    pub otlp_store: OtlpStore,
    events_lock: Mutex<()>,
}

impl HubApp {
    pub fn new(
        root: impl Into<std::path::PathBuf>,
        port: u16,
        webhook: Option<String>,
    ) -> Result<Self> {
        let paths = RootPaths::new(root);
        let hub_id = load_or_create_hub_id(&paths.hub_id_file())?;
        let issues = Arc::new(SqliteIssueStore::open_with_root(
            paths.issues_db(),
            paths.root.clone(),
        )?) as Arc<dyn IssueStore>;
        let runs = Arc::new(RunStore::open(
            paths.root.clone(),
            &paths.runs_db(),
            hub_id.clone(),
            port,
        )?);
        let resources = ResourceManager::new(paths.clone(), hub_id.clone(), port);
        if let Some(ref url) = webhook {
            crate::issues::validate_webhook_url(url)?;
        }
        Ok(Self {
            paths,
            hub_id,
            issues,
            runs,
            resources,
            webhook,
            port,
            otlp_store: OtlpStore::new(),
            events_lock: Mutex::new(()),
        })
    }

    pub fn append_event_envelope(&self, envelope: &EventEnvelope) -> std::io::Result<()> {
        self.append_event_envelopes(std::slice::from_ref(envelope))
    }

    pub fn append_event_envelopes(&self, envelopes: &[EventEnvelope]) -> std::io::Result<()> {
        if envelopes.is_empty() {
            return Ok(());
        }
        let mut buf = String::new();
        for envelope in envelopes {
            let line = serde_json::to_string(envelope)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            buf.push_str(&line);
            buf.push('\n');
        }
        let _guard = self.events_lock.lock().unwrap_or_else(|e| e.into_inner());
        let events_path = self.paths.events_file();
        if let Some(parent) = events_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(events_path)?;
        file.write_all(buf.as_bytes())
    }
}
