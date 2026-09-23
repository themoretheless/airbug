//! Read OTLP metric points written by the local collector file exporter.
use super::{any_value, nano_time, resource_attr, resource_attrs, split_json_values, tail_text};
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

pub fn read_recent(root: &Path, limit: usize) -> MetricsResponse {
    read_filtered(root, limit, None)
}

pub fn read_filtered(root: &Path, limit: usize, run_id: Option<&str>) -> MetricsResponse {
    let paths = RootPaths::new(root);
    let path = paths.metrics_file();
    let path_s = path.display().to_string();
    if !path.is_file() {
        return MetricsResponse {
            path: path_s,
            available: false,
            points: vec![],
            latest: vec![],
            series: vec![],
            histogram: empty_histogram(),
            note: "No metrics file yet. Start hub with --collector and emit OTLP metrics (e.g. airbug-mon --otlp).".into(),
        };
    }

    match tail_parse(&path, limit.saturating_mul(4).max(limit)) {
        Ok(mut points) => {
            if let Some(rid) = run_id {
                points.retain(|p| p.attr_map.get("airbug.run_id").map(|s| s.as_str()) == Some(rid));
            }
            if points.len() > limit {
                points = points.split_off(points.len() - limit);
            }
            let latest = latest_by_name(&points);
            let series = build_series(&points);
            let histogram = build_histogram(&points, 16);
            MetricsResponse {
                path: path_s,
                available: true,
                note: if points.is_empty() {
                    if run_id.is_some() {
                        "No metrics for this airbug.run_id yet.".into()
                    } else {
                        "Metrics file is empty or still buffering.".into()
                    }
                } else {
                    format!(
                        "{} recent point(s), {} series, {} unique name(s)",
                        points.len(),
                        series.len(),
                        latest.len()
                    )
                },
                points,
                latest,
                series,
                histogram,
            }
        }
        Err(err) => MetricsResponse {
            path: path_s,
            available: true,
            points: vec![],
            latest: vec![],
            series: vec![],
            histogram: empty_histogram(),
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
        s.points.dedup_by(|a, b| a.t == b.t);
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
    let mut map: HashMap<String, MetricPoint> = HashMap::new();
    for p in points {
        map.insert(p.name.clone(), p.clone());
    }
    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn tail_parse(path: &Path, limit: usize) -> std::io::Result<Vec<MetricPoint>> {
    let buf = tail_text(path, config::METRICS_TAIL_BYTES)?;
    let mut points = Vec::new();
    for chunk in split_json_values(&buf) {
        if let Ok(value) = serde_json::from_str::<Value>(chunk) {
            extract_points(&value, &mut points);
        }
    }
    if points.len() > limit {
        points = points.split_off(points.len() - limit);
    }
    Ok(points)
}

fn extract_points(value: &Value, out: &mut Vec<MetricPoint>) {
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
                for (kind, key) in [
                    ("gauge", "gauge"),
                    ("sum", "sum"),
                    ("histogram", "histogram"),
                    ("exponential_histogram", "exponentialHistogram"),
                    ("summary", "summary"),
                ] {
                    if let Some(block) = metric.get(key).or_else(|| {
                        metric.get(match key {
                            "exponentialHistogram" => "exponential_histogram",
                            other => other,
                        })
                    }) {
                        push_data_points(
                            block,
                            kind,
                            &name,
                            &unit,
                            &service,
                            &scope,
                            &base_attrs,
                            out,
                        );
                    }
                }
            }
        }
    }
}

fn push_data_points(
    block: &Value,
    kind: &str,
    name: &str,
    unit: &str,
    service: &str,
    scope: &str,
    base_attrs: &HashMap<String, String>,
    out: &mut Vec<MetricPoint>,
) {
    let points = block
        .get("dataPoints")
        .or_else(|| block.get("data_points"))
        .and_then(|v| v.as_array());
    let Some(points) = points else {
        return;
    };
    for dp in points {
        let (time, time_ms) = nano_time(dp, &["timeUnixNano", "time_unix_nano"]);
        let value = point_value(dp);
        let mut attr_map = base_attrs.clone();
        if let Some(attrs) = dp.get("attributes").and_then(|v| v.as_array()) {
            for a in attrs {
                let Some(k) = a.get("key").and_then(|v| v.as_str()) else {
                    continue;
                };
                attr_map.insert(
                    k.to_string(),
                    a.get("value").map(any_value).unwrap_or_default(),
                );
            }
        }
        let attrs = attr_map
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ");
        out.push(MetricPoint {
            time,
            time_ms,
            name: name.to_string(),
            kind: kind.to_string(),
            value,
            unit: unit.to_string(),
            service: service.to_string(),
            scope: scope.to_string(),
            attrs,
            attr_map,
        });
    }
}

fn point_value(dp: &Value) -> f64 {
    if let Some(n) = dp
        .get("asDouble")
        .or_else(|| dp.get("as_double"))
        .and_then(|v| v.as_f64())
    {
        return n;
    }
    if let Some(n) = dp.get("asInt").or_else(|| dp.get("as_int")).and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    }) {
        return n as f64;
    }
    if let Some(n) = dp
        .get("count")
        .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|u| u as f64)))
    {
        return n;
    }
    if let Some(n) = dp.get("sum").and_then(|v| v.as_f64()) {
        return n;
    }
    0.0
}

fn point_attrs(dp: &Value) -> String {
    let attrs = dp
        .get("attributes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    attrs
        .iter()
        .filter_map(|a| {
            let k = a.get("key")?.as_str()?;
            let v = a.get("value").map(any_value).unwrap_or_default();
            Some(format!("{k}={v}"))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

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
                attr_map: HashMap::new(),
            })
            .collect();
        let h = build_histogram(&points, 5);
        assert_eq!(h.bucket_count, 5);
        assert_eq!(h.counts.iter().sum::<u64>(), 10);
    }
}
