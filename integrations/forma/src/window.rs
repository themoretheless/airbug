//! Real main-thread window lifecycle. Present-call latency is NOT scanout latency.
use rbench::*;
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
struct Gpu {
    surface: wgpu::Surface<'static>,
    renderer: forma_vector::gpu::Renderer,
    config: wgpu::SurfaceConfiguration,
    adapter: String,
}
impl Gpu {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(window.clone()),
        ));
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| error("no surface configuration"))?;
        config.present_mode = wgpu::PresentMode::Fifo;
        let renderer = forma_vector::gpu::Renderer::new(&adapter, config.format)
            .await
            .map_err(error)?;
        surface.configure(&renderer.device, &config);
        Ok(Self {
            surface,
            renderer,
            config,
            adapter: format!("{:?}", adapter.get_info()),
        })
    }
}
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    model: forma_vector::Button,
    rows: Vec<(u128, u128, u128)>,
    draws: usize,
    resizes: usize,
    occlusions: usize,
    skipped: usize,
    occluded: bool,
    last: Option<Instant>,
    start: Instant,
    error: Option<String>,
}
impl App {
    fn frame(&mut self) -> Result<()> {
        if self.rows.len() >= 96 {
            return Ok(());
        }
        let w = self.window.as_ref().unwrap();
        let g = self.gpu.as_mut().unwrap();
        let size = w.inner_size();
        if self.occluded || size.width == 0 || size.height == 0 {
            self.skipped += 1;
            return Ok(());
        }
        if (g.config.width, g.config.height) != (size.width, size.height) {
            g.config.width = size.width;
            g.config.height = size.height;
            g.surface.configure(&g.renderer.device, &g.config);
        }
        let start = Instant::now();
        let frame = match g.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) => f,
            wgpu::CurrentSurfaceTexture::Suboptimal(f) => {
                g.config.width = 0;
                f
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                g.config.width = 0;
                self.skipped += 1;
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => {
                self.skipped += 1;
                return Ok(());
            }
            e => return Err(error(format!("surface failure: {e:?}"))),
        };
        self.model
            .pointer(if self.draws % 24 < 12 { 100. } else { -10. }, 90., 0);
        self.model.tick(16.);
        let size = frame.texture.size();
        g.renderer
            .draw(
                &self.model,
                &frame.texture.create_view(&Default::default()),
                size.width,
                size.height,
                w.scale_factor() as f32,
                true,
                true,
            )
            .map_err(error)?;
        let present = Instant::now();
        w.pre_present_notify();
        frame.present();
        let present_ns = present.elapsed().as_nanos();
        g.renderer
            .device
            .poll(wgpu::PollType::wait_indefinitely())?;
        let elapsed = start.elapsed().as_nanos();
        let interval = self
            .last
            .map(|v| start.duration_since(v).as_nanos())
            .unwrap_or(0);
        self.last = Some(start);
        if self.draws >= 20 {
            self.rows.push((elapsed, present_ns, interval));
        }
        self.draws += 1;
        if self.draws == 68 {
            let _ = w.request_inner_size(winit::dpi::LogicalSize::new(480., 240.));
        }
        Ok(())
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let init = (|| -> Result<()> {
            let w = Arc::new(
                events.create_window(
                    Window::default_attributes()
                        .with_title("rbench · Forma window benchmark")
                        .with_inner_size(winit::dpi::LogicalSize::new(400., 200.)),
                )?,
            );
            self.gpu = Some(pollster::block_on(Gpu::new(w.clone()))?);
            self.window = Some(w);
            Ok(())
        })();
        if let Err(e) = init {
            self.error = Some(e.to_string());
            events.exit();
        }
    }
    fn window_event(&mut self, events: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.error = Some("window closed before fixed frame budget".into());
                events.exit();
            }
            WindowEvent::Resized(_) => self.resizes += 1,
            WindowEvent::Occluded(v) => {
                self.occluded = v;
                self.occlusions += 1;
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.frame() {
                    self.error = Some(e.to_string());
                    events.exit();
                }
                if self.rows.len() == 96 {
                    events.exit();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, events: &ActiveEventLoop) {
        if self.start.elapsed() > Duration::from_secs(30) {
            self.error = Some("window deadline exceeded (possibly occluded)".into());
            events.exit();
            return;
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        events.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(8),
        ));
    }
}
fn main() -> Result<()> {
    for a in std::env::args().skip(1) {
        if a != "--json" {
            return Err(error(
                "window runner accepts only --json; fixed 20 warmup + 96 measured frames",
            ));
        }
    }
    let events = EventLoop::new()?;
    let mut app = App {
        window: None,
        gpu: None,
        model: forma_vector::Button::from_sources(
            forma_vector::EXAMPLE,
            forma_vector::BUTTON_COMPONENT,
        )
        .map_err(error)?,
        rows: vec![],
        draws: 0,
        resizes: 0,
        occlusions: 0,
        skipped: 0,
        occluded: false,
        last: None,
        start: Instant::now(),
        error: None,
    };
    events.run_app(&mut app)?;
    if let Some(e) = app.error {
        return Err(error(e));
    }
    if app.rows.len() != 96 {
        return Err(error(format!(
            "incomplete window frame sequence: {} measured, {} draws",
            app.rows.len(),
            app.draws
        )));
    }
    let mut r = Recorder::new();
    let id = "forma/window";
    r.case(Case{id:id.into(),contract:BTreeMap::from([
        ("adapter".into(),app.gpu.as_ref().unwrap().adapter.clone()),("dpi".into(),app.window.as_ref().unwrap().scale_factor().to_string()),("lifecycle".into(),"main thread; FIFO; 20 warmup; 96 measured frames; resize after measured frame 48; occlusion skipped, 30s deadline".into()),("fixture".into(),"Forma EXAMPLE/BUTTON_COMPONENT snapshot e2ef5f7792af363fe6e625febd44ced0c874cb9f".into())]),metrics:vec![Metric::duration("frame.completed","acquire + update + submit + present call + GPU completion; not compositor scanout","individual frame"),Metric::duration("present.call","pre_present_notify and surface present CPU wall; not display timing","individual frame"),Metric::duration("frame.interval","interval between successful redraw starts including event loop scheduling","individual frame")]})?;
    for (a, b, c) in app.rows {
        r.observe(id, "frame.completed", a)?;
        r.observe(id, "present.call", b)?;
        r.observe(id, "frame.interval", c)?;
    }
    r.note(format!("Observed resize events: {}; occlusion events: {}; skipped acquire/occluded frames: {}. These are not compositor dropped-frame counts. No window pixel golden verification; use offscreen correctness pass separately.",app.resizes,app.occlusions,app.skipped));
    println!("RBENCH_RESULT={}", serde_json::to_string(&r.finish()?)?);
    Ok(())
}
