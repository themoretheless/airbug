# Security

Airbug is a **local monorepo toolkit**, not a multi-tenant service.

## Hub (`airbug-hub`)

- Listens on **`127.0.0.1` only** (loopback). Do not reverse-proxy it to the public internet without adding authentication and TLS yourself.
- **No auth** on `/api/*` ingest or issue mutations — acceptable only because the bind address is loopback.
- Issue webhooks accept **`http://` URLs only** (no HTTPS in the built-in client). Prefer localhost hooks.
- Request bodies are capped (`MAX_BODY_BYTES` / `MAX_HEADER_BYTES` in hub config).

## Error SDK (`airbug-err`)

- Default endpoint is `http://127.0.0.1:8790/api/v1/errors` (legacy `/api/errors` still accepted).
- Events may include stack frames and breadcrumbs; treat the hub database as sensitive local data.

## Bench live UI

- Localhost HTTP report browser; path-token style access only where implemented. Same loopback assumption as hub.

## Reporting issues

If you need remote ingest, terminate TLS and auth at your own gateway; do not weaken the loopback default in-tree without a deliberate redesign.
