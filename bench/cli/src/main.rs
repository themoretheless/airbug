#![allow(clippy::collapsible_if, clippy::manual_is_multiple_of)]
mod args;
mod artifacts;
mod cmd;
mod experiment_report;
mod forma;
mod git_run;
mod hub;
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

fn main() {
    init_tracing();
    runner::install_cancel_handler();
    match cmd::execute() {
        Ok(0) => {}
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("bench: {e}");
            std::process::exit(2);
        }
    }
}
