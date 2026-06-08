#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: ./scripts/provider-live-certification-run.sh --stub-evidence [config_path] [evidence_root]

Generates explicit provider certification evidence files only when
--stub-evidence is provided. This stub/simulated mode does not call provider
endpoints and does not claim real live provider certification.

Live network certification is not implemented yet; omitting --stub-evidence
is rejected by the Rust runner.
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

stub_evidence=""
if [[ "${1:-}" == "--stub-evidence" ]]; then
  stub_evidence="yes"
  shift
fi

if [[ $# -gt 2 ]]; then
  usage
  exit 2
fi

config_path="${1:-}"
evidence_root="${2:-.}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
project_root="$(cd "${script_dir}/.." && pwd -P)"
cd "${project_root}"

args=(--evidence-root "${evidence_root}")
if [[ -n "${stub_evidence}" ]]; then
  args+=(--stub-evidence)
fi
if [[ -n "${config_path}" ]]; then
  args+=(--config-path "${config_path}")
fi

cargo run --quiet --bin provider_live_certification_run -- "${args[@]}"
