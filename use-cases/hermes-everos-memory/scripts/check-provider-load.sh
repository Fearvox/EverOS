#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
HERMES_AGENT_SRC="${HERMES_AGENT_SRC:-${HOME}/.hermes/hermes-agent}"

if [[ ! -d "${HERMES_AGENT_SRC}" ]]; then
  echo "Hermes agent source not found. Set HERMES_AGENT_SRC." >&2
  exit 2
fi

TMP_HOME="$(mktemp -d)"
cleanup() {
  rm -rf "${TMP_HOME}"
}
trap cleanup EXIT

mkdir -p "${TMP_HOME}/plugins/everos"
cp "${PLUGIN_DIR}/__init__.py" "${PLUGIN_DIR}/plugin.yaml" "${TMP_HOME}/plugins/everos/"

HERMES_HOME="${TMP_HOME}" PYTHONPATH="${HERMES_AGENT_SRC}" python3 - <<'PY'
from plugins.memory import discover_memory_providers, load_memory_provider

names = [name for name, _, _ in discover_memory_providers()]
assert "everos" in names, names

provider = load_memory_provider("everos")
assert provider is not None
assert provider.name == "everos"

tools = [schema["name"] for schema in provider.get_tool_schemas()]
expected = {"everos_search", "everos_store", "everos_health", "everos_flush"}
assert expected.issubset(set(tools)), tools

print("PASS provider-load everos " + ",".join(tools))
PY
