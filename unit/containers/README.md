# airbug-containers

Testcontainers-style helpers for ordinary Rust `#[test]`s. API follows
[Testcontainers for .NET](https://dotnet.testcontainers.org/) naming:
immutable builders, `Build` / `Start`, mapped ports, wait strategies, cleanup on `Drop`.

Uses the `docker` or `podman` CLI (no Docker SDK dependency).

## Example

```rust
use airbug_containers::prelude::*;

#[test]
fn with_redis() -> Result<(), ContainerError> {
    let redis = RedisBuilder::new().start()?;
    let url = redis.get_connection_string()?;
    Ok(())
}

#[test]
fn postgres() -> Result<(), ContainerError> {
    let pg = PostgreSqlBuilder::new()
        .with_database("app")
        .with_username("app")
        .with_password("secret")
        .start()?;
    let _uri = pg.get_connection_string()?;
    Ok(())
}

#[test]
fn mysql_mongo_rabbit() -> Result<(), ContainerError> {
    let _mysql = MySqlBuilder::new().start()?;
    let _mongo = MongoDbBuilder::new().start()?;
    let rabbit = RabbitMqBuilder::new().start()?;
    let _amqp = rabbit.get_connection_string()?;
    Ok(())
}
```

## Modules

| Builder | Default image | Helper |
|---------|---------------|--------|
| `ContainerBuilder` | any | generic |
| `RedisBuilder` | `redis:7.2.4` | `get_connection_string()` |
| `PostgreSqlBuilder` | `postgres:16-alpine` | `get_connection_string()` |
| `MySqlBuilder` | `mysql:8.4` | `get_connection_string()` |
| `MongoDbBuilder` | `mongo:7` | `get_connection_string()` |
| `RabbitMqBuilder` | `rabbitmq:3.13-management-alpine` | AMQP URI + management port |

## Notes

- Host override: `AIRBUG_CONTAINERS_HOST` (default `127.0.0.1`).
- Containers are labeled `airbug.containers=true` and removed on drop by default.
