//! PostgreSQL module (`PostgreSqlBuilder` / started container).
use crate::docker::{Container, ContainerBuilder, ContainerError};
use crate::wait::Wait;
use std::time::Duration;

/// Builder mirroring Testcontainers.PostgreSql.PostgreSqlBuilder.
#[derive(Clone, Debug)]
pub struct PostgreSqlBuilder {
    image: String,
    database: String,
    username: String,
    password: String,
    startup_timeout: Duration,
}

impl Default for PostgreSqlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgreSqlBuilder {
    /// Defaults: `postgres:16-alpine`, db/user `test`, password `test`.
    pub fn new() -> Self {
        Self {
            image: "postgres:16-alpine".into(),
            database: "test".into(),
            username: "test".into(),
            password: "test".into(),
            startup_timeout: Duration::from_secs(60),
        }
    }

    /// `WithImage`.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    /// `WithDatabase`.
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = database.into();
        self
    }

    /// `WithUsername`.
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    /// `WithPassword`.
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = password.into();
        self
    }

    /// Startup wait timeout.
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// `Build` + `Start` in one step (common in tests).
    pub fn start(self) -> Result<PostgreSqlContainer, ContainerError> {
        let database = self.database.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let inner = ContainerBuilder::new(self.image)
            .with_environment("POSTGRES_DB", &database)
            .with_environment("POSTGRES_USER", &username)
            .with_environment("POSTGRES_PASSWORD", &password)
            .with_port_binding(5432, true)
            .with_wait_strategy(Wait::tcp_port(5432))
            .with_startup_timeout(self.startup_timeout)
            .build()?
            .start()?;
        Ok(PostgreSqlContainer {
            inner,
            database,
            username,
            password,
        })
    }
}

/// Started PostgreSQL container with connection helpers.
#[derive(Debug)]
pub struct PostgreSqlContainer {
    inner: Container,
    database: String,
    username: String,
    password: String,
}

impl PostgreSqlContainer {
    /// `GetConnectionString` (libpq URI).
    pub fn get_connection_string(&self) -> Result<String, ContainerError> {
        let port = self.inner.get_mapped_public_port(5432)?;
        let host = self.inner.hostname();
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, host, port, self.database
        ))
    }

    /// Underlying generic container.
    pub fn container(&self) -> &Container {
        &self.inner
    }
}
