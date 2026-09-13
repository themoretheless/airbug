mod app;
mod gpu;
mod otlp;
mod sampler;
mod storage;
mod views;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let otlp = maybe_otlp(&args);

    if args.iter().any(|a| a == "--headless") {
        let seconds: u64 = args
            .windows(2)
            .find(|w| w[0] == "--seconds")
            .and_then(|w| w[1].parse().ok())
            .unwrap_or(15);
        run_headless(seconds, otlp);
        return Ok(());
    }

    let (tx, rx) = crossbeam_channel::unbounded();
    let (db_tx, db_rx) = crossbeam_channel::unbounded();
    let stop = Arc::new(AtomicBool::new(false));

    let db_path = storage::default_db_path();
    let storage = storage::Storage::open(&db_path).expect("не удалось открыть БД");
    std::thread::spawn(move || storage::run_db_writer(db_rx, storage));

    let sampler_stop = stop.clone();
    let otlp_sampler = otlp.clone();
    let sampler_handle = std::thread::spawn(move || {
        sampler::run_sampler(
            tx,
            db_tx,
            gpu::default_provider(),
            sampler_stop,
            otlp_sampler,
        )
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 750.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "airbug-mon",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_style(egui::Style {
                visuals: egui::Visuals::dark(),
                ..Default::default()
            });
            Ok(Box::new(app::OtelApp::new(rx)))
        }),
    );
    stop.store(true, Ordering::Relaxed);
    let _ = sampler_handle.join();
    shutdown_otlp(otlp);
    result
}

fn maybe_otlp(args: &[String]) -> Option<Arc<otlp::OtlpExport>> {
    if !otlp::want_otlp(args) {
        return None;
    }
    let endpoint = otlp::endpoint_from_args(args);
    match otlp::OtlpExport::start(endpoint) {
        Ok(export) => {
            eprintln!("otlp: exporting host metrics (--otlp / OTEL_EXPORTER_OTLP_ENDPOINT)");
            Some(Arc::new(export))
        }
        Err(err) => {
            eprintln!("otlp: disabled — {err}");
            None
        }
    }
}

fn shutdown_otlp(otlp: Option<Arc<otlp::OtlpExport>>) {
    let Some(otlp) = otlp else {
        return;
    };
    match Arc::try_unwrap(otlp) {
        Ok(export) => export.shutdown(),
        Err(_) => {
            // Brief wait if a stray clone remains.
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn run_headless(seconds: u64, otlp: Option<Arc<otlp::OtlpExport>>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (db_tx, db_rx) = crossbeam_channel::unbounded();
    let stop = Arc::new(AtomicBool::new(false));

    let db_path = storage::default_db_path();
    println!("headless: БД {}", db_path.display());
    let storage = storage::Storage::open(&db_path).expect("не удалось открыть БД");
    let writer = std::thread::spawn(move || storage::run_db_writer(db_rx, storage));

    let sampler_stop = stop.clone();
    let otlp_sampler = otlp.clone();
    let sampler_handle = std::thread::spawn(move || {
        sampler::run_sampler(
            tx,
            db_tx,
            gpu::default_provider(),
            sampler_stop,
            otlp_sampler,
        )
    });

    let deadline = std::time::Instant::now() + Duration::from_secs(seconds);
    let mut received = 0u64;
    while std::time::Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(_) => received += 1,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
        }
    }
    stop.store(true, Ordering::Relaxed);
    let _ = sampler_handle.join();
    drop(rx);
    let _ = writer.join().expect("db writer panicked");
    shutdown_otlp(otlp);
    println!("headless: получено сэмплов в UI-канал: {received}");
    let storage = storage::Storage::open(&db_path).expect("не удалось открыть БД");
    match storage.count_samples() {
        Ok(n) => println!("headless: записей в таблице samples: {n}"),
        Err(e) => println!("headless: ошибка подсчёта: {e}"),
    }
}
