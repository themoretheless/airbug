//! `airbug-mon` — host sampler: CPU, memory, swap, network, disk, GPU and top processes once
//! a second into SQLite, optionally exported as OTLP metrics. The egui window is an optional
//! view of the same samples (`--features gui`, `--gui`).
// History queries, CSV export and a few sample fields are read only by the window.
#![cfg_attr(not(feature = "gui"), allow(dead_code))]
#[cfg(feature = "gui")]
mod app;
mod gpu;
mod otlp;
mod sampler;
mod storage;
#[cfg(feature = "gui")]
mod views;

use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const USAGE: &str = "\
airbug-mon [--seconds N] [--otlp [ENDPOINT]] [--gui]
    samples the host once a second into SQLite until Ctrl+C (or for N seconds)
    --otlp   also export host metrics over OTLP (or set OTEL_EXPORTER_OTLP_ENDPOINT)
    --gui    open the window instead (needs a build with `--features gui`)";

fn init_tracing() {
    let filter = std::env::var("AIRBUG_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".into());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() -> ExitCode {
    init_tracing();
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let otlp = maybe_otlp(&args);
    if args.iter().any(|a| a == "--gui") {
        return run_window(otlp);
    }
    // `--headless` is the default now; the flag is still accepted.
    let seconds = args
        .windows(2)
        .find(|w| w[0] == "--seconds")
        .and_then(|w| w[1].parse().ok());
    run_headless(seconds, otlp);
    ExitCode::SUCCESS
}

#[cfg(not(feature = "gui"))]
fn run_window(otlp: Option<Arc<otlp::OtlpExport>>) -> ExitCode {
    shutdown_otlp(otlp);
    eprintln!(
        "airbug-mon: built without the window — run `cargo run -p airbug-mon --features gui -- --gui`"
    );
    ExitCode::from(2)
}

#[cfg(feature = "gui")]
fn run_window(otlp: Option<Arc<otlp::OtlpExport>>) -> ExitCode {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (db_tx, db_rx) = crossbeam_channel::unbounded();
    let stop = Arc::new(AtomicBool::new(false));

    let db_path = storage::default_db_path();
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
            Ok(Box::new(app::MonApp::new(rx)))
        }),
    );
    stop.store(true, Ordering::Relaxed);
    let _ = sampler_handle.join();
    let _ = writer.join();
    shutdown_otlp(otlp);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("airbug-mon: {err}");
            ExitCode::FAILURE
        }
    }
}

fn maybe_otlp(args: &[String]) -> Option<Arc<otlp::OtlpExport>> {
    if !otlp::want_otlp(args) {
        return None;
    }
    let endpoint = otlp::endpoint_from_args(args);
    match otlp::OtlpExport::start(endpoint) {
        Ok(export) => {
            tracing::info!("otlp: exporting host metrics (--otlp / OTEL_EXPORTER_OTLP_ENDPOINT)");
            Some(Arc::new(export))
        }
        Err(err) => {
            tracing::warn!(error = %err, "otlp: disabled");
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
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

/// Sample until Ctrl+C, or for `seconds` when given.
fn run_headless(seconds: Option<u64>, otlp: Option<Arc<otlp::OtlpExport>>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (db_tx, db_rx) = crossbeam_channel::unbounded();
    let stop = Arc::new(AtomicBool::new(false));
    let interrupted = stop.clone();
    let _ = ctrlc::set_handler(move || interrupted.store(true, Ordering::Relaxed));

    let db_path = storage::default_db_path();
    tracing::info!(path = %db_path.display(), "sampling into the database; Ctrl+C stops");
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

    let deadline = seconds.map(|s| std::time::Instant::now() + Duration::from_secs(s));
    let mut received = 0u64;
    while !stop.load(Ordering::Relaxed) && deadline.is_none_or(|d| std::time::Instant::now() < d) {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(sample) => {
                received += 1;
                if received % 10 == 1 {
                    tracing::info!("{}", summary(&sample));
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
        }
    }
    stop.store(true, Ordering::Relaxed);
    let _ = sampler_handle.join();
    drop(rx);
    writer.join().expect("db writer panicked");
    shutdown_otlp(otlp);
    let storage = storage::Storage::open(&db_path).expect("не удалось открыть БД");
    match storage.count_samples() {
        Ok(rows) => tracing::info!(received, rows, "stopped"),
        Err(e) => tracing::warn!(error = %e, "stopped; counting rows failed"),
    }
}

/// One line of what the host looks like right now.
fn summary(s: &sampler::Sample) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    let gpu = match &s.gpu {
        Some(g) if g.mem_total > 0 => format!(
            " · gpu {:.0}% {:.1}/{:.1} GiB{}",
            g.util,
            g.mem_used as f64 / GIB,
            g.mem_total as f64 / GIB,
            if g.unified { " (unified)" } else { "" }
        ),
        Some(g) => format!(" · gpu {:.0}%", g.util),
        None => String::new(),
    };
    format!(
        "cpu {:.0}% · mem {:.1}/{:.1} GiB{gpu}",
        s.cpu_total,
        s.mem_used as f64 / GIB,
        s.mem_total as f64 / GIB
    )
}
