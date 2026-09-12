//! Ready-made modules (C# `PostgreSqlBuilder` / `RedisBuilder` style).

mod postgres;
mod redis;

pub use postgres::{PostgreSqlBuilder, PostgreSqlContainer};
pub use redis::{RedisBuilder, RedisContainer};
