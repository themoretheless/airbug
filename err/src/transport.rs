//! Blocking HTTP transport to airbug-hub `/api/v1/errors`.
use crate::event::Event;
use std::time::Duration;

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
