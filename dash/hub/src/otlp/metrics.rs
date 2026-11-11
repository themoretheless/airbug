//! Read OTLP metric points, native Histograms / ExponentialHistograms / Summaries,
//! interpolated quantiles (`p50`/`p90`/`p95`/`p99`), and trace-linked Exemplars.
use super::{
    any_value, attrs_to_map, decode_id_hex, has_signal_file, nano_time, parse_f64, parse_i64,
    parse_u64, resource_attr, resource_attrs, split_json_values, tail_text,
};
use crate::config::{self, RootPaths};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub path: String,
    pub available: bool,
    pub points: Vec<MetricPoint>,
    pub latest: Vec<MetricPoint>,
    pub series: Vec<MetricSeries>,
    pub histogram: MetricHistogram,
    pub percentiles: Vec<HistogramSummary>,
    pub exemplars: Vec<MetricExemplar>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricHistogram {
    pub bucket_count: usize,
    pub min: f64,
    pub max: f64,
    pub edges: Vec<f64>,
    pub counts: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricPoint {
    pub time: String,
    pub time_ms: Option<u64>,
    pub name: String,
    pub kind: String,
    pub value: f64,
    pub unit: String,
    pub service: String,
    pub scope: String,
    pub attrs: String,
    pub temporality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monotonic: Option<bool>,
    #[serde(default)]
    pub attr_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSeries {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub service: String,
    pub attrs: String,
    pub points: Vec<SeriesPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeriesPoint {
    pub t: u64,
    pub v: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistogramSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub unit: String,
    pub service: String,
    pub scope: String,
    pub attrs: String,
    pub temporality: String,
    pub time_ms: Option<u64>,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub buckets: Vec<HistogramBucketBar>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistogramBucketBar {
    pub lower: f64,
    pub upper: f64,
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricExemplar {
    pub metric: String,
    pub kind: String,
    pub unit: String,
    pub service: String,
    pub value: f64,
    pub time: String,
    pub time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default)]
    pub attrs: HashMap<String, String>,
}

#[derive(Default)]
struct ExtractedMetrics {
    points: Vec<MetricPoint>,
    raw_histograms: Vec<RawHistogramSample>,
    exemplars: Vec<MetricExemplar>,
}

#[derive(Clone)]
struct RawHistogramSample {
    summary: HistogramSummary,
    attr_map: HashMap<String, String>,
}

pub fn read_filtered(root: &Path, limit: usize, run_id: Option<&str>) -> MetricsResponse {
    let paths = RootPaths::new(root);
    let path = paths.metrics_file();
    let path_s = path.display().to_string();
    if !has_signal_file(&path) {
        return MetricsResponse {
            path: path_s,
            available: false,
            points: vec![],
            latest: vec![],
            series: vec![],
            histogram: empty_histogram(),
            percentiles: vec![],
            exemplars: vec![],
            note: "No metrics file yet. Start hub with --collector (or POST to /v1/metrics) and emit OTLP metrics.".into(),
        };
    }

    match tail_parse(&path, limit.saturating_mul(4).max(limit)) {
        Ok(mut extracted) => {
            if let Some(rid) = run_id {
                extracted
                    .points
                    .retain(|p| p.attr_map.get("airbug.run_id").map(String::as_str) == Some(rid));
                extracted
                    .raw_histograms
                    .retain(|h| h.attr_map.get("airbug.run_id").map(String::as_str) == Some(rid));
                extracted
                    .exemplars
                    .retain(|e| e.attrs.get("airbug.run_id").map(String::as_str) == Some(rid));
            }
            if extracted.points.len() > limit {
                let drop_n = extracted.points.len() - limit;
                extracted.points.drain(..drop_n);
            }
            if extracted.exemplars.len() > limit {
                let drop_n = extracted.exemplars.len() - limit;
                extracted.exemplars.drain(..drop_n);
            }
            let points = extracted.points;
            let latest = latest_by_name(&points);
            let series = build_series(&points);
            let histogram = build_histogram(&points, 16);
            let percentiles = finalize_histogram_summaries(extracted.raw_histograms);
            let exemplars = extracted.exemplars;

            MetricsResponse {
                path: path_s,
                available: true,
                note: if points.is_empty() && percentiles.is_empty() {
                    if run_id.is_some() {
                        "No metrics for this airbug.run_id yet.".into()
                    } else {
                        "Metrics file is empty or still buffering.".into()
                    }
                } else {
                    let mut parts = vec![format!(
                        "{} recent point(s), {} series, {} unique name(s)",
                        points.len(),
                        series.len(),
                        latest.len()
                    )];
                    if !percentiles.is_empty() {
                        parts.push(format!("{} histogram(s)", percentiles.len()));
                    }
                    if !exemplars.is_empty() {
                        parts.push(format!("{} exemplar(s)", exemplars.len()));
                    }
                    parts.join(" · ")
                },
                points,
                latest,
                series,
                histogram,
                percentiles,
                exemplars,
            }
        }
        Err(err) => MetricsResponse {
            path: path_s,
            available: true,
            points: vec![],
            latest: vec![],
            series: vec![],
            histogram: empty_histogram(),
            percentiles: vec![],
            exemplars: vec![],
            note: format!("Failed to read metrics: {err}"),
        },
    }
}

pub(crate) fn empty_histogram() -> MetricHistogram {
    MetricHistogram {
        bucket_count: 0,
        min: 0.0,
        max: 0.0,
        edges: vec![],
        counts: vec![],
    }
}

pub(crate) fn build_histogram(points: &[MetricPoint], buckets: usize) -> MetricHistogram {
    let buckets = buckets.clamp(2, 64);
    let values: Vec<f64> = points
        .iter()
        .map(|p| p.value)
        .filter(|v| v.is_finite())
        .collect();
    if values.is_empty() {
        return empty_histogram();
    }
    let mut min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let mut max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        min -= 1.0;
        max += 1.0;
    }
    let width = (max - min) / buckets as f64;
    let mut counts = vec![0u64; buckets];
    for v in &values {
        let mut idx = ((v - min) / width).floor() as usize;
        if idx >= buckets {
            idx = buckets - 1;
        }
        counts[idx] += 1;
    }
    let mut edges = Vec::with_capacity(buckets + 1);
    for i in 0..=buckets {
        edges.push(min + width * i as f64);
    }
    MetricHistogram {
        bucket_count: buckets,
        min,
        max,
        edges,
        counts,
    }
}

fn build_series(points: &[MetricPoint]) -> Vec<MetricSeries> {
    let mut map: HashMap<String, MetricSeries> = HashMap::new();
    for p in points {
        let Some(t) = p.time_ms else {
            continue;
        };
        let id = format!("{}|{}|{}", p.name, p.service, p.attrs);
        let entry = map.entry(id.clone()).or_insert_with(|| MetricSeries {
            id: id.clone(),
            name: p.name.clone(),
            unit: p.unit.clone(),
            service: p.service.clone(),
            attrs: p.attrs.clone(),
            points: Vec::new(),
        });
        entry.points.push(SeriesPoint { t, v: p.value });
    }
    let mut series: Vec<_> = map.into_values().collect();
    for s in &mut series {
        s.points.sort_by_key(|p| p.t);
        s.points.dedup_by(|a, b| {
            if a.t == b.t {
                b.v = a.v;
                true
            } else {
                false
            }
        });
    }
    series.sort_by_key(|s| {
        let rank = match s.name.as_str() {
            "system.cpu.utilization" => 0,
            "system.memory.usage" => 1,
            "system.memory.limit" => 2,
            "system.network.io" => 3,
            "system.disk.io" => 4,
            "system.gpu.utilization" => 5,
            _ => 50,
        };
        (rank, s.name.clone(), s.attrs.clone())
    });
    series
}

fn latest_by_name(points: &[MetricPoint]) -> Vec<MetricPoint> {
    let mut map: HashMap<(String, String), MetricPoint> = HashMap::new();
    for p in points {
        map.insert((p.name.clone(), p.attrs.clone()), p.clone());
    }
    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.attrs.cmp(&b.attrs)));
    out
}

fn finalize_histogram_summaries(samples: Vec<RawHistogramSample>) -> Vec<HistogramSummary> {
    let mut map: HashMap<String, HistogramSummary> = HashMap::new();
    for sample in samples {
        let h = sample.summary;
        match map.get_mut(&h.id) {
            Some(existing)
                if h.temporality == "delta"
                    && existing.temporality == "delta"
                    && existing.kind == h.kind
                    && existing.buckets.len() == h.buckets.len()
                    && existing.buckets.iter().zip(&h.buckets).all(|(a, b)| {
                        (a.lower - b.lower).abs() < 1e-9 && (a.upper - b.upper).abs() < 1e-9
                    }) =>
            {
                existing.count = existing.count.saturating_add(h.count);
                existing.sum = match (existing.sum, h.sum) {
                    (Some(a), Some(b)) => Some(a + b),
                    (a, b) => a.or(b),
                };
                existing.min = match (existing.min, h.min) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
                existing.max = match (existing.max, h.max) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
                existing.time_ms = h.time_ms.or(existing.time_ms);
                for (dst, src) in existing.buckets.iter_mut().zip(&h.buckets) {
                    dst.count = dst.count.saturating_add(src.count);
                }
                let intervals: Vec<(f64, f64, u64)> = existing
                    .buckets
                    .iter()
                    .map(|b| (b.lower, b.upper, b.count))
                    .collect();
                existing.p50 = interpolate_quantile(&intervals, 0.50, existing.min, existing.max);
                existing.p90 = interpolate_quantile(&intervals, 0.90, existing.min, existing.max);
                existing.p95 = interpolate_quantile(&intervals, 0.95, existing.min, existing.max);
                existing.p99 = interpolate_quantile(&intervals, 0.99, existing.min, existing.max);
            }
            _ => {
                map.insert(h.id.clone(), h);
            }
        }
    }
    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.attrs.cmp(&b.attrs)));
    for h in &mut out {
        if h.buckets.len() > 36 {
            h.buckets = compact_bucket_bars(&h.buckets, 32);
        }
    }
    out
}

fn compact_bucket_bars(bars: &[HistogramBucketBar], max_bars: usize) -> Vec<HistogramBucketBar> {
    if bars.len() <= max_bars || max_bars == 0 {
        return bars.to_vec();
    }
    let step = bars.len().div_ceil(max_bars);
    bars.chunks(step)
        .map(|chunk| {
            let lower = chunk.first().map(|b| b.lower).unwrap_or(0.0);
            let upper = chunk.last().map(|b| b.upper).unwrap_or(0.0);
            let count: u64 = chunk.iter().map(|b| b.count).sum();
            HistogramBucketBar {
                lower,
                upper,
                label: format_bucket_label(lower, upper),
                count,
            }
        })
        .collect()
}

fn tail_parse(path: &Path, limit: usize) -> std::io::Result<ExtractedMetrics> {
    let buf = tail_text(path, config::METRICS_TAIL_BYTES)?;
    let mut out = ExtractedMetrics::default();
    for chunk in split_json_values(&buf) {
        if let Ok(value) = serde_json::from_str::<Value>(chunk) {
            extract_points(&value, &mut out);
        }
    }
    if out.points.len() > limit {
        let drop_n = out.points.len() - limit;
        out.points.drain(..drop_n);
    }
    Ok(out)
}

fn extract_points(value: &Value, out: &mut ExtractedMetrics) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            extract_points(item, out);
        }
        return;
    }

    let Some(resource_metrics) = value
        .get("resourceMetrics")
        .or_else(|| value.get("resource_metrics"))
        .and_then(|v| v.as_array())
    else {
        return;
    };

    for rm in resource_metrics {
        let service = resource_attr(rm, "service.name");
        let base_attrs = resource_attrs(rm);
        let scopes = rm
            .get("scopeMetrics")
            .or_else(|| rm.get("scope_metrics"))
            .and_then(|v| v.as_array());
        let Some(scopes) = scopes else { continue };
        for sm in scopes {
            let scope = sm
                .get("scope")
                .and_then(|s| s.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let metrics = sm.get("metrics").and_then(|v| v.as_array());
            let Some(metrics) = metrics else { continue };
            for metric in metrics {
                let name = metric
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let unit = metric
                    .get("unit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let data_container = metric.get("data").unwrap_or(metric);
                for (kind, key) in [
                    ("gauge", "gauge"),
                    ("sum", "sum"),
                    ("histogram", "histogram"),
                    ("exponential_histogram", "exponentialHistogram"),
                    ("summary", "summary"),
                ] {
                    if let Some(block) = data_container.get(key).or_else(|| {
                        data_container.get(match key {
                            "exponentialHistogram" => "exponential_histogram",
                            other => other,
                        })
                    }) {
                        push_data_points(
                            block,
                            kind,
                            &PointSource {
                                name: &name,
                                unit: &unit,
                                service: &service,
                                scope: &scope,
                                base_attrs: &base_attrs,
                            },
                            out,
                        );
                    }
                }
            }
        }
    }
}

/// Where a point came from: the same labels apply to every data point of one metric block.
struct PointSource<'a> {
    name: &'a str,
    unit: &'a str,
    service: &'a str,
    scope: &'a str,
    base_attrs: &'a HashMap<String, String>,
}

fn parse_temporality(block: &Value) -> String {
    let Some(raw) = block
        .get("aggregationTemporality")
        .or_else(|| block.get("aggregation_temporality"))
    else {
        return String::new();
    };
    if let Some(n) = parse_i64(raw) {
        return match n {
            1 => "delta".into(),
            2 => "cumulative".into(),
            _ => String::new(),
        };
    }
    if let Some(s) = raw.as_str() {
        let up = s.to_ascii_uppercase();
        if up.contains("DELTA") {
            return "delta".into();
        }
        if up.contains("CUMULATIVE") {
            return "cumulative".into();
        }
    }
    String::new()
}

fn parse_monotonic(block: &Value) -> Option<bool> {
    block
        .get("isMonotonic")
        .or_else(|| block.get("is_monotonic"))
        .and_then(Value::as_bool)
}

fn push_data_points(block: &Value, kind: &str, src: &PointSource<'_>, out: &mut ExtractedMetrics) {
    let points = block
        .get("dataPoints")
        .or_else(|| block.get("data_points"))
        .and_then(|v| v.as_array());
    let Some(points) = points else {
        return;
    };
    let temporality = parse_temporality(block);
    let monotonic = parse_monotonic(block);

    for dp in points {
        let (time, time_ms) = nano_time(dp, &["timeUnixNano", "time_unix_nano"]);
        let value = point_value(dp);
        let mut attr_map = src.base_attrs.clone();
        for (k, v) in attrs_to_map(dp.get("attributes")) {
            attr_map.insert(k, v);
        }
        let mut attr_pairs: Vec<String> = attr_map
            .iter()
            .filter(|(k, _)| *k != "service.name")
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        attr_pairs.sort();
        let attrs = attr_pairs.join(" ");

        extract_exemplars(dp, kind, src, &attr_map, &mut out.exemplars);

        match kind {
            "histogram" => {
                if let Some(summary) =
                    build_explicit_histogram_summary(dp, src, &attrs, &temporality, time_ms)
                {
                    out.raw_histograms.push(RawHistogramSample {
                        summary,
                        attr_map: attr_map.clone(),
                    });
                }
            }
            "exponential_histogram" => {
                if let Some(summary) =
                    build_exponential_histogram_summary(dp, src, &attrs, &temporality, time_ms)
                {
                    out.raw_histograms.push(RawHistogramSample {
                        summary,
                        attr_map: attr_map.clone(),
                    });
                }
            }
            "summary" => {
                if let Some(summary) =
                    build_summary_percentiles(dp, src, &attrs, &temporality, time_ms)
                {
                    out.raw_histograms.push(RawHistogramSample {
                        summary,
                        attr_map: attr_map.clone(),
                    });
                }
            }
            _ => {}
        }

        out.points.push(MetricPoint {
            time,
            time_ms,
            name: src.name.to_string(),
            kind: kind.to_string(),
            value,
            unit: src.unit.to_string(),
            service: src.service.to_string(),
            scope: src.scope.to_string(),
            attrs,
            temporality: temporality.clone(),
            monotonic,
            attr_map,
        });
    }
}

fn extract_exemplars(
    dp: &Value,
    kind: &str,
    src: &PointSource<'_>,
    point_attrs: &HashMap<String, String>,
    out: &mut Vec<MetricExemplar>,
) {
    let Some(exemplars) = dp.get("exemplars").and_then(Value::as_array) else {
        return;
    };
    for ex in exemplars {
        let (time, time_ms) = nano_time(ex, &["timeUnixNano", "time_unix_nano"]);
        let value = ex
            .get("asDouble")
            .or_else(|| ex.get("as_double"))
            .and_then(parse_f64)
            .or_else(|| {
                ex.get("asInt")
                    .or_else(|| ex.get("as_int"))
                    .and_then(parse_f64)
            })
            .or_else(|| ex.get("value").and_then(parse_f64))
            .unwrap_or(0.0);
        let trace_id = ex
            .get("traceId")
            .or_else(|| ex.get("trace_id"))
            .and_then(|v| decode_id_hex(v, 16));
        let span_id = ex
            .get("spanId")
            .or_else(|| ex.get("span_id"))
            .and_then(|v| decode_id_hex(v, 8));
        let mut attrs = point_attrs.clone();
        let filtered = ex
            .get("filteredAttributes")
            .or_else(|| ex.get("filtered_attributes"));
        for (k, v) in attrs_to_map(filtered) {
            attrs.insert(k, v);
        }
        out.push(MetricExemplar {
            metric: src.name.to_string(),
            kind: kind.to_string(),
            unit: src.unit.to_string(),
            service: src.service.to_string(),
            value,
            time,
            time_ms,
            trace_id,
            span_id,
            attrs,
        });
    }
}

fn build_explicit_histogram_summary(
    dp: &Value,
    src: &PointSource<'_>,
    attrs: &str,
    temporality: &str,
    time_ms: Option<u64>,
) -> Option<HistogramSummary> {
    let counts: Vec<u64> = dp
        .get("bucketCounts")
        .or_else(|| dp.get("bucket_counts"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(parse_u64).collect())
        .unwrap_or_default();
    let bounds: Vec<f64> = dp
        .get("explicitBounds")
        .or_else(|| dp.get("explicit_bounds"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(parse_f64).collect())
        .unwrap_or_default();
    let sum = dp.get("sum").and_then(parse_f64);
    let min = dp.get("min").and_then(parse_f64);
    let max = dp.get("max").and_then(parse_f64);
    let total_from_buckets: u64 = counts.iter().sum();
    let count = dp
        .get("count")
        .and_then(parse_u64)
        .unwrap_or(total_from_buckets);

    if counts.is_empty() && count == 0 {
        return None;
    }

    let mut intervals: Vec<(f64, f64, u64)> = Vec::new();
    if bounds.is_empty() {
        let lo = min
            .or_else(|| (count > 0).then(|| sum.unwrap_or(0.0) / count as f64))
            .unwrap_or(0.0);
        let hi = max.unwrap_or(lo);
        let c = counts.first().copied().unwrap_or(count);
        intervals.push((lo, hi, c));
    } else {
        let first_bound = bounds[0];
        let default_first_width = if bounds.len() >= 2 {
            (bounds[1] - bounds[0]).max(1e-6)
        } else {
            first_bound.abs().max(1.0)
        };
        let lo0 = match min {
            Some(m) if m <= first_bound => m,
            _ if first_bound >= 0.0 => 0.0,
            _ => first_bound - default_first_width,
        };
        intervals.push((lo0, first_bound, counts.first().copied().unwrap_or(0)));
        for i in 1..bounds.len() {
            let lo = bounds[i - 1];
            let hi = bounds[i].max(lo);
            intervals.push((lo, hi, counts.get(i).copied().unwrap_or(0)));
        }
        if counts.len() > bounds.len() {
            let last_bound = *bounds.last().unwrap_or(&0.0);
            let last_width = if bounds.len() >= 2 {
                (bounds[bounds.len() - 1] - bounds[bounds.len() - 2]).max(1e-6)
            } else {
                last_bound.abs().max(1.0)
            };
            let hi_last = match max {
                Some(mx) if mx >= last_bound => mx,
                _ => last_bound + last_width,
            };
            intervals.push((last_bound, hi_last, counts[bounds.len()]));
        }
    }

    let p50 = interpolate_quantile(&intervals, 0.50, min, max);
    let p90 = interpolate_quantile(&intervals, 0.90, min, max);
    let p95 = interpolate_quantile(&intervals, 0.95, min, max);
    let p99 = interpolate_quantile(&intervals, 0.99, min, max);

    let buckets = intervals
        .iter()
        .map(|&(lower, upper, c)| HistogramBucketBar {
            lower,
            upper,
            label: format_bucket_label(lower, upper),
            count: c,
        })
        .collect();

    Some(HistogramSummary {
        id: format!("{}|{}|{}", src.name, src.service, attrs),
        name: src.name.to_string(),
        kind: "histogram".into(),
        unit: src.unit.to_string(),
        service: src.service.to_string(),
        scope: src.scope.to_string(),
        attrs: attrs.to_string(),
        temporality: temporality.to_string(),
        time_ms,
        count,
        sum,
        min,
        max,
        p50,
        p90,
        p95,
        p99,
        buckets,
    })
}

fn build_exponential_histogram_summary(
    dp: &Value,
    src: &PointSource<'_>,
    attrs: &str,
    temporality: &str,
    time_ms: Option<u64>,
) -> Option<HistogramSummary> {
    let scale = dp.get("scale").and_then(parse_i64).unwrap_or(0) as i32;
    let scale_clamped = scale.clamp(-20, 20);
    let log_base = std::f64::consts::LN_2 * 2f64.powi(-scale_clamped);

    let zero_count = dp
        .get("zeroCount")
        .or_else(|| dp.get("zero_count"))
        .and_then(parse_u64)
        .unwrap_or(0);
    let zero_threshold = dp
        .get("zeroThreshold")
        .or_else(|| dp.get("zero_threshold"))
        .and_then(parse_f64)
        .unwrap_or(0.0)
        .abs();

    let parse_exp_buckets = |key: &str| -> (i32, Vec<u64>) {
        let Some(b) = dp.get(key) else {
            return (0, Vec::new());
        };
        let offset = b.get("offset").and_then(parse_i64).unwrap_or(0) as i32;
        let counts = b
            .get("bucketCounts")
            .or_else(|| b.get("bucket_counts"))
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(parse_u64).collect())
            .unwrap_or_default();
        (offset, counts)
    };

    let (neg_offset, neg_counts) = parse_exp_buckets("negative");
    let (pos_offset, pos_counts) = parse_exp_buckets("positive");

    let sum = dp.get("sum").and_then(parse_f64);
    let min = dp.get("min").and_then(parse_f64);
    let max = dp.get("max").and_then(parse_f64);

    let mut intervals: Vec<(f64, f64, u64)> = Vec::new();
    // Negative buckets in ascending value order (most negative first).
    for (idx, &c) in neg_counts.iter().enumerate().rev() {
        let k = neg_offset + idx as i32;
        let lo = -(log_base * (k + 1) as f64).exp();
        let hi = -(log_base * k as f64).exp();
        intervals.push((lo, hi, c));
    }
    if zero_count > 0 || (neg_counts.is_empty() && pos_counts.is_empty()) {
        intervals.push((-zero_threshold, zero_threshold, zero_count));
    }
    for (idx, &c) in pos_counts.iter().enumerate() {
        let k = pos_offset + idx as i32;
        let lo = (log_base * k as f64).exp();
        let hi = (log_base * (k + 1) as f64).exp();
        intervals.push((lo, hi, c));
    }

    let bucket_sum: u64 = intervals.iter().map(|x| x.2).sum();
    let count = dp.get("count").and_then(parse_u64).unwrap_or(bucket_sum);
    if intervals.is_empty() && count == 0 {
        return None;
    }

    let p50 = interpolate_quantile(&intervals, 0.50, min, max);
    let p90 = interpolate_quantile(&intervals, 0.90, min, max);
    let p95 = interpolate_quantile(&intervals, 0.95, min, max);
    let p99 = interpolate_quantile(&intervals, 0.99, min, max);

    let buckets = intervals
        .iter()
        .map(|&(lower, upper, c)| HistogramBucketBar {
            lower,
            upper,
            label: format_bucket_label(lower, upper),
            count: c,
        })
        .collect();

    Some(HistogramSummary {
        id: format!("{}|{}|{}", src.name, src.service, attrs),
        name: src.name.to_string(),
        kind: "exponential_histogram".into(),
        unit: src.unit.to_string(),
        service: src.service.to_string(),
        scope: src.scope.to_string(),
        attrs: attrs.to_string(),
        temporality: temporality.to_string(),
        time_ms,
        count,
        sum,
        min,
        max,
        p50,
        p90,
        p95,
        p99,
        buckets,
    })
}

fn build_summary_percentiles(
    dp: &Value,
    src: &PointSource<'_>,
    attrs: &str,
    temporality: &str,
    time_ms: Option<u64>,
) -> Option<HistogramSummary> {
    let qvs = dp
        .get("quantileValues")
        .or_else(|| dp.get("quantile_values"))
        .and_then(Value::as_array)?;
    let mut pairs: Vec<(f64, f64)> = qvs
        .iter()
        .filter_map(|item| {
            let q = item.get("quantile").and_then(parse_f64).unwrap_or(0.0);
            let v = item.get("value").and_then(parse_f64)?;
            Some((q, v))
        })
        .collect();
    if pairs.is_empty() {
        return None;
    }
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));

    let lookup_q = |target: f64| -> f64 {
        if pairs.len() == 1 {
            return pairs[0].1;
        }
        if target <= pairs[0].0 {
            return pairs[0].1;
        }
        if target >= pairs[pairs.len() - 1].0 {
            return pairs[pairs.len() - 1].1;
        }
        for w in pairs.windows(2) {
            let (q0, v0) = w[0];
            let (q1, v1) = w[1];
            if target >= q0 && target <= q1 {
                if (q1 - q0).abs() < 1e-12 {
                    return v1;
                }
                let t = (target - q0) / (q1 - q0);
                return v0 + t * (v1 - v0);
            }
        }
        pairs[pairs.len() - 1].1
    };

    let count = dp.get("count").and_then(parse_u64).unwrap_or(0);
    let sum = dp.get("sum").and_then(parse_f64);
    let min = pairs.iter().find(|(q, _)| *q <= 0.001).map(|(_, v)| *v);
    let max = pairs
        .iter()
        .rev()
        .find(|(q, _)| *q >= 0.999)
        .map(|(_, v)| *v);

    Some(HistogramSummary {
        id: format!("{}|{}|{}", src.name, src.service, attrs),
        name: src.name.to_string(),
        kind: "summary".into(),
        unit: src.unit.to_string(),
        service: src.service.to_string(),
        scope: src.scope.to_string(),
        attrs: attrs.to_string(),
        temporality: temporality.to_string(),
        time_ms,
        count,
        sum,
        min,
        max,
        p50: lookup_q(0.50),
        p90: lookup_q(0.90),
        p95: lookup_q(0.95),
        p99: lookup_q(0.99),
        buckets: vec![],
    })
}

pub(crate) fn interpolate_quantile(
    intervals: &[(f64, f64, u64)],
    q: f64,
    min: Option<f64>,
    max: Option<f64>,
) -> f64 {
    let total: u64 = intervals.iter().map(|&(_, _, c)| c).sum();
    if total == 0 {
        return min.or(max).unwrap_or(0.0);
    }
    let q = q.clamp(0.0, 1.0);
    let rank = q * (total as f64);
    let mut cum: u64 = 0;
    let mut est = intervals.last().map(|x| x.1).unwrap_or(0.0);

    for &(lo, hi, c) in intervals {
        if c == 0 {
            continue;
        }
        let prev = cum as f64;
        cum = cum.saturating_add(c);
        let next = cum as f64;
        if rank <= next || cum == total {
            let frac = ((rank - prev) / (c as f64)).clamp(0.0, 1.0);
            est = lo + frac * (hi - lo);
            break;
        }
    }

    if let Some(m) = min
        && m.is_finite()
    {
        est = est.max(m);
    }
    if let Some(mx) = max
        && mx.is_finite()
    {
        est = est.min(mx);
    }
    est
}

fn format_bucket_label(lower: f64, upper: f64) -> String {
    fn fmt_num(v: f64) -> String {
        let abs = v.abs();
        if abs == 0.0 {
            "0".into()
        } else if abs >= 100.0 {
            format!("{v:.0}")
        } else if abs >= 10.0 {
            format!("{v:.1}")
        } else if abs >= 1.0 {
            format!("{v:.2}")
        } else {
            format!("{v:.3}")
        }
    }
    format!("[{}, {}]", fmt_num(lower), fmt_num(upper))
}

fn point_value(dp: &Value) -> f64 {
    if let Some(n) = dp
        .get("asDouble")
        .or_else(|| dp.get("as_double"))
        .and_then(parse_f64)
    {
        return n;
    }
    if let Some(n) = dp
        .get("asInt")
        .or_else(|| dp.get("as_int"))
        .and_then(parse_f64)
    {
        return n;
    }
    let count = dp.get("count").and_then(parse_u64);
    let sum = dp.get("sum").and_then(parse_f64);
    if let (Some(s), Some(c)) = (sum, count)
        && c > 0
    {
        return s / (c as f64);
    }
    if let Some(c) = count {
        return c as f64;
    }
    if let Some(s) = sum {
        return s;
    }
    let _ = any_value;
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    #[test]
    fn histogram_buckets() {
        let points: Vec<MetricPoint> = (0..10)
            .map(|i| MetricPoint {
                time: String::new(),
                time_ms: Some(i),
                name: "x".into(),
                kind: "gauge".into(),
                value: i as f64,
                unit: String::new(),
                service: String::new(),
                scope: String::new(),
                attrs: String::new(),
                temporality: String::new(),
                monotonic: None,
                attr_map: HashMap::new(),
            })
            .collect();
        let h = build_histogram(&points, 5);
        assert_eq!(h.bucket_count, 5);
        assert_eq!(h.counts.iter().sum::<u64>(), 10);
    }

    #[test]
    fn explicit_histogram_quantiles_and_exemplars() {
        let dir = env::temp_dir().join(format!("airbug-metrics-hist-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let data = dir.join("dash/collector/data");
        fs::create_dir_all(&data).unwrap();

        let doc = serde_json::json!({
            "resourceMetrics": [{
                "resource": {
                    "attributes": [{"key": "service.name", "value": {"stringValue": "api"}}]
                },
                "scopeMetrics": [{
                    "scope": {"name": "http"},
                    "metrics": [{
                        "name": "http.server.duration",
                        "unit": "ms",
                        "histogram": {
                            "aggregationTemporality": 2,
                            "dataPoints": [{
                                "timeUnixNano": "1700000005000000000",
                                "count": "100",
                                "sum": 2450.0,
                                "min": 2.0,
                                "max": 95.0,
                                "explicitBounds": [10.0, 20.0, 50.0, 100.0],
                                "bucketCounts": ["0", "50", "45", "5", "0"],
                                "attributes": [{"key": "http.route", "value": {"stringValue": "/pay"}}],
                                "exemplars": [{
                                    "timeUnixNano": "1700000004500000000",
                                    "asDouble": 88.4,
                                    "traceId": "5b8efff798038103d269b633813fc60c",
                                    "spanId": "eee19b7ec3c1b174",
                                    "filteredAttributes": [{"key": "user.tier", "value": {"stringValue": "pro"}}]
                                }]
                            }]
                        }
                    }, {
                        "name": "http.server.requests",
                        "unit": "1",
                        "sum": {
                            "aggregationTemporality": "AGGREGATION_TEMPORALITY_DELTA",
                            "isMonotonic": true,
                            "dataPoints": [{
                                "timeUnixNano": "1700000005000000000",
                                "asInt": "100"
                            }]
                        }
                    }]
                }]
            }]
        });
        fs::write(data.join("metrics.json"), format!("{doc}\n")).unwrap();

        let resp = read_filtered(&dir, 100, None);
        assert!(resp.available);
        assert_eq!(resp.percentiles.len(), 1);
        let h = &resp.percentiles[0];
        assert_eq!(h.name, "http.server.duration");
        assert_eq!(h.temporality, "cumulative");
        assert_eq!(h.count, 100);
        assert!((h.p50 - 20.0).abs() < 1e-6, "p50 was {}", h.p50);
        assert!((h.p95 - 50.0).abs() < 1e-6, "p95 was {}", h.p95);
        assert!((h.p99 - 90.0).abs() < 1e-6, "p99 was {}", h.p99);

        assert_eq!(resp.exemplars.len(), 1);
        let ex = &resp.exemplars[0];
        assert_eq!(
            ex.trace_id.as_deref(),
            Some("5b8efff798038103d269b633813fc60c")
        );
        assert_eq!(ex.span_id.as_deref(), Some("eee19b7ec3c1b174"));
        assert_eq!(ex.attrs.get("user.tier").map(String::as_str), Some("pro"));

        let sum_pt = resp
            .points
            .iter()
            .find(|p| p.name == "http.server.requests")
            .unwrap();
        assert_eq!(sum_pt.temporality, "delta");
        assert_eq!(sum_pt.monotonic, Some(true));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn exponential_histogram_and_summary_quantiles() {
        let dir = env::temp_dir().join(format!("airbug-metrics-exphist-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let data = dir.join("dash/collector/data");
        fs::create_dir_all(&data).unwrap();

        // scale = 0 => base = 2.0; offset = 0 => buckets [1,2], [2,4], [4,8], [8,16]
        let doc = serde_json::json!({
            "resourceMetrics": [{
                "resource": {
                    "attributes": [{"key": "service.name", "value": {"stringValue": "db"}}]
                },
                "scopeMetrics": [{
                    "scope": {"name": "sql"},
                    "metrics": [{
                        "name": "db.query.duration",
                        "unit": "ms",
                        "exponentialHistogram": {
                            "aggregationTemporality": 2,
                            "dataPoints": [{
                                "timeUnixNano": "1700000005000000000",
                                "count": 100,
                                "sum": 310.0,
                                "scale": 0,
                                "zeroCount": 0,
                                "positive": {
                                    "offset": 0,
                                    "bucketCounts": [50, 30, 15, 5]
                                }
                            }]
                        }
                    }, {
                        "name": "rpc.summary",
                        "unit": "ms",
                        "summary": {
                            "dataPoints": [{
                                "timeUnixNano": "1700000005000000000",
                                "count": 40,
                                "sum": 400.0,
                                "quantileValues": [
                                    {"quantile": 0.5, "value": 8.0},
                                    {"quantile": 0.9, "value": 18.0},
                                    {"quantile": 0.99, "value": 42.0}
                                ]
                            }]
                        }
                    }]
                }]
            }]
        });
        fs::write(data.join("metrics.json"), format!("{doc}\n")).unwrap();

        let resp = read_filtered(&dir, 100, None);
        assert_eq!(resp.percentiles.len(), 2);

        let exp = resp
            .percentiles
            .iter()
            .find(|p| p.name == "db.query.duration")
            .unwrap();
        assert_eq!(exp.kind, "exponential_histogram");
        // 50 out of 100 in [1, 2] => p50 == 2.0
        assert!((exp.p50 - 2.0).abs() < 1e-5, "exp p50 was {}", exp.p50);
        // 50 + 30 + 15 = 95 => p95 == 8.0
        assert!((exp.p95 - 8.0).abs() < 1e-5, "exp p95 was {}", exp.p95);
        // 99th is 4/5 through [8, 16] => 8 + 0.8 * 8 = 14.4
        assert!((exp.p99 - 14.4).abs() < 1e-5, "exp p99 was {}", exp.p99);

        let sum = resp
            .percentiles
            .iter()
            .find(|p| p.name == "rpc.summary")
            .unwrap();
        assert_eq!(sum.kind, "summary");
        assert!((sum.p50 - 8.0).abs() < 1e-6);
        assert!((sum.p90 - 18.0).abs() < 1e-6);
        assert!((sum.p99 - 42.0).abs() < 1e-6);

        let _ = fs::remove_dir_all(&dir);
    }
}
