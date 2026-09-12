//! Native Forma offscreen scenarios. Golden readback/serialization are outside measurement.
use rbench::{image::RgbaImage, *};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};
#[global_allocator]
static ALLOC: alloc::TrackingAllocator<std::alloc::System> =
    alloc::TrackingAllocator::new(std::alloc::System);
const N: usize = 96;
const SCENARIOS: [&str; 7] = [
    "static",
    "hover",
    "animation",
    "scroll",
    "resize",
    "text",
    "image",
];
const SCROLL:&str="component Demo { Frame { width:400; height:200; padding:10; clip:true; background:#181e2a; Scroll { Button { width:600; height:400; text:'Scroll content'; } } } }";
struct Options {
    filter: String,
    goldens: PathBuf,
    record: bool,
    tolerance: u8,
    percent: f64,
    gpu: bool,
    cold: bool,
    width: u32,
    height: u32,
    dpi: f32,
    backend: wgpu::Backends,
}
fn main() -> Result<()> {
    let mut o = Options {
        filter: String::new(),
        goldens: PathBuf::from(".rbench/forma-goldens"),
        record: false,
        tolerance: 0,
        percent: 0.,
        gpu: false,
        cold: false,
        width: 800,
        height: 400,
        dpi: 2.,
        backend: wgpu::Backends::all(),
    };
    let mut args = std::env::args().skip(1);
    let mut list = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--list" => list = true,
            "--json" => {}
            "--gpu-timestamps" => o.gpu = true,
            "--cold" => o.cold = true,
            "--width" => {
                o.width = args
                    .next()
                    .ok_or_else(|| error("width required"))?
                    .parse()?
            }
            "--height" => {
                o.height = args
                    .next()
                    .ok_or_else(|| error("height required"))?
                    .parse()?
            }
            "--dpi" => o.dpi = args.next().ok_or_else(|| error("dpi required"))?.parse()?,
            "--backend" => {
                o.backend = match args.next().as_deref() {
                    Some("metal") => wgpu::Backends::METAL,
                    Some("vulkan") => wgpu::Backends::VULKAN,
                    Some("gl") => wgpu::Backends::GL,
                    Some("dx12") => wgpu::Backends::DX12,
                    _ => return Err(error("backend must be metal/vulkan/gl/dx12")),
                }
            }
            "--filter" => o.filter = args.next().ok_or_else(|| error("filter required"))?,
            "--goldens" => {
                o.goldens = args
                    .next()
                    .ok_or_else(|| error("golden directory required"))?
                    .into()
            }
            "--record-goldens" => {
                o.record = true;
                o.goldens = args
                    .next()
                    .ok_or_else(|| error("new golden directory required"))?
                    .into();
            }
            "--channel-tolerance" => {
                o.tolerance = args
                    .next()
                    .ok_or_else(|| error("tolerance required"))?
                    .parse()?
            }
            "--max-changed-percent" => {
                o.percent = args
                    .next()
                    .ok_or_else(|| error("percentage required"))?
                    .parse()?
            }
            "--help" => {
                println!("--gpu-timestamps | --cold --filter static|text; --width 16..4096 --height 16..4096 --dpi 0.5..4 --backend metal|vulkan|gl|dx12; --list --filter TEXT --record-goldens NEW_DIR | --goldens DIR [--channel-tolerance 0..255 --max-changed-percent 0..100] --json\nGoldens are required. Recording creates reference images only; inspect PNGs before using them. image is unsupported by this native renderer.");
                return Ok(());
            }
            _ => return Err(error(format!("unknown argument {a}"))),
        }
    }
    if list {
        for s in SCENARIOS.into_iter().filter(|s| s.contains(&o.filter)) {
            println!(
                "forma/{s}{}",
                if s == "image" {
                    " [unsupported: native image primitive absent]"
                } else {
                    ""
                }
            );
        }
        return Ok(());
    }
    if !(16..=4096).contains(&o.width)
        || !(16..=4096).contains(&o.height)
        || !o.dpi.is_finite()
        || !(0.5..=4.).contains(&o.dpi)
    {
        return Err(error("viewport 16..4096 and DPI 0.5..4 required"));
    }
    if !o.percent.is_finite() || !(0.0..=100.0).contains(&o.percent) {
        return Err(error("percentage must be 0..100"));
    }
    if SCENARIOS.iter().all(|s| !s.contains(&o.filter)) {
        return Err(error("no matching scenario"));
    }
    if o.record {
        if let Some(p) = o.goldens.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(p)?;
        }
        std::fs::create_dir(&o.goldens)?;
    }
    pollster::block_on(run(o))
}
fn source(name: &str) -> String {
    match name {
        "scroll" => SCROLL.into(),
        "text" => forma_vector::EXAMPLE.replace("Найти документы", "Текст ABC 123 — тест"),
        _ => forma_vector::EXAMPLE.into(),
    }
}
fn state(model: &mut forma_vector::Button, name: &str, frame: usize) -> (u32, u32) {
    match name {
        "hover" => {
            model.pointer(100., 90., 0);
            model.tick(1000.);
        }
        "animation" => {
            model.pointer(if (frame / 12) & 1 == 0 { 100. } else { -10. }, 90., 0);
            model.tick(16.);
        }
        "scroll" => {
            model.scroll(-10000., -10000.);
            model.scroll((frame % 40) as f32, (frame % 60) as f32);
        }
        _ => {}
    }
    if name == "resize" && frame % 2 == 1 {
        (640, 320)
    } else {
        (800, 400)
    }
}
struct Surface {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    w: u32,
    h: u32,
    dpi: f32,
}
fn surface(r: &forma_vector::gpu::Renderer, w: u32, h: u32, dpi: f32) -> Surface {
    let texture = r.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("rbench-golden-target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    Surface {
        texture,
        view,
        w,
        h,
        dpi,
    }
}
fn draw(r: &mut forma_vector::gpu::Renderer, m: &forma_vector::Button, s: &Surface) -> Result<()> {
    r.draw(m, &s.view, s.w, s.h, s.dpi, true, true)
        .map_err(error)?;
    Ok(())
}
fn pixels(r: &forma_vector::gpu::Renderer, s: &Surface) -> Result<RgbaImage> {
    let stride = (s.w * 4).div_ceil(256) * 256;
    let b = r.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("golden readback outside measurement"),
        size: u64::from(stride) * u64::from(s.h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut e = r.device.create_command_encoder(&Default::default());
    e.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &s.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &b,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(s.h),
            },
        },
        wgpu::Extent3d {
            width: s.w,
            height: s.h,
            depth_or_array_layers: 1,
        },
    );
    r.queue.submit(Some(e.finish()));
    let (send, recv) = std::sync::mpsc::channel();
    b.slice(..).map_async(wgpu::MapMode::Read, move |v| {
        let _ = send.send(v);
    });
    r.device.poll(wgpu::PollType::wait_indefinitely())?;
    recv.recv()??;
    let mapped = b.slice(..).get_mapped_range();
    let mut data = Vec::with_capacity((s.w * s.h * 4) as usize);
    for row in mapped.chunks_exact(stride as usize) {
        data.extend_from_slice(&row[..s.w as usize * 4]);
    }
    drop(mapped);
    b.unmap();
    Ok(RgbaImage {
        width: s.w,
        height: s.h,
        pixels: data,
    })
}
fn golden(
    o: &Options,
    name: &str,
    frame: usize,
    image: &RgbaImage,
    record: bool,
) -> Result<String> {
    let key = if o.width == 800 && o.height == 400 && o.dpi == 2. {
        name.to_string()
    } else {
        format!("{name}-{}x{}-dpi{}", o.width, o.height, o.dpi)
    };
    let path = o.goldens.join(format!("{key}-{frame}.rbimg"));
    if record {
        image.save_new(&path)?;
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.with_extension("png"))?;
        let mut encoder = png::Encoder::new(f, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&image.pixels)?;
    } else {
        let expected = RgbaImage::load(&path).map_err(|e| {
            error(format!(
                "{}: {e}; record and inspect reference goldens first",
                path.display()
            ))
        })?;
        let difference = image.compare(&expected, o.tolerance, o.percent)?;
        if !difference.accepted {
            return Err(error(format!(
                "golden mismatch {name} frame {frame}: {}",
                serde_json::to_string(&difference)?
            )));
        }
    }
    model::hash_file(&path)
}
fn metrics() -> Vec<Metric> {
    let phase = |id: &str, scope: &str| Metric {
        id: id.into(),
        unit: "calls".into(),
        scope: scope.into(),
        phase: "measurement".into(),
        statistic: "96-frame phase total".into(),
        direction: Direction::Lower,
    };
    let mut result = vec![
        Metric::duration(
            "cpu.submit",
            "scenario update + CPU encode and submit wall",
            "individual frame",
        ),
        Metric::duration(
            "frame.completed",
            "scenario update + CPU submit + GPU wait; no present",
            "individual frame",
        ),
        phase(
            "alloc.calls",
            "whole-process Rust allocator; native/driver excluded",
        ),
        phase("geometry.uploads", "Forma renderer geometry"),
    ];
    for (id,unit,scope) in [
        ("renderer.frames","calls","Forma successful draw frames; one render pass per frame in this snapshot"),
        ("renderer.uploaded_bytes","bytes","Forma application payload writes and initialized buffers; excludes driver traffic"),
        ("renderer.tile_uploads","calls","Forma tile upload counter"),
        ("renderer.buffer_allocations","calls","Forma application buffer creations"),
        ("renderer.bind_groups","calls","Forma bind group creations"),
        ("alloc.phase_peak","bytes","whole-process peak Rust live bytes during phase; caller quiescence required at boundaries"),
        ("alloc.live_end","bytes","whole-process Rust live bytes at phase end"),
    ] {let mut m=phase(id,scope);m.unit=unit.into();result.push(m);}
    result.push(phase(
        "renderer.draw_calls",
        "one pass.draw per successful Renderer frame in Forma snapshot e2ef5f7",
    ));
    result.push(phase(
        "geometry.rebuilds",
        "native display-list rebuild count; unavailable without a source counter",
    ));
    result.push(phase(
        "cache.hits",
        "native display-list cache hits; unavailable without a source counter",
    ));
    result
}
fn gpu_sample(
    recorder: &mut Recorder,
    id: &str,
    sample: std::result::Result<Option<f64>, String>,
) -> Result<()> {
    match sample {
        Ok(Some(ns)) if ns.is_finite() && ns > 0. && ns.round() > 0. => {
            recorder.observe(id, "gpu.duration", ns.round() as u128)
        }
        Ok(_) => recorder.unavailable(
            id,
            "gpu.duration",
            Availability::Invalid("missing, sub-nanosecond or invalid timestamp pair".into()),
        ),
        Err(e) => recorder.unavailable(id, "gpu.duration", Availability::Invalid(e)),
    }
}
async fn run(o: Options) -> Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: o.backend,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = match instance.request_adapter(&Default::default()).await {
        Ok(a) => a,
        Err(e) => {
            if o.record {
                return Err(e.into());
            }
            let mut r = Recorder::new();
            let id = format!(
                "forma/backend/{:?}/{}x{}/{}",
                o.backend, o.width, o.height, o.dpi
            );
            r.case(Case {
                id: id.clone(),
                contract: BTreeMap::from([("backend".into(), format!("{:?}", o.backend))]),
                metrics: metrics(),
            })?;
            for m in metrics() {
                r.unavailable(
                    &id,
                    &m.id,
                    Availability::Unsupported(format!("adapter unavailable: {e}")),
                )?;
            }
            println!("RBENCH_RESULT={}", serde_json::to_string(&r.finish()?)?);
            return Ok(());
        }
    };
    if o.cold {
        if o.record || o.gpu || !matches!(o.filter.as_str(), "static" | "text") {
            return Err(error(
                "cold requires --filter static|text, existing goldens and no GPU replay",
            ));
        }
        let src = source(&o.filter);
        let start = Instant::now();
        let model = forma_vector::Button::from_sources(&src, forma_vector::BUTTON_COMPONENT)
            .map_err(error)?;
        let mut renderer =
            forma_vector::gpu::Renderer::new(&adapter, wgpu::TextureFormat::Rgba8Unorm)
                .await
                .map_err(error)?;
        let target = surface(&renderer, o.width, o.height, o.dpi);
        draw(&mut renderer, &model, &target)?;
        renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
        let elapsed = start.elapsed().as_nanos();
        let golden_hash = golden(&o, &o.filter, 0, &pixels(&renderer, &target)?, false)?;
        let mut r = Recorder::new();
        let id = format!("forma/{}/cold", o.filter);
        r.case(Case{id:id.clone(),contract:BTreeMap::from([("temperature".into(),"first model parse + renderer/pipeline construction + target allocation + completed frame; adapter discovery excluded; no earlier draw in worker; external driver/OS caches uncontrolled".into()),("adapter".into(),format!("{:?}",adapter.get_info())),("viewport".into(),format!("{}x{}",o.width,o.height)),("dpi".into(),o.dpi.to_string()),("golden.sha256".into(),golden_hash),("fixture.sha256".into(),format!("{:x}",Sha256::digest(format!("{src}\n{}",forma_vector::BUTTON_COMPONENT))))]),metrics:vec![Metric::duration("first.completed","model + renderer + resources + first completed offscreen draw; post-validation excluded","first frame including initialization")]})?;
        r.observe(&id, "first.completed", elapsed)?;
        println!("RBENCH_RESULT={}", serde_json::to_string(&r.finish()?)?);
        return Ok(());
    }
    let mut recorder = Recorder::new();
    for name in SCENARIOS.into_iter().filter(|s| s.contains(&o.filter)) {
        let id = format!("forma/{name}");
        if name == "image" {
            if !o.record {
                recorder.case(Case {
                    id: id.clone(),
                    contract: BTreeMap::from([(
                        "scenario".into(),
                        "native image primitive".into(),
                    )]),
                    metrics: metrics(),
                })?;
                for metric in metrics() {
                    recorder.unavailable(&id,&metric.id,Availability::Unsupported("This Forma native snapshot has no Image display-list/GPU primitive; JS image component is not a native image workload".into()))?;
                }
            }
            continue;
        }
        eprintln!(
            "rbench: {id} — {}",
            if o.record {
                "recording reference checkpoints"
            } else {
                "golden pre-validation"
            }
        );
        let src = source(name);
        let mut hashes = BTreeMap::new();
        // Separate correctness pass, then a fresh measured instance with identical warmup/state sequence.
        for pass in 0..if o.record { 1 } else { 2 } {
            let mut model =
                forma_vector::Button::from_sources(&src, forma_vector::BUTTON_COMPONENT)
                    .map_err(error)?;
            let mut renderer =
                forma_vector::gpu::Renderer::new(&adapter, wgpu::TextureFormat::Rgba8Unorm)
                    .await
                    .map_err(error)?;
            let large = surface(&renderer, o.width, o.height, o.dpi);
            let small = surface(&renderer, o.width * 4 / 5, o.height * 4 / 5, o.dpi);
            for _ in 0..20 {
                draw(&mut renderer, &model, &large)?;
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
            }
            let mut submits = [0u128; N];
            let mut completes = [0u128; N];
            let uploads = renderer.uploads;
            if pass == 1 {
                eprintln!("rbench: {id} — measuring {N} frames");
            }
            let resources_before = renderer.resource_stats();
            let frames_before = renderer.frames;
            let before = ALLOC.snapshot();
            let allocation_phase = ALLOC.begin_phase()?;
            for i in 0..N {
                let start = Instant::now();
                let (w, _) = state(&mut model, name, i);
                let s = if w == 800 { &large } else { &small };
                draw(&mut renderer, &model, s)?;
                submits[i] = start.elapsed().as_nanos();
                renderer.device.poll(wgpu::PollType::wait_indefinitely())?;
                completes[i] = start.elapsed().as_nanos();
                if pass == 0 && (i == 0 || i == N - 1) {
                    let hash = golden(&o, name, i, &pixels(&renderer, s)?, o.record)?;
                    hashes.insert(format!("golden.{i}"), hash);
                }
            }
            let allocation_phase = allocation_phase.finish();
            let after = ALLOC.snapshot();
            let resources_after = renderer.resource_stats();
            if pass == 1 {
                let final_surface = if name == "resize" { &small } else { &large };
                let final_hash =
                    golden(&o, name, N - 1, &pixels(&renderer, final_surface)?, false)?;
                if hashes.get(&format!("golden.{}", N - 1)) != Some(&final_hash) {
                    return Err(error("golden changed during measurement"));
                }
                if name == "static" && renderer.uploads != uploads {
                    return Err(error("static geometry unexpectedly reuploaded"));
                }
                let mut contract = BTreeMap::from([
                    ("adapter".into(), format!("{:?}", adapter.get_info())),
                    (
                        "fixture.sha256".into(),
                        format!(
                            "{:x}",
                            Sha256::digest(format!("{src}\n{}", forma_vector::BUTTON_COMPONENT))
                        ),
                    ),
                    ("scenario".into(), name.into()),
                    ("scenario.version".into(), "1".into()),
                    (
                        "viewport".into(),
                        if name == "resize" {
                            format!(
                                "alternating {}x{}/{}x{}; preallocated targets",
                                o.width,
                                o.height,
                                o.width * 4 / 5,
                                o.height * 4 / 5
                            )
                        } else {
                            format!("{}x{}", o.width, o.height)
                        },
                    ),
                    ("dpi".into(), o.dpi.to_string()),
                    ("frames".into(), N.to_string()),
                    ("warmup".into(), "20 static frames".into()),
                    (
                        "completion".into(),
                        "serialized offscreen wait; no present".into(),
                    ),
                    (
                        "validation".into(),
                        format!(
                            "pre checkpoints 0,95 and post 95; RGBA8 tolerance {} / {}%",
                            o.tolerance, o.percent
                        ),
                    ),
                    (
                        "instrumentation".into(),
                        "atomic Rust allocator; recording/readback outside measured phase".into(),
                    ),
                ]);
                contract.extend(hashes.clone());
                recorder.case(Case {
                    id: id.clone(),
                    contract,
                    metrics: metrics(),
                })?;
                for (s, c) in submits.into_iter().zip(completes) {
                    recorder.observe(&id, "cpu.submit", s)?;
                    recorder.observe(&id, "frame.completed", c)?;
                }
                recorder.observe(
                    &id,
                    "alloc.calls",
                    u128::from(after.allocations - before.allocations),
                )?;
                recorder.observe(
                    &id,
                    "renderer.draw_calls",
                    (renderer.frames - frames_before) as u128,
                )?;
                for metric in ["geometry.rebuilds", "cache.hits"] {
                    recorder.unavailable(&id,metric,Availability::Unsupported("Forma snapshot exposes no native display-list counter; uploads and elapsed time are not substitutes".into()))?;
                }
                for (metric, value) in [
                    ("renderer.frames", renderer.frames - frames_before),
                    (
                        "renderer.uploaded_bytes",
                        resources_after.uploaded_bytes_total
                            - resources_before.uploaded_bytes_total,
                    ),
                    (
                        "renderer.tile_uploads",
                        resources_after.tile_uploads_total - resources_before.tile_uploads_total,
                    ),
                    (
                        "renderer.buffer_allocations",
                        resources_after.buffer_allocations_total
                            - resources_before.buffer_allocations_total,
                    ),
                    (
                        "renderer.bind_groups",
                        resources_after.bind_group_creations_total
                            - resources_before.bind_group_creations_total,
                    ),
                    ("alloc.phase_peak", allocation_phase.peak_live_bytes),
                    ("alloc.live_end", allocation_phase.live_end_bytes),
                ] {
                    recorder.observe(&id, metric, value as u128)?;
                }
                recorder.observe(
                    &id,
                    "geometry.uploads",
                    u128::from(renderer.uploads - uploads),
                )?;
            }
        }
    }
    // GPU query instrumentation lives in a separate replay, after normal measurement.
    if o.gpu && !o.record {
        for name in SCENARIOS
            .into_iter()
            .filter(|s| *s != "image" && s.contains(&o.filter))
        {
            let src = source(name);
            let mut model =
                forma_vector::Button::from_sources(&src, forma_vector::BUTTON_COMPONENT)
                    .map_err(error)?;
            let mut renderer = forma_vector::gpu::Renderer::new_profiled(
                &adapter,
                wgpu::TextureFormat::Rgba8Unorm,
            )
            .await
            .map_err(error)?;
            let id = format!("forma/{name}/gpu");
            let mut metric = Metric::duration(
                "gpu.duration",
                "GPU render pass timestamps; separate instrumented replay; readback excluded",
                "individual frame",
            );
            metric.phase = "gpu-instrumented-replay".into();
            recorder.case(Case{id:id.clone(),contract:BTreeMap::from([
                ("adapter".into(),format!("{:?}",adapter.get_info())),("viewport".into(),format!("{}x{}",o.width,o.height)),("dpi".into(),o.dpi.to_string()),("scenario".into(),name.into()),("fixture.sha256".into(),format!("{:x}",Sha256::digest(format!("{src}\n{}",forma_vector::BUTTON_COMPONENT)))),("instrumentation".into(),"timestamps; immediate serial query read after each draw; no normal timing comparison".into())]),metrics:vec![metric]})?;
            if !renderer.gpu_timestamps_enabled() {
                recorder.unavailable(
                    &id,
                    "gpu.duration",
                    Availability::Unsupported("adapter lacks TIMESTAMP_QUERY".into()),
                )?;
                continue;
            }
            let large = surface(&renderer, o.width, o.height, o.dpi);
            let small = surface(&renderer, o.width * 4 / 5, o.height * 4 / 5, o.dpi);
            for _ in 0..20 {
                draw(&mut renderer, &model, &large)?;
                let _ = renderer.read_gpu_duration_ns().map_err(error)?;
            }
            for i in 0..N {
                let (w, _) = state(&mut model, name, i);
                let s = if w == 800 { &large } else { &small };
                draw(&mut renderer, &model, s)?;
                gpu_sample(&mut recorder, &id, renderer.read_gpu_duration_ns())?;
                if renderer.read_gpu_duration_ns().map_err(error)?.is_some() {
                    return Err(error("GPU query consumed more than once"));
                }
                if i == 0 || i == N - 1 {
                    golden(&o, name, i, &pixels(&renderer, s)?, false)?;
                }
            }
        }
    }
    if o.record {
        model::write_new(
            &o.goldens.join("reference.json"),
            &serde_json::json!({"adapter":format!("{:?}",adapter.get_info()),"filter":o.filter,"frames":N,"checkpoints":[0,95],"note":"Captured references; inspect PNGs. This is not an independent correctness oracle."}),
        )?;
        println!(
            "Recorded {}. Inspect PNGs before benchmarking.",
            o.goldens.display()
        );
    } else {
        recorder.note("Six native offscreen scenarios; checkpoint golden equality is not full-frame-sequence equivalence or window FPS. Resize switches preallocated targets; texture allocation/window resize excluded. Image scenario is explicitly unsupported. Golden readback and recorder serialization are outside measurement.");
        println!(
            "RBENCH_RESULT={}",
            serde_json::to_string(&recorder.finish()?)?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_query_results_are_never_zero_durations() {
        let mut r = Recorder::new();
        r.case(Case {
            id: "gpu".into(),
            contract: BTreeMap::new(),
            metrics: vec![Metric::duration("gpu.duration", "test", "individual frame")],
        })
        .unwrap();
        for sample in [
            Ok(None),
            Ok(Some(f64::NAN)),
            Ok(Some(0.)),
            Err("query readback failed".into()),
            Ok(Some(123.)),
        ] {
            gpu_sample(&mut r, "gpu", sample).unwrap();
        }
        let r = r.finish().unwrap();
        assert!(r.observations[..4]
            .iter()
            .all(|o| o.value.is_none() && matches!(o.availability, Availability::Invalid(_))));
        assert_eq!(r.observations[4].value.as_deref(), Some("123"));
    }
}
