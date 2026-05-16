# Local Alpha Release Gate

## Scope

This gate is for the Local Product Alpha phase after the validated local MCP `stdio` MVP. The MVP gate in [`../release-gate.md`](../release-gate.md) remains the authority for the technical demo / MVP; this document adds the product alpha checks needed before the repository can be described as a Local Product Alpha.

Current wording must stay conservative until every gate below has fresh evidence: validated local MVP entering productization. Passing the MVP gate alone does not certify Local Alpha, GA, production self-governance, remote write admin, remote team service, or multi-tenancy.

## Minimum Local Alpha Gate

Run the same baseline commands from a clean, reviewable working tree:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

Expected evidence:

- formatting and whitespace checks exit with code `0`
- `cargo clippy` exits with code `0` and no warnings
- `cargo test` exits with code `0`
- `doctor` reports `status = ok`
- `doctor` continues to report `self_revision_write_path = run_reflection`
- `doctor` does not expose provider secrets
- bootstrap documentation remains config-first then doctor-second, and wrapper
  scripts keep the `[serve|doctor|bootstrap-local] [config_path]` contract with
  unsupported modes returning exit code `2`
- `bootstrap-local` must refuse to overwrite an existing config, must copy only
  the safe dev example profile, and must not create secrets, run `doctor`, start
  `serve`, enable daemon behavior, or claim installer / production readiness
- relative `bootstrap-local` targets are documented and tested as repository-root
  relative; cross-directory examples should prefer absolute config paths
- PowerShell runtime parity must have either a Windows runner / Windows machine
  execution record, or an explicit note that local verification only covered
  static script contract assertions because `pwsh` was unavailable

### Fresh Evidence: 2026-05-16 Formal Product Readiness Slice

Ran from branch `codex/support-bundle-log-excerpts` in an isolated
worktree:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

Result:

- `cargo fmt --check`: passed
- `git diff --check`: passed
- `cargo clippy --all-targets --all-features -- -D warnings`: passed
- `cargo test`: passed, 194 tests
- `./scripts/agent-llm-mm.sh doctor`: passed with `status = ok`
- `doctor.self_revision_write_path = "run_reflection"`
- `doctor.daemon_observe_only.writes_allowed = false`

This evidence predates the `bootstrap-local` first-run helper and is retained as
historical formal product readiness evidence, not as proof of the expanded
first-run bootstrap contract.

### Fresh Evidence: 2026-05-16 First-Run Bootstrap Slice

Ran from branch `codex/install-first-run-capability` in an isolated worktree:

```bash
cargo test --test bootstrap -v
git diff --check
```

Result:

- `cargo test --test bootstrap -v`: passed, 24 tests
- `git diff --check`: passed

Manual spot checks for this slice passed:

- `bootstrap-local` created a missing target from
  `examples/agent-llm-mm.dev.example.toml`
- `bootstrap-local` refused existing targets
- `bootstrap-local` refused missing parent directories
- `bootstrap-local` refused dangling symlink targets on Unix
- relative targets resolved from the repository root, not the caller working
  directory
- printed bash next commands quoted paths with spaces

Windows / PowerShell limitation: if the verification host lacks `pwsh`, this
slice can only cover the PowerShell wrapper through static contract assertions;
Windows runner or Windows machine evidence remains required before claiming
runtime parity.

## Product Smoke Gate

Run the local product smoke script from the repository root with the repo-relative script path:

```bash
./scripts/product-smoke-local.sh [config_path]
```

From another working directory, invoke the same script by absolute path:

```bash
/path/to/agent-llm-mm/scripts/product-smoke-local.sh [config_path]
```

If you pass `config_path` from another working directory, use an absolute config
path. The repo-relative example below assumes you are already in the repo root;
the absolute-path example below applies from any working directory.

Required evidence:

- the script exits with code `0`
- `doctor` exits with code `0`
- when `[config_path]` is provided, the path exists, is resolved to an absolute path, and is passed to `doctor`
- the deterministic self-revision demo wrapper generates artifacts in a staging directory and promotes them to `target/reports/self-revision-demo/latest` only after the same smoke run passes the artifact checks
- all 8 required self-revision demo artifacts listed in the Self-Revision Evidence Gate are present and non-empty

### Fresh Evidence: 2026-05-16

Ran from branch `codex/support-bundle-log-excerpts`:

```bash
rm -rf target/reports/self-revision-demo/latest
./scripts/product-smoke-local.sh
```

Result:

- product smoke exited with code `0`
- `doctor` ran as part of product smoke and returned `status = ok`
- deterministic self-revision demo ran in a staging directory
- staging artifacts were promoted to `target/reports/self-revision-demo/latest`
- the required 8 release artifacts were present and non-empty:
  - `doctor.json`
  - `snapshot-before.json`
  - `snapshot-after.json`
  - `decision-before.json`
  - `decision-after.json`
  - `timeline.json`
  - `sqlite-summary.json`
  - `report.md`

Generated demo support files in `latest`, such as `demo.sqlite` and
`agent-llm-mm.demo.toml`, remain local artifacts and are not support-bundle
shareables.

Important limitation: `[config_path]` applies only to `doctor`. `scripts/run-self-revision-demo.sh` currently accepts only an output directory, so the product smoke script keeps the existing deterministic demo contract and does not pass a config path to the demo wrapper.

This gate proves the current local wrapper path, optional config bootstrap health, and the self-revision demo evidence chain. It does not prove fresh-machine install, guided config profiles, backup/restore, GA readiness, production self-governance, remote write admin, remote team service, or multi-tenancy.

## Data Lifecycle Gate

Local Alpha data lifecycle rules are documented in
[`data-lifecycle.md`](data-lifecycle.md). Product evidence must keep formal,
test, and demo SQLite data separated by explicit `database_url` values.

Required boundary:

- formal data, manual test data, and demo data use separate SQLite files
- backup uses a local SQLite backup or conservative copy helper before schema
  migration or formal data path changes
- restore defaults to a new path first; never overwrite a formal database as
  the first restore step
- support bundle summaries are not full database exports
- demo SQLite artifacts are demo evidence, not formal user data export
- migration work validates against a test or restored database before changing
  the formal `database_url`

This gate does not certify fresh-machine install, remote backup service,
multi-tenant data lifecycle, or production disaster recovery.

## Support Bundle Gate

Local Alpha support bundle behavior now has a first local-only generator:

```bash
./scripts/generate-support-bundle.sh <output_dir> [config_path] [--log-file <path>]
```

Review [`support-bundle-local-alpha.md`](support-bundle-local-alpha.md) before
sharing debugging material. The generator is evidence for local diagnostic
packaging, not evidence that Local Alpha, production support, or remote upload
flows are complete.

Required boundary:

- `<output_dir>` must not exist yet or must be empty; generation must fail before
  writing bundle artifacts if the requested directory already contains files
- allowed contents are limited to redacted `doctor` shape, redacted config shape,
  bounded operation summaries, release metadata, product smoke evidence summary,
  bounded explicit local log excerpts, and bundle manifest metadata
- support bundle generation must not call normal runtime bootstrap, create or
  migrate SQLite databases, or seed default identity / commitments
- operation summaries must use read-only local SQLite access and mark themselves
  unavailable when the database or `operation_log` table is absent
- local log excerpts must be generated only from an explicit `--log-file <path>`
  argument; support bundle generation must not scan default log directories,
  home directories, system logs, browser profiles, SSH directories, shell
  history, or `target/` output
- local log excerpts must be bounded and redacted, must omit raw provider
  payloads, prompt text, request bodies, response bodies, tool arguments, cookies,
  browser session material, local private paths, provider URL userinfo/query
  values, API keys, bearer values, tokens, passwords, and secrets, and must not
  copy the raw `.log` file
- oversized explicit log files must be read from a bounded tail window, discard
  partial first retained lines, and mark tail-scoped line numbers instead of
  implying original file line numbers
- excluded contents include API keys, `Authorization` / `Bearer` values, raw
  provider payloads with secrets, full SQLite databases by default, unredacted
  TOML files, provider URL userinfo/query secrets, SSH keys, cookies, and
  browser session data
- support bundle tests must prove redaction, bounded operation summaries, and
  bounded explicit local log excerpts
- no support bundle flow may upload data or claim production support readiness

Recommended verification when support bundle behavior changes:

```bash
cargo test --test support_bundle -v
bash -n scripts/generate-support-bundle.sh
rm -rf target/support-bundles/manual-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
find target/support-bundles/manual-check -maxdepth 1 -type f -print | sort
rg -n 'api_key|api-key|x-api-key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///|token=' target/support-bundles/manual-check || true
find target/support-bundles/manual-check \( -name '*.sqlite' -o -name '*.toml' -o -name '*.log' \) -print
```

### Fresh Evidence: 2026-05-16

Ran from branch `codex/support-bundle-log-excerpts`:

```bash
rm -rf target/support-bundles/local-alpha-gate
./scripts/generate-support-bundle.sh target/support-bundles/local-alpha-gate
find target/support-bundles/local-alpha-gate -maxdepth 1 -type f -print | sort
rg -n 'api_key|api-key|x-api-key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///|token=' target/support-bundles/local-alpha-gate || true
find target/support-bundles/local-alpha-gate \( -name '*.sqlite' -o -name '*.toml' -o -name '*.log' \) -print
```

Result:

- support bundle generation exited with code `0`
- generated files were limited to:
  - `config-shape.json`
  - `doctor.json`
  - `local-log-excerpts.json`
  - `manifest.json`
  - `operation-summaries.json`
  - `product-smoke-summary.json`
  - `release-metadata.json`
- sensitive-token search produced no unredacted secret hits
- `.sqlite`, `.toml`, and raw `.log` exclusion check produced no files

## Correlation ID Gate

Runtime observability must keep MCP calls traceable without creating a new
semantic write path. Review
[`correlation-id-contract.md`](correlation-id-contract.md) when changing MCP
tool handlers, dashboard projection, operation-log runtime wiring, or support
bundle summaries.

Required evidence:

- dashboard event detail exposes `correlation_id`
- each MCP `tools/call` gets a generated `mcp-tool-call-<uuid-v4>` correlation ID
- distinct MCP calls get distinct correlation IDs
- best-effort auto-reflection dashboard diagnostics reuse the triggering MCP
  call correlation ID
- successful MCP tool calls append operation-log metadata with the same
  correlation ID
- local dashboard `GET /api/operation-log` can read durable operation-log
  entries by bounded filters such as `correlation_id`, with a default and
  maximum list limit of 100 entries, without changing the existing live
  `/api/events` in-memory recorder
- correlation metadata does not write identity, commitments, claims, or
  reflections outside governed `run_reflection`

## Self-Revision Evidence Gate

For Local Alpha release evidence, use the Product Smoke Gate above. It runs the
existing self-revision demo wrapper through `scripts/product-smoke-local.sh`,
validates the required artifacts in a staging directory, and promotes them to
`latest` only after the checks pass.

For manual demo-only diagnostics outside a release gate, write to a timestamped
or scratch directory instead of `latest`:

```bash
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/manual-$(date +%Y%m%d-%H%M%S)
```

Required artifacts under `target/reports/self-revision-demo/latest`:

- `doctor.json`
- `snapshot-before.json`
- `snapshot-after.json`
- `decision-before.json`
- `decision-after.json`
- `timeline.json`
- `sqlite-summary.json`
- `report.md`

The 8 artifacts must come from the same successful self-revision demo run. A pre-existing `latest` directory is not sufficient release evidence. The evidence must still show a before / after decision shift, refreshed snapshot state, SQLite summary, and a report that stays within the current MVP boundary.

## Dashboard Local-Only Gate

The dashboard remains a local-only, read-only inspection surface for Local Alpha.

Required boundary:

- bind only to localhost or an explicitly local address
- expose read-only routes for local observation, including bounded live events
  and durable operation-log history for known MCP tool calls whose
  object-shaped arguments reach project handlers
- do not expose write actions from the dashboard
- do not publish the dashboard through a public reverse proxy without a separate gate covering auth, authorization, audit, rollback, and transport risk

Remote/team boundaries are tracked in
[`remote-team-mode-boundary.md`](remote-team-mode-boundary.md) and the local /
remote threat model is tracked in
[`../security/threat-model-local-and-remote.md`](../security/threat-model-local-and-remote.md).
Those documents are planning gates, not evidence that remote/team mode exists.

Recommended verification when dashboard behavior changes:

```bash
cargo test --test dashboard_config --test dashboard_recorder --test dashboard_projection --test dashboard_http
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
cargo test --test mcp_stdio dashboard_exposes_durable_operation_log_history_for_mcp_calls -v
cargo test --test mcp_stdio mcp_tool_failure_appends_failed_operation_log_without_changing_error_semantics -v
cargo test --test mcp_stdio handler_reached_missing_fields_append_failed_operation_log_without_changing_error_semantics -v
cargo test --test mcp_stdio mcp_tool_failure_does_not_persist_provider_error_payload_in_operation_log -v
cargo test --test mcp_stdio dashboard_failed_tool_event_does_not_expose_provider_error_payload -v
cargo test --test mcp_stdio non_object_mcp_tool_arguments_do_not_reach_handler_operation_log -v
```

## Daemon Gate

The daemon remains disabled by default in Local Alpha. `doctor` may report daemon configuration, but it must not start a daemon as part of the preflight check.

Before any daemon write path exists, the daemon must first pass an observe-only gate:

- config defaults keep daemon disabled
- `doctor.daemon_observe_only` reports `mode = "observe_only"`
- `doctor.daemon_observe_only` keeps `write_gate_approved = false`,
  `writes_allowed = false`, and `remote_listener_enabled = false`
- observe-only diagnostics can read local `operation_log` failure/suppression
  candidates without identity, claim, reflection, event, or commitment writes
- observe-only mode does not call `run_reflection`
- daemon-triggered durable writes are blocked until separately gated
- any future daemon write path still uses governed `run_reflection`
- no remote listener or remote trigger ingestion is present
- no background autonomy claim is made from daemon config or doctor output

Recommended verification when daemon observe-only diagnostics change:

```bash
cargo test --test daemon_config -v
cargo test --test operation_log -v
./scripts/agent-llm-mm.sh doctor
```

The detailed gate is
[`daemon-observe-only-gate.md`](daemon-observe-only-gate.md). The older future
daemon trigger policy does not authorize write-capable daemon behavior in Local
Alpha.

## Release Engineering Gate

Release engineering rules are documented in
[`release-engineering.md`](release-engineering.md). For this productization
stage, the conservative release artifact is a source-only Git tag plus
evidence-backed release notes.

Required boundary:

- no binary package, installer, service manager, auto-updater, remote
  bootstrapper, or packaging automation claim is made before a separate gate
- release notes point to the exact evidence directory for the candidate
- compatibility matrix records what was actually checked instead of inferring
  platform parity
- soak evidence is required when runtime, persistence, dashboard, daemon,
  provider, or MCP behavior changes
- deprecations name the deprecated behavior, replacement path, announcement
  candidate, earliest removal candidate, and migration or rollback note

This gate does not certify Beta, GA, production support, remote write admin,
remote team service, or multi-tenancy.

## Product Wording Gate

Before declaring Local Alpha complete, public docs and release notes must not claim:

- GA or production-ready status
- production self-governance
- remote write admin
- multi-tenancy
- remote team service
- all-entry automatic self-revision
- replacement of `run_reflection` as the durable identity / commitment write path

Allowed wording before this gate passes:

- validated local MVP entering productization
- Local Product Alpha in progress
- local MCP `stdio` memory service
- controlled self-revision MVP with `run_reflection` as the durable write path

Allowed wording after this gate passes:

- Local Product Alpha
- local-only product alpha
- installable local alpha, if the install/bootstrap and product smoke evidence are also complete

## Release Decision

Local Alpha is not complete unless all sections above have fresh evidence. If a section lacks fresh evidence, record it as an open gate item rather than weakening the gate. The Product Smoke Gate is implemented by `scripts/product-smoke-local.sh`, but it only passes for a release when a current successful run provides the required evidence. If any future gate section remains unimplemented, record that section separately as open.
