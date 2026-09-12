//! MySQL module (`MySqlBuilder`).
use crate::docker::{Container, ContainerBuilder, ContainerError};
use crate::wait::Wait;
use std::time::Duration;

/// Builder mirroring Testcontainers.MySql.MySqlBuilder.
#[derive(Clone, Debug)]
pub struct MySqlBuilder {
    image: String,
    database: String,
    username: String,
    password: String,
    root_password: String,
    startup_timeout: Duration,
}

impl Default for MySqlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlBuilder {
    /// Defaults: `mysql:8.4`, db/user `test`, password `test`.
    pub fn new() -> Self {
        Self {
            image: "mysql:8.4".into(),
            database: "test".into(),
            username: "test".into(),
            password: "test".into(),
            root_password: "test".into(),
            startup_timeout: Duration::from_secs(90),
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

    /// Root account password (`MYSQL_ROOT_PASSWORD`).
    pub fn with_root_password(mut self, password: impl Into<String>) -> Self {
        self.root_password = password.into();
        self
    }

    /// Startup wait timeout.
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Build and start MySQL.
    pub fn start(self) -> Result<MySqlContainer, ContainerError> {
        let database = self.database.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let inner = ContainerBuilder::new(self.image)
            .with_environment("MYSQL_DATABASE", &database)
            .with_environment("MYSQL_USER", &username)
            .with_environment("MYSQL_PASSWORD", &password)
            .with_environment("MYSQL_ROOT_PASSWORD", &self.root_password)
            .with_port_binding(3306, true)
            .with_wait_strategy(Wait::tcp_port(3306))
            .with_startup_timeout(self.startup_timeout)
            .build()?
            .start()?;
        Ok(MySqlContainer {
            inner,
            database,
            username,
            password,
        })
    }
}

/// Started MySQL container.
#[derive(Debug)]
pub struct MySqlContainer {
    inner: Container,
    database: String,
    username: String,
    password: String,
}

impl MySqlContainer {
    /// MySQL URI (`mysql://user:pass@host:port/db`).
    pub fn get_connection_string(&self) -> Result<String, ContainerError> {
        let port = self.inner.get_mapped_public_port(3306)?;
        let host = self.inner.hostname();
        Ok(format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, host, port, self.database
        ))
    }

    /// Underlying generic container.
    pub fn container(&self) -> &Container {
        &self.inner
    }
}
