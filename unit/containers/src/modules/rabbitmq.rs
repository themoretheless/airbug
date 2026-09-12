//! RabbitMQ module (`RabbitMqBuilder`).
use crate::docker::{Container, ContainerBuilder, ContainerError};
use crate::wait::Wait;
use std::time::Duration;

/// Builder mirroring Testcontainers.RabbitMq.RabbitMqBuilder.
#[derive(Clone, Debug)]
pub struct RabbitMqBuilder {
    image: String,
    username: String,
    password: String,
    vhost: String,
    startup_timeout: Duration,
}

impl Default for RabbitMqBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RabbitMqBuilder {
    /// Defaults: `rabbitmq:3.13-management-alpine`, user/pass `guest`.
    pub fn new() -> Self {
        Self {
            image: "rabbitmq:3.13-management-alpine".into(),
            username: "guest".into(),
            password: "guest".into(),
            vhost: "/".into(),
            startup_timeout: Duration::from_secs(90),
        }
    }

    /// `WithImage`.
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
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

    /// Virtual host path (default `/`).
    pub fn with_vhost(mut self, vhost: impl Into<String>) -> Self {
        self.vhost = vhost.into();
        self
    }

    /// Startup wait timeout.
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Build and start RabbitMQ.
    pub fn start(self) -> Result<RabbitMqContainer, ContainerError> {
        let username = self.username.clone();
        let password = self.password.clone();
        let vhost = self.vhost.clone();
        let mut builder = ContainerBuilder::new(self.image)
            .with_environment("RABBITMQ_DEFAULT_USER", &username)
            .with_environment("RABBITMQ_DEFAULT_PASS", &password)
            .with_port_binding(5672, true)
            .with_port_binding(15672, true)
            .with_wait_strategy(Wait::tcp_port(5672))
            .with_startup_timeout(self.startup_timeout);
        if vhost != "/" {
            builder = builder.with_environment("RABBITMQ_DEFAULT_VHOST", &vhost);
        }
        let inner = builder.build()?.start()?;
        Ok(RabbitMqContainer {
            inner,
            username,
            password,
            vhost,
        })
    }
}

/// Started RabbitMQ container.
#[derive(Debug)]
pub struct RabbitMqContainer {
    inner: Container,
    username: String,
    password: String,
    vhost: String,
}

impl RabbitMqContainer {
    /// AMQP URI (`amqp://user:pass@host:port/vhost`).
    pub fn get_connection_string(&self) -> Result<String, ContainerError> {
        let port = self.inner.get_mapped_public_port(5672)?;
        let host = self.inner.hostname();
        let vhost = if self.vhost == "/" {
            "%2F".to_string()
        } else {
            self.vhost.trim_start_matches('/').to_string()
        };
        Ok(format!(
            "amqp://{}:{}@{}:{}/{vhost}",
            self.username, self.password, host, port
        ))
    }

    /// Management UI mapped port (image includes management plugin).
    pub fn get_management_port(&self) -> Result<u16, ContainerError> {
        self.inner.get_mapped_public_port(15672)
    }

    /// Underlying generic container.
    pub fn container(&self) -> &Container {
        &self.inner
    }
}
