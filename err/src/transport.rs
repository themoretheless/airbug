//! Blocking HTTP transport to airbug-hub `/api/v1/errors`.
use crate::event::Event;
use std::{
    sync::{Arc, Mutex},
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

/// Sends events synchronously over HTTP (suitable for `Drop` flush).
#[derive(Debug, Clone)]
pub struct HttpTransport {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl HttpTransport {
    pub fn new(endpoint: impl Into<String>) -> Result<Self, TransportError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| TransportError::Http(e.to_string()))?;
        Ok(Self {
            endpoint: endpoint.into(),
            client,
        })
    }
}

impl EventTransport for HttpTransport {
    fn send(&self, event: &Event) -> Result<(), TransportError> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(event)
            .send()
            .map_err(|e| TransportError::Http(e.to_string()))?;
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
