# Local Alpha Data Safety Runbook

This runbook is for the current validated local MVP entering productization. It
does not claim GA or production-ready status. The durable identity and
commitments write path remains `run_reflection`; this document only covers local
SQLite file handling for backup and restore.

## Database Separation

Use separate `database_url` values for formal, test, and demo data:

| Purpose | Use | Example shape |
| --- | --- | --- |
| formal | Local data you intend to preserve, inspect, or backup | `sqlite:///Users/<you>/agent-llm-mm/formal/agent-llm-mm.sqlite` |
| test | Manual smoke checks, local debugging, and integration trials | `sqlite:///Users/<you>/agent-llm-mm/test/manual-test.sqlite` |
| demo | Deterministic demos and generated evidence artifacts | `sqlite:///Users/<you>/agent-llm-mm/demo/demo.sqlite` |

Do not reuse one SQLite file across formal, test, and demo workflows. A smoke
test or demo run should never write into the formal database. Set the separation
explicitly in `agent-llm-mm.local.toml` or with `AGENT_LLM_MM_DATABASE_URL`.

## Backup Procedure

The repository provides a conservative backup helper:

```bash
./scripts/backup-sqlite.sh "sqlite:///absolute/path/to/formal.sqlite"
```

By default, backup files are written under:

```text
target/backups/sqlite/
```

That default is intended to keep backups outside normal live database
directories. The script also resolves the live database directory and the backup
directory before writing; if the backup directory is the live database directory
or any descendant under it, the script refuses to continue. A second argument can
select a different backup directory:

```bash
./scripts/backup-sqlite.sh "sqlite:///absolute/path/to/formal.sqlite" /safe/local/backup-dir
```

The script refuses an empty database path, refuses a backup directory inside the
live database directory tree, refuses backup paths containing double quotes,
backslashes, newlines, or `..` path components, and refuses to overwrite an existing backup file. It
accepts either a plain file path or a `sqlite://` file URL; Git Bash style
Windows drive URLs such as `sqlite:///D:/agent-llm-mm/formal.sqlite` are normalized to
`D:/agent-llm-mm/formal.sqlite`. SQLite file URLs use strict percent decoding:
only `%HH` hex escapes are decoded, invalid percent escapes are rejected,
control bytes are rejected, and backslashes or double quotes in SQLite URLs are
rejected instead of being interpreted as escape sequences. When the local
`sqlite3` command is available, the script uses SQLite's online backup command
after the path safety checks have passed.

If `sqlite3` is not available, the script falls back to `cp` and prints a
warning. For a busy live database, prefer running the script where `sqlite3` is
available, or stop the local MCP service before using the fallback copy. Backup
files are hardened to owner read/write permissions after creation, including the
fallback copy path.

When a checksum tool is available, the script writes a `.sha256` file next to the
backup. On macOS it prefers:

```bash
shasum -a 256 "$BACKUP_FILE" > "$BACKUP_FILE.sha256"
```

If `shasum` is missing but `sha256sum` exists, it uses `sha256sum`. If neither is
available, the backup still completes and the script reports that checksum
generation was skipped.

## Restore Procedure

Default restore rule: restore to a new path first, then switch `database_url` to
that new path for validation. Do not restore over the formal database file.

```bash
./scripts/restore-sqlite.sh \
  target/backups/sqlite/formal.sqlite.20260511-120000.12345.bak \
  "sqlite:///absolute/path/to/restore-check/formal-restore.sqlite"
```

The restore script refuses an empty backup path, refuses an empty target path,
and refuses to write when the target path already exists. It accepts either
plain file paths or `sqlite://` file URLs, including Git Bash style Windows drive
URLs. SQLite file URLs use the same strict percent decoder as the backup helper.
It creates the target directory if needed, reserves the target path before
copying, copies the backup into the new database path, hardens the restored file
to owner read/write permissions, and checks a sidecar `.sha256` file when one is
present and a compatible checksum tool is available.

After restore, point a private config at the restored database:

```toml
database_url = "sqlite:///absolute/path/to/restore-check/formal-restore.sqlite"
```

Then run a local validation pass:

```bash
AGENT_LLM_MM_CONFIG=/absolute/path/to/restore-check.toml ./scripts/agent-llm-mm.sh doctor
```

Only after the restored database has been validated should a human decide
whether to update the formal `database_url`. Keep the original formal database
untouched until that decision is made.
