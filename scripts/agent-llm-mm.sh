#!/usr/bin/env bash

set -euo pipefail

MODE="${1:-serve}"
CONFIG_PATH="${2:-}"

case "$MODE" in
  serve|doctor|bootstrap-local)
    ;;
  *)
    echo "unsupported mode: $MODE" >&2
    echo "usage: ./scripts/agent-llm-mm.sh [serve|doctor|bootstrap-local] [config_path]" >&2
    exit 2
    ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"

cd "$project_root"

if [[ "$MODE" == "bootstrap-local" ]]; then
  target_path="${CONFIG_PATH:-agent-llm-mm.local.toml}"
  source_path="examples/agent-llm-mm.dev.example.toml"
  target_parent="$(dirname "$target_path")"

  if [[ -e "$target_path" || -L "$target_path" ]]; then
    echo "target already exists; refusing to overwrite: $target_path" >&2
    exit 1
  fi
  if [[ ! -d "$target_parent" ]]; then
    echo "parent directory does not exist: $target_parent" >&2
    exit 1
  fi

  if ! (set -o noclobber; cat "$source_path" > "$target_path") 2>/dev/null; then
    echo "target already exists; refusing to overwrite: $target_path" >&2
    exit 1
  fi
  quoted_target_path="$(printf '%q' "$target_path")"
  echo "created local config: $target_path"
  echo "Next commands:"
  echo "  ./scripts/agent-llm-mm.sh doctor $quoted_target_path"
  echo "  ./scripts/agent-llm-mm.sh serve $quoted_target_path"
  exit 0
fi

if [[ -n "$CONFIG_PATH" ]]; then
  config_dir="$(cd "$(dirname "$CONFIG_PATH")" && pwd)"
  config_file="$(basename "$CONFIG_PATH")"
  export AGENT_LLM_MM_CONFIG="${config_dir}/${config_file}"
fi

exec cargo run --quiet --bin agent_llm_mm -- "$MODE"
