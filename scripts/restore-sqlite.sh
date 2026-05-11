#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage:
  restore-sqlite.sh <backup-file> <new-db-path>

Examples:
  ./scripts/restore-sqlite.sh target/backups/sqlite/formal.sqlite.20260511-120000.12345.bak ./target/restore-check.sqlite
  ./scripts/restore-sqlite.sh ./formal.sqlite.bak sqlite:///Users/me/agent-llm-mm/restore-check.sqlite

Restore always writes to a new target path and refuses to overwrite an existing file.
EOF
}

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

warn() {
  printf 'warning: %s\n' "$*" >&2
}

require_non_empty() {
  local name="$1"
  local value="$2"
  if [[ -z "${value//[[:space:]]/}" ]]; then
    fail "${name} must not be empty"
  fi
}

decode_percent_path() {
  local encoded="$1"
  local decoded=""
  local i=0
  local len=${#encoded}
  local char
  local hex
  local decoded_char

  while (( i < len )); do
    char="${encoded:i:1}"
    if [[ "$char" == "%" ]]; then
      if (( i + 2 >= len )); then
        fail "invalid percent-encoding in SQLite URL path: percent signs must be followed by two hex digits"
      fi
      hex="${encoded:i+1:2}"
      if [[ ! "$hex" =~ ^[0-9A-Fa-f]{2}$ ]]; then
        fail "invalid percent-encoding in SQLite URL path: percent signs must be followed by two hex digits"
      fi
      case "$hex" in
        0[0-9A-Fa-f]|1[0-9A-Fa-f]|7[Ff])
          fail "unsupported percent-encoding in SQLite URL path: control characters are not allowed"
          ;;
      esac
      printf -v decoded_char '%b' "\\x${hex}"
      if [[ "$decoded_char" == "\\" || "$decoded_char" == '"' ]]; then
        fail "unsupported percent-encoding in SQLite URL path: encoded backslashes and double quotes are not allowed"
      fi
      decoded+="$decoded_char"
      i=$((i + 3))
    else
      decoded+="$char"
      i=$((i + 1))
    fi
  done

  printf '%s' "$decoded"
}

normalize_sqlite_file_path() {
  local path="$1"
  case "$path" in
    /[A-Za-z]:/*|/[A-Za-z]:\\*|/[A-Za-z]:)
      printf '%s' "${path#/}"
      ;;
    *)
      printf '%s' "$path"
      ;;
  esac
}

sqlite_input_to_path() {
  local label="$1"
  local input="$2"
  require_non_empty "$label" "$input"

  case "$input" in
    sqlite::memory:|:memory:)
      fail "in-memory SQLite databases cannot be restored as files"
      ;;
    sqlite://*)
      local path_part="${input#sqlite://}"
      local decoded_path
      require_non_empty "$label" "$path_part"
      if [[ "$path_part" == *\\* ]]; then
        fail "SQLite file URLs must use forward slashes; backslashes are not supported"
      fi
      if ! decoded_path="$(decode_percent_path "$path_part")"; then
        return 1
      fi
      normalize_sqlite_file_path "$decoded_path"
      ;;
    *)
      printf '%s' "$input"
      ;;
  esac
}

verify_checksum_if_present() {
  local file="$1"
  local checksum_file="${file}.sha256"

  if [[ ! -f "$checksum_file" ]]; then
    return 0
  fi

  local dir
  local checksum_base
  dir="$(cd -- "$(dirname -- "$checksum_file")" && pwd -P)"
  checksum_base="$(basename -- "$checksum_file")"

  if command -v shasum >/dev/null 2>&1; then
    (cd -- "$dir" && shasum -a 256 -c "$checksum_base")
  elif command -v sha256sum >/dev/null 2>&1; then
    (cd -- "$dir" && sha256sum -c "$checksum_base")
  else
    warn "checksum file exists but no shasum or sha256sum was found; skipping verification"
  fi
}

harden_sqlite_file_permissions() {
  local file="$1"
  chmod 600 "$file"
}

reject_unsupported_file_path_chars() {
  local label="$1"
  local path="$2"

  if [[ "$path" == *\"* || "$path" == *\\* || "$path" == *$'\n'* ]]; then
    fail "${label} must not contain double quotes, backslashes, or newlines: ${path}"
  fi
}

reserve_new_file() {
  local file="$1"

  if ! (set -C; : > "$file") 2>/dev/null; then
    fail "target path already exists, refusing to overwrite: ${file}"
  fi
}

remove_file_on_failure() {
  local file="$1"
  rm -f -- "$file"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -ne 2 ]]; then
  usage
  exit 2
fi

if ! backup_file="$(sqlite_input_to_path "backup file" "$1")"; then
  exit 1
fi
if ! target_path="$(sqlite_input_to_path "target database path" "$2")"; then
  exit 1
fi
require_non_empty "backup file" "$backup_file"
require_non_empty "target database path" "$target_path"
reject_unsupported_file_path_chars "backup file" "$backup_file"
reject_unsupported_file_path_chars "target database path" "$target_path"

[[ -f "$backup_file" ]] || fail "backup file does not exist: ${backup_file}"

verify_checksum_if_present "$backup_file"

umask 077
mkdir -p -- "$(dirname -- "$target_path")"
reserve_new_file "$target_path"
if ! cp -- "$backup_file" "$target_path"; then
  remove_file_on_failure "$target_path"
  fail "restore copy failed"
fi
harden_sqlite_file_permissions "$target_path"

printf 'restored: %s\n' "$target_path"
printf 'next: set database_url to this new path and validate before touching the formal database\n'
