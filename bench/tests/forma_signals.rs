//! In-workspace verification of Forma signal contracts (`Frames` lifecycle, GPU span rejection,
//! and stacked `TrackingAllocator<memory::Allocator>` allocation stack capture).
use airbug_bench::frames::{self, Frames, GpuSpan};
use airbug_bench::{Availability, Run};
use std::collections::BTreeMap;

#[cfg(feature = "memory")]
#[global_allocator]
static ALLOC: airbug_bench::alloc::TrackingAllocator<
    airbug_bench::memory::Allocator<std::alloc::System>,
> = airbug_bench::alloc::TrackingAllocator::new(airbug_bench::memory::Allocator(
    std::alloc::System,
));

fn obs<'a>(run: &'a Run, metric: &str) -> Vec<&'a airbug_bench::Observation> {
    run.observations
        .iter()
        .filter(|o| o.metric == metric)
        .collect()
}

static TEST_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn frames_lifecycle_and_gpu_span_contracts() {
    let _guard = TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut frames = Frames::new(
        "forma/synthetic",
        BTreeMap::from([("mode".into(), "window-lifecycle".into())]),
        2,
        true,
    )
    .unwrap()
    .backlog(2);

    // 2 warmup frames + 3 measured frames.
    let f0 = frames.begin();
    frames.submitted(f0).unwrap();
    frames.presented(f0, true).unwrap();
    frames.completed(f0).unwrap();
    frames
        .gpu(GpuSpan {
            frame: f0,
            begin_ticks: 100,
            end_ticks: 200,
            period_ns: 1.0,
        })
        .unwrap();

    let f1 = frames.begin();
    frames.submitted(f1).unwrap();
    frames.presented(f1, false).unwrap();
    frames.completed(f1).unwrap();
    // Missing GPU span on warmup f1 is tolerated without emitting measured Incomplete.

    // Measured frame 2: valid lifecycle + valid GPU span.
    let f2 = frames.begin();
    frames.submitted(f2).unwrap();
    frames.presented(f2, true).unwrap();
    frames.completed(f2).unwrap();
    frames
        .gpu(GpuSpan {
            frame: f2,
            begin_ticks: 1_000,
            end_ticks: 2_500,
            period_ns: 2.0,
        })
        .unwrap();

    // Measured frame 3: duplicate / backwards / NaN GPU span rejections become Availability::Invalid, never 0.
    let f3 = frames.begin();
    frames.submitted(f3).unwrap();
    frames.presented(f3, true).unwrap();
    frames.completed(f3).unwrap();
    frames
        .gpu(GpuSpan {
            frame: f3,
            begin_ticks: 5_000,
            end_ticks: 4_000,
            period_ns: 1.0,
        })
        .unwrap();
    frames
        .gpu(GpuSpan {
            frame: f3,
            begin_ticks: 5_000,
            end_ticks: 6_000,
            period_ns: f64::NAN,
        })
        .unwrap();

    // Measured frame 4: completed without GPU span before finish() becomes Availability::Incomplete.
    let f4 = frames.begin();
    frames.submitted(f4).unwrap();
    frames.presented(f4, true).unwrap();
    frames.completed(f4).unwrap();

    let run = frames.finish().unwrap();
    assert!(run.validate().is_ok());
    assert_eq!(obs(&run, frames::SUBMIT).len(), 3);
    assert_eq!(obs(&run, frames::COMPLETED).len(), 3);
    assert_eq!(obs(&run, frames::PRESENT_INTERVAL).len(), 3);

    let gpu_obs = obs(&run, frames::GPU_PASS);
    assert_eq!(gpu_obs.len(), 4);
    assert_eq!(gpu_obs[0].availability, Availability::Available);
    assert_eq!(gpu_obs[0].value.as_deref(), Some("3000"));
    assert!(matches!(gpu_obs[1].availability, Availability::Invalid(_)));
    assert!(gpu_obs[1].value.is_none());
    assert!(matches!(gpu_obs[2].availability, Availability::Invalid(_)));
    assert!(matches!(
        gpu_obs[3].availability,
        Availability::Incomplete(_)
    ));
}

#[cfg(feature = "memory")]
#[test]
fn memory_allocator_captures_stacks_html_and_json() {
    let _guard = TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    std::hint::black_box(vec![0u8; 64]);

    let phase = ALLOC.begin_phase().unwrap();
    let session = airbug_bench::memory::Session::start().unwrap();

    let mut retained = Vec::with_capacity(64);
    for round in 0..4 {
        for i in 0..4 {
            let mut buf = vec![(round * 4 + i) as u8; 4096];
            buf[0] = 42;
            std::hint::black_box(&buf);
            if i % 2 == 0 {
                retained.push(buf);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    std::hint::black_box(&retained);

    let profile = session.finish();
    let phase_res = phase.finish();

    assert!(phase_res.allocations > 0);
    assert!(phase_res.allocated_bytes >= 8 * 4096);
    assert!(profile.total_allocations() > 0);
    assert!(profile.total_allocated_bytes() > 0);
    assert!(profile.peak_tracked_bytes > 0);
    assert!(!profile.sites().is_empty());

    let dir = std::env::temp_dir().join(format!(
        "airbug-forma-mem-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let json_path = dir.join("alloc-stacks.json");
    profile.save(&json_path).unwrap();
    let html = profile.html().unwrap();
    assert!(html.contains("<html") || html.contains("<!DOCTYPE") || html.contains("bench.memory"));
    let folded: String = profile
        .sites()
        .iter()
        .filter(|s| !s.frames.is_empty())
        .map(|s| format!("{} {}\n", s.frames.join(";"), s.allocated_bytes))
        .collect();
    assert!(!folded.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
