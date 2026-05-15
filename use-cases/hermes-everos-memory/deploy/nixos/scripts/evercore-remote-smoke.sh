#!/usr/bin/env bash
set -euo pipefail

mode="health"
base_url="${EVEROS_API_BASE_URL:-http://127.0.0.1:1995}"
user_id="${EVEROS_USER_ID:-hermes-remote-smoke}"
agent_id="${EVEROS_AGENT_ID:-hermes}"

usage() {
  cat <<'USAGE'
Usage: evercore-remote-smoke.sh [--mode health|write|full]

Modes:
  health  Check /health only.
  write   Check /health and POST one agent-memory smoke turn.
  full    Check /health, write one turn, then search for it.

Environment:
  EVEROS_API_BASE_URL  Default http://127.0.0.1:1995
  EVEROS_USER_ID       Default hermes-remote-smoke
  EVEROS_AGENT_ID      Default hermes
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$mode" in
  health|write|full) ;;
  *)
    echo "invalid mode: $mode" >&2
    exit 2
    ;;
esac

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 127
  }
}

need curl
need jq

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

curl_json() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local out="$4"

  if [[ -n "$body" ]]; then
    curl -fsS --max-time 30 \
      -X "$method" \
      -H 'Content-Type: application/json' \
      --data-binary @"$body" \
      "$base_url$path" > "$out"
  else
    curl -fsS --max-time 30 \
      -X "$method" \
      -H 'Content-Type: application/json' \
      "$base_url$path" > "$out"
  fi
}

health_out="$tmpdir/health.json"
curl_json GET /health "" "$health_out"
status="$(jq -r '.status // .data.status // "unknown"' "$health_out")"
if [[ "$status" != "healthy" ]]; then
  echo "BLOCK health status=$status"
  exit 1
fi
echo "PASS health status=healthy"

if [[ "$mode" == "health" ]]; then
  exit 0
fi

session_id="hermes-remote-smoke-$(date +%s)"
now_ms="$(($(date +%s) * 1000))"
body="$tmpdir/agent-add.json"
jq -n \
  --arg user_id "$user_id" \
  --arg agent_id "$agent_id" \
  --arg session_id "$session_id" \
  --argjson now_ms "$now_ms" \
  '{
    user_id: $user_id,
    session_id: $session_id,
    messages: [
      {
        role: "user",
        sender_id: $user_id,
        timestamp: $now_ms,
        content: "EverCore remote Hermes provider smoke write."
      },
      {
        role: "assistant",
        sender_id: $agent_id,
        timestamp: ($now_ms + 1),
        content: "EverCore remote smoke response persisted for provider validation."
      }
    ]
  }' > "$body"

write_out="$tmpdir/write.json"
curl_json POST /api/v1/memories/agent "$body" "$write_out"
write_status="$(jq -r '.data.status // .status // "accepted"' "$write_out")"
echo "PASS write status=$write_status"

if [[ "$mode" == "write" ]]; then
  exit 0
fi

flush_body="$tmpdir/agent-flush.json"
jq -n \
  --arg user_id "$user_id" \
  --arg session_id "$session_id" \
  '{user_id: $user_id, session_id: $session_id}' > "$flush_body"

flush_out="$tmpdir/flush.json"
curl_json POST /api/v1/memories/agent/flush "$flush_body" "$flush_out"
flush_status="$(jq -r '.data.status // .status // "flushed"' "$flush_out")"
echo "PASS flush status=$flush_status"

search_body="$tmpdir/search.json"
jq -n \
  --arg query "EverCore remote Hermes provider smoke write" \
  --arg user_id "$user_id" \
  '{
    query: $query,
    method: "hybrid",
    memory_types: ["episodic_memory", "raw_message", "profile", "agent_memory"],
    top_k: 5,
    filters: {user_id: $user_id}
  }' > "$search_body"

search_out="$tmpdir/search-out.json"
curl_json POST /api/v1/memories/search "$search_body" "$search_out"
episodes="$(jq '.data.episodes // [] | length' "$search_out")"
raw_messages="$(jq '.data.raw_messages // [] | length' "$search_out")"
profiles="$(jq '.data.profiles // [] | length' "$search_out")"
agent_cases="$(jq '.data.agent_memory.cases // [] | length' "$search_out")"
agent_skills="$(jq '.data.agent_memory.skills // [] | length' "$search_out")"

total="$((episodes + raw_messages + profiles + agent_cases + agent_skills))"
if [[ "$total" -lt 1 ]]; then
  echo "BLOCK search returned no retrievable memories"
  exit 1
fi
echo "PASS search episodes=$episodes raw_messages=$raw_messages profiles=$profiles agent_cases=$agent_cases agent_skills=$agent_skills"
