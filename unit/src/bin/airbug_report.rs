use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Options {
    all_features: bool,
    locked: bool,
    doc_tests: bool,
    output: PathBuf,
    toolchain: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("airbug-report: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_args()?;
    let root = repo_root();
    let output_dir = root.join(&options.output);
    fs::create_dir_all(&output_dir)?;

    let mut command = Command::new("cargo");
    if let Some(toolchain) = &options.toolchain {
        command.arg(format!("+{toolchain}"));
    }
    command.current_dir(&root).arg("test");

    if options.all_features {
        command.arg("--all-features");
    }
    if options.locked {
        command.arg("--locked");
    }
    command.arg("--workspace").arg("--exclude").arg("airbug-mon");
    if options.doc_tests {
        command.arg("--doc");
    }

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    stdout.push_str(&stderr);

    let outcome = if output.status.success() { "passed" } else { "failed" };
    let commit = git_head()?.trim().to_string();

    let report = format!(
        "{{\n  \"title\": \"Airbug report\",\n  \"commit\": \"{commit}\",\n  \"tests\": [\n    {{\"suite\": \"cargo test\", \"name\": \"workspace\", \"kind\": \"suite\", \"status\": \"{outcome}\"}}\n  ]\n}}\n"
    );
    fs::write(output_dir.join("report.json"), report)?;
    fs::write(output_dir.join("index.html"), render_html(&outcome, &commit, &stdout))?;

    if !output.status.success() {
        eprintln!("{stdout}");
    }

    Ok(())
}

fn parse_args() -> Result<Options, Box<dyn std::error::Error>> {
    let mut all_features = false;
    let mut locked = false;
    let mut doc_tests = false;
    let mut output = PathBuf::from("target/airbug-report");
    let mut toolchain = None;

    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--all-features" => all_features = true,
            "--locked" => locked = true,
            "--doc-tests" => doc_tests = true,
            "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("--output requires a value".into());
                }
                output = PathBuf::from(&args[i]);
            }
            "--toolchain" => {
                i += 1;
                if i >= args.len() {
                    return Err("--toolchain requires a value".into());
                }
                toolchain = Some(args[i].clone());
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            value => return Err(format!("unknown argument: {value}").into()),
        }
        i += 1;
    }

    Ok(Options { all_features, locked, doc_tests, output, toolchain })
}

fn print_help() {
    eprintln!(
        "Usage: airbug-report [--all-features] [--locked] [--doc-tests] [--output PATH] [--toolchain TOOLCHAIN]"
    );
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(&manifest_dir).to_path_buf()
}

fn git_head() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").output()?;
    if !output.status.success() {
        return Ok(format!("{:.0}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn render_html(outcome: &str, commit: &str, logs: &str) -> String {
    let escaped_logs = logs
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    let status_style = if outcome == "passed" { "#dcfce7; color: #166534;" } else { "#fef2f2; color: #991b1b;" };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Airbug report</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2rem; color: #111827; background: #f8fafc; }}
    .card {{ background: white; border-radius: 12px; padding: 1.25rem; max-width: 1100px; margin: auto; box-shadow: 0 4px 18px rgba(15,23,42,0.06); }}
    .status {{ font-weight: 700; display: inline-block; padding: .35rem .7rem; border-radius: 999px; background: {status_style} }}
    pre {{ background: #0f172a; color: #e2e8f0; padding: 1rem; border-radius: 10px; overflow: auto; white-space: pre-wrap; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Airbug report</h1>
    <p>Commit: <code>{commit}</code></p>
    <p class="status">{outcome}</p>
    <pre>{escaped_logs}</pre>
  </div>
</body>
</html>
"#
    )
}
