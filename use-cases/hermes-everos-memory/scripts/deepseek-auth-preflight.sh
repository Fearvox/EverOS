#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${EVERCORE_ENV_FILE:-deploy/nixos/evercore.env.example}"
REQUIRE_KEY=0

usage() {
  cat <<'EOF'
Usage: scripts/deepseek-auth-preflight.sh [--env <path>] [--require-key]

Checks that the EverCore remote LLM auth path is pinned to DeepSeek through
OpenRouter without printing any credential value.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env)
      ENV_FILE="${2:-}"
      shift 2
      ;;
    --require-key)
      REQUIRE_KEY=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "BLOCK unknown_arg=$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "${ENV_FILE}" || ! -f "${ENV_FILE}" ]]; then
  echo "BLOCK env_file_missing"
  exit 1
fi

get_env() {
  local key="$1"
  local raw
  raw="$(grep -E "^${key}=" "${ENV_FILE}" | tail -n 1 || true)"
  raw="${raw#*=}"
  raw="${raw%$'\r'}"
  printf '%s' "${raw}"
}

LLM_PROVIDER="$(get_env LLM_PROVIDER)"
LLM_MODEL="$(get_env LLM_MODEL)"
LLM_OPENROUTER_PROVIDER="$(get_env LLM_OPENROUTER_PROVIDER)"
LLM_BASE_URL="$(get_env LLM_BASE_URL)"
OPENROUTER_BASE_URL="$(get_env OPENROUTER_BASE_URL)"
OPENROUTER_API_KEY="$(get_env OPENROUTER_API_KEY)"

failures=()

[[ "${LLM_PROVIDER}" == "openrouter" ]] || failures+=("LLM_PROVIDER must be openrouter")
[[ "${LLM_MODEL}" == deepseek/* ]] || failures+=("LLM_MODEL must be a deepseek/* OpenRouter model")
[[ "${LLM_OPENROUTER_PROVIDER}" == "deepseek" ]] || failures+=("LLM_OPENROUTER_PROVIDER must be deepseek")
[[ "${LLM_BASE_URL}" == "https://openrouter.ai/api/v1" ]] || failures+=("LLM_BASE_URL must be OpenRouter")
[[ "${OPENROUTER_BASE_URL}" == "https://openrouter.ai/api/v1" ]] || failures+=("OPENROUTER_BASE_URL must be OpenRouter")

if [[ "${REQUIRE_KEY}" -eq 1 ]]; then
  if [[ -z "${OPENROUTER_API_KEY}" || "${OPENROUTER_API_KEY}" == "change-me" ]]; then
    failures+=("OPENROUTER_API_KEY must be present and non-placeholder")
  fi
fi

if [[ "${#failures[@]}" -gt 0 ]]; then
  echo "BLOCK deepseek_auth_preflight"
  for failure in "${failures[@]}"; do
    echo "- ${failure}"
  done
  exit 1
fi

echo "PASS deepseek_auth_preflight provider=openrouter model=${LLM_MODEL} route=deepseek key_check=$([[ "${REQUIRE_KEY}" -eq 1 ]] && echo required || echo skipped)"
