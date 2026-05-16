# Release Gate

## Scope

This gate is for the local MCP `stdio` technical demo / MVP. Passing it does not certify production autonomy, remote administration, multi-tenant deployment, or background daemon behavior.

For the next productization stage, use the separate [Local Alpha Release Gate](product/release-gate-local-alpha.md). That product alpha gate does not replace this MVP gate: this file remains the minimum release gate for the technical demo / MVP, while the Local Alpha gate adds install/config/product-smoke, dashboard local-only, daemon observe-only, and product wording checks.

Release engineering rules for version naming, source-only artifact shape,
changelog entries, evidence directories, compatibility matrix, soak evidence,
and deprecation policy live in
[`product/release-engineering.md`](product/release-engineering.md). Those rules
do not replace this gate; they define what must be recorded before a release
candidate is described publicly.

## Minimum Gate

- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `./scripts/agent-llm-mm.sh doctor`

## Release Engineering Gate

- Use a source-only tag or equivalent conservative source artifact for the first
  productization-stage release; do not claim binary packaging, installer,
  hosted service, Beta, GA, or production-ready delivery.
- Record a candidate-specific release evidence directory instead of relying on
  stale `latest` artifacts.
- Record version naming, changelog boundaries, compatibility matrix results,
  soak command results when required, and deprecation status according to
  [`product/release-engineering.md`](product/release-engineering.md).
- Confirm release notes and public docs keep remote write/admin claims blocked:
  no remote write admin, production self-governance, remote team service,
  multi-tenancy, or `run_reflection` replacement claim may be made from this
  gate.

## Self-Revision Evidence Gate

- `cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v`
- `./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest`

Required artifacts:

- `target/reports/self-revision-demo/latest/doctor.json`
- `target/reports/self-revision-demo/latest/snapshot-before.json`
- `target/reports/self-revision-demo/latest/snapshot-after.json`
- `target/reports/self-revision-demo/latest/decision-before.json`
- `target/reports/self-revision-demo/latest/decision-after.json`
- `target/reports/self-revision-demo/latest/timeline.json`
- `target/reports/self-revision-demo/latest/sqlite-summary.json`
- `target/reports/self-revision-demo/latest/report.md`

## Dashboard Gate

- `cargo test --test dashboard_config --test dashboard_recorder --test dashboard_projection --test dashboard_http`
- `cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v`

The dashboard must remain local-only and read-only unless a separate productization plan explicitly changes that boundary.

## Operation Log Gate

- `cargo test --test operation_log -v`

## Daemon Config Gate

- `cargo test --test daemon_config -v`

The daemon must remain disabled by default. Doctor reports config without starting the daemon.

## Failure Interpretation

Sandbox-only failures must be called out separately from code failures. If a command writes outside the workspace or opens a local listener, rerun it in an environment that permits that operation and record both results.
