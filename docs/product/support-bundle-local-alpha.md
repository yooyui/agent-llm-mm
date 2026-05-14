# Local Alpha Support Bundle Design

## Scope

This document defines what a Local Product Alpha user may share when asking for
debugging help. It is a design contract, not an implemented bundle generator.
Do not add a support-bundle script until durable operation-log wiring and log
locations are stable enough to test redaction end to end.

The bundle is local-first. It must not upload data, start remote diagnostics, or
claim a production support channel.

## Allowed Contents

A future support bundle may include:

- `doctor` JSON output from the same local run, including transport,
  `database_url` shape, provider kind, dashboard settings, daemon settings,
  runtime hook names, `self_revision_write_path`, and `status`
- configuration shape with secrets redacted, such as section names, enabled
  flags, host/port choices, and placeholder provider fields
- recent operation summaries after durable operation-log read paths are wired
  into runtime calls
- local log excerpts only after log locations and redaction rules are stable
- release metadata such as git commit, branch name, platform, Rust version, and
  relevant test command names
- product smoke evidence summary, including whether required artifacts exist and
  whether `doctor` still reports `self_revision_write_path = run_reflection`

## Excluded Contents

A support bundle must not include by default:

- API keys or provider tokens
- `Authorization` headers or `Bearer` values
- raw provider request or response payloads that may contain secrets
- full user SQLite databases
- unredacted local TOML files
- private prompt text unless the user explicitly extracts and redacts it
- remote host credentials, SSH keys, cookies, or browser session data

If a maintainer needs a SQLite database to reproduce a bug, that must be a
separate explicit action with a separate redaction and backup decision. The
default support bundle remains metadata and summaries only.

## Redaction Terms

Future tooling and tests must treat these terms as sensitive indicators:

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

Redaction should preserve structure while replacing values with a stable marker,
for example `<redacted>`. It should not delete whole sections unless the section
itself is secret-only.

## Expected Future Tests

Before implementing a bundle script, add tests proving:

- serialized `doctor` output does not expose provider API keys
- TOML redaction preserves section names and removes secret values
- support bundle output never includes `Authorization`, `Bearer`, `api_key`, or
  raw provider payload secrets
- the default bundle excludes full SQLite database files
- recent operation summaries use bounded, redacted fields only

Until those tests exist, this document is the gate contract and not evidence that
support bundle automation is complete.

## Release Gate Position

For Local Alpha, this design is sufficient only to document what can be safely
shared. A release may not claim a complete automated support package until a
script, redaction tests, operation-log source contract, and log-location contract
are implemented and verified.
