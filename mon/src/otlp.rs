//! Optional OTLP metrics export for host samples (via `airbug-otel`).
use airbug_otel::{init, meter, KeyValue, TelemetryConfig, TelemetryGuard};
use opentelemetry::metrics::Gauge;

use crate::sampler::Sample;

pub struct OtlpExport {
    guard: TelemetryGuard,
    cpu: Gauge<f64>,
    mem_used: Gauge<u64>,
    mem_total: Gauge<u64>,
    swap_used: Gauge<u64>,
    net_rx: Gauge<f64>,
    net_tx: Gauge<f64>,
    disk_read: Gauge<f64>,
    disk_write: Gauge<f64>,
    gpu_util: Gauge<f64>,
    gpu_mem_used: Gauge<u64>,
    host: String,
}

impl OtlpExport {
    pub fn start(endpoint: Option<String>) -> Result<Self, airbug_otel::TraceError> {
        let mut cfg = TelemetryConfig::new().service_name("airbug-mon");
        if let Some(endpoint) = endpoint {
            cfg = cfg.endpoint(endpoint);
        }
        let guard = init(cfg)?;
        let m = meter("airbug.mon");
        Ok(Self {
            guard,
            cpu: m
                .f64_gauge("system.cpu.utilization")
                .with_description("Host CPU utilization 0..1")
                .build(),
            mem_used: m
                .u64_gauge("system.memory.usage")
                .with_description("Used memory bytes")
                .build(),
            mem_total: m
                .u64_gauge("system.memory.limit")
                .with_description("Total memory bytes")
                .build(),
            swap_used: m
                .u64_gauge("system.paging.usage")
                .with_description("Used swap bytes")
                .build(),
            net_rx: m
                .f64_gauge("system.network.io")
                .with_description("Network bytes per sample interval")
                .build(),
            net_tx: m.f64_gauge("system.network.io").build(),
            disk_read: m
                .f64_gauge("system.disk.io")
                .with_description("Disk bytes per sample interval")
                .build(),
            disk_write: m.f64_gauge("system.disk.io").build(),
            gpu_util: m
                .f64_gauge("system.gpu.utilization")
                .with_description("GPU utilization 0..1")
                .build(),
            gpu_mem_used: m.u64_gauge("system.gpu.memory.usage").build(),
            host: hostname(),
        })
    }

    pub fn record(&self, sample: &Sample) {
        let host = [KeyValue::new("host.name", self.host.clone())];
        self.cpu
            .record((sample.cpu_total as f64 / 100.0).clamp(0.0, 1.0), &host);
        self.mem_used.record(sample.mem_used, &host);
        self.mem_total.record(sample.mem_total, &host);
        self.swap_used.record(sample.swap_used, &host);
        self.net_rx.record(
            sample.net_rx_rate,
            &[
                KeyValue::new("network.io.direction", "receive"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        self.net_tx.record(
            sample.net_tx_rate,
            &[
                KeyValue::new("network.io.direction", "transmit"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        self.disk_read.record(
            sample.disk_read_rate,
            &[
                KeyValue::new("disk.io.direction", "read"),
                KeyValue::new("host.name", self.host.clone()),
            ],
        );
        self.disk_write.record(
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

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".into())
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
