#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: ./scripts/generate-support-bundle.sh <output_dir> [config_path] [--log-file <path>]

Generates a local-only support bundle directory with redacted diagnostic JSON.
The bundle excludes full SQLite databases, raw TOML files, provider payloads,
raw log files, and secrets by default. A local log file is only summarized when
explicitly passed with --log-file.
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage
  exit 2
fi

positional=()
log_path=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --log-file)
      if [[ $# -lt 2 ]]; then
        printf 'support bundle failed: missing value for --log-file\n' >&2
        exit 2
      fi
      log_path="$2"
      shift 2
      ;;
    --)
      shift
      while [[ $# -gt 0 ]]; do
        positional+=("$1")
        shift
      done
      ;;
    -*)
      printf 'support bundle failed: unknown option: %s\n' "$1" >&2
      usage
      exit 2
      ;;
    *)
      positional+=("$1")
      shift
      ;;
  esac
done

if [[ ${#positional[@]} -lt 1 || ${#positional[@]} -gt 2 ]]; then
  usage
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
project_root="$(cd "${script_dir}/.." && pwd -P)"
output_dir="${positional[0]}"
config_path="${positional[1]:-}"

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

if [[ -n "${log_path}" ]]; then
  if [[ "${log_path}" != /* && -e "${log_path}" ]]; then
    log_path="$(cd "$(dirname "${log_path}")" && pwd -P)/$(basename "${log_path}")"
  fi
fi

if [[ "${output_dir}" != /* ]]; then
  output_parent="$(dirname "${output_dir}")"
  mkdir -p "${output_parent}"
  output_dir="$(cd "${output_parent}" && pwd -P)/$(basename "${output_dir}")"
fi

if [[ -n "${config_path}" && -n "${log_path}" ]]; then
  cargo run --quiet --bin generate_support_bundle -- "${output_dir}" "${config_path}" --log-file "${log_path}"
elif [[ -n "${config_path}" ]]; then
  cargo run --quiet --bin generate_support_bundle -- "${output_dir}" "${config_path}"
elif [[ -n "${log_path}" ]]; then
  cargo run --quiet --bin generate_support_bundle -- "${output_dir}" --log-file "${log_path}"
else
  cargo run --quiet --bin generate_support_bundle -- "${output_dir}"
fi
