//! Blocking HTTP transport to airbug-hub `/api/v1/errors`.
use crate::event::Event;
use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

/// Failure while delivering an event.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("hub rejected event: HTTP {status} ({body})")]
    Rejected { status: u16, body: String },
}

/// Pluggable delivery for [`Event`] (DIP / test doubles).
pub trait EventTransport: Send + Sync {
    fn send(&self, event: &Event) -> Result<(), TransportError>;
}

enum WorkerRequest {
    Send {
        url: String,
        event: Event,
        reply: mpsc::SyncSender<Result<(), TransportError>>,
    },
}

struct WorkerHandle {
    tx: mpsc::Sender<WorkerRequest>,
}

impl std::fmt::Debug for WorkerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerHandle").finish()
    }
}

/// Sends events synchronously over HTTP (suitable for `Drop` flush).
///
/// Uses a dedicated OS worker thread for `reqwest::blocking::Client` so that
/// constructing, sending, and dropping the transport never panics when called
/// from inside an active Tokio runtime or during panic unwinding on a Tokio
/// worker thread.
#[derive(Debug, Clone)]
pub struct HttpTransport {
    pub endpoint: String,
    pub hub_id: Option<String>,
    worker: Arc<Mutex<WorkerHandle>>,
}

impl HttpTransport {
    pub fn new(endpoint: impl Into<String>) -> Result<Self, TransportError> {
        Self::with_hub_id(endpoint, None)
    }

    pub fn with_hub_id(
        endpoint: impl Into<String>,
        hub_id: Option<String>,
    ) -> Result<Self, TransportError> {
        let endpoint = endpoint.into();
        let hub_id = hub_id.filter(|s| !s.trim().is_empty());
        let (init_tx, init_rx) = mpsc::sync_channel::<Result<(), TransportError>>(1);
        let (req_tx, req_rx) = mpsc::channel::<WorkerRequest>();

        std::thread::Builder::new()
            .name("airbug-err-http".into())
            .spawn(move || {
                let client = match reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build()
                {
                    Ok(c) => {
                        let _ = init_tx.send(Ok(()));
                        c
                    }
                    Err(e) => {
                        let _ = init_tx.send(Err(TransportError::Http(e.to_string())));
                        return;
                    }
                };

                while let Ok(WorkerRequest::Send { url, event, reply }) = req_rx.recv() {
                    let res = match client.post(&url).json(&event).send() {
                        Ok(response) => {
                            let status = response.status();
                            if status.is_success() {
                                Ok(())
                            } else {
                                let body = response.text().unwrap_or_default();
                                Err(TransportError::Rejected {
                                    status: status.as_u16(),
                                    body,
                                })
                            }
                        }
                        Err(e) => Err(TransportError::Http(e.to_string())),
                    };
                    let _ = reply.send(res);
                }
            })
            .map_err(|e| TransportError::Http(e.to_string()))?;

        init_rx
            .recv()
            .map_err(|e| TransportError::Http(e.to_string()))??;

        Ok(Self {
            endpoint,
            hub_id,
            worker: Arc::new(Mutex::new(WorkerHandle { tx: req_tx })),
        })
    }

    pub(crate) fn resolved_url(&self) -> String {
        let base = self.endpoint.trim_end_matches('/');
        if let Some(ref hub_id) = self.hub_id {
            let hub_id = hub_id.trim_matches('/');
            if !hub_id.is_empty() && !base.ends_with(&format!("/{hub_id}")) {
                return format!("{base}/{hub_id}");
            }
        }
        self.endpoint.clone()
    }
}

impl EventTransport for HttpTransport {
    fn send(&self, event: &Event) -> Result<(), TransportError> {
        let url = self.resolved_url();
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let tx = {
            let guard = self
                .worker
                .lock()
                .map_err(|_| TransportError::Http("transport worker lock poisoned".into()))?;
            guard.tx.clone()
        };
        tx.send(WorkerRequest::Send {
            url,
            event: event.clone(),
            reply: reply_tx,
        })
        .map_err(|e| TransportError::Http(e.to_string()))?;
        reply_rx
            .recv()
            .map_err(|e| TransportError::Http(e.to_string()))?
    }
}

/// Deprecated alias for [`HttpTransport`].
#[deprecated(note = "renamed to HttpTransport")]
#[allow(dead_code)]
pub type Transport = HttpTransport;

/// In-memory [`EventTransport`] for tests and local inspection.
#[derive(Clone, Default)]
pub struct MemoryTransport {
    events: Arc<Mutex<Vec<Event>>>,
}

impl MemoryTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<Event> {
        self.events.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn last(&self) -> Option<Event> {
        self.events.lock().ok()?.last().cloned()
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.events.lock() {
            g.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.events.lock().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl EventTransport for MemoryTransport {
    fn send(&self, event: &Event) -> Result<(), TransportError> {
        if let Ok(mut g) = self.events.lock() {
            g.push(event.clone());
        }
        Ok(())
    }
}
