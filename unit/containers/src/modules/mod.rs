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

pub(crate) fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

pub(crate) fn format_host(host: &str) -> String {
    if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_escapes_special_uri_characters() {
        assert_eq!(percent_encode("guest"), "guest");
        assert_eq!(percent_encode("p@ss:w/rd?#"), "p%40ss%3Aw%2Frd%3F%23");
        assert_eq!(percent_encode("my/vhost"), "my%2Fvhost");
    }

    #[test]
    fn format_host_brackets_bare_ipv6() {
        assert_eq!(format_host("127.0.0.1"), "127.0.0.1");
        assert_eq!(format_host("::1"), "[::1]");
        assert_eq!(format_host("[::1]"), "[::1]");
    }
}
