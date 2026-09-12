//! MongoDB module (`MongoDbBuilder`).
use crate::docker::{Container, ContainerBuilder, ContainerError};
use crate::wait::Wait;
use std::time::Duration;

/// Builder mirroring Testcontainers.MongoDb.MongoDbBuilder.
#[derive(Clone, Debug)]
pub struct MongoDbBuilder {
    image: String,
    username: Option<String>,
    password: Option<String>,
    startup_timeout: Duration,
}

impl Default for MongoDbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MongoDbBuilder {
    /// Defaults: `mongo:7`, no auth (typical for local tests).
    pub fn new() -> Self {
        Self {
            image: "mongo:7".into(),
            username: None,
            password: None,
            startup_timeout: Duration::from_secs(60),
        }
    }

    /// `WithImage`.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    /// Enable root auth (`MONGO_INITDB_ROOT_USERNAME` / `PASSWORD`).
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Root password (used with [`Self::with_username`]).
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Startup wait timeout.
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Build and start MongoDB.
    pub fn start(self) -> Result<MongoDbContainer, ContainerError> {
        let username = self.username.clone();
        let password = self.password.clone();
        let mut builder = ContainerBuilder::new(self.image)
            .with_port_binding(27017, true)
            .with_wait_strategy(Wait::tcp_port(27017))
            .with_startup_timeout(self.startup_timeout);
        if let (Some(user), Some(pass)) = (&username, &password) {
            builder = builder
                .with_environment("MONGO_INITDB_ROOT_USERNAME", user)
                .with_environment("MONGO_INITDB_ROOT_PASSWORD", pass);
        }
        let inner = builder.build()?.start()?;
        Ok(MongoDbContainer {
            inner,
            username,
            password,
        })
    }
}

/// Started MongoDB container.
#[derive(Debug)]
pub struct MongoDbContainer {
    inner: Container,
    username: Option<String>,
    password: Option<String>,
}

impl MongoDbContainer {
    /// Connection URI (`mongodb://host:port` or with credentials).
    pub fn get_connection_string(&self) -> Result<String, ContainerError> {
        let port = self.inner.get_mapped_public_port(27017)?;
        let host = self.inner.hostname();
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => Ok(format!("mongodb://{user}:{pass}@{host}:{port}")),
            _ => Ok(format!("mongodb://{host}:{port}")),
        }
    }

    /// Underlying generic container.
    pub fn container(&self) -> &Container {
        &self.inner
    }
}
