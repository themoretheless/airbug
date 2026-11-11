//! OpenTelemetry traces, metrics, logs, and profiles for airbug via OTLP.
//!
//! Call [`init`] (or [`install`]) once at startup and keep [`TelemetryGuard`] until shutdown so
//! exporters can flush. `OTEL_*` and `AIRBUG_*` environment variables are honored.

pub mod logs;
pub mod metrics;
pub mod profiles;
pub mod propagation;
pub mod traces;
#[cfg(feature = "tracing-bridge")]
pub mod tracing_bridge;

pub use logs::{
    LogRecordSpec, Severity, emit_log, emit_log_with_attrs, emit_structured_log, log_debug,
    log_error, log_fatal, log_info, log_trace, log_warn,
};
pub use metrics::{
    F64Counter, F64Gauge, F64Histogram, F64UpDownCounter, I64Gauge, I64UpDownCounter,
    MetricObserver, ObservableF64Gauge, ObservableI64UpDownCounter, ObservableU64Counter,
    U64Counter, U64Gauge, U64Histogram, add_counter, add_f64_counter, add_f64_up_down_counter,
    add_up_down_counter, meter, record_histogram, record_u64_histogram,
};
pub use opentelemetry::{Context, KeyValue, Value};
pub use profiles::{
    OtlpProfilesData, ProfileBatch, ProfileFrame, ProfileSample, encode_profiles_json,
    encode_profiles_protobuf, export_profiles_http,
};
pub use propagation::{
    baggage_value, current_span_id, current_trace_id, current_traceparent, extract_map,
    extract_with, inject_context_map, inject_map, inject_with, install_w3c_propagator,
    with_baggage, with_baggage_async, with_extracted_headers, with_extracted_headers_async,
};
pub use traces::{
    SpanKind, SpanLink, SpanOptions, add_span_event, in_span, in_span_async,
    in_span_async_with_options, in_span_with_options, record_exception, record_exception_message,
    set_attribute, set_attributes, set_span_status_error, set_span_status_ok, update_span_name,
};
#[cfg(feature = "tracing-bridge")]
pub use tracing_bridge::{TracingOtelLayer, tracing_layer};

use opentelemetry::global;
#[cfg(feature = "otlp-http")]
use opentelemetry_otlp::WithHttpConfig;
#[cfg(feature = "otlp-grpc")]
use opentelemetry_otlp::WithTonicConfig;
#[cfg(any(feature = "otlp-http", feature = "otlp-grpc"))]
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider,
    metrics::{Aggregation, Instrument, SdkMeterProvider, Stream},
    resource::Resource,
    trace::{Sampler, SdkTracerProvider},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

static LOGS: OnceLock<Mutex<Option<(u64, SdkLoggerProvider)>>> = OnceLock::new();
static ACTIVE_GUARD_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_GUARD_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn logs_slot() -> &'static Mutex<Option<(u64, SdkLoggerProvider)>> {
    LOGS.get_or_init(|| Mutex::new(None))
}

#[cfg(feature = "testing")]
static TEST_PROFILES: OnceLock<Mutex<Option<InMemoryProfileExporter>>> = OnceLock::new();

#[cfg(feature = "testing")]
fn test_profiles_slot() -> &'static Mutex<Option<InMemoryProfileExporter>> {
    TEST_PROFILES.get_or_init(|| Mutex::new(None))
}

#[cfg(feature = "testing")]
pub(crate) fn record_test_profile(data: profiles::OtlpProfilesData) {
    if let Ok(slot) = test_profiles_slot().lock()
        && let Some(exp) = slot.as_ref()
    {
        exp.push(data);
    }
}

/// Errors while building or shutting down providers.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error(transparent)]
    Exporter(#[from] opentelemetry_otlp::ExporterBuildError),
    #[error("provider shutdown failed: {0}")]
    Shutdown(String),
    #[error("telemetry configuration error: {0}")]
    Config(String),
    #[cfg(feature = "otlp-grpc")]
    #[error(transparent)]
    Runtime(#[from] std::io::Error),
}

/// Deprecated alias for [`TelemetryError`].
#[deprecated(note = "renamed to TelemetryError")]
pub type TraceError = TelemetryError;

/// OTLP wire protocol used by [`install`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// OTLP over gRPC (`grpc`); collector port 4317.
    Grpc,
    /// OTLP over HTTP/protobuf (`http/protobuf`); collector port 4318.
    Http,
    /// OTLP over HTTP/JSON (`http/json`); collector port 4318.
    HttpJson,
}

impl Protocol {
    /// Alias constant for [`Protocol::Http`] (`http/protobuf`).
    pub const HTTP_PROTO: Self = Self::Http;
}

/// Wire payload compression for OTLP exporters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    #[default]
    None,
    Gzip,
}

/// Trace sampling strategy applied to [`SdkTracerProvider`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraceSampler {
    AlwaysOn,
    AlwaysOff,
    TraceIdRatio(f64),
    ParentBasedAlwaysOn,
    ParentBasedAlwaysOff,
    ParentBasedRatio(f64),
}

impl TraceSampler {
    fn into_sdk_sampler(self) -> Sampler {
        match self {
            Self::AlwaysOn => Sampler::AlwaysOn,
            Self::AlwaysOff => Sampler::AlwaysOff,
            Self::TraceIdRatio(r) => Sampler::TraceIdRatioBased(r.clamp(0.0, 1.0)),
            Self::ParentBasedAlwaysOn => Sampler::ParentBased(Box::new(Sampler::AlwaysOn)),
            Self::ParentBasedAlwaysOff => Sampler::ParentBased(Box::new(Sampler::AlwaysOff)),
            Self::ParentBasedRatio(r) => {
                Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(r.clamp(0.0, 1.0))))
            }
        }
    }
}

fn resolve_sampler(explicit: Option<TraceSampler>) -> Sampler {
    if let Some(s) = explicit {
        return s.into_sdk_sampler();
    }
    parse_sampler_env(
        std::env::var("OTEL_TRACES_SAMPLER").ok().as_deref(),
        std::env::var("OTEL_TRACES_SAMPLER_ARG").ok().as_deref(),
    )
    .into_sdk_sampler()
}

fn parse_sampler_env(name: Option<&str>, arg: Option<&str>) -> TraceSampler {
    let ratio = || {
        arg.and_then(|s| s.trim().parse::<f64>().ok())
            .map(|r| r.clamp(0.0, 1.0))
            .unwrap_or(1.0)
    };
    match name.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("always_on") => TraceSampler::AlwaysOn,
        Some("always_off") => TraceSampler::AlwaysOff,
        Some("traceidratio") => TraceSampler::TraceIdRatio(ratio()),
        Some("parentbased_always_on") => TraceSampler::ParentBasedAlwaysOn,
        Some("parentbased_always_off") => TraceSampler::ParentBasedAlwaysOff,
        Some("parentbased_traceidratio") => TraceSampler::ParentBasedRatio(ratio()),
        _ => TraceSampler::ParentBasedAlwaysOn,
    }
}

#[cfg_attr(
    not(any(feature = "otlp-http", feature = "otlp-grpc")),
    allow(dead_code)
)]
fn resolve_compression(explicit: Option<Compression>, env_val: Option<&str>) -> Compression {
    if let Some(c) = explicit {
        return c;
    }
    match env_val.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("gzip") => Compression::Gzip,
        _ => Compression::None,
    }
}

/// Shared config for all signals. Unset fields fall back to `OTEL_*` and `AIRBUG_*`.
#[derive(Debug, Clone, Default)]
pub struct TelemetryConfig {
    /// Logical service name (`service.name`). Default: `OTEL_SERVICE_NAME` or `"airbug"`.
    pub service_name: Option<String>,
    /// Optional `service.version` resource attribute.
    pub service_version: Option<String>,
    /// Optional `deployment.environment.name` resource attribute.
    pub environment: Option<String>,
    /// Optional `airbug.hub_id` resource attribute (falls back to `AIRBUG_HUB_ID`).
    pub hub_id: Option<String>,
    /// Optional `airbug.run_id` resource attribute (falls back to `AIRBUG_RUN_ID`).
    pub run_id: Option<String>,
    /// Extra OpenTelemetry resource attributes.
    pub resource_attributes: Vec<KeyValue>,
    /// OTLP endpoint override (`http://127.0.0.1:4318` for HTTP, `http://127.0.0.1:4317` for gRPC).
    pub endpoint: Option<String>,
    /// Explicit OTLP wire protocol override (`OTEL_EXPORTER_OTLP_PROTOCOL` fallback).
    pub protocol: Option<Protocol>,
    /// Payload compression (`OTEL_EXPORTER_OTLP_COMPRESSION` fallback).
    pub compression: Option<Compression>,
    /// Additional HTTP/gRPC metadata headers (`OTEL_EXPORTER_OTLP_HEADERS` merged on top).
    pub headers: HashMap<String, String>,
    /// Exporter request timeout (`OTEL_EXPORTER_OTLP_TIMEOUT` fallback).
    pub timeout: Option<Duration>,
    /// Trace sampler (`OTEL_TRACES_SAMPLER` + `OTEL_TRACES_SAMPLER_ARG` fallback).
    pub sampler: Option<TraceSampler>,
    /// Custom explicit histogram bucket boundaries keyed by instrument name.
    pub histogram_views: Vec<(String, Vec<f64>)>,
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

    pub fn service_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = Some(version.into());
        self
    }

    pub fn environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    pub fn hub_id(mut self, hub_id: impl Into<String>) -> Self {
        self.hub_id = Some(hub_id.into());
        self
    }

    pub fn run_id(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }

    pub fn resource_attribute(
        mut self,
        key: impl Into<opentelemetry::Key>,
        value: impl Into<Value>,
    ) -> Self {
        self.resource_attributes.push(KeyValue::new(key, value));
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn compression(mut self, compression: Compression) -> Self {
        self.compression = Some(compression);
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn sampler(mut self, sampler: TraceSampler) -> Self {
        self.sampler = Some(sampler);
        self
    }

    pub fn histogram_buckets(
        mut self,
        instrument_name: impl Into<String>,
        boundaries: Vec<f64>,
    ) -> Self {
        self.histogram_views
            .push((instrument_name.into(), boundaries));
        self
    }
}

/// Owns tracer, meter, and logger providers (and a Tokio runtime when using gRPC).
pub struct TelemetryGuard {
    tracer: SdkTracerProvider,
    meter: SdkMeterProvider,
    logs: SdkLoggerProvider,
    guard_id: u64,
    shut_down: bool,
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
    pub fn meter(&self, name: &'static str) -> opentelemetry::metrics::Meter {
        global::meter(name)
    }

    /// Flush pending telemetry without shutting down.
    pub fn force_flush(&self) -> Result<(), TelemetryError> {
        let r1 = self
            .tracer
            .force_flush()
            .map_err(|e| TelemetryError::Shutdown(format!("traces flush: {e}")));
        let r2 = self
            .meter
            .force_flush()
            .map_err(|e| TelemetryError::Shutdown(format!("metrics flush: {e}")));
        let r3 = self
            .logs
            .force_flush()
            .map_err(|e| TelemetryError::Shutdown(format!("logs flush: {e}")));
        r1.and(r2).and(r3)
    }

    /// Flush pending telemetry and shut down all providers.
    pub fn shutdown(mut self) -> Result<(), TelemetryError> {
        self.shut_down = true;
        deactivate_guard(self.guard_id);
        let r1 = self
            .tracer
            .shutdown()
            .map_err(|e| TelemetryError::Shutdown(format!("traces: {e}")));
        let r2 = self
            .meter
            .shutdown()
            .map_err(|e| TelemetryError::Shutdown(format!("metrics: {e}")));
        let r3 = self
            .logs
            .shutdown()
            .map_err(|e| TelemetryError::Shutdown(format!("logs: {e}")));
        r1.and(r2).and(r3)
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if !self.shut_down {
            self.shut_down = true;
            deactivate_guard(self.guard_id);
            let _ = self.tracer.shutdown();
            let _ = self.meter.shutdown();
            let _ = self.logs.shutdown();
        }
        #[cfg(feature = "otlp-grpc")]
        if let Some(rt) = self._runtime.take() {
            if tokio::runtime::Handle::try_current().is_ok() {
                let _ = std::thread::spawn(move || drop(rt)).join();
            } else {
                drop(rt);
            }
        }
    }
}

fn service_resource(config: &TelemetryConfig) -> Resource {
    let mut builder = Resource::builder();
    let explicit_service = config
        .service_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let env_service = std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let resource_attrs = std::env::var("OTEL_RESOURCE_ATTRIBUTES")
        .ok()
        .unwrap_or_default();

    let has_env_attr = |target_key: &str| {
        resource_attrs.split(',').any(|pair| {
            pair.split_once('=')
                .map(|(k, v)| k.trim() == target_key && !v.trim().is_empty())
                .unwrap_or(false)
        })
    };

    if let Some(service) = explicit_service {
        builder = builder.with_service_name(service);
    } else if let Some(service) = env_service {
        builder = builder.with_service_name(service);
    } else if !has_env_attr("service.name") {
        builder = builder.with_service_name("airbug");
    }

    if let Some(version) = config
        .service_version
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        builder = builder.with_attribute(KeyValue::new("service.version", version.to_string()));
    }

    if let Some(env) = config
        .environment
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        builder = builder.with_attribute(KeyValue::new(
            "deployment.environment.name",
            env.to_string(),
        ));
    }

    let resolved_hub_id = config
        .hub_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            (!has_env_attr("airbug.hub_id"))
                .then(|| std::env::var("AIRBUG_HUB_ID").ok())
                .flatten()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });
    if let Some(hub_id) = resolved_hub_id {
        builder = builder.with_attribute(KeyValue::new("airbug.hub_id", hub_id));
    }

    let resolved_run_id = config
        .run_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            (!has_env_attr("airbug.run_id"))
                .then(|| std::env::var("AIRBUG_RUN_ID").ok())
                .flatten()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });
    if let Some(run_id) = resolved_run_id {
        builder = builder.with_attribute(KeyValue::new("airbug.run_id", run_id));
    }

    if !config.resource_attributes.is_empty() {
        builder = builder.with_attributes(config.resource_attributes.iter().cloned());
    }

    builder.build()
}

fn apply_histogram_views(
    mut builder: opentelemetry_sdk::metrics::MeterProviderBuilder,
    views: &[(String, Vec<f64>)],
) -> opentelemetry_sdk::metrics::MeterProviderBuilder {
    for (target_name, boundaries) in views {
        let target_name = target_name.clone();
        let boundaries = boundaries.clone();
        builder = builder.with_view(move |inst: &Instrument| {
            if inst.name() == target_name {
                Stream::builder()
                    .with_aggregation(Aggregation::ExplicitBucketHistogram {
                        boundaries: boundaries.clone(),
                        record_min_max: true,
                    })
                    .build()
                    .ok()
            } else {
                None
            }
        });
    }
    builder
}

fn activate_providers(
    tracer: &SdkTracerProvider,
    meter: &SdkMeterProvider,
    logs: SdkLoggerProvider,
) -> (u64, SdkLoggerProvider) {
    install_w3c_propagator();
    metrics::invalidate_instrument_cache();
    let id = NEXT_GUARD_ID.fetch_add(1, Ordering::Relaxed);
    ACTIVE_GUARD_ID.store(id, Ordering::SeqCst);
    if let Ok(mut slot) = logs_slot().lock() {
        *slot = Some((id, logs.clone()));
    }
    global::set_tracer_provider(tracer.clone());
    global::set_meter_provider(meter.clone());
    (id, logs)
}

fn deactivate_guard(id: u64) {
    if let Ok(mut slot) = logs_slot().lock()
        && slot
            .as_ref()
            .map(|(cur_id, _)| *cur_id == id)
            .unwrap_or(false)
    {
        *slot = None;
    }
    if ACTIVE_GUARD_ID
        .compare_exchange(id, 0, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        metrics::invalidate_instrument_cache();
        global::set_tracer_provider(opentelemetry::trace::noop::NoopTracerProvider::new());
        global::set_meter_provider(SdkMeterProvider::default());
    }
}

/// Install global OTLP tracer, meter, logger, and W3C propagation providers.
///
/// Prefer [`install`] when you want an explicit [`TelemetryHandle`] name.
pub fn init(config: TelemetryConfig) -> Result<TelemetryGuard, TelemetryError> {
    install(config)
}

/// Protocol for the value of `OTEL_EXPORTER_OTLP_PROTOCOL`, clamped to what
/// this build compiled in. `None` means the variable is unset.
#[cfg_attr(
    not(any(feature = "otlp-http", feature = "otlp-grpc")),
    allow(dead_code)
)]
fn resolve_protocol(explicit: Option<Protocol>, requested_env: Option<&str>) -> Protocol {
    let available = |protocol: Protocol| match protocol {
        Protocol::Grpc => cfg!(feature = "otlp-grpc"),
        Protocol::Http | Protocol::HttpJson => cfg!(feature = "otlp-http"),
    };
    let fallback = if available(Protocol::Http) || !available(Protocol::Grpc) {
        Protocol::Http
    } else {
        Protocol::Grpc
    };
    if let Some(proto) = explicit {
        return if available(proto) { proto } else { fallback };
    }
    let chosen = match requested_env.map(|value| value.trim().to_ascii_lowercase()) {
        Some(ref value) if value == "grpc" => Protocol::Grpc,
        Some(ref value) if value == "http/json" => Protocol::HttpJson,
        Some(ref value) if value.starts_with("http") => Protocol::Http,
        _ => fallback,
    };
    if available(chosen) { chosen } else { fallback }
}

/// Install providers and return a [`TelemetryHandle`] (same as [`init`]).
pub fn install(config: TelemetryConfig) -> Result<TelemetryHandle, TelemetryError> {
    #[cfg(not(any(feature = "otlp-http", feature = "otlp-grpc")))]
    {
        let _ = config;
        Err(TelemetryError::Config(
            "enable airbug-otel feature otlp-http or otlp-grpc".into(),
        ))
    }

    #[cfg(any(feature = "otlp-http", feature = "otlp-grpc"))]
    {
        let requested_env = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").ok();
        let protocol = resolve_protocol(config.protocol, requested_env.as_deref());
        match protocol {
            #[cfg(feature = "otlp-grpc")]
            Protocol::Grpc => install_grpc(&config),
            #[cfg(feature = "otlp-http")]
            Protocol::Http | Protocol::HttpJson => install_http(&config, protocol),
            #[cfg(not(all(feature = "otlp-http", feature = "otlp-grpc")))]
            _ => unreachable!("resolve_protocol only returns a compiled-in feature"),
        }
    }
}

#[cfg(feature = "otlp-grpc")]
fn build_tonic_metadata(headers: &HashMap<String, String>) -> tonic::metadata::MetadataMap {
    use std::str::FromStr;
    use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};

    let mut map = MetadataMap::new();
    for (k, v) in headers {
        if let (Ok(key), Ok(val)) = (
            MetadataKey::from_str(k.trim()),
            MetadataValue::try_from(v.as_str()),
        ) {
            map.insert(key, val);
        }
    }
    map
}

#[cfg(feature = "otlp-grpc")]
fn install_grpc(config: &TelemetryConfig) -> Result<TelemetryHandle, TelemetryError> {
    let resource = service_resource(config);
    let sampler = resolve_sampler(config.sampler);
    let compression = resolve_compression(
        config.compression,
        std::env::var("OTEL_EXPORTER_OTLP_COMPRESSION")
            .ok()
            .as_deref(),
    );

    let owned_runtime = if tokio::runtime::Handle::try_current().is_ok() {
        None
    } else {
        Some(tokio::runtime::Runtime::new()?)
    };
    let _enter = owned_runtime.as_ref().map(|rt| rt.enter());

    let mut spans = SpanExporter::builder().with_tonic();
    let mut metrics = MetricExporter::builder().with_tonic();
    let mut logs = LogExporter::builder().with_tonic();

    if let Some(ref endpoint) = config.endpoint {
        spans = spans.with_endpoint(endpoint.clone());
        metrics = metrics.with_endpoint(endpoint.clone());
        logs = logs.with_endpoint(endpoint.clone());
    }
    if let Some(timeout) = config.timeout {
        spans = spans.with_timeout(timeout);
        metrics = metrics.with_timeout(timeout);
        logs = logs.with_timeout(timeout);
    }
    if !config.headers.is_empty() {
        spans = spans.with_metadata(build_tonic_metadata(&config.headers));
        metrics = metrics.with_metadata(build_tonic_metadata(&config.headers));
        logs = logs.with_metadata(build_tonic_metadata(&config.headers));
    }
    if compression == Compression::Gzip {
        spans = spans.with_compression(opentelemetry_otlp::Compression::Gzip);
        metrics = metrics.with_compression(opentelemetry_otlp::Compression::Gzip);
        logs = logs.with_compression(opentelemetry_otlp::Compression::Gzip);
    }

    let span_exporter = spans.build()?;
    let metric_exporter = metrics.build()?;
    let log_exporter = logs.build()?;

    let tracer = SdkTracerProvider::builder()
        .with_sampler(sampler)
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    let meter_builder = apply_histogram_views(
        SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .with_resource(resource.clone()),
        &config.histogram_views,
    );
    let meter = meter_builder.build();
    let logs_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource)
        .build();

    drop(_enter);
    let (guard_id, logs) = activate_providers(&tracer, &meter, logs_provider);
    Ok(TelemetryGuard {
        tracer,
        meter,
        logs,
        guard_id,
        shut_down: false,
        _runtime: owned_runtime,
    })
}

#[cfg(feature = "otlp-http")]
#[derive(Debug)]
struct SafeBlockingClient(Option<reqwest::blocking::Client>);

#[cfg(feature = "otlp-http")]
impl Drop for SafeBlockingClient {
    fn drop(&mut self) {
        if let Some(client) = self.0.take() {
            let _ = std::thread::spawn(move || drop(client)).join();
        }
    }
}

#[cfg(feature = "otlp-http")]
#[derive(Clone, Debug)]
struct SafeBlockingHttpClient {
    inner: Arc<SafeBlockingClient>,
    gzip: bool,
}

#[cfg(feature = "otlp-http")]
impl SafeBlockingHttpClient {
    fn new(timeout: Option<Duration>, gzip: bool) -> Self {
        let client = std::thread::spawn(move || {
            let mut b = reqwest::blocking::Client::builder();
            if let Some(t) = timeout {
                b = b.timeout(t);
            }
            b.build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
        .join()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self {
            inner: Arc::new(SafeBlockingClient(Some(client))),
            gzip,
        }
    }
}

#[cfg(feature = "otlp-http")]
#[async_trait::async_trait]
impl opentelemetry_http::HttpClient for SafeBlockingHttpClient {
    async fn send_bytes(
        &self,
        request: opentelemetry_http::Request<opentelemetry_http::Bytes>,
    ) -> Result<
        opentelemetry_http::Response<opentelemetry_http::Bytes>,
        opentelemetry_http::HttpError,
    > {
        let req = if self.gzip {
            use flate2::{Compression as GzLevel, write::GzEncoder};
            use std::io::Write;

            let (mut parts, body) = request.into_parts();
            let mut encoder = GzEncoder::new(Vec::new(), GzLevel::default());
            encoder.write_all(&body)?;
            let compressed = encoder.finish()?;
            parts.headers.insert(
                "content-encoding",
                "gzip".parse().expect("valid gzip header value"),
            );
            opentelemetry_http::Request::from_parts(
                parts,
                opentelemetry_http::Bytes::from(compressed),
            )
        } else {
            request
        };
        let inner = self.inner.0.as_ref().ok_or("HTTP client already dropped")?;
        opentelemetry_http::HttpClient::send_bytes(inner, req).await
    }
}

#[cfg(feature = "otlp-http")]
fn http_signal_endpoint(endpoint: &str, signal_path: &str) -> String {
    let base = endpoint.trim().trim_end_matches('/');
    if base.ends_with(signal_path) {
        base.to_string()
    } else {
        format!("{base}{signal_path}")
    }
}

#[cfg(feature = "otlp-http")]
fn install_http(
    config: &TelemetryConfig,
    protocol: Protocol,
) -> Result<TelemetryHandle, TelemetryError> {
    let resource = service_resource(config);
    let sampler = resolve_sampler(config.sampler);
    let compression = resolve_compression(
        config.compression,
        std::env::var("OTEL_EXPORTER_OTLP_COMPRESSION")
            .ok()
            .as_deref(),
    );
    let otlp_proto = match protocol {
        Protocol::HttpJson => opentelemetry_otlp::Protocol::HttpJson,
        Protocol::Http | Protocol::Grpc => opentelemetry_otlp::Protocol::HttpBinary,
    };

    let mut spans = SpanExporter::builder()
        .with_http()
        .with_protocol(otlp_proto);
    let mut metrics = MetricExporter::builder()
        .with_http()
        .with_protocol(otlp_proto);
    let mut logs = LogExporter::builder().with_http().with_protocol(otlp_proto);

    if let Some(ref endpoint) = config.endpoint {
        spans = spans.with_endpoint(http_signal_endpoint(endpoint, "/v1/traces"));
        metrics = metrics.with_endpoint(http_signal_endpoint(endpoint, "/v1/metrics"));
        logs = logs.with_endpoint(http_signal_endpoint(endpoint, "/v1/logs"));
    }
    if let Some(timeout) = config.timeout {
        spans = spans.with_timeout(timeout);
        metrics = metrics.with_timeout(timeout);
        logs = logs.with_timeout(timeout);
    }
    if !config.headers.is_empty() {
        spans = spans.with_headers(config.headers.clone());
        metrics = metrics.with_headers(config.headers.clone());
        logs = logs.with_headers(config.headers.clone());
    }
    let http_client = SafeBlockingHttpClient::new(config.timeout, compression == Compression::Gzip);
    spans = spans.with_http_client(http_client.clone());
    metrics = metrics.with_http_client(http_client.clone());
    logs = logs.with_http_client(http_client);

    let tracer = SdkTracerProvider::builder()
        .with_sampler(sampler)
        .with_batch_exporter(spans.build()?)
        .with_resource(resource.clone())
        .build();
    let meter_builder = apply_histogram_views(
        SdkMeterProvider::builder()
            .with_periodic_exporter(metrics.build()?)
            .with_resource(resource.clone()),
        &config.histogram_views,
    );
    let meter = meter_builder.build();
    let logs_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(logs.build()?)
        .with_resource(resource)
        .build();

    let (guard_id, logs) = activate_providers(&tracer, &meter, logs_provider);
    Ok(TelemetryGuard {
        tracer,
        meter,
        logs,
        guard_id,
        shut_down: false,
        #[cfg(feature = "otlp-grpc")]
        _runtime: None,
    })
}

/// In-memory collector for OTLP [`profiles::OtlpProfilesData`] in tests (`feature = "testing"`).
#[cfg(feature = "testing")]
#[derive(Clone, Default, Debug)]
pub struct InMemoryProfileExporter {
    items: Arc<Mutex<Vec<profiles::OtlpProfilesData>>>,
}

#[cfg(feature = "testing")]
impl InMemoryProfileExporter {
    pub fn push(&self, data: profiles::OtlpProfilesData) {
        if let Ok(mut guard) = self.items.lock() {
            guard.push(data);
        }
    }

    pub fn get_exported_profiles(&self) -> Vec<profiles::OtlpProfilesData> {
        self.items.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.items.lock().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn reset(&self) {
        if let Ok(mut guard) = self.items.lock() {
            guard.clear();
        }
    }
}

/// In-memory OTLP stand-ins for tests (`feature = "testing"`).
#[cfg(feature = "testing")]
#[derive(Clone)]
pub struct TestExporters {
    /// Finished spans collected by the in-memory span exporter.
    pub spans: opentelemetry_sdk::trace::InMemorySpanExporter,
    /// Finished metrics collected by the in-memory metric exporter.
    pub metrics: opentelemetry_sdk::metrics::InMemoryMetricExporter,
    /// Emitted logs collected by the in-memory log exporter.
    pub logs: opentelemetry_sdk::logs::InMemoryLogExporter,
    /// Exported OTLP Profiles batches collected in-memory.
    pub profiles: InMemoryProfileExporter,
}

#[cfg(feature = "testing")]
impl TestExporters {
    pub fn reset(&self) {
        self.spans.reset();
        self.metrics.reset();
        self.logs.reset();
        self.profiles.reset();
    }
}

/// Install providers backed by in-memory exporters (no network).
///
/// Requires `--features testing`. Serialize tests that call this — globals are
/// process-wide.
#[cfg(feature = "testing")]
pub fn install_test(
    config: TelemetryConfig,
) -> Result<(TelemetryHandle, TestExporters), TelemetryError> {
    use opentelemetry_sdk::metrics::PeriodicReader;

    let resource = service_resource(&config);
    let sampler = resolve_sampler(config.sampler);
    let spans = opentelemetry_sdk::trace::InMemorySpanExporter::default();
    let metrics = opentelemetry_sdk::metrics::InMemoryMetricExporter::default();
    let logs_exporter = opentelemetry_sdk::logs::InMemoryLogExporter::default();
    let profiles_exporter = InMemoryProfileExporter::default();
    if let Ok(mut slot) = test_profiles_slot().lock() {
        *slot = Some(profiles_exporter.clone());
    }

    let tracer = SdkTracerProvider::builder()
        .with_sampler(sampler)
        .with_simple_exporter(spans.clone())
        .with_resource(resource.clone())
        .build();
    let meter_builder = apply_histogram_views(
        SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(metrics.clone()).build())
            .with_resource(resource.clone()),
        &config.histogram_views,
    );
    let meter = meter_builder.build();
    let logs_provider = SdkLoggerProvider::builder()
        .with_simple_exporter(logs_exporter.clone())
        .with_resource(resource)
        .build();

    let (guard_id, logs) = activate_providers(&tracer, &meter, logs_provider);

    let handle = TelemetryGuard {
        tracer,
        meter,
        logs,
        guard_id,
        shut_down: false,
        #[cfg(feature = "otlp-grpc")]
        _runtime: None,
    };
    let exporters = TestExporters {
        spans,
        metrics,
        logs: logs_exporter,
        profiles: profiles_exporter,
    };
    Ok((handle, exporters))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Globals are process-wide; serialize install/install_test.
    #[cfg(any(feature = "testing", feature = "otlp-grpc", feature = "otlp-http"))]
    static INIT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn config_builders_and_sampler_parsing() {
        let cfg = TelemetryConfig::new()
            .service_name("demo")
            .service_version("1.2.3")
            .environment("staging")
            .hub_id("hub-1")
            .run_id("run-9")
            .resource_attribute("host.arch", "aarch64")
            .endpoint("http://127.0.0.1:4318")
            .protocol(Protocol::HttpJson)
            .compression(Compression::Gzip)
            .header("x-api-key", "secret")
            .timeout(Duration::from_secs(5))
            .sampler(TraceSampler::TraceIdRatio(0.5))
            .histogram_buckets("req_ms", vec![1.0, 5.0, 25.0, 100.0]);
        assert_eq!(cfg.service_name.as_deref(), Some("demo"));
        assert_eq!(cfg.service_version.as_deref(), Some("1.2.3"));
        assert_eq!(cfg.environment.as_deref(), Some("staging"));
        assert_eq!(cfg.hub_id.as_deref(), Some("hub-1"));
        assert_eq!(cfg.run_id.as_deref(), Some("run-9"));
        assert_eq!(cfg.protocol, Some(Protocol::HttpJson));
        assert_eq!(cfg.compression, Some(Compression::Gzip));
        assert_eq!(
            cfg.headers.get("x-api-key").map(String::as_str),
            Some("secret")
        );
        assert_eq!(cfg.timeout, Some(Duration::from_secs(5)));
        assert_eq!(cfg.histogram_views.len(), 1);

        assert_eq!(
            parse_sampler_env(Some("always_off"), None),
            TraceSampler::AlwaysOff
        );
        assert_eq!(
            parse_sampler_env(Some("traceidratio"), Some("0.25")),
            TraceSampler::TraceIdRatio(0.25)
        );
        assert_eq!(
            parse_sampler_env(Some("parentbased_traceidratio"), Some("0.75")),
            TraceSampler::ParentBasedRatio(0.75)
        );
        assert_eq!(resolve_compression(None, Some("gzip")), Compression::Gzip);
    }

    #[test]
    fn protocol_defaults_to_a_compiled_feature() {
        let expected = if cfg!(feature = "otlp-http") || !cfg!(feature = "otlp-grpc") {
            Protocol::Http
        } else {
            Protocol::Grpc
        };
        assert_eq!(resolve_protocol(None, None), expected);
        assert_eq!(resolve_protocol(None, Some("")), expected);
        assert_eq!(resolve_protocol(None, Some("nonsense")), expected);
    }

    #[test]
    fn protocol_grpc_requires_the_grpc_feature() {
        let expected = if cfg!(feature = "otlp-grpc") {
            Protocol::Grpc
        } else {
            Protocol::Http
        };
        assert_eq!(resolve_protocol(None, Some("grpc")), expected);
        assert_eq!(resolve_protocol(None, Some("  gRPC ")), expected);
    }

    #[test]
    fn protocol_http_and_http_json_resolution() {
        let expected_proto = if cfg!(feature = "otlp-http") || !cfg!(feature = "otlp-grpc") {
            Protocol::Http
        } else {
            Protocol::Grpc
        };
        let expected_json = if cfg!(feature = "otlp-http") {
            Protocol::HttpJson
        } else if cfg!(feature = "otlp-grpc") {
            Protocol::Grpc
        } else {
            Protocol::Http
        };
        assert_eq!(
            resolve_protocol(None, Some("http/protobuf")),
            expected_proto
        );
        assert_eq!(resolve_protocol(None, Some("http/json")), expected_json);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn install_test_captures_span_and_auto_correlated_log() {
        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (handle, exporters) = install_test(
            TelemetryConfig::new()
                .service_name("test-service")
                .service_version("0.1.0")
                .environment("test"),
        )
        .unwrap();

        let (span_tid, span_sid) = in_span("airbug.test", "demo-span", || {
            set_attribute("k", "v");
            log_info("airbug.test", "hello inside span");
            emit_log_with_attrs(
                "airbug.test",
                Severity::Warn,
                "checkout.retry",
                "retrying payment",
                &[KeyValue::new("attempt", 2i64)],
            );
            (current_trace_id().unwrap(), current_span_id().unwrap())
        });

        handle.force_flush().unwrap();
        let spans = exporters.spans.get_finished_spans().unwrap();
        assert!(
            spans.iter().any(|span| {
                span.name == "demo-span"
                    && span.span_context.trace_id().to_string() == span_tid
                    && span
                        .attributes
                        .iter()
                        .any(|attribute| attribute.key.as_str() == "k")
            }),
            "expected named span with attribute"
        );
        let logs = exporters.logs.get_emitted_logs().unwrap();
        assert!(
            logs.iter().any(|log| {
                log.record.severity_number() == Some(Severity::Info)
                    && log
                        .record
                        .trace_context()
                        .map(|tc| {
                            tc.trace_id.to_string() == span_tid
                                && tc.span_id.to_string() == span_sid
                        })
                        .unwrap_or(false)
            }),
            "expected info log automatically correlated with active span"
        );
        assert!(
            logs.iter()
                .any(|log| log.record.event_name() == Some("checkout.retry")),
            "expected structured log with event_name"
        );
        drop(handle);
        let (handle2, _) = install_test(TelemetryConfig::new().service_name("test2")).unwrap();
        drop(handle2);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn async_spans_links_exceptions_and_w3c_propagation() {
        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (handle, exporters) =
            install_test(TelemetryConfig::new().service_name("async-svc")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let (wire_headers, extracted_tenant) = rt.block_on(async {
            let mut wire_headers = HashMap::new();
            let producer_tp = in_span_async("airbug.test", "producer", async {
                tokio::task::yield_now().await;
                with_baggage(&[KeyValue::new("tenant.id", "acme")], || {
                    inject_map(&mut wire_headers);
                });
                current_traceparent().unwrap()
            })
            .await;

            // Normalize header case to uppercase to verify case-insensitive extraction.
            let mut upper_headers = HashMap::new();
            for (k, v) in &wire_headers {
                upper_headers.insert(k.to_ascii_uppercase(), v.clone());
            }

            let extracted_tenant = with_extracted_headers_async(&upper_headers, async {
                in_span_async_with_options(
                    "airbug.test",
                    "consumer",
                    SpanOptions::new()
                        .kind(SpanKind::Consumer)
                        .attribute("messaging.system", "queue")
                        .link(
                            SpanLink::from_traceparent(
                                &producer_tp,
                                vec![KeyValue::new("link.reason", "batch")],
                            )
                            .unwrap(),
                        ),
                    async {
                        tokio::task::yield_now().await;
                        add_span_event("dequeued", &[KeyValue::new("items", 4i64)]);
                        let io_err = std::io::Error::other("downstream timeout");
                        record_exception(&io_err);
                        baggage_value("tenant.id")
                    },
                )
                .await
            })
            .await;

            (wire_headers, extracted_tenant)
        });

        assert!(wire_headers.contains_key("traceparent"));
        assert!(wire_headers.contains_key("baggage"));

        assert_eq!(extracted_tenant.as_deref(), Some("acme"));
        handle.force_flush().unwrap();

        let spans = exporters.spans.get_finished_spans().unwrap();
        let producer = spans.iter().find(|s| s.name == "producer").unwrap();
        let consumer = spans.iter().find(|s| s.name == "consumer").unwrap();
        assert_eq!(
            consumer.span_context.trace_id(),
            producer.span_context.trace_id()
        );
        assert_eq!(consumer.parent_span_id, producer.span_context.span_id());
        assert_eq!(consumer.span_kind, SpanKind::Consumer);
        assert_eq!(consumer.links.links.len(), 1);
        assert!(consumer.events.events.iter().any(|e| e.name == "exception"));
        drop(handle);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn all_seven_metric_instruments_and_custom_histogram_views() {
        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (handle, exporters) = install_test(
            TelemetryConfig::new()
                .service_name("metrics-svc")
                .histogram_buckets("custom_view_ms", vec![2.0, 10.0, 50.0]),
        )
        .unwrap();

        add_counter("airbug.test", "hits", 3, &[]);
        add_f64_counter("airbug.test", "cpu_seconds", 1.5, &[]);
        add_up_down_counter("airbug.test", "inflight", 2, &[]);
        F64UpDownCounter::new("airbug.test", "balance_delta").add(-4.5, &[]);
        record_histogram("airbug.test", "custom_view_ms", 7.5, &[]);
        U64Histogram::with_boundaries("airbug.test", "payload_bytes", vec![64.0, 512.0, 4096.0])
            .record(256, &[]);
        F64Gauge::with_description("airbug.test", "temperature", "Current temperature")
            .record(21.5, &[]);
        U64Gauge::new("airbug.test", "queue_depth").record(4, &[]);
        I64Gauge::new("airbug.test", "clock_offset").record(-3, &[]);

        let _obs_gauge = ObservableF64Gauge::new("airbug.test", "rss_mb", |obs| {
            obs.observe(128.5, &[KeyValue::new("pid", 1i64)]);
        });
        let _obs_counter = ObservableU64Counter::new("airbug.test", "ctx_switches", |obs| {
            obs.observe(42, &[]);
        });
        let _obs_updown = ObservableI64UpDownCounter::new("airbug.test", "open_fds", |obs| {
            obs.observe(19, &[]);
        });

        handle.force_flush().unwrap();
        let metrics = exporters.metrics.get_finished_metrics().unwrap();
        let names: Vec<_> = metrics
            .iter()
            .flat_map(|resource| resource.scope_metrics())
            .flat_map(|scope| scope.metrics())
            .map(|metric| metric.name().to_string())
            .collect();
        for expected in [
            "hits",
            "cpu_seconds",
            "inflight",
            "balance_delta",
            "custom_view_ms",
            "payload_bytes",
            "temperature",
            "queue_depth",
            "clock_offset",
            "rss_mb",
            "ctx_switches",
            "open_fds",
        ] {
            assert!(
                names.iter().any(|n| n == expected),
                "missing metric {expected:?} in {names:?}"
            );
        }
        drop(handle);
    }

    #[cfg(all(feature = "testing", feature = "tracing-bridge"))]
    #[test]
    fn tracing_otel_layer_bridges_spans_events_logs_and_airbug_err() {
        use tracing_subscriber::layer::SubscriberExt;

        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (handle, exporters) =
            install_test(TelemetryConfig::new().service_name("tracing-bridge-test")).unwrap();

        let err_transport = airbug_err::MemoryTransport::new();
        let _err_guard = airbug_err::init_with_transport(
            airbug_err::Options::new().endpoint("unused"),
            Arc::new(err_transport.clone()),
        )
        .unwrap();

        let subscriber = tracing_subscriber::registry().with(tracing_layer());
        let observed_tid = tracing::subscriber::with_default(subscriber, || {
            let outer = tracing::info_span!(
                "http_request",
                http.method = "POST",
                status = tracing::field::Empty
            );
            let _enter = outer.enter();
            outer.record("status", 500i64);
            tracing::info!(user_id = 7i64, "processing order");
            let tid =
                current_trace_id().expect("TracingOtelLayer must attach OTel Context on enter");
            let app_err = std::io::Error::other("inventory exhausted");
            airbug_err::capture_error(&app_err);
            tracing::error!("order failed");
            tid
        });

        handle.force_flush().unwrap();

        let err_event = err_transport.last().expect("airbug-err captured event");
        assert_eq!(err_event.trace_id.as_deref(), Some(observed_tid.as_str()));
        assert!(err_event.span_id.is_some());

        let spans = exporters.spans.get_finished_spans().unwrap();
        let req_span = spans
            .iter()
            .find(|s| s.name == "http_request")
            .expect("bridged http_request span");
        assert_eq!(req_span.span_context.trace_id().to_string(), observed_tid);
        assert!(
            req_span
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == "status" && kv.value == Value::I64(500))
        );
        assert!(matches!(
            req_span.status,
            opentelemetry::trace::Status::Error { .. }
        ));

        let logs = exporters.logs.get_emitted_logs().unwrap();
        assert!(
            logs.iter().any(|l| {
                l.record
                    .trace_context()
                    .map(|tc| tc.trace_id.to_string() == observed_tid)
                    .unwrap_or(false)
            }),
            "tracing::info!/error! must emit OTel LogRecords correlated with the span"
        );
        drop(handle);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn profiles_builder_deduplicates_dictionary_and_exports_in_memory() {
        use prost::Message;

        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (handle, exporters) =
            install_test(TelemetryConfig::new().service_name("profiles-test")).unwrap();

        let batch = in_span("airbug.test", "profiled-op", || {
            let s1 = ProfileSample::new(
                vec![
                    ProfileFrame::new("main", "src/main.rs", 10),
                    ProfileFrame::new("compute", "src/lib.rs", 42),
                ],
                2048,
            )
            .with_current_span()
            .with_attribute("thread.name", "worker-1");

            let s2 = ProfileSample::new(
                vec![
                    ProfileFrame::new("main", "src/main.rs", 10),
                    ProfileFrame::new("parse", "src/lib.rs", 88),
                ],
                1024,
            )
            .with_current_span();

            ProfileBatch::new("profiles-test", "allocations", "bytes")
                .duration_nanos(5_000_000)
                .run_id("run-abc")
                .add_sample(s1)
                .add_sample(s2)
        });

        let proto = batch.to_proto();
        assert_eq!(proto.string_table[0], "");
        // "main" and "src/main.rs" must appear only once in string_table & function_table!
        assert_eq!(
            proto
                .string_table
                .iter()
                .filter(|s| s.as_str() == "main")
                .count(),
            1
        );
        assert_eq!(proto.function_table.len(), 3);
        assert_eq!(proto.link_table.len(), 1);

        let wire_pb = encode_profiles_protobuf(&proto);
        let decoded_pb = profiles::OtlpProfilesData::decode(wire_pb.as_slice()).unwrap();
        assert_eq!(decoded_pb, proto);

        let wire_json = encode_profiles_json(&proto).unwrap();
        let decoded_json: profiles::OtlpProfilesData = serde_json::from_slice(&wire_json).unwrap();
        assert_eq!(decoded_json.function_table.len(), 3);

        export_profiles_http("memory://test", &proto, Protocol::Http, false).unwrap();
        assert_eq!(exporters.profiles.len(), 1);
        drop(handle);
    }

    #[cfg(feature = "otlp-grpc")]
    #[tokio::test]
    async fn install_grpc_inside_tokio_runtime_does_not_panic() {
        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let guard = install_grpc(
            &TelemetryConfig::new()
                .service_name("grpc-test")
                .endpoint("http://127.0.0.1:4317")
                .compression(Compression::Gzip)
                .header("x-airbug-test", "1"),
        )
        .unwrap();
        drop(guard);
    }

    #[test]
    fn install_grpc_outside_tokio_dropped_inside_tokio_does_not_panic() {
        #[cfg(feature = "otlp-grpc")]
        {
            let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let guard = install_grpc(
                &TelemetryConfig::new()
                    .service_name("grpc-drop-test")
                    .endpoint("http://127.0.0.1:4317"),
            )
            .unwrap();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                drop(guard);
            });
        }
    }

    #[cfg(feature = "otlp-http")]
    #[test]
    fn http_signal_endpoint_normalizes_base_and_explicit_paths() {
        assert_eq!(
            http_signal_endpoint("http://127.0.0.1:4318", "/v1/traces"),
            "http://127.0.0.1:4318/v1/traces"
        );
        assert_eq!(
            http_signal_endpoint("http://127.0.0.1:4318/", "/v1/metrics"),
            "http://127.0.0.1:4318/v1/metrics"
        );
        assert_eq!(
            http_signal_endpoint("http://127.0.0.1:4318/v1/logs", "/v1/logs"),
            "http://127.0.0.1:4318/v1/logs"
        );
    }

    #[cfg(feature = "otlp-http")]
    #[tokio::test]
    async fn install_http_json_and_gzip_inside_tokio_does_not_panic() {
        let _lock = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let guard = install_http(
            &TelemetryConfig::new()
                .service_name("http-json-gzip-test")
                .endpoint("http://127.0.0.1:4318")
                .compression(Compression::Gzip)
                .header("x-airbug-test", "json"),
            Protocol::HttpJson,
        )
        .unwrap();
        drop(guard);
    }
}
