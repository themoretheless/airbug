//! Blocking HTTP transport to airbug-hub `/api/errors`.
use crate::event::Event;
use std::{fmt, time::Duration};

/// Failure while delivering an event.
#[derive(Debug)]
pub enum TransportError {
    Http(String),
    Rejected { status: u16, body: String },
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP request failed: {msg}"),
            Self::Rejected { status, body } => {
                write!(f, "hub rejected event: HTTP {status} ({body})")
            }
        }
    }
}

impl std::error::Error for TransportError {}

/// Sends events synchronously (suitable for `Drop` flush).
#[derive(Debug, Clone)]
pub struct Transport {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl Transport {
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

    pub fn send(&self, event: &Event) -> Result<(), TransportError> {
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
