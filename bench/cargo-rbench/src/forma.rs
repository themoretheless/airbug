use rbench::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
fn string<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error(format!("Forma: missing string {k}")))
}
fn integer(v: &Value, k: &str) -> Result<u64> {
    v.get(k)
        .and_then(Value::as_u64)
        .filter(|n| *n > 0)
        .ok_or_else(|| error(format!("Forma: missing positive integer {k}")))
}
pub fn import(path: &Path) -> Result<Run> {
    let meta_path = path.join("metadata.json");
    let data_path = path.join("results.json");
    let meta: Value = serde_json::from_reader(fs::File::open(&meta_path)?)?;
    let data: Vec<Value> = serde_json::from_reader(fs::File::open(&data_path)?)?;
    let paired = meta.get("binaries").is_some();
    if paired && meta.get("status").and_then(Value::as_str) != Some("complete") {
        return Err(error(
            "Forma paired import requires metadata.status=complete",
        ));
    }
    let repeats = integer(&meta, "repeats")?;
    let frames = integer(&meta, "frames")?;
    if repeats > 10000 || frames > 1_000_000 {
        return Err(error("Forma counts exceed import limits"));
    }
    let mut run = Run::new();
    run.environment.clear();
    for k in ["platform", "osRelease", "arch", "cpu", "rust"] {
        run.environment.insert(k.into(), string(&meta, k)?.into());
    }
    for k in ["logicalCpus", "systemRamBytes"] {
        run.environment
            .insert(k.into(), integer(&meta, k)?.to_string());
    }
    run.provenance
        .insert("source".into(), "forma-v1-summary-import".into());
    run.provenance
        .insert("original_metadata".into(), serde_json::to_string(&meta)?);
    run.provenance
        .insert("metadata_sha256".into(), model::hash_file(&meta_path)?);
    run.provenance
        .insert("results_sha256".into(), model::hash_file(&data_path)?);
    let fixtures = if paired {
        let o = meta
            .get("fixtures")
            .and_then(Value::as_object)
            .ok_or_else(|| error("Forma paired fixtures missing"))?;
        o.iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    v.as_str()
                        .filter(|s| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()))
                        .ok_or_else(|| error("invalid fixture SHA"))?
                        .to_string(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?
    } else {
        let mut fixtures = BTreeMap::new();
        for e in fs::read_dir(path)? {
            let p = e?.path();
            if p.extension().is_some_and(|e| e == "ui") {
                fixtures.insert(
                    p.file_name().unwrap().to_string_lossy().into_owned(),
                    model::hash_file(&p)?,
                );
            }
        }
        fixtures
    };
    let mut expected = vec![
        ("image", "gpu", "animation", 800, 400),
        ("image", "gpu", "animation", 1920, 1080),
        ("image", "gpu", "animation", 3840, 2160),
        ("image", "gpu", "resize", 3840, 2160),
        ("text", "gpu", "animation", 1920, 1080),
        ("nested", "gpu", "forced", 1920, 1080),
    ];
    if !paired {
        expected.extend([
            ("image", "gpu", "forced", 3840, 2160),
            ("image", "gpu", "idle", 800, 400),
            ("image", "cpu", "animation", 800, 400),
            ("image", "cpu", "animation", 3840, 2160),
            ("image", "cpu", "resize", 3840, 2160),
        ]);
    }
    let expected: BTreeSet<_> = expected
        .into_iter()
        .map(|(s, b, m, w, h)| format!("{s}/{b}/{m}/{w}x{h}@2"))
        .collect();
    let expected_count = expected.len() as u64 * repeats * if paired { 2 } else { 1 };
    if data.len() as u64 != expected_count {
        return Err(error(format!(
            "Forma incomplete case set: expected {expected_count} process rows, found {}",
            data.len()
        )));
    }
    let mut seen = BTreeSet::new();
    let mut cases: BTreeMap<String, Case> = BTreeMap::new();
    for (process, row) in data.iter().enumerate() {
        let scene = string(row, "scene")?;
        let backend = string(row, "backend")?;
        let mode = string(row, "mode")?;
        let width = integer(row, "width")?;
        let height = integer(row, "height")?;
        if row.get("scale").and_then(Value::as_f64) != Some(2.0) {
            return Err(error("Forma v1 requires DPI 2"));
        }
        let id = format!("{scene}/{backend}/{mode}/{width}x{height}@2");
        if !expected.contains(&id) {
            return Err(error(format!("unexpected Forma case {id}")));
        }
        let repetition = integer(row, "repetition")?;
        if repetition > repeats {
            return Err(error("invalid repetition"));
        }
        if row.get("frames").and_then(Value::as_u64)
            != Some(if mode == "idle" { 0 } else { frames })
        {
            return Err(error("Forma frame count mismatch"));
        }
        let variant = if paired {
            match string(row, "variant")? {
                "A" => "baseline",
                "B" => "candidate",
                _ => return Err(error("invalid Forma variant")),
            }
        } else {
            "candidate"
        };
        if !seen.insert((id.clone(), repetition, variant)) {
            return Err(error("duplicate Forma process row"));
        }
        let mut contract = BTreeMap::new();
        for k in ["backend", "mode", "adapter"] {
            contract.insert(k.into(), string(row, k)?.into());
        }
        contract.insert("frames".into(), frames.to_string());
        contract.insert("warmup_frames".into(), "20".into());
        contract.insert("source_semantics".into(), "forma-summary-v1".into());
        for name in [format!("{scene}.ui"), format!("{scene}.template.ui")] {
            contract.insert(
                name.clone(),
                fixtures
                    .get(&name)
                    .ok_or_else(|| error(format!("missing fixture {name}")))?
                    .clone(),
            );
        }
        let mut metrics = vec![];
        let fields = [
            (
                "render_throughput_fps",
                "/render_throughput_fps",
                "frames/s",
                "serialized offscreen; no window/present",
                "process throughput",
                Direction::Higher,
            ),
            (
                "completed.p95",
                "/completed_ms/p95",
                "ms",
                "frame including GPU wait (CPU fallback excludes upload/present)",
                "per-process frame p95",
                Direction::Lower,
            ),
            (
                "cpu.submit.mean",
                "/cpu_submit_ms/mean",
                "ms",
                "tick/draw/submit wall time",
                "per-process frame mean",
                Direction::Lower,
            ),
            (
                "gpu.pass.mean",
                "/gpu_pass/mean_ms",
                "ms",
                "separate 30-frame profiled renderer; resize uses last static size",
                "per-process pass mean",
                Direction::Lower,
            ),
            (
                "cpu.utilization",
                "/cpu_percent_one_core",
                "percent",
                "all process threads; one core=100%",
                "process ratio",
                Direction::Neutral,
            ),
            (
                "rss.after",
                "/rss_after_bytes",
                "bytes",
                "whole process RSS after phase; not additive to GPU bytes",
                "gauge",
                Direction::Lower,
            ),
            (
                "alloc.requested",
                "/rust_requested_bytes",
                "bytes",
                "Rust requested allocation traffic; native driver excluded",
                "phase total",
                Direction::Lower,
            ),
            (
                "alloc.calls",
                "/rust_allocations",
                "calls",
                "successful Rust alloc calls; excludes realloc",
                "phase total",
                Direction::Lower,
            ),
            (
                "alloc.realloc",
                "/rust_reallocations",
                "calls",
                "successful Rust realloc calls",
                "phase total",
                Direction::Lower,
            ),
            (
                "alloc.live.after",
                "/rust_live_after_bytes",
                "bytes",
                "live requested Rust bytes",
                "gauge",
                Direction::Lower,
            ),
            (
                "gpu.owned_buffer",
                "/owned_gpu_buffer_bytes",
                "bytes",
                "renderer-owned buffers; not total VRAM",
                "gauge",
                Direction::Lower,
            ),
        ];
        for (metric, pointer, unit, scope, statistic, direction) in fields {
            let not_applicable = (mode == "idle"
                && [
                    "render_throughput_fps",
                    "completed.p95",
                    "cpu.submit.mean",
                    "gpu.pass.mean",
                ]
                .contains(&metric))
                || (backend == "cpu" && ["gpu.pass.mean", "gpu.owned_buffer"].contains(&metric));
            let value = row.pointer(pointer).filter(|v| !v.is_null());
            let availability = if not_applicable {
                Availability::NotApplicable("not measured by this workload".into())
            } else if value.is_none() {
                Availability::Incomplete("missing source metric; no value fabricated".into())
            } else {
                Availability::Available
            };
            let value = if availability == Availability::Available {
                let v = value.unwrap();
                let n = v
                    .as_f64()
                    .ok_or_else(|| error(format!("invalid metric {metric}")))?;
                if !n.is_finite() || n < 0.0 {
                    return Err(error("invalid Forma numeric value"));
                }
                Some(v.to_string())
            } else {
                None
            };
            metrics.push(Metric {
                id: metric.into(),
                unit: unit.into(),
                scope: scope.into(),
                phase: if metric == "gpu.pass.mean" {
                    "gpu_diagnostic"
                } else {
                    "measurement"
                }
                .into(),
                statistic: statistic.into(),
                direction,
            });
            run.observations.push(Observation {
                case: id.clone(),
                metric: metric.into(),
                variant: variant.into(),
                process: process as u32,
                pair: paired.then_some((repetition - 1) as u32),
                sequence: 0,
                value,
                operations: 1,
                availability,
            });
        }
        let c = Case {
            id: id.clone(),
            contract,
            metrics,
        };
        if let Some(old) = cases.get(&id) {
            if old != &c {
                return Err(error("Forma adapter/contract mismatch across repetitions"));
            }
        } else {
            cases.insert(id, c);
        }
    }
    run.cases = cases.into_values().collect();
    run.status = Status::Complete;
    run.notes.push("Imported process summaries, not raw frame arrays. Median of process p95 is not pooled p95. Source completeness verified against Forma v1 case matrix; original runs were not rerun. GPU phase is separate; CPU fallback excludes presentation. RSS and GPU bytes overlap on UMA.".into());
    run.validate()?;
    Ok(run)
}
