# Productization Follow-Up Reality Gates

This document tracks productization work that still has assumption, simulation,
planning-only, or not-yet-merged status. It is intentionally stricter than the
roadmap: a module is treated as complete only when the implementation, fresh
evidence, and product wording all line up.

Current baseline:

- implementation branch: `codex/physics-layer-report`
- carried implementation: P1/P2/P3 follow-up slices plus the read-only
  physics-informed architecture report and wording guards in the current branch
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

For current-branch plan/status sync, `implemented-unmerged` is treated as incomplete until the work lands in the checked-out branch.

## Remaining Incomplete Items After Physics-Informed Architecture Slice

The current branch implements the local-safe slice as read-only reports,
machine-readable gates, and product wording guards. The following items remain
incomplete and must not be promoted to completed capability without later code,
tests, docs, fresh evidence, and review:

- `partial`: Local Alpha full release gate; still missing real fresh-machine
  evidence, Windows runtime parity evidence, and human release decision.
- `simulation-only`: Fresh-machine first-run; local simulation is not real
  clone/unpack/install evidence.
- `planning-gate`: Windows parity; macOS execution and static PowerShell
  checks do not prove Windows runtime parity.
- `partial`: Provider matrix; only `mock` and `openai-compatible` are runnable,
  while Azure OpenAI, OpenRouter, and local providers remain planned-only.
- `partial`: Observe-only daemon; diagnostics and handle lifecycle exist, but
  daemon-triggered writes and connected background service behavior are blocked.
- `planning-gate`: Remote/team/security foundation; remote writes, team shared
  memory, support bundle upload, public dashboard exposure, auth,
  authorization, audit, rate limit, tenant isolation, and rollback remain
  unimplemented.
- `partial`: Release engineering; installer, binary package, service manager,
  auto-updater, compatibility matrix automation, Beta/GA evidence, and
  production-ready claims remain blocked.
- `partial`: Multi-layer memory; current support is read-only projection only,
  not procedural memory, slow variables, durable self-model writes, or complete
  multi-layer cognition.
- `blocked claim`: Physics-informed runtime, solver/controller behavior,
  constraint optimizer, physical controller, and scientific validation claims
  remain wording-gated non-claims.

## Follow-Up Modules

| Priority | Module | Current Status | Assumption / Gap | Required Follow-Up | Evidence To Accept |
| --- | --- | --- | --- | --- | --- |
| `P0` | Local Alpha evidence summary | `implemented` | Mainline previously had no single read-only rollup to distinguish open, not-verified, and satisfied gates. | Keep `scripts/local-alpha-evidence-summary.sh`, `src/bin/local_alpha_evidence_summary.rs`, `src/support/local_alpha_evidence.rs`, and `tests/local_alpha_release_evidence.rs` in the productization branch; run it after every release-gate refresh. | `cargo test --test local_alpha_release_evidence -v`; `cargo run --quiet --bin local_alpha_evidence_summary -- --evidence-root .` |
| `P0` | Local Alpha release-gate refresh | `implemented` | Manual refresh could skip product smoke, first-run simulation, support bundle, or summary output, leaving stale gate evidence. | Use `scripts/local-alpha-release-gate-refresh.sh [config_path]` to refresh the locally reproducible gate slice; keep missing real fresh-machine, Windows runner, remote/team, and release-decision evidence open. | `bash -n scripts/local-alpha-release-gate-refresh.sh`; `cargo test --test local_alpha_release_evidence -v`; optional manual run writes `target/reports/local-alpha/evidence-summary.json`. |
| `P0` | Local Alpha full release gate | `partial` | Product smoke, first-run simulation, support bundle, and summary can now be refreshed together locally, but Windows parity, real fresh-machine evidence, and release decision evidence can still be missing. | Run the refresh script for local evidence, then separately record real fresh-machine and Windows runner evidence without claiming completion from local simulation artifacts. | `./scripts/local-alpha-release-gate-refresh.sh`; evidence summary JSON showing every gate state; separate real fresh-machine and Windows parity summaries before human release review. |
| `P0` | Fresh-machine first-run | `simulation-only` | `first-run-bootstrap-smoke-local.sh` proves an isolated local simulation, not a real clone/unpack on a fresh machine. | Run the documented bootstrap -> doctor path in a clean checkout or real fresh-machine environment; record the summary separately from simulation evidence. | A dated `first-run-bootstrap/summary.json` or documented equivalent with `real_fresh_machine_evidence = true`, `doctor_status = ok`, `local_only = true`, no serve/product smoke side effects, and `run_reflection` write path. |
| `P0` | Windows parity | `planning-gate` | PowerShell/static contract tests do not prove Windows runtime parity. | Run Windows or Windows runner validation for bootstrap, doctor, and product smoke parity; do not infer Windows from macOS. | `windows-parity/summary.json` or `target/windows-parity/local-alpha-gate/summary.json` with `status = verified`, `runtime_parity = true`, and a Windows runner/platform marker. |
| `P1` | Product readiness gate checker | `implemented` | The checker is local-only and conservative; it intentionally reports blocked while release decision, real fresh-machine, Windows parity, remote/team, or security gates are missing. | Use it as a candidate review preflight, not as Local Alpha certification. | `cargo test --test product_readiness -v`; `./scripts/product-readiness-check.sh <candidate-name>`; `ready = false` until external gates exist. |
| `P1` | Release decision artifact | `implemented` | The generator writes a source-only decision artifact, but cannot supply the human decision itself. | Record actual human reviewer, decision, rollback note, open gates, and non-claims before treating a candidate as approved. | `cargo test --test release_decision -v`; `./scripts/release-decision-local.sh <candidate-name> <evidence-root>`. |
| `P1` | Plan/status synchronization | `implemented` | Planning checkboxes, test counts, and reality gate rows can drift from code. The current branch implements read-only cargo-test total drift plus plan/reality contradiction detection. | Use `status-sync-check` after each productization slice, then reconcile any flagged docs against commands run in the current branch. | `cargo test --test status_sync -v`; `./scripts/status-sync-check.sh`; matching test counts and no stale completed plan items against incomplete reality rows. |
| `P1` | System layer report | `implemented` | Architecture-layer planning previously lived in docs only. The current branch exposes a read-only `doctor.system_layer_report` with substrate, signal, memory, policy, control loop, actuator, interface, and release-boundary status plus blockers, physics principle mappings, dependency rules, Phase 0-8 coverage, and non-claims. | Keep it read-only and status-only. It must not grant daemon writes, remote/team behavior, provider adapters, durable memory layer writes, physics-informed runtime behavior, or Local Alpha certification. | `cargo test --test product_completion_read_models -v`; `./scripts/agent-llm-mm.sh doctor`; `doctor.system_layer_report.read_only = true`; every layer keeps `writes_allowed = false`. |
| `P1` | Support bundle and daemon lifecycle hardening | `implemented` | The current slice adds support-bundle integrity hashes and observe-only daemon lifecycle/write-blocker diagnostics; it does not make support bundles a production support channel or turn the daemon into a background product service. | Keep the underlying support bundle and daemon product rows partial until upload/service/write gates exist. | `cargo test --test support_bundle -v`; `cargo test --test daemon_config -v`; `./scripts/agent-llm-mm.sh doctor`. |
| `P1` | Observe-only daemon lifecycle | `partial` | `doctor.daemon_observe_only` is real, and `DaemonHandle` now has local start / stop proof, but it is still not connected as a running background product service. | Keep write capability blocked; use lifecycle tests to preserve clean shutdown and closed write / remote gates before any future daemon write path. | `cargo test --test daemon_config -v`; tests prove disabled quick exit, observe-only start / stop, no remote listener, no semantic writes, and clean shutdown. |
| `P1` | Runtime self-revision coverage | `partial` | Four hooks are wired, but this is not all-entry auto-reflection or continuous autonomy. | Tighten diagnostics and tests for the existing four hooks before widening trigger coverage. | Focused `mcp_stdio`, `failure_modes`, and `bootstrap` tests showing hook list, opt-in requirements, best-effort failure semantics, and no direct `run_reflection` recursion. |
| `P1` | Support bundle diagnostics | `partial` | Local support bundle is useful, but not a production support channel, remote upload flow, or automatic log collector. Integrity hashes now cover non-manifest bundle files. | Keep explicit log file and correlation filters; extend only after redaction tests exist. Current manifest must keep `safety_checks` explicit for read-only generation, no runtime bootstrap, no SQLite/TOML/raw log copies, and no provider payloads. | `cargo test --test support_bundle -v`; secret scans on generated bundle; `manifest.json` says local only, upload false, every safety check remains closed for copied raw artifacts, and integrity uses SHA-256. |
| `P2` | Structured decision protocol | `implemented` | `decide_with_snapshot` now has a v2 response envelope with `decision_id`, requested/selected action, local confidence metadata, policy checks, and non-claims, but the model decision remains a minimal action string. This is not a full decision engine. | Preserve legacy `blocked` / `decision` callers while adding any future schema fields. Do not add provider JSON decisions, calibrated confidence scoring, planning, or policy arbitration without separate tests and docs. | `docs/product/structured-decision-protocol.md`; `cargo test --test decision_flow -v`; future MCP schema compatibility checks when the wire schema changes. |
| `P2` | Evidence semantics v2 first slice | `implemented` | Current queries remain bounded narrowing only; the implemented slice is a read-only evidence relation report over the existing trigger window, not a full weight/ranking or widening engine. | Add one evidence-v2 slice at a time while preserving no-widening governance. | `cargo test --test product_completion_read_models -v`; focused `sqlite_store`, `failure_modes`, `evidence_query_dto`, and `application_use_cases` tests. |
| `P2` | Richer episode semantics first slice | `implemented` | The implemented slice projects objective/outcome/linked evidence ids as local metadata over existing episode events; it does not write identity or commitments and is not a complete autobiographical schema. | Keep it read-only until richer episode schema, migration, and evidence policy gates exist. | `cargo test --test product_completion_read_models -v`. |
| `P2` | Provider readiness hardening | `implemented` | The current slice exposes provider readiness rows and rejects planned-only providers as config values; it does not add new runnable providers. | Keep `azure-openai`, `openrouter`, and `local` non-configurable until real adapters and tests land in the same change. | `cargo test --test provider_config -v`; `./scripts/agent-llm-mm.sh doctor`. |
| `P2` | Provider matrix | `partial` | Only `mock` and first `openai-compatible` are runnable. The current branch exposes a read-only matrix in config and `doctor`; `azure-openai`, `openrouter`, and `local` remain planned-only and non-configurable, with missing implementation details listed. | Add adapters only with config, doctor, parsing, redaction, timeout/error-mode, and MCP stdio path tests. Do not turn matrix rows into accepted config values without a real adapter in the same change. | `docs/provider-contract.md`; `cargo test --test provider_config -v`; provider-specific tests before any future provider is marked supported. |
| `P2` | Data lifecycle and migration | `partial` | Backup/restore local scripts are covered, but this is not remote backup, cloud sync, scheduled backup, or production disaster recovery. | Add migration tests when schema changes; keep restore-to-new-path and manual database switch boundary. | `cargo test --test sqlite_backup_restore -v`; schema migration tests for any future migration. |
| `P3` | Remote/team read-only inventory | `implemented` | Machine-readable remote/team inventory exists in `doctor` and product readiness, but every remote/team capability remains blocked. | Keep every remote/team capability blocked until auth, authorization, audit, rate limit, tenant isolation, rollback, and route tests pass in a later implementation. | `cargo test --test product_completion_read_models -v`; `doctor.remote_team_capability_inventory`; no write-capable remote route or support-bundle upload exposed. |
| `P3` | Security and auth gate contracts | `implemented` | Machine-readable auth, authorization, audit, rate-limit, tenant-isolation, and rollback gate contracts exist; no remote write is enabled. | Keep these as blocking contracts until real auth/authz/audit/rate-limit/tenant/rollback implementation and route tests land. | `cargo test --test product_completion_read_models -v`; `doctor.remote_team_security_gates`; `remote_writes_allowed = false`. |
| `P3` | Release engineering | `partial` | Rules and a local release soak runner exist, but there is still no installer, binary package, service manager, auto-updater, Windows/fresh-machine release evidence, compatibility matrix automation, or release decision workflow. | Use `scripts/release-soak-local.sh <candidate-name> [config_path]` for local candidate evidence, keep first artifact source-only, and add real compatibility matrix / release decision evidence before Beta language. | `bash -n scripts/release-soak-local.sh`; `cargo test --test local_alpha_release_evidence release_soak -v`; candidate-specific `target/reports/releases/<candidate-name>/` with command logs, summary, support-bundle scans, SHA-256 manifests, and Local Alpha evidence summary. |
| `P3` | Multi-layer memory | `implemented` | A read-only layered projection exists and labels working / episodic / semantic / procedural / self-model layers as `partial` or `not_implemented`; it is not a complete memory architecture and does not add durable self-model writes. | Keep procedural memory, richer semantic extraction, slow variables, and durable self-model writes behind later schema/evidence/migration gates. | `cargo test --test product_completion_read_models -v`; no Local Alpha claim that multi-layer memory is complete. |
| `P3` | Product wording guard | `implemented` | Product readiness checks candidate wording for blocked Beta, GA, production-ready, remote/team, remote write admin, complete self-governance, physics-informed runtime, solver/controller, and scientific validation claims. | Keep this wired into product readiness and extend only when matching gates actually pass. | `cargo test --test product_readiness -v`; blocked claims appear as `product_wording` gate failures. |

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
`cargo test` totals and plan/reality-gate contradictions. It compares the
current `cargo test -- --list --format terse` test list against the total
declarations in:

- `README.md`
- `docs/testing-guide-2026-03-24.md`
- `docs/local-mcp-integration-2026-03-26.md`
- `docs/project-status.md`
- `docs/progress-tracker.md`
- `docs/product/follow-up-reality-gates.md`

Current branch `cargo test` total declaration: 295 tests.

It also reads
`docs/superpowers/plans/2026-05-24-p1-p2-p3-product-completion-plan.md` and
this document, then fails if a checked plan item has no matching reality-gate
row or if the matching row is still `implemented-unmerged`, `partial`,
`simulation-only`, `planning-gate`, or `not-implemented`.

This check protects the documented cargo-test total and the current
P1/P2/P3 plan-to-reality status alignment from drifting. It does not certify
Local Alpha, does not turn simulation evidence into real fresh-machine
evidence, does not prove Windows runner parity, and does not change the
`run_reflection` durable write-path boundary.
