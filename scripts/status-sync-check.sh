#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [[ -e "${REPO_ROOT}/not-a-sqlite-url" ]]; then
  echo "repository hygiene failed: unexpected root SQLite artifact not-a-sqlite-url" >&2
  exit 1
fi

cd "${REPO_ROOT}"
cargo run --quiet --bin status_sync_check
