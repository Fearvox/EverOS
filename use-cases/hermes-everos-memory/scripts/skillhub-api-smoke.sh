#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${SKILLHUB_PORT:-18765}"
HOST="${SKILLHUB_HOST:-127.0.0.1}"
TMP_DIR="$(mktemp -d)"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

node "$ROOT_DIR/bin/skillhub-mock-api.mjs" \
  --host "$HOST" \
  --port "$PORT" \
  >"$TMP_DIR/server.log" \
  2>&1 &
SERVER_PID="$!"

for _ in 1 2 3 4 5 6 7 8 9 10; do
  if curl -fsS "http://$HOST:$PORT/health" >"$TMP_DIR/health.json" 2>/dev/null; then
    break
  fi
  sleep 0.2
done

curl -fsS "http://$HOST:$PORT/health" >"$TMP_DIR/health.json"
curl -fsS "http://$HOST:$PORT/skills?target=hermes" >"$TMP_DIR/skills.json"
curl -fsS "http://$HOST:$PORT/skills/raven.operator-memory-recall/render" \
  >"$TMP_DIR/render.md"
curl -fsS "http://$HOST:$PORT/skills/raven.operator-memory-recall/views" \
  >"$TMP_DIR/views.json"
curl -fsS "http://$HOST:$PORT/skills/raven.operator-memory-recall/install-packet?target=hermes" \
  >"$TMP_DIR/install-packet.json"

rg '"ok": true' "$TMP_DIR/health.json" >/dev/null
rg 'raven.operator-memory-recall' "$TMP_DIR/skills.json" >/dev/null
rg 'Operator Memory Recall' "$TMP_DIR/render.md" >/dev/null
rg '"views_markdown":' "$TMP_DIR/views.json" >/dev/null
rg 'Trust Panel' "$TMP_DIR/views.json" >/dev/null
rg '"install_packet":' "$TMP_DIR/install-packet.json" >/dev/null
rg '"target": "hermes"' "$TMP_DIR/install-packet.json" >/dev/null

printf 'PASS skillhub mock api smoke host=%s port=%s\n' "$HOST" "$PORT"
