#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage:
  backup-sqlite.sh <sqlite-url-or-path> [backup-dir]

Examples:
  ./scripts/backup-sqlite.sh sqlite:///Users/me/agent-llm-mm/formal.sqlite
  ./scripts/backup-sqlite.sh ./target/manual-test.sqlite /tmp/agent-llm-mm-backups

The default backup directory is target/backups/sqlite/ under the repo root.
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
  local input="$1"
  require_non_empty "database path" "$input"

  case "$input" in
    sqlite::memory:|:memory:)
      fail "in-memory SQLite databases cannot be backed up as files"
      ;;
    sqlite://*)
      local path_part="${input#sqlite://}"
      local decoded_path
      require_non_empty "database path" "$path_part"
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

write_checksum() {
  local file="$1"
  local dir
  local base
  dir="$(cd -- "$(dirname -- "$file")" && pwd -P)"
  base="$(basename -- "$file")"

  if command -v shasum >/dev/null 2>&1; then
    (cd -- "$dir" && shasum -a 256 "$base" > "${base}.sha256")
    printf 'checksum: %s\n' "${file}.sha256"
  elif command -v sha256sum >/dev/null 2>&1; then
    (cd -- "$dir" && sha256sum "$base" > "${base}.sha256")
    printf 'checksum: %s\n' "${file}.sha256"
  else
    warn "no shasum or sha256sum found; checksum generation skipped"
  fi
}

reject_unsupported_backup_path_chars() {
  local label="$1"
  local path="$2"

  if [[ "$path" == *\"* || "$path" == *\\* || "$path" == *$'\n'* ]]; then
    fail "${label} must not contain double quotes, backslashes, or newlines: ${path}"
  fi
}

reject_parent_directory_component() {
  local label="$1"
  local path="$2"
  local part

  IFS='/' read -r -a parts <<< "$path"
  for part in "${parts[@]}"; do
    if [[ "$part" == ".." ]]; then
      fail "${label} must not contain '..' path components: ${path}"
    fi
  done
}

path_is_same_or_descendant() {
  local root="$1"
  local candidate="$2"

  if [[ "$candidate" == "$root" ]]; then
    return 0
  fi

  if [[ "$root" == "/" ]]; then
    return 0
  fi

  [[ "$candidate" == "$root"/* ]]
}

resolve_directory_target_path() {
  local path="$1"
  local current="$path"
  local suffix=""
  local parent
  local real

  while [[ ! -e "$current" ]]; do
    suffix="/$(basename -- "$current")${suffix}"
    parent="$(dirname -- "$current")"
    if [[ "$parent" == "$current" ]]; then
      fail "cannot resolve backup directory parent: ${path}"
    fi
    current="$parent"
  done

  if [[ ! -d "$current" ]]; then
    fail "backup directory parent is not a directory: ${current}"
  fi

  real="$(cd -- "$current" && pwd -P)"
  printf '%s%s' "$real" "$suffix"
}

harden_sqlite_file_permissions() {
  local file="$1"
  chmod 600 "$file"
}

remove_file_on_failure() {
  local file="$1"
  rm -f -- "$file"
}

reserve_new_file() {
  local file="$1"

  if ! (set -C; : > "$file") 2>/dev/null; then
    fail "backup file already exists, refusing to overwrite: ${file}"
  fi
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -- "${script_dir}/.." && pwd -P)"

if ! db_path="$(sqlite_input_to_path "$1")"; then
  exit 1
fi
require_non_empty "database path" "$db_path"
reject_unsupported_backup_path_chars "database path" "$db_path"

backup_dir="${2:-${repo_root}/target/backups/sqlite}"
require_non_empty "backup directory" "$backup_dir"
reject_unsupported_backup_path_chars "backup directory" "$backup_dir"
reject_parent_directory_component "backup directory" "$backup_dir"

[[ -f "$db_path" ]] || fail "database file does not exist: ${db_path}"

db_dir="$(cd -- "$(dirname -- "$db_path")" && pwd -P)"
backup_real_dir="$(resolve_directory_target_path "$backup_dir")"
if path_is_same_or_descendant "$db_dir" "$backup_real_dir"; then
  fail "backup directory must be outside the live database directory tree: ${backup_real_dir}"
fi

umask 077
mkdir -p -- "$backup_dir"
backup_real_dir="$(cd -- "$backup_dir" && pwd -P)"
if path_is_same_or_descendant "$db_dir" "$backup_real_dir"; then
  fail "backup directory must be outside the live database directory tree after path normalization: ${backup_real_dir}"
fi

timestamp="$(date +%Y%m%d-%H%M%S)"
db_base="$(basename -- "$db_path")"
reject_unsupported_backup_path_chars "database file name" "$db_base"
backup_file="${backup_real_dir}/${db_base}.${timestamp}.$$.bak"
reject_unsupported_backup_path_chars "backup file path" "$backup_file"
reserve_new_file "$backup_file"

if command -v sqlite3 >/dev/null 2>&1; then
  if ! sqlite3 "$db_path" ".backup \"${backup_file}\""; then
    remove_file_on_failure "$backup_file"
    fail "sqlite backup failed"
  fi
else
  warn "sqlite3 not found; falling back to cp. Stop writers first for a busy live database."
  if ! cp -- "$db_path" "$backup_file"; then
    remove_file_on_failure "$backup_file"
    fail "fallback copy backup failed"
  fi
fi

harden_sqlite_file_permissions "$backup_file"
write_checksum "$backup_file"
printf 'backup: %s\n' "$backup_file"
