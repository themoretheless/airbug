//! Static ASCII harness help with terminal-aware wrapping.
const SHORT: &str = "Airbug benchmarks\n\n  cargo bench                       Measure and save an HTML report\n  cargo bench FILTER                Run matching cases\n  cargo bench -- --list             List cases without running them\n  cargo bench -- --quick            Short exploratory run\n  cargo bench -- --test             Check each case once\n  cargo bench -- --help-all         Show all options\n\nResults are compared with the previous successful run automatically.\nReport: target/airbug-bench/reports/<run-id>/report.html";
const FULL: &str = "Airbug benchmarks\n\nStart here:\n  cargo bench                       Measure, compare with previous run, save HTML report\n  cargo bench FILTER                Run matching cases\n  cargo bench -- --list             List cases without measuring\n  cargo bench -- --quick            Short exploratory run\n  cargo bench -- --discard          Measure without saving results\n  cargo bench -- --quiet            Results without progress or detailed tables\n  cargo bench -- --verbose          Include environment and case contracts\n  cargo bench -- --output-format tree  Hierarchical results and lists\n  cargo bench -- --color auto       Color: auto, always or never (auto respects NO_COLOR)\n\nReports: target/airbug-bench/reports/<run-id>/report.html\nUse --output NEW_DIRECTORY to choose a destination.\nSampling environment: AIRBUG_BENCH_SAMPLES, AIRBUG_BENCH_SAMPLE_COUNT_UNIT,\nAIRBUG_BENCH_WORKER_START, AIRBUG_BENCH_ITERATIONS, AIRBUG_BENCH_WARMUP_MS,\nAIRBUG_BENCH_SAMPLE_MS, AIRBUG_BENCH_MEASUREMENT_MS, AIRBUG_BENCH_SAMPLING, AIRBUG_BENCH_MIN_TIME_MS,\nAIRBUG_BENCH_MAX_TIME_MS, AIRBUG_BENCH_EXCLUDE_EXTERNAL_TIME.\nAlso: AIRBUG_BENCH_TIMER, AIRBUG_BENCH_SORT, AIRBUG_BENCH_REVERSE, AIRBUG_BENCH_BYTES_FORMAT.\nTime flags ending in -ms accept decimal milliseconds (0.000001 ms = 1 ns).\nCLI flags take precedence; --forward overrides reverse sorting from the environment.\nEnable concurrent cases (except threads = false): --threads N[,N...] (repeatable; 0 = available parallelism), AIRBUG_BENCH_THREADS.\nCounter overrides: --bytes-count N, --items-count N, --chars-count N, --cycles-count N, --bits-count N.\nEnvironment: AIRBUG_BENCH_BYTES_COUNT, AIRBUG_BENCH_ITEMS_COUNT, AIRBUG_BENCH_CHARS_COUNT, AIRBUG_BENCH_CYCLES_COUNT, AIRBUG_BENCH_BITS_COUNT.\nThe first run establishes a baseline; insufficient evidence stays inconclusive.\n\nAll options:\nSelection:\n  [FILTER ...] --filter TEXT (repeatable; match any) --exact --glob --regex\n  --exclude GLOB --exclude-exact PATH --exclude-regex REGEX --tag TAG\n  --include-ignored --ignored --list [--format terse] --dry-run --test\n  --nocapture --show-output (workload output is already uncaptured)\n\nMeasurement:\n  --profile quick|normal|thorough --samples N --iterations N\n  --sample-count-unit batches|workers --worker-start shared|local\n  --sampling flat|linear|auto --sample-ms N | --measurement-ms N --warmup-ms N\n  --min-time-ms N --max-time-ms N\n  --exclude-external-time --include-external-time\n  --timer os|cpu --overhead raw|subtract --threads N[,N...]\n  --quick --quick-min-ms N --quick-max-ms N --quick-relative-deviation F\n  --profile-time-ms N\n\nComparison and baselines:\n  --load-baseline NAME --save-baseline NAME --replace-baseline NAME\n  --retain-baseline NAME --baseline NAME --baseline-lenient NAME\n  --baseline-store DIRECTORY --history-dir DIRECTORY --no-history\n  --significance-level FRACTION --noise-threshold-percent PERCENT\n  --hypothesis-resamples N --hypothesis-seed N --hypothesis-distribution\n\nStatistics:\n  --resamples N --confidence-level FRACTION --analysis-seed N\n  --bootstrap-distributions --no-bootstrap\n  Default: 95% confidence, 10000 resamples; analysis runs after measurement.\n\nReports and display:\n  --output NEW_DIRECTORY --discard --json --plots --no-plots --no-html\n  --quiet --verbose --output-format table|tree --color auto|always|never\n  --sort registration|lexical|natural|source|kind --reverse --forward\n  --bytes-format decimal|binary\n  --bytes-count N --items-count N --chars-count N --cycles-count N --bits-count N\n  --summary-parameter NAME --summary-estimator process-median|mean\n  --summary-scale linear|logarithmic\n\n  --version / -V: library version without registration or measurement.\n  -h: quick help. --bench: accepted for Cargo's benchmark harness.";

pub(crate) fn print_if_requested(args: &[String]) -> bool {
    let text = if args.iter().any(|arg| arg == "--help-all") {
        FULL
    } else if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        SHORT
    } else {
        return false;
    };
    let columns = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse().ok());
    let detected = terminal_size::terminal_size().map(|(width, _)| usize::from(width.0));
    println!("{}", wrap(text, width(columns, detected)));
    true
}

fn width(columns: Option<usize>, detected: Option<usize>) -> usize {
    columns
        .filter(|value| *value > 0)
        .or(detected.filter(|value| *value > 0))
        .unwrap_or(80)
}

// Help is ASCII. Keep indivisible option names and paths intact even when they
// exceed the available width; splitting them would make copied commands invalid.
fn wrap(text: &str, width: usize) -> String {
    let mut result = Vec::new();
    for line in text.lines() {
        if line.len() <= width {
            result.push(line.to_owned());
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let content = line.trim_start();
        // Example command/description rows become two blocks on narrow screens.
        if let Some((command, description)) = content.split_once("  ") {
            result.push(format!("{}{}", " ".repeat(indent), command));
            append_wrapped(&mut result, description.trim(), indent + 2, width);
        } else {
            append_wrapped(&mut result, content, indent, width);
        }
    }
    result.join("\n")
}

fn append_wrapped(result: &mut Vec<String>, text: &str, indent: usize, width: usize) {
    let prefix = " ".repeat(indent.min(width.saturating_sub(1)));
    let mut line = prefix.clone();
    for word in text.split_whitespace() {
        if line.len() > prefix.len() && line.len() + 1 + word.len() > width {
            result.push(line);
            line = prefix.clone();
        }
        if line.len() > prefix.len() {
            line.push(' ');
        }
        line.push_str(word);
    }
    result.push(line);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrapping_preserves_all_help_words_and_commands() {
        for text in [SHORT, FULL] {
            for width in [20, 40, 60, 80, 120] {
                let rendered = wrap(text, width);
                assert_eq!(
                    rendered.split_whitespace().collect::<Vec<_>>(),
                    text.split_whitespace().collect::<Vec<_>>()
                );
                assert!(rendered.contains("cargo bench -- --list"));
                for line in rendered.lines().filter(|line| line.len() > width) {
                    // Long tokens/commands stay intact; prose must fit.
                    assert!(
                        line.trim_start().starts_with("cargo bench")
                            || line.split_whitespace().count() == 1,
                        "{width}: {line}"
                    );
                }
            }
        }
    }
    #[test]
    fn width_override_and_fallback() {
        assert_eq!(width(Some(40), Some(100)), 40);
        assert_eq!(width(Some(0), Some(60)), 60);
        assert_eq!(width(None, None), 80);
        assert_eq!(width(Some(0), Some(0)), 80);
    }
}
