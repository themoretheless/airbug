//! Ready-made modules (C# Testcontainers builders style).

mod mongodb;
mod mysql;
mod postgres;
mod rabbitmq;
mod redis;

pub use mongodb::{MongoDbBuilder, MongoDbContainer};
pub use mysql::{MySqlBuilder, MySqlContainer};
pub use postgres::{PostgreSqlBuilder, PostgreSqlContainer};
pub use rabbitmq::{RabbitMqBuilder, RabbitMqContainer};
pub use redis::{RedisBuilder, RedisContainer};
