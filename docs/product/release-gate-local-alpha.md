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

## Product Smoke Gate

Task C2 is expected to add the product smoke command that proves install, config, MCP `stdio` sanity, and dashboard health from a local alpha install path. Until Task C2 is integrated, the Product Smoke Gate is a blocking gap: Local Alpha cannot be marked complete.

Interim diagnostic evidence is:

```bash
./scripts/agent-llm-mm.sh doctor
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

This interim evidence is intentionally narrower than the future product smoke command. It proves the current local wrapper path, config/bootstrap health, and self-revision demo evidence chain, but it does not satisfy the Product Smoke Gate and does not prove fresh-machine install, guided config profiles, backup/restore, or a complete product support path.

## Self-Revision Evidence Gate

Run the existing self-revision demo wrapper and regenerate `latest` in the same verification pass:

```bash
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
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
- daemon-triggered durable writes are blocked until separately gated
- any future daemon write path still uses governed `run_reflection`
- no background autonomy claim is made from daemon config or doctor output

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

Local Alpha is not complete unless all sections above have fresh evidence. If one section is not yet implemented, record that section as a blocking gap rather than weakening the gate.
