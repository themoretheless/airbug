//! OpenTelemetry traces, metrics, and logs for airbug via OTLP.
//!
//! Call [`init`] once at startup and keep [`TelemetryGuard`] until shutdown so
//! exporters can flush. `OTEL_*` environment variables are honored.
use opentelemetry::{
    global,
    logs::{LogRecord as _, Logger as _, LoggerProvider as _, Severity},
    metrics::Meter,
    trace::{TraceContextExt, Tracer as _},
};

pub use opentelemetry::KeyValue;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, resource::Resource,
    trace::SdkTracerProvider,
};
use std::{borrow::Cow, sync::OnceLock};

static LOGS: OnceLock<SdkLoggerProvider> = OnceLock::new();

/// Errors while building or shutting down providers.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error(transparent)]
    Exporter(#[from] opentelemetry_otlp::ExporterBuildError),
    #[error("provider shutdown failed: {0}")]
    Shutdown(String),
    #[cfg(feature = "otlp-grpc")]
    #[error(transparent)]
    Runtime(#[from] std::io::Error),
}

/// Deprecated alias for [`TelemetryError`].
#[deprecated(note = "renamed to TelemetryError")]
pub type TraceError = TelemetryError;

/// Shared config for all signals. Unset fields fall back to `OTEL_*`.
#[derive(Debug, Clone, Default)]
pub struct TelemetryConfig {
    /// Logical service name (`service.name`). Default: `OTEL_SERVICE_NAME` or `"airbug"`.
    pub service_name: Option<String>,
    /// OTLP endpoint override. HTTP default `http://localhost:4318`; gRPC `http://localhost:4317`.
    pub endpoint: Option<String>,
}

/// Alias kept for call sites written against the traces-only API.
#[deprecated(note = "renamed to TelemetryConfig")]
pub type TraceConfig = TelemetryConfig;

impl TelemetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }
}

/// Owns tracer, meter, and logger providers (and a Tokio runtime when using gRPC).
pub struct TelemetryGuard {
    tracer: SdkTracerProvider,
    meter: SdkMeterProvider,
    logs: SdkLoggerProvider,
    #[cfg(feature = "otlp-grpc")]
    _runtime: Option<tokio::runtime::Runtime>,
}

/// Explicit handle returned by [`install`] / [`init`] for embed and tests.
pub type TelemetryHandle = TelemetryGuard;

/// Alias for [`TelemetryGuard`].
#[deprecated(note = "renamed to TelemetryGuard")]
pub type TracerGuard = TelemetryGuard;

impl TelemetryGuard {
    /// Instrumentation-scope tracer on the global provider.
    pub fn tracer(&self, name: impl Into<Cow<'static, str>>) -> opentelemetry::global::BoxedTracer {
        global::tracer(name)
    }

    /// Instrumentation-scope meter on the global provider.
    pub fn meter(&self, name: &'static str) -> Meter {
        global::meter(name)
    }

    /// Flush pending telemetry and shut down all providers.
    pub fn shutdown(self) -> Result<(), TelemetryError> {
        self.tracer
            .shutdown()
            .map_err(|e| TelemetryError::Shutdown(format!("traces: {e}")))?;
        self.meter
            .shutdown()
            .map_err(|e| TelemetryError::Shutdown(format!("metrics: {e}")))?;
        self.logs
            .shutdown()
            .map_err(|e| TelemetryError::Shutdown(format!("logs: {e}")))?;
        Ok(())
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        let _ = self.tracer.shutdown();
        let _ = self.meter.shutdown();
        let _ = self.logs.shutdown();
    }
}

fn service_resource(config: &TelemetryConfig) -> Resource {
    let service = config
        .service_name
        .clone()
        .or_else(|| std::env::var("OTEL_SERVICE_NAME").ok())
        .unwrap_or_else(|| "airbug".into());
    Resource::builder().with_service_name(service).build()
}

fn install_logs(provider: SdkLoggerProvider) -> SdkLoggerProvider {
    let _ = LOGS.set(provider.clone());
    provider
}

/// Install global OTLP tracer, meter, and logger providers.
///
/// Prefer [`install`] when you want an explicit [`TelemetryHandle`] name.
///
/// Default feature `otlp-http` uses HTTP/protobuf. For gRPC:
/// `--no-default-features --features otlp-grpc`. If both features are enabled
/// (e.g. `--all-features`), gRPC wins.
pub fn init(config: TelemetryConfig) -> Result<TelemetryGuard, TelemetryError> {
    install(config)
}

/// Install providers and return a [`TelemetryHandle`] (same as [`init`]).
pub fn install(config: TelemetryConfig) -> Result<TelemetryHandle, TelemetryError> {
    #[cfg(not(any(feature = "otlp-http", feature = "otlp-grpc")))]
    compile_error!("enable airbug-otel feature otlp-http or otlp-grpc");

    let resource = service_resource(&config);

    #[cfg(feature = "otlp-grpc")]
    {
        let runtime = tokio::runtime::Runtime::new()?;
        let endpoint = config.endpoint.clone();
        let (span_exporter, metric_exporter, log_exporter) = runtime.block_on(async {
            let mut spans = SpanExporter::builder().with_tonic();
            let mut metrics = MetricExporter::builder().with_tonic();
            let mut logs = LogExporter::builder().with_tonic();
            if let Some(ref endpoint) = endpoint {
                spans = spans.with_endpoint(endpoint.clone());
                metrics = metrics.with_endpoint(endpoint.clone());
                logs = logs.with_endpoint(endpoint.clone());
            }
            Ok::<_, TelemetryError>((spans.build()?, metrics.build()?, logs.build()?))
        })?;

        let tracer = SdkTracerProvider::builder()
            .with_batch_exporter(span_exporter)
            .with_resource(resource.clone())
            .build();
        let meter = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .with_resource(resource.clone())
            .build();
        let logs = install_logs(
            SdkLoggerProvider::builder()
                .with_batch_exporter(log_exporter)
                .with_resource(resource)
                .build(),
        );

        global::set_tracer_provider(tracer.clone());
        global::set_meter_provider(meter.clone());
        Ok(TelemetryGuard {
            tracer,
            meter,
            logs,
            _runtime: Some(runtime),
        })
    }

    #[cfg(all(feature = "otlp-http", not(feature = "otlp-grpc")))]
    {
        let mut spans = SpanExporter::builder().with_http();
        let mut metrics = MetricExporter::builder().with_http();
        let mut logs = LogExporter::builder().with_http();
        if let Some(endpoint) = config.endpoint {
            spans = spans.with_endpoint(endpoint.clone());
            metrics = metrics.with_endpoint(endpoint.clone());
            logs = logs.with_endpoint(endpoint);
        }
        let tracer = SdkTracerProvider::builder()
            .with_batch_exporter(spans.build()?)
            .with_resource(resource.clone())
            .build();
        let meter = SdkMeterProvider::builder()
            .with_periodic_exporter(metrics.build()?)
            .with_resource(resource.clone())
            .build();
        let logs = install_logs(
            SdkLoggerProvider::builder()
                .with_batch_exporter(logs.build()?)
                .with_resource(resource)
                .build(),
        );

        global::set_tracer_provider(tracer.clone());
        global::set_meter_provider(meter.clone());
        Ok(TelemetryGuard {
            tracer,
            meter,
            logs,
        })
    }

    #[cfg(not(any(feature = "otlp-http", feature = "otlp-grpc")))]
    {
        let _ = (config, resource);
        unreachable!()
    }
}

/// Run `f` inside a named span on the global tracer `scope`.
pub fn in_span<T>(
    scope: &'static str,
    name: impl Into<Cow<'static, str>>,
    f: impl FnOnce() -> T,
) -> T {
    let tracer = global::tracer(scope);
    tracer.in_span(name, |_cx| f())
}

/// Attach a string attribute to the currently active span (no-op if none).
pub fn set_attribute(key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) {
    let cx = opentelemetry::Context::current();
    cx.span()
        .set_attribute(KeyValue::new(key.into(), value.into()));
}

/// Global meter for `scope` (after [`init`]).
pub fn meter(scope: &'static str) -> Meter {
    global::meter(scope)
}

/// Opaque `f64` gauge that hides the OpenTelemetry type from callers.
pub struct F64Gauge {
    inner: opentelemetry::metrics::Gauge<f64>,
}

impl F64Gauge {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).f64_gauge(name).build(),
        }
    }

    pub fn with_description(
        scope: &'static str,
        name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            inner: meter(scope)
                .f64_gauge(name)
                .with_description(description)
                .build(),
        }
    }

    pub fn record(&self, value: f64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// Opaque `u64` gauge that hides the OpenTelemetry type from callers.
pub struct U64Gauge {
    inner: opentelemetry::metrics::Gauge<u64>,
}

impl U64Gauge {
    pub fn new(scope: &'static str, name: &'static str) -> Self {
        Self {
            inner: meter(scope).u64_gauge(name).build(),
        }
    }

    pub fn with_description(
        scope: &'static str,
        name: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            inner: meter(scope)
                .u64_gauge(name)
                .with_description(description)
                .build(),
        }
    }

    pub fn record(&self, value: u64, attrs: &[KeyValue]) {
        self.inner.record(value, attrs);
    }
}

/// Increment a `u64` counter on the global meter.
pub fn add_counter(scope: &'static str, name: &'static str, value: u64, attrs: &[KeyValue]) {
    let counter = meter(scope).u64_counter(name).build();
    counter.add(value, attrs);
}

/// Record a `f64` histogram observation on the global meter.
pub fn record_histogram(scope: &'static str, name: &'static str, value: f64, attrs: &[KeyValue]) {
    let histogram = meter(scope).f64_histogram(name).build();
    histogram.record(value, attrs);
}

/// Emit a log record after [`init`] (no-op if telemetry was never installed).
pub fn emit_log(scope: &'static str, severity: Severity, body: impl AsRef<str>) {
    let Some(provider) = LOGS.get() else {
        return;
    };
    let logger = provider.logger(scope);
    let mut record = logger.create_log_record();
    record.set_severity_text(severity.name());
    record.set_severity_number(severity);
    record.set_body(body.as_ref().to_string().into());
    logger.emit(record);
}

/// Convenience: INFO log.
pub fn log_info(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Info, body);
}

/// Convenience: WARN log.
pub fn log_warn(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Warn, body);
}

/// Convenience: ERROR log.
pub fn log_error(scope: &'static str, body: impl AsRef<str>) {
    emit_log(scope, Severity::Error, body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builders() {
        let cfg = TelemetryConfig::new()
            .service_name("demo")
            .endpoint("http://127.0.0.1:4318");
        assert_eq!(cfg.service_name.as_deref(), Some("demo"));
        assert_eq!(cfg.endpoint.as_deref(), Some("http://127.0.0.1:4318"));
    }
}
