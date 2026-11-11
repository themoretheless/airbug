//! Native OTLP Receiver (HTTP `:8790` / `:4318` + Tonic gRPC `:4317`),
//! thread-safe NDJSON storage with 32 MiB file rotation, and unified `EventEnvelope` emission.
use super::{
    attrs_to_map, decode_id_hex, normalize_severity, parse_f64, resource_attr, resource_attrs,
};
use crate::{
    app::HubApp,
    config::{self, MAX_OTLP_DECOMPRESSED_BYTES, OTLP_MAX_BACKUPS, OTLP_ROTATE_BYTES, RootPaths},
    error::HubError,
    event_model::{EVENT_MODEL_VERSION, EventEnvelope, EventPayload},
    http::{self, Request},
};
use flate2::read::GzDecoder;
use opentelemetry_proto::tonic::{
    collector::{
        logs::v1::{
            ExportLogsServiceRequest, ExportLogsServiceResponse,
            logs_service_server::{LogsService, LogsServiceServer},
        },
        metrics::v1::{
            ExportMetricsServiceRequest, ExportMetricsServiceResponse,
            metrics_service_server::{MetricsService, MetricsServiceServer},
        },
        trace::v1::{
            ExportTraceServiceRequest, ExportTraceServiceResponse,
            trace_service_server::{TraceService, TraceServiceServer},
        },
    },
    profiles::v1::ProfilesData,
};
use prost::Message;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tonic::{
    body::Body as TonicBody,
    codegen::{BoxFuture, CompressionEncoding, Context, Poll, Service, async_trait, http as thttp},
    transport::{Server as TonicServer, server::TcpIncoming},
};

static ENVELOPE_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtlpSignal {
    Traces,
    Metrics,
    Logs,
    Profiles,
}

impl OtlpSignal {
    pub fn from_http_path(path: &str) -> Option<Self> {
        let stripped = path
            .strip_prefix("/api/v1/otlp")
            .or_else(|| path.strip_prefix("/api/otlp"))
            .unwrap_or(path);
        match stripped {
            "/v1/traces" => Some(Self::Traces),
            "/v1/metrics" => Some(Self::Metrics),
            "/v1/logs" => Some(Self::Logs),
            "/v1development/profiles" | "/v1/profiles" => Some(Self::Profiles),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireFormat {
    Protobuf,
    Json,
}

impl WireFormat {
    fn content_type(self) -> &'static str {
        match self {
            Self::Protobuf => "application/x-protobuf",
            Self::Json => "application/json; charset=utf-8",
        }
    }
}

/// Thread-safe NDJSON storage writer with size-based file rotation (`32 MiB`, `3` backups).
pub struct OtlpStore {
    traces_lock: Mutex<()>,
    metrics_lock: Mutex<()>,
    logs_lock: Mutex<()>,
    profiles_lock: Mutex<()>,
    rotate_bytes: u64,
    max_backups: usize,
}

impl Default for OtlpStore {
    fn default() -> Self {
        Self::new()
    }
}

impl OtlpStore {
    pub fn new() -> Self {
        Self::with_rotation(OTLP_ROTATE_BYTES, OTLP_MAX_BACKUPS)
    }

    pub fn with_rotation(rotate_bytes: u64, max_backups: usize) -> Self {
        Self {
            traces_lock: Mutex::new(()),
            metrics_lock: Mutex::new(()),
            logs_lock: Mutex::new(()),
            profiles_lock: Mutex::new(()),
            rotate_bytes: rotate_bytes.max(512),
            max_backups: max_backups.max(1),
        }
    }

    pub fn append_ndjson_line(
        &self,
        signal: OtlpSignal,
        paths: &RootPaths,
        line: &str,
    ) -> io::Result<()> {
        let (lock, file_path) = match signal {
            OtlpSignal::Traces => (&self.traces_lock, paths.traces_file()),
            OtlpSignal::Metrics => (&self.metrics_lock, paths.metrics_file()),
            OtlpSignal::Logs => (&self.logs_lock, paths.logs_file()),
            OtlpSignal::Profiles => (&self.profiles_lock, paths.profiles_file()),
        };
        let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let incoming_len = (line.len() + 1) as u64;
        if let Ok(meta) = fs::metadata(&file_path)
            && meta.len() > 0
            && meta.len().saturating_add(incoming_len) > self.rotate_bytes
        {
            rotate_signal_file(&file_path, self.max_backups)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}

fn rotate_signal_file(path: &Path, max_backups: usize) -> io::Result<()> {
    if max_backups == 0 || !path.is_file() {
        return Ok(());
    }
    let oldest = rotated_backup_path(path, max_backups);
    if oldest.exists() {
        let _ = fs::remove_file(&oldest);
    }
    for idx in (1..max_backups).rev() {
        let src = rotated_backup_path(path, idx);
        if src.exists() {
            let dst = rotated_backup_path(path, idx + 1);
            let _ = fs::rename(&src, &dst);
        }
    }
    let first = rotated_backup_path(path, 1);
    fs::rename(path, first)
}

pub(crate) fn rotated_backup_path(path: &Path, index: usize) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("signal");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("json");
    parent.join(format!("{stem}-{index}.{ext}"))
}

/// Handle an OTLP HTTP `POST` request (`/v1/{traces,metrics,logs}` or `/v1development/profiles`).
pub fn handle_http_otlp(
    stream: &mut TcpStream,
    app: &HubApp,
    req: &Request,
    signal: OtlpSignal,
) -> io::Result<()> {
    let raw_bytes = match decode_http_body(req) {
        Ok(bytes) => bytes,
        Err((status, msg)) => {
            return http::respond_err(stream, status, &HubError::msg(msg));
        }
    };

    let format = match detect_wire_format(req.header("content-type"), &raw_bytes) {
        Ok(f) => f,
        Err(msg) => {
            return http::respond_err(stream, "415 Unsupported Media Type", &HubError::msg(msg));
        }
    };

    match ingest_otlp_payload(app, signal, format, &raw_bytes) {
        Ok(resp_bytes) => http::respond_bytes(stream, "200 OK", format.content_type(), &resp_bytes),
        Err(IngestError::BadRequest(msg)) => {
            http::respond_err(stream, "400 Bad Request", &HubError::msg(msg))
        }
        Err(IngestError::Io(err)) => {
            http::respond_err(stream, "500 Internal Server Error", &HubError::from(err))
        }
    }
}

enum IngestError {
    BadRequest(String),
    Io(io::Error),
}

fn decode_http_body(req: &Request) -> Result<Vec<u8>, (&'static str, String)> {
    let encoding = req
        .header("content-encoding")
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();

    if encoding.is_empty() || encoding == "identity" {
        if req.body.len() > MAX_OTLP_DECOMPRESSED_BYTES {
            return Err((
                "413 Payload Too Large",
                format!("OTLP body exceeds {MAX_OTLP_DECOMPRESSED_BYTES} bytes"),
            ));
        }
        return Ok(req.body.clone());
    }

    if encoding == "gzip" || encoding == "x-gzip" {
        let decoder = GzDecoder::new(req.body.as_slice());
        let mut limited = decoder.take((MAX_OTLP_DECOMPRESSED_BYTES + 1) as u64);
        let mut out = Vec::new();
        limited
            .read_to_end(&mut out)
            .map_err(|e| ("400 Bad Request", format!("invalid gzip OTLP payload: {e}")))?;
        if out.len() > MAX_OTLP_DECOMPRESSED_BYTES {
            return Err((
                "413 Payload Too Large",
                format!("decompressed OTLP payload exceeds {MAX_OTLP_DECOMPRESSED_BYTES} bytes"),
            ));
        }
        return Ok(out);
    }

    Err((
        "415 Unsupported Media Type",
        format!("unsupported Content-Encoding: {encoding}"),
    ))
}

fn detect_wire_format(content_type: Option<&str>, body: &[u8]) -> Result<WireFormat, String> {
    let Some(ct) = content_type else {
        let trimmed = body.iter().copied().find(|b| !b.is_ascii_whitespace());
        return Ok(if trimmed == Some(b'{') {
            WireFormat::Json
        } else {
            WireFormat::Protobuf
        });
    };

    let media = ct
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match media.as_str() {
        "" => Ok(
            if body.iter().copied().find(|b| !b.is_ascii_whitespace()) == Some(b'{') {
                WireFormat::Json
            } else {
                WireFormat::Protobuf
            },
        ),
        "application/x-protobuf" | "application/protobuf" | "application/octet-stream" => {
            Ok(WireFormat::Protobuf)
        }
        "application/json" => Ok(WireFormat::Json),
        other if other.ends_with("+json") => Ok(WireFormat::Json),
        other if other.ends_with("+proto") => Ok(WireFormat::Protobuf),
        other => Err(format!("unsupported OTLP Content-Type: {other}")),
    }
}

fn ingest_otlp_payload(
    app: &HubApp,
    signal: OtlpSignal,
    format: WireFormat,
    bytes: &[u8],
) -> Result<Vec<u8>, IngestError> {
    match signal {
        OtlpSignal::Traces => {
            let val = match format {
                WireFormat::Protobuf => {
                    let req = ExportTraceServiceRequest::decode(bytes).map_err(|e| {
                        IngestError::BadRequest(format!(
                            "invalid protobuf ExportTraceServiceRequest: {e}"
                        ))
                    })?;
                    serde_json::to_value(&req)
                        .map_err(|e| IngestError::BadRequest(e.to_string()))?
                }
                WireFormat::Json => {
                    decode_json_request(bytes, &["resourceSpans", "resource_spans"])?
                }
            };
            persist_and_correlate(app, OtlpSignal::Traces, &val)?;
            let resp = ExportTraceServiceResponse::default();
            Ok(encode_response(&resp, format))
        }
        OtlpSignal::Metrics => {
            let val = match format {
                WireFormat::Protobuf => {
                    let req = ExportMetricsServiceRequest::decode(bytes).map_err(|e| {
                        IngestError::BadRequest(format!(
                            "invalid protobuf ExportMetricsServiceRequest: {e}"
                        ))
                    })?;
                    serde_json::to_value(&req)
                        .map_err(|e| IngestError::BadRequest(e.to_string()))?
                }
                WireFormat::Json => {
                    decode_json_request(bytes, &["resourceMetrics", "resource_metrics"])?
                }
            };
            persist_and_correlate(app, OtlpSignal::Metrics, &val)?;
            let resp = ExportMetricsServiceResponse::default();
            Ok(encode_response(&resp, format))
        }
        OtlpSignal::Logs => {
            let val = match format {
                WireFormat::Protobuf => {
                    let req = ExportLogsServiceRequest::decode(bytes).map_err(|e| {
                        IngestError::BadRequest(format!(
                            "invalid protobuf ExportLogsServiceRequest: {e}"
                        ))
                    })?;
                    serde_json::to_value(&req)
                        .map_err(|e| IngestError::BadRequest(e.to_string()))?
                }
                WireFormat::Json => decode_json_request(bytes, &["resourceLogs", "resource_logs"])?,
            };
            persist_and_correlate(app, OtlpSignal::Logs, &val)?;
            let resp = ExportLogsServiceResponse::default();
            Ok(encode_response(&resp, format))
        }
        OtlpSignal::Profiles => {
            let val = match format {
                WireFormat::Protobuf => {
                    let req = ProfilesData::decode(bytes).map_err(|e| {
                        IngestError::BadRequest(format!("invalid protobuf ProfilesData: {e}"))
                    })?;
                    serde_json::to_value(&req)
                        .map_err(|e| IngestError::BadRequest(e.to_string()))?
                }
                WireFormat::Json => {
                    decode_json_request(bytes, &["resourceProfiles", "resource_profiles"])?
                }
            };
            persist_and_correlate(app, OtlpSignal::Profiles, &val)?;
            Ok(match format {
                WireFormat::Protobuf => Vec::new(),
                WireFormat::Json => b"{}".to_vec(),
            })
        }
    }
}

fn decode_json_request(bytes: &[u8], allowed_root_keys: &[&str]) -> Result<Value, IngestError> {
    let val: Value = serde_json::from_slice(bytes)
        .map_err(|e| IngestError::BadRequest(format!("invalid OTLP JSON: {e}")))?;
    let Some(obj) = val.as_object() else {
        return Err(IngestError::BadRequest(
            "OTLP JSON payload must be a JSON object".into(),
        ));
    };
    if !obj.is_empty() && !allowed_root_keys.iter().any(|k| obj.contains_key(*k)) {
        return Err(IngestError::BadRequest(format!(
            "missing expected OTLP root field (expected one of {allowed_root_keys:?})"
        )));
    }
    Ok(val)
}

fn encode_response<M: Message + serde::Serialize>(msg: &M, format: WireFormat) -> Vec<u8> {
    match format {
        WireFormat::Protobuf => msg.encode_to_vec(),
        WireFormat::Json => serde_json::to_vec(msg).unwrap_or_else(|_| b"{}".to_vec()),
    }
}

fn persist_and_correlate(app: &HubApp, signal: OtlpSignal, val: &Value) -> Result<(), IngestError> {
    let compact = serde_json::to_string(val).map_err(|e| IngestError::BadRequest(e.to_string()))?;
    app.otlp_store
        .append_ndjson_line(signal, &app.paths, &compact)
        .map_err(IngestError::Io)?;

    let envelopes = build_envelopes(signal, val, 64);
    if !envelopes.is_empty() {
        let _ = app.append_event_envelopes(&envelopes);
    }
    Ok(())
}

fn next_envelope_id(prefix: &str) -> String {
    let seq = ENVELOPE_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{seq}")
}

fn resource_meta(
    res_container: &Value,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    BTreeMap<String, String>,
) {
    let r_attrs = resource_attrs(res_container);
    let svc = resource_attr(res_container, "service.name");
    let service = (!svc.trim().is_empty()).then_some(svc);
    let env = r_attrs
        .get("deployment.environment.name")
        .or_else(|| r_attrs.get("deployment.environment"))
        .or_else(|| r_attrs.get("environment"))
        .cloned()
        .filter(|s| !s.is_empty());
    let release = r_attrs
        .get("service.version")
        .or_else(|| r_attrs.get("release"))
        .cloned()
        .filter(|s| !s.is_empty());
    let mut tags = BTreeMap::new();
    for (k, v) in r_attrs {
        if k != "service.name" {
            tags.insert(k, v);
        }
    }
    (service, env, release, tags)
}

fn build_envelopes(signal: OtlpSignal, val: &Value, cap: usize) -> Vec<EventEnvelope> {
    let mut out = Vec::new();
    let now = crate::issues::now_iso();

    match signal {
        OtlpSignal::Traces => {
            let Some(rss) = val
                .get("resourceSpans")
                .or_else(|| val.get("resource_spans"))
                .and_then(Value::as_array)
            else {
                return out;
            };
            for rs in rss {
                let (service, environment, release, base_tags) = resource_meta(rs);
                let Some(sss) = rs
                    .get("scopeSpans")
                    .or_else(|| rs.get("scope_spans"))
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                for ss in sss {
                    let Some(spans) = ss.get("spans").and_then(Value::as_array) else {
                        continue;
                    };
                    for sp in spans {
                        if out.len() >= cap {
                            return out;
                        }
                        let Some(trace_id) = sp
                            .get("traceId")
                            .or_else(|| sp.get("trace_id"))
                            .and_then(|v| decode_id_hex(v, 16))
                        else {
                            continue;
                        };
                        let Some(span_id) = sp
                            .get("spanId")
                            .or_else(|| sp.get("span_id"))
                            .and_then(|v| decode_id_hex(v, 8))
                        else {
                            continue;
                        };
                        let parent_span_id = sp
                            .get("parentSpanId")
                            .or_else(|| sp.get("parent_span_id"))
                            .and_then(|v| decode_id_hex(v, 8));
                        let name = sp
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("span")
                            .to_string();
                        let s_attrs = attrs_to_map(sp.get("attributes"));
                        let mut tags = base_tags.clone();
                        for (k, v) in &s_attrs {
                            tags.insert(k.clone(), v.clone());
                        }
                        let attrs_json = serde_json::to_value(&s_attrs)
                            .unwrap_or_else(|_| serde_json::json!({}));
                        out.push(EventEnvelope {
                            model_version: EVENT_MODEL_VERSION,
                            event_id: next_envelope_id("otlp-span"),
                            timestamp: now.clone(),
                            service: service.clone(),
                            environment: environment.clone(),
                            release: release.clone(),
                            trace_id: Some(trace_id.clone()),
                            span_id: Some(span_id.clone()),
                            tags,
                            payload: EventPayload::Trace {
                                trace_id,
                                span_id,
                                parent_span_id,
                                name,
                                attributes: attrs_json,
                            },
                        });
                    }
                }
            }
        }
        OtlpSignal::Metrics => {
            let Some(rms) = val
                .get("resourceMetrics")
                .or_else(|| val.get("resource_metrics"))
                .and_then(Value::as_array)
            else {
                return out;
            };
            for rm in rms {
                let (service, environment, release, base_tags) = resource_meta(rm);
                let Some(sms) = rm
                    .get("scopeMetrics")
                    .or_else(|| rm.get("scope_metrics"))
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                for sm in sms {
                    let Some(metrics) = sm.get("metrics").and_then(Value::as_array) else {
                        continue;
                    };
                    for m in metrics {
                        if out.len() >= cap {
                            return out;
                        }
                        let name = m
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("metric")
                            .to_string();
                        let data_c = m.get("data").unwrap_or(m);
                        for (kind, key) in [
                            ("gauge", "gauge"),
                            ("sum", "sum"),
                            ("histogram", "histogram"),
                            ("exponential_histogram", "exponentialHistogram"),
                            ("summary", "summary"),
                        ] {
                            let Some(block) = data_c.get(key).or_else(|| {
                                (key == "exponentialHistogram")
                                    .then(|| data_c.get("exponential_histogram"))
                                    .flatten()
                            }) else {
                                continue;
                            };
                            let Some(dps) = block
                                .get("dataPoints")
                                .or_else(|| block.get("data_points"))
                                .and_then(Value::as_array)
                            else {
                                continue;
                            };
                            for dp in dps {
                                if out.len() >= cap {
                                    return out;
                                }
                                let value = dp
                                    .get("asDouble")
                                    .or_else(|| dp.get("as_double"))
                                    .and_then(parse_f64)
                                    .or_else(|| {
                                        dp.get("asInt")
                                            .or_else(|| dp.get("as_int"))
                                            .and_then(parse_f64)
                                    })
                                    .or_else(|| dp.get("sum").and_then(parse_f64))
                                    .or_else(|| dp.get("count").and_then(parse_f64))
                                    .unwrap_or(0.0);
                                let dp_attrs = attrs_to_map(dp.get("attributes"));
                                let mut tags = base_tags.clone();
                                for (k, v) in &dp_attrs {
                                    tags.insert(k.clone(), v.clone());
                                }
                                out.push(EventEnvelope {
                                    model_version: EVENT_MODEL_VERSION,
                                    event_id: next_envelope_id("otlp-metric"),
                                    timestamp: now.clone(),
                                    service: service.clone(),
                                    environment: environment.clone(),
                                    release: release.clone(),
                                    trace_id: None,
                                    span_id: None,
                                    tags,
                                    payload: EventPayload::Metric {
                                        name: name.clone(),
                                        value,
                                        kind: kind.to_string(),
                                        attributes: serde_json::to_value(&dp_attrs)
                                            .unwrap_or_else(|_| serde_json::json!({})),
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }
        OtlpSignal::Logs => {
            let Some(rls) = val
                .get("resourceLogs")
                .or_else(|| val.get("resource_logs"))
                .and_then(Value::as_array)
            else {
                return out;
            };
            for rl in rls {
                let (service, environment, release, base_tags) = resource_meta(rl);
                let Some(sls) = rl
                    .get("scopeLogs")
                    .or_else(|| rl.get("scope_logs"))
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                for sl in sls {
                    let Some(recs) = sl
                        .get("logRecords")
                        .or_else(|| sl.get("log_records"))
                        .and_then(Value::as_array)
                    else {
                        continue;
                    };
                    for rec in recs {
                        if out.len() >= cap {
                            return out;
                        }
                        let trace_id = rec
                            .get("traceId")
                            .or_else(|| rec.get("trace_id"))
                            .and_then(|v| decode_id_hex(v, 16));
                        let span_id = rec
                            .get("spanId")
                            .or_else(|| rec.get("span_id"))
                            .and_then(|v| decode_id_hex(v, 8));
                        let body = rec.get("body").map(super::any_value).unwrap_or_default();
                        let sev_raw = rec
                            .get("severityText")
                            .or_else(|| rec.get("severity_text"))
                            .and_then(Value::as_str)
                            .unwrap_or("INFO");
                        let r_attrs = attrs_to_map(rec.get("attributes"));
                        let mut tags = base_tags.clone();
                        for (k, v) in &r_attrs {
                            tags.insert(k.clone(), v.clone());
                        }
                        out.push(EventEnvelope {
                            model_version: EVENT_MODEL_VERSION,
                            event_id: next_envelope_id("otlp-log"),
                            timestamp: now.clone(),
                            service: service.clone(),
                            environment: environment.clone(),
                            release: release.clone(),
                            trace_id,
                            span_id,
                            tags,
                            payload: EventPayload::Log {
                                body,
                                severity: normalize_severity(sev_raw).to_string(),
                                attributes: serde_json::to_value(&r_attrs)
                                    .unwrap_or_else(|_| serde_json::json!({})),
                            },
                        });
                    }
                }
            }
        }
        OtlpSignal::Profiles => {}
    }
    out
}

// ---------------------------------------------------------------------------
// Tonic gRPC OTLP Services + Service Multiplexer (`127.0.0.1:4317`)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct NativeGrpcExporter {
    app: Arc<HubApp>,
}

#[async_trait]
impl TraceService for NativeGrpcExporter {
    async fn export(
        &self,
        request: tonic::Request<ExportTraceServiceRequest>,
    ) -> Result<tonic::Response<ExportTraceServiceResponse>, tonic::Status> {
        let msg = request.into_inner();
        let val = serde_json::to_value(&msg)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        persist_and_correlate(&self.app, OtlpSignal::Traces, &val)
            .map_err(grpc_status_from_ingest)?;
        Ok(tonic::Response::new(ExportTraceServiceResponse::default()))
    }
}

#[async_trait]
impl MetricsService for NativeGrpcExporter {
    async fn export(
        &self,
        request: tonic::Request<ExportMetricsServiceRequest>,
    ) -> Result<tonic::Response<ExportMetricsServiceResponse>, tonic::Status> {
        let msg = request.into_inner();
        let val = serde_json::to_value(&msg)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        persist_and_correlate(&self.app, OtlpSignal::Metrics, &val)
            .map_err(grpc_status_from_ingest)?;
        Ok(tonic::Response::new(ExportMetricsServiceResponse::default()))
    }
}

#[async_trait]
impl LogsService for NativeGrpcExporter {
    async fn export(
        &self,
        request: tonic::Request<ExportLogsServiceRequest>,
    ) -> Result<tonic::Response<ExportLogsServiceResponse>, tonic::Status> {
        let msg = request.into_inner();
        let val = serde_json::to_value(&msg)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        persist_and_correlate(&self.app, OtlpSignal::Logs, &val)
            .map_err(grpc_status_from_ingest)?;
        Ok(tonic::Response::new(ExportLogsServiceResponse::default()))
    }
}

fn grpc_status_from_ingest(err: IngestError) -> tonic::Status {
    match err {
        IngestError::BadRequest(m) => tonic::Status::invalid_argument(m),
        IngestError::Io(e) => tonic::Status::internal(e.to_string()),
    }
}

#[derive(Clone)]
struct OtlpGrpcMux {
    trace: TraceServiceServer<NativeGrpcExporter>,
    metrics: MetricsServiceServer<NativeGrpcExporter>,
    logs: LogsServiceServer<NativeGrpcExporter>,
}

impl OtlpGrpcMux {
    fn new(app: Arc<HubApp>) -> Self {
        let svc = NativeGrpcExporter { app };
        let trace = TraceServiceServer::new(svc.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(config::MAX_OTLP_DECOMPRESSED_BYTES);
        let metrics = MetricsServiceServer::new(svc.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(config::MAX_OTLP_DECOMPRESSED_BYTES);
        let logs = LogsServiceServer::new(svc)
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(config::MAX_OTLP_DECOMPRESSED_BYTES);
        Self {
            trace,
            metrics,
            logs,
        }
    }
}

impl Service<thttp::Request<TonicBody>> for OtlpGrpcMux {
    type Response = thttp::Response<TonicBody>;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: thttp::Request<TonicBody>) -> Self::Future {
        let path = req.uri().path();
        if path.starts_with("/opentelemetry.proto.collector.metrics.v1.MetricsService/") {
            self.metrics.call(req)
        } else if path.starts_with("/opentelemetry.proto.collector.logs.v1.LogsService/") {
            self.logs.call(req)
        } else {
            // Handles TraceService and emits standard gRPC UNIMPLEMENTED for unknown paths.
            self.trace.call(req)
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in Collector Handle (HTTP `127.0.0.1:4318` + Tonic gRPC `127.0.0.1:4317`)
// ---------------------------------------------------------------------------

pub struct BuiltinCollector {
    http_addr: SocketAddr,
    grpc_addr: SocketAddr,
    http_stop: Arc<AtomicBool>,
    http_thread: Option<JoinHandle<()>>,
    grpc_shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    grpc_thread: Option<JoinHandle<()>>,
}

impl BuiltinCollector {
    pub fn start(app: Arc<HubApp>, http_port: u16, grpc_port: u16) -> io::Result<Self> {
        let http_listener = TcpListener::bind(("127.0.0.1", http_port))?;
        let http_addr = http_listener.local_addr()?;

        let grpc_std_listener = TcpListener::bind(("127.0.0.1", grpc_port))?;
        grpc_std_listener.set_nonblocking(true)?;
        let grpc_addr = grpc_std_listener.local_addr()?;

        let http_stop = Arc::new(AtomicBool::new(false));
        let http_stop_bg = Arc::clone(&http_stop);
        let app_http = Arc::clone(&app);

        let http_thread = thread::Builder::new()
            .name("airbug-otlp-http".into())
            .spawn(move || run_builtin_http(http_listener, app_http, http_stop_bg))?;

        let (grpc_tx, grpc_rx) = tokio::sync::oneshot::channel::<()>();
        let app_grpc = Arc::clone(&app);
        let grpc_thread = thread::Builder::new()
            .name("airbug-otlp-grpc".into())
            .spawn(move || {
                let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                else {
                    return;
                };
                rt.block_on(async move {
                    let Ok(tokio_listener) = tokio::net::TcpListener::from_std(grpc_std_listener)
                    else {
                        return;
                    };
                    let incoming = TcpIncoming::from(tokio_listener).with_nodelay(Some(true));
                    let mux = OtlpGrpcMux::new(app_grpc);
                    let server = TonicServer::builder().serve_with_incoming(mux, incoming);
                    tokio::pin!(server);
                    tokio::select! {
                        _ = &mut server => {}
                        _ = grpc_rx => {}
                    }
                });
            })?;

        Ok(Self {
            http_addr,
            grpc_addr,
            http_stop,
            http_thread: Some(http_thread),
            grpc_shutdown: Some(grpc_tx),
            grpc_thread: Some(grpc_thread),
        })
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn grpc_addr(&self) -> SocketAddr {
        self.grpc_addr
    }

    pub fn stop(&mut self) {
        self.http_stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect_timeout(&self.http_addr, Duration::from_millis(100));
        if let Some(handle) = self.http_thread.take() {
            let _ = handle.join();
        }
        if let Some(tx) = self.grpc_shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.grpc_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BuiltinCollector {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_builtin_http(listener: TcpListener, app: Arc<HubApp>, stop: Arc<AtomicBool>) {
    for stream in listener.incoming() {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let Ok(mut stream) = stream else {
            continue;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
        let app = Arc::clone(&app);
        thread::spawn(move || {
            let Ok(req) = http::read_request(&mut stream) else {
                return;
            };
            if !req.is_local_host() {
                let _ = http::respond_err(
                    &mut stream,
                    "403 Forbidden",
                    &HubError::msg("non-local Host header rejected"),
                );
                return;
            }
            if req.body_too_large {
                let _ = http::respond_err(
                    &mut stream,
                    "413 Payload Too Large",
                    &HubError::msg("OTLP request body exceeds maximum allowed size"),
                );
                return;
            }
            if req.method == "POST" && !req.allows_browser_post() {
                let _ = http::respond_err(
                    &mut stream,
                    "403 Forbidden",
                    &HubError::msg("cross-origin browser POST rejected"),
                );
                return;
            }
            if req.method == "POST"
                && let Some(sig) = OtlpSignal::from_http_path(&req.path)
            {
                let _ = handle_http_otlp(&mut stream, &app, &req, sig);
                return;
            }
            if req.method == "GET" && (req.path == "/" || req.path == "/healthz") {
                let _ = http::respond_json(
                    &mut stream,
                    "200 OK",
                    &serde_json::json!({"ok": true, "receiver": "airbug-hub-builtin-otlp"}),
                );
                return;
            }
            let _ = http::respond_err(
                &mut stream,
                "404 Not Found",
                &HubError::msg(
                    "expected POST /v1/{traces,metrics,logs} or /v1development/profiles",
                ),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};
    use opentelemetry_proto::tonic::{
        collector::{
            logs::v1::logs_service_client::LogsServiceClient,
            metrics::v1::metrics_service_client::MetricsServiceClient,
            trace::v1::trace_service_client::TraceServiceClient,
        },
        common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value as ProtoAny},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        metrics::v1::{
            Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
            number_data_point,
        },
        profiles::v1::{
            AggregationTemporality, Function, Line, Location, Profile, ResourceProfiles, Sample,
            ScopeProfiles, ValueType,
        },
        resource::v1::Resource,
        trace::v1::{ResourceSpans, ScopeSpans, Span},
    };
    use std::env;

    fn post_raw(
        addr: SocketAddr,
        path: &str,
        content_type: &str,
        encoding: Option<&str>,
        body: &[u8],
    ) -> (u16, String, Vec<u8>) {
        let mut s = TcpStream::connect(addr).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut head = format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
            addr.port(),
            body.len()
        );
        if let Some(enc) = encoding {
            head.push_str(&format!("Content-Encoding: {enc}\r\n"));
        }
        head.push_str("\r\n");
        s.write_all(head.as_bytes()).unwrap();
        s.write_all(body).unwrap();
        let mut resp = Vec::new();
        s.read_to_end(&mut resp).unwrap();
        let sep = resp.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let hdr = String::from_utf8_lossy(&resp[..sep]).into_owned();
        let code = hdr
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|c| c.parse::<u16>().ok())
            .unwrap_or(0);
        let ct = hdr
            .lines()
            .find_map(|l| {
                let (k, v) = l.split_once(':')?;
                k.trim()
                    .eq_ignore_ascii_case("content-type")
                    .then(|| v.trim().to_string())
            })
            .unwrap_or_default();
        (code, ct, resp[sep + 4..].to_vec())
    }

    fn gzip_bytes(raw: &[u8]) -> Vec<u8> {
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(raw).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn store_rotates_files_at_threshold_up_to_max_backups() {
        let dir = env::temp_dir().join(format!("airbug-otlp-rotate-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let paths = RootPaths::new(&dir);
        let store = OtlpStore::with_rotation(512, 3);

        for i in 0..12 {
            let line = serde_json::json!({
                "seq": i,
                "padding": "x".repeat(180)
            })
            .to_string();
            store
                .append_ndjson_line(OtlpSignal::Logs, &paths, &line)
                .unwrap();
        }

        assert!(paths.logs_file().is_file());
        assert!(rotated_backup_path(&paths.logs_file(), 1).is_file());
        assert!(rotated_backup_path(&paths.logs_file(), 2).is_file());
        assert!(rotated_backup_path(&paths.logs_file(), 3).is_file());
        assert!(!rotated_backup_path(&paths.logs_file(), 4).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn builtin_http_protobuf_gzip_and_json_ingest_all_four_signals() {
        let dir = env::temp_dir().join(format!("airbug-otlp-http-rx-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let app = Arc::new(HubApp::new(&dir, 0, None).unwrap());
        let mut collector = BuiltinCollector::start(Arc::clone(&app), 0, 0).unwrap();
        let addr = collector.http_addr();

        // 1. Traces via Protobuf + gzip
        let trace_id = hex::decode("5b8efff798038103d269b633813fc60c").unwrap();
        let span_id = hex::decode("eee19b7ec3c1b174").unwrap();
        let trace_req = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(ProtoAny::StringValue("orders".into())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "orders.http".into(),
                        version: "1.0".into(),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                    }),
                    spans: vec![Span {
                        trace_id: trace_id.clone(),
                        span_id: span_id.clone(),
                        parent_span_id: vec![],
                        trace_state: String::new(),
                        flags: 1,
                        name: "POST /order".into(),
                        kind: 2,
                        start_time_unix_nano: 1_700_000_000_000_000_000,
                        end_time_unix_nano: 1_700_000_000_025_000_000,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: None,
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let gz_trace = gzip_bytes(&trace_req.encode_to_vec());
        let (code, ct, resp_body) = post_raw(
            addr,
            "/v1/traces",
            "application/x-protobuf",
            Some("gzip"),
            &gz_trace,
        );
        assert_eq!(code, 200);
        assert!(ct.starts_with("application/x-protobuf"), "ct={ct}");
        let _ = ExportTraceServiceResponse::decode(resp_body.as_slice()).unwrap();

        // 2. Metrics via HTTP JSON
        let metrics_json = serde_json::to_vec(&serde_json::json!({
            "resourceMetrics": [{
                "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "orders"}}]},
                "scopeMetrics": [{
                    "scope": {"name": "orders.metrics"},
                    "metrics": [{
                        "name": "orders.created",
                        "unit": "1",
                        "sum": {
                            "aggregationTemporality": 1,
                            "isMonotonic": true,
                            "dataPoints": [{"timeUnixNano": "1700000000000000000", "asInt": "7"}]
                        }
                    }]
                }]
            }]
        }))
        .unwrap();
        let (code, ct, _) = post_raw(addr, "/v1/metrics", "application/json", None, &metrics_json);
        assert_eq!(code, 200);
        assert!(ct.starts_with("application/json"), "ct={ct}");

        // 3. Logs via HTTP JSON + gzip
        let logs_json = gzip_bytes(
            serde_json::to_vec(&serde_json::json!({
                "resourceLogs": [{
                    "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "orders"}}]},
                    "scopeLogs": [{
                        "logRecords": [{
                            "timeUnixNano": "1700000000010000000",
                            "severityText": "INFO",
                            "body": {"stringValue": "order created"},
                            "traceId": "5b8efff798038103d269b633813fc60c",
                            "spanId": "eee19b7ec3c1b174"
                        }]
                    }]
                }]
            }))
            .unwrap()
            .as_slice(),
        );
        let (code, _, _) = post_raw(
            addr,
            "/v1/logs",
            "application/json",
            Some("gzip"),
            &logs_json,
        );
        assert_eq!(code, 200);

        // 4. Profiles via Protobuf + gzip on /v1development/profiles
        let pdata = ProfilesData {
            string_table: vec!["".into(), "cpu".into(), "nanoseconds".into(), "main".into()],
            function_table: vec![Function {
                name_strindex: 3,
                system_name_strindex: 3,
                filename_strindex: 0,
                start_line: 1,
            }],
            location_table: vec![Location {
                mapping_index: None,
                address: 0,
                line: vec![Line {
                    function_index: 0,
                    line: 1,
                    column: 0,
                }],
                is_folded: false,
                attribute_indices: vec![],
            }],
            resource_profiles: vec![ResourceProfiles {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(ProtoAny::StringValue("orders".into())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                schema_url: String::new(),
                scope_profiles: vec![ScopeProfiles {
                    scope: None,
                    schema_url: String::new(),
                    profiles: vec![Profile {
                        sample_type: vec![ValueType {
                            type_strindex: 1,
                            unit_strindex: 2,
                            aggregation_temporality: AggregationTemporality::Delta as i32,
                        }],
                        sample: vec![Sample {
                            locations_start_index: 0,
                            locations_length: 1,
                            value: vec![50_000],
                            attribute_indices: vec![],
                            link_index: None,
                            timestamps_unix_nano: vec![],
                        }],
                        location_indices: vec![0],
                        time_nanos: 1_700_000_000_000_000_000,
                        duration_nanos: 10_000_000,
                        period_type: None,
                        period: 1,
                        comment_strindices: vec![],
                        default_sample_type_index: 0,
                        profile_id: vec![2; 16],
                        dropped_attributes_count: 0,
                        original_payload_format: String::new(),
                        original_payload: vec![],
                        attribute_indices: vec![],
                    }],
                }],
            }],
            mapping_table: vec![],
            link_table: vec![],
            attribute_table: vec![],
            attribute_units: vec![],
        };
        let gz_prof = gzip_bytes(&pdata.encode_to_vec());
        let (code, _, _) = post_raw(
            addr,
            "/v1development/profiles",
            "application/x-protobuf",
            Some("gzip"),
            &gz_prof,
        );
        assert_eq!(code, 200);

        collector.stop();

        // Verify all 4 signals persisted and readable!
        let tr = crate::otlp::read_traces(
            &dir,
            &crate::otlp::TracesQuery {
                limit: 20,
                ..Default::default()
            },
        );
        assert_eq!(tr.traces.len(), 1);
        assert_eq!(tr.traces[0].root_name, "POST /order");

        let mt = crate::otlp::read_metrics_filtered(&dir, 50, None);
        assert_eq!(mt.points.len(), 1);
        assert_eq!(mt.points[0].value, 7.0);

        let lg =
            crate::otlp::read_logs_query(&dir, 50, None, Some("5b8efff798038103d269b633813fc60c"));
        assert_eq!(lg.entries.len(), 1);
        assert_eq!(lg.entries[0].body, "order created");

        let pr = crate::otlp::read_profiles(
            &dir,
            &crate::otlp::ProfilesQuery {
                limit: 20,
                ..Default::default()
            },
        );
        assert_eq!(pr.profiles.len(), 1);
        assert_eq!(pr.total_value, 50_000);

        // Unified events.jsonl also received trace + metric + log envelopes!
        let ev_raw = fs::read_to_string(app.paths.events_file()).unwrap();
        assert_eq!(ev_raw.lines().count(), 3);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn builtin_tonic_grpc_server_ingests_traces_metrics_and_logs_with_gzip() {
        let dir = env::temp_dir().join(format!("airbug-otlp-grpc-rx-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let app = Arc::new(HubApp::new(&dir, 0, None).unwrap());
        let mut collector = BuiltinCollector::start(Arc::clone(&app), 0, 0).unwrap();
        let endpoint = format!("http://{}", collector.grpc_addr());

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let mut trace_client = TraceServiceClient::connect(endpoint.clone())
                .await
                .unwrap()
                .send_compressed(CompressionEncoding::Gzip)
                .accept_compressed(CompressionEncoding::Gzip);
            trace_client
                .export(ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: "service.name".into(),
                                value: Some(AnyValue {
                                    value: Some(ProtoAny::StringValue("grpc-svc".into())),
                                }),
                            }],
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: None,
                            spans: vec![Span {
                                trace_id: vec![9; 16],
                                span_id: vec![8; 8],
                                parent_span_id: vec![],
                                trace_state: String::new(),
                                flags: 1,
                                name: "grpc.op".into(),
                                kind: 2,
                                start_time_unix_nano: 10_000_000,
                                end_time_unix_nano: 30_000_000,
                                attributes: vec![],
                                dropped_attributes_count: 0,
                                events: vec![],
                                dropped_events_count: 0,
                                links: vec![],
                                dropped_links_count: 0,
                                status: None,
                            }],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                })
                .await
                .unwrap();

            let mut metrics_client = MetricsServiceClient::connect(endpoint.clone())
                .await
                .unwrap()
                .send_compressed(CompressionEncoding::Gzip);
            metrics_client
                .export(ExportMetricsServiceRequest {
                    resource_metrics: vec![ResourceMetrics {
                        resource: None,
                        scope_metrics: vec![ScopeMetrics {
                            scope: None,
                            metrics: vec![Metric {
                                name: "grpc.gauge".into(),
                                description: String::new(),
                                unit: "ms".into(),
                                metadata: vec![],
                                data: Some(metric::Data::Gauge(Gauge {
                                    data_points: vec![NumberDataPoint {
                                        attributes: vec![],
                                        start_time_unix_nano: 0,
                                        time_unix_nano: 20_000_000,
                                        exemplars: vec![],
                                        flags: 0,
                                        value: Some(number_data_point::Value::AsDouble(42.5)),
                                    }],
                                })),
                            }],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                })
                .await
                .unwrap();

            let mut logs_client = LogsServiceClient::connect(endpoint)
                .await
                .unwrap()
                .send_compressed(CompressionEncoding::Gzip);
            logs_client
                .export(ExportLogsServiceRequest {
                    resource_logs: vec![ResourceLogs {
                        resource: None,
                        scope_logs: vec![ScopeLogs {
                            scope: None,
                            log_records: vec![LogRecord {
                                time_unix_nano: 25_000_000,
                                observed_time_unix_nano: 25_000_000,
                                severity_number: 9,
                                severity_text: "INFO".into(),
                                body: Some(AnyValue {
                                    value: Some(ProtoAny::StringValue("hello over grpc".into())),
                                }),
                                attributes: vec![],
                                dropped_attributes_count: 0,
                                flags: 1,
                                trace_id: vec![9; 16],
                                span_id: vec![8; 8],
                                event_name: "grpc.ready".into(),
                            }],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                })
                .await
                .unwrap();
        });

        collector.stop();

        let tr = crate::otlp::read_traces(
            &dir,
            &crate::otlp::TracesQuery {
                limit: 10,
                ..Default::default()
            },
        );
        assert_eq!(tr.traces.len(), 1);
        assert_eq!(tr.traces[0].root_name, "grpc.op");

        let mt = crate::otlp::read_metrics_filtered(&dir, 10, None);
        assert_eq!(mt.points.len(), 1);
        assert!((mt.points[0].value - 42.5).abs() < 1e-6);

        let lg = crate::otlp::read_logs_filtered(&dir, 10, None);
        assert_eq!(lg.entries.len(), 1);
        assert_eq!(lg.entries[0].body, "hello over grpc");
        assert_eq!(lg.entries[0].event_name.as_deref(), Some("grpc.ready"));

        let _ = fs::remove_dir_all(&dir);
    }
}
