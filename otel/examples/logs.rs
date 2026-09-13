//! Emit OTLP logs for the hub panel. Needs collector on :4318.
//!
//! ```sh
//! cargo run -p airbug-hub -- serve --root . --collector
//! cargo run -p airbug-otel --example logs
//! ```
use airbug_otel::{init, log_error, log_info, log_warn, TelemetryConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guard = init(TelemetryConfig::new().service_name("airbug-otel-logs"))?;

    log_info("example", "hub logs panel smoke: info");
    log_warn("example", "hub logs panel smoke: warn");
    log_error("example", "hub logs panel smoke: error");

    std::thread::sleep(Duration::from_millis(500));
    guard.shutdown()?;
    Ok(())
}
