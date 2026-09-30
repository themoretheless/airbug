use airbug::report::live::{Run, project_root};
use serde_json::Value;
use std::{
    env, fs,
    io::{self, Read},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Options {
    all_features: bool,
    locked: bool,
    doc_tests: bool,
    output: PathBuf,
    root: Option<PathBuf>,
    toolchain: Option<String>,
    filter: String,
    ignored: bool,
    include_ignored: bool,
    timeout: u64,
    cargo_args: Vec<String>,
}

fn main() {
    match run() {
        Ok(true) => (),
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("airbug-report: {error}");
            std::process::exit(1);
        }
    }
}

fn cargo(options: &Options) -> Command {
    let mut cmd = Command::new("cargo");
    if let Some(toolchain) = &options.toolchain {
        cmd.arg(format!("+{toolchain}"));
    }
    cmd.arg("test");
    if options.all_features {
        cmd.arg("--all-features");
    }
    if options.locked {
        cmd.arg("--locked");
    }
    if options.cargo_args.is_empty() {
        cmd.args(["--workspace", "--exclude", "airbug-mon"]);
    } else {
        cmd.args(&options.cargo_args);
    }
    cmd
}

fn run() -> Result<bool> {
    let options = parse_args_from(env::args().skip(1))?;
    let root = options
        .root
        .clone()
        .map_or_else(project_root, |p| p.canonicalize())?;
    let output = root.join(&options.output);
    let mut run = Run::new(&output, "test", "Cargo tests")?;
    eprintln!(
        "Airbug run: {}\nHistory: {}",
        run.id,
        run.directory.display()
    );
    run.message("Building test executables")?;
    let result = execute(&options, &root, &mut run);
    let passed = match &result {
        Ok(passed) => *passed,
        Err(_) => false,
    };
    let message = match &result {
        Ok(true) => "Tests finished".into(),
        Ok(false) => "Some tests failed; open a case for its output".into(),
        Err(error) => error.to_string(),
    };
    run.finish(passed, &message)?;
    // Retain the existing report paths for CI artifacts and the hub's /report/ page.
    fs::copy(run.directory.join("run.json"), output.join("report.json"))?;
    fs::write(output.join("index.html"), render_html(&run, &message))?;
    result
}

struct TestCase {
    executable: PathBuf,
    working_dir: PathBuf,
    name: String,
    index: usize,
}

fn execute(options: &Options, root: &std::path::Path, run: &mut Run) -> Result<bool> {
    let build = cargo(options)
        .current_dir(root)
        .args(["--no-run", "--message-format=json-render-diagnostics"])
        .output()?;
    io::Write::write_all(&mut io::stderr(), &build.stderr)?;
    let messages: Vec<Value> = String::from_utf8_lossy(&build.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    if !build.status.success() {
        let diagnostics = messages
            .iter()
            .filter_map(|v| v["message"]["rendered"].as_str())
            .collect::<String>();
        return Err(format!(
            "Compilation failed\n{diagnostics}\n{}",
            String::from_utf8_lossy(&build.stderr)
        )
        .into());
    }
    // Cargo supplies a concrete compiler to test processes. Preserve that contract
    // for tests that themselves invoke Cargo from a temporary consumer workspace.
    let mut compiler = Command::new(env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    compiler.current_dir(root).args(["--print", "sysroot"]);
    if let Some(toolchain) = &options.toolchain {
        compiler.env("RUSTUP_TOOLCHAIN", toolchain);
    }
    let compiler = compiler.output()?;
    if !compiler.status.success() {
        return Err(format!(
            "Could not resolve test compiler: {}",
            String::from_utf8_lossy(&compiler.stderr)
        )
        .into());
    }
    let compiler_root = PathBuf::from(String::from_utf8_lossy(&compiler.stdout).trim());
    let executable_suffix = std::env::consts::EXE_SUFFIX;
    let mut tests = Vec::new();
    let mut executables = std::collections::BTreeSet::new();
    for message in &messages {
        if message["reason"] != "compiler-artifact" || message["profile"]["test"] != true {
            continue;
        }
        let Some(executable) = message["executable"].as_str() else {
            continue;
        };
        if !executables.insert(executable.to_owned()) {
            continue;
        }
        let suite = format!(
            "{} · {}",
            message["package_id"]
                .as_str()
                .unwrap_or("package")
                .rsplit('#')
                .next()
                .unwrap_or("package"),
            message["target"]["name"].as_str().unwrap_or("tests")
        );
        let working_dir = message["manifest_path"]
            .as_str()
            .and_then(|p| std::path::Path::new(p).parent())
            .unwrap_or(root);
        run.message(&format!("Discovering {suite}"))?;
        let all = list_tests(executable, working_dir, false)?;
        let ignored = list_tests(executable, working_dir, true)?;
        for name in all {
            if !name.contains(&options.filter) {
                continue;
            }
            let is_ignored = ignored.contains(&name);
            if options.ignored && !is_ignored {
                continue;
            }
            let skip = is_ignored && !options.ignored && !options.include_ignored;
            let index = run.add_case(&suite, &name, skip);
            if !skip {
                tests.push(TestCase {
                    executable: executable.into(),
                    working_dir: working_dir.into(),
                    name,
                    index,
                });
            }
        }
    }
    run.save()?;
    let mut passed = true;
    for test in tests {
        let diagnostics = run.start_case(test.index)?;
        let mut cmd = Command::new(&test.executable);
        cmd.current_dir(&test.working_dir)
            .args(["--exact", &test.name, "--show-output", "--color", "never"])
            .env("AIRBUG_REPORT_DIR", &diagnostics)
            .env("CARGO_MANIFEST_DIR", &test.working_dir)
            .env(
                "RUSTC",
                env::var_os("RUSTC").unwrap_or_else(|| {
                    compiler_root
                        .join("bin")
                        .join(format!("rustc{executable_suffix}"))
                        .into_os_string()
                }),
            )
            .env(
                "RUSTDOC",
                env::var_os("RUSTDOC").unwrap_or_else(|| {
                    compiler_root
                        .join("bin")
                        .join(format!("rustdoc{executable_suffix}"))
                        .into_os_string()
                }),
            );
        if let Some(toolchain) = &options.toolchain {
            cmd.env("RUSTUP_TOOLCHAIN", toolchain);
        }
        if options.ignored || options.include_ignored {
            cmd.arg("--include-ignored");
        }
        let (success, timed_out, output) = capture(cmd, options.timeout)?;
        // Exit zero alone is not proof that a custom harness executed the selected test.
        let executed = output.contains("1 passed; 0 failed;");
        let status = if timed_out {
            "broken"
        } else if success && executed {
            "passed"
        } else {
            "failed"
        };
        passed &= status == "passed";
        run.finish_case(test.index, status, &output, None)?;
        eprintln!("{status}: {}", test.name);
    }
    if options.doc_tests {
        let index = run.add_case("cargo test --doc", "Documentation tests (suite)", false);
        run.start_case(index)?;
        let mut docs = cargo(options);
        docs.current_dir(root).arg("--doc");
        let (success, timed_out, output) = capture(docs, options.timeout)?;
        passed &= success;
        run.finish_case(
            index,
            if timed_out {
                "broken"
            } else if success {
                "passed"
            } else {
                "failed"
            },
            &output,
            None,
        )?;
    }
    Ok(passed)
}

fn list_tests(executable: &str, root: &std::path::Path, ignored: bool) -> Result<Vec<String>> {
    let mut command = Command::new(executable);
    command
        .current_dir(root)
        .args(["--list", "--format", "pretty"]);
    if ignored {
        command.arg("--ignored");
    }
    let output = command.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || !stdout
            .lines()
            .any(|l| l.ends_with(" benchmarks") || l.ends_with(" benchmark"))
    {
        return Err(format!("Test executable does not support the standard Rust test harness: {executable}\n{stdout}\n{}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test").map(str::to_owned))
        .collect())
}

fn capture(mut command: Command, timeout: u64) -> Result<(bool, bool, String)> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("missing stdout")?;
    let stderr = child.stderr.take().ok_or("missing stderr")?;
    let (sender, receiver) = std::sync::mpsc::channel();
    let errors = sender.clone();
    thread::spawn(move || {
        let _ = sender.send(bounded_read(stdout));
    });
    thread::spawn(move || {
        let _ = errors.send(bounded_read(stderr));
    });
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > Duration::from_secs(timeout) {
            timed_out = true;
            child.kill()?;
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let mut output = String::new();
    for _ in 0..2 {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(text) => output.push_str(&text?),
            Err(_) => {
                output.push_str(
                    "\nOutput incomplete: a descendant process kept the capture pipe open",
                );
                break;
            }
        }
    }
    if timed_out {
        output.push_str("\nTest process exceeded timeout and was killed");
    }
    Ok((status.success(), timed_out, output))
}

fn bounded_read(mut input: impl Read) -> io::Result<String> {
    let mut result = Vec::new();
    let mut truncated = false;
    let mut buf = [0; 8192];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        result.extend_from_slice(&buf[..n]);
        if result.len() > 64 * 1024 {
            result.drain(..result.len() - 64 * 1024);
            truncated = true;
        }
    }
    Ok(format!(
        "{}{}",
        if truncated {
            "[earlier output truncated]\n"
        } else {
            ""
        },
        String::from_utf8_lossy(&result)
    ))
}

fn parse_args_from<I, S>(args: I) -> Result<Options>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut options = Options {
        all_features: false,
        locked: false,
        doc_tests: false,
        output: "target/airbug-report".into(),
        root: None,
        toolchain: None,
        filter: String::new(),
        ignored: false,
        include_ignored: false,
        timeout: 300,
        cargo_args: Vec::new(),
    };
    let mut args = args.into_iter().map(|s| s.as_ref().to_owned());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--all-features" => options.all_features = true,
            "--locked" => options.locked = true,
            "--doc-tests" => options.doc_tests = true,
            "--ignored" => options.ignored = true,
            "--include-ignored" => options.include_ignored = true,
            "--output" | "--toolchain" | "--root" | "--filter" | "--timeout-seconds" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                match arg.as_str() {
                    "--output" => options.output = value.into(),
                    "--root" => options.root = Some(value.into()),
                    "--filter" => options.filter = value,
                    "--timeout-seconds" => {
                        options.timeout = value.parse()?;
                        if options.timeout == 0 {
                            return Err("timeout must be positive".into());
                        }
                    }
                    _ => options.toolchain = Some(value),
                }
            }
            "--" => {
                options.cargo_args.extend(args);
                break;
            }
            "--help" | "-h" => {
                println!(
                    "airbug_report [--root PATH] [--all-features] [--locked] [--doc-tests] [--filter TEXT] [--ignored|--include-ignored] [--timeout-seconds N] [--output PATH] [--toolchain NAME] [-- CARGO TEST OPTIONS]\nEach test runs in its own process. Duration includes process startup. Doctests are one suite. History: target/airbug-report/runs; hub: #/runs."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    // These options change the discovery protocol rather than selecting/building targets.
    for arg in &options.cargo_args {
        if [
            "--",
            "--doc",
            "--no-run",
            "--message-format",
            "--manifest-path",
        ]
        .contains(&arg.as_str())
            || arg.starts_with("--message-format=")
            || arg.starts_with("--manifest-path=")
        {
            return Err(format!(
                "unsupported discovery option {arg}; use --root, --filter or --doc-tests before --"
            )
            .into());
        }
    }
    Ok(options)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn render_html(run: &Run, message: &str) -> String {
    let rows = run.cases.iter().map(|c| format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.3} s</td><td><details><summary>Output</summary><pre>{}</pre></details></td></tr>", escape(&c.suite), escape(&c.name), escape(&c.status), c.duration.unwrap_or_default(), escape(&c.output))).collect::<String>();
    format!(
        "<!doctype html><meta charset=utf-8><title>Airbug report</title><style>body{{font:15px system-ui;margin:2rem}}td,th{{padding:.6rem;text-align:left;border-bottom:1px solid #ddd}}pre{{white-space:pre-wrap;max-width:60rem}}table{{width:100%}}</style><h1>Airbug test report</h1><p>{}</p><p>Run {}</p><table><tr><th>Suite</th><th>Test</th><th>Status</th><th>Process duration</th><th>Details</th></tr>{rows}</table>",
        escape(message),
        escape(&run.id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_args_accepts_rust_report_flags() {
        let options = parse_args_from([
            "--all-features",
            "--locked",
            "--doc-tests",
            "--output",
            "artifact/report",
            "--toolchain",
            "stable",
        ])
        .unwrap();
        assert!(options.all_features && options.locked && options.doc_tests);
        assert_eq!(options.output, PathBuf::from("artifact/report"));
        assert_eq!(options.toolchain.as_deref(), Some("stable"));
    }
    #[test]
    fn parse_args_rejects_missing_value() {
        assert!(
            parse_args_from(["--output"])
                .unwrap_err()
                .to_string()
                .contains("requires a value")
        );
    }
    #[test]
    fn parse_args_builds_default_options() {
        let options = parse_args_from(std::iter::empty::<&str>()).unwrap();
        assert_eq!(options.output, PathBuf::from("target/airbug-report"));
        assert!(!options.all_features && !options.locked && !options.doc_tests);
    }
    #[test]
    fn accepts_target_selection_and_rejects_protocol_override() {
        assert_eq!(
            parse_args_from(["--filter", "hello", "--", "-p", "airbug"])
                .unwrap()
                .cargo_args,
            ["-p", "airbug"]
        );
        assert!(parse_args_from(["--", "--message-format=json"]).is_err());
    }
}
