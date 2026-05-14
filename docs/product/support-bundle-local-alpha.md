# Local Alpha Support Bundle

## Scope

This document defines what a Local Product Alpha user may share when asking for
debugging help. The repository now includes a first local-only generator:

```bash
./scripts/generate-support-bundle.sh <output_dir> [config_path]
```

`<output_dir>` must not exist yet or must be empty. The generator rejects a
non-empty directory so stale local TOML, SQLite, log, or scratch files cannot be
accidentally shared as part of the support bundle.

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
  `upload_performed = false`, excluded content list, and file inventory
- `doctor.json`: config-derived safe `doctor` shape, including transport,
  database URL shape, provider kind, URL shape, model, dashboard settings,
  daemon settings, runtime hook names, `self_revision_write_path`,
  `runtime_bootstrap_performed = false`, and `status`
- `config-shape.json`: configuration shape with secrets removed, including
  transport, database URL shape, provider kind, URL shape, model, timeout,
  credential presence as a boolean, dashboard settings, daemon settings, and a
  redacted config file name when one was provided
- `operation-summaries.json`: up to 25 recent durable operation-log metadata
  rows, limited to operation id, timestamp, namespace, entrypoint, kind, status,
  correlation id, and a `read_only` marker; when the local database or
  `operation_log` table is unavailable, this file records `available = false`
  and an unavailable reason instead of creating or migrating the database
- `release-metadata.json`: generated timestamp, git branch, git commit, local
  platform, Rust version, and verification command names
- `product-smoke-summary.json`: whether the latest self-revision demo evidence
  directory contains the required artifacts, plus the expected
  `self_revision_write_path = run_reflection` boundary

The generator must not call the normal runtime bootstrap path. It may inspect
local SQLite operation-log metadata only through a read-only connection. It does
not create or migrate the database, does not seed default identity or
commitments, does not copy `.sqlite` files into the bundle, and does not
serialize raw provider request or response summaries.

## Excluded Contents

A support bundle must not include by default:

- API keys or provider tokens
- `Authorization` headers or `Bearer` values
- raw provider request or response payloads that may contain secrets
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

Redaction should preserve structure while replacing values with a stable marker
or shape, for example `<redacted>`, `sqlite://<local-path>`, or a provider URL
without userinfo or query values. It should not delete whole sections unless the
section itself is secret-only.

## Current Test Coverage

`tests/support_bundle.rs` currently proves:

- the generator creates the expected six JSON files
- non-empty output directories are rejected before bundle artifacts are written
- provider credentials are represented as `credential_configured = true`, not as
  secret values
- SQLite database paths are generalized to `sqlite://<local-path>`
- provider URL userinfo and query secrets are removed from the emitted URL shape
- bundle output does not contain `Authorization`, `Bearer`, or the serialized
  provider secret indicators used by the test fixture
- no `.sqlite` file is copied into the bundle
- recent operation summaries are bounded to 25 entries and omit request /
  response payload summaries
- missing SQLite databases are not created or bootstrapped; operation summaries
  are marked unavailable instead
- `doctor.json` records `runtime_bootstrap_performed = false`
- the shell script delegates to the dedicated `generate_support_bundle` binary
  and creates relative output parents before running it

## Remaining Gaps

The first generator does not make support packaging complete for a formal
product release. Remaining Local Alpha gaps include:

- no local log excerpts are included yet; logs stay excluded until log locations
  and log redaction rules are stable
- the full Local Alpha release gate still needs fresh evidence for the same
  release candidate
- observe-only daemon diagnostics remain a later task
- this bundle is not a remote upload flow, support ticket integration, or
  production support readiness claim

## Verification

Use these commands when changing support bundle behavior:

```bash
cargo test --test support_bundle -v
bash -n scripts/generate-support-bundle.sh
rm -rf target/support-bundles/manual-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
find target/support-bundles/manual-check -maxdepth 1 -type f -print | sort
rg -n 'api_key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///' target/support-bundles/manual-check || true
find target/support-bundles/manual-check \( -name '*.sqlite' -o -name '*.toml' \) -print
```

The sensitive-term scan should return no bundle leaks. The final `find` command
should print no SQLite database or TOML files.
