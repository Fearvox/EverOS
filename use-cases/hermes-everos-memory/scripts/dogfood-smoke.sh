#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
HERMES_AGENT_SRC="${HERMES_AGENT_SRC:-${HOME}/.hermes/hermes-agent}"
MODE="provider-only"

usage() {
  cat <<'USAGE'
Usage: dogfood-smoke.sh [--mode provider-only|health|full]

Modes:
  provider-only  Load the Hermes provider and verify schemas. No EverCore required.
  health         Load provider and require EverCore /health to be reachable.
  full           Require health, store one turn, then search/prefetch.

Environment:
  HERMES_AGENT_SRC      Hermes agent source checkout. Default ~/.hermes/hermes-agent
  EVEROS_API_BASE_URL   EverCore base URL. Default http://127.0.0.1:1995
  EVEROS_USER_ID        EverOS user id for smoke. Default hermes-dogfood-smoke
  EVEROS_AGENT_ID       EverOS agent id. Default hermes-dogfood
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      MODE="${2:-}"
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

case "${MODE}" in
  provider-only|health|full) ;;
  *)
    echo "invalid mode: ${MODE}" >&2
    exit 2
    ;;
esac

if [[ ! -d "${HERMES_AGENT_SRC}" ]]; then
  echo "BLOCK hermes_agent_src_missing"
  exit 2
fi

MODE="${MODE}" \
PLUGIN_DIR="${PLUGIN_DIR}" \
EVEROS_USER_ID="${EVEROS_USER_ID:-hermes-dogfood-smoke}" \
EVEROS_AGENT_ID="${EVEROS_AGENT_ID:-hermes-dogfood}" \
PYTHONPATH="${HERMES_AGENT_SRC}" \
python3 - <<'PY'
import importlib.util
import json
import os
import pathlib
import sys
import time

mode = os.environ["MODE"]
plugin_dir = pathlib.Path(os.environ["PLUGIN_DIR"])
module_path = plugin_dir / "__init__.py"

spec = importlib.util.spec_from_file_location("everos_memory_provider_smoke", module_path)
if spec is None or spec.loader is None:
    print("BLOCK provider_spec_unavailable")
    sys.exit(1)

module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

provider = module.EverOSMemoryProvider()
provider.initialize(
    session_id=f"hermes-dogfood-smoke-{int(time.time())}",
    agent_identity="dogfood",
    user_id=os.environ["EVEROS_USER_ID"],
)

tools = {schema["name"] for schema in provider.get_tool_schemas()}
expected = {"everos_search", "everos_store", "everos_health", "everos_flush"}
missing = sorted(expected - tools)
if missing:
    print("BLOCK provider_schema_missing " + ",".join(missing))
    sys.exit(1)

prompt = provider.system_prompt_block()
if "EverOS Memory" not in prompt:
    print("BLOCK provider_prompt_missing")
    sys.exit(1)

print("PASS provider_load tools=" + ",".join(sorted(tools)))

if mode == "provider-only":
    sys.exit(0)

if not provider.is_available():
    print("BLOCK evercore_unavailable")
    sys.exit(1)

health_raw = provider.handle_tool_call("everos_health", {})
try:
    health = json.loads(health_raw)
except json.JSONDecodeError:
    print("BLOCK health_non_json")
    sys.exit(1)

status = (health.get("result") or {}).get("status", "unknown")
if status not in {"healthy", "ok"}:
    print(f"BLOCK health_status={status}")
    sys.exit(1)

print("PASS health status=" + status)

if mode == "health":
    sys.exit(0)

stamp = int(time.time())
needle = f"Hermes EverOS dogfood smoke Raven SkillHub {stamp}"

store_raw = provider.handle_tool_call(
    "everos_store",
    {
        "role": "user",
        "content": (
            needle
            + ": provider-level store/search/recall smoke for Raven and EverMe SkillHub."
        ),
    },
)
try:
    store = json.loads(store_raw)
except json.JSONDecodeError:
    print("BLOCK store_non_json")
    sys.exit(1)

if store.get("result") != "stored":
    print("BLOCK store_failed")
    sys.exit(1)

print("PASS store result=stored")

flush_raw = provider.handle_tool_call("everos_flush", {})
try:
    flush = json.loads(flush_raw)
except json.JSONDecodeError:
    print("BLOCK flush_non_json")
    sys.exit(1)

if flush.get("result") != "flushed":
    print("BLOCK flush_failed")
    sys.exit(1)

print("PASS flush result=flushed")

time.sleep(2)

search_raw = provider.handle_tool_call(
    "everos_search",
    {
        "query": needle,
        "top_k": 5,
        "memory_types": ["episodic_memory", "profile", "agent_memory", "raw_message"],
    },
)
try:
    search = json.loads(search_raw)
except json.JSONDecodeError:
    print("BLOCK search_non_json")
    sys.exit(1)

count = int(search.get("count") or 0)
if count < 1:
    print("BLOCK search_count=0")
    sys.exit(1)

print(f"PASS search count={count}")

prefetch = provider.prefetch(needle)
if not prefetch:
    print("BLOCK prefetch_empty")
    sys.exit(1)

print("PASS prefetch chars=" + str(len(prefetch)))

provider.shutdown()
PY
