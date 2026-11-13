# mon

Host sampler: CPU, memory, swap, network, disk, GPU and the top processes once a second,
stored in SQLite and optionally exported as OpenTelemetry metrics. Runs without a window on
Linux and macOS; an egui window over the same samples is an optional feature.

Package: `airbug-mon` (sysinfo + SQLite). OTLP lives in `otel/` (`airbug-otel`).

## Run

```sh
cargo run -p airbug-mon --release                       # sample until Ctrl+C
cargo run -p airbug-mon --release -- --seconds 15       # sample for 15 s
cargo run -p airbug-mon --release -- --otlp             # also export to the collector on :4318
# or: export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
```

Every tenth sample is logged as one line (`cpu 12% · mem 3.1/16.0 GiB · gpu 40% 1.2/16.0 GiB (unified)`).
Samples go to `~/.local/share/airbug-mon/airbug-mon.db`: raw samples for 24 hours, per-minute
aggregates for 30 days. Host metrics sent over OTLP show up in the hub's Metrics tab — link the
hub, mon has no port of its own.

`--headless` is accepted for older scripts; it is the default.

## Window (optional)

```sh
cargo run -p airbug-mon --release --features gui -- --gui
```

- **Overview**: live charts of CPU (total and per core), RAM/swap, GPU, network and disk for the last 10 minutes.
- **Processes**: top 10 by CPU+RAM, sortable, with a filter and a mini history of the selected process.
- **History**: 1 h / 6 h / 24 h / 7 d / 30 d from SQLite (long ranges use the minute aggregates), CSV export.
- **Alerts**: CPU/RAM/disk thresholds with native notifications and a log.

## GPU

| Platform | Source | Notes |
|---|---|---|
| Linux, NVIDIA | `nvidia-smi --query-gpu=…` | load, VRAM used/total, temperature; several GPUs are combined (busiest load, memory summed) |
| Linux, other | `/sys/class/drm/card*/device/gpu_busy_percent`, `mem_info_vram_*`, `hwmon/*/temp1_input` | amdgpu and drivers that expose the same files |
| macOS | `powermetrics --samplers gpu_power` | needs sudo; without it there is no GPU sample |

On a unified-memory machine (Apple Silicon) CPU and GPU allocate from one pool of RAM, so there
is no VRAM total to report: `mem_used` is what the GPU holds and `mem_total` is the size of system
RAM. A host without a readable GPU is sampled like any other, with no GPU columns.

The first sample after start has zero network/disk rates (no previous point for the delta).

## Tests

```sh
cargo test -p airbug-mon
```
