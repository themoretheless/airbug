//! Testcontainers-style Docker helpers for ordinary `#[test]`s.
//!
//! API mirrors the familiar .NET shape: immutable builders, `Build` / `Start`,
//! mapped ports, wait strategies, and automatic cleanup on [`Drop`].
//!
//! Requires a working `docker` (or `podman`) CLI on `PATH`.
//!
//! ```ignore
//! use airbug_containers::prelude::*;
//!
//! let redis = ContainerBuilder::new("redis:7.2.4")
//!     .with_port_binding(6379, true)
//!     .with_wait_strategy(Wait::tcp_or_message(6379, "Ready to accept connections"))
//!     .build()
//!     .start()?;
//!
//! let port = redis.get_mapped_public_port(6379)?;
//! // Drop stops and removes the container.
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod docker;
mod wait;

pub mod modules;

pub use docker::{Container, ContainerBuilder, ContainerError, Runtime};
pub use wait::Wait;

/// Common imports for integration tests.
pub mod prelude {
    pub use crate::modules::{
        MongoDbBuilder, MySqlBuilder, PostgreSqlBuilder, RabbitMqBuilder, RedisBuilder,
    };
    pub use crate::{Container, ContainerBuilder, ContainerError, Wait};
}
