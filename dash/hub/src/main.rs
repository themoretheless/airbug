mod collector;
mod config;
mod error;
mod http;
mod issues;
mod otlp;
mod scan;
mod serve;

use std::{env, path::PathBuf, process, sync::Arc};

fn main() {
    let mut args = env::args().skip(1).peekable();
    let command = args.next().unwrap_or_else(|| "serve".into());
    let mut root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut port = config::DEFAULT_PORT;
    let mut with_collector = false;
    let mut webhook = env::var("AIRBUG_ISSUES_WEBHOOK").ok();

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
            "--webhook" => {
                let value = args
                    .next()
                    .unwrap_or_else(|| usage_exit("--webhook needs a URL"));
                webhook = Some(value);
            }
            "-h" | "--help" => usage_exit(""),
            other => usage_exit(&format!("unknown argument: {other}")),
        }
    }

    let root = root.canonicalize().unwrap_or(root);

    match command.as_str() {
        "serve" => {
            let collector = if with_collector {
                match collector::CollectorHandle::start(&root) {
                    Ok(handle) => handle,
                    Err(err) => {
                        eprintln!("hub collector failed: {err}");
                        process::exit(1);
                    }
                }
            } else {
                None
            };

            if collector.is_some() {
                let root_for_handler = Arc::new(root.clone());
                if let Err(err) = ctrlc::set_handler(move || {
                    eprintln!("\ncollector: stopping (Ctrl+C) …");
                    collector::emergency_stop(root_for_handler.as_ref());
                    process::exit(0);
                }) {
                    eprintln!("collector: could not install Ctrl+C handler: {err}");
                }
            }

            let opts = serve::ServeOpts { webhook };
            if let Err(err) = serve::serve(root, port, opts) {
                if let Some(handle) = collector {
                    handle.stop();
                }
                eprintln!("hub serve failed: {err}");
                process::exit(1);
            }
        }
        "status" => {
            print!("{}", serve::status_json(&root));
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
  airbug-hub serve [--root PATH] [--port N] [--collector] [--webhook URL]
      # dashboard :8790; --collector prefers Docker, else otelcol on PATH
      # --webhook / AIRBUG_ISSUES_WEBHOOK fires on new issues (http:// only)
  airbug-hub status [--root PATH]
"
    );
    process::exit(if message.is_empty() { 0 } else { 2 });
}
