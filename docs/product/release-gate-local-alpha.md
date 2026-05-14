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
- bootstrap documentation remains doctor-first, and wrapper scripts keep the
  `[serve|doctor] [config_path]` contract with unsupported modes returning exit
  code `2`

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

Important limitation: `[config_path]` applies only to `doctor`. `scripts/run-self-revision-demo.sh` currently accepts only an output directory, so the product smoke script keeps the existing deterministic demo contract and does not pass a config path to the demo wrapper.

This gate proves the current local wrapper path, optional config bootstrap health, and the self-revision demo evidence chain. It does not prove fresh-machine install, guided config profiles, backup/restore, GA readiness, production self-governance, remote write admin, remote team service, or multi-tenancy.

## Support Bundle Gate

Local Alpha support bundle behavior is currently a design contract, not an
implemented generator. Review
[`support-bundle-local-alpha.md`](support-bundle-local-alpha.md) before sharing
debugging material.

Required boundary:

- allowed contents are limited to `doctor` output, redacted config shape, bounded
  operation summaries after durable log wiring exists, log excerpts after log
  locations are stable, release metadata, and product smoke evidence summaries
- excluded contents include API keys, `Authorization` / `Bearer` values, raw
  provider payloads with secrets, full SQLite databases by default, unredacted
  TOML files, SSH keys, cookies, and browser session data
- future automation must have redaction tests before it is treated as a product
  feature
- no support bundle flow may upload data or claim production support readiness

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
- expose read-only routes for local observation
- do not expose write actions from the dashboard
- do not publish the dashboard through a public reverse proxy without a separate gate covering auth, authorization, audit, rollback, and transport risk

Recommended verification when dashboard behavior changes:

```bash
cargo test --test dashboard_config --test dashboard_recorder --test dashboard_projection --test dashboard_http
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
```

## Daemon Gate

The daemon remains disabled by default in Local Alpha. `doctor` may report daemon configuration, but it must not start a daemon as part of the preflight check.

Before any daemon write path exists, the daemon must first pass an observe-only gate:

- config defaults keep daemon disabled
- observe-only mode can run without identity or commitment writes
- observe-only mode does not call `run_reflection`
- daemon-triggered durable writes are blocked until separately gated
- any future daemon write path still uses governed `run_reflection`
- no remote listener or remote trigger ingestion is present
- no background autonomy claim is made from daemon config or doctor output

The detailed gate is
[`daemon-observe-only-gate.md`](daemon-observe-only-gate.md). The older future
daemon trigger policy does not authorize write-capable daemon behavior in Local
Alpha.

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
