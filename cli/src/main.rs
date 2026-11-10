//! `cargo airbug <command>` — one entry point for the toolkit.
//!
//! This binary owns no behaviour: it routes a command to the tool that implements it and
//! becomes that process, so flags, output and exit codes are the tool's own.
use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::{self, Command},
};

/// A tool this entry point can hand over to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Hub,
    Bench,
}

impl Tool {
    fn package(self) -> &'static str {
        match self {
            Tool::Hub => "airbug-hub",
            Tool::Bench => "cargo-airbug-bench",
        }
    }

    fn binary(self) -> &'static str {
        self.package()
    }

    /// Arguments the tool expects before the user's own.
    fn prefix(self) -> &'static [&'static str] {
        match self {
            Tool::Hub => &[],
            // The bench CLI is a cargo subcommand: cargo passes its name first.
            Tool::Bench => &["airbug-bench"],
        }
    }

    /// Benchmarks are only meaningful optimised; the hub starts faster unoptimised.
    fn release(self) -> bool {
        self == Tool::Bench
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Help,
    Unknown(String),
    Run(Tool, Vec<String>),
}

/// `(command, tool, what it does)` — the single list help and routing are built from.
const COMMANDS: &[(&str, Tool, &str)] = &[
    (
        "test",
        Tool::Hub,
        "run cargo test as a live per-test run   [-- <cargo test args>]",
    ),
    (
        "serve",
        Tool::Hub,
        "start the hub on http://127.0.0.1:8790/  [--collector] [--port N]",
    ),
    (
        "status",
        Tool::Hub,
        "print the hub's status snapshot as JSON",
    ),
    (
        "bench",
        Tool::Bench,
        "benchmarks: bench run | matrix | compare | report | … (bench --help)",
    ),
];

fn route(args: &[String]) -> Route {
    // `cargo airbug …` passes the subcommand name first; a direct call does not.
    let args = match args.first().map(String::as_str) {
        Some("airbug") => &args[1..],
        _ => args,
    };
    let Some(command) = args.first() else {
        return Route::Help;
    };
    if matches!(command.as_str(), "-h" | "--help" | "help") {
        return Route::Help;
    }
    match COMMANDS.iter().find(|(name, ..)| name == command) {
        // The hub parses its own command word; the bench CLI gets what follows `bench`.
        Some((_, Tool::Hub, _)) => Route::Run(Tool::Hub, args.to_vec()),
        Some((_, Tool::Bench, _)) => {
            let rest = &args[1..];
            let rest = if rest.is_empty() {
                vec!["--help".to_string()]
            } else {
                rest.to_vec()
            };
            Route::Run(Tool::Bench, rest)
        }
        None => Route::Unknown(command.clone()),
    }
}

fn help() -> String {
    let mut text = String::from(
        "cargo airbug — run, watch and compare tests and benchmarks\n\nUsage: cargo airbug <command> [args]\n\n",
    );
    for (name, _, about) in COMMANDS {
        text.push_str(&format!("  {name:<7} {about}\n"));
    }
    text.push_str("\nEvery run lands in .airbug/runs/ and shows up in the hub.\n");
    text
}

/// How to start `tool`: through cargo inside a workspace checkout (never a stale build), else
/// the binary installed next to this one, else whatever `PATH` has.
fn command_for(tool: Tool) -> Command {
    // `cargo run` sets this for the process it starts; an installed binary never sees it.
    if env::var_os("CARGO_MANIFEST_DIR").is_some() {
        let mut command =
            Command::new(env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")));
        command.args(["run", "-q", "-p", tool.package()]);
        if tool.release() {
            command.arg("--release");
        }
        command.arg("--");
        return command;
    }
    Command::new(sibling(tool).unwrap_or_else(|| PathBuf::from(tool.binary())))
}

fn sibling(tool: Tool) -> Option<PathBuf> {
    let path = env::current_exe().ok()?.with_file_name(format!(
        "{}{}",
        tool.binary(),
        env::consts::EXE_SUFFIX
    ));
    path.is_file().then_some(path)
}

fn hand_over(tool: Tool, args: Vec<String>) -> ! {
    let mut command = command_for(tool);
    command.args(tool.prefix()).args(args);
    // The tool reads its own configuration; this is only how it was reached.
    command.env_remove("CARGO_MANIFEST_DIR");
    #[cfg(unix)]
    let error = {
        use std::os::unix::process::CommandExt;
        // Replace this process: Ctrl+C and the exit code belong to the tool.
        command.exec()
    };
    #[cfg(not(unix))]
    let error = match command.status() {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(error) => error,
    };
    eprintln!(
        "cargo airbug: cannot start {}: {error}\n  install it with `cargo install {}`",
        tool.binary(),
        tool.package()
    );
    process::exit(127);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match route(&args) {
        Route::Help => print!("{}", help()),
        Route::Unknown(command) => {
            eprint!("cargo airbug: unknown command `{command}`\n\n{}", help());
            process::exit(2);
        }
        Route::Run(tool, args) => hand_over(tool, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn hub_commands_keep_their_command_word() {
        assert_eq!(
            route(&args(&["airbug", "test", "--", "--workspace"])),
            Route::Run(Tool::Hub, args(&["test", "--", "--workspace"]))
        );
        assert_eq!(
            route(&args(&["serve", "--collector"])),
            Route::Run(Tool::Hub, args(&["serve", "--collector"]))
        );
    }

    #[test]
    fn bench_passes_on_what_follows_it() {
        assert_eq!(
            route(&args(&["airbug", "bench", "run", "--program", "x"])),
            Route::Run(Tool::Bench, args(&["run", "--program", "x"]))
        );
        assert_eq!(
            route(&args(&["bench"])),
            Route::Run(Tool::Bench, args(&["--help"]))
        );
    }

    #[test]
    fn help_and_unknown() {
        assert_eq!(route(&args(&["airbug"])), Route::Help);
        assert_eq!(route(&args(&["--help"])), Route::Help);
        assert_eq!(route(&args(&["deploy"])), Route::Unknown("deploy".into()));
        for (name, ..) in COMMANDS {
            assert!(help().contains(name));
        }
    }
}
