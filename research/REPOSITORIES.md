# Корпус исследования

Автоматический скрининг README; сигналы означают упоминание, а не проверенную возможность. Подробные решения по ключевым проектам находятся в FOCUSED.md.

## sharkdp/hyperfine

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sharkdp/hyperfine
- Категория: benchmark/testing candidate
- Описание: A command-line benchmarking tool
- Уровень: automated README evidence extraction
- Снимок: [sources/sharkdp__hyperfine/README.md](sources/sharkdp__hyperfine/README.md); SHA-256: `399ef574c247bb7421b02eaa4b410b068eda747dce3e2a4690a9358b023ae16b`
- statistics: строка 15: * Statistical analysis across multiple runs.
- comparison: строка 324: track benchmarks and catch performance regressions in CI.
- lifecycle: строка 18: * Warmup runs can be executed before the actual benchmark.
- report: строка 21: * Export results to various formats: CSV, JSON, Markdown, AsciiDoc.

## hatoo/oha

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/hatoo/oha
- Категория: benchmark/testing candidate
- Описание: Ohayou(おはよう), HTTP load generator, inspired by rakyll/hey with tui animation.
- Уровень: automated README evidence extraction
- Снимок: [sources/hatoo__oha/README.md](sources/hatoo__oha/README.md); SHA-256: `1877be412102932dbe4101f96ccc2f0788430d14df9eb2884b2ab783210d7bb2`
- async: строка 121: Number of connections to run concurrently. You may should increase limit to number of open files for larger `-c`. [default: 50]
- report: строка 230: Output format [default: text] [possible values: text, json, csv, quiet]

## bheisler/criterion.rs

- Итог: alias — Historical Criterion location; canonical criterion-rs/criterion.rs counted separately.
- Источник: https://github.com/bheisler/criterion.rs
- Категория: benchmark/testing candidate
- Описание: Statistics-driven benchmarking library for Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/bheisler__criterion.rs/README.md](sources/bheisler__criterion.rs/README.md); SHA-256: `273c1b55c2ed47236bb45190b82cb9c9111df0441070a1c6c531156398310e4e`

## pawurb/hotpath-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pawurb/hotpath-rs
- Категория: profiling
- Описание: Rust profiler for CPU, memory, SQL, HTTP, and async performance, with Prometheus and Grafana support.
- Уровень: automated README evidence extraction
- Снимок: [sources/pawurb__hotpath-rs/README.md](sources/pawurb__hotpath-rs/README.md); SHA-256: `5d79a2b9ca0135d37bccd635d909af81caae071ed987e2ad6c773e4bbde11dd9`
- comparison: строка 54: - **CI regression detection** - benchmark every PR automatically.
- lifecycle: строка 6: It helps you distinguish between functions that are slow because they wait on I/O and those that are CPU-intensive. Instrument functions, channels, futures, streams, SQL queries, HTTP calls, and byte-level I/O to find bottlenecks and focus optimizations where
- memory: строка 4: hotpath-rs is an easy-to-configure Rust performance profiler that shows exactly where your code spends time, burns CPU, and allocates memory.
- async: строка 6: It helps you distinguish between functions that are slow because they wait on I/O and those that are CPU-intensive. Instrument functions, channels, futures, streams, SQL queries, HTTP calls, and byte-level I/O to find bottlenecks and focus optimizations where
- report: строка 6: It helps you distinguish between functions that are slow because they wait on I/O and those that are CPU-intensive. Instrument functions, channels, futures, streams, SQL queries, HTTP calls, and byte-level I/O to find bottlenecks and focus optimizations where

## nvzqz/divan

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nvzqz/divan
- Категория: benchmark/testing candidate
- Описание: Fast and simple benchmarking for Rust projects
- Уровень: automated README evidence extraction
- Снимок: [sources/nvzqz__divan/README.md](sources/nvzqz__divan/README.md); SHA-256: `27d8ea2c8361591b1e2c1aa4a1d5a916d27b79f6771327eaa9da56e68e905d9c`

## djkoloski/rust_serialization_benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/djkoloski/rust_serialization_benchmark
- Категория: benchmark/testing candidate
- Описание: Benchmarks for rust serialization frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/djkoloski__rust_serialization_benchmark/README.md](sources/djkoloski__rust_serialization_benchmark/README.md); SHA-256: `d5b749ff3df5f570290b742ad11c1d1d11f76f8d0f0038766e40d5cde00a6e54`
- async: строка 116: Vulnerability Tsx async abort:           Not affected
- report: строка 143: | json:<br> [flexon 0.4.8][flexon] | 2.7574 ms | 4.2182 ms | † | 1827461 | 470560 | 360727 | 5.9997 ms |
- correctness: строка 155: | protobuf:<br> [protobuf 4.35.1-release][protobuf4] | <span title="encode">*1.6699 ms\**</span> <span title="populate + encode">*7.4644 ms\**</span> | <span title="decode + convert">*7.4883 ms\**</span> <span title="decode, unvalidated">*2.9341 ms\**</span> |

## us/crw

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/us/crw
- Категория: benchmark/testing candidate
- Описание: Fast, lightweight Firecrawl/Tavily alternative in Rust. Web scraper, crawler & search API with MCP server for AI agents. Drop-in Firecrawl-compatible API (/scrape, /crawl, /search). 2.3x faster than Tavily, 1.5x faster than Firecrawl in 1K-URL benchmarks. 6 MB RAM, single binary. Self-host or use managed cloud.
- Уровень: automated README evidence extraction
- Снимок: [sources/us__crw/README.md](sources/us__crw/README.md); SHA-256: `19266f96c668557691cde488ebb80d97b892120cf21fba842051102e2f1eb035`
- lifecycle: строка 56: tool. Add `CRW_NO_AGENTS=1` to skip that step, or run `crw setup` on its own to
- async: строка 81: Crawl4AI and Firecrawl, matched the fastest median latency, and idled at
- report: строка 9: <strong>JSON</strong> with one engine for search, scrape, map, crawl, and extract.

## bencherdev/bencher

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bencherdev/bencher
- Категория: benchmark/testing candidate
- Описание: 🐰 Bencher - Continuous Benchmarking
- Уровень: automated README evidence extraction
- Снимок: [sources/bencherdev__bencher/README.md](sources/bencherdev__bencher/README.md); SHA-256: `5c77954acf51b6762265e18fc5631c2fc3911c4701ba60b10a7fd01bd759c4b8`
- statistics: строка 40: - Typical CI runners: **>30% variance**
- comparison: строка 16: It tracks results over time and fails the PR when there's a performance regression.
- gpu: строка 12: **Run locally. Run in CI. Same bare metal every time.**
- async: строка 92: <h2><a href="https://bencher.dev/perf/bencher/reports/36a1eeff-57f5-4b99-b058-8c9c240a9f2c?utm_medium=referral&utm_source=github&utm_content=readme&utm_campaign=readme&utm_term=bencher"><img src="https://bencher.dev/favicon.svg" width="24" height="24" alt="🐰"
- report: строка 92: <h2><a href="https://bencher.dev/perf/bencher/reports/36a1eeff-57f5-4b99-b058-8c9c240a9f2c?utm_medium=referral&utm_source=github&utm_content=readme&utm_campaign=readme&utm_term=bencher"><img src="https://bencher.dev/favicon.svg" width="24" height="24" alt="🐰"
- correctness: строка 74: Local benchmarks aren't reproducible. Every check means stopping work to pull the baseline branch and wait on a comparison. Most engineers skip it.

## suyoumo/ClawProBench

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/suyoumo/ClawProBench
- Категория: benchmark/testing candidate
- Описание: ClawProBench is a live-first benchmark harness for evaluating LLM agents   in the OpenClaw runtime with deterministic grading and repeated-trial   reliability.
- Уровень: automated README evidence extraction
- Снимок: [sources/suyoumo__ClawProBench/README.md](sources/suyoumo__ClawProBench/README.md); SHA-256: `d5129e67c12a66021460545a59f8f884c7a03cc702b82584ad28580e909453ee`
- comparison: строка 175: | `coverage` | 7 | Lower-stakes breadth and regression slice |
- lifecycle: строка 46: If domestic third-party API gateway providers would like their served models, such as Claude 4.7 Opus or GPT-5.5, to appear on the leaderboard, please contact us. We can run the benchmark and publish reproducible results when the evaluation setup is stable.
- async: строка 104: - Reports expose `avg_score`, `max_score`, coverage-aware summaries, cost, latency, and resume metadata
- report: строка 28: ClawProBench focuses on real OpenClaw execution with deterministic grading, structured reports, and benchmark-profile selection. The default ranking path is the `core` profile; broader active coverage remains available through `intelligence`, `coverage`, `nati
- correctness: строка 46: If domestic third-party API gateway providers would like their served models, such as Claude 4.7 Opus or GPT-5.5, to appear on the leaderboard, please contact us. We can run the benchmark and publish reproducible results when the evaluation setup is stable.

## bheisler/iai

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bheisler/iai
- Категория: profiling
- Описание: Experimental one-shot benchmarking/profiling harness for Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/bheisler__iai/README.md](sources/bheisler__iai/README.md); SHA-256: `ab7d7ca89942ca4ebfaae420a84eee3784919ce56c66ebbb556f9d6830d3ff3e`
- statistics: строка 37: - __Performance__: Since Iai only executes a benchmark once, it is typically faster to run than statistical benchmarks
- comparison: строка 128: For benchmarks that run in CI (especially if you're checking for performance regressions in pull
- lifecycle: строка 123: - Con: Iai cannot exclude setup code from the measurements, where Criterion-rs can.
- report: строка 115: - Temporary Con: Right now, Iai is lacking many features of Criterion-rs, including reports and configuration of any kind.

## lnx-search/rewrk

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lnx-search/rewrk
- Категория: benchmark/testing candidate
- Описание: A more modern http framework benchmarker supporting HTTP/1 and HTTP/2 benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/lnx-search__rewrk/README.md](sources/lnx-search__rewrk/README.md); SHA-256: `ef86677fc886fda166433c73484d105dfc63aad94e789069a0639d23734023c9`
- async: строка 20: |   Percentile    |   Avg Latency   |
- report: строка 48: - JSON deserialization and validation benchmarks and checking.
- correctness: строка 48: - JSON deserialization and validation benchmarks and checking.

## Voultapher/sort-research-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Voultapher/sort-research-rs
- Категория: benchmark/testing candidate
- Описание: Test and benchmark suite for sort implementations.
- Уровень: automated README evidence extraction
- Снимок: [sources/Voultapher__sort-research-rs/README.md](sources/Voultapher__sort-research-rs/README.md); SHA-256: `8849fbe66d17a2831fd4610ee9d2323be4d23a85c402e89517945320eb15ec87`
- comparison: строка 113: Please **open an issue before** investing the effort of adding a new sort implementation. The maintainer of this project is currently not interested in building an up-to-date database of all existing sort implementations. Implementations are added based on sit
- report: строка 67: # Will write results to my_test_zen3.json

## lance0/xfr

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lance0/xfr
- Категория: benchmark/testing candidate
- Описание: A modern iperf3 alternative with a live TUI, multi-client server, and QUIC support. Built in Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/lance0__xfr/README.md](sources/lance0__xfr/README.md); SHA-256: `32a283e505dcb4d3571ad2f188a57a29fcc0d1737c74b7611d03edc6ab810977`
- comparison: строка 44: - **Result comparison** - `xfr diff` to detect performance regressions
- lifecycle: строка 27: See [Installation](#installation) below for setup instructions.
- memory: строка 599: - **DataHello flood protection**: DataHello messages for unknown test IDs are rejected immediately without allocating resources.
- async: строка 32: <img src="docs/demo.gif" alt="xfr live TUI: real-time throughput graph, per-second stats, and settings" width="900">
- report: строка 38: - **Server dashboard** - `xfr serve --tui` for monitoring active tests
- correctness: строка 591: - **Single-port TCP**: TCP uses single-port mode by default -- control and data connections share port 5201. Data connections are validated against the control connection's IP address, preventing unauthorized access.
- isolation: строка 571: VPN when you need centrally managed endpoint identities or isolation between

## reyamira/models

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/reyamira/models
- Категория: benchmark/testing candidate
- Описание: TUI and CLI for browsing models.dev, benchmarks, coding agents, and statuses for AI providers.
- Уровень: automated README evidence extraction
- Снимок: [sources/reyamira__models/README.md](sources/reyamira__models/README.md); SHA-256: `7025f23ed2ae9eb42263bf8d30d99393bd76426189fbb741d430e8e0fb6e8511`
- report: строка 28: - **CLI parity** — `models show` and `--json` include the new description, structured-output, reasoning-controls, and pricing fields.

## twitter/rpc-perf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/twitter/rpc-perf
- Категория: benchmark/testing candidate
- Описание: A tool for benchmarking RPC services
- Уровень: automated README evidence extraction
- Снимок: [sources/twitter__rpc-perf/README.md](sources/twitter__rpc-perf/README.md); SHA-256: `63146fe54d69e1786efdbafb1a93a1cf72cacc46195d6a4205ed9dfc6cc614c8`
- lifecycle: строка 61: * If comparing latency between two setups, be sure to set a ratelimit that's
- async: строка 61: * If comparing latency between two setups, be sure to set a ratelimit that's
- report: строка 107: Please report sensitive security issues via Twitter's bug-bounty program

## alacritty/vtebench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alacritty/vtebench
- Категория: benchmark/testing candidate
- Описание: Generate benchmarks for terminal emulators
- Уровень: automated README evidence extraction
- Снимок: [sources/alacritty__vtebench/README.md](sources/alacritty__vtebench/README.md); SHA-256: `0c26807447f62897329f21a33b7cba0dbc84c5978719b5021ebf07ae990e6e9b`
- lifecycle: строка 18: defined as a directory with a `benchmark` and an optional `setup` executable.
- async: строка 9: frame rate or latency. The only factor this benchmark stresses is the speed at

## BurntSushi/cargo-benchcmp

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/BurntSushi/cargo-benchcmp
- Категория: benchmark/testing candidate
- Описание: A small utility to compare Rust micro-benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/BurntSushi__cargo-benchcmp/README.md](sources/BurntSushi__cargo-benchcmp/README.md); SHA-256: `65812a3ff0b9a06d8090404f3ef32ed930b5e4562c3adafa1b720d629e08f70f`
- comparison: строка 103: Or only see regressions:

## CodSpeedHQ/codspeed

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/CodSpeedHQ/codspeed
- Категория: benchmark/testing candidate
- Описание: CodSpeed is the all-in-one performance testing toolkit. Optimize code performance and catch regressions early.
- Уровень: automated README evidence extraction
- Снимок: [sources/CodSpeedHQ__codspeed/README.md](sources/CodSpeedHQ__codspeed/README.md); SHA-256: `2c910fcad675aaadc890f7ffc5ab64134a6e1efa8cc23a19a7a03af4ea4c84e6`
- statistics: строка 28: - 🎯 **<1% variance** in measurements using CPU simulation - no more flaky benchmarks.
- comparison: строка 9: <h3 align="center">Optimize code performance and catch regressions early.</h3>
- lifecycle: строка 117: warmup-time: "0.2s"
- memory: строка 161: Tracks heap allocations (peak usage, count, allocation size) with eBPF profiling.
- report: строка 21: <a href="https://codspeed.io/?utm_source=badge"><img src="https://img.shields.io/endpoint?url=https://codspeed.io/badge.json" alt="CodSpeed Badge"></a>

## gungraun/gungraun

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gungraun/gungraun
- Категория: benchmark/testing candidate
- Описание: High-precision, one-shot and consistent benchmarking framework/harness for Rust. All Valgrind tools at your fingertips.
- Уровень: automated README evidence extraction
- Снимок: [sources/gungraun__gungraun/README.md](sources/gungraun__gungraun/README.md); SHA-256: `0427cb09cbf87889f3800bf25bceba7810ed06b1077b2a7d1d1c7f4f37619194`
- statistics: строка 43: detect outliers, filter out noise, etc.
- comparison: строка 36: small optimizations and regressions of your code.
- memory: строка 136: - [dhat-rs]: Provides heap profiling and ad hoc profiling capabilities to Rust
- async: строка 83: with full support for benchmarking async, multi-threaded and multi-process

## hoodie/concatenation_benchmarks-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/hoodie/concatenation_benchmarks-rs
- Категория: benchmark/testing candidate
- Описание: 📈 benchmarking different ways to concatenate strings in rust
- Уровень: automated README evidence extraction
- Снимок: [sources/hoodie__concatenation_benchmarks-rs/README.md](sources/hoodie__concatenation_benchmarks-rs/README.md); SHA-256: `50b7e51ed8aebc7cd0c91743084aabd860abdb6018e1a43f92007bdf5457ad14`

## MystenLabs/fastcrypto

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/MystenLabs/fastcrypto
- Категория: benchmark/testing candidate
- Описание: Common cryptographic library used in software at Mysten Labs.
- Уровень: automated README evidence extraction
- Снимок: [sources/MystenLabs__fastcrypto/README.md](sources/MystenLabs__fastcrypto/README.md); SHA-256: `a4780c1c20f41ec1b6a3dad47d7b141cb947e1566e2045dfa4b595b026d20a62`
- report: строка 95: A [report of the benchmarks](https://mystenlabs.github.io/fastcrypto/benchmarks/criterion/reports/) is generated for each release, allowing easy comparison of the performance of the different cryptographic primitives and schemes available in `fastcrypto`. As a
- correctness: строка 55: - Encoding: Base64 and Hex are defined with an encoding trait with its customized serialization and validations, backed by [base64ct](https://crates.io/crates/base64ct) and [hex]((https://crates.io/crates/base64ct)). Notably, the base64ct crate has been chosen

## programatik29/rust-web-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/programatik29/rust-web-benchmarks
- Категория: benchmark/testing candidate
- Описание: Benchmarking web frameworks written in rust with rewrk tool.
- Уровень: automated README evidence extraction
- Снимок: [sources/programatik29__rust-web-benchmarks/README.md](sources/programatik29__rust-web-benchmarks/README.md); SHA-256: `948d8bc6c6ca8f1a66a7f3afd9db9a75defc7967da0967370e601fd2e25782ca`

## BurntSushi/critcmp

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/BurntSushi/critcmp
- Категория: benchmark/testing candidate
- Описание: A command line tool for comparing benchmarks run by Criterion.
- Уровень: automated README evidence extraction
- Снимок: [sources/BurntSushi__critcmp/README.md](sources/BurntSushi__critcmp/README.md); SHA-256: `90a802ea9bc83f36db8e4dd9e9ba5291002cb31c41b4c75baec0123de5978c35`
- comparison: строка 4: comparing benchmarks both across and inside baselines, where a "baseline" is
- report: строка 69: A baseline can exported to one JSON file for more permanent storage outside

## rsasaki0109/rust_robotics

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rsasaki0109/rust_robotics
- Категория: benchmark/testing candidate
- Описание: 100+ robotics algorithms in Rust: planning, localization, SLAM, control, and benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/rsasaki0109__rust_robotics/README.md](sources/rsasaki0109__rust_robotics/README.md); SHA-256: `acc4fe9adf02c203162d63133ecfada33ba3d1a554e7e4849667a33050330914`
- statistics: строка 395: That switches the mission stack to `NAV_ODOM_TOPIC=/slam_odom` and `NAV_GLOBAL_FRAME=map`. In this mode, `slam_node` publishes `/slam_pose` plus `/slam_odom`, the map is integrated in `map`, and [map_odom_tf_broadcaster.py](./ros2_nodes/launch/map_odom_tf_broa
- comparison: строка 404: For a local ROS2/Gazebo regression check, run:
- lifecycle: строка 357: source /opt/ros/jazzy/setup.bash
- gpu: строка 57: - **`no_std` Kalman filters** — the localization stack cross-compiles for bare-metal
- report: строка 305: The workspace also includes a minimal `dora-rs` planning demo that wraps the existing headless A* planner in a dora node and sends a structured JSON path report to a sink node.
- correctness: строка 56: benchmarks with reproducible commands.

## datafusion-contrib/tpcgen-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/datafusion-contrib/tpcgen-rs
- Категория: benchmark/testing candidate
- Описание: TPC-H benchmark data generation in pure Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/datafusion-contrib__tpcgen-rs/README.md](sources/datafusion-contrib__tpcgen-rs/README.md); SHA-256: `e78712152ee1ea4c8c6280b9a3d6b9bb38da28ba7559521ec685b9f88365d425`
- memory: строка 25: 3. Fully parallel, streaming, constant memory usage 🧠
- correctness: строка 86: This crate has extensive tests to ensure correctness and produces exactly the

## rosetta-rs/template-benchmarks-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rosetta-rs/template-benchmarks-rs
- Категория: benchmark/testing candidate
- Описание: Collected benchmarks for templating crates written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/rosetta-rs__template-benchmarks-rs/README.md](sources/rosetta-rs__template-benchmarks-rs/README.md); SHA-256: `5b92395cd2b4e2769051a1695028c9f2ce7b5325acbea256029e52e30371e011`

## wfxr/rlt

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/wfxr/rlt
- Категория: benchmark/testing candidate
- Описание: A universal load testing framework for Rust, with real-time tui support.
- Уровень: automated README evidence extraction
- Снимок: [sources/wfxr__rlt/README.md](sources/wfxr__rlt/README.md); SHA-256: `ad756c1edd8959cc02bd0416b529859ab3275e044309e7b3e3ffda1e7c34e713`
- comparison: строка 23: - **Baseline Comparison**: Save and compare results to track performance changes.
- lifecycle: строка 78: async fn setup(&mut self, _worker_id: u32) -> BenchResult<Self::WorkerState> {
- async: строка 49: async fn bench(&mut self, _: &IterInfo) -> BenchResult<IterReport> {
- report: строка 41: use rlt::{cli::BenchCli, BenchResult, IterInfo, IterReport, StatelessBenchSuite, Status};

## bheisler/cargo-criterion

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bheisler/cargo-criterion
- Категория: benchmark/testing candidate
- Описание: Cargo extension for running Criterion.rs benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/bheisler__cargo-criterion/README.md](sources/bheisler__cargo-criterion/README.md); SHA-256: `62f13e717f980bfcc24726b3696332dbc48c4f821012d03e9171a06f2fcdf6b7`
- report: строка 16: reporting on [Criterion-rs](https://github.com/bheisler/criterion.rs) benchmarks.

## facebookexperimental/resctl-demo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/facebookexperimental/resctl-demo
- Категория: benchmark/testing candidate
- Описание: Demonstrate and benchmark various features of Linux resource control in a self-contained package.
- Уровень: automated README evidence extraction
- Снимок: [sources/facebookexperimental__resctl-demo/README.md](sources/facebookexperimental__resctl-demo/README.md); SHA-256: `28d32387fc935feb341d9baade6152b62abae5de3a0dd4ab343f8f260bd9a9d1`
- async: строка 35: realistic latency-sensitive workload simulator and other secondary
- report: строка 37: generates easily understandable reports.
- isolation: строка 242: user-space helpers such as oomd and sideloader implement resource isolation to

## beling/bsuccinct-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/beling/bsuccinct-rs
- Категория: benchmark/testing candidate
- Описание: Rust libraries and programs focused on succinct data structures
- Уровень: automated README evidence extraction
- Снимок: [sources/beling__bsuccinct-rs/README.md](sources/beling__bsuccinct-rs/README.md); SHA-256: `2e6f5d4a485da0f98c367da4af3686271aa9d86eb1c758af6c47b378a5282b16`
- memory: строка 17: - `dyn_size_of` ([crate](https://crates.io/crates/dyn_size_of), [doc](https://docs.rs/dyn_size_of)) - calculate memory consumed by variables, including the memory allocated on heap,

## huggingface/inference-benchmarker

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/huggingface/inference-benchmarker
- Категория: benchmark/testing candidate
- Описание: Inference server benchmarking tool
- Уровень: automated README evidence extraction
- Снимок: [sources/huggingface__inference-benchmarker/README.md](sources/huggingface__inference-benchmarker/README.md); SHA-256: `7949386274e1f44ac428e64c69971cc7429e891683e7164b41cc1ab7b7854786`
- statistics: строка 146: --prompt-options "num_tokens=200,max_tokens=220,min_tokens=180,variance=10" \
- lifecycle: строка 141: --warmup 30s \
- gpu: строка 264: CUDA graphs are used to optimize the GPU usage by minimizing the overhead of launching kernels. This can lead to
- async: строка 11: With **Inference Benchmarker**, you can easily test your model's throughput and efficiency under various workloads,
- report: строка 23: * JSON Output: Delivers performance results in a structured, easy-to-analyze format.

## OxidizeLabs/cachekit

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/OxidizeLabs/cachekit
- Категория: benchmark/testing candidate
- Описание: High-performance cache policies and supporting data structures for Rust systems, with optional metrics and benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/OxidizeLabs__cachekit/README.md](sources/OxidizeLabs__cachekit/README.md); SHA-256: `6d30f20a2acad6314d1a399d5a225fef4b521375f10f907ba44271b9a5e7f4a6`
- comparison: строка 216: | Random  | Baseline/uniform distribution | Random selection |
- memory: строка 111: | `policy-heap-lfu` | Heap LFU | LFU with heap-based eviction |
- async: строка 98: | `concurrency` | Concurrent wrappers (requires `parking_lot`) |
- correctness: строка 16: - Optional metrics and benchmarks to validate trade-offs.

## ashvardanian/StringWars

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ashvardanian/StringWars
- Категория: benchmark/testing candidate
- Описание: Comparing performance-oriented string-processing libraries for substring search, multi-pattern matching, hashing, edit-distances, sketching, and sorting across CPUs and GPUs in Rust 🦀 and Python 🐍
- Уровень: automated README evidence extraction
- Снимок: [sources/ashvardanian__StringWars/README.md](sources/ashvardanian__StringWars/README.md); SHA-256: `98f4a944c5be947bcdce0dbcd431bb97960d2f9c116d4225dabd304aa3ea90cc`
- comparison: строка 418: - __English (en)__: Mostly 1-byte ASCII baseline
- lifecycle: строка 309: Every one of them includes a few seconds of a warm-up phase to ensure that the CPU caches are filled and the results are not affected by cold start or SIMD-related frequency scaling.
- memory: строка 55: Bloom and cuckoo filters need many independent hashes of the same key; StringZilla's `hash_multiseed` prepares the key once and replays cheap per-seed rounds, while the alternatives re-prepare it every 64–128 bits.
- gpu: строка 142: Two things that table has to say out loud: every rival is single-threaded and CPU-only, so the honest head-to-head is against `stringzillas::Substrings<1cpu>` rather than its multi-core or GPU rows; and BM25 here is a scan against a fixed query, not an inverte
- async: строка 22: So, I focus on the workloads for which StringZilla was designed and compare the throughput of the core operations.
- report: строка 396: mkdir -p data/xlsum && curl -fL -o data/xlsum/xlsum.csv.gz https://github.com/ashvardanian/xl-sum/releases/download/v1.0.0/xlsum.csv.gz
- correctness: строка 34: Many hashing libraries exist, but they often lack reproducible outputs, streaming support, or cross-language availability.

## abundant-ai/swe-marathon

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/abundant-ai/swe-marathon
- Категория: benchmark/testing candidate
- Описание: SWE-Marathon: an ultra long-horizon SWE benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/abundant-ai__swe-marathon/README.md](sources/abundant-ai__swe-marathon/README.md); SHA-256: `82cdf6a877c61370f85b8f16d5cf2daad9a91177a66ee39c959235335a45104d`

## jcaromiq/goku

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/jcaromiq/goku
- Категория: benchmark/testing candidate
- Описание: Goku is an HTTP load testing application written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/jcaromiq__goku/README.md](sources/jcaromiq__goku/README.md); SHA-256: `5884dca06a70339f125bb163bcd873ace383098e0d6dbc22876174b8c7b6c3f4`
- comparison: строка 121: goku compare <BASELINE> <CANDIDATE>
- async: строка 17: * ASCII latency histogram in text output
- report: строка 21: * Multiple output formats: `text`, `json`, `csv`

## flashbots/contender

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/flashbots/contender
- Категория: benchmark/testing candidate
- Описание: spam EVM execution nodes over JSON-RPC & run benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/flashbots__contender/README.md](sources/flashbots__contender/README.md); SHA-256: `7198495006c16a17c9f3d75d720fe24f6d75024841c10394df9662acdfdf9b7e`
- lifecycle: строка 22: contender setup scenario:stress.toml -r $RPC_URL -p $PRIVATE_KEY
- report: строка 51: > `-v` maps `/tmp/.contender` on the host machine to `/root/.local/state/contender` in the container, which contains the DB; used for generating reports and saving contract deployments.
- correctness: строка 62: It supports both **per-second** (TPS) and **per-block** (TPB) timing, seeded fuzzing for reproducibility, and SQLite-backed state for contracts, runs, and reports.

## raphaelmansuy/edgeparse

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/raphaelmansuy/edgeparse
- Категория: benchmark/testing candidate
- Описание: EdgeParse converts any digital PDF into Markdown, JSON (with bounding boxes), HTML, or plain text — deterministically, without a JVM, without a GPU, and with best-in-class accuracy on the 200-document benchmark suite included in this repository.
- Уровень: automated README evidence extraction
- Снимок: [sources/raphaelmansuy__edgeparse/README.md](sources/raphaelmansuy__edgeparse/README.md); SHA-256: `2662949495360a35bdc9d039f9246114d4bae4a40b461c7f7d76145bd8abd00f`
- comparison: строка 130: | EdgeParse (pre-frontier baseline) | 0.859 | 0.493 | 0.500 | 0.482 | 0.891 | 0.849 | 0.232 s/doc | 0.751 |
- gpu: строка 3: **Fastest PDF extraction engine. Rust-native. Zero GPU, zero JVM, zero OCR models.**
- async: строка 134: EdgeParse now leads the entire comparison set on every reported benchmark metric, including speed. Relative to the previous EdgeParse baseline, the current pipeline increases reading-order accuracy, table structure similarity, paragraph boundaries, text qualit
- report: строка 11: Extract Markdown, JSON (with bounding boxes), and HTML from any born-digital PDF
- correctness: строка 222: | Deterministic / reproducible | ✅ | Same input → same output, always |

## delendum-xyz/zk-benchmarking

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/delendum-xyz/zk-benchmarking
- Категория: benchmark/testing candidate
- Описание: comparing the performance of different zero-knowledge proof libraries
- Уровень: automated README evidence extraction
- Снимок: [sources/delendum-xyz__zk-benchmarking/README.md](sources/delendum-xyz__zk-benchmarking/README.md); SHA-256: `efc14393bc20de3bbe8108d53c17e85629b303f30ee7d47250efec5298f87cb1`
- memory: строка 5: With zk-benchmarking, you can run a suite of standardized benchmarks against different zero-knowledge proof libraries and see how they perform in terms of speed, memory usage, and other metrics. This can help you make informed decisions about which library is
- gpu: строка 182: - On Apple-based systems, RISC Zero prover can take advantage of GPU resources.
- correctness: строка 36: ### Reproducible

## ashvardanian/less_slow.rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ashvardanian/less_slow.rs
- Категория: benchmark/testing candidate
- Описание: Playing around "Less Slow" coding practices in Rust, from numerical micro-kernels to coroutines, ranges, and polymorphic state machines
- Уровень: automated README evidence extraction
- Снимок: [sources/ashvardanian__less_slow.rs/README.md](sources/ashvardanian__less_slow.rs/README.md); SHA-256: `00b02234bf2734e9c3d0ef7854c301fb8b9fbd813ba0224945a82ecd1625a9a8`
- async: строка 14: - Experimental coroutines can be 3x faster than [`async-stream`](https://crates.io/crates/async-stream).

## wkwan/flo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/wkwan/flo
- Категория: workload: graphics
- Описание: Fast Vulkan 3D Renderer Integrated with the Bevy Game Engine
- Уровень: automated README evidence extraction
- Снимок: [sources/wkwan__flo/README.md](sources/wkwan__flo/README.md); SHA-256: `85a0271c0f647cf5f596454391d9c6dbd101e3356aebba6c5bbb2d710f68bb89`
- lifecycle: строка 20: ## Setup
- gpu: строка 1: # Flo: Vulkan/Ash 3D Renderer Integrated with the Bevy Game Engine
- correctness: строка 27: 3. Install glslc or glslangValidator to compile shaders (only needed if you're modifying the shaders)

## anoma/zkp-compiler-shootout

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/anoma/zkp-compiler-shootout
- Категория: benchmark/testing candidate
- Описание: Evaluating & benchmarking ZKP compilation strategies.
- Уровень: automated README evidence extraction
- Снимок: [sources/anoma__zkp-compiler-shootout/README.md](sources/anoma__zkp-compiler-shootout/README.md); SHA-256: `dec30f1e5e6777090ce0cecf8966488949072655f6445be8b67240605e912e5a`
- lifecycle: строка 258: fn prove(&self, setup: &mut Self::C) -> Self::R;
- report: строка 83: 3. The HTML results should be in `./shootout/target/criterion/reports/index.html`

## jonhoo/bustle

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/jonhoo/bustle
- Категория: benchmark/testing candidate
- Описание: A benchmarking harness for concurrent key-value collections
- Уровень: automated README evidence extraction
- Снимок: [sources/jonhoo__bustle/README.md](sources/jonhoo__bustle/README.md); SHA-256: `e36238121e19697511a87baa6502f1769a76f77b21f5aa9c3e22d05c65acc0f0`
- async: строка 4: Bustle is a benchmarking harness for concurrent key-value collections.
- report: строка 11: statistics as it goes, and gives you a report at the end about how you did. There are many

## rousan/rust-web-frameworks-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rousan/rust-web-frameworks-benchmark
- Категория: benchmark/testing candidate
- Описание: A hello world benchmark for the available Rust Web Frameworks: hyper vs gotham vs actix-web vs warp vs rocket
- Уровень: automated README evidence extraction
- Снимок: [sources/rousan__rust-web-frameworks-benchmark/README.md](sources/rousan__rust-web-frameworks-benchmark/README.md); SHA-256: `7b6c7804afab86c580768199208f2392ff34ec0d0337365b30353d6e657d9158`
- async: строка 12: $ wrk --latency -t4 -c200 -d8s http://127.0.0.1:8081

## Proryanator/encoder-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Proryanator/encoder-benchmark
- Категория: benchmark/testing candidate
- Описание: A tool to benchmark your hardware's real-time video encoding capabilities.
- Уровень: automated README evidence extraction
- Снимок: [sources/Proryanator__encoder-benchmark/README.md](sources/Proryanator__encoder-benchmark/README.md); SHA-256: `cb2f06a92cd7ed40d3f12688570fbd966d1223d26ac247b8e7d4cbf804bcc13a`
- lifecycle: строка 8: - [Installation and Setup](#installation-and-setup)
- gpu: строка 27: - easily compare GPU encoders of different generations, in terms of produced quality and maximum fps with ease

## fede1024/kafka-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fede1024/kafka-benchmark
- Категория: benchmark/testing candidate
- Описание: A tool to run benchmarks on Kafka clusters
- Уровень: automated README evidence extraction
- Снимок: [sources/fede1024__kafka-benchmark/README.md](sources/fede1024__kafka-benchmark/README.md); SHA-256: `bcf57c286fc00794b9cb397cfd7d39b4f24228f527c3d0827db4b565a55e034a`

## goobolabs/somali-language-standard

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/goobolabs/somali-language-standard
- Категория: benchmark/testing candidate
- Описание: The open, machine-readable standard for the Somali language — orthography, grammar, terminology, translation, and AI/benchmark resources, as a versioned, citable RFC-style catalog.
- Уровень: automated README evidence extraction
- Снимок: [sources/goobolabs__somali-language-standard/README.md](sources/goobolabs__somali-language-standard/README.md); SHA-256: `95e288821cb99494638456c23132bc5a700eb0a4d0670fd51102e737e62a12e4`
- comparison: строка 12: > evidence baseline is complete. The
- report: строка 28: (ECMA-262, Unicode Technical Reports) is to a technical domain: the canonical
- correctness: строка 18: > the Rust repository validator, and pull-request validation workflow are in

## khvzak/script-bench-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/khvzak/script-bench-rs
- Категория: benchmark/testing candidate
- Описание: Rust embedded scripting languages benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/khvzak__script-bench-rs/README.md](sources/khvzak__script-bench-rs/README.md); SHA-256: `cb35782c2d16164315356499fe483e54880589fa7b514feb78f6847e4eff5e4d`

## erickt/rust-serialization-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/erickt/rust-serialization-benchmarks
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/erickt__rust-serialization-benchmarks/README.md](sources/erickt__rust-serialization-benchmarks/README.md); SHA-256: `f694d1f51e82e821230165b0407c7629f84775f1218c3ee40f7a07edfd2cc0e9`
- gpu: строка 49: "server_name": "metal.cloudflare.com",
- report: строка 4: * [rapidjson](https://github.com/erickt/rapidjson)

## optuna/kurobako

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/optuna/kurobako
- Категория: benchmark/testing candidate
- Описание: A black-box optimization benchmark tool
- Уровень: automated README evidence extraction
- Снимок: [sources/optuna__kurobako/README.md](sources/optuna__kurobako/README.md); SHA-256: `2a63279b9a9270eb43b369afdf640a49ebddce92a5afc2d7be7477f84234c692`
- async: строка 22: - Simulating a concurrent environment in which an optimization process is executed by multiple workers simultaneously
- report: строка 20: - Generating a markdown report and PNG plots from benchmarking results
- correctness: строка 23: - Reproducible

## bluss/bencher

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bluss/bencher
- Категория: benchmark/testing candidate
- Описание: bencher is just a port of the libtest (unstable) benchmark runner to Rust stable releases. `cargo bench` on stable. "Not a better bencher!" = No feature development. Go build a better stable benchmarking library.
- Уровень: automated README evidence extraction
- Снимок: [sources/bluss__bencher/README.rst](sources/bluss__bencher/README.rst); SHA-256: `2e6347d34f4acffa261fb2165a62dbc676c33fbcc83f10a882134fc8f7509657`
- correctness: строка 43: original version. (Since reproducibility is key, we will use the same

## omarmhaimdat/pepe

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/omarmhaimdat/pepe
- Категория: benchmark/testing candidate
- Описание: HTTP Load Generator
- Уровень: automated README evidence extraction
- Снимок: [sources/omarmhaimdat__pepe/README.md](sources/omarmhaimdat__pepe/README.md); SHA-256: `8b48dd129455d5e280eff99c1837f05adf63a22aa2574ee2e1c19198f7bdb402`
- async: строка 3: Pepe is a command-line HTTP load generator designed to test the performance and reliability of web servers. It allows you to send a large number of HTTP requests to a specified URL and measure various performance metrics such as response times, throughput, and
- report: строка 73: pepe -n 1000 -c 20 -t 10 -u "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_2) AppleWebKit/601.3.9 (KHTML, like Gecko) Version/9.0.2 Safari/601.3.9" -H "Accept: application/json" -H "Content-Type: application/json" -m GET https://example.com

## sharkbench/sharkbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sharkbench/sharkbench
- Категория: benchmark/testing candidate
- Описание: Benchmarking programming languages and web frameworks.
- Уровень: automated README evidence extraction
- Снимок: [sources/sharkbench__sharkbench/README.md](sources/sharkbench__sharkbench/README.md); SHA-256: `dd123bbbefda926375e30e0afcbd5a768f2cf35e5729fa95bcfee5fb6c266716`
- lifecycle: строка 163: extended_warmup: true # set to true if the benchmark needs a longer warmup
- memory: строка 11: This benchmark tests how fast a programming language can perform mathematical computations without any I/O or memory allocation.
- async: строка 21: This benchmark tests how fast a framework can perform concurrent HTTP requests, I/O operations, and JSON de/serialization.
- report: строка 21: This benchmark tests how fast a framework can perform concurrent HTTP requests, I/O operations, and JSON de/serialization.

## clouedoc/hzfind

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/clouedoc/hzfind
- Категория: benchmark/testing candidate
- Описание: A TUI to find the best Hetzner dedicated server auction deals, with PassMark CPU benchmarks built-in
- Уровень: automated README evidence extraction
- Снимок: [sources/clouedoc__hzfind/README.md](sources/clouedoc__hzfind/README.md); SHA-256: `e3aafd21f007fb185202600c0016b966f98a0d83cb68b3f9826b7c4ecb1bf283`
- comparison: строка 41: 4. In the TUI, optionally compares each server against a **CCX33** cloud baseline (€62.99/mo) to show relative value
- report: строка 28: hzfind list                      # list all servers as JSON

## klingtnet/rb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/klingtnet/rb
- Категория: benchmark/testing candidate
- Описание: A thread-safe fixed-size circular buffer written in safe Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/klingtnet__rb/README.md](sources/klingtnet__rb/README.md); SHA-256: `b9b8f400af0762c10a19f71ea2fd1207d1ca4e440b7a0a7c74fdf73df74e4262`

## trinhminhtriet/spiko

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/trinhminhtriet/spiko
- Категория: benchmark/testing candidate
- Описание: 🚀 Spiko is a fast, Rust-based load testing tool with a beautiful TUI for real-time insights.
- Уровень: automated README evidence extraction
- Снимок: [sources/trinhminhtriet__spiko/README.md](sources/trinhminhtriet__spiko/README.md); SHA-256: `f89af73da5d60abb74a35d1afbda1fbc073ba24ffea3b88ddfe1ddc330cdb1ce`
- async: строка 23: - 📊 Visual feedback on request rates, latency, and more
- report: строка 94: -j, --json

## solstackapp/geyserbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/solstackapp/geyserbench
- Категория: benchmark/testing candidate
- Описание: Benchmark multiple Yellowstone gRPC endpoints!
- Уровень: automated README evidence extraction
- Снимок: [sources/solstackapp__geyserbench/README.md](sources/solstackapp__geyserbench/README.md); SHA-256: `2bb32ec27cb8e56139e24d3a5efb4e6382edb71a86dba6b0fae1b0d3fd6750ac`
- async: строка 8: - Track first-detection share, latency percentiles (P50/P95/P99), valid transaction counts, and backfill events
- report: строка 9: - Stream results to the SolStack backend for shareable reports, or keep runs local with a single flag

## gamelife1314/rsb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gamelife1314/rsb
- Категория: benchmark/testing candidate
- Описание: a http server benchmark tool written in rust 🦀
- Уровень: automated README evidence extraction
- Снимок: [sources/gamelife1314__rsb/README.md](sources/gamelife1314__rsb/README.md); SHA-256: `b3db6073b9aa7b7e9ed04d4857f10a04be7b27fb2ffc78b7632e1195a84b56f5`
- async: строка 63: | `Latency`              | Record the time taken for each request from sending to receiving the response, and then calculate the average, maximum and standard deviation |
- report: строка 72: `Content-Type` is set, but `--json-file`, `--json-body`, `--text-file`, `--text-body`, `--mp`, `--mp-file`, `--form`

## CodSpeedHQ/codspeed-rust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/CodSpeedHQ/codspeed-rust
- Категория: benchmark/testing candidate
- Описание: Crates to benchmark your Rust code
- Уровень: automated README evidence extraction
- Снимок: [sources/CodSpeedHQ__codspeed-rust/README.md](sources/CodSpeedHQ__codspeed-rust/README.md); SHA-256: `6f7e153ee8e097bbb51fe5ea98fbcfed214b9ba67834f6bdfb148b919a6c43ea`
- report: строка 7: [![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/CodSpeedHQ/codspeed-rust)

## lschmierer/ecs_bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lschmierer/ecs_bench
- Категория: benchmark/testing candidate
- Описание: Benchmarks of various Entity Component Systems in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/lschmierer__ecs_bench/README.md](sources/lschmierer__ecs_bench/README.md); SHA-256: `739ce5d35d0ae0f1d95442c4c574a7c9da987f94137fbaa6c010f1757c842ebb`

## marvin-j97/rust-storage-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/marvin-j97/rust-storage-bench
- Категория: benchmark/testing candidate
- Описание: Benchmarking Rust key-value storage engines
- Уровень: automated README evidence extraction
- Снимок: [sources/marvin-j97__rust-storage-bench/README.md](sources/marvin-j97__rust-storage-bench/README.md); SHA-256: `ef02f42527f28dd84617d2c53c7ed2645527d4ee30169ec881a45d2c7dbded33`
- report: строка 30: bencher --out task_e_fjall_lcs.jsonl --workload task-e --backend fjall --minutes 5 --key-size 8 --value-size 256 --items 1000 --cache-size 1000000

## TeXitoi/benchmarksgame-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/TeXitoi/benchmarksgame-rs
- Категория: benchmark/testing candidate
- Описание: The Computer Language Benchmarks Game: Rust implementations
- Уровень: automated README evidence extraction
- Снимок: [sources/TeXitoi__benchmarksgame-rs/README.md](sources/TeXitoi__benchmarksgame-rs/README.md); SHA-256: `e60d31ffc5abf5771bd1cfdb64fd46e1dd7d1de648839d9405b20caf13c5d2ef`

## tczajka/bigint-benchmark-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tczajka/bigint-benchmark-rs
- Категория: benchmark/testing candidate
- Описание: Bechmarks for Rust big integer implementations
- Уровень: automated README evidence extraction
- Снимок: [sources/tczajka__bigint-benchmark-rs/README.md](sources/tczajka__bigint-benchmark-rs/README.md); SHA-256: `ef788ee2cae47b9f9155034c54687a954df0c11a0bfb030283a635a25ce13f7c`

## briansmith/crypto-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/briansmith/crypto-bench
- Категория: benchmark/testing candidate
- Описание: Benchmarks for crypto libraries (in Rust, or with Rust bindings)
- Уровень: automated README evidence extraction
- Снимок: [sources/briansmith__crypto-bench/README.md](sources/briansmith__crypto-bench/README.md); SHA-256: `94be514197df4957812a40bf1d554c697273f837549a96e7813e1ba51f915847`
- report: строка 44: For example, it would be great to be able to get a graph or a table, or JSON

## ultralytics/template-rust

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ultralytics/template-rust
- Категория: benchmark/testing candidate
- Описание: Ultralytics Rust project template with library and CLI examples, tests, benchmarks, rustfmt, clippy, coverage, dependency checks, and crates.io publishing.
- Уровень: automated README evidence extraction
- Снимок: [sources/ultralytics__template-rust/README.md](sources/ultralytics__template-rust/README.md); SHA-256: `275e0b8f528b57f7840a7ba9ce189d70f6f9e3f6aaa79a305c8d612c468d3f19`
- lifecycle: строка 8: ensure consistency, maintain high quality standards, and accelerate the setup process for new Rust-based software.
- report: строка 135: Ultralytics thrives on community collaboration, and we deeply value your contributions! Whether it's reporting bugs,
- correctness: строка 68: Integration tests in `tests/` validate user-facing behavior. Add unit tests alongside the code they cover in `src/` and

## bluss/twoway

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bluss/twoway
- Категория: benchmark/testing candidate
- Описание: Twoway / Fast substring search for strings and byte strings (Rust) / Also assorted benchmarks and string search snippets
- Уровень: automated README evidence extraction
- Снимок: [sources/bluss__twoway/README.rst](sources/bluss__twoway/README.rst); SHA-256: `4dd476c59fabedb988c3a5a6e84fed519c8b8d70c66df43e099c66b923eee8b8`

## SwiftyPop/TimerResBenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SwiftyPop/TimerResBenchmark
- Категория: benchmark/testing candidate
- Описание: TimerResBenchmark is a tool rewritten in Rust for fine-tuning system timer resolution to achieve precise sleep intervals close to 1 millisecond.
- Уровень: automated README evidence extraction
- Снимок: [sources/SwiftyPop__TimerResBenchmark/README.md](sources/SwiftyPop__TimerResBenchmark/README.md); SHA-256: `4e0ae3802d15f6cb1774e0faccf8afb9a5258c2fb2587665376943d2b45b1bcf`
- lifecycle: строка 13: - Checks if HPET is enabled or disabled during benchmark setup.
- async: строка 3: <p align="center"><b>A Rust-based tool for benchmarking system timer resolution to achieve precise sleep intervals, optimizing performance and consistency for high-performance tasks like gaming, especially in low-latency scenarios. It automatically detects HPE
- report: строка 34: - You can adjust the benchmark parameters directly in the program or modify them manually in the 'appsettings.json' file(default value).

## a16z/zkvm-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/a16z/zkvm-benchmarks
- Категория: benchmark/testing candidate
- Описание: Benchmarks of popular zkVMs including Jolt
- Уровень: automated README evidence extraction
- Снимок: [sources/a16z__zkvm-benchmarks/README.md](sources/a16z__zkvm-benchmarks/README.md); SHA-256: `0d5e7b00b6b55b0808ecb8fbb2a430ac0fd000116b37f8d483885085609d85e0`
- report: строка 32: The benchmark results should be outputted in CSV form in `benchmark_outputs`.
- correctness: строка 39: *This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to the safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as intended, and users may e

## EmbarkStudios/tiny-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/EmbarkStudios/tiny-bench
- Категория: benchmark/testing candidate
- Описание: A tiny benchmarking library
- Уровень: automated README evidence extraction
- Снимок: [sources/EmbarkStudios__tiny-bench/README.md](sources/EmbarkStudios__tiny-bench/README.md); SHA-256: `5765bed637a3489cb3f8697340200902b74dabdb17848fe198fca890966722e0`
- statistics: строка 25: statistical analysis of results, trimming that down, and leaving much of the customizability out.

## unixpickle/Benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/unixpickle/Benchmarks
- Категория: benchmark/testing candidate
- Описание: Some language performance comparisons.
- Уровень: automated README evidence extraction
- Снимок: [sources/unixpickle__Benchmarks/README.md](sources/unixpickle__Benchmarks/README.md); SHA-256: `aaf1716000f0af4327b9bff65679141f7b729c441a5c21c6358c8ee5a3249ba1`
- memory: строка 115: Allocating memory through object construction should be very fast. In a langauge like C++, it is sometimes possible to avoid dynamic memory allocation altogether in favor of stack allocation. On the other hand, a langauge like Java forces you to create objects

## rust-lang/libtest

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rust-lang/libtest
- Категория: benchmark/testing candidate
- Описание: Rust's built-in testing and benchmarking framework
- Уровень: automated README evidence extraction
- Снимок: [sources/rust-lang__libtest/README.md](sources/rust-lang__libtest/README.md); SHA-256: `cad1712f10b88b55a47bd08ca57b24ffaec80a757a31e402c606c0782302c7b9`

## rylev/welle

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rylev/welle
- Категория: benchmark/testing candidate
- Описание: Apache-Benchmark-Like Tool Written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/rylev__welle/README.md](sources/rylev__welle/README.md); SHA-256: `5580c15e7a9c6ddcf580c407755cab95e74b896ee6c8c5d3bd8c25cf300f4cf9`
- async: строка 16: -c, --concurrent-requests <NUMBER>    Number of in flight requests allowed at a time [default: 1]

## samuell/gccontent-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/samuell/gccontent-benchmark
- Категория: benchmark/testing candidate
- Описание: Benchmarking different languages for a simple bioinformatics task (Counting the GC fraction of DNA in a FASTA file)
- Уровень: automated README evidence extraction
- Снимок: [sources/samuell__gccontent-benchmark/README.md](sources/samuell__gccontent-benchmark/README.md); SHA-256: `76fb024cfe7e174a804f552a33799840a08bbd99a7793916ecfbf5725d648d60`
- report: строка 9: cat report.md

## sassman/ssd-benchmark-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sassman/ssd-benchmark-rs
- Категория: benchmark/testing candidate
- Описание: Super Simple Disk Benchmark - benchmarks the writing performance of your disk
- Уровень: automated README evidence extraction
- Снимок: [sources/sassman__ssd-benchmark-rs/README.md](sources/sassman__ssd-benchmark-rs/README.md); SHA-256: `e5b9fe4a836bab99b8cc07a140e89365c5b00060ae964b9954972b670761fbb3`
- async: строка 18: It used random data and writes first sequentially chunks of 8MB until a total 1GB is written. It measures writing time and throughput.

## surrealdb/crud-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/surrealdb/crud-bench
- Категория: benchmark/testing candidate
- Описание: A benchmarking tool for testing and comparing the performance of both embedded and networked SQL and NoSQL databases.
- Уровень: automated README evidence extraction
- Снимок: [sources/surrealdb__crud-bench/README.md](sources/surrealdb__crud-bench/README.md); SHA-256: `e7fa0b5b2dde266db45d3d4b51ac32b4a28a85822c797c98470bce41fb176033`
- lifecycle: строка 605: #### TiKV Cluster Setup (Primary Use Case)
- async: строка 31: - Create: inserting N unique records, with the specified concurrency.
- report: строка 126: -n, --name <NAME>                            An optional name for the test, used as a suffix for the JSON result file name

## fereidani/rust-channel-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fereidani/rust-channel-benchmarks
- Категория: benchmark/testing candidate
- Описание: rust channel benchmarks to keep stat of performance of Kanal library in comparison with other competitors.
- Уровень: automated README evidence extraction
- Снимок: [sources/fereidani__rust-channel-benchmarks/README.md](sources/fereidani__rust-channel-benchmarks/README.md); SHA-256: `f82e149d59c1a43abbe9ab1b3ff6cbef74c9e6c7cdcfa900adb4844207760fbd`
- memory: строка 48: It's because of Tokio's context-switching performance, like Golang, Tokio context-switch in the same thread to the next coroutine when the channel message is ready which is much cheaper than communicating between different threads, It's the same reason why asy
- async: строка 47: #### Why in some tests async is much faster than sync?
- report: строка 6: Runs benchmarks, stores results into `*.csv` files in the target folder, and generates multiple png file for each test category:

## PSeitz/binggan

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/PSeitz/binggan
- Категория: benchmark/testing candidate
- Описание: Benchmarking library for stable Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/PSeitz__binggan/README.md](sources/PSeitz__binggan/README.md); SHA-256: `aba192204dced05531bf8a0ee50a234eb3e89b624a3e6ed832c5a2cbbf4c84b7`
- comparison: строка 19: * 🔀 Interleaving Test Runs (More accurate results)
- memory: строка 13: * 📊 Peak Memory Usage
- async: строка 78: // Enables throughput reporting for this input
- report: строка 23: * 📈 Custom Reporter
- correctness: строка 141: If you want reproducible iteration counts without changing code, you can override them with environment variables:

## BurntSushi/rust-sorts

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/BurntSushi/rust-sorts
- Категория: benchmark/testing candidate
- Описание: Implementations of common sorting algorithms in Rust with comprehensive tests and benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/BurntSushi__rust-sorts/README.md](sources/BurntSushi__rust-sorts/README.md); SHA-256: `ce50c942ae196292e9b3a84bcb2b555aba433c9948c001729472f0678c4663c7`
- memory: строка 6: quicksort, heapsort, insertion sort, selection sort, bubble sort and even bogo

## egraphs-good/extraction-gym

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/egraphs-good/extraction-gym
- Категория: benchmark/testing candidate
- Описание: benchmarking e-graph extraction
- Уровень: automated README evidence extraction
- Снимок: [sources/egraphs-good__extraction-gym/README.md](sources/egraphs-good__extraction-gym/README.md); SHA-256: `102ac00aee81d11cad91265b2fbafa3ac68fcc059a674573bf835318d640c7e6`
- report: строка 16: Please add data! It's just a JSON! See the `data/` directory for examples.

## xemantic/java-2-times-faster-than-c

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/xemantic/java-2-times-faster-than-c
- Категория: benchmark/testing candidate
- Описание: An inquiry into nondogmatic software development. An experiment showing double performance of the code running on JVM comparing to equivalent native C code.
- Уровень: automated README evidence extraction
- Снимок: [sources/xemantic__java-2-times-faster-than-c/README.md](sources/xemantic__java-2-times-faster-than-c/README.md); SHA-256: `5f5ccae3a89abd258f485af04e790aa6d6f420e5585bd369a850d2e5330b4541`
- statistics: строка 145: slower than the equivalent optimized native code, with big outliers in favor of the
- memory: строка 109: in algorithmic sense, because memory allocation, and releasing it implicitly or explicitly,
- gpu: строка 221: * For the code relying mostly on GPU performance gains on CPU might be neglectable.
- async: строка 160: performance is usually impacted by IO throughput. Overall performance improvement will come not
- report: строка 166: streamed, and converted to JSON on the fly, each request typically involves myriads of new data
- correctness: строка 141: "always", becomes "usually", and "usually" implies that from now on we should rather revalidate for

## vllm-project/vllm-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vllm-project/vllm-bench
- Категория: benchmark/testing candidate
- Описание: High-performance Rust benchmark client for vLLM serving endpoints.
- Уровень: automated README evidence extraction
- Снимок: [sources/vllm-project__vllm-bench/README.md](sources/vllm-project__vllm-bench/README.md); SHA-256: `419b634d14e319d93ba03e2ea509a278c15218ddcc9695b6713759572299f9be`
- statistics: строка 473: When `--max-concurrency` is set and `--request-rate` is `inf` (closed-loop mode), the benchmark automatically reports an additional **Steady-State Metrics** block. It measures throughput and latency only over the window during which in-flight concurrency stays
- comparison: строка 324: vllm-bench --compare baseline.json optimized.json
- lifecycle: строка 610: | `--num-warmups` | `0` | Warmup requests before benchmarking |
- memory: строка 791: - **mimalloc** — global allocator to reduce contention under high concurrency (1400+ tasks); page-agnostic, runs on aarch64 4K- and 64K-page kernels
- async: строка 27: --num-prompts 1000 --max-concurrency 200
- report: строка 38: - **Parity** — JSON output schema and timing semantics match Python `vllm bench serve` exactly.
- correctness: строка 522: | `--seed` | `0` | Random seed for reproducibility |
- isolation: строка 685: **Router affinity:** every turn sends `X-Session-ID: {conversation_id}` for KV-cache reuse behind a vLLM router.

## tempoxyz/schelk

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tempoxyz/schelk
- Категория: benchmark/testing candidate
- Описание: Fast filesystem snapshot and rollback tool for benchmarking
- Уровень: automated README evidence extraction
- Снимок: [sources/tempoxyz__schelk/README.md](sources/tempoxyz__schelk/README.md); SHA-256: `5bd6f43727e88a2a6deb14f420a2afd3a38c880afef6f00c422b8b0cb701ad50`
- statistics: строка 21: machinery. Overhead matters, but variance matters more. If a benchmark varies by 10%
- comparison: строка 3: schelk restores a block device to a known baseline quickly. It is designed for benchmarking
- lifecycle: строка 89: - `dmsetup` (shipped with most distributions).
- memory: строка 218: under test avoids contention with the benchmark. Second, the metadata is cheap to recreate,
- report: строка 169: - `schelk status` - report the current state (initialized, mounted, and so on).
- correctness: строка 12: > install schelk, validate prerequisites, initialize volumes, and run the workflow safely.

## apitap/apitap-lib

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/apitap/apitap-lib
- Категория: benchmark/testing candidate
- Описание: Move whole tables between databases fast — Postgres, MySQL, ClickHouse, BigQuery. Rust engine, one-line Python API, bounded memory.
- Уровень: automated README evidence extraction
- Снимок: [sources/apitap__apitap-lib/README.md](sources/apitap__apitap-lib/README.md); SHA-256: `54ba5796e4e6b48c8b4b96d801b746093f82a7e0bfcc130409122d1e4f197dbd`
- statistics: строка 294: snapshot as the data** — bootstrapped from parquet footer stats, so it
- memory: строка 57: container (~26M rows/minute, peak RSS 170.8 MB), the same table still completes
- async: строка 104: primary key (auto-detected) and each range streams concurrently.
- report: строка 23: report = apitap.transfer(
- correctness: строка 59: takes **30.3 seconds** (~3.3 GB/s, checksum-matched). The tools we compare
- isolation: строка 266: counts, shared pools/auth, per-table failure isolation; peak memory stays at

## haraldh/rust_echo_bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/haraldh/rust_echo_bench
- Категория: benchmark/testing candidate
- Описание: rust echo server benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/haraldh__rust_echo_bench/README.md](sources/haraldh__rust_echo_bench/README.md); SHA-256: `18eb69ec3b31d0b14a9b82df47a97f4c9920a5bd33c983942e3dc1fb79c1aeca`

## kaist-cp/smr-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kaist-cp/smr-benchmark
- Категория: benchmark/testing candidate
- Описание: SMR Benchmark: A Microbenchmark Suite for Concurrent Safe Memory Reclamation Schemes
- Уровень: automated README evidence extraction
- Снимок: [sources/kaist-cp__smr-benchmark/README.md](sources/kaist-cp__smr-benchmark/README.md); SHA-256: `0d178ce218cf534223496e184e5ec110092548194eccdfeee9d931edbd69b6dd`
- comparison: строка 176: * `nr`: A baseline that does not reclaim memory
- memory: строка 197: It runs a single map data structure benchmark with the given configuration, and measures the throughput (operations per second) and memory usage (bytes).
- async: строка 1: # SMR Benchmark: A Microbenchmark Suite for Concurrent Safe Memory Reclamation Schemes
- report: строка 249: To run the entire benchmark, execute `experiment.sh` script in `bench-scripts`. This takes several hours and creates raw CSV data and figures under `./results/`.
- correctness: строка 8: * Janggun Lee, Jeonghyeon Kim, Jeehoon Kang, Leveraging Immutability to Validate Hazard Pointers for Optimistic Traversals, PLDI 2025 \[18\].

## polybase/zk-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/polybase/zk-benchmarks
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/polybase__zk-benchmarks/README.md](sources/polybase__zk-benchmarks/README.md); SHA-256: `605a9914f70d0656c24fe0d2afd5cf7d82d56ee6ad9de7c7ac5ac27f1907e4d9`

## apache/sedona-spatialbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/apache/sedona-spatialbench
- Категория: benchmark/testing candidate
- Описание: A benchmark for assessing geospatial SQL analytics query performance across database systems
- Уровень: automated README evidence extraction
- Снимок: [sources/apache__sedona-spatialbench/README.md](sources/apache__sedona-spatialbench/README.md); SHA-256: `c6fa69edc7c2f86ebcec2296a0c8dc8e9c4d81ad753bc629e7988fdc79ebbf56`
- statistics: строка 244: - [Neyman, J. & Scott, E. L. (1958). Statistical Approach to Problems of Cosmology. Journal of the Royal Statistical Society: Series B (Methodological), 20(1)](https://doi.org/10.1111/j.2517-6161.1958.tb00272.x)
- comparison: строка 242: - [tpchgen-rs for inspiration and baseline performance](https://datafusion.apache.org/blog/2025/04/10/fastest-tpch-generator/)
- async: строка 137: - **Multithreaded from the ground up**: Leverages all CPU cores for high-throughput generation.
- report: строка 222: The S3 writer uses streaming multipart upload, buffering data in 32MB chunks before uploading parts. This ensures memory-efficient generation even for large datasets. All output formats (Parquet, CSV, TBL) are supported, and the generated files are byte-for-by
- correctness: строка 3: SpatialBench is a benchmark for assessing geospatial SQL analytics query performance across database systems. It provides a reproducible and scalable way to evaluate the performance of spatial data engines using realistic synthetic workloads.

## eth-act/zkevm-benchmark-workload

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/eth-act/zkevm-benchmark-workload
- Категория: benchmark/testing candidate
- Описание: zkVM benchmarking for Ethereum
- Уровень: automated README evidence extraction
- Снимок: [sources/eth-act__zkevm-benchmark-workload/README.md](sources/eth-act__zkevm-benchmark-workload/README.md); SHA-256: `c3d98b89d1fc2a41f94616a919bd43afef11141080d915fdaa8f98c9e5db9d51`
- report: строка 41: Obtain an EEST fixture bundle from [ethereum/execution-specs](https://github.com/ethereum/execution-specs) whose `blockchain_tests` cases contain canonical stateless bytes. Then benchmark either the checkout's fixture root, a directory of EEST JSON files, or o
- correctness: строка 7: This repository benchmarks Ethereum stateless-validator guests across multiple zkVMs. The normal workflow has two phases:

## deepu105/concurrency-benchmarks

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/deepu105/concurrency-benchmarks
- Категория: benchmark/testing candidate
- Описание: concurrency-benchmarks for languages
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## Canop/glassbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Canop/glassbench
- Категория: benchmark/testing candidate
- Описание: A micro-benchmark framework to use with cargo bench
- Уровень: automated README evidence extraction
- Снимок: [sources/Canop__glassbench/README.md](sources/Canop__glassbench/README.md); SHA-256: `8651bb0421967f1b58775b411c2d02b9d1c739519746882459512384276bcd30`
- report: строка 266: [Criterion](https://docs.rs/crate/criterion/0.3.4) is very similar. It produces detailed reports, and has more options than Glassbench, but doesn't have an history past the previous `cargo bench` (which is usually the one you most want). Glassbench tries to of

## privacy-ethereum/nova-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/privacy-ethereum/nova-bench
- Категория: benchmark/testing candidate
- Описание: A collection of comparison-benchmarks for Nova & related Proving systems
- Уровень: automated README evidence extraction
- Снимок: [sources/privacy-ethereum__nova-bench/README.md](sources/privacy-ethereum__nova-bench/README.md); SHA-256: `f559e96a7f08caad7c50b88110c8cfb045afbb4a312b5abc4dc22c3d6c2999c4`
- memory: строка 66: ### Memory usage and SRS
- gpu: строка 110: 6) GPU comparison. GPU should show significant improvement vs CPU, but so far we've not been able to get it to work / show big improvement.
- correctness: строка 7: *NOTE: Disclaimer - these benchmarks are preliminary and should be taken with a grain of salt. Some shortcuts were taken as these were done and we want to check the correctness of these calculations before being 100% confident in them.*

## askama-rs/template-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/askama-rs/template-benchmark
- Категория: benchmark/testing candidate
- Описание: Comparison of template engines written in and for Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/askama-rs__template-benchmark/README.md](sources/askama-rs__template-benchmark/README.md); SHA-256: `f77141ac5372f76ddd900915510e7819587a1ec5b53ce71fb0e851184d53b927`
- report: строка 54: and the file `target/criterion/report/index.html` will contain more information.

## xnuter/perf-gauge

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/xnuter/perf-gauge
- Категория: benchmark/testing candidate
- Описание: Benchmarking tool for network services.
- Уровень: automated README evidence extraction
- Снимок: [sources/xnuter__perf-gauge/README.md](sources/xnuter__perf-gauge/README.md); SHA-256: `918cbb658769206f3da3f614a8c20b7e8a20768ace821a229a151fc82a8f01e8`
- comparison: строка 21: For instance: ![](./examples/prom/baseline-nginx-stable-p50-99.png).
- async: строка 15: 1. Unlimited request rate (to find the max throughput).
- report: строка 19: 1. It can report metrics to `Prometheus` via a `pushgateway`.

## nu11ptr/criterion-table

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nu11ptr/criterion-table
- Категория: benchmark/testing candidate
- Описание: Generate markdown comparison tables from `cargo-criterion` JSON output
- Уровень: automated README evidence extraction
- Снимок: [sources/nu11ptr__criterion-table/README.md](sources/nu11ptr__criterion-table/README.md); SHA-256: `427ef5f5f125bc0672faaec6953fe681f1ba5cb5d3529b90cac413e9ad8cc4b9`
- comparison: строка 47: * The first column seen in each row will be the baseline everything else
- report: строка 7: [Cargo Criterion](https://github.com/bheisler/cargo-criterion) benchmark JSON

## dmitryikh/rust-vs-cpp-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dmitryikh/rust-vs-cpp-bench
- Категория: benchmark/testing candidate
- Описание: Performance benchmark of Rust vs. C++ on simple algorithmic problems
- Уровень: automated README evidence extraction
- Снимок: [sources/dmitryikh__rust-vs-cpp-bench/README.md](sources/dmitryikh__rust-vs-cpp-bench/README.md); SHA-256: `2dd072ebfab21f232d5967f7b2aceef4afc8ed6640fffeb0eab1826a2c14f044`
- memory: строка 19: work with priority queque (heap data structure), binary tree and hash map.

## ruvnet/rupixel

- Итог: excluded — Visual RAG product; outside the selected workload scope
- Источник: https://github.com/ruvnet/rupixel
- Категория: benchmark/testing candidate
- Описание: Pixel-native visual RAG ported to Rust on the ruvector ANN substrate (HNSW + IVF-Flat) — screenshot/document retrieval over visual embeddings, a Rust port of PixelRAG, with a metaharness benchmark CLI: npx rupixel
- Уровень: automated README evidence extraction
- Снимок: [sources/ruvnet__rupixel/README.md](sources/ruvnet__rupixel/README.md); SHA-256: `035e64845ab0b8ca4c71ee28d02e42fa8602c13ff1a8958ec924a44bca879a00`
- lifecycle: строка 125: npx rupixel doctor     # check your setup
- gpu: строка 30: - **Runs on your GPU when available** (WebGPU via transformers.js v3), falling back
- report: строка 79: match), **not** a time. Speed is reported separately, in **milliseconds**.
- correctness: строка 143: - ✅ **Two live in-browser demos** + a reproducible benchmark.

## SpaceCell/lightstream

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SpaceCell/lightstream
- Категория: benchmark/testing candidate
- Описание: Fast data transport : Apache Arrow, Protobuf and Message Pack data on the one stream. Measured faster than Arrow Flight on all open benchmarks. HTTP, TCP, QUIC, Websocket, Webtransport, Stdio, UDS, all interchangeable, with 64-byte SIMD compatibilty maintained over the network and to/from disk via custom Arrow Mmap and Parquet Readers and Writers
- Уровень: automated README evidence extraction
- Снимок: [sources/SpaceCell__lightstream/README.md](sources/SpaceCell__lightstream/README.md); SHA-256: `5a9ee7c682b4bbf4d5bfd59668dd7a666812194bef0b89ef63ea0d7a827ad505`
- statistics: строка 215: - **Median of five** - every cell is the warm median of five runs, smoothing transient cloud variance.
- lifecycle: строка 34: See the [Python README](python/README.md) and [Rust README](rust/README.md) for setup details.
- async: строка 159: **Movement Friction**: Right now, moving Arrow data *(the common tabular interface standard)* between high-throughput services is not as easy as it could be.
- report: строка 195: Lightstream performed faster than the industry-standard alternative in every measured comparison on open AWS EC2 benchmarks (see `benchmarks/`). This is despite returning a single globally ordered stream after parallelising connections for delivery, which the

## nuald/simple-web-benchmark

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/nuald/simple-web-benchmark
- Категория: benchmark/testing candidate
- Описание: A simple web benchmark of C++, Crystal, Go, Java, Node.js, PHP, Python, Rust and Scala.
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## codecrafters-io/ccbench

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/codecrafters-io/ccbench
- Категория: benchmark/testing candidate
- Описание: A benchmark for coding agents
- Уровень: automated README evidence extraction
- Снимок: [sources/codecrafters-io__ccbench/README.md](sources/codecrafters-io__ccbench/README.md); SHA-256: `f4dbc32201632093d0f70c80b475e5f73bc6920297ed52ebdd0d612f5931ca42`
- correctness: строка 65: ## Transparency & Reproducibility

## DoHoonKim8/halo2-lasso

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DoHoonKim8/halo2-lasso
- Категория: benchmark/testing candidate
- Описание: Benchmark for adding Lasso lookup argument to halo2 backend
- Уровень: automated README evidence extraction
- Снимок: [sources/DoHoonKim8__halo2-lasso/README.md](sources/DoHoonKim8__halo2-lasso/README.md); SHA-256: `1654df1fa918136f6f2a35cd459cf1eb7a27bfacba938fdd9914bb21a2a4381c`

## danoctavian/c10k-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/danoctavian/c10k-bench
- Категория: benchmark/testing candidate
- Описание: echo server benchmark for runtimes that use event loops
- Уровень: automated README evidence extraction
- Снимок: [sources/danoctavian__c10k-bench/README.md](sources/danoctavian__c10k-bench/README.md); SHA-256: `7a5836b1b6560c469d4bb49f27cce06b88b3a677ea40c4cfffa9e63f58b6ce6e`
- memory: строка 100: it crashed with memory allocation errors.
- async: строка 9: CONN_COUNT = 3000 concurrent connections

## openai/evals

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/openai/evals
- Категория: benchmark/testing candidate
- Описание: Evals is a framework for evaluating LLMs and LLM systems, and an open-source registry of benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/openai__evals/README.md](sources/openai__evals/README.md); SHA-256: `f0b50bd53e2fbc81be774a6bfb0cf20997aa7509bd22925ae779a87d57c7c914`
- lifecycle: строка 11: ## Setup
- report: строка 3: > You can now configure and run Evals directly in the OpenAI Dashboard. [Get started →](https://platform.openai.com/docs/guides/evals)

## TechEmpower/FrameworkBenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/TechEmpower/FrameworkBenchmarks
- Категория: benchmark/testing candidate
- Описание: Source for the TechEmpower Framework Benchmarks project
- Уровень: automated README evidence extraction
- Снимок: [sources/TechEmpower__FrameworkBenchmarks/README.md](sources/TechEmpower__FrameworkBenchmarks/README.md); SHA-256: `48c48cac1e96bd205f7eedaa7ef101ec3149ba2d0e355adf8c2ae8b5392964a6`
- report: строка 7: This project provides representative performance measures across a wide field of web application frameworks. With much help from the community, coverage is quite broad and we are happy to broaden it further with contributions. The project presently includes fr

## krausest/js-framework-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/krausest/js-framework-benchmark
- Категория: benchmark/testing candidate
- Описание: A comparison of the performance of a few popular javascript frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/krausest__js-framework-benchmark/README.md](sources/krausest__js-framework-benchmark/README.md); SHA-256: `b5919af7200df2fe3e8e6f9936f0d0dcf5d1be9e381e748418a4c58f177c21d1`
- statistics: строка 341: 2. Make sure your HTML elements have the same classes and structure as VanillaJS, otherwise benchmarks won't be able to find your elements on the page, and you will not get the global CSS (Bootstrap)
- comparison: строка 570: - Do not start your implementation using vanillajs as the reference. It uses direct DOM manipulation (and thus has note [#772](https://github.com/krausest/js-framework-benchmark/issues/772)) and serves only as a performance baseline but not as a best practice
- lifecycle: строка 19: - create rows: Duration for creating 1,000 rows after the page loaded (no warmup).
- memory: строка 28: - ready memory: Memory usage after page load.
- report: строка 202: Some frameworks like binding.scala or ember can't be opened that way, because they need a 'dist' or 'target/web/stage' or something in the URL. You can find out the correct URL in the [index.html](http://localhost:8080/index.html) you've opened before or take
- correctness: строка 553: - **Please make sure your implementation is validated by the test tool.** cd to the root directory and perform a check  `npm run rebuild-ci [keyed|non-keyed]/[FrameworkName]`. It'll print an error if your framework doesn't build, the benchmark can't be run or

## the-benchmarker/web-frameworks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/the-benchmarker/web-frameworks
- Категория: benchmark/testing candidate
- Описание: Which is the fastest web framework?
- Уровень: automated README evidence extraction
- Снимок: [sources/the-benchmarker__web-frameworks/README.md](sources/the-benchmarker__web-frameworks/README.md); SHA-256: `8fc5fbec995a78180345aa40d159293cebcf8b374a60e648601f346e129f8d6d`
- lifecycle: строка 88: make -f javascript/fastify/.Makefile warmup
- async: строка 19: > These are minimal HTTP throughput and latency tests, not a substitute for profiling a production application. Framework features, maintainability, ecosystem, security, database access, and your real workload all matter.
- report: строка 7: [Explore the results](https://web-frameworks-benchmark.netlify.app/) · [Add a framework](CONTRIBUTING.md) · [Report a problem](https://github.com/the-benchmarker/web-frameworks/issues) · [Join the discussion](https://github.com/the-benchmarker/web-frameworks/d
- correctness: строка 5: **A community-maintained, reproducible comparison of backend web frameworks**

## AgentOps-AI/agentops

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/AgentOps-AI/agentops
- Категория: benchmark/testing candidate
- Описание: Python SDK for AI agent monitoring, LLM cost tracking, benchmarking, and more. Integrates with most LLMs and agent frameworks including CrewAI, Agno, OpenAI Agents SDK, Langchain, Autogen, AG2, and CamelAI
- Уровень: automated README evidence extraction
- Снимок: [sources/AgentOps-AI__agentops/README.md](sources/AgentOps-AI__agentops/README.md); SHA-256: `6cf775117207bfda15954e839da8a8cd64fa7f95325db682c2512ffad1d90c41`
- comparison: строка 815: | 🔜 Regression testing                     | ✅ Multi-agent framework visualization                                              |                                             |                                                   |
- lifecycle: строка 114: Looking to run the full AgentOps app (Dashboard + API backend) on your machine? Follow the setup guide in `app/README.md`:
- async: строка 221: - Async/await functions
- report: строка 37: <img src="https://img.shields.io/badge/Dashboard-blue.svg?style=flat-square" alt="Dashboard" style="height: 20px;">
- correctness: строка 812: | 🚧 Success validators (external)          | 🔜 Execution containers                                                             | 🔜 Context limit overflow flags             | 🔜 Generative code validators                     |

## modelscope/evalscope

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/modelscope/evalscope
- Категория: benchmark/testing candidate
- Описание: A streamlined and customizable framework for efficient large model (LLM, VLM, AIGC) evaluation and performance benchmarking.
- Уровень: automated README evidence extraction
- Снимок: [sources/modelscope__evalscope/README.md](sources/modelscope__evalscope/README.md); SHA-256: `8e92617f71b8f21798e85eb6d76928e0f7473638dffb386abf4dece5cc5101e4`
- gpu: строка 95: - 🔥 **[2026.05.08]** Partnered with [LightSeek](https://lightseek.org/) to launch [TokenSpeed](https://lightseek.org/blog/lightseek-tokenspeed.html), a speed-of-light LLM inference engine for agentic workloads. EvalScope provides the SWE-smith benchmarking pip
- async: строка 86: - 🔥 **[2026.05.27]** Added **Trie agentic trace replay** for perf benchmarking: three new dataset plugins (`trie_agentic_coding` / `trie_code_qa` / `trie_office_work`) replay real multi-turn agent traces with per-turn token caps and tool-call latency simulatio
- report: строка 43: - **📊 Interactive Reports**: Provides a Web Dashboard for multi-dimensional model comparison, report overview and detailed inspection.
- correctness: строка 78: - 🔥 **[2026.08.24] v1.11.0** Introduced published evaluation versions for reproducible benchmark results; improved report semantics and incomplete-run handling; strengthened multimodal media loading and task-config validation.

## mani-skill/ManiSkill

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/mani-skill/ManiSkill
- Категория: benchmark/testing candidate
- Описание: Manipulation Skill Framework, an open source GPU parallelized robotics simulator and benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/mani-skill__ManiSkill/README.md](sources/mani-skill__ManiSkill/README.md); SHA-256: `304da9c2fdf69bf8944e1293a5c1bf146c434b2e6acd7604a46b3cfbfdfac10f`
- comparison: строка 21: - Many tuned robot learning baselines in Reinforcement Learning (e.g. PPO, SAC, [TD-MPC2](https://github.com/nicklashansen/tdmpc2)), Imitation Learning (e.g. Behavior Cloning, [Diffusion Policy](https://github.com/real-stanford/diffusion_policy)), and large Vi
- lifecycle: строка 30: Installation of ManiSkill is extremely simple, you only need to run a few pip installs and setup Vulkan for rendering.
- memory: строка 23: For more details we encourage you to take a look at our [paper](https://arxiv.org/abs/2410.00425), published at [RSS 2025](https://roboticsconference.org/).
- gpu: строка 14: - GPU parallelized visual data collection system. On the high end you can collect RGBD + Segmentation data at 30,000+ FPS on a 4090 GPU
- async: строка 15: - GPU parallelized simulation, enabling high throughput state-based synthetic data collection in simulation

## llm-as-a-verifier/llm-as-a-verifier

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/llm-as-a-verifier/llm-as-a-verifier
- Категория: benchmark/testing candidate
- Описание: LLM-as-a-Verifier is a general-purpose framework that provides fine-grained feedback for any agent without requiring additional training. It achieves SOTA performance across coding, robotics, and medical agentic benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/llm-as-a-verifier__llm-as-a-verifier/README.md](sources/llm-as-a-verifier__llm-as-a-verifier/README.md); SHA-256: `a3379312d68777977f4202ab5b3929eec48dad9f64d6feadcb80dcfef4d40ef2`
- lifecycle: строка 304: configuration and setup details.
- report: строка 484: `.reset()` to zero it, and `format_usage(...)` for the report block. Counts
- correctness: строка 76: criteria={"Correctness": "Does the code actually reverse the string?"},

## microsoftarchive/promptbench

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/microsoftarchive/promptbench
- Категория: benchmark/testing candidate
- Описание: A unified evaluation framework for large language models
- Уровень: automated README evidence extraction
- Снимок: [sources/microsoftarchive__promptbench/README.md](sources/microsoftarchive__promptbench/README.md); SHA-256: `82d92bd0035de72332bac6cb10c88798f16387b7a696a7eeec4be93d8fe0a328`
- report: строка 85: **PromptBench** is a Pytorch-based Python package for Evaluation of Large Language Models (LLMs). It provides user-friendly APIs for researchers to conduct evaluation on LLMs. Check the technical report: https://arxiv.org/abs/2312.07910.

## processone/tsung

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/processone/tsung
- Категория: benchmark/testing candidate
- Описание: Tsung is a high-performance benchmark framework for various protocols including HTTP, XMPP, LDAP, etc.
- Уровень: automated README evidence extraction
- Снимок: [sources/processone__tsung/README.md](sources/processone__tsung/README.md); SHA-256: `f4666ee6e00d0d780fdd6e18d5e208f438c0290f12325cfb2baf150524afae31`

## yandex/yandex-tank

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yandex/yandex-tank
- Категория: benchmark/testing candidate
- Описание: Load and performance benchmark tool
- Уровень: automated README evidence extraction
- Снимок: [sources/yandex__yandex-tank/README.md](sources/yandex__yandex-tank/README.md); SHA-256: `3a618010e069d575cb7ab20666b61397fa5caab4526572f533da8d25ba969c57`

## ARISE-Initiative/robosuite

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ARISE-Initiative/robosuite
- Категория: benchmark/testing candidate
- Описание: robosuite: A Modular Simulation Framework and Benchmark for Robot Learning
- Уровень: automated README evidence extraction
- Снимок: [sources/ARISE-Initiative__robosuite/README.md](sources/ARISE-Initiative__robosuite/README.md); SHA-256: `598d8346889d54454a89c164bdb2551aef416f806af3327e0e81e1168d00b767`
- comparison: строка 24: Data-driven algorithms, such as reinforcement learning and imitation learning, provide a powerful and generic tool in robotics. These learning paradigms, fueled by new advances in deep learning, have achieved some exciting successes in a variety of robot contr
- correctness: строка 22: **robosuite** is a simulation framework powered by the [MuJoCo](http://mujoco.org/) physics engine for robot learning. It also offers a suite of benchmark environments for reproducible research. The current release (v1.5) features support for diverse robot emb

## EMI-Group/evox

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/EMI-Group/evox
- Категория: benchmark/testing candidate
- Описание: Distributed GPU-Accelerated Framework for Evolutionary Computation. Comprehensive Library of Evolutionary Algorithms & Benchmark Problems.
- Уровень: automated README evidence extraction
- Снимок: [sources/EMI-Group__evox/README.md](sources/EMI-Group__evox/README.md); SHA-256: `fb6a389cb2edb1bd0fa05d27cc87ae3af127aebed72069cff6a9565e8ca9c5ff`
- lifecycle: строка 80: - Ensures effortless setup with **one-click installation** for Windows users.
- gpu: строка 43: <h3 align="center"> 🌟Distributed GPU-accelerated Framework for Scalable Evolutionary Computation🌟 </h3>

## IBM/AssetOpsBench

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/IBM/AssetOpsBench
- Категория: benchmark/testing candidate
- Описание: AssetOpsBench - Industry 4.0: A unified benchmark and framework for building, orchestrating, and evaluating domain-specific AI agents for Industry 4.0 asset operations and maintenance, with 460+ scenarios, 5 specialist agents (IoT, FMSR, TSFM, Work Order,...), and multi-agent orchestration blueprints (MetaAgent, AgentHive) over MCP.
- Уровень: automated README evidence extraction
- Снимок: [sources/IBM__AssetOpsBench/README.md](sources/IBM__AssetOpsBench/README.md); SHA-256: `72715f877289e5f82fab587001d3119befa55165eb1ebe1b025427f5bbc4f72e`
- lifecycle: строка 72: - 📖 **[Read INSTRUCTIONS.md](./INSTRUCTIONS.md)** — full setup, MCP servers, plan-execute runner
- async: строка 263: - **AgentOpsBench** — High-throughput battery analytics MCP server with DNN prognostics (RUL prediction) and 3.3× latency optimization. [Siddharth Gowda, Rushin Bhatt, Aryaman Agrawal, Winston Li](https://github.com/siddharthgowda), Columbia University · [repo
- report: строка 93: | **Utilities** | `json_reader`, `get_sensor_catalog`, `get_asset_catalog`, `get_failure_mode_catalog`, `current_date_time`, `current_time_english` |
- correctness: строка 81: AssetOpsBench is a **unified framework for developing, orchestrating, and evaluating domain-specific AI agents** in industrial asset operations and maintenance. It provides reproducible scenarios, agent tooling, and evaluation pipelines for multi-step workflow

## hexo-ai/sia

- Итог: excluded — AI task-quality evaluation, not a systems performance harness
- Источник: https://github.com/hexo-ai/sia
- Категория: benchmark/testing candidate
- Описание: SIA is a Self Improving AI framework to autonomously improve the performance of any AI system (Model / Agent) on a benchmark task.
- Уровень: automated README evidence extraction
- Снимок: [sources/hexo-ai__sia/README.md](sources/hexo-ai__sia/README.md); SHA-256: `910bb564b8b8205b669ee49491ba48222a81b8c9ad49f0cb57e7d2491c5b22fc`
- statistics: строка 229: **Or bring an MLE-Bench competition.** SIA can bootstrap a task directory directly from any [MLE-Bench](https://github.com/openai/mle-bench) competition — it pulls the dataset via the Kaggle API, sets up the public/private split, and drops in the reference age
- comparison: строка 12: Official implementation of [**SIA: Self Improving AI with Harness & Weight Updates**](https://arxiv.org/abs/2605.27276) (Hebbar et al., 2026) — a self-improving loop where a language-model agent updates both the harness and the weights of a task-specific agent
- lifecycle: строка 20: - [SIA setup](https://www.loom.com/share/be0534bc818d408bab937033c6457ec9)
- gpu: строка 12: Official implementation of [**SIA: Self Improving AI with Harness & Weight Updates**](https://arxiv.org/abs/2605.27276) (Hebbar et al., 2026) — a self-improving loop where a language-model agent updates both the harness and the weights of a task-specific agent
- async: строка 44: <p align="center"><img src="docs/trimul_cuda.png" alt="TriMul CUDA Results" width="720"><br><i>AlphaFold-3 TriMul Triton Kernel: implement and optimize the Triangle Multiplicative Update as a Triton kernel, preserving correctness while hitting H100 latency tar
- report: строка 12: Official implementation of [**SIA: Self Improving AI with Harness & Weight Updates**](https://arxiv.org/abs/2605.27276) (Hebbar et al., 2026) — a self-improving loop where a language-model agent updates both the harness and the weights of a task-specific agent
- correctness: строка 44: <p align="center"><img src="docs/trimul_cuda.png" alt="TriMul CUDA Results" width="720"><br><i>AlphaFold-3 TriMul Triton Kernel: implement and optimize the Triangle Multiplicative Update as a Triton kernel, preserving correctness while hitting H100 latency tar

## smallnest/go-web-framework-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/smallnest/go-web-framework-benchmark
- Категория: benchmark/testing candidate
- Описание: :zap: Go web framework benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/smallnest__go-web-framework-benchmark/README.md](sources/smallnest__go-web-framework-benchmark/README.md); SHA-256: `3344912b3917007770adaa5d04441ef03ae63f999d33cbaf28e28b756374ec52`
- memory: строка 113: Allocs is the heap allocations by web servers when test is running. The unit is MB. The smaller is the better.
- async: строка 107: the concurrency clients are 5000.
- report: строка 154: It will  generate test results in processtime.csv and concurrency.csv. You can modify test.sh to execute your customized test cases.

## phpbench/phpbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/phpbench/phpbench
- Категория: benchmark/testing candidate
- Описание: PHP Benchmarking framework
- Уровень: automated README evidence extraction
- Снимок: [sources/phpbench__phpbench/README.md](sources/phpbench__phpbench/README.md); SHA-256: `1e7a14272fc189ffacacc7132be4845fe14e8736328d1698f7a16376f989b988`
- statistics: строка 23: statistical data.
- comparison: строка 27: - Report [storage](https://phpbench.readthedocs.io/en/latest/guides/storage.html) and [comparison](https://phpbench.readthedocs.io/en/latest/guides/regression-testing.html): Store benchmarks locally to be used as a
- memory: строка 29: - **Memory Usage**: Keep an eye on the amount of memory used by benchmarking
- report: строка 25: - [Reporting](https://phpbench.readthedocs.io/en/latest/guides/reports.html): Customizable reports and various output formats (e.g.
- correctness: строка 16: correctness.
- isolation: строка 24: - **Process Isolation**: Each iteration is executed in a separate process.

## Elanis/web-to-desktop-framework-comparison

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Elanis/web-to-desktop-framework-comparison
- Категория: benchmark/testing candidate
- Описание: An objective comparison of multiple frameworks that allow us to "transform" our web apps to desktop applications.
- Уровень: automated README evidence extraction
- Снимок: [sources/Elanis__web-to-desktop-framework-comparison/README.md](sources/Elanis__web-to-desktop-framework-comparison/README.md); SHA-256: `c6ebb581c9e640e1553e946e2164976f9fb66c2a484e6691c8d3d9d2967d4330`
- lifecycle: строка 44: | **Angular** | Yes | [Yes, official via Vite](https://tauri.app/v1/guides/getting-started/setup/vite) | Yes | [Yes, community](https://github.com/irustm/angular-nodegui) | Yes | [Yes](https://wails.io/docs/guides/angular/) | No | No
- memory: строка 110: ### Memory Usage - (Average of runs) Median of used memory for main process and children ones)
- report: строка 73: **See [benchmarks.json](https://github.com/Elanis/web-to-desktop-framework-comparison/blob/main/runner/benchmarks.json) to get more information about the following data.**

## IntelLabs/fastRAG

- Итог: excluded — RAG framework; no benchmark architecture evidence selected
- Источник: https://github.com/IntelLabs/fastRAG
- Категория: benchmark/testing candidate
- Описание: Efficient Retrieval Augmentation and Generation Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/IntelLabs__fastRAG/README.md](sources/IntelLabs__fastRAG/README.md); SHA-256: `63861748376ed2d1a2a36deea698948594cfb5667b38f2cc170eda8b326b8e35`
- gpu: строка 133: pip install fastrag[faiss-gpu]           # GPU-based Faiss library
- report: строка 29: > Now compatible with Haystack v2+. Please report any possible issues you find.

## julienschmidt/go-http-routing-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/julienschmidt/go-http-routing-benchmark
- Категория: benchmark/testing candidate
- Описание: Go HTTP request router and web framework benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/julienschmidt__go-http-routing-benchmark/README.md](sources/julienschmidt__go-http-routing-benchmark/README.md); SHA-256: `ead34be7b5c7aee9ca5ec2ecefd59b1ff6adc0f519b0b629823b7da4ff985578`
- memory: строка 34: Unfortunately, most of the (early) routers use pretty bad routing algorithms. Moreover, many of them are very wasteful with memory allocations, which can become a problem in a language with Garbage Collection like Go, since every (heap) allocation results in m
- report: строка 13: * [go-json-rest](https://github.com/ant0ine/go-json-rest)

## yoshitomo-matsubara/torchdistill

- Итог: excluded — Model-quality evaluation outside current scope
- Источник: https://github.com/yoshitomo-matsubara/torchdistill
- Категория: benchmark/testing candidate
- Описание: A coding-free framework built on PyTorch for reproducible deep learning studies. PyTorch Ecosystem. 🏆26 knowledge distillation methods presented at TPAMI, CVPR, ICLR, ECCV, NeurIPS, ICCV, AAAI, etc are implemented so far. 🎁 Trained models, training logs and configurations are available for ensuring the reproducibiliy and benchmark.
- Уровень: automated README evidence extraction
- Снимок: [sources/yoshitomo-matsubara__torchdistill/README.md](sources/yoshitomo-matsubara__torchdistill/README.md); SHA-256: `97c8973faa53fad32f0426bd07219bea67aa716d5a1d9f5811a5268370c51b76`
- lifecycle: строка 197: ## How to setup
- gpu: строка 165: if you have your own GPU(s).
- correctness: строка 17: (**WITHOUT coding**) for reproducible deep learning studies. i.e., it enables you to train models without teachers

## kengz/SLM-Lab

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/kengz/SLM-Lab
- Категория: benchmark/testing candidate
- Описание: Modular Deep Reinforcement Learning framework in PyTorch. Companion library of the book "Foundations of Deep Reinforcement Learning".
- Уровень: automated README evidence extraction
- Снимок: [sources/kengz__SLM-Lab/README.md](sources/kengz__SLM-Lab/README.md); SHA-256: `0e957fec7447909f026b5032114017e50be743287fe0899b1acc171abcd5cf4a`
- lifecycle: строка 93: # Setup
- gpu: строка 36: | **Cloud integration** | dstack for GPU training, HuggingFace for sharing results |
- report: строка 33: | **Easy configuration** | JSON spec files fully define experiments—no code changes needed |
- correctness: строка 32: | **Ready-to-use algorithms** | PPO, SAC, CrossQ, DQN, A2C, REINFORCE—validated on 70+ environments |

## ZJU-REAL/ClawGUI

- Итог: excluded — GUI-agent training and task success evaluation
- Источник: https://github.com/ZJU-REAL/ClawGUI
- Категория: benchmark/testing candidate
- Описание: Build, Evaluate, and Deploy GUI Agents — online RL training, standardized benchmarks, and real-device deployment in one framework.
- Уровень: automated README evidence extraction
- Снимок: [sources/ZJU-REAL__ClawGUI/README.md](sources/ZJU-REAL__ClawGUI/README.md); SHA-256: `3f0aaefce32210dfba428fcf9e609e04e75d4d1520d24a3a6377611a9b83fa6d`
- comparison: строка 42: + 🔥 **[2026/4/13]** ClawGUI is released — train with ClawGUI-RL (GiGPO), evaluate with ClawGUI-Eval, deploy with ClawGUI-Agent. ClawGUI-2B, a 2B agent trained end-to-end with this pipeline, hits **17.1** MobileWorld SR vs. the **11.1** baseline. See [Quick Sta
- lifecycle: строка 166: > 📁 [`clawgui-app/`](clawgui-app/) · 📖 [Setup Guide](clawgui-app/SETUP.md)
- gpu: строка 119: - **Dual backend** — Local GPU (`transformers`) or remote API (OpenAI-compatible)
- report: строка 157: - **Structured packages** — `meta_info.json`, `plan.md`, `backup.md`, `recover.md`, and `failure_examples/`
- correctness: строка 70: | 🧩 **[ClawGUI-Skills](clawgui-skills/)** | **Self-evolving skills** — Training-free skill evolution proposed and validated in our paper: structured packages, retrieval, failure diagnosis, restricted revision, and reuse |

## flow-project/flow

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/flow-project/flow
- Категория: benchmark/testing candidate
- Описание: Computational framework for reinforcement learning in traffic control
- Уровень: automated README evidence extraction
- Снимок: [sources/flow-project__flow/README.md](sources/flow-project__flow/README.md); SHA-256: `8489019ff8ced05181d1f860f5ac9f5e656db2cb76865a6b9d91afde7c60598e`
- lifecycle: строка 18: - [Installation instructions](http://flow.readthedocs.io/en/latest/flow_setup.html)
- report: строка 24: If you have a bug, please report it. Otherwise, join the [Flow Users group](https://join.slack.com/t/flow-users/shared_invite/enQtODQ0NDYxMTQyNDY2LTY1ZDVjZTljM2U0ODIxNTY5NTQ2MmUxMzYzNzc5NzU4ZTlmNGI2ZjFmNGU4YjVhNzE3NjcwZTBjNzIxYTg5ZmY) on Slack!

## PKU-Alignment/omnisafe

- Итог: excluded — Safe RL quality benchmark outside current scope
- Источник: https://github.com/PKU-Alignment/omnisafe
- Категория: benchmark/testing candidate
- Описание: JMLR: OmniSafe is an infrastructural framework for accelerating SafeRL research.
- Уровень: automated README evidence extraction
- Снимок: [sources/PKU-Alignment__omnisafe/README.md](sources/PKU-Alignment__omnisafe/README.md); SHA-256: `e787c2c25cd56af60d6582b5378836715582c66d5eeb7b61bf309f3fca641c3f`
- async: строка 348: # 2. num_pool(how much processes are concurrent)
- correctness: строка 359: # Quick training some algorithms to validate your thoughts

## tensorflow/benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tensorflow/benchmarks
- Категория: benchmark/testing candidate
- Описание:  A benchmark framework for Tensorflow
- Уровень: automated README evidence extraction
- Снимок: [sources/tensorflow__benchmarks/README.md](sources/tensorflow__benchmarks/README.md); SHA-256: `cc1aae64b642b43f0479410eb8ce634a3b5fcb18a965989845b58ed9b2b6d824`

## mikeroyal/Open-Source-Security-Guide

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/mikeroyal/Open-Source-Security-Guide
- Категория: benchmark/testing candidate
- Описание: Open Source Security Guide. Learn all about Security Standards (FIPS, CIS, FedRAMP, FISMA, etc.), Frameworks, Threat Models, Encryption, and Benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/mikeroyal__Open-Source-Security-Guide/README.md](sources/mikeroyal__Open-Source-Security-Guide/README.md); SHA-256: `e379bd4555f5cbb0396989ad8337fd59936c3c9e48f80d367a2f463a136214b7`
- statistics: строка 461: * **Analytics** — UEBA solutions detect anomalies using a variety of analytics approaches–statistical models, machine learning, rules, threat signatures and more.
- comparison: строка 582: [OpenSCAP](https://www.open-scap.org/) is U.S. standard maintained by [National Institute of Standards and Technology (NIST)](https://www.nist.gov/). It provides multiple tools to assist administrators and auditors with assessment, measurement, and enforcement
- lifecycle: строка 612: [Cloudflare Tunnel client](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/tunnel-guide) is a tunneling daemon that proxies traffic from the Cloudflare network to your origins. This daemon sits between Cloudflare net
- memory: строка 666: [Parca](https://parca.dev/) is a tool for continuous profiling for analysis of CPU and memory usage, down to the line number and throughout time. Saving infrastructure cost, improving performance, and increasing reliability.
- async: строка 726: [Ory Hydra](https://github.com/ory/hydra) is a hardened, OpenID Certified OAuth 2.0 Server and OpenID Connect Provider optimized for low-latency, high throughput, and low resource consumption. Ory Hydra is not an identity provider (user sign up, user login, pa
- report: строка 237: [SOC 2](https://www.aicpa.org/interestareas/frc/assuranceadvisoryservices/aicpasoc2report.html) is an auditing procedure that ensures your service providers securely manage your data to protect the interests of your comapny/organization and the privacy of thei
- correctness: строка 191: [![Simulate Attacks with Infection Monkey | Cyber Security Simulation, Validation, and Mitigation](https://ytcards.demolab.com/?id=PAwTLfR5pGU&lang=en&background_color=%230d1117&title_color=%23ffffff&stats_color=%23dedede&width=240 "Simulate Attacks with Infec

## kenjis/php-framework-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kenjis/php-framework-benchmark
- Категория: benchmark/testing candidate
- Описание: PHP Framework Benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/kenjis__php-framework-benchmark/README.md](sources/kenjis__php-framework-benchmark/README.md); SHA-256: `23e9d22a5663f838f2ba0df2a8c1d6d915906fb5e63691172a55a489eada6363`
- statistics: строка 3: This project attempts to measure minimum overhead (minimum bootstrap cost) of PHP frameworks in the real world.
- lifecycle: строка 86: $ bash setup.sh

## DigitalInBlue/Celero

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DigitalInBlue/Celero
- Категория: benchmark/testing candidate
- Описание: C++ Benchmark Authoring Library/Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/DigitalInBlue__Celero/README.md](sources/DigitalInBlue__Celero/README.md); SHA-256: `c085211c0c8613788ae55cab68baccb9f70dfb33ddd41b1c683a369095a1c7af`
- statistics: строка 225: Celero helps with this by allowing you to specify zero samples.  Zero samples will tell Celero to make some statistically significant number of samples based on how long it takes to complete your specified number of operations.  These numbers will be reported
- comparison: строка 53: -   Supports fixed-time benchmark baselines.
- lifecycle: строка 77: The goal of writing correct benchmarking code is to eliminate all of the noise and overhead and measure only the code under test.  Sources of noise in the measurements include clock resolution noise, operating system background operations, test setup/teardown,
- async: строка 292: Celero can automatically run threaded benchmarks.  `BASELINE_T` and `BENCHMARK_T` can be used to launch the given code on its own thread using a user-defined number of concurrent executions.  `celeroDemoMultithread` illustrates using this feature.  When defini
- report: строка 27: -   `celero-<version>-<triplet>.spdx.json` describes what you link against. Celero has **no third-party runtime dependencies** — only operating-system libraries (`powrprof` and `psapi` on Windows, `pthread` elsewhere). GoogleTest is test-only and never enters
- isolation: строка 83: Once this measurement is obtained, it has little meaning in isolation.  It is essential to create a baseline test by which to compare.  A baseline should generally be a "classic" or "pure" solution to the problem on which you measure a solution.   Once you hav

## capgym/cap-x

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/capgym/cap-x
- Категория: benchmark/testing candidate
- Описание: [ICML 2026] A Framework for Benchmarking and Improving Coding Agents for Robot Manipulation
- Уровень: automated README evidence extraction
- Снимок: [sources/capgym__cap-x/README.md](sources/capgym__cap-x/README.md); SHA-256: `03fcc86b900211f40cdfb908794a16beb5901c067fecaa2f383880cd4c7c0a86`
- comparison: строка 99: See [docs/behavior-tasks.md](docs/behavior-tasks.md) for task details and expected baselines.
- lifecycle: строка 53: ### Simulator-specific setup
- memory: строка 120: # Start SAM3 + GraspNet + PyRoKi with automatic GPU allocation
- gpu: строка 36: CaP-X uses [uv](https://docs.astral.sh/uv/) for dependency management. Requires **Python 3.10** and a **CUDA-capable GPU**.
- report: строка 96: sudo rm -f /usr/share/vulkan/icd.d/nvidia_icd.json

## eknkc/ssr-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/eknkc/ssr-benchmark
- Категория: benchmark/testing candidate
- Описание: Benchmarking JS web framework SSR performance
- Уровень: automated README evidence extraction
- Снимок: [sources/eknkc__ssr-benchmark/readme.md](sources/eknkc__ssr-benchmark/readme.md); SHA-256: `32c343554baef0573a42433513820d070839d40853b8fd67b8c364bd62815454`
- comparison: строка 21: - **react** is here only as a baseline renderer to compare framework performance with.
- lifecycle: строка 5: This is not a comprehensive or scientific test. Just wanted to compare each in a setup a little complex than just printing `hello world`.
- async: строка 49: - The table data is emulated as async and requires Suspense on react, solid and vue. On Next it is loaded in an async RSC component. On Remix it is loaded in a route `loader` function.

## caderek/benny

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/caderek/benny
- Категория: benchmark/testing candidate
- Описание: A dead simple benchmarking framework for JS/TS libs
- Уровень: automated README evidence extraction
- Снимок: [sources/caderek__benny/README.md](sources/caderek__benny/README.md); SHA-256: `59faacc4fbcf12d8a031b5c949c8fc0319fa49bcf511cab81b085a812102ce88`
- statistics: строка 181: "sampleVariance": 2.439038395786062e-20,
- lifecycle: строка 34: - prepare local setup (sync or async) for each case,
- async: строка 18: 6. [Working with async code](#async-code)
- report: строка 36: - save results to a JSON / CSV / HTML (table or chart) file,

## melix/jmh-gradle-plugin

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/melix/jmh-gradle-plugin
- Категория: benchmark/testing candidate
- Описание: Integrates the JMH benchmarking framework with Gradle
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## klen/py-frameworks-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/klen/py-frameworks-bench
- Категория: benchmark/testing candidate
- Описание: Another benchmark for some python frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/klen__py-frameworks-bench/README.md](sources/klen__py-frameworks-bench/README.md); SHA-256: `b199fa49e6bd2c1f60315a4700c21cc6dc31e84269250e3bc260b1169045701b`
- async: строка 1: # Async Python Web Frameworks comparison
- report: строка 26: * [Parse path params, query string, JSON body and return a json response](#api)

## networknt/microservices-framework-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/networknt/microservices-framework-benchmark
- Категория: benchmark/testing candidate
- Описание: Raw benchmarks on throughput, latency and transfer of Hello World on popular microservices frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/networknt__microservices-framework-benchmark/README.md](sources/networknt__microservices-framework-benchmark/README.md); SHA-256: `bf1411f569f08c2643e07fb76b1e6d0a65065b6220911451eccb537b3f80e5a8`
- async: строка 10: | Framework    | Language   | Max Throughput | Avg Latency | Transfer |

## hyperledger-caliper/caliper

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/hyperledger-caliper/caliper
- Категория: benchmark/testing candidate
- Описание:  A blockchain benchmark framework to measure performance of multiple blockchain solutions https://wiki.hyperledger.org/display/caliper
- Уровень: automated README evidence extraction
- Снимок: [sources/hyperledger-caliper__caliper/README.md](sources/hyperledger-caliper__caliper/README.md); SHA-256: `334ad94ddc60fa8202bcc3ea1b0805481215e793b83813de5a1b7388ef6b64f1`
- async: строка 21: * Transaction/Read throughput

## mroth/phoenix-showdown

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mroth/phoenix-showdown
- Категория: benchmark/testing candidate
- Описание: :horse_racing: benchmark Sinatra-like web frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/mroth__phoenix-showdown/README.md](sources/mroth__phoenix-showdown/README.md); SHA-256: `941cd1ed6a0b0b42c2610f54b9a600b1b8493bb70013dc43fa5a1e5f0f6fa323`
- async: строка 54: was disabled to be more comparable (netting a ~35% throughput increase right

## Picovoice/speech-to-text-benchmark

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/Picovoice/speech-to-text-benchmark
- Категория: benchmark/testing candidate
- Описание: speech to text benchmark framework
- Уровень: automated README evidence extraction
- Снимок: [sources/Picovoice__speech-to-text-benchmark/README.md](sources/Picovoice__speech-to-text-benchmark/README.md); SHA-256: `e4bec178e54b34233e69ed527e967f96d1b912c67b7a7d96bbb55ba14a4b3de3`
- async: строка 41: ### Word Emission Latency
- report: строка 33: Punctuation Error Rate (PER) is the ratio of punctuation-specific errors between a reference transcript and the output of a speech-to-text engine to the number of punctuation-related operations in the reference transcript (more details in Section 3 of [Meister

## JuliaCI/BenchmarkTools.jl

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/JuliaCI/BenchmarkTools.jl
- Категория: benchmark/testing candidate
- Описание: A benchmarking framework for the Julia language
- Уровень: automated README evidence extraction
- Снимок: [sources/JuliaCI__BenchmarkTools.jl/README.md](sources/JuliaCI__BenchmarkTools.jl/README.md); SHA-256: `802cb341a302fc24aeabde5d7b63e2e94836800944646d3b31767988ee72e664`
- statistics: строка 125: 2. The estimates used to characterize benchmark results and to detect regressions were statistically vulnerable to noise (i.e. not robust).
- comparison: строка 120: Our story begins with two packages, "Benchmarks" and "BenchmarkTrackers". The Benchmarks package implemented an execution strategy for collecting and summarizing individual benchmark results, while BenchmarkTrackers implemented a framework for organizing, runn
- lifecycle: строка 57: # The `setup` expression is run once per sample, and is not included in the
- memory: строка 78: 4.361 ns (0 allocations: 0 bytes)
- report: строка 106: As a rule of thumb, if a benchmark reports that it took less than a nanosecond to perform, this hoisting probably occurred. You can avoid this using interpolation:

## cmu-db/benchbase

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cmu-db/benchbase
- Категория: benchmark/testing candidate
- Описание: Multi-DBMS SQL Benchmarking Framework via JDBC
- Уровень: automated README evidence extraction
- Снимок: [sources/cmu-db__benchbase/README.md](sources/cmu-db__benchbase/README.md); SHA-256: `640e647e75beba2c92f0f43a9cdd7a30f7ee43b5503ceb505ccf90e9b47f4d55`
- async: строка 59: features, e.g., per-transaction-type latency and throughput logs.
- report: строка 140: -jh,--json-histograms <arg>    Export histograms to JSON file

## fastify/benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fastify/benchmarks
- Категория: benchmark/testing candidate
- Описание: Fast and low overhead web framework fastify benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/fastify__benchmarks/README.md](sources/fastify__benchmarks/README.md); SHA-256: `c329b302d2a2479e6713334ee308ff6c3c770a345da3ff6184510040dcf40fbd`
- lifecycle: строка 76: * __Method:__ `autocannon -c 100 -d 40 -p 10 localhost:3000` (two rounds; one to warm-up, one to measure)
- async: строка 78: |                          | Version     | Router | Requests/s | Latency (ms) | Throughput/Mb |
- report: строка 101: | fastify-big-json         | 5.12.1      | ✓      | 19929.2    | 49.65        | 229.31        |

## ServiceNow/AgentLab

- Итог: excluded — Agent task evaluation outside current scope
- Источник: https://github.com/ServiceNow/AgentLab
- Категория: benchmark/testing candidate
- Описание: AgentLab: An open-source framework for developing, testing, and benchmarking web agents on diverse tasks, designed for scalability and reproducibility.
- Уровень: automated README evidence extraction
- Снимок: [sources/ServiceNow__AgentLab/README.md](sources/ServiceNow__AgentLab/README.md); SHA-256: `2766b7c1e9619377c1508cc6440a2c508b9baaae706f4ac8e4eb7ba9b0c68c99`
- lifecycle: строка 15: [🛠️ Setup](#%EF%B8%8F-setup-agentlab) &nbsp;|&nbsp;
- report: строка 270: [`reproducibility_journal.csv`](reproducibility_journal.csv). This makes it easier to populate a
- correctness: строка 22: [↻ Reproducibility](#-reproducibility) &nbsp;|&nbsp;

## facebookresearch/MLGym

- Итог: excluded — AI research agent evaluation
- Источник: https://github.com/facebookresearch/MLGym
- Категория: benchmark/testing candidate
- Описание: MLGym A New Framework and Benchmark for Advancing AI Research Agents
- Уровень: automated README evidence extraction
- Снимок: [sources/facebookresearch__MLGym/README.md](sources/facebookresearch__MLGym/README.md); SHA-256: `db82bf4d05805fb7ef24b1e539423b78df32ec867f466c48f50f26754dfc8652`
- gpu: строка 68: 5. If you are working on a Linux machine, please install the `nvidia-container-runtime`. This is required to start docker containers with GPU support.

## amazon-archives/aws-security-benchmark

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/amazon-archives/aws-security-benchmark
- Категория: benchmark/testing candidate
- Описание: Open source demos, concept and guidance related to the AWS CIS Foundation framework.
- Уровень: automated README evidence extraction
- Снимок: [sources/amazon-archives__aws-security-benchmark/README.md](sources/amazon-archives__aws-security-benchmark/README.md); SHA-256: `482004bc695a11f98841fe4a935b6e227d1e55cb27583750df64b0c97b3fa0cb`

## SalesforceAIResearch/MCP-Universe

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/SalesforceAIResearch/MCP-Universe
- Категория: benchmark/testing candidate
- Описание: MCP-Universe is a comprehensive framework designed for RL training, benchmarking, and developing AI agents for general tool-use.
- Уровень: automated README evidence extraction
- Снимок: [sources/SalesforceAIResearch__MCP-Universe/README.md](sources/SalesforceAIResearch__MCP-Universe/README.md); SHA-256: `cb5cbfe7050566e646d746c333edf39662397545064608a176c8c4f1bd662223`
- lifecycle: строка 248: #### 1. Initial Setup
- async: строка 221: async def test():
- report: строка 52: - [Save the benchmark result to a report](#save-the-benchmark-result-to-a-report)

## allenai/vla-evaluation-harness

- Итог: excluded — Robot policy evaluation outside current scope
- Источник: https://github.com/allenai/vla-evaluation-harness
- Категория: benchmark/testing candidate
- Описание: One framework to evaluate any VLA model on any robot simulation benchmark.
- Уровень: automated README evidence extraction
- Снимок: [sources/allenai__vla-evaluation-harness/README.md](sources/allenai__vla-evaluation-harness/README.md); SHA-256: `d00666d0b8c4e9d735e979284e98ca9c1631a7f986b35e9e883c1e7a89e2a96b`
- lifecycle: строка 38: | **Zero Setup** | Benchmarks in Docker and model servers as single-file [uv scripts](https://docs.astral.sh/uv/guides/scripts/), avoiding dependency conflicts. |
- gpu: строка 37: | **Batch Parallel Evaluation** | Episode sharding + batched GPU inference → **47× throughput** (2 000 LIBERO episodes in 18 min on 1× H100). [Details](#batch-parallel-evaluation) |
- async: строка 37: | **Batch Parallel Evaluation** | Episode sharding + batched GPU inference → **47× throughput** (2 000 LIBERO episodes in 18 min on 1× H100). [Details](#batch-parallel-evaluation) |
- report: строка 7: [![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)
- correctness: строка 25: - [2026/07] [v0.4.0](https://github.com/allenai/vla-evaluation-harness/releases/tag/v0.4.0) released. Recording on by default, pinned reproducible Docker rebuilds, DuoBench, and the LeRobot bridge below.
- isolation: строка 137: # ... (each shard is a separate process)

## SaltyAom/bun-http-framework-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SaltyAom/bun-http-framework-benchmark
- Категория: benchmark/testing candidate
- Описание: Compare throughput benchmark from various Bun HTTP framework
- Уровень: automated README evidence extraction
- Снимок: [sources/SaltyAom__bun-http-framework-benchmark/README.md](sources/SaltyAom__bun-http-framework-benchmark/README.md); SHA-256: `15aa279b0cb9c8f65da8db90db22746dad04d8e5bb717ff5df8b63400a9d7227`
- memory: строка 42: the emitted minified bundle size, startup time, and server RSS memory as
- async: строка 3: Compare throughput benchmarks from various JavaScript HTTP framework
- report: строка 43: `before / after MB` in one column. Deno reports `n/a` for bundle size because it
- correctness: строка 63: every run reproducible.

## petabridge/NBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/petabridge/NBench
- Категория: benchmark/testing candidate
- Описание: Performance benchmarking and testing framework for .NET applications :chart_with_upwards_trend:
- Уровень: automated README evidence extraction
- Снимок: [sources/petabridge__NBench/README.md](sources/petabridge__NBench/README.md); SHA-256: `94f80e016c21833f8674a3da32658ba72f019c9996ddf7d692519897ecdc4804`
- correctness: строка 14: | Windows Tests     | [![Build Status](https://dev.azure.com/petabridge/NBench/_apis/build/status/NBench%20PR%20Validation?branchName=dev&jobName=Windows%20Tests)](https://dev.azure.com/petabridge/NBench/_build/latest?definitionId=115&branchName=dev)     |

## itbench-hub/ITBench

- Итог: excluded — IT agent evaluation outside current scope
- Источник: https://github.com/itbench-hub/ITBench
- Категория: benchmark/testing candidate
- Описание: An open source benchmarking framework for IT automation
- Уровень: automated README evidence extraction
- Снимок: [sources/itbench-hub__ITBench/README.md](sources/itbench-hub__ITBench/README.md); SHA-256: `5ad388d1cec81dfdae6c2fc2f44e9a8533685e684249eb517b8ec95f93d473d6`
- comparison: строка 20: - **[February 7, 2025]** 🎉 **Initial release!** Includes research paper, self-hosted environment setup tooling, sample scenarios, and baseline agents.
- lifecycle: строка 20: - **[February 7, 2025]** 🎉 **Initial release!** Includes research paper, self-hosted environment setup tooling, sample scenarios, and baseline agents.
- report: строка 142: - [**Create a GitHub issue**](https://github.com/itbench-hub/ITBench/issues) for bug reports or feature requests
- correctness: строка 115: ITBench is part of IBM Research's **[Enterprise Agents and Benchmarks](https://huggingface.co/collections/ibm-research/enterprise-agents-and-benchmarks)** collection on Hugging Face, a family of open benchmarks for evaluating AI agents on enterprise tasks. Thi

## shchur/gnn-benchmark

- Итог: excluded — Graph model quality evaluation
- Источник: https://github.com/shchur/gnn-benchmark
- Категория: benchmark/testing candidate
- Описание: Framework for evaluating Graph Neural Network models on semi-supervised node classification task
- Уровень: automated README evidence extraction
- Снимок: [sources/shchur__gnn-benchmark/README.md](sources/shchur__gnn-benchmark/README.md); SHA-256: `9b5b8743b32882cd67687d4c882cc485dfad570cdfc67488d67772367cf4ca62`
- comparison: строка 180: - Baseline models: Multilayer Perceptron, Logistic Regression and Label Propagation.
- lifecycle: строка 54: pip install -e .  # has to be run in the directory with setup.py file, i.e. in gnn-benchmark/
- gpu: строка 134: python scripts/spawn_worker.py -c configs/fixed_configs.conf.yaml --gpu 0
- report: строка 72: The results are retrieved from the database, aggregated and stored in CSV format.
- correctness: строка 4: The framework provides a simple interface for running different models on several datasets while using multiple train/validation/test splits.
- isolation: строка 131: The script works by retrieving pending jobs (i.e. records) from the `pending` database and executing them in a subprocess.

## BuilderIO/framework-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/BuilderIO/framework-benchmarks
- Категория: benchmark/testing candidate
- Описание: Test each framework for it's performance cost
- Уровень: automated README evidence extraction
- Снимок: [sources/BuilderIO__framework-benchmarks/README.md](sources/BuilderIO__framework-benchmarks/README.md); SHA-256: `1969936dd81993d69c1f7d073859a909aa0b4cdc3468a0377898fe2f68f32a68`
- lifecycle: строка 265: ### Setup
- async: строка 75: - [SSR throughput (req/second)](#ssr-throughput-reqsecond)
- report: строка 71: - [Dashboard](#dashboard)

## openml/automlbenchmark

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/openml/automlbenchmark
- Категория: benchmark/testing candidate
- Описание: OpenML AutoML Benchmarking Framework
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## openmessaging/benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/openmessaging/benchmark
- Категория: benchmark/testing candidate
- Описание: OpenMessaging Benchmark Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/openmessaging__benchmark/README.md](sources/openmessaging__benchmark/README.md); SHA-256: `105a32975540e7d971dcf3bfc51eaebff15015d85b98037c49f6532801f0eff4`

## 0ca/BoxPwnr

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/0ca/BoxPwnr
- Категория: benchmark/testing candidate
- Описание: A modular framework for benchmarking LLMs and agentic strategies on security challenges across HackTheBox, TryHackMe, PortSwigger Labs, Cybench, picoCTF and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/0ca__BoxPwnr/README.md](sources/0ca__BoxPwnr/README.md); SHA-256: `555ab2950eddc9be5c1a262d81a229542944bff2b53201ed623e1d80bc853580`
- lifecycle: строка 158: - **`ssh`**: Executes commands on a remote host via SSH. Useful for custom networking setups or when running on your own infrastructure. Requires `--ssh-host` (and optionally `--ssh-username`, `--ssh-key-path`, `--ssh-port`).
- report: строка 114: #### Analysis and Reporting
- isolation: строка 144: - `--external-timeout`: Timeout for external solver subprocess in seconds (default: 3600)

## layoutBox/LayoutFrameworkBenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/layoutBox/LayoutFrameworkBenchmark
- Категория: benchmark/testing candidate
- Описание: Benchmark the performances of various Swift layout frameworks (autolayout, UIStackView, PinLayout, LayoutKit, FlexLayout, Yoga, ...)
- Уровень: automated README evidence extraction
- Снимок: [sources/layoutBox__LayoutFrameworkBenchmark/README.md](sources/layoutBox__LayoutFrameworkBenchmark/README.md); SHA-256: `4fd499a8c25ec1c366ad29e2b69c3b3cc48baa3793f9f28f44613be4321e7e0b`

## harsha-simhadri/big-ann-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/harsha-simhadri/big-ann-benchmarks
- Категория: benchmark/testing candidate
- Описание: Framework for evaluating ANNS algorithms on billion scale datasets.
- Уровень: automated README evidence extraction
- Снимок: [sources/harsha-simhadri__big-ann-benchmarks/README.md](sources/harsha-simhadri__big-ann-benchmarks/README.md); SHA-256: `99c012bb41414573ecff0f56e44d440cec1d3c6533ca9f289ac4ca0ed4e44862`

## wil3/gymfc

- Итог: excluded — Controller training benchmark outside current scope
- Источник: https://github.com/wil3/gymfc
- Категория: benchmark/testing candidate
- Описание: A universal flight control tuning framework
- Уровень: automated README evidence extraction
- Снимок: [sources/wil3__gymfc/README.md](sources/wil3__gymfc/README.md); SHA-256: `3504b69ace4a5a4a486885c891a018cd5ce92667bd22d15e08deec722c6741a4`
- comparison: строка 246: flight controller and tuner are one in the same, e.g., OpenAI baselines) This will expand the flight control research that
- lifecycle: строка 129: may need to change the location of the Gazebo `setup.sh` defined by the
- report: строка 3: [![All-Contributors](https://img.shields.io/badge/dynamic/json?color=orange&label=all%20contributors&query=%24.contributors.length&url=https%3A%2F%2Fraw.githubusercontent.com%2Fwil3%2Fgymfc%2Fmaster%2F.all-contributorsrc)](#contributors-)

## u39kun/deep-learning-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/u39kun/deep-learning-benchmark
- Категория: benchmark/testing candidate
- Описание: Deep Learning Benchmark for comparing the performance of DL frameworks, GPUs, and single vs half precision
- Уровень: automated README evidence extraction
- Снимок: [sources/u39kun__deep-learning-benchmark/README.md](sources/u39kun__deep-learning-benchmark/README.md); SHA-256: `9080eb2bd3ae725d6371d2b7f16ceeb72d0c4fd2fe4803f309af1a5477138ff7`
- lifecycle: строка 66: In both scenarios, 20 runs of warm up is performed and those are not counted towards the measured numbers.
- gpu: строка 7: Note: Docker images available from NVIDIA GPU Cloud were used so as to make benchmarking controlled and repeatable by anyone.

## oltpbenchmark/oltpbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/oltpbenchmark/oltpbench
- Категория: benchmark/testing candidate
- Описание: Database Benchmarking Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/oltpbenchmark__oltpbench/README.md](sources/oltpbenchmark__oltpbench/README.md); SHA-256: `ea3c775ced55959e4e2e4734de87d0be34b109a6ea91ecb0f6d458581750423f`
- async: строка 19: features, e.g., per-transaction-type latency and throughput logs.
- isolation: строка 34: * [SIBench](http://sydney.edu.au/engineering/it/~fekete/teaching/serializableSI-Fekete.pdf) (Snapshot Isolation)

## the-benchmarker/website

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/the-benchmarker/website
- Категория: benchmark/testing candidate
- Описание: Source Code for Web Frameworks Benchmark Website
- Уровень: automated README evidence extraction
- Снимок: [sources/the-benchmarker__website/README.md](sources/the-benchmarker__website/README.md); SHA-256: `c07a5978bdeff961be28963b01fc41972b640ac641962f785fc0898226367a83`
- report: строка 41: - `sitemap.xml`, `robots.txt`, `llms.txt`, `llms-full.txt` and `data.json`

## irobot-ros/ros2-performance

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/irobot-ros/ros2-performance
- Категория: benchmark/testing candidate
- Описание: Framework to evaluate peformance of ROS 2
- Уровень: automated README evidence extraction
- Снимок: [sources/irobot-ros__ros2-performance/README.md](sources/irobot-ros__ros2-performance/README.md); SHA-256: `3dc345e03b23195c459627cec22567bf23c204c0e8eb6ae3178d11ba98ad5fbe`
- lifecycle: строка 42: source ~/performance_ws/install/setup.bash
- memory: строка 10: - Memory usage
- async: строка 7: - Latency
- report: строка 4: The system topology can be provided at runtime using JSON files or with command line options.

## NVIDIA/SkillEvaluator

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/NVIDIA/SkillEvaluator
- Категория: benchmark/testing candidate
- Описание: Multi-tier framework for evaluating AI agent skills with quality gates, semantic overlap detection, synthetic evaluation dataset generation, and live agent evaluation that measures how skills affect agent behavior.
- Уровень: automated README evidence extraction
- Снимок: [sources/NVIDIA__SkillEvaluator/README.md](sources/NVIDIA__SkillEvaluator/README.md); SHA-256: `0cdc8377bec877267e774b56f134aacc79a6be304d813aa3cb34da1f34d3af8b`
- lifecycle: строка 58: ## LLM provider setup
- report: строка 20: ![SkillEvaluator three-tier pipeline: Skill → Tier 1 Validation → Tier 2 Deduplication → Tier 3 Live Evaluation → Reports](docs/assets/three-tier-overview.svg)
- correctness: строка 20: ![SkillEvaluator three-tier pipeline: Skill → Tier 1 Validation → Tier 2 Deduplication → Tier 3 Live Evaluation → Reports](docs/assets/three-tier-overview.svg)

## isri-aist/RoboManipBaselines

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/isri-aist/RoboManipBaselines
- Категория: benchmark/testing candidate
- Описание: A software framework integrating various imitation learning methods and benchmark environments for robotic manipulation
- Уровень: automated README evidence extraction
- Снимок: [sources/isri-aist__RoboManipBaselines/README.md](sources/isri-aist__RoboManipBaselines/README.md); SHA-256: `679daa5b7af78d28d16ffce750ab24f3ba1a0fed1d409b48e16c32d0802ac966`
- comparison: строка 2: <a href="https://isri-aist.github.io/RoboManipBaselines-ProjectPage">
- lifecycle: строка 75: - 🎮 [Multiple SpaceMouse](./doc/use_multiple_spacemouse.md): Setup multiple SpaceMouse for high-degree-of-freedom robots

## qdrant/vector-db-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/qdrant/vector-db-benchmark
- Категория: benchmark/testing candidate
- Описание: Framework for benchmarking vector search engines
- Уровень: automated README evidence extraction
- Снимок: [sources/qdrant__vector-db-benchmark/README.md](sources/qdrant__vector-db-benchmark/README.md); SHA-256: `f1568cb9008080d5af17ab93af401ef926260bede2b24a809677f5a703a455b2`
- lifecycle: строка 98: * `BaseConfigurator` - defines methods to create collections, setup indexing parameters.
- report: строка 91: Datasets are configured in the [datasets/datasets.json](./datasets/datasets.json) file.

## libnonius/nonius

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/libnonius/nonius
- Категория: benchmark/testing candidate
- Описание: A C++ micro-benchmarking framework
- Уровень: automated README evidence extraction
- Снимок: [sources/libnonius__nonius/README.md](sources/libnonius__nonius/README.md); SHA-256: `6fcb04732fa9220b898b91b66bcb6e9ee48a64e12df508c986fb01d807dd5e07`
- statistics: строка 8: statistical analysis on those measurements.

## nickbruun/hayai

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nickbruun/hayai
- Категория: benchmark/testing candidate
- Описание: C++ benchmarking framework
- Уровень: automated README evidence extraction
- Снимок: [sources/nickbruun__hayai/README.md](sources/nickbruun__hayai/README.md); SHA-256: `bd0548de8724f0672f33733f2a1f7a0c5a889d8eeec0024da52358539692602a`

## AlexeyZatsepin/Android-ORM-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/AlexeyZatsepin/Android-ORM-benchmark
- Категория: benchmark/testing candidate
- Описание: Performance comparison of Android ORM Frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/AlexeyZatsepin__Android-ORM-benchmark/README.md](sources/AlexeyZatsepin__Android-ORM-benchmark/README.md); SHA-256: `14582cc307aaca75248edc7cf8a15d99fd70f4b129d0dc2e318bc879610e2620`

## openstack/rally

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/openstack/rally
- Категория: benchmark/testing candidate
- Описание: Rally provides a framework for performance analysis and benchmarking of individual OpenStack components as well as full production OpenStack cloud deployments. Mirror of code maintained at opendev.org.
- Уровень: automated README evidence extraction
- Снимок: [sources/openstack__rally/README.rst](sources/openstack__rally/README.rst); SHA-256: `a92355e07e461a3209fae60efb1285827371b1bd60df1dc0cf0dc6a23c671e10`
- report: строка 73: task specs, persisting and reporting results.
- correctness: строка 35: capable to perform **specific**, **complicated** and **reproducible**

## chronoxor/CppBenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chronoxor/CppBenchmark
- Категория: benchmark/testing candidate
- Описание: Performance benchmark framework for C++ with nanoseconds measure precision
- Уровень: automated README evidence extraction
- Снимок: [sources/chronoxor__CppBenchmark/README.md](sources/chronoxor__CppBenchmark/README.md); SHA-256: `4e07919549a05bf477461a7a82cb27ec7cddc778360cf2df133f5c59253adf08`
- lifecycle: строка 86: ### Setup repository
- async: строка 16: average/minimal/maximal execution time, items processing processing speed, I/O throughput.
- report: строка 56: * Different reporting formats: console, csv, json

## remotebiosensing/rppg

- Итог: excluded — Physiological prediction quality evaluation
- Источник: https://github.com/remotebiosensing/rppg
- Категория: benchmark/testing candidate
- Описание: Benchmark Framework for fair evaluation of rPPG
- Уровень: automated README evidence extraction
- Снимок: [sources/remotebiosensing__rppg/README.md](sources/remotebiosensing__rppg/README.md); SHA-256: `d5dcde48ea3c7ef4a6fd729527a5de41f25e87d100fb3295d87730e5ff40429a`
- statistics: строка 84: | 2018 |   TR    |        LGI        |             O             |                               [paper](https://openaccess.thecvf.com/content_cvpr_2018_workshops/papers/w27/Pilz_Local_Group_Invariance_CVPR_2018_paper.pdf)                                |
- correctness: строка 137: - All evaluations are based on the model with the lowest loss value during validation.

## fperf/fperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fperf/fperf
- Категория: benchmark/testing candidate
- Описание: Framework of performance testing
- Уровень: automated README evidence extraction
- Снимок: [sources/fperf__fperf/README.md](sources/fperf__fperf/README.md); SHA-256: `0fdfef6b6bb79fb29e4cc2ef5d1012d5416af79f4b828ae8d8f9082fcf75fb94`
- async: строка 6: **You create the client and send requests, fperf do the concurrency and statistics, then give you a report about qps and latency.**
- report: строка 3: [![Go Report Card](https://goreportcard.com/badge/github.com/fperf/fperf)](https://goreportcard.com/report/github.com/fperf/fperf)

## MILVLG/openvqa

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/MILVLG/openvqa
- Категория: benchmark/testing candidate
- Описание: A lightweight, scalable, and general framework for visual question answering research
- Уровень: automated README evidence extraction
- Снимок: [sources/MILVLG__openvqa/README.md](sources/MILVLG__openvqa/README.md); SHA-256: `6fa386618f9837b60d453b4126e7a6d284f54eae905b0c401ba950f960ad6922`

## Project-AgML/AgML

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/Project-AgML/AgML
- Категория: benchmark/testing candidate
- Описание: AgML is a centralized framework for agricultural machine learning. AgML provides access to public agricultural datasets for common agricultural deep learning tasks, with standard benchmarks and pretrained models, as well the ability to generate synthetic data and annotations.
- Уровень: automated README evidence extraction
- Снимок: [sources/Project-AgML__AgML/README.md](sources/Project-AgML__AgML/README.md); SHA-256: `a69ec4b78257cfdd2f74630ea4dcb4e8610ad705e63a111dd2155981530e5a80`
- report: строка 167: - **Object Detection**: [COCO JSON](https://cocodataset.org/#format-data)
- correctness: строка 48: is not limited to, batching data, shuffling data, splitting data into training, validation, and test sets, and applying transforms.

## benchopt/benchopt

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/benchopt/benchopt
- Категория: benchmark/testing candidate
- Описание: A framework for reproducible, comparable benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/benchopt__benchopt/README.rst](sources/benchopt__benchopt/README.rst); SHA-256: `f539eda7a18ccdc08673ecfdbe8fd9f1c4ebad07b090be0c1e829f649f751e7f`
- comparison: строка 63: Here is how to do so for the `L2-logistic Regression benchmark <https://github.com/benchopt/benchmark_logreg_l2>`_.
- memory: строка 126: and Larsson, Johan and Lai, En and Lefort, Tanguy
- correctness: строка 5: *—A framework for reproducible, comparable benchmarks—*

## cavalab/srbench

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/cavalab/srbench
- Категория: benchmark/testing candidate
- Описание: A living benchmark framework for symbolic regression
- Уровень: automated README evidence extraction
- Снимок: [sources/cavalab__srbench/README.md](sources/cavalab__srbench/README.md); SHA-256: `fc8c90ad89bf3c4f0144a8d53be359192a946968cbf9e4d838503397a6894158`
- statistics: строка 42: Black-box regression (median and 95% confidence interval across datasets):
- comparison: строка 2: # SRBench: A Living Benchmark for Symbolic Regression
- lifecycle: строка 22: The current edition of the benchmark (SRBench 2025, reported in our [_call for action_ paper](#call-for-action)) evaluates **25** symbolic regression methods under a unified experimental setup: every method runs from a docker container, with hyperparameter tun
- report: строка 22: The current edition of the benchmark (SRBench 2025, reported in our [_call for action_ paper](#call-for-action)) evaluates **25** symbolic regression methods under a unified experimental setup: every method runs from a docker container, with hyperparameter tun

## OpenBMB/UltraEval-Audio

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/OpenBMB/UltraEval-Audio
- Категория: benchmark/testing candidate
- Описание: Your faithful, impartial partner for audio evaluation — know yourself, know your rivals. 真实评测，知己知彼。A unified benchmark framework for ASR/TTS/Audio Codec/audio LLM evaluation
- Уровень: automated README evidence extraction
- Снимок: [sources/OpenBMB__UltraEval-Audio/README.md](sources/OpenBMB__UltraEval-Audio/README.md); SHA-256: `f280f6eee97b759f04348285eaca81f0a65016c363d29d6ecdab07a61f2f4a1b`
- gpu: строка 56: - GPU parallel acceleration for faster evaluation/inference
- report: строка 60: - Support Step-Audio-R1.1 evaluation, with replication report: [Step-Audio-R1.1](replication/step-audio-r1_1.md)
- correctness: строка 27: - **Direct Replication of Popular Models 🔬**: Provides detailed [replication documentation and commands](./replication/), ensuring you can easily reproduce evaluation results of open-source models with complete transparency and reproducibility.

## bark-simulator/bark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bark-simulator/bark
- Категория: benchmark/testing candidate
- Описание: Open-Source Framework for Development, Simulation and Benchmarking of Behavior Planning Algorithms for Autonomous Driving
- Уровень: automated README evidence extraction
- Снимок: [sources/bark-simulator__bark/README.md](sources/bark-simulator__bark/README.md); SHA-256: `c08dcb20ae1d856564376018df328130d14d1adf7246d55c95010b5db62e03cf`
- memory: строка 13: ![CI RSS Build](https://github.com/bark-simulator/bark/workflows/CI_RSS/badge.svg)
- report: строка 15: [![Codacy Badge](https://app.codacy.com/project/badge/Grade/b9f484c42194487e9b9b33742381e992)](https://www.codacy.com/gh/bark-simulator/bark/dashboard?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=bark-simulator/bark&amp;utm_campaign=Badge_Grad
- correctness: строка 65: *   [BARK-DB](https://github.com/bark-simulator/bark-databasse/): Provides a framework to integrate multiple BARK scenario sets into a database. The database module supports binary serialization of randomly generated scenarios to ensure exact reproducibility o

## abeimler/ecs_benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/abeimler/ecs_benchmark
- Категория: benchmark/testing candidate
- Описание: Benchmarks of common ECS (Entity-Component-System)-Frameworks in C++ (or C)
- Уровень: automated README evidence extraction
- Снимок: [sources/abeimler__ecs_benchmark/README.md](sources/abeimler__ecs_benchmark/README.md); SHA-256: `4029a975a3600f5f29ce56b623ba6f4f86029c1698671b77f0e181e506d9fcca`
- lifecycle: строка 601: - [Dependency Setup](doc/README_dependencies.md)
- report: строка 536: Additionally, you can write tests for the framework example and add some metadata to the [plot.config.json](plot.config.json) file.

## android-bench/android-bench

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/android-bench/android-bench
- Категория: benchmark/testing candidate
- Описание: Android Bench is a framework for benchmarking Large Language Models (LLMs) on Android development tasks. It evaluates an AI model's ability to understand mobile codebases, generate accurate patches, and solve Android-specific engineering problems.
- Уровень: automated README evidence extraction
- Снимок: [sources/android-bench__android-bench/README.md](sources/android-bench__android-bench/README.md); SHA-256: `8b8645326f4861e24f8e5de43c8461d431fb2094ff8d5ecad462e5b964b410f1`
- comparison: строка 145: - [Technical Report](docs/tech_report.md): A deep-dive into the methodology, dataset construction, and baseline results.
- lifecycle: строка 12: - [Setup (Quickstart)](#setup)
- report: строка 43: 3.  Generates the `summary.json` for the dataset explorer.

## pathwaycom/pathway-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pathwaycom/pathway-benchmarks
- Категория: benchmark/testing candidate
- Описание: Benchmarks for data processing systems: Pathway Live Data Framework, Spark, Flink, Kafka Streams
- Уровень: automated README evidence extraction
- Снимок: [sources/pathwaycom__pathway-benchmarks/README.md](sources/pathwaycom__pathway-benchmarks/README.md); SHA-256: `18fe605a7ffbdcc62e432f49a6ee32c05dbaa0ba703268cee2cebc2318515994`
- lifecycle: строка 55: The Pathway Live Data Framework clearly outperforms the default Flink setup in terms of sustained throughput, and dominates the Flink minibatching setup in terms of latency for all of the throughput spectrum we could measure. For most throughputs, the Pathway
- memory: строка 39: Below we present the results of the benchmarks. For these results, all benchmarks were run on dedicated machines with: 12-core AMD Ryzen 9 5900X Processor, 128GB of RAM and SSD drives. For all multithreaded benchmarks we explicitly allocate cores to ensure tha
- async: строка 18: [Pathway Live Data Framework](https://www.pathway.com) is a reactive data processing framework designed for high-throughput and low-latency realtime data processing. The Pathway Live Data Framework's unified Rust engine processes code seamlessly in both batch
- report: строка 39: Below we present the results of the benchmarks. For these results, all benchmarks were run on dedicated machines with: 12-core AMD Ryzen 9 5900X Processor, 128GB of RAM and SSD drives. For all multithreaded benchmarks we explicitly allocate cores to ensure tha
- correctness: строка 24: The benchmarks are reproducible using the code in this repository. Find the instructions below under "Reproducing the benchmarks".

## chaoran/fast-wait-free-queue

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chaoran/fast-wait-free-queue
- Категория: benchmark/testing candidate
- Описание: A benchmark framework for concurrent queue implementations
- Уровень: automated README evidence extraction
- Снимок: [sources/chaoran__fast-wait-free-queue/README.md](sources/chaoran__fast-wait-free-queue/README.md); SHA-256: `817681a0cd732309dfe6f895dbc6505c49ef30c257bd30debf6ca96f051c744d`
- memory: строка 20: - **jemalloc** (optional): `jemalloc` eliminates the bottleneck of the memory allocator. You can link with `jemalloc` by setting `JEMALLOC_PATH` environment variable to the path where your `jemalloc` is installed.
- async: строка 3: This is a benchmark framework for evaluating the performance of concurrent queues. Currently, it contains four concurrent queue implementations. They are:
- report: строка 56: You can use the `benchmark` script, which invokes `driver` on all combinations of a list of binaries and a list of numbers of threads, and report the `mean running time` and `margin of error` for each combination. You can specify the list of binaries using the
- correctness: строка 105: - `void * benchmark(int id, int nprocs)`: run the benchmark once, called by each thread to run the benchmark. Each call will be timed and report as one iteration. It can return a result, which will be passed to `verify` to verify correctness.
- isolation: строка 18: - **glibc 2.3**: we use `sched_setaffinity` to bind threads to cores.

## numbbo/coco

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/numbbo/coco
- Категория: benchmark/testing candidate
- Описание: Numerical Black-Box Optimization Benchmarking Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/numbbo__coco/README.md](sources/numbbo__coco/README.md); SHA-256: `c5e2faf12cdb39283ef4718559c24e65aef295a519baf14061d50940601a5137`
- lifecycle: строка 52: - The [COCO experimental setup](http://numbbo.github.io/coco-doc/experimental-setup) description

## polyfractal/athletic

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/polyfractal/athletic
- Категория: benchmark/testing candidate
- Описание: PHP Benchmarking Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/polyfractal__athletic/readme.md](sources/polyfractal__athletic/readme.md); SHA-256: `6c5482cfd6ce23fd97b7bd95af94c38e99d5a20fab94051d356c68ae00c5c7f2`
- statistics: строка 167: | -b | --bootstrap | | Sets the path to an optional bootstrap file which is included before anything else.  This is often used to include an autoloader for your project. |
- lifecycle: строка 67: public function setUp()
- report: строка 189: The default formatter outputs the Event class name, each method name, the number of iterations, average time and operations per second.  More advanced formatters will be created in the near future (CSVFormatter, database export, advanced statistics, etc).

## seedifferently/the-great-web-framework-shootout

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/seedifferently/the-great-web-framework-shootout
- Категория: benchmark/testing candidate
- Описание: Benchmarks and test code for The Great Web Framework Shootout [DEPRECATED -- See techempower's benchmarks instead]
- Уровень: automated README evidence extraction
- Снимок: [sources/seedifferently__the-great-web-framework-shootout/README.rst](sources/seedifferently__the-great-web-framework-shootout/README.rst); SHA-256: `93d94b84ac25c7f8ed1f3a5f4e55d2fd8820ed8618d23370e3a994e3c1547e94`
- comparison: строка 160: Apache's market share I figured it would be a good baseline.)
- lifecycle: строка 149: Test Platform Setup

## sosy-lab/benchexec

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sosy-lab/benchexec
- Категория: benchmark/testing candidate
- Описание: BenchExec: A Framework for Reliable Benchmarking and Resource Measurement
- Уровень: automated README evidence extraction
- Снимок: [sources/sosy-lab__benchexec/README.md](sources/sosy-lab__benchexec/README.md); SHA-256: `5bcb84cc13a5b90f56409454d40a1cf53abf89ad11f67856177247c828c2ffd4`
- statistics: строка 85: and extract further statistical data from the output.
- comparison: строка 189: - [CPAchecker](https://cpachecker.sosy-lab.org), also for regression testing
- lifecycle: строка 29: > a brief explanation of the issues of common setups as well as the (few)
- memory: строка 79: It measures CPU time, wall time, and memory usage of a tool,
- report: строка 86: Results from multiple runs can be combined into CSV and interactive HTML tables,
- correctness: строка 34: that takes care of important low-level details for accurate, precise, and reproducible measurements
- isolation: строка 46: and isolation against other running processes

## arey/java-object-mapper-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/arey/java-object-mapper-benchmark
- Категория: benchmark/testing candidate
- Описание: JMH benchmark of Java object-to-object mapping frameworks
- Уровень: automated README evidence extraction
- Снимок: [sources/arey__java-object-mapper-benchmark/readme.md](sources/arey__java-object-mapper-benchmark/readme.md); SHA-256: `2e2c22f128915f2cfc321308bdd5ace45c4f39a58c452bf1d31fb8544441899b`
- async: строка 62: The benchmarks measure throughput, given in "ops/time". The time unit used is seconds.
- report: строка 96: 1. Run benchmark while exporting results to csv with `java @jvm.options -jar target/benchmarks.jar -rff results.csv -rf csv`

## reframe-hpc/reframe

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/reframe-hpc/reframe
- Категория: benchmark/testing candidate
- Описание: A powerful Python framework for writing and running portable regression tests and benchmarks for HPC systems.
- Уровень: automated README evidence extraction
- Снимок: [sources/reframe-hpc__reframe/README.md](sources/reframe-hpc__reframe/README.md); SHA-256: `ece2d6043447d6b368df80807e9d5ce36608478a3a860e88f82eebb49c360d6d`
- comparison: строка 23: ReFrame is a powerful framework for writing system regression tests and benchmarks, specifically targeted to HPC systems.
- lifecycle: строка 24: The goal of the framework is to abstract away the complexity of the interactions with the system, separating the logic of a test from the low-level details, which pertain to the system configuration and setup.
- gpu: строка 16: [![Slack](https://badgen.net/badge/icon/slack?icon=slack&label)](https://join.slack.com/t/reframetalk/shared_invite/zt-3706f0tj6-2CjHh07HdQNbmLw1qAasjg)

## a-r-j/ProteinWorkshop

- Итог: excluded — Protein representation learning benchmark
- Источник: https://github.com/a-r-j/ProteinWorkshop
- Категория: benchmark/testing candidate
- Описание: Benchmarking framework for protein representation learning. Includes a large number of pre-training and downstream task datasets, models and training/task utilities. (ICLR 2024)
- Уровень: automated README evidence extraction
- Снимок: [sources/a-r-j__ProteinWorkshop/README.md](sources/a-r-j__ProteinWorkshop/README.md); SHA-256: `32d770b48793ce1c7f2b38c7c6cdb9b5cc725678130749883edbc634ea5a9f08`
- comparison: строка 217: # reproduce the baseline tasks sweep (i.e., those performed without pre-training each model)
- lifecycle: строка 133: Datasets can either be built from the source structures or downloaded from [Zenodo](https://zenodo.org/record/8282470). Datasets will be built from source the first time a dataset is used in a run (or by calling the appropriate `setup()` method in the correspo
- gpu: строка 153: Launching an experiment minimally requires specification of a dataset, structural encoder, and task (devices can be specified with `trainer=cpu/gpu`):
- correctness: строка 272: python proteinworkshop/validate_config.py dataset=cath features=full_atom task=inverse_folding

## ARM-software/Tool-Solutions

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ARM-software/Tool-Solutions
- Категория: benchmark/testing candidate
- Описание: Scripts to build a wheel and a Docker image containing a complete ML framework stack, including dependencies, for AArch64 CPUs, as well as a selection of examples and benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/ARM-software__Tool-Solutions/README.md](sources/ARM-software__Tool-Solutions/README.md); SHA-256: `0a906d2dbe9cba251c816551290ca7b20dde9963880be59e4daf5d8404f0c08d`

## sheredom/ubench.h

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sheredom/ubench.h
- Категория: benchmark/testing candidate
- Описание: ⏱️ single header benchmark framework for C and C++
- Уровень: automated README evidence extraction
- Снимок: [sources/sheredom__ubench.h/README.md](sources/sheredom__ubench.h/README.md); SHA-256: `11c0558a83eedb7fff0c1d55550c947bd8f0fb622f80d366fe747a0a171e3866`
- statistics: строка 40: [       OK ] foo.bar (mean 536.235us, confidence interval +- 1.457878%)
- lifecycle: строка 111: ## Define a Benchmark with setup
- report: строка 27: * `--output=<output>`will output a CSV file of the results.
- correctness: строка 58: that is sufficiently low to enable reproducible results.

## efficientgo/e2e

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/efficientgo/e2e
- Категория: benchmark/testing candidate
- Описание: Robust framework for running complex workload scenarios in isolation, using Go; for integration, e2e tests, benchmarks and more! 💪
- Уровень: automated README evidence extraction
- Снимок: [sources/efficientgo__e2e/README.md](sources/efficientgo__e2e/README.md); SHA-256: `c49a9da81da0e50b25394d4d1b0417e466c56b75d0e5e5869b147442d8b75f96`
- lifecycle: строка 19: * *Standalone use* ([see example](examples/thanos/standalone.go)). Use `e2e` to run setups in interactive mode where you spin up workloads as you want *programmatically* and poke with it on your own using your browser or other tools. No longer need to deploy f
- memory: строка 112: This will start Prometheus with automatic discovery for every new and old instrumented runnables. It also runs cadvisor that monitors docker itself if `env.DockerEnvironment` is started and shows generic performance metrics per container (e.g `container_memory
- isolation: строка 5: Go Module providing robust framework for running complex workload scenarios in isolation, using Go and Docker. For integration, e2e tests, benchmarks and more! 💪

## gormanm/mmtests

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gormanm/mmtests
- Категория: benchmark/testing candidate
- Описание: MMTests: Benchmarking framework primarily aimed at Linux kernel testing
- Уровень: automated README evidence extraction
- Снимок: [sources/gormanm__mmtests/README.md](sources/gormanm__mmtests/README.md); SHA-256: `87ff52e201b6f00ac47506b80b200e27426ff81236c39fa2bb8cb1b3db48c129`
- statistics: строка 50: attempting to highlight whether performance differences are statistically
- comparison: строка 84: It accepts several values. `none`, `numad` or `interleave`, are
- gpu: строка 441: highlighting their impact on both baremetal and virtualization workloads,
- async: строка 230: * `MONITORS_WITH_LATENCY`:
- report: строка 6: representative and reproducible. Reporting and analysis is common across
- correctness: строка 6: representative and reproducible. Reporting and analysis is common across

## alirezazareian/ovr-cnn

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/alirezazareian/ovr-cnn
- Категория: benchmark/testing candidate
- Описание: A new framework for open-vocabulary object detection, based on maskrcnn-benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/alirezazareian__ovr-cnn/README.md](sources/alirezazareian__ovr-cnn/README.md); SHA-256: `727e4598be7f5d8f7123bf76de0b06528d9dc80a533c509b04a85b77f205e1c4`
- report: строка 26: For the zero-shot experiment to work, you need to first create a new annotation json using [this notebook](ipynb/003.ipynb). Then run:

## stalkermustang/llm-bulls-and-cows-benchmark

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/stalkermustang/llm-bulls-and-cows-benchmark
- Категория: benchmark/testing candidate
- Описание: A mini-framework for evaluating LLM performance on the Bulls and Cows number guessing game, supporting multiple LLM providers.
- Уровень: automated README evidence extraction
- Снимок: [sources/stalkermustang__llm-bulls-and-cows-benchmark/README.md](sources/stalkermustang__llm-bulls-and-cows-benchmark/README.md); SHA-256: `45482dfa7883325ea44e2aa8b550224ed9a4c0170a614845496e4d0c6d4f72ed`
- statistics: строка 41: > For most of the runs, 50 games were played (excl. o1-mini), thus, Confidence Intervals are wide. If you'd like to spend $100-200 in API credits on tests to achieve more accurate results and make CIs narrower, please feel free to reach me or open a PR with yo
- memory: строка 51: - Even small and cheap models handle response formatting well (e.g., Gemini Flash <1% Format Failures, although Google models tend to add a newline after a guess — `.strip()` was added to address this).
- async: строка 14: - ✅ **Rich Progress Bars**: Don't be bored while running the benchmark: all intermediate results for all concurrent games will be displayed, with live metric updates!
- report: строка 48: - Structured Outputs/JSON-mode are intentionally avoided for two reasons:
- correctness: строка 151: - YAML validation

## neurosim/MLP_NeuroSim_V3.0

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/neurosim/MLP_NeuroSim_V3.0
- Категория: benchmark/testing candidate
- Описание: Benchmark framework of synaptic device technologies for a simple neural network
- Уровень: automated README evidence extraction
- Снимок: [sources/neurosim__MLP_NeuroSim_V3.0/README.md](sources/neurosim__MLP_NeuroSim_V3.0/README.md); SHA-256: `386206fc21e501d1b6e7b923061804ef37a0d1c965a5a162e4dcc368fc620558`
- lifecycle: строка 48: 2. Calibrate FinFET technology library (<20nm)
- async: строка 5: This is the released version 3.0 (Mar. 1st, 2019) for the tool. This version extends the algoritihm weights from (0,1) in V2.0 to (-1,1) in V3.0. Besides, more optimization methods such as momentum method, Adagrad, RMSprop, Adam are added. The digital eNVMs (e

## codefuse-ai/OpAgent

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/codefuse-ai/OpAgent
- Категория: benchmark/testing candidate
- Описание: The OpAgent framework is used to automate web browser operations, achieving State-of-the-Art (SOTA) performance on the WebArena benchmark.
- Уровень: automated README evidence extraction
- Снимок: [sources/codefuse-ai__OpAgent/README.md](sources/codefuse-ai__OpAgent/README.md); SHA-256: `60f162bb7616b5d4811a9670dd68258ab5243fcf62526796fbb000cca6d0c56b`
- comparison: строка 75: We employ an innovative **Online Agentic Reinforcement Learning (RL)** pipeline to significantly improve the capability of a single VLM. Our RL-enhanced model (`RL-HybridReward-Zero`) achieves a **38.1%** success rate (@Pass5) on WebArena, outperforming other
- lifecycle: строка 68: *   Designed for researchers and engineers who want to reproduce, extend, or adapt our training workflow to their own environments and agent setups.
- gpu: строка 92: #### Demo — INT4 Quantized Model on a  24 GB GPU (headed browser)
- report: строка 48: 📄📄📄 **[2026/02/14]** We have released our technical report. Please refer to [OpAgent Technical Report](https://arxiv.org/pdf/2602.13559) for details.

## google/benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/google/benchmark
- Категория: benchmark/testing candidate
- Описание: A microbenchmark support library
- Уровень: automated README evidence extraction
- Снимок: [sources/google__benchmark/README.md](sources/google__benchmark/README.md); SHA-256: `b6f45a8909df4e7a8e01760917253c317387ab2c65b82b53ec561b88f2589adc`
- lifecycle: строка 18: // Perform setup here

## evanwashere/mitata

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/evanwashere/mitata
- Категория: benchmark/testing candidate
- Описание: benchmark tooling that loves you ❤️
- Уровень: automated README evidence extraction
- Снимок: [sources/evanwashere__mitata/readme.md](sources/evanwashere__mitata/readme.md); SHA-256: `1a3c225d96635536b70d4ac5ae971408c855e3df86e6943ead4e9811a3addd7a`
- statistics: строка 210: *(note: concurrent benchmarks may have higher variance due to scheduling, contention, event loop and async overhead)*
- lifecycle: строка 106: By default, on runtimes with exposed manual gc (like bun or node with `--expose-gc`), mitata runs garbage collection once after each benchmark warmup.
- memory: строка 111: bench('lots of allocations', () => {
- async: строка 203: ### concurrency
- report: строка 95: await run({ format: 'json' }) // output json

## martinus/nanobench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/martinus/nanobench
- Категория: benchmark/testing candidate
- Описание: Simple, fast, accurate single-header microbenchmarking functionality for C++11/14/17/20
- Уровень: automated README evidence extraction
- Снимок: [sources/martinus__nanobench/README.md](sources/martinus__nanobench/README.md); SHA-256: `a0a22748bdaca9bd3040c513d161b62a5c70c016e9f02be4c9e09f276642bee5`
- statistics: строка 53: * **Robust**: Be robust against outliers, warn if results are not reliable.

## attaswift/Attabench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/attaswift/Attabench
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking app for Swift with nice log-log plots
- Уровень: automated README evidence extraction
- Снимок: [sources/attaswift__Attabench/README.md](sources/attaswift__Attabench/README.md); SHA-256: `b174e1e6758ba54fe054906c837560d21f995a8027690cb4d3e8588c29a7861d`
- statistics: строка 67: carthage bootstrap --platform Mac
- comparison: строка 352: Attabench runs `run.sh` with two parameters: the first is the constant string `attabench`, identifying the protocol version, and the second is a path to a named FIFO file that will serve as the report channel for the benchmark. (Benchmarking progress is not wr
- lifecycle: строка 311: 2. After the initial warmup, the cost of looking up an element using
- memory: строка 124: - *Maximum* displays the slowest measurement only. This is probably not that useful on its own, but it was really cheap to implement! (And it can be interesting to combine it with the stddev-based error bands.)
- report: строка 35: Attabench was originally created to supply nice log-log charts for my [dotSwift 2017 talk][dotswift] and [Optimizing Collections][oc] book. At the time, it seemed easier to build a custom chart renderer from scratch using Core Graphics than to mess with a bunc

## atemerev/skynet

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/atemerev/skynet
- Категория: benchmark/testing candidate
- Описание: Skynet 1M threads microbenchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/atemerev__skynet/README.md](sources/atemerev__skynet/README.md); SHA-256: `654bce5a3728540ce458f1d3547e5a8def724b3cbe1d1bcb8d20ffd33f9e4789`
- lifecycle: строка 44: - Node-bluebird (Promise) 285ms / 195ms (after warmup)
- async: строка 1: # Skynet 1M concurrency microbenchmark

## sbt/sbt-jmh

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sbt/sbt-jmh
- Категория: benchmark/testing candidate
- Описание: "Trust no one, bench everything." - sbt plugin for JMH (Java Microbenchmark Harness)
- Уровень: automated README evidence extraction
- Снимок: [sources/sbt__sbt-jmh/README.md](sources/sbt__sbt-jmh/README.md); SHA-256: `211e97ea47e86b5aedcbde376fcca58e0e592fe9c5ab3d31eaeb4029977e1ffd`
- statistics: строка 138: [info]   Confidence interval (99.9%): [95470.135, 105420.532]
- comparison: строка 103: For example it's possible to keep the benchmark's results as csv or json files for later regression analysis.
- lifecycle: строка 75: Which means "3 iterations" "3 warmup iterations" "1 fork" "1 thread". Please note that benchmarks should be usually executed at least in 10 iterations (as a rule of thumb), but more is better.
- async: строка 25: | [`0.4.2`](https://github.com/ktoso/sbt-jmh/releases/tag/v0.4.2)  (sbt 1.3.0+)              | [`1.31`](https://github.com/openjdk/jmh/releases/tag/1.31) | JMH `-prof async` supports 2.x  |
- report: строка 103: For example it's possible to keep the benchmark's results as csv or json files for later regression analysis.

## andreas-abel/nanoBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/andreas-abel/nanoBench
- Категория: benchmark/testing candidate
- Описание: A tool for running small microbenchmarks on recent Intel and AMD x86 CPUs.
- Уровень: automated README evidence extraction
- Снимок: [sources/andreas-abel__nanoBench/README.md](sources/andreas-abel__nanoBench/README.md); SHA-256: `c3de2528516b21ae1e585699708d2c11bc16be0909ab60ff24af83b0aff123d5`
- statistics: строка 145: | `-warm_up_count <n>`         | Number of runs of the generated benchmark code sequence (in each invocation of `run(...)`) before the first measurement result gets recorded . This can, for example, be useful for excluding outliers due to cold caches.   `[Defa
- lifecycle: строка 102: for i=-warm_up_count to n_measurements
- async: строка 8: *nanoBench* is used for running the microbenchmarks for obtaining the latency, throughput, and port usage data that is available on [uops.info](http:www.uops.info).
- report: строка 122: The result that is finally reported by *nanoBench* is the difference between these two executions divided by `max(loop_count * unroll_count, unroll_count)`.

## alco/benchfella

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alco/benchfella
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking tool for Elixir
- Уровень: automated README evidence extraction
- Снимок: [sources/alco__benchfella/README.md](sources/alco__benchfella/README.md); SHA-256: `fcc34bf477d3173c67dd9d30d99d999e770276b8230be7d92716df95d2fbd542`
- lifecycle: строка 92: ### `setup_all` and `teardown_all`

## scalameter/scalameter

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/scalameter/scalameter
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking and performance regression testing framework for the JVM platform.
- Уровень: automated README evidence extraction
- Снимок: [sources/scalameter__scalameter/README.md](sources/scalameter__scalameter/README.md); SHA-256: `65d29275e90fd215e37a1b81bb25a656367e5af77287034c49e0f02db054e396`
- comparison: строка 11: Microbenchmarking and performance regression testing framework for the JVM platform.
- report: строка 13: and then produce nice reports, or store your data.

## p-ranav/criterion

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/p-ranav/criterion
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking for Modern C++
- Уровень: automated README evidence extraction
- Снимок: [sources/p-ranav__criterion/README.md](sources/p-ranav__criterion/README.md); SHA-256: `01c1665e22d4ec73740bfecaabc2b20dfd5363efa64ce5c7a1ad436c99d7befa`
- statistics: строка 15: * Statistical analysis across multiple runs
- lifecycle: строка 54: * Use `SETUP_BENCHMARK` and `TEARDOWN_BENCHMARK` to perform setup and teardown tasks
- report: строка 27: *    [Exporting Results (csv, json etc.)](#exporting-results-csv-json-etc)

## iboB/picobench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/iboB/picobench
- Категория: benchmark/testing candidate
- Описание: A micro microbenchmarking library for C++11 in a single header file
- Уровень: automated README evidence extraction
- Снимок: [sources/iboB__picobench/README.md](sources/iboB__picobench/README.md); SHA-256: `a7e6b359ec2fd39eef84f3616c7d646ecb27f30028d4f918cfc509c16f7c22d4`
- comparison: строка 49: Name (* = baseline)      |   Dim   |  Total ms |  ns/op  |Baseline| Ops/second
- memory: строка 166: Sometimes the code being benchmarked is very sensitive to external factors such as syscalls (which include memory allocation and deallocation). Those external factors can have take greatly different times between runs. In such cases several samples of a benchm
- report: строка 75: The library will run the benchmark function several times with different numbers of iterations, to simulate different problem spaces, then collect the results in a report.

## clamchowder/Microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/clamchowder/Microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Trying to figure various CPU things out
- Уровень: automated README evidence extraction
- Снимок: [sources/clamchowder__Microbenchmarks/README.md](sources/clamchowder__Microbenchmarks/README.md); SHA-256: `c217ac2d1e40c03a0405f62f2a3ffeb8728fb4c900df9285f62566bc4b6ce7ec`
- gpu: строка 2: Trying to figure various CPU (or GPU) things out.
- async: строка 4: Basically my playground to microbenchmark various CPU-related things like ROB/register file sizes, lock/cache coherency latency, and cache/memory performance. This repo is loose collection of various experiments and is more of a playground than a well maintain

## clamchowder/MicrobenchmarksGui

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/clamchowder/MicrobenchmarksGui
- Категория: benchmark/testing candidate
- Описание: An attempt to make a more accessible microbenchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/clamchowder__MicrobenchmarksGui/README.md](sources/clamchowder__MicrobenchmarksGui/README.md); SHA-256: `ed86a4e9a2d6bd1973b52699ab1fa450c2e2edf7b9355188020ce1ec890acffa`
- memory: строка 58: Then if you select Large Pages under Paging Mode, the test will allocate 1 GB (the largest test size for mem latency) upfront and run all test sizes within that. That means you need to have 1 GB of contiguous memory free. If you have a system without much memo
- async: строка 6: Unlike another well known cache and memory benchmark that's spelled AIDA, this aims to be a free and more advanced tool. It runs through a lot of tests sizes designed to cover most cache capacities. Then, you can look through the results to determine bandwidth

## Ichoran/thyme

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Ichoran/thyme
- Категория: benchmark/testing candidate
- Описание: Thyme is a microbenchmark utility for Scala.  It includes Parsley, a (simple) local profiling tool.
- Уровень: automated README evidence extraction
- Снимок: [sources/Ichoran__thyme/README.md](sources/Ichoran__thyme/README.md); SHA-256: `a8448e666a44a1fcf752674b5b0860b7dd98ff2bd507986ee5fb44f4d887265e`
- lifecycle: строка 92: scala> // If you don't want to wait for warmup...

## bazhenov/tango

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bazhenov/tango
- Категория: benchmark/testing candidate
- Описание: Rust microbenchmarking harness based on paired-testing methodology
- Уровень: automated README evidence extraction
- Снимок: [sources/bazhenov__tango/README.md](sources/bazhenov__tango/README.md); SHA-256: `239d31686a4193ae087dfbb81107998ea7e393ed0cece915e380a19861e07b34`
- statistics: строка 25: Compared to traditional pointwise benchmarking, paired benchmarking is significantly more sensitive to changes. This heightened sensitivity enables the early detection of statistically significant performance variations.
- comparison: строка 12: Introducing Tango.rs, a novel benchmarking framework that employs [paired benchmarking](https://www.bazhenov.me/posts/paired-benchmarking/) to assess code performance. This approach capitalizes on the fact that it's far more efficient to measure the performanc
- lifecycle: строка 100: To use Tango.rs in an asynchronous setup, follow these steps:
- async: строка 20: - async support using tokio.rs;
- report: строка 161: - `-d [path]` – dump CSV with raw samples in a given directory

## tylertreat/go-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tylertreat/go-benchmarks
- Категория: benchmark/testing candidate
- Описание: A few miscellaneous Go microbenchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/tylertreat__go-benchmarks/README.md](sources/tylertreat__go-benchmarks/README.md); SHA-256: `5e674dd9667a6ec5da21049d84223b29ec31496202d81c1241023cfb6f5c51bd`

## aayasin/perf-tools

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/aayasin/perf-tools
- Категория: benchmark/testing candidate
- Описание: A collection of performance analysis tools, recipes, handy scripts, microbenchmarks & more
- Уровень: automated README evidence extraction
- Снимок: [sources/aayasin__perf-tools/README.md](sources/aayasin__perf-tools/README.md); SHA-256: `23aafbd731c42a7f76db489c9fc3053e3aa4d30cb127c345a4ef728b996e80ac`
- lifecycle: строка 49: * to setup the perf tool, invoke `./do.py setup-perf`
- report: строка 64: * **advanced sampling** steps: deeper profiling using advanced capabilities of the PMU, and output certain reports
- isolation: строка 110: * **n-copies** -- invokes N-copies of an app, with CPU affinity (uses sibling thread N=2, 1 thread/core when N <= nproc)

## sjfeng1999/gpu-arch-microbenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sjfeng1999/gpu-arch-microbenchmark
- Категория: benchmark/testing candidate
- Описание: Dissecting NVIDIA GPU Architecture
- Уровень: automated README evidence extraction
- Снимок: [sources/sjfeng1999__gpu-arch-microbenchmark/README.md](sources/sjfeng1999__gpu-arch-microbenchmark/README.md); SHA-256: `cf67d9f2737d9d103370d22148e4b4791d8a9da010d14fc523a8a84140c34d65`
- lifecycle: строка 8: > `python setup.py install`
- gpu: строка 1: # GPU Arch Microbenchmark
- async: строка 14: 4. `./(memory_latency|reg_bankconflict|...)`

## svanoort/python-client-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/svanoort/python-client-benchmarks
- Категория: benchmark/testing candidate
- Описание: Microbenchmark of different python HTTP clients
- Уровень: automated README evidence extraction
- Снимок: [sources/svanoort__python-client-benchmarks/README.md](sources/svanoort__python-client-benchmarks/README.md); SHA-256: `eaf72d90df72a5ef3642fdc6f79f5a9d5f4ccb378c95f0a1624dce017256cb90`
- lifecycle: строка 45: # Benchmark Setup
- async: строка 36: PyCurl is much faster than Requests (or other HTTP client libraries), generally completing smaller requests 2-3x as fast, and requires 3-10x less CPU time.  This is most visible with connection creation for small requests; given large enough requests (above 10
- report: строка 20: * /bigger - returns a fixed ~585B JSON response

## ChipsandCheese/Microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ChipsandCheese/Microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Trying to figure various CPU things out
- Уровень: automated README evidence extraction
- Снимок: [sources/ChipsandCheese__Microbenchmarks/README.md](sources/ChipsandCheese__Microbenchmarks/README.md); SHA-256: `e99837401fdd0bc1c3a252f8e6fcd2d6dc956a553e3a0fbbd0c9cd18f3c24eb2`
- async: строка 4: Basically my playground to microbenchmark various CPU-related things like ROB/register file sizes, lock/cache coherency latency, and cache/memory performance.

## joshuaulrich/microbenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/joshuaulrich/microbenchmark
- Категория: benchmark/testing candidate
- Описание: Infrastructure to accurately measure and compare the execution time of R expressions
- Уровень: automated README evidence extraction
- Снимок: [sources/joshuaulrich__microbenchmark/README.md](sources/joshuaulrich__microbenchmark/README.md); SHA-256: `9f11b4b9d60ed315f6e99e5d6bdf5849a91139c6ac1b422fc378325760dfdb26`

## JuliaLang/Microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/JuliaLang/Microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks comparing the Julia Programming language with other languages
- Уровень: automated README evidence extraction
- Снимок: [sources/JuliaLang__Microbenchmarks/README.md](sources/JuliaLang__Microbenchmarks/README.md); SHA-256: `fabc13372c3716ead80bbef8db966f42da3f66c3ed5719299afbdbeae7cd519b`

## mp911de/microbenchmark-runner

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mp911de/microbenchmark-runner
- Категория: benchmark/testing candidate
- Описание: JUnit extensions to launch JMH benchmarks from your IDE during development
- Уровень: automated README evidence extraction
- Снимок: [sources/mp911de__microbenchmark-runner/README.md](sources/mp911de__microbenchmark-runner/README.md); SHA-256: `1618a80368f4a1a11791dc76dc99ca87ff61544c871c3d3ffc28d260610f918c`
- lifecycle: строка 117: * `warmupIterations` (`integer`, defaults to `-1`) Global override of warmup iterations. Uses `@Warmup` or JMH defaults if set to `-1`
- report: строка 116: * `benchmarkReportDir` (`File`, defaults to `none`) Writes JMH benchmark results to this directory.

## Dr-Noob/peakperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Dr-Noob/peakperf
- Категория: benchmark/testing candidate
- Описание: Achieve peak performance on x86 CPUs and NVIDIA GPUs
- Уровень: automated README evidence extraction
- Снимок: [sources/Dr-Noob__peakperf/README.md](sources/Dr-Noob__peakperf/README.md); SHA-256: `816984883fc3c3d6814dac07fca2f2969d5bcac143de993eee50b63046a38cb4`
- lifecycle: строка 132: * - warm-up, not included in average
- gpu: строка 15: - [2.2 Enabling and disabling support for CPU/GPU](#22-enabling-and-disabling-support-for-cpugpu)
- async: строка 267: This means CPU benchmarks can achieve peak performance by simply matching the number of independent operations to the FMA latency (e.g., 4 independent FMA chains for 4-cycle latency).
- report: строка 79: -- peakperf build report:
- correctness: строка 403: - References are not provided because NVIDIA does not officially publish FP32 FMA instruction latencies in their documentation. However, these latencies are experimentally validated using peakperf, since these are the latencies used by the microbenchmark (see

## frappe/caffeine

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/frappe/caffeine
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks and load testing for Frappe Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/frappe__caffeine/README.md](sources/frappe__caffeine/README.md); SHA-256: `f5cfd423dcd5b36a360193951d9fbe30a8d1e18e56da96545972a07ecc137a48`
- statistics: строка 32: - `pyperf compare_to` compares two results and applies statistical significance tests.
- lifecycle: строка 37: Your local setup might not be fit for benchmarking. Follow these steps before running benchmarks:
- async: строка 77: # Run the load test with 100 virtual concurrent users
- report: строка 31: - `-o output.json` can be used to store detailed results for analysis later.

## ucb-bar/ccbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ucb-bar/ccbench
- Категория: benchmark/testing candidate
- Описание: Memory System Microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/ucb-bar__ccbench/README.md](sources/ucb-bar__ccbench/README.md); SHA-256: `cd728f090e2ef3890e35e96447d97135250b12f112906631c5f7108c029098ae`
- comparison: строка 159: - mem_interleaving- preliminary attempt to measure best interleaving of
- async: строка 145: - cache2cache  - cache-to-cache latency, bandwidth (ping pong arrays
- report: строка 50: other set types). Data is written to the ./report/report.txt file (actually, a

## stanford-mast/iBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/stanford-mast/iBench
- Категория: benchmark/testing candidate
- Описание: Suite of contentious microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/stanford-mast__iBench/README.md](sources/stanford-mast__iBench/README.md); SHA-256: `bfe9ab1ad307eda77faf32d34e6b287eb524bf2e8a2b7087031cfee71879af50`

## ibireme/yybench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ibireme/yybench
- Категория: benchmark/testing candidate
- Описание: A microbenchmark library for C
- Уровень: automated README evidence extraction
- Снимок: [sources/ibireme__yybench/README.md](sources/ibireme__yybench/README.md); SHA-256: `c76de9f167e50f8a8f64eca6085193c84c3768fa9d03ef2e39fc02886783582e`

## forresti/osu-micro-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/forresti/osu-micro-benchmarks
- Категория: benchmark/testing candidate
- Описание: MPI Microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/forresti__osu-micro-benchmarks/README](sources/forresti__osu-micro-benchmarks/README); SHA-256: `3da52e08fc92d733391910ffb7cfdbe1fa1da92e8a82b0a23aedf5bbad923856`
- comparison: строка 113: * (the window size) back-to-back to the paired receiving process before
- lifecycle: строка 161: * "-x" can be used to set the number of warmup iterations to skip for each
- memory: строка 166: default the benchmarks are limited to 512MB allocations.
- gpu: строка 190: In addition to support for communications to and from GPU memories allocated
- async: строка 71: osu_latency - Latency Test
- report: строка 88: * latency numbers are reported. This test is available here.
- isolation: строка 782: Setting GPU affinity

## Vitorian/hft-challenges

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Vitorian/hft-challenges
- Категория: benchmark/testing candidate
- Описание: C++ microbenchmark challenges for HFT University
- Уровень: automated README evidence extraction
- Снимок: [sources/Vitorian__hft-challenges/README.md](sources/Vitorian__hft-challenges/README.md); SHA-256: `5c16d383b492d466d108257a5fd93bedba68db3239e615a36970f78bade506ef`
- lifecycle: строка 15: 3. **Set up** your repo at [hftuniversity.com/challenges/setup](https://hftuniversity.com/challenges/setup)

## VerticalResearchGroup/microbench

- Итог: excluded — README too thin for content analysis
- Источник: https://github.com/VerticalResearchGroup/microbench
- Категория: benchmark/testing candidate
- Описание: Extremely Simple Microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/VerticalResearchGroup__microbench/README.md](sources/VerticalResearchGroup__microbench/README.md); SHA-256: `6085e23eac3dcd257ce70ccd5cf2231bba381c3a9ddf5539d4b1a730b8e1c878`

## openjdk/jmh-jdk-microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/openjdk/jmh-jdk-microbenchmarks
- Категория: benchmark/testing candidate
- Описание: https://openjdk.org/projects/code-tools/jmh-jdk-microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/openjdk__jmh-jdk-microbenchmarks/README.md](sources/openjdk__jmh-jdk-microbenchmarks/README.md); SHA-256: `122d85818ec8479c95e1205471a32e938e667a26fb1bfd779b747c1e890acd43`
- comparison: строка 67: * `SingleJavacBenchmark` (which is parametrized) measures each single javac compilation stage in an isolated run. This benchmark is designed for exact automated performance regression testing and it takes several hours to execute completely.

## ldbc/lsqb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ldbc/lsqb
- Категория: benchmark/testing candidate
- Описание: Labelled Subgraph Query Benchmark – A lightweight benchmark suite focusing on subgraph matching queries. Note: This is a microbenchmark for system developers and not an official LDBC benchmark.
- Уровень: automated README evidence extraction
- Снимок: [sources/ldbc__lsqb/README.md](sources/ldbc__lsqb/README.md); SHA-256: `498b71d8b6d9d352f236944f4b9cd44fff8172378abe55ddd4f6855b3d3edbf7`
- lifecycle: строка 131: * First and foremost, this benchmark is designed to be *simple*. In the spirit of this, we do not provide auditing guidelines – it's the user's responsibility to ensure that the benchmark setup is meaningful. We do not provide a common Java/Python driver compo
- correctness: строка 118: ## Validation of results

## Sayi/jmh-visual-chart

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Sayi/jmh-visual-chart
- Категория: benchmark/testing candidate
- Описание: :bar_chart: A visual chart for Java MicroBenchmark Harness.
- Уровень: automated README evidence extraction
- Снимок: [sources/Sayi__jmh-visual-chart/README.md](sources/Sayi__jmh-visual-chart/README.md); SHA-256: `4ed8a6c04a2f86c50e3ef6564b59067f7e034979fe93d8cb127da5e0364ded05`
- lifecycle: строка 15: @Warmup(iterations = 1)
- async: строка 11: @BenchmarkMode(Mode.Throughput)
- report: строка 43: .result("result.json")

## hax/my-benchmark

- Итог: excluded — README too thin for content analysis
- Источник: https://github.com/hax/my-benchmark
- Категория: benchmark/testing candidate
- Описание: A JavaScript microbenchmark support library
- Уровень: automated README evidence extraction
- Снимок: [sources/hax__my-benchmark/README.md](sources/hax__my-benchmark/README.md); SHA-256: `0bbf1840d3cb4391bb47f5881b507d5dae0031d917d0cf2bd769ed3c0332e9c0`

## junkdog/entity-system-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/junkdog/entity-system-benchmarks
- Категория: benchmark/testing candidate
- Описание: microbenchmarks comparing ECS (entity component system) frameworks for java
- Уровень: automated README evidence extraction
- Снимок: [sources/junkdog__entity-system-benchmarks/README.md](sources/junkdog__entity-system-benchmarks/README.md); SHA-256: `016fb9c88899c20513bd509cfe4c3d227da91ae6b70924b813a58ec90b256faa`
- comparison: строка 25: - **baseline:** position isn't updated, not included in charts.
- async: строка 12: - All benchmarks measure throughput; higher score is better.

## shoaibkamil/stencilprobe

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/shoaibkamil/stencilprobe
- Категория: benchmark/testing candidate
- Описание: Stencil Probe - a stencil microbenchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/shoaibkamil__stencilprobe/README.md](sources/shoaibkamil__stencilprobe/README.md); SHA-256: `051bcdbce0d1ad13b4c54051e0084b3be6593897d9fe4903ab75878d9876288d`

## SFU-HiAccel/uBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SFU-HiAccel/uBench
- Категория: benchmark/testing candidate
- Описание: [FPGA'21] Microbenchmarks for Demystifying the Memory System of Modern Datacenter FPGAs for Software Programmers
- Уровень: automated README evidence extraction
- Снимок: [sources/SFU-HiAccel__uBench/README.md](sources/SFU-HiAccel__uBench/README.md); SHA-256: `8f86f841e4303eb980307b8ee83c7fc8fd290f3546d8be61889a6bccad6babdd`
- lifecycle: строка 12: ## Environmental Setup
- gpu: строка 55: The KNN algorithm is widely used in many computational demanding applications including image classification, similarity search, and big-data query search. For this case study, we apply a series of HLS optimization techniques and demonstrate the pratical usage
- async: строка 3: uBench is a set of HLS-based microbenchmarks to quantitatively evaluate the performance of the Xilinx Alveo FPGA memory systems under a comprehensive set of factors that affect the memory bandwidth, including 1) the clock frequency of the accelerator design, 2

## PeterTh/uCLbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/PeterTh/uCLbench
- Категория: benchmark/testing candidate
- Описание: Set of OpenCL microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/PeterTh__uCLbench/README](sources/PeterTh__uCLbench/README); SHA-256: `6112ac1ca2c892c048bd064b07e4607cd445ba8c708171d7bc6724853a3c8b66`
- gpu: строка 6: GPU and accelerator architectures using a single unified programming inter-

## scivision/python-performance

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/scivision/python-performance
- Категория: benchmark/testing candidate
- Описание: Performance benchmarks of Python, Numpy, etc. vs. other languages such as Matlab, Julia, Fortran.
- Уровень: automated README evidence extraction
- Снимок: [sources/scivision__python-performance/README.md](sources/scivision__python-performance/README.md); SHA-256: `aa16beb1ba2ec6a0ab79c9dd581b3cbad3c4c86df8382516b3a2a5b46f8b1685`
- gpu: строка 6: CuPy tests require an NVIDIA GPU with CUDA toolkit installed.

## IBM/microprobe

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/IBM/microprobe
- Категория: benchmark/testing candidate
- Описание: Microprobe: Microbenchmark generation framework
- Уровень: automated README evidence extraction
- Снимок: [sources/IBM__microprobe/README.md](sources/IBM__microprobe/README.md); SHA-256: `224346e51c9179a811dc73869dd39aa7c343dfb364698b07643f0d7157110f27`
- statistics: строка 47: bootstrap_environment.sh
- report: строка 50: Hopefully, the installation is complete. Otherwise, report the

## tycho/clockperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tycho/clockperf
- Категория: benchmark/testing candidate
- Описание: clocksource behavior microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/tycho__clockperf/README.md](sources/tycho__clockperf/README.md); SHA-256: `c0ecff9407e3a533dbb0b650aeb6d1e3d3300f18947b4e046498ca396f48d102`

## adoptium/bumblebench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/adoptium/bumblebench
- Категория: benchmark/testing candidate
- Описание: A microbenchmarking test framework for Eclipse Adoptium
- Уровень: automated README evidence extraction
- Снимок: [sources/adoptium__bumblebench/README.md](sources/adoptium__bumblebench/README.md); SHA-256: `baf32ee0895ce9d247fef39fce7eff40ba492ac37b3ee2ffa5c6e96d0676623f`
- comparison: строка 2: This microbenchmarking test framework for [Eclipse Adoptium](https://adoptium.net), along with [jmh](http://openjdk.java.net/projects/code-tools/jmh/) will be used to create and run different types of benchmarks on binaries produced at Adoptium (to verify that

## shen203/GPU_Microbenchmark

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/shen203/GPU_Microbenchmark
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## AI-Hypercomputer/accelerator-microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/AI-Hypercomputer/accelerator-microbenchmarks
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/AI-Hypercomputer__accelerator-microbenchmarks/README.md](sources/AI-Hypercomputer__accelerator-microbenchmarks/README.md); SHA-256: `9b0cfded450d6f207cb97b6d3983b703e34900e594f6c8ac42126336cc928ca4`
- lifecycle: строка 23: parameter sweeps, warm-up iterations, and matrix shapes without modifying Python
- async: строка 14: the performance (latency, throughput, memory bandwidth) of various JAX
- report: строка 36: ├── results/            # Can create output directory for benchmark metrics (JSON, CSV)

## berestovskyy/applied-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/berestovskyy/applied-benchmarks
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks and Google Benchmark library
- Уровень: automated README evidence extraction
- Снимок: [sources/berestovskyy__applied-benchmarks/README.md](sources/berestovskyy__applied-benchmarks/README.md); SHA-256: `4a14341dba576a34adbbd2628459277c92c5c3141796082e4758fb9ddfdc33fb`

## jorendorff/dht

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/jorendorff/dht
- Категория: benchmark/testing candidate
- Описание: Deterministic hash table (implementation and microbenchmarks)
- Уровень: automated README evidence extraction
- Снимок: [sources/jorendorff__dht/README.md](sources/jorendorff__dht/README.md); SHA-256: `6a67b64f79c7f582459cad59a19692a1a04590dfd1ba486081ca29b62626cabf`
- memory: строка 29: * figure-1.png shows how much memory each implementation allocates. figure-1-data.txt is the raw data.
- report: строка 31: * The images InsertSmallTest-speed.png and friends show how fast each implementation is at each test. Higher is better. The file hashbench-data.txt contains the raw data for all these graphs. It's JSON.

## rdyro/tune-jax

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rdyro/tune-jax
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking hyperparameter tuning for JAX functions.
- Уровень: automated README evidence extraction
- Снимок: [sources/rdyro__tune-jax/README.md](sources/rdyro__tune-jax/README.md); SHA-256: `3c0948d60424c2cdf57b7c474914a2e8b95a788e27ea13a1df74a28f5ec1134f`
- gpu: строка 28: ## Example: Tuning Attention on GPU

## maximenajim/java-vs-node-react-rendering-microbenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/maximenajim/java-vs-node-react-rendering-microbenchmark
- Категория: benchmark/testing candidate
- Описание: Isomorphic (server-side) rendering of a simple react (comment box) component - Java 8's nashorn vs node.js microbenchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/maximenajim__java-vs-node-react-rendering-microbenchmark/README.md](sources/maximenajim__java-vs-node-react-rendering-microbenchmark/README.md); SHA-256: `dfcf912f310343313bdbc3edb6ce3520f972d560f733b7bc1476ff92b879ee09`
- comparison: строка 84: *Java team is fixing the react.js performance regression:* https://bugs.openjdk.java.net/browse/JDK-8134403
- lifecycle: строка 23: Performance comparing Nashorn with node.js. Nashorn takes a little longer to warm-up but after 10000 iterations it matches node.js performance.

## arkanis/syscall-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/arkanis/syscall-benchmark
- Категория: benchmark/testing candidate
- Описание: A set of small system call microbenchmarks. Just for fun, nothing serious.
- Уровень: automated README evidence extraction
- Снимок: [sources/arkanis__syscall-benchmark/README.md](sources/arkanis__syscall-benchmark/README.md); SHA-256: `5e909e03deba166bfdf1e050e9ce8cebe8c10920544bbaf5eea45f833dfb10be`

## wsargent/slf4j-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/wsargent/slf4j-benchmark
- Категория: benchmark/testing candidate
- Описание: Microbenchmark of SLF4J / Logback using JMH
- Уровень: automated README evidence extraction
- Снимок: [sources/wsargent__slf4j-benchmark/README.md](sources/wsargent__slf4j-benchmark/README.md); SHA-256: `51a6005aaae2ecc067029fe8d4753ef8f4566594a69611f9cf924e3f7bd333e8`
- statistics: строка 94: There is a case to be made for logging the control flow of every request/response, first noted in [Log Everything All the Time](http://highscalability.com/log-everything-all-time) and popularized by Honeycomb as [event based logging](https://docs.honeycomb.io/
- memory: строка 19: When disabled, logging has effectively no cost.  When enabled, logging is still very cheap, but does add up with large amounts of indiscriminate logging.
- async: строка 17: If you're logging in background, use a disruptor based async appender and then log to a buffered filewriter, or to network, and use a shutdown hook.

## eigenform/lamina

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/eigenform/lamina
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking experiments on Zen 2 machines
- Уровень: automated README evidence extraction
- Снимок: [sources/eigenform__lamina/README.md](sources/eigenform__lamina/README.md); SHA-256: `2829f1e6481a2bce5ad3fd88777dea55362be225a127b94b2acb21554de80231`

## Deleplace/microbenchmarks

- Итог: excluded — README too thin for content analysis
- Источник: https://github.com/Deleplace/microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Benchmarks in Go
- Уровень: automated README evidence extraction
- Снимок: [sources/Deleplace__microbenchmarks/README.md](sources/Deleplace__microbenchmarks/README.md); SHA-256: `568374319bd593406f1e32481fdeab9156142bb88367ee16f4f4b3d4c8a63d51`

## pyk/bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pyk/bench
- Категория: benchmark/testing candidate
- Описание: Fast & Accurate Microbenchmarking for Zig
- Уровень: automated README evidence extraction
- Снимок: [sources/pyk__bench/README.md](sources/pyk__bench/README.md); SHA-256: `f6baf2df6668283dd3b7adbb6c7d4dc2c2c09eab288cab6e696adfbdf5f6e6ff`
- statistics: строка 228: median, variance, cycles, etc.). You can use this to generate JSON, CSV, or
- comparison: строка 57: .baseline_index = 0, // naive as baseline
- lifecycle: строка 50: .warmup_iters = 3,
- memory: строка 47: const allocator = std.heap.smp_allocator;
- async: строка 89: - **Easy Throughput Metrics**: Automatically calculates operations per second
- report: строка 55: try bench.report(.{

## RealTimeChris/benchmarksuite

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/RealTimeChris/benchmarksuite
- Категория: benchmark/testing candidate
- Описание: Header-only C++20/CUDA microbenchmarking with hardware performance counters, adaptive iteration sampling, and 95% confidence-interval statistics.
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## seriyps/rebar3_bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/seriyps/rebar3_bench
- Категория: benchmark/testing candidate
- Описание: Microbenchmark plugin for rebar3
- Уровень: automated README evidence extraction
- Снимок: [sources/seriyps__rebar3_bench/README.md](sources/seriyps__rebar3_bench/README.md); SHA-256: `e3318589f26d1a24e0d8190662d2cf5a6d0848b31849a5df785fa2eef90182ff`
- statistics: строка 9: It relies on [eministat](https://hex.pm/packages/eministat) for statistical calculations.
- comparison: строка 117: regression:
- memory: строка 81: Each benchmark is executed in a separate process with `{priority, high}, {min_heap_size, 5mb}`.
- report: строка 175: standard destination, you can use `rebar3 cover` to see the report.
- isolation: строка 81: Each benchmark is executed in a separate process with `{priority, high}, {min_heap_size, 5mb}`.

## ning/ub

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ning/ub
- Категория: benchmark/testing candidate
- Описание: PHP Microbenchmarking Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/ning__ub/README](sources/ning__ub/README); SHA-256: `2d13e35b881975e6f3575af1b92b6a8570c225bbb7de5b83960658c29c5e9bba`
- lifecycle: строка 36: Frequently, you need some setup code that you don't want to include in
- report: строка 33: times and then reports the results across all 1000 iterations. The

## biboudis/clashofthelambdas

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/biboudis/clashofthelambdas
- Категория: benchmark/testing candidate
- Описание: Microbenchmarking Stream APIs of Java 8, Scala, C#, F#.
- Уровень: automated README evidence extraction
- Снимок: [sources/biboudis__clashofthelambdas/README.md](sources/biboudis__clashofthelambdas/README.md); SHA-256: `b482ba4f75a4c2aea1c5440d582ca3d7795c6f1b187081c168719f323e2e9b1a`
- memory: строка 11: 3GB of free space for heap allocation. Regarding execution time, a run on a

## blitz/kernel_entry_benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/blitz/kernel_entry_benchmark
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks for x86_64 kernel entry methods
- Уровень: automated README evidence extraction
- Снимок: [sources/blitz__kernel_entry_benchmark/README.md](sources/blitz__kernel_entry_benchmark/README.md); SHA-256: `ddec27671a20b79077d3cb02075473acef3819dfae964c412033ca2cfce3c471`
- report: строка 65: The output is in CSV format. The first column is the kernel entry

## vickiegpt/A16-microbenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vickiegpt/A16-microbenchmark
- Категория: benchmark/testing candidate
- Описание: A repo to store what M1 explainer does and ~Armv9~ Armv8.6 ISA on A16
- Уровень: automated README evidence extraction
- Снимок: [sources/vickiegpt__A16-microbenchmark/README.md](sources/vickiegpt__A16-microbenchmark/README.md); SHA-256: `05d6a0df8aaa16344ddb8be34300aa8042eafec1c121e94be6af9ebc85b3054a`
- gpu: строка 13: - [ ] 🚧 CPU to GPU latency
- async: строка 10: - [ ] 🚧 Big core to big core latency

## martinbonnin/run-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/martinbonnin/run-benchmarks
- Категория: benchmark/testing candidate
- Описание: A GitHub action that runs and publishes your Android microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/martinbonnin__run-benchmarks/README.md](sources/martinbonnin__run-benchmarks/README.md); SHA-256: `3492a54d6e2ab623ccd7f97865bff0ac2dc909d26ecdea40be920896a94c3ac5`
- lifecycle: строка 26: - uses: actions/setup-java@v3
- memory: строка 56: - ${dd_metric_prefix}.allocs: the median number of allocations per test
- report: строка 7: - publishes the metrics to Datadog ([see here for a sample dashboard](https://p.datadoghq.com/sb/5218edc4-01bd-11ed-a9be-da7ad0900002-8b732d527dbbc83641c63ef56364d8d1))

## timoheimonen/macOS-memory-benchmark

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/timoheimonen/macOS-memory-benchmark
- Категория: benchmark/testing candidate
- Описание: Low-level command-line tool for measuring CPU and Metal GPU memory bandwidth, synthetic LLM decode and prefill memory traffic, cache and main-memory latency, access-pattern performance, TLB behavior, and two-thread cache-line handoff protocol latency on Apple Silicon Macs.
- Уровень: automated README evidence extraction
- Снимок: [sources/timoheimonen__macOS-memory-benchmark/README.md](sources/timoheimonen__macOS-memory-benchmark/README.md); SHA-256: `f76c498bf625ab12d09dbe8ba1d57deb998d062d5ae3003b00743ff207b42e51`
- statistics: строка 22: - **Dedicated TLB analysis:** paired spread/packed chains, adaptive rounds, confidence intervals, and independent boundary validation.
- comparison: строка 22: - **Dedicated TLB analysis:** paired spread/packed chains, adaptive rounds, confidence intervals, and independent boundary validation.
- lifecycle: строка 11: expose calibration, workload, completion, and repeatability metadata so results can be audited and compared.
- memory: строка 307: compute-memory overlap, ANE paths, GPU execution outside the defined Metal kernels, runtime page allocation,
- gpu: строка 5: `memory_benchmark` is a low-level command-line tool for measuring CPU and Metal GPU memory bandwidth, synthetic LLM
- async: строка 6: decode and prefill memory traffic, cache and main-memory latency, access-pattern performance, TLB behavior, and
- report: строка 15: *Cache latency on a MacBook Air M5 across working-set sizes, pointer strides, and TLB-locality configurations. Generated from multiple JSON result files using the included plotting tools.*
- correctness: строка 22: - **Dedicated TLB analysis:** paired spread/packed chains, adaptive rounds, confidence intervals, and independent boundary validation.

## spthm/cudabmk

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/spthm/cudabmk
- Категория: benchmark/testing candidate
- Описание: Source for Demystifying GPU Microarchitecture through Microbenchmarking
- Уровень: automated README evidence extraction
- Снимок: [sources/spthm__cudabmk/README.md](sources/spthm__cudabmk/README.md); SHA-256: `71a944a45be887548cb2d9f475112a6cee357372d814a1daf5f13755737a5e0d`
- gpu: строка 1: ### Demystifying GPU Microarchitecture through Microbenchmarking

## RasterDuck/vectormathbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/RasterDuck/vectormathbench
- Категория: benchmark/testing candidate
- Описание: A collection of microbenchmarks for various game- and graphics-centric vectormath libraries.
- Уровень: automated README evidence extraction
- Снимок: [sources/RasterDuck__vectormathbench/README.md](sources/RasterDuck__vectormathbench/README.md); SHA-256: `9fe9256165cdcb1f0b139e650021e7a06abe69cd0e10306d9b0651381d2df81c`
- comparison: строка 3: This project benchmarks several open source vector math libraries against one another to establish a baseline for performance.  Currently, it tests [GLM](https://github.com/g-truc/glm), [DirectXMath](https://github.com/microsoft/DirectXMath), [SimpleMath from
- lifecycle: строка 55: capability uses 15 epochs, a warm-up phase, and a minimum epoch duration of
- async: строка 36: Intersection latency and throughput are intentionally separate capabilities.
- report: строка 38: 256 varied rays with a realistic mixture of hits and misses and report the

## softprops/cappi

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/softprops/cappi
- Категория: benchmark/testing candidate
- Описание: the sweetest sbt plugin your microbenchmarks will ever meet
- Уровень: automated README evidence extraction
- Снимок: [sources/softprops__cappi/README.md](sources/softprops__cappi/README.md); SHA-256: `2e8317c4315ddaad883a14fff352516d379a14223a8e106aa0523597d0fd2ed9`

## silversquirl/benchmark.zig

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/silversquirl/benchmark.zig
- Категория: benchmark/testing candidate
- Описание: Tiny Zig package for writing microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/silversquirl__benchmark.zig/README.md](sources/silversquirl__benchmark.zig/README.md); SHA-256: `1360c3012b3620bd5696cad2a98faf90948855ea76c849c50744280cf6d589eb`
- lifecycle: строка 17: // Setup is not timed
- memory: строка 18: var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);

## maxim-saplin/mandelbrot

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/maxim-saplin/mandelbrot
- Категория: benchmark/testing candidate
- Описание: Microbenchmark testing Python, Numba, Mojo, Dart, C/gcc, Rust, Go, JavaScript, C#, Java, Kotlin, Pascal, Ruby, Haskell performance in Mandelbrot set generation
- Уровень: automated README evidence extraction
- Снимок: [sources/maxim-saplin__mandelbrot/README.md](sources/maxim-saplin__mandelbrot/README.md); SHA-256: `25ba81a4c186dcbe8d4a2b19e07b3e06e3933cae1ca69396cfc6f8f8e0fbb536`

## fullzer4/pybenchx

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fullzer4/pybenchx
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks that mean it
- Уровень: automated README evidence extraction
- Снимок: [sources/fullzer4__pybenchx/README.md](sources/fullzer4__pybenchx/README.md); SHA-256: `7b9ec69d9de165f3357e105241041beb9527e0a31deac1c816ebe6147697ebab`
- comparison: строка 11: Practical microbenchmarks for Python—tight iteration cycles, precise hot-path timing, and storage that makes comparisons and regression gates effortless.
- lifecycle: строка 24: - On-the-fly overrides: `-P key=value` adjusts `n`, `repeat`, `warmup`, `group`, or custom params without editing code.
- report: строка 27: - Rich reports: aligned tables with percentiles, iter/s, min…max, baseline markers, and speedups vs. base.

## chrirocca/GPUNetBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chrirocca/GPUNetBench
- Категория: benchmark/testing candidate
- Описание: Collection of memory microbenchmarks to investigate NVIDIA GPUs Network on Chip architectures
- Уровень: automated README evidence extraction
- Снимок: [sources/chrirocca__GPUNetBench/README.md](sources/chrirocca__GPUNetBench/README.md); SHA-256: `52eb935651a71eb25f31e2de8061678ce904981888ab7d65f985cdfcf2612b8a`
- gpu: строка 1: # GPU Microbenchmarks
- async: строка 3: This repository contains CUDA-based benchmarks designed to evaluate various aspects of GPU memory and interconnection networks on NVIDIA GPUs (V100, A100, H100). Each benchmark focuses on distinct architectural components, using bandwidth, latency, and executi

## JimZeyuYang/GPU_Power_Benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/JimZeyuYang/GPU_Power_Benchmark
- Категория: benchmark/testing candidate
- Описание: Microbenchmark that unveals the mechanisms behind power readings reported by nvidia-smi on your NVIDIA GPU.
- Уровень: automated README evidence extraction
- Снимок: [sources/JimZeyuYang__GPU_Power_Benchmark/README.md](sources/JimZeyuYang__GPU_Power_Benchmark/README.md); SHA-256: `2e3919ab5b12f681e9cc1f7066476056b83862b9b83f7dfdbf3ef00a370e5354`
- lifecycle: строка 19: ## Setup
- gpu: строка 1: # GPU Power Microbenchmark
- report: строка 4: Microbenchmark that unveals the mechanisms behind power readings reported by nvidia-smi/MVML on your NVIDIA GPU.

## peter-lawrey-admin/jvm-micro-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/peter-lawrey-admin/jvm-micro-benchmarks
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks for JVM code.
- Уровень: automated README evidence extraction
- Снимок: [sources/peter-lawrey-admin__jvm-micro-benchmarks/README.md](sources/peter-lawrey-admin__jvm-micro-benchmarks/README.md); SHA-256: `209c7ac392cf9ceb1653c9c9a4a17edf238bd6e7b2bb68471c4ffa7d709b78b5`
- async: строка 4: Some benchmarks use JMH for latency test.

## biboudis/LambdaMicrobenchmarking

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/biboudis/LambdaMicrobenchmarking
- Категория: benchmark/testing candidate
- Описание: A library to microbenchmark lambdas in C# and F# that runs on both Windows (clr) and Linux (mono).
- Уровень: automated README evidence extraction
- Снимок: [sources/biboudis__LambdaMicrobenchmarking/README.md](sources/biboudis__LambdaMicrobenchmarking/README.md); SHA-256: `fe08c4b7697ea9c6f37bf187c155db711f13f8b74d02d7e0dc364dbdeaf166ca`

## gormanm/pft

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gormanm/pft
- Категория: benchmark/testing candidate
- Описание: Page fault test microbenchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/gormanm__pft/README](sources/gormanm__pft/README); SHA-256: `a7f501eefea1f329f1f99f86089ada454442ea823ecc2394a9a3c71dac81dfee`
- memory: строка 77: 0.01  - first version of pft_mpol for testing mempolicy fault/allocation
- report: строка 96: "improve readability" of final results reporting.

## peterszatmary/jmh-benchmark-demo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/peterszatmary/jmh-benchmark-demo
- Категория: benchmark/testing candidate
- Описание: Java Microbenchmark Harness (JMH) that runs with Junit and Maven.
- Уровень: automated README evidence extraction
- Снимок: [sources/peterszatmary__jmh-benchmark-demo/README.md](sources/peterszatmary__jmh-benchmark-demo/README.md); SHA-256: `a01e9b0fff0eefb94ad8080f3b485838eeaf242550068578c41670bfdb94f593`

## lemire/microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lemire/microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Private microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/lemire__microbenchmarks/README.md](sources/lemire__microbenchmarks/README.md); SHA-256: `3cd9695995b40b7d1ec442de8ec14bddd4af56ae7d75947df6b4d3005e960a82`
- comparison: строка 56: java -cp target/microbenchmarks-0.0.1-jar-with-dependencies.jar me/lemire/hashing/InterleavedHash
- memory: строка 27: Compare direct vs. heap buffers

## CoreyLeath-code/AutoGuard-AI-Real-Time-Autonomous-Vehicle-Safety-Geofencing-Platform

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/CoreyLeath-code/AutoGuard-AI-Real-Time-Autonomous-Vehicle-Safety-Geofencing-Platform
- Категория: benchmark/testing candidate
- Описание: Prototype geofencing and telemetry reference with Haversine radius checks, FastAPI health/readiness, reproducible microbenchmarks, CI, and versioned packages; simulation only.
- Уровень: automated README evidence extraction
- Снимок: [sources/CoreyLeath-code__AutoGuard-AI-Real-Time-Autonomous-Vehicle-Safety-Geofencing-Platform/README.md](sources/CoreyLeath-code__AutoGuard-AI-Real-Time-Autonomous-Vehicle-Safety-Geofencing-Platform/README.md); SHA-256: `d2441a0ac04efae12084d3aa1426fbeb4f6a5f6db0ea3fda1193334b792aa55a`
- async: строка 102: FastAPI validation ──> async thread offload ──> prototype geocoding adapter
- report: строка 63: python -m pip install ".[full]"   # optional dashboard, ML, streaming, and experiment dependencies
- correctness: строка 11: > **Safety boundary:** this is not an autonomous-vehicle safety system. It has not been validated on a vehicle, connected to actuators or braking hardware, shown to meet real-time requirements, or developed as a safety-certified product. Do not use it to make

## bdice/python_microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bdice/python_microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks showing relative performance of different Python functions/patterns.
- Уровень: automated README evidence extraction
- Снимок: [sources/bdice__python_microbenchmarks/README.md](sources/bdice__python_microbenchmarks/README.md); SHA-256: `1883a9459d463562182e1b17e9c6fd4ec90ff64b60bc48e53676b17710c776e2`

## SEALABQualityGroup/icpe-data-challenge-jmh

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SEALABQualityGroup/icpe-data-challenge-jmh
- Категория: benchmark/testing candidate
- Описание: Dataset for the ICPE 2023 Data Challenge track: JMH microbenchmarks measurements from Java open source projects.
- Уровень: automated README evidence extraction
- Снимок: [sources/SEALABQualityGroup__icpe-data-challenge-jmh/README.md](sources/SEALABQualityGroup__icpe-data-challenge-jmh/README.md); SHA-256: `0cc105a8f6acce8f8c13d6997fc3d7dd7c2ee7a417f316c26e83cd0d1d8d1b62`
- report: строка 6: The dataset contains performance measurements of JMH microbenchmarks from 30 Java open source projects. The list of projects, along with the revision at which the microbenchmarks were executed, can be found in [benchmarks_revision.csv](benchmarks_revision.csv)

## nsrip-dd/profiler-overhead-testing

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nsrip-dd/profiler-overhead-testing
- Категория: benchmark/testing candidate
- Описание: Testing effects of the Go CPU profiler on microbenchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/nsrip-dd__profiler-overhead-testing/README.md](sources/nsrip-dd__profiler-overhead-testing/README.md); SHA-256: `91ce0d9015c655708f16eba841c065e33799e399def4b1c5837a8e33ccd69c7e`
- memory: строка 3: and memory usage. Building off of Felix’s [work](https://github.com/felixge/go-observability-bench) which he [presented](https://www.gophercon.com/agenda/session/596212) at
- async: строка 2: the effects of enabling the Go CPU profiler on CPU usage, benchmark latency,
- report: строка 14: * Unmarshal and marshal some JSON (~4500 line randomly generated data)

## chrishantha/microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chrishantha/microbenchmarks
- Категория: benchmark/testing candidate
- Описание: JMH projects http://openjdk.java.net/projects/code-tools/jmh/
- Уровень: automated README evidence extraction
- Снимок: [sources/chrishantha__microbenchmarks/README.md](sources/chrishantha__microbenchmarks/README.md); SHA-256: `b0ab084dd6c795afd6fe88b654554b5f08a652b0f68b575bb93fcbd242d06311`

## thomwiggers/microbenchmark-aarch64

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/thomwiggers/microbenchmark-aarch64
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks for Aarch64 (Cortex A53)
- Уровень: automated README evidence extraction
- Снимок: [sources/thomwiggers__microbenchmark-aarch64/README.md](sources/thomwiggers__microbenchmark-aarch64/README.md); SHA-256: `5d108b663d077d7545288f7be316eb9ce86a6a122dddc01070cae59647a8566f`

## buchgr/rules_jmh

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/buchgr/rules_jmh
- Категория: benchmark/testing candidate
- Описание: Bazel rules for generating and running microbenchmarks with JMH
- Уровень: automated README evidence extraction
- Снимок: [sources/buchgr__rules_jmh/README.md](sources/buchgr__rules_jmh/README.md); SHA-256: `1cdff8b893de884f76e0a7d9f892e4e4f6f9d2e1de60b60a9879e573206152f5`

## efficient/microservices_microbenchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/efficient/microservices_microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Code for the benchmarks presented in https://www.usenix.org/conference/atc18/presentation/boucher
- Уровень: automated README evidence extraction
- Снимок: [sources/efficient__microservices_microbenchmarks/README.md](sources/efficient__microservices_microbenchmarks/README.md); SHA-256: `563858768081d89fc12ef3e7002a91269eccb266c7ea7b75466624a1e773f217`
- statistics: строка 118: This will run the experiment, then print the absolute values of recorded deviations followed by a statistical summary.
- lifecycle: строка 119: We recommend treating the very first invocation as a warmup round.
- async: строка 63: Invocation latency experiment (section 2.1)
- report: строка 56: Data files with reported numbers
- isolation: строка 45: * `launcher` uses worker processes to demonstrate what the paper refers to as "language-based isolation"

## rolfl/MicroBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rolfl/MicroBench
- Категория: benchmark/testing candidate
- Описание: Enabling simpler microbenchmarks of Java8 code
- Уровень: automated README evidence extraction
- Снимок: [sources/rolfl__MicroBench/README.md](sources/rolfl__MicroBench/README.md); SHA-256: `83b69cee448d80292d0cb297e1b3e8bc6d98fdbdf6cffa288c89bd92fc7fb4b3`
- report: строка 55: .report();

## sjenning/zsmapbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sjenning/zsmapbench
- Категория: benchmark/testing candidate
- Описание: Microbenchmark for zsmalloc allocation mapping
- Уровень: automated README evidence extraction
- Снимок: [sources/sjenning__zsmapbench/README](sources/sjenning__zsmapbench/README); SHA-256: `9642ddc0e7af5e67771dbb063d39031a2a583aa4aad9be65165cf800902f4cd6`
- memory: строка 1: Microbenchmark for zsmalloc allocation mapping

## zhisbug/ray-scalable-ml-design

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/zhisbug/ray-scalable-ml-design
- Категория: benchmark/testing candidate
- Описание: Some microbenchmarks and design docs before commencement
- Уровень: automated README evidence extraction
- Снимок: [sources/zhisbug__ray-scalable-ml-design/README.md](sources/zhisbug__ray-scalable-ml-design/README.md); SHA-256: `25ca827896d0a9b5fae429dad419cf6d45ce174df64f8f69ee400ccc67e7248c`

## mhirki/idq-bench2

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mhirki/idq-bench2
- Категория: benchmark/testing candidate
- Описание: Instruction decoder microbenchmark suite
- Уровень: automated README evidence extraction
- Снимок: [sources/mhirki__idq-bench2/README.md](sources/mhirki__idq-bench2/README.md); SHA-256: `5ba9aef2068357e4f3e57c8f4e533492d473c13cfed2039db89e3f5f139e9a0e`

## HicrestLaboratory/Blink-GPU

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/HicrestLaboratory/Blink-GPU
- Категория: benchmark/testing candidate
- Описание: A suite of microbenchmarks developed for systems with multi-GPU per node.
- Уровень: automated README evidence extraction
- Снимок: [sources/HicrestLaboratory__Blink-GPU/README.md](sources/HicrestLaboratory__Blink-GPU/README.md); SHA-256: `d4add8a123ec3055786e98afaeec07a0ab4d7b6076565a3954e5002bc0d9098d`
- comparison: строка 36: 1. *Baseline*: the data **m** is first copied by **SD** to **SH** with a cudaMemcpy, then moved from **SH** to **TH** with an MPI primitive and finaly copied to **TD** with another cudaMemcpy.
- lifecycle: строка 246: 1. Some iteration has a negative iteration number; those ones represent the warm-up iteration and are not involved in the average computation.
- gpu: строка 1: # Blink: A Benchmark for Large-Scale Multi-GPU Interconnects
- report: строка 81: Regarding the Ping-pong and the All-to-all communication scheme, after initialising the send buffer with a fixed value, each process computes the sum reduction of the sending buffer and stores them inside "my_cpu_check". Once the benchmarked communication is c
- correctness: строка 77: ### Automatic correctness checks

## jeffzi/luamark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/jeffzi/luamark
- Категория: benchmark/testing candidate
- Описание: A lightweight, portable microbenchmarking library for Lua
- Уровень: automated README evidence extraction
- Снимок: [sources/jeffzi__luamark/README.md](sources/jeffzi__luamark/README.md); SHA-256: `42d209996dfb8ba261917a86bdc9353b0d4f88f7b608e4b2bc5ca46bc9cf70de`
- statistics: строка 18: - **Statistics**: median with 95% confidence intervals
- lifecycle: строка 119: With parameters and setup:
- memory: строка 12: execution time and memory usage with sensible defaults and optional high-precision clocks.

## DiegoEliasCosta/spotjmhbugs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DiegoEliasCosta/spotjmhbugs
- Категория: benchmark/testing candidate
- Описание: A SpotBugs plugin that detects bad practices on JMH microbenchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/DiegoEliasCosta__spotjmhbugs/README.md](sources/DiegoEliasCosta__spotjmhbugs/README.md); SHA-256: `5f33be507f50e56ffe60221ade586ab5b191611438c9fa39d6dc791a14517abb`

## vitinh0z/java-performance

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vitinh0z/java-performance
- Категория: benchmark/testing candidate
- Описание: Estudos aprofundados de performance em Java: Microbenchmarks (JMH), Concorrência e otimizações na JVM
- Уровень: automated README evidence extraction
- Снимок: [sources/vitinh0z__java-performance/README.md](sources/vitinh0z__java-performance/README.md); SHA-256: `1469fcbae7e14c4181293556aae9ca591f54876fd2df0f48c6cf572705a3fbf8`

## mazalves/microbenchmarks

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/mazalves/microbenchmarks
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks to evaluate architectural details
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## epickrram/journalling-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/epickrram/journalling-benchmark
- Категория: benchmark/testing candidate
- Описание: Microbenchmark for comparing the cost of seek/write vs pwrite from Java
- Уровень: automated README evidence extraction
- Снимок: [sources/epickrram__journalling-benchmark/README.md](sources/epickrram__journalling-benchmark/README.md); SHA-256: `8edbb7eb4f8cd8f5ff04c7c48b776d678f97c55f1a128f046c5f511050b98995`
- isolation: строка 18: * -c - cpu affinity for journaller thread (default none)

## karnajitsen/NVLink-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/karnajitsen/NVLink-benchmark
- Категория: benchmark/testing candidate
- Описание: NVLink microbenchmark  with IBM Power8 and NVIDIA P100 GPU - Master Thesis
- Уровень: automated README evidence extraction
- Снимок: [sources/karnajitsen__NVLink-benchmark/README.md](sources/karnajitsen__NVLink-benchmark/README.md); SHA-256: `423a1d780bb2062f1a772be49a41c3ace45d5c4ade32829a624f20acd2aa5944`
- gpu: строка 25: START_THREAD = 1024     							   : Starting gpu block size
- async: строка 12: LATDATATYPE = int                                      : data type for latency experiement
- report: строка 51: After execution, output will be stored inside /data/ folder as csv and text file with following naming conventions.

## yushinliu/cuda-ptx-microbenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yushinliu/cuda-ptx-microbenchmark
- Категория: benchmark/testing candidate
- Описание: CUDA+PTX Microbenchmark for RTX 4070 GPU
- Уровень: automated README evidence extraction
- Снимок: [sources/yushinliu__cuda-ptx-microbenchmark/README.md](sources/yushinliu__cuda-ptx-microbenchmark/README.md); SHA-256: `28857aa898f600119c636e98a479b5bdfdcf4fd42f35e1526d240e02e46b4f53`
- lifecycle: строка 32: ### WSL2 Setup
- gpu: строка 28: - NVIDIA GPU with Compute Capability 8.0+ (optimized for sm_89 / RTX 4070)
- async: строка 12: - **Integer Instruction Benchmarks**: IADD3, LOP3, SEL, SHFL latency and throughput (RTX 4070)

## YKTian-x2b/AmpereArchViaMicroBenchMark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/YKTian-x2b/AmpereArchViaMicroBenchMark
- Категория: benchmark/testing candidate
- Описание: 通过MicroBenchmark获悉Ampere微架构知识
- Уровень: automated README evidence extraction
- Снимок: [sources/YKTian-x2b__AmpereArchViaMicroBenchMark/README.md](sources/YKTian-x2b__AmpereArchViaMicroBenchMark/README.md); SHA-256: `484708b3be052147baf5c355a26a2068ba42c3750c74863c289ff6cbd1f0fa45`
- gpu: строка 56: # 锁频 但是warning Setting applications clocks is not supported for GPU 00000000:01:00.0.
- async: строка 6: |               | Size                    | Bandwidth                       | BW理论上限            | Latency     |

## yvt/farcri-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yvt/farcri-rs
- Категория: benchmark/testing candidate
- Описание: FarCri.rs: Criterion.rs-based microbenchmarking library for remote resource-constrained systems
- Уровень: automated README evidence extraction
- Снимок: [sources/yvt__farcri-rs/README.md](sources/yvt__farcri-rs/README.md); SHA-256: `1e5c259e68e7577cf36c59efaa6d925f7432b14b4effae348fadf5843eb7da9d`
- async: строка 31: use farcri::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

## cameron314/atomic_bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cameron314/atomic_bench
- Категория: benchmark/testing candidate
- Описание: Microbenchmarks for some C++11 atomic primitives
- Уровень: automated README evidence extraction
- Снимок: [sources/cameron314__atomic_bench/README.md](sources/cameron314__atomic_bench/README.md); SHA-256: `df579e0dcc6c13039c97c9c84265c4b77ba8446845b544ec9692132c0a8697fe`

## neozhang307/SyncMicrobenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/neozhang307/SyncMicrobenchmark
- Категория: benchmark/testing candidate
- Описание: This work aims at characterizing the synchronization methods in CUDA.
- Уровень: automated README evidence extraction
- Снимок: [sources/neozhang307__SyncMicrobenchmark/README.md](sources/neozhang307__SyncMicrobenchmark/README.md); SHA-256: `82ba34fb3994a83863b7a11b162242d52b1eccf29420f884061a499cdfc2b1c2`
- gpu: строка 6: * Multi-GPU synchronization with OpenMP in a single node.
- async: строка 21: The sleep function is only available after sm_70. We tried to use other instructions to control the kernel execute latency, but the result is not so stable as sleep instruction, and larger than sleep instruction.

## benfred/py-spy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/benfred/py-spy
- Категория: profiling
- Описание: Sampling profiler for Python programs
- Уровень: automated README evidence extraction
- Снимок: [sources/benfred__py-spy/README.md](sources/benfred__py-spy/README.md); SHA-256: `edad5396d6b6ece0de5d523965fbaf6c980afdcdcd4c39a905fe9f8ef62c8831`
- isolation: строка 64: showing thread-ids, profiling subprocesses and more.

## flamegraph-rs/flamegraph

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/flamegraph-rs/flamegraph
- Категория: profiling
- Описание: Easy flamegraphs for Rust projects and everything else, without Perl or pipes <3
- Уровень: automated README evidence extraction
- Снимок: [sources/flamegraph-rs__flamegraph/README.md](sources/flamegraph-rs__flamegraph/README.md); SHA-256: `555e025188c1795a8645ce86ef8639cbb50977f004ece2fce16f1a34dacb1786`
- memory: строка 341: micro-optimizations, allocation-minimization, etc...
- async: строка 438: maximum throughput for a workload, but the latency per request
- report: строка 290: It has been reported that `addr2line` can run very slowly in several issues ([#74][i74], [#199][i199], [#294][i294]). One solution is to use [gimli-rs/addr2line](https://github.com/gimli-rs/addr2line) instead of the system `addr2line` binary. This is suggested

## mstange/samply

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mstange/samply
- Категория: profiling
- Описание: Command-line sampling profiler for macOS, Linux, and Windows
- Уровень: automated README evidence extraction
- Снимок: [sources/mstange__samply/README.md](sources/mstange__samply/README.md); SHA-256: `1aac83083f49924dd7066e648992032c952b83d8601672afe4bc94a0fb3d03e6`
- lifecycle: строка 137: But you can profile any binaries that you've compiled yourself, or which are unsigned or locally-signed (such as anything installed by `cargo install` or by [Homebrew](https://brew.sh)). In order to attach to running processes on macOS, run `samply setup` once
- report: строка 97: - If using Linux 5.8 or later, you can try setting the `CAP_PERFMON` capability as effective and permitted for samply, though people have reported mixed results with this approach:
- isolation: строка 64: This spawns `./my-application my-arguments` in a subprocess and records a profile of its execution. When the command finishes, samply opens

## edison7009/EchoBird

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/edison7009/EchoBird
- Категория: profiling
- Описание: One-click install + model switch:Claude Code,Codex CLI (OpenAI), Grok Build (xAI), DeepSeek Harness, Kimi Code (Moonshot) ,Qwen Code,Aider,OpenCode,MiMo Code (Xiaomi),ZCode (Z.AI),OpenClaw,Pi,OpenScience,Vibe-Trading,Claude Desktop (3P profile),ChatGPT desktop,OpenCode Desktop,
- Уровень: automated README evidence extraction
- Снимок: [sources/edison7009__EchoBird/README.md](sources/edison7009__EchoBird/README.md); SHA-256: `442f9f8461bd9dc1687eb916b6133e0d67407ada5083edefd28157ff786291aa`
- lifecycle: строка 59: Friends kept asking me to install **Claude Code**, **OpenClaw**, **Hermes Agent**… every machine was different, and some refused to pay for an LLM. Setup and explanations took forever. So I built **EchoBird** — an Agent inspired by **Songbird**, the genius net
- async: строка 41: Thanks to <strong>CompShare (优云智算)</strong> for sponsoring EchoBird! CompShare is UCloud's AI cloud platform, offering stable, comprehensive access to domestic and overseas model APIs through a single API key. Its flagship Coding Plan (monthly or pay-per-call)
- report: строка 89: any supported tool at it; no manual TOML / JSON editing, no per-CLI re-login.

## rbspy/rbspy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rbspy/rbspy
- Категория: profiling
- Описание: Sampling CPU profiler for Ruby
- Уровень: automated README evidence extraction
- Снимок: [sources/rbspy__rbspy/README.md](sources/rbspy__rbspy/README.md); SHA-256: `efd80fe8f170fd99b66d575ee1c4de9b3914b2ceff6bb607c1156fe7c88a90d6`

## serokell/deploy-rs

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/serokell/deploy-rs
- Категория: profiling
- Описание: A simple multi-profile Nix-flake deploy tool.
- Уровень: automated README evidence extraction
- Снимок: [sources/serokell__deploy-rs/README.md](sources/serokell__deploy-rs/README.md); SHA-256: `5d8390f607f215e05c293ec84c33e12f657663a09fbc13a7006a84f3134222c0`
- report: строка 63: There are full working deploy-rs Nix expressions in the [examples folder](./examples), and there is a JSON schema [here](./interface.json) which is used internally by the `deployChecks` mentioned above to validate your expressions.
- correctness: строка 63: There are full working deploy-rs Nix expressions in the [examples folder](./examples), and there is a JSON schema [here](./interface.json) which is used internally by the `deployChecks` mentioned above to validate your expressions.

## EmbarkStudios/puffin

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/EmbarkStudios/puffin
- Категория: profiling
- Описание: 🐦 Friendly little instrumentation profiler for Rust 🦀
- Уровень: automated README evidence extraction
- Снимок: [sources/EmbarkStudios__puffin/README.md](sources/EmbarkStudios__puffin/README.md); SHA-256: `f98492fde5bde0037672d16e2af136e7665f978deb60d726cc2de267e2068000`

## tikv/pprof-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tikv/pprof-rs
- Категория: profiling
- Описание: A Rust CPU profiler implemented with the help of backtrace-rs
- Уровень: automated README evidence extraction
- Снимок: [sources/tikv__pprof-rs/README.md](sources/tikv__pprof-rs/README.md); SHA-256: `9ed8c590bf96922ce38fdbd16ba0b1da125fb6d706744ddc0cb73d9358ee8d91`
- memory: строка 184: 1. `gperftools` is a collection of performance analysis tools which contains cpu profiler, heap profiler... `pprof-rs` focuses on cpu profiler now.
- report: строка 18: During the profiling time, you can get a report with the guard.

## AlexEne/twiggy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/AlexEne/twiggy
- Категория: profiling
- Описание: Twiggy🌱 is a code size profiler
- Уровень: automated README evidence extraction
- Снимок: [sources/AlexEne__twiggy/README.md](sources/AlexEne__twiggy/README.md); SHA-256: `29d3c4b4ff333c7f0febf52e51eb27b757af0d1d3e438600611783ca88a37529`

## nnethercote/dhat-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nnethercote/dhat-rs
- Категория: profiling
- Описание: Heap profiling and ad hoc profiling for Rust programs.
- Уровень: automated README evidence extraction
- Снимок: [sources/nnethercote__dhat-rs/README.md](sources/nnethercote__dhat-rs/README.md); SHA-256: `a6f2098b90c76e6f3fe70683e529b55eafe73ac76134ba5d5263a77e5f314331`
- memory: строка 9: This crate provides heap profiling and ad hoc profiling capabilities to Rust

## pythonspeed/filprofiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pythonspeed/filprofiler
- Категория: profiling
- Описание: A Python memory profiler for data processing and scientific computing applications
- Уровень: automated README evidence extraction
- Снимок: [sources/pythonspeed__filprofiler/README.md](sources/pythonspeed__filprofiler/README.md); SHA-256: `8a2a397c01ac855b6db140ffb48f55b31ac94f32f139240c81faf952ac9ccd96`
- memory: строка 4: In order to reduce memory usage, you first need to figure out:

## koute/not-perf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/koute/not-perf
- Категория: profiling
- Описание: A sampling CPU profiler for Linux
- Уровень: automated README evidence extraction
- Снимок: [sources/koute__not-perf/README.md](sources/koute__not-perf/README.md); SHA-256: `a67db6ec974c050864cb716c0d3c8b128b10ab1b94672f9df0970823ae880548`
- report: строка 27: on ARM then you also need to run `perf report` either on ARM or under QEMU,

## cmyr/cargo-instruments

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cmyr/cargo-instruments
- Категория: profiling
- Описание: A cargo plugin to generate Xcode Instruments trace files
- Уровень: automated README evidence extraction
- Снимок: [sources/cmyr__cargo-instruments/README.md](sources/cmyr__cargo-instruments/README.md); SHA-256: `5168af65600afcf3eb37142292c2fd9ea14a8e2fc308ddc63794a59e4a2be632`
- memory: строка 97: using the `Allocations` Instruments template:
- gpu: строка 208: Metal System Trace
- async: строка 213: Swift Concurrency

## westpoint-io/lazyrsync

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/westpoint-io/lazyrsync
- Категория: profiling
- Описание: 🦀 A friendly terminal UI for rsync, written in Rust. Reusable profiles, an honest dry-run diff, and live progress, even over SSH.
- Уровень: automated README evidence extraction
- Снимок: [sources/westpoint-io__lazyrsync/README.md](sources/westpoint-io__lazyrsync/README.md); SHA-256: `67e1a89fe8ba17ee650d8ba4f57115b49ae2c49453deabe5918b4b7bbb283074`
- report: строка 444: shown in the TUI and `lazyrsync list` reports.
- correctness: строка 361: checksum = false

## llogiq/flame

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/llogiq/flame
- Категория: profiling
- Описание: An intrusive flamegraph profiling tool for rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/llogiq__flame/readme.md](sources/llogiq__flame/readme.md); SHA-256: `9e3bbb8fac1e95ef7922082fc3c18d6b1e846dd0779a63721eb3a6abaf794b67`
- report: строка 10: program repeatedly and reports on every function in your callstack,

## pop-os/system76-power

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pop-os/system76-power
- Категория: profiling
- Описание: Power profile management for Linux
- Уровень: automated README evidence extraction
- Снимок: [sources/pop-os__system76-power/README.md](sources/pop-os__system76-power/README.md); SHA-256: `cd33838f4a00500403675dcf7c85bf037d2a27b7e146cb0a280b48ffa65e0dcb`
- gpu: строка 36: render on the dGPU even when requested. Vulkan applications must be launched
- report: строка 47: `supported-gpus.json` file provided by the driver. e.g.:

## supermemoryai/smfs

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/supermemoryai/smfs
- Категория: profiling
- Описание: A filesystem designed for agents, with SOTA retrieval, automatic memory profiles, sync engine. Drop any file type (pdf, images, videos), and grep through them.
- Уровень: automated README evidence extraction
- Снимок: [sources/supermemoryai__smfs/README.md](sources/supermemoryai__smfs/README.md); SHA-256: `431bfb8ca8383369b1c3a0e8f3974cec3c65535e80308dd085d2fedaa0569954`
- report: строка 101: --memory-paths "<csv>"   which paths produce memories (see above)

## svenstaro/cargo-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/svenstaro/cargo-profiler
- Категория: profiling
- Описание: Cargo subcommand to profile binaries
- Уровень: automated README evidence extraction
- Снимок: [sources/svenstaro__cargo-profiler/README.md](sources/svenstaro__cargo-profiler/README.md); SHA-256: `b31b6db8c8c653680d5849ee19f6be99b67e0517533bff72bc90f23dca75340f`

## aclysma/profiling

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/aclysma/profiling
- Категория: profiling
- Описание: Provides a very thin abstraction over instrumented profiling crates like puffin, optick, tracy, and superluminal-perf.
- Уровень: automated README evidence extraction
- Снимок: [sources/aclysma__profiling/README.md](sources/aclysma__profiling/README.md); SHA-256: `970c9abdb06b09f7cdb52ab5b5d578e2c928d0b909320c24fcdac9e526d45b1a`
- lifecycle: строка 157: in code to setup and configure a backend.
- gpu: строка 222: As a point of reference, currently the most popular crate relying on profiling is wgpu, and their

## rust-lang/measureme

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rust-lang/measureme
- Категория: profiling
- Описание: Support crate for rustc's self-profiling feature
- Уровень: automated README evidence extraction
- Снимок: [sources/rust-lang__measureme/README.md](sources/rust-lang__measureme/README.md); SHA-256: `1db7f79c1130680a4502af7fe1ab18ebcab653c3d354199d990bd6e5cf0672fe`

## desbma/shh

- Итог: excluded — Systemd hardening rather than performance measurement
- Источник: https://github.com/desbma/shh
- Категория: profiling
- Описание: Systemd Hardening Helper - Automatic systemd service hardening guided by strace profiling
- Уровень: automated README evidence extraction
- Снимок: [sources/desbma__shh/README.md](sources/desbma__shh/README.md); SHA-256: `56cb290b718084e9eeec3a70179ee60fa2c6007c55de30e91d5f17d00dbee624`

## HdrHistogram/HdrHistogram_rust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/HdrHistogram/HdrHistogram_rust
- Категория: profiling
- Описание: A port of HdrHistogram to Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/HdrHistogram__HdrHistogram_rust/README.md](sources/HdrHistogram__HdrHistogram_rust/README.md); SHA-256: `d043d5202136a975e6be843780de4460321fc35ddfa536bf5bd364f561b37aed`
- statistics: строка 101: At any time, the histogram can be queried to return interesting statistical measurements, such
- memory: строка 39: space and time. A Histogram's memory footprint is constant, with no allocation operations
- async: строка 11: accurate analysis of the extreme ranges of data with non-normal distributions, like latency.
- report: строка 185: convenient if we implemented some relevant traits (CSV, JSON, and possibly simple

## nnethercote/counts

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nnethercote/counts
- Категория: profiling
- Описание: A tool for ad hoc profiling
- Уровень: automated README evidence extraction
- Снимок: [sources/nnethercote__counts/README.md](sources/nnethercote__counts/README.md); SHA-256: `799fb27fd6238b035c394575aad025ab9b74eeb8dc2ecad4ca760f8c64e7b4af`
- memory: строка 96: As an example, I added print statements to Firefox's heap allocator so it

## ZakisM/bl3_save_edit

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ZakisM/bl3_save_edit
- Категория: profiling
- Описание: Borderlands 3 Save/Profile Editor for Windows/MacOS and Linux!
- Уровень: automated README evidence extraction
- Снимок: [sources/ZakisM__bl3_save_edit/README.md](sources/ZakisM__bl3_save_edit/README.md); SHA-256: `81fd9fd4fb0e351832e122807169df161e41f083d7947270650806661d644b9e`

## bluenote-1577/sylph

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/bluenote-1577/sylph
- Категория: profiling
- Описание: ultrafast taxonomic profiling and genome querying for metagenomic samples by abundance-corrected minhash.
- Уровень: automated README evidence extraction
- Снимок: [sources/bluenote-1577__sylph/README.md](sources/bluenote-1577__sylph/README.md); SHA-256: `ffe3ddb226ce423f3adf43bee1863fe5b247fa1f208246bf0505a24dc751f913`
- statistics: строка 38: sylph uses a k-mer containment method. sylph's novelty lies in **using a statistical technique to estimate k-mer containment for low coverage genomes** , giving accurate results for low abundance organisms. See [here for more information on what sylph can and
- comparison: строка 50: # multi-sample paired-end profiling (sylph version >= 0.6)

## polarsignals/rust-jemalloc-pprof

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/polarsignals/rust-jemalloc-pprof
- Категория: profiling
- Описание: Convert jemalloc heap profiles to pprof to understand memory usage, fix memory leaks, and fix OOM Kills.
- Уровень: automated README evidence extraction
- Снимок: [sources/polarsignals__rust-jemalloc-pprof/README.md](sources/polarsignals__rust-jemalloc-pprof/README.md); SHA-256: `655f6e57f337ffbd6344427a53a3c7a0cfa82f395e271aa6f78eeb2fe64027ff`
- lifecycle: строка 150: Polar Signals Cloud supports anything in the pprof format, so a process exposing the above explained pprof endpoint, can then be scraped as elaborated in the [scraping docs](https://www.polarsignals.com/docs/setup-scraper).
- memory: строка 5: A rust library to collect and convert Heap profiling data from the [jemalloc](https://jemalloc.net/) allocator and convert it to the [pprof](https://github.com/google/pprof/tree/main/proto) format.
- async: строка 53: async fn main() {

## TheYkk/git-switcher

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/TheYkk/git-switcher
- Категория: profiling
- Описание: Easily switch between your git profiles
- Уровень: automated README evidence extraction
- Снимок: [sources/TheYkk__git-switcher/README.md](sources/TheYkk__git-switcher/README.md); SHA-256: `19ec41e2de407ded63c2c367a4c14dac6b9a2dc8b560072793fae89709173c94`

## grafana/pyroscope-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/grafana/pyroscope-rs
- Категория: profiling
- Описание: Pyroscope Profiler for Rust. Profile your Rust applications.
- Уровень: automated README evidence extraction
- Снимок: [sources/grafana__pyroscope-rs/README.md](sources/grafana__pyroscope-rs/README.md); SHA-256: `64f95bb02229d13e28945785f115fb3ba00c91dd529e8b423269bc081a2a0d73`

## alexcrichton/coz-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alexcrichton/coz-rs
- Категория: profiling
- Описание: Rust support for the coz Causal profiler, code now lives upstream -- https://github.com/plasma-umass/coz
- Уровень: automated README evidence extraction
- Снимок: [sources/alexcrichton__coz-rs/README.md](sources/alexcrichton__coz-rs/README.md); SHA-256: `40fa5cd6a61423768b331d9c99c7d5d8cd6b6a97e7108098d92493d0b425f168`
- async: строка 27: Then you'll want to either at throughput or latency tracepoints. More
- report: строка 88: Known caveats so far to generate a report that collects information are:

## siketyan/ghr

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/siketyan/ghr
- Категория: profiling
- Описание: 🚀 Yet another repository management with auto-attaching profiles.
- Уровень: automated README evidence extraction
- Снимок: [sources/siketyan__ghr/README.md](sources/siketyan__ghr/README.md); SHA-256: `12e09d704b80340c1ff0953032367473d05d4ea86a4608d6b36c8f2f1fa6db29`
- report: строка 131: -q, --quiet    Operates quietly. Errors will be reported even if this option is enabled

## hw0lff/shikane

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/hw0lff/shikane
- Категория: profiling
- Описание: [mirror] A deterministic dynamic output configuration tool that automatically detects and configures connected outputs based on a set of profiles.
- Уровень: automated README evidence extraction
- Снимок: [sources/hw0lff__shikane/README.md](sources/hw0lff__shikane/README.md); SHA-256: `001a965bf26a58c377fb72cdc7c23b1d38fd1ad3bb9c619859865f021a217c03`
- lifecycle: строка 22: - export current display setup as shikane config.toml

## Wumpf/wgpu-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Wumpf/wgpu-profiler
- Категория: profiling
- Описание: Simple profiler scopes for wgpu using timer queries
- Уровень: automated README evidence extraction
- Снимок: [sources/Wumpf__wgpu-profiler/README.md](sources/Wumpf__wgpu-profiler/README.md); SHA-256: `a09884b92f604003df1a4e8924a8073273ac61d4a26c60819162113af58915fc`
- gpu: строка 1: # wgpu-profiler
- report: строка 18: * chrome trace flamegraph json export

## null-dev/firefox-profile-switcher-connector

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/null-dev/firefox-profile-switcher-connector
- Категория: profiling
- Описание: Native connector software for the 'Profile Switcher for Firefox' extension.
- Уровень: automated README evidence extraction
- Снимок: [sources/null-dev__firefox-profile-switcher-connector/README.md](sources/null-dev__firefox-profile-switcher-connector/README.md); SHA-256: `e37cce32c8c7c1e90353b898b0cb3fc7fd482fda02d8ac65cbe36b5931422b2f`

## elastic/devfiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/elastic/devfiler
- Категория: profiling
- Описание: Universal Profiling as a desktop app
- Уровень: automated README evidence extraction
- Снимок: [sources/elastic__devfiler/README.md](sources/elastic__devfiler/README.md); SHA-256: `dd2bc3c2602ca722335bf3ede69f5a0024889e8924840ff39368be8bc8558c42`

## shvbsle/k10s

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/shvbsle/k10s
- Категория: profiling
- Описание: Profiler for ML Training jobs in pure Rust. See straggler ranks and idle GPUs.
- Уровень: automated README evidence extraction
- Снимок: [sources/shvbsle__k10s/README.md](sources/shvbsle__k10s/README.md); SHA-256: `3f1866daf223442eb34d94409153f56e33a4d9d25e334ec5ff3d862d9a9b2220`
- gpu: строка 1: # k10s: GPU-Aware Kubernetes Toolkit
- async: строка 48: | ✅ | Network latency | `net.tcp_rtt_us` | Jumps from ~2ms to 50ms+ on the affected rank |

## thoren-d/tracing-chrome

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/thoren-d/tracing-chrome
- Категория: profiling
- Описание: A library for generating chrome://tracing traces in Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/thoren-d__tracing-chrome/README.md](sources/thoren-d__tracing-chrome/README.md); SHA-256: `b4c602b4a835ab83f1318d954dd15b4f2a6d04dc21024bd024f6d9d52b49ccdb`
- report: строка 24: When `_guard` is dropped, your trace will be in a file like `trace-1668480819035032.json`.

## javierhonduco/rbperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/javierhonduco/rbperf
- Категория: profiling
- Описание: Low-overhead sampling profiler and tracer for Ruby for Linux
- Уровень: automated README evidence extraction
- Снимок: [sources/javierhonduco__rbperf/README.md](sources/javierhonduco__rbperf/README.md); SHA-256: `435ac21e221b785fffa30e5ec0a53db6b65ef50302ccaf4f5b234de053f29e36`

## jzbor/nix-sweep

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/jzbor/nix-sweep
- Категория: profiling
- Описание: Utility to clean up old Nix profile generations and left-over garbage collection roots
- Уровень: automated README evidence extraction
- Снимок: [sources/jzbor__nix-sweep/README.md](sources/jzbor__nix-sweep/README.md); SHA-256: `98e670e4fb2dde756d054eca68014bd043fa606aabbe6454e7f4edf36e40162b`
- report: строка 39: If you have any feedback, ideas or bugreports feel free to open a [new issue](https://github.com/jzbor/nix-sweep/issues/new)

## lucifer1004/VeloQ

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lucifer1004/VeloQ
- Категория: profiling
- Описание: Agent-friendly GPU profile-query CLI
- Уровень: automated README evidence extraction
- Снимок: [sources/lucifer1004__VeloQ/README.md](sources/lucifer1004__VeloQ/README.md); SHA-256: `f7e2c660be0cbe9f9c5b624ff149bb69cc6c7fde4aeaffc44994203ada5d0c64`
- lifecycle: строка 59: | Zero setup per query         |     ✓      |                ✓                 |            ✗            |                 ✓                  |
- gpu: строка 11: VeloQ is designed for coding agents and scripts that need GPU profile
- async: строка 273: veloq metrics path/to/trace.nsys-rep --type gpu --counter '*Throughput*' --bucket 50ms
- report: строка 7: <p align="center"><em>Pure CLI in / JSON contract out; no GUI required.</em></p>
- correctness: строка 44: `parquetdir/` child with ctime invalidation.

## soth-ai/mcp-reticle

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/soth-ai/mcp-reticle
- Категория: profiling
- Описание: Reticle intercepts, visualizes, and profiles JSON-RPC traffic between your LLM and MCP servers in real-time, with zero latency overhead. Stop debugging blind. Start seeing everything.
- Уровень: automated README evidence extraction
- Снимок: [sources/soth-ai__mcp-reticle/README.md](sources/soth-ai__mcp-reticle/README.md); SHA-256: `355aa65a23570cce492d6c38aba84e612e66af96e8c14ce809bacd3e79e585ae`
- lifecycle: строка 173: - Dev setup and commands: [Development guide](https://github.com/labterminal/mcp-reticle/wiki/Development)
- async: строка 62: - profile latency and token estimates
- report: строка 36: Reticle intercepts, visualizes, and profiles MCP JSON-RPC traffic in real time — designed for microsecond-level overhead.

## sassman/amoxide-rs

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/sassman/amoxide-rs
- Категория: profiling
- Описание: amoxide (am) helps to manage your shell aliases either globally, profile- or project-specific. It loads context specific relevant aliases automatically
- Уровень: automated README evidence extraction
- Снимок: [sources/sassman__amoxide-rs/README.md](sources/sassman__amoxide-rs/README.md); SHA-256: `f277f4fce643c1f2b93c8312b001523fd5745c53a70f356447de54e1f3833de6`
- lifecycle: строка 52: - [Getting Started](https://amoxide.rs/guide/) — install, shell setup, first aliases

## burakdede/aisw

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/burakdede/aisw
- Категория: profiling
- Описание: AISW | AI Switcher - Switch between multiple Claude Code, Codex CLI, Antigravity and Gemini CLI accounts in one command. Named profile manager for AI coding agents.
- Уровень: automated README evidence extraction
- Снимок: [sources/burakdede__aisw/README.md](sources/burakdede__aisw/README.md); SHA-256: `31800b62cafb505b524462ef1e1040d1b84c6d094a4bf76b08f1cd8db018c34c`
- statistics: строка 142: # Bootstrap ~/.aisw/, install shell integration, and import
- lifecycle: строка 207: Use these when your setup is a little different from the normal two-account OAuth flow above.
- report: строка 42: The underlying problem is not just "multiple accounts." It is that each upstream CLI stores auth differently, in different places, with different side effects. Manual switching usually means editing hidden files, copying `auth.json`, juggling `CLAUDE_CONFIG_DI
- isolation: строка 274: For GUI or other subprocess-driven clients, `aisw` also exposes machine-oriented commands such as:

## aralroca/aralroca

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/aralroca/aralroca
- Категория: profiling
- Описание: My GitHub profile
- Уровень: automated README evidence extraction
- Снимок: [sources/aralroca__aralroca/README.md](sources/aralroca__aralroca/README.md); SHA-256: `5ef8f51858b221661c75bc9f7b5859f3eb7ac455259cd1aa5439a51999aa153e`

## EmbarkStudios/superluminal-perf-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/EmbarkStudios/superluminal-perf-rs
- Категория: profiling
- Описание: 🔆 Superluminal Performance profiler Rust API 🦀
- Уровень: automated README evidence extraction
- Снимок: [sources/EmbarkStudios__superluminal-perf-rs/README.md](sources/EmbarkStudios__superluminal-perf-rs/README.md); SHA-256: `ef59a74e47991798c2e5095edbf5975fa781c420fef2f10dc56b99d0a82a27da`

## gyroflow/lens_profiles

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/gyroflow/lens_profiles
- Категория: profiling
- Описание: Lens profile database for Gyroflow
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## 0xdeafbeef/jeprofl

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/0xdeafbeef/jeprofl
- Категория: profiling
- Описание: Allocations profiler built using ebpf
- Уровень: automated README evidence extraction
- Снимок: [sources/0xdeafbeef__jeprofl/README.md](sources/0xdeafbeef__jeprofl/README.md); SHA-256: `cc67ff131eda596306e6f902af27c51c39fc8a3788fc28aa33740eeaf34b6767`
- memory: строка 3: jeprofl is a memory allocation profiling tool that uses eBPF technology to
- report: строка 21: - Generate CSV output and flame graphs

## dr-dotnet/dr-dotnet

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dr-dotnet/dr-dotnet
- Категория: profiling
- Описание: 🩺 One-click diagnosis of your dotnet applications. Works both locally or remotely as a web service. Based on the lowest level dotnet profiling APIs and using the rust language 🦀 for a minimal runtime penalty.
- Уровень: automated README evidence extraction
- Снимок: [sources/dr-dotnet__dr-dotnet/README.md](sources/dr-dotnet__dr-dotnet/README.md); SHA-256: `72035dd5b471b679531c745b90f25fb47aa1a3735f902811882753f0c8b0fb8d`

## rogercoll/eprofiler-tui

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rogercoll/eprofiler-tui
- Категория: profiling
- Описание: eBPF profiler flamegraph based TUI 🐧🐝
- Уровень: automated README evidence extraction
- Снимок: [sources/rogercoll__eprofiler-tui/README.md](sources/rogercoll__eprofiler-tui/README.md); SHA-256: `3b30426ab807b007e96d910eae392203c69d584656f56fe83b1afce17ee9e079`

## MarlinDiary/worklouder-input-cli

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/MarlinDiary/worklouder-input-cli
- Категория: profiling
- Описание: Full-configuration CLI for Codex Micro and Work Louder Input: profiles, layers, keymaps, actions, provider handoff, verified apply, and rollback.
- Уровень: automated README evidence extraction
- Снимок: [sources/MarlinDiary__worklouder-input-cli/README.md](sources/MarlinDiary__worklouder-input-cli/README.md); SHA-256: `f065c252ab58b737ee61c8129d80e2027c98a9ca3c3a4f0312d99ed06edce129`
- comparison: строка 179: a per-user device-operation lock, so a macOS focus event cannot interleave with
- report: строка 51: | **Automation** | Stable JSON output, JSON Schemas, shell-free agent envelopes and generated Bash/Zsh/Fish completions |
- correctness: строка 73: checksum, fixed archive inventory, manifest, Developer ID signature, and binary

## ShayBox/Wooting-Profile-Switcher

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ShayBox/Wooting-Profile-Switcher
- Категория: profiling
- Описание: Automatically switch Wooting keyboard profiles based on focused window
- Уровень: automated README evidence extraction
- Снимок: [sources/ShayBox__Wooting-Profile-Switcher/README.md](sources/ShayBox__Wooting-Profile-Switcher/README.md); SHA-256: `de1fe54583a2e979b71b87b836020efcbf2666ccef7dd76695fc74cc9c2e6a5e`
- report: строка 39: ```json5

## emberian/hprof

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/emberian/hprof
- Категория: profiling
- Описание: A real-time hierarchical profiler
- Уровень: automated README evidence extraction
- Снимок: [sources/emberian__hprof/README.md](sources/emberian__hprof/README.md); SHA-256: `3afa15902c23c2f0f6b278246985bae39f81b13ecfef17d5f166c718e350160e`
- lifecycle: строка 58: setup - 1133523ns (6.725068%)
- gpu: строка 35: - GPU wait
- report: строка 45: Studios. They report having thousands of profile nodes active at a time.

## alumet-dev/alumet

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alumet-dev/alumet
- Категория: profiling
- Описание: Next-gen monitoring with high-frequency energy measurement.
- Уровень: automated README evidence extraction
- Снимок: [sources/alumet-dev__alumet/README.md](sources/alumet-dev__alumet/README.md); SHA-256: `b0fe45352bb1458a27141bb40b0260db4f6fba73c4a1ab076649d74a05b93269`
- lifecycle: строка 28: - **Genericity**: Alumet's core does not depend on a specific hardware nor on a single software stack. You can leverage a common tool and data model for every setup.
- memory: строка 16: - system metrics (CPU usage, memory usage, network bandwidth, etc.)
- gpu: строка 19: - energy metrics (e.g. real-time energy consumption of your CPU or GPU)

## yfractal/sdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yfractal/sdb
- Категория: profiling
- Описание: A Ruby stack profiler without GVL.
- Уровень: automated README evidence extraction
- Снимок: [sources/yfractal__sdb/README.md](sources/yfractal__sdb/README.md); SHA-256: `a0eaea6477d5539981973a51d76097675cb351741a55a96576a8ec85106d317d`
- async: строка 13: SDB, inspired by [LDB](https://www.usenix.org/conference/nsdi24/presentation/cho#:~:text=LDB%20observes%20the%20latency%20of,costs%20away%20from%20program%20threads.), operates by pulling stack traces on a separate thread, which minimizes the impact on our app

## Th0rgal/shard

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/Th0rgal/shard
- Категория: profiling
- Описание: Minecraft launcher with deduplicated mod storage. Content-addressed library stores mods once, shares across profiles. CLI + desktop app. Supports Fabric, Forge, Quilt, NeoForge. Modrinth & CurseForge integration.
- Уровень: automated README evidence extraction
- Снимок: [sources/Th0rgal__shard/README.md](sources/Th0rgal__shard/README.md); SHA-256: `b1dc25351b7d7d03d7a1ab1d710617f2edfdc223463a678cccb597dc81d8ca99`
- lifecycle: строка 46: Your entire setup is a single JSON file. Version control it with Git, share it with friends, diff changes between versions, restore it anytime. Profiles are declarative: the launcher materializes clean instances on demand.
- report: строка 34: Shard is an open-source Minecraft launcher with a global deduplicated library and declarative profiles (plain JSON). It materializes clean instances from a single source of truth, integrates with Modrinth and CurseForge, and supports scriptable workflows via a
- correctness: строка 8: <strong>Reproducible profiles. One deduplicated library. Scriptable workflows.</strong><br>

## AlexEne/rust_hawktracer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/AlexEne/rust_hawktracer
- Категория: profiling
- Описание: Rust bindings for Amazon's Hawktracer profiler
- Уровень: automated README evidence extraction
- Снимок: [sources/AlexEne__rust_hawktracer/README.md](sources/AlexEne__rust_hawktracer/README.md); SHA-256: `0eaafe7706c1ac9071ccae8d68d99b175651b6e8a0c1abafe22b9408b8ef7903`
- report: строка 81: .\hawktracer-converter.exe --source trace.bin --output trace.json

## zz85/profile-bee

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/zz85/profile-bee
- Категория: profiling
- Описание: 🐝🦀🔥 Rust-based eBPF CPU profiler, supports stack unwinding
- Уровень: automated README evidence extraction
- Снимок: [sources/zz85__profile-bee/README.md](sources/zz85__profile-bee/README.md); SHA-256: `1e7b86449e4fc231c848affffabc7e3c812ba73e1aebd5c4209a448e5e25deb9`
- lifecycle: строка 82: # Continuous profiling to Pyroscope (pre-symbolized, simplest setup)
- async: строка 121: - **async-profiler engine (experimental)** — `--java-engine async-profiler` delegates Java stacks to [async-profiler](https://github.com/async-profiler/async-profiler) (AsyncGetCallTrace), so JIT/interpreter/inlined frames resolve **without `-XX:+PreserveFrame
- report: строка 15: - Outputs to interactive TUI, SVG, HTML, JSON, stackcollapse, pprof, or a real-time web server — just use `-o file.svg` and the format is inferred from the extension
- correctness: строка 430: The cache integrates with eBPF lifecycle events: exec events invalidate entries (same PID, new binary), exit events remove them. PID reuse is detected via `/proc/[pid]/stat` starttime comparison.

## DataDog/libdatadog

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DataDog/libdatadog
- Категория: profiling
- Описание: Datadog shared rust-based library. For now only used in other products (e.g. Ruby or PHP libraries).
- Уровень: automated README evidence extraction
- Снимок: [sources/DataDog__libdatadog/README.md](sources/DataDog__libdatadog/README.md); SHA-256: `8020c77e801b56383dffb49e9fcbe4b2b3f86865871d1d225bb75bb7348311f2`
- correctness: строка 98: The Nix flake provides a reproducible, pinned shell with Rust, `cbindgen`, and native build tools.

## kguardian-dev/kguardian

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/kguardian-dev/kguardian
- Категория: profiling
- Описание: A Kubernetes tool leveraging eBPF for advanced Kubernetes security, auto-generating Network Policies, Seccomp Profiles, and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/kguardian-dev__kguardian/README.md](sources/kguardian-dev__kguardian/README.md); SHA-256: `a7248fab9ea2505861ce0fd7591c15f1ebf256aaef8a6fc14d45f64f1308b097`
- comparison: строка 39: It's built for platform and security teams who want policy-as-code without writing rules by hand: the Controller (an eBPF DaemonSet) captures every TCP/UDP connection and syscall on each node, the Broker stores the per-pod baseline in PostgreSQL, and the `kube
- lifecycle: строка 139: Review the generated YAML, then apply it yourself (`kubectl apply -f ./policies`). Manual download, custom Helm values, Kind setup, verification, upgrades, and uninstall are covered in the [Installation Guide](https://docs.kguardian.dev/installation).
- memory: строка 69: - **Seccomp profile generation** — per-workload syscall allowlists derived from runtime traces, exported as a `SeccompProfile` CR you commit; the controller places the file on every node only once you apply it. Capture is tiered (`full` by default, cheap thank
- report: строка 70: - **Policy auditing before enforcement** — the `AuditNetworkPolicy` CRD is byte-identical to an upstream `NetworkPolicy`, but instead of dropping packets the evaluator reports every flow the policy *would* deny. Ship policies with confidence instead of blackho

## xjoker/codex-switch

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/xjoker/codex-switch
- Категория: profiling
- Описание: Multi-account profile manager for OpenAI Codex CLI with usage dashboard and interactive TUI.
- Уровень: automated README evidence extraction
- Снимок: [sources/xjoker__codex-switch/README.md](sources/xjoker__codex-switch/README.md); SHA-256: `8aeb94d766d66c89632d7093b72705485e728eaf5319e13a4dc9c72ee223c6f2`
- lifecycle: строка 49: - Supports reset cards, quota warmup, JSON output, proxies, and a Beta background daemon (LaunchAgent, systemd, or Windows Task Scheduler; tune `cache_refresh_interval_secs` and `auto_warmup`).
- report: строка 7: > `codex-switch` manages local authentication files. Never publish profiles, `auth.json`, tokens, proxy credentials, or unredacted debug output.

## zmitchell/proctrace

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/zmitchell/proctrace
- Категория: profiling
- Описание: A high-level profiler for process-level events such as fork, exec, exit, setpgid, and setsid
- Уровень: automated README evidence extraction
- Снимок: [sources/zmitchell__proctrace/README.md](sources/zmitchell__proctrace/README.md); SHA-256: `d6eb5197afee2e91e8c93aaf3578913f4b46fb55b64256d9a647622c4e3cd9fb`

## naftulikay/aws-env

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/naftulikay/aws-env
- Категория: profiling
- Описание: A utility script for exporting an AWS profile as environment variables.
- Уровень: automated README evidence extraction
- Снимок: [sources/naftulikay__aws-env/README.md](sources/naftulikay__aws-env/README.md); SHA-256: `2dbb80dec2b468e1769761092413a12a1d0a5e8d9a1a4a1729a36364d3895396`
- report: строка 68: -F, --format <format>    The output format [default: table]  [possible values: table, plain, csv, json]

## gleich/profile_stack

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/gleich/profile_stack
- Категория: profiling
- Описание: 🚀 Display your tech stack on your GitHub profile's README
- Уровень: automated README evidence extraction
- Снимок: [sources/gleich__profile_stack/README.md](sources/gleich__profile_stack/README.md); SHA-256: `61015f1b26a1793aeda417e94a0494f1b4bd01522ed2a18952169bf235fda543`

## LuoGroup2023/PanTax

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/LuoGroup2023/PanTax
- Категория: profiling
- Описание: PanTax: Strain-level metagenomic profiling using pangenome graphs
- Уровень: automated README evidence extraction
- Снимок: [sources/LuoGroup2023__PanTax/README.md](sources/LuoGroup2023__PanTax/README.md); SHA-256: `e8ccad26b3a775fa2d9d686b939e2bd2d1970c4f6fdc0c90fc97cf7bd68eac8f`
- comparison: строка 252: -p, --paired
- report: строка 301: -R, --report <PANTAX_REPORT>

## kornelski/rust-lcms2

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/kornelski/rust-lcms2
- Категория: profiling
- Описание: ICC color profiles in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/kornelski__rust-lcms2/README.md](sources/kornelski__rust-lcms2/README.md); SHA-256: `9ff0ef45b3ddfc8f33dd9f2f08f70e5cff5c20906967971c9e8480a45ea43d42`

## coolreader18/flamescope

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/coolreader18/flamescope
- Категория: profiling
- Описание: Export flame data to speedscope's format
- Уровень: automated README evidence extraction
- Снимок: [sources/coolreader18__flamescope/README.md](sources/coolreader18__flamescope/README.md); SHA-256: `90c6315c178ba1ca977b6bb6e48c3127f22123ff7e1adc0bd81d50d0fda885f6`
- report: строка 19: flamescope::dump(&mut File::create("flamescope.json").unwrap()).unwrap();

## yaahc/spandoc

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yaahc/spandoc
- Категория: profiling
- Описание: Proc macro for using doc comments as context for errors/logs/profiling/whatever via `tracing`
- Уровень: automated README evidence extraction
- Снимок: [sources/yaahc__spandoc/README.md](sources/yaahc__spandoc/README.md); SHA-256: `5f64d8235c771397a491ae50d565f18aff565e106076b855a7ca312440256fe1`
- async: строка 85: will not recurse into `async` blocks.

## cleyton1986/predator-sense

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/cleyton1986/predator-sense
- Категория: profiling
- Описание: 🎮 Predator Sense for Linux — Unofficial hardware control for Acer Predator/Helios/Nitro gaming laptops. RGB keyboard, GPU monitoring, performance profiles, fan control. Built with Rust + GTK4.
- Уровень: automated README evidence extraction
- Снимок: [sources/cleyton1986__predator-sense/README.md](sources/cleyton1986__predator-sense/README.md); SHA-256: `b379dd813085fac3a6d4167772becd02396b4f0300f0ea4a5edfd8bd5083bbf3`
- statistics: строка 243: The installer, privileged helper, hotkey listener, and tray service are all provided by the same Rust multicall binary. The installer downloads and configures everything without a shell-script bootstrap.
- lifecycle: строка 191: If that's your case, the community [Linuwu-Sense](https://github.com/0x7375646F/Linuwu-Sense) module (loaded with `predator_v4=1`) exposes the full profile set through the same generic `platform_profile`/`intel_pstate`/`acer-wmi-battery` interfaces this app al
- gpu: строка 49: <p align="center"><b>Dashboard</b> — Laptop photo and full system specs at a glance: CPU, GPU, RAM, storage, network and OS.</p>
- report: строка 39: > The laptop photos under `predator-sense-gui/resources/models/` depict official Acer Predator/Nitro products and are used solely to let the app visually identify the model detected on the user's own machine (matched against the `product_name` reported by the
- correctness: строка 121: | **AI Assistant** 🧪 | Local, opt-in AI assistant powered by [Ollama](https://ollama.com) — reads live hardware state and suggests or applies changes through a fixed, already-validated set of actions (thermal profile, fan mode, CoolBoost, RGB, GPU power limit,

## Tails8521/md-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Tails8521/md-profiler
- Категория: profiling
- Описание: A tracing profiler for the Sega MegaDrive/Genesis
- Уровень: automated README evidence extraction
- Снимок: [sources/Tails8521__md-profiler/README.md](sources/Tails8521__md-profiler/README.md); SHA-256: `3873c709bd5c501b411c50feed1004ddc3925ae5b4ae8bb998ada0af3658ff74`
- report: строка 38: ## Generating the json trace

## wasmerio/loupe

- Итог: excluded — README too thin for content analysis
- Источник: https://github.com/wasmerio/loupe
- Категория: profiling
- Описание: Profiling tool for Rust code.
- Уровень: automated README evidence extraction
- Снимок: [sources/wasmerio__loupe/README.md](sources/wasmerio__loupe/README.md); SHA-256: `43bc9f3bedbd1b297495300e928170b66a2210fbabe1cbc9d8bb72640476193b`

## Arc-blroth/memory-stats

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Arc-blroth/memory-stats
- Категория: profiling
- Описание: A cross-platform-ish memory profiler for Rust!
- Уровень: automated README evidence extraction
- Снимок: [sources/Arc-blroth__memory-stats/README.md](sources/Arc-blroth__memory-stats/README.md); SHA-256: `53d7594b7d0710b83ed7f944e2d6d4036b269cabe46f70328ad92527ca540445`
- memory: строка 28: Here's an example that prints out the current memory usage:

## javierhonduco/lightswitch

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/javierhonduco/lightswitch
- Категория: profiling
- Описание: Lightweight CPU profiler for Linux as a library
- Уровень: automated README evidence extraction
- Снимок: [sources/javierhonduco__lightswitch/README.md](sources/javierhonduco__lightswitch/README.md); SHA-256: `7311a18a7e7c5f67470da2f1a263a9e405cbc2cf2007aa48d49666b43fca2f71`
- report: строка 37: Stop it with <kbd>Ctrl</kbd>+<kbd>C</kbd>, or alternatively, pass a `--duration` in seconds. By default, a flamegraph in SVG format will be written to disk. Pprof, Firefox Profiler, and Perfetto traces are also supported with `--profile-format=pprof`, `--profi

## terror/vim-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/terror/vim-profiler
- Категория: profiling
- Описание: A vim plugin profiler and data plotter
- Уровень: automated README evidence extraction
- Снимок: [sources/terror__vim-profiler/README.md](sources/terror__vim-profiler/README.md); SHA-256: `c2a91f6abe97dd07c29ae400f54a4be4e0994335b652dc094de95e779e5a7fac`
- report: строка 43: -e, --export    <path>         Export the results to a CSV file

## roniel-rhack/envi

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/roniel-rhack/envi
- Категория: profiling
- Описание: A terminal UI for managing .env files — diff, scan, edit, and validate environment variables across projects and profiles.
- Уровень: automated README evidence extraction
- Снимок: [sources/roniel-rhack__envi/README.md](sources/roniel-rhack__envi/README.md); SHA-256: `476616572236c91d0b1cb98d0c67ffb27551726fd0b3c9572646a3c2c7fcf431`
- report: строка 28: **envi** gives you a single dashboard for all of it.
- correctness: строка 7: Diff, scan, edit, and validate environment variables across all your profiles — right from the terminal.

## inceptyon-labs/TARS

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/inceptyon-labs/TARS
- Категория: profiling
- Описание: TARS is a centralized hub for discovering, creating, editing, and managing Claude Code resources. It provides a visual interface for managing skills, agents, commands, hooks, MCP servers, and plugins across multiple projects with safe apply/rollback operations and profile-based configuration sharing.
- Уровень: automated README evidence extraction
- Снимок: [sources/inceptyon-labs__TARS/README.md](sources/inceptyon-labs__TARS/README.md); SHA-256: `c3e375fc9e5509a3b4c023dcef7db766ff98f9272d1d0e469bebaffa57dd1eee`
- lifecycle: строка 111: Profiles let you create reusable configuration bundles that can be shared across multiple projects. Think of them as "presets" for your Claude Code setup that install as native Claude Code plugins through a local marketplace.
- gpu: строка 193: - **TARS design system** - Brushed metal aesthetic inspired by Interstellar
- async: строка 368: - Tokio (async runtime)
- report: строка 54: - Test coverage from lcov.info or tarpaulin reports
- correctness: строка 64: - **Real-time YAML validation** - Inline error highlighting for frontmatter syntax issues

## Christopher-06/rustmeter

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Christopher-06/rustmeter
- Категория: profiling
- Описание: A profiling and tracing system for Embedded Rust that visualizes Embassy tasks, function calls, metrics and defmt logs in the Perfetto UI.
- Уровень: automated README evidence extraction
- Снимок: [sources/Christopher-06__rustmeter/README.md](sources/Christopher-06__rustmeter/README.md); SHA-256: `90c8131a66ba89723c6403a6c7ad3c7ab6d003a37549ed780bb3a171d5546a3d`
- lifecycle: строка 23: ## 🚀 Installation & Setup
- memory: строка 11: - **Custom Metrics:** Record values over time (e.g., sensor data, memory usage) using `monitor_value!`.
- async: строка 3: **RustMeter** is a comprehensive profiling, tracing, and monitoring system designed specifically for **Embedded Rust** applications. It is highly integrated with the [Embassy](https://github.com/embassy-rs/embassy) async framework and [defmt](https://github.co
- report: строка 21: 2.  **`rustmeter-cli`**: The command-line tool for the developer PC. It runs the project, collects tracing data, and creates the JSON file for Perfetto.

## camdenreslink/clr-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/camdenreslink/clr-profiler
- Категория: profiling
- Описание: A CLR (.NET Runtime) profiler written purely in Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/camdenreslink__clr-profiler/README.md](sources/camdenreslink__clr-profiler/README.md); SHA-256: `38b80be269dea1d373f1a1e691eb4ae604245839de6595ff80c7ed1e7298b4f6`
- lifecycle: строка 27: - https://docs.datadoghq.com/tracing/setup/dotnet/?tab=netcoreonlinux
- memory: строка 38: - https://www.reddit.com/r/rust/comments/4w6vjt/allocating_a_raw_double_pointer_in_rust/

## software-mansion/cairo-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/software-mansion/cairo-profiler
- Категория: profiling
- Описание: Profiler for Cairo programming language & Starknet
- Уровень: automated README evidence extraction
- Снимок: [sources/software-mansion__cairo-profiler/README.md](sources/software-mansion__cairo-profiler/README.md); SHA-256: `0860a5260896f432a48d0c372d79419de39a12c9a234d31692c30dd6ca51ead2`
- lifecycle: строка 199: ### Environment setup
- report: строка 82: the path to the json file with the trace to be profiled. You can also specify the path to the output file via `--output-path <OUTPUT_PATH>` -

## Vedant-Asati03/Telelog

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Vedant-Asati03/Telelog
- Категория: profiling
- Описание: High-performance structured logging library for Rust and Python with rich visualization capabilities
- Уровень: automated README evidence extraction
- Снимок: [sources/Vedant-Asati03__Telelog/README.md](sources/Vedant-Asati03__Telelog/README.md); SHA-256: `30d2382a5d18214c3ee74ef541e349e8301c4f5e5adb91df8341c149c9f230b7`
- memory: строка 22: - **High Performance** - Thread-local buffer pooling reduces allocations (~260 ns per log, ~11 ns when filtered)
- async: строка 23: - **Async Support** - Bounded async channels with backpressure (requires `async` feature)
- report: строка 7: [![Benchmarks](https://img.shields.io/badge/benchmarks-view%20results-blue)](https://vedant-asati03.github.io/Telelog/benchmarks/report/index.html)

## AlexGladkov/quickai

- Итог: excluded — Coding assistant usage profiler outside current scope
- Источник: https://github.com/AlexGladkov/quickai
- Категория: profiling
- Описание: Profiler for Claude Code — where your tokens, money and time go (CLI + HTML report + MCP)
- Уровень: automated README evidence extraction
- Снимок: [sources/AlexGladkov__quickai/README.md](sources/AlexGladkov__quickai/README.md); SHA-256: `071feb5c88d302d21e2d40527c8a71e237db9ed8d2f195bb32851d0f6940ec0d`
- async: строка 100: cache-read: 39461.8M (96% throughput — context re-reads)
- report: строка 16: - **Per-project reports** and a self-contained **HTML report** you open in the browser

## vobst/btf2json

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/vobst/btf2json
- Категория: profiling
- Описание: Generate Volatility3 profiles from BTF.
- Уровень: automated README evidence extraction
- Снимок: [sources/vobst__btf2json/README.md](sources/vobst__btf2json/README.md); SHA-256: `41aa357ded807681169cb4618a6e09d3d07b80bfa46d64e6ef104a34fe101089`
- report: строка 1: # `btf2json`
- correctness: строка 30: Volatility uses a JSON schema to validate profiles before using them. In order

## loocor/mcpmate

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/loocor/mcpmate
- Категория: profiling
- Описание: MCPMate is a progressive MCP management center for organizing servers, clients, profiles, capabilities, and runtime visibility in one local workspace.
- Уровень: automated README evidence extraction
- Снимок: [sources/loocor__mcpmate/README.md](sources/loocor__mcpmate/README.md); SHA-256: `4c784ba8ef458e7bd43640449f8f1659fd5550e8799fbba44cb791abfa9bb259`
- lifecycle: строка 26: > **Import MCP once. Start simple, then add profiles, per-client tools, and setup modes as your workflow grows.**
- report: строка 8: <img src="./assets/readme-hero-en.png" alt="MCPMate dashboard" width="100%">

## emoon/remotery-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/emoon/remotery-rs
- Категория: profiling
- Описание: Realtime CPU Profiler with web browser viewer
- Уровень: automated README evidence extraction
- Снимок: [sources/emoon__remotery-rs/README.md](sources/emoon__remotery-rs/README.md); SHA-256: `3a0b5e523a25ff8c5e20656e124672cfbdadd7b72e562f75f125fd576c2e06b0`
- gpu: строка 3: Remotery is a realtime CPU/GPU profiler with a viewer that runs in a web browser. This lib is a [Rust](https://www.rust-lang.org) wrapper around the C API provided by Remotery and the original repo over here https://github.com/Celtoys/Remotery where more infor

## de-husk/cosm-orc

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/de-husk/cosm-orc
- Категория: profiling
- Описание: 👹 Cosmwasm integration testing and gas profiling library 👹
- Уровень: automated README evidence extraction
- Снимок: [sources/de-husk__cosm-orc/README.md](sources/de-husk__cosm-orc/README.md); SHA-256: `5ce985a1501300410273efb5f48c37ac01420e1d92f2996276b7971b163f1a27`
- statistics: строка 15: * Deployments / Bootstrapping environments
- report: строка 121: let reports = cosm_orc.gas_profiler_report();
- correctness: строка 28: name: "validator".to_string(),

## LyonSyonII/profi

- Итог: excluded — README too thin for content analysis
- Источник: https://github.com/LyonSyonII/profi
- Категория: profiling
- Описание: Scope-based single and multithreaded profiling.
- Уровень: automated README evidence extraction
- Снимок: [sources/LyonSyonII__profi/README.md](sources/LyonSyonII__profi/README.md); SHA-256: `fa99d5fbfca943de2991e70d212ff806b248c1aeb678bb0394e3a721f9af275b`

## alexandretrotel/dotfiles-manager

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/alexandretrotel/dotfiles-manager
- Категория: profiling
- Описание: Keep your dotfiles organized, safe, and consistent across machines using profiles.
- Уровень: automated README evidence extraction
- Снимок: [sources/alexandretrotel__dotfiles-manager/README.md](sources/alexandretrotel__dotfiles-manager/README.md); SHA-256: `f375fbb63eb62e1f687e626c1356d1aab5a5385acd89573afac021796c5882e3`
- lifecycle: строка 5: A profile is a named set of configuration choices that represents a context, like work, personal, or minimal. With profiles, you can keep multiple setups and switch between them so the right settings are active for the situation.
- report: строка 68: **Scripting output:** `doctor` and `profile` accept `--json` for structured output instead of colored text.

## ImShyMike/hackatime-heatmap

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ImShyMike/hackatime-heatmap
- Категория: profiling
- Описание: Easy to set up Hackatime coding activity heatmap for your profile!
- Уровень: automated README evidence extraction
- Снимок: [sources/ImShyMike__hackatime-heatmap/README.md](sources/ImShyMike__hackatime-heatmap/README.md); SHA-256: `c92db97812b62c02a78dc1b78891a798bc69f3f81c44a118cc0aa6dc8828d831`

## h3r2tic/gpu-profiler

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/h3r2tic/gpu-profiler
- Категория: profiling
- Описание: Vulkan & OpenGL GPU profiler with `puffin` support
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## home-mangler/home-mangler

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/home-mangler/home-mangler
- Категория: profiling
- Описание: A friendly Nix profile manager
- Уровень: automated README evidence extraction
- Снимок: [sources/home-mangler__home-mangler/README.md](sources/home-mangler__home-mangler/README.md); SHA-256: `8a2bd5ce6ce464e5f37c9874d0ee25ea0a57b572948b450b9946019d6ba5f2ee`

## mstange/fxprof-perf-convert

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mstange/fxprof-perf-convert
- Категория: profiling
- Описание: Convert perf.data files to the Firefox Profiler format
- Уровень: automated README evidence extraction
- Снимок: [sources/mstange__fxprof-perf-convert/README.md](sources/mstange__fxprof-perf-convert/README.md); SHA-256: `985f9197d4f01653486cfa6500cd5be4359bfed60edfbe2a3c53824c3201ed2a`
- report: строка 22: This creates a file called `profile-conv.json`.

## microsoft/one-collect

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/microsoft/one-collect
- Категория: profiling
- Описание: An extremely fast Rust based framework for collecting event and profiling data
- Уровень: automated README evidence extraction
- Снимок: [sources/microsoft__one-collect/README.md](sources/microsoft__one-collect/README.md); SHA-256: `5b80622c655d6a145f3423b75b2e075300ac96e81ed8f8b80e56987e26fc74a1`

## sachinkg12/heaplens

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sachinkg12/heaplens
- Категория: profiling
- Описание: Heap-dump analysis inside VS Code — a native Rust engine with interactive views, leak detection, HeapQL, source navigation, and AI explanations for Java and Android.
- Уровень: automated README evidence extraction
- Снимок: [sources/sachinkg12__heaplens/README.md](sources/sachinkg12__heaplens/README.md); SHA-256: `4112d3625007e3d2895cebf43839859fdc727cf9637072acc6d097895b12e3d2`
- lifecycle: строка 178: ### AI Chat Setup (Optional)
- memory: строка 1: # HeapLens — Java & Android Heap Dump Analyzer
- async: строка 26: Built on a native Rust engine using zero-copy mmap parsing with a two-phase CSR architecture, HeapLens handles production-sized heap dumps with ease. A 1.5 GB heap dump analyzes in about 1 second; a 14.8 GB dump completes in about 9.5 seconds on an Apple M2 Ma
- report: строка 10: [![Clones](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/sachinkg12/heaplens/main/traffic/clones-badge.json)](https://github.com/sachinkg12/heaplens/graphs/traffic)

## SparkyPotato/tracy_full

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SparkyPotato/tracy_full
- Категория: profiling
- Описание: Safe, fully-featured bindings to the Tracy profiler
- Уровень: automated README evidence extraction
- Снимок: [sources/SparkyPotato__tracy_full/README.md](sources/SparkyPotato__tracy_full/README.md); SHA-256: `9bf19838627afb2b73dd44aaa07c5817bb40fcb1fbb44e51e06cd876a1b844d1`
- memory: строка 21: ### Allocation Tracking
- gpu: строка 169: ### `wgpu`
- async: строка 85: This can be things like async asset loading on different threads.

## OCamlPro/memthol

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/OCamlPro/memthol
- Категория: profiling
- Описание: Memthol is a visualizer for memory profiling data generated from OCaml programs.
- Уровень: automated README evidence extraction
- Снимок: [sources/OCamlPro__memthol/README.md](sources/OCamlPro__memthol/README.md); SHA-256: `a8585a7cc35a6e54bd8ba080ea33ce268b7603abc24efc2ab704f60ab168644b`
- statistics: строка 121: Most icons used in memthol come from the [bootstrap library][bootstrap].
- memory: строка 18: based on size, allocation lifetime, source locations in the allocation callstack, *etc.*

## mkmik/heappy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mkmik/heappy
- Категория: profiling
- Описание: heap profiler for rust
- Уровень: automated README evidence extraction
- Снимок: [sources/mkmik__heappy/README.md](sources/mkmik__heappy/README.md); SHA-256: `1e5ff0a132e9a030d146c301c356052c6d1876216e684151c505f822092fe5ec`
- memory: строка 1: # heappy

## tcr/macos-profiler

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tcr/macos-profiler
- Категория: profiling
- Описание: Profiler for Rust programs on macOS
- Уровень: automated README evidence extraction
- Снимок: [sources/tcr__macos-profiler/README.md](sources/tcr__macos-profiler/README.md); SHA-256: `7b17b039a5e26283756852c3262c60b964635729f68f063b1cae4ea9817ae168`

## ElNiak/cupp-rs

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ElNiak/cupp-rs
- Категория: profiling
- Описание: Common User Passwords Profiler (CUPP)  in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/ElNiak__cupp-rs/README.md](sources/ElNiak__cupp-rs/README.md); SHA-256: `d71da015fd8f672229fb01c3b3788a9bf18fcd732012cd1436c41e6b5a7816c1`
- report: строка 77: Alecto URL: https://github.com/yangbh/Hammer/raw/b0446396e8d67a7d4e53d6666026e078262e5bab/lib/cupp/alectodb.csv.gz

## riverscn/any-switch

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/riverscn/any-switch
- Категория: profiling
- Описание: Safely switch local app accounts, profiles, and state for Claude Code, OpenAI Codex, and extensible app definitions.
- Уровень: automated README evidence extraction
- Снимок: [sources/riverscn__any-switch/README.md](sources/riverscn__any-switch/README.md); SHA-256: `c362ad66f271dfe23a1fbe20134e7351a6092437decb85ba8de0adbebc38b7d8`
- lifecycle: строка 7: Use it when one app has several local setups and you want to move between them
- report: строка 31: - JSON, TOML, file, Keychain, or environment fragments declared by an app
- correctness: строка 236: schema validation, or secret redaction. `add` and ordinary `import-current`

## Ghostlock-AI/capsule

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/Ghostlock-AI/capsule
- Категория: profiling
- Описание: Generate VM's with kernel tracing, code sandboxing and security profiles for long running agents.
- Уровень: automated README evidence extraction
- Снимок: [sources/Ghostlock-AI__capsule/README.md](sources/Ghostlock-AI__capsule/README.md); SHA-256: `14a5b2e7a633770bc893f08885429d3bbcfe56bed95890b69280c537d3f9ebfc`
- gpu: строка 52: | Device access        | Access to `/dev/*` (KVM, tun/tap, GPU, disks, USB/TTY).     |
- async: строка 62: - **Userspace Daemon**: stream ingestion, async enrichment of syscalls for better readability.

## grafana/k6

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/grafana/k6
- Категория: benchmark/testing candidate
- Описание: A modern load testing tool, using Go and JavaScript
- Уровень: automated README evidence extraction
- Снимок: [sources/grafana__k6/README.md](sources/grafana__k6/README.md); SHA-256: `54f111f450700924b62026e08797e5ee386879d5304219c55a1df34cd94321ce`
- report: строка 22: <a href="https://goreportcard.com/report/github.com/grafana/k6"><img src="https://goreportcard.com/badge/github.com/grafana/k6" alt="Go Report Card"></a>
- correctness: строка 78: // Validate response status

## locustio/locust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/locustio/locust
- Категория: benchmark/testing candidate
- Описание: Write scalable load tests in plain Python 🚗💨
- Уровень: automated README evidence extraction
- Снимок: [sources/locustio__locust/README.md](sources/locustio__locust/README.md); SHA-256: `af6047121a02f30ce1a00310822690a2d00a7b34ef38a162be7c23109d50001f`
- async: строка 12: Locust tests can be run from command line or using its web-based UI. Throughput, response times and errors can be viewed in real time and/or exported for later analysis.
- report: строка 31: self.client.post("/login", json={"username":"foo", "password":"bar"})

## promptfoo/promptfoo

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/promptfoo/promptfoo
- Категория: benchmark/testing candidate
- Описание: Test your prompts, agents, and RAGs. Red teaming/pentesting/vulnerability scanning for AI. Compare performance of GPT, Claude, Gemini, DeepSeek, and more. Simple declarative configs with command line and CI/CD integration.  Used by OpenAI and Anthropic.
- Уровень: automated README evidence extraction
- Снимок: [sources/promptfoo__promptfoo/README.md](sources/promptfoo__promptfoo/README.md); SHA-256: `426270e1b7c45d0bf456b1284cdc1af055218db73869864b9590b1e0bdaab5cb`
- report: строка 70: It also can generate [security vulnerability reports](https://www.promptfoo.dev/docs/red-team/):

## avajs/ava

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/avajs/ava
- Категория: benchmark/testing candidate
- Описание: Node.js test runner that lets you develop with confidence 🚀
- Уровень: automated README evidence extraction
- Снимок: [sources/avajs__ava/readme.md](sources/avajs__ava/readme.md); SHA-256: `00513787c092e062378d3c3781a0d06d85a4ed93b45bf31ebea6712fe04e8732`
- lifecycle: строка 151: - [Test setup](docs/recipes/test-setup.md)
- async: строка 23: - Runs tests concurrently
- report: строка 13: ![](media/verbose-reporter.png)
- isolation: строка 7: AVA is a test runner for Node.js with a concise API, detailed error output, embrace of new language features and thread isolation that lets you develop with confidence 🚀

## alibaba/MNN

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alibaba/MNN
- Категория: benchmark/testing candidate
- Описание: MNN: A blazing-fast, lightweight inference engine battle-tested by Alibaba, powering high-performance on-device LLMs and Edge AI.
- Уровень: automated README evidence extraction
- Снимок: [sources/alibaba__MNN/README.md](sources/alibaba__MNN/README.md); SHA-256: `9e64c51e4493f319c5c0f6ddf46d738d008870bfa892c27586d68750f2dbdfe2`
- gpu: строка 115: - Supports hybrid computing on multiple devices. Currently supports CPU and GPU.
- report: строка 213: - [rapidjson](https://github.com/Tencent/rapidjson)

## webdriverio/webdriverio

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/webdriverio/webdriverio
- Категория: benchmark/testing candidate
- Описание: Next-gen browser and mobile automation test framework for Node.js
- Уровень: automated README evidence extraction
- Снимок: [sources/webdriverio__webdriverio/README.md](sources/webdriverio__webdriverio/README.md); SHA-256: `a0b56586ea0ddae6cd63c70b14412fd55e9bbfaf06585654ff2a269a89903778`
- lifecycle: строка 142: - [create-wdio](https://github.com/webdriverio/webdriverio/tree/main/packages/create-wdio) - A CLI and utility to install and setup a WebdriverIO
- memory: строка 231: <p align="center"><a href="https://github.com/webdriverio/webdriverio#nastyox"><img src="https://randojs.com/images/barsSmall.gif" alt="Animated footer bars" width="100%"/></a></p>
- report: строка 60: A codespace will open in a web-based version of Visual Studio Code. The [dev container](.devcontainer/devcontainer.json) is fully configured with the software needed for this project.
- correctness: строка 94: - [@wdio/config](https://github.com/webdriverio/webdriverio/blob/main/packages/wdio-config) - A helper utility to parse and validate WebdriverIO options

## apache/jmeter

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/apache/jmeter
- Категория: benchmark/testing candidate
- Описание: Apache JMeter open-source load testing tool for analyzing and measuring the performance of a variety of services
- Уровень: automated README evidence extraction
- Снимок: [sources/apache__jmeter/README.md](sources/apache__jmeter/README.md); SHA-256: `778259993cb24b29eaa4a17727dffd9b952536c06fe08a441bf1be6bcd26f063`
- async: строка 28: Multi-threading allows concurrent sampling by many threads and
- report: строка 56: ### Reporting

## artilleryio/artillery

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/artilleryio/artillery
- Категория: benchmark/testing candidate
- Описание: The complete load testing platform. Everything you need for production-grade load tests. Serverless & distributed. Load test with Playwright. Load test HTTP APIs, GraphQL, WebSocket, and more. Use any Node.js module.
- Уровень: automated README evidence extraction
- Снимок: [sources/artilleryio__artillery/README.md](sources/artilleryio__artillery/README.md); SHA-256: `a092f0db9495a57ddce52d8598a6547af6b879302bfa266a0e27a0f57a42ea8f`

## getanteon/anteon

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/getanteon/anteon
- Категория: benchmark/testing candidate
- Описание: Anteon (formerly Ddosify): eBPF-based Kubernetes Monitoring and Performance Testing
- Уровень: automated README evidence extraction
- Снимок: [sources/getanteon__anteon/README.md](sources/getanteon__anteon/README.md); SHA-256: `a37630bd5b150328b6d2afaca9ab6e0e909ae6f92bdaa7961faa3700fe7ee906`
- async: строка 18: <i>Anteon automatically generates Service Map of your K8s cluster without code instrumentation or sidecars. So you can easily find the bottlenecks in your system. Red lines indicate the high latency between services.</i>

## Blazity/next-enterprise

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/Blazity/next-enterprise
- Категория: benchmark/testing candidate
- Описание: 💼 An enterprise-grade Next.js boilerplate for high-performance, maintainable apps. Packed with features like Tailwind CSS, TypeScript, ESLint, Prettier, testing tools, and more to accelerate your development.
- Уровень: automated README evidence extraction
- Снимок: [sources/Blazity__next-enterprise/README.md](sources/Blazity__next-enterprise/README.md); SHA-256: `a939020d15b38536043d8bd7157117a2623b1cf819a806259f4835f15281e97c`
- statistics: строка 50: * [CVA](http://cva.style/) (Class Variance Authority) - Consistent design system creation
- lifecycle: строка 75: * Scalable & secure setup using:

## baidu/dperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/baidu/dperf
- Категория: benchmark/testing candidate
- Описание: dperf: High-Performance Network Load Testing Tool Based on DPDK
- Уровень: automated README evidence extraction
- Снимок: [sources/baidu__dperf/README.md](sources/baidu__dperf/README.md); SHA-256: `3db1f81c5302f4c6581473459977b14e74e28d14c424d0e0b82337edf6167999`
- memory: строка 49: | Client Cores | Server Cores | Connections (Billion) | Client CPU Usage (%) | Server CPU Usage (%) | Memory Usage (GB) |
- async: строка 9: Built on DPDK, dperf can generate massive traffic using a single x86 server — achieving tens of millions of HTTP Connections Per Second (CPS), hundreds of Gbps throughput, and billions of concurrent connections.

## sitespeedio/sitespeed.io

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sitespeedio/sitespeed.io
- Категория: benchmark/testing candidate
- Описание: sitespeed.io is an open-source tool for comprehensive web performance analysis, enabling you to test, monitor, and optimize your website’s speed using real browsers in various environments.
- Уровень: automated README evidence extraction
- Снимок: [sources/sitespeedio__sitespeed.io/README.md](sources/sitespeedio__sitespeed.io/README.md); SHA-256: `c21e6fc062df1a06e926763e2ef241a62f779dfdb5140276cf9cc54ef1e928e8`
- comparison: строка 57: - **CI regression testing.** Run on every PR, fail the build if a budget is exceeded.
- lifecycle: строка 129: Setup guides: [Android](https://www.sitespeed.io/documentation/sitespeed.io/mobile-phones/#test-on-android), [iOS](https://www.sitespeed.io/documentation/sitespeed.io/mobile-phones/#test-on-ios).
- memory: строка 179: - **RSS** — [release feed](https://github.com/sitespeedio/sitespeed.io/releases.atom) for new versions.
- report: строка 34: - [Reporting issues](#reporting-issues)

## session-replay-tools/tcpcopy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/session-replay-tools/tcpcopy
- Категория: benchmark/testing candidate
- Описание: An online request replication and TCP stream replay tool, ideal for real testing, performance testing, stability testing, stress testing, load testing, smoke testing, and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/session-replay-tools__tcpcopy/README.md](sources/session-replay-tools__tcpcopy/README.md); SHA-256: `9a4e770ca45474490c34f38627bd962d678686fa22d99d239cd74d55ad6a286e`
- comparison: строка 27: * Regression testing
- lifecycle: строка 220: - **If the packet hasn't reached the assistant server,** it suggests that the routing setup is not effective, and therefore `intercept` cannot capture the second handshake packet, preventing further replay. A potential solution is to run `intercept` directly o
- async: строка 19: TCPCopy minimally impacts the production system, consuming only additional CPU, memory, and bandwidth. The reproduced workload mirrors the production environment in terms of request diversity, network latency, and resource usage.
- correctness: строка 26: - Validate the stability of new systems and identify bugs that only manifest in real-world scenarios

## marmelab/awesome-rest

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/marmelab/awesome-rest
- Категория: benchmark/testing candidate
- Описание: A collaborative list of great resources about RESTful API architecture, development, test, and performance
- Уровень: automated README evidence extraction
- Снимок: [sources/marmelab__awesome-rest/README.md](sources/marmelab__awesome-rest/README.md); SHA-256: `263fc61d354ede6563dea478f3408c46b113817eaa7e39c9faeb2941b62b9d43`
- gpu: строка 181: * [Falcon](https://github.com/falconry/falcon) - Falcon is a bare-metal Python web API framework for building high-performance microservices, app backends, and higher-level frameworks.
- report: строка 63: * [JSON API](https://jsonapi.org/) - Standard for building APIs in JSON.
- correctness: строка 256: ### Validating

## fortio/fortio

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fortio/fortio
- Категория: benchmark/testing candidate
- Описание: Fortio load testing library, command line tool, advanced echo server and web UI in go (golang). Allows to specify a set query-per-second load and record latency histograms and other useful stats.
- Уровень: automated README evidence extraction
- Снимок: [sources/fortio__fortio/README.md](sources/fortio__fortio/README.md); SHA-256: `1f1a1d9d7d8ddd9197cccfd1b8beeef1c406fb4663eed8003da9cc33a6033f34`
- lifecycle: строка 179: Allow and don't abort on initial warmup errors
- async: строка 22: the server includes a simple web UI and REST API to trigger run and see graphical representation of the results (both a single latency graph and a multiple results comparative min, max, avg, qps and percentiles graphs).
- report: строка 6: [![Go Report Card](https://goreportcard.com/badge/fortio.org/fortio)](https://goreportcard.com/report/fortio.org/fortio)

## LinShunKang/MyPerf4J

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/LinShunKang/MyPerf4J
- Категория: benchmark/testing candidate
- Описание: High performance Java APM. Powered by ASM. Try it. Test it. If you feel its better, use it.
- Уровень: automated README evidence extraction
- Снимок: [sources/LinShunKang__MyPerf4J/README.md](sources/LinShunKang__MyPerf4J/README.md); SHA-256: `a8a545276dae8ca1efd820fd4eb78a2909a65bcba8d6dbf2583034b140b5444d`
- report: строка 31: - **[Method Metrics](https://grafana.com/dashboards/7766)**<br/>

## phoronix-test-suite/phoronix-test-suite

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/phoronix-test-suite/phoronix-test-suite
- Категория: benchmark/testing candidate
- Описание: The Phoronix Test Suite open-source, cross-platform automated testing/benchmarking software.
- Уровень: automated README evidence extraction
- Снимок: [sources/phoronix-test-suite__phoronix-test-suite/README.md](sources/phoronix-test-suite__phoronix-test-suite/README.md); SHA-256: `1c64acc8d62afcfa0e0aca6d5d3c26ec9ec7510eca4124969d5af5f67e2b2421`
- comparison: строка 33: regression testing, system sensor monitoring, and other extras.
- lifecycle: строка 49: Full details on the Phoronix Test Suite setup and usage is available from the
- report: строка 7: automated manner from test installation to execution and reporting. All tests
- correctness: строка 8: are meant to be easily reproducible, easy-to-use, and support fully automated

## albinotonnina/albinotonnina.com

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/albinotonnina/albinotonnina.com
- Категория: benchmark/testing candidate
- Описание: Personal portfolio website with interactive animations and visual storytelling. Built with React, featuring modular scene management, comprehensive testing, and smooth cross-device performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/albinotonnina__albinotonnina.com/README.md](sources/albinotonnina__albinotonnina.com/README.md); SHA-256: `60546d92160d3cc907fbc2298937b56523467bd43209289bc9715c3f7e04b3f0`
- lifecycle: строка 57: ### Recommended Development Setup
- async: строка 281: - **React 18** with latest features and concurrent mode support
- report: строка 180: ├── settings.json    # Editor settings

## pod4g/hiper

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pod4g/hiper
- Категория: benchmark/testing candidate
- Описание: 🚀 A statistical analysis tool for performance testing
- Уровень: automated README evidence extraction
- Снимок: [sources/pod4g__hiper/README.md](sources/pod4g__hiper/README.md); SHA-256: `ed20aed5c516ffdc4e1075e8345f3f29adbe3ee536cbef72c861048e5bf683fc`
- statistics: строка 5: <p align="center">🚀 A statistical analysis tool for performance testing</p>
- report: строка 107: #### Support `.json` and `.js` config

## alexfernandez/loadtest

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alexfernandez/loadtest
- Категория: benchmark/testing candidate
- Описание: Runs a load test on the selected URL. Fast and easy to use. Can be integrated in your own workflow using the API.
- Уровень: automated README evidence extraction
- Снимок: [sources/alexfernandez__loadtest/README.md](sources/alexfernandez__loadtest/README.md); SHA-256: `d46fe2c50fddfd702b82f166c3d89b7934391d32d795b8981f00d3d10c2babee`
- async: строка 44: $ loadtest [-n requests] [-c concurrency] [-k] URL
- report: строка 70: Even if `ab` reported a rate of 200 rps,

## cleanbrowsing/dnsperftest

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cleanbrowsing/dnsperftest
- Категория: benchmark/testing candidate
- Описание: DNS Performance test
- Уровень: automated README evidence extraction
- Снимок: [sources/cleanbrowsing__dnsperftest/README.md](sources/cleanbrowsing__dnsperftest/README.md); SHA-256: `69334cfe34c4d001285ecf958dafb4a30e30b3ebc152098d8707dee6f3236f4b`

## fcsonline/drill

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fcsonline/drill
- Категория: benchmark/testing candidate
- Описание: Drill is an HTTP load testing application written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/fcsonline__drill/README.md](sources/fcsonline__drill/README.md); SHA-256: `5d796a7eac589a829e9ba49d3665f57581c1cbc274a52303e815d741e26da2a1`
- async: строка 21: concurrency: 4
- report: строка 32: url: /api/users.json

## dfeneyrou/palanteer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dfeneyrou/palanteer
- Категория: benchmark/testing candidate
- Описание: Visual Python and C++ nanosecond profiler, logger, tests enabler
- Уровень: automated README evidence extraction
- Снимок: [sources/dfeneyrou__palanteer/README.md](sources/dfeneyrou__palanteer/README.md); SHA-256: `82df7dd96491be6cba7faf02107c25074c3e44bbf2a79f2a21f14dc59e9a501d`
- statistics: строка 146: # Bootstrap
- memory: строка 23: - [Automatic instrumentation](https://dfeneyrou.github.io/palanteer/instrumentation_api_python.md.html#automaticinstrumentationwithoutcodemodification) of functions enter/leave, memory allocations, raised exceptions, garbage collection runs

## Blazemeter/taurus

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Blazemeter/taurus
- Категория: benchmark/testing candidate
- Описание: Automation-friendly framework for Continuous Testing by
- Уровень: automated README evidence extraction
- Снимок: [sources/Blazemeter__taurus/README.md](sources/Blazemeter__taurus/README.md); SHA-256: `38966b320d144548bfac5a17bcae524ed1c81e245f06b93e8b5f063ddd31d82b`
- async: строка 25: - concurrency: 10
- report: строка 38: Then run `bzt test.yml`. After the tool finishes, observe resulting summary stats in console log (for more reporting options, see [Generating Test Reports](https://gettaurus.org/docs/Reporting.md)). All artifact files from the run will be placed in the directo

## naver/ngrinder

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/naver/ngrinder
- Категория: benchmark/testing candidate
- Описание: enterprise level performance testing solution
- Уровень: automated README evidence extraction
- Снимок: [sources/naver__ngrinder/README.md](sources/naver__ngrinder/README.md); SHA-256: `10578c27db41ffa60120b0d5d632a790bc1be51c99e707b054a8968d0b1fd4f0`
- statistics: строка 87: For transparency and insight into our release cycle, and to strive to maintain backward compatibility, Bootstrap will be maintained under the Semantic Versioning guidelines to the greatest extent possible.
- async: строка 41: * Run multiple tests concurrently. Assign the pre-installed multiple agents to maximize each agent's utilization.
- report: строка 11: nGrinder is a platform for stress tests that enables you to execute script creation, test execution, monitoring, and result report generator simultaneously. The open-source nGrinder offers easy ways to conduct stress tests by eliminating inconveniences and pro

## reactopt/reactopt

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/reactopt/reactopt
- Категория: benchmark/testing candidate
- Описание: A CLI React performance optimization tool that identifies potential unnecessary re-rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/reactopt__reactopt/README.md](sources/reactopt__reactopt/README.md); SHA-256: `106e9433ad12728fd3d8a64cfa164ca6ba8f6db4164da97fc8dc330ec210f8c6`
- async: строка 19: 1.5.0 is the first working version of this module. Utilizes async/await, which is natively supported in Node 7.6+.
- report: строка 48: Include this script in your package.json:

## addyosmani/puppeteer-webperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/addyosmani/puppeteer-webperf
- Категория: benchmark/testing candidate
- Описание: Automating Web Performance testing with Puppeteer 🎪
- Уровень: automated README evidence extraction
- Снимок: [sources/addyosmani__puppeteer-webperf/README.md](sources/addyosmani__puppeteer-webperf/README.md); SHA-256: `3fa94fb22432cdf282c106cd80f0d4f2079671b14d32b6648940bdf378d2eaed`
- memory: строка 691: Checking the number of objects retained on the heap can be a good basic start to measuring memory leaks in JavaScript. In Puppeteer, `queryObjects()` can be used to count all the objects with the same prototype somewhere in the prototype chain.
- async: строка 39: (async () => {
- report: строка 15: * [Generate a Lighthouse report](#lighthouse-report)

## sqlancer/sqlancer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sqlancer/sqlancer
- Категория: benchmark/testing candidate
- Описание: Automated testing to find logic and performance bugs in database systems
- Уровень: automated README evidence extraction
- Снимок: [sources/sqlancer__sqlancer/README.md](sources/sqlancer__sqlancer/README.md); SHA-256: `e5f5180fefc22551e27d9fb83cb9337cad8625677ee6185fe8c312715e154042`
- lifecycle: строка 34: **DBMSs.** To run SQLancer on SQLite, it was not necessary to install and set up a DBMS. The reason for this is that embedded DBMSs run in the same process as the application and thus require no separate installation or setup. Embedded DBMSs supported by SQLan
- report: строка 43: **Testing the latest DBMS version.** For most DBMSs, SQLancer supports only a previous *release* version. Thus, potential bugs that SQLancer finds could be already fixed in the latest *development* version of the DBMS. If you are not a developer of the DBMS th
- correctness: строка 9: 1. **Test input generation**: SQLancer implements approaches for automatically generating SQL statements. It contains various hand-written SQL generators that operate in multiple phases. First, a database schema is created, which refers to a set of tables and

## avoidwork/filesize.js

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/avoidwork/filesize.js
- Категория: benchmark/testing candidate
- Описание: A lightweight, high-performance file size utility that converts bytes to human-readable strings. Zero dependencies. 100% test coverage.
- Уровень: automated README evidence extraction
- Снимок: [sources/avoidwork__filesize.js/README.md](sources/avoidwork__filesize.js/README.md); SHA-256: `776986ef43e48a94528fdfa322774f442ae7dc4fcfa80b7bfe77ca570a013481`

## bamlab/flashlight

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bamlab/flashlight
- Категория: benchmark/testing candidate
- Описание: 📱⚡️ Lighthouse for Mobile - audits your app and gives a performance score to your Android apps (native, React Native, Flutter..). Measure performance on CLI, E2E tests, CI...
- Уровень: automated README evidence extraction
- Снимок: [sources/bamlab__flashlight/README.md](sources/bamlab__flashlight/README.md); SHA-256: `7b946faccede197398058730c9c10294bc898e61e55e4c8e22be1d2dcfa9605f`
- lifecycle: строка 9: 🙅 No setup required in your app
- report: строка 11: ✨ Generates beautiful reports ([like this Flatlist/Flashlist comparison](https://docs.flashlight.dev/examples/flashlist/report.html))

## Inspiaaa/UnityHFSM

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Inspiaaa/UnityHFSM
- Категория: benchmark/testing candidate
- Описание: A fast, tried-and-tested hierarchical finite state machine library for Unity, designed to be easy to use yet powerful without compromising performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/Inspiaaa__UnityHFSM/README.md](sources/Inspiaaa__UnityHFSM/README.md); SHA-256: `e1da6aa042849807385baf1d7cfff909b50d4823c9721cb54b042c96f3e8f6ad`
- lifecycle: строка 210: To be able to define the states' logic concisely, we'll need to create some helper methods and properties first. Their implementation is just an example and may differ for your scene setup.
- memory: строка 38: - **No GC Allocations** for state changes / updates / ... after setting up the state machine (-> No unwanted GC related lag spikes because of the state machine)

## TraceMachina/nativelink

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/TraceMachina/nativelink
- Категория: benchmark/testing candidate
- Описание: NativeLink is a Nix-powered, open source, high-performance build cache and remote execution server, compatible with Bazel, Soong, Pants, Buck2, Reclient, and other RE-compatible build systems. It offers drastically faster builds, reduced test flakiness, and support for specialized hardware.
- Уровень: automated README evidence extraction
- Снимок: [sources/TraceMachina__nativelink/README.md](sources/TraceMachina__nativelink/README.md); SHA-256: `1fe7f0be348a419e022f64a9911ed0c8b67ee064b068ece7cade77224d9ac8e1`
- lifecycle: строка 60: To start, you can deploy NativeLink as a Docker image (as shown below). The setups below are **production-grade** installations. See the [contribution docs](https://nativelink.com/docs/contribute/nix/) for instructions on how to build from source with [Bazel](
- report: строка 75: https://raw.githubusercontent.com/TraceMachina/nativelink/v1.4.0/nativelink-config/examples/basic_cas.json5

## callstack/reassure

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/callstack/reassure
- Категория: benchmark/testing candidate
- Описание: Performance testing companion for React and React Native
- Уровень: automated README evidence extraction
- Снимок: [sources/callstack__reassure/README.md](sources/callstack__reassure/README.md); SHA-256: `6b8dc5cd48ec419f4387191f39752b9810cb76b96d65bc8dfa304c280ae0476d`
- statistics: строка 67: Reassure works by measuring render characteristics – duration and count – of the testing scenario you provide and comparing that to the stable version. It repeats the scenario multiple times to reduce the impact of random variations in render times caused by t
- comparison: строка 59: You want your React Native app to perform well and fast at all times. As a part of this goal, you profile the app, observe render patterns, apply memoization in the right places, etc. But it's all manual and too easy to unintentionally introduce performance re
- lifecycle: строка 22: - [Installation and setup](#installation-and-setup)
- async: строка 24: - [Writing async tests](#writing-async-tests)
- report: строка 67: Reassure works by measuring render characteristics – duration and count – of the testing scenario you provide and comparing that to the stable version. It repeats the scenario multiple times to reduce the impact of random variations in render times caused by t

## d3ward/toolz

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/d3ward/toolz
- Категория: benchmark/testing candidate
- Описание: Project Archived - Easy-to-use interface and extensive collection of testing tools for optimizing performance and user experience.
- Уровень: automated README evidence extraction
- Снимок: [sources/d3ward__toolz/README.md](sources/d3ward__toolz/README.md); SHA-256: `34f7142b8ea5f25ebbc745a8dd6fa02c9a2c27735323412c563ed8f21fa232c4`

## microsoft/diskspd

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/microsoft/diskspd
- Категория: benchmark/testing candidate
- Описание: DISKSPD is a storage load generator / performance test tool from the Windows/Windows Server and Cloud Server Infrastructure Engineering teams
- Уровень: automated README evidence extraction
- Снимок: [sources/microsoft__diskspd/README.md](sources/microsoft__diskspd/README.md); SHA-256: `140db9fff1717f087d03f2a649ab17e0c59c3ccb048b9abbe631cdd054916e66`
- lifecycle: строка 56: * verbose output is more consistent; includes actual warmup, measured and cooldown intervals v. expected
- memory: строка 22: **NOTE:** two default changes may require rebaselining. Thread affinity now orders P-cores before E-cores within each group by default (`-aup` restores unordered assignment), which changes placement on heterogeneous systems. I/O buffers are now separated by PD
- async: строка 42: The new design drains the completion queue more aggressively, shrinking delays that impacted latency measurement
- report: строка 31: * New processor name and cache topology reporting in text and XML results.
- correctness: строка 95: * Added support for validating XML profiles using an in-built XSD
- isolation: строка 22: **NOTE:** two default changes may require rebaselining. Thread affinity now orders P-cores before E-cores within each group by default (`-aup` restores unordered assignment), which changes placement on heterogeneous systems. I/O buffers are now separated by PD

## shulieTech/Takin

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/shulieTech/Takin
- Категория: benchmark/testing candidate
- Описание: Takin is an Java-based, open-source system designed to measure online environmental performance test for full-links, Especially for microservices. Through Takin, middlewares and applications can identify real online traffic and test traffic, ensure that they enter the right databases.
- Уровень: automated README evidence extraction
- Снимок: [sources/shulieTech__Takin/README.md](sources/shulieTech__Takin/README.md); SHA-256: `ee6764edea695ab25328dd8a3d925f62f74b9a8d4be0d8eafb71598fc027ba4e`
- report: строка 37: vim /etc/docker/daemon.json

## treosh/lighthouse-ci-action

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/treosh/lighthouse-ci-action
- Категория: benchmark/testing candidate
- Описание: Audit URLs using Lighthouse and test performance with Lighthouse CI.
- Уровень: automated README evidence extraction
- Снимок: [sources/treosh__lighthouse-ci-action/README.md](sources/treosh__lighthouse-ci-action/README.md); SHA-256: `748ddb743d0811c09ebba52d9dcab494c6e1177664a2a7019a67aa83e816010a`
- lifecycle: строка 385: - uses: browser-actions/setup-chrome@latest
- gpu: строка 162: { "ci": { "collect": { "numberOfRuns": 1, "settings": { "chromeFlags": "--disable-gpu --no-sandbox --no-zygote" } } } }
- report: строка 42: budgetPath: ./budget.json # test performance budgets
- correctness: строка 66: <summary>Run Lighthouse and validate against Lighthouse CI assertions.</summary><br>

## dmaicher/doctrine-test-bundle

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/dmaicher/doctrine-test-bundle
- Категория: benchmark/testing candidate
- Описание: Symfony bundle to isolate your app's doctrine database tests and improve the test performance
- Уровень: automated README evidence extraction
- Снимок: [sources/dmaicher__doctrine-test-bundle/README.md](sources/dmaicher__doctrine-test-bundle/README.md); SHA-256: `a76701068df94b3b0b9d3912f17b045803ca66c82cd937bf8ef61bd3fff15a07`
- statistics: строка 52: <bootstrap class="DAMA\DoctrineTestBundle\PHPUnit\PHPUnitExtension" />
- memory: строка 12: It also includes a `Psr6StaticArrayCache` that will be automatically configured as meta data & query cache for all EntityManagers. This improved the speed and memory usage for my testsuites dramatically! This is especially beneficial if you have a lot of tests

## YahooArchive/boomerang

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/YahooArchive/boomerang
- Категория: benchmark/testing candidate
- Описание: End user oriented web performance testing and beaconing
- Уровень: automated README evidence extraction
- Снимок: [sources/YahooArchive__boomerang/README.md](sources/YahooArchive__boomerang/README.md); SHA-256: `024371a6b00025731c88415b86624daea921f269287bbdab3c248e6f354fd474`

## DoneDeal0/superdiff

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/DoneDeal0/superdiff
- Категория: benchmark/testing candidate
- Описание: Superdiff provides a rich and readable diff for arrays, objects, texts and coordinates. It supports stream and file inputs for handling large datasets efficiently, is battle-tested, has zero dependencies, and offers a top-tier performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/DoneDeal0__superdiff/README.md](sources/DoneDeal0__superdiff/README.md); SHA-256: `67bc1d853e2df27174c1049219ed533683d305e200b8149ee481d51e7151f8f0`
- lifecycle: строка 86: Method: Warm up runs, then each script is executed 20 times, and we keep the median time. To minimize garbage collection and cross‑benchmark interference, all scenarios are run individually. All benchmark scripts are included so you can reproduce the results l
- report: строка 390: > In a server environment, `Readable` refers to Node.js streams, and `FilePath` refers to the path of a file (e.g., `./list.json`). Examples are provided in the #usage section below.

## fabienrenaud/java-json-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fabienrenaud/java-json-benchmark
- Категория: benchmark/testing candidate
- Описание: Performance testing of serialization and deserialization of Java JSON libraries
- Уровень: automated README evidence extraction
- Снимок: [sources/fabienrenaud__java-json-benchmark/README.md](sources/fabienrenaud__java-json-benchmark/README.md); SHA-256: `d6f873d372336dd1d0b38195472a45c452f5fa8d30fd89020209203f50bb801f`
- lifecycle: строка 129: # Warmup: 5 iterations, 10 s each
- memory: строка 50: * write data to reusable output streams (when possible); this reduces allocation pressure
- async: строка 7: This project benchmarks the throughput performance of a variety of Java Json libraries
- report: строка 1: [![Java CI](https://github.com/fabienrenaud/java-json-benchmark/actions/workflows/gradle.yml/badge.svg)](https://github.com/fabienrenaud/java-json-benchmark/actions/workflows/gradle.yml)

## linux-rdma/perftest

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/linux-rdma/perftest
- Категория: benchmark/testing candidate
- Описание: Infiniband Verbs Performance Tests
- Уровень: automated README evidence extraction
- Снимок: [sources/linux-rdma__perftest/README](sources/linux-rdma__perftest/README); SHA-256: `af452992361cf298f6d4ae2926ed65ddc56332c0a2c972978d9f5dd486ff4749`
- statistics: строка 94: statistical analysis programs.
- lifecycle: строка 76: Typically, the first value measured is the maximum value, due to warmup effects.
- memory: строка 227: In chaining we mean allocating <list_size> array, and setting 'next' pointer of each WQE in the array
- gpu: строка 259: ./ib_write_bw -d ib_dev --use_cuda=<gpu index> -a
- async: строка 23: The collection contains a set of bandwidth and latency benchmark such as:
- report: строка 65: - The latency benchmarks measure round-trip time but report half of that as one-way
- correctness: строка 429: 8. Data Validation (--data_validation)
- isolation: строка 376: 7. CPU/NUMA Affinity (--pin_cores, --numa_node, --disable_numa)

## tag1consulting/goose

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tag1consulting/goose
- Категория: benchmark/testing candidate
- Описание: Load testing framework, inspired by Locust
- Уровень: automated README evidence extraction
- Снимок: [sources/tag1consulting__goose/README.md](sources/tag1consulting__goose/README.md); SHA-256: `3e0e08fe55467fb704aa7355cf1f691d899957d90acc27727477c3135d7d72b2`
- memory: строка 4: Load testing is a critical step in ensuring your web application can handle real-world traffic patterns. It helps you identify performance bottlenecks, optimize resource allocation, and ensure a seamless user experience.
- correctness: строка 28: You can also use [Goose Eggs](https://github.com/tag1consulting/goose-eggs), a helper crate that provides useful functions for writing load tests, such as validation helpers for HTTP responses.

## kubernetes/perf-tests

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kubernetes/perf-tests
- Категория: benchmark/testing candidate
- Описание: Performance tests and benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/kubernetes__perf-tests/README.md](sources/kubernetes__perf-tests/README.md); SHA-256: `9fa43b17d34649e6eca928a3bd9ee74273c3d33f550ea09723d9d6d178740aff`
- lifecycle: строка 9: ## Repository setup
- report: строка 3: [![Go Report Card](https://goreportcard.com/badge/github.com/kubernetes/perf-tests)](https://goreportcard.com/report/github.com/kubernetes/perf-tests)

## HewlettPackard/netperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/HewlettPackard/netperf
- Категория: benchmark/testing candidate
- Описание: Netperf is a benchmark that can be used to measure the performance of many different types of networking. It provides tests for both unidirectional throughput, and end-to-end latency.
- Уровень: automated README evidence extraction
- Снимок: [sources/HewlettPackard__netperf/README](sources/HewlettPackard__netperf/README); SHA-256: `d14c9bf9d14b879ac947b0db5686dcce53584d09532a9a98d5488c50c515a2d8`
- report: строка 18: Feel free to report netperf results in public forums, but please be

## alloy-rs/core

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alloy-rs/core
- Категория: benchmark/testing candidate
- Описание: High-performance, well-tested & documented core libraries for Ethereum, in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/alloy-rs__core/README.md](sources/alloy-rs__core/README.md); SHA-256: `b2582e1119d7740a87a33b6319b06416aae7a311acdd781250fe3d05c495c180`
- report: строка 31: - [`alloy-json-abi`] - Full Ethereum [JSON-ABI] implementation

## janreges/siteone-crawler

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/janreges/siteone-crawler
- Категория: benchmark/testing candidate
- Описание: SiteOne Crawler is a cross-platform website crawler and analyzer for SEO, security, accessibility, and performance optimization—ideal for developers, DevOps, QA engineers, and consultants. Supports Windows, macOS, and Linux (x64 and arm64).
- Уровень: automated README evidence extraction
- Снимок: [sources/janreges__siteone-crawler/README.md](sources/janreges__siteone-crawler/README.md); SHA-256: `62d93cc8bf9c7531043e683db5d9ea0e1e7c70d23dc4daaa588d796c49a47be2`
- comparison: строка 759: | `--ci-baseline=<file>` | Path to a previous `--output=json` file used as a baseline for regression checks. A missing/unreadable file is warned about (the check is skipped, not silently passed). |
- lifecycle: строка 128: - will help you **warm up the application cache** or the **cache on the reverse proxy** of the entire website
- memory: строка 270: | **musl** (compatible) | Any Linux distribution (statically linked, no dependencies) | ~50–80% slower due to musl memory allocator |
- async: строка 80: - **🛠️ Dev/DevOps assistant** - offers stress/load testing with configurable concurrent workers (`--workers`) and request
- report: строка 11: *   **Rich Output Formats:** Interactive **HTML audit report** 📊 with sortable tables and quality scoring (0.0-10.0) (see [nextjs.org sample](https://crawler.siteone.io/html/2024-08-23/forever/cl8xw4r-fdag8wg-44dd.html)), detailed **JSON** for programmatic con
- correctness: строка 109: - it will **clearly warn you** ⚠️ of any wrong use of the tool (e.g. input parameters validation or wrong permissions)
- isolation: строка 793: | `--browser-no-sandbox` | off | Add `--no-sandbox` (often required in Docker/CI/WSL/root; weakens isolation). |

## ubugeeei-prod/vize

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ubugeeei-prod/vize
- Категория: benchmark/testing candidate
- Описание: blazing fast Vue.js toolchain. compiler, linter, type checker, formatter, lsp, story system, editors.  already passed 13k+ tests.
- Уровень: automated README evidence extraction
- Снимок: [sources/ubugeeei-prod__vize/README.md](sources/ubugeeei-prod__vize/README.md); SHA-256: `5c82471e002a2cb4726c59e1e461419d8b25ffb1e9ffad33301720737ebc913c`
- lifecycle: строка 79: ### New setup
- report: строка 144: `tools/benchmarks/results/tool-benchmark-latest.json`
- correctness: строка 192: building a Nuxt-based conference website with Vize, then carrying that validation all the way to

## maxim-saplin/CrossPlatformDiskTest

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/maxim-saplin/CrossPlatformDiskTest
- Категория: benchmark/testing candidate
- Описание: Windows, macOS and Android storage (HDD, SSD, RAM) speed testing/performance benchmarking app
- Уровень: automated README evidence extraction
- Снимок: [sources/maxim-saplin__CrossPlatformDiskTest/README.md](sources/maxim-saplin__CrossPlatformDiskTest/README.md); SHA-256: `bf27b2a3a66a685d6b7c00a42217d2bf0be603aae52ed25270d0364b973105f7`
- async: строка 3: Measuring storage performance (SSD, HDD, USB Flash etc.) and RAM speed across Windows, macOS and Android devices. Random and sequential throughput (read/write operations) is calculted in MB/s and can be compared in consistent and reliable manner between mobile

## andyedinborough/stress-css

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/andyedinborough/stress-css
- Категория: benchmark/testing candidate
- Описание: JavaScript to test each CSS class on a page and report which are hindering performance
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## AirportR/fulltclash

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/AirportR/fulltclash
- Категория: benchmark/testing candidate
- Описание: General proxy performance testing tool based on Clash using Telegram API.
- Уровень: automated README evidence extraction
- Снимок: [sources/AirportR__fulltclash/README.md](sources/AirportR__fulltclash/README.md); SHA-256: `6220d2d3dc18d3f7e8ab66b117430ac797679ad49323875f0353805750db1177`
- async: строка 274: - [async-timeout](https://github.com/aio-libs/async-timeout)  [Apache2]
- report: строка 8: <a href="https://app.codacy.com/gh/AirportR/fulltclash/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade"><img src="https://app.codacy.com/project/badge/Grade/389b2787eb7647dfad486ccaa70eabf4"></a>

## LucaCanali/sparkMeasure

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/LucaCanali/sparkMeasure
- Категория: benchmark/testing candidate
- Описание: This repository contains the development code for sparkMeasure, an Apache Spark performance analysis and troubleshooting library. It simplifies collecting, aggregating, and exporting Spark task/stage metrics, and is designed for practical use by developers and data engineers in interactive analysis, testing, and production monitoring workflows.
- Уровень: automated README evidence extraction
- Снимок: [sources/LucaCanali__sparkMeasure/README.md](sources/LucaCanali__sparkMeasure/README.md); SHA-256: `1c3c5a183d16eb6c6e2cbf4de5983c3787071e61bad6da46da3efcb9de54805e`
- comparison: строка 406: not allow you to understand the root causes of performance regression.
- lifecycle: строка 45: - [Setup Examples](#setup-examples)
- memory: строка 191: Additional stage-level executor metrics (memory usage info updated at each heartbeat):
- async: строка 364: snapshots taken at the start and end of an execution. If multiple Spark actions run concurrently on
- report: строка 40: - [Memory report](#memory-report)
- correctness: строка 299: To ensure the integrity of the sparkMeasure codebase and validate your setup, you can run the built-in unit tests. These tests are designed to verify core functionality.

## dwyl/book

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/dwyl/book
- Категория: benchmark/testing candidate
- Описание: 📗 Our Book on Full-Stack Web Application Development covering User Experience (UX) Design, Mobile/Offline/Security First, Progressive Enhancement, Continuous Integration/Deployment, Testing (UX/TDD/BDD), Performance-Driven-Development and much more!
- Уровень: automated README evidence extraction
- Снимок: [sources/dwyl__book/README.md](sources/dwyl__book/README.md); SHA-256: `ed9bb2b8a7ec95dddc139f203a6486bb9651659545e57bfa9891a5b1c7e6ec41`
- report: строка 391: (*any of the NoSQL datastores which store JSON documents*), it's easier to go "full stack".

## rogerwelin/cassowary

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rogerwelin/cassowary
- Категория: benchmark/testing candidate
- Описание: :rocket: Modern cross-platform HTTP load-testing tool written in Go
- Уровень: automated README evidence extraction
- Снимок: [sources/rogerwelin__cassowary/README.md](sources/rogerwelin__cassowary/README.md); SHA-256: `c31fe97386ebba7ab0f9e52f3df093b0a8f4917d360abb3321125cd229fc0c0c`
- async: строка 120: Example running **cassowary** against www.example.com with 100 requests spread out over 10 concurrent users:
- report: строка 3: <a href="https://goreportcard.com/badge/github.com/rogerwelin/cassowary"><img src="https://goreportcard.com/badge/github.com/rogerwelin/cassowary?style=for-the-badge&logo=go" alt="Go Report Card"></a>

## grafana/k6-operator

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/grafana/k6-operator
- Категория: benchmark/testing candidate
- Описание: An operator for running distributed k6 tests.
- Уровень: automated README evidence extraction
- Снимок: [sources/grafana__k6-operator/README.md](sources/grafana__k6-operator/README.md); SHA-256: `ce61087c2edcd8d74fd02f3d70ca5f5a43fb16a49cdfff96bac1eec4bed04b10`
- lifecycle: строка 10: The `TestRun` CRD is a representation of a single k6 test executed once. `TestRun` supports various configuration options that allow you to adapt to different Kubernetes setups. You can find a description of the more common options [here](https://grafana.com/d

## YueChen-C/py-ios-device

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/YueChen-C/py-ios-device
- Категория: benchmark/testing candidate
- Описание: IOS Professional Performance Testing Tool . You can get CPU, GPU, Memory , Lifecycle  and other metrics from real  iOS devices . iOS 性能测试工具
- Уровень: automated README evidence extraction
- Снимок: [sources/YueChen-C__py-ios-device/README.md](sources/YueChen-C__py-ios-device/README.md); SHA-256: `f73b116b4a04de500740cb75a0f70d1a59278aa320f793bc6ceb128e3a657528`
- gpu: строка 32: - [x] Get Metal GPU Counters

## kube-burner/kube-burner

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kube-burner/kube-burner
- Категория: benchmark/testing candidate
- Описание: Kubernetes performance and scale test orchestration framework written in golang
- Уровень: automated README evidence extraction
- Снимок: [sources/kube-burner__kube-burner/README.md](sources/kube-burner__kube-burner/README.md); SHA-256: `8b40e80cd8438b9f364cf39a6982736c9bd309326bb7954063bc70c6b5d1342a`
- report: строка 3: [![Go Report Card](https://goreportcard.com/badge/github.com/kube-burner/kube-burner)](https://goreportcard.com/report/github.com/kube-burner/kube-burner)

## dotnet/performance

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dotnet/performance
- Категория: benchmark/testing candidate
- Описание: This repo contains benchmarks used for testing the performance of all .NET Runtimes
- Уровень: automated README evidence extraction
- Снимок: [sources/dotnet__performance/README.md](sources/dotnet__performance/README.md); SHA-256: `b27a912ebe2f654757228359bdee6141f072aaf4d1e83b6c9a2fc6742e91b045`

## alibaba/mobileperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alibaba/mobileperf
- Категория: benchmark/testing candidate
- Описание: Android performance test
- Уровень: automated README evidence extraction
- Снимок: [sources/alibaba__mobileperf/readme.md](sources/alibaba__mobileperf/readme.md); SHA-256: `3fbe6423078ad831efb1dfd0ec2d9954e52627027f9dd52d8fdc03d7748c27f8`

## swc-project/jest

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/swc-project/jest
- Категория: benchmark/testing candidate
- Описание: Super-fast alternative for babel-jest or ts-jest without type checking. Please use main repository for issues
- Уровень: automated README evidence extraction
- Снимок: [sources/swc-project__jest/README.md](sources/swc-project__jest/README.md); SHA-256: `f2e3a2de3664985a3d979c655614e9531b5a57a30b60c71e4e6084f052428216`

## themidnightgospel/Imposter

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/themidnightgospel/Imposter
- Категория: benchmark/testing candidate
- Описание: Mocking library with the perfect balance of Performance and Intuitive API
- Уровень: automated README evidence extraction
- Снимок: [sources/themidnightgospel__Imposter/README.md](sources/themidnightgospel__Imposter/README.md); SHA-256: `b0aa9f004b5d60055770e72fbe64f470de4dfa9a11038927cb6cbde54b8fc6fd`
- lifecycle: строка 86: Choose how unmocked members behave. **Implicit** mode returns defaults silently — great for prototyping. **Explicit** mode throws on any call without a setup — ideal for unit tests that must be precise. See more [here](https://themidnightgospel.github.io/Impos
- memory: строка 103: | Method      | Iteration |                Mean |     Allocated |
- async: строка 185: - [Async Support](https://themidnightgospel.github.io/Imposter/latest/methods/#async-methods)

## MarathonLabs/marathon

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/MarathonLabs/marathon
- Категория: benchmark/testing candidate
- Описание: Cross-platform test runner
- Уровень: automated README evidence extraction
- Снимок: [sources/MarathonLabs__marathon/README.md](sources/MarathonLabs__marathon/README.md); SHA-256: `cdf6fcadd5797f703a8e302b65fa3f03dba62f41119bf20d2a9f63e0adfc02f9`
- statistics: строка 33: * The flakiness strategy queues up preventive retries for tests which are expected to fail during the test run according to the current real-time statistical data
- report: строка 44: outputDir: "build/reports/marathon"

## wix-incubator/DetoxInstruments

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/wix-incubator/DetoxInstruments
- Категория: benchmark/testing candidate
- Описание: Detox Instruments is a performance–analysis and testing framework, designed to help developers profile their mobile apps in order to better understand and optimize their app's behavior and performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/wix-incubator__DetoxInstruments/README.md](sources/wix-incubator__DetoxInstruments/README.md); SHA-256: `22c440019467d2377248b5dc32d38e889c92e6c1643f72ea46fb5e98f90d9c8c`
- memory: строка 18: * Memory usage
- async: строка 29: * Async storage metrics & data

## piotrmurach/rspec-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/piotrmurach/rspec-benchmark
- Категория: benchmark/testing candidate
- Описание: Performance testing matchers for RSpec
- Уровень: automated README evidence extraction
- Снимок: [sources/piotrmurach__rspec-benchmark/README.md](sources/piotrmurach__rspec-benchmark/README.md); SHA-256: `0e222f36fc75806f69ad2ff8cb9b1ed9869cf97109a3b72e216cbaceb57e46d9`
- comparison: строка 406: - **too relaxed** boundaries may also lead to false positives missing actual performance regressions
- lifecycle: строка 129: You can also use `warmup` matcher that can run your code before the actual samples are taken to reduce erratic execution times.
- memory: строка 23: * [benchmark-malloc](https://github.com/piotrmurach/benchmark-malloc) for measuring object and memory allocations.
- isolation: строка 42: * [3.2 :run_in_subprocess](#32-run_in_subprocess)

## lafikl/RWDPerf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lafikl/RWDPerf
- Категория: benchmark/testing candidate
- Описание: Performance testing tool for Responsive web designs.
- Уровень: automated README evidence extraction
- Снимок: [sources/lafikl__RWDPerf/README.md](sources/lafikl__RWDPerf/README.md); SHA-256: `c6b8a72629cbb850ae3ddac3c71753b0a345fa552ed96f123d78d05d5132152c`
- report: строка 59: -j, --json                     Return results as JSON

## jsdelivr/globalping

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/jsdelivr/globalping
- Категория: benchmark/testing candidate
- Описание: A global network of probes to run network tests like ping, traceroute and DNS resolve
- Уровень: automated README evidence extraction
- Снимок: [sources/jsdelivr__globalping/README.md](sources/jsdelivr__globalping/README.md); SHA-256: `b4a679949d6fce335a2597f18024ea0134e3081126dcfaed8d2b499ee9d7753f`
- lifecycle: строка 355: ### Setup instructions
- async: строка 53: [![globalping latency test from google cloud](https://github.com/jsdelivr/globalping/assets/1834071/760c9031-9292-4e2a-9901-68ef7f0745ca)](https://globalping.io)
- report: строка 15: - Help us with quality control and testing across systems. Report any problems, bugs, or bad user experience you find.

## Zooz/predator

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Zooz/predator
- Категория: benchmark/testing candidate
- Описание: A powerful open-source platform for load testing APIs.
- Уровень: automated README evidence extraction
- Снимок: [sources/Zooz__predator/README.md](sources/Zooz__predator/README.md); SHA-256: `a5edf6b37ccac20a5ec98cd69ee2b05bd71b5a44ea7186427040482bd22118b3`
- statistics: строка 13: It has a simple, one-click installation, built with support for Kubernetes, DC/OS and Docker Engine, and can persist the created performance tests and their reports in 5 different databases. It also supports running distributed load out of the box. Bootstrappe
- comparison: строка 33: | Benchmarks                     | :sparkle:          |Set benchmarks to compare test runs to ensure performance degradation is discovered early in development. Allows to measure every build and release against specified baseline results guaranteeing safer rel
- lifecycle: строка 104: npm run setup-local-env
- async: строка 24: | Distributed Load               | :sparkle:          |Predator supports an unlimited number of load generators that produce multiple load runners concurrently.
- report: строка 11: Predator manages the entire lifecycle of stress-testing servers, from creating performance tests, to running these tests on a scheduled and on-demand basis, and finally viewing the test results in a highly informative and live report.

## quii/mockingjay-server

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/quii/mockingjay-server
- Категория: benchmark/testing candidate
- Описание: Fake server, Consumer Driven Contracts and help with testing performance from one configuration file with zero system dependencies and no coding whatsoever
- Уровень: automated README evidence extraction
- Снимок: [sources/quii__mockingjay-server/README.md](sources/quii__mockingjay-server/README.md); SHA-256: `fe3e3036a10d03feec80c7826f37ce7fafa703970264b19342c98b45e13441cf`
- report: строка 33: content-type: application/json

## quick-perf/quickperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/quick-perf/quickperf
- Категория: benchmark/testing candidate
- Описание: QuickPerf is a testing library for Java to quickly evaluate and improve some performance-related properties
- Уровень: automated README evidence extraction
- Снимок: [sources/quick-perf__quickperf/README.md](sources/quick-perf__quickperf/README.md); SHA-256: `476a74cfbb404d195c65fe55a1065376abe487a773fba08ce57b285b5a2ac1f8`
- memory: строка 77: @MeasureHeapAllocation
- report: строка 189: <a href="https://github.com/quick-perf/quickperf/commits?author=ablanchard" title="Bug reports">🐛</a>
- correctness: строка 18: <a href="https://github.com/jvm-repo-rebuild/reproducible-central#org.quickperf:quick-perf">

## abstracta/jmeter-java-dsl

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/abstracta/jmeter-java-dsl
- Категория: benchmark/testing candidate
- Описание: Simple JMeter performance tests API
- Уровень: automated README evidence extraction
- Снимок: [sources/abstracta__jmeter-java-dsl/README.md](sources/abstracta__jmeter-java-dsl/README.md); SHA-256: `ebfb4b30f7ffff36610561417ed89d62f5be9612edb41e7ab66220cf3f5498f7`
- memory: строка 97: * [PerfOps - faster and cheaper through a service approach](https://habr.com/ru/company/oleg-bunin/blog/682746/): A nice analysis on implementing a performance experts service while using JMeter DSL as basics for creating a framework on top of it by Kirill Yur
- report: строка 12: Please join [discord server](https://discord.gg/WNSn5hqmSd) or create GitHub [issues](https://github.com/abstracta/jmeter-java-dsl/issues) and [discussions](https://github.com/abstracta/jmeter-java-dsl/discussions) to be part of the community and clear out dou
- correctness: строка 4: [![Reproducible Builds](https://img.shields.io/badge/Reproducible_Builds-ok-green?labelColor=1e5b96)](https://github.com/jvm-repo-rebuild/reproducible-central/blob/master/content/us/abstracta/jmeter/jmeter-java-dsl/README.md)

## kpdecker/six-speed

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kpdecker/six-speed
- Категория: benchmark/testing candidate
- Описание: ES6 polyfill vs. feature performance tests
- Уровень: automated README evidence extraction
- Снимок: [sources/kpdecker__six-speed/README.md](sources/kpdecker__six-speed/README.md); SHA-256: `22827ef1782a1535a8fce2f9f187aee543ca76e07575f7b2c7c6f10e89f09078`
- lifecycle: строка 72: ### VM Setup
- report: строка 5: Report is located at http://kpdecker.github.io/six-speed/
- isolation: строка 62: Babel, in both loose+runtime and runtime mode, and Babel was then used to compile the ES6 version to an ES5 compliant version, utilizing the runtime over polyfill to maintain test isolation and avoid native implementations where possible.

## krkn-chaos/krkn

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/krkn-chaos/krkn
- Категория: benchmark/testing candidate
- Описание: Chaos and resiliency testing tool for Kubernetes with a focus on improving performance under failure conditions. A CNCF sandbox project.
- Уровень: automated README evidence extraction
- Снимок: [sources/krkn-chaos__krkn/README.md](sources/krkn-chaos__krkn/README.md); SHA-256: `bed78f8d721c43095c66471e7906919de0e0c11285c4334e1f2be4a8b208e5f6`
- lifecycle: строка 25: Instructions on how to setup, configure and run Kraken can be found in the [documentation](https://krkn-chaos.dev/docs/).
- report: строка 42: We are always looking for more enhancements, fixes to make it better, any contributions are most welcome. Feel free to report or work on the issues filed on github.

## LucaCanali/Miscellaneous

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/LucaCanali/Miscellaneous
- Категория: benchmark/testing candidate
- Описание: Includes notes on using Apache Spark, with drill down on  Spark for Physics, how to run TPCDS on PySpark, how to create histograms with Spark. Also tools for stress testing, measuring CPUs' performance, and I/O latency heat maps. Jupyter notebooks examples for using various DB systems.
- Уровень: automated README evidence extraction
- Снимок: [sources/LucaCanali__Miscellaneous/README.md](sources/LucaCanali__Miscellaneous/README.md); SHA-256: `51b9ccb14650853e4e47a347ba55191e7561110f9d6f48d711b5f6bfaf004a66`
- report: строка 9: | [**Spark Dashboard**](Spark_Dashboard)                           | A tool for Apache monitoring, use to build a performance dashboard and troubleshoot Spark jobs.

## cgoldberg/multi-mechanize

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cgoldberg/multi-mechanize
- Категория: benchmark/testing candidate
- Описание: Performance Test Framework in Python
- Уровень: automated README evidence extraction
- Снимок: [sources/cgoldberg__multi-mechanize/README.rst](sources/cgoldberg__multi-mechanize/README.rst); SHA-256: `d70c1a38bd2e438b861ed98ddb71b3408483980c5f8e714e0a612bb2bdd672cc`
- lifecycle: строка 24: Install / Setup

## jacksonh/manos

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/jacksonh/manos
- Категория: benchmark/testing candidate
- Описание: Manos is an easy to use, easy to test, high performance web application framework that stays out of your way and makes your life ridiculously simple.
- Уровень: automated README evidence extraction
- Снимок: [sources/jacksonh__manos/README.md](sources/jacksonh__manos/README.md); SHA-256: `fb55b2435a7ea792576e4a4ae4e9cc66c7a194145e81bfdaf2e77ba573b33046`

## yswenli/SAEA

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yswenli/SAEA
- Категория: benchmark/testing candidate
- Описание: SAEA.Socket is a high-performance IOCP framework TCP based on dotnet standard 2.0; Src contains its application test scenarios, such as websocket,rpc, redis driver, MVC WebAPI, lightweight message server, ultra large file transmission, etc. SAEA.Socket是一个高性能IOCP框架的 TCP，基于dotnet standard 2.0；Src中含有其应用测试场景，例如websocket、rpc、redis驱动、MVC WebAPI、轻量级消息服务器、超大文件传输等
- Уровень: automated README evidence extraction
- Снимок: [sources/yswenli__SAEA/README.md](sources/yswenli__SAEA/README.md); SHA-256: `a14bf87c74d8217a053cb12da675c2606a65a291227d795afd48fbb238c9f4e9`
- report: строка 222: | **序列化** | Protobuf | JSON |

## patrickfav/BlurTestAndroid

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/patrickfav/BlurTestAndroid
- Категория: benchmark/testing candidate
- Описание: This is a simple App to test some blur algorithms on their visual quality and performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/patrickfav__BlurTestAndroid/README.md](sources/patrickfav__BlurTestAndroid/README.md); SHA-256: `12955fb89f0478daa5b027c02e8f5a75f9dc8a15d26f0161134a22bb614b512d`
- statistics: строка 34: The time of each round will be saved and from this data certain simple statistic can be calculated, like average and 95% confidence intervals.
- lifecycle: строка 31: A Benchmark consist of blurring a single image a defined number of rounds with a certain pixel radius. Each benchmark has a warm up
- memory: строка 21: After running some benchmarks you are presented with the results view, where you can click on each element and see a diagram on the length of each round. This also reveals the benchmarks usually are polluted by heap garbage collection.

## mgasiorowski/performance_testing

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mgasiorowski/performance_testing
- Категория: benchmark/testing candidate
- Описание: Tools, articles, etc. related to performance/load/etc. testing.
- Уровень: automated README evidence extraction
- Снимок: [sources/mgasiorowski__performance_testing/README.md](sources/mgasiorowski__performance_testing/README.md); SHA-256: `add51e6c2d38f2a31d101b11e135839829291d311bef3d96751bc6bf7447ba9b`
- statistics: строка 350: * [Statistical approaches for performance analysis](https://aakinshin.net/posts/statistics-for-performance/)
- memory: строка 280: * [speed-demon](https://github.com/morsssss/speed-demon)
- async: строка 386: * [Measuring of API performance of container cluster systems](https://docs.openstack.org/developer/performance-docs/test_plans/container_cluster_systems/API_latency.html#measuring-of-api-performance-of-container-cluster-system)
- report: строка 21: * [Reports](#reports)

## serputko/performance-testing-framework

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/serputko/performance-testing-framework
- Категория: benchmark/testing candidate
- Описание: Framework allows to perform load testing with Apache Jmeter, view application/server metrics in real-time with Grafana, analyze errors cause with detailed traces for failed requests, compare different test runs in scripted dashboard and perform frontend performance testing with sitespeed.io+webpagetest
- Уровень: automated README evidence extraction
- Снимок: [sources/serputko__performance-testing-framework/README.md](sources/serputko__performance-testing-framework/README.md); SHA-256: `27248994ac724ccc0553db96fc28aaf9b0ae98c42dfc27eb5ee97801d700880a`
- async: строка 144: #43	Threads: 3/10	Samples: 16	Latency: 23	Resp.Time: 197	Errors: 2
- report: строка 21: ### Custom Grafana dashboards features:

## aws-solutions/distributed-load-testing-on-aws

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/aws-solutions/distributed-load-testing-on-aws
- Категория: benchmark/testing candidate
- Описание: Distributed Load Testing on AWS automates performance testing at scale, demonstrating how systems behave under different load conditions and helping identify potential performance issues throughout their lifecycle.
- Уровень: automated README evidence extraction
- Снимок: [sources/aws-solutions__distributed-load-testing-on-aws/README.md](sources/aws-solutions__distributed-load-testing-on-aws/README.md); SHA-256: `9a25831bb08465e3263e25230dd08337d8c5a865298c791c1a2b6eecbf1b4210`
- statistics: строка 84: **AWS CDK Deployment**: For custom deployments and direct infrastructure-as-code management, the solution's infrastructure is developed with AWS CDK v2. See the [infrastructure README](source/infrastructure/README.md) for detailed CDK deployment instructions i
- lifecycle: строка 145: ### Environment Setup
- async: строка 6: - Simulate tens of thousands of concurrent users across multiple AWS Regions generating requests at a continuous pace.
- report: строка 109: A TypeScript package containing utilities for operational metrics collection and reporting used across the solution.
- correctness: строка 64: 15. An MCP client (AI development tool) connects to the [AWS AgentCore Gateway](https://aws.amazon.com/bedrock/agentcore/) endpoint to access the Distributed Load Testing solution's data through the Model Context Protocol. AgentCore Gateway validates the user'

## xullexer/PYDNS-Scanner

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/xullexer/PYDNS-Scanner
- Категория: benchmark/testing candidate
- Описание: A modern, high-performance DNS scanner with a beautiful Terminal User Interface (TUI) built with Textual. This tool can scan millions of IP addresses to find working DNS servers with optional Slipstream proxy testing and automatic multi-platform client download.
- Уровень: automated README evidence extraction
- Снимок: [sources/xullexer__PYDNS-Scanner/README.md](sources/xullexer__PYDNS-Scanner/README.md); SHA-256: `a5e7b2fa8e86aa122924582341058d64b64b4e34a1d85d91755a3ad200a4f956`
- lifecycle: строка 598: ### Development Setup
- memory: строка 535: ### High memory usage
- async: строка 44: - **DNS Scan** — New lightweight mode: just insert a domain, set concurrency, and scan — no proxy testing, no authentication. Tests security, DNS types, ping, resolved IP, EDNS0, and more
- report: строка 92: - 🌍 **IPv4 / IPv6 Detection** — Reports which IP versions each server supports

## techinz/browsers-benchmark

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/techinz/browsers-benchmark
- Категория: benchmark/testing candidate
- Описание: Browser automation engine benchmark - Test bypass rates, performance & stealth against Cloudflare, DataDome, reCAPTCHA, Kasada, Imperva, Akamai, PerimeterX  and other bot detection systems. Find the best browser for scraping.
- Уровень: automated README evidence extraction
- Снимок: [sources/techinz__browsers-benchmark/README.md](sources/techinz__browsers-benchmark/README.md); SHA-256: `335173f5db67cd0f8e8c6888ff32cd3177d847e0a75a29718f4ba52f4d14fffa`
- memory: строка 13: - **Performance Metrics**: Memory usage, CPU consumption, and page load times
- async: строка 66: **RapidProxy — a high-performance proxy provider built for developers working on web scraping, browser automation, multi-account management, and large-scale data collection. 90M+ global residential IPs, native static ISP proxies, smart rotation, stable session
- report: строка 44: - Automated report generation with visualizations
- correctness: строка 15: - **Network Analysis**: IP detection (proxy validation) and WebRTC leak testing

## ZoranPandovski/awesome-testing-tools

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/ZoranPandovski/awesome-testing-tools
- Категория: benchmark/testing candidate
- Описание: A curated collection of the best software testing tools for QA, automation, accessibility, performance, API, security, and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/ZoranPandovski__awesome-testing-tools/readme.md](sources/ZoranPandovski__awesome-testing-tools/readme.md); SHA-256: `5d3c98dd83724f08cad67bae412a3e617c6128119b31122c9177d5761aea0253`
- comparison: строка 23: - [BitDive](https://bitdive.io/) - Zero-code API testing platform for Java/Kotlin. Captures runtime behavior (HTTP, SQL, methods), auto-generates mocks from real traffic, and enables Live Context Replay for regression testing and debugging.
- lifecycle: строка 29: - [Cypress](https://www.cypress.io/) - JavaScript End-to-End testing framework. It allows you to run e2e tests effortlessly (no need to do a Java/Selenium setup in order to use it) with features such as debugging with Chrome DevTools and screenshots for tests
- async: строка 141: - [Beeceptor](https://beeceptor.com/) - Cloud based platform for API mocking, debugging, and traffic inspection that provides customizable endpoints for multi-protocol (HTTP, SOAP, GraphQL and gRPC) API services. It offers rule based request and state matching
- report: строка 33: - [Gwirian](https://www.gwirian.com/) - Open source BDD test management platform for QA teams. Create and organize features, define scenarios in Given/When/Then format, and track execution status—with full-text search, dashboards, and a modern UI. Self-host fo
- correctness: строка 51: - [Phpstan](https://phpstan.org) - Focuses on finding errors in your code without actually running it. It catches whole classes of bugs even before you write tests for the code. It moves PHP closer to compiled languages in the sense that the correctness of eac
- isolation: строка 22: - [Ava](https://github.com/avajs/ava) - Test runner for Node.js with a concise API, detailed error output, process isolation, and support for new language features.

## net-benchmark/dns-benchmark-tool

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/net-benchmark/dns-benchmark-tool
- Категория: benchmark/testing candidate
- Описание: Fast, comprehensive DNS performance testing with DNSSEC validation, DoH/DoT support, and enterprise features
- Уровень: automated README evidence extraction
- Снимок: [sources/net-benchmark__dns-benchmark-tool/README.md](sources/net-benchmark__dns-benchmark-tool/README.md); SHA-256: `a0766e467b43352260f5e18f1eaf2868f2b5ea622164d1aaf531d30c9c5a2074`
- statistics: строка 309: - **Statistical analysis** - Mean, median, P95, P99, jitter, consistency
- comparison: строка 348: | Plain UDP | *(default)* | baseline | Latency benchmarking |
- lifecycle: строка 129: - [📦 Installation \& Setup](#-installation--setup)
- async: строка 44: - 🔒 **DoH / DoT / DNSSEC** — encrypted DNS benchmarking with real latency tradeoff data
- report: строка 18: dns-benchmark benchmark --use-defaults --formats csv,excel
- correctness: строка 14: **Fast, comprehensive DNS performance testing with DNSSEC validation, DoH/DoT support, and enterprise features**

## arcprize/arc-agi-benchmarking

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/arcprize/arc-agi-benchmarking
- Категория: benchmark/testing candidate
- Описание: Testing baseline LLMs performance across various models
- Уровень: automated README evidence extraction
- Снимок: [sources/arcprize__arc-agi-benchmarking/README.md](sources/arcprize__arc-agi-benchmarking/README.md); SHA-256: `e448e6d42a861461099ace16b1da10d24764c9dfd7441d118124887dfe15d8f3`
- comparison: строка 21: 2) Single-task dry run (no API keys) with the local `random-baseline` adapter:
- async: строка 67: - `--configs`: Space-separated model config names to run concurrently.
- report: строка 58: - `--data_dir`: Folder containing ARC task `.json` files (e.g., `data/sample/tasks`).

## lemire/fastmod

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lemire/fastmod
- Категория: benchmark/testing candidate
- Описание: A C/C++ header file for fast 32-bit division remainders (and divisibility tests) on 64-bit hardware.
- Уровень: automated README evidence extraction
- Снимок: [sources/lemire__fastmod/README.md](sources/lemire__fastmod/README.md); SHA-256: `f2ca15aadd4f6843cb5bf8993c180a26a3f3cfd223c7971f75684dfeccf44c0e`
- correctness: строка 26: - **Comprehensive Tests**: Exhaustive unit tests ensure correctness.

## zszszszsz/.config

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/zszszszsz/.config
- Категория: benchmark/testing candidate
- Описание: # # Automatically generated file; DO NOT EDIT. # OpenWrt Configuration # CONFIG_MODULES=y CONFIG_HAVE_DOT_CONFIG=y # CONFIG_TARGET_sunxi is not set # CONFIG_TARGET_apm821xx is not set # CONFIG_TARGET_ath25 is not set CONFIG_TARGET_ar71xx=y # CONFIG_TARGET_ath79 is not set # CONFIG_TARGET_bcm27xx is not set # CONFIG_TARGET_bcm53xx is not set # CONFIG_TARGET_brcm47xx is not set # CONFIG_TARGET_brcm63xx is not set # CONFIG_TARGET_cns3xxx is not set # CONFIG_TARGET_octeon is not set # CONFIG_TARGET_gemini is not set # CONFIG_TARGET_mpc85xx is not set # CONFIG_TARGET_imx6 is not set # CONFIG_TARGET_mxs is not set # CONFIG_TARGET_ixp4xx is not set # CONFIG_TARGET_lantiq is not set # CONFIG_TARGET_malta is not set # CONFIG_TARGET_pistachio is not set # CONFIG_TARGET_mvebu is not set # CONFIG_TARGET_kirkwood is not set # CONFIG_TARGET_mediatek is not set # CONFIG_TARGET_ramips is not set # CONFIG_TARGET_at91 is not set # CONFIG_TARGET_rb532 is not set # CONFIG_TARGET_tegra is not set # CONFIG_TARGET_layerscape is not set # CONFIG_TARGET_octeontx is not set # CONFIG_TARGET_oxnas is not set # CONFIG_TARGET_armvirt is not set # CONFIG_TARGET_ipq40xx is not set # CONFIG_TARGET_ipq806x is not set # CONFIG_TARGET_ipq807x is not set # CONFIG_TARGET_samsung is not set # CONFIG_TARGET_arc770 is not set # CONFIG_TARGET_archs38 is not set # CONFIG_TARGET_ar7 is not set # CONFIG_TARGET_omap is not set # CONFIG_TARGET_uml is not set # CONFIG_TARGET_zynq is not set # CONFIG_TARGET_x86 is not set CONFIG_TARGET_ar71xx_generic=y # CONFIG_TARGET_ar71xx_tiny is not set # CONFIG_TARGET_ar71xx_nand is not set # CONFIG_TARGET_ar71xx_mikrotik is not set # CONFIG_TARGET_MULTI_PROFILE is not set # CONFIG_TARGET_ar71xx_generic_Default is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_carambola2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_lima is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ALFAAP120C is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ap121f is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ap91-5g is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ALFAAP96 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_HORNETUB is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_HORNETUBx2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ALFANX is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_n5q is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_r36a is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_TUBE2H16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_TUBE2H8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_fritz4020 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_fritz300e is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_fritz450e is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_sc1750 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_sc300m is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_sc450 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_c-55 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ALL0258N is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ALL0305 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ALL0315N is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_antminer-s1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_antminer-s3 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_antrouter-r1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_arduino-yun is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP121_16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP121_8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP132 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP135 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP136_010 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP136_020 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP96 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DB120 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_PB42 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_PB44 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_BXU2000N2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WZR450HP2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WZR600DHP is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WZRHPAG300H is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WZRHPG300NH is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WZRHPG300NH2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WZRHPG450H is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e316n-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e320n-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e355ac-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e355ac-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e375ac is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e380ac-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e380ac-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e385ac is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e520n is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cf-e530n is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WP543_16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WP543_8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WPE72_16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WPE72_8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wpj342 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wpj344 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wpj531 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wpj558 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wpj563 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dap-1330-a1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dap-2695-a1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DGL5500A1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DHP1565A1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DIR505A1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DIR825B1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DIR825C1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DIR835A1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dir-869-a1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dragino2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_el-m150 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_el-mini is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ew-balin is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ew-dorin is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ew-dorin-router is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_EAP300V2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ens202ext is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_EPG5000 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ESR1750 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ESR900 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-inet-6408A-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-inet-6416A-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-domino is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-ar150 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-ar300 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-ar300m is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-ar750 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-ar750s is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-mifi is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_gl-usb150 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_SGRW500N85BV2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_minibox-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_oolite-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_oolite-v5.2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_oolite-v5.2-dev is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_lan-turtle is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_packet-squirrel is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wifi-pineapple-nano is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_hiwifi-hc6361 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP147_010 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WRT160NL is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WRT400N is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_mr12 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_mr16 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_mc-mac1200r is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_mc-mw4530r is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_smart-300 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WNDAP360 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wndr3700 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wndr3700v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wndr3800 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wndr3800ch is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wndrmac is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wndrmacv2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_WNR2200 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_koala is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_omy-g1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_omy-x1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_onion-omega is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_som9331 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_A60 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_MR1750 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_MR600 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_MR900 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_OM2P is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_OM5PAC is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_OM5P is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cpe505n is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_r602n is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ts-d084 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_pqi-air-pen is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_k2t is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_MZKW04NU is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_MZKW300NH is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cap324 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cr3000 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cr5000 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_DLRTDEV01 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_qihoo-c301 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP143_16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP143_8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_AP152_16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e1700ac-v2-16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e1700ac-v2-8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e558-v2-16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e558-v2-8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e600g-v2-16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e600g-v2-8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e600gac-v2-16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e600gac-v2-8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e750a-v4-16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e750a-v4-8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e750g-v8-16M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_e750g-v8-8M is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ap531b0 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wam250 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_CAP4200AG is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_EAP7660D is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wlr8100 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_bsb is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c25-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c5-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c58-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c59-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c59-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c60-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c60-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c7-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c7-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c7-v2-il is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr7500-v3 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c7-v4 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_archer-c7-v5 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cpe210-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cpe210-220-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cpe510-520-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_eap120-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_re355-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_re450-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-mr6400-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr3227-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr3500-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr3600-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr4300-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr4300-v1-il is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr4310-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr4900-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr5800-v1 is not set CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr6500-v2=y # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr6500-v6 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wdr8500-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wpa8630-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr1043n-v5 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr1043nd-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr1043nd-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr1043nd-v3 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr1043nd-v4 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr2041n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr2041n-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr2543-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr710n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr710n-v2.1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr810n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr810n-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr842n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr842n-v2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr842n-v3 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr880n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr881n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr902ac-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr941n-v7 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tl-wr942n-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wbs210-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wbs510-v1 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_TEW673GRU is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_TEW732BR is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_TEW823DRU is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_tellstick-znet-lite is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_rut900 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-air-gateway is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-air-gateway-pro is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-airrouter is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-bullet-m is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-ls-sr71 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-lbe-m5 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-loco-m-xw is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-nano-m-xw is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-nano-m is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_rw2458n is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-rocket-m-ti is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-rocket-m-xw is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-rocket-m is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-rs is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-rspro is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-uap-pro is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-unifi is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-unifiac-lite is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-unifiac-mesh is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-unifiac-pro is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-unifi-outdoor is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubnt-unifi-outdoor-plus is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ubdev01 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_wrtnode2q is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dr342 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dr531 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_weio is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_mynet-n600 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_mynet-n750 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_MYNETREXT is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ap90q is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cpe830 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_cpe870 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_sr3200 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_t830 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_xd3200 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ZCN1523H28 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_ZCN1523H516 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_NBG6616 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dLAN_Hotspot is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dLAN_pro_1200_ac is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_dLAN_pro_500_wp is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_rme-eg200 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_JA76PF is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_JA76PF2 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_JWAP003 is not set # CONFIG_TARGET_ar71xx_generic_DEVICE_jwap230 is not set CONFIG_HAS_SUBTARGETS=y CONFIG_HAS_DEVICES=y CONFIG_TARGET_BOARD="ar71xx" CONFIG_TARGET_SUBTARGET="generic" CONFIG_TARGET_PROFILE="DEVICE_tl-wdr6500-v2" CONFIG_TARGET_ARCH_PACKAGES="mips_24kc" CONFIG_DEFAULT_TARGET_OPTIMIZATION="-Os -pipe -mno-branch-likely -mips32r2 -mtune=24kc" CONFIG_CPU_TYPE="24kc" CONFIG_LINUX_4_9=y CONFIG_DEFAULT_ath10k-firmware-qca988x=y CONFIG_DEFAULT_base-files=y CONFIG_DEFAULT_block-mount=y CONFIG_DEFAULT_busybox=y CONFIG_DEFAULT_ca-certificates=y CONFIG_DEFAULT_coremark=y CONFIG_DEFAULT_ddns-scripts_aliyun=y CONFIG_DEFAULT_ddns-scripts_dnspod=y CONFIG_DEFAULT_default-settings=y CONFIG_DEFAULT_dnsmasq-full=y CONFIG_DEFAULT_dropbear=y CONFIG_DEFAULT_firewall=y CONFIG_DEFAULT_fstools=y CONFIG_DEFAULT_iptables=y CONFIG_DEFAULT_kmod-ath10k=y CONFIG_DEFAULT_kmod-ath9k=y CONFIG_DEFAULT_kmod-gpio-button-hotplug=y CONFIG_DEFAULT_kmod-ipt-offload=y CONFIG_DEFAULT_kmod-ipt-raw=y CONFIG_DEFAULT_kmod-nf-nathelper=y CONFIG_DEFAULT_kmod-nf-nathelper-extra=y CONFIG_DEFAULT_kmod-tcp-bbr=y CONFIG_DEFAULT_kmod-usb-core=y CONFIG_DEFAULT_kmod-usb-ledtrig-usbport=y CONFIG_DEFAULT_kmod-usb2=y CONFIG_DEFAULT_libc=y CONFIG_DEFAULT_libgcc=y CONFIG_DEFAULT_libustream-openssl=y CONFIG_DEFAULT_logd=y CONFIG_DEFAULT_luci=y CONFIG_DEFAULT_luci-app-accesscontrol=y CONFIG_DEFAULT_luci-app-adbyby-plus=y CONFIG_DEFAULT_luci-app-arpbind=y CONFIG_DEFAULT_luci-app-autoreboot=y CONFIG_DEFAULT_luci-app-cpufreq=y CONFIG_DEFAULT_luci-app-ddns=y CONFIG_DEFAULT_luci-app-filetransfer=y CONFIG_DEFAULT_luci-app-flowoffload=y CONFIG_DEFAULT_luci-app-nlbwmon=y CONFIG_DEFAULT_luci-app-ramfree=y CONFIG_DEFAULT_luci-app-sfe=y CONFIG_DEFAULT_luci-app-ssr-plus=y CONFIG_DEFAULT_luci-app-unblockmusic=y CONFIG_DEFAULT_luci-app-upnp=y CONFIG_DEFAULT_luci-app-vlmcsd=y CONFIG_DEFAULT_luci-app-vsftpd=y CONFIG_DEFAULT_luci-app-wol=y CONFIG_DEFAULT_mtd=y CONFIG_DEFAULT_netifd=y CONFIG_DEFAULT_opkg=y CONFIG_DEFAULT_ppp=y CONFIG_DEFAULT_ppp-mod-pppoe=y CONFIG_DEFAULT_swconfig=y CONFIG_DEFAULT_uboot-envtools=y CONFIG_DEFAULT_uci=y CONFIG_DEFAULT_uclient-fetch=y CONFIG_DEFAULT_urandom-seed=y CONFIG_DEFAULT_urngd=y CONFIG_DEFAULT_wget=y CONFIG_DEFAULT_wpad-openssl=y CONFIG_AUDIO_SUPPORT=y CONFIG_GPIO_SUPPORT=y CONFIG_PCI_SUPPORT=y CONFIG_USB_SUPPORT=y CONFIG_USB_GADGET_SUPPORT=y CONFIG_BIG_ENDIAN=y CONFIG_USES_SQUASHFS=y CONFIG_SMALL_FLASH=y CONFIG_HAS_MIPS16=y CONFIG_mips=y CONFIG_ARCH="mips"  # # Target Images # # CONFIG_TARGET_ROOTFS_INITRAMFS is not set CONFIG_EXTERNAL_CPIO=""  # # Root filesystem archives # # CONFIG_TARGET_ROOTFS_CPIOGZ is not set # CONFIG_TARGET_ROOTFS_TARGZ is not set  # # Root filesystem images # # CONFIG_TARGET_ROOTFS_EXT4FS is not set CONFIG_TARGET_ROOTFS_SQUASHFS=y CONFIG_TARGET_SQUASHFS_BLOCK_SIZE=1024 CONFIG_TARGET_UBIFS_FREE_SPACE_FIXUP=y CONFIG_TARGET_UBIFS_JOURNAL_SIZE=""  # # Image Options #  # # Global build settings # # CONFIG_JSON_ADD_IMAGE_INFO is not set # CONFIG_ALL_NONSHARED is not set # CONFIG_ALL_KMODS is not set # CONFIG_ALL is not set # CONFIG_BUILDBOT is not set # CONFIG_SIGNED_PACKAGES is not set # CONFIG_SIGNATURE_CHECK is not set  # # General build options # # CONFIG_DISPLAY_SUPPORT is not set CONFIG_BUILD_PATENTED=y # CONFIG_BUILD_NLS is not set CONFIG_SHADOW_PASSWORDS=y # CONFIG_CLEAN_IPKG is not set # CONFIG_INCLUDE_CONFIG is not set # CONFIG_COLLECT_KERNEL_DEBUG is not set  # # Kernel build options # CONFIG_KERNEL_BUILD_USER="" CONFIG_KERNEL_BUILD_DOMAIN="" CONFIG_KERNEL_PRINTK=y CONFIG_KERNEL_CRASHLOG=y # CONFIG_KERNEL_SWAP is not set CONFIG_KERNEL_DEBUG_FS=y CONFIG_KERNEL_MIPS_FPU_EMULATOR=y # CONFIG_KERNEL_PERF_EVENTS is not set # CONFIG_KERNEL_PROFILING is not set # CONFIG_KERNEL_TASKSTATS is not set # CONFIG_KERNEL_KALLSYMS is not set # CONFIG_KERNEL_FTRACE is not set # CONFIG_KERNEL_DEBUG_KERNEL is not set # CONFIG_KERNEL_DEBUG_INFO is not set # CONFIG_KERNEL_DYNAMIC_DEBUG is not set # CONFIG_KERNEL_KPROBES is not set # CONFIG_KERNEL_AIO is not set # CONFIG_KERNEL_FHANDLE is not set # CONFIG_KERNEL_FANOTIFY is not set # CONFIG_KERNEL_BLK_DEV_BSG is not set CONFIG_KERNEL_MAGIC_SYSRQ=y # CONFIG_KERNEL_DEBUG_PINCTRL is not set # CONFIG_KERNEL_DEBUG_GPIO is not set # CONFIG_KERNEL_ELF_CORE is not set # CONFIG_KERNEL_PROVE_LOCKING is not set CONFIG_KERNEL_PRINTK_TIME=y # CONFIG_KERNEL_SLABINFO is not set # CONFIG_KERNEL_PROC_PAGE_MONITOR is not set # CONFIG_KERNEL_KEXEC is not set # CONFIG_USE_RFKILL is not set # CONFIG_USE_SPARSE is not set # CONFIG_KERNEL_DEVTMPFS is not set CONFIG_KERNEL_KEYS=y # CONFIG_KERNEL_PERSISTENT_KEYRINGS is not set # CONFIG_KERNEL_BIG_KEYS is not set # CONFIG_KERNEL_ENCRYPTED_KEYS is not set CONFIG_KERNEL_CGROUPS=y # CONFIG_KERNEL_CGROUP_DEBUG is not set CONFIG_KERNEL_FREEZER=y CONFIG_KERNEL_CGROUP_FREEZER=y CONFIG_KERNEL_CGROUP_DEVICE=y CONFIG_KERNEL_CGROUP_PIDS=y CONFIG_KERNEL_CPUSETS=y # CONFIG_KERNEL_PROC_PID_CPUSET is not set CONFIG_KERNEL_CGROUP_CPUACCT=y # CONFIG_KERNEL_RESOURCE_COUNTERS is not set CONFIG_KERNEL_MM_OWNER=y CONFIG_KERNEL_MEMCG=y # CONFIG_KERNEL_MEMCG_SWAP is not set # CONFIG_KERNEL_MEMCG_KMEM is not set # CONFIG_KERNEL_CGROUP_PERF is not set CONFIG_KERNEL_CGROUP_SCHED=y # CONFIG_KERNEL_FAIR_GROUP_SCHED is not set # CONFIG_KERNEL_RT_GROUP_SCHED is not set CONFIG_KERNEL_BLK_CGROUP=y # CONFIG_KERNEL_CFQ_GROUP_IOSCHED is not set # CONFIG_KERNEL_BLK_DEV_THROTTLING is not set # CONFIG_KERNEL_DEBUG_BLK_CGROUP is not set CONFIG_KERNEL_NET_CLS_CGROUP=y CONFIG_KERNEL_NETPRIO_CGROUP=y CONFIG_KERNEL_NAMESPACES=y CONFIG_KERNEL_UTS_NS=y CONFIG_KERNEL_IPC_NS=y CONFIG_KERNEL_USER_NS=y CONFIG_KERNEL_PID_NS=y CONFIG_KERNEL_NET_NS=y CONFIG_KERNEL_DEVPTS_MULTIPLE_INSTANCES=y CONFIG_KERNEL_POSIX_MQUEUE=y # CONFIG_KERNEL_SECCOMP_FILTER is not set # CONFIG_KERNEL_SECCOMP is not set CONFIG_KERNEL_IP_MROUTE=y CONFIG_KERNEL_IPV6=y CONFIG_KERNEL_IPV6_MULTIPLE_TABLES=y CONFIG_KERNEL_IPV6_SUBTREES=y CONFIG_KERNEL_IPV6_MROUTE=y # CONFIG_KERNEL_IPV6_PIMSM_V2 is not set # CONFIG_KERNEL_IP_PNP is not set  # # Filesystem ACL and attr support options # # CONFIG_USE_FS_ACL_ATTR is not set # CONFIG_KERNEL_FS_POSIX_ACL is not set # CONFIG_KERNEL_BTRFS_FS_POSIX_ACL is not set # CONFIG_KERNEL_EXT4_FS_POSIX_ACL is not set # CONFIG_KERNEL_F2FS_FS_POSIX_ACL is not set # CONFIG_KERNEL_JFFS2_FS_POSIX_ACL is not set # CONFIG_KERNEL_TMPFS_POSIX_ACL is not set # CONFIG_KERNEL_CIFS_ACL is not set # CONFIG_KERNEL_HFS_FS_POSIX_ACL is not set # CONFIG_KERNEL_HFSPLUG_FS_POSIX_ACL is not set # CONFIG_KERNEL_NFS_ACL_SUPPORT is not set # CONFIG_KERNEL_NFS_V3_ACL_SUPPORT is not set # CONFIG_KERNEL_NFSD_V2_ACL_SUPPORT is not set # CONFIG_KERNEL_NFSD_V3_ACL_SUPPORT is not set # CONFIG_KERNEL_REISER_FS_POSIX_ACL is not set # CONFIG_KERNEL_XFS_POSIX_ACL is not set # CONFIG_KERNEL_JFS_POSIX_ACL is not set # CONFIG_KERNEL_DEVMEM is not set # CONFIG_KERNEL_DEVKMEM is not set CONFIG_KERNEL_SQUASHFS_FRAGMENT_CACHE_SIZE=2 # CONFIG_KERNEL_CC_OPTIMIZE_FOR_PERFORMANCE is not set CONFIG_KERNEL_CC_OPTIMIZE_FOR_SIZE=y  # # Package build options # # CONFIG_DEBUG is not set CONFIG_IPV6=y  # # Stripping options # # CONFIG_NO_STRIP is not set # CONFIG_USE_STRIP is not set CONFIG_USE_SSTRIP=y # CONFIG_STRIP_KERNEL_EXPORTS is not set # CONFIG_USE_MKLIBS is not set CONFIG_USE_UCLIBCXX=y # CONFIG_USE_LIBCXX is not set # CONFIG_USE_LIBSTDCXX is not set  # # Hardening build options # CONFIG_PKG_CHECK_FORMAT_SECURITY=y CONFIG_PKG_ASLR_PIE_NONE=y # CONFIG_PKG_ASLR_PIE_REGULAR is not set # CONFIG_PKG_ASLR_PIE_ALL is not set # CONFIG_PKG_CC_STACKPROTECTOR_NONE is not set CONFIG_PKG_CC_STACKPROTECTOR_REGULAR=y # CONFIG_KERNEL_CC_STACKPROTECTOR_NONE is not set CONFIG_KERNEL_CC_STACKPROTECTOR_REGULAR=y # CONFIG_KERNEL_CC_STACKPROTECTOR_STRONG is not set CONFIG_KERNEL_STACKPROTECTOR=y # CONFIG_KERNEL_STACKPROTECTOR_STRONG is not set # CONFIG_PKG_FORTIFY_SOURCE_NONE is not set CONFIG_PKG_FORTIFY_SOURCE_1=y # CONFIG_PKG_FORTIFY_SOURCE_2 is not set # CONFIG_PKG_RELRO_NONE is not set # CONFIG_PKG_RELRO_PARTIAL is not set CONFIG_PKG_RELRO_FULL=y # CONFIG_DEVEL is not set # CONFIG_BROKEN is not set CONFIG_BINARY_FOLDER="" CONFIG_DOWNLOAD_FOLDER="" CONFIG_LOCALMIRROR="" CONFIG_AUTOREBUILD=y # CONFIG_AUTOREMOVE is not set CONFIG_BUILD_SUFFIX="" CONFIG_TARGET_ROOTFS_DIR="" # CONFIG_CCACHE is not set CONFIG_EXTERNAL_KERNEL_TREE="" CONFIG_KERNEL_GIT_CLONE_URI="" CONFIG_EXTRA_OPTIMIZATION="-fno-caller-saves -fno-plt" CONFIG_TARGET_OPTIMIZATION="-Os -pipe -mno-branch-likely -mips32r2 -mtune=24kc" CONFIG_SOFT_FLOAT=y CONFIG_USE_MIPS16=y # CONFIG_EXTRA_TARGET_ARCH is not set CONFIG_EXTRA_BINUTILS_CONFIG_OPTIONS="" CONFIG_EXTRA_GCC_CONFIG_OPTIONS="" # CONFIG_GCC_DEFAULT_PIE is not set # CONFIG_GCC_DEFAULT_SSP is not set # CONFIG_SJLJ_EXCEPTIONS is not set # CONFIG_INSTALL_GFORTRAN is not set CONFIG_GDB=y CONFIG_USE_MUSL=y CONFIG_SSP_SUPPORT=y CONFIG_BINUTILS_VERSION_2_31_1=y CONFIG_BINUTILS_VERSION="2.31.1" # CONFIG_GCC_USE_EMBEDDED_PATH_REMAP is not set CONFIG_GCC_VERSION="7.5.0" CONFIG_LIBC="musl" CONFIG_TARGET_SUFFIX="musl" # CONFIG_IB is not set # CONFIG_SDK is not set # CONFIG_MAKE_TOOLCHAIN is not set # CONFIG_IMAGEOPT is not set # CONFIG_PREINITOPT is not set CONFIG_TARGET_PREINIT_SUPPRESS_STDERR=y # CONFIG_TARGET_PREINIT_DISABLE_FAILSAFE is not set CONFIG_TARGET_PREINIT_TIMEOUT=2 # CONFIG_TARGET_PREINIT_SHOW_NETMSG is not set # CONFIG_TARGET_PREINIT_SUPPRESS_FAILSAFE_NETMSG is not set CONFIG_TARGET_PREINIT_IFNAME="" CONFIG_TARGET_PREINIT_IP="192.168.1.1" CONFIG_TARGET_PREINIT_NETMASK="255.255.255.0" CONFIG_TARGET_PREINIT_BROADCAST="192.168.1.255" # CONFIG_INITOPT is not set CONFIG_TARGET_INIT_PATH="/usr/sbin:/usr/bin:/sbin:/bin" CONFIG_TARGET_INIT_ENV="" CONFIG_TARGET_INIT_CMD="/sbin/init" CONFIG_TARGET_INIT_SUPPRESS_STDERR=y # CONFIG_VERSIONOPT is not set CONFIG_PER_FEED_REPO=y CONFIG_FEED_packages=y CONFIG_FEED_luci=y CONFIG_FEED_routing=y  # # Base system # # CONFIG_PACKAGE_attendedsysupgrade-common is not set # CONFIG_PACKAGE_auc is not set CONFIG_PACKAGE_base-files=y CONFIG_PACKAGE_block-mount=y # CONFIG_PACKAGE_blockd is not set # CONFIG_PACKAGE_bridge is not set CONFIG_PACKAGE_busybox=y # CONFIG_BUSYBOX_CUSTOM is not set CONFIG_BUSYBOX_DEFAULT_HAVE_DOT_CONFIG=y # CONFIG_BUSYBOX_DEFAULT_DESKTOP is not set # CONFIG_BUSYBOX_DEFAULT_EXTRA_COMPAT is not set # CONFIG_BUSYBOX_DEFAULT_FEDORA_COMPAT is not set CONFIG_BUSYBOX_DEFAULT_INCLUDE_SUSv2=y CONFIG_BUSYBOX_DEFAULT_LONG_OPTS=y CONFIG_BUSYBOX_DEFAULT_SHOW_USAGE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VERBOSE_USAGE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_COMPRESS_USAGE=y CONFIG_BUSYBOX_DEFAULT_LFS=y # CONFIG_BUSYBOX_DEFAULT_PAM is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_DEVPTS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_UTMP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WTMP is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_PIDFILE=y CONFIG_BUSYBOX_DEFAULT_PID_FILE_PATH="/var/run" # CONFIG_BUSYBOX_DEFAULT_BUSYBOX is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SHOW_SCRIPT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSTALLER is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL_NO_USR is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SUID=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_SUID_CONFIG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SUID_CONFIG_QUIET is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_PREFER_APPLETS=y CONFIG_BUSYBOX_DEFAULT_BUSYBOX_EXEC_PATH="/proc/self/exe" # CONFIG_BUSYBOX_DEFAULT_SELINUX is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CLEAN_UP is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SYSLOG=y CONFIG_BUSYBOX_DEFAULT_PLATFORM_LINUX=y # CONFIG_BUSYBOX_DEFAULT_STATIC is not set # CONFIG_BUSYBOX_DEFAULT_PIE is not set # CONFIG_BUSYBOX_DEFAULT_NOMMU is not set # CONFIG_BUSYBOX_DEFAULT_BUILD_LIBBUSYBOX is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LIBBUSYBOX_STATIC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INDIVIDUAL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SHARED_BUSYBOX is not set CONFIG_BUSYBOX_DEFAULT_CROSS_COMPILER_PREFIX="" CONFIG_BUSYBOX_DEFAULT_SYSROOT="" CONFIG_BUSYBOX_DEFAULT_EXTRA_CFLAGS="" CONFIG_BUSYBOX_DEFAULT_EXTRA_LDFLAGS="" CONFIG_BUSYBOX_DEFAULT_EXTRA_LDLIBS="" # CONFIG_BUSYBOX_DEFAULT_USE_PORTABLE_CODE is not set # CONFIG_BUSYBOX_DEFAULT_STACK_OPTIMIZATION_386 is not set CONFIG_BUSYBOX_DEFAULT_INSTALL_APPLET_SYMLINKS=y # CONFIG_BUSYBOX_DEFAULT_INSTALL_APPLET_HARDLINKS is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL_APPLET_SCRIPT_WRAPPERS is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL_APPLET_DONT is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL_SH_APPLET_SYMLINK is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL_SH_APPLET_HARDLINK is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL_SH_APPLET_SCRIPT_WRAPPER is not set CONFIG_BUSYBOX_DEFAULT_PREFIX="./_install" # CONFIG_BUSYBOX_DEFAULT_DEBUG is not set # CONFIG_BUSYBOX_DEFAULT_DEBUG_PESSIMIZE is not set # CONFIG_BUSYBOX_DEFAULT_DEBUG_SANITIZE is not set # CONFIG_BUSYBOX_DEFAULT_UNIT_TEST is not set # CONFIG_BUSYBOX_DEFAULT_WERROR is not set CONFIG_BUSYBOX_DEFAULT_NO_DEBUG_LIB=y # CONFIG_BUSYBOX_DEFAULT_DMALLOC is not set # CONFIG_BUSYBOX_DEFAULT_EFENCE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_USE_BSS_TAIL is not set # CONFIG_BUSYBOX_DEFAULT_FLOAT_DURATION is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_RTMINMAX is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_RTMINMAX_USE_LIBC_DEFINITIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_BUFFERS_USE_MALLOC is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_BUFFERS_GO_ON_STACK=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_BUFFERS_GO_IN_BSS is not set CONFIG_BUSYBOX_DEFAULT_PASSWORD_MINLEN=6 CONFIG_BUSYBOX_DEFAULT_MD5_SMALL=1 CONFIG_BUSYBOX_DEFAULT_SHA3_SMALL=1 CONFIG_BUSYBOX_DEFAULT_FEATURE_FAST_TOP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_ETC_NETWORKS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_ETC_SERVICES is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING=y CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_MAX_LEN=512 # CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_VI is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_HISTORY=256 # CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_SAVEHISTORY is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_SAVE_ON_EXIT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_REVERSE_SEARCH is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_TAB_COMPLETION=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_USERNAME_COMPLETION is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_FANCY_PROMPT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_WINCH is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_EDITING_ASK_TERMINAL is not set # CONFIG_BUSYBOX_DEFAULT_LOCALE_SUPPORT is not set # CONFIG_BUSYBOX_DEFAULT_UNICODE_SUPPORT is not set # CONFIG_BUSYBOX_DEFAULT_UNICODE_USING_LOCALE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHECK_UNICODE_IN_ENV is not set CONFIG_BUSYBOX_DEFAULT_SUBST_WCHAR=0 CONFIG_BUSYBOX_DEFAULT_LAST_SUPPORTED_WCHAR=0 # CONFIG_BUSYBOX_DEFAULT_UNICODE_COMBINING_WCHARS is not set # CONFIG_BUSYBOX_DEFAULT_UNICODE_WIDE_WCHARS is not set # CONFIG_BUSYBOX_DEFAULT_UNICODE_BIDI_SUPPORT is not set # CONFIG_BUSYBOX_DEFAULT_UNICODE_NEUTRAL_TABLE is not set # CONFIG_BUSYBOX_DEFAULT_UNICODE_PRESERVE_BROKEN is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_NON_POSIX_CP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_VERBOSE_CP_MESSAGE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_USE_SENDFILE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_COPYBUF_KB=4 # CONFIG_BUSYBOX_DEFAULT_FEATURE_SKIP_ROOTFS is not set # CONFIG_BUSYBOX_DEFAULT_MONOTONIC_SYSCALL is not set CONFIG_BUSYBOX_DEFAULT_IOCTL_HEX2STR_ERROR=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_HWIB is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SEAMLESS_XZ is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SEAMLESS_LZMA is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SEAMLESS_BZ2 is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SEAMLESS_GZ=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_SEAMLESS_Z is not set # CONFIG_BUSYBOX_DEFAULT_AR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_AR_LONG_FILENAMES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_AR_CREATE is not set # CONFIG_BUSYBOX_DEFAULT_UNCOMPRESS is not set CONFIG_BUSYBOX_DEFAULT_GUNZIP=y CONFIG_BUSYBOX_DEFAULT_ZCAT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_GUNZIP_LONG_OPTIONS is not set CONFIG_BUSYBOX_DEFAULT_BUNZIP2=y CONFIG_BUSYBOX_DEFAULT_BZCAT=y # CONFIG_BUSYBOX_DEFAULT_UNLZMA is not set # CONFIG_BUSYBOX_DEFAULT_LZCAT is not set # CONFIG_BUSYBOX_DEFAULT_LZMA is not set # CONFIG_BUSYBOX_DEFAULT_UNXZ is not set # CONFIG_BUSYBOX_DEFAULT_XZCAT is not set # CONFIG_BUSYBOX_DEFAULT_XZ is not set # CONFIG_BUSYBOX_DEFAULT_BZIP2 is not set CONFIG_BUSYBOX_DEFAULT_BZIP2_SMALL=0 CONFIG_BUSYBOX_DEFAULT_FEATURE_BZIP2_DECOMPRESS=y # CONFIG_BUSYBOX_DEFAULT_CPIO is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CPIO_O is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CPIO_P is not set # CONFIG_BUSYBOX_DEFAULT_DPKG is not set # CONFIG_BUSYBOX_DEFAULT_DPKG_DEB is not set CONFIG_BUSYBOX_DEFAULT_GZIP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_GZIP_LONG_OPTIONS is not set CONFIG_BUSYBOX_DEFAULT_GZIP_FAST=0 # CONFIG_BUSYBOX_DEFAULT_FEATURE_GZIP_LEVELS is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_GZIP_DECOMPRESS=y # CONFIG_BUSYBOX_DEFAULT_LZOP is not set # CONFIG_BUSYBOX_DEFAULT_UNLZOP is not set # CONFIG_BUSYBOX_DEFAULT_LZOPCAT is not set # CONFIG_BUSYBOX_DEFAULT_LZOP_COMPR_HIGH is not set # CONFIG_BUSYBOX_DEFAULT_RPM is not set # CONFIG_BUSYBOX_DEFAULT_RPM2CPIO is not set CONFIG_BUSYBOX_DEFAULT_TAR=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_LONG_OPTIONS is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_CREATE=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_AUTODETECT is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_FROM=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_OLDGNU_COMPATIBILITY is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_OLDSUN_COMPATIBILITY is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_GNU_EXTENSIONS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_TO_COMMAND is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_UNAME_GNAME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_NOPRESERVE_TIME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TAR_SELINUX is not set # CONFIG_BUSYBOX_DEFAULT_UNZIP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UNZIP_CDF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UNZIP_BZIP2 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UNZIP_LZMA is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UNZIP_XZ is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LZMA_FAST is not set CONFIG_BUSYBOX_DEFAULT_BASENAME=y CONFIG_BUSYBOX_DEFAULT_CAT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_CATN is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CATV is not set CONFIG_BUSYBOX_DEFAULT_CHGRP=y CONFIG_BUSYBOX_DEFAULT_CHMOD=y CONFIG_BUSYBOX_DEFAULT_CHOWN=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHOWN_LONG_OPTIONS is not set CONFIG_BUSYBOX_DEFAULT_CHROOT=y # CONFIG_BUSYBOX_DEFAULT_CKSUM is not set # CONFIG_BUSYBOX_DEFAULT_COMM is not set CONFIG_BUSYBOX_DEFAULT_CP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_CP_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CP_REFLINK is not set CONFIG_BUSYBOX_DEFAULT_CUT=y CONFIG_BUSYBOX_DEFAULT_DATE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_DATE_ISOFMT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_DATE_NANO is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DATE_COMPAT is not set CONFIG_BUSYBOX_DEFAULT_DD=y CONFIG_BUSYBOX_DEFAULT_FEATURE_DD_SIGNAL_HANDLING=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_DD_THIRD_STATUS_LINE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_DD_IBS_OBS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_DD_STATUS is not set CONFIG_BUSYBOX_DEFAULT_DF=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_DF_FANCY is not set CONFIG_BUSYBOX_DEFAULT_DIRNAME=y # CONFIG_BUSYBOX_DEFAULT_DOS2UNIX is not set # CONFIG_BUSYBOX_DEFAULT_UNIX2DOS is not set CONFIG_BUSYBOX_DEFAULT_DU=y CONFIG_BUSYBOX_DEFAULT_FEATURE_DU_DEFAULT_BLOCKSIZE_1K=y CONFIG_BUSYBOX_DEFAULT_ECHO=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FANCY_ECHO=y CONFIG_BUSYBOX_DEFAULT_ENV=y # CONFIG_BUSYBOX_DEFAULT_EXPAND is not set # CONFIG_BUSYBOX_DEFAULT_UNEXPAND is not set CONFIG_BUSYBOX_DEFAULT_EXPR=y CONFIG_BUSYBOX_DEFAULT_EXPR_MATH_SUPPORT_64=y # CONFIG_BUSYBOX_DEFAULT_FACTOR is not set CONFIG_BUSYBOX_DEFAULT_FALSE=y # CONFIG_BUSYBOX_DEFAULT_FOLD is not set CONFIG_BUSYBOX_DEFAULT_FSYNC=y CONFIG_BUSYBOX_DEFAULT_HEAD=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FANCY_HEAD=y # CONFIG_BUSYBOX_DEFAULT_HOSTID is not set CONFIG_BUSYBOX_DEFAULT_ID=y # CONFIG_BUSYBOX_DEFAULT_GROUPS is not set # CONFIG_BUSYBOX_DEFAULT_INSTALL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSTALL_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_LINK is not set CONFIG_BUSYBOX_DEFAULT_LN=y # CONFIG_BUSYBOX_DEFAULT_LOGNAME is not set CONFIG_BUSYBOX_DEFAULT_LS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_FILETYPES=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_FOLLOWLINKS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_RECURSIVE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_WIDTH=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_SORTFILES=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_TIMESTAMPS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_USERNAME=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_COLOR=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LS_COLOR_IS_DEFAULT=y CONFIG_BUSYBOX_DEFAULT_MD5SUM=y # CONFIG_BUSYBOX_DEFAULT_SHA1SUM is not set CONFIG_BUSYBOX_DEFAULT_SHA256SUM=y # CONFIG_BUSYBOX_DEFAULT_SHA512SUM is not set # CONFIG_BUSYBOX_DEFAULT_SHA3SUM is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_MD5_SHA1_SUM_CHECK=y CONFIG_BUSYBOX_DEFAULT_MKDIR=y CONFIG_BUSYBOX_DEFAULT_MKFIFO=y CONFIG_BUSYBOX_DEFAULT_MKNOD=y CONFIG_BUSYBOX_DEFAULT_MKTEMP=y CONFIG_BUSYBOX_DEFAULT_MV=y CONFIG_BUSYBOX_DEFAULT_NICE=y # CONFIG_BUSYBOX_DEFAULT_NL is not set # CONFIG_BUSYBOX_DEFAULT_NOHUP is not set # CONFIG_BUSYBOX_DEFAULT_NPROC is not set # CONFIG_BUSYBOX_DEFAULT_OD is not set # CONFIG_BUSYBOX_DEFAULT_PASTE is not set # CONFIG_BUSYBOX_DEFAULT_PRINTENV is not set CONFIG_BUSYBOX_DEFAULT_PRINTF=y CONFIG_BUSYBOX_DEFAULT_PWD=y CONFIG_BUSYBOX_DEFAULT_READLINK=y CONFIG_BUSYBOX_DEFAULT_FEATURE_READLINK_FOLLOW=y # CONFIG_BUSYBOX_DEFAULT_REALPATH is not set CONFIG_BUSYBOX_DEFAULT_RM=y CONFIG_BUSYBOX_DEFAULT_RMDIR=y CONFIG_BUSYBOX_DEFAULT_SEQ=y # CONFIG_BUSYBOX_DEFAULT_SHRED is not set # CONFIG_BUSYBOX_DEFAULT_SHUF is not set CONFIG_BUSYBOX_DEFAULT_SLEEP=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FANCY_SLEEP=y CONFIG_BUSYBOX_DEFAULT_SORT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_SORT_BIG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SORT_OPTIMIZE_MEMORY is not set # CONFIG_BUSYBOX_DEFAULT_SPLIT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SPLIT_FANCY is not set # CONFIG_BUSYBOX_DEFAULT_STAT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_STAT_FORMAT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_STAT_FILESYSTEM is not set # CONFIG_BUSYBOX_DEFAULT_STTY is not set # CONFIG_BUSYBOX_DEFAULT_SUM is not set CONFIG_BUSYBOX_DEFAULT_SYNC=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_SYNC_FANCY is not set # CONFIG_BUSYBOX_DEFAULT_TAC is not set CONFIG_BUSYBOX_DEFAULT_TAIL=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FANCY_TAIL=y CONFIG_BUSYBOX_DEFAULT_TEE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_TEE_USE_BLOCK_IO=y CONFIG_BUSYBOX_DEFAULT_TEST=y CONFIG_BUSYBOX_DEFAULT_TEST1=y CONFIG_BUSYBOX_DEFAULT_TEST2=y CONFIG_BUSYBOX_DEFAULT_FEATURE_TEST_64=y # CONFIG_BUSYBOX_DEFAULT_TIMEOUT is not set CONFIG_BUSYBOX_DEFAULT_TOUCH=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TOUCH_NODEREF is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_TOUCH_SUSV3=y CONFIG_BUSYBOX_DEFAULT_TR=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TR_CLASSES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TR_EQUIV is not set CONFIG_BUSYBOX_DEFAULT_TRUE=y # CONFIG_BUSYBOX_DEFAULT_TRUNCATE is not set # CONFIG_BUSYBOX_DEFAULT_TTY is not set CONFIG_BUSYBOX_DEFAULT_UNAME=y CONFIG_BUSYBOX_DEFAULT_UNAME_OSNAME="GNU/Linux" # CONFIG_BUSYBOX_DEFAULT_BB_ARCH is not set CONFIG_BUSYBOX_DEFAULT_UNIQ=y # CONFIG_BUSYBOX_DEFAULT_UNLINK is not set # CONFIG_BUSYBOX_DEFAULT_USLEEP is not set # CONFIG_BUSYBOX_DEFAULT_UUDECODE is not set # CONFIG_BUSYBOX_DEFAULT_BASE64 is not set # CONFIG_BUSYBOX_DEFAULT_UUENCODE is not set CONFIG_BUSYBOX_DEFAULT_WC=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_WC_LARGE is not set # CONFIG_BUSYBOX_DEFAULT_WHO is not set # CONFIG_BUSYBOX_DEFAULT_W is not set # CONFIG_BUSYBOX_DEFAULT_USERS is not set # CONFIG_BUSYBOX_DEFAULT_WHOAMI is not set CONFIG_BUSYBOX_DEFAULT_YES=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_VERBOSE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_PRESERVE_HARDLINKS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_HUMAN_READABLE=y # CONFIG_BUSYBOX_DEFAULT_CHVT is not set CONFIG_BUSYBOX_DEFAULT_CLEAR=y # CONFIG_BUSYBOX_DEFAULT_DEALLOCVT is not set # CONFIG_BUSYBOX_DEFAULT_DUMPKMAP is not set # CONFIG_BUSYBOX_DEFAULT_FGCONSOLE is not set # CONFIG_BUSYBOX_DEFAULT_KBD_MODE is not set # CONFIG_BUSYBOX_DEFAULT_LOADFONT is not set # CONFIG_BUSYBOX_DEFAULT_SETFONT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SETFONT_TEXTUAL_MAP is not set CONFIG_BUSYBOX_DEFAULT_DEFAULT_SETFONT_DIR="" # CONFIG_BUSYBOX_DEFAULT_FEATURE_LOADFONT_PSF2 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LOADFONT_RAW is not set # CONFIG_BUSYBOX_DEFAULT_LOADKMAP is not set # CONFIG_BUSYBOX_DEFAULT_OPENVT is not set CONFIG_BUSYBOX_DEFAULT_RESET=y # CONFIG_BUSYBOX_DEFAULT_RESIZE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_RESIZE_PRINT is not set # CONFIG_BUSYBOX_DEFAULT_SETCONSOLE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SETCONSOLE_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_SETKEYCODES is not set # CONFIG_BUSYBOX_DEFAULT_SETLOGCONS is not set # CONFIG_BUSYBOX_DEFAULT_SHOWKEY is not set # CONFIG_BUSYBOX_DEFAULT_PIPE_PROGRESS is not set # CONFIG_BUSYBOX_DEFAULT_RUN_PARTS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_RUN_PARTS_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_RUN_PARTS_FANCY is not set CONFIG_BUSYBOX_DEFAULT_START_STOP_DAEMON=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_START_STOP_DAEMON_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_START_STOP_DAEMON_FANCY is not set CONFIG_BUSYBOX_DEFAULT_WHICH=y # CONFIG_BUSYBOX_DEFAULT_MINIPS is not set # CONFIG_BUSYBOX_DEFAULT_NUKE is not set # CONFIG_BUSYBOX_DEFAULT_RESUME is not set # CONFIG_BUSYBOX_DEFAULT_RUN_INIT is not set CONFIG_BUSYBOX_DEFAULT_AWK=y CONFIG_BUSYBOX_DEFAULT_FEATURE_AWK_LIBM=y CONFIG_BUSYBOX_DEFAULT_FEATURE_AWK_GNU_EXTENSIONS=y CONFIG_BUSYBOX_DEFAULT_CMP=y # CONFIG_BUSYBOX_DEFAULT_DIFF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DIFF_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DIFF_DIR is not set # CONFIG_BUSYBOX_DEFAULT_ED is not set # CONFIG_BUSYBOX_DEFAULT_PATCH is not set CONFIG_BUSYBOX_DEFAULT_SED=y CONFIG_BUSYBOX_DEFAULT_VI=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_MAX_LEN=1024 # CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_8BIT is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_COLON=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_YANKMARK=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_SEARCH=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_REGEX_SEARCH is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_USE_SIGNALS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_DOT_CMD=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_READONLY=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_SETOPTS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_SET=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_WIN_RESIZE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_ASK_TERMINAL=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_UNDO is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_UNDO_QUEUE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_VI_UNDO_QUEUE_MAX=0 CONFIG_BUSYBOX_DEFAULT_FEATURE_ALLOW_EXEC=y CONFIG_BUSYBOX_DEFAULT_FIND=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_PRINT0=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_MTIME=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_MMIN is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_PERM=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_TYPE=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_EXECUTABLE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_XDEV=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_MAXDEPTH=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_NEWER=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_INUM is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_EXEC=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_EXEC_PLUS is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_USER=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_GROUP=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_NOT=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_DEPTH=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_PAREN=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_SIZE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_PRUNE=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_QUIT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_DELETE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_PATH=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_REGEX=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_CONTEXT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FIND_LINKS is not set CONFIG_BUSYBOX_DEFAULT_GREP=y CONFIG_BUSYBOX_DEFAULT_EGREP=y CONFIG_BUSYBOX_DEFAULT_FGREP=y CONFIG_BUSYBOX_DEFAULT_FEATURE_GREP_CONTEXT=y CONFIG_BUSYBOX_DEFAULT_XARGS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_CONFIRMATION=y CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_QUOTES=y CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_TERMOPT=y CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_ZERO_TERM=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_REPL_STR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_PARALLEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_XARGS_SUPPORT_ARGS_FILE is not set # CONFIG_BUSYBOX_DEFAULT_BOOTCHARTD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_BOOTCHARTD_BLOATED_HEADER is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_BOOTCHARTD_CONFIG_FILE is not set CONFIG_BUSYBOX_DEFAULT_HALT=y CONFIG_BUSYBOX_DEFAULT_POWEROFF=y CONFIG_BUSYBOX_DEFAULT_REBOOT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_WAIT_FOR_INIT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CALL_TELINIT is not set CONFIG_BUSYBOX_DEFAULT_TELINIT_PATH="" # CONFIG_BUSYBOX_DEFAULT_INIT is not set # CONFIG_BUSYBOX_DEFAULT_LINUXRC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_USE_INITTAB is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_KILL_REMOVED is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_KILL_DELAY=0 # CONFIG_BUSYBOX_DEFAULT_FEATURE_INIT_SCTTY is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INIT_SYSLOG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INIT_QUIET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INIT_COREDUMPS is not set CONFIG_BUSYBOX_DEFAULT_INIT_TERMINAL_TYPE="" # CONFIG_BUSYBOX_DEFAULT_FEATURE_INIT_MODIFY_CMDLINE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SHADOWPASSWDS=y # CONFIG_BUSYBOX_DEFAULT_USE_BB_PWD_GRP is not set # CONFIG_BUSYBOX_DEFAULT_USE_BB_SHADOW is not set # CONFIG_BUSYBOX_DEFAULT_USE_BB_CRYPT is not set # CONFIG_BUSYBOX_DEFAULT_USE_BB_CRYPT_SHA is not set # CONFIG_BUSYBOX_DEFAULT_ADD_SHELL is not set # CONFIG_BUSYBOX_DEFAULT_REMOVE_SHELL is not set # CONFIG_BUSYBOX_DEFAULT_ADDGROUP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_ADDUSER_TO_GROUP is not set # CONFIG_BUSYBOX_DEFAULT_ADDUSER is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHECK_NAMES is not set CONFIG_BUSYBOX_DEFAULT_LAST_ID=0 CONFIG_BUSYBOX_DEFAULT_FIRST_SYSTEM_ID=0 CONFIG_BUSYBOX_DEFAULT_LAST_SYSTEM_ID=0 # CONFIG_BUSYBOX_DEFAULT_CHPASSWD is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_DEFAULT_PASSWD_ALGO="md5" # CONFIG_BUSYBOX_DEFAULT_CRYPTPW is not set # CONFIG_BUSYBOX_DEFAULT_MKPASSWD is not set # CONFIG_BUSYBOX_DEFAULT_DELUSER is not set # CONFIG_BUSYBOX_DEFAULT_DELGROUP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DEL_USER_FROM_GROUP is not set # CONFIG_BUSYBOX_DEFAULT_GETTY is not set CONFIG_BUSYBOX_DEFAULT_LOGIN=y CONFIG_BUSYBOX_DEFAULT_LOGIN_SESSION_AS_CHILD=y # CONFIG_BUSYBOX_DEFAULT_LOGIN_SCRIPTS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_NOLOGIN is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SECURETTY is not set CONFIG_BUSYBOX_DEFAULT_PASSWD=y CONFIG_BUSYBOX_DEFAULT_FEATURE_PASSWD_WEAK_CHECK=y # CONFIG_BUSYBOX_DEFAULT_SU is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SU_SYSLOG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SU_CHECKS_SHELLS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SU_BLANK_PW_NEEDS_SECURE_TTY is not set # CONFIG_BUSYBOX_DEFAULT_SULOGIN is not set # CONFIG_BUSYBOX_DEFAULT_VLOCK is not set # CONFIG_BUSYBOX_DEFAULT_CHATTR is not set # CONFIG_BUSYBOX_DEFAULT_FSCK is not set # CONFIG_BUSYBOX_DEFAULT_LSATTR is not set # CONFIG_BUSYBOX_DEFAULT_TUNE2FS is not set # CONFIG_BUSYBOX_DEFAULT_MODPROBE_SMALL is not set # CONFIG_BUSYBOX_DEFAULT_DEPMOD is not set # CONFIG_BUSYBOX_DEFAULT_INSMOD is not set # CONFIG_BUSYBOX_DEFAULT_LSMOD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LSMOD_PRETTY_2_6_OUTPUT is not set # CONFIG_BUSYBOX_DEFAULT_MODINFO is not set # CONFIG_BUSYBOX_DEFAULT_MODPROBE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MODPROBE_BLACKLIST is not set # CONFIG_BUSYBOX_DEFAULT_RMMOD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CMDLINE_MODULE_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MODPROBE_SMALL_CHECK_ALREADY_LOADED is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_2_4_MODULES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSMOD_VERSION_CHECKING is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSMOD_KSYMOOPS_SYMBOLS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSMOD_LOADINKMEM is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSMOD_LOAD_MAP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSMOD_LOAD_MAP_FULL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHECK_TAINTED_MODULE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INSMOD_TRY_MMAP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MODUTILS_ALIAS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MODUTILS_SYMBOLS is not set CONFIG_BUSYBOX_DEFAULT_DEFAULT_MODULES_DIR="" CONFIG_BUSYBOX_DEFAULT_DEFAULT_DEPMOD_FILE="" # CONFIG_BUSYBOX_DEFAULT_ACPID is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_ACPID_COMPAT is not set # CONFIG_BUSYBOX_DEFAULT_BLKDISCARD is not set # CONFIG_BUSYBOX_DEFAULT_BLKID is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_BLKID_TYPE is not set # CONFIG_BUSYBOX_DEFAULT_BLOCKDEV is not set # CONFIG_BUSYBOX_DEFAULT_CAL is not set # CONFIG_BUSYBOX_DEFAULT_CHRT is not set CONFIG_BUSYBOX_DEFAULT_DMESG=y CONFIG_BUSYBOX_DEFAULT_FEATURE_DMESG_PRETTY=y # CONFIG_BUSYBOX_DEFAULT_EJECT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_EJECT_SCSI is not set # CONFIG_BUSYBOX_DEFAULT_FALLOCATE is not set # CONFIG_BUSYBOX_DEFAULT_FATATTR is not set # CONFIG_BUSYBOX_DEFAULT_FBSET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FBSET_FANCY is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FBSET_READMODE is not set # CONFIG_BUSYBOX_DEFAULT_FDFORMAT is not set # CONFIG_BUSYBOX_DEFAULT_FDISK is not set # CONFIG_BUSYBOX_DEFAULT_FDISK_SUPPORT_LARGE_DISKS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FDISK_WRITABLE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_AIX_LABEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SGI_LABEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SUN_LABEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_OSF_LABEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_GPT_LABEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FDISK_ADVANCED is not set # CONFIG_BUSYBOX_DEFAULT_FINDFS is not set CONFIG_BUSYBOX_DEFAULT_FLOCK=y # CONFIG_BUSYBOX_DEFAULT_FDFLUSH is not set # CONFIG_BUSYBOX_DEFAULT_FREERAMDISK is not set # CONFIG_BUSYBOX_DEFAULT_FSCK_MINIX is not set # CONFIG_BUSYBOX_DEFAULT_FSFREEZE is not set # CONFIG_BUSYBOX_DEFAULT_FSTRIM is not set # CONFIG_BUSYBOX_DEFAULT_GETOPT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_GETOPT_LONG is not set CONFIG_BUSYBOX_DEFAULT_HEXDUMP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_HEXDUMP_REVERSE is not set # CONFIG_BUSYBOX_DEFAULT_HD is not set # CONFIG_BUSYBOX_DEFAULT_XXD is not set CONFIG_BUSYBOX_DEFAULT_HWCLOCK=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_HWCLOCK_ADJTIME_FHS is not set # CONFIG_BUSYBOX_DEFAULT_IONICE is not set # CONFIG_BUSYBOX_DEFAULT_IPCRM is not set # CONFIG_BUSYBOX_DEFAULT_IPCS is not set # CONFIG_BUSYBOX_DEFAULT_LAST is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LAST_FANCY is not set # CONFIG_BUSYBOX_DEFAULT_LOSETUP is not set # CONFIG_BUSYBOX_DEFAULT_LSPCI is not set # CONFIG_BUSYBOX_DEFAULT_LSUSB is not set # CONFIG_BUSYBOX_DEFAULT_MDEV is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MDEV_CONF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MDEV_RENAME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MDEV_RENAME_REGEXP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MDEV_EXEC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MDEV_LOAD_FIRMWARE is not set # CONFIG_BUSYBOX_DEFAULT_MESG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MESG_ENABLE_ONLY_GROUP is not set # CONFIG_BUSYBOX_DEFAULT_MKE2FS is not set # CONFIG_BUSYBOX_DEFAULT_MKFS_EXT2 is not set # CONFIG_BUSYBOX_DEFAULT_MKFS_MINIX is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MINIX2 is not set # CONFIG_BUSYBOX_DEFAULT_MKFS_REISER is not set # CONFIG_BUSYBOX_DEFAULT_MKDOSFS is not set # CONFIG_BUSYBOX_DEFAULT_MKFS_VFAT is not set CONFIG_BUSYBOX_DEFAULT_MKSWAP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_MKSWAP_UUID is not set # CONFIG_BUSYBOX_DEFAULT_MORE is not set CONFIG_BUSYBOX_DEFAULT_MOUNT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_FAKE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_VERBOSE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_HELPERS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_LABEL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_NFS is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_CIFS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_FLAGS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_FSTAB=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_OTHERTAB is not set # CONFIG_BUSYBOX_DEFAULT_MOUNTPOINT is not set # CONFIG_BUSYBOX_DEFAULT_NOLOGIN is not set # CONFIG_BUSYBOX_DEFAULT_NOLOGIN_DEPENDENCIES is not set # CONFIG_BUSYBOX_DEFAULT_NSENTER is not set CONFIG_BUSYBOX_DEFAULT_PIVOT_ROOT=y # CONFIG_BUSYBOX_DEFAULT_RDATE is not set # CONFIG_BUSYBOX_DEFAULT_RDEV is not set # CONFIG_BUSYBOX_DEFAULT_READPROFILE is not set # CONFIG_BUSYBOX_DEFAULT_RENICE is not set # CONFIG_BUSYBOX_DEFAULT_REV is not set # CONFIG_BUSYBOX_DEFAULT_RTCWAKE is not set # CONFIG_BUSYBOX_DEFAULT_SCRIPT is not set # CONFIG_BUSYBOX_DEFAULT_SCRIPTREPLAY is not set # CONFIG_BUSYBOX_DEFAULT_SETARCH is not set # CONFIG_BUSYBOX_DEFAULT_LINUX32 is not set # CONFIG_BUSYBOX_DEFAULT_LINUX64 is not set # CONFIG_BUSYBOX_DEFAULT_SETPRIV is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SETPRIV_DUMP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SETPRIV_CAPABILITIES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SETPRIV_CAPABILITY_NAMES is not set # CONFIG_BUSYBOX_DEFAULT_SETSID is not set # CONFIG_BUSYBOX_DEFAULT_SWAPON is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SWAPON_DISCARD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SWAPON_PRI is not set # CONFIG_BUSYBOX_DEFAULT_SWAPOFF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SWAPONOFF_LABEL is not set CONFIG_BUSYBOX_DEFAULT_SWITCH_ROOT=y # CONFIG_BUSYBOX_DEFAULT_TASKSET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TASKSET_FANCY is not set # CONFIG_BUSYBOX_DEFAULT_UEVENT is not set CONFIG_BUSYBOX_DEFAULT_UMOUNT=y CONFIG_BUSYBOX_DEFAULT_FEATURE_UMOUNT_ALL=y # CONFIG_BUSYBOX_DEFAULT_UNSHARE is not set # CONFIG_BUSYBOX_DEFAULT_WALL is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_LOOP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_MOUNT_LOOP_CREATE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MTAB_SUPPORT is not set # CONFIG_BUSYBOX_DEFAULT_VOLUMEID is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_BCACHE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_BTRFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_CRAMFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_EXFAT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_EXT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_F2FS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_FAT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_HFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_ISO9660 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_JFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_LFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_LINUXRAID is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_LINUXSWAP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_LUKS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_MINIX is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_NILFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_NTFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_OCFS2 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_REISERFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_ROMFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_SQUASHFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_SYSV is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_UBIFS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_UDF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_VOLUMEID_XFS is not set # CONFIG_BUSYBOX_DEFAULT_ADJTIMEX is not set # CONFIG_BUSYBOX_DEFAULT_BBCONFIG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_COMPRESS_BBCONFIG is not set # CONFIG_BUSYBOX_DEFAULT_BC is not set # CONFIG_BUSYBOX_DEFAULT_DC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DC_BIG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DC_LIBM is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_BC_INTERACTIVE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_BC_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_BEEP is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_BEEP_FREQ=0 CONFIG_BUSYBOX_DEFAULT_FEATURE_BEEP_LENGTH_MS=0 # CONFIG_BUSYBOX_DEFAULT_CHAT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_NOFAIL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_TTY_HIFI is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_IMPLICIT_CR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_SWALLOW_OPTS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_SEND_ESCAPES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_VAR_ABORT_LEN is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CHAT_CLR_ABORT is not set # CONFIG_BUSYBOX_DEFAULT_CONSPY is not set CONFIG_BUSYBOX_DEFAULT_CROND=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_CROND_D is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CROND_CALL_SENDMAIL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_CROND_SPECIAL_TIMES is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_CROND_DIR="/etc" CONFIG_BUSYBOX_DEFAULT_CRONTAB=y # CONFIG_BUSYBOX_DEFAULT_DEVFSD is not set # CONFIG_BUSYBOX_DEFAULT_DEVFSD_MODLOAD is not set # CONFIG_BUSYBOX_DEFAULT_DEVFSD_FG_NP is not set # CONFIG_BUSYBOX_DEFAULT_DEVFSD_VERBOSE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_DEVFS is not set # CONFIG_BUSYBOX_DEFAULT_DEVMEM is not set # CONFIG_BUSYBOX_DEFAULT_FBSPLASH is not set # CONFIG_BUSYBOX_DEFAULT_FLASH_ERASEALL is not set # CONFIG_BUSYBOX_DEFAULT_FLASH_LOCK is not set # CONFIG_BUSYBOX_DEFAULT_FLASH_UNLOCK is not set # CONFIG_BUSYBOX_DEFAULT_FLASHCP is not set # CONFIG_BUSYBOX_DEFAULT_HDPARM is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HDPARM_GET_IDENTITY is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HDPARM_HDIO_SCAN_HWIF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HDPARM_HDIO_UNREGISTER_HWIF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HDPARM_HDIO_DRIVE_RESET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HDPARM_HDIO_TRISTATE_HWIF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HDPARM_HDIO_GETSET_DMA is not set # CONFIG_BUSYBOX_DEFAULT_HEXEDIT is not set # CONFIG_BUSYBOX_DEFAULT_I2CGET is not set # CONFIG_BUSYBOX_DEFAULT_I2CSET is not set # CONFIG_BUSYBOX_DEFAULT_I2CDUMP is not set # CONFIG_BUSYBOX_DEFAULT_I2CDETECT is not set # CONFIG_BUSYBOX_DEFAULT_INOTIFYD is not set CONFIG_BUSYBOX_DEFAULT_LESS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_MAXLINES=9999999 # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_BRACKETS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_FLAGS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_TRUNCATE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_MARKS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_REGEXP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_WINCH is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_ASK_TERMINAL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_DASHCMD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_LINENUMS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_RAW is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LESS_ENV is not set CONFIG_BUSYBOX_DEFAULT_LOCK=y # CONFIG_BUSYBOX_DEFAULT_LSSCSI is not set # CONFIG_BUSYBOX_DEFAULT_MAKEDEVS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MAKEDEVS_LEAF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_MAKEDEVS_TABLE is not set # CONFIG_BUSYBOX_DEFAULT_MAN is not set # CONFIG_BUSYBOX_DEFAULT_MICROCOM is not set # CONFIG_BUSYBOX_DEFAULT_MT is not set # CONFIG_BUSYBOX_DEFAULT_NANDWRITE is not set # CONFIG_BUSYBOX_DEFAULT_NANDDUMP is not set # CONFIG_BUSYBOX_DEFAULT_PARTPROBE is not set # CONFIG_BUSYBOX_DEFAULT_RAIDAUTORUN is not set # CONFIG_BUSYBOX_DEFAULT_READAHEAD is not set # CONFIG_BUSYBOX_DEFAULT_RFKILL is not set # CONFIG_BUSYBOX_DEFAULT_RUNLEVEL is not set # CONFIG_BUSYBOX_DEFAULT_RX is not set # CONFIG_BUSYBOX_DEFAULT_SETFATTR is not set # CONFIG_BUSYBOX_DEFAULT_SETSERIAL is not set CONFIG_BUSYBOX_DEFAULT_STRINGS=y CONFIG_BUSYBOX_DEFAULT_TIME=y # CONFIG_BUSYBOX_DEFAULT_TTYSIZE is not set # CONFIG_BUSYBOX_DEFAULT_UBIATTACH is not set # CONFIG_BUSYBOX_DEFAULT_UBIDETACH is not set # CONFIG_BUSYBOX_DEFAULT_UBIMKVOL is not set # CONFIG_BUSYBOX_DEFAULT_UBIRMVOL is not set # CONFIG_BUSYBOX_DEFAULT_UBIRSVOL is not set # CONFIG_BUSYBOX_DEFAULT_UBIUPDATEVOL is not set # CONFIG_BUSYBOX_DEFAULT_UBIRENAME is not set # CONFIG_BUSYBOX_DEFAULT_VOLNAME is not set # CONFIG_BUSYBOX_DEFAULT_WATCHDOG is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_IPV6=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_UNIX_LOCAL is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_PREFER_IPV4_ADDRESS=y CONFIG_BUSYBOX_DEFAULT_VERBOSE_RESOLUTION_ERRORS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TLS_SHA1 is not set # CONFIG_BUSYBOX_DEFAULT_ARP is not set # CONFIG_BUSYBOX_DEFAULT_ARPING is not set CONFIG_BUSYBOX_DEFAULT_BRCTL=y CONFIG_BUSYBOX_DEFAULT_FEATURE_BRCTL_FANCY=y CONFIG_BUSYBOX_DEFAULT_FEATURE_BRCTL_SHOW=y # CONFIG_BUSYBOX_DEFAULT_DNSD is not set # CONFIG_BUSYBOX_DEFAULT_ETHER_WAKE is not set # CONFIG_BUSYBOX_DEFAULT_FTPD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FTPD_WRITE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FTPD_ACCEPT_BROKEN_LIST is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FTPD_AUTHENTICATION is not set # CONFIG_BUSYBOX_DEFAULT_FTPGET is not set # CONFIG_BUSYBOX_DEFAULT_FTPPUT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_FTPGETPUT_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_HOSTNAME is not set # CONFIG_BUSYBOX_DEFAULT_DNSDOMAINNAME is not set # CONFIG_BUSYBOX_DEFAULT_HTTPD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_RANGES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_SETUID is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_BASIC_AUTH is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_AUTH_MD5 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_CGI is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_CONFIG_WITH_SCRIPT_INTERPR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_SET_REMOTE_PORT_TO_ENV is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_ENCODE_URL_STR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_ERROR_PAGES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_PROXY is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_HTTPD_GZIP is not set CONFIG_BUSYBOX_DEFAULT_IFCONFIG=y CONFIG_BUSYBOX_DEFAULT_FEATURE_IFCONFIG_STATUS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFCONFIG_SLIP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFCONFIG_MEMSTART_IOADDR_IRQ is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_IFCONFIG_HW=y CONFIG_BUSYBOX_DEFAULT_FEATURE_IFCONFIG_BROADCAST_PLUS=y # CONFIG_BUSYBOX_DEFAULT_IFENSLAVE is not set # CONFIG_BUSYBOX_DEFAULT_IFPLUGD is not set # CONFIG_BUSYBOX_DEFAULT_IFUP is not set # CONFIG_BUSYBOX_DEFAULT_IFDOWN is not set CONFIG_BUSYBOX_DEFAULT_IFUPDOWN_IFSTATE_PATH="" # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFUPDOWN_IP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFUPDOWN_IPV4 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFUPDOWN_IPV6 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFUPDOWN_MAPPING is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IFUPDOWN_EXTERNAL_DHCP is not set # CONFIG_BUSYBOX_DEFAULT_INETD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INETD_SUPPORT_BUILTIN_ECHO is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INETD_SUPPORT_BUILTIN_DISCARD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INETD_SUPPORT_BUILTIN_TIME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INETD_SUPPORT_BUILTIN_DAYTIME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INETD_SUPPORT_BUILTIN_CHARGEN is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_INETD_RPC is not set CONFIG_BUSYBOX_DEFAULT_IP=y # CONFIG_BUSYBOX_DEFAULT_IPADDR is not set # CONFIG_BUSYBOX_DEFAULT_IPLINK is not set # CONFIG_BUSYBOX_DEFAULT_IPROUTE is not set # CONFIG_BUSYBOX_DEFAULT_IPTUNNEL is not set # CONFIG_BUSYBOX_DEFAULT_IPRULE is not set # CONFIG_BUSYBOX_DEFAULT_IPNEIGH is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_ADDRESS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_LINK=y CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_ROUTE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_ROUTE_DIR="/etc/iproute2" # CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_TUNNEL is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_RULE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_NEIGH=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_IP_RARE_PROTOCOLS is not set # CONFIG_BUSYBOX_DEFAULT_IPCALC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IPCALC_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_IPCALC_FANCY is not set # CONFIG_BUSYBOX_DEFAULT_FAKEIDENTD is not set # CONFIG_BUSYBOX_DEFAULT_NAMEIF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_NAMEIF_EXTENDED is not set # CONFIG_BUSYBOX_DEFAULT_NBDCLIENT is not set CONFIG_BUSYBOX_DEFAULT_NC=y # CONFIG_BUSYBOX_DEFAULT_NETCAT is not set # CONFIG_BUSYBOX_DEFAULT_NC_SERVER is not set # CONFIG_BUSYBOX_DEFAULT_NC_EXTRA is not set # CONFIG_BUSYBOX_DEFAULT_NC_110_COMPAT is not set CONFIG_BUSYBOX_DEFAULT_NETMSG=y CONFIG_BUSYBOX_DEFAULT_NETSTAT=y CONFIG_BUSYBOX_DEFAULT_FEATURE_NETSTAT_WIDE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_NETSTAT_PRG=y # CONFIG_BUSYBOX_DEFAULT_NSLOOKUP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_NSLOOKUP_BIG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_NSLOOKUP_LONG_OPTIONS is not set CONFIG_BUSYBOX_DEFAULT_NSLOOKUP_OPENWRT=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_NSLOOKUP_OPENWRT_LONG_OPTIONS is not set CONFIG_BUSYBOX_DEFAULT_NTPD=y CONFIG_BUSYBOX_DEFAULT_FEATURE_NTPD_SERVER=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_NTPD_CONF is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_NTP_AUTH is not set CONFIG_BUSYBOX_DEFAULT_PING=y CONFIG_BUSYBOX_DEFAULT_PING6=y CONFIG_BUSYBOX_DEFAULT_FEATURE_FANCY_PING=y # CONFIG_BUSYBOX_DEFAULT_PSCAN is not set CONFIG_BUSYBOX_DEFAULT_ROUTE=y # CONFIG_BUSYBOX_DEFAULT_SLATTACH is not set # CONFIG_BUSYBOX_DEFAULT_SSL_CLIENT is not set # CONFIG_BUSYBOX_DEFAULT_TC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TC_INGRESS is not set # CONFIG_BUSYBOX_DEFAULT_TCPSVD is not set # CONFIG_BUSYBOX_DEFAULT_UDPSVD is not set # CONFIG_BUSYBOX_DEFAULT_TELNET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TELNET_TTYPE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TELNET_AUTOLOGIN is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TELNET_WIDTH is not set # CONFIG_BUSYBOX_DEFAULT_TELNETD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TELNETD_STANDALONE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TELNETD_INETD_WAIT is not set # CONFIG_BUSYBOX_DEFAULT_TFTP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TFTP_PROGRESS_BAR is not set # CONFIG_BUSYBOX_DEFAULT_TFTPD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TFTP_GET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TFTP_PUT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TFTP_BLOCKSIZE is not set # CONFIG_BUSYBOX_DEFAULT_TFTP_DEBUG is not set # CONFIG_BUSYBOX_DEFAULT_TLS is not set CONFIG_BUSYBOX_DEFAULT_TRACEROUTE=y CONFIG_BUSYBOX_DEFAULT_TRACEROUTE6=y CONFIG_BUSYBOX_DEFAULT_FEATURE_TRACEROUTE_VERBOSE=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TRACEROUTE_USE_ICMP is not set # CONFIG_BUSYBOX_DEFAULT_TUNCTL is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TUNCTL_UG is not set # CONFIG_BUSYBOX_DEFAULT_VCONFIG is not set # CONFIG_BUSYBOX_DEFAULT_WGET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WGET_LONG_OPTIONS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WGET_STATUSBAR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WGET_AUTHENTICATION is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WGET_TIMEOUT is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WGET_HTTPS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_WGET_OPENSSL is not set # CONFIG_BUSYBOX_DEFAULT_WHOIS is not set # CONFIG_BUSYBOX_DEFAULT_ZCIP is not set # CONFIG_BUSYBOX_DEFAULT_UDHCPD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPD_BASE_IP_ON_MAC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPD_WRITE_LEASES_EARLY is not set CONFIG_BUSYBOX_DEFAULT_DHCPD_LEASES_FILE="" # CONFIG_BUSYBOX_DEFAULT_DUMPLEASES is not set # CONFIG_BUSYBOX_DEFAULT_DHCPRELAY is not set CONFIG_BUSYBOX_DEFAULT_UDHCPC=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPC_ARPING is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPC_SANITIZEOPT is not set CONFIG_BUSYBOX_DEFAULT_UDHCPC_DEFAULT_SCRIPT="/usr/share/udhcpc/default.script" # CONFIG_BUSYBOX_DEFAULT_UDHCPC6 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPC6_RFC3646 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPC6_RFC4704 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPC6_RFC4833 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCPC6_RFC5970 is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCP_PORT is not set CONFIG_BUSYBOX_DEFAULT_UDHCP_DEBUG=0 CONFIG_BUSYBOX_DEFAULT_UDHCPC_SLACK_FOR_BUGGY_SERVERS=80 CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCP_RFC3397=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_UDHCP_8021Q is not set CONFIG_BUSYBOX_DEFAULT_IFUPDOWN_UDHCPC_CMD_OPTIONS="" # CONFIG_BUSYBOX_DEFAULT_LPD is not set # CONFIG_BUSYBOX_DEFAULT_LPR is not set # CONFIG_BUSYBOX_DEFAULT_LPQ is not set # CONFIG_BUSYBOX_DEFAULT_MAKEMIME is not set # CONFIG_BUSYBOX_DEFAULT_POPMAILDIR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_POPMAILDIR_DELIVERY is not set # CONFIG_BUSYBOX_DEFAULT_REFORMIME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_REFORMIME_COMPAT is not set # CONFIG_BUSYBOX_DEFAULT_SENDMAIL is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_MIME_CHARSET="" CONFIG_BUSYBOX_DEFAULT_FREE=y # CONFIG_BUSYBOX_DEFAULT_FUSER is not set # CONFIG_BUSYBOX_DEFAULT_IOSTAT is not set CONFIG_BUSYBOX_DEFAULT_KILL=y CONFIG_BUSYBOX_DEFAULT_KILLALL=y # CONFIG_BUSYBOX_DEFAULT_KILLALL5 is not set # CONFIG_BUSYBOX_DEFAULT_LSOF is not set # CONFIG_BUSYBOX_DEFAULT_MPSTAT is not set # CONFIG_BUSYBOX_DEFAULT_NMETER is not set CONFIG_BUSYBOX_DEFAULT_PGREP=y # CONFIG_BUSYBOX_DEFAULT_PKILL is not set CONFIG_BUSYBOX_DEFAULT_PIDOF=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_PIDOF_SINGLE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_PIDOF_OMIT is not set # CONFIG_BUSYBOX_DEFAULT_PMAP is not set # CONFIG_BUSYBOX_DEFAULT_POWERTOP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_POWERTOP_INTERACTIVE is not set CONFIG_BUSYBOX_DEFAULT_PS=y CONFIG_BUSYBOX_DEFAULT_FEATURE_PS_WIDE=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_PS_LONG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_PS_TIME is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_PS_UNUSUAL_SYSTEMS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_PS_ADDITIONAL_COLUMNS is not set # CONFIG_BUSYBOX_DEFAULT_PSTREE is not set # CONFIG_BUSYBOX_DEFAULT_PWDX is not set # CONFIG_BUSYBOX_DEFAULT_SMEMCAP is not set CONFIG_BUSYBOX_DEFAULT_BB_SYSCTL=y CONFIG_BUSYBOX_DEFAULT_TOP=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TOP_INTERACTIVE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_TOP_CPU_USAGE_PERCENTAGE=y CONFIG_BUSYBOX_DEFAULT_FEATURE_TOP_CPU_GLOBAL_PERCENTS=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_TOP_SMP_CPU is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TOP_DECIMALS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TOP_SMP_PROCESS is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_TOPMEM is not set CONFIG_BUSYBOX_DEFAULT_UPTIME=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_UPTIME_UTMP_SUPPORT is not set # CONFIG_BUSYBOX_DEFAULT_WATCH is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SHOW_THREADS is not set # CONFIG_BUSYBOX_DEFAULT_CHPST is not set # CONFIG_BUSYBOX_DEFAULT_SETUIDGID is not set # CONFIG_BUSYBOX_DEFAULT_ENVUIDGID is not set # CONFIG_BUSYBOX_DEFAULT_ENVDIR is not set # CONFIG_BUSYBOX_DEFAULT_SOFTLIMIT is not set # CONFIG_BUSYBOX_DEFAULT_RUNSV is not set # CONFIG_BUSYBOX_DEFAULT_RUNSVDIR is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_RUNSVDIR_LOG is not set # CONFIG_BUSYBOX_DEFAULT_SV is not set CONFIG_BUSYBOX_DEFAULT_SV_DEFAULT_SERVICE_DIR="" # CONFIG_BUSYBOX_DEFAULT_SVC is not set # CONFIG_BUSYBOX_DEFAULT_SVOK is not set # CONFIG_BUSYBOX_DEFAULT_SVLOGD is not set # CONFIG_BUSYBOX_DEFAULT_CHCON is not set # CONFIG_BUSYBOX_DEFAULT_GETENFORCE is not set # CONFIG_BUSYBOX_DEFAULT_GETSEBOOL is not set # CONFIG_BUSYBOX_DEFAULT_LOAD_POLICY is not set # CONFIG_BUSYBOX_DEFAULT_MATCHPATHCON is not set # CONFIG_BUSYBOX_DEFAULT_RUNCON is not set # CONFIG_BUSYBOX_DEFAULT_SELINUXENABLED is not set # CONFIG_BUSYBOX_DEFAULT_SESTATUS is not set # CONFIG_BUSYBOX_DEFAULT_SETENFORCE is not set # CONFIG_BUSYBOX_DEFAULT_SETFILES is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SETFILES_CHECK_OPTION is not set # CONFIG_BUSYBOX_DEFAULT_RESTORECON is not set # CONFIG_BUSYBOX_DEFAULT_SETSEBOOL is not set CONFIG_BUSYBOX_DEFAULT_SH_IS_ASH=y # CONFIG_BUSYBOX_DEFAULT_SH_IS_HUSH is not set # CONFIG_BUSYBOX_DEFAULT_SH_IS_NONE is not set # CONFIG_BUSYBOX_DEFAULT_BASH_IS_ASH is not set # CONFIG_BUSYBOX_DEFAULT_BASH_IS_HUSH is not set CONFIG_BUSYBOX_DEFAULT_BASH_IS_NONE=y CONFIG_BUSYBOX_DEFAULT_ASH=y # CONFIG_BUSYBOX_DEFAULT_ASH_OPTIMIZE_FOR_SIZE is not set CONFIG_BUSYBOX_DEFAULT_ASH_INTERNAL_GLOB=y CONFIG_BUSYBOX_DEFAULT_ASH_BASH_COMPAT=y # CONFIG_BUSYBOX_DEFAULT_ASH_BASH_SOURCE_CURDIR is not set # CONFIG_BUSYBOX_DEFAULT_ASH_BASH_NOT_FOUND_HOOK is not set CONFIG_BUSYBOX_DEFAULT_ASH_JOB_CONTROL=y CONFIG_BUSYBOX_DEFAULT_ASH_ALIAS=y # CONFIG_BUSYBOX_DEFAULT_ASH_RANDOM_SUPPORT is not set CONFIG_BUSYBOX_DEFAULT_ASH_EXPAND_PRMT=y # CONFIG_BUSYBOX_DEFAULT_ASH_IDLE_TIMEOUT is not set # CONFIG_BUSYBOX_DEFAULT_ASH_MAIL is not set CONFIG_BUSYBOX_DEFAULT_ASH_ECHO=y CONFIG_BUSYBOX_DEFAULT_ASH_PRINTF=y CONFIG_BUSYBOX_DEFAULT_ASH_TEST=y # CONFIG_BUSYBOX_DEFAULT_ASH_HELP is not set CONFIG_BUSYBOX_DEFAULT_ASH_GETOPTS=y CONFIG_BUSYBOX_DEFAULT_ASH_CMDCMD=y # CONFIG_BUSYBOX_DEFAULT_CTTYHACK is not set # CONFIG_BUSYBOX_DEFAULT_HUSH is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_BASH_COMPAT is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_BRACE_EXPANSION is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_LINENO_VAR is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_BASH_SOURCE_CURDIR is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_INTERACTIVE is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_SAVEHISTORY is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_JOB is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_TICK is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_IF is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_LOOPS is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_CASE is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_FUNCTIONS is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_LOCAL is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_RANDOM_SUPPORT is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_MODE_X is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_ECHO is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_PRINTF is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_TEST is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_HELP is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_EXPORT is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_EXPORT_N is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_READONLY is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_KILL is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_WAIT is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_COMMAND is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_TRAP is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_TYPE is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_TIMES is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_READ is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_SET is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_UNSET is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_ULIMIT is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_UMASK is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_GETOPTS is not set # CONFIG_BUSYBOX_DEFAULT_HUSH_MEMLEAK is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_MATH=y CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_MATH_64=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_EXTRA_QUIET is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_STANDALONE is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_NOFORK=y # CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_READ_FRAC is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_HISTFILESIZE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SH_EMBEDDED_SCRIPTS is not set # CONFIG_BUSYBOX_DEFAULT_KLOGD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_KLOGD_KLOGCTL is not set CONFIG_BUSYBOX_DEFAULT_LOGGER=y # CONFIG_BUSYBOX_DEFAULT_LOGREAD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_LOGREAD_REDUCED_LOCKING is not set # CONFIG_BUSYBOX_DEFAULT_SYSLOGD is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_ROTATE_LOGFILE is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_REMOTE_LOG is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SYSLOGD_DUP is not set # CONFIG_BUSYBOX_DEFAULT_FEATURE_SYSLOGD_CFG is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_SYSLOGD_READ_BUFFER_SIZE=0 # CONFIG_BUSYBOX_DEFAULT_FEATURE_IPC_SYSLOG is not set CONFIG_BUSYBOX_DEFAULT_FEATURE_IPC_SYSLOG_BUFFER_SIZE=0 # CONFIG_BUSYBOX_DEFAULT_FEATURE_KMSG_SYSLOG is not set # CONFIG_PACKAGE_ca-bundle is not set CONFIG_PACKAGE_ca-certificates=y # CONFIG_PACKAGE_dnsmasq is not set # CONFIG_PACKAGE_dnsmasq-dhcpv6 is not set CONFIG_PACKAGE_dnsmasq-full=y CONFIG_PACKAGE_dnsmasq_full_dhcp=y # CONFIG_PACKAGE_dnsmasq_full_dhcpv6 is not set # CONFIG_PACKAGE_dnsmasq_full_dnssec is not set # CONFIG_PACKAGE_dnsmasq_full_auth is not set CONFIG_PACKAGE_dnsmasq_full_ipset=y # CONFIG_PACKAGE_dnsmasq_full_conntrack is not set # CONFIG_PACKAGE_dnsmasq_full_noid is not set # CONFIG_PACKAGE_dnsmasq_full_broken_rtc is not set CONFIG_PACKAGE_dropbear=y  # # Configuration # CONFIG_DROPBEAR_CURVE25519=y # CONFIG_DROPBEAR_ECC is not set # CONFIG_DROPBEAR_ZLIB is not set CONFIG_DROPBEAR_DBCLIENT=y # CONFIG_PACKAGE_ead is not set CONFIG_PACKAGE_firewall=y CONFIG_PACKAGE_fstools=y # CONFIG_FSTOOLS_OVL_MOUNT_FULL_ACCESS_TIME is not set # CONFIG_FSTOOLS_OVL_MOUNT_COMPRESS_ZLIB is not set CONFIG_PACKAGE_fwtool=y CONFIG_PACKAGE_getrandom=y CONFIG_PACKAGE_jsonfilter=y # CONFIG_PACKAGE_libatomic is not set CONFIG_PACKAGE_libc=y CONFIG_PACKAGE_libgcc=y # CONFIG_PACKAGE_libgomp is not set CONFIG_PACKAGE_libpthread=y CONFIG_PACKAGE_librt=y # CONFIG_PACKAGE_libstdcpp is not set CONFIG_PACKAGE_logd=y CONFIG_PACKAGE_mtd=y CONFIG_PACKAGE_netifd=y # CONFIG_PACKAGE_nft-qos is not set # CONFIG_PACKAGE_nvram is not set # CONFIG_PACKAGE_om-watchdog is not set # CONFIG_PACKAGE_openwrt-keyring is not set CONFIG_PACKAGE_opkg=y CONFIG_PACKAGE_procd=y  # # Configuration # # CONFIG_PROCD_SHOW_BOOT is not set # CONFIG_PROCD_ZRAM_TMPFS is not set # CONFIG_PACKAGE_procd-ujail is not set # CONFIG_PACKAGE_qos-scripts is not set # CONFIG_PACKAGE_resolveip is not set CONFIG_PACKAGE_rpcd=y # CONFIG_PACKAGE_rpcd-mod-file is not set # CONFIG_PACKAGE_rpcd-mod-iwinfo is not set # CONFIG_PACKAGE_rpcd-mod-rpcsys is not set # CONFIG_PACKAGE_snapshot-tool is not set # CONFIG_PACKAGE_sqm-scripts is not set # CONFIG_PACKAGE_sqm-scripts-extra is not set CONFIG_PACKAGE_swconfig=y CONFIG_PACKAGE_ubox=y CONFIG_PACKAGE_ubus=y CONFIG_PACKAGE_ubusd=y # CONFIG_PACKAGE_ucert is not set # CONFIG_PACKAGE_ucert-full is not set CONFIG_PACKAGE_uci=y CONFIG_PACKAGE_urandom-seed=y CONFIG_PACKAGE_urngd=y # CONFIG_PACKAGE_usign is not set # CONFIG_PACKAGE_wireless-tools is not set # CONFIG_PACKAGE_zram-swap is not set  # # Administration #  # # openwisp # # CONFIG_PACKAGE_openwisp-config-cyassl is not set # CONFIG_PACKAGE_openwisp-config-mbedtls is not set # CONFIG_PACKAGE_openwisp-config-nossl is not set # CONFIG_PACKAGE_openwisp-config-openssl is not set  # # zabbix # # CONFIG_PACKAGE_zabbix-agentd is not set # CONFIG_PACKAGE_zabbix-extra-mac80211 is not set # CONFIG_PACKAGE_zabbix-extra-network is not set # CONFIG_PACKAGE_zabbix-extra-wifi is not set # CONFIG_PACKAGE_zabbix-get is not set # CONFIG_PACKAGE_zabbix-proxy is not set # CONFIG_PACKAGE_zabbix-sender is not set # CONFIG_PACKAGE_zabbix-server is not set # CONFIG_PACKAGE_htop is not set # CONFIG_PACKAGE_ipmitool is not set # CONFIG_PACKAGE_monit is not set # CONFIG_PACKAGE_monit-nossl is not set # CONFIG_PACKAGE_muninlite is not set # CONFIG_PACKAGE_netdata is not set # CONFIG_PACKAGE_sudo is not set # CONFIG_PACKAGE_syslog-ng is not set  # # Boot Loaders #  # # Development #  # # Libraries # # CONFIG_PACKAGE_libncurses-dev is not set # CONFIG_PACKAGE_libxml2-dev is not set # CONFIG_PACKAGE_zlib-dev is not set # CONFIG_PACKAGE_ar is not set # CONFIG_PACKAGE_autoconf is not set # CONFIG_PACKAGE_automake is not set # CONFIG_PACKAGE_binutils is not set # CONFIG_PACKAGE_diffutils is not set # CONFIG_PACKAGE_gcc is not set # CONFIG_PACKAGE_gdb is not set # CONFIG_PACKAGE_gdbserver is not set # CONFIG_PACKAGE_libtool-bin is not set # CONFIG_PACKAGE_lpc21isp is not set # CONFIG_PACKAGE_lttng-tools is not set # CONFIG_PACKAGE_m4 is not set # CONFIG_PACKAGE_make is not set # CONFIG_PACKAGE_meson is not set # CONFIG_PACKAGE_meson-src is not set # CONFIG_PACKAGE_ninja is not set # CONFIG_PACKAGE_objdump is not set # CONFIG_PACKAGE_patch is not set # CONFIG_PACKAGE_pkg-config is not set # CONFIG_PACKAGE_trace-cmd is not set # CONFIG_PACKAGE_trace-cmd-extra is not set # CONFIG_PACKAGE_valgrind is not set  # # Extra packages # # CONFIG_PACKAGE_automount is not set # CONFIG_PACKAGE_autosamba is not set # CONFIG_PACKAGE_ipv6helper is not set # CONFIG_PACKAGE_k3wifi is not set # CONFIG_PACKAGE_wireguard-tools is not set  # # Firmware #  # # ath10k IPQ4019 Boarddata # # CONFIG_PACKAGE_aircard-pcmcia-firmware is not set # CONFIG_PACKAGE_amdgpu-firmware is not set # CONFIG_PACKAGE_ar3k-firmware is not set # CONFIG_PACKAGE_ath10k-firmware-qca4019 is not set # CONFIG_PACKAGE_ath10k-firmware-qca4019-ct is not set # CONFIG_PACKAGE_ath10k-firmware-qca4019-ct-htt is not set # CONFIG_PACKAGE_ath10k-firmware-qca6174 is not set # CONFIG_PACKAGE_ath10k-firmware-qca9887 is not set # CONFIG_PACKAGE_ath10k-firmware-qca9887-ct is not set # CONFIG_PACKAGE_ath10k-firmware-qca9887-ct-htt is not set # CONFIG_PACKAGE_ath10k-firmware-qca9888 is not set # CONFIG_PACKAGE_ath10k-firmware-qca9888-ct is not set # CONFIG_PACKAGE_ath10k-firmware-qca9888-ct-htt is not set CONFIG_PACKAGE_ath10k-firmware-qca988x=y # CONFIG_PACKAGE_ath10k-firmware-qca988x-ct is not set # CONFIG_PACKAGE_ath10k-firmware-qca988x-ct-htt is not set # CONFIG_PACKAGE_ath10k-firmware-qca9984 is not set # CONFIG_PACKAGE_ath10k-firmware-qca9984-ct is not set # CONFIG_PACKAGE_ath10k-firmware-qca9984-ct-htt is not set # CONFIG_PACKAGE_ath10k-firmware-qca99x0 is not set # CONFIG_PACKAGE_ath10k-firmware-qca99x0-ct is not set # CONFIG_PACKAGE_ath10k-firmware-qca99x0-ct-htt is not set # CONFIG_PACKAGE_ath6k-firmware is not set # CONFIG_PACKAGE_ath9k-htc-firmware is not set # CONFIG_PACKAGE_b43legacy-firmware is not set # CONFIG_PACKAGE_bnx2-firmware is not set # CONFIG_PACKAGE_bnx2x-firmware is not set # CONFIG_PACKAGE_brcmfmac-firmware-4329-sdio is not set # CONFIG_PACKAGE_brcmfmac-firmware-43362-sdio is not set # CONFIG_PACKAGE_brcmfmac-firmware-43430-sdio is not set # CONFIG_PACKAGE_brcmfmac-firmware-43430-sdio-rpi-3b is not set # CONFIG_PACKAGE_brcmfmac-firmware-43430-sdio-rpi-zero-w is not set # CONFIG_PACKAGE_brcmfmac-firmware-43430a0-sdio is not set # CONFIG_PACKAGE_brcmfmac-firmware-43455-sdio is not set # CONFIG_PACKAGE_brcmfmac-firmware-43455-sdio-rpi-3b-plus is not set # CONFIG_PACKAGE_brcmfmac-firmware-43455-sdio-rpi-4b is not set # CONFIG_PACKAGE_brcmfmac-firmware-43602a1-pcie is not set # CONFIG_PACKAGE_brcmfmac-firmware-4366b1-pcie is not set # CONFIG_PACKAGE_brcmfmac-firmware-4366c0-pcie is not set # CONFIG_PACKAGE_brcmfmac-firmware-usb is not set # CONFIG_PACKAGE_brcmsmac-firmware is not set # CONFIG_PACKAGE_carl9170-firmware is not set # CONFIG_PACKAGE_cypress-firmware-43012-sdio is not set # CONFIG_PACKAGE_cypress-firmware-43340-sdio is not set # CONFIG_PACKAGE_cypress-firmware-43362-sdio is not set # CONFIG_PACKAGE_cypress-firmware-4339-sdio is not set # CONFIG_PACKAGE_cypress-firmware-43430-sdio is not set # CONFIG_PACKAGE_cypress-firmware-43455-sdio is not set # CONFIG_PACKAGE_cypress-firmware-4354-sdio is not set # CONFIG_PACKAGE_cypress-firmware-4356-pcie is not set # CONFIG_PACKAGE_cypress-firmware-4356-sdio is not set # CONFIG_PACKAGE_cypress-firmware-43570-pcie is not set # CONFIG_PACKAGE_cypress-firmware-4359-pcie is not set # CONFIG_PACKAGE_cypress-firmware-4359-sdio is not set # CONFIG_PACKAGE_cypress-firmware-4373-sdio is not set # CONFIG_PACKAGE_cypress-firmware-4373-usb is not set # CONFIG_PACKAGE_cypress-firmware-89459-pcie is not set # CONFIG_PACKAGE_e100-firmware is not set # CONFIG_PACKAGE_edgeport-firmware is not set # CONFIG_PACKAGE_ibt-firmware is not set # CONFIG_PACKAGE_iwl3945-firmware is not set # CONFIG_PACKAGE_iwl4965-firmware is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl100 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl1000 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl105 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl135 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl2000 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl2030 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl3160 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl3168 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl5000 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl5150 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl6000g2 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl6000g2a is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl6000g2b is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl6050 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl7260 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl7265 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl7265d is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl8260c is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl8265 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl9000 is not set # CONFIG_PACKAGE_iwlwifi-firmware-iwl9260 is not set # CONFIG_PACKAGE_libertas-sdio-firmware is not set # CONFIG_PACKAGE_libertas-spi-firmware is not set # CONFIG_PACKAGE_libertas-usb-firmware is not set # CONFIG_PACKAGE_mt7601u-firmware is not set # CONFIG_PACKAGE_mt7622bt-firmware is not set # CONFIG_PACKAGE_mwifiex-pcie-firmware is not set # CONFIG_PACKAGE_mwifiex-sdio-firmware is not set # CONFIG_PACKAGE_mwl8k-firmware is not set # CONFIG_PACKAGE_p54-pci-firmware is not set # CONFIG_PACKAGE_p54-spi-firmware is not set # CONFIG_PACKAGE_p54-usb-firmware is not set # CONFIG_PACKAGE_prism54-firmware is not set # CONFIG_PACKAGE_r8169-firmware is not set # CONFIG_PACKAGE_radeon-firmware is not set # CONFIG_PACKAGE_rs9113-firmware is not set # CONFIG_PACKAGE_rt2800-pci-firmware is not set # CONFIG_PACKAGE_rt2800-usb-firmware is not set # CONFIG_PACKAGE_rt61-pci-firmware is not set # CONFIG_PACKAGE_rt73-usb-firmware is not set # CONFIG_PACKAGE_rtl8188eu-firmware is not set # CONFIG_PACKAGE_rtl8192ce-firmware is not set # CONFIG_PACKAGE_rtl8192cu-firmware is not set # CONFIG_PACKAGE_rtl8192de-firmware is not set # CONFIG_PACKAGE_rtl8192eu-firmware is not set # CONFIG_PACKAGE_rtl8192se-firmware is not set # CONFIG_PACKAGE_rtl8192su-firmware is not set # CONFIG_PACKAGE_rtl8723au-firmware is not set # CONFIG_PACKAGE_rtl8723bs-firmware is not set # CONFIG_PACKAGE_rtl8723bu-firmware is not set # CONFIG_PACKAGE_rtl8821ae-firmware is not set # CONFIG_PACKAGE_rtl8822be-firmware is not set # CONFIG_PACKAGE_rtl8822ce-firmware is not set # CONFIG_PACKAGE_ti-3410-firmware is not set # CONFIG_PACKAGE_ti-5052-firmware is not set CONFIG_PACKAGE_wireless-regdb=y # CONFIG_PACKAGE_wl12xx-firmware is not set # CONFIG_PACKAGE_wl18xx-firmware is not set  # # Fonts #  # # DejaVu # # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuMathTeXGyre is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSans is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSans-Bold is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSans-BoldOblique is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSans-ExtraLight is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSans-Oblique is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansCondensed is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansCondensed-Bold is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansCondensed-BoldOblique is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansCondensed-Oblique is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansMono is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansMono-Bold is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansMono-BoldOblique is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSansMono-Oblique is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerif is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerif-Bold is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerif-BoldItalic is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerif-Italic is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerifCondensed is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerifCondensed-Bold is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerifCondensed-BoldItalic is not set # CONFIG_PACKAGE_dejavu-fonts-ttf-DejaVuSerifCondensed-Italic is not set  # # Kernel modules #  # # Block Devices # # CONFIG_PACKAGE_kmod-aoe is not set # CONFIG_PACKAGE_kmod-ata-core is not set # CONFIG_PACKAGE_kmod-block2mtd is not set # CONFIG_PACKAGE_kmod-dm is not set # CONFIG_PACKAGE_kmod-dm-raid is not set # CONFIG_PACKAGE_kmod-loop is not set # CONFIG_PACKAGE_kmod-md-mod is not set # CONFIG_PACKAGE_kmod-nbd is not set # CONFIG_PACKAGE_kmod-scsi-cdrom is not set # CONFIG_PACKAGE_kmod-scsi-core is not set # CONFIG_PACKAGE_kmod-scsi-generic is not set # CONFIG_PACKAGE_kmod-scsi-tape is not set  # # CAN Support # # CONFIG_PACKAGE_kmod-can is not set  # # Cryptographic API modules # CONFIG_PACKAGE_kmod-crypto-aead=y CONFIG_PACKAGE_kmod-crypto-arc4=y CONFIG_PACKAGE_kmod-crypto-authenc=y # CONFIG_PACKAGE_kmod-crypto-cbc is not set # CONFIG_PACKAGE_kmod-crypto-ccm is not set # CONFIG_PACKAGE_kmod-crypto-cmac is not set CONFIG_PACKAGE_kmod-crypto-crc32c=y # CONFIG_PACKAGE_kmod-crypto-ctr is not set # CONFIG_PACKAGE_kmod-crypto-cts is not set # CONFIG_PACKAGE_kmod-crypto-deflate is not set # CONFIG_PACKAGE_kmod-crypto-des is not set CONFIG_PACKAGE_kmod-crypto-ecb=y # CONFIG_PACKAGE_kmod-crypto-ecdh is not set # CONFIG_PACKAGE_kmod-crypto-echainiv is not set # CONFIG_PACKAGE_kmod-crypto-fcrypt is not set # CONFIG_PACKAGE_kmod-crypto-gcm is not set # CONFIG_PACKAGE_kmod-crypto-gf128 is not set # CONFIG_PACKAGE_kmod-crypto-ghash is not set CONFIG_PACKAGE_kmod-crypto-hash=y # CONFIG_PACKAGE_kmod-crypto-hmac is not set # CONFIG_PACKAGE_kmod-crypto-hw-ccp is not set # CONFIG_PACKAGE_kmod-crypto-hw-geode is not set # CONFIG_PACKAGE_kmod-crypto-hw-hifn-795x is not set # CONFIG_PACKAGE_kmod-crypto-hw-padlock is not set # CONFIG_PACKAGE_kmod-crypto-hw-talitos is not set CONFIG_PACKAGE_kmod-crypto-manager=y # CONFIG_PACKAGE_kmod-crypto-md4 is not set # CONFIG_PACKAGE_kmod-crypto-md5 is not set # CONFIG_PACKAGE_kmod-crypto-michael-mic is not set # CONFIG_PACKAGE_kmod-crypto-misc is not set CONFIG_PACKAGE_kmod-crypto-null=y # CONFIG_PACKAGE_kmod-crypto-pcbc is not set CONFIG_PACKAGE_kmod-crypto-pcompress=y # CONFIG_PACKAGE_kmod-crypto-rmd160 is not set # CONFIG_PACKAGE_kmod-crypto-rng is not set # CONFIG_PACKAGE_kmod-crypto-seqiv is not set CONFIG_PACKAGE_kmod-crypto-sha1=y # CONFIG_PACKAGE_kmod-crypto-sha256 is not set # CONFIG_PACKAGE_kmod-crypto-sha512 is not set # CONFIG_PACKAGE_kmod-crypto-test is not set # CONFIG_PACKAGE_kmod-crypto-user is not set # CONFIG_PACKAGE_kmod-crypto-wq is not set # CONFIG_PACKAGE_kmod-crypto-xcbc is not set # CONFIG_PACKAGE_kmod-crypto-xts is not set CONFIG_PACKAGE_kmod-cryptodev=y  # # Filesystems # # CONFIG_PACKAGE_kmod-fs-afs is not set # CONFIG_PACKAGE_kmod-fs-antfs is not set # CONFIG_PACKAGE_kmod-fs-autofs4 is not set # CONFIG_PACKAGE_kmod-fs-btrfs is not set # CONFIG_PACKAGE_kmod-fs-cifs is not set # CONFIG_PACKAGE_kmod-fs-configfs is not set # CONFIG_PACKAGE_kmod-fs-cramfs is not set # CONFIG_PACKAGE_kmod-fs-exfat is not set # CONFIG_PACKAGE_kmod-fs-exportfs is not set # CONFIG_PACKAGE_kmod-fs-ext4 is not set # CONFIG_PACKAGE_kmod-fs-f2fs is not set # CONFIG_PACKAGE_kmod-fs-fscache is not set # CONFIG_PACKAGE_kmod-fs-hfs is not set # CONFIG_PACKAGE_kmod-fs-hfsplus is not set # CONFIG_PACKAGE_kmod-fs-isofs is not set # CONFIG_PACKAGE_kmod-fs-jfs is not set # CONFIG_PACKAGE_kmod-fs-ksmbd is not set # CONFIG_PACKAGE_kmod-fs-minix is not set # CONFIG_PACKAGE_kmod-fs-msdos is not set # CONFIG_PACKAGE_kmod-fs-nfs is not set # CONFIG_PACKAGE_kmod-fs-nfs-common is not set # CONFIG_PACKAGE_kmod-fs-nfs-common-rpcsec is not set # CONFIG_PACKAGE_kmod-fs-nfs-v3 is not set # CONFIG_PACKAGE_kmod-fs-nfs-v4 is not set # CONFIG_PACKAGE_kmod-fs-nfsd is not set # CONFIG_PACKAGE_kmod-fs-ntfs is not set # CONFIG_PACKAGE_kmod-fs-reiserfs is not set # CONFIG_PACKAGE_kmod-fs-squashfs is not set # CONFIG_PACKAGE_kmod-fs-udf is not set # CONFIG_PACKAGE_kmod-fs-vfat is not set # CONFIG_PACKAGE_kmod-fs-xfs is not set # CONFIG_PACKAGE_kmod-fuse is not set  # # FireWire support # # CONFIG_PACKAGE_kmod-firewire is not set  # # Hardware Monitoring Support # CONFIG_PACKAGE_kmod-hwmon-core=y # CONFIG_PACKAGE_kmod-hwmon-adcxx is not set # CONFIG_PACKAGE_kmod-hwmon-ads1015 is not set # CONFIG_PACKAGE_kmod-hwmon-adt7410 is not set # CONFIG_PACKAGE_kmod-hwmon-adt7475 is not set # CONFIG_PACKAGE_kmod-hwmon-gpiofan is not set # CONFIG_PACKAGE_kmod-hwmon-ina209 is not set # CONFIG_PACKAGE_kmod-hwmon-ina2xx is not set # CONFIG_PACKAGE_kmod-hwmon-it87 is not set # CONFIG_PACKAGE_kmod-hwmon-lm63 is not set # CONFIG_PACKAGE_kmod-hwmon-lm75 is not set # CONFIG_PACKAGE_kmod-hwmon-lm77 is not set # CONFIG_PACKAGE_kmod-hwmon-lm85 is not set # CONFIG_PACKAGE_kmod-hwmon-lm90 is not set # CONFIG_PACKAGE_kmod-hwmon-lm92 is not set # CONFIG_PACKAGE_kmod-hwmon-lm95241 is not set # CONFIG_PACKAGE_kmod-hwmon-ltc4151 is not set # CONFIG_PACKAGE_kmod-hwmon-mcp3021 is not set # CONFIG_PACKAGE_kmod-hwmon-pwmfan is not set # CONFIG_PACKAGE_kmod-hwmon-sch5627 is not set # CONFIG_PACKAGE_kmod-hwmon-sht21 is not set # CONFIG_PACKAGE_kmod-hwmon-tmp102 is not set # CONFIG_PACKAGE_kmod-hwmon-tmp103 is not set # CONFIG_PACKAGE_kmod-hwmon-tmp421 is not set # CONFIG_PACKAGE_kmod-hwmon-vid is not set # CONFIG_PACKAGE_kmod-hwmon-w83793 is not set # CONFIG_PACKAGE_kmod-pmbus-core is not set # CONFIG_PACKAGE_kmod-pmbus-zl6100 is not set  # # I2C support # # CONFIG_PACKAGE_kmod-i2c-core is not set # CONFIG_PACKAGE_kmod-i2c-gpio-custom is not set  # # Industrial I/O Modules # # CONFIG_PACKAGE_kmod-iio-ad799x is not set # CONFIG_PACKAGE_kmod-iio-am2315 is not set # CONFIG_PACKAGE_kmod-iio-bh1750 is not set # CONFIG_PACKAGE_kmod-iio-bme680 is not set # CONFIG_PACKAGE_kmod-iio-bme680-i2c is not set # CONFIG_PACKAGE_kmod-iio-bme680-spi is not set # CONFIG_PACKAGE_kmod-iio-bmp280 is not set # CONFIG_PACKAGE_kmod-iio-bmp280-i2c is not set # CONFIG_PACKAGE_kmod-iio-bmp280-spi is not set # CONFIG_PACKAGE_kmod-iio-ccs811 is not set # CONFIG_PACKAGE_kmod-iio-core is not set # CONFIG_PACKAGE_kmod-iio-fxos8700 is not set # CONFIG_PACKAGE_kmod-iio-fxos8700-i2c is not set # CONFIG_PACKAGE_kmod-iio-fxos8700-spi is not set # CONFIG_PACKAGE_kmod-iio-hmc5843 is not set # CONFIG_PACKAGE_kmod-iio-htu21 is not set # CONFIG_PACKAGE_kmod-iio-kfifo-buf is not set # CONFIG_PACKAGE_kmod-iio-si7020 is not set # CONFIG_PACKAGE_kmod-iio-sps30 is not set # CONFIG_PACKAGE_kmod-iio-st_accel is not set # CONFIG_PACKAGE_kmod-iio-st_sensors-i2c is not set # CONFIG_PACKAGE_kmod-iio-st_sensors-spi is not set # CONFIG_PACKAGE_kmod-iio-tsl4531 is not set # CONFIG_PACKAGE_kmod-industrialio-triggered-buffer is not set  # # Input modules # # CONFIG_PACKAGE_kmod-hid is not set # CONFIG_PACKAGE_kmod-hid-generic is not set # CONFIG_PACKAGE_kmod-input-core is not set # CONFIG_PACKAGE_kmod-input-evdev is not set # CONFIG_PACKAGE_kmod-input-gpio-encoder is not set # CONFIG_PACKAGE_kmod-input-gpio-keys is not set # CONFIG_PACKAGE_kmod-input-gpio-keys-polled is not set # CONFIG_PACKAGE_kmod-input-joydev is not set # CONFIG_PACKAGE_kmod-input-matrixkmap is not set # CONFIG_PACKAGE_kmod-input-polldev is not set # CONFIG_PACKAGE_kmod-input-touchscreen-ads7846 is not set # CONFIG_PACKAGE_kmod-input-uinput is not set  # # LED modules # # CONFIG_PACKAGE_kmod-leds-gpio is not set # CONFIG_PACKAGE_kmod-leds-nu801 is not set # CONFIG_PACKAGE_kmod-leds-pca963x is not set # CONFIG_PACKAGE_kmod-leds-rb750 is not set # CONFIG_PACKAGE_kmod-leds-wndr3700-usb is not set # CONFIG_PACKAGE_kmod-ledtrig-default-on is not set # CONFIG_PACKAGE_kmod-ledtrig-gpio is not set # CONFIG_PACKAGE_kmod-ledtrig-heartbeat is not set # CONFIG_PACKAGE_kmod-ledtrig-netdev is not set # CONFIG_PACKAGE_kmod-ledtrig-oneshot is not set # CONFIG_PACKAGE_kmod-ledtrig-timer is not set # CONFIG_PACKAGE_kmod-ledtrig-transient is not set  # # Libraries # # CONFIG_PACKAGE_kmod-lib-cordic is not set CONFIG_PACKAGE_kmod-lib-crc-ccitt=y # CONFIG_PACKAGE_kmod-lib-crc-itu-t is not set # CONFIG_PACKAGE_kmod-lib-crc16 is not set CONFIG_PACKAGE_kmod-lib-crc32c=y # CONFIG_PACKAGE_kmod-lib-crc7 is not set # CONFIG_PACKAGE_kmod-lib-crc8 is not set CONFIG_PACKAGE_kmod-lib-textsearch=y  # # Native Language Support # CONFIG_PACKAGE_kmod-nls-base=y # CONFIG_PACKAGE_kmod-nls-cp1250 is not set # CONFIG_PACKAGE_kmod-nls-cp1251 is not set # CONFIG_PACKAGE_kmod-nls-cp437 is not set # CONFIG_PACKAGE_kmod-nls-cp775 is not set # CONFIG_PACKAGE_kmod-nls-cp850 is not set # CONFIG_PACKAGE_kmod-nls-cp852 is not set # CONFIG_PACKAGE_kmod-nls-cp862 is not set # CONFIG_PACKAGE_kmod-nls-cp864 is not set # CONFIG_PACKAGE_kmod-nls-cp866 is not set # CONFIG_PACKAGE_kmod-nls-cp932 is not set # CONFIG_PACKAGE_kmod-nls-cp936 is not set # CONFIG_PACKAGE_kmod-nls-cp950 is not set # CONFIG_PACKAGE_kmod-nls-iso8859-1 is not set # CONFIG_PACKAGE_kmod-nls-iso8859-13 is not set # CONFIG_PACKAGE_kmod-nls-iso8859-15 is not set # CONFIG_PACKAGE_kmod-nls-iso8859-2 is not set # CONFIG_PACKAGE_kmod-nls-iso8859-6 is not set # CONFIG_PACKAGE_kmod-nls-iso8859-8 is not set # CONFIG_PACKAGE_kmod-nls-koi8r is not set # CONFIG_PACKAGE_kmod-nls-utf8 is not set  # # Netfilter Extensions # # CONFIG_PACKAGE_kmod-arptables is not set # CONFIG_PACKAGE_kmod-br-netfilter is not set # CONFIG_PACKAGE_kmod-ebtables is not set # CONFIG_PACKAGE_kmod-ebtables-ipv4 is not set # CONFIG_PACKAGE_kmod-ebtables-ipv6 is not set # CONFIG_PACKAGE_kmod-ebtables-watchers is not set # CONFIG_PACKAGE_kmod-ip6tables is not set # CONFIG_PACKAGE_kmod-ip6tables-extra is not set # CONFIG_PACKAGE_kmod-ipt-account is not set # CONFIG_PACKAGE_kmod-ipt-chaos is not set # CONFIG_PACKAGE_kmod-ipt-checksum is not set # CONFIG_PACKAGE_kmod-ipt-cluster is not set # CONFIG_PACKAGE_kmod-ipt-clusterip is not set # CONFIG_PACKAGE_kmod-ipt-compat-xtables is not set # CONFIG_PACKAGE_kmod-ipt-condition is not set CONFIG_PACKAGE_kmod-ipt-conntrack=y # CONFIG_PACKAGE_kmod-ipt-conntrack-extra is not set # CONFIG_PACKAGE_kmod-ipt-conntrack-label is not set CONFIG_PACKAGE_kmod-ipt-core=y # CONFIG_PACKAGE_kmod-ipt-debug is not set # CONFIG_PACKAGE_kmod-ipt-delude is not set # CONFIG_PACKAGE_kmod-ipt-dhcpmac is not set # CONFIG_PACKAGE_kmod-ipt-dnetmap is not set # CONFIG_PACKAGE_kmod-ipt-extra is not set # CONFIG_PACKAGE_kmod-ipt-filter is not set CONFIG_PACKAGE_kmod-ipt-fullconenat=y # CONFIG_PACKAGE_kmod-ipt-fuzzy is not set # CONFIG_PACKAGE_kmod-ipt-geoip is not set # CONFIG_PACKAGE_kmod-ipt-hashlimit is not set # CONFIG_PACKAGE_kmod-ipt-iface is not set # CONFIG_PACKAGE_kmod-ipt-ipmark is not set # CONFIG_PACKAGE_kmod-ipt-ipopt is not set # CONFIG_PACKAGE_kmod-ipt-ipp2p is not set # CONFIG_PACKAGE_kmod-ipt-iprange is not set # CONFIG_PACKAGE_kmod-ipt-ipsec is not set CONFIG_PACKAGE_kmod-ipt-ipset=y # CONFIG_PACKAGE_kmod-ipt-ipv4options is not set # CONFIG_PACKAGE_kmod-ipt-led is not set # CONFIG_PACKAGE_kmod-ipt-length2 is not set # CONFIG_PACKAGE_kmod-ipt-logmark is not set # CONFIG_PACKAGE_kmod-ipt-lscan is not set # CONFIG_PACKAGE_kmod-ipt-lua is not set CONFIG_PACKAGE_kmod-ipt-nat=y # CONFIG_PACKAGE_kmod-ipt-nat-extra is not set # CONFIG_PACKAGE_kmod-ipt-nat6 is not set # CONFIG_PACKAGE_kmod-ipt-nathelper-rtsp is not set # CONFIG_PACKAGE_kmod-ipt-nflog is not set # CONFIG_PACKAGE_kmod-ipt-nfqueue is not set # CONFIG_PACKAGE_kmod-ipt-physdev is not set # CONFIG_PACKAGE_kmod-ipt-psd is not set # CONFIG_PACKAGE_kmod-ipt-quota2 is not set CONFIG_PACKAGE_kmod-ipt-raw=y # CONFIG_PACKAGE_kmod-ipt-raw6 is not set # CONFIG_PACKAGE_kmod-ipt-rpfilter is not set # CONFIG_PACKAGE_kmod-ipt-sysrq is not set # CONFIG_PACKAGE_kmod-ipt-tarpit is not set # CONFIG_PACKAGE_kmod-ipt-tee is not set # CONFIG_PACKAGE_kmod-ipt-tproxy is not set # CONFIG_PACKAGE_kmod-ipt-u32 is not set # CONFIG_PACKAGE_kmod-ipt-ulog is not set CONFIG_PACKAGE_kmod-nf-conntrack=y CONFIG_PACKAGE_kmod-nf-conntrack-netlink=y CONFIG_PACKAGE_kmod-nf-conntrack6=y CONFIG_PACKAGE_kmod-nf-ipt=y # CONFIG_PACKAGE_kmod-nf-ipt6 is not set # CONFIG_PACKAGE_kmod-nf-ipvs is not set CONFIG_PACKAGE_kmod-nf-nat=y # CONFIG_PACKAGE_kmod-nf-nat6 is not set CONFIG_PACKAGE_kmod-nf-nathelper=y CONFIG_PACKAGE_kmod-nf-nathelper-extra=y CONFIG_PACKAGE_kmod-nf-reject=y # CONFIG_PACKAGE_kmod-nf-reject6 is not set CONFIG_PACKAGE_kmod-nfnetlink=y # CONFIG_PACKAGE_kmod-nfnetlink-log is not set # CONFIG_PACKAGE_kmod-nfnetlink-queue is not set # CONFIG_PACKAGE_kmod-nft-arp is not set # CONFIG_PACKAGE_kmod-nft-bridge is not set # CONFIG_PACKAGE_kmod-nft-core is not set # CONFIG_PACKAGE_kmod-nft-fib is not set # CONFIG_PACKAGE_kmod-nft-nat is not set # CONFIG_PACKAGE_kmod-nft-nat6 is not set # CONFIG_PACKAGE_kmod-nft-netdev is not set  # # Network Devices # # CONFIG_PACKAGE_kmod-3c59x is not set # CONFIG_PACKAGE_kmod-8139cp is not set # CONFIG_PACKAGE_kmod-8139too is not set # CONFIG_PACKAGE_kmod-alx is not set # CONFIG_PACKAGE_kmod-atl1 is not set # CONFIG_PACKAGE_kmod-atl1c is not set # CONFIG_PACKAGE_kmod-atl1e is not set # CONFIG_PACKAGE_kmod-atl2 is not set # CONFIG_PACKAGE_kmod-b44 is not set # CONFIG_PACKAGE_kmod-be2net is not set # CONFIG_PACKAGE_kmod-bnx2 is not set # CONFIG_PACKAGE_kmod-bnx2x is not set # CONFIG_PACKAGE_kmod-dm9000 is not set # CONFIG_PACKAGE_kmod-dummy is not set # CONFIG_PACKAGE_kmod-e100 is not set # CONFIG_PACKAGE_kmod-e1000 is not set # CONFIG_PACKAGE_kmod-et131x is not set # CONFIG_PACKAGE_kmod-ethoc is not set # CONFIG_PACKAGE_kmod-forcedeth is not set # CONFIG_PACKAGE_kmod-gigaset is not set # CONFIG_PACKAGE_kmod-hfcmulti is not set # CONFIG_PACKAGE_kmod-hfcpci is not set # CONFIG_PACKAGE_kmod-i40e is not set # CONFIG_PACKAGE_kmod-i40evf is not set # CONFIG_PACKAGE_kmod-ifb is not set # CONFIG_PACKAGE_kmod-igb is not set # CONFIG_PACKAGE_kmod-ixgbe is not set # CONFIG_PACKAGE_kmod-ixgbevf is not set # CONFIG_PACKAGE_kmod-libphy is not set CONFIG_PACKAGE_kmod-macvlan=y # CONFIG_PACKAGE_kmod-mdio-gpio is not set # CONFIG_PACKAGE_kmod-mii is not set # CONFIG_PACKAGE_kmod-mlx4-core is not set # CONFIG_PACKAGE_kmod-mlx5-core is not set # CONFIG_PACKAGE_kmod-natsemi is not set # CONFIG_PACKAGE_kmod-ne2k-pci is not set # CONFIG_PACKAGE_kmod-niu is not set # CONFIG_PACKAGE_kmod-of-mdio is not set # CONFIG_PACKAGE_kmod-pcnet32 is not set # CONFIG_PACKAGE_kmod-phy-broadcom is not set # CONFIG_PACKAGE_kmod-phy-realtek is not set # CONFIG_PACKAGE_kmod-r6040 is not set # CONFIG_PACKAGE_kmod-r8125 is not set # CONFIG_PACKAGE_kmod-r8168 is not set # CONFIG_PACKAGE_kmod-r8169 is not set # CONFIG_PACKAGE_kmod-sfc is not set # CONFIG_PACKAGE_kmod-siit is not set # CONFIG_PACKAGE_kmod-sis190 is not set # CONFIG_PACKAGE_kmod-sis900 is not set # CONFIG_PACKAGE_kmod-skge is not set # CONFIG_PACKAGE_kmod-sky2 is not set # CONFIG_PACKAGE_kmod-solos-pci is not set # CONFIG_PACKAGE_kmod-spi-ks8995 is not set # CONFIG_PACKAGE_kmod-swconfig is not set # CONFIG_PACKAGE_kmod-switch-ip17xx is not set # CONFIG_PACKAGE_kmod-switch-mvsw61xx is not set # CONFIG_PACKAGE_kmod-switch-rtl8306 is not set # CONFIG_PACKAGE_kmod-switch-rtl8366-smi is not set # CONFIG_PACKAGE_kmod-switch-rtl8366rb is not set # CONFIG_PACKAGE_kmod-switch-rtl8366s is not set # CONFIG_PACKAGE_kmod-switch-rtl8367b is not set # CONFIG_PACKAGE_kmod-tg3 is not set # CONFIG_PACKAGE_kmod-tulip is not set # CONFIG_PACKAGE_kmod-via-rhine is not set # CONFIG_PACKAGE_kmod-via-velocity is not set # CONFIG_PACKAGE_kmod-vmxnet3 is not set  # # Network Support # # CONFIG_PACKAGE_kmod-atm is not set # CONFIG_PACKAGE_kmod-ax25 is not set # CONFIG_PACKAGE_kmod-batman-adv is not set # CONFIG_PACKAGE_kmod-bonding is not set # CONFIG_PACKAGE_kmod-bpf-test is not set # CONFIG_PACKAGE_kmod-capi is not set # CONFIG_PACKAGE_kmod-dnsresolver is not set CONFIG_PACKAGE_kmod-fast-classifier=y # CONFIG_PACKAGE_kmod-fou is not set # CONFIG_PACKAGE_kmod-fou6 is not set # CONFIG_PACKAGE_kmod-geneve is not set # CONFIG_PACKAGE_kmod-gre is not set # CONFIG_PACKAGE_kmod-gre6 is not set # CONFIG_PACKAGE_kmod-ip6-tunnel is not set # CONFIG_PACKAGE_kmod-ipip is not set # CONFIG_PACKAGE_kmod-ipsec is not set # CONFIG_PACKAGE_kmod-iptunnel6 is not set # CONFIG_PACKAGE_kmod-isdn4linux is not set # CONFIG_PACKAGE_kmod-jool is not set # CONFIG_PACKAGE_kmod-l2tp is not set # CONFIG_PACKAGE_kmod-l2tp-eth is not set # CONFIG_PACKAGE_kmod-l2tp-ip is not set # CONFIG_PACKAGE_kmod-macremapper is not set # CONFIG_PACKAGE_kmod-macsec is not set # CONFIG_PACKAGE_kmod-misdn is not set # CONFIG_PACKAGE_kmod-mpls is not set # CONFIG_PACKAGE_kmod-nat46 is not set # CONFIG_PACKAGE_kmod-netem is not set # CONFIG_PACKAGE_kmod-nlmon is not set # CONFIG_PACKAGE_kmod-nsh is not set # CONFIG_PACKAGE_kmod-openvswitch is not set # CONFIG_PACKAGE_kmod-openvswitch-geneve is not set # CONFIG_PACKAGE_kmod-openvswitch-gre is not set # CONFIG_PACKAGE_kmod-openvswitch-vxlan is not set # CONFIG_PACKAGE_kmod-pktgen is not set CONFIG_PACKAGE_kmod-ppp=y CONFIG_PACKAGE_kmod-mppe=y # CONFIG_PACKAGE_kmod-ppp-synctty is not set # CONFIG_PACKAGE_kmod-pppoa is not set CONFIG_PACKAGE_kmod-pppoe=y # CONFIG_PACKAGE_kmod-pppol2tp is not set CONFIG_PACKAGE_kmod-pppox=y # CONFIG_PACKAGE_kmod-pptp is not set CONFIG_PACKAGE_kmod-sched=y # CONFIG_PACKAGE_kmod-sched-act-vlan is not set # CONFIG_PACKAGE_kmod-sched-bpf is not set # CONFIG_PACKAGE_kmod-sched-cake is not set # CONFIG_PACKAGE_kmod-sched-connmark is not set CONFIG_PACKAGE_kmod-sched-core=y # CONFIG_PACKAGE_kmod-sched-ctinfo is not set # CONFIG_PACKAGE_kmod-sched-flower is not set # CONFIG_PACKAGE_kmod-sched-ipset is not set # CONFIG_PACKAGE_kmod-sched-mqprio is not set # CONFIG_PACKAGE_kmod-sctp is not set CONFIG_PACKAGE_kmod-shortcut-fe=y # CONFIG_PACKAGE_kmod-shortcut-fe-cm is not set # CONFIG_PACKAGE_kmod-sit is not set CONFIG_PACKAGE_kmod-slhc=y # CONFIG_PACKAGE_kmod-slip is not set CONFIG_PACKAGE_kmod-tcp-bbr=y # CONFIG_PACKAGE_kmod-trelay is not set # CONFIG_PACKAGE_kmod-tun is not set # CONFIG_PACKAGE_kmod-veth is not set # CONFIG_PACKAGE_kmod-vxlan is not set # CONFIG_PACKAGE_kmod-wireguard is not set  # # Other modules # # CONFIG_PACKAGE_kmod-6lowpan is not set # CONFIG_PACKAGE_kmod-ath3k is not set # CONFIG_PACKAGE_kmod-bcma is not set # CONFIG_PACKAGE_kmod-bluetooth is not set # CONFIG_PACKAGE_kmod-bluetooth_6lowpan is not set # CONFIG_PACKAGE_kmod-bmp085 is not set # CONFIG_PACKAGE_kmod-bmp085-i2c is not set # CONFIG_PACKAGE_kmod-bmp085-spi is not set # CONFIG_PACKAGE_kmod-btmrvl is not set # CONFIG_PACKAGE_kmod-button-hotplug is not set # CONFIG_PACKAGE_kmod-echo is not set # CONFIG_PACKAGE_kmod-eeprom-93cx6 is not set # CONFIG_PACKAGE_kmod-eeprom-at24 is not set # CONFIG_PACKAGE_kmod-eeprom-at25 is not set # CONFIG_PACKAGE_kmod-gpio-beeper is not set CONFIG_PACKAGE_kmod-gpio-button-hotplug=y # CONFIG_PACKAGE_kmod-gpio-dev is not set # CONFIG_PACKAGE_kmod-gpio-mcp23s08 is not set # CONFIG_PACKAGE_kmod-gpio-nxp-74hc164 is not set # CONFIG_PACKAGE_kmod-gpio-pca953x is not set # CONFIG_PACKAGE_kmod-gpio-pcf857x is not set # CONFIG_PACKAGE_kmod-ikconfig is not set # CONFIG_PACKAGE_kmod-it87-wdt is not set # CONFIG_PACKAGE_kmod-itco-wdt is not set # CONFIG_PACKAGE_kmod-lp is not set # CONFIG_PACKAGE_kmod-mmc is not set # CONFIG_PACKAGE_kmod-mtd-rw is not set # CONFIG_PACKAGE_kmod-mtdoops is not set # CONFIG_PACKAGE_kmod-mtdram is not set # CONFIG_PACKAGE_kmod-mtdtests is not set # CONFIG_PACKAGE_kmod-parport-pc is not set # CONFIG_PACKAGE_kmod-ppdev is not set # CONFIG_PACKAGE_kmod-pps is not set # CONFIG_PACKAGE_kmod-pps-gpio is not set # CONFIG_PACKAGE_kmod-pps-ldisc is not set # CONFIG_PACKAGE_kmod-ptp is not set # CONFIG_PACKAGE_kmod-random-core is not set # CONFIG_PACKAGE_kmod-random-tpm is not set # CONFIG_PACKAGE_kmod-rtc-ds1307 is not set # CONFIG_PACKAGE_kmod-rtc-ds1374 is not set # CONFIG_PACKAGE_kmod-rtc-ds1672 is not set # CONFIG_PACKAGE_kmod-rtc-em3027 is not set # CONFIG_PACKAGE_kmod-rtc-isl1208 is not set # CONFIG_PACKAGE_kmod-rtc-pcf2123 is not set # CONFIG_PACKAGE_kmod-rtc-pcf2127 is not set # CONFIG_PACKAGE_kmod-rtc-pcf8563 is not set # CONFIG_PACKAGE_kmod-rtc-pt7c4338 is not set # CONFIG_PACKAGE_kmod-rtc-rs5c372a is not set # CONFIG_PACKAGE_kmod-rtc-rx8025 is not set # CONFIG_PACKAGE_kmod-sdhci is not set # CONFIG_PACKAGE_kmod-serial-8250 is not set # CONFIG_PACKAGE_kmod-serial-8250-exar is not set # CONFIG_PACKAGE_kmod-softdog is not set # CONFIG_PACKAGE_kmod-ssb is not set CONFIG_PACKAGE_kmod-thermal=y # CONFIG_PACKAGE_kmod-tpm is not set # CONFIG_PACKAGE_kmod-tpm-i2c-atmel is not set # CONFIG_PACKAGE_kmod-tpm-i2c-infineon is not set # CONFIG_PACKAGE_kmod-w83627hf-wdt is not set # CONFIG_PACKAGE_kmod-wifidog-ng is not set # CONFIG_PACKAGE_kmod-zram is not set  # # PCMCIA support #  # # SPI Support # # CONFIG_PACKAGE_kmod-mmc-spi is not set # CONFIG_PACKAGE_kmod-spi-bitbang is not set # CONFIG_PACKAGE_kmod-spi-dev is not set # CONFIG_PACKAGE_kmod-spi-gpio is not set # CONFIG_PACKAGE_kmod-spi-gpio-custom is not set # CONFIG_PACKAGE_kmod-spi-vsc7385 is not set  # # Sound Support # # CONFIG_PACKAGE_kmod-sound-core is not set  # # USB Support # # CONFIG_PACKAGE_kmod-chaoskey is not set # CONFIG_PACKAGE_kmod-usb-acm is not set # CONFIG_PACKAGE_kmod-usb-atm is not set # CONFIG_PACKAGE_kmod-usb-chipidea is not set # CONFIG_PACKAGE_kmod-usb-chipidea2 is not set # CONFIG_PACKAGE_kmod-usb-cm109 is not set CONFIG_PACKAGE_kmod-usb-core=y # CONFIG_PACKAGE_kmod-usb-dwc2 is not set # CONFIG_PACKAGE_kmod-usb-dwc3 is not set CONFIG_PACKAGE_kmod-usb-ehci=y # CONFIG_PACKAGE_kmod-usb-gadget-cdc-composite is not set # CONFIG_PACKAGE_kmod-usb-gadget-ehci-debug is not set # CONFIG_PACKAGE_kmod-usb-gadget-eth is not set # CONFIG_PACKAGE_kmod-usb-gadget-hid is not set # CONFIG_PACKAGE_kmod-usb-gadget-mass-storage is not set # CONFIG_PACKAGE_kmod-usb-gadget-serial is not set # CONFIG_PACKAGE_kmod-usb-hid is not set CONFIG_PACKAGE_kmod-usb-ledtrig-usbport=y # CONFIG_PACKAGE_kmod-usb-net is not set # CONFIG_PACKAGE_kmod-usb-net2280 is not set # CONFIG_PACKAGE_kmod-usb-ohci is not set # CONFIG_PACKAGE_kmod-usb-ohci-pci is not set # CONFIG_PACKAGE_kmod-usb-printer is not set # CONFIG_PACKAGE_kmod-usb-serial is not set # CONFIG_PACKAGE_kmod-usb-storage is not set # CONFIG_PACKAGE_kmod-usb-storage-extras is not set # CONFIG_PACKAGE_kmod-usb-storage-uas is not set # CONFIG_PACKAGE_kmod-usb-uhci is not set # CONFIG_PACKAGE_kmod-usb-wdm is not set # CONFIG_PACKAGE_kmod-usb-yealink is not set CONFIG_PACKAGE_kmod-usb2=y # CONFIG_PACKAGE_kmod-usb2-pci is not set # CONFIG_PACKAGE_kmod-usb3 is not set # CONFIG_PACKAGE_kmod-usbip is not set # CONFIG_PACKAGE_kmod-usbip-client is not set # CONFIG_PACKAGE_kmod-usbip-server is not set # CONFIG_PACKAGE_kmod-usbmon is not set  # # Video Support # # CONFIG_PACKAGE_kmod-video-core is not set  # # Virtualization #  # # Voice over IP #  # # W1 support # # CONFIG_PACKAGE_kmod-w1 is not set  # # WPAN 802.15.4 Support # # CONFIG_PACKAGE_kmod-at86rf230 is not set # CONFIG_PACKAGE_kmod-atusb is not set # CONFIG_PACKAGE_kmod-cc2520 is not set # CONFIG_PACKAGE_kmod-fakelb is not set # CONFIG_PACKAGE_kmod-ieee802154 is not set # CONFIG_PACKAGE_kmod-ieee802154_6lowpan is not set # CONFIG_PACKAGE_kmod-mac802154 is not set # CONFIG_PACKAGE_kmod-mrf24j40 is not set  # # Wireless Drivers # # CONFIG_PACKAGE_kmod-adm8211 is not set CONFIG_PACKAGE_kmod-ath=y CONFIG_ATH_USER_REGD=y # CONFIG_PACKAGE_ATH_DEBUG is not set CONFIG_PACKAGE_ATH_DFS=y # CONFIG_PACKAGE_ATH_DYNACK is not set CONFIG_PACKAGE_kmod-ath10k=y CONFIG_ATH10K_LEDS=y CONFIG_ATH10K_THERMAL=y # CONFIG_PACKAGE_kmod-ath10k-ct is not set # CONFIG_PACKAGE_kmod-ath10k-ct-smallbuffers is not set # CONFIG_PACKAGE_kmod-ath5k is not set # CONFIG_PACKAGE_kmod-ath6kl-sdio is not set # CONFIG_PACKAGE_kmod-ath6kl-usb is not set CONFIG_PACKAGE_kmod-ath9k=y # CONFIG_ATH9K_SUPPORT_PCOEM is not set # CONFIG_ATH9K_TX99 is not set CONFIG_ATH9K_UBNTHSR=y CONFIG_PACKAGE_kmod-ath9k-common=y # CONFIG_PACKAGE_kmod-ath9k-htc is not set # CONFIG_PACKAGE_kmod-b43 is not set # CONFIG_PACKAGE_kmod-b43legacy is not set # CONFIG_PACKAGE_kmod-brcmfmac is not set # CONFIG_PACKAGE_kmod-brcmsmac is not set # CONFIG_PACKAGE_kmod-brcmutil is not set # CONFIG_PACKAGE_kmod-carl9170 is not set CONFIG_PACKAGE_kmod-cfg80211=y # CONFIG_PACKAGE_kmod-hermes is not set # CONFIG_PACKAGE_kmod-hermes-pci is not set # CONFIG_PACKAGE_kmod-hermes-plx is not set # CONFIG_PACKAGE_kmod-iwl-legacy is not set # CONFIG_PACKAGE_kmod-iwl3945 is not set # CONFIG_PACKAGE_kmod-iwl4965 is not set # CONFIG_PACKAGE_kmod-iwlwifi is not set # CONFIG_PACKAGE_kmod-lib80211 is not set # CONFIG_PACKAGE_kmod-libertas-sdio is not set # CONFIG_PACKAGE_kmod-libertas-spi is not set # CONFIG_PACKAGE_kmod-libertas-usb is not set CONFIG_PACKAGE_kmod-mac80211=y CONFIG_PACKAGE_MAC80211_DEBUGFS=y # CONFIG_PACKAGE_MAC80211_TRACING is not set CONFIG_PACKAGE_MAC80211_MESH=y # CONFIG_PACKAGE_kmod-mac80211-hwsim is not set # CONFIG_PACKAGE_kmod-mt76 is not set # CONFIG_PACKAGE_kmod-mt7601u is not set # CONFIG_PACKAGE_kmod-mt7603 is not set # CONFIG_PACKAGE_kmod-mt7615e is not set # CONFIG_PACKAGE_kmod-mt76x0e is not set # CONFIG_PACKAGE_kmod-mt76x0u is not set # CONFIG_PACKAGE_kmod-mt76x2 is not set # CONFIG_PACKAGE_kmod-mt76x2u is not set # CONFIG_PACKAGE_kmod-mt_wifi is not set # CONFIG_PACKAGE_kmod-mwifiex-pcie is not set # CONFIG_PACKAGE_kmod-mwifiex-sdio is not set # CONFIG_PACKAGE_kmod-mwl8k is not set # CONFIG_PACKAGE_kmod-net-prism54 is not set # CONFIG_PACKAGE_kmod-net-rtl8192su is not set # CONFIG_PACKAGE_kmod-owl-loader is not set # CONFIG_PACKAGE_kmod-p54-common is not set # CONFIG_PACKAGE_kmod-p54-pci is not set # CONFIG_PACKAGE_kmod-p54-usb is not set # CONFIG_PACKAGE_kmod-rsi91x is not set # CONFIG_PACKAGE_kmod-rsi91x-sdio is not set # CONFIG_PACKAGE_kmod-rsi91x-usb is not set # CONFIG_PACKAGE_kmod-rt2400-pci is not set # CONFIG_PACKAGE_kmod-rt2500-pci is not set # CONFIG_PACKAGE_kmod-rt2500-usb is not set # CONFIG_PACKAGE_kmod-rt2800-pci is not set # CONFIG_PACKAGE_kmod-rt2800-usb is not set # CONFIG_PACKAGE_kmod-rt2x00-lib is not set # CONFIG_PACKAGE_kmod-rt61-pci is not set # CONFIG_PACKAGE_kmod-rt73-usb is not set # CONFIG_PACKAGE_kmod-rtl8180 is not set # CONFIG_PACKAGE_kmod-rtl8187 is not set # CONFIG_PACKAGE_kmod-rtl8192ce is not set # CONFIG_PACKAGE_kmod-rtl8192cu is not set # CONFIG_PACKAGE_kmod-rtl8192de is not set # CONFIG_PACKAGE_kmod-rtl8192se is not set # CONFIG_PACKAGE_kmod-rtl8821ae is not set # CONFIG_PACKAGE_kmod-rtl8xxxu is not set # CONFIG_PACKAGE_kmod-wl12xx is not set # CONFIG_PACKAGE_kmod-wl18xx is not set # CONFIG_PACKAGE_kmod-wlcore is not set # CONFIG_PACKAGE_kmod-zd1211rw is not set # CONFIG_PACKAGE_wifi-l1profile is not set  # # Languages #  # # Erlang # # CONFIG_PACKAGE_erlang is not set # CONFIG_PACKAGE_erlang-asn1 is not set # CONFIG_PACKAGE_erlang-compiler is not set # CONFIG_PACKAGE_erlang-crypto is not set # CONFIG_PACKAGE_erlang-hipe is not set # CONFIG_PACKAGE_erlang-inets is not set # CONFIG_PACKAGE_erlang-mnesia is not set # CONFIG_PACKAGE_erlang-runtime-tools is not set # CONFIG_PACKAGE_erlang-snmp is not set # CONFIG_PACKAGE_erlang-ssh is not set # CONFIG_PACKAGE_erlang-ssl is not set # CONFIG_PACKAGE_erlang-syntax-tools is not set  # # Go # # CONFIG_PACKAGE_golang is not set # CONFIG_PACKAGE_golang-doc is not set # CONFIG_PACKAGE_golang-github-nextdns-nextdns-dev is not set # CONFIG_PACKAGE_golang-src is not set  # # Java # # CONFIG_PACKAGE_jamvm is not set  # # Lua # # CONFIG_PACKAGE_dkjson is not set # CONFIG_PACKAGE_json4lua is not set # CONFIG_PACKAGE_ldbus is not set CONFIG_PACKAGE_libiwinfo-lua=y # CONFIG_PACKAGE_lpeg is not set # CONFIG_PACKAGE_lsqlite3 is not set CONFIG_PACKAGE_lua=y # CONFIG_PACKAGE_lua-bencode is not set # CONFIG_PACKAGE_lua-cjson is not set # CONFIG_PACKAGE_lua-copas is not set # CONFIG_PACKAGE_lua-coxpcall is not set # CONFIG_PACKAGE_lua-examples is not set # CONFIG_PACKAGE_lua-lzlib is not set # CONFIG_PACKAGE_lua-md5 is not set # CONFIG_PACKAGE_lua-mobdebug is not set # CONFIG_PACKAGE_lua-mosquitto is not set # CONFIG_PACKAGE_lua-openssl is not set # CONFIG_PACKAGE_lua-penlight is not set # CONFIG_PACKAGE_lua-rings is not set # CONFIG_PACKAGE_lua-rs232 is not set # CONFIG_PACKAGE_lua-sha2 is not set # CONFIG_PACKAGE_lua-wsapi-base is not set # CONFIG_PACKAGE_lua-wsapi-xavante is not set # CONFIG_PACKAGE_lua-xavante is not set # CONFIG_PACKAGE_luabitop is not set # CONFIG_PACKAGE_luac is not set # CONFIG_PACKAGE_luaexpat is not set # CONFIG_PACKAGE_luafilesystem is not set # CONFIG_PACKAGE_luajit is not set # CONFIG_PACKAGE_lualanes is not set # CONFIG_PACKAGE_luaposix is not set # CONFIG_PACKAGE_luarocks is not set # CONFIG_PACKAGE_luasec is not set # CONFIG_PACKAGE_luasoap is not set # CONFIG_PACKAGE_luasocket is not set # CONFIG_PACKAGE_luasql-mysql is not set # CONFIG_PACKAGE_luasql-pgsql is not set # CONFIG_PACKAGE_luasql-sqlite3 is not set CONFIG_PACKAGE_luci-lib-fs=y # CONFIG_PACKAGE_luv is not set # CONFIG_PACKAGE_lzmq is not set # CONFIG_PACKAGE_uuid is not set  # # Node.js # # CONFIG_PACKAGE_node is not set # CONFIG_PACKAGE_node-arduino-firmata is not set # CONFIG_PACKAGE_node-cylon is not set # CONFIG_PACKAGE_node-cylon-firmata is not set # CONFIG_PACKAGE_node-cylon-gpio is not set # CONFIG_PACKAGE_node-cylon-i2c is not set # CONFIG_PACKAGE_node-hid is not set # CONFIG_PACKAGE_node-homebridge is not set # CONFIG_PACKAGE_node-javascript-obfuscator is not set # CONFIG_PACKAGE_node-npm is not set # CONFIG_PACKAGE_node-serialport is not set # CONFIG_PACKAGE_node-serialport-bindings is not set  # # PHP # # CONFIG_PACKAGE_php7 is not set  # # Perl # # CONFIG_PACKAGE_perl is not set  # # Python # # CONFIG_PACKAGE_gunicorn is not set # CONFIG_PACKAGE_gunicorn3 is not set # CONFIG_PACKAGE_micropython is not set # CONFIG_PACKAGE_micropython-lib is not set # CONFIG_PACKAGE_python is not set # CONFIG_PACKAGE_python-astral is not set # CONFIG_PACKAGE_python-astral-src is not set # CONFIG_PACKAGE_python-attrs is not set # CONFIG_PACKAGE_python-attrs-src is not set # CONFIG_PACKAGE_python-automat is not set # CONFIG_PACKAGE_python-automat-src is not set # CONFIG_PACKAGE_python-awscli is not set # CONFIG_PACKAGE_python-awscli-src is not set # CONFIG_PACKAGE_python-base is not set # CONFIG_PACKAGE_python-base-src is not set # CONFIG_PACKAGE_python-bcrypt is not set # CONFIG_PACKAGE_python-bcrypt-src is not set # CONFIG_PACKAGE_python-botocore is not set # CONFIG_PACKAGE_python-botocore-src is not set # CONFIG_PACKAGE_python-certifi is not set # CONFIG_PACKAGE_python-certifi-src is not set # CONFIG_PACKAGE_python-cffi is not set # CONFIG_PACKAGE_python-cffi-src is not set # CONFIG_PACKAGE_python-chardet is not set # CONFIG_PACKAGE_python-chardet-src is not set # CONFIG_PACKAGE_python-codecs is not set # CONFIG_PACKAGE_python-codecs-src is not set # CONFIG_PACKAGE_python-colorama is not set # CONFIG_PACKAGE_python-colorama-src is not set # CONFIG_PACKAGE_python-compiler is not set # CONFIG_PACKAGE_python-compiler-src is not set # CONFIG_PACKAGE_python-constantly is not set # CONFIG_PACKAGE_python-constantly-src is not set # CONFIG_PACKAGE_python-crcmod is not set # CONFIG_PACKAGE_python-crypto is not set # CONFIG_PACKAGE_python-crypto-src is not set # CONFIG_PACKAGE_python-cryptodome is not set # CONFIG_PACKAGE_python-cryptodome-src is not set # CONFIG_PACKAGE_python-cryptodomex is not set # CONFIG_PACKAGE_python-cryptodomex-src is not set # CONFIG_PACKAGE_python-cryptography is not set # CONFIG_PACKAGE_python-cryptography-src is not set # CONFIG_PACKAGE_python-ctypes is not set # CONFIG_PACKAGE_python-ctypes-src is not set # CONFIG_PACKAGE_python-curl is not set # CONFIG_PACKAGE_python-curl-src is not set # CONFIG_PACKAGE_python-dateutil is not set # CONFIG_PACKAGE_python-dateutil-src is not set # CONFIG_PACKAGE_python-db is not set # CONFIG_PACKAGE_python-db-src is not set # CONFIG_PACKAGE_python-decimal is not set # CONFIG_PACKAGE_python-decimal-src is not set # CONFIG_PACKAGE_python-defusedxml is not set # CONFIG_PACKAGE_python-defusedxml-src is not set # CONFIG_PACKAGE_python-dev is not set # CONFIG_PACKAGE_python-dev-src is not set # CONFIG_PACKAGE_python-distutils is not set # CONFIG_PACKAGE_python-distutils-src is not set # CONFIG_PACKAGE_python-django1 is not set # CONFIG_PACKAGE_python-django1-common is not set # CONFIG_PACKAGE_python-django1-src is not set # CONFIG_PACKAGE_python-dns is not set # CONFIG_PACKAGE_python-dns-src is not set # CONFIG_PACKAGE_python-docutils is not set # CONFIG_PACKAGE_python-docutils-src is not set # CONFIG_PACKAGE_python-dpkt is not set # CONFIG_PACKAGE_python-egenix-mx-base is not set # CONFIG_PACKAGE_python-egenix-mx-base-src is not set # CONFIG_PACKAGE_python-email is not set # CONFIG_PACKAGE_python-email-src is not set # CONFIG_PACKAGE_python-enum34 is not set # CONFIG_PACKAGE_python-enum34-src is not set # CONFIG_PACKAGE_python-et_xmlfile is not set # CONFIG_PACKAGE_python-et_xmlfile-src is not set # CONFIG_PACKAGE_python-evdev is not set # CONFIG_PACKAGE_python-evdev-src is not set # CONFIG_PACKAGE_python-flup is not set # CONFIG_PACKAGE_python-flup-src is not set # CONFIG_PACKAGE_python-futures is not set # CONFIG_PACKAGE_python-futures-src is not set # CONFIG_PACKAGE_python-gdbm is not set # CONFIG_PACKAGE_python-gdbm-src is not set # CONFIG_PACKAGE_python-gmpy2 is not set # CONFIG_PACKAGE_python-gmpy2-src is not set # CONFIG_PACKAGE_python-gnupg is not set # CONFIG_PACKAGE_python-gunicorn is not set # CONFIG_PACKAGE_python-gunicorn-src is not set # CONFIG_PACKAGE_python-hyperlink is not set # CONFIG_PACKAGE_python-hyperlink-src is not set # CONFIG_PACKAGE_python-idna is not set # CONFIG_PACKAGE_python-idna-src is not set # CONFIG_PACKAGE_python-incremental is not set # CONFIG_PACKAGE_python-incremental-src is not set # CONFIG_PACKAGE_python-ipaddress is not set # CONFIG_PACKAGE_python-ipaddress-src is not set # CONFIG_PACKAGE_python-jdcal is not set # CONFIG_PACKAGE_python-jdcal-src is not set # CONFIG_PACKAGE_python-jmespath is not set # CONFIG_PACKAGE_python-jmespath-src is not set # CONFIG_PACKAGE_python-ldap is not set # CONFIG_PACKAGE_python-lib2to3 is not set # CONFIG_PACKAGE_python-lib2to3-src is not set # CONFIG_PACKAGE_python-libmodbus is not set # CONFIG_PACKAGE_python-light is not set  # # Configuration # # CONFIG_PYTHON_BLUETOOTH_SUPPORT is not set # CONFIG_PACKAGE_python-light-src is not set # CONFIG_PACKAGE_python-logging is not set # CONFIG_PACKAGE_python-logging-src is not set # CONFIG_PACKAGE_python-lxml is not set # CONFIG_PACKAGE_python-lxml-src is not set # CONFIG_PACKAGE_python-multiprocessing is not set # CONFIG_PACKAGE_python-multiprocessing-src is not set # CONFIG_PACKAGE_python-mysqlclient is not set # CONFIG_PACKAGE_python-mysqlclient-src is not set # CONFIG_PACKAGE_python-ncurses is not set # CONFIG_PACKAGE_python-ncurses-src is not set # CONFIG_PACKAGE_python-oauthlib is not set # CONFIG_PACKAGE_python-oauthlib-src is not set # CONFIG_PACKAGE_python-openpyxl is not set # CONFIG_PACKAGE_python-openpyxl-src is not set # CONFIG_PACKAGE_python-openssl is not set # CONFIG_PACKAGE_python-openssl-src is not set # CONFIG_PACKAGE_python-parsley is not set # CONFIG_PACKAGE_python-parsley-src is not set # CONFIG_PACKAGE_python-passlib is not set # CONFIG_PACKAGE_python-passlib-src is not set # CONFIG_PACKAGE_python-pcapy is not set # CONFIG_PACKAGE_python-pillow is not set # CONFIG_PACKAGE_python-pillow-src is not set # CONFIG_PACKAGE_python-pip is not set # CONFIG_PACKAGE_python-pip-conf is not set # CONFIG_PACKAGE_python-pip-src is not set # CONFIG_PACKAGE_python-pkg-resources is not set # CONFIG_PACKAGE_python-pkg-resources-src is not set # CONFIG_PACKAGE_python-ply is not set # CONFIG_PACKAGE_python-ply-src is not set # CONFIG_PACKAGE_python-psycopg2 is not set # CONFIG_PACKAGE_python-pyasn1 is not set # CONFIG_PACKAGE_python-pyasn1-modules is not set # CONFIG_PACKAGE_python-pyasn1-modules-src is not set # CONFIG_PACKAGE_python-pyasn1-src is not set # CONFIG_PACKAGE_python-pycparser is not set # CONFIG_PACKAGE_python-pycparser-src is not set # CONFIG_PACKAGE_python-pydoc is not set # CONFIG_PACKAGE_python-pydoc-src is not set # CONFIG_PACKAGE_python-pyjwt is not set # CONFIG_PACKAGE_python-pyjwt-src is not set # CONFIG_PACKAGE_python-pyodbc is not set # CONFIG_PACKAGE_python-pyopenssl is not set # CONFIG_PACKAGE_python-pyopenssl-src is not set # CONFIG_PACKAGE_python-pyptlib is not set # CONFIG_PACKAGE_python-pyptlib-src is not set # CONFIG_PACKAGE_python-pyserial is not set # CONFIG_PACKAGE_python-pyserial-src is not set # CONFIG_PACKAGE_python-pytz is not set # CONFIG_PACKAGE_python-pytz-src is not set # CONFIG_PACKAGE_python-qrcode is not set # CONFIG_PACKAGE_python-qrcode-src is not set # CONFIG_PACKAGE_python-rcssmin is not set # CONFIG_PACKAGE_python-rcssmin-src is not set # CONFIG_PACKAGE_python-requests is not set # CONFIG_PACKAGE_python-requests-oauthlib is not set # CONFIG_PACKAGE_python-requests-oauthlib-src is not set # CONFIG_PACKAGE_python-requests-src is not set # CONFIG_PACKAGE_python-rsa is not set # CONFIG_PACKAGE_python-rsa-src is not set # CONFIG_PACKAGE_python-ruamel-yaml is not set # CONFIG_PACKAGE_python-ruamel-yaml-src is not set # CONFIG_PACKAGE_python-s3transfer is not set # CONFIG_PACKAGE_python-s3transfer-src is not set # CONFIG_PACKAGE_python-service-identity is not set # CONFIG_PACKAGE_python-service-identity-src is not set # CONFIG_PACKAGE_python-setuptools is not set # CONFIG_PACKAGE_python-setuptools-src is not set # CONFIG_PACKAGE_python-simplejson is not set # CONFIG_PACKAGE_python-simplejson-src is not set # CONFIG_PACKAGE_python-six is not set # CONFIG_PACKAGE_python-six-src is not set # CONFIG_PACKAGE_python-smbus is not set # CONFIG_PACKAGE_python-sqlite3 is not set # CONFIG_PACKAGE_python-sqlite3-src is not set # CONFIG_PACKAGE_python-text-unidecode is not set # CONFIG_PACKAGE_python-text-unidecode-src is not set # CONFIG_PACKAGE_python-twisted is not set # CONFIG_PACKAGE_python-twisted-src is not set # CONFIG_PACKAGE_python-txsocksx is not set # CONFIG_PACKAGE_python-txsocksx-src is not set # CONFIG_PACKAGE_python-unittest is not set # CONFIG_PACKAGE_python-unittest-src is not set # CONFIG_PACKAGE_python-urllib3 is not set # CONFIG_PACKAGE_python-urllib3-src is not set # CONFIG_PACKAGE_python-vobject is not set # CONFIG_PACKAGE_python-vobject-src is not set # CONFIG_PACKAGE_python-voluptuous is not set # CONFIG_PACKAGE_python-voluptuous-src is not set # CONFIG_PACKAGE_python-xml is not set # CONFIG_PACKAGE_python-xml-src is not set # CONFIG_PACKAGE_python-yaml is not set # CONFIG_PACKAGE_python-yaml-src is not set # CONFIG_PACKAGE_python-zope-interface is not set # CONFIG_PACKAGE_python-zope-interface-src is not set # CONFIG_PACKAGE_python3 is not set # CONFIG_PACKAGE_python3-aiohttp is not set # CONFIG_PACKAGE_python3-aiohttp-cors is not set # CONFIG_PACKAGE_python3-aiohttp-cors-src is not set # CONFIG_PACKAGE_python3-aiohttp-src is not set # CONFIG_PACKAGE_python3-appdirs is not set # CONFIG_PACKAGE_python3-appdirs-src is not set # CONFIG_PACKAGE_python3-asgiref is not set # CONFIG_PACKAGE_python3-asgiref-src is not set # CONFIG_PACKAGE_python3-asn1crypto is not set # CONFIG_PACKAGE_python3-asn1crypto-src is not set # CONFIG_PACKAGE_python3-astral is not set # CONFIG_PACKAGE_python3-astral-src is not set # CONFIG_PACKAGE_python3-async-timeout is not set # CONFIG_PACKAGE_python3-async-timeout-src is not set # CONFIG_PACKAGE_python3-asyncio is not set # CONFIG_PACKAGE_python3-asyncio-src is not set # CONFIG_PACKAGE_python3-atomicwrites is not set # CONFIG_PACKAGE_python3-atomicwrites-src is not set # CONFIG_PACKAGE_python3-attrs is not set # CONFIG_PACKAGE_python3-attrs-src is not set # CONFIG_PACKAGE_python3-automat is not set # CONFIG_PACKAGE_python3-automat-src is not set # CONFIG_PACKAGE_python3-awscli is not set # CONFIG_PACKAGE_python3-awscli-src is not set # CONFIG_PACKAGE_python3-base is not set # CONFIG_PACKAGE_python3-base-src is not set # CONFIG_PACKAGE_python3-bcrypt is not set # CONFIG_PACKAGE_python3-bcrypt-src is not set # CONFIG_PACKAGE_python3-boto3 is not set # CONFIG_PACKAGE_python3-boto3-src is not set # CONFIG_PACKAGE_python3-botocore is not set # CONFIG_PACKAGE_python3-botocore-src is not set # CONFIG_PACKAGE_python3-bottle is not set # CONFIG_PACKAGE_python3-bottle-src is not set # CONFIG_PACKAGE_python3-cachelib is not set # CONFIG_PACKAGE_python3-cachelib-src is not set # CONFIG_PACKAGE_python3-cachetools is not set # CONFIG_PACKAGE_python3-cachetools-src is not set # CONFIG_PACKAGE_python3-certifi is not set # CONFIG_PACKAGE_python3-certifi-src is not set # CONFIG_PACKAGE_python3-cffi is not set # CONFIG_PACKAGE_python3-cffi-src is not set # CONFIG_PACKAGE_python3-cgi is not set # CONFIG_PACKAGE_python3-cgi-src is not set # CONFIG_PACKAGE_python3-cgitb is not set # CONFIG_PACKAGE_python3-cgitb-src is not set # CONFIG_PACKAGE_python3-chardet is not set # CONFIG_PACKAGE_python3-chardet-src is not set # CONFIG_PACKAGE_python3-click is not set # CONFIG_PACKAGE_python3-click-log is not set # CONFIG_PACKAGE_python3-click-log-src is not set # CONFIG_PACKAGE_python3-click-src is not set # CONFIG_PACKAGE_python3-codecs is not set # CONFIG_PACKAGE_python3-codecs-src is not set # CONFIG_PACKAGE_python3-colorama is not set # CONFIG_PACKAGE_python3-colorama-src is not set # CONFIG_PACKAGE_python3-constantly is not set # CONFIG_PACKAGE_python3-constantly-src is not set # CONFIG_PACKAGE_python3-contextlib2 is not set # CONFIG_PACKAGE_python3-contextlib2-src is not set # CONFIG_PACKAGE_python3-crypto is not set # CONFIG_PACKAGE_python3-crypto-src is not set # CONFIG_PACKAGE_python3-cryptodome is not set # CONFIG_PACKAGE_python3-cryptodome-src is not set # CONFIG_PACKAGE_python3-cryptodomex is not set # CONFIG_PACKAGE_python3-cryptodomex-src is not set # CONFIG_PACKAGE_python3-cryptography is not set # CONFIG_PACKAGE_python3-cryptography-src is not set # CONFIG_PACKAGE_python3-ctypes is not set # CONFIG_PACKAGE_python3-ctypes-src is not set # CONFIG_PACKAGE_python3-curl is not set # CONFIG_PACKAGE_python3-curl-src is not set # CONFIG_PACKAGE_python3-dateutil is not set # CONFIG_PACKAGE_python3-dateutil-src is not set # CONFIG_PACKAGE_python3-dbm is not set # CONFIG_PACKAGE_python3-dbm-src is not set # CONFIG_PACKAGE_python3-decimal is not set # CONFIG_PACKAGE_python3-decimal-src is not set # CONFIG_PACKAGE_python3-decorator is not set # CONFIG_PACKAGE_python3-decorator-src is not set # CONFIG_PACKAGE_python3-defusedxml is not set # CONFIG_PACKAGE_python3-defusedxml-src is not set # CONFIG_PACKAGE_python3-dev is not set # CONFIG_PACKAGE_python3-dev-src is not set # CONFIG_PACKAGE_python3-distutils is not set # CONFIG_PACKAGE_python3-distutils-src is not set # CONFIG_PACKAGE_python3-django is not set # CONFIG_PACKAGE_python3-django-appconf is not set # CONFIG_PACKAGE_python3-django-appconf-src is not set # CONFIG_PACKAGE_python3-django-compressor is not set # CONFIG_PACKAGE_python3-django-compressor-src is not set # CONFIG_PACKAGE_python3-django-constance is not set # CONFIG_PACKAGE_python3-django-constance-src is not set # CONFIG_PACKAGE_python3-django-cors-headers is not set # CONFIG_PACKAGE_python3-django-cors-headers-src is not set # CONFIG_PACKAGE_python3-django-etesync-journal is not set # CONFIG_PACKAGE_python3-django-etesync-journal-src is not set # CONFIG_PACKAGE_python3-django-formtools is not set # CONFIG_PACKAGE_python3-django-formtools-src is not set # CONFIG_PACKAGE_python3-django-jsonfield is not set # CONFIG_PACKAGE_python3-django-jsonfield-src is not set # CONFIG_PACKAGE_python3-django-picklefield is not set # CONFIG_PACKAGE_python3-django-picklefield-src is not set # CONFIG_PACKAGE_python3-django-postoffice is not set # CONFIG_PACKAGE_python3-django-postoffice-src is not set # CONFIG_PACKAGE_python3-django-ranged-response is not set # CONFIG_PACKAGE_python3-django-ranged-response-src is not set # CONFIG_PACKAGE_python3-django-restframework is not set # CONFIG_PACKAGE_python3-django-restframework-src is not set # CONFIG_PACKAGE_python3-django-restframework39 is not set # CONFIG_PACKAGE_python3-django-restframework39-src is not set # CONFIG_PACKAGE_python3-django-simple-captcha is not set # CONFIG_PACKAGE_python3-django-simple-captcha-src is not set # CONFIG_PACKAGE_python3-django-src is not set # CONFIG_PACKAGE_python3-django-statici18n is not set # CONFIG_PACKAGE_python3-django-statici18n-src is not set # CONFIG_PACKAGE_python3-django-webpack-loader is not set # CONFIG_PACKAGE_python3-django-webpack-loader-src is not set # CONFIG_PACKAGE_python3-django1 is not set # CONFIG_PACKAGE_python3-django1-src is not set # CONFIG_PACKAGE_python3-dns is not set # CONFIG_PACKAGE_python3-dns-src is not set # CONFIG_PACKAGE_python3-docutils is not set # CONFIG_PACKAGE_python3-docutils-src is not set # CONFIG_PACKAGE_python3-drf-nested-routers is not set # CONFIG_PACKAGE_python3-drf-nested-routers-src is not set # CONFIG_PACKAGE_python3-email is not set # CONFIG_PACKAGE_python3-email-src is not set # CONFIG_PACKAGE_python3-et_xmlfile is not set # CONFIG_PACKAGE_python3-et_xmlfile-src is not set # CONFIG_PACKAGE_python3-evdev is not set # CONFIG_PACKAGE_python3-evdev-src is not set # CONFIG_PACKAGE_python3-flask is not set # CONFIG_PACKAGE_python3-flask-login is not set # CONFIG_PACKAGE_python3-flask-login-src is not set # CONFIG_PACKAGE_python3-flask-src is not set # CONFIG_PACKAGE_python3-flup is not set # CONFIG_PACKAGE_python3-flup-src is not set # CONFIG_PACKAGE_python3-gdbm is not set # CONFIG_PACKAGE_python3-gdbm-src is not set # CONFIG_PACKAGE_python3-gmpy2 is not set # CONFIG_PACKAGE_python3-gmpy2-src is not set # CONFIG_PACKAGE_python3-gnupg is not set # CONFIG_PACKAGE_python3-gunicorn is not set # CONFIG_PACKAGE_python3-gunicorn-src is not set # CONFIG_PACKAGE_python3-hyperlink is not set # CONFIG_PACKAGE_python3-hyperlink-src is not set # CONFIG_PACKAGE_python3-idna is not set # CONFIG_PACKAGE_python3-idna-src is not set # CONFIG_PACKAGE_python3-ifaddr is not set # CONFIG_PACKAGE_python3-ifaddr-src is not set # CONFIG_PACKAGE_python3-incremental is not set # CONFIG_PACKAGE_python3-incremental-src is not set # CONFIG_PACKAGE_python3-influxdb is not set # CONFIG_PACKAGE_python3-influxdb-src is not set # CONFIG_PACKAGE_python3-intelhex is not set # CONFIG_PACKAGE_python3-intelhex-src is not set # CONFIG_PACKAGE_python3-itsdangerous is not set # CONFIG_PACKAGE_python3-itsdangerous-src is not set # CONFIG_PACKAGE_python3-jdcal is not set # CONFIG_PACKAGE_python3-jdcal-src is not set # CONFIG_PACKAGE_python3-jinja2 is not set # CONFIG_PACKAGE_python3-jinja2-src is not set # CONFIG_PACKAGE_python3-jmespath is not set # CONFIG_PACKAGE_python3-jmespath-src is not set # CONFIG_PACKAGE_python3-jsonpath-ng is not set # CONFIG_PACKAGE_python3-jsonpath-ng-src is not set # CONFIG_PACKAGE_python3-lib2to3 is not set # CONFIG_PACKAGE_python3-lib2to3-src is not set # CONFIG_PACKAGE_python3-libmodbus is not set # CONFIG_PACKAGE_python3-light is not set  # # Configuration # # CONFIG_PYTHON3_BLUETOOTH_SUPPORT is not set # CONFIG_PACKAGE_python3-light-src is not set # CONFIG_PACKAGE_python3-logging is not set # CONFIG_PACKAGE_python3-logging-src is not set # CONFIG_PACKAGE_python3-lxml is not set # CONFIG_PACKAGE_python3-lxml-src is not set # CONFIG_PACKAGE_python3-lzma is not set # CONFIG_PACKAGE_python3-lzma-src is not set # CONFIG_PACKAGE_python3-markdown is not set # CONFIG_PACKAGE_python3-markdown-src is not set # CONFIG_PACKAGE_python3-markupsafe is not set # CONFIG_PACKAGE_python3-markupsafe-src is not set # CONFIG_PACKAGE_python3-maxminddb is not set # CONFIG_PACKAGE_python3-maxminddb-src is not set # CONFIG_PACKAGE_python3-more-itertools is not set # CONFIG_PACKAGE_python3-more-itertools-src is not set # CONFIG_PACKAGE_python3-multidict is not set # CONFIG_PACKAGE_python3-multidict-src is not set # CONFIG_PACKAGE_python3-multiprocessing is not set # CONFIG_PACKAGE_python3-multiprocessing-src is not set # CONFIG_PACKAGE_python3-mysqlclient is not set # CONFIG_PACKAGE_python3-mysqlclient-src is not set # CONFIG_PACKAGE_python3-ncurses is not set # CONFIG_PACKAGE_python3-ncurses-src is not set # CONFIG_PACKAGE_python3-netdisco is not set # CONFIG_PACKAGE_python3-netdisco-src is not set # CONFIG_PACKAGE_python3-netifaces is not set # CONFIG_PACKAGE_python3-netifaces-src is not set # CONFIG_PACKAGE_python3-oauthlib is not set # CONFIG_PACKAGE_python3-oauthlib-src is not set # CONFIG_PACKAGE_python3-openpyxl is not set # CONFIG_PACKAGE_python3-openpyxl-src is not set # CONFIG_PACKAGE_python3-openssl is not set # CONFIG_PACKAGE_python3-openssl-src is not set # CONFIG_PACKAGE_python3-paho-mqtt is not set # CONFIG_PACKAGE_python3-paho-mqtt-src is not set # CONFIG_PACKAGE_python3-parsley is not set # CONFIG_PACKAGE_python3-parsley-src is not set # CONFIG_PACKAGE_python3-passlib is not set # CONFIG_PACKAGE_python3-passlib-src is not set # CONFIG_PACKAGE_python3-pillow is not set # CONFIG_PACKAGE_python3-pillow-src is not set # CONFIG_PACKAGE_python3-pip is not set # CONFIG_PACKAGE_python3-pip-src is not set # CONFIG_PACKAGE_python3-pkg-resources is not set # CONFIG_PACKAGE_python3-pkg-resources-src is not set # CONFIG_PACKAGE_python3-ply is not set # CONFIG_PACKAGE_python3-ply-src is not set # CONFIG_PACKAGE_python3-py is not set # CONFIG_PACKAGE_python3-py-src is not set # CONFIG_PACKAGE_python3-pyasn1 is not set # CONFIG_PACKAGE_python3-pyasn1-modules is not set # CONFIG_PACKAGE_python3-pyasn1-modules-src is not set # CONFIG_PACKAGE_python3-pyasn1-src is not set # CONFIG_PACKAGE_python3-pycparser is not set # CONFIG_PACKAGE_python3-pycparser-src is not set # CONFIG_PACKAGE_python3-pydoc is not set # CONFIG_PACKAGE_python3-pydoc-src is not set # CONFIG_PACKAGE_python3-pyjwt is not set # CONFIG_PACKAGE_python3-pyjwt-src is not set # CONFIG_PACKAGE_python3-pymysql is not set # CONFIG_PACKAGE_python3-pymysql-src is not set # CONFIG_PACKAGE_python3-pyodbc is not set # CONFIG_PACKAGE_python3-pyopenssl is not set # CONFIG_PACKAGE_python3-pyopenssl-src is not set # CONFIG_PACKAGE_python3-pyotp is not set # CONFIG_PACKAGE_python3-pyotp-src is not set # CONFIG_PACKAGE_python3-pyparsing is not set # CONFIG_PACKAGE_python3-pyparsing-src is not set # CONFIG_PACKAGE_python3-pyroute2 is not set # CONFIG_PACKAGE_python3-pyroute2-src is not set # CONFIG_PACKAGE_python3-pyrsistent is not set # CONFIG_PACKAGE_python3-pyrsistent-src is not set # CONFIG_PACKAGE_python3-pyserial is not set # CONFIG_PACKAGE_python3-pyserial-src is not set # CONFIG_PACKAGE_python3-pytz is not set # CONFIG_PACKAGE_python3-pytz-src is not set # CONFIG_PACKAGE_python3-qrcode is not set # CONFIG_PACKAGE_python3-qrcode-src is not set # CONFIG_PACKAGE_python3-rcssmin is not set # CONFIG_PACKAGE_python3-rcssmin-src is not set # CONFIG_PACKAGE_python3-requests is not set # CONFIG_PACKAGE_python3-requests-oauthlib is not set # CONFIG_PACKAGE_python3-requests-oauthlib-src is not set # CONFIG_PACKAGE_python3-requests-src is not set # CONFIG_PACKAGE_python3-rsa is not set # CONFIG_PACKAGE_python3-rsa-src is not set # CONFIG_PACKAGE_python3-ruamel-yaml is not set # CONFIG_PACKAGE_python3-ruamel-yaml-src is not set # CONFIG_PACKAGE_python3-s3transfer is not set # CONFIG_PACKAGE_python3-s3transfer-src is not set # CONFIG_PACKAGE_python3-schedule is not set # CONFIG_PACKAGE_python3-schedule-src is not set # CONFIG_PACKAGE_python3-schema is not set # CONFIG_PACKAGE_python3-schema-src is not set # CONFIG_PACKAGE_python3-seafile-ccnet is not set # CONFIG_PACKAGE_python3-seafile-ccnet-src is not set # CONFIG_PACKAGE_python3-seafile-server is not set # CONFIG_PACKAGE_python3-seafile-server-src is not set # CONFIG_PACKAGE_python3-searpc is not set # CONFIG_PACKAGE_python3-searpc-src is not set # CONFIG_PACKAGE_python3-sentry-sdk is not set # CONFIG_PACKAGE_python3-sentry-sdk-src is not set # CONFIG_PACKAGE_python3-service-identity is not set # CONFIG_PACKAGE_python3-service-identity-src is not set # CONFIG_PACKAGE_python3-setuptools is not set # CONFIG_PACKAGE_python3-setuptools-src is not set # CONFIG_PACKAGE_python3-simplejson is not set # CONFIG_PACKAGE_python3-simplejson-src is not set # CONFIG_PACKAGE_python3-six is not set # CONFIG_PACKAGE_python3-six-src is not set # CONFIG_PACKAGE_python3-slugify is not set # CONFIG_PACKAGE_python3-slugify-src is not set # CONFIG_PACKAGE_python3-smbus is not set # CONFIG_PACKAGE_python3-speedtest-cli is not set # CONFIG_PACKAGE_python3-speedtest-cli-src is not set # CONFIG_PACKAGE_python3-sqlalchemy is not set # CONFIG_PACKAGE_python3-sqlalchemy-src is not set # CONFIG_PACKAGE_python3-sqlite3 is not set # CONFIG_PACKAGE_python3-sqlite3-src is not set # CONFIG_PACKAGE_python3-sqlparse is not set # CONFIG_PACKAGE_python3-sqlparse-src is not set # CONFIG_PACKAGE_python3-stem is not set # CONFIG_PACKAGE_python3-stem-src is not set # CONFIG_PACKAGE_python3-text-unidecode is not set # CONFIG_PACKAGE_python3-text-unidecode-src is not set # CONFIG_PACKAGE_python3-twisted is not set # CONFIG_PACKAGE_python3-twisted-src is not set # CONFIG_PACKAGE_python3-unidecode is not set # CONFIG_PACKAGE_python3-unidecode-src is not set # CONFIG_PACKAGE_python3-unittest is not set # CONFIG_PACKAGE_python3-unittest-src is not set # CONFIG_PACKAGE_python3-urllib is not set # CONFIG_PACKAGE_python3-urllib-src is not set # CONFIG_PACKAGE_python3-urllib3 is not set # CONFIG_PACKAGE_python3-urllib3-src is not set # CONFIG_PACKAGE_python3-vobject is not set # CONFIG_PACKAGE_python3-vobject-src is not set # CONFIG_PACKAGE_python3-voluptuous is not set # CONFIG_PACKAGE_python3-voluptuous-serialize is not set # CONFIG_PACKAGE_python3-voluptuous-serialize-src is not set # CONFIG_PACKAGE_python3-voluptuous-src is not set # CONFIG_PACKAGE_python3-wcwidth is not set # CONFIG_PACKAGE_python3-wcwidth-src is not set # CONFIG_PACKAGE_python3-werkzeug is not set # CONFIG_PACKAGE_python3-werkzeug-src is not set # CONFIG_PACKAGE_python3-xml is not set # CONFIG_PACKAGE_python3-xml-src is not set # CONFIG_PACKAGE_python3-xmltodict is not set # CONFIG_PACKAGE_python3-xmltodict-src is not set # CONFIG_PACKAGE_python3-yaml is not set # CONFIG_PACKAGE_python3-yaml-src is not set # CONFIG_PACKAGE_python3-yarl is not set # CONFIG_PACKAGE_python3-yarl-src is not set # CONFIG_PACKAGE_python3-zeroconf is not set # CONFIG_PACKAGE_python3-zeroconf-src is not set # CONFIG_PACKAGE_python3-zipp is not set # CONFIG_PACKAGE_python3-zipp-src is not set # CONFIG_PACKAGE_python3-zope-interface is not set # CONFIG_PACKAGE_python3-zope-interface-src is not set  # # Ruby # # CONFIG_PACKAGE_ruby is not set  # # Tcl # # CONFIG_PACKAGE_vala is not set  # # Libraries #  # # Compression # # CONFIG_PACKAGE_libbz2 is not set # CONFIG_PACKAGE_liblzma is not set # CONFIG_PACKAGE_libunrar is not set # CONFIG_PACKAGE_libzstd is not set CONFIG_ZSTD_OPTIMIZE_O3=y  # # Filesystem # # CONFIG_PACKAGE_libacl is not set # CONFIG_PACKAGE_libattr is not set # CONFIG_PACKAGE_libfuse is not set # CONFIG_PACKAGE_libow is not set # CONFIG_PACKAGE_libow-capi is not set # CONFIG_PACKAGE_libsysfs is not set  # # Firewall # # CONFIG_PACKAGE_libfko is not set CONFIG_PACKAGE_libip4tc=y CONFIG_PACKAGE_libip6tc=y CONFIG_PACKAGE_libxtables=y # CONFIG_PACKAGE_libxtables-nft is not set  # # Instant Messaging # # CONFIG_PACKAGE_quasselc is not set  # # IoT # # CONFIG_PACKAGE_libupm is not set # CONFIG_PACKAGE_libupm-a110x is not set # CONFIG_PACKAGE_libupm-ad8232 is not set # CONFIG_PACKAGE_libupm-adafruitss is not set # CONFIG_PACKAGE_libupm-adc121c021 is not set # CONFIG_PACKAGE_libupm-adis16448 is not set # CONFIG_PACKAGE_libupm-adxl335 is not set # CONFIG_PACKAGE_libupm-adxl345 is not set # CONFIG_PACKAGE_libupm-am2315 is not set # CONFIG_PACKAGE_libupm-apds9002 is not set # CONFIG_PACKAGE_libupm-at42qt1070 is not set # CONFIG_PACKAGE_libupm-biss0001 is not set # CONFIG_PACKAGE_libupm-bmpx8x is not set # CONFIG_PACKAGE_libupm-buzzer is not set # CONFIG_PACKAGE_libupm-cjq4435 is not set # CONFIG_PACKAGE_libupm-ds1307 is not set # CONFIG_PACKAGE_libupm-ecs1030 is not set # CONFIG_PACKAGE_libupm-enc03r is not set # CONFIG_PACKAGE_libupm-flex is not set # CONFIG_PACKAGE_libupm-gas is not set # CONFIG_PACKAGE_libupm-gp2y0a is not set # CONFIG_PACKAGE_libupm-grove is not set # CONFIG_PACKAGE_libupm-grovecircularled is not set # CONFIG_PACKAGE_libupm-grovecollision is not set # CONFIG_PACKAGE_libupm-groveehr is not set # CONFIG_PACKAGE_libupm-groveeldriver is not set # CONFIG_PACKAGE_libupm-groveelectromagnet is not set # CONFIG_PACKAGE_libupm-groveemg is not set # CONFIG_PACKAGE_libupm-grovegprs is not set # CONFIG_PACKAGE_libupm-grovegsr is not set # CONFIG_PACKAGE_libupm-grovelinefinder is not set # CONFIG_PACKAGE_libupm-grovemd is not set # CONFIG_PACKAGE_libupm-grovemoisture is not set # CONFIG_PACKAGE_libupm-groveo2 is not set # CONFIG_PACKAGE_libupm-grovescam is not set # CONFIG_PACKAGE_libupm-grovespeaker is not set # CONFIG_PACKAGE_libupm-grovevdiv is not set # CONFIG_PACKAGE_libupm-grovewater is not set # CONFIG_PACKAGE_libupm-grovewfs is not set # CONFIG_PACKAGE_libupm-guvas12d is not set # CONFIG_PACKAGE_libupm-h3lis331dl is not set # CONFIG_PACKAGE_libupm-hcsr04 is not set # CONFIG_PACKAGE_libupm-hm11 is not set # CONFIG_PACKAGE_libupm-hmc5883l is not set # CONFIG_PACKAGE_libupm-hmtrp is not set # CONFIG_PACKAGE_libupm-hp20x is not set # CONFIG_PACKAGE_libupm-ht9170 is not set # CONFIG_PACKAGE_libupm-htu21d is not set # CONFIG_PACKAGE_libupm-hx711 is not set # CONFIG_PACKAGE_libupm-i2clcd is not set # CONFIG_PACKAGE_libupm-ina132 is not set # CONFIG_PACKAGE_libupm-isd1820 is not set # CONFIG_PACKAGE_libupm-itg3200 is not set # CONFIG_PACKAGE_libupm-joystick12 is not set # CONFIG_PACKAGE_libupm-l298 is not set # CONFIG_PACKAGE_libupm-ldt0028 is not set # CONFIG_PACKAGE_libupm-lm35 is not set # CONFIG_PACKAGE_libupm-lol is not set # CONFIG_PACKAGE_libupm-loudness is not set # CONFIG_PACKAGE_libupm-lpd8806 is not set # CONFIG_PACKAGE_libupm-lsm303 is not set # CONFIG_PACKAGE_libupm-lsm9ds0 is not set # CONFIG_PACKAGE_libupm-m24lr64e is not set # CONFIG_PACKAGE_libupm-max31723 is not set # CONFIG_PACKAGE_libupm-max31855 is not set # CONFIG_PACKAGE_libupm-max44000 is not set # CONFIG_PACKAGE_libupm-max5487 is not set # CONFIG_PACKAGE_libupm-maxds3231m is not set # CONFIG_PACKAGE_libupm-maxsonarez is not set # CONFIG_PACKAGE_libupm-mg811 is not set # CONFIG_PACKAGE_libupm-mhz16 is not set # CONFIG_PACKAGE_libupm-mic is not set # CONFIG_PACKAGE_libupm-mlx90614 is not set # CONFIG_PACKAGE_libupm-mma7455 is not set # CONFIG_PACKAGE_libupm-mma7660 is not set # CONFIG_PACKAGE_libupm-mpl3115a2 is not set # CONFIG_PACKAGE_libupm-mpr121 is not set # CONFIG_PACKAGE_libupm-mpu9150 is not set # CONFIG_PACKAGE_libupm-mq303a is not set # CONFIG_PACKAGE_libupm-my9221 is not set # CONFIG_PACKAGE_libupm-nrf24l01 is not set # CONFIG_PACKAGE_libupm-nrf8001 is not set # CONFIG_PACKAGE_libupm-nunchuck is not set # CONFIG_PACKAGE_libupm-otp538u is not set # CONFIG_PACKAGE_libupm-pn532 is not set # CONFIG_PACKAGE_libupm-ppd42ns is not set # CONFIG_PACKAGE_libupm-pulsensor is not set # CONFIG_PACKAGE_libupm-rfr359f is not set # CONFIG_PACKAGE_libupm-rgbringcoder is not set # CONFIG_PACKAGE_libupm-rotaryencoder is not set # CONFIG_PACKAGE_libupm-rpr220 is not set # CONFIG_PACKAGE_libupm-servo is not set # CONFIG_PACKAGE_libupm-si114x is not set # CONFIG_PACKAGE_libupm-sm130 is not set # CONFIG_PACKAGE_libupm-st7735 is not set # CONFIG_PACKAGE_libupm-stepmotor is not set # CONFIG_PACKAGE_libupm-sx6119 is not set # CONFIG_PACKAGE_libupm-ta12200 is not set # CONFIG_PACKAGE_libupm-tcs3414cs is not set # CONFIG_PACKAGE_libupm-th02 is not set # CONFIG_PACKAGE_libupm-tm1637 is not set # CONFIG_PACKAGE_libupm-tsl2561 is not set # CONFIG_PACKAGE_libupm-ttp223 is not set # CONFIG_PACKAGE_libupm-ublox6 is not set # CONFIG_PACKAGE_libupm-uln200xa is not set # CONFIG_PACKAGE_libupm-waterlevel is not set # CONFIG_PACKAGE_libupm-wheelencoder is not set # CONFIG_PACKAGE_libupm-wt5001 is not set # CONFIG_PACKAGE_libupm-yg1006 is not set # CONFIG_PACKAGE_libupm-zfm20 is not set  # # Languages # # CONFIG_PACKAGE_libyaml is not set  # # Networking # # CONFIG_PACKAGE_libdcwproto is not set # CONFIG_PACKAGE_libdcwsocket is not set # CONFIG_PACKAGE_libsctp is not set # CONFIG_PACKAGE_libuhttpd-mbedtls is not set # CONFIG_PACKAGE_libuhttpd-nossl is not set # CONFIG_PACKAGE_libuhttpd-openssl is not set # CONFIG_PACKAGE_libuhttpd-wolfssl is not set # CONFIG_PACKAGE_libunbound is not set # CONFIG_PACKAGE_libunbound-heavy is not set # CONFIG_PACKAGE_libuwsc-mbedtls is not set # CONFIG_PACKAGE_libuwsc-nossl is not set # CONFIG_PACKAGE_libuwsc-openssl is not set # CONFIG_PACKAGE_libuwsc-wolfssl is not set  # # Qt5 # # CONFIG_PACKAGE_qt5-core is not set # CONFIG_PACKAGE_qt5-network is not set # CONFIG_PACKAGE_qt5-xml is not set  # # SSL # # CONFIG_PACKAGE_libgnutls is not set # CONFIG_PACKAGE_libmbedtls is not set CONFIG_PACKAGE_libopenssl=y  # # Build Options # CONFIG_OPENSSL_OPTIMIZE_SPEED=y CONFIG_OPENSSL_WITH_ASM=y CONFIG_OPENSSL_WITH_DEPRECATED=y # CONFIG_OPENSSL_NO_DEPRECATED is not set # CONFIG_OPENSSL_WITH_ERROR_MESSAGES is not set  # # Protocol Support # CONFIG_OPENSSL_WITH_TLS13=y # CONFIG_OPENSSL_WITH_DTLS is not set # CONFIG_OPENSSL_WITH_NPN is not set CONFIG_OPENSSL_WITH_SRP=y CONFIG_OPENSSL_WITH_CMS=y  # # Algorithm Selection # # CONFIG_OPENSSL_WITH_EC2M is not set CONFIG_OPENSSL_WITH_CHACHA_POLY1305=y CONFIG_OPENSSL_PREFER_CHACHA_OVER_GCM=y CONFIG_OPENSSL_WITH_PSK=y  # # Less commonly used build options # # CONFIG_OPENSSL_WITH_ARIA is not set # CONFIG_OPENSSL_WITH_CAMELLIA is not set # CONFIG_OPENSSL_WITH_IDEA is not set # CONFIG_OPENSSL_WITH_SEED is not set # CONFIG_OPENSSL_WITH_SM234 is not set # CONFIG_OPENSSL_WITH_BLAKE2 is not set # CONFIG_OPENSSL_WITH_MDC2 is not set # CONFIG_OPENSSL_WITH_WHIRLPOOL is not set # CONFIG_OPENSSL_WITH_COMPRESSION is not set # CONFIG_OPENSSL_WITH_RFC3779 is not set  # # Engine/Hardware Support # CONFIG_OPENSSL_ENGINE=y CONFIG_OPENSSL_ENGINE_BUILTIN=y CONFIG_OPENSSL_ENGINE_BUILTIN_DEVCRYPTO=y # CONFIG_OPENSSL_WITH_GOST is not set CONFIG_PACKAGE_libopenssl-conf=y # CONFIG_PACKAGE_libopenssl-devcrypto is not set # CONFIG_PACKAGE_libpolarssl is not set # CONFIG_PACKAGE_libwolfssl is not set  # # Sound # # CONFIG_PACKAGE_liblo is not set  # # database # # CONFIG_PACKAGE_libpq is not set # CONFIG_PACKAGE_libsqlite3 is not set # CONFIG_PACKAGE_pgsqlodbc is not set # CONFIG_PACKAGE_psqlodbca is not set # CONFIG_PACKAGE_psqlodbcw is not set # CONFIG_PACKAGE_tdb is not set # CONFIG_PACKAGE_unixodbc is not set  # # libelektra # # CONFIG_PACKAGE_libelektra-boost is not set # CONFIG_PACKAGE_libelektra-core is not set # CONFIG_PACKAGE_libelektra-cpp is not set # CONFIG_PACKAGE_libelektra-crypto is not set # CONFIG_PACKAGE_libelektra-curlget is not set # CONFIG_PACKAGE_libelektra-dbus is not set # CONFIG_PACKAGE_libelektra-extra is not set # CONFIG_PACKAGE_libelektra-lua is not set # CONFIG_PACKAGE_libelektra-plugins is not set # CONFIG_PACKAGE_libelektra-python2 is not set # CONFIG_PACKAGE_libelektra-python3 is not set # CONFIG_PACKAGE_libelektra-resolvers is not set # CONFIG_PACKAGE_libelektra-xerces is not set # CONFIG_PACKAGE_libelektra-xml is not set # CONFIG_PACKAGE_libelektra-yajl is not set # CONFIG_PACKAGE_libelektra-yamlcpp is not set # CONFIG_PACKAGE_alsa-lib is not set # CONFIG_PACKAGE_argp-standalone is not set # CONFIG_PACKAGE_avro-c is not set # CONFIG_PACKAGE_bind-libs is not set # CONFIG_PACKAGE_bluez-libs is not set # CONFIG_PACKAGE_boost is not set # CONFIG_boost-context-exclude is not set # CONFIG_boost-coroutine-exclude is not set # CONFIG_boost-fiber-exclude is not set # CONFIG_PACKAGE_ccid is not set # CONFIG_PACKAGE_check is not set # CONFIG_PACKAGE_classpath is not set # CONFIG_PACKAGE_classpath-tools is not set # CONFIG_PACKAGE_confuse is not set # CONFIG_PACKAGE_dtndht is not set # CONFIG_PACKAGE_fcgi is not set # CONFIG_PACKAGE_fftw3 is not set # CONFIG_PACKAGE_fftw3f is not set # CONFIG_PACKAGE_getdns is not set # CONFIG_PACKAGE_giflib is not set # CONFIG_PACKAGE_glib2 is not set # CONFIG_PACKAGE_glog is not set # CONFIG_PACKAGE_hidapi is not set # CONFIG_PACKAGE_ibrcommon is not set # CONFIG_PACKAGE_ibrdtn is not set # CONFIG_PACKAGE_icu is not set # CONFIG_PACKAGE_icu-data-tools is not set # CONFIG_PACKAGE_icu-full-data is not set # CONFIG_PACKAGE_jansson is not set # CONFIG_PACKAGE_knot-libs is not set # CONFIG_PACKAGE_knot-libzscanner is not set # CONFIG_PACKAGE_libaio is not set # CONFIG_PACKAGE_libalac is not set # CONFIG_PACKAGE_libantlr3c is not set # CONFIG_PACKAGE_libao is not set # CONFIG_PACKAGE_libapr is not set # CONFIG_PACKAGE_libaprutil is not set # CONFIG_PACKAGE_libarchive is not set # CONFIG_PACKAGE_libarchive-noopenssl is not set # CONFIG_PACKAGE_libartnet is not set # CONFIG_PACKAGE_libasm is not set # CONFIG_PACKAGE_libaudiofile is not set # CONFIG_PACKAGE_libavahi-client is not set # CONFIG_PACKAGE_libavahi-compat-libdnssd is not set # CONFIG_PACKAGE_libavahi-dbus-support is not set # CONFIG_PACKAGE_libavahi-nodbus-support is not set # CONFIG_PACKAGE_libavl is not set # CONFIG_PACKAGE_libbfd is not set # CONFIG_PACKAGE_libblkid is not set CONFIG_PACKAGE_libblobmsg-json=y # CONFIG_PACKAGE_libbsd is not set # CONFIG_PACKAGE_libcanfestival is not set # CONFIG_PACKAGE_libcap is not set # CONFIG_PACKAGE_libcares is not set # CONFIG_PACKAGE_libcharset is not set # CONFIG_PACKAGE_libcoap is not set # CONFIG_PACKAGE_libcomerr is not set # CONFIG_PACKAGE_libconfig is not set # CONFIG_PACKAGE_libcryptopp is not set # CONFIG_PACKAGE_libcurl is not set # CONFIG_PACKAGE_libcxx is not set # CONFIG_PACKAGE_libdaemon is not set # CONFIG_PACKAGE_libdaq is not set # CONFIG_PACKAGE_libdb47 is not set # CONFIG_PACKAGE_libdb47xx is not set # CONFIG_PACKAGE_libdbi is not set # CONFIG_PACKAGE_libdbus is not set # CONFIG_PACKAGE_libdevmapper is not set # CONFIG_PACKAGE_libdmapsharing is not set # CONFIG_PACKAGE_libdnet is not set # CONFIG_PACKAGE_libdouble-conversion is not set # CONFIG_PACKAGE_libdrm is not set # CONFIG_PACKAGE_libdw is not set # CONFIG_PACKAGE_libecdsautil is not set # CONFIG_PACKAGE_libedit is not set CONFIG_PACKAGE_libelf=y # CONFIG_PACKAGE_libesmtp is not set # CONFIG_PACKAGE_libestr is not set # CONFIG_PACKAGE_libev is not set # CONFIG_PACKAGE_libevdev is not set # CONFIG_PACKAGE_libevent2 is not set # CONFIG_PACKAGE_libevent2-core is not set # CONFIG_PACKAGE_libevent2-extra is not set # CONFIG_PACKAGE_libevent2-openssl is not set # CONFIG_PACKAGE_libevent2-pthreads is not set # CONFIG_PACKAGE_libeventlog is not set # CONFIG_PACKAGE_libevhtp is not set # CONFIG_PACKAGE_libexif is not set # CONFIG_PACKAGE_libexpat is not set # CONFIG_PACKAGE_libexslt is not set # CONFIG_PACKAGE_libext2fs is not set # CONFIG_PACKAGE_libextractor is not set # CONFIG_PACKAGE_libf2fs is not set # CONFIG_PACKAGE_libfaad2 is not set # CONFIG_PACKAGE_libfastjson is not set # CONFIG_PACKAGE_libfdisk is not set # CONFIG_PACKAGE_libfdt is not set # CONFIG_PACKAGE_libffi is not set # CONFIG_PACKAGE_libffmpeg-audio-dec is not set # CONFIG_PACKAGE_libffmpeg-custom is not set # CONFIG_PACKAGE_libffmpeg-full is not set # CONFIG_PACKAGE_libffmpeg-mini is not set # CONFIG_PACKAGE_libflac is not set # CONFIG_PACKAGE_libfmt is not set # CONFIG_PACKAGE_libfreetype is not set # CONFIG_PACKAGE_libftdi is not set # CONFIG_PACKAGE_libftdi1 is not set # CONFIG_PACKAGE_libgcrypt is not set # CONFIG_PACKAGE_libgd is not set # CONFIG_PACKAGE_libgdbm is not set # CONFIG_PACKAGE_libgee is not set # CONFIG_PACKAGE_libglpk is not set # CONFIG_PACKAGE_libgmp is not set # CONFIG_PACKAGE_libgnurl is not set # CONFIG_PACKAGE_libgpg-error is not set # CONFIG_PACKAGE_libgphoto2 is not set # CONFIG_PACKAGE_libgpiod is not set # CONFIG_PACKAGE_libgps is not set # CONFIG_PACKAGE_libhamlib is not set # CONFIG_PACKAGE_libhavege is not set # CONFIG_PACKAGE_libhiredis is not set # CONFIG_PACKAGE_libhttp-parser is not set # CONFIG_PACKAGE_libhwloc is not set # CONFIG_PACKAGE_libical is not set # CONFIG_PACKAGE_libiconv is not set # CONFIG_PACKAGE_libiconv-full is not set # CONFIG_PACKAGE_libid3tag is not set # CONFIG_PACKAGE_libidn is not set # CONFIG_PACKAGE_libidn2 is not set # CONFIG_PACKAGE_libiio is not set # CONFIG_PACKAGE_libimobiledevice is not set # CONFIG_PACKAGE_libinotifytools is not set # CONFIG_PACKAGE_libinput is not set # CONFIG_PACKAGE_libintl is not set # CONFIG_PACKAGE_libintl-full is not set # CONFIG_PACKAGE_libiw is not set CONFIG_PACKAGE_libiwinfo=y # CONFIG_PACKAGE_libjpeg is not set CONFIG_PACKAGE_libjson-c=y # CONFIG_PACKAGE_libkeyutils is not set # CONFIG_PACKAGE_libkmod is not set # CONFIG_PACKAGE_libldns is not set # CONFIG_PACKAGE_libltdl is not set CONFIG_PACKAGE_liblua=y # CONFIG_PACKAGE_liblz4 is not set # CONFIG_PACKAGE_liblzo is not set # CONFIG_PACKAGE_libmad is not set # CONFIG_PACKAGE_libmagic is not set # CONFIG_PACKAGE_libmariadb is not set # CONFIG_PACKAGE_libmaxminddb is not set # CONFIG_PACKAGE_libmbim is not set # CONFIG_PACKAGE_libmcrypt is not set # CONFIG_PACKAGE_libmicrohttpd-no-ssl is not set # CONFIG_PACKAGE_libmicrohttpd-ssl is not set # CONFIG_PACKAGE_libmilter-sendmail is not set # CONFIG_PACKAGE_libminiupnpc is not set # CONFIG_PACKAGE_libmms is not set CONFIG_PACKAGE_libmnl=y # CONFIG_PACKAGE_libmodbus is not set # CONFIG_PACKAGE_libmosquitto-nossl is not set # CONFIG_PACKAGE_libmosquitto-ssl is not set # CONFIG_PACKAGE_libmount is not set # CONFIG_PACKAGE_libmpdclient is not set # CONFIG_PACKAGE_libmpeg2 is not set # CONFIG_PACKAGE_libmpg123 is not set # CONFIG_PACKAGE_libmraa is not set # CONFIG_PACKAGE_libnatpmp is not set # CONFIG_PACKAGE_libncurses is not set # CONFIG_PACKAGE_libndpi is not set # CONFIG_PACKAGE_libneon is not set # CONFIG_PACKAGE_libnet-1.2.x is not set # CONFIG_PACKAGE_libnetconf2 is not set # CONFIG_PACKAGE_libnetfilter-acct is not set # CONFIG_PACKAGE_libnetfilter-conntrack is not set # CONFIG_PACKAGE_libnetfilter-cthelper is not set # CONFIG_PACKAGE_libnetfilter-cttimeout is not set # CONFIG_PACKAGE_libnetfilter-log is not set # CONFIG_PACKAGE_libnetfilter-queue is not set # CONFIG_PACKAGE_libnetsnmp is not set # CONFIG_PACKAGE_libnettle is not set # CONFIG_PACKAGE_libnfnetlink is not set # CONFIG_PACKAGE_libnftnl is not set # CONFIG_PACKAGE_libnghttp2 is not set # CONFIG_PACKAGE_libnl is not set # CONFIG_PACKAGE_libnl-core is not set # CONFIG_PACKAGE_libnl-genl is not set # CONFIG_PACKAGE_libnl-nf is not set # CONFIG_PACKAGE_libnl-route is not set CONFIG_PACKAGE_libnl-tiny=y # CONFIG_PACKAGE_libnopoll is not set # CONFIG_PACKAGE_libogg is not set # CONFIG_PACKAGE_liboil is not set # CONFIG_PACKAGE_libopcodes is not set # CONFIG_PACKAGE_libopendkim is not set # CONFIG_PACKAGE_libopenobex is not set # CONFIG_PACKAGE_libopensc is not set # CONFIG_PACKAGE_libopenzwave is not set # CONFIG_PACKAGE_liboping is not set # CONFIG_PACKAGE_libopus is not set # CONFIG_PACKAGE_libout123 is not set # CONFIG_PACKAGE_libowfat is not set # CONFIG_PACKAGE_libp11 is not set # CONFIG_PACKAGE_libpagekite is not set # CONFIG_PACKAGE_libpam is not set # CONFIG_PACKAGE_libpcap is not set # CONFIG_PACKAGE_libpciaccess is not set CONFIG_PACKAGE_libpcre=y # CONFIG_PACKAGE_libpcre16 is not set # CONFIG_PACKAGE_libpcre2 is not set # CONFIG_PACKAGE_libpcre2-16 is not set # CONFIG_PACKAGE_libpcre2-32 is not set # CONFIG_PACKAGE_libpcrecpp is not set # CONFIG_PACKAGE_libpcsclite is not set # CONFIG_PACKAGE_libpkcs11-spy is not set # CONFIG_PACKAGE_libplist is not set # CONFIG_PACKAGE_libplistcxx is not set # CONFIG_PACKAGE_libpng is not set # CONFIG_PACKAGE_libpopt is not set # CONFIG_PACKAGE_libprotobuf-c is not set # CONFIG_PACKAGE_libqmi is not set # CONFIG_PACKAGE_libqrencode is not set # CONFIG_PACKAGE_libradcli is not set # CONFIG_PACKAGE_libreadline is not set # CONFIG_PACKAGE_libredblack is not set # CONFIG_PACKAGE_librouteros is not set # CONFIG_PACKAGE_libroxml is not set # CONFIG_PACKAGE_librpc is not set # CONFIG_PACKAGE_librrd1 is not set # CONFIG_PACKAGE_librtlsdr is not set # CONFIG_PACKAGE_libruby is not set # CONFIG_PACKAGE_libsamplerate is not set # CONFIG_PACKAGE_libsane is not set # CONFIG_PACKAGE_libsasl2 is not set # CONFIG_PACKAGE_libsearpc is not set # CONFIG_PACKAGE_libseccomp is not set # CONFIG_PACKAGE_libsensors is not set # CONFIG_PACKAGE_libshout is not set # CONFIG_PACKAGE_libshout-full is not set # CONFIG_PACKAGE_libshout-nossl is not set # CONFIG_PACKAGE_libsigcxx is not set # CONFIG_PACKAGE_libsmartcols is not set # CONFIG_PACKAGE_libsndfile is not set # CONFIG_PACKAGE_libsoc is not set # CONFIG_PACKAGE_libsocks is not set # CONFIG_PACKAGE_libsodium is not set # CONFIG_PACKAGE_libsoup is not set # CONFIG_PACKAGE_libsoxr is not set # CONFIG_PACKAGE_libspeex is not set # CONFIG_PACKAGE_libspeexdsp is not set # CONFIG_PACKAGE_libspice-server is not set # CONFIG_PACKAGE_libss is not set # CONFIG_PACKAGE_libssh is not set # CONFIG_PACKAGE_libssh2 is not set # CONFIG_PACKAGE_libstoken is not set # CONFIG_PACKAGE_libstrophe is not set # CONFIG_PACKAGE_libtalloc is not set # CONFIG_PACKAGE_libtasn1 is not set # CONFIG_PACKAGE_libtheora is not set # CONFIG_PACKAGE_libtiff is not set # CONFIG_PACKAGE_libtiffxx is not set # CONFIG_PACKAGE_libtins is not set # CONFIG_PACKAGE_libtirpc is not set # CONFIG_PACKAGE_libtorrent is not set CONFIG_PACKAGE_libubox=y # CONFIG_PACKAGE_libubox-lua is not set CONFIG_PACKAGE_libubus=y CONFIG_PACKAGE_libubus-lua=y CONFIG_PACKAGE_libuci=y CONFIG_PACKAGE_libuci-lua=y CONFIG_PACKAGE_libuclient=y # CONFIG_PACKAGE_libudev-fbsd is not set # CONFIG_PACKAGE_libudns is not set # CONFIG_PACKAGE_libuecc is not set # CONFIG_PACKAGE_libugpio is not set # CONFIG_PACKAGE_libunistring is not set # CONFIG_PACKAGE_libunwind is not set # CONFIG_PACKAGE_libupnp is not set # CONFIG_PACKAGE_libupnpp is not set # CONFIG_PACKAGE_liburcu is not set # CONFIG_PACKAGE_libusb-1.0 is not set # CONFIG_PACKAGE_libusb-compat is not set # CONFIG_PACKAGE_libusbmuxd is not set # CONFIG_PACKAGE_libustream-mbedtls is not set CONFIG_PACKAGE_libustream-openssl=y # CONFIG_PACKAGE_libustream-wolfssl is not set CONFIG_PACKAGE_libuuid=y # CONFIG_PACKAGE_libuv is not set # CONFIG_PACKAGE_libuvc is not set # CONFIG_PACKAGE_libv4l is not set # CONFIG_PACKAGE_libvorbis is not set # CONFIG_PACKAGE_libvorbisidec is not set # CONFIG_PACKAGE_libvpx is not set # CONFIG_PACKAGE_libwebcam is not set # CONFIG_PACKAGE_libwebsockets-full is not set # CONFIG_PACKAGE_libwebsockets-mbedtls is not set # CONFIG_PACKAGE_libwebsockets-openssl is not set # CONFIG_PACKAGE_libwrap is not set # CONFIG_PACKAGE_libwxbase is not set # CONFIG_PACKAGE_libx264 is not set # CONFIG_PACKAGE_libxerces-c is not set # CONFIG_PACKAGE_libxerces-c-samples is not set # CONFIG_PACKAGE_libxml2 is not set # CONFIG_PACKAGE_libxslt is not set # CONFIG_PACKAGE_libyaml-cpp is not set # CONFIG_PACKAGE_libyang is not set # CONFIG_PACKAGE_libzdb is not set # CONFIG_PACKAGE_libzmq-curve is not set # CONFIG_PACKAGE_libzmq-nc is not set # CONFIG_PACKAGE_linux-atm is not set # CONFIG_PACKAGE_loudmouth is not set # CONFIG_PACKAGE_lttng-ust is not set # CONFIG_PACKAGE_mtdev is not set # CONFIG_PACKAGE_musl-fts is not set # CONFIG_PACKAGE_mxml is not set # CONFIG_PACKAGE_nacl is not set # CONFIG_PACKAGE_oniguruma is not set # CONFIG_PACKAGE_opencv is not set # CONFIG_PACKAGE_p11-kit is not set # CONFIG_PACKAGE_pixman is not set # CONFIG_PACKAGE_poco is not set # CONFIG_PACKAGE_protobuf is not set # CONFIG_PACKAGE_protobuf-lite is not set # CONFIG_PACKAGE_pthsem is not set # CONFIG_PACKAGE_rblibtorrent is not set CONFIG_PACKAGE_rpcd-mod-rrdns=y # CONFIG_PACKAGE_rxtx is not set # CONFIG_PACKAGE_sbc is not set # CONFIG_PACKAGE_spice-protocol is not set # CONFIG_PACKAGE_terminfo is not set # CONFIG_PACKAGE_tinycdb is not set # CONFIG_PACKAGE_uclibcxx is not set # CONFIG_PACKAGE_uw-imap is not set # CONFIG_PACKAGE_xmlrpc-c is not set # CONFIG_PACKAGE_xmlrpc-c-client is not set # CONFIG_PACKAGE_xmlrpc-c-server is not set # CONFIG_PACKAGE_yajl is not set CONFIG_PACKAGE_zlib=y  # # Configuration # # CONFIG_ZLIB_OPTIMIZE_SPEED is not set  # # LuCI #  # # 1. Collections # CONFIG_PACKAGE_luci=y # CONFIG_PACKAGE_luci-nginx is not set # CONFIG_PACKAGE_luci-ssl-nginx is not set # CONFIG_PACKAGE_luci-ssl-openssl is not set  # # 2. Modules # CONFIG_PACKAGE_luci-base=y CONFIG_LUCI_SRCDIET=y  # # Translations # # CONFIG_LUCI_LANG_hu is not set # CONFIG_LUCI_LANG_pt is not set # CONFIG_LUCI_LANG_sk is not set # CONFIG_LUCI_LANG_ko is not set # CONFIG_LUCI_LANG_en is not set # CONFIG_LUCI_LANG_el is not set # CONFIG_LUCI_LANG_uk is not set # CONFIG_LUCI_LANG_ja is not set # CONFIG_LUCI_LANG_vi is not set # CONFIG_LUCI_LANG_he is not set # CONFIG_LUCI_LANG_no is not set # CONFIG_LUCI_LANG_ms is not set # CONFIG_LUCI_LANG_pl is not set CONFIG_LUCI_LANG_zh-cn=y # CONFIG_LUCI_LANG_ro is not set # CONFIG_LUCI_LANG_de is not set # CONFIG_LUCI_LANG_zh-tw is not set # CONFIG_LUCI_LANG_tr is not set # CONFIG_LUCI_LANG_sv is not set # CONFIG_LUCI_LANG_ru is not set # CONFIG_LUCI_LANG_pt-br is not set # CONFIG_LUCI_LANG_ca is not set # CONFIG_LUCI_LANG_es is not set # CONFIG_LUCI_LANG_cs is not set # CONFIG_LUCI_LANG_fr is not set # CONFIG_LUCI_LANG_it is not set CONFIG_PACKAGE_luci-mod-admin-full=y # CONFIG_PACKAGE_luci-mod-failsafe is not set # CONFIG_PACKAGE_luci-mod-freifunk is not set # CONFIG_PACKAGE_luci-mod-freifunk-community is not set # CONFIG_PACKAGE_luci-mod-rpc is not set  # # 3. Applications # # CONFIG_PACKAGE_luci-app-accesscontrol is not set # CONFIG_PACKAGE_luci-app-acme is not set # CONFIG_PACKAGE_luci-app-adblock is not set # CONFIG_PACKAGE_luci-app-adbyby-plus is not set # CONFIG_PACKAGE_luci-app-advanced-reboot is not set # CONFIG_PACKAGE_luci-app-ahcp is not set # CONFIG_PACKAGE_luci-app-airplay2 is not set # CONFIG_PACKAGE_luci-app-amule is not set # CONFIG_PACKAGE_luci-app-aria2 is not set # CONFIG_PACKAGE_luci-app-arpbind is not set # CONFIG_PACKAGE_luci-app-asterisk is not set # CONFIG_PACKAGE_luci-app-attendedsysupgrade is not set CONFIG_PACKAGE_luci-app-autoreboot=y # CONFIG_PACKAGE_luci-app-baidupcs-web is not set # CONFIG_PACKAGE_luci-app-bcp38 is not set # CONFIG_PACKAGE_luci-app-bird1-ipv4 is not set # CONFIG_PACKAGE_luci-app-bird1-ipv6 is not set # CONFIG_PACKAGE_luci-app-bmx6 is not set # CONFIG_PACKAGE_luci-app-bmx7 is not set # CONFIG_PACKAGE_luci-app-cifs-mount is not set # CONFIG_PACKAGE_luci-app-cifsd is not set # CONFIG_PACKAGE_luci-app-cjdns is not set # CONFIG_PACKAGE_luci-app-clamav is not set # CONFIG_PACKAGE_luci-app-commands is not set # CONFIG_PACKAGE_luci-app-cshark is not set # CONFIG_PACKAGE_luci-app-ddns is not set # CONFIG_PACKAGE_luci-app-diag-core is not set # CONFIG_PACKAGE_luci-app-diskman is not set # CONFIG_PACKAGE_luci-app-diskman_INCLUDE_btrfs_progs is not set # CONFIG_PACKAGE_luci-app-diskman_INCLUDE_lsblk is not set # CONFIG_PACKAGE_luci-app-diskman_INCLUDE_mdadm is not set # CONFIG_PACKAGE_luci-app-diskman_INCLUDE_kmod_md_raid456 is not set # CONFIG_PACKAGE_luci-app-diskman_INCLUDE_kmod_md_linear is not set # CONFIG_PACKAGE_luci-app-dnscrypt-proxy is not set # CONFIG_PACKAGE_luci-app-dump1090 is not set # CONFIG_PACKAGE_luci-app-dynapoint is not set # CONFIG_PACKAGE_luci-app-e2guardian is not set # CONFIG_PACKAGE_luci-app-familycloud is not set # CONFIG_PACKAGE_luci-app-filetransfer is not set CONFIG_PACKAGE_luci-app-firewall=y # CONFIG_PACKAGE_luci-app-freifunk-diagnostics is not set # CONFIG_PACKAGE_luci-app-freifunk-policyrouting is not set # CONFIG_PACKAGE_luci-app-freifunk-widgets is not set # CONFIG_PACKAGE_luci-app-frpc is not set # CONFIG_PACKAGE_luci-app-frps is not set # CONFIG_PACKAGE_luci-app-fwknopd is not set # CONFIG_PACKAGE_luci-app-guest-wifi is not set # CONFIG_PACKAGE_luci-app-haproxy-tcp is not set # CONFIG_PACKAGE_luci-app-hd-idle is not set # CONFIG_PACKAGE_luci-app-hnet is not set # CONFIG_PACKAGE_luci-app-ipsec-vpnd is not set # CONFIG_PACKAGE_luci-app-kodexplorer is not set # CONFIG_PACKAGE_luci-app-lxc is not set # CONFIG_PACKAGE_luci-app-meshwizard is not set # CONFIG_PACKAGE_luci-app-minidlna is not set # CONFIG_PACKAGE_luci-app-mjpg-streamer is not set # CONFIG_PACKAGE_luci-app-mtwifi is not set # CONFIG_PACKAGE_luci-app-music-remote-center is not set # CONFIG_PACKAGE_luci-app-mwan3 is not set # CONFIG_PACKAGE_luci-app-mwan3helper is not set # CONFIG_PACKAGE_luci-app-n2n_v2 is not set # CONFIG_PACKAGE_luci-app-netdata is not set # CONFIG_PACKAGE_luci-app-nfs is not set # CONFIG_PACKAGE_luci-app-nft-qos is not set # CONFIG_PACKAGE_luci-app-nlbwmon is not set # CONFIG_PACKAGE_luci-app-noddos is not set # CONFIG_PACKAGE_luci-app-nps is not set # CONFIG_PACKAGE_luci-app-ntpc is not set # CONFIG_PACKAGE_luci-app-ocserv is not set # CONFIG_PACKAGE_luci-app-olsr is not set # CONFIG_PACKAGE_luci-app-olsr-services is not set # CONFIG_PACKAGE_luci-app-olsr-viz is not set # CONFIG_PACKAGE_luci-app-openvpn is not set # CONFIG_PACKAGE_luci-app-openvpn-server is not set # CONFIG_PACKAGE_luci-app-p910nd is not set # CONFIG_PACKAGE_luci-app-pagekitec is not set # CONFIG_PACKAGE_luci-app-polipo is not set # CONFIG_PACKAGE_luci-app-pppoe-relay is not set # CONFIG_PACKAGE_luci-app-privoxy is not set # CONFIG_PACKAGE_luci-app-ps3netsrv is not set # CONFIG_PACKAGE_luci-app-qbittorrent is not set # CONFIG_PACKAGE_luci-app-qos is not set # CONFIG_PACKAGE_luci-app-radicale is not set CONFIG_PACKAGE_luci-app-ramfree=y # CONFIG_PACKAGE_luci-app-rclone is not set # CONFIG_PACKAGE_luci-app-rp-pppoe-server is not set # CONFIG_PACKAGE_luci-app-samba is not set # CONFIG_PACKAGE_luci-app-samba4 is not set CONFIG_PACKAGE_luci-app-sfe=y # CONFIG_PACKAGE_luci-app-shadowsocks-libev is not set # CONFIG_PACKAGE_luci-app-shairplay is not set # CONFIG_PACKAGE_luci-app-siitwizard is not set # CONFIG_PACKAGE_luci-app-simple-adblock is not set # CONFIG_PACKAGE_luci-app-softethervpn is not set # CONFIG_PACKAGE_luci-app-splash is not set # CONFIG_PACKAGE_luci-app-sqm is not set # CONFIG_PACKAGE_luci-app-squid is not set # CONFIG_PACKAGE_luci-app-ssrserver-python is not set # CONFIG_PACKAGE_luci-app-statistics is not set # CONFIG_PACKAGE_luci-app-syncdial is not set # CONFIG_PACKAGE_luci-app-tinyproxy is not set # CONFIG_PACKAGE_luci-app-transmission is not set # CONFIG_PACKAGE_luci-app-travelmate is not set # CONFIG_PACKAGE_luci-app-ttyd is not set # CONFIG_PACKAGE_luci-app-udpxy is not set # CONFIG_PACKAGE_luci-app-uhttpd is not set # CONFIG_PACKAGE_luci-app-unblockmusic is not set # CONFIG_UnblockNeteaseMusic_Go is not set # CONFIG_UnblockNeteaseMusic_NodeJS is not set # CONFIG_PACKAGE_luci-app-unbound is not set # CONFIG_PACKAGE_luci-app-upnp is not set # CONFIG_PACKAGE_luci-app-usb-printer is not set CONFIG_PACKAGE_luci-app-v2ray-server=y # CONFIG_PACKAGE_luci-app-verysync is not set # CONFIG_PACKAGE_luci-app-vlmcsd is not set # CONFIG_PACKAGE_luci-app-vnstat is not set # CONFIG_PACKAGE_luci-app-vpnbypass is not set # CONFIG_PACKAGE_luci-app-vsftpd is not set # CONFIG_PACKAGE_luci-app-watchcat is not set # CONFIG_PACKAGE_luci-app-webadmin is not set # CONFIG_PACKAGE_luci-app-wifischedule is not set # CONFIG_PACKAGE_luci-app-wireguard is not set # CONFIG_PACKAGE_luci-app-wol is not set # CONFIG_PACKAGE_luci-app-wrtbwmon is not set # CONFIG_PACKAGE_luci-app-xlnetacc is not set # CONFIG_PACKAGE_luci-app-zerotier is not set  # # 4. Themes # # CONFIG_PACKAGE_luci-theme-argon is not set CONFIG_PACKAGE_luci-theme-bootstrap=y # CONFIG_PACKAGE_luci-theme-material is not set CONFIG_PACKAGE_luci-theme-netgear=y  # # 5. Protocols # # CONFIG_PACKAGE_luci-proto-3g is not set # CONFIG_PACKAGE_luci-proto-bonding is not set # CONFIG_PACKAGE_luci-proto-ipip is not set # CONFIG_PACKAGE_luci-proto-ipv6 is not set # CONFIG_PACKAGE_luci-proto-ncm is not set # CONFIG_PACKAGE_luci-proto-openconnect is not set CONFIG_PACKAGE_luci-proto-ppp=y # CONFIG_PACKAGE_luci-proto-qmi is not set # CONFIG_PACKAGE_luci-proto-relay is not set # CONFIG_PACKAGE_luci-proto-vpnc is not set # CONFIG_PACKAGE_luci-proto-wireguard is not set  # # 6. Libraries # # CONFIG_PACKAGE_luci-lib-docker is not set # CONFIG_PACKAGE_luci-lib-dracula is not set # CONFIG_PACKAGE_luci-lib-httpclient is not set # CONFIG_PACKAGE_luci-lib-httpprotoutils is not set CONFIG_PACKAGE_luci-lib-ip=y # CONFIG_PACKAGE_luci-lib-iptparser is not set # CONFIG_PACKAGE_luci-lib-jquery-1-4 is not set # CONFIG_PACKAGE_luci-lib-json is not set CONFIG_PACKAGE_luci-lib-jsonc=y # CONFIG_PACKAGE_luci-lib-luaneightbl is not set CONFIG_PACKAGE_luci-lib-nixio=y # CONFIG_PACKAGE_luci-lib-px5g is not set  # # 9. Freifunk # # CONFIG_PACKAGE_freifunk-common is not set # CONFIG_PACKAGE_freifunk-firewall is not set # CONFIG_PACKAGE_freifunk-policyrouting is not set # CONFIG_PACKAGE_freifunk-watchdog is not set # CONFIG_PACKAGE_meshwizard is not set CONFIG_PACKAGE_default-settings=y CONFIG_PACKAGE_luci-i18n-autoreboot-zh-cn=y # CONFIG_PACKAGE_luci-i18n-base-ca is not set # CONFIG_PACKAGE_luci-i18n-base-cs is not set # CONFIG_PACKAGE_luci-i18n-base-de is not set # CONFIG_PACKAGE_luci-i18n-base-el is not set # CONFIG_PACKAGE_luci-i18n-base-en is not set # CONFIG_PACKAGE_luci-i18n-base-es is not set # CONFIG_PACKAGE_luci-i18n-base-fr is not set # CONFIG_PACKAGE_luci-i18n-base-he is not set # CONFIG_PACKAGE_luci-i18n-base-hu is not set # CONFIG_PACKAGE_luci-i18n-base-it is not set # CONFIG_PACKAGE_luci-i18n-base-ja is not set # CONFIG_PACKAGE_luci-i18n-base-ko is not set # CONFIG_PACKAGE_luci-i18n-base-ms is not set # CONFIG_PACKAGE_luci-i18n-base-no is not set # CONFIG_PACKAGE_luci-i18n-base-pl is not set # CONFIG_PACKAGE_luci-i18n-base-pt is not set # CONFIG_PACKAGE_luci-i18n-base-pt-br is not set # CONFIG_PACKAGE_luci-i18n-base-ro is not set # CONFIG_PACKAGE_luci-i18n-base-ru is not set # CONFIG_PACKAGE_luci-i18n-base-sk is not set # CONFIG_PACKAGE_luci-i18n-base-sv is not set # CONFIG_PACKAGE_luci-i18n-base-tr is not set # CONFIG_PACKAGE_luci-i18n-base-uk is not set # CONFIG_PACKAGE_luci-i18n-base-vi is not set CONFIG_PACKAGE_luci-i18n-base-zh-cn=y # CONFIG_PACKAGE_luci-i18n-base-zh-tw is not set # CONFIG_PACKAGE_luci-i18n-firewall-ca is not set # CONFIG_PACKAGE_luci-i18n-firewall-cs is not set # CONFIG_PACKAGE_luci-i18n-firewall-de is not set # CONFIG_PACKAGE_luci-i18n-firewall-el is not set # CONFIG_PACKAGE_luci-i18n-firewall-en is not set # CONFIG_PACKAGE_luci-i18n-firewall-es is not set # CONFIG_PACKAGE_luci-i18n-firewall-fr is not set # CONFIG_PACKAGE_luci-i18n-firewall-he is not set # CONFIG_PACKAGE_luci-i18n-firewall-hu is not set # CONFIG_PACKAGE_luci-i18n-firewall-it is not set # CONFIG_PACKAGE_luci-i18n-firewall-ja is not set # CONFIG_PACKAGE_luci-i18n-firewall-ko is not set # CONFIG_PACKAGE_luci-i18n-firewall-ms is not set # CONFIG_PACKAGE_luci-i18n-firewall-no is not set # CONFIG_PACKAGE_luci-i18n-firewall-pl is not set # CONFIG_PACKAGE_luci-i18n-firewall-pt is not set # CONFIG_PACKAGE_luci-i18n-firewall-pt-br is not set # CONFIG_PACKAGE_luci-i18n-firewall-ro is not set # CONFIG_PACKAGE_luci-i18n-firewall-ru is not set # CONFIG_PACKAGE_luci-i18n-firewall-sk is not set # CONFIG_PACKAGE_luci-i18n-firewall-sv is not set # CONFIG_PACKAGE_luci-i18n-firewall-tr is not set # CONFIG_PACKAGE_luci-i18n-firewall-uk is not set # CONFIG_PACKAGE_luci-i18n-firewall-vi is not set CONFIG_PACKAGE_luci-i18n-firewall-zh-cn=y # CONFIG_PACKAGE_luci-i18n-firewall-zh-tw is not set CONFIG_PACKAGE_luci-i18n-ramfree-zh-cn=y CONFIG_PACKAGE_luci-i18n-sfe-zh-cn=y CONFIG_PACKAGE_luci-i18n-v2ray-server-zh-cn=y  # # Mail # # CONFIG_PACKAGE_alpine is not set # CONFIG_PACKAGE_alpine-nossl is not set # CONFIG_PACKAGE_bogofilter is not set # CONFIG_PACKAGE_clamsmtp is not set # CONFIG_PACKAGE_dovecot is not set # CONFIG_PACKAGE_dovecot-pigeonhole is not set # CONFIG_PACKAGE_dovecot-utils is not set # CONFIG_PACKAGE_emailrelay is not set # CONFIG_PACKAGE_fdm is not set # CONFIG_PACKAGE_greyfix is not set # CONFIG_PACKAGE_mailman is not set # CONFIG_PACKAGE_mailsend is not set # CONFIG_PACKAGE_mailsend-nossl is not set # CONFIG_PACKAGE_msmtp is not set # CONFIG_PACKAGE_msmtp-nossl is not set # CONFIG_PACKAGE_mutt is not set # CONFIG_PACKAGE_nail is not set # CONFIG_PACKAGE_opendkim is not set # CONFIG_PACKAGE_opendkim-tools is not set # CONFIG_PACKAGE_postfix is not set  # # Select postfix build options # CONFIG_POSTFIX_TLS=y CONFIG_POSTFIX_SASL=y CONFIG_POSTFIX_LDAP=y # CONFIG_POSTFIX_DB is not set CONFIG_POSTFIX_CDB=y CONFIG_POSTFIX_SQLITE=y # CONFIG_POSTFIX_PGSQL is not set CONFIG_POSTFIX_PCRE=y # CONFIG_POSTFIX_EAI is not set # CONFIG_PACKAGE_ssmtp is not set  # # Multimedia #  # # Streaming # # CONFIG_PACKAGE_oggfwd is not set # CONFIG_PACKAGE_crtmpserver is not set # CONFIG_PACKAGE_ffmpeg is not set # CONFIG_PACKAGE_ffprobe is not set # CONFIG_PACKAGE_ffserver is not set # CONFIG_PACKAGE_fswebcam is not set # CONFIG_PACKAGE_gmediarender is not set # CONFIG_PACKAGE_gphoto2 is not set # CONFIG_PACKAGE_grilo is not set # CONFIG_PACKAGE_grilo-plugins is not set # CONFIG_PACKAGE_gst1-libav is not set # CONFIG_PACKAGE_gstreamer1-libs is not set # CONFIG_PACKAGE_gstreamer1-plugins-bad is not set # CONFIG_PACKAGE_gstreamer1-plugins-base is not set # CONFIG_PACKAGE_gstreamer1-plugins-good is not set # CONFIG_PACKAGE_gstreamer1-plugins-ugly is not set # CONFIG_PACKAGE_gstreamer1-utils is not set # CONFIG_PACKAGE_icecast is not set # CONFIG_PACKAGE_lcdgrilo is not set # CONFIG_PACKAGE_minidlna is not set # CONFIG_PACKAGE_mjpg-streamer is not set # CONFIG_PACKAGE_motion is not set # CONFIG_PACKAGE_tvheadend is not set # CONFIG_PACKAGE_v4l2rtspserver is not set # CONFIG_PACKAGE_vips is not set # CONFIG_PACKAGE_xupnpd is not set # CONFIG_PACKAGE_youtube-dl is not set  # # Network #  # # BitTorrent # # CONFIG_PACKAGE_mktorrent is not set # CONFIG_PACKAGE_opentracker is not set # CONFIG_PACKAGE_opentracker6 is not set # CONFIG_PACKAGE_qBittorrent is not set # CONFIG_PACKAGE_rtorrent is not set # CONFIG_PACKAGE_rtorrent-rpc is not set # CONFIG_PACKAGE_transmission-cli-openssl is not set # CONFIG_PACKAGE_transmission-daemon-openssl is not set # CONFIG_PACKAGE_transmission-remote-openssl is not set # CONFIG_PACKAGE_transmission-web is not set # CONFIG_PACKAGE_transmission-web-control is not set  # # Captive Portals # # CONFIG_PACKAGE_coova-chilli is not set # CONFIG_PACKAGE_nodogsplash is not set # CONFIG_PACKAGE_wifidog is not set # CONFIG_PACKAGE_wifidog-ng-mbedtls is not set # CONFIG_PACKAGE_wifidog-ng-nossl is not set # CONFIG_PACKAGE_wifidog-ng-openssl is not set # CONFIG_PACKAGE_wifidog-ng-wolfssl is not set # CONFIG_PACKAGE_wifidog-tls is not set  # # Cloud Manager # # CONFIG_PACKAGE_rclone-ng is not set # CONFIG_PACKAGE_rclone-webui-react is not set  # # Download Manager # # CONFIG_PACKAGE_ariang is not set # CONFIG_PACKAGE_webui-aria2 is not set # CONFIG_PACKAGE_yaaw is not set  # # File Transfer # # CONFIG_PACKAGE_aria2 is not set # CONFIG_PACKAGE_atftp is not set # CONFIG_PACKAGE_atftpd is not set # CONFIG_PACKAGE_curl is not set # CONFIG_PACKAGE_gnurl is not set # CONFIG_PACKAGE_lftp is not set # CONFIG_PACKAGE_ps3netsrv is not set # CONFIG_PACKAGE_rosy-file-server is not set # CONFIG_PACKAGE_rsync is not set # CONFIG_PACKAGE_rsyncd is not set # CONFIG_PACKAGE_vsftpd is not set CONFIG_PACKAGE_vsftpd-alt=y CONFIG_VSFTPD_USE_UCI_SCRIPTS=y # CONFIG_PACKAGE_vsftpd-tls is not set CONFIG_PACKAGE_wget=y # CONFIG_PACKAGE_wget-nossl is not set  # # Filesystem # # CONFIG_PACKAGE_davfs2 is not set # CONFIG_PACKAGE_ksmbd-avahi-service is not set # CONFIG_PACKAGE_ksmbd-server is not set # CONFIG_PACKAGE_ksmbd-utils is not set # CONFIG_PACKAGE_netatalk is not set # CONFIG_PACKAGE_nfs-kernel-server is not set # CONFIG_PACKAGE_owftpd is not set # CONFIG_PACKAGE_owhttpd is not set # CONFIG_PACKAGE_owserver is not set # CONFIG_PACKAGE_sshfs is not set  # # Firewall # # CONFIG_PACKAGE_arptables is not set # CONFIG_PACKAGE_conntrack is not set # CONFIG_PACKAGE_conntrackd is not set # CONFIG_PACKAGE_ebtables is not set # CONFIG_PACKAGE_fwknop is not set # CONFIG_PACKAGE_fwknopd is not set # CONFIG_PACKAGE_ip6tables is not set CONFIG_PACKAGE_iptables=y # CONFIG_IPTABLES_CONNLABEL is not set # CONFIG_IPTABLES_NFTABLES is not set # CONFIG_PACKAGE_iptables-mod-account is not set # CONFIG_PACKAGE_iptables-mod-chaos is not set # CONFIG_PACKAGE_iptables-mod-checksum is not set # CONFIG_PACKAGE_iptables-mod-cluster is not set # CONFIG_PACKAGE_iptables-mod-clusterip is not set # CONFIG_PACKAGE_iptables-mod-condition is not set # CONFIG_PACKAGE_iptables-mod-conntrack-extra is not set # CONFIG_PACKAGE_iptables-mod-delude is not set # CONFIG_PACKAGE_iptables-mod-dhcpmac is not set # CONFIG_PACKAGE_iptables-mod-dnetmap is not set # CONFIG_PACKAGE_iptables-mod-extra is not set # CONFIG_PACKAGE_iptables-mod-filter is not set CONFIG_PACKAGE_iptables-mod-fullconenat=y # CONFIG_PACKAGE_iptables-mod-fuzzy is not set # CONFIG_PACKAGE_iptables-mod-geoip is not set # CONFIG_PACKAGE_iptables-mod-hashlimit is not set # CONFIG_PACKAGE_iptables-mod-iface is not set # CONFIG_PACKAGE_iptables-mod-ipmark is not set # CONFIG_PACKAGE_iptables-mod-ipopt is not set # CONFIG_PACKAGE_iptables-mod-ipp2p is not set # CONFIG_PACKAGE_iptables-mod-iprange is not set # CONFIG_PACKAGE_iptables-mod-ipsec is not set # CONFIG_PACKAGE_iptables-mod-ipv4options is not set # CONFIG_PACKAGE_iptables-mod-led is not set # CONFIG_PACKAGE_iptables-mod-length2 is not set # CONFIG_PACKAGE_iptables-mod-logmark is not set # CONFIG_PACKAGE_iptables-mod-lscan is not set # CONFIG_PACKAGE_iptables-mod-lua is not set # CONFIG_PACKAGE_iptables-mod-nat-extra is not set # CONFIG_PACKAGE_iptables-mod-nflog is not set # CONFIG_PACKAGE_iptables-mod-nfqueue is not set # CONFIG_PACKAGE_iptables-mod-physdev is not set # CONFIG_PACKAGE_iptables-mod-psd is not set # CONFIG_PACKAGE_iptables-mod-quota2 is not set # CONFIG_PACKAGE_iptables-mod-rpfilter is not set # CONFIG_PACKAGE_iptables-mod-sysrq is not set # CONFIG_PACKAGE_iptables-mod-tarpit is not set # CONFIG_PACKAGE_iptables-mod-tee is not set # CONFIG_PACKAGE_iptables-mod-tproxy is not set # CONFIG_PACKAGE_iptables-mod-trace is not set # CONFIG_PACKAGE_iptables-mod-u32 is not set # CONFIG_PACKAGE_iptables-mod-ulog is not set # CONFIG_PACKAGE_iptaccount is not set # CONFIG_PACKAGE_iptgeoip is not set # CONFIG_PACKAGE_miniupnpc is not set CONFIG_PACKAGE_miniupnpd=y # CONFIG_MINIUPNPD_IGDv2 is not set # CONFIG_PACKAGE_natpmpc is not set # CONFIG_PACKAGE_nftables is not set # CONFIG_PACKAGE_shorewall is not set # CONFIG_PACKAGE_shorewall-core is not set # CONFIG_PACKAGE_shorewall-lite is not set # CONFIG_PACKAGE_shorewall6 is not set # CONFIG_PACKAGE_shorewall6-lite is not set # CONFIG_PACKAGE_snort is not set  # # Firewall Tunnel # # CONFIG_PACKAGE_iodine is not set # CONFIG_PACKAGE_iodined is not set  # # FreeRADIUS (version 3) # # CONFIG_PACKAGE_freeradius3 is not set # CONFIG_PACKAGE_freeradius3-common is not set # CONFIG_PACKAGE_freeradius3-utils is not set  # # IP Addresses and Names # # CONFIG_PACKAGE_aggregate is not set # CONFIG_PACKAGE_announce is not set # CONFIG_PACKAGE_avahi-autoipd is not set # CONFIG_PACKAGE_avahi-daemon-service-http is not set # CONFIG_PACKAGE_avahi-daemon-service-ssh is not set # CONFIG_PACKAGE_avahi-dbus-daemon is not set # CONFIG_PACKAGE_avahi-dnsconfd is not set # CONFIG_PACKAGE_avahi-nodbus-daemon is not set # CONFIG_PACKAGE_avahi-utils is not set # CONFIG_PACKAGE_bind-check is not set # CONFIG_PACKAGE_bind-client is not set # CONFIG_PACKAGE_bind-dig is not set # CONFIG_PACKAGE_bind-dnssec is not set # CONFIG_PACKAGE_bind-host is not set # CONFIG_PACKAGE_bind-rndc is not set # CONFIG_PACKAGE_bind-server is not set # CONFIG_PACKAGE_bind-tools is not set # CONFIG_PACKAGE_danish is not set CONFIG_PACKAGE_ddns-scripts=y CONFIG_PACKAGE_ddns-scripts_aliyun=y # CONFIG_PACKAGE_ddns-scripts_cloudflare.com-v4 is not set CONFIG_PACKAGE_ddns-scripts_dnspod=y # CONFIG_PACKAGE_ddns-scripts_freedns_42_pl is not set # CONFIG_PACKAGE_ddns-scripts_godaddy.com-v1 is not set # CONFIG_PACKAGE_ddns-scripts_no-ip_com is not set # CONFIG_PACKAGE_ddns-scripts_nsupdate is not set # CONFIG_PACKAGE_ddns-scripts_route53-v1 is not set # CONFIG_PACKAGE_dhcp-forwarder is not set # CONFIG_PACKAGE_dns2socks is not set # CONFIG_PACKAGE_dnscrypt-proxy is not set # CONFIG_PACKAGE_dnscrypt-proxy-resolvers is not set # CONFIG_PACKAGE_drill is not set # CONFIG_PACKAGE_hostip is not set # CONFIG_PACKAGE_idn is not set # CONFIG_PACKAGE_idn2 is not set # CONFIG_PACKAGE_inadyn is not set # CONFIG_PACKAGE_isc-dhcp-client-ipv4 is not set # CONFIG_PACKAGE_isc-dhcp-client-ipv6 is not set # CONFIG_PACKAGE_isc-dhcp-omshell-ipv4 is not set # CONFIG_PACKAGE_isc-dhcp-omshell-ipv6 is not set # CONFIG_PACKAGE_isc-dhcp-relay-ipv4 is not set # CONFIG_PACKAGE_isc-dhcp-relay-ipv6 is not set # CONFIG_PACKAGE_isc-dhcp-server-ipv4 is not set # CONFIG_PACKAGE_isc-dhcp-server-ipv6 is not set # CONFIG_PACKAGE_kadnode is not set # CONFIG_PACKAGE_knot is not set # CONFIG_PACKAGE_knot-dig is not set # CONFIG_PACKAGE_knot-host is not set # CONFIG_PACKAGE_knot-keymgr is not set # CONFIG_PACKAGE_knot-nsupdate is not set # CONFIG_PACKAGE_knot-tests is not set # CONFIG_PACKAGE_knot-zonecheck is not set # CONFIG_PACKAGE_ldns-examples is not set # CONFIG_PACKAGE_mdns-utils is not set # CONFIG_PACKAGE_mdnsd is not set # CONFIG_PACKAGE_mdnsresponder is not set # CONFIG_PACKAGE_nsd is not set # CONFIG_PACKAGE_nsd-control is not set # CONFIG_PACKAGE_nsd-control-setup is not set # CONFIG_PACKAGE_nsd-nossl is not set # CONFIG_PACKAGE_ohybridproxy is not set # CONFIG_PACKAGE_stubby is not set # CONFIG_PACKAGE_unbound-anchor is not set # CONFIG_PACKAGE_unbound-checkconf is not set # CONFIG_PACKAGE_unbound-control is not set # CONFIG_PACKAGE_unbound-control-setup is not set # CONFIG_PACKAGE_unbound-daemon is not set # CONFIG_PACKAGE_unbound-daemon-heavy is not set # CONFIG_PACKAGE_unbound-host is not set # CONFIG_PACKAGE_wsdd2 is not set # CONFIG_PACKAGE_zonestitcher is not set  # # Instant Messaging # # CONFIG_PACKAGE_bitlbee is not set # CONFIG_PACKAGE_irssi is not set # CONFIG_PACKAGE_ngircd is not set # CONFIG_PACKAGE_ngircd-nossl is not set # CONFIG_PACKAGE_prosody is not set # CONFIG_PACKAGE_quassel-irssi is not set # CONFIG_PACKAGE_umurmur-mbedtls is not set # CONFIG_PACKAGE_umurmur-openssl is not set # CONFIG_PACKAGE_znc is not set  # # Linux ATM tools # # CONFIG_PACKAGE_atm-aread is not set # CONFIG_PACKAGE_atm-atmaddr is not set # CONFIG_PACKAGE_atm-atmdiag is not set # CONFIG_PACKAGE_atm-atmdump is not set # CONFIG_PACKAGE_atm-atmloop is not set # CONFIG_PACKAGE_atm-atmsigd is not set # CONFIG_PACKAGE_atm-atmswitch is not set # CONFIG_PACKAGE_atm-atmtcp is not set # CONFIG_PACKAGE_atm-awrite is not set # CONFIG_PACKAGE_atm-bus is not set # CONFIG_PACKAGE_atm-debug-tools is not set # CONFIG_PACKAGE_atm-diagnostics is not set # CONFIG_PACKAGE_atm-esi is not set # CONFIG_PACKAGE_atm-ilmid is not set # CONFIG_PACKAGE_atm-ilmidiag is not set # CONFIG_PACKAGE_atm-lecs is not set # CONFIG_PACKAGE_atm-les is not set # CONFIG_PACKAGE_atm-mpcd is not set # CONFIG_PACKAGE_atm-saaldump is not set # CONFIG_PACKAGE_atm-sonetdiag is not set # CONFIG_PACKAGE_atm-svc_recv is not set # CONFIG_PACKAGE_atm-svc_send is not set # CONFIG_PACKAGE_atm-tools is not set # CONFIG_PACKAGE_atm-ttcp_atm is not set # CONFIG_PACKAGE_atm-zeppelin is not set # CONFIG_PACKAGE_br2684ctl is not set  # # NMAP Suite # # CONFIG_PACKAGE_ncat is not set # CONFIG_PACKAGE_ncat-ssl is not set # CONFIG_PACKAGE_ndiff is not set # CONFIG_PACKAGE_nmap is not set # CONFIG_PACKAGE_nmap-ssl is not set # CONFIG_PACKAGE_nping is not set  # # NTRIP # # CONFIG_PACKAGE_ntripcaster is not set # CONFIG_PACKAGE_ntripclient is not set # CONFIG_PACKAGE_ntripserver is not set  # # NeteaseMusic # # CONFIG_PACKAGE_UnblockNeteaseMusic is not set # CONFIG_PACKAGE_UnblockNeteaseMusicGo is not set  # # OLSR.org network framework # # CONFIG_PACKAGE_oonf-dlep-proxy is not set # CONFIG_PACKAGE_oonf-dlep-radio is not set # CONFIG_PACKAGE_oonf-init-scripts is not set # CONFIG_PACKAGE_oonf-olsrd2 is not set  # # Open vSwitch # # CONFIG_PACKAGE_openvswitch is not set # CONFIG_PACKAGE_openvswitch-ovn-host is not set # CONFIG_PACKAGE_openvswitch-ovn-north is not set # CONFIG_PACKAGE_openvswitch-python is not set # CONFIG_PACKAGE_openvswitch-python3 is not set  # # OpenLDAP # # CONFIG_PACKAGE_libopenldap is not set CONFIG_OPENLDAP_DEBUG=y # CONFIG_OPENLDAP_CRYPT is not set # CONFIG_OPENLDAP_MONITOR is not set # CONFIG_OPENLDAP_DB47 is not set # CONFIG_OPENLDAP_ICU is not set # CONFIG_PACKAGE_openldap-server is not set # CONFIG_PACKAGE_openldap-utils is not set  # # P2P # # CONFIG_PACKAGE_amule is not set # CONFIG_AMULE_CRYPTOPP_STATIC_LINKING is not set # CONFIG_PACKAGE_antileech is not set  # # Printing # # CONFIG_PACKAGE_p910nd is not set  # # Project V # CONFIG_PACKAGE_v2ray=y  # # V2Ray Configuration # # CONFIG_V2RAY_COMPRESS_GOPROXY is not set # CONFIG_V2RAY_JSON_V2CTL is not set CONFIG_V2RAY_JSON_INTERNAL=y # CONFIG_V2RAY_JSON_NONE is not set CONFIG_V2RAY_EXCLUDE_V2CTL=y CONFIG_V2RAY_EXCLUDE_ASSETS=y CONFIG_V2RAY_COMPRESS_UPX=y CONFIG_V2RAY_DISABLE_NONE=y # CONFIG_V2RAY_DISABLE_CUSTOM is not set # CONFIG_PACKAGE_v2ray-plugin is not set # CONFIG_v2ray-plugin_INCLUDE_GOPROXY is not set  # # Routing and Redirection # # CONFIG_PACKAGE_babel-pinger is not set # CONFIG_PACKAGE_babeld is not set # CONFIG_PACKAGE_batmand is not set # CONFIG_PACKAGE_bcp38 is not set # CONFIG_PACKAGE_bird1-ipv4 is not set # CONFIG_PACKAGE_bird1-ipv4-uci is not set # CONFIG_PACKAGE_bird1-ipv6 is not set # CONFIG_PACKAGE_bird1-ipv6-uci is not set # CONFIG_PACKAGE_bird1c-ipv4 is not set # CONFIG_PACKAGE_bird1c-ipv6 is not set # CONFIG_PACKAGE_bird1cl-ipv4 is not set # CONFIG_PACKAGE_bird1cl-ipv6 is not set # CONFIG_PACKAGE_bird2 is not set # CONFIG_PACKAGE_bird2c is not set # CONFIG_PACKAGE_bird2cl is not set # CONFIG_PACKAGE_bmx6 is not set # CONFIG_PACKAGE_bmx7 is not set # CONFIG_PACKAGE_cjdns is not set # CONFIG_PACKAGE_cjdns-tests is not set # CONFIG_PACKAGE_dcwapd is not set # CONFIG_PACKAGE_devlink is not set # CONFIG_PACKAGE_genl is not set # CONFIG_PACKAGE_igmpproxy is not set # CONFIG_PACKAGE_ip-bridge is not set CONFIG_PACKAGE_ip-full=y # CONFIG_PACKAGE_ip-tiny is not set # CONFIG_PACKAGE_lldpd is not set # CONFIG_PACKAGE_mcproxy is not set # CONFIG_PACKAGE_mrmctl is not set # CONFIG_PACKAGE_mwan3 is not set # CONFIG_PACKAGE_nstat is not set # CONFIG_PACKAGE_olsrd is not set # CONFIG_PACKAGE_prince is not set # CONFIG_PACKAGE_quagga is not set # CONFIG_PACKAGE_rdma is not set # CONFIG_PACKAGE_relayd is not set # CONFIG_PACKAGE_ss is not set # CONFIG_PACKAGE_sslh is not set # CONFIG_PACKAGE_tc is not set # CONFIG_PACKAGE_tcpproxy is not set # CONFIG_PACKAGE_vis is not set  # # SSH # # CONFIG_PACKAGE_autossh is not set # CONFIG_PACKAGE_openssh-client is not set # CONFIG_PACKAGE_openssh-client-utils is not set # CONFIG_PACKAGE_openssh-keygen is not set # CONFIG_PACKAGE_openssh-moduli is not set # CONFIG_PACKAGE_openssh-server is not set # CONFIG_PACKAGE_openssh-server-pam is not set # CONFIG_PACKAGE_openssh-sftp-avahi-service is not set # CONFIG_PACKAGE_openssh-sftp-client is not set # CONFIG_PACKAGE_openssh-sftp-server is not set # CONFIG_PACKAGE_sshtunnel is not set  # # THC-IPv6 attack and analyzing toolkit # # CONFIG_PACKAGE_thc-ipv6-address6 is not set # CONFIG_PACKAGE_thc-ipv6-alive6 is not set # CONFIG_PACKAGE_thc-ipv6-covert-send6 is not set # CONFIG_PACKAGE_thc-ipv6-covert-send6d is not set # CONFIG_PACKAGE_thc-ipv6-denial6 is not set # CONFIG_PACKAGE_thc-ipv6-detect-new-ip6 is not set # CONFIG_PACKAGE_thc-ipv6-detect-sniffer6 is not set # CONFIG_PACKAGE_thc-ipv6-dnsdict6 is not set # CONFIG_PACKAGE_thc-ipv6-dnsrevenum6 is not set # CONFIG_PACKAGE_thc-ipv6-dos-new-ip6 is not set # CONFIG_PACKAGE_thc-ipv6-dump-router6 is not set # CONFIG_PACKAGE_thc-ipv6-exploit6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-advertise6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-dhcps6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-dns6d is not set # CONFIG_PACKAGE_thc-ipv6-fake-dnsupdate6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-mipv6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-mld26 is not set # CONFIG_PACKAGE_thc-ipv6-fake-mld6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-mldrouter6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-router26 is not set # CONFIG_PACKAGE_thc-ipv6-fake-router6 is not set # CONFIG_PACKAGE_thc-ipv6-fake-solicitate6 is not set # CONFIG_PACKAGE_thc-ipv6-flood-advertise6 is not set # CONFIG_PACKAGE_thc-ipv6-flood-dhcpc6 is not set # CONFIG_PACKAGE_thc-ipv6-flood-mld26 is not set # CONFIG_PACKAGE_thc-ipv6-flood-mld6 is not set # CONFIG_PACKAGE_thc-ipv6-flood-mldrouter6 is not set # CONFIG_PACKAGE_thc-ipv6-flood-router26 is not set # CONFIG_PACKAGE_thc-ipv6-flood-router6 is not set # CONFIG_PACKAGE_thc-ipv6-flood-solicitate6 is not set # CONFIG_PACKAGE_thc-ipv6-fragmentation6 is not set # CONFIG_PACKAGE_thc-ipv6-fuzz-dhcpc6 is not set # CONFIG_PACKAGE_thc-ipv6-fuzz-dhcps6 is not set # CONFIG_PACKAGE_thc-ipv6-fuzz-ip6 is not set # CONFIG_PACKAGE_thc-ipv6-implementation6 is not set # CONFIG_PACKAGE_thc-ipv6-implementation6d is not set # CONFIG_PACKAGE_thc-ipv6-inverse-lookup6 is not set # CONFIG_PACKAGE_thc-ipv6-kill-router6 is not set # CONFIG_PACKAGE_thc-ipv6-ndpexhaust6 is not set # CONFIG_PACKAGE_thc-ipv6-node-query6 is not set # CONFIG_PACKAGE_thc-ipv6-parasite6 is not set # CONFIG_PACKAGE_thc-ipv6-passive-discovery6 is not set # CONFIG_PACKAGE_thc-ipv6-randicmp6 is not set # CONFIG_PACKAGE_thc-ipv6-redir6 is not set # CONFIG_PACKAGE_thc-ipv6-rsmurf6 is not set # CONFIG_PACKAGE_thc-ipv6-sendpees6 is not set # CONFIG_PACKAGE_thc-ipv6-sendpeesmp6 is not set # CONFIG_PACKAGE_thc-ipv6-smurf6 is not set # CONFIG_PACKAGE_thc-ipv6-thcping6 is not set # CONFIG_PACKAGE_thc-ipv6-toobig6 is not set # CONFIG_PACKAGE_thc-ipv6-trace6 is not set  # # Time Synchronization # # CONFIG_PACKAGE_chrony is not set # CONFIG_PACKAGE_htpdate is not set # CONFIG_PACKAGE_linuxptp is not set # CONFIG_PACKAGE_ntp-keygen is not set # CONFIG_PACKAGE_ntp-utils is not set # CONFIG_PACKAGE_ntpclient is not set # CONFIG_PACKAGE_ntpd is not set # CONFIG_PACKAGE_ntpdate is not set  # # VPN # # CONFIG_PACKAGE_chaosvpn is not set # CONFIG_PACKAGE_fastd is not set # CONFIG_PACKAGE_ipsec-tools is not set # CONFIG_PACKAGE_n2n-edge is not set # CONFIG_PACKAGE_n2n-supernode is not set # CONFIG_PACKAGE_ocserv is not set # CONFIG_PACKAGE_openconnect is not set # CONFIG_PACKAGE_opennhrp is not set # CONFIG_PACKAGE_openvpn-easy-rsa is not set # CONFIG_PACKAGE_openvpn-mbedtls is not set # CONFIG_PACKAGE_openvpn-nossl is not set # CONFIG_PACKAGE_openvpn-openssl is not set # CONFIG_PACKAGE_pptpd is not set # CONFIG_PACKAGE_softethervpn-base is not set # CONFIG_PACKAGE_softethervpn-bridge is not set # CONFIG_PACKAGE_softethervpn-client is not set # CONFIG_PACKAGE_softethervpn-server is not set # CONFIG_PACKAGE_softethervpn5-bridge is not set # CONFIG_PACKAGE_softethervpn5-client is not set # CONFIG_PACKAGE_softethervpn5-server is not set # CONFIG_PACKAGE_sstp-client is not set # CONFIG_PACKAGE_strongswan is not set # CONFIG_PACKAGE_strongswan-charon is not set # CONFIG_PACKAGE_strongswan-charon-cmd is not set # CONFIG_PACKAGE_strongswan-default is not set # CONFIG_PACKAGE_strongswan-ipsec is not set # CONFIG_PACKAGE_strongswan-isakmp is not set # CONFIG_PACKAGE_strongswan-libtls is not set # CONFIG_PACKAGE_strongswan-minimal is not set # CONFIG_PACKAGE_strongswan-mod-addrblock is not set # CONFIG_PACKAGE_strongswan-mod-aes is not set # CONFIG_PACKAGE_strongswan-mod-af-alg is not set # CONFIG_PACKAGE_strongswan-mod-agent is not set # CONFIG_PACKAGE_strongswan-mod-attr is not set # CONFIG_PACKAGE_strongswan-mod-attr-sql is not set # CONFIG_PACKAGE_strongswan-mod-blowfish is not set # CONFIG_PACKAGE_strongswan-mod-ccm is not set # CONFIG_PACKAGE_strongswan-mod-cmac is not set # CONFIG_PACKAGE_strongswan-mod-connmark is not set # CONFIG_PACKAGE_strongswan-mod-constraints is not set # CONFIG_PACKAGE_strongswan-mod-coupling is not set # CONFIG_PACKAGE_strongswan-mod-ctr is not set # CONFIG_PACKAGE_strongswan-mod-curl is not set # CONFIG_PACKAGE_strongswan-mod-curve25519 is not set # CONFIG_PACKAGE_strongswan-mod-des is not set # CONFIG_PACKAGE_strongswan-mod-dhcp is not set # CONFIG_PACKAGE_strongswan-mod-dnskey is not set # CONFIG_PACKAGE_strongswan-mod-duplicheck is not set # CONFIG_PACKAGE_strongswan-mod-eap-identity is not set # CONFIG_PACKAGE_strongswan-mod-eap-md5 is not set # CONFIG_PACKAGE_strongswan-mod-eap-mschapv2 is not set # CONFIG_PACKAGE_strongswan-mod-eap-radius is not set # CONFIG_PACKAGE_strongswan-mod-eap-tls is not set # CONFIG_PACKAGE_strongswan-mod-farp is not set # CONFIG_PACKAGE_strongswan-mod-fips-prf is not set # CONFIG_PACKAGE_strongswan-mod-forecast is not set # CONFIG_PACKAGE_strongswan-mod-gcm is not set # CONFIG_PACKAGE_strongswan-mod-gcrypt is not set # CONFIG_PACKAGE_strongswan-mod-gmp is not set # CONFIG_PACKAGE_strongswan-mod-gmpdh is not set # CONFIG_PACKAGE_strongswan-mod-ha is not set # CONFIG_PACKAGE_strongswan-mod-hmac is not set # CONFIG_PACKAGE_strongswan-mod-kernel-libipsec is not set # CONFIG_PACKAGE_strongswan-mod-kernel-netlink is not set # CONFIG_PACKAGE_strongswan-mod-ldap is not set # CONFIG_PACKAGE_strongswan-mod-led is not set # CONFIG_PACKAGE_strongswan-mod-load-tester is not set # CONFIG_PACKAGE_strongswan-mod-md4 is not set # CONFIG_PACKAGE_strongswan-mod-md5 is not set # CONFIG_PACKAGE_strongswan-mod-mysql is not set # CONFIG_PACKAGE_strongswan-mod-nonce is not set # CONFIG_PACKAGE_strongswan-mod-openssl is not set # CONFIG_PACKAGE_strongswan-mod-pem is not set # CONFIG_PACKAGE_strongswan-mod-pgp is not set # CONFIG_PACKAGE_strongswan-mod-pkcs1 is not set # CONFIG_PACKAGE_strongswan-mod-pkcs11 is not set # CONFIG_PACKAGE_strongswan-mod-pkcs12 is not set # CONFIG_PACKAGE_strongswan-mod-pkcs7 is not set # CONFIG_PACKAGE_strongswan-mod-pkcs8 is not set # CONFIG_PACKAGE_strongswan-mod-pubkey is not set # CONFIG_PACKAGE_strongswan-mod-random is not set # CONFIG_PACKAGE_strongswan-mod-rc2 is not set # CONFIG_PACKAGE_strongswan-mod-resolve is not set # CONFIG_PACKAGE_strongswan-mod-revocation is not set # CONFIG_PACKAGE_strongswan-mod-sha1 is not set # CONFIG_PACKAGE_strongswan-mod-sha2 is not set # CONFIG_PACKAGE_strongswan-mod-smp is not set # CONFIG_PACKAGE_strongswan-mod-socket-default is not set # CONFIG_PACKAGE_strongswan-mod-socket-dynamic is not set # CONFIG_PACKAGE_strongswan-mod-sql is not set # CONFIG_PACKAGE_strongswan-mod-sqlite is not set # CONFIG_PACKAGE_strongswan-mod-sshkey is not set # CONFIG_PACKAGE_strongswan-mod-stroke is not set # CONFIG_PACKAGE_strongswan-mod-test-vectors is not set # CONFIG_PACKAGE_strongswan-mod-uci is not set # CONFIG_PACKAGE_strongswan-mod-unity is not set # CONFIG_PACKAGE_strongswan-mod-updown is not set # CONFIG_PACKAGE_strongswan-mod-vici is not set # CONFIG_PACKAGE_strongswan-mod-whitelist is not set # CONFIG_PACKAGE_strongswan-mod-x509 is not set # CONFIG_PACKAGE_strongswan-mod-xauth-eap is not set # CONFIG_PACKAGE_strongswan-mod-xauth-generic is not set # CONFIG_PACKAGE_strongswan-mod-xcbc is not set # CONFIG_PACKAGE_strongswan-pki is not set # CONFIG_PACKAGE_strongswan-scepclient is not set # CONFIG_PACKAGE_strongswan-swanctl is not set # CONFIG_PACKAGE_tinc is not set # CONFIG_PACKAGE_uanytun is not set # CONFIG_PACKAGE_uanytun-nettle is not set # CONFIG_PACKAGE_uanytun-nocrypt is not set # CONFIG_PACKAGE_uanytun-sslcrypt is not set # CONFIG_PACKAGE_vpnc is not set # CONFIG_PACKAGE_vpnc-scripts is not set # CONFIG_PACKAGE_wireguard is not set # CONFIG_PACKAGE_xl2tpd is not set # CONFIG_PACKAGE_zerotier is not set  # # Version Control Systems # # CONFIG_PACKAGE_fossil is not set # CONFIG_PACKAGE_git is not set # CONFIG_PACKAGE_git-http is not set # CONFIG_PACKAGE_subversion-client is not set # CONFIG_PACKAGE_subversion-libs is not set # CONFIG_PACKAGE_subversion-server is not set  # # WWAN # # CONFIG_PACKAGE_adb-enablemodem is not set # CONFIG_PACKAGE_comgt is not set # CONFIG_PACKAGE_comgt-directip is not set # CONFIG_PACKAGE_comgt-ncm is not set # CONFIG_PACKAGE_uqmi is not set  # # Web Servers/Proxies # # CONFIG_PACKAGE_apache is not set # CONFIG_PACKAGE_cgi-io is not set # CONFIG_PACKAGE_clamav is not set # CONFIG_PACKAGE_e2guardian is not set # CONFIG_PACKAGE_freshclam is not set # CONFIG_PACKAGE_frpc is not set # CONFIG_PACKAGE_frps is not set # CONFIG_PACKAGE_haproxy is not set # CONFIG_PACKAGE_haproxy-nossl is not set # CONFIG_PACKAGE_kcptun-client is not set # CONFIG_PACKAGE_kcptun-server is not set # CONFIG_PACKAGE_lighttpd is not set # CONFIG_PACKAGE_nginx is not set # CONFIG_PACKAGE_nginx-all-module is not set # CONFIG_PACKAGE_nginx-mod-luci is not set # CONFIG_PACKAGE_nginx-mod-luci-ssl is not set # CONFIG_PACKAGE_nginx-ssl is not set CONFIG_PACKAGE_pdnsd-alt=y # CONFIG_PACKAGE_polipo is not set # CONFIG_PACKAGE_privoxy is not set # CONFIG_PACKAGE_radicale-py2 is not set # CONFIG_PACKAGE_radicale-py3 is not set # CONFIG_PACKAGE_radicale2 is not set # CONFIG_PACKAGE_radicale2-examples is not set # CONFIG_PACKAGE_radicale2-src is not set # CONFIG_PACKAGE_shadowsocks-client is not set # CONFIG_PACKAGE_shadowsocks-libev-ss-local is not set # CONFIG_PACKAGE_shadowsocks-libev-ss-redir is not set # CONFIG_PACKAGE_shadowsocks-libev-ss-rules is not set # CONFIG_PACKAGE_shadowsocks-libev-ss-server is not set # CONFIG_PACKAGE_shadowsocks-libev-ss-tunnel is not set # CONFIG_PACKAGE_shadowsocksr-libev is not set # CONFIG_PACKAGE_shadowsocksr-libev-alt is not set # CONFIG_PACKAGE_shadowsocksr-libev-server is not set # CONFIG_PACKAGE_shadowsocksr-libev-ssr-local is not set # CONFIG_PACKAGE_sockd is not set # CONFIG_PACKAGE_socksify is not set # CONFIG_PACKAGE_spawn-fcgi is not set # CONFIG_PACKAGE_squid is not set # CONFIG_PACKAGE_srelay is not set # CONFIG_PACKAGE_tinyproxy is not set CONFIG_PACKAGE_uhttpd=y # CONFIG_PACKAGE_uhttpd-mod-lua is not set CONFIG_PACKAGE_uhttpd-mod-ubus=y # CONFIG_PACKAGE_uwsgi-cgi is not set # CONFIG_PACKAGE_uwsgi-cgi-luci-support is not set  # # Wireless # # CONFIG_PACKAGE_aircrack-ng is not set # CONFIG_PACKAGE_airmon-ng is not set  # # dial-in/up # # CONFIG_PACKAGE_rp-pppoe-common is not set # CONFIG_PACKAGE_rp-pppoe-relay is not set # CONFIG_PACKAGE_rp-pppoe-server is not set  # # tcprelay # # CONFIG_PACKAGE_tcpbridge is not set # CONFIG_PACKAGE_tcpcapinfo is not set # CONFIG_PACKAGE_tcpliveplay is not set # CONFIG_PACKAGE_tcpprep is not set # CONFIG_PACKAGE_tcpreplay is not set # CONFIG_PACKAGE_tcpreplay-all is not set # CONFIG_PACKAGE_tcpreplay-edit is not set # CONFIG_PACKAGE_tcprewrite is not set  # # wireless # # CONFIG_PACKAGE_dynapoint is not set # CONFIG_PACKAGE_horst is not set # CONFIG_PACKAGE_kismet-client is not set # CONFIG_PACKAGE_kismet-drone is not set # CONFIG_PACKAGE_kismet-server is not set # CONFIG_PACKAGE_pixiewps is not set # CONFIG_PACKAGE_reaver is not set # CONFIG_PACKAGE_wavemon is not set # CONFIG_PACKAGE_wifischedule is not set # CONFIG_PACKAGE_464xlat is not set # CONFIG_PACKAGE_6in4 is not set # CONFIG_PACKAGE_6rd is not set # CONFIG_PACKAGE_6to4 is not set # CONFIG_PACKAGE_acme is not set # CONFIG_PACKAGE_acme-dnsapi is not set # CONFIG_PACKAGE_adblock is not set # CONFIG_PACKAGE_adbyby is not set # CONFIG_PACKAGE_addrwatch is not set # CONFIG_PACKAGE_ahcpd is not set # CONFIG_PACKAGE_alfred is not set # CONFIG_PACKAGE_apcupsd is not set # CONFIG_PACKAGE_apcupsd-cgi is not set # CONFIG_PACKAGE_apinger is not set # CONFIG_PACKAGE_arp-scan is not set # CONFIG_PACKAGE_baidupcs-web is not set # CONFIG_PACKAGE_banip is not set # CONFIG_PACKAGE_batctl-default is not set # CONFIG_PACKAGE_batctl-full is not set # CONFIG_PACKAGE_batctl-tiny is not set # CONFIG_PACKAGE_beanstalkd is not set # CONFIG_PACKAGE_bmon is not set # CONFIG_PACKAGE_bwm-ng is not set # CONFIG_PACKAGE_chat is not set # CONFIG_PACKAGE_cifsmount is not set # CONFIG_PACKAGE_coap-server is not set # CONFIG_PACKAGE_conserver is not set # CONFIG_PACKAGE_cshark is not set # CONFIG_PACKAGE_daemonlogger is not set # CONFIG_PACKAGE_darkstat is not set # CONFIG_PACKAGE_dhcpcd is not set # CONFIG_PACKAGE_dmapd is not set # CONFIG_PACKAGE_ds-lite is not set # CONFIG_PACKAGE_dsmboot is not set # CONFIG_PACKAGE_eapol-test is not set # CONFIG_PACKAGE_eapol-test-openssl is not set # CONFIG_PACKAGE_eapol-test-wolfssl is not set # CONFIG_PACKAGE_esniper is not set # CONFIG_PACKAGE_etherwake is not set # CONFIG_PACKAGE_ethtool is not set # CONFIG_PACKAGE_fakeidentd is not set # CONFIG_PACKAGE_foolsm is not set # CONFIG_PACKAGE_fping is not set # CONFIG_PACKAGE_gnunet is not set # CONFIG_PACKAGE_gre is not set # CONFIG_PACKAGE_hnet-full is not set # CONFIG_PACKAGE_hnet-full-l2tp is not set # CONFIG_PACKAGE_hnet-full-secure is not set # CONFIG_PACKAGE_hnetd-nossl is not set # CONFIG_PACKAGE_hnetd-openssl is not set # CONFIG_PACKAGE_hostapd is not set # CONFIG_PACKAGE_hostapd-basic is not set CONFIG_PACKAGE_hostapd-common=y # CONFIG_PACKAGE_hostapd-mini is not set # CONFIG_PACKAGE_hostapd-openssl is not set # CONFIG_PACKAGE_hostapd-utils is not set # CONFIG_PACKAGE_hostapd-wolfssl is not set # CONFIG_PACKAGE_httping is not set # CONFIG_PACKAGE_httping-nossl is not set # CONFIG_PACKAGE_https_dns_proxy is not set # CONFIG_PACKAGE_i2pd is not set # CONFIG_PACKAGE_ibrdtn-tools is not set # CONFIG_PACKAGE_ibrdtnd is not set # CONFIG_PACKAGE_ifstat is not set # CONFIG_PACKAGE_iftop is not set # CONFIG_PACKAGE_iiod is not set # CONFIG_PACKAGE_iotivity is not set # CONFIG_PACKAGE_iotivity-cpp is not set # CONFIG_PACKAGE_iotivity-example-garage is not set # CONFIG_PACKAGE_iotivity-example-simple is not set # CONFIG_PACKAGE_iotivity-oic-middle is not set # CONFIG_PACKAGE_iotivity-resource-container-hue is not set # CONFIG_PACKAGE_iotivity-resource-container-lib is not set # CONFIG_PACKAGE_iotivity-resource-container-sample is not set # CONFIG_PACKAGE_iotivity-resource-directory-lib is not set # CONFIG_PACKAGE_iperf is not set # CONFIG_PACKAGE_iperf3 is not set # CONFIG_PACKAGE_iperf3-ssl is not set # CONFIG_PACKAGE_ipip is not set CONFIG_PACKAGE_ipset=y # CONFIG_PACKAGE_ipset-dns is not set # CONFIG_PACKAGE_ipset-lists is not set # CONFIG_PACKAGE_ipt2socks is not set # CONFIG_PACKAGE_iptraf-ng is not set # CONFIG_PACKAGE_iputils-arping is not set # CONFIG_PACKAGE_iputils-clockdiff is not set # CONFIG_PACKAGE_iputils-ping is not set # CONFIG_PACKAGE_iputils-ping6 is not set # CONFIG_PACKAGE_iputils-tftpd is not set # CONFIG_PACKAGE_iputils-tracepath is not set # CONFIG_PACKAGE_iputils-tracepath6 is not set # CONFIG_PACKAGE_iputils-traceroute6 is not set CONFIG_PACKAGE_iw=y # CONFIG_PACKAGE_iw-full is not set # CONFIG_PACKAGE_jool is not set # CONFIG_PACKAGE_jool-tools is not set # CONFIG_PACKAGE_keepalived is not set # CONFIG_PACKAGE_knxd is not set # CONFIG_PACKAGE_kplex is not set # CONFIG_PACKAGE_krb5-client is not set # CONFIG_PACKAGE_krb5-libs is not set # CONFIG_PACKAGE_krb5-server is not set CONFIG_PACKAGE_libipset=y # CONFIG_PACKAGE_linknx is not set # CONFIG_PACKAGE_lispd is not set # CONFIG_PACKAGE_mac-telnet-client is not set # CONFIG_PACKAGE_mac-telnet-discover is not set # CONFIG_PACKAGE_mac-telnet-ping is not set # CONFIG_PACKAGE_mac-telnet-server is not set # CONFIG_PACKAGE_map is not set # CONFIG_PACKAGE_memcached is not set # CONFIG_PACKAGE_microsocks is not set # CONFIG_PACKAGE_mii-tool is not set # CONFIG_PACKAGE_mikrotik-btest is not set # CONFIG_PACKAGE_mini_snmpd is not set # CONFIG_PACKAGE_minimalist-pcproxy is not set # CONFIG_PACKAGE_modemmanager is not set # CONFIG_PACKAGE_mosquitto-client-nossl is not set # CONFIG_PACKAGE_mosquitto-client-ssl is not set # CONFIG_PACKAGE_mosquitto-nossl is not set # CONFIG_PACKAGE_mosquitto-ssl is not set # CONFIG_PACKAGE_mrd6 is not set # CONFIG_PACKAGE_mtr is not set # CONFIG_PACKAGE_nbd is not set # CONFIG_PACKAGE_nbd-server is not set # CONFIG_PACKAGE_ncp is not set # CONFIG_PACKAGE_ndppd is not set # CONFIG_PACKAGE_netcat is not set # CONFIG_PACKAGE_netdiscover is not set # CONFIG_PACKAGE_netperf is not set # CONFIG_PACKAGE_nextdns is not set CONFIG_PACKAGE_nlbwmon=y # CONFIG_PACKAGE_noddos is not set # CONFIG_PACKAGE_noping is not set # CONFIG_PACKAGE_npc is not set # CONFIG_PACKAGE_nut is not set # CONFIG_PACKAGE_obfsproxy is not set # CONFIG_PACKAGE_obfsproxy-src is not set # CONFIG_PACKAGE_odhcp6c is not set # CONFIG_PACKAGE_odhcpd is not set # CONFIG_PACKAGE_odhcpd-ipv6only is not set # CONFIG_PACKAGE_ola is not set # CONFIG_PACKAGE_omcproxy is not set # CONFIG_PACKAGE_oping is not set # CONFIG_PACKAGE_pagekitec is not set # CONFIG_PACKAGE_pen is not set # CONFIG_PACKAGE_pimbd is not set # CONFIG_PACKAGE_pingcheck is not set # CONFIG_PACKAGE_port-mirroring is not set # CONFIG_PACKAGE_portmap is not set CONFIG_PACKAGE_ppp=y # CONFIG_PACKAGE_ppp-mod-passwordfd is not set # CONFIG_PACKAGE_ppp-mod-pppoa is not set CONFIG_PACKAGE_ppp-mod-pppoe=y # CONFIG_PACKAGE_ppp-mod-pppol2tp is not set # CONFIG_PACKAGE_ppp-mod-pptp is not set # CONFIG_PACKAGE_ppp-mod-radius is not set # CONFIG_PACKAGE_ppp-multilink is not set # CONFIG_PACKAGE_pppdump is not set # CONFIG_PACKAGE_pppoe-discovery is not set # CONFIG_PACKAGE_pppossh is not set # CONFIG_PACKAGE_pppstats is not set # CONFIG_PACKAGE_proto-bonding is not set # CONFIG_PACKAGE_proxychains-ng is not set # CONFIG_PACKAGE_radsecproxy is not set # CONFIG_PACKAGE_redsocks is not set # CONFIG_PACKAGE_redsocks2 is not set # CONFIG_PACKAGE_remserial is not set # CONFIG_PACKAGE_rpcbind is not set # CONFIG_PACKAGE_rssileds is not set # CONFIG_PACKAGE_rsyslog is not set # CONFIG_PACKAGE_samba36-client is not set # CONFIG_PACKAGE_samba36-net is not set # CONFIG_PACKAGE_samba36-server is not set # CONFIG_PACKAGE_samba4-admin is not set # CONFIG_PACKAGE_samba4-client is not set # CONFIG_PACKAGE_samba4-libs is not set # CONFIG_PACKAGE_samba4-server is not set # CONFIG_PACKAGE_samba4-utils is not set # CONFIG_PACKAGE_scapy is not set # CONFIG_PACKAGE_sctp is not set # CONFIG_PACKAGE_sctp-tools is not set # CONFIG_PACKAGE_seafile-ccnet is not set # CONFIG_PACKAGE_seafile-seahub is not set # CONFIG_PACKAGE_seafile-seahub-src is not set # CONFIG_PACKAGE_ser2net is not set # CONFIG_PACKAGE_simple-adblock is not set # CONFIG_PACKAGE_simple-obfs is not set # CONFIG_PACKAGE_simple-obfs-server is not set # CONFIG_PACKAGE_smartsnmpd is not set # CONFIG_PACKAGE_snmp-mibs is not set # CONFIG_PACKAGE_snmp-utils is not set # CONFIG_PACKAGE_snmpd is not set # CONFIG_PACKAGE_snmpd-static is not set # CONFIG_PACKAGE_snmptrapd is not set # CONFIG_PACKAGE_socat is not set # CONFIG_PACKAGE_softflowd is not set # CONFIG_PACKAGE_soloscli is not set # CONFIG_PACKAGE_stunnel is not set # CONFIG_PACKAGE_tayga is not set # CONFIG_PACKAGE_tcpdump is not set # CONFIG_PACKAGE_tcpdump-mini is not set # CONFIG_PACKAGE_tcpping is not set # CONFIG_PACKAGE_tor is not set # CONFIG_PACKAGE_tor-gencert is not set # CONFIG_PACKAGE_tor-geoip is not set # CONFIG_PACKAGE_tor-resolve is not set # CONFIG_PACKAGE_travelmate is not set # CONFIG_PACKAGE_trojan is not set # CONFIG_PACKAGE_u2pnpd is not set CONFIG_PACKAGE_uclient-fetch=y # CONFIG_PACKAGE_udpxy is not set # CONFIG_PACKAGE_ulogd is not set # CONFIG_PACKAGE_umbim is not set # CONFIG_PACKAGE_umdns is not set # CONFIG_PACKAGE_usbip is not set # CONFIG_PACKAGE_vallumd is not set # CONFIG_PACKAGE_verysync is not set CONFIG_PACKAGE_vlmcsd=y # CONFIG_PACKAGE_vncrepeater is not set # CONFIG_PACKAGE_vnstat is not set # CONFIG_PACKAGE_vpnbypass is not set # CONFIG_PACKAGE_vsc7385-ucode-pb44 is not set # CONFIG_PACKAGE_vsc7395-ucode-pb44 is not set # CONFIG_PACKAGE_vti is not set # CONFIG_PACKAGE_vxlan is not set # CONFIG_PACKAGE_wakeonlan is not set # CONFIG_PACKAGE_wpa-cli is not set # CONFIG_PACKAGE_wpa-supplicant is not set # CONFIG_WPA_RFKILL_SUPPORT is not set CONFIG_WPA_MSG_MIN_PRIORITY=3 # CONFIG_WPA_WOLFSSL is not set # CONFIG_DRIVER_WEXT_SUPPORT is not set CONFIG_DRIVER_11N_SUPPORT=y CONFIG_DRIVER_11AC_SUPPORT=y CONFIG_DRIVER_11W_SUPPORT=y # CONFIG_PACKAGE_wpa-supplicant-basic is not set # CONFIG_PACKAGE_wpa-supplicant-mesh-openssl is not set # CONFIG_PACKAGE_wpa-supplicant-mesh-wolfssl is not set # CONFIG_PACKAGE_wpa-supplicant-mini is not set # CONFIG_PACKAGE_wpa-supplicant-openssl is not set # CONFIG_PACKAGE_wpa-supplicant-p2p is not set # CONFIG_PACKAGE_wpa-supplicant-wolfssl is not set # CONFIG_PACKAGE_wpad is not set # CONFIG_PACKAGE_wpad-basic is not set # CONFIG_PACKAGE_wpad-mesh-openssl is not set # CONFIG_PACKAGE_wpad-mesh-wolfssl is not set # CONFIG_PACKAGE_wpad-mini is not set CONFIG_PACKAGE_wpad-openssl=y # CONFIG_PACKAGE_wpad-wolfssl is not set # CONFIG_PACKAGE_wpan-tools is not set # CONFIG_PACKAGE_wwan is not set # CONFIG_PACKAGE_xinetd is not set  # # Sound # # CONFIG_PACKAGE_alsa-utils is not set # CONFIG_PACKAGE_alsa-utils-seq is not set # CONFIG_PACKAGE_alsa-utils-tests is not set # CONFIG_PACKAGE_espeak is not set # CONFIG_PACKAGE_faad2 is not set # CONFIG_PACKAGE_fdk-aac is not set # CONFIG_PACKAGE_forked-daapd is not set # CONFIG_PACKAGE_ices is not set # CONFIG_PACKAGE_lame is not set # CONFIG_PACKAGE_lame-lib is not set # CONFIG_PACKAGE_liblo-utils is not set # CONFIG_PACKAGE_madplay is not set # CONFIG_PACKAGE_madplay-alsa is not set # CONFIG_PACKAGE_moc is not set # CONFIG_PACKAGE_mpc is not set # CONFIG_PACKAGE_mpd-avahi-service is not set # CONFIG_PACKAGE_mpd-full is not set # CONFIG_PACKAGE_mpd-mini is not set # CONFIG_PACKAGE_mpg123 is not set # CONFIG_PACKAGE_opus-tools is not set # CONFIG_PACKAGE_pianod is not set # CONFIG_PACKAGE_pianod-client is not set # CONFIG_PACKAGE_portaudio is not set # CONFIG_PACKAGE_pulseaudio-daemon is not set # CONFIG_PACKAGE_pulseaudio-daemon-avahi is not set # CONFIG_PACKAGE_shairplay is not set # CONFIG_PACKAGE_shairport-sync-mbedtls is not set # CONFIG_PACKAGE_shairport-sync-mini is not set # CONFIG_PACKAGE_shairport-sync-openssl is not set # CONFIG_PACKAGE_shine is not set # CONFIG_PACKAGE_sox is not set # CONFIG_PACKAGE_squeezelite-full is not set # CONFIG_PACKAGE_squeezelite-mini is not set # CONFIG_PACKAGE_svox is not set # CONFIG_PACKAGE_upmpdcli is not set  # # Utilities #  # # Boot Loaders # # CONFIG_PACKAGE_fconfig is not set # CONFIG_PACKAGE_rbcfg is not set CONFIG_PACKAGE_uboot-envtools=y  # # Compression # # CONFIG_PACKAGE_bsdtar is not set # CONFIG_PACKAGE_bzip2 is not set # CONFIG_PACKAGE_gzip is not set # CONFIG_PACKAGE_pigz is not set # CONFIG_PACKAGE_unrar is not set # CONFIG_PACKAGE_unzip is not set # CONFIG_PACKAGE_xz-utils is not set # CONFIG_PACKAGE_zip is not set # CONFIG_PACKAGE_zstd is not set  # # Database # # CONFIG_PACKAGE_mariadb-common is not set  # # Disc # # CONFIG_PACKAGE_blkdiscard is not set # CONFIG_PACKAGE_blkid is not set # CONFIG_PACKAGE_blockdev is not set # CONFIG_PACKAGE_cfdisk is not set # CONFIG_PACKAGE_cgdisk is not set # CONFIG_PACKAGE_eject is not set # CONFIG_PACKAGE_fdisk is not set # CONFIG_PACKAGE_findfs is not set # CONFIG_PACKAGE_fixparts is not set # CONFIG_PACKAGE_gdisk is not set # CONFIG_PACKAGE_hd-idle is not set # CONFIG_PACKAGE_hdparm is not set # CONFIG_PACKAGE_lsblk is not set # CONFIG_PACKAGE_lvm2 is not set # CONFIG_PACKAGE_mdadm is not set # CONFIG_PACKAGE_parted is not set # CONFIG_PACKAGE_partx-utils is not set # CONFIG_PACKAGE_sfdisk is not set # CONFIG_PACKAGE_sgdisk is not set # CONFIG_PACKAGE_wipefs is not set  # # Editors # # CONFIG_PACKAGE_joe is not set # CONFIG_PACKAGE_nano is not set # CONFIG_PACKAGE_vim is not set # CONFIG_PACKAGE_vim-full is not set # CONFIG_PACKAGE_vim-fuller is not set # CONFIG_PACKAGE_vim-help is not set # CONFIG_PACKAGE_vim-runtime is not set # CONFIG_PACKAGE_zile is not set  # # Encryption # # CONFIG_PACKAGE_ccrypt is not set # CONFIG_PACKAGE_certtool is not set # CONFIG_PACKAGE_cryptsetup is not set # CONFIG_PACKAGE_cryptsetup-openssl is not set # CONFIG_PACKAGE_gnupg is not set # CONFIG_PACKAGE_gnutls-utils is not set # CONFIG_PACKAGE_gpgv is not set # CONFIG_PACKAGE_keyctl is not set # CONFIG_PACKAGE_px5g-mbedtls is not set # CONFIG_PACKAGE_px5g-standalone is not set # CONFIG_PACKAGE_stoken is not set  # # Filesystem # # CONFIG_PACKAGE_acl is not set # CONFIG_PACKAGE_antfs-mount is not set # CONFIG_PACKAGE_attr is not set # CONFIG_PACKAGE_badblocks is not set # CONFIG_PACKAGE_btrfs-progs is not set # CONFIG_PACKAGE_chattr is not set # CONFIG_PACKAGE_debugfs is not set # CONFIG_PACKAGE_dosfstools is not set # CONFIG_PACKAGE_dumpe2fs is not set # CONFIG_PACKAGE_e2freefrag is not set # CONFIG_PACKAGE_e2fsprogs is not set # CONFIG_PACKAGE_f2fs-tools is not set # CONFIG_PACKAGE_f2fsck is not set # CONFIG_PACKAGE_filefrag is not set # CONFIG_PACKAGE_fstrim is not set # CONFIG_PACKAGE_fuse-utils is not set # CONFIG_PACKAGE_hfsfsck is not set # CONFIG_PACKAGE_lsattr is not set # CONFIG_PACKAGE_mkf2fs is not set # CONFIG_PACKAGE_mkhfs is not set # CONFIG_PACKAGE_ncdu is not set # CONFIG_PACKAGE_nfs-utils is not set # CONFIG_PACKAGE_nfs-utils-libs is not set # CONFIG_PACKAGE_ntfs-3g is not set # CONFIG_PACKAGE_ntfs-3g-low is not set # CONFIG_PACKAGE_ntfs-3g-utils is not set # CONFIG_PACKAGE_owfs is not set # CONFIG_PACKAGE_owshell is not set # CONFIG_PACKAGE_resize2fs is not set # CONFIG_PACKAGE_squashfs-tools-mksquashfs is not set # CONFIG_PACKAGE_squashfs-tools-unsquashfs is not set # CONFIG_PACKAGE_swap-utils is not set # CONFIG_PACKAGE_sysfsutils is not set # CONFIG_PACKAGE_tune2fs is not set # CONFIG_PACKAGE_xfs-admin is not set # CONFIG_PACKAGE_xfs-fsck is not set # CONFIG_PACKAGE_xfs-growfs is not set # CONFIG_PACKAGE_xfs-mkfs is not set  # # Image Manipulation # # CONFIG_PACKAGE_jpeg-tools is not set # CONFIG_PACKAGE_tiff-utils is not set  # # Microcontroller programming # # CONFIG_PACKAGE_avrdude is not set # CONFIG_PACKAGE_dfu-programmer is not set # CONFIG_PACKAGE_stm32flash is not set  # # RTKLIB Suite # # CONFIG_PACKAGE_convbin is not set # CONFIG_PACKAGE_pos2kml is not set # CONFIG_PACKAGE_rnx2rtkp is not set # CONFIG_PACKAGE_rtkrcv is not set # CONFIG_PACKAGE_str2str is not set  # # Shells # # CONFIG_PACKAGE_bash is not set # CONFIG_PACKAGE_klish is not set # CONFIG_PACKAGE_mksh is not set # CONFIG_PACKAGE_tcsh is not set # CONFIG_PACKAGE_zsh is not set  # # Terminal # # CONFIG_PACKAGE_agetty is not set # CONFIG_PACKAGE_dvtm is not set # CONFIG_PACKAGE_minicom is not set # CONFIG_PACKAGE_picocom is not set # CONFIG_PACKAGE_rtty-mbedtls is not set # CONFIG_PACKAGE_rtty-nossl is not set # CONFIG_PACKAGE_rtty-openssl is not set # CONFIG_PACKAGE_rtty-wolfssl is not set # CONFIG_PACKAGE_screen is not set # CONFIG_PACKAGE_script-utils is not set # CONFIG_PACKAGE_serialconsole is not set # CONFIG_PACKAGE_setterm is not set # CONFIG_PACKAGE_tio is not set # CONFIG_PACKAGE_tmux is not set # CONFIG_PACKAGE_ttyd is not set # CONFIG_PACKAGE_wall is not set  # # Virtualization #  # # Zoneinfo # # CONFIG_PACKAGE_zoneinfo-africa is not set # CONFIG_PACKAGE_zoneinfo-asia is not set # CONFIG_PACKAGE_zoneinfo-atlantic is not set # CONFIG_PACKAGE_zoneinfo-australia-nz is not set # CONFIG_PACKAGE_zoneinfo-core is not set # CONFIG_PACKAGE_zoneinfo-europe is not set # CONFIG_PACKAGE_zoneinfo-india is not set # CONFIG_PACKAGE_zoneinfo-northamerica is not set # CONFIG_PACKAGE_zoneinfo-pacific is not set # CONFIG_PACKAGE_zoneinfo-poles is not set # CONFIG_PACKAGE_zoneinfo-simple is not set # CONFIG_PACKAGE_zoneinfo-southamerica is not set  # # database # # CONFIG_PACKAGE_pgsql-cli is not set # CONFIG_PACKAGE_pgsql-cli-extra is not set # CONFIG_PACKAGE_pgsql-server is not set # CONFIG_PACKAGE_rrdcgi1 is not set # CONFIG_PACKAGE_rrdtool1 is not set # CONFIG_PACKAGE_sqlite3-cli is not set # CONFIG_PACKAGE_unixodbc-tools is not set # CONFIG_PACKAGE_adb is not set # CONFIG_PACKAGE_ap51-flash is not set # CONFIG_PACKAGE_at is not set # CONFIG_PACKAGE_bandwidthd is not set # CONFIG_PACKAGE_bandwidthd-pgsql is not set # CONFIG_PACKAGE_bandwidthd-php is not set # CONFIG_PACKAGE_bandwidthd-sqlite is not set # CONFIG_PACKAGE_banhostlist is not set # CONFIG_PACKAGE_bc is not set # CONFIG_PACKAGE_bluelog is not set # CONFIG_PACKAGE_bluez-daemon is not set # CONFIG_PACKAGE_bluez-utils is not set # CONFIG_PACKAGE_bluez-utils-extra is not set # CONFIG_PACKAGE_bonniexx is not set # CONFIG_PACKAGE_bsdiff is not set # CONFIG_PACKAGE_bspatch is not set # CONFIG_PACKAGE_cal is not set # CONFIG_PACKAGE_canutils is not set # CONFIG_PACKAGE_cgroupfs-mount is not set # CONFIG_PACKAGE_cmdpad is not set # CONFIG_PACKAGE_coap-client is not set # CONFIG_PACKAGE_collectd is not set CONFIG_PACKAGE_coremark=y # CONFIG_PACKAGE_coreutils is not set # CONFIG_PACKAGE_crconf is not set # CONFIG_PACKAGE_crelay is not set # CONFIG_PACKAGE_csstidy is not set # CONFIG_PACKAGE_ct-bugcheck is not set # CONFIG_PACKAGE_dbus is not set # CONFIG_PACKAGE_dfu-util is not set # CONFIG_PACKAGE_digitemp is not set # CONFIG_PACKAGE_digitemp-usb is not set # CONFIG_PACKAGE_dmesg is not set  # # Kernel features for Docker # CONFIG_DOCKER_KERNEL_OPTIONS=y # CONFIG_DOCKER_SECCOMP is not set # CONFIG_DOCKER_RES_SHAPE is not set  # # Network # # CONFIG_DOCKER_NET_OVERLAY is not set # CONFIG_DOCKER_NET_MACVLAN is not set # CONFIG_DOCKER_NET_TFTP is not set  # # Storage # # CONFIG_DOCKER_STO_EXT4 is not set # CONFIG_DOCKER_STO_BTRFS is not set # CONFIG_PACKAGE_dropbearconvert is not set # CONFIG_PACKAGE_dtc is not set # CONFIG_PACKAGE_dump1090 is not set # CONFIG_PACKAGE_ecdsautils is not set # CONFIG_PACKAGE_elektra-kdb is not set # CONFIG_PACKAGE_evtest is not set # CONFIG_PACKAGE_extract is not set # CONFIG_PACKAGE_fdt-utils is not set # CONFIG_PACKAGE_file is not set # CONFIG_PACKAGE_findutils-find is not set # CONFIG_PACKAGE_findutils-locate is not set # CONFIG_PACKAGE_findutils-xargs is not set # CONFIG_PACKAGE_flashrom is not set # CONFIG_PACKAGE_flashrom-pci is not set # CONFIG_PACKAGE_flashrom-spi is not set # CONFIG_PACKAGE_flashrom-usb is not set # CONFIG_PACKAGE_flent-tools is not set # CONFIG_PACKAGE_flock is not set # CONFIG_PACKAGE_fritz-caldata is not set # CONFIG_PACKAGE_fritz-tffs is not set # CONFIG_PACKAGE_ftdi_eeprom is not set # CONFIG_PACKAGE_gammu is not set # CONFIG_PACKAGE_gawk is not set # CONFIG_PACKAGE_getopt is not set # CONFIG_PACKAGE_giflib-utils is not set # CONFIG_PACKAGE_gkermit is not set # CONFIG_PACKAGE_gpioctl-sysfs is not set # CONFIG_PACKAGE_gpiod-tools is not set # CONFIG_PACKAGE_gpsd is not set # CONFIG_PACKAGE_gpsd-clients is not set # CONFIG_PACKAGE_grep is not set # CONFIG_PACKAGE_hamlib is not set # CONFIG_PACKAGE_haserl is not set # CONFIG_PACKAGE_haveged is not set # CONFIG_PACKAGE_hub-ctrl is not set # CONFIG_PACKAGE_hwclock is not set # CONFIG_PACKAGE_hwloc-utils is not set # CONFIG_PACKAGE_i2c-tools is not set # CONFIG_PACKAGE_iconv is not set # CONFIG_PACKAGE_iio-utils is not set # CONFIG_PACKAGE_inotifywait is not set # CONFIG_PACKAGE_inotifywatch is not set # CONFIG_PACKAGE_io is not set # CONFIG_PACKAGE_irqbalance is not set # CONFIG_PACKAGE_iwcap is not set # CONFIG_PACKAGE_iwinfo is not set # CONFIG_PACKAGE_jq is not set CONFIG_PACKAGE_jshn=y # CONFIG_PACKAGE_kmod is not set # CONFIG_PACKAGE_lcd4linux-custom is not set # CONFIG_PACKAGE_lcdproc-clients is not set # CONFIG_PACKAGE_lcdproc-drivers is not set # CONFIG_PACKAGE_lcdproc-server is not set # CONFIG_PACKAGE_less is not set # CONFIG_PACKAGE_less-wide is not set # CONFIG_PACKAGE_libimobiledevice-utils is not set CONFIG_PACKAGE_libjson-script=y # CONFIG_PACKAGE_libplist-utils is not set # CONFIG_PACKAGE_libsysrepo is not set # CONFIG_PACKAGE_libusbmuxd-utils is not set # CONFIG_PACKAGE_libxml2-utils is not set # CONFIG_PACKAGE_lm-sensors is not set # CONFIG_PACKAGE_lm-sensors-detect is not set # CONFIG_PACKAGE_logger is not set # CONFIG_PACKAGE_logrotate is not set # CONFIG_PACKAGE_look is not set # CONFIG_PACKAGE_losetup is not set # CONFIG_PACKAGE_lrzsz is not set # CONFIG_PACKAGE_lscpu is not set # CONFIG_PACKAGE_lsof is not set # CONFIG_PACKAGE_lxc is not set # CONFIG_PACKAGE_lxc-unprivileged is not set # CONFIG_PACKAGE_maccalc is not set # CONFIG_PACKAGE_macchanger is not set # CONFIG_PACKAGE_mbedtls-util is not set # CONFIG_PACKAGE_mbim-utils is not set # CONFIG_PACKAGE_mbtools is not set # CONFIG_PACKAGE_mc is not set # CONFIG_PACKAGE_mcookie is not set # CONFIG_PACKAGE_mmc-utils is not set # CONFIG_PACKAGE_moreutils is not set # CONFIG_PACKAGE_mount-utils is not set # CONFIG_PACKAGE_mpack is not set # CONFIG_PACKAGE_mt-st is not set # CONFIG_PACKAGE_namei is not set # CONFIG_PACKAGE_netopeer2-cli is not set # CONFIG_PACKAGE_netopeer2-keystored is not set # CONFIG_PACKAGE_netopeer2-server is not set # CONFIG_PACKAGE_netwhere is not set # CONFIG_PACKAGE_nsenter is not set # CONFIG_PACKAGE_oath-toolkit is not set # CONFIG_PACKAGE_open-plc-utils is not set # CONFIG_PACKAGE_open2300 is not set # CONFIG_PACKAGE_openobex is not set # CONFIG_PACKAGE_openobex-apps is not set # CONFIG_PACKAGE_openocd is not set # CONFIG_PACKAGE_opensc-utils is not set CONFIG_PACKAGE_openssl-util=y # CONFIG_PACKAGE_openzwave is not set # CONFIG_PACKAGE_openzwave-config is not set # CONFIG_PACKAGE_owipcalc is not set # CONFIG_PACKAGE_pciutils is not set # CONFIG_PACKAGE_pcsc-tools is not set # CONFIG_PACKAGE_pcscd is not set # CONFIG_PACKAGE_pps-tools is not set # CONFIG_PACKAGE_prlimit is not set # CONFIG_PACKAGE_procps-ng is not set # CONFIG_PACKAGE_progress is not set # CONFIG_PACKAGE_prometheus-node-exporter-lua is not set # CONFIG_PACKAGE_pv is not set # CONFIG_PACKAGE_qmi-utils is not set # CONFIG_PACKAGE_qrencode is not set # CONFIG_PACKAGE_rclone is not set # CONFIG_PACKAGE_relayctl is not set # CONFIG_PACKAGE_rename is not set # CONFIG_PACKAGE_rng-tools is not set # CONFIG_PACKAGE_rtl-ais is not set # CONFIG_PACKAGE_rtl-sdr is not set # CONFIG_PACKAGE_rtl_433 is not set # CONFIG_PACKAGE_sane-backends is not set # CONFIG_PACKAGE_sane-daemon is not set # CONFIG_PACKAGE_sane-frontends is not set # CONFIG_PACKAGE_setserial is not set # CONFIG_PACKAGE_shadow-utils is not set CONFIG_PACKAGE_shellsync=y # CONFIG_PACKAGE_sispmctl is not set # CONFIG_PACKAGE_slide-switch is not set # CONFIG_PACKAGE_smartd is not set # CONFIG_PACKAGE_smartmontools is not set # CONFIG_PACKAGE_smartmontools-drivedb is not set # CONFIG_PACKAGE_smstools3 is not set # CONFIG_PACKAGE_sockread is not set # CONFIG_PACKAGE_spi-tools is not set # CONFIG_PACKAGE_spidev-test is not set # CONFIG_PACKAGE_strace is not set CONFIG_STRACE_NONE=y # CONFIG_STRACE_LIBDW is not set # CONFIG_STRACE_LIBUNWIND is not set # CONFIG_PACKAGE_stress is not set # CONFIG_PACKAGE_sumo is not set # CONFIG_PACKAGE_sysrepo is not set # CONFIG_PACKAGE_sysrepocfg is not set # CONFIG_PACKAGE_sysrepoctl is not set # CONFIG_PACKAGE_sysstat is not set # CONFIG_PACKAGE_tar is not set # CONFIG_PACKAGE_taskwarrior is not set # CONFIG_PACKAGE_tini is not set # CONFIG_PACKAGE_tracertools is not set # CONFIG_PACKAGE_tree is not set # CONFIG_PACKAGE_triggerhappy is not set # CONFIG_PACKAGE_udns-dnsget is not set # CONFIG_PACKAGE_udns-ex-rdns is not set # CONFIG_PACKAGE_udns-rblcheck is not set # CONFIG_PACKAGE_ugps is not set # CONFIG_PACKAGE_uledd is not set # CONFIG_PACKAGE_unshare is not set # CONFIG_PACKAGE_usb-modeswitch is not set # CONFIG_PACKAGE_usbmuxd is not set # CONFIG_PACKAGE_usbreset is not set # CONFIG_PACKAGE_usbutils is not set # CONFIG_PACKAGE_uuidd is not set # CONFIG_PACKAGE_uuidgen is not set # CONFIG_PACKAGE_uvcdynctrl is not set # CONFIG_PACKAGE_v4l-utils is not set # CONFIG_PACKAGE_view1090 is not set # CONFIG_PACKAGE_watchcat is not set # CONFIG_PACKAGE_whereis is not set # CONFIG_PACKAGE_wifitoggle is not set # CONFIG_PACKAGE_xsltproc is not set # CONFIG_PACKAGE_xxd is not set # CONFIG_PACKAGE_yanglint is not set # CONFIG_PACKAGE_yara is not set # CONFIG_PACKAGE_yunbridge is not set  # # Xorg #  # # font-utils # # CONFIG_PACKAGE_fontconfig is not set
- Уровень: automated README evidence extraction
- Снимок: [sources/zszszszsz__.config/README.md](sources/zszszszsz__.config/README.md); SHA-256: `68f2db7e1deb66266027415a68b22057ac5fec575d4309b06def102c21a9d04e`

## linhay/harmony-next.skills

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/linhay/harmony-next.skills
- Категория: benchmark/testing candidate
- Описание: 🚀 Expert guidance for HarmonyOS NEXT (API 12+) development. Covers IDE operations, performance tuning, architecture (HAP/HAR/HSP), and automation testing.
- Уровень: automated README evidence extraction
- Снимок: [sources/linhay__harmony-next.skills/README.md](sources/linhay__harmony-next.skills/README.md); SHA-256: `a8a1c63135e1a27cef0ab5019b48cceb2c7a64c8e242d1819dcf9ce7da776fdb`
- report: строка 132: Bundle 只注册 `harmony-next` skill 及其离线参考资源，不安装 MCP、tools 或 apps。DSH bundle 的 manifest 是根目录的 `package.json`，patch 是 `cordis.patch.yml`。

## bartoszlenar/Validot

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/bartoszlenar/Validot
- Категория: benchmark/testing candidate
- Описание: Validot is a performance-first, compact library for advanced model validation. Using a simple declarative fluent interface, it efficiently handles classes, structs, nested members, collections, nullables, plus any relation or combination of them. It also supports translations, custom logic extensions with tests, and DI containers.
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## DNS-OARC/flamethrower

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DNS-OARC/flamethrower
- Категория: benchmark/testing candidate
- Описание: a DNS performance and functional testing utility supporting UDP, TCP, DoT and DoH
- Уровень: automated README evidence extraction
- Снимок: [sources/DNS-OARC__flamethrower/README.md](sources/DNS-OARC__flamethrower/README.md); SHA-256: `0c01746418e5ab58c7023bc35211c9de564f17142526f2030d41cb3cdc7f3302`
- lifecycle: строка 80: Flamethrower can adjust its QPS flow over time. This is useful for generating a "signal" of traffic (e.g. a square wave) for calibrating metrics collection. For example, to send 10 QPS for 120000ms, then 80 QPS for 120000ms, etc use `--qps-flow "10,120000;80,1
- async: строка 84: Flamethrower can generate detailed metrics for each of its concurrent senders. Metrics include send and receive counts, timeouts, min, max and average latency, errors, and the like. The output format is JSON, and is suitable for ingestion into databases such a
- report: строка 84: Flamethrower can generate detailed metrics for each of its concurrent senders. Metrics include send and receive counts, timeouts, min, max and average latency, errors, and the like. The output format is JSON, and is suitable for ingestion into databases such a

## crossoverJie/ptg

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/crossoverJie/ptg
- Категория: benchmark/testing candidate
- Описание: 💥Performance testing tool (Go), It is also a GUI gRPC client.
- Уровень: automated README evidence extraction
- Снимок: [sources/crossoverJie__ptg/README.md](sources/crossoverJie__ptg/README.md); SHA-256: `3f6d51c3d0d4442a53721ecfab2c91812e3a2bf30837c5d99c0c230810630f20`
- report: строка 74: --bodyPath value, --body value        -body bodyPath.json

## chrisneagu/FTC-Skystone-Dark-Angels-Romania-2020

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/chrisneagu/FTC-Skystone-Dark-Angels-Romania-2020
- Категория: benchmark/testing candidate
- Описание: NOTICE This repository contains the public FTC SDK for the SKYSTONE (2019-2020) competition season. If you are looking for the current season's FTC SDK software, please visit the new and permanent home of the public FTC SDK:  FtcRobotController repository  Welcome! This GitHub repository contains the source code that is used to build an Android app to control a FIRST Tech Challenge competition robot. To use this SDK, download/clone the entire project to your local computer.  Getting Started If you are new to robotics or new to the FIRST Tech Challenge, then you should consider reviewing the FTC Blocks Tutorial to get familiar with how to use the control system:        FTC Blocks Online Tutorial  Even if you are an advanced Java programmer, it is helpful to start with the FTC Blocks tutorial, and then migrate to the OnBot Java Tool or to Android Studio afterwards.  Downloading the Project If you are an Android Studio programmer, there are several ways to download this repo. Note that if you use the Blocks or OnBot Java Tool to program your robot, then you do not need to download this repository.  If you are a git user, you can clone the most current version of the repository:             git clone https://github.com/FIRST-Tech-Challenge/SKYSTONE.git  Or, if you prefer, you can use the "Download Zip" button available through the main repository page. Downloading the project as a .ZIP file will keep the size of the download manageable.  You can also download the project folder (as a .zip or .tar.gz archive file) from the Downloads subsection of the Releases page for this repository.  Once you have downloaded and uncompressed (if needed) your folder, you can use Android Studio to import the folder ("Import project (Eclipse ADT, Gradle, etc.)").  Getting Help User Documentation and Tutorials FIRST maintains online documentation with information and tutorials on how to use the FIRST Tech Challenge software and robot control system. You can access this documentation using the following link:        SKYSTONE Online Documentation  Note that the online documentation is an "evergreen" document that is constantly being updated and edited. It contains the most current information about the FIRST Tech Challenge software and control system.  Javadoc Reference Material The Javadoc reference documentation for the FTC SDK is now available online. Click on the following link to view the FTC SDK Javadoc documentation as a live website:        FTC Javadoc Documentation  Documentation for the FTC SDK is also included with this repository. There is a subfolder called "doc" which contains several subfolders:  The folder "apk" contains the .apk files for the FTC Driver Station and FTC Robot Controller apps. The folder "javadoc" contains the JavaDoc user documentation for the FTC SDK. Online User Forum For technical questions regarding the Control System or the FTC SDK, please visit the FTC Technology forum:        FTC Technology Forum  Release Information Version 5.5 (20200824-090813) Version 5.5 requires Android Studio 4.0 or later.  New features Adds support for calling custom Java classes from Blocks OpModes (fixes SkyStone issue #161). Classes must be in the org.firstinspires.ftc.teamcode package. Methods must be public static and have no more than 21 parameters. Parameters declared as OpMode, LinearOpMode, Telemetry, and HardwareMap are supported and the argument is provided automatically, regardless of the order of the parameters. On the block, the sockets for those parameters are automatically filled in. Parameters declared as char or java.lang.Character will accept any block that returns text and will only use the first character in the text. Parameters declared as boolean or java.lang.Boolean will accept any block that returns boolean. Parameters declared as byte, java.lang.Byte, short, java.lang.Short, int, java.lang.Integer, long, or java.lang.Long, will accept any block that returns a number and will round that value to the nearest whole number. Parameters declared as float, java.lang.Float, double, java.lang.Double will accept any block that returns a number. Adds telemetry API method for setting display format Classic Monospace HTML (certain tags only) Adds blocks support for switching cameras. Adds Blocks support for TensorFlow Object Detection with a custom model. Adds support for uploading a custom TensorFlow Object Detection model in the Manage page, which is especially useful for Blocks and OnBotJava users. Shows new Control Hub blink codes when the WiFi band is switched using the Control Hub's button (only possible on Control Hub OS 1.1.2) Adds new warnings which can be disabled in the Advanced RC Settings Mismatched app versions warning Unnecessary 2.4 GHz WiFi usage warning REV Hub is running outdated firmware (older than version 1.8.2) Adds support for Sony PS4 gamepad, and reworks how gamepads work on the Driver Station Removes preference which sets gamepad type based on driver position. Replaced with menu which allows specifying type for gamepads with unknown VID and PID Attempts to auto-detect gamepad type based on USB VID and PID If gamepad VID and PID is not known, use type specified by user for that VID and PID If gamepad VID and PID is not known AND the user has not specified a type for that VID and PID, an educated guess is made about how to map the gamepad Driver Station will now attempt to automatically recover from a gamepad disconnecting, and re-assign it to the position it was assigned to when it dropped If only one gamepad is assigned and it drops: it can be recovered If two gamepads are assigned, and have different VID/PID signatures, and only one drops: it will be recovered If two gamepads are assigned, and have different VID/PID signatures, and BOTH drop: both will be recovered If two gamepads are assigned, and have the same VID/PID signatures, and only one drops: it will be recovered If two gamepads are assigned, and have the same VID/PID signatures, and BOTH drop: neither will be recovered, because of the ambiguity of the gamepads when they re-appear on the USB bus. There is currently one known edge case: if there are two gamepads with the same VID/PID signature plugged in, but only one is assigned, and they BOTH drop, it's a 50-50 chance of which one will be chosen for automatic recovery to the assigned position: it is determined by whichever one is re-enumerated first by the USB bus controller. Adds landscape user interface to Driver Station New feature: practice timer with audio cues New feature (Control Hub only): wireless network connection strength indicator (0-5 bars) New feature (Control Hub only): tapping on the ping/channel display will switch to an alternate display showing radio RX dBm and link speed (tap again to switch back) The layout will NOT autorotate. You can switch the layout from the Driver Station's settings menu. Breaking changes Removes support for Android versions 4.4 through 5.1 (KitKat and Lollipop). The minSdkVersion is now 23. Removes the deprecated LinearOpMode methods waitOneFullHardwareCycle() and waitForNextHardwareCycle() Enhancements Handles RS485 address of Control Hub automatically The Control Hub is automatically given a reserved address Existing configuration files will continue to work All addresses in the range of 1-10 are still available for Expansion Hubs The Control Hub light will now normally be solid green, without blinking to indicate the address The Control Hub will not be shown on the Expansion Hub Address Change settings page Improves REV Hub firmware updater The user can now choose between all available firmware update files Version 1.8.2 of the REV Hub firmware is bundled into the Robot Controller app. Text was added to clarify that Expansion Hubs can only be updated via USB. Firmware update speed was reduced to improve reliability Allows REV Hub firmware to be updated directly from the Manage webpage Improves log viewer on Robot Controller Horizontal scrolling support (no longer word wrapped) Supports pinch-to-zoom Uses a monospaced font Error messages are highlighted New color scheme Attempts to force-stop a runaway/stuck OpMode without restarting the entire app Not all types of runaway conditions are stoppable, but if the user code attempts to talk to hardware during the runaway, the system should be able to capture it. Makes various tweaks to the Self Inspect screen Renames "OS version" entry to "Android version" Renames "WiFi Direct Name" to "WiFi Name" Adds Control Hub OS version, when viewing the report of a Control Hub Hides the airplane mode entry, when viewing the report of a Control Hub Removes check for ZTE Speed Channel Changer Shows firmware version for all Expansion and Control Hubs Reworks network settings portion of Manage page All network settings are now applied with a single click The WiFi Direct channel of phone-based Robot Controllers can now be changed from the Manage page WiFi channels are filtered by band (2.4 vs 5 GHz) and whether they overlap with other channels The current WiFi channel is pre-selected on phone-based Robot Controllers, and Control Hubs running OS 1.1.2 or later. On Control Hubs running OS 1.1.2 or later, you can choose to have the system automatically select a channel on the 5 GHz band Improves OnBotJava New light and dark themes replace the old themes (chaos, github, chrome,...) the new default theme is light and will be used when you first update to this version OnBotJava now has a tabbed editor Read-only offline mode Improves function of "exit" menu item on Robot Controller and Driver Station Now guaranteed to be fully stopped and unloaded from memory Shows a warning message if a LinearOpMode exists prematurely due to failure to monitor for the start condition Improves error message shown when the Driver Station and Robot Controller are incompatible with each other Driver Station OpMode Control Panel now disabled while a Restart Robot is in progress Disables advanced settings related to WiFi direct when the Robot Controller is a Control Hub. Tint phone battery icons on Driver Station when low/critical. Uses names "Control Hub Portal" and "Control Hub" (when appropriate) in new configuration files Improve I2C read performance Very large improvement on Control Hub; up to ~2x faster with small (e.g. 6 byte) reads Not as apparent on Expansion Hubs connected to a phone Update/refresh build infrastructure Update to 'androidx' support library from 'com.android.support:appcompat', which is end-of-life Update targetSdkVersion and compileSdkVersion to 28 Update Android Studio's Android plugin to latest Fix reported build timestamp in 'About' screen Add sample illustrating manual webcam use: ConceptWebcam Bug fixes Fixes SkyStone issue #248 Fixes SkyStone issue #232 and modifies bulk caching semantics to allow for cache-preserving MANUAL/AUTO transitions. Improves performance when REV 2M distance sensor is unplugged Improves readability of Toast messages on certain devices Allows a Driver Station to connect to a Robot Controller after another has disconnected Improves generation of fake serial numbers for UVC cameras which do not provide a real serial number Previously some devices would assign such cameras a serial of 0:0 and fail to open and start streaming Fixes ftc_app issue #638. Fixes a slew of bugs with the Vuforia camera monitor including: Fixes bug where preview could be displayed with a wonky aspect ratio Fixes bug where preview could be cut off in landscape Fixes bug where preview got totally messed up when rotating phone Fixes bug where crosshair could drift off target when using webcams Fixes issue in UVC driver on some devices (ftc_app 681) if streaming was started/stopped multiple times in a row Issue manifested as kernel panic on devices which do not have this kernel patch. On affected devices which do have the patch, the issue was manifest as simply a failure to start streaming. The Tech Team believes that the root cause of the issue is a bug in the Linux kernel XHCI driver. A workaround was implemented in the SDK UVC driver. Fixes bug in UVC driver where often half the frames from the camera would be dropped (e.g. only 15FPS delivered during a streaming session configured for 30FPS). Fixes issue where TensorFlow Object Detection would show results whose confidence was lower than the minimum confidence parameter. Fixes a potential exploitation issue of CVE-2019-11358 in OnBotJava Fixes changing the address of an Expansion Hub with additional Expansion Hubs connected to it Preserves the Control Hub's network connection when "Restart Robot" is selected Fixes issue where device scans would fail while the Robot was restarting Fix RenderScript usage Use androidx.renderscript variant: increased compatibility Use RenderScript in Java mode, not native: simplifies build Fixes webcam-frame-to-bitmap conversion problem: alpha channel wasn't being initialized, only R, G, & B Fixes possible arithmetic overflow in Deadline Fixes deadlock in Vuforia webcam support which could cause 5-second delays when stopping OpMode Version 5.4 (20200108-101156) Fixes SkyStone issue #88 Adds an inspection item that notes when a robot controller (Control Hub) is using the factory default password. Fixes SkyStone issue #61 Fixes SkyStone issue #142 Fixes ftc_app issue #417 by adding more current and voltage monitoring capabilities for REV Hubs. Fixes a crash sometimes caused by OnBotJava activity Improves OnBotJava autosave functionality ftc_app #738 Fixes system responsiveness issue when an Expansion Hub is disconnected Fixes issue where IMU initialization could prevent Op Modes from stopping Fixes issue where AndroidTextToSpeech.speak() would fail if it was called too early Adds telemetry.speak() methods and blocks, which cause the Driver Station (if also updated) to speak text Adds and improves Expansion Hub-related warnings Improves Expansion Hub low battery warning Displays the warning immediately after the hub reports it Specifies whether the condition is current or occurred temporarily during an OpMode run Displays which hubs reported low battery Displays warning when hub loses and regains power during an OpMode run Fixes the hub's LED pattern after this condition Displays warning when Expansion Hub is not responding to commands Specifies whether the condition is current or occurred temporarily during an OpMode run Clarifies warning when Expansion Hub is not present at startup Specifies that this condition requires a Robot Restart before the hub can be used. The hub light will now accurately reflect this state Improves logging and reduces log spam during these conditions Syncs the Control Hub time and timezone to a connected web browser programming the robot, if a Driver Station is not available. Adds bulk read functionality for REV Hubs A bulk caching mode must be set at the Hub level with LynxModule#setBulkCachingMode(). This applies to all relevant SDK hardware classes that reference that Hub. The following following Hub bulk caching modes are available: BulkCachingMode.OFF (default): All hardware calls operate as usual. Bulk data can read through LynxModule#getBulkData() and processed manually. BulkCachingMode.AUTO: Applicable hardware calls are served from a bulk read cache that is cleared/refreshed automatically to ensure identical commands don't hit the same cache. The cache can also be cleared manually with LynxModule#clearBulkCache(), although this is not recommended. (advanced users) BulkCachingMode.MANUAL: Same as BulkCachingMode.AUTO except the cache is never cleared automatically. To avoid getting stale data, the cache must be manually cleared at the beginning of each loop body or as the user deems appropriate. Removes PIDF Annotation values added in Rev 5.3 (to AndyMark, goBILDA and TETRIX motor configurations). The new motor types will still be available but their Default control behavior will revert back to Rev 5.2 Adds new ConceptMotorBulkRead sample Opmode to demonstrate and compare Motor Bulk-Read modes for reducing I/O latencies. Version 5.3 (20191004-112306) Fixes external USB/UVC webcam support Makes various bugfixes and improvements to Blocks page, including but not limited to: Many visual tweaks Browser zoom and window resize behave better Resizing the Java preview pane works better and more consistently across browsers The Java preview pane consistently gets scrollbars when needed The Java preview pane is hidden by default on phones Internet Explorer 11 should work Large dropdown lists display properly on lower res screens Disabled buttons are now visually identifiable as disabled A warning is shown if a user selects a TFOD sample, but their device is not compatible Warning messages in a Blocks op mode are now visible by default. Adds goBILDA 5201 and 5202 motors to Robot Configurator Adds PIDF Annotation values to AndyMark, goBILDA and TETRIX motor configurations. This has the effect of causing the RUN_USING_ENCODERS and RUN_TO_POSITION modes to use PIDF vs PID closed loop control on these motors. This should provide more responsive, yet stable, speed control. PIDF adds Feedforward control to the basic PID control loop. Feedforward is useful when controlling a motor's speed because it "anticipates" how much the control voltage must change to achieve a new speed set-point, rather than requiring the integrated error to change sufficiently. The PIDF values were chosen to provide responsive, yet stable, speed control on a lightly loaded motor. The more heavily a motor is loaded (drag or friction), the more noticable the PIDF improvement will be. Fixes startup crash on Android 10 Fixes ftc_app issue #712 (thanks to FROGbots-4634) Fixes ftc_app issue #542 Allows "A" and lowercase letters when naming device through RC and DS apps. Version 5.2 (20190905-083277) Fixes extra-wide margins on settings activities, and placement of the new configuration button Adds Skystone Vuforia image target data. Includes sample Skystone Vuforia Navigation op modes (Java). Includes sample Skystone Vuforia Navigation op modes (Blocks). Adds TensorFlow inference model (.tflite) for Skystone game elements. Includes sample Skystone TensorFlow op modes (Java). Includes sample Skystone TensorFlow op modes (Blocks). Removes older (season-specific) sample op modes. Includes 64-bit support (to comply with Google Play requirements). Protects against Stuck OpModes when a Restart Robot is requested. (Thanks to FROGbots-4634) (ftc_app issue #709) Blocks related changes: Fixes bug with blocks generated code when hardware device name is a java or javascript reserved word. Shows generated java code for blocks, even when hardware items are missing from the active configuration. Displays warning icon when outdated Vuforia and TensorFlow blocks are used (SkyStone issue #27) Version 5.1 (20190820-222104) Defines default PIDF parameters for the following motors: REV Core Hex Motor REV 20:1 HD Hex Motor REV 40:1 HD Hex Motor Adds back button when running on a device without a system back button (such as a Control Hub) Allows a REV Control Hub to update the firmware on a REV Expansion Hub via USB Fixes SkyStone issue #9 Fixes ftc_app issue #715 Prevents extra DS User clicks by filtering based on current state. Prevents incorrect DS UI state changes when receiving new OpMode list from RC Adds support for REV Color Sensor V3 Adds a manual-refresh DS Camera Stream for remotely viewing RC camera frames. To show the stream on the DS, initialize but do not run a stream-enabled opmode, select the Camera Stream option in the DS menu, and tap the image to refresh. This feature is automatically enabled when using Vuforia or TFOD—no additional RC configuration is required for typical use cases. To hide the stream, select the same menu item again. Note that gamepads are disabled and the selected opmode cannot be started while the stream is open as a safety precaution. To use custom streams, consult the API docs for CameraStreamServer#setSource and CameraStreamSource. Adds many Star Wars sounds to RobotController resources. Added SKYSTONE Sounds Chooser Sample Program. Switches out startup, connect chimes, and error/warning sounds for Star Wars sounds Updates OnBot Java to use a WebSocket for communication with the robot The OnBot Java page no longer has to do a full refresh when a user switches from editing one file to another Known issues:  Camera Stream The Vuforia camera stream inherits the issues present in the phone preview (namely ftc_app issue #574). This problem does not affect the TFOD camera stream even though it receives frames from Vuforia. The orientation of the stream frames may not always match the phone preview. For now, these frames may be rotated manually via a custom CameraStreamSource if desired. OnBotJava Browser back button may not always work correctly It's possible for a build to be queued, but not started. The OnBot Java build console will display a warning if this occurs. A user might not realize they are editing a different file if the user inadvertently switches from one file to another since this switch is now seamless. The name of the currently open file is displayed in the browser tab. Version 5.0 (built on 19.06.14) Support for the REV Robotics Control Hub. Adds a Java preview pane to the Blocks editor. Adds a new offline export feature to the Blocks editor. Display wifi channel in Network circle on Driver Station. Adds calibration for Logitech C270 Updates build tooling and target SDK. Compliance with Google's permissions infrastructure (Required after build tooling update). Keep Alives to mitigate the Motorola wifi scanning problem. Telemetry substitute no longer necessary. Improves Vuforia error reporting. Fixes ftctechnh/ftc_app issues 621, 713. Miscellaneous bug fixes and improvements. Version 4.3 (built on 18.10.31) Includes missing TensorFlow-related libraries and files. Version 4.2 (built on 18.10.30) Includes fix to avoid deadlock situation with WatchdogMonitor which could result in USB communication errors. Comm error appeared to require that user disconnect USB cable and restart the Robot Controller app to recover. robotControllerLog.txt would have error messages that included the words "E RobotCore: lynx xmit lock: #### abandoning lock:" Includes fix to correctly list the parent module address for a REV Robotics Expansion Hub in a configuration (.xml) file. Bug in versions 4.0 and 4.1 would incorrect list the address module for a parent REV Robotics device as "1". If the parent module had a higher address value than the daisy-chained module, then this bug would prevent the Robot Controller from communicating with the downstream Expansion Hub. Added requirement for ACCESS_COARSE_LOCATION to allow a Driver Station running Android Oreo to scan for Wi-Fi Direct devices. Added google() repo to build.gradle because aapt2 must be downloaded from the google() repository beginning with version 3.2 of the Android Gradle Plugin. Important Note: Android Studio users will need to be connected to the Internet the first time build the ftc_app project. Internet connectivity is required for the first build so the appropriate files can be downloaded from the Google repository. Users should not need to be connected to the Internet for subsequent builds. This should also fix buid issue where Android Studio would complain that it "Could not find com.android.tools.lint:lint-gradle:26.1.4" (or similar). Added support for REV Spark Mini motor controller as part of the configuration menu for a servo/PWM port on the REV Expansion Hub. Provide examples for playing audio files in an Op Mode. Block Development Tool Changes Includes a fix for a problem with the Velocity blocks that were reported in the FTC Technology forum (Blocks Programming subforum). Change the "Save completed successfully." message to a white color so it will contrast with a green background. Fixed the "Download image" feature so it will work if there are text blocks in the op mode. Introduce support for Google's TensorFlow Lite technology for object detetion for 2018-2019 game. TensorFlow lite can recognize Gold Mineral and Silver Mineral from 2018-2019 game. Example Java and Block op modes are included to show how to determine the relative position of the gold block (left, center, right). Version 4.1 (released on 18.09.24) Changes include:  Fix to prevent crash when deprecated configuration annotations are used. Change to allow FTC Robot Controller APK to be auto-updated using FIRST Global Control Hub update scripts. Removed samples for non supported / non legal hardware. Improvements to Telemetry.addData block with "text" socket. Updated Blocks sample op mode list to include Rover Ruckus Vuforia example. Update SDK library version number. Version 4.0 (released on 18.09.12) Changes include:  Initial support for UVC compatible cameras  If UVC camera has a unique serial number, RC will detect and enumerate by serial number. If UVC camera lacks a unique serial number, RC will only support one camera of that type connected. Calibration settings for a few cameras are included (see TeamCode/src/main/res/xml/teamwebcamcalibrations.xml for details). User can upload calibration files from Program and Manage web interface. UVC cameras seem to draw a fair amount of electrical current from the USB bus. This does not appear to present any problems for the REV Robotics Control Hub. This does seem to create stability problems when using some cameras with an Android phone-based Robot Controller. FTC Tech Team is investigating options to mitigate this issue with the phone-based Robot Controllers. Updated sample Vuforia Navigation and VuMark Op Modes to demonstrate how to use an internal phone-based camera and an external UVC webcam. Support for improved motor control.  REV Robotics Expansion Hub firmware 1.8 and greater will support a feed forward mechanism for closed loop motor control. FTC SDK has been modified to support PIDF coefficients (proportional, integral, derivative, and feed forward). FTC Blocks development tool modified to include PIDF programming blocks. Deprecated older PID-related methods and variables. REV's 1.8.x PIDF-related changes provide a more linear and accurate way to control a motor. Wireless  Added 5GHz support for wireless channel changing for those devices that support it. Tested with Moto G5 and E4 phones. Also tested with other (currently non-approved) phones such as Samsung Galaxy S8. Improved Expansion Hub firmware update support in Robot Controller app  Changes to make the system more robust during the firmware update process (when performed through Robot Controller app). User no longer has to disconnect a downstream daisy-chained Expansion Hub when updating an Expansion Hub's firmware. If user is updating an Expansion Hub's firmware through a USB connection, he/she does not have to disconnect RS485 connection to other Expansion Hubs. The user still must use a USB connection to update an Expansion Hub's firmware. The user cannot update the Expansion Hub firmware for a downstream device that is daisy chained through an RS485 connection. If an Expansion Hub accidentally gets "bricked" the Robot Controller app is now more likely to recognize the Hub when it scans the USB bus. Robot Controller app should be able to detect an Expansion Hub, even if it accidentally was bricked in a previous update attempt. Robot Controller app should be able to install the firmware onto the Hub, even if if accidentally was bricked in a previous update attempt. Resiliency  FTC software can detect and enable an FTDI reset feature that is available with REV Robotics v1.8 Expansion Hub firmware and greater. When enabled, the Expansion Hub can detect if it hasn't communicated with the Robot Controller over the FTDI (USB) connection. If the Hub hasn't heard from the Robot Controller in a while, it will reset the FTDI connection. This action helps system recover from some ESD-induced disruptions. Various fixes to improve reliability of FTC software. Blocks  Fixed errors with string and list indices in blocks export to java. Support for USB connected UVC webcams. Refactored optimized Blocks Vuforia code to support Rover Ruckus image targets. Added programming blocks to support PIDF (proportional, integral, derivative and feed forward) motor control. Added formatting options (under Telemetry and Miscellaneous categories) so user can set how many decimal places to display a numerical value. Support to play audio files (which are uploaded through Blocks web interface) on Driver Station in addition to the Robot Controller. Fixed bug with Download Image of Blocks feature. Support for REV Robotics Blinkin LED Controller. Support for REV Robotics 2m Distance Sensor. Added support for a REV Touch Sensor (no longer have to configure as a generic digital device). Added blocks for DcMotorEx methods. These are enhanced methods that you can use when supported by the motor controller hardware. The REV Robotics Expansion Hub supports these enhanced methods. Enhanced methods include methods to get/set motor velocity (in encoder pulses per second), get/set PIDF coefficients, etc.. Modest Improvements in Logging  Decrease frequency of battery checker voltage statements. Removed non-FTC related log statements (wherever possible). Introduced a "Match Logging" feature. Under "Settings" a user can enable/disable this feature (it's disabled by default). If enabled, user provides a "Match Number" through the Driver Station user interface (top of the screen). The Match Number is used to create a log file specifically with log statements from that particular Op Mode run. Match log files are stored in /sdcard/FIRST/matlogs on the Robot Controller. Once an op mode run is complete, the Match Number is cleared. This is a convenient way to create a separate match log with statements only related to a specific op mode run. New Devices  Support for REV Robotics Blinkin LED Controller. Support for REV Robotics 2m Distance Sensor. Added configuration option for REV 20:1 HD Hex Motor. Added support for a REV Touch Sensor (no longer have to configure as a generic digital device). Miscellaneous  Fixed some errors in the definitions for acceleration and velocity in our javadoc documentation. Added ability to play audio files on Driver Station When user is configuring an Expansion Hub, the LED on the Expansion Hub will change blink pattern (purple-cyan) to indicate which Hub is currently being configured. Renamed I2cSensorType to I2cDeviceType. Added an external sample Op Mode that demonstrates localization using 2018-2019 (Rover Ruckus presented by QualComm) Vuforia targets. Added an external sample Op Mode that demonstrates how to use the REV Robotics 2m Laser Distance Sensor. Added an external sample Op Mode that demonstrates how to use the REV Robotics Blinkin LED Controller. Re-categorized external Java sample Op Modes to "TeleOp" instead of "Autonomous". Known issues:  Initial support for UVC compatible cameras  UVC cameras seem to draw significant amount of current from the USB bus. This does not appear to present any problems for the REV Robotics Control Hub. This does seem to create stability problems when using some cameras with an Android phone-based Robot Controller. FTC Tech Team is investigating options to mitigate this issue with the phone-based Robot Controllers. There might be a possible deadlock which causes the RC to become unresponsive when using a UVC webcam with a Nougat Android Robot Controller. Wireless  When user selects a wireless channel, this channel does not necessarily persist if the phone is power cycled. Tech Team is hoping to eventually address this issue in a future release. Issue has been present since apps were introduced (i.e., it is not new with the v4.0 release). Wireless channel is not currently displayed for WiFi Direct connections. Miscellaneous  The blink indication feature that shows which Expansion Hub is currently being configured does not work for a newly created configuration file. User has to first save a newly created configuration file and then close and re-edit the file in order for blink indicator to work. Version 3.6 (built on 17.12.18) Changes include:  Blocks Changes Uses updated Google Blockly software to allow users to edit their op modes on Apple iOS devices (including iPad and iPhone). Improvement in Blocks tool to handle corrupt op mode files. Autonomous op modes should no longer get switched back to tele-op after re-opening them to be edited. The system can now detect type mismatches during runtime and alert the user with a message on the Driver Station. Updated javadoc documentation for setPower() method to reflect correct range of values (-1 to +1). Modified VuforiaLocalizerImpl to allow for user rendering of frames Added a user-overrideable onRenderFrame() method which gets called by the class's renderFrame() method. Version 3.5 (built on 17.10.30) Changes with version 3.5 include:  Introduced a fix to prevent random op mode stops, which can occur after the Robot Controller app has been paused and then resumed (for example, when a user temporarily turns off the display of the Robot Controller phone, and then turns the screen back on). Introduced a fix to prevent random op mode stops, which were previously caused by random peer disconnect events on the Driver Station. Fixes issue where log files would be closed on pause of the RC or DS, but not re-opened upon resume. Fixes issue with battery handler (voltage) start/stop race. Fixes issue where Android Studio generated op modes would disappear from available list in certain situations. Fixes problem where OnBot Java would not build on REV Robotics Control Hub. Fixes problem where OnBot Java would not build if the date and time on the Robot Controller device was "rewound" (set to an earlier date/time). Improved error message on OnBot Java that occurs when renaming a file fails. Removed unneeded resources from android.jar binaries used by OnBot Java to reduce final size of Robot Controller app. Added MR_ANALOG_TOUCH_SENSOR block to Blocks Programming Tool. Version 3.4 (built on 17.09.06) Changes with version 3.4 include:  Added telemetry.update() statement for BlankLinearOpMode template. Renamed sample Block op modes to be more consistent with Java samples. Added some additional sample Block op modes. Reworded OnBot Java readme slightly. Version 3.3 (built on 17.09.04) This version of the software includes improves for the FTC Blocks Programming Tool and the OnBot Java Programming Tool.  Changes with verion 3.3 include:  Android Studio ftc_app project has been updated to use Gradle Plugin 2.3.3. Android Studio ftc_app project is already using gradle 3.5 distribution. Robot Controller log has been renamed to /sdcard/RobotControllerLog.txt (note that this change was actually introduced w/ v3.2). Improvements in I2C reliability. Optimized I2C read for REV Expansion Hub, with v1.7 firmware or greater. Updated all external/samples (available through OnBot and in Android project folder). Vuforia Added support for VuMarks that will be used for the 2017-2018 season game. Blocks Update to latest Google Blockly release. Sample op modes can be selected as a template when creating new op mode. Fixed bug where the blocks would disappear temporarily when mouse button is held down. Added blocks for Range.clip and Range.scale. User can now disable/enable Block op modes. Fix to prevent occasional Blocks deadlock. OnBot Java Significant improvements with autocomplete function for OnBot Java editor. Sample op modes can be selected as a template when creating new op mode. Fixes and changes to complete hardware setup feature. Updated (and more useful) onBot welcome message. Known issues:  Android Studio After updating to the new v3.3 Android Studio project folder, if you get error messages indicating "InvalidVirtualFileAccessException" then you might need to do a File->Invalidate Caches / Restart to clear the error. OnBot Java Sometimes when you push the build button to build all op modes, the RC returns an error message that the build failed. If you press the build button a second time, the build typically suceeds. Version 3.2 (built on 17.08.02) This version of the software introduces the "OnBot Java" Development Tool. Similar to the FTC Blocks Development Tool, the FTC OnBot Java Development Tool allows a user to create, edit and build op modes dynamically using only a Javascript-enabled web browser.  The OnBot Java Development Tool is an integrated development environment (IDE) that is served up by the Robot Controller. Op modes are created and edited using a Javascript-enabled browser (Google Chromse is recommended). Op modes are saved on the Robot Controller Android device directly.  The OnBot Java Development Tool provides a Java programming environment that does NOT need Android Studio.  Changes with version 3.2 include:  Enhanced web-based development tools  Introduction of OnBot Java Development Tool. Web-based programming and management features are "always on" (user no longer needs to put Robot Controller into programming mode). Web-based management interface (where user can change Robot Controller name and also easily download Robot Controller log file). OnBot Java, Blocks and Management features available from web based interface. Blocks Programming Development Tool:  Changed "LynxI2cColorRangeSensor" block to "REV Color/range sensor" block. Fixed tooltip for ColorSensor.isLightOn block. Added blocks for ColorSensor.getNormalizedColors and LynxI2cColorRangeSensor.getNormalizedColors. Added example op modes for digital touch sensor and REV Robotics Color Distance sensor.  User selectable color themes.  Includes many minor enhancements and fixes (too numerous to list).  Known issues:  Auto complete function is incomplete and does not support the following (for now): Access via this keyword Access via super keyword Members of the super cloass, not overridden by the class Any methods provided in the current class Inner classes Can't handle casted objects Any objects coming from an parenthetically enclosed expression Version 3.10 (built on 17.05.09) This version of the software provides support for the REV Robotics Expansion Hub. This version also includes improvements in the USB communication layer in an effort to enhance system resiliency. If you were using a 2.x version of the software previously, updating to version 3.1 requires that you also update your Driver Station software in addition to updating the Robot Controller software.  Also note that in version 3.10 software, the setMaxSpeed and getMaxSpeed methods are no longer available (not deprecated, they have been removed from the SDK). Also note that the the new 3.x software incorporates motor profiles that a user can select as he/she configures the robot.  Changes include:  Blocks changes Added VuforiaTrackableDefaultListener.getPose and Vuforia.trackPose blocks. Added optimized blocks support for Vuforia extended tracking. Added atan2 block to the math category. Added useCompetitionFieldTargetLocations parameter to Vuforia.initialize block. If set to false, the target locations are placed at (0,0,0) with target orientation as specified in https://github.com/gearsincorg/FTCVuforiaDemo/blob/master/Robot_Navigation.java tutorial op mode. Incorporates additional improvements to USB comm layer to improve system resiliency (to recover from a greater number of communication disruptions). Additional Notes Regarding Version 3.00 (built on 17.04.13)  In addition to the release changes listed below (see section labeled "Version 3.00 (built on 17.04.013)"), version 3.00 has the following important changes:  Version 3.00 software uses a new version of the FTC Robocol (robot protocol). If you upgrade to v3.0 on the Robot Controller and/or Android Studio side, you must also upgrade the Driver Station software to match the new Robocol. Version 3.00 software removes the setMaxSpeed and getMaxSpeed methods from the DcMotor class. If you have an op mode that formerly used these methods, you will need to remove the references/calls to these methods. Instead, v3.0 provides the max speed information through the use of motor profiles that are selected by the user during robot configuration. Version 3.00 software currently does not have a mechanism to disable extra i2c sensors. We hope to re-introduce this function with a release in the near future. Version 3.00 (built on 17.04.13) *** Use this version of the software at YOUR OWN RISK!!! ***  This software is being released as an "alpha" version. Use this version at your own risk!  This pre-release software contains SIGNIFICANT changes, including changes to the Wi-Fi Direct pairing mechanism, rewrites of the I2C sensor classes, changes to the USB/FTDI layer, and the introduction of support for the REV Robotics Expansion Hub and the REV Robotics color-range-light sensor. These changes were implemented to improve the reliability and resiliency of the FTC control system.  Please note, however, that version 3.00 is considered "alpha" code. This code is being released so that the FIRST community will have an opportunity to test the new REV Expansion Hub electronics module when it becomes available in May. The developers do not recommend using this code for critical applications (i.e., competition use).  *** Use this version of the software at YOUR OWN RISK!!! ***  Changes include:  Major rework of sensor-related infrastructure. Includes rewriting sensor classes to implement synchronous I2C communication. Fix to reset Autonomous timer back to 30 seconds. Implementation of specific motor profiles for approved 12V motors (includes Tetrix, AndyMark, Matrix and REV models). Modest improvements to enhance Wi-Fi P2P pairing. Fixes telemetry log addition race. Publishes all the sources (not just a select few). Includes Block programming improvements Addition of optimized Vuforia blocks. Auto scrollbar to projects and sounds pages. Fixed blocks paste bug. Blocks execute after while-opModeIsActive loop (to allow for cleanup before exiting op mode). Added gyro integratedZValue block. Fixes bug with projects page for Firefox browser. Added IsSpeaking block to AndroidTextToSpeech. Implements support for the REV Robotics Expansion Hub Implements support for integral REV IMU (physically installed on I2C bus 0, uses same Bosch BNO055 9 axis absolute orientation sensor as Adafruit 9DOF abs orientation sensor). - Implements support for REV color/range/light sensor. Provides support to update Expansion Hub firmware through FTC SDK. Detects REV firmware version and records in log file. Includes support for REV Control Hub (note that the REV Control Hub is not yet approved for FTC use). Implements FTC Blocks programming support for REV Expansion Hub and sensor hardware. Detects and alerts when I2C device disconnect. Version 2.62 (built on 17.01.07) Added null pointer check before calling modeToByte() in finishModeSwitchIfNecessary method for ModernRoboticsUsbDcMotorController class. Changes to enhance Modern Robotics USB protocol robustness. Version 2.61 (released on 16.12.19) Blocks Programming mode changes: Fix to correct issue when an exception was thrown because an OpticalDistanceSensor object appears twice in the hardware map (the second time as a LightSensor). Version 2.6 (released on 16.12.16) Fixes for Gyro class: Improve (decrease) sensor refresh latency. fix isCalibrating issues. Blocks Programming mode changes: Blocks now ignores a device in the configuration xml if the name is empty. Other devices work in configuration work fine. Version 2.5 (internal release on released on 16.12.13) Blocks Programming mode changes: Added blocks support for AdafruitBNO055IMU. Added Download Op Mode button to FtcBocks.html. Added support for copying blocks in one OpMode and pasting them in an other OpMode. The clipboard content is stored on the phone, so the programming mode server must be running. Modified Utilities section of the toolbox. In Programming Mode, display information about the active connections. Fixed paste location when workspace has been scrolled. Added blocks support for the android Accelerometer. Fixed issue where Blocks Upload Op Mode truncated name at first dot. Added blocks support for Android SoundPool. Added type safety to blocks for Acceleration. Added type safety to blocks for AdafruitBNO055IMU.Parameters. Added type safety to blocks for AnalogInput. Added type safety to blocks for AngularVelocity. Added type safety to blocks for Color. Added type safety to blocks for ColorSensor. Added type safety to blocks for CompassSensor. Added type safety to blocks for CRServo. Added type safety to blocks for DigitalChannel. Added type safety to blocks for ElapsedTime. Added type safety to blocks for Gamepad. Added type safety to blocks for GyroSensor. Added type safety to blocks for IrSeekerSensor. Added type safety to blocks for LED. Added type safety to blocks for LightSensor. Added type safety to blocks for LinearOpMode. Added type safety to blocks for MagneticFlux. Added type safety to blocks for MatrixF. Added type safety to blocks for MrI2cCompassSensor. Added type safety to blocks for MrI2cRangeSensor. Added type safety to blocks for OpticalDistanceSensor. Added type safety to blocks for Orientation. Added type safety to blocks for Position. Added type safety to blocks for Quaternion. Added type safety to blocks for Servo. Added type safety to blocks for ServoController. Added type safety to blocks for Telemetry. Added type safety to blocks for Temperature. Added type safety to blocks for TouchSensor. Added type safety to blocks for UltrasonicSensor. Added type safety to blocks for VectorF. Added type safety to blocks for Velocity. Added type safety to blocks for VoltageSensor. Added type safety to blocks for VuforiaLocalizer.Parameters. Added type safety to blocks for VuforiaTrackable. Added type safety to blocks for VuforiaTrackables. Added type safety to blocks for enums in AdafruitBNO055IMU.Parameters. Added type safety to blocks for AndroidAccelerometer, AndroidGyroscope, AndroidOrientation, and AndroidTextToSpeech. Version 2.4 (released on 16.11.13) Fix to avoid crashing for nonexistent resources. Blocks Programming mode changes: Added blocks to support OpenGLMatrix, MatrixF, and VectorF. Added blocks to support AngleUnit, AxesOrder, AxesReference, CameraDirection, CameraMonitorFeedback, DistanceUnit, and TempUnit. Added blocks to support Acceleration. Added blocks to support LinearOpMode.getRuntime. Added blocks to support MagneticFlux and Position. Fixed typos. Made blocks for ElapsedTime more consistent with other objects. Added blocks to support Quaternion, Velocity, Orientation, AngularVelocity. Added blocks to support VuforiaTrackables, VuforiaTrackable, VuforiaLocalizer, VuforiaTrackableDefaultListener. Fixed a few blocks. Added type checking to new blocks. Updated to latest blockly. Added default variable blocks to navigation and matrix blocks. Fixed toolbox entry for openGLMatrix_rotation_withAxesArgs. When user downloads Blocks-generated op mode, only the .blk file is downloaded. When user uploads Blocks-generated op mode (.blk file), Javascript code is auto generated. Added DbgLog support. Added logging when a blocks file is read/written. Fixed bug to properly render blocks even if missing devices from configuration file. Added support for additional characters (not just alphanumeric) for the block file names (for download and upload). Added support for OpMode flavor (“Autonomous” or “TeleOp”) and group. Changes to Samples to prevent tutorial issues. Incorporated suggested changes from public pull 216 (“Replace .. paths”). Remove Servo Glitches when robot stopped. if user hits “Cancels” when editing a configuration file, clears the unsaved changes and reverts to original unmodified configuration. Added log info to help diagnose why the Robot Controller app was terminated (for example, by watch dog function). Added ability to transfer log from the controller. Fixed inconsistency for AngularVelocity Limit unbounded growth of data for telemetry. If user does not call telemetry.update() for LinearOpMode in a timely manner, data added for telemetry might get lost if size limit is exceeded. Version 2.35 (released on 16.10.06) Blockly programming mode - Removed unnecesary idle() call from blocks for new project. Version 2.30 (released on 16.10.05) Blockly programming mode: Mechanism added to save Blockly op modes from Programming Mode Server onto local device To avoid clutter, blocks are displayed in categorized folders Added support for DigitalChannel Added support for ModernRoboticsI2cCompassSensor Added support for ModernRoboticsI2cRangeSensor Added support for VoltageSensor Added support for AnalogInput Added support for AnalogOutput Fix for CompassSensor setMode block Vuforia Fix deadlock / make camera data available while Vuforia is running. Update to Vuforia 6.0.117 (recommended by Vuforia and Google to close security loophole). Fix for autonomous 30 second timer bug (where timer was in effect, even though it appeared to have timed out). opModeIsActive changes to allow cleanup after op mode is stopped (with enforced 2 second safety timeout). Fix to avoid reading i2c twice. Updated sample Op Modes. Improved logging and fixed intermittent freezing. Added digital I/O sample. Cleaned up device names in sample op modes to be consistent with Pushbot guide. Fix to allow use of IrSeekerSensorV3. Version 2.20 (released on 16.09.08) Support for Modern Robotics Compass Sensor. Support for Modern Robotics Range Sensor. Revise device names for Pushbot templates to match the names used in Pushbot guide. Fixed bug so that IrSeekerSensorV3 device is accessible as IrSeekerSensor in hardwareMap. Modified computer vision code to require an individual Vuforia license (per legal requirement from PTC). Minor fixes. Blockly enhancements: Support for Voltage Sensor. Support for Analog Input. Support for Analog Output. Support for Light Sensor. Support for Servo Controller. Version 2.10 (released on 16.09.03) Support for Adafruit IMU. Improvements to ModernRoboticsI2cGyro class Block on reset of z axis. isCalibrating() returns true while gyro is calibration. Updated sample gyro program. Blockly enhancements support for android.graphics.Color. added support for ElapsedTime. improved look and legibility of blocks. support for compass sensor. support for ultrasonic sensor. support for IrSeeker. support for LED. support for color sensor. support for CRServo prompt user to configure robot before using programming mode. Provides ability to disable audio cues. various bug fixes and improvements. Version 2.00 (released on 16.08.19) This is the new release for the upcoming 2016-2017 FIRST Tech Challenge Season. Channel change is enabled in the FTC Robot Controller app for Moto G 2nd and 3rd Gen phones. Users can now use annotations to register/disable their Op Modes. Changes in the Android SDK, JDK and build tool requirements (minsdk=19, java 1.7, build tools 23.0.3). Standardized units in analog input. Cleaned up code for existing analog sensor classes. setChannelMode and getChannelMode were REMOVED from the DcMotorController class. This is important - we no longer set the motor modes through the motor controller. setMode and getMode were added to the DcMotor class. ContinuousRotationServo class has been added to the FTC SDK. Range.clip() method has been overloaded so it can support this operation for int, short and byte integers. Some changes have been made (new methods added) on how a user can access items from the hardware map. Users can now set the zero power behavior for a DC motor so that the motor will brake or float when power is zero. Prototype Blockly Programming Mode has been added to FTC Robot Controller. Users can place the Robot Controller into this mode, and then use a device (such as a laptop) that has a Javascript enabled browser to write Blockly-based Op Modes directly onto the Robot Controller. Users can now configure the robot remotely through the FTC Driver Station app. Android Studio project supports Android Studio 2.1.x and compile SDK Version 23 (Marshmallow). Vuforia Computer Vision SDK integrated into FTC SDK. Users can use sample vision targets to get localization information on a standard FTC field. Project structure has been reorganized so that there is now a TeamCode package that users can use to place their local/custom Op Modes into this package. Inspection function has been integrated into the FTC Robot Controller and Driver Station Apps (Thanks Team HazMat… 9277 & 10650!). Audio cues have been incorporated into FTC SDK. Swap mechanism added to FTC Robot Controller configuration activity. For example, if you have two motor controllers on a robot, and you misidentified them in your configuration file, you can use the Swap button to swap the devices within the configuration file (so you do not have to manually re-enter in the configuration info for the two devices). Fix mechanism added to all user to replace an electronic module easily. For example, suppose a servo controller dies on your robot. You replace the broken module with a new module, which has a different serial number from the original servo controller. You can use the Fix button to automatically reconfigure your configuration file to use the serial number of the new module. Improvements made to fix resiliency and responsiveness of the system. For LinearOpMode the user now must for a telemetry.update() to update the telemetry data on the driver station. This update() mechanism ensures that the driver station gets the updated data properly and at the same time. The Auto Configure function of the Robot Controller is now template based. If there is a commonly used robot configuration, a template can be created so that the Auto Configure mechanism can be used to quickly configure a robot of this type. The logic to detect a runaway op mode (both in the LinearOpMode and OpMode types) and to abort the run, then auto recover has been improved/implemented. Fix has been incorporated so that Logitech F310 gamepad mappings will be correct for Marshmallow users. Release 16.07.08 For the ftc_app project, the gradle files have been modified to support Android Studio 2.1.x. Release 16.03.30 For the MIT App Inventor, the design blocks have new icons that better represent the function of each design component. Some changes were made to the shutdown logic to ensure the robust shutdown of some of our USB services. A change was made to LinearOpMode so as to allow a given instance to be executed more than once, which is required for the App Inventor. Javadoc improved/updated. Release 16.03.09 Changes made to make the FTC SDK synchronous (significant change!) waitOneFullHardwareCycle() and waitForNextHardwareCycle() are no longer needed and have been deprecated. runOpMode() (for a LinearOpMode) is now decoupled from the system's hardware read/write thread. loop() (for an OpMode) is now decoupled from the system's hardware read/write thread. Methods are synchronous. For example, if you call setMode(DcMotorController.RunMode.RESET_ENCODERS) for a motor, the encoder is guaranteed to be reset when the method call is complete. For legacy module (NXT compatible), user no longer has to toggle between read and write modes when reading from or writing to a legacy device. Changes made to enhance reliability/robustness during ESD event. Changes made to make code thread safe. Debug keystore added so that user-generated robot controller APKs will all use the same signed key (to avoid conflicts if a team has multiple developer laptops for example). Firmware version information for Modern Robotics modules are now logged. Changes made to improve USB comm reliability and robustness. Added support for voltage indicator for legacy (NXT-compatible) motor controllers. Changes made to provide auto stop capabilities for op modes. A LinearOpMode class will stop when the statements in runOpMode() are complete. User does not have to push the stop button on the driver station. If an op mode is stopped by the driver station, but there is a run away/uninterruptible thread persisting, the app will log an error message then force itself to crash to stop the runaway thread. Driver Station UI modified to display lowest measured voltage below current voltage (12V battery). Driver Station UI modified to have color background for current voltage (green=good, yellow=caution, red=danger, extremely low voltage). javadoc improved (edits and additional classes). Added app build time to About activity for driver station and robot controller apps. Display local IP addresses on Driver Station About activity. Added I2cDeviceSynchImpl. Added I2cDeviceSync interface. Added seconds() and milliseconds() to ElapsedTime for clarity. Added getCallbackCount() to I2cDevice. Added missing clearI2cPortActionFlag. Added code to create log messages while waiting for LinearOpMode shutdown. Fix so Wifi Direct Config activity will no longer launch multiple times. Added the ability to specify an alternate i2c address in software for the Modern Robotics gyro. Release 16.02.09 Improved battery checker feature so that voltage values get refreshed regularly (every 250 msec) on Driver Station (DS) user interface. Improved software so that Robot Controller (RC) is much more resilient and “self-healing” to USB disconnects: If user attempts to start/restart RC with one or more module missing, it will display a warning but still start up. When running an op mode, if one or more modules gets disconnected, the RC & DS will display warnings,and robot will keep on working in spite of the missing module(s). If a disconnected module gets physically reconnected the RC will auto detect the module and the user will regain control of the recently connected module. Warning messages are more helpful (identifies the type of module that’s missing plus its USB serial number). Code changes to fix the null gamepad reference when users try to reference the gamepads in the init() portion of their op mode. NXT light sensor output is now properly scaled. Note that teams might have to readjust their light threshold values in their op modes. On DS user interface, gamepad icon for a driver will disappear if the matching gamepad is disconnected or if that gamepad gets designated as a different driver. Robot Protocol (ROBOCOL) version number info is displayed in About screen on RC and DS apps. Incorporated a display filter on pairing screen to filter out devices that don’t use the “-“ format. This filter can be turned off to show all WiFi Direct devices. Updated text in License file. Fixed formatting error in OpticalDistanceSensor.toString(). Fixed issue on with a blank (“”) device name that would disrupt WiFi Direct Pairing. Made a change so that the WiFi info and battery info can be displayed more quickly on the DS upon connecting to RC. Improved javadoc generation. Modified code to make it easier to support language localization in the future. Release 16.01.04 Updated compileSdkVersion for apps Prevent Wifi from entering power saving mode removed unused import from driver station Corrrected "Dead zone" joystick code. LED.getDeviceName and .getConnectionInfo() return null apps check for ROBOCOL_VERSION mismatch Fix for Telemetry also has off-by-one errors in its data string sizing / short size limitations error User telemetry output is sorted. added formatting variants to DbgLog and RobotLog APIs code modified to allow for a long list of op mode names. changes to improve thread safety of RobocolDatagramSocket Fix for "missing hardware leaves robot controller disconnected from driver station" error fix for "fast tapping of Init/Start causes problems" (toast is now only instantiated on UI thread). added some log statements for thread life cycle. moved gamepad reset logic inside of initActiveOpMode() for robustness changes made to mitigate risk of race conditions on public methods. changes to try and flag when WiFi Direct name contains non-printable characters. fix to correct race condition between .run() and .close() in ReadWriteRunnableStandard. updated FTDI driver made ReadWriteRunnableStanard interface public. fixed off-by-one errors in Command constructor moved specific hardware implmentations into their own package. moved specific gamepad implemnatations to the hardware library. changed LICENSE file to new BSD version. fixed race condition when shutting down Modern Robotics USB devices. methods in the ColorSensor classes have been synchronized. corrected isBusy() status to reflect end of motion. corrected "back" button keycode. the notSupported() method of the GyroSensor class was changed to protected (it should not be public). Release 15.11.04.001 Added Support for Modern Robotics Gyro. The GyroSensor class now supports the MR Gyro Sensor. Users can access heading data (about Z axis) Users can also access raw gyro data (X, Y, & Z axes). Example MRGyroTest.java op mode included. Improved error messages More descriptive error messages for exceptions in user code. Updated DcMotor API Enable read mode on new address in setI2cAddress Fix so that driver station app resets the gamepads when switching op modes. USB-related code changes to make USB comm more responsive and to display more explicit error messages. Fix so that USB will recover properly if the USB bus returns garbage data. Fix USB initializtion race condition. Better error reporting during FTDI open. More explicit messages during USB failures. Fixed bug so that USB device is closed if event loop teardown method was not called. Fixed timer UI issue Fixed duplicate name UI bug (Legacy Module configuration). Fixed race condition in EventLoopManager. Fix to keep references stable when updating gamepad. For legacy Matrix motor/servo controllers removed necessity of appending "Motor" and "Servo" to controller names. Updated HT color sensor driver to use constants from ModernRoboticsUsbLegacyModule class. Updated MR color sensor driver to use constants from ModernRoboticsUsbDeviceInterfaceModule class. Correctly handle I2C Address change in all color sensors Updated/cleaned up op modes. Updated comments in LinearI2cAddressChange.java example op mode. Replaced the calls to "setChannelMode" with "setMode" (to match the new of the DcMotor method). Removed K9AutoTime.java op mode. Added MRGyroTest.java op mode (demonstrates how to use MR Gyro Sensor). Added MRRGBExample.java op mode (demonstrates how to use MR Color Sensor). Added HTRGBExample.java op mode (demonstrates how to use HT legacy color sensor). Added MatrixControllerDemo.java (demonstrates how to use legacy Matrix controller). Updated javadoc documentation. Updated release .apk files for Robot Controller and Driver Station apps. Release 15.10.06.002 Added support for Legacy Matrix 9.6V motor/servo controller. Cleaned up build.gradle file. Minor UI and bug fixes for driver station and robot controller apps. Throws error if Ultrasonic sensor (NXT) is not configured for legacy module port 4 or 5. Release 15.08.03.001 New user interfaces for FTC Driver Station and FTC Robot Controller apps. An init() method is added to the OpMode class. For this release, init() is triggered right before the start() method. Eventually, the init() method will be triggered when the user presses an "INIT" button on driver station. The init() and loop() methods are now required (i.e., need to be overridden in the user's op mode). The start() and stop() methods are optional. A new LinearOpMode class is introduced. Teams can use the LinearOpMode mode to create a linear (not event driven) program model. Teams can use blocking statements like Thread.sleep() within a linear op mode. The API for the Legacy Module and Core Device Interface Module have been updated. Support for encoders with the Legacy Module is now working. The hardware loop has been updated for better performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/chrisneagu__FTC-Skystone-Dark-Angels-Romania-2020/README.md](sources/chrisneagu__FTC-Skystone-Dark-Angels-Romania-2020/README.md); SHA-256: `93fce103d97af2d75c911cb9cfe83f6f4ae20cf3efa71ed0ac09ed91069b6e47`
- lifecycle: строка 121: * Adds calibration for Logitech C270
- async: строка 483: - Improve (decrease) sensor refresh latency.
- report: строка 125: * Improves Vuforia error reporting.
- correctness: строка 362: - After updating to the new v3.3 Android Studio project folder, if you get error messages indicating "InvalidVirtualFileAccessException" then you might need to do a File->Invalidate Caches / Restart to clear the error.

## cloudflare/telescope

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cloudflare/telescope
- Категория: benchmark/testing candidate
- Описание: Cross-browser web performance testing agent
- Уровень: automated README evidence extraction
- Снимок: [sources/cloudflare__telescope/README.md](sources/cloudflare__telescope/README.md); SHA-256: `0617b205cf52aa9f67e7c31a7c9f4fd8d74de1e551a131250f6e96a3c53574e5`
- lifecycle: строка 62: npm run dev:setup  # one-time setup: DB, migrations, Prisma client, type generation
- report: строка 21: - Web Vitals and performance metrics (`metrics.json`)

## UCSBarchlab/PyRTL

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/UCSBarchlab/PyRTL
- Категория: benchmark/testing candidate
- Описание: A collection of classes providing simple hardware specification, simulation, tracing, and testing suitable for teaching and research.  Simplicity, usability, clarity, and extensibility are the overarching goals, rather than performance or optimization.
- Уровень: automated README evidence extraction
- Снимок: [sources/UCSBarchlab__PyRTL/README.md](sources/UCSBarchlab__PyRTL/README.md); SHA-256: `7d65b85dfd6c119e1b2666092d24f0d6f0c2614ba57d23be98d254834dfeae68`

## testinggospels/camouflage

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/testinggospels/camouflage
- Категория: benchmark/testing candidate
- Описание: Camouflage is a backend mocking tool for HTTP, gRPC, Websockets and Thrift protocols, which helps you carry out your front end prototyping, unit testing, functional/performance testing in silos, in absence of one or more Microservices/APIs.
- Уровень: automated README evidence extraction
- Снимок: [sources/testinggospels__camouflage/README.md](sources/testinggospels__camouflage/README.md); SHA-256: `efb72f1aa8339e4dbd67144497b53349c4206063f1797011c076f3d31ef3f66d`
- report: строка 38: 🧮 Ability to fetch and condition the response using external data. Currently supported data sources are CSV and postgres. 🧮
- correctness: строка 52: ✅ Validation of requests and responses using your OpenApi schema's. ✅

## FyroxEngine/Fyrox

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/FyroxEngine/Fyrox
- Категория: workload: graphics
- Описание: 3D and 2D game engine written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/FyroxEngine__Fyrox/README.md](sources/FyroxEngine__Fyrox/README.md); SHA-256: `ffdc07a3c3049071c48fe50938f88d26a3da7edfe88f5ec4ad5a270e8e91abae`

## DioxusLabs/blitz

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DioxusLabs/blitz
- Категория: workload: graphics
- Описание: A radically modular HTML/CSS rendering engine
- Уровень: automated README evidence extraction
- Снимок: [sources/DioxusLabs__blitz/README.md](sources/DioxusLabs__blitz/README.md); SHA-256: `66b68853f20b244b7f321f6fb7d82a930baaedd15d1e3613314e2ec159052f31`
- gpu: строка 38: - WGPU texture integration example: `cargo run -rp wgpu_texture`

## linebender/resvg

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/linebender/resvg
- Категория: workload: graphics
- Описание: An SVG rendering library.
- Уровень: automated README evidence extraction
- Снимок: [sources/linebender__resvg/README.md](sources/linebender__resvg/README.md); SHA-256: `2e6b214593e6a0ff609a9fe5be3fb7f8c23d729b79d5b77a3081ba347a6e3e96`
- comparison: строка 24: And those are only SVG-to-PNG regression tests. This doesn't include tests in `resvg` dependencies.
- correctness: строка 23: To prove its correctness, `resvg` has a vast test suite that includes around 1600 tests.

## nical/lyon

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nical/lyon
- Категория: workload: graphics
- Описание: 2D graphics rendering on the GPU in rust using path tessellation.
- Уровень: automated README evidence extraction
- Снимок: [sources/nical__lyon/README.md](sources/nical__lyon/README.md); SHA-256: `5b9b16b870382cb86343fda1338c91fb8cf150ee7cae0cf0872b6ff816b11d06`
- gpu: строка 2: A path tessellation library written in rust for GPU-based 2D graphics rendering.

## cristicbz/rust-doom

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cristicbz/rust-doom
- Категория: workload: graphics
- Описание: A Doom Renderer written in Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/cristicbz__rust-doom/README.md](sources/cristicbz__rust-doom/README.md); SHA-256: `e7f1b4395fe1dc773e6a49d2484d9ed95a54a1eb427b6f9647a75bc1107b2e5a`

## chinedufn/percy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chinedufn/percy
- Категория: workload: graphics
- Описание: Build frontend browser apps with Rust + WebAssembly. Supports server side rendering.
- Уровень: automated README evidence extraction
- Снимок: [sources/chinedufn__percy/README.md](sources/chinedufn__percy/README.md); SHA-256: `aaab69c911a4640e5b41499c551bf138c73d67e8b0fb1e229165caf90ae57b77`
- async: строка 203: async function run ()  {

## framesurge/perseus

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/framesurge/perseus
- Категория: workload: graphics
- Описание: A state-driven web development framework for Rust with full support for server-side rendering and static generation.
- Уровень: automated README evidence extraction
- Снимок: [sources/framesurge__perseus/README.md](sources/framesurge__perseus/README.md); SHA-256: `680f0360e49052f32554d8f3734715f3ebf4c079158cec09e4d73630e5e07a8b`
- correctness: строка 13: -   🔧 Supports revalidation after time and/or with custom logic (updating rendered pages)

## not-fl3/miniquad

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/not-fl3/miniquad
- Категория: workload: graphics
- Описание: Cross platform rendering in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/not-fl3__miniquad/README.md](sources/not-fl3__miniquad/README.md); SHA-256: `b247dee6983326d965fb6cd060f468de031a59675d9545815083e88e0bd24241`
- gpu: строка 11: Miniquad aims to provide a graphics abstraction that works the same way on any platform with a GPU, being as light weight as possible while covering as many machines as possible.

## face-hh/webx

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/face-hh/webx
- Категория: workload: graphics
- Описание: An alternative for the World Wide Web - browse websites such as buss://yippie.rizz made in HTML, CSS and Lua. Custom web browser, custom HTML rendering engine, custom search engine, and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/face-hh__webx/README.md](sources/face-hh__webx/README.md); SHA-256: `31606de8d44049ac241f0b2990e3e0380903b61b97d6c2a0027423fa4a8c8fb3`
- report: строка 151: By publishing content to this platform ("Bussin Napture"/"Bussin Web X"), you agree to comply with all rules and regulations set forth by the administrators. The administrators reserve the right to interpret and enforce these rules at their discretion. To repo
- correctness: строка 112: 4.1 Validate if the libraries are installed adequately and set in PKG_CONFIG_PATH, command below should return the path to the libraries without any errors.

## mbrubeck/robinson

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mbrubeck/robinson
- Категория: workload: graphics
- Описание: A toy web rendering engine
- Уровень: automated README evidence extraction
- Снимок: [sources/mbrubeck__robinson/README.md](sources/mbrubeck__robinson/README.md); SHA-256: `af34c91c28684a645c399e89ba7804de2b409804afd68b64f54f247608d83a85`

## StarArawn/bevy_ecs_tilemap

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/StarArawn/bevy_ecs_tilemap
- Категория: workload: graphics
- Описание: A tilemap rendering crate for bevy which is more ECS friendly.
- Уровень: automated README evidence extraction
- Снимок: [sources/StarArawn__bevy_ecs_tilemap/README.md](sources/StarArawn__bevy_ecs_tilemap/README.md); SHA-256: `e5808a4bf7e5a57398bbc314191fd55addb89410326a44a69e973539d123a244`
- gpu: строка 15: - GPU powered animations.

## askama-rs/askama

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/askama-rs/askama
- Категория: workload: graphics
- Описание: A template rendering engine based on Jinja, generating type-safe Rust code at compile time.
- Уровень: automated README evidence extraction
- Снимок: [sources/askama-rs__askama/README.md](sources/askama-rs__askama/README.md); SHA-256: `49a6b37ce24f1a0556c9db40c920adca8c8d3ecf74d805ada242560c5f9d823e`

## BVE-Reborn/rend3

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/BVE-Reborn/rend3
- Категория: workload: graphics
- Описание: MAINTENCE MODE ---- Easy to use, customizable, efficient 3D renderer library built on wgpu.
- Уровень: automated README evidence extraction
- Снимок: [sources/BVE-Reborn__rend3/README.md](sources/BVE-Reborn__rend3/README.md); SHA-256: `3d5356997a92db3dd3a98c0de1bfdaf5bcf21aa0cbd8d0592f1f174b249a62ca`
- gpu: строка 12: Easy to use, customizable, efficient 3D renderer library built on wgpu.

## fschutt/printpdf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fschutt/printpdf
- Категория: workload: graphics
- Описание: Rust / WASM library for reading, writing and rendering PDF
- Уровень: automated README evidence extraction
- Снимок: [sources/fschutt__printpdf/README.md](sources/fschutt__printpdf/README.md); SHA-256: `5c3f7db2d4a8f41f526e4f02a87ef370577110ccf5a7af702b85d88c319dc378`
- report: строка 33: - Good enough for basic page layouting, book rendering, reports and forms,
- correctness: строка 301: - [PDF X/1-a Validator](https://www.pdf-online.com/osa/validate.aspx)

## HakanSeven12/OpenCADStudio

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/HakanSeven12/OpenCADStudio
- Категория: workload: graphics
- Описание: A CAD application built with Rust — 2D/3D drawing, DWG/DXF support, and GPU-accelerated rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/HakanSeven12__OpenCADStudio/README.md](sources/HakanSeven12__OpenCADStudio/README.md); SHA-256: `27efa6a9b8ddf7bd4715f7da620bec4183e3163a07b2c7635fb0b1e59d76b2eb`
- gpu: строка 66: - **GPU rendering** — accelerated 2D and 3D viewports through `wgpu`, with orthographic and perspective cameras.
- report: строка 58: The project is under active development. Keep backups of important production drawings and report reproducible problems through [GitHub Issues](https://github.com/HakanSeven12/OpenCADStudio/issues).
- correctness: строка 58: The project is under active development. Keep backups of important production drawings and report reproducible problems through [GitHub Issues](https://github.com/HakanSeven12/OpenCADStudio/issues).
- isolation: строка 191: Desktop plugins run in separate processes and communicate with the host through the versioned plugin API. The browser build does not load native plugins.

## josueBarretogit/manga-tui

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/josueBarretogit/manga-tui
- Категория: workload: graphics
- Описание: Terminal-based manga reader and downloader with image rendering support
- Уровень: automated README evidence extraction
- Снимок: [sources/josueBarretogit__manga-tui/README.md](sources/josueBarretogit__manga-tui/README.md); SHA-256: `27369f2c016e0026f8efd6a4079b97edf166f1b04b774eb3d7c410e893a88ea0`

## dfrg/swash

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dfrg/swash
- Категория: workload: graphics
- Описание: Font introspection, complex text shaping and glyph rendering.
- Уровень: automated README evidence extraction
- Снимок: [sources/dfrg__swash/README.md](sources/dfrg__swash/README.md); SHA-256: `043748ad4034902c2e835537e8a205bfa1f70bf1ef2609ce753193e5f0d37e80`
- memory: строка 46: - Zero transient heap allocations. All scratch buffers and caches are maintained by

## wahn/rs_pbrt

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/wahn/rs_pbrt
- Категория: workload: graphics
- Описание: Rust crate to implement a counterpart to the PBRT book's (3rd edition) C++ code. See also https://www.rs-pbrt.org/about ...
- Уровень: automated README evidence extraction
- Снимок: [sources/wahn__rs_pbrt/README.md](sources/wahn__rs_pbrt/README.md); SHA-256: `dcd50d099c38957e70010603a4b455bde49a601d15c66e0495cd3b30d5d99d63`

## alexheretic/glyph-brush

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/alexheretic/glyph-brush
- Категория: workload: graphics
- Описание: Fast GPU cached text rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/alexheretic__glyph-brush/README.md](sources/alexheretic__glyph-brush/README.md); SHA-256: `e35695b31df649bd9f2871562606dfe6b94d49059cb1a6c9fa72b4a9689f5a39`

## milos-agathon/forge3d

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/milos-agathon/forge3d
- Категория: workload: graphics
- Описание: Rust‑first, cross‑platform wgpu/WebGPU renderer exposed to Python for fast, headless 3D rendering. Built in Rust, shipped as Python wheels.
- Уровень: automated README evidence extraction
- Снимок: [sources/milos-agathon__forge3d/README.md](sources/milos-agathon__forge3d/README.md); SHA-256: `b73a3ad5c2b420c4073abaa2053c929a26ce95994f01989b006a4b81e5d33e7e`
- report: строка 139: | **Data** | COG streaming, CRS helpers, LAZ/COPC/EPT point clouds, 3D Tiles, GeoJSON, CityJSON, on-demand sample datasets |

## galileo-map/galileo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/galileo-map/galileo
- Категория: workload: graphics
- Описание: General purpose cross-platform GIS-rendering library written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/galileo-map__galileo/README.md](sources/galileo-map__galileo/README.md); SHA-256: `5fe193866c6ec61652284ebd5159f2b143a3bf87917ff838769a63b94c5db69a`
- memory: строка 101: * [ ] Advanced support for projections and CRSs
- gpu: строка 24: * Enjoy 3.6 million points heat up your room with GPU.
- report: строка 213: Feature requests, pull requests, bug reports, comments, questions and discussion are welcome. Please, follow the code

## fintelia/terra

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fintelia/terra
- Категория: workload: graphics
- Описание: A large scale terrain rendering library written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/fintelia__terra/README.md](sources/fintelia__terra/README.md); SHA-256: `434b8a3700aa99a3c86606a36bff6ea0a76a565a6a69591fd149e22479a147f2`
- gpu: строка 7: [wgpu](https://github.com/gfx-rs/wgpu).

## project-blinc/Blinc

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/project-blinc/Blinc
- Категория: workload: graphics
- Описание: A declarative, reactive UI system with first-class state machines, spring physics animations, and GPU-accelerated rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/project-blinc__Blinc/README.md](sources/project-blinc__Blinc/README.md); SHA-256: `e6e795552c2c8ddeab9037e8396399c7b060307132625cbcc438324f38331ac9`
- lifecycle: строка 45: A fresh clone builds every example with no extra setup — the sibling crates
- gpu: строка 14: **A GPU-accelerated, cross-platform UI framework** for building desktop, mobile, and web applications from a single Rust codebase.

## chinedufn/webgl-water-tutorial

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chinedufn/webgl-water-tutorial
- Категория: workload: graphics
- Описание: The source code for a tutorial on rendering water using WebGL + Rust + WebAssembly
- Уровень: automated README evidence extraction
- Снимок: [sources/chinedufn__webgl-water-tutorial/README.md](sources/chinedufn__webgl-water-tutorial/README.md); SHA-256: `ec0546bbca0ddc9b1ff14d3debbfdf81e3e84c91d929700bba1054d9a1bf0b29`

## faisalkindi/DLSS5oneclick

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/faisalkindi/DLSS5oneclick
- Категория: workload: graphics
- Описание: One-click setup of the leaked DLSS 5 neural-rendering build for any DX11/DX12 game on RTX 20–50, with or without DLSS. ReShade + RenoDX add-on (or OptiScaler engine); DLSS5-Feeder + LumeniteFX for games without DLSS; dlss5-bridge for DX11. Rust, single exe.
- Уровень: automated README evidence extraction
- Снимок: [sources/faisalkindi__DLSS5oneclick/README.md](sources/faisalkindi__DLSS5oneclick/README.md); SHA-256: `b6c6faa218366e68b63dfea0abc46b5c5a0650e5fb9bb7fbe98a6078ca14caca`
- lifecycle: строка 41: | 1 | ReShade **with add-on support**, dropped as `dxgi.dll` | `ReShade_Setup_<ver>_Addon.exe` on [reshade.me](https://reshade.me) (DLL pulled straight out of the installer, nothing is run) |
- gpu: строка 29: **Hybrid machines (laptop iGPU + dGPU, or an AMD/Intel display adapter alongside the NVIDIA card).** Windows decides which GPU a process starts on, and a process started on the integrated GPU has no NGX at all — every `NVSDK_NGX_D3D12_Init` answers `0xBAD00001
- report: строка 23: **RenoDX HDR mod (optional, since 0.8.0; engine-independent since 0.8.1).** The [RenoDX](https://github.com/clshortfuse/renodx) project publishes game-specific HDR / tone-mapping mods as ReShade add-ons (`renodx-<game>.addon64`). When the tool recognises the g

## mkeeter/fidget

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mkeeter/fidget
- Категория: workload: graphics
- Описание: blazing fast implicit surface evaluation
- Уровень: automated README evidence extraction
- Снимок: [sources/mkeeter__fidget/README.md](sources/mkeeter__fidget/README.md); SHA-256: `0f796257935aa05f620be81503b19b4145704ac6f403484feef48159150f2ad2`
- gpu: строка 35: GPU.
- correctness: строка 178: various axes: correctness, performance, usability, documentation, etc.  Since

## james-j-obrien/bevy_vector_shapes

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/james-j-obrien/bevy_vector_shapes
- Категория: workload: graphics
- Описание: A library for rendering vector shapes using the Bevy game engine
- Уровень: automated README evidence extraction
- Снимок: [sources/james-j-obrien__bevy_vector_shapes/README.md](sources/james-j-obrien__bevy_vector_shapes/README.md); SHA-256: `51986290b8960f0e05b65cf14b40168c972f5668bf1643ebc979a490324bb7bd`
- lifecycle: строка 47: .add_systems(Startup, setup)
- report: строка 18: Bevy Vector Shapes is in the early stages of development. You may encounter issues, but feel free to report them.

## renpenguin/display3d

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/renpenguin/display3d
- Категория: workload: graphics
- Описание: A command line interface for rendering and animating 3D objects
- Уровень: automated README evidence extraction
- Снимок: [sources/renpenguin__display3d/README.md](sources/renpenguin__display3d/README.md); SHA-256: `13a74b785d41e2cce3af254b2bfee44e70b41638b1bc3a7c447e62b8a6ecf321`

## Patryk27/strolle

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Patryk27/strolle
- Категория: workload: graphics
- Описание: Experimental real-time renderer with support for dynamic global illumination
- Уровень: automated README evidence extraction
- Снимок: [sources/Patryk27__strolle/README.md](sources/Patryk27__strolle/README.md); SHA-256: `8a11881f4564e3dde9f901f3ff1027088f024e734ddcc0c37da1e1e357e311fe`
- statistics: строка 137: - [SVGF](https://research.nvidia.com/publication/2017-07_spatiotemporal-variance-guided-filtering-real-time-reconstruction-path-traced)
- lifecycle: строка 99: 3. Setup & enjoy!
- gpu: строка 15: used on its own (through `wgpu`).

## ekzhang/rpt

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ekzhang/rpt
- Категория: workload: graphics
- Описание: A physically-based path tracer
- Уровень: automated README evidence extraction
- Снимок: [sources/ekzhang__rpt/README.md](sources/ekzhang__rpt/README.md); SHA-256: `2326621916cd57e93c3399cc7350241bce3ff47f063d293e2cfa26a95cb231b7`
- statistics: строка 21: - Supports iterative rendering, variance estimation, and firefly reduction
- async: строка 23: - Uses all CPU cores concurrently, scaling linearly up to 96 cores

## gdt050579/AppCUI-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gdt050579/AppCUI-rs
- Категория: workload: graphics
- Описание: AppCUI is a fast, cross-platform console and text-based user interface (CUI/TUI) framework for Rust. It combines a low-level console engine for input (mouse, keyboard, clipboard, etc.), colors, and rendering with a high-level, rich toolkit of widgets such as windows, menus, buttons, checkboxes, and many more, available for Windows, Linux and Mac.
- Уровень: automated README evidence extraction
- Снимок: [sources/gdt050579__AppCUI-rs/README.md](sources/gdt050579__AppCUI-rs/README.md); SHA-256: `fc988c1353f948f357132deb3c0b08bfa6481dbd415f0e3f829106b17ec5cec0`
- gpu: строка 293: - [ ] OpenGL / SDL / Vulkan support
- report: строка 229: - **Utilities** such as [Calculator](examples/calculator/), [CSV Viewer](examples/csv_viewer/), [Temperature Converter](examples/temperature_convertor/), [HexViewer](examples/hexview/), or a [Timer](examples/timer/)

## ratatui/ratatui-image

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ratatui/ratatui-image
- Категория: workload: graphics
- Описание: Ratatui widget for rendering image graphics in terminals that support it
- Уровень: automated README evidence extraction
- Снимок: [sources/ratatui__ratatui-image/README.md](sources/ratatui__ratatui-image/README.md); SHA-256: `62e58948d95a0db05c8b116e1abfa14e39d6c418e79bf3b48a263a6dcee2a29f`
- memory: строка 87: // Rendering the transformed data is now cheap.
- async: строка 117: be offloaded to another thread or async task, to keep the UI responsive (see

## bonsairobo/building-blocks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/bonsairobo/building-blocks
- Категория: workload: graphics
- Описание: A voxel library for real-time applications.
- Уровень: automated README evidence extraction
- Снимок: [sources/bonsairobo__building-blocks/README.md](sources/bonsairobo__building-blocks/README.md); SHA-256: `46d2552cca94cb5b007cfd2d4f26338cc37918b7155a0fbd3712340293e03782`

## veeenu/hudhook

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/veeenu/hudhook
- Категория: workload: graphics
- Описание: A videogame overlay framework written in Rust, supporting DirectX and OpenGL
- Уровень: automated README evidence extraction
- Снимок: [sources/veeenu__hudhook/README.md](sources/veeenu__hudhook/README.md); SHA-256: `fc8022c2a9af8d266b97d0bcc7c7525822c2109d3db026e751980c6e52101402`

## zesterer/euc

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/zesterer/euc
- Категория: workload: graphics
- Описание: A software rendering crate that lets you write shaders with Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/zesterer__euc/README.md](sources/zesterer__euc/README.md); SHA-256: `fefa150986d26725b8375e7665f1623c1d24b49ac0584f42a1601c325ceee54d`
- gpu: строка 14: // You can add fields to this type, like uniforms in traditional GPU shader programs.
- correctness: строка 173: - Correctness, where doing so doesn't significantly compromise performance.

## cxreiff/bevy_ratatui_camera

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cxreiff/bevy_ratatui_camera
- Категория: workload: graphics
- Описание: A bevy plugin for rendering your bevy app to the terminal using ratatui.
- Уровень: automated README evidence extraction
- Снимок: [sources/cxreiff__bevy_ratatui_camera/README.md](sources/cxreiff__bevy_ratatui_camera/README.md); SHA-256: `35a18dd3165ba39718771889bbbbbd01bf46a10b6b57cc9d3145535f3755b4b8`
- lifecycle: строка 47: .add_systems(Startup, setup_scene_system)
- gpu: строка 167: prepass will be copied back from the GPU alongside your camera's render image:

## rustq/vue-skia

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rustq/vue-skia
- Категория: workload: graphics
- Описание: Skia based 2d graphics vue rendering library. It is based on Rust to implement software rasterization to perform rendering. 基于 Skia 的 2D 图形 Vue 渲染库 —— 使用 Rust 语言实现纯软件光栅化
- Уровень: automated README evidence extraction
- Снимок: [sources/rustq__vue-skia/README.md](sources/rustq__vue-skia/README.md); SHA-256: `9c8d7b7c85a6a7d949a50a38763d8eac82ceab17221150d9481bc4e04b4808e8`

## thatmagicalcat/txm

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/thatmagicalcat/txm
- Категория: workload: graphics
- Описание: Terminal Math rendering engine
- Уровень: automated README evidence extraction
- Снимок: [sources/thatmagicalcat__txm/README.md](sources/thatmagicalcat__txm/README.md); SHA-256: `e65813d7bf188b13d3ff41b05f029f7e0282049f38441ba7e65e2e1fb4bdd82b`

## kurtkuehnert/bevy_terrain

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kurtkuehnert/bevy_terrain
- Категория: workload: graphics
- Описание: A terrain rendering plugin for the bevy game engine.
- Уровень: automated README evidence extraction
- Снимок: [sources/kurtkuehnert__bevy_terrain/README.md](sources/kurtkuehnert__bevy_terrain/README.md); SHA-256: `280e39b6d5f5ffafd7967f5bc02f297061662a83c60489999f1148fdd78b23b1`

## 001TMF/ProteinView

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/001TMF/ProteinView
- Категория: workload: graphics
- Описание: Terminal protein structure viewer — interactive 3D visualization of PDB/mmCIF structures with cartoon ribbons, braille rendering, and Sixel/Kitty graphics
- Уровень: automated README evidence extraction
- Снимок: [sources/001TMF__ProteinView/README.md](sources/001TMF__ProteinView/README.md); SHA-256: `fd191c0f4251cdd050b0b0a8b6e3c7502a64d30f1517f74a6480b5aef7b9d2a8`
- lifecycle: строка 187: setup. It is therefore safe to call from another terminal application.
- gpu: строка 261: | **Element** | CPK coloring (C, N, O, S, P, metals). |
- report: строка 211: The server renders the initial frame, then writes a `ready` JSON object to

## toxoidengine/toxoid

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/toxoidengine/toxoid
- Категория: workload: graphics
- Описание: A modern, cross-platform, highly modular / decoupled, data-driven, ECS-based game engine written in Rust with scripting support for C#, JavaScript and Rust to Rust (WASM), hot-reloading, WebGPU rendering, and web target support.
- Уровень: automated README evidence extraction
- Снимок: [sources/toxoidengine__toxoid/README.md](sources/toxoidengine__toxoid/README.md); SHA-256: `0cf44f96ea8cc1c3d5966e175d7714332bbb7d8c67ae3d5f2e0c4d26ccb31dac`

## guilhermeprokisch/see

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/guilhermeprokisch/see
- Категория: workload: graphics
- Описание: A cute cat(1) for the terminal with advanced code viewing, Markdown rendering, 🌳  tree-sitter syntax highlighting, images view and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/guilhermeprokisch__see/README.md](sources/guilhermeprokisch__see/README.md); SHA-256: `da00d7be805cd5a4df58d9c06700948d1bd63c2ceb899be51586770b866dba48`
- lifecycle: строка 99: If your shell config is managed by Nix/Home Manager or another setup that makes files like `~/.zshrc` read-only, disable the installer's PATH edits and source Cargo's env file yourself:

## staff-rs/staff

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/staff-rs/staff
- Категория: workload: graphics
- Описание: Music theory and score rendering library with midi, notes, chords, scales, and more.
- Уровень: automated README evidence extraction
- Снимок: [sources/staff-rs__staff/README.md](sources/staff-rs__staff/README.md); SHA-256: `11295aa4020848ab922208414c8445c12026f3479e0d5ce741181d5fd4cec5d0`

## Dicklesworthstone/frankentui

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Dicklesworthstone/frankentui
- Категория: workload: graphics
- Описание: Minimal, high-performance terminal UI kernel with diff-based rendering, inline mode, and RAII terminal cleanup
- Уровень: automated README evidence extraction
- Снимок: [sources/Dicklesworthstone__frankentui/README.md](sources/Dicklesworthstone__frankentui/README.md); SHA-256: `5545efca1aaf37c92373b67bc08c055e34d7728f72ff0cc8520e0da1bfcfdd5c`
- statistics: строка 67: | **Bayesian intelligence** | Statistical diff strategy, resize coalescing, capability detection | BOCPD, VOI, conformal prediction, e‑processes |
- comparison: строка 120: Each screen is also a snapshot test target. `BLESS=1 cargo test -p ftui-demo-showcase` updates baselines.
- lifecycle: строка 200: 5. **Zero-surprise teardown.** RAII cleanup, even when apps crash.
- memory: строка 993: - Allocation budget alerts
- async: строка 970: - `max_interval_ms=1000` (latency bound)
- report: строка 69: | **46 demo screens** | Dashboard, visual effects, widget gallery, layout lab, and more | `cargo run -p ftui-demo-showcase` |
- correctness: строка 16: High‑performance terminal UI kernel -- 850K+ lines of Rust across 20 crates, 80+ widget/stateful-widget implementations, 46 interactive demo screens, a Bayesian intelligence layer, resizable pane workspaces, and in-tree web/WASM backends -- focused on correctn

## Valerioageno/ssr-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Valerioageno/ssr-rs
- Категория: workload: graphics
- Описание: Server side rendering on rust servers using the v8 engine for parse and evaluate the JavaScript code.
- Уровень: automated README evidence extraction
- Снимок: [sources/Valerioageno__ssr-rs/README.md](sources/Valerioageno__ssr-rs/README.md); SHA-256: `cee2511be6d9e7a598c51f9b85b7714d467cb859e91c0452e55220c239e0b0fb`
- lifecycle: строка 112: For the reasons above parallel computation is a better choice. Following actix-web setup:
- async: строка 131: async fn main() -> std::io::Result<()> {

## lzanini/mdbook-katex

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lzanini/mdbook-katex
- Категория: workload: graphics
- Описание: A preprocessor for mdBook, rendering LaTex equations to HTML at build time.
- Уровень: automated README evidence extraction
- Снимок: [sources/lzanini__mdbook-katex/README.md](sources/lzanini__mdbook-katex/README.md); SHA-256: `29bcbe7ff6fb1e5243ec7f1826e426ff7be4ee832afeb74208e3a9a1be521114`
- lifecycle: строка 41: ### Basic setup

## TritonDataCenter/statemap

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/TritonDataCenter/statemap
- Категория: workload: graphics
- Описание: Software for rendering statemaps
- Уровень: automated README evidence extraction
- Снимок: [sources/TritonDataCenter__statemap/README.md](sources/TritonDataCenter__statemap/README.md); SHA-256: `d854ed30956f958500304b8e31b4699c2d9b0c0bffa866b5fdc0fcf2b4211940`
- report: строка 95: concatenated JSON.

## kazan-3d/kazan

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kazan-3d/kazan
- Категория: workload: graphics
- Описание: Mirror; Work-in-progress software-rendering Vulkan implementation
- Уровень: automated README evidence extraction
- Снимок: [sources/kazan-3d__kazan/README.md](sources/kazan-3d__kazan/README.md); SHA-256: `9ae9e589d49d30a38823fa516189e5cb9ff9efeafe4d3796bb31e5d0502c6a3f`
- gpu: строка 3: Kazan is an in-progress Vulkan driver that supports cross-platform software rendering, and (eventually) is a driver for [libre-riscv.org's RISC-V based GPU](https://libre-riscv.org/3d_gpu/).

## kurtkuehnert/terrain_renderer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kurtkuehnert/terrain_renderer
- Категория: workload: graphics
- Описание: Real-world terrain rendering prototype built in Rust with Bevy, developed as part of my Bachelor thesis on large-scale GPU terrain systems.
- Уровень: automated README evidence extraction
- Снимок: [sources/kurtkuehnert__terrain_renderer/README.md](sources/kurtkuehnert__terrain_renderer/README.md); SHA-256: `7118789a30f49f1666c00b8230b373eee9bd23e029e84813839f4fbf09990f07`
- gpu: строка 24: This fully GPU-based algorithm subdivides a quadtree covering the terrain into small tiles, which can be culled in parallel, and are morphed seamlessly in the vertex shader, resulting in a densely and temporally consistent triangulated mesh.
- report: строка 82: Simply copy the appropriate one into the directory of the terrain. (E.g. `my_terrain_dir/MyTerrain/my_urls.csv`)

## fu5ha/rendy-pbr

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fu5ha/rendy-pbr
- Категория: workload: graphics
- Описание: PBR rendering example/experiment with rendy
- Уровень: automated README evidence extraction
- Снимок: [sources/fu5ha__rendy-pbr/README.md](sources/fu5ha__rendy-pbr/README.md); SHA-256: `ef804cdcfb6cbfdb52eec1c2ac0ac61f9809c1a9847b9ac58c460c4863b3f82e`
- lifecycle: строка 32: Second, one of the dependencies, shaderc, is a little more complex to set up. See the [Setup](https://github.com/google/shaderc-rs#setup) section of the `shaderc-rs` repo for more information.
- gpu: строка 36: cargo run --features <vulkan | metal> [--release]

## fslabs/bevy_polyline

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fslabs/bevy_polyline
- Категория: workload: graphics
- Описание: Polyline Rendering for Bevy
- Уровень: automated README evidence extraction
- Снимок: [sources/fslabs__bevy_polyline/README.md](sources/fslabs__bevy_polyline/README.md); SHA-256: `f94f9732db7fe66281553e4cf1f192d24bd8f9b21e1258f2a3163ffaeefe48db`
- gpu: строка 22: Bevy Polyline closely mimics the way `Mesh`es are rendered in Bevy. It works internally by passing a minimal Instance Buffer to the GPU, containing only the line segment endpoints and then completely determines all vertex positions within the vertex shader, su

## openooxml/betteroffice

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/openooxml/betteroffice
- Категория: workload: graphics
- Описание: Native OOXML engines in Rust - DOCX, XLSX & PPTX editing, rendering and real-time collaboration, compiled to WASM for the web.
- Уровень: automated README evidence extraction
- Снимок: [sources/openooxml__betteroffice/README.md](sources/openooxml__betteroffice/README.md); SHA-256: `ab11383f159dc76e2f72401e7703d8ac52ba81dc1113d32c3fc727e47d7468c5`

## panxinmiao/myth

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/panxinmiao/myth
- Категория: workload: graphics
- Описание: A High-Performance Rendering Engine for Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/panxinmiao__myth/README.md](sources/panxinmiao__myth/README.md); SHA-256: `64856d511fa0cc645d25e1e8a63f8e1532e35b967abaa9110602e9021197b521`
- gpu: строка 8: **A high-performance Rust rendering engine based on wgpu.**

## teoxoy/encase

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/teoxoy/encase
- Категория: workload: graphics
- Описание: Provides a mechanism to lay out data into GPU buffers according to WGSL's memory layout rules
- Уровень: automated README evidence extraction
- Снимок: [sources/teoxoy__encase/README.md](sources/teoxoy__encase/README.md); SHA-256: `e3cdeac622d8ed4df9289fd1d1116863ea15a4b2ad4a20ee9dfb59e1bebe91e8`
- gpu: строка 3: Provides a mechanism to lay out data into GPU buffers ensuring WGSL's memory layout requirements are met.

## junkdog/beamterm

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/junkdog/beamterm
- Категория: workload: graphics
- Описание: A sub-millisecond terminal text rendering system for WebGL2 and OpenGL 3.3
- Уровень: automated README evidence extraction
- Снимок: [sources/junkdog__beamterm/README.md](sources/junkdog__beamterm/README.md); SHA-256: `7280c3401068bd10abef48e00cbf702300d1180a9f2cb9b501870c6dffcc2086`
- lifecycle: строка 467: ### Development Setup (Native)
- memory: строка 39: | Memory Usage                    | ~8.9MB           | ~8.9MB                   |
- gpu: строка 1: ## beamterm - A GPU-Accelerated Terminal Renderer
- async: строка 11: in rendering throughput). It also powers [Ratzilla][rz]'s WebGL2 backend.

## Dylanmurzello/zed-android-port

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Dylanmurzello/zed-android-port
- Категория: workload: graphics
- Описание: Native Android port of the Zed code editor,  gpui rendering through Vulkan, integrated Termux for LSPs and tooling, remote SSH support. Runs as an APK on tablets and Android desktop modes.
- Уровень: automated README evidence extraction
- Снимок: [sources/Dylanmurzello__zed-android-port/README.md](sources/Dylanmurzello__zed-android-port/README.md); SHA-256: `63874157187ed7c3aadefde632c6778b6f99fa5b75d2ed2dd000101d845ec0d3`
- statistics: строка 43: 2. **Runtime adapter.** A picker asks where every subprocess (shells, LSPs, terminal, git, ssh) should run. Nothing is pre-selected; you pick one of three. **Bootstrap** is the no-root option: a Termux-derived userland that runs entirely from the app's data di
- lifecycle: строка 81: Setup, once the bridge lands:
- gpu: строка 19: Zdroid is an independent port of [Zed](https://zed.dev) for Android, not affiliated with Zed Industries. Upstream's `Editor`, `Workspace`, `Project`, `Search`, `GitGraph`, `Extensions`, and `Terminal` crates run unchanged on a custom `gpui_android` platform ba
- isolation: строка 43: 2. **Runtime adapter.** A picker asks where every subprocess (shells, LSPs, terminal, git, ssh) should run. Nothing is pre-selected; you pick one of three. **Bootstrap** is the no-root option: a Termux-derived userland that runs entirely from the app's data di

## radiant-labs/radiantkit

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/radiant-labs/radiantkit
- Категория: workload: graphics
- Описание: Cross-platform open-source framework to build graphics applications (like Figma, Canva, Miro, etc). Uses Rust, Wasm, wgpu and epaint.
- Уровень: automated README evidence extraction
- Снимок: [sources/radiant-labs__radiantkit/README.md](sources/radiant-labs__radiantkit/README.md); SHA-256: `5f691a9371e672271fc17215f33afda4443f985a221b2fb45c148b64af14f431`

## Indosaram/hwpers

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Indosaram/hwpers
- Категория: workload: graphics
- Описание: A Rust library for parsing Korean Hangul Word Processor (HWP) files with full layout rendering support
- Уровень: automated README evidence extraction
- Снимок: [sources/Indosaram__hwpers/README.md](sources/Indosaram__hwpers/README.md); SHA-256: `fc0294804cd28387d1b0b3d0ae1d9da0dccfdfc16fc6172804389005fdec98ef`
- comparison: строка 82: show_baselines: false,
- memory: строка 18: - **Zero-copy Parsing**: Efficient parsing with minimal memory allocation

## asinglebit/guitar

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/asinglebit/guitar
- Категория: workload: graphics
- Описание: A terminal based git client with fast topological & chronological graph rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/asinglebit__guitar/README.md](sources/asinglebit__guitar/README.md); SHA-256: `241ba10b066ce0dc4b9b07a2e0f6dc3fb292ef30959778457fcb32d14e8cdc3a`
- report: строка 54: Use it carefully on important repositories. Keep backups, understand what the selected row and focused pane mean before using action mode, and report issues when behavior is surprising.
- correctness: строка 629: The `+ add remote` row opens prompts for a remote name and fetch URL. Remote names are validated with Git's remote-name rules.

## nikomatsakis/skill-tree

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/nikomatsakis/skill-tree
- Категория: workload: graphics
- Описание: Skill-tree rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/nikomatsakis__skill-tree/README.md](sources/nikomatsakis__skill-tree/README.md); SHA-256: `e72e6daa61b9d3cf2e03eeda5541abf9d5ac8bb124d0a37ab3cd5ed08823b2fe`
- correctness: строка 37: skill-tree validate

## StarArawn/bevy_tiled

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/StarArawn/bevy_tiled
- Категория: workload: graphics
- Описание: A plugin for rendering tiled maps.
- Уровень: automated README evidence extraction
- Снимок: [sources/StarArawn__bevy_tiled/README.md](sources/StarArawn__bevy_tiled/README.md); SHA-256: `30fdea18b0240a450d98b7d598a7ffc096f8863e38e289b42472a3acaafbcd55`
- lifecycle: строка 19: ## Basic Setup

## sarkahn/bevy_ascii_terminal

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sarkahn/bevy_ascii_terminal
- Категория: workload: graphics
- Описание: A simple terminal for rendering ascii in bevy.
- Уровень: automated README evidence extraction
- Снимок: [sources/sarkahn__bevy_ascii_terminal/README.md](sources/sarkahn__bevy_ascii_terminal/README.md); SHA-256: `e10d2ad10f89cad63e580a99d3fc339a37178a32731457eded479cb20a923999`
- lifecycle: строка 28: .add_systems(Startup, setup)

## avsaase/walksnail-osd-tool

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/avsaase/walksnail-osd-tool
- Категория: workload: graphics
- Описание: Cross-platform tool for rendering the flight controller OSD and SRT data from the Walksnail Avatar HD FPV system on top of the goggle or VRX recording
- Уровень: automated README evidence extraction
- Снимок: [sources/avsaase__walksnail-osd-tool/README.md](sources/avsaase__walksnail-osd-tool/README.md); SHA-256: `e70da271128a5d121a4b5fabd3da1a44b0a69829736dc559ed66174424f8ad6b`
- lifecycle: строка 56: - [kirek007/ws-osd-py](https://github.com/kirek007/ws-osd-py): Python-based tool with GUI and CLI. No longer maintained in favor of this project but has a few features that this project currently lacks. Depending on your OS it can require some manual setup due

## chinedufn/landon

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chinedufn/landon
- Категория: workload: graphics
- Описание: A collection of tools, data structures and methods for exporting Blender data (such as meshes and armatures) and preparing it for your rendering pipeline.
- Уровень: automated README evidence extraction
- Снимок: [sources/chinedufn__landon/README.md](sources/chinedufn__landon/README.md); SHA-256: `199624af9cc4649d8b81d09181e4bd4e86f6b3b212a18d054446f360afedaf40`
- report: строка 19: and then parse that COLLADA into JSON.

## austintheriot/wrend

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/austintheriot/wrend
- Категория: workload: graphics
- Описание: A framework-agnostic Rust/WASM + WebGL2 Rendering library, compatible with calling from both Rust and JavaScript on the web.
- Уровень: automated README evidence extraction
- Снимок: [sources/austintheriot__wrend/README.md](sources/austintheriot__wrend/README.md); SHA-256: `7c4922418707cdf0f8e03c03bf986b53292f36bb5f365e28d893d1e4f8b94e28`
- gpu: строка 44: I initially started this project as a software ray tracer running on Rust/WASM alone, but the render times that I experienced were so frustratingly slow that I quickly looked into implementing a hardware ray tracer that could take better advantage of the GPU's

## m13253/FaithType

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/m13253/FaithType
- Категория: workload: graphics
- Описание: Modify fonts to remove bitmap and disable gridfit for Windows font rendering
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## manankarnik/bevy_generative

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/manankarnik/bevy_generative
- Категория: workload: graphics
- Описание: Real-time procedural generation of maps, textures, terrain, planets and more!
- Уровень: automated README evidence extraction
- Снимок: [sources/manankarnik__bevy_generative/README.md](sources/manankarnik__bevy_generative/README.md); SHA-256: `28df1a38650a5797bdf5941a310ef4774b6315e73e234efe54ac74b295c52735`
- report: строка 51: Contributions are welcome! Issues, pull requests, feature requests and bug reports are appreciated. If you'd like to contribute to this project, please follow these steps:

## Jesterhearts/ratatui-wgpu

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Jesterhearts/ratatui-wgpu
- Категория: workload: graphics
- Описание: A wgpu based rendering backend for ratatui.
- Уровень: automated README evidence extraction
- Снимок: [sources/Jesterhearts__ratatui-wgpu/README.md](sources/Jesterhearts__ratatui-wgpu/README.md); SHA-256: `32e31dd4bcc67ca1cfa405b30a2f7b37584a0a82886b90c965af989e08ae7ac7`
- gpu: строка 1: # ratatui-wgpu

## atuinsh/eye-declare

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/atuinsh/eye-declare
- Категория: workload: graphics
- Описание: A declarative inline TUI rendering library for Rust, built on Ratatui
- Уровень: automated README evidence extraction
- Снимок: [sources/atuinsh__eye-declare/README.md](sources/atuinsh__eye-declare/README.md); SHA-256: `44e06218fc52cd5b1e6dd42783ec39637ee6019fbd7aa617a15359981457d2bb`
- memory: строка 19: Emitting a finished block is I/O, irreversible like `println!`, so it happens in your update function (`ctx.push`). The live tail is the only thing that behaves like a screen, so it's the only thing described by a view function — re-run every frame, pure, chea
- async: строка 21: If you've written Elm, iced, or Redux, the rest is familiar: your struct is the model, messages drive `update`, async work arrives as streams of messages, and cancellation is dropping a handle. There's no reconciliation, no keys, no dirty tracking, no framewor

## Logicalshift/flo_draw

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Logicalshift/flo_draw
- Категория: workload: graphics
- Описание: 2D rendering libraries for Rust and FlowBetween
- Уровень: automated README evidence extraction
- Снимок: [sources/Logicalshift__flo_draw/README.md](sources/Logicalshift__flo_draw/README.md); SHA-256: `2bf4df30fedc19b4b373e9e7b78e551a46953b36e0ce10002e78657782871473`
- gpu: строка 62: * `flo_render` is an abstraction API that converts low-level rendering instructions to a graphics API (OpenGL and Metal are supported)

## DavidPeicho/albedo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DavidPeicho/albedo
- Категория: workload: graphics
- Описание: Efficient and easy to use rendering framework for real-time visualization based on WebGPU
- Уровень: automated README evidence extraction
- Снимок: [sources/DavidPeicho__albedo/README.md](sources/DavidPeicho__albedo/README.md); SHA-256: `f4607fccc8d1c9cc04b9a9e357a7adc03d119ac70c5bf96a9d356e70c6983809`
- gpu: строка 26: * As close as [wgpu](https://github.com/gfx-rs/wgpu) as possible for integration

## gents83/INOX

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gents83/INOX
- Категория: workload: graphics
- Описание: Rust Game engine integrated in Blender [WebGPU ready]
- Уровень: automated README evidence extraction
- Снимок: [sources/gents83__INOX/README.md](sources/gents83__INOX/README.md); SHA-256: `96a497d644bdda85be4da26609ef3925fc40d1c912b7e6fb139939af123e9904`
- gpu: строка 42: - The rendering engine should support different GFX API as well (like Vulkan, DirectX, Metal, etc)

## lcnr/crow

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lcnr/crow
- Категория: workload: graphics
- Описание: A simple pixel perfect 2D rendering engine
- Уровень: automated README evidence extraction
- Снимок: [sources/lcnr__crow/README.md](sources/lcnr__crow/README.md); SHA-256: `9c86f83c17bc5afb0590fd9f292888b17bf206d80b0c5b07b68fa9ffa7c1b00a`
- gpu: строка 25: This crate requires a GPU supporting OpenGL Version **3.3**.

## Smithay/drm-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Smithay/drm-rs
- Категория: workload: graphics
- Описание: A low-level abstraction of the Direct Rendering Manager API
- Уровень: automated README evidence extraction
- Снимок: [sources/Smithay__drm-rs/README.md](sources/Smithay__drm-rs/README.md); SHA-256: `10a848728050b068bf4f6671b475aa2e5e046d778a3ebd302280ddec85a8963b`
- gpu: строка 60: let gpu = Card::open("/dev/dri/card0");

## fslabs/bevy_aabb_instancing

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fslabs/bevy_aabb_instancing
- Категория: workload: graphics
- Описание: Fast AABB rendering plugin for Bevy Engine.
- Уровень: automated README evidence extraction
- Снимок: [sources/fslabs__bevy_aabb_instancing/README.md](sources/fslabs__bevy_aabb_instancing/README.md); SHA-256: `45483d165e401945dc93aaad161c68076b98574badc56deff8461032f496fe08`

## yeicor/sdf-viewer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/yeicor/sdf-viewer
- Категория: workload: graphics
- Описание: A fast and cross-platform Signed Distance Function (SDF) viewer, easily integrated with your SDF library.
- Уровень: automated README evidence extraction
- Снимок: [sources/yeicor__sdf-viewer/README.md](sources/yeicor__sdf-viewer/README.md); SHA-256: `b1e5f79dc220d1304797b00f332841ec52e00a5afaab738b4da9fe06439b8c34`
- gpu: строка 43: - [x] Interactive framerate, even while loading (uses the GPU for viewing the SDF).
- report: строка 146: from the hit points. The shader simply walks along the ray for each pixel, moving by the amount of distance reported by

## skyzh/raytracer.rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/skyzh/raytracer.rs
- Категория: workload: graphics
- Описание: ⚡ A high-performance path tracer implemented in Rust based on "Ray Tracing in One Weekend" featuring static dispatch, multi-threaded rendering and a variety of preset scenes.
- Уровень: automated README evidence extraction
- Снимок: [sources/skyzh__raytracer.rs/README.md](sources/skyzh__raytracer.rs/README.md); SHA-256: `4e20dbb177a0ccd3e9e732ccb15bc891477fc2cc804dc82b642a839e2a26fbd2`

## kosumic/rustracer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kosumic/rustracer
- Категория: workload: graphics
- Описание: Vulkan path tracing with Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/kosumic__rustracer/README.md](sources/kosumic__rustracer/README.md); SHA-256: `4aded56c1a1f6ce40d316b1d5fce5cc5a344d04026ec392387857c32162fb415`
- gpu: строка 12: A PBR [glTF 2.0](https://www.khronos.org/gltf) renderer based on Vulkan ray-tracing, written in Rust.
- async: строка 64: * [x] Async model loading

## dtcristo/bevy_pixels

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dtcristo/bevy_pixels
- Категория: workload: graphics
- Описание: Bevy plugin that uses Pixels (a tiny pixel buffer) for rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/dtcristo__bevy_pixels/README.md](sources/dtcristo__bevy_pixels/README.md); SHA-256: `13e5730e89e9ba7a4fcadaaac58d81ff7255ec33c0ebb4cb85f6b74a5e1c2d36`
- gpu: строка 34: Add `bevy` and `bevy_pixels` to `Cargo.toml`. Be sure to disable `bevy`'s `render` and `bevy_wgpu` features (with `default-features = false`) as they will conflict with rendering provided by `bevy_pixels`.

## micahrj/ochre

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/micahrj/ochre
- Категория: workload: graphics
- Описание: high-quality anti-aliased vector graphics rendering on the GPU
- Уровень: automated README evidence extraction
- Снимок: [sources/micahrj__ochre/README.md](sources/micahrj__ochre/README.md); SHA-256: `d793d0efec5adaa52a4592591663b2705eec66945e6f1eb58a9f8223100a5fc8`
- gpu: строка 6: High-quality anti-aliased vector graphics rendering on the GPU.

## arcana-engine/sierra

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/arcana-engine/sierra
- Категория: workload: graphics
- Описание: Vulkan-lite GPU API
- Уровень: automated README evidence extraction
- Снимок: [sources/arcana-engine__sierra/README.md](sources/arcana-engine__sierra/README.md); SHA-256: `b770779be250fdeed395231e308af8c9bf0abcfa6ad83c28193af89cd6053ad1`
- memory: строка 13: sierra does both memory and descriptor allocation automatically.
- gpu: строка 9: Sierra is Vulkan-lite API, focused on ease of use

## alibaba/font-toolkit

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/alibaba/font-toolkit
- Категория: workload: graphics
- Описание: A Rust 🦀️ font loading, positioning and rendering toolkit
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## Dicklesworthstone/rich_rust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Dicklesworthstone/rich_rust
- Категория: workload: graphics
- Описание: Beautiful terminal output for Rust inspired by Python's Rich: tables, panels, syntax highlighting, progress bars, and full-color rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/Dicklesworthstone__rich_rust/README.md](sources/Dicklesworthstone__rich_rust/README.md); SHA-256: `1a98c98f0828339e867f15306d3a066f807715b29331871e6ca85badc83fa9e7`
- memory: строка 232: - `owo-colors`: Zero-allocation, const colors
- async: строка 875: - **No async:** Rendering is synchronous; wrap in `spawn_blocking` if needed
- report: строка 59: - **JSON** — pretty-printed, theme-aware output
- correctness: строка 816: - `--scene` must validate known names and print an “available scenes” list on error.

## microsoft/webui

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/microsoft/webui
- Категория: workload: graphics
- Описание: Server Side Rendering without JavaScript on the server. No virtual DOM. No JSX. Just HTML, streamed from any language.
- Уровень: automated README evidence extraction
- Снимок: [sources/microsoft__webui/README.md](sources/microsoft__webui/README.md); SHA-256: `f586ce8696d92b710a9c0fc395708bbb4f73d3a6dae4322791639544dcff2240`
- report: строка 143: | Report a bug | [Choose an issue template](https://github.com/microsoft/webui/issues/new/choose) |
- correctness: строка 33: The NuGet package restores platform-specific `Microsoft.WebUI.Runtime.*` native assets transitively. Azure release validation retains unsigned npm tarballs, unsigned crate archives, signed NuGet packages, and a standalone asset folder containing direct-downloa

## riley-williams/buoyant

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/riley-williams/buoyant
- Категория: workload: graphics
- Описание: Declarative UI layout and rendering for no_std targets, inspired by SwiftUI
- Уровень: automated README evidence extraction
- Снимок: [sources/riley-williams__buoyant/README.md](sources/riley-williams__buoyant/README.md); SHA-256: `f93700d0e7d58a9db27e6db79fe6ef0ad15549e9d315cca9b99db6fec963b626`

## Panzerschrek/Square-Wheel

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Panzerschrek/Square-Wheel
- Категория: workload: graphics
- Описание: Advanced software renderer written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/Panzerschrek__Square-Wheel/README.md](sources/Panzerschrek__Square-Wheel/README.md); SHA-256: `85182e7e060a5334378d61c2b9fd502e12d36bd648f7330f04c73a24f8b911ed`
- memory: строка 293: Such animation is cheap (requires no run-time computations) but may be boring.
- gpu: строка 21: * Specular lighting (directional lightmap-based) for metals and non-metals
- report: строка 282: JSON files are used to store materials.

## mikialex/rendiation

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mikialex/rendiation
- Категория: workload: graphics
- Описание: Rendiation Rendering Framework
- Уровень: automated README evidence extraction
- Снимок: [sources/mikialex__rendiation/README.md](sources/mikialex__rendiation/README.md); SHA-256: `3c09860f04a2a3e6e4c2e53402b947601b0656b3fdf0f478625bc019178eb984`
- memory: строка 29: - heap-tools: useful tools to debug and monitoring memory leak related issue
- gpu: строка 5: For performance, RRF combines the innovative ideas of incremental computation and relational reactive programming, creating an incremental compute solution: the reactive query graph. Leveraging this, RRF reactively manages all derived data, internal state, cac

## o2sh/provok

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/o2sh/provok
- Категория: workload: graphics
- Описание: Text rendering
- Уровень: automated README evidence extraction
- Снимок: [sources/o2sh__provok/README.md](sources/o2sh__provok/README.md); SHA-256: `908e10b9a9d7bfdd06b3466f20a6cd20854f4a0294587df09f97a8c66c8b2368`
- report: строка 9: Provok is fed with a [JSON file](./examples/0.json) that consists of an array of word alongside their display parameters (fg_color, boldness, italic, etc.):

## maplibre/maplibre-native-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/maplibre/maplibre-native-rs
- Категория: workload: graphics
- Описание: Rust bindings to the MapLibre Native map rendering engine
- Уровень: automated README evidence extraction
- Снимок: [sources/maplibre__maplibre-native-rs/README.md](sources/maplibre__maplibre-native-rs/README.md); SHA-256: `4469840e1973a44d09ae38fe93812c5e98be16ced4c883de1bf26f98525f953e`
- gpu: строка 21: - `vulkan` (default on Linux/Windows): `cargo build --features vulkan`
- report: строка 29: - `json`: load styles and layers from JSON via [`serde_json`](https://lib.rs/serde_json)

## jadedbay/bevy_procedural_grass

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/jadedbay/bevy_procedural_grass
- Категория: workload: graphics
- Описание: A bevy plugin for rendering grass
- Уровень: automated README evidence extraction
- Снимок: [sources/jadedbay__bevy_procedural_grass/README.md](sources/jadedbay__bevy_procedural_grass/README.md); SHA-256: `3657c3cc3cb70dab7ec47731faf878f5e8456093b9952da9e522c1ee88160cf5`
- lifecycle: строка 30: .add_systems(Startup, setup)
- memory: строка 81: - Compute Shaders, use compute shaders to generate grass instance data each frame to optimize memory usage.
- gpu: строка 71: - GPU Instancing

## beltegeuse/rustlight

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/beltegeuse/rustlight
- Категория: workload: graphics
- Описание: physically-based rendering engine implemented with Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/beltegeuse__rustlight/README.md](sources/beltegeuse__rustlight/README.md); SHA-256: `2034fa3f0fdf0228b91ad97e53da83222aa37754ca3a39bea50c586db9dc4423`
- statistics: строка 95: * Image-space control variate with uniform and variance-based weights [7]
- report: строка 34: <scene>    JSON file description

## carloskiki/pulldown-latex

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/carloskiki/pulldown-latex
- Категория: workload: graphics
- Описание: A pull parser for LaTeX parsing and mathml rendering.
- Уровень: automated README evidence extraction
- Снимок: [sources/carloskiki__pulldown-latex/README.md](sources/carloskiki__pulldown-latex/README.md); SHA-256: `49ba7b3c4bb024c952ba39849d2f793bd05320c24e9b4d54645bcfcfc00f8d0a`

## num3ric/sol-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/num3ric/sol-rs
- Категория: workload: graphics
- Описание: Vulkan rendering sandbox for raytracing
- Уровень: automated README evidence extraction
- Снимок: [sources/num3ric__sol-rs/README.md](sources/num3ric__sol-rs/README.md); SHA-256: `2da5c3ab3192c63ef323fdb26ad4fe598d0f557a4fc71c581d542fb82c9b3c79`
- gpu: строка 5: `sol-rs` is a small rendering toolkit for Vulkan, with a focus on real-time raytracing (which is not currently available via other APIs such as WebGPU). It hosts convenience wrappers but also exposes [ash](https://github.com/MaikKlein/ash) directly. Tested on

## Alex6357/alloy

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Alex6357/alloy
- Категория: workload: graphics
- Описание: A "No-DOM" GUI Runtime: SolidJS Logic driving Rust GPUI Rendering.
- Уровень: automated README evidence extraction
- Снимок: [sources/Alex6357__alloy/README.md](sources/Alex6357__alloy/README.md); SHA-256: `78ac63383f85eb4688fa3c9fe77222c30e508f5c881fe26bd9b9259c3c185c0b`
- report: строка 64: - **Binary Protocol:** Commands are encoded in a compact binary format (avg 75% smaller than JSON).
- correctness: строка 115: This is a **Proof of Concept** validating the _SolidJS → Binary → Rust_ architecture, realized through **AI-Accelerated Development ("Vibe Coding")**.

## ixlab/vidformer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ixlab/vidformer
- Категория: workload: graphics
- Описание: A drop-in declarative optimization framework for rendering video data visualizations.
- Уровень: automated README evidence extraction
- Снимок: [sources/ixlab__vidformer/README.md](sources/ixlab__vidformer/README.md); SHA-256: `62dd38cf53e62916348c1f883a5b62ef14e64c26bb5e206a9d74bb5df0c3d41c`

## jinleili/sdf-text-view

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/jinleili/sdf-text-view
- Категория: workload: graphics
- Описание: Real time SDF (signed distance fields) calculate and rendering based on WebGPU.
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## FreddyWordingham/antler

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/FreddyWordingham/antler
- Категория: workload: graphics
- Описание: Antler rendering engine
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## VisualGMQ/rs-cpurenderer

- Итог: unavailable — No README body; excluded from content-review count.
- Источник: https://github.com/VisualGMQ/rs-cpurenderer
- Категория: workload: graphics
- Описание: a cpu/gpu soft renderer in rust
- Уровень: metadata only
- README не получен; не засчитывается в анализ содержимого.

## mgth/Omniphony

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mgth/Omniphony
- Категория: workload: graphics
- Описание: Real-time spatial / object-based audio rendering engine (VBAP + binaural) for multichannel and 3D audio — speakers or headphones.
- Уровень: automated README evidence extraction
- Снимок: [sources/mgth__Omniphony/README.md](sources/mgth__Omniphony/README.md); SHA-256: `d66de6da7168851f8c3441de7c9e610c0a6470e3f3a22aa3d66a47f0376f0911`
- lifecycle: строка 68: the CLI and `liborender.so`, installable on its own for a player-less setup.

## meilisearch/meilisearch

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/meilisearch/meilisearch
- Категория: workload: storage
- Описание: A lightning-fast search engine API bringing AI-powered hybrid search to your sites and applications.
- Уровень: automated README evidence extraction
- Снимок: [sources/meilisearch__meilisearch/README.md](sources/meilisearch__meilisearch/README.md); SHA-256: `83929117b73b31f7233abb5237bfd04141dc278f6d6597c38ee3b65b1a3501c3`
- async: строка 125: - `external-crates/async-openai` and `external-crates/async-openai-macros` from <https://github.com/64bit/async-openai>

## qdrant/qdrant

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/qdrant/qdrant
- Категория: workload: storage
- Описание: Qdrant - High-performance, massive-scale Vector Database and Vector Search Engine for the next generation of AI. Also available in the cloud https://cloud.qdrant.io/
- Уровень: automated README evidence extraction
- Снимок: [sources/qdrant__qdrant/README.md](sources/qdrant__qdrant/README.md); SHA-256: `f1455d12252bf40235ad0128d8e98b2cd6d42583e441ac832b01f4daf08df2e4`
- gpu: строка 161: * **GPU Support** - for accelerated indexing, with support for NVIDIA and AMD GPUs.
- async: строка 81: [Qdrant Edge](https://qdrant.tech/documentation/edge/) is a lightweight version of Qdrant designed for edge devices and resource-constrained environments. Unlike Qdrant Server, which uses a client-server architecture, Qdrant Edge runs inside the application pr
- report: строка 123: You can also download the raw [OpenAPI definitions](https://github.com/qdrant/qdrant/blob/master/docs/redoc/master/openapi.json).
- isolation: строка 42: Qdrant provides a collection of ready-to-use [agent skills](https://github.com/qdrant/skills) that bring Qdrant's vector search capabilities directly into your AI coding assistant. Install these skills to empower your agent in making critical engineering decis

## surrealdb/surrealdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/surrealdb/surrealdb
- Категория: workload: storage
- Описание: A scalable, distributed, collaborative, document-graph database, for the realtime web
- Уровень: automated README evidence extraction
- Снимок: [sources/surrealdb__surrealdb/README.md](sources/surrealdb__surrealdb/README.md); SHA-256: `c7d37b9cb4cbf4c9b4d5b7001552662f95b4434a5e43858c073c4b9bd685d1b4`
- memory: строка 74: - **Reduces development time**: SurrealDB simplifies your database and API stack by removing the need for most server-side components, allowing you to build secure, performant apps faster and cheaper.
- report: строка 315: Store GeoJSON geographical data types, including points, lines and polygons.

## influxdata/influxdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/influxdata/influxdb
- Категория: workload: storage
- Описание: Scalable datastore for metrics, events, and real-time analytics
- Уровень: automated README evidence extraction
- Снимок: [sources/influxdata__influxdb/README.md](sources/influxdata__influxdb/README.md); SHA-256: `ed54ece09e4f6c299e947062c7286ed5d972176c056ec9eb6997810e9e1f73d9`
- report: строка 27: need to return quickly to support user experiences such as dashboards and interactive user interfaces.

## chroma-core/chroma

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/chroma-core/chroma
- Категория: workload: storage
- Описание: Search infrastructure for AI
- Уровень: automated README evidence extraction
- Снимок: [sources/chroma-core__chroma/README.md](sources/chroma-core__chroma/README.md); SHA-256: `a71f91e5fadfb03772be1f76b0bab7116eea1bfc6cc9bc4fb190ce95dd275264`
- lifecycle: строка 41: # setup Chroma in-memory, for easy prototyping. Can add persistence easily!

## clockworklabs/SpacetimeDB

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/clockworklabs/SpacetimeDB
- Категория: workload: storage
- Описание: Development at the speed of light
- Уровень: automated README evidence extraction
- Снимок: [sources/clockworklabs__SpacetimeDB/README.md](sources/clockworklabs__SpacetimeDB/README.md); SHA-256: `5347baf424ff8f6f049d3d56301ae4ac0f3015a93f96fd2058acef35b9147073`
- async: строка 85: SpacetimeDB is optimized for maximum speed and minimum latency. SpacetimeDB provides all the ACID guarantees of a traditional RDBMS, with all the speed of an optimized web server. All application state is held in memory for fast access, while a commit log on d

## tursodatabase/turso

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tursodatabase/turso
- Категория: workload: storage
- Описание: A SQL database in Rust: SQLite-compatible, now also speaking Postgres (experimental). The LLVM of databases.
- Уровень: automated README evidence extraction
- Снимок: [sources/tursodatabase__turso/README.md](sources/tursodatabase__turso/README.md); SHA-256: `5062cf5f8713144d245b51d1a4b964438d142dbe4a34c1d91ccff486a9fea134`
- lifecycle: строка 307: #### Quick Setup
- async: строка 45: * **`BEGIN CONCURRENT`** for improved write throughput using multi-version concurrency control (MVCC).
- report: строка 268: ```json

## neondatabase/neon

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/neondatabase/neon
- Категория: workload: storage
- Описание: Neon: Serverless Postgres. We separated storage and compute to offer autoscaling, code-like database branching, and scale to zero.
- Уровень: automated README evidence extraction
- Снимок: [sources/neondatabase__neon/README.md](sources/neondatabase__neon/README.md); SHA-256: `5ec2ed5074f09777ed9f5f8020a581d3fba081d0152fcec5088e8ea29cb3e810`
- lifecycle: строка 247: If you encounter errors during setting up the initial tenant, it's best to stop everything (`cargo neon stop`) and remove the `.neon` directory. Then fix the problems, and start the setup again.

## valeriansaliou/sonic

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/valeriansaliou/sonic
- Категория: workload: storage
- Описание: 🦔 Fast, lightweight & schema-less search backend. An alternative to Elasticsearch that runs on a few MBs of RAM.
- Уровень: automated README evidence extraction
- Снимок: [sources/valeriansaliou__sonic/README.md](sources/valeriansaliou__sonic/README.md); SHA-256: `3546fbb7673e85842f742bf68501b2de962864c213c7c7d0d177e279768e969f`
- memory: строка 113: Note that the following optional features can be enabled upon building Sonic: `allocator-jemalloc`, `tokenizer-chinese` and `tokenizer-japanese` (some might be already enabled by default).
- report: строка 216: * **[jsonic](https://github.com/alohaking/jsonic)** by [@alohaking](https://github.com/alohaking)
- correctness: строка 183: Sonic distributes official Sonic integration libraries for your programming language (official means that those libraries have been reviewed and validated by a core maintainer):

## t8y2/dbx

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/t8y2/dbx
- Категория: workload: storage
- Описание: 20 MB lightweight cross-platform database client for 90+ databases, including MySQL, PostgreSQL, SQLite, Redis, MongoDB, DuckDB, SQL Server, and Dameng. Built-in AI, MCP Server, CLI, desktop and Docker. | 轻量级跨平台数据库管理工具，支持 MySQL、PostgreSQL、SQLite、Redis、MongoDB、达梦等 90+ 数据库，提供桌面端、Docker、CLI、内置 AI 助手和 MCP Server。
- Уровень: automated README evidence extraction
- Снимок: [sources/t8y2__dbx/README.md](sources/t8y2__dbx/README.md); SHA-256: `bb03b44d2e7ff8d7fcc036640c3600d78645ad8bae08744903b378a2c3cdbd90`
- report: строка 9: <a href="https://github.com/t8y2/dbx/releases"><img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fshieldcn.dev%2Fgithub%2Fdownloads%2Ft8y2%2Fdbx%2Fshields.json&amp;style=for-the-badge" /></a>
- correctness: строка 461: For clean, reproducible local database instances, use the versioned Docker Compose recipes under [`deploy/database/`](deploy/database/README.md):

## tikv/tikv

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tikv/tikv
- Категория: workload: storage
- Описание: Distributed transactional key-value database, originally created to complement TiDB
- Уровень: automated README evidence extraction
- Снимок: [sources/tikv__tikv/README.md](sources/tikv__tikv/README.md); SHA-256: `58a21aba3c4496806139219d5f61ad2ad1ae44ecfec555812c9d1c371489a450`
- lifecycle: строка 132: The easiest way to run a complete TiKV cluster (3 PD + 3 TiKV nodes) for development and testing is using Docker Compose. The setup uses pre-built TiKV and PD nightly images from Docker Hub, so no building is required.
- report: строка 74: When a node starts, the metadata of the Node, Store and Region are recorded into PD. The status of each Region and Store is reported to PD regularly.
- isolation: строка 25: With the implementation of the Raft consensus algorithm in Rust and consensus state stored in RocksDB, TiKV guarantees data consistency. [Placement Driver (PD)](https://github.com/pingcap/pd/), which is introduced to implement auto-sharding, enables automatic

## pingcap/talent-plan

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pingcap/talent-plan
- Категория: workload: storage
- Описание: open source training courses about distributed database and distributed systems
- Уровень: automated README evidence extraction
- Снимок: [sources/pingcap__talent-plan/README.md](sources/pingcap__talent-plan/README.md); SHA-256: `7b8b76b68c3f4d5ed04185b9d8cd54568386b383520d7b7895abfedde2597ed6`
- report: строка 58: We love our community and take great care to ensure it is fun, safe and rewarding. Please review our [Code of Conduct](/CODE_OF_CONDUCT.md) for community expectations and guidelines for reporting concerns.

## SeaQL/sea-orm

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SeaQL/sea-orm
- Категория: workload: storage
- Описание: 🐚 A powerful relational ORM for Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/SeaQL__sea-orm/README.md](sources/SeaQL__sea-orm/README.md); SHA-256: `68f8ea90fb70e89b96214aa1868545d97ddc84856403ffecc45795d9deb6cabc`
- async: строка 197: [`sea-orm-sync`](https://crates.io/crates/sea-orm-sync) provides the full SeaORM API without requiring an async runtime, making it ideal for lightweight CLI programs with SQLite.
- report: строка 47: + [jsonrpsee Example](https://github.com/SeaQL/sea-orm/tree/master/examples/jsonrpsee_example)

## databendlabs/databend

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/databendlabs/databend
- Категория: workload: storage
- Описание: Data Agent Ready Warehouse : One for  Analytics, Search, AI, Python Sandbox.  — rebuilt from scratch. Unified architecture on your S3.
- Уровень: automated README evidence extraction
- Снимок: [sources/databendlabs__databend/README.md](sources/databendlabs__databend/README.md); SHA-256: `c4cc36d4edaebcc66cc274ba2a0ba7710ffa341f09317e44e0e4acf6e689832c`
- correctness: строка 70: - **Control Plane**: Resource scheduling, permission validation, sandbox lifecycle management

## risingwavelabs/risingwave

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/risingwavelabs/risingwave
- Категория: workload: storage
- Описание: Event streaming platform for agentic AI. Continuously ingest, transform, and serve event streams in real time, at scale.
- Уровень: automated README evidence extraction
- Снимок: [sources/risingwavelabs__risingwave/README.md](sources/risingwavelabs__risingwave/README.md); SHA-256: `58b9d2e50c3a29bd9ea8188ea0893a0bd0c3c561a571440f0e79273a2904e6d4`
- memory: строка 117: Internal state, tables, and materialized views are stored in object storage (S3 or equivalent), which is roughly 100x cheaper than RAM. This enables elastic scaling without data rebalancing and failure recovery in seconds. For latency-sensitive workloads, [ela
- async: строка 48: RisingWave is an event streaming platform for agentic AI. It continuously ingests data from databases, event streams, and webhooks, processes it incrementally, and serves fresh results at low latency, replacing the traditional event streaming stack (e.g., Debe
- report: строка 107: - **Live dashboards**: materialized views updated incrementally, no scheduled refreshes

## spacejam/sled

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/spacejam/sled
- Категория: workload: storage
- Описание: the champagne of beta embedded databases
- Уровень: automated README evidence extraction
- Снимок: [sources/spacejam__sled/README.md](sources/spacejam__sled/README.md); SHA-256: `bdb4a0c55bd34f4ea71643fbc474d052ac01df32c3f875c3a0e49859c9521890`
- async: строка 130: # interaction with async

## warp-tech/warpgate

- Итог: excluded — Access gateway; no relevant benchmark design evidence
- Источник: https://github.com/warp-tech/warpgate
- Категория: workload: storage
- Описание: Fully transparent SSH, HTTPS, Kubernetes, database and RDP/VNC bastion/PAM that doesn't need additional client-side software
- Уровень: automated README evidence extraction
- Снимок: [sources/warp-tech__warpgate/README.md](sources/warp-tech__warpgate/README.md); SHA-256: `62d99803bd8215ce5b858cd709141c8a14534b2fdec9d04088b3784858d1c088`
- statistics: строка 144: * Bootstrap
- lifecycle: строка 73: | ✅ **Built-in brute-force protection** | 🟡 Requires fail2ban setup | 🟡 Depends on the provider | ✅ **Built-in brute-force protection** |
- memory: строка 193: <td align="center" valign="top" width="14.28%"><a href="https://github.com/LarsSven"><img src="https://avatars.githubusercontent.com/u/60571459?v=4?s=100" width="100px;" alt="Lars"/><br /><sub><b>Lars</b></sub></a><br /><a href="https://github.com/warp-tech/wa
- report: строка 93: ## Reporting security issues

## erikgrinaker/toydb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/erikgrinaker/toydb
- Категория: workload: storage
- Описание: Distributed SQL database in Rust, written as an educational project
- Уровень: automated README evidence extraction
- Снимок: [sources/erikgrinaker__toydb/README.md](sources/erikgrinaker__toydb/README.md); SHA-256: `ccec66a8427847d0c1d1c0d2ae54394d13c29dd17ed8d38c85e839b87da65d86`
- report: строка 178: extension can be used to debug toyDB, with the debug configuration under `.vscode/launch.json`.
- isolation: строка 7: * [ACID transactions][txn] with MVCC-based snapshot isolation.

## GreptimeTeam/greptimedb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/GreptimeTeam/greptimedb
- Категория: workload: storage
- Описание: The open-source observability database. One columnar engine for metrics, logs, and traces, on object storage.
- Уровень: automated README evidence extraction
- Снимок: [sources/GreptimeTeam__greptimedb/README.md](sources/GreptimeTeam__greptimedb/README.md); SHA-256: `cdac09f91d5c8f07afa529f533e94e537118d55916a47555272e7796e2db3af0`
- comparison: строка 206: make sqlness-test   # SQL regression tests
- report: строка 127: * [GreptimeDB tops JSONBench's billion-record cold run test](https://greptime.com/blogs/2025-03-18-jsonbench-greptimedb-performance)
- isolation: строка 135: Read replicas, workload isolation, and automated repartitioning are **GreptimeDB Enterprise** features, along with enterprise security and governance. The [Enterprise overview](https://docs.greptime.com/enterprise/overview/) has the current list, and [pricing]

## MaterializeInc/materialize

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/MaterializeInc/materialize
- Категория: workload: storage
- Описание: The live data layer for apps and AI agents. Create up-to-the-second views into your business, just using SQL
- Уровень: automated README evidence extraction
- Снимок: [sources/MaterializeInc__materialize/README.md](sources/MaterializeInc__materialize/README.md); SHA-256: `4353bee7c9a2c5378d12336bffb6254854571e423fe1b0460f9e4737a37baba1`
- async: строка 29: Materialize focuses on providing correct and [consistent](https://materialize.com/docs/overview/isolation-level/) answers with minimal latency, and does not ask you to accept either approximate answers or eventual consistency. This guarantee holds even when jo
- report: строка 9: Use Materialize to do things like deliver fresh context for AI/RAG pipelines, power operational dashboards, and create more dynamic customer experiences without building time-consuming custom data pipelines.
- correctness: строка 14: - Query Offload (CQRS) - Scale complex read queries more efficiently than a read replica, and without the headaches of cache invalidation.
- isolation: строка 29: Materialize focuses on providing correct and [consistent](https://materialize.com/docs/overview/isolation-level/) answers with minimal latency, and does not ask you to accept either approximate answers or eventual consistency. This guarantee holds even when jo

## FalkorDB/FalkorDB

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/FalkorDB/FalkorDB
- Категория: workload: storage
- Описание: A super fast Graph Database uses GraphBLAS under the hood for its sparse adjacency matrix graph representation. Our goal is to provide the best Knowledge Graph for LLM (GraphRAG).
- Уровень: automated README evidence extraction
- Снимок: [sources/FalkorDB__FalkorDB/README.md](sources/FalkorDB__FalkorDB/README.md); SHA-256: `5fa7394af26aaa7bda2c535da0e79697953a6cfb6233c42caf15e58ca944c1b8`
- lifecycle: строка 306: ### Manual Setup
- async: строка 39: Our goal is to build a high-performance Knowledge Graph tailored for Large Language Models (LLMs), prioritizing exceptionally low latency to ensure fast and efficient information delivery through our Graph Database.

## HelixDB/helix-db

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/HelixDB/helix-db
- Категория: workload: storage
- Описание: HelixDB is an OLTP graph database with native vector and full-text search built in Rust on Object Storage.
- Уровень: automated README evidence extraction
- Снимок: [sources/HelixDB__helix-db/README.md](sources/HelixDB__helix-db/README.md); SHA-256: `9fb62049d446cea652e04d7e9dc989cf91337f6515d094d7be7b7a043333701a`
- statistics: строка 56: `helix chef` is an interactive, one-shot bootstrapper. It installs the HelixDB query skills and docs MCP, scaffolds a project, starts a local instance, seeds some example data, and writes a `HELIX_CHEF_PROMPT.md`. It detects supported agents in this order: Cla
- lifecycle: строка 64: ### 3. Manual local setup
- async: строка 119: async fn main() -> Result<(), Box<dyn std::error::Error>> {
- report: строка 72: Queries are authored with the Rust, TypeScript, Go, or Python DSL and sent straight to a running instance through `POST /v2/query` — no build or deploy step. The SDKs produce the same JSON AST. The examples below talk to a local instance on `http://localhost:6

## trailbaseio/trailbase

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/trailbaseio/trailbase
- Категория: workload: storage
- Описание: An open, sub-millisecond, single-executable Firebase alternative with type-safe APIs, built-in WebAssembly runtime, realtime subscriptions, auth, MCP and admin UI built on Rust, SQLite (PG) & Wasmtime.
- Уровень: automated README evidence extraction
- Снимок: [sources/trailbaseio__trailbase/README.md](sources/trailbaseio__trailbase/README.md); SHA-256: `4a86d28afa8c46252c4fcd20af90fd6bb7409bb225fd4de82af6565e04c49f91`
- statistics: строка 129: On first start, a `./traildepot` folder will be bootstrapped, an admin user
- lifecycle: строка 182: We're not sure yet what the best setup or exact license is for compatibility
- report: строка 132: in your browser and use the credentials to log into the admin dashboard.

## pgdogdev/pgdog

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/pgdogdev/pgdog
- Категория: workload: storage
- Описание: PostgreSQL connection pooler, load balancer and database sharder.
- Уровень: automated README evidence extraction
- Снимок: [sources/pgdogdev__pgdog/README.md](sources/pgdogdev__pgdog/README.md); SHA-256: `731d61f13026b1b86ca1556b779198aa396ea3f807f86a4e019f65a75bc5ba0b`
- statistics: строка 464: | Aggregates            | Partial   | `count`, `min`, `max`, `stddev`, `variance`, `sum`, `avg` are supported.                     |
- comparison: строка 679: PgDog is heavily optimized for performance. We use Rust, [Tokio](https://tokio.rs/), [bytes crate](https://docs.rs/bytes/latest/bytes/) to avoid unnecessary memory allocations, and profile for performance regressions on a regular basis.
- memory: строка 679: PgDog is heavily optimized for performance. We use Rust, [Tokio](https://tokio.rs/), [bytes crate](https://docs.rs/bytes/latest/bytes/) to avoid unnecessary memory allocations, and profile for performance regressions on a regular basis.
- report: строка 476: PgDog has a text, CSV & binary parser and can split rows sent via `COPY` command between all shards automatically. This allows clients to ingest data into sharded PostgreSQL without preprocessing

## achristmascarl/rainfrog

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/achristmascarl/rainfrog
- Категория: workload: storage
- Описание: 🐸 a database tool for the terminal
- Уровень: automated README evidence extraction
- Снимок: [sources/achristmascarl__rainfrog/README.md](sources/achristmascarl__rainfrog/README.md); SHA-256: `4af9deba2bc4527a3001b23ca4a894c9055940b34a00bce049da36d639b42365`
- report: строка 538: | `P`                       | export results to csv          |
- correctness: строка 470: | `F7`              | Bypass parser to execute query (cannot rollback, no validation) |

## readysettech/readyset

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/readysettech/readyset
- Категория: workload: storage
- Описание: Readyset is a MySQL and Postgres wire-compatible caching layer that sits in front of existing databases to speed up queries and horizontally scale read throughput. Under the hood, ReadySet caches the results of cached select statements and incrementally updates these results over time as the underlying data changes.
- Уровень: automated README evidence extraction
- Снимок: [sources/readysettech__readyset/README.md](sources/readysettech__readyset/README.md); SHA-256: `cadd335495f46e19fdaec7526fbd8685c2c134814b546cbc449a13c8b80cb20c`
- report: строка 45: * **[GitHub](https://github.com/readysettech/readyset/issues/new/choose)**: For bug reports and feature requests.
- correctness: строка 5: Readyset is a transparent database cache for Postgres & MySQL that gives you the performance and scalability of an in-memory key-value store without requiring that you rewrite your app or manually handle cache invalidation. Readyset sits between your applicati

## malisper/pgrust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/malisper/pgrust
- Категория: workload: storage
- Описание: Postgres rewritten in Rust, now faster than Postgres and Clickhouse
- Уровень: automated README evidence extraction
- Снимок: [sources/malisper__pgrust/README.md](sources/malisper__pgrust/README.md); SHA-256: `c1d36e9a431de2a730e150e5eb8582a2e9c70fd39f11eef9e7bc36c2e2dcffa3`
- comparison: строка 9: <img alt="Regression suite: 100%" src="https://img.shields.io/badge/regression_suite-46%2C066%2F46%2C066-brightgreen">
- lifecycle: строка 389: Please do still open an issue if something breaks, if setup is confusing, or
- async: строка 41: - A thread based concurrency model
- report: строка 88: Two honest caveats. We had previously reported that pgrust was over 50%
- correctness: строка 162: # Download pgrust and verify the checksum. Download with curl: curl does not

## cberner/redb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cberner/redb
- Категория: workload: storage
- Описание: An embedded key-value database in pure Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/cberner__redb/README.md](sources/cberner__redb/README.md); SHA-256: `8452936cd123eb1b4e07a8af517fe7fb6ef1526570ebe82cdd17236adc5f5273`
- lifecycle: строка 61: Toolchain setup, dependency vendoring, and `cargo-deny` checks may use network access without
- async: строка 45: * MVCC support for concurrent readers & writer, without blocking

## typedb/typedb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/typedb/typedb
- Категория: workload: storage
- Описание: TypeDB: Built for systems, not records
- Уровень: automated README evidence extraction
- Снимок: [sources/typedb__typedb/README.md](sources/typedb__typedb/README.md); SHA-256: `a353b79e3ffde3c0f6d7bb0151998f3b1a405929085f3abdacca725d289848e2`

## Qovery/Replibyte

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Qovery/Replibyte
- Категория: workload: storage
- Описание: Seed your development database with real data ⚡️
- Уровень: automated README evidence extraction
- Снимок: [sources/Qovery__Replibyte/README.md](sources/Qovery__Replibyte/README.md); SHA-256: `edc479ddd1ed72411f5c69fd4e63ee6096735108e0769875b70ba4e894d1c882`
- lifecycle: строка 79: 2. Initial setup:

## skyzh/mini-lsm

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/skyzh/mini-lsm
- Категория: workload: storage
- Описание: learn database internals by building a storage engine in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/skyzh__mini-lsm/README.md](sources/skyzh__mini-lsm/README.md); SHA-256: `b66e01bbecd90dd5e8f72ec2b6f541c1db4f9f74daa4636893e6b7c43a30bad5`
- async: строка 11: Week 1 produces a working storage engine. Weeks 2 and 3 add production-inspired compaction, durability, concurrency control, and multi-version transactions.
- correctness: строка 23: * snapshots, MVCC garbage collection, optimistic concurrency control, and serializable validation for tracked keys.
- isolation: строка 34: You need basic Rust, but you do not need prior knowledge of LSM trees, compaction, MVCC, or transaction isolation. The course is a good fit if you have used systems such as PostgreSQL, MySQL, Redis, or RocksDB and want to understand what happens below their AP

## cozodb/cozo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cozodb/cozo
- Категория: workload: storage
- Описание: A transactional, relational-graph-vector database that uses Datalog for query. The hippocampus for AI!
- Уровень: automated README evidence extraction
- Снимок: [sources/cozodb__cozo/README.md](sources/cozodb__cozo/README.md); SHA-256: `0f81224513378f145760f869120f61cd1abe248fac3a541b1009f136dded5322`
- lifecycle: строка 84: > generally require no setup and can be used in a much wider range of environments.
- memory: строка 142: read/write/update transactional queries, and more than 250K QPS for read-only queries, with database peak memory usage
- async: строка 54: * As with all mutations in CozoDB, the index is protected from corruption in the face of concurrent writes by using
- report: строка 33: search, Json value support and more! See [here](https://docs.cozodb.org/en/latest/releases/v0.7.html) for more details.

## ekzhang/rustpad

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ekzhang/rustpad
- Категория: workload: storage
- Описание: Efficient and minimal collaborative code editor, self-hosted, no database required
- Уровень: automated README evidence extraction
- Снимок: [sources/ekzhang__rustpad/README.md](sources/ekzhang__rustpad/README.md); SHA-256: `d2996cac11e894bd7a1686273428d733493738427a2a12ba7a385aaf73be123d`
- lifecycle: строка 34: ## Development setup

## vlcn-io/cr-sqlite

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vlcn-io/cr-sqlite
- Категория: workload: storage
- Описание: Convergent, Replicated SQLite. Multi-writer and CRDT support for SQLite
- Уровень: automated README evidence extraction
- Снимок: [sources/vlcn-io__cr-sqlite/README.md](sources/vlcn-io__cr-sqlite/README.md); SHA-256: `83c634554ca3c462dce26601c33b56a9fced2428fb273538cd2497d64cbb87fb`
- lifecycle: строка 21: - Basic setup & sync via an [Observable Notebook](https://observablehq.com/@tantaman/cr-sqlite-basic-setup)
- correctness: строка 290: cd ../py/correctness

## frectonz/sql-studio

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/frectonz/sql-studio
- Категория: workload: storage
- Описание: SQL Database Explorer [SQLite, libSQL, PostgreSQL, MySQL/MariaDB, ClickHouse, DuckDB, Microsoft SQL Server]
- Уровень: automated README evidence extraction
- Снимок: [sources/frectonz__sql-studio/README.md](sources/frectonz__sql-studio/README.md); SHA-256: `5f1c0a68ffe7b8c362a154c7f6cc4edbcfa66f42238e264597313418c6fa08ec`
- report: строка 5: Single binary, single command SQL database explorer. SQL studio supports *SQLite*, *libSQL*, *PostgreSQL*, *MySQL*, *DuckDB*, *ClickHouse*, *Microsoft SQL Server*, *Parquet* and *CSV*.

## AmrDeveloper/GQL

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/AmrDeveloper/GQL
- Категория: workload: storage
- Описание: GitQL is a extensible SQL-like query language and SDK to perform queries on various data sources such .git files with supports of most of SQL features such as grouping, ordering and aggregation and window functions and allow customization like user-defined types and functions
- Уровень: automated README evidence extraction
- Снимок: [sources/AmrDeveloper__GQL/README.md](sources/AmrDeveloper__GQL/README.md); SHA-256: `ff69d123850e37f2673bd4a53655060057880eb26ac79709560c6c6ea92fbd27`
- lifecycle: строка 81: - [Install or Build](https://amrdeveloper.github.io/GQL/setup)

## slatedb/slatedb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/slatedb/slatedb
- Категория: workload: storage
- Описание: A cloud native embedded storage engine built on object storage.
- Уровень: automated README evidence extraction
- Снимок: [sources/slatedb__slatedb/README.md](sources/slatedb__slatedb/README.md); SHA-256: `3ce87ca11b4f336a5c87339c87fbedbf063a4b71c0da8810d9d17b302229d315`
- lifecycle: строка 44: // Setup
- async: строка 15: [SlateDB](https://slatedb.io) is an embedded storage engine built as a [log-structured merge-tree](https://en.wikipedia.org/wiki/Log-structured_merge-tree). Unlike traditional LSM-tree storage engines, SlateDB writes data to object storage (S3, GCS, ABS, MinIO

## TaKO8Ki/gobang

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/TaKO8Ki/gobang
- Категория: workload: storage
- Описание: A cross-platform TUI database management tool written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/TaKO8Ki__gobang/README.md](sources/TaKO8Ki__gobang/README.md); SHA-256: `bbdb6458843e57c792c1bad753eaff2fa252c10ef4932f2d34c24fa4df2333aa`
- report: строка 44: If you're a Windows Scoop user, then you can install gobang from the [official bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/gobang.json):

## gluesql/gluesql

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/gluesql/gluesql
- Категория: workload: storage
- Описание: GlueSQL is quite sticky. It sticks to anything.
- Уровень: automated README evidence extraction
- Снимок: [sources/gluesql__gluesql/README.md](sources/gluesql__gluesql/README.md); SHA-256: `db10ed74610be7c660b98e356be4fbf12265b86b1fcbacd3bbb5f8f715aa4b5c`
- async: строка 102: Shared Memory Storage is a storage option designed to provide more comfortable usage of Memory Storage in concurrent environments. It wraps the Memory Storage with a read-write lock and an atomic reference count, allowing you to clone the storage instance and
- report: строка 94: GlueSQL provides a variety of reference storages out of the box, including simple in-memory storage, key-value databases, and log file-based storage like JSON & JSONL. These reference storages are readily available for use and can be easily adapted to a variet
- correctness: строка 159: GlueSQL is a database project that is simpler than you might think. You only need to know three common Rust project commands: `cargo fmt`, `cargo clippy`, and `cargo test`. Don't hesitate to make pull requests and change the code as you see fit. We have set up

## spiceai/spiceai

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/spiceai/spiceai
- Категория: workload: storage
- Описание: Add a real-time analytics node to your operational database. Spice is a portable, accelerated SQL query, search, and LLM-inference engine in Rust for data-grounded AI apps and agents.
- Уровень: automated README evidence extraction
- Снимок: [sources/spiceai__spiceai/README.md](sources/spiceai__spiceai/README.md); SHA-256: `2b846b340790e507e95818930c16639d2b865ff7206d52df18c86385f249a997`
- statistics: строка 84: - **PostgreSQL (WAL), MySQL (binlog), and MongoDB (change streams)** — native replication with auto-managed replication state (slots, binlog positions, resume tokens) and bootstrapped initial snapshots. **No Debezium or Kafka required.**
- gpu: строка 62: 3. **OpenAI-Compatible APIs**: Hosted LLM gateway (OpenAI, Anthropic, xAI, Bedrock) and local model serving (CUDA/Metal accelerated). Includes the OpenAI Responses API, web search, and tool calls.
- async: строка 33: > 🆕 **New in Spice 2.0 — add a real-time analytics node to your operational database.** Point Spice at **PostgreSQL, MySQL, or MongoDB** and it maintains a sandboxed, analytics-ready replica with high-throughput **CDC replication** — **sub-second queries, ~2-s
- report: строка 86: - **Debezium** — Kafka consumer (`from: debezium:…`) or **push ingest without Kafka** (`from: cdc:…` + `POST /v1/datasets/{name}/cdc`, JSON/Avro).
- isolation: строка 120: Spin up one Spice runtime per tenant or agent — each with its own sandboxed datasets, accelerators, secrets, and policies. Or share a runtime with config-level tenant isolation. Or do both with a hybrid model. The lightweight **~140MB** runtime makes "one Spic

## orbitinghail/sqlsync

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/orbitinghail/sqlsync
- Категория: workload: storage
- Описание: SQLSync is a collaborative offline-first wrapper around SQLite. It is designed to synchronize web application state between users, devices, and the edge.
- Уровень: automated README evidence extraction
- Снимок: [sources/orbitinghail__sqlsync/README.md](sources/orbitinghail__sqlsync/README.md); SHA-256: `e5b4e20b71fb8325280bee636f97c4dcde67501333fe4c066e5c3a8555dd27be`

## obi1kenobi/trustfall

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/obi1kenobi/trustfall
- Категория: workload: storage
- Описание: A query engine for any combination of data sources. Query your files and APIs as if they were databases!
- Уровень: automated README evidence extraction
- Снимок: [sources/obi1kenobi__trustfall/README.md](sources/obi1kenobi__trustfall/README.md); SHA-256: `0a2d47f71f99c8c54fd8d36c94d9e664b33fd87d0b57d727cc9d612cc5f46d43`
- memory: строка 76: - [RSS/Atom feeds](./trustfall/examples/feeds/), showing how to query structured data
- report: строка 14: - the rustdoc JSON of top Rust crates: https://play.predr.ag/rustdoc

## apache/horaedb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/apache/horaedb
- Категория: workload: storage
- Описание: Apache HoraeDB (incubating) is a high-performance, distributed, cloud native time-series database.
- Уровень: automated README evidence extraction
- Снимок: [sources/apache__horaedb/README.md](sources/apache__horaedb/README.md); SHA-256: `ad3f59f402ea2559fa8d27001ce888ca510ddeada6c99c9cba3b132d8a4e66b3`

## skytable/skytable

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/skytable/skytable
- Категория: workload: storage
- Описание: Skytable is a modern scalable NoSQL database with BlueQL, designed for performance, scalability and flexibility. Skytable gives you spaces, models, data types, complex collections and more to build powerful experiences
- Уровень: automated README evidence extraction
- Снимок: [sources/skytable__skytable/README.md](sources/skytable__skytable/README.md); SHA-256: `108c1293a78f0e73f6d971e2fecfb780ea5d77c9a19666b472a126c73f1c0b75`
- lifecycle: строка 43: 4. Your setup is now complete.

## sfu-db/connector-x

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sfu-db/connector-x
- Категория: workload: storage
- Описание: Fastest library to load data from DB to DataFrames in Rust and Python
- Уровень: automated README evidence extraction
- Снимок: [sources/sfu-db__connector-x/README.md](sources/sfu-db__connector-x/README.md); SHA-256: `814387d68f799159c2226d74f9f1aaeeff94b7141d82f82c647276ab27afa062`
- lifecycle: строка 41: By default, we pushdown all joins from the same data source. More details for setup and configuration can be found [here](https://github.com/sfu-db/connector-x/blob/main/Federation.md).
- memory: строка 84: Finally, ConnectorX will use the schema info as well as the count info to allocate memory and download data by executing the queries normally.

## sqlpage/SQLPage

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sqlpage/SQLPage
- Категория: workload: storage
- Описание: Fast SQL-only data application builder. Automatically build a UI on top of SQL queries.
- Уровень: automated README evidence extraction
- Снимок: [sources/sqlpage__SQLPage/README.md](sources/sqlpage__SQLPage/README.md); SHA-256: `7c157f86b1a313095a8f8f7be043ec1dc71fa009303de24f75c2b60aaf4b7f91`
- lifecycle: строка 187: ### ODBC Setup
- memory: строка 176: a cheaper ARM cloud instance, using the docker image is the easiest way to do it.
- report: строка 168: - And place your website in a folder named `source` and your `sqlpage.json` in a folder named `configuration`.
- correctness: строка 73: 'Create new user' as validate;

## indradb/indradb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/indradb/indradb
- Категория: workload: storage
- Описание: A graph database written in rust
- Уровень: automated README evidence extraction
- Снимок: [sources/indradb__indradb/README.md](sources/indradb__indradb/README.md); SHA-256: `53434163fffc20007cf11328d6bf6a62b204e56bb6d7da0da7887d39864dc21c`
- report: строка 22: * JSON-based properties tied to vertices and edges.

## hydra-db/hydradb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/hydra-db/hydradb
- Категория: workload: storage
- Описание: HydraDB - fast graph database on object storage
- Уровень: automated README evidence extraction
- Снимок: [sources/hydra-db__hydradb/README.md](sources/hydra-db__hydradb/README.md); SHA-256: `5fea29178b8c0e8672715c4409875b6229e250c38d0025df94b5b6ffdfc54b2b`
- comparison: строка 493: | [Correctness casebook](docs/bugs-found-fixed/README.md) | Reproduced storage and query invariants with regression evidence |
- async: строка 261: # graph-node's async query futures exceed the default thread stack. Without
- report: строка 35: typed JSON and streaming NDJSON HTTP API.
- correctness: строка 473: examples/           smoke, import, benchmark, and correctness programs

## fjall-rs/fjall

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fjall-rs/fjall
- Категория: workload: storage
- Описание: 🗻 Log-structured, embeddable key-value storage engine written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/fjall-rs__fjall/README.md](sources/fjall-rs__fjall/README.md); SHA-256: `e1ca0a77dcbfba9a56ce9b07a8a4f8e78c533afb2941cedd07eb1d60fad648d8`
- memory: строка 129: ## Memory usage
- async: строка 120: ## Multithreading, Async and Multiprocess
- report: строка 201: - [Open an issue](https://github.com/fjall-rs/fjall/issues/new) (bug report, weirdness)
- isolation: строка 149: However this isolation level can not do read-modify-write operations without the chance of lost updates.

## tensorchord/pgvecto.rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tensorchord/pgvecto.rs
- Категория: workload: storage
- Описание: Scalable, Low-latency and Hybrid-enabled Vector Search in Postgres. Revolutionize Vector Search, not Database.
- Уровень: automated README evidence extraction
- Снимок: [sources/tensorchord__pgvecto.rs/README.md](sources/tensorchord__pgvecto.rs/README.md); SHA-256: `b1f2efcf2c50f50e00b7202435a97711a4b770e8a725309b63a9812cda12e82a`
- memory: строка 129: `vecf16` type is the same with `vector` in anything but the scalar type. It stores 16-bit floating point numbers. If you want to reduce the memory usage to get better performance, you can try to replace `vector` type with `vecf16` type.
- report: строка 164: <td align="center" valign="top" width="14.28%"><a href="https://blog.mapotofu.org/"><img src="https://avatars.githubusercontent.com/u/12974685?v=4?s=70" width="70px;" alt="Keming"/><br /><sub><b>Keming</b></sub></a><br /><a href="https://github.com/tensorchord

## rust-rocksdb/rust-rocksdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rust-rocksdb/rust-rocksdb
- Категория: workload: storage
- Описание: rust wrapper for rocksdb
- Уровень: automated README evidence extraction
- Снимок: [sources/rust-rocksdb__rust-rocksdb/README.md](sources/rust-rocksdb__rust-rocksdb/README.md); SHA-256: `c2e6e01f8dfc7f6a14f01c3bb4e865999db91aadb8c9a321e7fa01cb9c882664`
- async: строка 51: from multiple threads concurrently, but this crate doesn't allow it by default

## feldera/feldera

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/feldera/feldera
- Категория: workload: storage
- Описание: The Feldera Incremental Computation Engine
- Уровень: automated README evidence extraction
- Снимок: [sources/feldera__feldera/README.md](sources/feldera__feldera/README.md); SHA-256: `421983b41c41010610e1209a5440272e6b5b43e6571fcec3fdd813c4e19d65af`
- async: строка 182: <img alt="Nexmark throughput in events per second by query, comparing Feldera in-memory, Feldera with storage, and Flink" src="https://cdn.sanity.io/images/nlte859i/production/c80a9d592fb6f6e4cf2c7a665add24da65998123-1740x493.png?auto=format&fit=max&w=1740" wi
- report: строка 60: 2. **Fast out-of-the-box performance.**  Feldera users have reported getting complex use cases

## toeverything/OctoBase

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/toeverything/OctoBase
- Категория: workload: storage
- Описание: 🐙 OctoBase is the open-source database behind AFFiNE, local-first, yet collaborative. A light-weight, scalable, data engine written in Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/toeverything__OctoBase/README.md](sources/toeverything__OctoBase/README.md); SHA-256: `7e8949753394e164e0d289a801340f70838aad486b651795b0d6f6626638ad28`

## tranxuanthang/lrclib

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tranxuanthang/lrclib
- Категория: workload: storage
- Описание: LRCLIB server written in Rust with Axum and SQLite3 database
- Уровень: automated README evidence extraction
- Снимок: [sources/tranxuanthang__lrclib/README.md](sources/tranxuanthang__lrclib/README.md); SHA-256: `f2ec3eaeb37c9edf28e006f9a70813575be32e6dd5e252029c6e8bb97c6f8c04`
- lifecycle: строка 9: ## Setup
- memory: строка 32: Use these variables to optimize memory usage in low traffic and/or low memory situations. The default values are generously sized to be used on a production system.

## nubskr/walrus

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/nubskr/walrus
- Категория: workload: storage
- Описание: 🦭 Distributed log streaming engine built from first principles
- Уровень: automated README evidence extraction
- Снимок: [sources/nubskr__walrus/README.md](sources/nubskr__walrus/README.md); SHA-256: `d38c9d5935bfaa79fa475b2b55a91425cc9f549a0e558f0b155829503cef7188`
- statistics: строка 72: make cluster-bootstrap
- async: строка 183: make cluster-test-stress       # Concurrent writes
- report: строка 107: STATE <topic>          → Get topic metadata (JSON)
- correctness: строка 195: ## Correctness
- isolation: строка 300: - **New**: Namespace isolation via `_for_key` constructors

## microsoft/DiskANN

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/microsoft/DiskANN
- Категория: workload: storage
- Описание: A vector indexing library to bring fast, fresh and filtered search to your database
- Уровень: automated README evidence extraction
- Снимок: [sources/microsoft__DiskANN/README.md](sources/microsoft__DiskANN/README.md); SHA-256: `f1f012a1d99fd5c41cadfddbeda6561915b4c8c36c4cdca3f918ff0d75050cb0`
- async: строка 8: - In-memory providers, for maximum performance. These are volatile and not intended for use in databases. DiskANN3 + in-memory providers [outperforms](https://github.com/microsoft/DiskANN/wiki/Perf:-In%E2%80%90memory-providers) HNSWlib on throughput.
- report: строка 9: - Disk provider, for larger than memory support. This is intended to match the performormance of the first version of DiskANN reported in [NeurIPS'19 Paper](https://papers.nips.cc/paper/9527-rand-nsg-fast-accurate-billion-point-nearest-neighbor-search-on-a-sin

## Brendonovich/prisma-client-rust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Brendonovich/prisma-client-rust
- Категория: workload: storage
- Описание: Type-safe database access for Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/Brendonovich__prisma-client-rust/README.md](sources/Brendonovich__prisma-client-rust/README.md); SHA-256: `223297ff79dbbb5ad4317bbead55b4eb7595bca9d60ef09bba12b53e35a0a8ff`
- lifecycle: строка 35: Read the [installation instructions](https://prisma.brendonovich.dev/getting-started/installation) to get started and setup the CLI.

## oxigraph/oxigraph

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/oxigraph/oxigraph
- Категория: workload: storage
- Описание: SPARQL graph database
- Уровень: automated README evidence extraction
- Снимок: [sources/oxigraph__oxigraph/README.md](sources/oxigraph__oxigraph/README.md); SHA-256: `ea28c020425b4d215f54a63be633daf46435d29befceb722c01ae8037093e4b8`
- memory: строка 96: * [Albin Larsson](https://byabbe.se/) who is building [GovDirectory](https://www.govdirectory.org/), a directory of public agencies based on Wikidata.
- report: строка 24: - [Turtle](https://www.w3.org/TR/turtle/), [TriG](https://www.w3.org/TR/trig/), [N-Triples](https://www.w3.org/TR/n-triples/), [N-Quads](https://www.w3.org/TR/n-quads/), [RDF/XML](https://www.w3.org/TR/rdf-syntax-grammar/) and [JSON-LD](https://www.w3.org/TR/j

## risinglightdb/risinglight

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/risinglightdb/risinglight
- Категория: workload: storage
- Описание: An educational OLAP database system.
- Уровень: automated README evidence extraction
- Снимок: [sources/risinglightdb__risinglight/README.md](sources/risinglightdb__risinglight/README.md); SHA-256: `a8bbee7fab69762fbdd0802a2e6dc1d265d5b33ccb44c20d5a678af973b7118e`
- report: строка 53: If you have a bug report or feature request, welcome to open an [issue](https://github.com/risinglightdb/risinglight/issues).

## superfly/corrosion

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/superfly/corrosion
- Категория: workload: storage
- Описание: Gossip-based service discovery (and more) for large distributed systems.
- Уровень: automated README evidence extraction
- Снимок: [sources/superfly__corrosion/README.md](sources/superfly__corrosion/README.md); SHA-256: `b09dda19f7a26294d935d6794277ea26daa18ecd0d62d99ee4cd17b2e2005379`

## cnosdb/cnosdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cnosdb/cnosdb
- Категория: workload: storage
- Описание: A cloud-native open source distributed time series database with high performance, high compression ratio and high availability.
- Уровень: automated README evidence extraction
- Снимок: [sources/cnosdb__cnosdb/README.md](sources/cnosdb__cnosdb/README.md); SHA-256: `3473ac63620ee53a382ffb58771c8e8feb2c45d7b1a643ddbaff84048c62d11d`
- report: строка 67: Please [report](https://github.com/cnosdb/cnosdb/issues) to us.

## SeaQL/sea-query

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SeaQL/sea-query
- Категория: workload: storage
- Описание: 🔱 A dynamic SQL query builder for MySQL, Postgres and SQLite
- Уровень: automated README evidence extraction
- Снимок: [sources/SeaQL__sea-query/README.md](sources/SeaQL__sea-query/README.md); SHA-256: `7ce56b5cca17f0b5b7a49972956621823b81357c54ef9e68d85fe16c817b22b0`
- async: строка 24: SeaQuery is the foundation of [SeaORM](https://github.com/SeaQL/sea-orm), an async & dynamic ORM for Rust.
- report: строка 51: Type support: `with-chrono`, `with-time`, `with-json`, `with-rust_decimal`, `with-bigdecimal`, `with-uuid`,

## cswinter/LocustDB

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cswinter/LocustDB
- Категория: workload: storage
- Описание: Blazingly fast analytics database that will rapidly devour all of your data.
- Уровень: automated README evidence extraction
- Снимок: [sources/cswinter__LocustDB/README.md](sources/cswinter__LocustDB/README.md); SHA-256: `a9d2513af2cbfdec6dc8f83775dbfadf0584914dce0678d72f947ad5361357a4`
- lifecycle: строка 85: LocustDB should be usable with minimal configuration or schema-setup as:
- memory: строка 46: --mem-lz4          Keep data cached in memory lz4 encoded. Decreases memory usage and query speeds.
- async: строка 78: ### Low latency
- report: строка 19: Download the [latest binary release][latest-release], which can be run from the command line on most x64 Linux systems, including Windows Subsystem for Linux. For example, to load the file `test_data/nyc-taxi.csv.gz` in this repository and start the repl run:

## tonbo-io/tonbo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tonbo-io/tonbo
- Категория: workload: storage
- Описание: Tonbo is an embedded database for serverless and edge runtimes.
- Уровень: automated README evidence extraction
- Снимок: [sources/tonbo-io__tonbo/README.md](sources/tonbo-io__tonbo/README.md); SHA-256: `4cb38ff1074dbd71d09f8322acc02a1f219cfc8eb07865b97f48a4bf6f34d372`
- comparison: строка 201: - `benches/compaction/results/compaction_local_baseline.md` for raw baseline evidence
- lifecycle: строка 113: **Quick setup for development:**
- async: строка 23: - **Async-first**: The entire storage and query engine is fully async, built for serverless and edge environments.
- report: строка 121: **Production with JSON output:**
- isolation: строка 100: Tonbo implements a merge-tree optimized for object storage: writes go to WAL → MemTable → Parquet SSTables, with MVCC for snapshot isolation and a manifest for coordination via compare-and-swap:

## losfair/mvsqlite

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/losfair/mvsqlite
- Категория: workload: storage
- Описание: Distributed, MVCC SQLite that runs on FoundationDB.
- Уровень: automated README evidence extraction
- Снимок: [sources/losfair__mvsqlite/README.md](sources/losfair__mvsqlite/README.md); SHA-256: `ed6fc085b27d90096fafa544de11238421e1f2f112958d2d18aabcb26fe0e7aa`
- async: строка 19: - **Lock-free, scalable reads and writes**: Optimistic fine-grained concurrency with [BEGIN CONCURRENT](https://www.sqlite.org/cgi/src/doc/begin-concurrent/doc/begin_concurrent.md)-like semantics. mvSQLite inherits FoundationDB's lock-free property - not a sin
- correctness: строка 20: - **Get the nice properties from FoundationDB, without its limits**: [Correctness](https://apple.github.io/foundationdb/testing.html), [really fast and scalable](https://apple.github.io/foundationdb/performance.html) distributed transactions, synchronous and a

## orbitinghail/graft

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/orbitinghail/graft
- Категория: workload: storage
- Описание: Graft is an open-source transactional storage engine optimized for lazy, partial, and strongly consistent replication—perfect for edge, offline-first, and distributed applications.
- Уровень: automated README evidence extraction
- Снимок: [sources/orbitinghail__graft/README.md](sources/orbitinghail__graft/README.md); SHA-256: `33cb2a600e96309bb64b5ce03d8e71cb13a776642a805ef93b638b91c50f5836`
- isolation: строка 21: - **Strong Consistency**: Serializable Snapshot Isolation ensures correct, consistent data views.

## skyzh/type-exercise-in-rust

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/skyzh/type-exercise-in-rust
- Категория: workload: storage
- Описание: Learn advanced Rust by building a type-safe, auto-vectorized database expression engine
- Уровень: automated README evidence extraction
- Снимок: [sources/skyzh__type-exercise-in-rust/README.md](sources/skyzh__type-exercise-in-rust/README.md); SHA-256: `26c11aff1459e483916639d4553e572576ba6a600945e88aca593cd46141b8a5`
- comparison: строка 54: Choose another branch name if `course-work` already exists. The starter baseline should compile.
- lifecycle: строка 45: [environment setup](https://skyzh.github.io/type-exercise-in-rust/setup.html), then work only in
- async: строка 23: The five modules follow the engine from physical values to its outer async boundary:

## postgresml/korvus

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/postgresml/korvus
- Категория: workload: storage
- Описание: Korvus is a search SDK that unifies the entire RAG pipeline in a single database query. Built on top of Postgres with bindings for Python, JavaScript, Rust and C.
- Уровень: automated README evidence extraction
- Снимок: [sources/postgresml__korvus/README.md](sources/postgresml__korvus/README.md); SHA-256: `2d378ca14bae0f8fe888b37991a283639755f868f8b8bc227a4b29faac2b56c4`
- async: строка 57: 1. **Postgres-Native RAG**: Korvus leverages Postgres' robust capabilities, allowing you to perform complex RAG operations directly within your database. This approach eliminates the need for external services and API calls, significantly reducing latency and

## tensorbase/tensorbase

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tensorbase/tensorbase
- Категория: workload: storage
- Описание: TensorBase is a new big data warehousing with modern efforts.
- Уровень: automated README evidence extraction
- Снимок: [sources/tensorbase__tensorbase/README.md](sources/tensorbase__tensorbase/README.md); SHA-256: `2436236201cdeb3b7aac1e1621d21ff87430e0a441520ae1ac9a60c8c5b96654`
- async: строка 24: 2. 2x faster write throughput than that of ClickHouse (based on [our bug fixed Rust client](https://github.com/tensorbase/tensorbase/tree/main/crates/client), you can get ~1.7x speedup by [our another simple concurrent script here](https://github.com/tensorbas

## lnx-search/lnx

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lnx-search/lnx
- Категория: workload: storage
- Описание: A flexible, performant and reliable search database without the AI bullshit.
- Уровень: automated README evidence extraction
- Снимок: [sources/lnx-search__lnx/README.md](sources/lnx-search__lnx/README.md); SHA-256: `9bd37a9712380a9ecfdef815257425921051b3db554ef013a5bc599e398d2a47`
- async: строка 45: lnx can provide the ability to fine tune the system to your particular use case. You can customise the async runtime threads. The concurrency thread pool, threads per reader and writer threads, all per index.
- report: строка 50: The bellow figures were taken by our `lnx-cli` on the small `movies.json` dataset, we didn't try any higher as Meilisearch takes an incredibly long time to index millions of docs although the new Meilisearch engine has improved this somewhat.

## PumpkinDB/PumpkinDB

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/PumpkinDB/PumpkinDB
- Категория: workload: storage
- Описание: Immutable Ordered Key-Value Database Engine
- Уровень: automated README evidence extraction
- Снимок: [sources/PumpkinDB__PumpkinDB/README.md](sources/PumpkinDB__PumpkinDB/README.md); SHA-256: `72980811cc973e5a5c63b6de2ec144508a9e91e7410a1c33ebc8e59204f80d6c`
- async: строка 45: PumpkinDB offers a wide array of primitives for concurrency, storage, journalling, indexing and other common building blocks.
- report: строка 21: * Binary keys and values (allows any encoding to be used: JSON, XML, Protobuf, Cap'n Proto, etc.)

## teodevgroup/teo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/teodevgroup/teo
- Категория: workload: storage
- Описание: High performance ergonomic ORM for Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/teodevgroup__teo/README.md](sources/teodevgroup__teo/README.md); SHA-256: `cd57d1fa96710c494bf0fc2918809decc12ab1c67185c901261845d8780d3771`

## exonum/exonum

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/exonum/exonum
- Категория: workload: storage
- Описание: An extensible open-source framework for creating private/permissioned blockchain applications
- Уровень: automated README evidence extraction
- Снимок: [sources/exonum__exonum/README.md](sources/exonum__exonum/README.md); SHA-256: `7634e8e9d809341ad32a91cfeda496b0717c394b73f501e3170d292f6f16d71e`
- memory: строка 98: but no specific effort is allocated into supporting them.

## PoloDB/PoloDB

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/PoloDB/PoloDB
- Категория: workload: storage
- Описание: PoloDB is an embedded document database.
- Уровень: automated README evidence extraction
- Снимок: [sources/PoloDB__PoloDB/README.md](sources/PoloDB__PoloDB/README.md); SHA-256: `2890a31b95e8c4f3aaebd83b2be49bd92554e11c5e7ec2290e77cdc245f4cf35`
- async: строка 27: - Concurrent use through clonable database handles

## stoolap/stoolap

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/stoolap/stoolap
- Категория: workload: storage
- Описание: A Modern Embedded SQL Database written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/stoolap__stoolap/README.md](sources/stoolap__stoolap/README.md); SHA-256: `5eff20fc38b7d5990201479406ccd4a516660faf7244ffb805774d739216f7b3`
- async: строка 25: It targets low-latency transactional workloads and real-time analytical queries, with modern SQL features and no external server process.
- correctness: строка 248: Benchmark figures are point-in-time and workload-dependent. Validate on your own hardware, data distribution, and query patterns.
- isolation: строка 31: - **ACID + MVCC**: concurrent reads and writes with transaction isolation

## cornucopia-rs/cornucopia

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cornucopia-rs/cornucopia
- Категория: workload: storage
- Описание: Generate type-checked Rust from your PostgreSQL.
- Уровень: automated README evidence extraction
- Снимок: [sources/cornucopia-rs__cornucopia/README.md](sources/cornucopia-rs__cornucopia/README.md); SHA-256: `94ebdf2c076c60be8276724ccaeeb82a4282188ec8b43c34a63c34691da68284`
- async: строка 34: - **Flexible** - Works with sync/async code and connection pools.
- correctness: строка 19: Cornucopia generates type-checked Rust interfaces from PostgreSQL queries, with an emphasis on compile-time safety and high performance. It works by preparing your queries against an actual database and then running an extensive validation suite on them. Rust

## joaoh82/rust_sqlite

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/joaoh82/rust_sqlite
- Категория: workload: storage
- Описание: SQLRite - Simple embedded database modeled off SQLite in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/joaoh82__rust_sqlite/README.md](sources/joaoh82__rust_sqlite/README.md); SHA-256: `5536ba3529164607278304d8516ec56a6f8873f51765d2fdbf9f22856157faee`
- lifecycle: строка 306: - [x] **6c — Trusted publisher setup + branch protection runbook**: [`docs/release-secrets.md`](docs/release-secrets.md) captures the one-time web-UI setup — crates.io token in the `release` environment, OIDC trusted publishers on PyPI (`sqlrite`) and npm (`@j
- memory: строка 321: - [x] **7c — Bounded-heap top-k optimization** *(v0.1.12)*
- async: строка 283: **Phase 4 — Durability and concurrency** *(done)*
- report: строка 95: Wire it into Claude Code (`~/.claude.json`):
- correctness: строка 248: - [x] CLI + rustyline REPL with history, syntax highlighting, bracket matching, line validation

## khonsulabs/bonsaidb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/khonsulabs/bonsaidb
- Категория: workload: storage
- Описание: A developer-friendly document database that grows with you, written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/khonsulabs__bonsaidb/README.md](sources/khonsulabs__bonsaidb/README.md); SHA-256: `2d2da307364c2b7d15559b4ecad026f1cefca2b82c49cae53a3d341dfcc4514d`
- async: строка 144: - `async`: Enables async support with Tokio.
- report: строка 7: [![HTML Coverage Report for `main`](https://dev.bonsaidb.io/main/coverage/badge.svg)](https://dev.bonsaidb.io/main/coverage/)

## Evokoa/pgGraph

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Evokoa/pgGraph
- Категория: workload: storage
- Описание: Open-source graph database superpowers for your existing Postgres data.
- Уровень: automated README evidence extraction
- Снимок: [sources/Evokoa__pgGraph/README.md](sources/Evokoa__pgGraph/README.md); SHA-256: `27a8ec18e2af72efdf7cfc923854d3f5d912690aa8db9c62404fb99fc3c53a95`
- lifecycle: строка 176: - `setup`: build and start Postgres with pgGraph installed, but do not load the
- async: строка 292: private snapshot prevents a concurrent write or truncation of the source file
- correctness: строка 289: atomically. When a new Postgres backend spins up, it validates the artifact

## GlareDB/glaredb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/GlareDB/glaredb
- Категория: workload: storage
- Описание: GlareDB: A light and fast SQL database for analytics
- Уровень: automated README evidence extraction
- Снимок: [sources/GlareDB__glaredb/README.md](sources/GlareDB__glaredb/README.md); SHA-256: `46608afc0bd302533096498bcd047f235c52e98d048dc3a1adc820bbecc4ae24`

## duckdb/duckdb-rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/duckdb/duckdb-rs
- Категория: workload: storage
- Описание:  Ergonomic bindings to duckdb for Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/duckdb__duckdb-rs/README.md](sources/duckdb__duckdb-rs/README.md); SHA-256: `8a7c7dc2877e6f8929dc89daf4d5100d0b4a2d3dfece1e32a7c0c30146fe6e5c`
- memory: строка 86: - It enables upstream jemalloc on supported 64-bit, non-musl Linux targets. Set `DUCKDB_DISABLE_JEMALLOC=1` to force the standard allocator.
- report: строка 12: - Read and write Arrow, Parquet, JSON, and CSV formats natively.

## ovexro/dockpanel

- Итог: excluded — Server administration product outside selected scope
- Источник: https://github.com/ovexro/dockpanel
- Категория: workload: storage
- Описание: Modern server management panel built with Rust and React. Sites, databases, Docker apps, Git deploy, mail, DNS, monitoring, backups, and security — all in one panel.
- Уровень: automated README evidence extraction
- Снимок: [sources/ovexro__dockpanel/README.md](sources/ovexro__dockpanel/README.md); SHA-256: `8a51578af720e154a212c4f42f4dc31d8c0716fa123da2558c9f790e18631d88`
- comparison: строка 9: Self-hosted. Docker-native. Written in Rust. Panel services run on <strong>~49MB of RAM</strong>. 837 HTTP routes. 147 app templates. 4333 regression assertions. ~49MB binaries. Zero subscriptions.
- lifecycle: строка 65: features whose setup half worked and whose payoff half had never once run.
- gpu: строка 45: No other free panel gives you Git push-to-deploy with blue-green zero-downtime updates, 147 one-click Docker app templates, per-image CVE scanning with deploy gating, a WAF, passkey login, GPU passthrough, multi-server management, reseller accounts, a develope
- report: строка 2: <img src=".github/screenshots/dp-dashboard.png" alt="DockPanel Dashboard" width="800">
- isolation: строка 192: - **Container Management** — Auto-sleep (stop idle containers), auto-update detection, per-user isolation policies.

## djc/bb8

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/djc/bb8
- Категория: workload: storage
- Описание: Full-featured async (tokio-based) postgres connection pool (like r2d2)
- Уровень: automated README evidence extraction
- Снимок: [sources/djc__bb8/README.md](sources/djc__bb8/README.md); SHA-256: `46a26fdb13ebbb76a2bd8a472494b3dbdbb37868d98145d22cf72a3902dedc5d`
- async: строка 34: [memcache-async](https://github.com/vavrusa/memcache-async) | [bb8-memcached](https://crates.io/crates/bb8-memcached)

## feigeCode/navop

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/feigeCode/navop
- Категория: workload: storage
- Описание: A native, all-in-one workspace for databases, SSH, SFTP, terminals, remote desktop, monitoring, and AI.
- Уровень: automated README evidence extraction
- Снимок: [sources/feigeCode__navop/README.md](sources/feigeCode__navop/README.md); SHA-256: `c477285c73632ab448b851e94977ff49f7f500fa727e0a0374abfb379f5510da`
- statistics: строка 145: ./script/bootstrap
- gpu: строка 6: <p>Built with <a href="https://gpui.rs">GPUI</a> and Rust · GPU-accelerated rendering</p>
- report: строка 135: If macOS Gatekeeper reports that Apple cannot check the app, run `sudo xattr -rd com.apple.quarantine /Applications/Navop.app`.
- correctness: строка 126: Download the latest build from [GitHub Releases](https://github.com/feigeCode/navop/releases/latest). Each release includes `sha256sums.txt` for checksum verification. Artifacts are available for macOS (DMG and tar.gz, Apple Silicon / Intel), Windows (MSI and

## antoniosarosi/mkdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/antoniosarosi/mkdb
- Категория: workload: storage
- Описание: Toy Database
- Уровень: automated README evidence extraction
- Снимок: [sources/antoniosarosi__mkdb/README.md](sources/antoniosarosi__mkdb/README.md); SHA-256: `858489be4e57e80bb866d0b5383fdfcc6a4e85fb9a91e82936444408ef3406c6`

## lanterndata/lantern

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lanterndata/lantern
- Категория: workload: storage
- Описание: PostgreSQL vector database extension for building AI applications
- Уровень: automated README evidence extraction
- Снимок: [sources/lanterndata__lantern/README.md](sources/lanterndata__lantern/README.md); SHA-256: `54da83c5944e7bd9f3f981e121cd0a5bfef77ff91c937becc5907f4ce4d24fbb`
- async: строка 145: - There's three key metrics we track. `CREATE INDEX` time, `SELECT` throughput, and `SELECT` latency.
- report: строка 168: - [GitHub issues](https://github.com/lanterndata/lantern/issues): report bugs or issues with Lantern

## supabase/wrappers

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/supabase/wrappers
- Категория: workload: storage
- Описание: Postgres Foreign Data Wrapper development framework in Rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/supabase__wrappers/README.md](sources/supabase__wrappers/README.md); SHA-256: `a59ade901965e47cecf13b830a7b7eecaceeb13f58c87457640d653f5941f83e`
- async: строка 49: - Support both sync and async backends, such as RDBMS, RESTful APIs, flat files and etc.

## sunng87/pgwire

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sunng87/pgwire
- Категория: workload: storage
- Описание: PostgreSQL wire protocol implemented as a rust library.
- Уровень: automated README evidence extraction
- Снимок: [sources/sunng87__pgwire/README.md](sources/sunng87__pgwire/README.md); SHA-256: `eeb085408027016c363f00a86dce611659357b0fbbe048c62f74372200d3f036`

## GrafeoDB/grafeo

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/GrafeoDB/grafeo
- Категория: workload: storage
- Описание: Grafeo is a pure-Rust, high-performance graph database that can be embedded as a library or run as a standalone database, with optional in-memory or persistent storage. Grafeo supports both LPG and RDF and all major query languages.
- Уровень: automated README evidence extraction
- Снимок: [sources/GrafeoDB__grafeo/README.md](sources/GrafeoDB__grafeo/README.md); SHA-256: `e87080433629094db959ffb300a4565b8c1b4ad83d18090576e96924da5a4e2a`
- lifecycle: строка 80: - **Encryption at rest** (`encryption` feature): AES-256-GCM for WAL records and `.grafeo` sections, password-based (Argon2id) or raw-key setup
- memory: строка 301: db.detailed_stats() # Memory usage, index counts
- async: строка 91: - **Async storage** (`async-storage` feature): non-blocking WAL and snapshot I/O via tokio
- report: строка 4: [![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/GrafeoDB/grafeo?utm_source=badge)
- correctness: строка 47: - **SPARQL** (W3C 1.1) with SHACL validation and Ring Index WCOJ planner
- isolation: строка 39: - MVCC transactions with snapshot isolation

## 0b01/tectonicdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/0b01/tectonicdb
- Категория: workload: storage
- Описание: Database for L2 orderbook
- Уровень: automated README evidence extraction
- Снимок: [sources/0b01__tectonicdb/README.md](sources/0b01__tectonicdb/README.md); SHA-256: `45c7e97285c4046bc198d3b1be0c53336204f14da5b6472748055fb458c42d53`
- lifecycle: строка 59: It's very easy to setup.
- async: строка 224: * 0.3.0: Refactor to async
- report: строка 144: 1. `INFO` reports the current tick count in memory and on disk.

## KipData/KiteSQL

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/KipData/KiteSQL
- Категория: workload: storage
- Описание: Embedded relational database and native Rust data API.
- Уровень: automated README evidence extraction
- Снимок: [sources/KipData__KiteSQL/README.md](sources/KipData__KiteSQL/README.md); SHA-256: `cdef8e3e61c6339bc3d3f35f9c7dc02ebfe35b7698b5c6966149c9ef918885fe`
- lifecycle: строка 168: - Build: `wasm-pack build --release --target nodejs` (outputs to `./pkg`; use `--target web` or `--target bundler` for browser/bundler setups).
- memory: строка 183: - Constructor is explicit: `Database(path, backend="rocksdb")`; use `backend="lmdb"` to open LMDB. In-memory usage is `Database.in_memory()`.
- isolation: строка 142: - Transaction isolation is documented in [`docs/transaction-isolation.md`](docs/transaction-isolation.md).

## opral/lix

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/opral/lix
- Категория: workload: storage
- Описание: Embeddable repository that combines files, database, and version control.
- Уровень: automated README evidence extraction
- Снимок: [sources/opral__lix/README.md](sources/opral__lix/README.md); SHA-256: `1186d9c7c5f7f35987f69d0b06ee1648338feb4740b6f734f9ff8d11c08c98dc`
- report: строка 151: <img src="./website/public/assets/file-to-rows.svg" alt="A plugin maps /orders.csv to SQL rows with row, field, and value columns" width="760" />

## s2-streamstore/s2

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/s2-streamstore/s2
- Категория: workload: storage
- Описание: Durable Streams API
- Уровень: automated README evidence extraction
- Снимок: [sources/s2-streamstore__s2/README.md](sources/s2-streamstore__s2/README.md); SHA-256: `fd03e4ba0cade52d0c72ec3debabc3123665a86b95028b0aa6e2cb813633f4a9`
- async: строка 222: - Appends are pipelined to improve performance against high-latency object storage

## vincent-herlemont/native_db

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vincent-herlemont/native_db
- Категория: workload: storage
- Описание: Drop-in embedded database in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/vincent-herlemont__native_db/README.md](sources/vincent-herlemont__native_db/README.md); SHA-256: `a2fac552aa544834f833a2acf1b4c444bf54624951aad31b8f74d7bc0ef44d9b`

## blackbeam/rust-mysql-simple

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/blackbeam/rust-mysql-simple
- Категория: workload: storage
- Описание: Mysql client library implemented in rust.
- Уровень: automated README evidence extraction
- Снимок: [sources/blackbeam__rust-mysql-simple/README.md](sources/blackbeam__rust-mysql-simple/README.md); SHA-256: `cc8fdfefc0f501dda7a9cce03aa19b8c590364e440646e2afd6a4a5cabb31c38`
- memory: строка 728: that helps to avoid allocations for basic scenarios. You can control its characteristics using
- report: строка 497: Wrapper structures for cases, when you need to provide a value for a JSON cell,

## earth-mover/icechunk

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/earth-mover/icechunk
- Категория: workload: storage
- Описание: Open-source, cloud-native transactional tensor storage engine
- Уровень: automated README evidence extraction
- Снимок: [sources/earth-mover__icechunk/README.md](sources/earth-mover__icechunk/README.md); SHA-256: `38a5f3fe86da986457bffbd90b7a2dc70ab772f64966863e0f1cdd1e965eed4d`
- async: строка 85: 1. **Serializable isolation** - Reads are isolated from concurrent writes and always use a committed snapshot of a repo. Writes are committed atomically and are never partially visible. No locks are required for reading.
- report: строка 115: Arbitrary JSON-style key-value metadata can be attached to both arrays and groups.
- isolation: строка 70: - **Transactional** - The key improvement that Icechunk brings on top of regular Zarr is to provide consistent serializable isolation between transactions.

## KOBA789/relly

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/KOBA789/relly
- Категория: workload: storage
- Описание: RDBMS のしくみを学ぶための小さな RDBMS 実装
- Уровень: automated README evidence extraction
- Снимок: [sources/KOBA789__relly/README.md](sources/KOBA789__relly/README.md); SHA-256: `c92dd5595ea5a0be674556228e1d4f2d4a509003b9f5527313a154590900cbf5`

## vrmiguel/pgpad

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vrmiguel/pgpad
- Категория: workload: storage
- Описание: A small, fast cross-platform database client
- Уровень: automated README evidence extraction
- Снимок: [sources/vrmiguel__pgpad/README.md](sources/vrmiguel__pgpad/README.md); SHA-256: `2b178832bdc02fddeb106d4f90f6297674b812158e303d9b42eaf125a541315a`
- lifecycle: строка 50: ### Setup
- report: строка 72: Feel free to open issues for bug reports and feature requests.

## Pometry/Raphtory

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Pometry/Raphtory
- Категория: workload: storage
- Описание: Scalable graph analytics database powered by a multithreaded, vectorized temporal engine, written in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/Pometry__Raphtory/README.md](sources/Pometry__Raphtory/README.md); SHA-256: `02f6671a1104d38ab9276861dc78c65f4c7500cee817d87b483eba7701540b76`
- report: строка 40: <a href="https://github.com/Raphtory/Raphtory/issues">🐛 Report a Bug</a>

## Fullstop000/wickdb

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Fullstop000/wickdb
- Категория: workload: storage
- Описание: Pure Rust LSM-tree based embedded storage engine
- Уровень: automated README evidence extraction
- Снимок: [sources/Fullstop000__wickdb/README.md](sources/Fullstop000__wickdb/README.md); SHA-256: `6e34995497d97a8cc19ba6f3adb219949c5b7ffdfed521fa926cc3d0f4c0ceed`

## ProjectPhysX/FluidX3D

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ProjectPhysX/FluidX3D
- Категория: benchmark/testing candidate
- Описание: The fastest and most memory efficient lattice Boltzmann CFD software, running on all GPUs and CPUs via OpenCL. Free for non-commercial use.
- Уровень: automated README evidence extraction
- Снимок: [sources/ProjectPhysX__FluidX3D/README.md](sources/ProjectPhysX__FluidX3D/README.md); SHA-256: `c59f40c9305a73c4abe1ec2e74a124a06547bb2052ac676019931613b8102a5b`
- lifecycle: строка 20: - added Stokes drag validation setup
- memory: строка 41: - displayed GPU memory allocation size is now fully accurate
- gpu: строка 13: - [v1.1](https://github.com/ProjectPhysX/FluidX3D/releases/tag/v1.1) (29.09.2022) [changes](https://github.com/ProjectPhysX/FluidX3D/compare/v1.0...v1.1) (GPU voxelization)
- report: строка 60: - patched OpenCL issues of Intel Arc GPUs: now VRAM allocations >4GB are possible and correct VRAM capacity is reported
- correctness: строка 20: - added Stokes drag validation setup

## idealvin/coost

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/idealvin/coost
- Категория: benchmark/testing candidate
- Описание: A tiny boost library in C++11.
- Уровень: automated README evidence extraction
- Снимок: [sources/idealvin__coost/readme.md](sources/idealvin__coost/readme.md); SHA-256: `50a9e9d09ae95829286fc66867ede72368165a79d0f9c09b34f7bc28e7b6921f`
- memory: строка 50: - **Fast memory allocator**
- report: строка 30: - **JSON RPC framework**

## felixguendling/cista

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/felixguendling/cista
- Категория: benchmark/testing candidate
- Описание: Cista is a simple, high-performance, zero-copy C++ serialization & reflection library.
- Уровень: automated README evidence extraction
- Снимок: [sources/felixguendling__cista/README.md](sources/felixguendling__cista/README.md); SHA-256: `8013c743a7c1e1643b7dafd382daf32d35c32db4e807b955d596665629cfb222`
- report: строка 175: Feel free to contribute (bug reports, pull requests, etc.)!

## ashvardanian/less_slow.cpp

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ashvardanian/less_slow.cpp
- Категория: benchmark/testing candidate
- Описание: Playing around "Less Slow" coding practices in C++ 20, C, CUDA, PTX, & Assembly, from numerics & SIMD to coroutines, ranges, exception handling, networking and user-space IO
- Уровень: automated README evidence extraction
- Снимок: [sources/ashvardanian__less_slow.cpp/README.md](sources/ashvardanian__less_slow.cpp/README.md); SHA-256: `e9e5d63be6f2dd595526bb53a544242eee8f76c60106416666f510bebf7bd4d6`
- comparison: строка 116: To enhance stability and reproducibility, disable Simultaneous Multi-Threading __(SMT)__ on your CPU and use the `--benchmark_enable_random_interleaving=true` flag, which shuffles and interleaves benchmarks as described [here](https://github.com/google/benchma
- lifecycle: строка 180: - `->MinWarmUpTime(n)` - To warm up the data caches
- memory: строка 19: - __100x cheaper random inputs?!__ Discover how input generation sometimes costs more than the algorithm.
- gpu: строка 86: - Nvidia's [CCCL](https://github.com/nvidia/cccl) for GPU-accelerated algorithms.
- async: строка 24: - __Scaling AI?__ Measure the gap between theoretical [ALU](https://en.wikipedia.org/wiki/Arithmetic_logic_unit) throughput and your [BLAS](https://en.wikipedia.org/wiki/Basic_Linear_Algebra_Subprograms).
- report: строка 29: - __How to handle [JSON](https://www.json.org/json-en.html) avoiding memory allocations?__ Is it easier with C++ 20 or old-school C 99 tools?
- correctness: строка 116: To enhance stability and reproducibility, disable Simultaneous Multi-Threading __(SMT)__ on your CPU and use the `--benchmark_enable_random_interleaving=true` flag, which shuffles and interleaves benchmarks as described [here](https://github.com/google/benchma

## JonMagon/KDiskMark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/JonMagon/KDiskMark
- Категория: benchmark/testing candidate
- Описание: A simple open-source disk benchmark tool for Linux distros
- Уровень: automated README evidence extraction
- Снимок: [sources/JonMagon__KDiskMark/README.md](sources/JonMagon__KDiskMark/README.md); SHA-256: `240ad66914313d9cfe7e64f06b474aa0720a9878334d8dd722077908e724fe18`
- report: строка 15: * Report generation

## openai/procgen

- Итог: excluded — Policy training environment benchmark
- Источник: https://github.com/openai/procgen
- Категория: benchmark/testing candidate
- Описание: Procgen Benchmark: Procedurally-Generated Game-Like Gym-Environments
- Уровень: automated README evidence extraction
- Снимок: [sources/openai__procgen/README.md](sources/openai__procgen/README.md); SHA-256: `27022515ed7879b95bb8b4a7db01c5f7c0207468f48441bf3180cdcc0a279c7f`
- lifecycle: строка 80: A [`Dockerfile`](docker/Dockerfile) is included to demonstrate a minimal Docker-based setup that works for running random agent.
- correctness: строка 130: Rather than patch these issues, we plan to keep the environments in their originally released form, in order to ease the reproducibility of results that are already published.

## kimwalisch/primesieve

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/kimwalisch/primesieve
- Категория: benchmark/testing candidate
- Описание: 🚀 Fast prime number generator
- Уровень: automated README evidence extraction
- Снимок: [sources/kimwalisch__primesieve/README.md](sources/kimwalisch__primesieve/README.md); SHA-256: `a146dd42418865afa28e861dcb9a189ae2b9a07de477145abcadcd00dfafa26a`
- memory: строка 8: It is very cache efficient, it detects your CPU's L1 & L2 cache sizes and allocates its main
- async: строка 85: maximum throughput, generate primes in memory using [libprimesieve](doc/C_API.md).
- correctness: строка 144: --test                 Run various correctness tests (< 1 minute).

## baidu-research/DeepBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/baidu-research/DeepBench
- Категория: benchmark/testing candidate
- Описание: Benchmarking Deep Learning operations on different hardware
- Уровень: automated README evidence extraction
- Снимок: [sources/baidu-research__DeepBench/README.md](sources/baidu-research__DeepBench/README.md); SHA-256: `d56cb04f1f3799a171f6e48536fa6526efa0f9c79dc23f942d9c0cb76e063ac7`
- comparison: строка 347: dense baselines. However, current implementations of sparse matrix multiply are optimized for much higher
- gpu: строка 208: #### Topology for NVIDIA 8 GPU System
- async: строка 202: We report the shortest latency achieved from all implementations for each configuration.
- report: строка 202: We report the shortest latency achieved from all implementations for each configuration.

## redis/memtier_benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/redis/memtier_benchmark
- Категория: benchmark/testing candidate
- Описание: NoSQL Redis and Memcache traffic generation and benchmarking tool.
- Уровень: automated README evidence extraction
- Снимок: [sources/redis__memtier_benchmark/README.md](sources/redis__memtier_benchmark/README.md); SHA-256: `dcbd2c6d22c0a36a96df4f81a7bc2a104fb70426ce618064ae390e498c2edd83`
- statistics: строка 291: A follow-up fixture that bootstraps a real `redis-cli --cluster create`
- lifecycle: строка 296: When you impose a rate limit on your benchmark tests, you're essentially mimicking a controlled production environment. This setup is crucial for understanding how latency behaves under certain throughput constraints. Here's why benchmarking latency in a rate-
- memory: строка 333: When used for normally distributed data, the samples are usually taken at regular intervals. However, since the data does not obey to a normal distribution it would be very expensive to keep equally spaced intervals of latency records while enabling large valu
- async: строка 22: * [A High Throughput Benchmarking Tool for Redis and Memcached](https://redis.io/blog/memtier_benchmark-a-high-throughput-benchmarking-tool-for-redis-memcached)
- report: строка 196: > **Note:** When using `--command-stats-breakdown=command`, the JSON output's Time-Serie percentiles (p50, p99, etc.) for aggregated command types are approximate. They reflect only one of the underlying commands rather than a true merge of all commands of tha

## travisdowns/uarch-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/travisdowns/uarch-bench
- Категория: benchmark/testing candidate
- Описание: A benchmark for low-level CPU micro-architectural features
- Уровень: automated README evidence extraction
- Снимок: [sources/travisdowns__uarch-bench/README.md](sources/travisdowns__uarch-bench/README.md); SHA-256: `6bb33c5904faf3f612612b39df42ebe24710126e8ad86a6105d907b95a2fb56b`
- statistics: строка 28: test itself is a few lines of code, but the cost is in all the infrastructure: implementing the timing code, converting measurements to cycles, removing outliers, running the tests for various parameters, reporting the results, whatever. This
- report: строка 28: test itself is a few lines of code, but the cost is in all the infrastructure: implementing the timing code, converting measurements to cycles, removing outliers, running the tests for various parameters, reporting the results, whatever. This
- correctness: строка 15: At the moment it supports only x86, using mostly assembly and a few C++ benchmarks. In the future, I'd like to have more C or C++ benchmarks, allowing coverage (in principle) of more platforms (non-x86 assembly level benchmarks are also welcome). Of course, fo

## thekvs/cpp-serializers

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/thekvs/cpp-serializers
- Категория: benchmark/testing candidate
- Описание: Benchmark comparing various data serialization libraries (thrift, protobuf etc.) for C++
- Уровень: automated README evidence extraction
- Снимок: [sources/thekvs__cpp-serializers/README.md](sources/thekvs__cpp-serializers/README.md); SHA-256: `ee20fc41d47dfea5bf1d569e8f371acbd642ca645d4c1eaccf26e5ab9abc6dfb`
- report: строка 36: -c, --csv              output in CSV format

## Mellanox/sockperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Mellanox/sockperf
- Категория: benchmark/testing candidate
- Описание: Network Benchmarking Utility
- Уровень: automated README evidence extraction
- Снимок: [sources/Mellanox__sockperf/README.md](sources/Mellanox__sockperf/README.md); SHA-256: `a3d7300c0829d521b1b0080cddee54b2ff99908436b84bd90fc90cac60d0b1db`
- async: строка 3: **sockperf** is a network benchmarking utility over socket API that was designed for testing performance (latency and throughput) of high-performance systems (it is also good for testing performance of regular networking systems). It covers most of the socket

## Jittor/JNeRF

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/Jittor/JNeRF
- Категория: benchmark/testing candidate
- Описание: JNeRF is a NeRF benchmark based on Jittor. JNeRF re-implemented instant-ngp and achieved same performance with original paper.
- Уровень: automated README evidence extraction
- Снимок: [sources/Jittor__JNeRF/README.md](sources/Jittor__JNeRF/README.md); SHA-256: `5260615c95ee0e25db6d95a5146a19e2042d8cbbc367f122cf00a818a5ce6f11`
- comparison: строка 58: If you want to train JNerf with your own dataset, then you should follow the format of our datasets. You should split your datasets into training, validation and testing sets. Each set should be paired with a json file that describes the camera parameters of e
- gpu: строка 21: * GPU compiler (optional)
- report: строка 58: If you want to train JNerf with your own dataset, then you should follow the format of our datasets. You should split your datasets into training, validation and testing sets. Each set should be paired with a json file that describes the camera parameters of e
- correctness: строка 58: If you want to train JNerf with your own dataset, then you should follow the format of our datasets. You should split your datasets into training, validation and testing sets. Each set should be paired with a json file that describes the camera parameters of e

## openxrlab/xrslam

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/openxrlab/xrslam
- Категория: benchmark/testing candidate
- Описание: OpenXRLab Visual-inertial SLAM Toolbox and Benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/openxrlab__xrslam/README.md](sources/openxrlab__xrslam/README.md); SHA-256: `2bff2cb4b3093c41936f1a001758106620e33a3512377af78876eebdf818a575`

## openxrlab/xrnerf

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/openxrlab/xrnerf
- Категория: benchmark/testing candidate
- Описание: OpenXRLab Neural Radiance Field (NeRF) Toolbox and Benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/openxrlab__xrnerf/README.md](sources/openxrlab__xrnerf/README.md); SHA-256: `75e0859d8fbe40fcdcfbc1fa8e1b63404933d6e0824e6a5671c0702dce4b63cf`

## QingyongHu/SensatUrban

- Итог: excluded — Semantic segmentation dataset
- Источник: https://github.com/QingyongHu/SensatUrban
- Категория: benchmark/testing candidate
- Описание: 🔥Urban-scale point cloud dataset (CVPR 2021 & IJCV 2022)
- Уровень: automated README evidence extraction
- Снимок: [sources/QingyongHu__SensatUrban/README.md](sources/QingyongHu__SensatUrban/README.md); SHA-256: `e15bc50caf1884da08603c266c03fb2329f4058496f73cddbac3e93cd03c1d69`
- lifecycle: строка 82: - Setup the environment
- gpu: строка 115: python main_SensatUrban.py --mode train --gpu 0

## p-ranav/alpaca

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/p-ranav/alpaca
- Категория: benchmark/testing candidate
- Описание: Serialization library written in C++17 - Pack C++ structs into a compact byte-array without any macros or boilerplate code
- Уровень: automated README evidence extraction
- Снимок: [sources/p-ranav__alpaca/README.md](sources/p-ranav__alpaca/README.md); SHA-256: `43f4ac608baecd4c60b5b04db7cc76b7fde6b153eb1d1ad064332ebf3cbd06d9`
- report: строка 539: alpaca also support `std::variant`. Although this is an uncommon data structure for one to use in a messaging framework, it is supported and available. Miscellaneous configuration parameters, like in JSON, can be serialized as variant values and sent to server
- correctness: строка 27: - Optional integrity checking - detects data corruption during deserialization using checksums

## frol/completely-unscientific-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/frol/completely-unscientific-benchmarks
- Категория: benchmark/testing candidate
- Описание: Naive performance comparison of a few programming languages (JavaScript, Kotlin, Rust, Swift, Nim, Python, Go, Haskell, D, C++, Java, C#, Object Pascal, Ada, Lua, Ruby)
- Уровень: automated README evidence extraction
- Снимок: [sources/frol__completely-unscientific-benchmarks/README.md](sources/frol__completely-unscientific-benchmarks/README.md); SHA-256: `b707f37ed177c8ecea27ddade6c918eb312ca0e656fa32bb2f3e1d6bcf10d50b`
- comparison: строка 33: experience in a given language would implement as a baseline "good enough"
- memory: строка 67: leverages CGroup capabilities to capture the high-water RSS+CACHE memory usage,
- gpu: строка 77: a "bare metal" performance. Over time, we received optimized solutions in other
- correctness: строка 34: solution where correctness is more important than performance.

## RRZE-HPC/gpu-benches

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/RRZE-HPC/gpu-benches
- Категория: benchmark/testing candidate
- Описание: collection of benchmarks to measure basic GPU capabilities
- Уровень: automated README evidence extraction
- Снимок: [sources/RRZE-HPC__gpu-benches/README.md](sources/RRZE-HPC__gpu-benches/README.md); SHA-256: `c4113132955c2b186ee0f86f64b60901bac1b4b77282ab8ff41c1a252412806b`
- memory: строка 19: Measures the bandwidth of streaming kernels for varying occupancy. A shared memory allocation serves as a spoiler, so that only two thread blocks can run per SM. Scanning the thread block size from 32 to 1024 scans the occupancy from 3% to 100%.
- gpu: строка 1: # GPU benchmarks
- async: строка 38: ## gpu-latency

## krrishnarraj/clpeak

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/krrishnarraj/clpeak
- Категория: benchmark/testing candidate
- Описание: A synthetic micro-benchmark that measures peak compute, bandwidth, and matrix throughput of GPUs and CPUs
- Уровень: automated README evidence extraction
- Снимок: [sources/krrishnarraj__clpeak/README.md](sources/krrishnarraj__clpeak/README.md); SHA-256: `34a7a3eaccf9032e2e1c64888f3cf672958fe1fc3b610bcc4a8479e3174d187e`
- comparison: строка 16: Peak lines from real runs, condensed (see `results/` for full baselines).
- gpu: строка 10: Originally an OpenCL benchmark, clpeak now supports OpenCL, Vulkan, CUDA, ROCm/HIP, Metal, oneAPI/SYCL, ONNX and native CPU execution, enabling direct cross-backend comparisons on the same hardware.
- async: строка 8: **clpeak &mdash; "Compute Latency PEAK".** A synthetic micro-benchmark for measuring the peak achievable compute performance of CPUs, GPUs and NPUs. It exercises tight vector, MAD, and MMA kernels, together with vendor-optimized GEMM libraries, to expose peak
- report: строка 62: Same engine as the CLI, with device detection, live results and run history — one Flutter app for **macOS, Linux and Windows** (plus Android/iOS from the same codebase, over the `clpeak_ffi` C ABI). Grab it from the [latest release](https://github.com/krrishna

## zupat/related_post_gen

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/zupat/related_post_gen
- Категория: benchmark/testing candidate
- Описание: Data Processing benchmark featuring Rust, Go, Swift, Zig, Julia etc.
- Уровень: automated README evidence extraction
- Снимок: [sources/zupat__related_post_gen/readme.md](sources/zupat__related_post_gen/readme.md); SHA-256: `a925fcf4dd66653149c6865ec607f56f51f2df2533d4711349e330a95ed9b12f`
- lifecycle: строка 20: See [setup.md](setup.md) for complete guide.
- memory: строка 151: | Rust v3    | -               | 1.28s         | Preallocate and reuse map and unstable sort by [vdrmn](https://www.reddit.com/r/rust/comments/16plgok/comment/k1rzo7g/?utm_source=share&utm_medium=web2x&context=3) and [Darksonn](https://www.reddit.com/r/rust/co
- async: строка 119: | D Concurrent (v2) | 6.22 ms | 46.71 ms | $\textsf{\color{lightgreen}273.33 ms}$ | 326.26 ms |
- report: строка 8: -   Read the posts JSON file.

## ekondis/mixbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ekondis/mixbench
- Категория: benchmark/testing candidate
- Описание: A GPU benchmark tool for evaluating GPUs and CPUs on mixed operational intensity kernels (CUDA, OpenCL, HIP, SYCL, OpenMP)
- Уровень: automated README evidence extraction
- Снимок: [sources/ekondis__mixbench/README.md](sources/ekondis__mixbench/README.md); SHA-256: `bf7dcd0c7f98cbb8d7b5298817e96f3bb83f0b08ecb46632401e861e4d444a19`
- gpu: строка 40: A typical execution output on an NVidia RTX-2070 GPU is:
- async: строка 2: The purpose of this benchmark tool is to evaluate performance bounds of GPUs (or CPUs) on mixed operational intensity kernels. The executed kernel is customized on a range of different operational intensity values. Modern GPUs are able to hide memory latency b
- report: строка 63: ----------------------------------------------------------------------------- CSV data -----------------------------------------------------------------------------

## davidstutz/superpixel-benchmark

- Итог: excluded — Segmentation quality evaluation
- Источник: https://github.com/davidstutz/superpixel-benchmark
- Категория: benchmark/testing candidate
- Описание: An extensive evaluation and comparison of 28 state-of-the-art superpixel algorithms on 5 datasets.
- Уровень: automated README evidence extraction
- Снимок: [sources/davidstutz__superpixel-benchmark/README.md](sources/davidstutz__superpixel-benchmark/README.md); SHA-256: `2b9880460020013d0d9695650ad61adebe8bc390707759c7e6468837d978e031`
- report: строка 162: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. You agree to cite the corresponding papers (see above) in documents and papers that report on research using the Software.

## UWMILab/HA-VLN

- Итог: excluded — Navigation task-quality benchmark
- Источник: https://github.com/UWMILab/HA-VLN
- Категория: benchmark/testing candidate
- Описание: Official implementation for "HA-VLN 2.0: An Open Benchmark and Leaderboard for Human-Aware Navigation in Discrete and Continuous Environments with Dynamic Multi-Human Interactions".
- Уровень: automated README evidence extraction
- Снимок: [sources/UWMILab__HA-VLN/README.md](sources/UWMILab__HA-VLN/README.md); SHA-256: `9292341cb73df0cfa20c730c4df14e06e594d0766309549d54bc8d35b7bb8c4a`
- comparison: строка 110: pip install -r habitat_baselines/rl/requirements.txt
- lifecycle: строка 103: python setup.py install --headless
- report: строка 169: - human_motion.json
- correctness: строка 285: HAPS 2.0 mitigates the limitations of existing human motion datasets by identifying **26 distinct regions** across **90 architectural scenes** and generating **486 human activity descriptions**, encompassing both **indoor and outdoor environments**. These desc

## JokerJohn/PALoc

- Итог: excluded — SLAM trajectory accuracy benchmark
- Источник: https://github.com/JokerJohn/PALoc
- Категория: benchmark/testing candidate
- Описание: [TMECH'2024] PALoc: Advancing SLAM Benchmarking with Prior-Assisted 6-DoF Trajectory Generation and Uncertainty Estimation
- Уровень: automated README evidence extraction
- Снимок: [sources/JokerJohn__PALoc/README.md](sources/JokerJohn__PALoc/README.md); SHA-256: `9aa1f18851e122a6eba31595633c8677f706fbb0cb981153397b091fe36cbe30`
- statistics: строка 21: - **Advanced Uncertainty Analysis**: Detailed covariance derivation within factor graphs, enabling precise uncertainty propagation and pose analysis.
- lifecycle: строка 58: | Sensor setup           | Download link                    |

## sbeamer/gapbs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/sbeamer/gapbs
- Категория: benchmark/testing candidate
- Описание: GAP Benchmark Suite
- Уровень: automated README evidence extraction
- Снимок: [sources/sbeamer__gapbs/README.md](sources/sbeamer__gapbs/README.md); SHA-256: `ae3cbf4a19667ce645939f6b64f0aeeef92ce15666ea7037a5fb231eb268cbdc`
- comparison: строка 6: This is the reference implementation for the [GAP](http://gap.cs.berkeley.edu/) [Benchmark Suite](http://gap.cs.berkeley.edu/benchmark.html). It is designed to be a portable high-performance baseline that only requires a compiler with support for C++11. It use

## XiaoMi/mobile-ai-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/XiaoMi/mobile-ai-bench
- Категория: benchmark/testing candidate
- Описание: Benchmarking Neural Network Inference on Mobile Devices
- Уровень: automated README evidence extraction
- Снимок: [sources/XiaoMi__mobile-ai-bench/README.md](sources/XiaoMi__mobile-ai-bench/README.md); SHA-256: `ab024b0543e8d1e5456ea43eee56b8ca7e8f06e72b0cde19a3d8396e5cf4a8b9`
- statistics: строка 49: **A**: Most modern Android phones use [ARM big.LITTLE](https://en.wikipedia.org/wiki/ARM_big.LITTLE) architecture which can lead to significant variance between different runs of the benchmark, we use only available big cores to reduce this variance by `taskse
- lifecycle: строка 62: | Android NDK  | [NDK installation guide](https://developer.android.com/ndk/guides/setup#install) | Required by Android build, r15c |
- gpu: строка 180: | --device_types | str  | all         | DeviceTypes(CPU/GPU/DSP/NPU), comma separated list or all. |
- report: строка 228: python report/csv_to_html.py

## UoB-HPC/BabelStream

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/UoB-HPC/BabelStream
- Категория: benchmark/testing candidate
- Описание: STREAM, for lots of devices written in many programming models
- Уровень: automated README evidence extraction
- Снимок: [sources/UoB-HPC__BabelStream/README.md](sources/UoB-HPC__BabelStream/README.md); SHA-256: `2e6a1c9ec1d9225ae2846b50d23cb362b84b932305aacea43f9e4f925157fcaa`
- memory: строка 57: * the arrays are allocated on the heap
- gpu: строка 10: Unlike other GPU memory bandwidth benchmarks this does *not* include the PCIe transfer time.

## fastvideo/gpu-camera-sample

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fastvideo/gpu-camera-sample
- Категория: benchmark/testing candidate
- Описание: Image processing software on GPU (Windows, Linux, ARM) for real time machine vision camera applications. Performance benchmarks and Glass-to-Glass time measurements. MIPI CSI cameras support. RAW2RGB processing on CUDA with 16-bit ISP. Software for Jetson.
- Уровень: automated README evidence extraction
- Снимок: [sources/fastvideo__gpu-camera-sample/README.md](sources/fastvideo__gpu-camera-sample/README.md); SHA-256: `a68eda5f76daae405c38808cf8c7162a34f2abeee5ab707b03759646ce6a34d2`
- lifecycle: строка 180: * If you need built-in MIPI cameras support on Jetson TX2 or Leopard Imaging IMX477 MIPI SCI camera support on Jetson AGX Xavier, read  [this guide](LI-IMX477-MIPI.md)  how to setup units and install required drivers.
- memory: строка 261: We also recommend to check PCI-Express bandwidth for Host-to-Device and Device-to-Host transfers. For GPU with Gen3 x16 it should be in the range of 10-12 GB/s, and for GPU with Gen4 x16 it should be in the range of 20-24 GB/s. GPU memory size could be a bottl
- gpu: строка 1: # gpu-camera-sample
- async: строка 56: | Glass-to-glass latency (XIMEA USB3, 3 MPix, 120 fps, 144 Hz monitor) | **~35–40 ms** | [FastVCR](https://www.fastcompression.com/products/fastvcr-ximea-software.htm) |
- correctness: строка 356: * [JPEG2000 benchmark: reproducible measurement of Fastvideo JPEG2000 vs nvJPEG2000 on RTX 4090](https://github.com/fastvideo/jpeg2000-benchmark) — code, raw logs and machine-readable results
- isolation: строка 211: NVIDIA Jetson TX2 has two CPU core types. These are Denver2 and A57. During benchmarking of <a href="https://www.fastcompression.com/products/sdk.htm" target="_blank">Fastvideo SDK</a> we have realized that better performance for J2K encoder and <a href="https

## Xingyu-Lin/softgym

- Итог: excluded — Robot policy manipulation environments
- Источник: https://github.com/Xingyu-Lin/softgym
- Категория: benchmark/testing candidate
- Описание: SoftGym is a set of benchmark environments for deformable object manipulation.
- Уровень: automated README evidence extraction
- Снимок: [sources/Xingyu-Lin__softgym/README.md](sources/Xingyu-Lin__softgym/README.md); SHA-256: `3e36ae2812b3c852a44802f7a02352baf35db9d735bf0c66a1229d170632d4fe`

## hpcg-benchmark/hpcg

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/hpcg-benchmark/hpcg
- Категория: benchmark/testing candidate
- Описание: Official HPCG benchmark source code
- Уровень: automated README evidence extraction
- Снимок: [sources/hpcg-benchmark__hpcg/README.md](sources/hpcg-benchmark__hpcg/README.md); SHA-256: `502c73b6436b5a0fbd89961b641c8ac77dfe91cacc8fe58cb31f4309eaca3474`
- report: строка 47: runs must be at least 1800 seconds (30 minutes) as reported in the output file.

## 1a1a11a/libCacheSim

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/1a1a11a/libCacheSim
- Категория: benchmark/testing candidate
- Описание: a high performance library for building cache simulators
- Уровень: automated README evidence extraction
- Снимок: [sources/1a1a11a__libCacheSim/README.md](sources/1a1a11a__libCacheSim/README.md); SHA-256: `2f856aede5f446ac28d7f240e01e7a9280d2b1a1618406fe32abc47795f47eb0`
- lifecycle: строка 122: <summary> Developer setup </summary>
- report: строка 176: # use a csv trace, note the quotation marks when you have multiple options

## Compaile/ctrack

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Compaile/ctrack
- Категория: benchmark/testing candidate
- Описание: A lightweight, high-performance C++ benchmarking and tracking library for effortless function profiling in both development and production environments. Features single-header integration, minimal overhead, multi-threaded support, customizable output, and advanced metrics for quick bottleneck detection in complex codebases.
- Уровень: automated README evidence extraction
- Снимок: [sources/Compaile__ctrack/README.md](sources/Compaile__ctrack/README.md); SHA-256: `1994909f2251a0868a8d2a09049591cef64fb82a00df6a26ec49532d9f567cec`
- statistics: строка 134: - **time [x-y]**: Shows event times within specified percentile ranges (default [0-100] and [1-99]) to exclude outliers.
- comparison: строка 370: - **Comprehensive Benchmarking Suite**: Added complete benchmark framework with baseline comparison capabilities for tracking performance across releases
- lifecycle: строка 5: CTRACK is a powerful tool that can be seamlessly integrated into both development and production environments. It allows developers to effortlessly monitor applications and identify bottlenecks, requiring minimal setup and maintenance.
- memory: строка 379: - Reduced memory usage by avoiding nested maps in event storage
- async: строка 350: 9. **Instant Bottleneck Detection**: CTRACK's unique "time active" and "time active exclusive" metrics allow developers to instantly spot bottlenecks, even in complex multithreaded codebases. This feature sets CTRACK apart from other tools that struggle to pro
- report: строка 16: - Multiple output formats (stdout(with color), string, and more soon JSON export, SQL)
- correctness: строка 428: - Validating optimization improvements

## ProjectPhysX/OpenCL-Benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ProjectPhysX/OpenCL-Benchmark
- Категория: benchmark/testing candidate
- Описание: A small OpenCL benchmark program to measure peak GPU/CPU performance.
- Уровень: automated README evidence extraction
- Снимок: [sources/ProjectPhysX__OpenCL-Benchmark/README.md](sources/ProjectPhysX__OpenCL-Benchmark/README.md); SHA-256: `1c699fa75260c175fc4099c1add5753fd4cf64e85ee096967a0413cabf6a69e7`
- gpu: строка 3: A small [OpenCL](https://github.com/ProjectPhysX/OpenCL-Wrapper "OpenCL-Wrapper") benchmark program to measure peak GPU/CPU performance.
- report: строка 19: - closest possible fraction/multiplicator of `measured compute performance` divided by `reported theoretical FP32 performance` is shown in `(round brackets)`

## martinus/map_benchmark

- Итог: excluded — README redirects to article; article not analysed in this corpus
- Источник: https://github.com/martinus/map_benchmark
- Категория: benchmark/testing candidate
- Описание: Comprehensive benchmarks of C++ maps
- Уровень: automated README evidence extraction
- Снимок: [sources/martinus__map_benchmark/README.md](sources/martinus__map_benchmark/README.md); SHA-256: `0cf85003db14c162ded87a0bfe665bb25b5e057a44d209e1bd0d3ee3ae512ec1`

## apigee/apib

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/apigee/apib
- Категория: benchmark/testing candidate
- Описание: A simple, fast HTTP and API benchmarking tool
- Уровень: automated README evidence extraction
- Снимок: [sources/apigee__apib/README.md](sources/apigee__apib/README.md); SHA-256: `1394cce02c79a10f7da1a1debad66f4184da86f714f9eb9696fc91036c64f6e5`
- memory: строка 60: 2. Allocate a subset of connections to each, and in each:
- async: строка 22: 60 seconds using 100 concurrent network connections.
- report: строка 66: 8. Back in main thread, report on shared results periodically

## learnedsystems/SOSD

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/learnedsystems/SOSD
- Категория: benchmark/testing candidate
- Описание: A Benchmark for Learned Indexes
- Уровень: automated README evidence extraction
- Снимок: [sources/learnedsystems__SOSD/README.md](sources/learnedsystems__SOSD/README.md); SHA-256: `13c16664c895b4d2da1cf803cdb5211ddf98cad7b8a805f8c34f91d5d15ad5e1`
- comparison: строка 16: It comes with state-of-the-art baseline implementations to compare against and many datasets to compare on.
- report: строка 40: - `./scripts/execute.sh` executes the benchmark on each workload, storing the results in `results`. You can use the `-c` flag to output a .csv file of results rather than a .txt.

## FrontierCS/Frontier-CS

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/FrontierCS/Frontier-CS
- Категория: benchmark/testing candidate
- Описание: A benchmark for evaluating LLMs on open-ended CS problems. Exploring the Next Frontier of Computer Science.
- Уровень: automated README evidence extraction
- Снимок: [sources/FrontierCS__Frontier-CS/README.md](sources/FrontierCS__Frontier-CS/README.md); SHA-256: `364d91e19be07a9f73cd61adfb66f70b9b5164f0cb58bf22fe725f8acd018e52`
- lifecycle: строка 179: # Use Docker instead (no cloud setup needed)
- report: строка 98: `result.json` and `verifier/reward.json` artifacts.

## ORNL/HeCBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ORNL/HeCBench
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/ORNL__HeCBench/README.md](sources/ORNL__HeCBench/README.md); SHA-256: `79c470e47b69926fed19151e95e825491237f84b00368676fa3f8737b15ef03e`
- statistics: строка 840: Divide channels into groups and computes mean/variance for normalization within each group (https://github.com/clu0/unet.cu)
- comparison: строка 85: adjacent, aligned-types, asta, asyncAllocation, awbarrier, blockAccess, blockexchange, blockScan, collision, concurrentKernels, conversion, dispatch, dp4a, graphExecution, ert, interleave, intrinsics-cast, kernelLaunch, layout, mallocFree, maxFlops, mixbench,
- lifecycle: строка 182: By default it will run a warmup iteration before running each benchmark,
- memory: строка 85: adjacent, aligned-types, asta, asyncAllocation, awbarrier, blockAccess, blockexchange, blockScan, collision, concurrentKernels, conversion, dispatch, dp4a, graphExecution, ert, interleave, intrinsics-cast, kernelLaunch, layout, mallocFree, maxFlops, mixbench,
- gpu: строка 13: Certain benchmarks require [Boost](https://www.boost.org/releases/latest/), [Eigen](https://eigen.tuxfamily.org), [GDAL](https://github.com/OSGeo/gdal), GPU-aware Message Passing Interface(MPI) or vendors' collective communication libraries (e.g. NCCL).<br>
- async: строка 85: adjacent, aligned-types, asta, asyncAllocation, awbarrier, blockAccess, blockexchange, blockScan, collision, concurrentKernels, conversion, dispatch, dp4a, graphExecution, ert, interleave, intrinsics-cast, kernelLaunch, layout, mallocFree, maxFlops, mixbench,
- report: строка 145: It works with a `.json` file containing the benchmark names, a regex to
- correctness: строка 1449: Reproducible floating sum (https://github.com/facebookarchive/fbcuda)

## VITA-Group/Open-L2O

- Итог: excluded — Learned optimizer quality benchmark
- Источник: https://github.com/VITA-Group/Open-L2O
- Категория: benchmark/testing candidate
- Описание: Open-L2O: A Comprehensive and Reproducible Benchmark for Learning to Optimize Algorithms
- Уровень: automated README evidence extraction
- Снимок: [sources/VITA-Group__Open-L2O/README.md](sources/VITA-Group__Open-L2O/README.md); SHA-256: `4026fb1afb4b7e2f8db83ad28baba5ecd73a78ec75aa5523267fbdf10f99f4d3`
- comparison: строка 63: 4. L2O-enhanced from *Training Stronger Baselines for Learning to Optimize* [[Paper](https://arxiv.org/pdf/2010.09089.pdf)] [[Code](https://github.com/Tianlong-Chen/Awesome-L2O/blob/main/Model_Free_L2O/L2O-DM%20and%20L2O-RNNProp/README.md)]
- correctness: строка 3: This repository establishes the first comprehensive benchmark efforts of existing learning to optimize (L2O) approaches on a number of problems and settings. We release our software implementation and data as the Open-L2O package, for reproducible research and

## rcedgar/muscle

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/rcedgar/muscle
- Категория: benchmark/testing candidate
- Описание: Multiple sequence and structure alignment with top benchmark scores scalable to thousands of sequences. Generates replicate alignments, enabling assessment of downstream analyses such as trees and predicted structures.
- Уровень: automated README evidence extraction
- Снимок: [sources/rcedgar__muscle/README.md](sources/rcedgar__muscle/README.md); SHA-256: `02565e42718e20378b57da10e7a9e65ae96e6721e086d904337ab0d589b70188`

## breuner/elbencho

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/breuner/elbencho
- Категория: benchmark/testing candidate
- Описание: A distributed storage benchmark for file systems, object stores & block devices with support for GPUs
- Уровень: automated README evidence extraction
- Снимок: [sources/breuner__elbencho/README.md](sources/breuner__elbencho/README.md); SHA-256: `46fd91d1be3b1c067921c6d33e7815801e22924349ed85d03759e118508f9701`
- memory: строка 112: If your cluster is using Slurm to allocate nodes, you can find examples [here](docs/slurm-examples.md). Kubernetes/K8s examples are available [here](docs/k8s-examples.md).
- gpu: строка 36: * GPU storage access performance testing through Nvidia CUDA or GPUDirect Storage (GDS)
- async: строка 33: * Unified latency, throughput, IOPS benchmark for file, object & block storage
- report: строка 40: * CSV file output to easily create graphs in spreadsheet apps or via elbencho-chart tool

## t-dillon/tdoku

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/t-dillon/tdoku
- Категория: benchmark/testing candidate
- Описание: A fast Sudoku solver and generator with a benchmark suite for comparing the fastest known solvers.
- Уровень: automated README evidence extraction
- Снимок: [sources/t-dillon__tdoku/README.md](sources/t-dillon__tdoku/README.md); SHA-256: `2a50d600b796b280f61954b925023feba8b94ec5586d980a5ac1587acf070348`
- lifecycle: строка 114: other/setup_jczsolve.sh
- report: строка 136: -c [0|1]            // output csv instead of table [default 0]
- correctness: строка 144: -v [0|1]            // validate during warmup [default 1]

## davidstutz/superpixels-revisited

- Итог: excluded — Segmentation quality evaluation
- Источник: https://github.com/davidstutz/superpixels-revisited
- Категория: benchmark/testing candidate
- Описание: Library containing 7 state-of-the-art superpixel algorithms with a total of 9 implementations used for evaluation purposes in [1] utilizing an extended version of the Berkeley Segmentation Benchmark.
- Уровень: automated README evidence extraction
- Снимок: [sources/davidstutz__superpixels-revisited/README.md](sources/davidstutz__superpixels-revisited/README.md); SHA-256: `b10fc35d13e6465e938de89f991c962d881b162f0be45ce1888291bdb48d1bf6`
- report: строка 57: Technical report, École Polytechnique Fédérale de Lausanne, 2010.

## jwlim/tracker_benchmark

- Итог: excluded — Visual tracking accuracy benchmark
- Источник: https://github.com/jwlim/tracker_benchmark
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/jwlim__tracker_benchmark/readme.md](sources/jwlim__tracker_benchmark/readme.md); SHA-256: `f9d1e709caf4fef216198728ff4e67d36d3afb5135ec54ab7ee3a206d34a6b75`

## google/uVkCompute

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/google/uVkCompute
- Категория: benchmark/testing candidate
- Описание: A micro Vulkan compute pipeline and a collection of benchmarking compute shaders
- Уровень: automated README evidence extraction
- Снимок: [sources/google__uVkCompute/README.md](sources/google__uVkCompute/README.md); SHA-256: `42086d9df8b0c217f457df7f1797ddb1edda12cb145b6c53b7563603515e98e5`
- memory: строка 45: the system allocator and allocates separate memory for each buffer. Simplicity
- gpu: строка 16: µVkCompute is a micro Vulkan compute pipeline and a collection of compute

## miloyip/dtoa-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/miloyip/dtoa-benchmark
- Категория: benchmark/testing candidate
- Описание: C++ double-to-string conversion benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/miloyip__dtoa-benchmark/readme.md](sources/miloyip__dtoa-benchmark/readme.md); SHA-256: `754e614a6c9c916075bab5bc81e0eba96309be9e34c07d147e02eaa60a960662`
- memory: строка 98: It may introduce heap allocation, which is a big overhead. User can easily wrap these low-level functions to return `std::string`, if needed.
- report: строка 35: 7. The results in CSV format will be written to `dtoa-benchmark/result`.
- correctness: строка 19: Firstly the program verifies the correctness of implementations.

## zhouxian/FluidLab

- Итог: excluded — Differentiable manipulation environments
- Источник: https://github.com/zhouxian/FluidLab
- Категория: benchmark/testing candidate
- Описание: [ICLR 2023] FluidLab: A Differentiable Environment for Benchmarking Complex Fluid Manipulation
- Уровень: automated README evidence extraction
- Снимок: [sources/zhouxian__FluidLab/README.md](sources/zhouxian__FluidLab/README.md); SHA-256: `72b05e8237c59feff78337ce2404b1bcd0f31852898f9e84e3ce690d0eec229f`
- gpu: строка 60: GLRenderer (`fluidlab/fluidengine/renderers/ggui_renderer.py`) is an OpenGL-based GPU-acelerated rendering pipeline. It produces much better visual effects.

## vkmark/vkmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/vkmark/vkmark
- Категория: benchmark/testing candidate
- Описание: Vulkan benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/vkmark__vkmark/README.md](sources/vkmark__vkmark/README.md); SHA-256: `49ff14bb8bb1a91349a765661359e9695182903b6121c3255e4140d955384f50`
- comparison: строка 92: 'interleave' set to 'false':
- gpu: строка 3: vkmark is an extensible Vulkan benchmarking suite with targeted,

## cmuratori/termbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cmuratori/termbench
- Категория: benchmark/testing candidate
- Описание: Simple benchmark for terminal output
- Уровень: automated README evidence extraction
- Снимок: [sources/cmuratori__termbench/README.md](sources/cmuratori__termbench/README.md); SHA-256: `e719f5d3a1d6e8930f3453ef83502c17af4de0728c3bd7433f5ccecbde8962cc`
- async: строка 31: On modern Windows machines with memory bandwidth in the 10-20gb/s range, the expected throughput for these tests would be in the 0.5-2.0gb/s range for a reasonable terminal.  Numbers significantly higher than that might indicate a well-optimized terminal, and

## johnmcfarlane/fixed_point

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/johnmcfarlane/fixed_point
- Категория: benchmark/testing candidate
- Описание: C++ Binary Fixed-Point Arithmetic
- Уровень: automated README evidence extraction
- Снимок: [sources/johnmcfarlane__fixed_point/README.md](sources/johnmcfarlane__fixed_point/README.md); SHA-256: `7b0c901adbbae8ef4e66363b4c2bbd34fd9213ebaba3f7fd9882a4b3e0a9e646`
- report: строка 99: $ perf report -g 'graph,0.5,caller'

## SJTU-IPADS/ServerlessBench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/SJTU-IPADS/ServerlessBench
- Категория: benchmark/testing candidate
- Описание: A benchmark suite for serverless computing
- Уровень: automated README evidence extraction
- Снимок: [sources/SJTU-IPADS__ServerlessBench/README.md](sources/SJTU-IPADS__ServerlessBench/README.md); SHA-256: `d143f98800ba121b5e337afe9b12ec7387aa0a7f7de9ea845ea798ada68851f6`
- async: строка 41: * instance-parallel version (parallel.py, doAlu.py), with multiple function instances executing concurrently; You need to modify the AluFunctionArn to the arn of the alu function (doAlu.py)
- report: строка 14: * Monthly reports: https://serverlessbench.systems/en-us/docs/report

## leggedrobotics/SimBenchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/leggedrobotics/SimBenchmark
- Категория: benchmark/testing candidate
- Описание: Physics engine benchmark for robotics applications: RaiSim vs Bullet vs ODE vs MuJoCo vs DartSim
- Уровень: automated README evidence extraction
- Снимок: [sources/leggedrobotics__SimBenchmark/README.md](sources/leggedrobotics__SimBenchmark/README.md); SHA-256: `ee4d45cefb53ade6fd3dc4df84743dd531cdf69a3a874af9d579bd7b4af1994c`

## ParAlg/gbbs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ParAlg/gbbs
- Категория: benchmark/testing candidate
- Описание: GBBS: Graph Based Benchmark Suite
- Уровень: automated README evidence extraction
- Снимок: [sources/ParAlg__gbbs/README.md](sources/ParAlg__gbbs/README.md); SHA-256: `4c3d421ff6e2bd838851c294bc8d4e305194394633172361fe3bbc5617e31524`

## Aider-AI/polyglot-benchmark

- Итог: excluded — Programming problem success evaluation
- Источник: https://github.com/Aider-AI/polyglot-benchmark
- Категория: benchmark/testing candidate
- Описание: Coding problems used in aider's polyglot benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/Aider-AI__polyglot-benchmark/README.md](sources/Aider-AI__polyglot-benchmark/README.md); SHA-256: `9a06b876910a1566e5152658213d70e62922642513c7dfeb04bed868d510856e`

## DBAIWangGroup/nns_benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/DBAIWangGroup/nns_benchmark
- Категория: benchmark/testing candidate
- Описание: Benchmark of Nearest Neighbor Search on High Dimensional Data
- Уровень: automated README evidence extraction
- Снимок: [sources/DBAIWangGroup__nns_benchmark/README.md](sources/DBAIWangGroup__nns_benchmark/README.md); SHA-256: `24b7736e3c17df2a2645f7c7ecd40d684a716c14fa8b4b942020b84719b63d8c`
- lifecycle: строка 5: To aid researchers and practitioners working on or whose work depends on the problem, we setup this benchmark for Nearest Neighbor Search (NNS) based on the Euclidean distance on High Dimensional Data. The benefit is twofold:
- report: строка 26: Currently, we evaluate **15 representative NNS algorithms** on **20 datasets** where details are reported in our [Nearest Neighbor Search (NNS) Experimental Evaluation Paper](https://arxiv.org/abs/1610.02455)[1].

## url-kaist/Ground-Segmentation-Benchmark

- Итог: excluded — Ground segmentation quality benchmark
- Источник: https://github.com/url-kaist/Ground-Segmentation-Benchmark
- Категория: benchmark/testing candidate
- Описание: Ground segmentation benchmark in SemanticKITTI dataset
- Уровень: automated README evidence extraction
- Снимок: [sources/url-kaist__Ground-Segmentation-Benchmark/README.md](sources/url-kaist__Ground-Segmentation-Benchmark/README.md); SHA-256: `3a26edbea70bede042609b46530af886af1ec7e3452bd085d1bb06a0c5778804`
- comparison: строка 4: This repository contains various Ground Segmentation baseline methods. Currently, 7 projects are organized for *SemanticKITTI dataset*:
- lifecycle: строка 114: 1. Download [SemanticKITTI](http://www.semantic-kitti.org/dataset.html#download) Odometry dataset including Velodyne point clouds, calibration data, and label data.
- report: строка 65: * The benchmark calculates the performance of each method and save the results as *csv* files.

## openxrlab/xrsfm

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/openxrlab/xrsfm
- Категория: benchmark/testing candidate
- Описание: OpenXRLab Structure-from-Motion Toolbox and Benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/openxrlab__xrsfm/README.md](sources/openxrlab__xrsfm/README.md); SHA-256: `93d42314df2e144cf813d24f205dc1445e46f0ba85ec04bc54814342cc15079d`

## rigtorp/c2clat

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/rigtorp/c2clat
- Категория: benchmark/testing candidate
- Описание: A tool to measure CPU core to core latency
- Уровень: automated README evidence extraction
- Снимок: [sources/rigtorp__c2clat/README.md](sources/rigtorp__c2clat/README.md); SHA-256: `1ed96a51e40bb2bbcc7f51ee7918b8260327e2889d70fb26790b5b50f00dc3b6`
- async: строка 3: A tool to measure CPU core to core latency (inter-core latency).

## neurosim/DNN_NeuroSim_V2.1

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/neurosim/DNN_NeuroSim_V2.1
- Категория: benchmark/testing candidate
- Описание: Benchmark framework of compute-in-memory based accelerators for deep neural network (on-chip training chip focused)
- Уровень: automated README evidence extraction
- Снимок: [sources/neurosim__DNN_NeuroSim_V2.1/README.md](sources/neurosim__DNN_NeuroSim_V2.1/README.md); SHA-256: `9602c1259a2654258b5eb5c275e31fda9c837d04aab92c17e082b95df0df22c7`
- lifecycle: строка 9: 1. Calibrate FinFET technology library: temperature-related features.
- report: строка 73: - Input activity of every layer for each epoch: `input_activity.csv`

## ETH3D/dataset-pipeline

- Итог: excluded — Image reconstruction dataset preparation
- Источник: https://github.com/ETH3D/dataset-pipeline
- Категория: benchmark/testing candidate
- Описание: Pipeline for creating multi-view benchmark datasets from laser scans and images.
- Уровень: automated README evidence extraction
- Снимок: [sources/ETH3D__dataset-pipeline/README.md](sources/ETH3D__dataset-pipeline/README.md); SHA-256: `e1508b06d9c48cffabeb94611bf410c61794aa05afcefb9da782cc08618b699a`
- statistics: строка 13: This includes tools for laser scan processing (outlier removal, scan alignment, ...) and image alignment wrt. laser scans (by optimizing for color consistency among images and the scans).
- lifecycle: строка 80: * The `dslr_calibration_jpg` and `occlusion` folders will not be used, since this data is generated by the pipeline.
- report: строка 201: In the case of using a camera rig, the rig configuration must be given as a file `rigs.json` in the format used by COLMAP's rig bundle adjuster (with the difference that the `image_prefix` must be the folder name which contains a rig camera's images and cannot
- correctness: строка 814: remains as long as it is not invalidated by a change and as long as the same

## serde-rs/json-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/serde-rs/json-benchmark
- Категория: benchmark/testing candidate
- Описание: nativejson-benchmark in Rust
- Уровень: automated README evidence extraction
- Снимок: [sources/serde-rs__json-benchmark/README.md](sources/serde-rs__json-benchmark/README.md); SHA-256: `c3534944ecb50a2d0a25e0b5d7c3425d1244c6d551ca5e7e4d1998f9be792c0b`
- report: строка 1: # Rust JSON Benchmark

## cornell-zhang/rosetta

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cornell-zhang/rosetta
- Категория: benchmark/testing candidate
- Описание: Rosetta: A Realistic High-level Synthesis Benchmark Suite for Software Programmable FPGAs (FPGA'18)
- Уровень: automated README evidence extraction
- Снимок: [sources/cornell-zhang__rosetta/README.md](sources/cornell-zhang__rosetta/README.md); SHA-256: `71c27b0b8ee3b62464251864a3725370cf326e67eae1393a9caefa8831f742c9`
- lifecycle: строка 109: 7. To run simulation, please run `make emu_setup OCL_PLATFORM=<path_to_custom_platform_xfpm_file>`
- async: строка 37: | Benchmark | #LUTs | #FFs | #BRAMs | #DSPs | Runtime (ms) | Throughput |
- report: строка 96: The report `system_estimate.xtxt` shows latency and resource estimate after high-level synthesis.

## project-gemmi/benchmarking-fft

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/project-gemmi/benchmarking-fft
- Категория: benchmark/testing candidate
- Описание: choosing FFT library...
- Уровень: automated README evidence extraction
- Снимок: [sources/project-gemmi__benchmarking-fft/README.md](sources/project-gemmi__benchmarking-fft/README.md); SHA-256: `f239d922807a6d03eca323ce3ff9fdf9f50cb93899a56ae390a993b1ec96902f`
- lifecycle: строка 190: **plan / setup** (`plan1d.cpp`)
- gpu: строка 48: I don't plan to use GPU for computations, so I won't try
- report: строка 28: [FFTE](http://www.ffte.jp/) (East) are reported to be faster than FFTW,

## learn-more/findpattern-bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/learn-more/findpattern-bench
- Категория: benchmark/testing candidate
- Описание: Simple benchmark for findpattern implementations.
- Уровень: automated README evidence extraction
- Снимок: [sources/learn-more__findpattern-bench/README.md](sources/learn-more__findpattern-bench/README.md); SHA-256: `0174bb39e93f14e4eb403a44a11c21784a641d4c57a86f80b60a361a08ab2bfe`

## MCG-NKU/SalBenchmark

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/MCG-NKU/SalBenchmark
- Категория: benchmark/testing candidate
- Описание: Salient Object Detection: A Benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/MCG-NKU__SalBenchmark/README.md](sources/MCG-NKU__SalBenchmark/README.md); SHA-256: `ef5de0e7ea432be9f887c81f77c1044e2cacf72ba6f09b35e0bc36d6ea0291d4`
- comparison: строка 5: The Saliency Benchmark compare, qualitatively and quantitatively, 42 state-of-the-art models(30 salient object detection, 10 fixation prediction, 1 objectness, and 1 baseline) over 6 challenging datasets for the purpose of benchmarking salient object detection

## mattreecebentley/plf_nanotimer

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mattreecebentley/plf_nanotimer
- Категория: benchmark/testing candidate
- Описание: A simple C++ 03/11/etc timer class for ~microsecond-precision cross-platform benchmarking. The implementation is as limited and as simple as possible to create the lowest amount of overhead.
- Уровень: automated README evidence extraction
- Снимок: [sources/mattreecebentley__plf_nanotimer/README.md](sources/mattreecebentley__plf_nanotimer/README.md); SHA-256: `322131dab154137ae1ae63dac21539bed2bd095cbb4c659c8517dd06befcae41`

## ivafanas/sltbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ivafanas/sltbench
- Категория: benchmark/testing candidate
- Описание: C++ benchmark tool. Practical, stable and fast performance testing framework.
- Уровень: automated README evidence extraction
- Снимок: [sources/ivafanas__sltbench/README.md](sources/ivafanas__sltbench/README.md); SHA-256: `516517c732ce74e76f3fa3c9c061fb676829f3b5d98f4ec6f89b4b8060c8d5d2`
- comparison: строка 45: 4.7x times speedup might be useful for projects with big performance tests count. For the original project regression performance testing tooks about a week and testing time reduction matters.
- correctness: строка 8: - stable - correctness and reproducibility is a goal

## ArashPartow/math-parser-benchmark-project

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ArashPartow/math-parser-benchmark-project
- Категория: benchmark/testing candidate
- Описание: C++ Mathematical Expression Parser Benchmark
- Уровень: automated README evidence extraction
- Снимок: [sources/ArashPartow__math-parser-benchmark-project/readme.md](sources/ArashPartow__math-parser-benchmark-project/readme.md); SHA-256: `b80b5e315454037ecbcfd58ff25aa5a09bd49c7f9a82c3272297f0deb433fb8f`
- lifecycle: строка 44: ## The Setup
- memory: строка 97: [03] muparserSSE          ( 40.698 ns, 3.744852304458618164, -15768715.739250183105468750)
- correctness: строка 39: 1. Correctness and precision of results relative to the *floating point type* used

## TheLartians/EasyIterator

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/TheLartians/EasyIterator
- Категория: benchmark/testing candidate
- Описание: 🏃 Iterators made easy! Zero cost abstractions for designing and using C++ iterators.
- Уровень: automated README evidence extraction
- Снимок: [sources/TheLartians__EasyIterator/README.md](sources/TheLartians__EasyIterator/README.md); SHA-256: `31c9566050991cefa1f2e297588536aa623733190c13b6a7b86241ecd956dce9`

## qi7chen/timer-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/qi7chen/timer-benchmarks
- Категория: benchmark/testing candidate
- Описание: Benchmark of different timer implementations(min-heap, red-black tree, timing wheel) 不同数据结构实现的定时器测试
- Уровень: automated README evidence extraction
- Снимок: [sources/qi7chen__timer-benchmarks/README.md](sources/qi7chen__timer-benchmarks/README.md); SHA-256: `1e69f99730a801b7b95dff038ed655311daca7eadd78486c6d52a6a26dbe8dd8`
- memory: строка 22: use [min-heap](https://en.wikipedia.org/wiki/Heap_(data_structure)), quaternary heap( [4-ary heap](https://en.wikipedia.org/wiki/D-ary_heap) ),

## google/fleetbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/google/fleetbench
- Категория: benchmark/testing candidate
- Описание: Benchmarking suite for Google workloads
- Уровень: automated README evidence extraction
- Снимок: [sources/google__fleetbench/README.md](sources/google__fleetbench/README.md); SHA-256: `b5a7b6fa74dc9d925a5e0915461e9f5b29ebade482e7748ed2af919cf6e1fd2f`
- statistics: строка 239: ### Reducing run-to-run variance
- comparison: строка 279: contender vs baseline?
- lifecycle: строка 94: ### Setup
- memory: строка 197: TCMalloc is the underlying memory allocator in this benchmark suite. By default
- async: строка 26: accurately characterize system performance under realistic, concurrent
- report: строка 190: Some benchmarks also provide counter reports after completion. Adding
- isolation: строка 255: *   Bind the process to a core by setting its affinity

## NoAvailableAlias/signal-slot-benchmarks

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/NoAvailableAlias/signal-slot-benchmarks
- Категория: benchmark/testing candidate
- Описание: Comprehensive benchmarks for a majority of GitHub c++ signal slot implementations and others.
- Уровень: automated README evidence extraction
- Снимок: [sources/NoAvailableAlias__signal-slot-benchmarks/README.md](sources/NoAvailableAlias__signal-slot-benchmarks/README.md); SHA-256: `36daa788421b61037af8816115c120065d0096831b86b84aed6159c46c153391`
- correctness: строка 50: | [validation_assert](benchmark.hpp#L19) | Make sure each signal implementation is functioning correctly. |

## cksystemsgroup/scal

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/cksystemsgroup/scal
- Категория: benchmark/testing candidate
- Описание: High-performance multicore-scalable data structures and benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/cksystemsgroup__scal/README.md](sources/cksystemsgroup__scal/README.md); SHA-256: `e02db440a385cacf0fe3791d9ba98ba0b137110b893e01ff4c5cfc921c98146c`
- async: строка 4: Scal is an open-source benchmarking framework that provides (1) software infrastructure for executing concurrent data structure algorithms, (2) workloads for benchmarking their performance and scalability, and (3) implementations of a large set of concurrent d
- report: строка 125: 12. <a name="ref-treiber-1986"></a>R.K. Treiber. Systems programming: Coping with parallelism. Technical Report RJ-5118, IBM Research Center, 1986.

## mmperf/mmperf

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/mmperf/mmperf
- Категория: benchmark/testing candidate
- Описание: MatMul Performance Benchmarks for a Single CPU Core comparing both hand engineered and codegen kernels.
- Уровень: automated README evidence extraction
- Снимок: [sources/mmperf__mmperf/README.md](sources/mmperf__mmperf/README.md); SHA-256: `13df9097b2866e651a7e5fcf3900d36a8ed24e4c681e4e3181145ff6b6984f6b`
- gpu: строка 82: ### Building specified backends on GPU

## lemire/StronglyUniversalStringHashing

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/lemire/StronglyUniversalStringHashing
- Категория: benchmark/testing candidate
- Описание: Benchmark showing the we can randomly hash strings very quickly with good universality
- Уровень: automated README evidence extraction
- Снимок: [sources/lemire__StronglyUniversalStringHashing/README.md](sources/lemire__StronglyUniversalStringHashing/README.md); SHA-256: `4cc06761b7501a6fe2a2aff830fa12837ae19fe4c5a0205179bb182c22255e54`
- correctness: строка 42: To test correctness of hash functions using PCLMULQDQ:

## Geekgineer/motcpp

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/Geekgineer/motcpp
- Категория: benchmark/testing candidate
- Описание: A blazing-fast, modern C++ multi-object tracking library featuring SORT, ByteTrack, OC-SORT, BoostTrack, and more built for real-time performance, cross-platform deployment, and MOT benchmarks.
- Уровень: automated README evidence extraction
- Снимок: [sources/Geekgineer__motcpp/README.md](sources/Geekgineer__motcpp/README.md); SHA-256: `bf2ce5aa68c714e361059a0fe7f546716255e4a1cbcc83fbda34581ca2842f0a`
- comparison: строка 58: **motcpp** is a high-performance C++ library for multi-object tracking (MOT). It implements 10 state-of-the-art algorithms with a unified, modern C++17 API — covering everything from the lightweight SORT baseline to the current BoostTrack SOTA — all ready to d
- async: строка 259: Need maximum throughput (>500 FPS)?

## mattiaspaul/deedsBCV

- Итог: excluded — Search false positive or domain outside Rust performance harness requirements.
- Источник: https://github.com/mattiaspaul/deedsBCV
- Категория: benchmark/testing candidate
- Описание: new and more efficient version for 3D discrete deformable registration, reaching the highest accuracy in several benchmarks
- Уровень: automated README evidence extraction
- Снимок: [sources/mattiaspaul__deedsBCV/README.md](sources/mattiaspaul__deedsBCV/README.md); SHA-256: `d0c844459e2d69f17d62ebb09c1de67de11862e3548c3f901577c376e822864d`
- correctness: строка 199: It is best to now check visually (e.g. with ITK Snap), whether all scans and segmentations are processed correctly. You can also validate this by loading the two manual kidney segmentations provided by myself in /segment. Afterwards, simply run the linear and

## RALC88/riscv-vectorized-benchmark-suite

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/RALC88/riscv-vectorized-benchmark-suite
- Категория: benchmark/testing candidate
- Описание: RiVEC Bencmark Suite
- Уровень: automated README evidence extraction
- Снимок: [sources/RALC88__riscv-vectorized-benchmark-suite/README.md](sources/RALC88__riscv-vectorized-benchmark-suite/README.md); SHA-256: `da344da111e8094cdc983fb293caa341156147e32ddef482a9bf16dd70321c2c`
- gpu: строка 39: ## Building the rv64gcv baremetal toolchain with LLVM

## tuxalin/THST

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tuxalin/THST
- Категория: benchmark/testing candidate
- Описание: Templated hierarchical spatial trees designed for high-peformance.
- Уровень: automated README evidence extraction
- Снимок: [sources/tuxalin__THST/README.md](sources/tuxalin__THST/README.md); SHA-256: `b83d8c846e440e86ec0b05e23797927e75c1dd07949dc6e5c0773d3e701728ec`
- lifecycle: строка 131: Benchmark setup is based on [spatial_index_benchmark](https://github.com/mloskot/spatial_index_benchmark) by Mateusz Loskot and Adam Wulkiewicz.
- memory: строка 20: - support for custom allocators for internal nodes
- report: строка 201: Bug reports and pull requests are welcome on GitHub at https://github.com/tuxalin/thst.

## guteksan/REST-CPP-benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/guteksan/REST-CPP-benchmark
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/guteksan__REST-CPP-benchmark/README.md](sources/guteksan__REST-CPP-benchmark/README.md); SHA-256: `cde85b0a6c1eae08bec0f7ae6761c9481dee845a2464e8a931344fd375bf3fd6`
- async: строка 24: * C=28 (corresponding to the value of ```hardware_concurrency()``` on the machine used to run tests.
- report: строка 16: * ```/test``` - handles POST message (JSON) and sends it back with additional text and status code 200.

## ttvd/spatial-collision-datastructures

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/ttvd/spatial-collision-datastructures
- Категория: benchmark/testing candidate
- Описание: Benchmark of various spatial data structures for collision detection.
- Уровень: automated README evidence extraction
- Снимок: [sources/ttvd__spatial-collision-datastructures/README.md](sources/ttvd__spatial-collision-datastructures/README.md); SHA-256: `14030d287bb36cefe454948be23b018296111e2f79fa4dbb08b9936fabd8fc4b`

## fraillt/cpp_serializers_benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/fraillt/cpp_serializers_benchmark
- Категория: benchmark/testing candidate
- Описание: C++ serializers benchmark with realistic data
- Уровень: automated README evidence extraction
- Снимок: [sources/fraillt__cpp_serializers_benchmark/README.md](sources/fraillt__cpp_serializers_benchmark/README.md); SHA-256: `7b5723dfef8aedc672f3255c3f2bdacc1c382e4e9efb2013fd6daf4054a1f416`
- lifecycle: строка 139: * warmup step, in which serialization and deserialization is run 5 times, to warmup cpu cache and check if deserialized data equals to original data.
- memory: строка 74: 7. check buffer size on reading, but writing buffer is preallocated std::array&lt;uint8\_t, 1000000&gt;

## emilk/ram_bench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/emilk/ram_bench
- Категория: benchmark/testing candidate
- Описание: A benchmark for random memory accesses
- Уровень: automated README evidence extraction
- Снимок: [sources/emilk__ram_bench/README.md](sources/emilk__ram_bench/README.md); SHA-256: `1ea0a0c9dec581c05989bd5e55a53bd5ed51fa100f50c86e462a367c0384b8ea`
- async: строка 33: Now you may be thinking that this is all trivial. Surely I (or someone richer than me) could purchase enough fast L1 type memory to fit all the data, and that would yield a flat graph. Sadly, 6 GiB of L1 memory would be way more that could fit on a CPU die, so

## 4ku/Place-recognition-evaluation

- Итог: excluded — Place recognition quality evaluation
- Источник: https://github.com/4ku/Place-recognition-evaluation
- Категория: benchmark/testing candidate
- Описание: Benchmarking and evaluation framework for place recognition methods, featuring SuperPoint+SuperGlue, LoGG3D-Net, Scan Context, DBoW2, MixVPR, STD
- Уровень: automated README evidence extraction
- Снимок: [sources/4ku__Place-recognition-evaluation/README.md](sources/4ku__Place-recognition-evaluation/README.md); SHA-256: `96a6dc427329bec7edddf06b8c9d66ed0e6be0ee7a8b9b61aa541cb896abc381`
- lifecycle: строка 6: - [LoGG3D-Net Setup](#logg3d-net-setup)
- gpu: строка 385: | SuperPoint + SuperGlue| CPU + GPU          | 359.36             |
- correctness: строка 183: It's recommended to read rosbag internally to avoid any delays. To do so, set `use_rosbag` to `true` in the `base.launch` file. If you do evaluation on rosbag file, this is the best option to make results reproducible.

## NKU-MobFly-Robotics/local-planning-benchmark

- Итог: excluded — Navigation algorithm evaluation outside current scope
- Источник: https://github.com/NKU-MobFly-Robotics/local-planning-benchmark
- Категория: benchmark/testing candidate
- Описание: [ICRA2021] A unified benchmark for the evaluation of mobile robot local planning approaches
- Уровень: automated README evidence extraction
- Снимок: [sources/NKU-MobFly-Robotics__local-planning-benchmark/README.md](sources/NKU-MobFly-Robotics__local-planning-benchmark/README.md); SHA-256: `47fd47b76c4b3f974e0969e760aef53870eb51ec755ca931432da35aa97e58e8`
- lifecycle: строка 20: * [Setup](#2-Setup)

## HPMLL/NVIDIA-Hopper-Benchmark

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/HPMLL/NVIDIA-Hopper-Benchmark
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/HPMLL__NVIDIA-Hopper-Benchmark/README.md](sources/HPMLL__NVIDIA-Hopper-Benchmark/README.md); SHA-256: `c670f6b88462d2bd4a0d47b44d74fd5bbdcc14307814f1c2fd0578aeab478095`
- gpu: строка 3: This repository contains the code for benchmarking NVIDIA GPU performance. The relevant papers are as follows:

## criterion-rs/criterion.rs

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/criterion-rs/criterion.rs
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/criterion-rs__criterion.rs/README.md](sources/criterion-rs__criterion.rs/README.md); SHA-256: `c3cd50c84cd846fda13c04b2dc71d30ea619330b8265bdeea7fd132de95be78f`
- statistics: строка 38: - __Statistics__: Statistical analysis detects if, and by how much, performance has changed since the last benchmark run
- comparison: строка 21: Criterion.<span></span>rs helps you write fast code by detecting and measuring performance improvements or regressions, even small ones, quickly and accurately. You can optimize with confidence, knowing how each change affects the performance of your code.
- report: строка 50: criterion = { version = "0.8", features = ["html_reports"] }

## dotnet/BenchmarkDotNet

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/dotnet/BenchmarkDotNet
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/dotnet__BenchmarkDotNet/README.md](sources/dotnet__BenchmarkDotNet/README.md); SHA-256: `c7e977ad35d944c4c253663eb11ee9c2b20d1ac897dedd8f4776d24d9e0ffc42`
- statistics: строка 30: Under the hood, it performs a lot of [magic](#automation) that guarantees [reliable and precise](#reliability) results thanks to the [perfolizer](https://github.com/AndreyAkinshin/perfolizer) and [pragmastat](https://github.com/AndreyAkinshin/pragmastat) stati
- comparison: строка 43: [SimpleJob(RuntimeMoniker.Net472, baseline: true)]
- lifecycle: строка 57: [GlobalSetup]
- report: строка 111: The measured data can be exported to different formats (md, html, csv, xml, json, etc.) including plots:
- correctness: строка 28: **BenchmarkDotNet** helps you to transform methods into benchmarks, track their performance, and share reproducible measurement experiments.

## imazen/zenbench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/imazen/zenbench
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/imazen__zenbench/README.md](sources/imazen__zenbench/README.md); SHA-256: `76bbe11a3d482ba99138756869bc864cd0210987febd3aef787b5fbe6e2ceea0`
- statistics: строка 41: | Bootstrap confidence intervals | ✅ | ❌ | ✅ |
- comparison: строка 3: Interleaved microbenchmarking for Rust with paired statistics, CI regression testing, and hardware-adaptive measurement.
- lifecycle: строка 335: # In-process passes: resets calibration, warmup, heap addresses
- memory: строка 51: | Allocation profiling (GlobalAlloc) | ❌ | ✅ | ✅ |
- async: строка 62: | Bar chart | ❌ | ❌ | ✅ sorted, throughput |
- report: строка 5: **[Documentation](https://imazen.github.io/zenbench)** · **[Example HTML Report](https://imazen.github.io/zenbench/example-report.html)** · **[Tutorial](https://imazen.github.io/zenbench/getting-started/)**

## envidera/zench

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/envidera/zench
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/envidera__zench/README.md](sources/envidera__zench/README.md); SHA-256: `1b680b2b621ed2fd047536cc2ab21fcc68676e140952744987e84cffefc845a3`
- statistics: строка 65: name  │  median   │  cv   │  std.dev   │ outliers │ samples/iters
- comparison: строка 189: Currently, Zench focuses on relative comparisons and regression detection within the same run.
- lifecycle: строка 272: Run benchmarks directly from your editor by clicking `▶ Run Test`. See the [pre-configured setups](https://github.com/envidera/zench/blob/main/docs/configure-editors.md).
- memory: строка 309: - **Not a profiler** – Zench measures execution time and stability; it does not provide CPU flame graphs or memory allocation analysis.
- report: строка 61: You'll get a detailed report directly in your terminal:

## tokio-rs/loom

- Итог: retained — Benchmark design, diagnostic integration, or representative workload candidate; not a certification of repository quality.
- Источник: https://github.com/tokio-rs/loom
- Категория: benchmark/testing candidate
- Описание: None
- Уровень: automated README evidence extraction
- Снимок: [sources/tokio-rs__loom/README.md](sources/tokio-rs__loom/README.md); SHA-256: `1c0d0db3f1332c7a9591928d618b5ab35a7196f641c6da966b78753377e64a74`
- async: строка 3: Loom is a testing tool for concurrent Rust code. It runs a test many
