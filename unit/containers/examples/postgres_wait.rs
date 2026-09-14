//! Postgres-shaped wait example.
//!
//! Compiles without Docker. At runtime, exits early when no container
//! runtime is available — the same soft-skip policy used in CI.
//!
//! ```text
//! cargo run -p airbug-containers --example postgres_wait
//! ```

use airbug_containers::prelude::*;
use airbug_containers::Runtime;
use std::time::Duration;

fn main() -> Result<(), ContainerError> {
    if Runtime::detect().is_err() {
        eprintln!("skip: no docker/podman — CI should soft-skip the same way");
        return Ok(());
    }

    let pg = PostgreSqlBuilder::new()
        .with_database("app")
        .with_username("app")
        .with_password("secret")
        .start()?;
    let uri = pg.get_connection_string()?;
    eprintln!("postgres ready: {uri}");

    // Illustrative HTTP probe shape (Postgres itself is not an HTTP server).
    // After you front the DB with an app health endpoint, wait like this:
    let _ = Wait::http("http://127.0.0.1:9/health").poll(Duration::from_millis(1));
    Ok(())
}
