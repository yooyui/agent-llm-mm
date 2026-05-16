#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: ./scripts/generate-support-bundle.sh <output_dir> [config_path] [--log-file <path>] [--correlation-id <id>]

Generates a local-only support bundle directory with redacted diagnostic JSON.
The bundle excludes full SQLite databases, raw TOML files, provider payloads,
raw log files, and secrets by default. A local log file is only summarized when
explicitly passed with --log-file. A correlation id can be forwarded when
explicitly passed with --correlation-id.
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
log_path_requested=0
correlation_id=""
correlation_id_requested=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --log-file)
      if [[ $# -lt 2 ]]; then
        printf 'support bundle failed: missing value for --log-file\n' >&2
        exit 2
      fi
      log_path_requested=1
      log_path="$2"
      shift 2
      ;;
    --correlation-id)
      if [[ $# -lt 2 ]]; then
        printf 'support bundle failed: missing value for --correlation-id\n' >&2
        exit 2
      fi
      correlation_id_requested=1
      correlation_id="$2"
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
config_path_requested=0
if [[ ${#positional[@]} -eq 2 ]]; then
  config_path_requested=1
fi

cd "${project_root}"

if [[ "${config_path_requested}" -eq 1 ]]; then
  if [[ ! -e "${config_path}" ]]; then
    printf 'support bundle failed: config path does not exist: %s\n' "${config_path}" >&2
    exit 2
  fi
  if [[ "${config_path}" != /* ]]; then
    config_path="$(cd "$(dirname "${config_path}")" && pwd -P)/$(basename "${config_path}")"
  fi
fi

if [[ "${log_path_requested}" -eq 1 ]]; then
  if [[ "${log_path}" != /* && -e "${log_path}" ]]; then
    log_path="$(cd "$(dirname "${log_path}")" && pwd -P)/$(basename "${log_path}")"
  fi
fi

if [[ "${output_dir}" != /* ]]; then
  output_parent="$(dirname "${output_dir}")"
  mkdir -p "${output_parent}"
  output_dir="$(cd "${output_parent}" && pwd -P)/$(basename "${output_dir}")"
fi

args=("${output_dir}")
if [[ "${config_path_requested}" -eq 1 ]]; then
  args+=("${config_path}")
fi
if [[ "${log_path_requested}" -eq 1 ]]; then
  args+=(--log-file "${log_path}")
fi
if [[ "${correlation_id_requested}" -eq 1 ]]; then
  args+=(--correlation-id "${correlation_id}")
fi

cargo run --quiet --bin generate_support_bundle -- "${args[@]}"
