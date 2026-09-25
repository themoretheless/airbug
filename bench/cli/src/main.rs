#![allow(clippy::collapsible_if, clippy::manual_is_multiple_of)]
mod args;
mod artifacts;
mod cmd;
mod experiment_report;
mod forma;
mod git_run;
mod matrix;
mod privacy;
mod project;
mod revisions;
mod runner;
mod sessions;
mod web_ui;

fn init_tracing() {
    let filter = std::env::var("AIRBUG_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".into());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}

/// Stack the CLI work runs on instead of the process' main thread.
///
/// Windows reserves 1 MiB for a main thread, and a debug build overflows that inside argument
/// parsing, which kills every `cargo test` on that platform; release frames fit.
const MAIN_STACK: usize = 16 * 1024 * 1024;

fn main() {
    let worker = std::thread::Builder::new()
        .stack_size(MAIN_STACK)
        .spawn(run)
        .expect("cli: start worker thread");
    let code = match worker.join() {
        Ok(code) => code,
        // The worker reported its own panic; 101 is what a panicking main would have given.
        Err(_) => 101,
    };
    std::process::exit(code);
}

fn run() -> i32 {
    init_tracing();
    runner::install_cancel_handler();
    match cmd::execute() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("bench: {e}");
            2
        }
    }
}
