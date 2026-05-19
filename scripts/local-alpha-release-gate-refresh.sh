#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: ./scripts/local-alpha-release-gate-refresh.sh [config_path]

Refreshes locally reproducible Local Alpha release-gate evidence:
  1. scripts/product-smoke-local.sh [config_path]
  2. scripts/first-run-bootstrap-smoke-local.sh target/first-run-bootstrap-smoke/local-alpha-gate
  3. scripts/generate-support-bundle.sh target/support-bundles/local-alpha-gate [config_path]
  4. scripts/local-alpha-evidence-summary.sh into target/reports/local-alpha/

This command only refreshes local evidence. It does not create Windows runner
evidence, real fresh-machine evidence, remote/team evidence, upload artifacts,
or automatically certify Local Alpha completion.
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
    printf 'local alpha release gate refresh failed: config path does not exist: %s\n' "${config_path}" >&2
    exit 2
  fi
  if [[ "${config_path}" = /* ]]; then
    resolved_config_path="${config_path}"
  else
    resolved_config_path="$(cd "$(dirname "${config_path}")" && pwd -P)/$(basename "${config_path}")"
  fi
fi

cd "${project_root}"

first_run_dir="target/first-run-bootstrap-smoke/local-alpha-gate"
support_bundle_dir="target/support-bundles/local-alpha-gate"
summary_json="target/reports/local-alpha/evidence-summary.json"
summary_md="target/reports/local-alpha/evidence-summary.md"
summary_dir="$(dirname "${summary_json}")"

printf 'local alpha release gate refresh: running product smoke\n'
if [[ -n "${resolved_config_path}" ]]; then
  scripts/product-smoke-local.sh "${resolved_config_path}"
else
  scripts/product-smoke-local.sh
fi

printf 'local alpha release gate refresh: refreshing first-run simulation evidence at %s\n' "${first_run_dir}"
rm -rf "${first_run_dir}"
scripts/first-run-bootstrap-smoke-local.sh "${first_run_dir}"

printf 'local alpha release gate refresh: refreshing support bundle evidence at %s\n' "${support_bundle_dir}"
rm -rf "${support_bundle_dir}"
if [[ -n "${resolved_config_path}" ]]; then
  scripts/generate-support-bundle.sh "${support_bundle_dir}" "${resolved_config_path}"
else
  scripts/generate-support-bundle.sh "${support_bundle_dir}"
fi

printf 'local alpha release gate refresh: writing evidence summary under %s\n' "${summary_dir}"
scripts/local-alpha-evidence-summary.sh \
  --evidence-root . \
  --output-json "${summary_json}" \
  --output-md "${summary_md}"

printf 'local alpha release gate refresh: summary JSON: %s\n' "${summary_json}"
printf 'local alpha release gate refresh: summary Markdown: %s\n' "${summary_md}"
printf 'local alpha release gate refresh: local refresh does not create Windows runner or real fresh-machine evidence; open/not_verified gates must remain open until separately verified.\n'
