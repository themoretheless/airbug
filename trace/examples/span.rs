//! Minimal span export demo. Needs a collector on :4318 (or set OTEL_EXPORTER_OTLP_ENDPOINT).
//!
//! ```sh
//! cargo run -p airbug-trace --example span --release
//! ```
use airbug_trace::{in_span, set_attribute, init, TelemetryConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guard = init(TelemetryConfig::new().service_name("airbug-trace-example"))?;

    in_span("example", "root", || {
        set_attribute("demo", "true");
        in_span("example", "child", || {
            std::thread::sleep(Duration::from_millis(20));
        });
    });

    guard.shutdown()?;
    Ok(())
}
