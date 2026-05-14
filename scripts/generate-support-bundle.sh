#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: ./scripts/generate-support-bundle.sh <output_dir> [config_path]

Generates a local-only support bundle directory with redacted diagnostic JSON.
The bundle excludes full SQLite databases, raw TOML files, provider payloads,
and secrets by default.
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
project_root="$(cd "${script_dir}/.." && pwd -P)"
output_dir="$1"
config_path="${2:-}"

cd "${project_root}"

if [[ -n "${config_path}" ]]; then
  if [[ ! -e "${config_path}" ]]; then
    printf 'support bundle failed: config path does not exist: %s\n' "${config_path}" >&2
    exit 2
  fi
  if [[ "${config_path}" != /* ]]; then
    config_path="$(cd "$(dirname "${config_path}")" && pwd -P)/$(basename "${config_path}")"
  fi
fi

if [[ "${output_dir}" != /* ]]; then
  output_parent="$(dirname "${output_dir}")"
  mkdir -p "${output_parent}"
  output_dir="$(cd "${output_parent}" && pwd -P)/$(basename "${output_dir}")"
fi

if [[ -n "${config_path}" ]]; then
  cargo run --quiet --bin generate_support_bundle -- "${output_dir}" "${config_path}"
else
  cargo run --quiet --bin generate_support_bundle -- "${output_dir}"
fi
