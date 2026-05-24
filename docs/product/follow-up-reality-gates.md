# Productization Follow-Up Reality Gates

This document tracks productization work that still has assumption, simulation,
planning-only, or not-yet-merged status. It is intentionally stricter than the
roadmap: a module is treated as complete only when the implementation, fresh
evidence, and product wording all line up.

Current baseline:

- branch at audit start: `dev-work` / `669dbe3`
- follow-up branch: `codex/follow-up-reality-gates`
- carried implementation: `local-alpha-evidence-summary` in current branch
  commit `cff4a7a`
- `run_reflection` remains the only durable identity / commitment / reflection
  write path
- Local Product Alpha is still in progress until every release gate has fresh,
  reviewable evidence and a human release decision

## Status Labels

| Label | Meaning |
| --- | --- |
| `implemented` | Code, tests, docs, and fresh evidence are aligned in the current branch. |
| `implemented-unmerged` | Implemented in an isolated branch, but not yet part of the current mainline branch. |
| `partial` | Useful code exists, but product behavior, evidence, or coverage is incomplete. |
| `simulation-only` | Local simulation exists, but real environment evidence is still missing. |
| `planning-gate` | Boundary or design docs exist, but no product implementation is present. |
| `not-implemented` | The module is still future work. |

## Follow-Up Modules

| Priority | Module | Current Status | Assumption / Gap | Required Follow-Up | Evidence To Accept |
| --- | --- | --- | --- | --- | --- |
| `P0` | Local Alpha evidence summary | `implemented` on `codex/follow-up-reality-gates` | Mainline previously had no single read-only rollup to distinguish open, not-verified, and satisfied gates. | Keep `scripts/local-alpha-evidence-summary.sh`, `src/bin/local_alpha_evidence_summary.rs`, `src/support/local_alpha_evidence.rs`, and `tests/local_alpha_release_evidence.rs` in the productization branch; run it after every release-gate refresh. | `cargo test --test local_alpha_release_evidence -v`; `cargo run --quiet --bin local_alpha_evidence_summary -- --evidence-root .` |
| `P0` | Local Alpha release-gate refresh | `implemented` on `codex/follow-up-reality-gates` | Manual refresh could skip product smoke, first-run simulation, support bundle, or summary output, leaving stale gate evidence. | Use `scripts/local-alpha-release-gate-refresh.sh [config_path]` to refresh the locally reproducible gate slice; keep missing real fresh-machine, Windows runner, remote/team, and release-decision evidence open. | `bash -n scripts/local-alpha-release-gate-refresh.sh`; `cargo test --test local_alpha_release_evidence -v`; optional manual run writes `target/reports/local-alpha/evidence-summary.json`. |
| `P0` | Local Alpha full release gate | `partial` | Product smoke, first-run simulation, support bundle, and summary can now be refreshed together locally, but Windows parity, real fresh-machine evidence, and release decision evidence can still be missing. | Run the refresh script for local evidence, then separately record real fresh-machine and Windows runner evidence without claiming completion from local simulation artifacts. | `./scripts/local-alpha-release-gate-refresh.sh`; evidence summary JSON showing every gate state; separate real fresh-machine and Windows parity summaries before human release review. |
| `P0` | Fresh-machine first-run | `simulation-only` | `first-run-bootstrap-smoke-local.sh` proves an isolated local simulation, not a real clone/unpack on a fresh machine. | Run the documented bootstrap -> doctor path in a clean checkout or real fresh-machine environment; record the summary separately from simulation evidence. | A dated `first-run-bootstrap/summary.json` or documented equivalent with `real_fresh_machine_evidence = true`, `doctor_status = ok`, `local_only = true`, no serve/product smoke side effects, and `run_reflection` write path. |
| `P0` | Windows parity | `planning-gate` | PowerShell/static contract tests do not prove Windows runtime parity. | Run Windows or Windows runner validation for bootstrap, doctor, and product smoke parity; do not infer Windows from macOS. | `windows-parity/summary.json` or `target/windows-parity/local-alpha-gate/summary.json` with `status = verified`, `runtime_parity = true`, and a Windows runner/platform marker. |
| `P1` | Plan/status synchronization | `partial` | Planning checkboxes, test counts, and status docs can drift from code. The current branch implements only the read-only `cargo test` total drift slice, not full roadmap/task-state reconciliation. | Use `status-sync-check` after each productization slice, then reconcile any flagged docs against commands run in the current branch. Keep broader plan/status state review manual until a separate tracker verifier exists. | `cargo test --test status_sync -v`; `./scripts/status-sync-check.sh`; matching test counts, dated verification commands, and no stale claims such as old test totals or completed workstream labels without evidence. |
| `P1` | Observe-only daemon lifecycle | `partial` | `doctor.daemon_observe_only` is real, and `DaemonHandle` now has local start / stop proof, but it is still not connected as a running background product service. | Keep write capability blocked; use lifecycle tests to preserve clean shutdown and closed write / remote gates before any future daemon write path. | `cargo test --test daemon_config -v`; tests prove disabled quick exit, observe-only start / stop, no remote listener, no semantic writes, and clean shutdown. |
| `P1` | Runtime self-revision coverage | `partial` | Four hooks are wired, but this is not all-entry auto-reflection or continuous autonomy. | Tighten diagnostics and tests for the existing four hooks before widening trigger coverage. | Focused `mcp_stdio`, `failure_modes`, and `bootstrap` tests showing hook list, opt-in requirements, best-effort failure semantics, and no direct `run_reflection` recursion. |
| `P1` | Support bundle diagnostics | `partial` | Local support bundle is useful, but not a production support channel, remote upload flow, or automatic log collector. | Keep explicit log file and correlation filters; extend only after redaction tests exist. Current manifest must keep `safety_checks` explicit for read-only generation, no runtime bootstrap, no SQLite/TOML/raw log copies, and no provider payloads. | `cargo test --test support_bundle -v`; secret scans on generated bundle; `manifest.json` says local only, upload false, and every safety check remains closed for copied raw artifacts. |
| `P2` | Structured decision protocol | `partial` | `decide_with_snapshot` now has a v1 response envelope with protocol metadata, status, reason, and gate metadata, but the model decision remains a minimal action string. This is not a full decision engine. | Preserve legacy `blocked` / `decision` callers while adding any future schema fields. Do not add provider JSON decisions, confidence scoring, planning, or policy arbitration without separate tests and docs. | `docs/product/structured-decision-protocol.md`; `cargo test --test decision_flow -v`; future MCP schema compatibility checks when the wire schema changes. |
| `P2` | Evidence and reflection semantics v2 | `partial` | Current queries are bounded narrowing only; no richer relation, weight, ranking, or widening engine. | Add one evidence-v2 slice at a time while preserving no-widening governance. | Focused `sqlite_store`, `failure_modes`, `evidence_query_dto`, and `application_use_cases` tests. |
| `P2` | Provider matrix | `partial` | Only `mock` and first `openai-compatible` are runnable. The current branch exposes a read-only matrix in config and `doctor`; `azure-openai`, `openrouter`, and `local` remain planned-only and non-configurable. | Add adapters only with config, doctor, parsing, redaction, timeout/error-mode, and MCP stdio path tests. Do not turn matrix rows into accepted config values without a real adapter in the same change. | `docs/provider-contract.md`; `cargo test --test provider_config -v`; provider-specific tests before any future provider is marked supported. |
| `P2` | Data lifecycle and migration | `partial` | Backup/restore local scripts are covered, but this is not remote backup, cloud sync, scheduled backup, or production disaster recovery. | Add migration tests when schema changes; keep restore-to-new-path and manual database switch boundary. | `cargo test --test sqlite_backup_restore -v`; schema migration tests for any future migration. |
| `P3` | Remote/team/auth/security | `planning-gate` | Boundary and threat model docs exist, but remote/team mode is not implemented. | Start with remote read-only inventory and auth/audit design; block remote writes until separate security gates pass. | Security review, route inventory, auth/authorization tests, audit tests, and no write-capable remote routes before gate approval. |
| `P3` | Release engineering | `partial` | Rules and a local release soak runner exist, but there is still no installer, binary package, service manager, auto-updater, Windows/fresh-machine release evidence, compatibility matrix automation, or release decision workflow. | Use `scripts/release-soak-local.sh <candidate-name> [config_path]` for local candidate evidence, keep first artifact source-only, and add real compatibility matrix / release decision evidence before Beta language. | `bash -n scripts/release-soak-local.sh`; `cargo test --test local_alpha_release_evidence release_soak -v`; candidate-specific `target/reports/releases/<candidate-name>/` with command logs, summary, support-bundle scans, SHA-256 manifests, and Local Alpha evidence summary. |
| `P3` | Multi-layer memory | `planning-gate` | Roadmap exists; working / episodic / semantic / procedural memory layers are not product implementation. | Wait until schema, evidence policy, data lifecycle, and migration gates stabilize; then implement one small richer episode slice. | New specs and tests for the first slice only; no Local Alpha claim that multi-layer memory is complete. |

## Required Review Loop

For every follow-up module:

1. Add or update a failing test before behavior changes.
2. Implement the smallest passing slice.
3. Update the corresponding docs in the same commit.
4. Run the targeted test and the project-level verification commands that prove
   the claim.
5. Update this document only with evidence from the current branch.

## Current Branch Verification Commands

Use this set after changing reality-gate code or docs:

```bash
cargo fmt --check
git diff --check
bash -n scripts/local-alpha-evidence-summary.sh
bash -n scripts/local-alpha-release-gate-refresh.sh
bash -n scripts/release-soak-local.sh
bash -n scripts/status-sync-check.sh
cargo test --test status_sync -v
./scripts/status-sync-check.sh
cargo test --test local_alpha_release_evidence -v
cargo test --test local_alpha_release_evidence release_soak -v
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
cargo run --quiet --bin local_alpha_evidence_summary -- --evidence-root .
```

If `local_alpha_evidence_summary` reports `in_progress` or `not_verified`, keep
the corresponding module open. Do not rewrite open gates into completed product
claims.

## Plan / Status Sync Check

The local `status-sync-check` slice is a read-only drift detector for declared
`cargo test` totals. It compares the current `cargo test -- --list --format
terse` test list against the total declarations in:

- `README.md`
- `docs/testing-guide-2026-03-24.md`
- `docs/local-mcp-integration-2026-03-26.md`
- `docs/project-status.md`
- `docs/progress-tracker.md`
- `docs/product/follow-up-reality-gates.md`

Current branch `cargo test` total declaration: 272 tests.

This check only protects the documented cargo-test total from drifting. It does
not certify Local Alpha, does not turn simulation evidence into real
fresh-machine evidence, does not prove Windows runner parity, and does not
change the `run_reflection` durable write-path boundary.
