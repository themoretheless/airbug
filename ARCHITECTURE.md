# Airbug architecture

Local developer toolkit (not hosted SaaS). Domains live in folders; packages are `airbug` or `airbug-<role>`.

## Composition roots

| Binary | Root | Notes |
|--------|------|-------|
| `airbug-hub` | `HubApp` in `dash/hub` | Injects `IssueStore`, paths, webhook |
| `cargo-airbug-bench` | `bench/cli` | Clap → action modules |
| `airbug-mon` | `mon/src/main.rs` | Channels + sampler + egui `MonApp` |

Libraries (`airbug`, `airbug-bench`, `airbug-err`, `airbug-otel`) expose builders/traits; they do not install a global `tracing` subscriber.

## Error conventions

- **Libraries:** typed `thiserror` enums (`HubError`, `TransportError`, `BenchError`, `TelemetryError`).
- **Binaries:** `tracing` + typed/`anyhow`-style `?` at the edge.
- **Hub JSON errors:** `{ "ok": false, "error": "<message>" }` only.

## Contracts

| Contract | Version field | Notes |
|----------|---------------|-------|
| `airbug_err::Event` | `schema_version` (u32, default 1) | Hub rejects unsupported majors |
| Unit report JSON | `UnitReportV1` shape in hub scan | Producers write `target/airbug-report/report.json` |
| Bench runs | filesystem `run.json` trees under `.airbug-bench` | Scanned, not a shared crate |

## Threat model

Hub binds **`127.0.0.1` only**. Issue webhooks accept **`http://` only**. See [SECURITY.md](SECURITY.md).

## Artifact paths

```text
target/airbug-report/          # unit HTML/JSON reports
.airbug-bench/                 # bench store (runs, baselines)
dash/collector/data/           # OTLP file export
dash/hub/data/issues.sqlite    # error issues
~/.local/share/airbug-mon/     # mon history (platform-specific)
```

## Logging

Binary roots install `tracing_subscriber` with `RUST_LOG` / `AIRBUG_LOG` env filters. Libraries may emit `tracing` events; they never force a global subscriber.
