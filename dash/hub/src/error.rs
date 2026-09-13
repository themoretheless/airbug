//! Typed errors for the hub API / IO boundary.
use thiserror::Error;

pub type Result<T> = std::result::Result<T, HubError>;

#[derive(Debug, Error)]
pub enum HubError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

impl HubError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }

    pub fn json_body(&self) -> String {
        let msg = self.to_string();
        match serde_json::to_string(&serde_json::json!({ "error": msg })) {
            Ok(s) => s,
            Err(_) => format!("{{\"error\":{}}}", serde_json::to_string(&msg).unwrap_or_else(|_| "\"error\"".into())),
        }
    }

    pub fn json_body_ok_false(&self) -> String {
        let msg = self.to_string();
        match serde_json::to_string(&serde_json::json!({ "ok": false, "error": msg })) {
            Ok(s) => s,
            Err(_) => format!("{{\"ok\":false,\"error\":{}}}", serde_json::to_string(&msg).unwrap_or_else(|_| "\"error\"".into())),
        }
    }
}

impl From<String> for HubError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for HubError {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}
