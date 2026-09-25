# Changelog

## Unreleased

- With both OTLP features compiled in, the transport is no longer decided at
  compile time: HTTP stays the default (matching `otlp-http` and the `:4318`
  endpoint every airbug tool prints) and `OTEL_EXPORTER_OTLP_PROTOCOL=grpc`
  opts into gRPC. Previously an `--all-features` build forced gRPC against an
  HTTP endpoint, and the mismatch only surfaced at shutdown as a transport
  error.

## 0.6.1

- Feature `testing`: `install_test` + in-memory exporters
- `TelemetryGuard::force_flush`; log provider clears on Drop for re-init in tests

## 0.6.0

- Initial family-aligned 0.6.0 release
