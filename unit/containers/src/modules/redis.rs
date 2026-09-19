//! Redis module (`RedisBuilder`).
use crate::docker::{Container, ContainerBuilder, ContainerError};
use crate::wait::Wait;
use std::time::Duration;

/// Builder mirroring a typical Redis Testcontainers module.
#[derive(Clone, Debug)]
pub struct RedisBuilder {
    image: String,
    startup_timeout: Duration,
}

impl Default for RedisBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RedisBuilder {
    /// Defaults: `redis:7.2.4`.
    pub fn new() -> Self {
        Self {
            image: "redis:7.2.4".into(),
            startup_timeout: Duration::from_secs(60),
        }
    }

    /// `WithImage`.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    /// Startup wait timeout.
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Build and start Redis.
    pub fn start(self) -> Result<RedisContainer, ContainerError> {
        // Redis startup logs are not fully stable across Docker/Podman images and
        // runtimes, so prefer a TCP readiness check over matching a specific log line.
        let inner = ContainerBuilder::new(self.image)
            .with_port_binding(6379, true)
            .with_wait_strategy(Wait::tcp_port(6379))
            .with_startup_timeout(self.startup_timeout)
            .build()?
            .start()?;
        Ok(RedisContainer { inner })
    }
}

/// Started Redis container.
#[derive(Debug)]
pub struct RedisContainer {
    inner: Container,
}

impl RedisContainer {
    /// `redis://host:port` connection URI.
    pub fn get_connection_string(&self) -> Result<String, ContainerError> {
        let port = self.inner.get_mapped_public_port(6379)?;
        Ok(format!("redis://{}:{port}", self.inner.hostname()))
    }

    /// Underlying generic container.
    pub fn container(&self) -> &Container {
        &self.inner
    }
}
