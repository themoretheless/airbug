//! Export a counter + histogram via OTLP. Needs a collector on :4318.
//!
//! ```sh
//! cargo run -p airbug-otel --example metrics
//! ```
use airbug_otel::{add_counter, init, record_histogram, KeyValue, TelemetryConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guard = init(TelemetryConfig::new().service_name("airbug-otel-metrics"))?;

    let attrs = [KeyValue::new("example", "metrics")];
    add_counter("example", "airbug.demo.requests", 3, &attrs);
    record_histogram("example", "airbug.demo.latency_ms", 12.5, &attrs);

    // Give the periodic reader a moment before forced shutdown flush.
    std::thread::sleep(Duration::from_millis(100));
    guard.shutdown()?;
    Ok(())
}
