# Changelog

## Unreleased

## 0.7.0

- A headless sampler by default on Linux and macOS: samples until Ctrl+C (or `--seconds N`) and logs a one-line summary; `--headless` is still accepted.
- The egui window is an optional feature: `--features gui`, opened with `--gui`.
- GPU on Linux: `nvidia-smi`, else the DRM sysfs counters (`gpu_busy_percent`, `mem_info_vram_*`, hwmon temperature); several GPUs are combined.
- Unified memory: `mem_used` is what the GPU holds and `mem_total` is the size of system RAM, instead of reporting the allocated figure as both.
