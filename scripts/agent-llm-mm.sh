#!/usr/bin/env bash

set -euo pipefail

MODE="${1:-serve}"
CONFIG_PATH="${2:-}"

case "$MODE" in
  serve|doctor)
    ;;
  *)
    echo "unsupported mode: $MODE" >&2
    echo "usage: ./scripts/agent-llm-mm.sh [serve|doctor] [config_path]" >&2
    exit 2
    ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"

cd "$project_root"

if [[ -n "$CONFIG_PATH" ]]; then
  config_dir="$(cd "$(dirname "$CONFIG_PATH")" && pwd)"
  config_file="$(basename "$CONFIG_PATH")"
  export AGENT_LLM_MM_CONFIG="${config_dir}/${config_file}"
fi

exec cargo run --quiet --bin agent_llm_mm -- "$MODE"
