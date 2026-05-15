#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
HERMES_HOME="${HERMES_HOME:-${HOME}/.hermes}"
DEST_DIR="${HERMES_HOME}/plugins/everos"

mkdir -p "${DEST_DIR}"

for item in __init__.py plugin.yaml README.md bin package.json justfile scripts skillhub raven deploy; do
  rm -rf "${DEST_DIR}/${item}"
  cp -R "${PLUGIN_DIR}/${item}" "${DEST_DIR}/${item}"
done

echo "Installed Hermes EverOS memory provider to active Hermes plugins dir."
echo "Activate with: hermes config set memory.provider everos"
