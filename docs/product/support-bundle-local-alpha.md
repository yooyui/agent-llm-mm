# Local Alpha Support Bundle

## Scope

This document defines what a Local Product Alpha user may share when asking for
debugging help. The repository now includes a first local-only generator:

```bash
./scripts/generate-support-bundle.sh <output_dir> [config_path] [--log-file <path>] [--correlation-id <id>]
```

`<output_dir>` must not exist yet or must be empty. The generator rejects a
non-empty directory so stale local TOML, SQLite, log, or scratch files cannot be
accidentally shared as part of the support bundle.

`--log-file <path>` is optional and explicit. The generator must not discover
logs by scanning the repository, the user's home directory, browser profiles,
SSH directories, system logs, shell history, or `target/` output. When a log file
is requested, the bundle may include only a bounded, redacted JSON summary and a
small number of safe excerpts. It must not copy the original `.log` file.

`--correlation-id <id>` is also optional and explicit. When provided, it filters
`operation-summaries.json` to operation-log metadata rows that match that
generated MCP correlation ID. The accepted value must use the generated
canonical `mcp-tool-call-<uuid-v4>` shape so arbitrary secret-like text is not
copied into the support bundle filter metadata.

The bundle is still a Local Alpha diagnostic aid, not a production support
channel. It must not upload data, start remote diagnostics, copy the full user
database, or create a new durable write path. `run_reflection` remains the only
durable identity / commitments / reflection write path.

Local Alpha itself is still in progress until the full release gate has fresh
evidence.

## Generated Files

The current generator writes these JSON files into the requested output
directory:

- `manifest.json`: bundle format, generated timestamp, local-only flag,
  `artifact_scope = "local-only-diagnostic-artifact"`,
  `upload_performed = false`, `production_support_channel = false`,
  `remote_support_surface = false`, explicit `safety_checks`, excluded content
  list, and file inventory
- `doctor.json`: config-derived safe `doctor` shape, including transport,
  database URL shape, provider kind, URL shape, model, dashboard settings,
  daemon settings, runtime hook names, `self_revision_write_path`,
  `runtime_bootstrap_performed = false`, and `status`
- `config-shape.json`: configuration shape with secrets removed, including
  transport, database URL shape, provider kind, URL shape, model, timeout,
  credential presence as a boolean, dashboard settings, daemon settings, and a
  redacted config file name when one was provided; secret-like config file names
  are collapsed to `<local-path>/<redacted-name>`
- `operation-summaries.json`: up to 25 recent durable operation-log metadata
  rows, limited to safe-shaped operation id, timestamp, namespace, entrypoint,
  `operation_kind`, status, correlation id, an optional
  `filter.correlation_id`, and a `read_only` marker; when
  `--correlation-id <id>` is provided, only matching generated MCP
  correlation IDs are exported; when the local database or `operation_log` table
  is unavailable, this file records `available = false` and an unavailable
  reason instead of creating or migrating the database; user/project namespace
  values are shape-only (`project/<namespace>` / `user/<namespace>`), and
  secret-like operation metadata is replaced with `<redacted-metadata>`
- `release-metadata.json`: generated timestamp, git branch, git commit, local
  platform, Rust version, and verification command names
- `product-smoke-summary.json`: whether the latest self-revision demo evidence
  directory contains the required artifacts, plus the expected
  `self_revision_write_path = run_reflection` boundary
- `local-log-excerpts.json`: local log availability, source shape, redaction
  version, retained line count, line-number scope, truncation marker, hard
  bounds, and at most a small number of redacted recent excerpts when
  `--log-file <path>` was explicitly provided; when no log file is requested,
  this file records `available = false` instead of scanning for logs; secret-like
  explicit log file names are collapsed to `<local-path>/<redacted-name>`

The generator must not call the normal runtime bootstrap path. It may inspect
local SQLite operation-log metadata only through a read-only connection. It does
not create or migrate the database, does not seed default identity or
commitments, does not copy `.sqlite` files into the bundle, and does not
serialize raw provider request or response summaries.

`manifest.json` now records the support-bundle boundary as machine-readable
`safety_checks`: `read_only = true`, `runtime_bootstrap_performed = false`,
`sqlite_files_included = false`, `toml_files_included = false`,
`raw_log_files_included = false`, and `provider_payloads_included = false`.
Together with `artifact_scope`, `production_support_channel = false`, and
`remote_support_surface = false`, these fields are evidence markers only; they
do not turn the support bundle into a remote upload channel, team support
surface, remote management surface, or production support workflow.

## Local Log Excerpts

Local log excerpts are allowed only through an explicit `--log-file <path>`
argument. This is a diagnostic read surface, not runtime logging setup. The
support bundle generator must not create, rotate, upload, or retain logs outside
the requested output directory.

The log summary must be bounded and conservative:

- source shape is limited to the file name or `<local-path>/<file-name>`
- input reading is capped by a maximum byte count
- excerpt selection is capped by tail-line, excerpt-count, per-excerpt character,
  and total excerpt byte bounds
- oversized logs are read from a bounded tail window, discard the first partial
  retained line, and mark `line_number_scope = "tail"`; untruncated logs use
  `line_number_scope = "file"`
- excerpts are recent and short; the raw `.log` file is never copied
- JSON provider payloads, prompt text, request bodies, response bodies, tool
  arguments, cookies, browser session material, local private paths, provider URL
  userinfo/query values, API keys, bearer values, tokens, passwords, and secrets
  must be redacted or omitted
- raw provider payload labels and raw diagnostic labels, including
  `provider_payload`, provider request/response body labels,
  `provider_diagnostic`, `raw_diagnostics`, and `diagnostic_summary_json`, must
  be omitted rather than excerpted

If the explicit log path is missing, unreadable, too large, or otherwise
unavailable, the Rust generator records `available = false` and a reason instead
of starting services or scanning fallback locations. The shell wrapper forwards
missing explicit `--log-file` paths so the bundle can record that unavailable
state in `local-log-excerpts.json`.

## Excluded Contents

A support bundle must not include by default:

- API keys or provider tokens
- `Authorization` headers or `Bearer` values
- cookies, session IDs, or browser session material
- raw provider request or response payloads that may contain secrets
- prompt text, request bodies, response bodies, or tool arguments from provider
  log lines
- full user SQLite databases
- unredacted local TOML files
- private prompt text unless the user explicitly extracts and redacts it
- remote host credentials, SSH keys, cookies, or browser session data
- local file-system paths for SQLite databases or config parents
- provider URL userinfo or query strings

If a maintainer needs a SQLite database to reproduce a bug, that must be a
separate explicit action with a separate redaction and backup decision. The
default support bundle remains metadata and summaries only.

## Redaction Terms

Tooling and tests treat these terms as sensitive indicators:

- `api_key`
- `API key`
- `authorization`
- `Authorization`
- `bearer`
- `Bearer`
- `token`
- `secret`
- `password`
- `provider_token`
- `openai_api_key`
- `AGENT_LLM_MM_CONFIG`
- `AGENT_LLM_MM_DATABASE_URL` when it contains a private local path that should
  be generalized before sharing
- `sk-`-prefixed values and secret-like local file names

Redaction should preserve structure while replacing values with a stable marker
or shape, for example `<redacted>`, `sqlite://<local-path>`, or a provider URL
without userinfo or query values. It should not delete whole sections unless the
section itself is secret-only.

## Current Test Coverage

`tests/support_bundle.rs` currently proves:

- the generator creates the expected seven JSON files
- `manifest.json` includes explicit safety checks for read-only generation, no
  runtime bootstrap, no SQLite/TOML/raw-log copies, and no provider payloads
- non-empty output directories are rejected before bundle artifacts are written
- provider credentials are represented as `credential_configured = true`, not as
  secret values
- SQLite database paths are generalized to `sqlite://<local-path>`
- provider URL userinfo and query secrets are removed from the emitted URL shape
- bundle output does not contain `Authorization`, `Bearer`, or the serialized
  provider secret indicators used by the test fixture
- no `.sqlite` file is copied into the bundle
- no raw `.log` file is copied into the bundle
- operation summaries can be explicitly filtered by generated MCP
  `mcp-tool-call-<uuid-v4>` correlation ID
- non-generated, non-canonical, or non-v4 correlation ID filter values are
  rejected before the bundle output directory is created
- secret-like operation metadata and user/project namespace values are redacted
  or reduced to stable shapes before writing `operation-summaries.json`
- explicit local log excerpts are bounded, redacted, and written only as
  `local-log-excerpts.json`
- secret-like explicit config/log file names are redacted before being written
  to `config-shape.json` or `local-log-excerpts.json`
- missing or unrequested local logs are represented as unavailable instead of
  triggering directory scans or runtime bootstrap
- recent operation summaries are bounded to 25 entries and omit request /
  response / diagnostic payload summaries
- raw provider payload and raw diagnostic log labels are skipped even when their
  values do not contain obvious secret markers
- `manifest.json` marks the bundle as a local-only diagnostic artifact and
  explicitly records no production support channel or remote support surface
- missing SQLite databases are not created or bootstrapped; operation summaries
  are marked unavailable instead
- `doctor.json` records `runtime_bootstrap_performed = false`
- the shell script delegates to the dedicated `generate_support_bundle` binary
  and creates relative output parents before running it

## Remaining Gaps

The generator does not make support packaging complete for a formal product
release. Remaining Local Alpha gaps include:

- local log excerpts now require explicit `--log-file <path>` input and still do
  not establish an automatic stable runtime log location
- the full Local Alpha release gate still needs fresh evidence for the same
  release candidate
- observe-only daemon diagnostics remain separate from support packaging
- this bundle is not a remote upload flow, support ticket integration, or
  production support readiness claim

## Verification

Use these commands when changing support bundle behavior:

```bash
cargo test --test support_bundle -v
bash -n scripts/generate-support-bundle.sh
rm -rf target/support-bundles/manual-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
rm -rf target/support-bundles/manual-correlation-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-correlation-check --correlation-id mcp-tool-call-018fbc89-9ac1-4f5d-8b2a-1f6f5f27b205
find target/support-bundles/manual-check -maxdepth 1 -type f -print | sort
rg -n 'api_key|api-key|x-api-key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///|token=' target/support-bundles/manual-check || true
find target/support-bundles/manual-check \( -name '*.sqlite' -o -name '*.toml' -o -name '*.log' \) -print
```

The sensitive-term scan should return no bundle leaks. The final `find` command
should print no SQLite database, TOML, or raw log files.
