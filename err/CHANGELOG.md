# Changelog

## Unreleased

- Configurable PII and secret scrubbing engine (`scrub.rs`: `ScrubAction`, `ScrubRule`) across headers, tags, extra fields, and exception/breadcrumb text.
- Isolated scope stack helpers (`push_scope` / scope contexts), frame instruction/symbol addresses, and `debug_images` metadata.
- Ecosystem integrations (`integrations.rs`): `HttpRequestMeta`, `HttpMiddleware`, `tracing-subscriber` layer, and optional `tower` service middleware.

## 0.11.0

- Version aligned with the rest of the airbug workspace: every package is 0.11.0 from here on. No functional change.

## 0.6.1

- Stable fingerprints (type + frames; volatile ids stripped from messages)
- Concrete `capture_error` type names; public `MemoryTransport` and `Guard::flush`

## 0.6.0

- Initial family-aligned 0.6.0 release
