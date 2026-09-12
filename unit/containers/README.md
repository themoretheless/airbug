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
    // talk to Redis at `url`
    Ok(())
}

#[test]
fn generic_container() -> Result<(), ContainerError> {
    let nginx = ContainerBuilder::new("nginx:1.27-alpine")
        .with_port_binding(80, true)
        .with_wait_strategy(Wait::tcp_port(80))
        .build()?
        .start()?;
    let port = nginx.get_mapped_public_port(80)?;
    assert!(port > 0);
    Ok(())
}

#[test]
fn postgres() -> Result<(), ContainerError> {
    let pg = PostgreSqlBuilder::new()
        .with_database("app")
        .with_username("app")
        .with_password("secret")
        .start()?;
    let uri = pg.get_connection_string()?;
    assert!(uri.contains("@"));
    Ok(())
}
```

## Modules

| Builder | Default image | Helper |
|---------|---------------|--------|
| `ContainerBuilder` | any | generic |
| `RedisBuilder` | `redis:7.2.4` | `get_connection_string()` |
| `PostgreSqlBuilder` | `postgres:16-alpine` | `get_connection_string()` |

## Notes

- Host override: `AIRBUG_CONTAINERS_HOST` (default `127.0.0.1`).
- Containers are labeled `airbug.containers=true` and removed on drop by default.
