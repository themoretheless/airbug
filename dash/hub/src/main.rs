mod app;
mod assets;
mod benchcmp;
mod collector;
mod config;
mod error;
mod event_model;
mod http;
mod issues;
mod otlp;
mod rerun;
mod resources;
mod runs;
mod scan;
mod sentry;
mod serve;
mod test_runs;

mod store;
mod testrun;

use std::{env, path::PathBuf, process, sync::Arc};

fn init_tracing() {
    let filter = env::var("AIRBUG_LOG")
        .or_else(|_| env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".into());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() {
    init_tracing();
    let mut args = env::args().skip(1).peekable();
    let command = args.next().unwrap_or_else(|| "serve".into());
    if command == "test" {
        process::exit(testrun::main(args.collect()));
    }
    let mut root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut port = config::DEFAULT_PORT;
    let mut with_collector = command == "up";
    let mut collector_mode = collector::CollectorMode::Auto;
    let mut collector_http_port = collector::DEFAULT_OTLP_HTTP_PORT;
    let mut collector_grpc_port = collector::DEFAULT_OTLP_GRPC_PORT;
    let mut webhook = env::var("AIRBUG_ISSUES_WEBHOOK").ok();
    let mut stack_path: Option<PathBuf> = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--root" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--root needs a path"));
                root = PathBuf::from(value);
            }
            "--port" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--port needs a number"));
                port = value
                    .parse()
                    .unwrap_or_else(|_| usage_exit("invalid --port"));
            }
            "--collector" => with_collector = true,
            "--collector-mode" => {
                let value = args.next().unwrap_or_else(|| {
                    usage_exit("--collector-mode needs auto|builtin|docker|binary")
                });
                collector_mode = collector::CollectorMode::parse(&value).unwrap_or_else(|| {
                    usage_exit("invalid --collector-mode (expected auto|builtin|docker|binary)")
                });
                with_collector = true;
            }
            "--collector-http-port" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--collector-http-port needs a port"));
                collector_http_port = value
                    .parse()
                    .unwrap_or_else(|_| usage_exit("invalid --collector-http-port"));
            }
            "--collector-grpc-port" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--collector-grpc-port needs a port"));
                collector_grpc_port = value
                    .parse()
                    .unwrap_or_else(|_| usage_exit("invalid --collector-grpc-port"));
            }
            "--webhook" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--webhook needs a URL"));
                webhook = Some(value);
            }
            "--stack" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--stack needs a path"));
                stack_path = Some(PathBuf::from(value));
            }
            "-h" | "--help" => usage_exit(""),
            other => usage_exit(&format!("unknown argument: {other}")),
        }
    }

    let root = root.canonicalize().unwrap_or(root);

    match command.as_str() {
        "serve" | "up" => {
            let app = match app::HubApp::new(root.clone(), port, webhook) {
                Ok(app) => Arc::new(app),
                Err(err) => {
                    tracing::error!(error = %err, "hub init failed");
                    process::exit(1);
                }
            };
            if let Some(stack) = stack_path {
                app.resources.set_stack_path(Some(stack));
            }

            let collector = if with_collector {
                match collector::CollectorHandle::start(
                    Arc::clone(&app),
                    collector_mode,
                    collector_http_port,
                    collector_grpc_port,
                ) {
                    Ok(handle) => handle,
                    Err(err) => {
                        tracing::error!(error = %err, "hub collector failed");
                        process::exit(1);
                    }
                }
            } else {
                None
            };

            if command == "up" {
                match app.resources.start_all() {
                    Ok(started) if !started.is_empty() => {
                        tracing::info!(?started, "resources: started stack resources");
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::warn!(error = %err, "resources: could not start all stack resources");
                    }
                }
            }

            if collector.is_some() || command == "up" {
                let root_for_handler = Arc::new(root.clone());
                let app_for_handler = Arc::clone(&app);
                if let Err(err) = ctrlc::set_handler(move || {
                    tracing::info!("hub: stopping (Ctrl+C)");
                    app_for_handler.resources.stop_all();
                    collector::emergency_stop(root_for_handler.as_ref());
                    process::exit(0);
                }) {
                    tracing::warn!(error = %err, "could not install Ctrl+C handler");
                }
            }

            if let Err(err) = serve::serve(Arc::clone(&app)) {
                app.resources.stop_all();
                if let Some(handle) = collector {
                    handle.stop();
                }
                tracing::error!(error = %err, "hub serve failed");
                process::exit(1);
            }
        }
        "status" => {
            print!("{}", serve::status_json_with_port(&root, port));
        }
        other => usage_exit(&format!("unknown command: {other}")),
    }
}

fn usage_exit(message: &str) -> ! {
    if !message.is_empty() {
        eprintln!("{message}");
    }
    eprintln!(
        "\
airbug-hub — monorepo dashboard for unit / bench / mon / otel / err

Usage:
  airbug-hub serve [--root PATH] [--port N] [--collector] [--collector-mode auto|builtin|docker|binary] [--stack PATH] [--webhook URL]
      # dashboard :8790 (+ built-in HTTP OTLP at /v1/*, Sentry DSN http://airbug@127.0.0.1:8790/1)
      # --collector brings up :4318 HTTP + :4317 gRPC (Docker -> otelcol -> built-in Rust collector)
      # --webhook / AIRBUG_ISSUES_WEBHOOK fires on new issues (http:// only)
  airbug-hub up [--root PATH] [--port N] [--stack PATH]
      # starts hub + OTLP collector + all resources declared in .airbug/stack.json
  airbug-hub status [--root PATH]
  {}
",
        testrun::USAGE
    );
    process::exit(if message.is_empty() { 0 } else { 2 });
}
