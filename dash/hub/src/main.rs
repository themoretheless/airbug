mod collector;
mod scan;
mod serve;

use std::{env, path::PathBuf, process, sync::Arc};

fn main() {
    let mut args = env::args().skip(1).peekable();
    let command = args.next().unwrap_or_else(|| "serve".into());
    let mut root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut port = 8790u16;
    let mut with_collector = false;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--root" => {
                let value = args.next().unwrap_or_else(|| usage_exit("--root needs a path"));
                root = PathBuf::from(value);
            }
            "--port" => {
                let value = args.next().unwrap_or_else(|| usage_exit("--port needs a number"));
                port = value.parse().unwrap_or_else(|_| usage_exit("invalid --port"));
            }
            "--collector" => with_collector = true,
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

            if let Some(ref handle) = collector {
                let compose = Arc::new(handle.compose_file.clone());
                let compose_for_handler = Arc::clone(&compose);
                if let Err(err) = ctrlc::set_handler(move || {
                    eprintln!("\ncollector: stopping stack (Ctrl+C) …");
                    let _ = std::process::Command::new("docker")
                        .args(["compose", "-f"])
                        .arg(compose_for_handler.as_ref())
                        .arg("down")
                        .status();
                    process::exit(0);
                }) {
                    eprintln!("collector: could not install Ctrl+C handler: {err}");
                }
            }

            if let Err(err) = serve::serve(root, port) {
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
airbug-hub — monorepo dashboard for unit / bench / mon / trace

Usage:
  airbug-hub serve [--root PATH] [--port N] [--collector]
      # dashboard :8790; --collector starts dash/collector via Docker
  airbug-hub status [--root PATH]
"
    );
    process::exit(if message.is_empty() { 0 } else { 2 });
}
