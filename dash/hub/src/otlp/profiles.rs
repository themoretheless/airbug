//! OTLP Profiles (`v1development`) engine: dictionary decoding, trace-linked sample filtering,
//! top-functions Self vs Total ranking, and merged Flamechart / Icicle tree construction.
use super::{
    any_value, decode_id_hex, has_signal_file, parse_i64, parse_u64, resource_attr, resource_attrs,
    split_json_values, tail_text,
};
use crate::config::{self, RootPaths};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
};

#[derive(Debug, Default, Clone)]
pub struct ProfilesQuery {
    pub limit: usize,
    pub service: Option<String>,
    pub sample_type: Option<String>,
    pub run_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfilesResponse {
    pub path: String,
    pub available: bool,
    pub profiles: Vec<ProfileSummary>,
    pub services: Vec<String>,
    pub sample_types: Vec<String>,
    pub unit: String,
    pub total_samples: usize,
    pub total_value: i64,
    pub top_functions: Vec<TopFunction>,
    pub flamechart: FlameNode,
    pub linked_traces: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummary {
    pub profile_id: String,
    pub service: String,
    pub scope: String,
    pub sample_type: String,
    pub unit: String,
    pub samples_count: usize,
    pub total_value: i64,
    pub duration_ms: f64,
    pub time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub linked_traces: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopFunction {
    pub name: String,
    pub filename: String,
    pub line: i64,
    pub self_value: i64,
    pub total_value: i64,
    pub self_pct: f64,
    pub total_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlameNode {
    pub name: String,
    pub file: String,
    pub line: i64,
    pub value: i64,
    pub self_value: i64,
    pub children: Vec<FlameNode>,
}

#[derive(Clone, Debug)]
struct ResolvedFrame {
    name: String,
    file: String,
    line: i64,
}

#[derive(Clone, Debug)]
struct DecodedSample {
    frames: Vec<ResolvedFrame>,
    value: i64,
    trace_id: Option<String>,
    #[allow(dead_code)]
    span_id: Option<String>,
    attrs: HashMap<String, String>,
}

#[derive(Clone, Debug)]
struct DecodedProfile {
    summary: ProfileSummary,
    samples: Vec<DecodedSample>,
}

pub fn read_profiles(root: &Path, query: &ProfilesQuery) -> ProfilesResponse {
    let paths = RootPaths::new(root);
    let path = paths.profiles_file();
    let path_s = path.display().to_string();

    if !has_signal_file(&path) {
        return ProfilesResponse {
            path: path_s,
            available: false,
            profiles: vec![],
            services: vec![],
            sample_types: vec![],
            unit: String::new(),
            total_samples: 0,
            total_value: 0,
            top_functions: vec![],
            flamechart: empty_flame_root(),
            linked_traces: vec![],
            note: "No profiles file yet. POST OTLP ProfilesData to /v1development/profiles to inspect flamecharts.".into(),
        };
    }

    let buf = match tail_text(&path, config::PROFILES_TAIL_BYTES) {
        Ok(b) => b,
        Err(err) => {
            return ProfilesResponse {
                path: path_s,
                available: true,
                profiles: vec![],
                services: vec![],
                sample_types: vec![],
                unit: String::new(),
                total_samples: 0,
                total_value: 0,
                top_functions: vec![],
                flamechart: empty_flame_root(),
                linked_traces: vec![],
                note: format!("Failed to read profiles: {err}"),
            };
        }
    };

    let mut decoded_profiles = Vec::new();
    for chunk in split_json_values(&buf) {
        if let Ok(val) = serde_json::from_str::<Value>(chunk) {
            extract_profiles(&val, &mut decoded_profiles);
        }
    }

    let mut all_services = BTreeSet::new();
    let mut all_sample_types = BTreeSet::new();
    for p in &decoded_profiles {
        if !p.summary.service.is_empty() {
            all_services.insert(p.summary.service.clone());
        }
        if !p.summary.sample_type.is_empty() {
            all_sample_types.insert(p.summary.sample_type.clone());
        }
    }

    let wanted_service = query
        .service
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("ALL"));
    let wanted_type = query
        .sample_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("ALL"));
    let wanted_run = query
        .run_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let wanted_trace = query
        .trace_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let mut filtered_profiles: Vec<ProfileSummary> = Vec::new();
    let mut matching_samples: Vec<DecodedSample> = Vec::new();
    let mut chosen_unit = String::new();
    let mut linked_traces_set = BTreeSet::new();

    for mut dp in decoded_profiles {
        if let Some(svc) = wanted_service
            && dp.summary.service != svc
        {
            continue;
        }
        if let Some(st) = wanted_type
            && !dp.summary.sample_type.eq_ignore_ascii_case(st)
        {
            continue;
        }
        if let Some(rid) = wanted_run {
            let profile_matches = dp.summary.run_id.as_deref() == Some(rid);
            if !profile_matches {
                dp.samples
                    .retain(|s| s.attrs.get("airbug.run_id").map(String::as_str) == Some(rid));
            }
            if dp.samples.is_empty() {
                continue;
            }
        }
        if let Some(tid) = wanted_trace {
            dp.samples.retain(|s| {
                s.trace_id
                    .as_deref()
                    .is_some_and(|t| t.eq_ignore_ascii_case(tid))
            });
            if dp.samples.is_empty() {
                continue;
            }
        }

        // Recompute filtered summary counts if sample-level filtering narrowed the profile.
        dp.summary.samples_count = dp.samples.len();
        dp.summary.total_value = dp.samples.iter().map(|s| s.value).sum();
        let mut p_traces = BTreeSet::new();
        for s in &dp.samples {
            if let Some(ref tid) = s.trace_id {
                p_traces.insert(tid.clone());
                linked_traces_set.insert(tid.clone());
            }
        }
        dp.summary.linked_traces = p_traces.into_iter().collect();

        if chosen_unit.is_empty() && !dp.summary.unit.is_empty() {
            chosen_unit = dp.summary.unit.clone();
        }
        filtered_profiles.push(dp.summary);
        matching_samples.extend(dp.samples);
    }

    filtered_profiles.reverse(); // newest first
    let limit = query.limit.max(1);
    if filtered_profiles.len() > limit {
        filtered_profiles.truncate(limit);
    }

    let total_samples = matching_samples.len();
    let total_value: i64 = matching_samples.iter().map(|s| s.value.max(0)).sum();
    let flamechart = build_flamechart(&matching_samples);
    let top_functions = build_top_functions(&matching_samples, total_value);
    let linked_traces: Vec<String> = linked_traces_set.into_iter().collect();

    let note = if filtered_profiles.is_empty() {
        if wanted_service.is_some()
            || wanted_type.is_some()
            || wanted_run.is_some()
            || wanted_trace.is_some()
        {
            "No profiles match the active filters.".into()
        } else {
            "Profiles file is empty or still buffering.".into()
        }
    } else {
        format!(
            "{} profile(s) · {} sample(s) · total {} {}",
            filtered_profiles.len(),
            total_samples,
            total_value,
            chosen_unit
        )
        .trim()
        .to_string()
    };

    ProfilesResponse {
        path: path_s,
        available: true,
        profiles: filtered_profiles,
        services: all_services.into_iter().collect(),
        sample_types: all_sample_types.into_iter().collect(),
        unit: chosen_unit,
        total_samples,
        total_value,
        top_functions,
        flamechart,
        linked_traces,
        note,
    }
}

fn empty_flame_root() -> FlameNode {
    FlameNode {
        name: "root".into(),
        file: String::new(),
        line: 0,
        value: 0,
        self_value: 0,
        children: vec![],
    }
}

fn build_flamechart(samples: &[DecodedSample]) -> FlameNode {
    let mut root = empty_flame_root();
    for sample in samples {
        let v = sample.value.max(0);
        if v == 0 || sample.frames.is_empty() {
            continue;
        }
        root.value = root.value.saturating_add(v);
        insert_stack(&mut root, &sample.frames, v);
    }
    sort_flame_tree(&mut root);
    root
}

fn insert_stack(node: &mut FlameNode, frames: &[ResolvedFrame], value: i64) {
    let Some((head, tail)) = frames.split_first() else {
        return;
    };
    let child_pos = node
        .children
        .iter()
        .position(|c| c.name == head.name && c.file == head.file && c.line == head.line);
    let idx = match child_pos {
        Some(i) => i,
        None => {
            node.children.push(FlameNode {
                name: head.name.clone(),
                file: head.file.clone(),
                line: head.line,
                value: 0,
                self_value: 0,
                children: Vec::new(),
            });
            node.children.len() - 1
        }
    };
    let child = &mut node.children[idx];
    child.value = child.value.saturating_add(value);
    if tail.is_empty() {
        child.self_value = child.self_value.saturating_add(value);
    } else {
        insert_stack(child, tail, value);
    }
}

fn sort_flame_tree(node: &mut FlameNode) {
    node.children
        .sort_by(|a, b| b.value.cmp(&a.value).then_with(|| a.name.cmp(&b.name)));
    for child in &mut node.children {
        sort_flame_tree(child);
    }
}

fn build_top_functions(samples: &[DecodedSample], grand_total: i64) -> Vec<TopFunction> {
    struct FnAcc {
        filename: String,
        line: i64,
        self_value: i64,
        total_value: i64,
    }
    let mut map: HashMap<(String, String), FnAcc> = HashMap::new();

    for s in samples {
        let v = s.value.max(0);
        if v == 0 || s.frames.is_empty() {
            continue;
        }
        // Avoid double-counting recursive appearances of the same function in `total_value`.
        let mut seen_in_sample = BTreeSet::new();
        for (i, f) in s.frames.iter().enumerate() {
            let key = (f.name.clone(), f.file.clone());
            let entry = map.entry(key.clone()).or_insert_with(|| FnAcc {
                filename: f.file.clone(),
                line: f.line,
                self_value: 0,
                total_value: 0,
            });
            if seen_in_sample.insert(key) {
                entry.total_value = entry.total_value.saturating_add(v);
            }
            if i + 1 == s.frames.len() {
                entry.self_value = entry.self_value.saturating_add(v);
            }
        }
    }

    let denom = (grand_total.max(1)) as f64;
    let mut out: Vec<TopFunction> = map
        .into_iter()
        .map(|((name, _), acc)| TopFunction {
            name,
            filename: acc.filename,
            line: acc.line,
            self_value: acc.self_value,
            total_value: acc.total_value,
            self_pct: (acc.self_value as f64 / denom) * 100.0,
            total_pct: (acc.total_value as f64 / denom) * 100.0,
        })
        .collect();
    out.sort_by(|a, b| {
        b.self_value
            .cmp(&a.self_value)
            .then_with(|| b.total_value.cmp(&a.total_value))
            .then_with(|| a.name.cmp(&b.name))
    });
    if out.len() > 50 {
        out.truncate(50);
    }
    out
}

fn extract_profiles(value: &Value, out: &mut Vec<DecodedProfile>) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            extract_profiles(item, out);
        }
        return;
    }

    let dict = value.get("dictionary").unwrap_or(value);
    let string_table: Vec<String> = dict
        .get("stringTable")
        .or_else(|| dict.get("string_table"))
        .or_else(|| value.get("stringTable"))
        .or_else(|| value.get("string_table"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default();

    let get_str = |idx: i64| -> String {
        if idx <= 0 {
            return String::new();
        }
        string_table.get(idx as usize).cloned().unwrap_or_default()
    };

    let function_table = dict
        .get("functionTable")
        .or_else(|| dict.get("function_table"))
        .or_else(|| value.get("functionTable"))
        .or_else(|| value.get("function_table"))
        .and_then(Value::as_array);

    let location_table = dict
        .get("locationTable")
        .or_else(|| dict.get("location_table"))
        .or_else(|| value.get("locationTable"))
        .or_else(|| value.get("location_table"))
        .and_then(Value::as_array);

    let attribute_table = dict
        .get("attributeTable")
        .or_else(|| dict.get("attribute_table"))
        .or_else(|| value.get("attributeTable"))
        .or_else(|| value.get("attribute_table"))
        .and_then(Value::as_array);

    let link_table = dict
        .get("linkTable")
        .or_else(|| dict.get("link_table"))
        .or_else(|| value.get("linkTable"))
        .or_else(|| value.get("link_table"))
        .and_then(Value::as_array);

    let Some(resource_profiles) = value
        .get("resourceProfiles")
        .or_else(|| value.get("resource_profiles"))
        .and_then(Value::as_array)
    else {
        return;
    };

    let mut seq_fallback = out.len() + 1;

    for rp in resource_profiles {
        let raw_svc = resource_attr(rp, "service.name");
        let service = if raw_svc.trim().is_empty() {
            "unknown-service".to_string()
        } else {
            raw_svc
        };
        let res_attrs = resource_attrs(rp);
        let res_run_id = res_attrs.get("airbug.run_id").cloned();

        let Some(scopes) = rp
            .get("scopeProfiles")
            .or_else(|| rp.get("scope_profiles"))
            .and_then(Value::as_array)
        else {
            continue;
        };

        for sp in scopes {
            let scope = sp
                .get("scope")
                .and_then(|s| s.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();

            let Some(profiles) = sp.get("profiles").and_then(Value::as_array) else {
                continue;
            };

            for prof in profiles {
                let profile_id = prof
                    .get("profileId")
                    .or_else(|| prof.get("profile_id"))
                    .and_then(|v| decode_id_hex(v, 16))
                    .unwrap_or_else(|| {
                        let id = format!("profile-{seq_fallback:04}");
                        seq_fallback += 1;
                        id
                    });

                let default_st_idx = prof
                    .get("defaultSampleTypeIndex")
                    .or_else(|| prof.get("default_sample_type_index"))
                    .and_then(parse_i64)
                    .unwrap_or(0)
                    .max(0) as usize;

                let (sample_type, unit) = prof
                    .get("sampleType")
                    .or_else(|| prof.get("sample_type"))
                    .and_then(Value::as_array)
                    .and_then(|arr| arr.get(default_st_idx).or_else(|| arr.first()))
                    .map(|vt| {
                        let t_idx = vt
                            .get("typeStrindex")
                            .or_else(|| vt.get("type_strindex"))
                            .and_then(parse_i64)
                            .unwrap_or(0);
                        let u_idx = vt
                            .get("unitStrindex")
                            .or_else(|| vt.get("unit_strindex"))
                            .and_then(parse_i64)
                            .unwrap_or(0);
                        let t_s = get_str(t_idx);
                        let u_s = get_str(u_idx);
                        (
                            if t_s.is_empty() {
                                "samples".into()
                            } else {
                                t_s
                            },
                            u_s,
                        )
                    })
                    .unwrap_or_else(|| ("samples".into(), "count".into()));

                let time_ms = prof
                    .get("timeNanos")
                    .or_else(|| prof.get("time_nanos"))
                    .and_then(parse_u64)
                    .filter(|&n| n > 0)
                    .map(|n| n / 1_000_000);
                let duration_ms = prof
                    .get("durationNanos")
                    .or_else(|| prof.get("duration_nanos"))
                    .and_then(parse_i64)
                    .filter(|&n| n > 0)
                    .map(|n| n as f64 / 1_000_000.0)
                    .unwrap_or(0.0);

                let prof_attrs = resolve_attr_indices(
                    prof.get("attributeIndices")
                        .or_else(|| prof.get("attribute_indices")),
                    attribute_table,
                    &get_str,
                );
                let run_id = res_run_id
                    .clone()
                    .or_else(|| prof_attrs.get("airbug.run_id").cloned());

                let loc_indices: Vec<i64> = prof
                    .get("locationIndices")
                    .or_else(|| prof.get("location_indices"))
                    .and_then(Value::as_array)
                    .map(|arr| arr.iter().filter_map(parse_i64).collect())
                    .unwrap_or_default();

                let raw_samples = prof
                    .get("sample")
                    .or_else(|| prof.get("samples"))
                    .and_then(Value::as_array);

                let mut decoded_samples = Vec::new();
                let mut linked_set = BTreeSet::new();
                let mut total_val = 0i64;

                if let Some(samples) = raw_samples {
                    for sm in samples {
                        let val = sm
                            .get("value")
                            .and_then(Value::as_array)
                            .and_then(|arr| {
                                arr.get(default_st_idx)
                                    .or_else(|| arr.first())
                                    .and_then(parse_i64)
                            })
                            .unwrap_or(1);

                        let frames = resolve_sample_frames(
                            sm,
                            &loc_indices,
                            location_table,
                            function_table,
                            &get_str,
                        );

                        let (trace_id, span_id) = sm
                            .get("linkIndex")
                            .or_else(|| sm.get("link_index"))
                            .and_then(parse_i64)
                            .filter(|&idx| idx >= 0)
                            .and_then(|idx| link_table.and_then(|lt| lt.get(idx as usize)))
                            .map(|lk| {
                                let tid = lk
                                    .get("traceId")
                                    .or_else(|| lk.get("trace_id"))
                                    .and_then(|v| decode_id_hex(v, 16));
                                let sid = lk
                                    .get("spanId")
                                    .or_else(|| lk.get("span_id"))
                                    .and_then(|v| decode_id_hex(v, 8));
                                (tid, sid)
                            })
                            .unwrap_or((None, None));

                        if let Some(ref tid) = trace_id {
                            linked_set.insert(tid.clone());
                        }

                        let mut s_attrs = prof_attrs.clone();
                        for (k, v) in resolve_attr_indices(
                            sm.get("attributeIndices")
                                .or_else(|| sm.get("attribute_indices")),
                            attribute_table,
                            &get_str,
                        ) {
                            s_attrs.insert(k, v);
                        }

                        total_val = total_val.saturating_add(val);
                        decoded_samples.push(DecodedSample {
                            frames,
                            value: val,
                            trace_id,
                            span_id,
                            attrs: s_attrs,
                        });
                    }
                }

                let summary = ProfileSummary {
                    profile_id,
                    service: service.clone(),
                    scope: scope.clone(),
                    sample_type,
                    unit,
                    samples_count: decoded_samples.len(),
                    total_value: total_val,
                    duration_ms,
                    time_ms,
                    run_id,
                    linked_traces: linked_set.into_iter().collect(),
                };
                out.push(DecodedProfile {
                    summary,
                    samples: decoded_samples,
                });
            }
        }
    }
}

fn resolve_attr_indices(
    indices_val: Option<&Value>,
    attribute_table: Option<&Vec<Value>>,
    get_str: &dyn Fn(i64) -> String,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let (Some(arr), Some(table)) = (indices_val.and_then(Value::as_array), attribute_table) else {
        return out;
    };
    for idx_val in arr {
        let Some(idx) = parse_i64(idx_val).filter(|&i| i >= 0) else {
            continue;
        };
        let Some(kv) = table.get(idx as usize) else {
            continue;
        };
        let key = kv
            .get("key")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                kv.get("keyStrindex")
                    .or_else(|| kv.get("key_strindex"))
                    .and_then(parse_i64)
                    .map(get_str)
            })
            .unwrap_or_default();
        if key.is_empty() {
            continue;
        }
        let val = kv.get("value").map(any_value).unwrap_or_default();
        out.insert(key, val);
    }
    out
}

fn resolve_sample_frames(
    sm: &Value,
    profile_loc_indices: &[i64],
    location_table: Option<&Vec<Value>>,
    function_table: Option<&Vec<Value>>,
    get_str: &dyn Fn(i64) -> String,
) -> Vec<ResolvedFrame> {
    let mut loc_ids: Vec<usize> = Vec::new();

    let start = sm
        .get("locationsStartIndex")
        .or_else(|| sm.get("locations_start_index"))
        .and_then(parse_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let len = sm
        .get("locationsLength")
        .or_else(|| sm.get("locations_length"))
        .and_then(parse_i64)
        .unwrap_or(0)
        .max(0) as usize;

    if len > 0 {
        if !profile_loc_indices.is_empty() {
            let end = (start + len).min(profile_loc_indices.len());
            for &li in &profile_loc_indices[start.min(end)..end] {
                if li >= 0 {
                    loc_ids.push(li as usize);
                }
            }
        } else {
            for li in start..(start + len) {
                loc_ids.push(li);
            }
        }
    } else if let Some(explicit) = sm
        .get("locationIndex")
        .or_else(|| sm.get("location_index"))
        .and_then(Value::as_array)
    {
        for v in explicit {
            if let Some(i) = parse_i64(v).filter(|&i| i >= 0) {
                loc_ids.push(i as usize);
            }
        }
    }

    let mut frames = Vec::new();
    let Some(loc_table) = location_table else {
        return frames;
    };

    for loc_idx in loc_ids {
        let Some(loc) = loc_table.get(loc_idx) else {
            continue;
        };
        if let Some(lines) = loc.get("line").and_then(Value::as_array)
            && !lines.is_empty()
        {
            for ln in lines {
                let fn_idx = ln
                    .get("functionIndex")
                    .or_else(|| ln.get("function_index"))
                    .and_then(parse_i64)
                    .unwrap_or(-1);
                let line_no = ln.get("line").and_then(parse_i64).unwrap_or(0);
                let (fn_name, file_name, start_line) = if fn_idx >= 0
                    && let Some(ft) = function_table
                    && let Some(func) = ft.get(fn_idx as usize)
                {
                    let name_si = func
                        .get("nameStrindex")
                        .or_else(|| func.get("name_strindex"))
                        .and_then(parse_i64)
                        .unwrap_or(0);
                    let sys_si = func
                        .get("systemNameStrindex")
                        .or_else(|| func.get("system_name_strindex"))
                        .and_then(parse_i64)
                        .unwrap_or(0);
                    let file_si = func
                        .get("filenameStrindex")
                        .or_else(|| func.get("filename_strindex"))
                        .and_then(parse_i64)
                        .unwrap_or(0);
                    let sl = func
                        .get("startLine")
                        .or_else(|| func.get("start_line"))
                        .and_then(parse_i64)
                        .unwrap_or(0);
                    let nm = get_str(name_si);
                    (
                        if nm.is_empty() { get_str(sys_si) } else { nm },
                        get_str(file_si),
                        sl,
                    )
                } else {
                    (String::new(), String::new(), 0)
                };

                let resolved_name = if fn_name.is_empty() {
                    let addr = loc.get("address").and_then(parse_u64).unwrap_or(0);
                    format!("0x{addr:x}")
                } else {
                    fn_name
                };
                frames.push(ResolvedFrame {
                    name: resolved_name,
                    file: file_name,
                    line: if line_no > 0 { line_no } else { start_line },
                });
            }
        } else {
            let addr = loc.get("address").and_then(parse_u64).unwrap_or(0);
            frames.push(ResolvedFrame {
                name: format!("0x{addr:x}"),
                file: String::new(),
                line: 0,
            });
        }
    }

    // If a pprof-style producer emitted leaf-first frames where the outermost entrypoint
    // (`main`, `_start`, `__libc_start_main`) sits at the end while the first frame is not an
    // entrypoint, flip to root-to-leaf order so the flamechart always grows downward from root.
    if frames.len() >= 2
        && is_entrypoint_frame(&frames[frames.len() - 1].name)
        && !is_entrypoint_frame(&frames[0].name)
    {
        frames.reverse();
    }

    frames
}

fn is_entrypoint_frame(name: &str) -> bool {
    matches!(
        name,
        "main" | "_start" | "__libc_start_main" | "runtime.main" | "std::rt::lang_start"
    ) || name.ends_with("::main")
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::{
        common::v1::{AnyValue, KeyValue, any_value::Value as ProtoAny},
        profiles::v1::{
            AggregationTemporality, Function, Line, Link, Location, Profile, ProfilesData,
            ResourceProfiles, Sample, ScopeProfiles, ValueType,
        },
        resource::v1::Resource,
    };
    use std::{env, fs};

    #[test]
    fn decodes_profiles_dictionary_links_and_merges_flamechart() {
        let dir = env::temp_dir().join(format!("airbug-profiles-engine-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let data_dir = dir.join("dash/collector/data");
        fs::create_dir_all(&data_dir).unwrap();

        let trace_bytes = hex::decode("5b8efff798038103d269b633813fc60c").unwrap();
        let span_bytes = hex::decode("eee19b7ec3c1b174").unwrap();

        let pdata = ProfilesData {
            string_table: vec![
                "".into(),            // 0
                "allocations".into(), // 1
                "bytes".into(),       // 2
                "main".into(),        // 3
                "src/main.rs".into(), // 4
                "compute".into(),     // 5
                "src/lib.rs".into(),  // 6
                "parse".into(),       // 7
            ],
            function_table: vec![
                Function {
                    name_strindex: 3,
                    system_name_strindex: 3,
                    filename_strindex: 4,
                    start_line: 10,
                },
                Function {
                    name_strindex: 5,
                    system_name_strindex: 5,
                    filename_strindex: 6,
                    start_line: 42,
                },
                Function {
                    name_strindex: 7,
                    system_name_strindex: 7,
                    filename_strindex: 6,
                    start_line: 88,
                },
            ],
            location_table: vec![
                Location {
                    mapping_index: None,
                    address: 0,
                    line: vec![Line {
                        function_index: 0,
                        line: 10,
                        column: 0,
                    }],
                    is_folded: false,
                    attribute_indices: vec![],
                },
                Location {
                    mapping_index: None,
                    address: 0,
                    line: vec![Line {
                        function_index: 1,
                        line: 42,
                        column: 0,
                    }],
                    is_folded: false,
                    attribute_indices: vec![],
                },
                Location {
                    mapping_index: None,
                    address: 0,
                    line: vec![Line {
                        function_index: 2,
                        line: 88,
                        column: 0,
                    }],
                    is_folded: false,
                    attribute_indices: vec![],
                },
            ],
            link_table: vec![Link {
                trace_id: trace_bytes,
                span_id: span_bytes,
            }],
            attribute_table: vec![KeyValue {
                key: "thread.name".into(),
                value: Some(AnyValue {
                    value: Some(ProtoAny::StringValue("worker-1".into())),
                }),
            }],
            mapping_table: vec![],
            attribute_units: vec![],
            resource_profiles: vec![ResourceProfiles {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".into(),
                            value: Some(AnyValue {
                                value: Some(ProtoAny::StringValue("bench-worker".into())),
                            }),
                        },
                        KeyValue {
                            key: "airbug.run_id".into(),
                            value: Some(AnyValue {
                                value: Some(ProtoAny::StringValue("run-prof-1".into())),
                            }),
                        },
                    ],
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
                        sample: vec![
                            Sample {
                                locations_start_index: 0,
                                locations_length: 2,
                                value: vec![2048],
                                attribute_indices: vec![0],
                                link_index: Some(0),
                                timestamps_unix_nano: vec![1_700_000_000_000_000_000],
                            },
                            Sample {
                                locations_start_index: 2,
                                locations_length: 2,
                                value: vec![1024],
                                attribute_indices: vec![],
                                link_index: None,
                                timestamps_unix_nano: vec![1_700_000_001_000_000_000],
                            },
                        ],
                        location_indices: vec![0, 1, 0, 2],
                        time_nanos: 1_700_000_000_000_000_000,
                        duration_nanos: 25_000_000,
                        period_type: None,
                        period: 1,
                        comment_strindices: vec![],
                        default_sample_type_index: 0,
                        profile_id: vec![1; 16],
                        dropped_attributes_count: 0,
                        original_payload_format: String::new(),
                        original_payload: vec![],
                        attribute_indices: vec![],
                    }],
                }],
            }],
        };

        let line = serde_json::to_string(&pdata).unwrap();
        fs::write(data_dir.join("profiles.json"), format!("{line}\n")).unwrap();

        let resp = read_profiles(
            &dir,
            &ProfilesQuery {
                limit: 20,
                ..Default::default()
            },
        );
        assert!(resp.available);
        assert_eq!(resp.profiles.len(), 1);
        assert_eq!(resp.total_value, 3072);
        assert_eq!(resp.unit, "bytes");
        assert_eq!(
            resp.linked_traces,
            vec!["5b8efff798038103d269b633813fc60c".to_string()]
        );

        // Flamechart: root (3072) -> main (3072) -> [compute (2048), parse (1024)]
        assert_eq!(resp.flamechart.value, 3072);
        assert_eq!(resp.flamechart.children.len(), 1);
        let main_node = &resp.flamechart.children[0];
        assert_eq!(main_node.name, "main");
        assert_eq!(main_node.value, 3072);
        assert_eq!(main_node.self_value, 0);
        assert_eq!(main_node.children.len(), 2);
        assert_eq!(main_node.children[0].name, "compute");
        assert_eq!(main_node.children[0].self_value, 2048);
        assert_eq!(main_node.children[1].name, "parse");
        assert_eq!(main_node.children[1].self_value, 1024);

        // Top functions: compute (self 2048), parse (self 1024), main (self 0, total 3072)
        assert_eq!(resp.top_functions[0].name, "compute");
        assert_eq!(resp.top_functions[0].self_value, 2048);

        // Filter by trace_id narrows to the 2048 sample only!
        let by_trace = read_profiles(
            &dir,
            &ProfilesQuery {
                limit: 20,
                trace_id: Some("5b8efff798038103d269b633813fc60c".into()),
                ..Default::default()
            },
        );
        assert_eq!(by_trace.total_value, 2048);
        assert_eq!(by_trace.flamechart.children[0].children.len(), 1);
        assert_eq!(by_trace.flamechart.children[0].children[0].name, "compute");

        let _ = fs::remove_dir_all(&dir);
    }
}
