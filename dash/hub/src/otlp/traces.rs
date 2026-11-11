//! OTLP Traces engine: span waterfall reconstruction, Critical Path, Service Topology,
//! RED metrics, zPages Tracez latency matrix, and Trace ↔ Logs ↔ Issues correlation.
use super::{
    attrs_to_map, decode_id_hex, has_signal_file, nano_u64, parse_i64, resource_attr,
    resource_attrs, split_json_values, tail_text,
};
use crate::{
    app::HubApp,
    config::{self, RootPaths},
    issues::IssueSummary,
    otlp::logs::LogEntry,
};
use opentelemetry_proto::tonic::tracez::v1::TracezCounts;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};

pub const TRACEZ_BUCKET_LABELS: [&str; 8] = [
    "<10µs", "<100µs", "<1ms", "<10ms", "<100ms", "<1s", "<10s", ">=10s",
];

#[derive(Debug, Default, Clone)]
pub struct TracesQuery {
    pub limit: usize,
    pub service: Option<String>,
    pub status: Option<String>,
    pub min_ms: Option<f64>,
    pub run_id: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TracesResponse {
    pub path: String,
    pub available: bool,
    pub jaeger_available: bool,
    pub traces: Vec<TraceSummary>,
    pub services: Vec<String>,
    pub topology: ServiceTopology,
    pub red: Vec<RedOperation>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub trace_id: String,
    pub root_service: String,
    pub root_name: String,
    pub root_kind: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub time_ms: u64,
    pub duration_ms: f64,
    pub span_count: usize,
    pub error_count: usize,
    pub status: String,
    pub services: Vec<String>,
    pub service_counts: Vec<ServiceSpanCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceSpanCount {
    pub service: String,
    pub count: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceDetail {
    pub trace_id: String,
    pub root_service: String,
    pub root_name: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub time_ms: u64,
    pub duration_ms: f64,
    pub critical_path_ms: f64,
    pub span_count: usize,
    pub error_count: usize,
    pub status: String,
    pub services: Vec<String>,
    pub service_counts: Vec<ServiceSpanCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub spans: Vec<SpanView>,
}

#[derive(Debug, Serialize)]
pub struct TraceCorrelatedResponse {
    pub trace: TraceDetail,
    pub logs: Vec<LogEntry>,
    pub issues: Vec<IssueSummary>,
    pub jaeger_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jaeger_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanView {
    pub trace_id: String,
    pub span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub offset_ms: f64,
    pub duration_ms: f64,
    pub self_time_ms: f64,
    pub status: String,
    pub status_message: String,
    pub service: String,
    pub scope: String,
    pub scope_version: String,
    pub resource_attrs: HashMap<String, String>,
    pub attrs: HashMap<String, String>,
    pub events: Vec<SpanEventView>,
    pub links: Vec<SpanLinkView>,
    pub depth: usize,
    pub orphan: bool,
    pub on_critical_path: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanEventView {
    pub name: String,
    pub time_ns: u64,
    pub offset_ms: f64,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanLinkView {
    pub trace_id: String,
    pub span_id: String,
    pub trace_state: String,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ServiceTopology {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopologyNode {
    pub service: String,
    pub spans: usize,
    pub errors: usize,
    pub error_rate_pct: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopologyEdge {
    pub caller: String,
    pub callee: String,
    pub calls: usize,
    pub errors: usize,
    pub error_rate_pct: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedOperation {
    pub service: String,
    pub operation: String,
    pub kind: String,
    pub count: usize,
    pub rate_per_sec: f64,
    pub errors: usize,
    pub error_rate_pct: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct TracezResponse {
    pub available: bool,
    pub bucket_labels: Vec<&'static str>,
    pub operations: Vec<TracezOperation>,
    pub proto_counts: Vec<TracezCounts>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TracezOperation {
    pub service: String,
    pub operation: String,
    pub total: usize,
    pub running_count: usize,
    pub error_count: usize,
    pub buckets: BTreeMap<String, u64>,
    pub bucket_counts: [u64; 8],
    pub sample_trace_ids: HashMap<String, String>,
    pub error_samples: Vec<TracezErrorSample>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TracezErrorSample {
    pub trace_id: String,
    pub span_id: String,
    pub duration_ms: f64,
    pub message: String,
}

pub fn read_traces(root: &Path, query: &TracesQuery) -> TracesResponse {
    let paths = RootPaths::new(root);
    let path = paths.traces_file();
    let path_s = path.display().to_string();
    let jaeger_available = crate::collector::probe().jaeger_ui;

    if !has_signal_file(&path) {
        return TracesResponse {
            path: path_s,
            available: false,
            jaeger_available,
            traces: vec![],
            services: vec![],
            topology: ServiceTopology::default(),
            red: vec![],
            note: "No traces file yet. Start hub with --collector (or POST to /v1/traces) and emit OTLP spans.".into(),
        };
    }

    match load_all_spans(&path) {
        Ok(raw_spans) => {
            let mut details = group_traces(raw_spans);
            if let Some(ref rid) = query.run_id {
                details.retain(|t| t.run_id.as_deref() == Some(rid.as_str()));
            }

            let all_services = collect_distinct_services(&details);
            let topology = compute_topology(&details);
            let red = compute_red(&details);

            if let Some(ref svc) = query.service
                && !svc.is_empty()
                && !svc.eq_ignore_ascii_case("ALL")
            {
                details.retain(|t| t.services.iter().any(|s| s == svc));
            }
            if let Some(ref st) = query.status
                && !st.is_empty()
                && !st.eq_ignore_ascii_case("ALL")
            {
                details.retain(|t| t.status.eq_ignore_ascii_case(st));
            }
            if let Some(min_ms) = query.min_ms
                && min_ms > 0.0
            {
                details.retain(|t| t.duration_ms >= min_ms);
            }
            if let Some(ref q) = query.search {
                let needle = q.trim().to_ascii_lowercase();
                if !needle.is_empty() {
                    details.retain(|t| trace_matches_query(t, &needle));
                }
            }

            details.sort_by_key(|b| std::cmp::Reverse(b.start_ns));
            let limit = query.limit.max(1);
            if details.len() > limit {
                details.truncate(limit);
            }

            let summaries: Vec<TraceSummary> = details.iter().map(detail_to_summary).collect();
            let note = if summaries.is_empty() {
                if query.run_id.is_some()
                    || query.service.is_some()
                    || query.status.is_some()
                    || query.min_ms.is_some()
                    || query.search.is_some()
                {
                    "No traces match the active filters.".into()
                } else {
                    "Traces file is empty or still buffering.".into()
                }
            } else {
                format!(
                    "{} trace(s) · {} service(s) · {} operation(s)",
                    summaries.len(),
                    all_services.len(),
                    red.len()
                )
            };

            TracesResponse {
                path: path_s,
                available: true,
                jaeger_available,
                traces: summaries,
                services: all_services,
                topology,
                red,
                note,
            }
        }
        Err(err) => TracesResponse {
            path: path_s,
            available: true,
            jaeger_available,
            traces: vec![],
            services: vec![],
            topology: ServiceTopology::default(),
            red: vec![],
            note: format!("Failed to read traces: {err}"),
        },
    }
}

pub fn read_trace_detail(root: &Path, trace_id: &str) -> Option<TraceDetail> {
    let wanted = trace_id.trim();
    if wanted.is_empty() {
        return None;
    }
    let paths = RootPaths::new(root);
    let path = paths.traces_file();
    if !has_signal_file(&path) {
        return None;
    }
    let raw_spans = load_all_spans(&path).ok()?;
    let matched: Vec<SpanView> = raw_spans
        .into_iter()
        .filter(|s| s.trace_id.eq_ignore_ascii_case(wanted))
        .collect();
    if matched.is_empty() {
        return None;
    }
    Some(reconstruct_trace(matched))
}

pub fn read_trace_detail_with_correlations(
    app: &HubApp,
    trace_id: &str,
) -> Option<TraceCorrelatedResponse> {
    let trace = read_trace_detail(&app.paths.root, trace_id)?;
    let logs =
        super::logs::read_logs_query(&app.paths.root, 200, None, Some(&trace.trace_id)).entries;
    let issues = app.issues.find_by_trace_id(&trace.trace_id);
    let jaeger_available = crate::collector::probe().jaeger_ui;
    let jaeger_url =
        jaeger_available.then(|| format!("http://127.0.0.1:16686/trace/{}", trace.trace_id));
    Some(TraceCorrelatedResponse {
        trace,
        logs,
        issues,
        jaeger_available,
        jaeger_url,
    })
}

pub fn read_tracez(root: &Path, service_filter: Option<&str>) -> TracezResponse {
    let paths = RootPaths::new(root);
    let path = paths.traces_file();
    if !has_signal_file(&path) {
        return TracezResponse {
            available: false,
            bucket_labels: TRACEZ_BUCKET_LABELS.to_vec(),
            operations: vec![],
            proto_counts: vec![],
            note: "No traces file yet.".into(),
        };
    }

    let Ok(raw_spans) = load_all_spans(&path) else {
        return TracezResponse {
            available: true,
            bucket_labels: TRACEZ_BUCKET_LABELS.to_vec(),
            operations: vec![],
            proto_counts: vec![],
            note: "Failed to parse traces file.".into(),
        };
    };

    let mut groups: HashMap<(String, String), TracezOperation> = HashMap::new();
    for s in raw_spans {
        if let Some(svc) = service_filter
            && !svc.is_empty()
            && !svc.eq_ignore_ascii_case("ALL")
            && s.service != svc
        {
            continue;
        }
        let key = (s.service.clone(), s.name.clone());
        let entry = groups.entry(key).or_insert_with(|| {
            let mut buckets = BTreeMap::new();
            for label in TRACEZ_BUCKET_LABELS {
                buckets.insert(label.to_string(), 0);
            }
            TracezOperation {
                service: s.service.clone(),
                operation: s.name.clone(),
                total: 0,
                running_count: 0,
                error_count: 0,
                buckets,
                bucket_counts: [0; 8],
                sample_trace_ids: HashMap::new(),
                error_samples: Vec::new(),
            }
        });

        entry.total += 1;
        if s.end_ns == 0 || s.end_ns < s.start_ns {
            entry.running_count += 1;
            continue;
        }

        let dur_ns = s.end_ns.saturating_sub(s.start_ns);
        let bucket_idx = tracez_bucket_index(dur_ns);
        let label = TRACEZ_BUCKET_LABELS[bucket_idx];
        entry.bucket_counts[bucket_idx] += 1;
        *entry.buckets.entry(label.to_string()).or_insert(0) += 1;
        entry
            .sample_trace_ids
            .entry(label.to_string())
            .or_insert_with(|| s.trace_id.clone());

        if s.status == "ERROR" {
            entry.error_count += 1;
            if entry.error_samples.len() < 5 {
                let msg = if !s.status_message.is_empty() {
                    s.status_message.clone()
                } else {
                    s.events
                        .iter()
                        .find_map(|ev| {
                            ev.attrs
                                .get("exception.message")
                                .or_else(|| ev.attrs.get("exception.type"))
                                .cloned()
                        })
                        .unwrap_or_else(|| "ERROR".into())
                };
                entry.error_samples.push(TracezErrorSample {
                    trace_id: s.trace_id.clone(),
                    span_id: s.span_id.clone(),
                    duration_ms: s.duration_ms,
                    message: msg,
                });
            }
        }
    }

    let mut operations: Vec<TracezOperation> = groups.into_values().collect();
    operations.sort_by(|a, b| {
        b.error_count
            .cmp(&a.error_count)
            .then_with(|| b.total.cmp(&a.total))
            .then_with(|| a.service.cmp(&b.service))
            .then_with(|| a.operation.cmp(&b.operation))
    });

    let proto_counts: Vec<TracezCounts> = operations
        .iter()
        .map(|op| TracezCounts {
            spanname: format!("{}/{}", op.service, op.operation),
            latency: op.bucket_counts.iter().map(|&c| c as u32).collect(),
            running: op.running_count as u32,
            error: op.error_count as u32,
        })
        .collect();

    let note = format!("{} zPages operation bucket(s)", operations.len());
    TracezResponse {
        available: true,
        bucket_labels: TRACEZ_BUCKET_LABELS.to_vec(),
        operations,
        proto_counts,
        note,
    }
}

fn tracez_bucket_index(dur_ns: u64) -> usize {
    if dur_ns < 10_000 {
        0 // <10µs
    } else if dur_ns < 100_000 {
        1 // <100µs
    } else if dur_ns < 1_000_000 {
        2 // <1ms
    } else if dur_ns < 10_000_000 {
        3 // <10ms
    } else if dur_ns < 100_000_000 {
        4 // <100ms
    } else if dur_ns < 1_000_000_000 {
        5 // <1s
    } else if dur_ns < 10_000_000_000 {
        6 // <10s
    } else {
        7 // >=10s
    }
}

fn detail_to_summary(d: &TraceDetail) -> TraceSummary {
    let root_kind = d
        .spans
        .first()
        .map(|s| s.kind.clone())
        .unwrap_or_else(|| "INTERNAL".into());
    TraceSummary {
        trace_id: d.trace_id.clone(),
        root_service: d.root_service.clone(),
        root_name: d.root_name.clone(),
        root_kind,
        start_ns: d.start_ns,
        end_ns: d.end_ns,
        time_ms: d.time_ms,
        duration_ms: d.duration_ms,
        span_count: d.span_count,
        error_count: d.error_count,
        status: d.status.clone(),
        services: d.services.clone(),
        service_counts: d.service_counts.clone(),
        run_id: d.run_id.clone(),
    }
}

fn trace_matches_query(t: &TraceDetail, needle: &str) -> bool {
    if t.trace_id.to_ascii_lowercase().contains(needle)
        || t.root_name.to_ascii_lowercase().contains(needle)
        || t.root_service.to_ascii_lowercase().contains(needle)
    {
        return true;
    }
    t.spans.iter().any(|s| {
        s.name.to_ascii_lowercase().contains(needle)
            || s.service.to_ascii_lowercase().contains(needle)
            || s.span_id.to_ascii_lowercase().contains(needle)
            || s.status_message.to_ascii_lowercase().contains(needle)
            || s.attrs.iter().any(|(k, v)| {
                k.to_ascii_lowercase().contains(needle) || v.to_ascii_lowercase().contains(needle)
            })
            || s.events.iter().any(|ev| {
                ev.name.to_ascii_lowercase().contains(needle)
                    || ev
                        .attrs
                        .values()
                        .any(|v| v.to_ascii_lowercase().contains(needle))
            })
    })
}

fn collect_distinct_services(traces: &[TraceDetail]) -> Vec<String> {
    let mut set = HashSet::new();
    for t in traces {
        for s in &t.services {
            if !s.is_empty() {
                set.insert(s.clone());
            }
        }
    }
    let mut out: Vec<String> = set.into_iter().collect();
    out.sort();
    out
}

fn group_traces(raw_spans: Vec<SpanView>) -> Vec<TraceDetail> {
    let mut by_trace: HashMap<String, Vec<SpanView>> = HashMap::new();
    for span in raw_spans {
        by_trace
            .entry(span.trace_id.clone())
            .or_default()
            .push(span);
    }
    by_trace
        .into_values()
        .filter(|spans| !spans.is_empty())
        .map(reconstruct_trace)
        .collect()
}

fn reconstruct_trace(raw_spans: Vec<SpanView>) -> TraceDetail {
    // Deduplicate spans by span_id (keep latest record if exported twice).
    let mut by_id: HashMap<String, SpanView> = HashMap::new();
    for s in raw_spans {
        by_id.insert(s.span_id.clone(), s);
    }
    let mut unordered: Vec<SpanView> = by_id.into_values().collect();
    unordered.sort_by(|a, b| {
        a.start_ns
            .cmp(&b.start_ns)
            .then_with(|| b.end_ns.cmp(&a.end_ns))
            .then_with(|| a.span_id.cmp(&b.span_id))
    });

    let trace_id = unordered
        .first()
        .map(|s| s.trace_id.clone())
        .unwrap_or_default();
    let trace_start_ns = unordered
        .iter()
        .map(|s| s.start_ns)
        .filter(|&n| n > 0)
        .min()
        .unwrap_or(0);
    let trace_end_ns = unordered
        .iter()
        .map(|s| s.end_ns.max(s.start_ns))
        .max()
        .unwrap_or(trace_start_ns);
    let trace_duration_ms = (trace_end_ns.saturating_sub(trace_start_ns)) as f64 / 1_000_000.0;

    let id_to_idx: HashMap<String, usize> = unordered
        .iter()
        .enumerate()
        .map(|(i, s)| (s.span_id.clone(), i))
        .collect();

    let n = unordered.len();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut roots: Vec<usize> = Vec::new();

    for (i, item) in unordered.iter_mut().enumerate() {
        let parent_opt = item
            .parent_span_id
            .as_ref()
            .and_then(|pid| id_to_idx.get(pid).copied())
            .filter(|&p| p != i);
        if let Some(p) = parent_opt {
            children[p].push(i);
        } else {
            if item.parent_span_id.is_some() {
                item.orphan = true;
            }
            roots.push(i);
        }
    }

    // Compute self_time_ms per span and offsets relative to trace_start_ns.
    for i in 0..n {
        let s_start = unordered[i].start_ns;
        let s_end = unordered[i].end_ns.max(s_start);
        unordered[i].offset_ms = (s_start.saturating_sub(trace_start_ns)) as f64 / 1_000_000.0;
        unordered[i].duration_ms = (s_end.saturating_sub(s_start)) as f64 / 1_000_000.0;

        let mut child_intervals: Vec<(u64, u64)> = children[i]
            .iter()
            .filter_map(|&c| {
                let cs = unordered[c].start_ns.max(s_start);
                let ce = unordered[c].end_ns.max(unordered[c].start_ns).min(s_end);
                (ce > cs).then_some((cs, ce))
            })
            .collect();
        let covered_ns = merged_interval_length(&mut child_intervals);
        let self_ns = s_end.saturating_sub(s_start).saturating_sub(covered_ns);
        unordered[i].self_time_ms = self_ns as f64 / 1_000_000.0;

        for ev in &mut unordered[i].events {
            ev.offset_ms = (ev.time_ns.saturating_sub(trace_start_ns)) as f64 / 1_000_000.0;
        }
    }

    // Compute subtree max end_ns for critical-path traversal (guards against cycles).
    let mut subtree_end = vec![0u64; n];
    let mut visiting = vec![false; n];
    for i in 0..n {
        if subtree_end[i] == 0 {
            compute_subtree_end(i, &unordered, &children, &mut subtree_end, &mut visiting);
        }
    }

    // Critical path: walk backwards across top-level root intervals, recursing into children.
    if n == 1 {
        unordered[0].on_critical_path = true;
    } else if !roots.is_empty() {
        let mut critical_visited = vec![false; n];
        walk_critical_siblings(
            &roots,
            trace_start_ns,
            trace_end_ns,
            &mut unordered,
            &children,
            &subtree_end,
            &mut critical_visited,
        );
    }

    // DFS pre-order traversal to produce clean Waterfall order + depths.
    let mut ordered: Vec<SpanView> = Vec::with_capacity(n);
    let mut visited = vec![false; n];
    for &r in &roots {
        dfs_order(r, 0, &unordered, &children, &mut visited, &mut ordered);
    }
    // Include any cycle-isolated spans so nothing is ever dropped.
    for i in 0..n {
        if !visited[i] {
            dfs_order(i, 0, &unordered, &children, &mut visited, &mut ordered);
        }
    }

    let root_span = ordered.first();
    let root_service = root_span
        .map(|s| s.service.clone())
        .unwrap_or_else(|| "unknown-service".into());
    let root_name = root_span
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "trace".into());

    let error_count = ordered.iter().filter(|s| s.status == "ERROR").count();
    let status = if error_count > 0 {
        "ERROR".into()
    } else {
        "OK".into()
    };

    let mut svc_map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut run_id: Option<String> = None;
    for s in &ordered {
        let entry = svc_map.entry(s.service.clone()).or_insert((0, 0));
        entry.0 += 1;
        if s.status == "ERROR" {
            entry.1 += 1;
        }
        if run_id.is_none() {
            run_id = s
                .attrs
                .get("airbug.run_id")
                .or_else(|| s.resource_attrs.get("airbug.run_id"))
                .cloned();
        }
    }
    let services: Vec<String> = svc_map.keys().cloned().collect();
    let service_counts: Vec<ServiceSpanCount> = svc_map
        .into_iter()
        .map(|(service, (count, errors))| ServiceSpanCount {
            service,
            count,
            errors,
        })
        .collect();

    // Critical path self-time sum.
    let critical_path_ms: f64 = ordered
        .iter()
        .filter(|s| s.on_critical_path)
        .map(|s| s.self_time_ms)
        .sum::<f64>()
        .min(trace_duration_ms.max(0.0));

    TraceDetail {
        trace_id,
        root_service,
        root_name,
        start_ns: trace_start_ns,
        end_ns: trace_end_ns,
        time_ms: trace_start_ns / 1_000_000,
        duration_ms: trace_duration_ms,
        critical_path_ms,
        span_count: ordered.len(),
        error_count,
        status,
        services,
        service_counts,
        run_id,
        spans: ordered,
    }
}

fn compute_subtree_end(
    u: usize,
    spans: &[SpanView],
    children: &[Vec<usize>],
    memo: &mut [u64],
    visiting: &mut [bool],
) -> u64 {
    if memo[u] > 0 {
        return memo[u];
    }
    if visiting[u] {
        return spans[u].end_ns.max(spans[u].start_ns);
    }
    visiting[u] = true;
    let mut mx = spans[u].end_ns.max(spans[u].start_ns);
    for &c in &children[u] {
        mx = mx.max(compute_subtree_end(c, spans, children, memo, visiting));
    }
    visiting[u] = false;
    memo[u] = mx;
    mx
}

fn walk_critical_siblings(
    siblings: &[usize],
    window_start: u64,
    window_end: u64,
    spans: &mut [SpanView],
    children: &[Vec<usize>],
    subtree_end: &[u64],
    visited: &mut [bool],
) {
    if siblings.is_empty() {
        return;
    }
    if window_end <= window_start {
        if let Some(&first) = siblings.first()
            && !visited[first]
        {
            visited[first] = true;
            spans[first].on_critical_path = true;
        }
        return;
    }

    let mut cursor = window_end;
    while cursor > window_start {
        // Pick the unvisited sibling whose effective end clamped to `cursor` is latest
        // and actually extends backward before `cursor`.
        let best = siblings
            .iter()
            .copied()
            .filter(|&idx| !visited[idx])
            .filter_map(|idx| {
                let s = spans[idx].start_ns.max(window_start);
                let e = subtree_end[idx].max(spans[idx].end_ns).min(cursor);
                (e > s).then_some((idx, s, e))
            })
            .max_by(|a, b| a.2.cmp(&b.2).then_with(|| (b.2 - b.1).cmp(&(a.2 - a.1))));

        let Some((chosen, s, e)) = best else {
            break;
        };
        visited[chosen] = true;
        spans[chosen].on_critical_path = true;
        if !children[chosen].is_empty() {
            walk_critical_siblings(
                &children[chosen],
                s,
                e,
                spans,
                children,
                subtree_end,
                visited,
            );
        }
        if s >= cursor {
            break;
        }
        cursor = s;
    }
}

fn dfs_order(
    u: usize,
    depth: usize,
    spans: &[SpanView],
    children: &[Vec<usize>],
    visited: &mut [bool],
    out: &mut Vec<SpanView>,
) {
    if visited[u] {
        return;
    }
    visited[u] = true;
    let mut view = spans[u].clone();
    view.depth = depth;
    out.push(view);

    let mut sorted_children = children[u].clone();
    sorted_children.sort_by(|&a, &b| {
        spans[a]
            .start_ns
            .cmp(&spans[b].start_ns)
            .then_with(|| spans[b].end_ns.cmp(&spans[a].end_ns))
    });
    for c in sorted_children {
        dfs_order(c, depth + 1, spans, children, visited, out);
    }
}

fn merged_interval_length(intervals: &mut [(u64, u64)]) -> u64 {
    if intervals.is_empty() {
        return 0;
    }
    intervals.sort_by_key(|&(s, _)| s);
    let mut total = 0u64;
    let (mut cur_s, mut cur_e) = intervals[0];
    for &(s, e) in &intervals[1..] {
        if s <= cur_e {
            cur_e = cur_e.max(e);
        } else {
            total = total.saturating_add(cur_e.saturating_sub(cur_s));
            cur_s = s;
            cur_e = e;
        }
    }
    total.saturating_add(cur_e.saturating_sub(cur_s))
}

fn compute_topology(traces: &[TraceDetail]) -> ServiceTopology {
    struct NodeAcc {
        spans: usize,
        errors: usize,
        durations: Vec<f64>,
    }
    struct EdgeAcc {
        calls: usize,
        errors: usize,
        durations: Vec<f64>,
    }

    let mut nodes: HashMap<String, NodeAcc> = HashMap::new();
    let mut edges: HashMap<(String, String), EdgeAcc> = HashMap::new();

    for t in traces {
        let by_id: HashMap<&str, &SpanView> =
            t.spans.iter().map(|s| (s.span_id.as_str(), s)).collect();
        for s in &t.spans {
            let node = nodes.entry(s.service.clone()).or_insert_with(|| NodeAcc {
                spans: 0,
                errors: 0,
                durations: Vec::new(),
            });
            node.spans += 1;
            if s.status == "ERROR" {
                node.errors += 1;
            }
            node.durations.push(s.duration_ms);

            if let Some(ref pid) = s.parent_span_id
                && let Some(parent) = by_id.get(pid.as_str())
                && parent.service != s.service
            {
                let edge = edges
                    .entry((parent.service.clone(), s.service.clone()))
                    .or_insert_with(|| EdgeAcc {
                        calls: 0,
                        errors: 0,
                        durations: Vec::new(),
                    });
                edge.calls += 1;
                if s.status == "ERROR" {
                    edge.errors += 1;
                }
                edge.durations.push(s.duration_ms);
            }
        }
    }

    let mut node_list: Vec<TopologyNode> = nodes
        .into_iter()
        .map(|(service, mut acc)| {
            acc.durations.sort_by(|a, b| a.total_cmp(b));
            let error_rate_pct = if acc.spans > 0 {
                (acc.errors as f64 / acc.spans as f64) * 100.0
            } else {
                0.0
            };
            TopologyNode {
                service,
                spans: acc.spans,
                errors: acc.errors,
                error_rate_pct,
                p50_ms: exact_percentile(&acc.durations, 0.50),
                p95_ms: exact_percentile(&acc.durations, 0.95),
                p99_ms: exact_percentile(&acc.durations, 0.99),
            }
        })
        .collect();
    node_list.sort_by(|a, b| {
        b.spans
            .cmp(&a.spans)
            .then_with(|| a.service.cmp(&b.service))
    });

    let mut edge_list: Vec<TopologyEdge> = edges
        .into_iter()
        .map(|((caller, callee), mut acc)| {
            acc.durations.sort_by(|a, b| a.total_cmp(b));
            let error_rate_pct = if acc.calls > 0 {
                (acc.errors as f64 / acc.calls as f64) * 100.0
            } else {
                0.0
            };
            TopologyEdge {
                caller,
                callee,
                calls: acc.calls,
                errors: acc.errors,
                error_rate_pct,
                p50_ms: exact_percentile(&acc.durations, 0.50),
                p95_ms: exact_percentile(&acc.durations, 0.95),
            }
        })
        .collect();
    edge_list.sort_by(|a, b| {
        b.calls
            .cmp(&a.calls)
            .then_with(|| a.caller.cmp(&b.caller))
            .then_with(|| a.callee.cmp(&b.callee))
    });

    ServiceTopology {
        nodes: node_list,
        edges: edge_list,
    }
}

fn compute_red(traces: &[TraceDetail]) -> Vec<RedOperation> {
    struct RedAcc {
        kind: String,
        count: usize,
        errors: usize,
        durations: Vec<f64>,
        min_start_ns: u64,
        max_end_ns: u64,
    }

    let mut map: HashMap<(String, String), RedAcc> = HashMap::new();
    let mut global_min_ns = u64::MAX;
    let mut global_max_ns = 0u64;

    for t in traces {
        let by_id: HashMap<&str, &SpanView> =
            t.spans.iter().map(|s| (s.span_id.as_str(), s)).collect();
        for s in &t.spans {
            if s.start_ns > 0 {
                global_min_ns = global_min_ns.min(s.start_ns);
            }
            global_max_ns = global_max_ns.max(s.end_ns.max(s.start_ns));

            // Entry / boundary operations: root spans, SERVER/CONSUMER spans, or cross-service entry spans.
            let is_service_entry = match &s.parent_span_id {
                None => true,
                Some(pid) => match by_id.get(pid.as_str()) {
                    None => true,
                    Some(parent) => parent.service != s.service,
                },
            };
            if !is_service_entry && s.kind != "SERVER" && s.kind != "CONSUMER" {
                continue;
            }

            let acc = map
                .entry((s.service.clone(), s.name.clone()))
                .or_insert_with(|| RedAcc {
                    kind: s.kind.clone(),
                    count: 0,
                    errors: 0,
                    durations: Vec::new(),
                    min_start_ns: s.start_ns,
                    max_end_ns: s.end_ns,
                });
            acc.count += 1;
            if s.status == "ERROR" {
                acc.errors += 1;
            }
            acc.durations.push(s.duration_ms);
            if s.start_ns > 0 {
                acc.min_start_ns = acc.min_start_ns.min(s.start_ns);
            }
            acc.max_end_ns = acc.max_end_ns.max(s.end_ns);
        }
    }

    let window_secs = if global_max_ns > global_min_ns {
        ((global_max_ns - global_min_ns) as f64 / 1_000_000_000.0).max(1.0)
    } else {
        1.0
    };

    let mut out: Vec<RedOperation> = map
        .into_iter()
        .map(|((service, operation), mut acc)| {
            acc.durations.sort_by(|a, b| a.total_cmp(b));
            let error_rate_pct = if acc.count > 0 {
                (acc.errors as f64 / acc.count as f64) * 100.0
            } else {
                0.0
            };
            let max_ms = acc.durations.last().copied().unwrap_or(0.0);
            RedOperation {
                service,
                operation,
                kind: acc.kind,
                count: acc.count,
                rate_per_sec: acc.count as f64 / window_secs,
                errors: acc.errors,
                error_rate_pct,
                p50_ms: exact_percentile(&acc.durations, 0.50),
                p95_ms: exact_percentile(&acc.durations, 0.95),
                p99_ms: exact_percentile(&acc.durations, 0.99),
                max_ms,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.errors
            .cmp(&a.errors)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.service.cmp(&b.service))
            .then_with(|| a.operation.cmp(&b.operation))
    });
    out
}

pub(crate) fn exact_percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let pos = q.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = pos - lo as f64;
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }
}

fn load_all_spans(path: &Path) -> std::io::Result<Vec<SpanView>> {
    let buf = tail_text(path, config::TRACES_TAIL_BYTES)?;
    let mut out = Vec::new();
    for chunk in split_json_values(&buf) {
        if let Ok(value) = serde_json::from_str::<Value>(chunk) {
            extract_spans(&value, &mut out);
        }
    }
    Ok(out)
}

fn extract_spans(value: &Value, out: &mut Vec<SpanView>) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            extract_spans(item, out);
        }
        return;
    }
    let Some(resource_spans) = value
        .get("resourceSpans")
        .or_else(|| value.get("resource_spans"))
        .and_then(Value::as_array)
    else {
        return;
    };

    for rs in resource_spans {
        let raw_svc = resource_attr(rs, "service.name");
        let service = if raw_svc.trim().is_empty() {
            "unknown-service".to_string()
        } else {
            raw_svc
        };
        let res_attrs = resource_attrs(rs);
        let Some(scopes) = rs
            .get("scopeSpans")
            .or_else(|| rs.get("scope_spans"))
            .and_then(Value::as_array)
        else {
            continue;
        };

        for ss in scopes {
            let scope_obj = ss.get("scope");
            let scope = scope_obj
                .and_then(|s| s.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let scope_version = scope_obj
                .and_then(|s| s.get("version"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();

            let Some(spans) = ss.get("spans").and_then(Value::as_array) else {
                continue;
            };

            for sp in spans {
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
                    .filter(|s| !s.is_empty())
                    .unwrap_or("unnamed-span")
                    .to_string();
                let kind = parse_span_kind(sp.get("kind"));
                let start_ns =
                    nano_u64(sp, &["startTimeUnixNano", "start_time_unix_nano"]).unwrap_or(0);
                let end_ns = nano_u64(sp, &["endTimeUnixNano", "end_time_unix_nano"]).unwrap_or(0);
                let (status, status_message) = parse_span_status(sp.get("status"));
                let attrs = attrs_to_map(sp.get("attributes"));

                let events = sp
                    .get("events")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .map(|ev| {
                                let ev_name = ev
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("event")
                                    .to_string();
                                let time_ns = nano_u64(ev, &["timeUnixNano", "time_unix_nano"])
                                    .unwrap_or(start_ns);
                                let ev_attrs = attrs_to_map(ev.get("attributes"));
                                SpanEventView {
                                    name: ev_name,
                                    time_ns,
                                    offset_ms: 0.0,
                                    attrs: ev_attrs,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let links = sp
                    .get("links")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|lk| {
                                let l_tid = lk
                                    .get("traceId")
                                    .or_else(|| lk.get("trace_id"))
                                    .and_then(|v| decode_id_hex(v, 16))?;
                                let l_sid = lk
                                    .get("spanId")
                                    .or_else(|| lk.get("span_id"))
                                    .and_then(|v| decode_id_hex(v, 8))
                                    .unwrap_or_default();
                                let trace_state = lk
                                    .get("traceState")
                                    .or_else(|| lk.get("trace_state"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string();
                                let l_attrs = attrs_to_map(lk.get("attributes"));
                                Some(SpanLinkView {
                                    trace_id: l_tid,
                                    span_id: l_sid,
                                    trace_state,
                                    attrs: l_attrs,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let duration_ms = if end_ns >= start_ns {
                    (end_ns - start_ns) as f64 / 1_000_000.0
                } else {
                    0.0
                };

                out.push(SpanView {
                    trace_id,
                    span_id,
                    parent_span_id,
                    name,
                    kind,
                    start_ns,
                    end_ns,
                    offset_ms: 0.0,
                    duration_ms,
                    self_time_ms: duration_ms,
                    status,
                    status_message,
                    service: service.clone(),
                    scope: scope.clone(),
                    scope_version: scope_version.clone(),
                    resource_attrs: res_attrs.clone(),
                    attrs,
                    events,
                    links,
                    depth: 0,
                    orphan: false,
                    on_critical_path: false,
                });
            }
        }
    }
}

fn parse_span_kind(val: Option<&Value>) -> String {
    let Some(v) = val else {
        return "INTERNAL".into();
    };
    if let Some(n) = parse_i64(v) {
        return match n {
            2 => "SERVER",
            3 => "CLIENT",
            4 => "PRODUCER",
            5 => "CONSUMER",
            _ => "INTERNAL",
        }
        .into();
    }
    if let Some(s) = v.as_str() {
        let up = s.to_ascii_uppercase();
        if up.contains("SERVER") {
            return "SERVER".into();
        }
        if up.contains("CLIENT") {
            return "CLIENT".into();
        }
        if up.contains("PRODUCER") {
            return "PRODUCER".into();
        }
        if up.contains("CONSUMER") {
            return "CONSUMER".into();
        }
    }
    "INTERNAL".into()
}

fn parse_span_status(val: Option<&Value>) -> (String, String) {
    let Some(st) = val else {
        return ("UNSET".into(), String::new());
    };
    let msg = st
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let code_val = st.get("code");
    let code = if let Some(v) = code_val {
        if let Some(n) = parse_i64(v) {
            match n {
                1 => "OK",
                2 => "ERROR",
                _ => "UNSET",
            }
        } else if let Some(s) = v.as_str() {
            let up = s.to_ascii_uppercase();
            if up.contains("ERROR") {
                "ERROR"
            } else if up.contains("OK") {
                "OK"
            } else {
                "UNSET"
            }
        } else {
            "UNSET"
        }
    } else {
        "UNSET"
    };
    (code.into(), msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    #[test]
    fn reconstructs_waterfall_critical_path_topology_red_and_tracez() {
        let dir = env::temp_dir().join(format!("airbug-traces-engine-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let data = dir.join("dash/collector/data");
        fs::create_dir_all(&data).unwrap();

        let tid = "5b8efff798038103d269b633813fc60c";
        let root_sid = "1111111111111111";
        let fast_cache_sid = "2222222222222222";
        let slow_auth_sid = "3333333333333333";
        let db_sid = "4444444444444444";

        // Base = 1_000_000_000 ns
        // root (gateway, SERVER): [0ms, 100ms]
        //   ├─ cache.get (redis, CLIENT): [5ms, 15ms] (parallel fast branch - NOT critical)
        //   ├─ auth.verify (auth-svc, SERVER): [5ms, 40ms] (critical)
        //   └─ db.charge (payments-db, CLIENT, ERROR): [45ms, 95ms] (critical)
        let doc = serde_json::json!({
            "resourceSpans": [
                {
                    "resource": {
                        "attributes": [
                            {"key": "service.name", "value": {"stringValue": "gateway"}},
                            {"key": "airbug.run_id", "value": {"stringValue": "run-xyz"}}
                        ]
                    },
                    "scopeSpans": [{
                        "scope": {"name": "http.router", "version": "1.0.0"},
                        "spans": [{
                            "traceId": tid,
                            "spanId": root_sid,
                            "parentSpanId": "0000000000000000",
                            "name": "POST /checkout",
                            "kind": 2,
                            "startTimeUnixNano": "1000000000",
                            "endTimeUnixNano": "1100000000",
                            "status": {"code": 2, "message": "upstream failure"},
                            "attributes": [{"key": "http.method", "value": {"stringValue": "POST"}}]
                        }]
                    }]
                },
                {
                    "resource": {
                        "attributes": [{"key": "service.name", "value": {"stringValue": "redis"}}]
                    },
                    "scopeSpans": [{
                        "scope": {"name": "cache"},
                        "spans": [{
                            "traceId": tid,
                            "spanId": fast_cache_sid,
                            "parentSpanId": root_sid,
                            "name": "GET session",
                            "kind": "SPAN_KIND_CLIENT",
                            "startTimeUnixNano": "1005000000",
                            "endTimeUnixNano": "1015000000",
                            "status": {"code": 1}
                        }]
                    }]
                },
                {
                    "resource": {
                        "attributes": [{"key": "service.name", "value": {"stringValue": "auth-svc"}}]
                    },
                    "scopeSpans": [{
                        "scope": {"name": "jwt"},
                        "spans": [{
                            "traceId": tid,
                            "spanId": slow_auth_sid,
                            "parentSpanId": root_sid,
                            "name": "VerifyToken",
                            "kind": 2,
                            "startTimeUnixNano": "1005000000",
                            "endTimeUnixNano": "1040000000",
                            "status": {"code": 1}
                        }]
                    }]
                },
                {
                    "resource": {
                        "attributes": [{"key": "service.name", "value": {"stringValue": "payments-db"}}]
                    },
                    "scopeSpans": [{
                        "scope": {"name": "pg"},
                        "spans": [{
                            "traceId": tid,
                            "spanId": db_sid,
                            "parentSpanId": root_sid,
                            "name": "INSERT ledger",
                            "kind": 3,
                            "startTimeUnixNano": "1045000000",
                            "endTimeUnixNano": "1095000000",
                            "status": {"code": 2, "message": "deadlock detected"},
                            "events": [{
                                "name": "exception",
                                "timeUnixNano": "1090000000",
                                "attributes": [
                                    {"key": "exception.type", "value": {"stringValue": "PgDeadlock"}},
                                    {"key": "exception.message", "value": {"stringValue": "deadlock on accounts"}},
                                    {"key": "exception.stacktrace", "value": {"stringValue": "at ledger.rs:42"}}
                                ]
                            }],
                            "links": [{
                                "traceId": "aaaabbbbccccddddeeeeffff00001111",
                                "spanId": "9999888877776666",
                                "traceState": "vendor=1"
                            }]
                        }]
                    }]
                }
            ]
        });
        fs::write(data.join("traces.json"), format!("{doc}\n")).unwrap();

        let list = read_traces(
            &dir,
            &TracesQuery {
                limit: 50,
                ..Default::default()
            },
        );
        assert!(list.available);
        assert_eq!(list.traces.len(), 1);
        assert_eq!(list.traces[0].status, "ERROR");
        assert_eq!(list.traces[0].error_count, 2);
        assert_eq!(list.traces[0].run_id.as_deref(), Some("run-xyz"));
        assert_eq!(list.services.len(), 4);

        // Topology: gateway -> redis, gateway -> auth-svc, gateway -> payments-db
        assert_eq!(list.topology.nodes.len(), 4);
        assert_eq!(list.topology.edges.len(), 3);
        let db_edge = list
            .topology
            .edges
            .iter()
            .find(|e| e.caller == "gateway" && e.callee == "payments-db")
            .unwrap();
        assert_eq!(db_edge.calls, 1);
        assert_eq!(db_edge.errors, 1);
        assert!((db_edge.p95_ms - 50.0).abs() < 1e-6);

        // RED metrics
        let gw_red = list
            .red
            .iter()
            .find(|r| r.service == "gateway" && r.operation == "POST /checkout")
            .unwrap();
        assert_eq!(gw_red.count, 1);
        assert_eq!(gw_red.errors, 1);
        assert!((gw_red.p95_ms - 100.0).abs() < 1e-6);

        // Detailed waterfall + critical path
        let detail = read_trace_detail(&dir, tid).unwrap();
        assert_eq!(detail.spans.len(), 4);
        assert_eq!(detail.spans[0].span_id, root_sid);
        assert_eq!(detail.spans[0].depth, 0);
        assert!(detail.spans[0].on_critical_path);
        // Root total = 100ms; children cover [5,40] (35ms) + [45,95] (50ms) = 85ms => self_time = 15ms
        assert!((detail.spans[0].self_time_ms - 15.0).abs() < 1e-5);

        let cache_sp = detail
            .spans
            .iter()
            .find(|s| s.span_id == fast_cache_sid)
            .unwrap();
        assert_eq!(cache_sp.depth, 1);
        assert!(
            !cache_sp.on_critical_path,
            "fast parallel cache should not be on critical path"
        );

        let auth_sp = detail
            .spans
            .iter()
            .find(|s| s.span_id == slow_auth_sid)
            .unwrap();
        assert!(
            auth_sp.on_critical_path,
            "slower parallel auth should be on critical path"
        );

        let db_sp = detail.spans.iter().find(|s| s.span_id == db_sid).unwrap();
        assert!(db_sp.on_critical_path);
        assert_eq!(db_sp.events.len(), 1);
        assert_eq!(db_sp.links.len(), 1);
        assert!((db_sp.events[0].offset_ms - 90.0).abs() < 1e-5);

        // zPages Tracez
        let tz = read_tracez(&dir, None);
        assert!(tz.available);
        assert_eq!(tz.operations.len(), 4);
        assert_eq!(tz.proto_counts.len(), 4);
        let db_tz = tz
            .operations
            .iter()
            .find(|o| o.service == "payments-db")
            .unwrap();
        assert_eq!(db_tz.buckets["<100ms"], 1);
        assert_eq!(db_tz.error_count, 1);
        assert_eq!(db_tz.error_samples.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
