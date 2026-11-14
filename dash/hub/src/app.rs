//! Composition root for the localhost hub.
use crate::{
    config::RootPaths,
    error::{HubError, Result},
    event_model::EventEnvelope,
    issues::{IngestResponse, IssueStore, SqliteIssueStore},
    otlp::receiver::OtlpStore,
    resources::ResourceManager,
    runs::{RunStore, load_or_create_hub_id},
    stream::StreamHub,
    symbols::SymbolStore,
};
use std::sync::{
    Arc, Mutex, OnceLock, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

fn relay_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new())
    })
}

/// Injected dependencies for request handlers.
pub struct HubApp {
    pub paths: RootPaths,
    pub hub_id: String,
    pub issues: Arc<dyn IssueStore>,
    pub runs: Arc<RunStore>,
    pub resources: Arc<ResourceManager>,
    pub symbols: Arc<SymbolStore>,
    pub stream: Arc<StreamHub>,
    pub webhook: Option<String>,
    pub port: u16,
    pub otlp_store: OtlpStore,
    scrub_pii: AtomicBool,
    upstream: RwLock<Option<String>>,
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
        let symbols = Arc::new(SymbolStore::open(paths.symbols_dir())?);
        let stream = Arc::new(StreamHub::new());
        if let Some(ref url) = webhook {
            crate::issues::validate_webhook_url(url)?;
        }
        Ok(Self {
            paths,
            hub_id,
            issues,
            runs,
            resources,
            symbols,
            stream,
            webhook,
            port,
            otlp_store: OtlpStore::new(),
            scrub_pii: AtomicBool::new(false),
            upstream: RwLock::new(None),
            events_lock: Mutex::new(()),
        })
    }

    pub fn set_scrub_pii(&self, enabled: bool) {
        self.scrub_pii.store(enabled, Ordering::Relaxed);
    }

    pub fn scrub_pii(&self) -> bool {
        self.scrub_pii.load(Ordering::Relaxed)
    }

    pub fn set_upstream(&self, url: Option<String>) -> Result<()> {
        if let Some(ref u) = url {
            let trimmed = u.trim();
            if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
                return Err(HubError::msg("upstream relay must be http:// or https://"));
            }
            let after_scheme = trimmed.split_once("://").map(|x| x.1).unwrap_or("");
            let authority = after_scheme
                .split(['/', '?', '#'])
                .next()
                .unwrap_or(after_scheme);
            if authority.contains('@') {
                return Err(HubError::msg(
                    "upstream relay URL must not contain userinfo",
                ));
            }
        }
        let mut guard = self.upstream.write().unwrap_or_else(|e| e.into_inner());
        *guard = url.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        Ok(())
    }

    pub fn upstream(&self) -> Option<String> {
        self.upstream
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Apply server-side symbolication and optional hub-wide PII scrubbing in place.
    pub fn prepare_error_event(&self, event: &mut airbug_err::Event) -> usize {
        let resolved = self.symbols.symbolicate_event(event);
        if resolved > 0
            && let Some(ref ex) = event.exception
            && let Some(ref st) = ex.stacktrace
            && let Some(top) = st
                .frames
                .iter()
                .find(|f| f.in_app)
                .or_else(|| st.frames.first())
            && (event.fingerprint.is_empty()
                || (event.fingerprint.len() == 1
                    && (event.fingerprint[0] == "{{ default }}"
                        || event.fingerprint[0].contains("<unsymbolicated>")
                        || event.fingerprint[0].contains("<unknown>"))))
        {
            event.fingerprint = vec![format!("{}:{}", ex.ty, top.function)];
        }
        if self.scrub_pii() {
            airbug_err::Scrubber::default_pii().scrub_event(event);
        }
        resolved
    }

    /// Normalize, optionally scrub, persist, broadcast, and optionally relay an error event.
    pub fn ingest_error_event(&self, mut event: airbug_err::Event) -> Result<IngestResponse> {
        self.prepare_error_event(&mut event);
        let envelope = EventEnvelope::from_error(event.clone());
        let resp = self.issues.ingest(event.clone(), self.webhook.as_deref())?;
        let _ = self.append_event_envelope(&envelope);
        self.stream.publish(
            "error",
            serde_json::json!({
                "event_id": event.event_id,
                "issue_id": resp.issue_id,
                "level": event.level,
                "title": event.title(),
                "is_new": resp.is_new,
                "is_regression": resp.is_regression,
            }),
        );
        self.relay_event(&event);
        Ok(resp)
    }

    /// Forward an error event to the configured upstream Relay URL without blocking ingest.
    pub fn relay_event(&self, event: &airbug_err::Event) {
        let Some(url) = self.upstream() else {
            return;
        };
        let payload = event.clone();
        let _ = std::thread::Builder::new()
            .name("airbug-hub-relay".into())
            .spawn(move || {
                let _ = relay_client().post(&url).json(&payload).send();
            });
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
