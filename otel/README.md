# otel

Runtime observability: live host metrics, history, alerts.

Today: `monik` (egui + sysinfo + SQLite).

Next (when needed): OTLP exporters, shared metric types, scrapers —
extract shared sampling into a small lib under this domain; keep the UI in `monik`.
