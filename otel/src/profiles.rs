//! 4th OpenTelemetry signal: OTLP Profiles (`opentelemetry.proto.profiles.v1development`).
//!
//! Builds canonical [`ProfilesData`] with deduplicated `string_table` (`string_table[0] == ""`),
//! `function_table`, `location_table`, `attribute_table`, and `link_table` (linking stack samples
//! directly to active `trace_id` / `span_id`), and exports to `{endpoint}/v1development/profiles`.

use crate::{Protocol, TelemetryError};
use opentelemetry::{Context, trace::TraceContextExt};
use opentelemetry_proto::tonic::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue as ProtoKeyValue, any_value::Value},
    profiles::v1::{
        AggregationTemporality, Function, Line, Link, Location, Profile, ProfilesData,
        ResourceProfiles, Sample, ScopeProfiles, ValueType,
    },
    resource::v1::Resource as ProtoResource,
};
use prost::Message;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub use opentelemetry_proto::tonic::profiles::v1::ProfilesData as OtlpProfilesData;

static PROFILE_SEQ: AtomicU64 = AtomicU64::new(1);

/// One resolved stack frame inside a [`ProfileSample`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFrame {
    pub function_name: String,
    pub filename: String,
    pub line: i64,
}

impl ProfileFrame {
    pub fn new(function_name: impl Into<String>, filename: impl Into<String>, line: i64) -> Self {
        Self {
            function_name: function_name.into(),
            filename: filename.into(),
            line,
        }
    }
}

/// One stack sample (root-to-leaf or leaf-to-root frames, value, timestamp, and optional span link).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSample {
    pub frames: Vec<ProfileFrame>,
    pub value: i64,
    pub timestamp_unix_nano: u64,
    pub trace_id: Option<[u8; 16]>,
    pub span_id: Option<[u8; 8]>,
    pub attributes: Vec<(String, String)>,
}

impl ProfileSample {
    pub fn new(frames: Vec<ProfileFrame>, value: i64) -> Self {
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Self {
            frames,
            value,
            timestamp_unix_nano: now_nanos,
            trace_id: None,
            span_id: None,
            attributes: Vec::new(),
        }
    }

    pub fn with_timestamp_unix_nano(mut self, ts: u64) -> Self {
        self.timestamp_unix_nano = ts;
        self
    }

    pub fn with_trace_context(mut self, trace_id: [u8; 16], span_id: [u8; 8]) -> Self {
        self.trace_id = Some(trace_id);
        self.span_id = Some(span_id);
        self
    }

    /// Automatically attach the currently active OpenTelemetry span's `trace_id` and `span_id`, if valid.
    pub fn with_current_span(mut self) -> Self {
        Context::map_current(|cx| {
            let sc = cx.span().span_context().clone();
            if sc.is_valid() {
                self.trace_id = Some(sc.trace_id().to_bytes());
                self.span_id = Some(sc.span_id().to_bytes());
            }
        });
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((key.into(), value.into()));
        self
    }
}

/// High-level builder for a batch of profile samples from one service/workload.
#[derive(Debug, Clone)]
pub struct ProfileBatch {
    pub service_name: String,
    pub sample_type: String,
    pub unit: String,
    pub period_type: String,
    pub period_unit: String,
    pub period: i64,
    pub duration_nanos: i64,
    pub run_id: Option<String>,
    pub samples: Vec<ProfileSample>,
}

impl ProfileBatch {
    pub fn new(
        service_name: impl Into<String>,
        sample_type: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        let st = sample_type.into();
        let u = unit.into();
        Self {
            service_name: service_name.into(),
            period_type: st.clone(),
            period_unit: u.clone(),
            sample_type: st,
            unit: u,
            period: 1,
            duration_nanos: 0,
            run_id: std::env::var("AIRBUG_RUN_ID")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            samples: Vec::new(),
        }
    }

    pub fn period(
        mut self,
        period_type: impl Into<String>,
        period_unit: impl Into<String>,
        period: i64,
    ) -> Self {
        self.period_type = period_type.into();
        self.period_unit = period_unit.into();
        self.period = period;
        self
    }

    pub fn duration_nanos(mut self, duration_nanos: i64) -> Self {
        self.duration_nanos = duration_nanos;
        self
    }

    pub fn run_id(mut self, run_id: impl Into<String>) -> Self {
        let r = run_id.into();
        self.run_id = (!r.trim().is_empty()).then_some(r);
        self
    }

    pub fn add_sample(mut self, sample: ProfileSample) -> Self {
        self.samples.push(sample);
        self
    }

    pub fn samples(mut self, samples: impl IntoIterator<Item = ProfileSample>) -> Self {
        self.samples.extend(samples);
        self
    }

    /// Build a canonical OpenTelemetry [`ProfilesData`] protobuf structure with full dictionary deduplication.
    pub fn to_proto(&self) -> ProfilesData {
        let mut string_table = vec![String::new()];
        let mut string_idx: HashMap<String, i32> = HashMap::new();
        string_idx.insert(String::new(), 0);

        let mut intern_str = |s: &str, table: &mut Vec<String>| -> i32 {
            if let Some(&idx) = string_idx.get(s) {
                return idx;
            }
            let idx = table.len() as i32;
            table.push(s.to_string());
            string_idx.insert(s.to_string(), idx);
            idx
        };

        let mut function_table: Vec<Function> = Vec::new();
        let mut function_idx: HashMap<(i32, i32), i32> = HashMap::new();

        let mut location_table: Vec<Location> = Vec::new();
        let mut location_idx: HashMap<(i32, i64), i32> = HashMap::new();

        let mut attribute_table: Vec<ProtoKeyValue> = Vec::new();
        let mut attribute_idx: HashMap<(String, String), i32> = HashMap::new();

        let mut link_table: Vec<Link> = Vec::new();
        let mut link_idx: HashMap<([u8; 16], [u8; 8]), i32> = HashMap::new();

        let sample_type_str = intern_str(&self.sample_type, &mut string_table);
        let sample_unit_str = intern_str(&self.unit, &mut string_table);
        let period_type_str = intern_str(&self.period_type, &mut string_table);
        let period_unit_str = intern_str(&self.period_unit, &mut string_table);

        let mut location_indices: Vec<i32> = Vec::new();
        let mut proto_samples: Vec<Sample> = Vec::with_capacity(self.samples.len());

        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        for s in &self.samples {
            let start_idx = location_indices.len() as i32;
            for frame in &s.frames {
                let fn_str = intern_str(&frame.function_name, &mut string_table);
                let file_str = intern_str(&frame.filename, &mut string_table);
                let f_idx = *function_idx.entry((fn_str, file_str)).or_insert_with(|| {
                    let idx = function_table.len() as i32;
                    function_table.push(Function {
                        name_strindex: fn_str,
                        system_name_strindex: fn_str,
                        filename_strindex: file_str,
                        start_line: frame.line.max(0),
                    });
                    idx
                });

                let loc_id = *location_idx.entry((f_idx, frame.line)).or_insert_with(|| {
                    let idx = location_table.len() as i32;
                    location_table.push(Location {
                        mapping_index: None,
                        address: 0,
                        line: vec![Line {
                            function_index: f_idx,
                            line: frame.line.max(0),
                            column: 0,
                        }],
                        is_folded: false,
                        attribute_indices: Vec::new(),
                    });
                    idx
                });

                location_indices.push(loc_id);
            }
            let loc_len = (location_indices.len() as i32) - start_idx;

            let mut attr_indices = Vec::with_capacity(s.attributes.len());
            for (k, v) in &s.attributes {
                let a_idx = *attribute_idx
                    .entry((k.clone(), v.clone()))
                    .or_insert_with(|| {
                        let idx = attribute_table.len() as i32;
                        attribute_table.push(ProtoKeyValue {
                            key: k.clone(),
                            value: Some(AnyValue {
                                value: Some(Value::StringValue(v.clone())),
                            }),
                        });
                        idx
                    });
                attr_indices.push(a_idx);
            }

            let sample_link = match (s.trace_id, s.span_id) {
                (Some(tid), Some(sid)) if tid != [0u8; 16] && sid != [0u8; 8] => {
                    let l_idx = *link_idx.entry((tid, sid)).or_insert_with(|| {
                        let idx = link_table.len() as i32;
                        link_table.push(Link {
                            trace_id: tid.to_vec(),
                            span_id: sid.to_vec(),
                        });
                        idx
                    });
                    Some(l_idx)
                }
                _ => None,
            };

            let timestamps = if s.timestamp_unix_nano > 0 {
                vec![s.timestamp_unix_nano]
            } else {
                Vec::new()
            };

            proto_samples.push(Sample {
                locations_start_index: start_idx,
                locations_length: loc_len,
                value: vec![s.value],
                attribute_indices: attr_indices,
                link_index: sample_link,
                timestamps_unix_nano: timestamps,
            });
        }

        let seq = PROFILE_SEQ.fetch_add(1, Ordering::Relaxed);
        let mut profile_id = [0u8; 16];
        profile_id[..8].copy_from_slice(&(now_nanos as u64).to_be_bytes());
        profile_id[8..].copy_from_slice(&seq.to_be_bytes());

        let mut resource_attrs = vec![ProtoKeyValue {
            key: "service.name".into(),
            value: Some(AnyValue {
                value: Some(Value::StringValue(self.service_name.clone())),
            }),
        }];
        if let Some(run_id) = &self.run_id {
            resource_attrs.push(ProtoKeyValue {
                key: "airbug.run_id".into(),
                value: Some(AnyValue {
                    value: Some(Value::StringValue(run_id.clone())),
                }),
            });
        }

        let profile = Profile {
            sample_type: vec![ValueType {
                type_strindex: sample_type_str,
                unit_strindex: sample_unit_str,
                aggregation_temporality: AggregationTemporality::Delta as i32,
            }],
            sample: proto_samples,
            location_indices,
            time_nanos: now_nanos,
            duration_nanos: self.duration_nanos,
            period_type: Some(ValueType {
                type_strindex: period_type_str,
                unit_strindex: period_unit_str,
                aggregation_temporality: AggregationTemporality::Delta as i32,
            }),
            period: self.period,
            comment_strindices: Vec::new(),
            default_sample_type_index: 0,
            profile_id: profile_id.to_vec(),
            dropped_attributes_count: 0,
            original_payload_format: String::new(),
            original_payload: Vec::new(),
            attribute_indices: Vec::new(),
        };

        ProfilesData {
            resource_profiles: vec![ResourceProfiles {
                resource: Some(ProtoResource {
                    attributes: resource_attrs,
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_profiles: vec![ScopeProfiles {
                    scope: Some(InstrumentationScope {
                        name: "airbug.otel.profiles".into(),
                        version: env!("CARGO_PKG_VERSION").into(),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                    }),
                    profiles: vec![profile],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
            mapping_table: Vec::new(),
            location_table,
            function_table,
            link_table,
            string_table,
            attribute_table,
            attribute_units: Vec::new(),
        }
    }
}

/// Encode [`ProfilesData`] to canonical OTLP binary protobuf (`application/x-protobuf`).
pub fn encode_profiles_protobuf(data: &ProfilesData) -> Vec<u8> {
    data.encode_to_vec()
}

/// Encode [`ProfilesData`] to canonical OTLP JSON (`application/json`).
pub fn encode_profiles_json(data: &ProfilesData) -> Result<Vec<u8>, TelemetryError> {
    serde_json::to_vec(data)
        .map_err(|e| TelemetryError::Config(format!("profiles JSON encode failed: {e}")))
}

/// Export [`ProfilesData`] over HTTP (`application/x-protobuf` or `application/json`, with optional `gzip`)
/// to `{endpoint}/v1development/profiles` (or record into [`crate::TestExporters`] when `testing` is active
/// and `endpoint` is `"memory://"`).
pub fn export_profiles_http(
    endpoint: &str,
    data: &ProfilesData,
    protocol: Protocol,
    gzip: bool,
) -> Result<(), TelemetryError> {
    #[cfg(feature = "testing")]
    crate::record_test_profile(data.clone());

    if endpoint.trim().is_empty() || endpoint.starts_with("memory://") {
        return Ok(());
    }

    #[cfg(not(feature = "otlp-http"))]
    {
        let _ = (data, protocol, gzip);
        Err(TelemetryError::Config(
            "enable airbug-otel feature otlp-http to export profiles over HTTP".into(),
        ))
    }

    #[cfg(feature = "otlp-http")]
    {
        use flate2::{Compression as GzLevel, write::GzEncoder};
        use std::io::Write;

        let (raw_body, content_type) = match protocol {
            Protocol::HttpJson => (encode_profiles_json(data)?, "application/json"),
            Protocol::Http | Protocol::Grpc => {
                (encode_profiles_protobuf(data), "application/x-protobuf")
            }
        };

        let body = if gzip {
            let mut enc = GzEncoder::new(Vec::new(), GzLevel::default());
            enc.write_all(&raw_body)
                .map_err(|e| TelemetryError::Config(format!("gzip profiles failed: {e}")))?;
            enc.finish()
                .map_err(|e| TelemetryError::Config(format!("gzip finish failed: {e}")))?
        } else {
            raw_body
        };

        let base = endpoint.trim_end_matches('/');
        let url = if base.ends_with("/v1development/profiles") {
            base.to_string()
        } else {
            format!("{base}/v1development/profiles")
        };

        // Run reqwest::blocking::Client on a dedicated OS thread so calling export_profiles_http
        // from inside an async Tokio runtime never panics on nested runtime drop.
        std::thread::scope(|s| {
            s.spawn(move || -> Result<(), TelemetryError> {
                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .map_err(|e| TelemetryError::Config(format!("http client: {e}")))?;
                let mut req = client.post(&url).header("Content-Type", content_type);
                if gzip {
                    req = req.header("Content-Encoding", "gzip");
                }
                let resp = req.body(body).send().map_err(|e| {
                    TelemetryError::Config(format!("profiles export POST {url}: {e}"))
                })?;
                if !resp.status().is_success() {
                    return Err(TelemetryError::Config(format!(
                        "profiles export POST {url} returned HTTP {}",
                        resp.status()
                    )));
                }
                Ok(())
            })
            .join()
            .unwrap_or_else(|_| {
                Err(TelemetryError::Config(
                    "profiles export thread panicked".into(),
                ))
            })
        })
    }
}
