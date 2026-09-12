mod scan;
mod serve;

use std::{env, path::PathBuf, process};

fn main() {
    let mut args = env::args().skip(1).peekable();
    let command = args.next().unwrap_or_else(|| "serve".into());
    let mut root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut port = 8790u16;

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
            "-h" | "--help" => usage_exit(""),
            other => usage_exit(&format!("unknown argument: {other}")),
        }
    }

    let root = root.canonicalize().unwrap_or(root);

    match command.as_str() {
        "serve" => {
            if let Err(err) = serve::serve(root, port) {
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
airbug-hub — monorepo dashboard for unit / bench / otel / trace

Usage:
  airbug-hub serve [--root PATH] [--port N]   # default port 8790
  airbug-hub status [--root PATH]             # JSON snapshot to stdout
"
    );
    process::exit(if message.is_empty() { 0 } else { 2 });
}
