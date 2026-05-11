#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/product-smoke-local.sh [config_path]

Runs the Local Alpha product smoke check:
  1. ./scripts/agent-llm-mm.sh doctor [config_path]
  2. ./scripts/run-self-revision-demo.sh into a staging directory
  3. non-empty artifact checks before promoting staging to latest

The optional config path applies only to doctor. The deterministic self-revision
demo wrapper does not accept a config path and is intentionally run with its
existing demo contract. When running outside the repo root, pass an absolute
config path.
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -gt 1 ]]; then
  usage
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
project_root="$(cd "${script_dir}/.." && pwd -P)"
config_path="${1:-}"
resolved_config_path=""

if [[ -n "${config_path}" ]]; then
  if [[ ! -e "${config_path}" ]]; then
    printf 'product smoke failed: config path does not exist: %s\n' "${config_path}" >&2
    exit 2
  fi
  if [[ "${config_path}" = /* ]]; then
    resolved_config_path="${config_path}"
  else
    resolved_config_path="$(cd "$(dirname "${config_path}")" && pwd -P)/$(basename "${config_path}")"
  fi
fi

cd "${project_root}"

latest_dir="${project_root}/target/reports/self-revision-demo/latest"
reports_dir="${project_root}/target/reports/self-revision-demo"
staging_dir=""
previous_dir=""
promoted=0
lock_dir="${reports_dir}/.product-smoke.lock"
lock_acquired=0

cleanup() {
  local status=$?

  if [[ -n "${staging_dir}" && "${promoted}" -ne 1 ]]; then
    rm -rf "${staging_dir}"
    if [[ -n "${previous_dir}" && -e "${previous_dir}" && ! -e "${latest_dir}" ]]; then
      mv "${previous_dir}" "${latest_dir}" || true
    fi
  elif [[ "${promoted}" -eq 1 && -n "${previous_dir}" ]]; then
    rm -rf "${previous_dir}"
  fi

  if [[ "${lock_acquired}" -eq 1 ]]; then
    rm -rf "${lock_dir}"
  fi

  exit "${status}"
}
trap cleanup EXIT

required_artifacts=(
  doctor.json
  snapshot-before.json
  snapshot-after.json
  decision-before.json
  decision-after.json
  timeline.json
  sqlite-summary.json
  report.md
)

mkdir -p "${reports_dir}"
if ! mkdir "${lock_dir}" 2>/dev/null; then
  printf 'product smoke failed: another product smoke run is already updating %s\n' "${latest_dir}" >&2
  exit 1
fi
lock_acquired=1

printf 'product smoke: running doctor'
if [[ -n "${resolved_config_path}" ]]; then
  printf ' with config %s' "${resolved_config_path}"
fi
printf '\n'

if [[ -n "${resolved_config_path}" ]]; then
  ./scripts/agent-llm-mm.sh doctor "${resolved_config_path}"
else
  ./scripts/agent-llm-mm.sh doctor
fi

staging_dir="$(mktemp -d "${reports_dir}/.staging.XXXXXX")"

printf 'product smoke: running deterministic self-revision demo in staging dir %s\n' "${staging_dir}"
printf 'product smoke: validated artifacts will replace %s after the demo succeeds.\n' "${latest_dir}"
printf 'product smoke: config path is not passed to the demo wrapper; scripts/run-self-revision-demo.sh only accepts an output directory.\n'

./scripts/run-self-revision-demo.sh "${staging_dir}"

missing=0
for artifact in "${required_artifacts[@]}"; do
  artifact_path="${staging_dir}/${artifact}"
  if [[ ! -s "${artifact_path}" ]]; then
    printf 'product smoke failed: required artifact missing or empty: %s\n' "${artifact_path}" >&2
    missing=1
  fi
done

if [[ "${missing}" -ne 0 ]]; then
  exit 1
fi

if [[ -e "${latest_dir}" ]]; then
  previous_dir="$(mktemp -d "${project_root}/target/reports/self-revision-demo/.latest.previous.XXXXXX")"
  rmdir "${previous_dir}"
  mv "${latest_dir}" "${previous_dir}"
fi

mv "${staging_dir}" "${latest_dir}"
promoted=1
if [[ -n "${previous_dir}" ]]; then
  rm -rf "${previous_dir}"
  previous_dir=""
fi
rm -rf "${lock_dir}"
lock_acquired=0
trap - EXIT

printf 'product smoke: required demo artifacts are present and non-empty under %s\n' "${latest_dir}"
