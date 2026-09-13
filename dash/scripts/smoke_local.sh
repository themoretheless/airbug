#!/usr/bin/env bash
# Local smoke: hub APIs + optional collector telemetry + err ingest.
# Does not start Docker; skips collector-dependent checks when ports are down.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

PORT="${AIRBUG_HUB_PORT:-8790}"
BASE="http://127.0.0.1:${PORT}"

echo "== build hub =="
cargo build -p airbug-hub --locked -q

echo "== start hub =="
cargo run -p airbug-hub --quiet -- serve --root . --port "$PORT" &
HUB_PID=$!
cleanup() {
  kill "$HUB_PID" 2>/dev/null || true
  wait "$HUB_PID" 2>/dev/null || true
}
trap cleanup EXIT

for _ in $(seq 1 40); do
  if curl -sf "$BASE/api/status" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -sf "$BASE/api/status" | head -c 200 >/dev/null
echo "status ok"

curl -sf "$BASE/api/logs?limit=5" >/dev/null && echo "logs ok"
curl -sf "$BASE/api/metrics?limit=5" >/dev/null && echo "metrics ok"
curl -sf "$BASE/api/issues" >/dev/null && echo "issues ok"

EVENT='{"event_id":"smoke-1","timestamp":"t","level":"error","message":"smoke","fingerprint":["smoke"],"breadcrumbs":[],"tags":{},"extra":{},"contexts":{}}'
curl -sf -X POST "$BASE/api/errors" -H 'content-type: application/json' -d "$EVENT" >/dev/null
echo "errors ingest ok"

if curl -sf --max-time 1 "http://127.0.0.1:4318" >/dev/null 2>&1 \
  || nc -z 127.0.0.1 4318 >/dev/null 2>&1; then
  echo "== collector up: emit otel examples =="
  export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
  cargo run -p airbug-otel --example metrics --quiet || true
  cargo run -p airbug-otel --example logs --quiet || true
else
  echo "== collector not on :4318 — skip otel examples =="
fi

echo "smoke_local: done"
