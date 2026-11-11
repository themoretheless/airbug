//! Optional OTLP metrics export for host samples (via `airbug-otel` only).
use airbug_otel::{
    F64Gauge, KeyValue, TelemetryConfig, TelemetryError, TelemetryGuard, U64Gauge, init,
};

use crate::sampler::Sample;

const SCOPE: &str = "airbug.mon";

pub struct OtlpExport {
    guard: TelemetryGuard,
    cpu: F64Gauge,
    mem_used: U64Gauge,
    mem_total: U64Gauge,
    swap_used: U64Gauge,
    net_io: F64Gauge,
    disk_io: F64Gauge,
    gpu_util: F64Gauge,
    gpu_mem_used: U64Gauge,
    host: String,
}

impl OtlpExport {
    pub fn start(endpoint: Option<String>) -> Result<Self, TelemetryError> {
        let mut cfg = TelemetryConfig::new().service_name("airbug-mon");
        if let Some(endpoint) = endpoint {
            cfg = cfg.endpoint(endpoint);
        }
        let guard = init(cfg)?;
        Ok(Self {
            guard,
            cpu: F64Gauge::with_description(
                SCOPE,
                "system.cpu.utilization",
                "Host CPU utilization 0..1",
            ),
            mem_used: U64Gauge::with_description(SCOPE, "system.memory.usage", "Used memory bytes"),
            mem_total: U64Gauge::with_description(
                SCOPE,
                "system.memory.limit",
                "Total memory bytes",
            ),
            swap_used: U64Gauge::with_description(SCOPE, "system.paging.usage", "Used swap bytes"),
            net_io: F64Gauge::with_description(
                SCOPE,
                "system.network.io",
                "Network bytes per sample interval",
            ),
            disk_io: F64Gauge::with_description(
                SCOPE,
                "system.disk.io",
                "Disk bytes per sample interval",
            ),
            gpu_util: F64Gauge::with_description(
                SCOPE,
                "system.gpu.utilization",
                "GPU utilization 0..1",
            ),
            gpu_mem_used: U64Gauge::new(SCOPE, "system.gpu.memory.usage"),
            host: resolve_hostname(),
        })
    }

    pub fn record(&self, sample: &Sample) {
        let host = [KeyValue::new("host.name", self.host.clone())];
        self.cpu
            .record((sample.cpu_total as f64 / 100.0).clamp(0.0, 1.0), &host);
        self.mem_used.record(sample.mem_used, &host);
        self.mem_total.record(sample.mem_total, &host);
        self.swap_used.record(sample.swap_used, &host);
        self.net_io.record(
            sample.net_rx_rate,
            &[
                KeyValue::new("network.io.direction", "receive"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        self.net_io.record(
            sample.net_tx_rate,
            &[
                KeyValue::new("network.io.direction", "transmit"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        self.disk_io.record(
            sample.disk_read_rate,
            &[
                KeyValue::new("disk.io.direction", "read"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        self.disk_io.record(
            sample.disk_write_rate,
            &[
                KeyValue::new("disk.io.direction", "write"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        if let Some(gpu) = &sample.gpu {
            self.gpu_util
                .record((gpu.util as f64 / 100.0).clamp(0.0, 1.0), &host);
            self.gpu_mem_used.record(gpu.mem_used, &host);
        }
    }

    pub fn shutdown(self) {
        let _ = self.guard.shutdown();
    }
}

fn resolve_hostname() -> String {
    if let Ok(h) = std::env::var("HOSTNAME").or_else(|_| std::env::var("HOST")) {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Ok(s) = std::fs::read_to_string("/etc/hostname") {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Ok(out) = std::process::Command::new("hostname").output()
        && out.status.success()
        && let Ok(s) = String::from_utf8(out.stdout)
    {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "localhost".into()
}

/// Enable when `--otlp` is passed or `OTEL_EXPORTER_OTLP_ENDPOINT` / `AIRBUG_MON_OTLP=1`.
pub fn want_otlp(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--otlp") {
        return true;
    }
    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        return true;
    }
    matches!(
        std::env::var("AIRBUG_MON_OTLP").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

pub fn endpoint_from_args(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == "--otlp-endpoint")
        .map(|w| w[1].clone())
        .or_else(|| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok())
}
