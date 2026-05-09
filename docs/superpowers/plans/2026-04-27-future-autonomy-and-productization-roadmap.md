# Future Autonomy and Productization Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Plan the later-stage path from the current local MCP memory MVP toward a more autonomous and productizable system without confusing those future goals with the current usable demo boundary.

**Architecture:** Advance in gated layers: first stabilize the local MVP contract, then add durable operation observability, then add local daemon orchestration, then add policy-controlled broader auto-reflection, then evaluate remote administration, authentication, multi-tenancy, and production self-governance. Each layer must preserve auditable writes and must define rollback, safety, and verification gates before the next layer starts.

**Tech Stack:** Rust, Tokio, RMCP, SQLite initially, future durable operation store, local daemon/service wrappers, HTTP dashboard/API only after explicit productization review, TOML configuration, Markdown specs, integration tests, and deterministic replay artifacts.

---

## Relationship to the Current MVP Plan

This document is the future roadmap for the capabilities that are intentionally not implemented in the immediate hardening plan:

- 完整自治 agent
- 生产级 self-governing 系统
- 远程管理后台
- 带认证、多租户、持久化 operation log 的产品化服务
- “所有入口都会自动反思”的后台 daemon

The immediate plan remains `docs/superpowers/plans/2026-04-27-post-mvp-hardening-and-boundary-plan.md`.

Do not start this roadmap until the immediate plan has at least completed:

- verification baseline refresh
- release gate runbook
- current 4-hook runtime contract stabilization
- self-revision diagnostic contract
- reflection deeper-update contract
- at least one bounded evidence-query hardening slice

## Non-Negotiable Safety Rules

- `run_reflection` remains the only durable identity/commitment write path until a later architecture decision record explicitly replaces that rule.
- Any new daemon or broader auto-reflection entry point must be opt-in by config at first.
- Any remote/admin capability must start read-only and local-first before exposing writes.
- Authentication, authorization, and audit logging are prerequisites for write-capable remote administration.
- Multi-tenancy requires namespace, database, config, dashboard, and operation-log isolation to be specified and tested before runtime support is enabled.
- Product claims must distinguish prototype, internal alpha, beta, and production readiness.

## File Map

- `docs/superpowers/specs/`: architecture specs for each future capability.
- `docs/roadmap.md`: high-level ordering and public framing.
- `docs/project-status.md`: current implementation state after each milestone.
- `docs/progress-tracker.md`: task-level status and remaining gaps.
- `docs/release-gate.md`: verification gate once created by the immediate plan.
- `src/application/auto_reflect_if_needed.rs`: governed auto-reflection coordinator.
- `src/interfaces/mcp/server.rs`: current MCP stdio entrypoints and hook constants.
- `src/interfaces/dashboard/`: current local read-only dashboard.
- `src/adapters/sqlite/`: current persistence adapter and likely starting point for operation-log persistence.
- `tests/failure_modes.rs`: safety, rejection, suppression, cooldown, and governance tests.
- `tests/mcp_stdio.rs`: MCP entrypoint behavior tests.
- `tests/dashboard_*.rs`: current dashboard boundary tests.

## Phase 0: Future Capability Spec Pack

### Task 0.1: Write an Autonomy/Productization Spec Index

**Recommended owner:** planner, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-future-autonomy-productization-index.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/progress-tracker.md`

- [ ] **Step 1: Create the spec index**

Create `docs/superpowers/specs/2026-04-27-future-autonomy-productization-index.md` with these sections:

```markdown
# Future Autonomy and Productization Spec Index

## Purpose

This index tracks future capabilities beyond the current local MCP memory MVP. It does not claim these capabilities are implemented.

## Capability Tracks

| Track | Future Goal | Current Status | First Required Spec |
| --- | --- | --- | --- |
| Autonomous agent | Complete autonomous agent behavior | Not implemented | autonomy-governance-spec |
| Production self-governance | Production-grade self-governing system | Not implemented | production-governance-spec |
| Remote management | Remote management backend | Not implemented | remote-admin-boundary-spec |
| Product service | Auth, multi-tenancy, durable operation log | Not implemented | productization-foundation-spec |
| Background daemon | All-entry auto-reflection daemon | Not implemented | daemon-trigger-policy-spec |

## Entry Criteria

- Current MVP release gate is stable.
- Current 4 runtime hooks have documented and tested boundaries.
- Self-revision diagnostics are inspectable.
- Evidence policy has bounded server-side validation.

## Exit Criteria

Each track has a separate implementation plan with tests, rollback strategy, and explicit non-goals before code work starts.
```

- [ ] **Step 2: Link it from roadmap and progress tracker**

Add a future-stage pointer in:

- `docs/roadmap.md`
- `docs/progress-tracker.md`

Required wording:

```markdown
完整自治、生产级 self-governing、远程管理、认证/多租户/持久化 operation log、以及“所有入口自动反思”的 daemon 均进入未来规划，但不属于当前 MVP hardening 的实现范围。
```

- [ ] **Step 3: Verify future-vs-current wording**

Run:

```bash
rg -n 'Not implemented|不属于当前 MVP|未来规划|已实现.*完整自治|已实现.*生产级|已实现.*daemon' docs/superpowers/specs/2026-04-27-future-autonomy-productization-index.md docs/roadmap.md docs/progress-tracker.md
```

Expected:

- future tracks are described as planned or not implemented
- no sentence claims complete autonomy, production self-governance, or daemon behavior is currently implemented

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-future-autonomy-productization-index.md docs/roadmap.md docs/progress-tracker.md
git commit -m "docs: index future autonomy productization tracks"
```

## Phase 1: Durable Operation Log Foundation

### Task 1.1: Specify Durable Operation Log Boundaries

**Recommended owner:** planner, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-durable-operation-log-design.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`

- [ ] **Step 1: Write the design spec**

The spec must include:

- what counts as an operation event
- difference between current bounded in-memory dashboard recorder and future durable operation log
- retention policy options
- redaction requirements
- correlation id requirements
- local-only default
- migration plan from SQLite tables or separate operation-log database
- why durable operation log is required before remote management or production self-governance

- [ ] **Step 2: Define minimum schema**

Document this minimum schema:

```text
operation_id
occurred_at
namespace
actor_kind
actor_id
entrypoint
operation_kind
status
correlation_id
request_summary_json
response_summary_json
diagnostic_summary_json
redaction_version
```

- [ ] **Step 3: Write acceptance tests as prose before implementation**

The spec must list these future tests:

- operation log records MCP tool start and completion
- operation log redacts secrets from model/provider config
- operation log links auto-reflection diagnostics by correlation id
- operation log can be queried by namespace and time range
- operation log write failure does not corrupt MCP stdout

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-durable-operation-log-design.md docs/project-status.md docs/roadmap.md
git commit -m "docs: design durable operation log foundation"
```

### Task 1.2: Implement Local Durable Operation Log MVP

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Create: `src/domain/operation_log.rs`
- Create: `src/ports/operation_log_store.rs`
- Modify: `src/ports/mod.rs`
- Modify: `src/adapters/sqlite/schema.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Modify: `src/interfaces/dashboard/recorder.rs`
- Modify: `src/interfaces/dashboard/projection.rs`
- Create: `tests/operation_log.rs`
- Modify: `tests/dashboard_projection.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Write failing persistence tests**

Create `tests/operation_log.rs` with tests for:

- inserting one operation event
- querying recent operation events by namespace
- redacting a fake API key value from summary JSON
- preserving MCP stdout safety when dashboard is enabled

Run:

```bash
cargo test --test operation_log -v
```

Expected:

- fails because operation log types and store do not exist

- [ ] **Step 2: Implement domain and port types**

Create:

- `OperationLogEntry`
- `OperationLogStatus`
- `OperationLogStore`
- `OperationLogQuery`

Keep all fields explicit and serializable where needed by the dashboard projection.

- [ ] **Step 3: Implement SQLite storage**

Add an `operation_log` table with the minimum schema from Task 1.1. Use deterministic ordering by `occurred_at` and `operation_id`.

- [ ] **Step 4: Wire dashboard projection read path**

Dashboard may read operation-log summaries, but this task must not add write-capable dashboard routes.

- [ ] **Step 5: Verify**

Run:

```bash
cargo test --test operation_log -v
cargo test --test dashboard_projection -v
cargo test --test dashboard_http -v
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
cargo test
```

Expected:

- all tests pass
- dashboard remains read-only

- [ ] **Step 6: Commit**

Run:

```bash
git add src/domain/operation_log.rs src/ports/operation_log_store.rs src/ports/mod.rs src/adapters/sqlite/schema.rs src/adapters/sqlite/store.rs src/interfaces/dashboard/recorder.rs src/interfaces/dashboard/projection.rs tests/operation_log.rs tests/dashboard_projection.rs docs/testing-guide-2026-03-24.md
git commit -m "feat: add local durable operation log"
```

## Phase 2: Background Daemon, Local and Opt-In

### Task 2.1: Specify Local Daemon Trigger Policy

**Recommended owner:** planner, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-local-daemon-trigger-policy.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/project-status.md`

- [ ] **Step 1: Write daemon policy spec**

The spec must define:

- daemon is local-only in the first stage
- daemon is disabled by default
- daemon only reads from durable operation log and existing stores
- daemon writes identity/commitment updates only by calling the existing governed `run_reflection` path
- daemon has cooldown, concurrency limit, and shutdown semantics
- daemon never changes MCP tool responses after the fact

- [ ] **Step 2: Define trigger classes**

Document exactly these initial trigger classes:

| Trigger Class | Source | Initial Status |
| --- | --- | --- |
| repeated failure | durable operation log | future |
| unresolved conflict | claims and trigger ledger | future |
| scheduled review | local timer | future |
| stale commitment review | commitments and audit log | future |

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-local-daemon-trigger-policy.md docs/roadmap.md docs/project-status.md
git commit -m "docs: specify local daemon trigger policy"
```

### Task 2.2: Implement Disabled-by-Default Local Daemon Skeleton

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `src/support/config.rs`
- Modify: `examples/agent-llm-mm.example.toml`
- Modify: `src/support/doctor.rs`
- Create: `src/application/daemon.rs`
- Modify: `src/application/mod.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `tests/bootstrap.rs`
- Create: `tests/daemon_config.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Write failing config tests**

Create `tests/daemon_config.rs` with tests for:

- daemon defaults to disabled
- config rejects zero polling interval
- doctor reports daemon config without starting daemon

Run:

```bash
cargo test --test daemon_config -v
```

Expected:

- fails because daemon config does not exist

- [ ] **Step 2: Add config shape**

Add:

```toml
[daemon]
enabled = false
poll_interval_ms = 60000
max_concurrent_tasks = 1
```

- [ ] **Step 3: Add doctor report fields**

Doctor should report:

- `daemon_enabled`
- `daemon_poll_interval_ms`
- `daemon_max_concurrent_tasks`

Doctor must not start the daemon.

- [ ] **Step 4: Implement daemon skeleton**

Create a daemon module that can start and stop cleanly but performs no autonomous reflection until a later trigger task.

- [ ] **Step 5: Verify**

Run:

```bash
cargo test --test daemon_config -v
cargo test --test bootstrap -v
cargo test
./scripts/agent-llm-mm.sh doctor
```

Expected:

- all tests pass
- default daemon state is disabled
- doctor reports config only

- [ ] **Step 6: Commit**

Run:

```bash
git add src/support/config.rs examples/agent-llm-mm.example.toml src/support/doctor.rs src/application/daemon.rs src/application/mod.rs src/interfaces/mcp/server.rs tests/bootstrap.rs tests/daemon_config.rs docs/testing-guide-2026-03-24.md
git commit -m "feat: add disabled local daemon skeleton"
```

## Phase 3: Broader Auto-Reflection Entry Points

### Task 3.1: Specify All-Entry Auto-Reflection Governance

**Recommended owner:** planner, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-all-entry-auto-reflection-governance.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`

- [ ] **Step 1: Define what all-entry means**

The spec must distinguish:

- observing all entrypoints
- evaluating all entrypoints for trigger candidates
- actually running governed self-revision
- writing durable identity/commitment updates

Only the final step is a durable write, and it must still go through `run_reflection` unless a later ADR changes the architecture.

- [ ] **Step 2: Define mandatory gates**

Mandatory gates:

- explicit namespace resolution
- evidence window bounded by operation or event context
- model proposal confidence threshold
- server-side evidence validation
- cooldown and duplicate suppression
- operation log correlation id
- human-review mode before autonomous write mode

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-all-entry-auto-reflection-governance.md docs/project-status.md docs/roadmap.md
git commit -m "docs: specify all-entry auto-reflection governance"
```

### Task 3.2: Implement Observe-Only All-Entry Candidate Recording

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/domain/self_revision.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Modify: `tests/mcp_stdio.rs`
- Modify: `tests/failure_modes.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Write failing observe-only tests**

Add tests proving:

- every MCP tool can record a trigger candidate when observe-only mode is enabled
- observe-only mode does not call the model
- observe-only mode does not call `run_reflection`
- observe-only mode records operation-log correlation id

Run:

```bash
cargo test --test mcp_stdio observe_only -v
```

Expected:

- fails because observe-only candidate recording is not implemented

- [ ] **Step 2: Implement observe-only candidate recording**

Add candidate recording behind explicit config. Do not add autonomous writes.

- [ ] **Step 3: Verify**

Run:

```bash
cargo test --test mcp_stdio -v
cargo test --test failure_modes -v
cargo test
./scripts/agent-llm-mm.sh doctor
```

Expected:

- all tests pass
- doctor distinguishes observe-only candidate recording from active auto-reflection

- [ ] **Step 4: Commit**

Run:

```bash
git add src/application/auto_reflect_if_needed.rs src/interfaces/mcp/server.rs src/domain/self_revision.rs src/adapters/sqlite/store.rs tests/mcp_stdio.rs tests/failure_modes.rs docs/testing-guide-2026-03-24.md
git commit -m "feat: record all-entry reflection candidates in observe-only mode"
```

## Phase 4: Remote Management, Read-Only First

### Task 4.1: Specify Remote Management Boundary

**Recommended owner:** planner/security reviewer pair, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-remote-management-boundary.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/project-status.md`

- [ ] **Step 1: Write remote boundary spec**

The first remote management stage must be read-only and must include:

- bind address policy
- authentication requirement before non-local bind
- authorization model
- audit event requirements
- redaction policy
- rate limit policy
- no write-capable admin routes in first stage

- [ ] **Step 2: Define route inventory**

Allowed first-stage routes:

```text
GET /health
GET /api/summary
GET /api/events
GET /api/operations
GET /api/reflection-candidates
```

Forbidden first-stage routes:

```text
POST /api/reflections
POST /api/commitments
POST /api/identity
POST /api/daemon/start
POST /api/daemon/stop
```

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-remote-management-boundary.md docs/roadmap.md docs/project-status.md
git commit -m "docs: specify read-only remote management boundary"
```

### Task 4.2: Implement Read-Only Remote API Behind Explicit Config

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `src/support/config.rs`
- Modify: `src/interfaces/dashboard/http.rs`
- Modify: `tests/dashboard_http.rs`
- Create: `tests/remote_management.rs`
- Modify: `examples/agent-llm-mm.example.toml`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Write failing config and route tests**

Tests must prove:

- remote API is disabled by default
- non-local bind without auth config is rejected
- allowed read-only routes return expected JSON
- forbidden write routes return non-success status

Run:

```bash
cargo test --test remote_management -v
cargo test --test dashboard_http -v
```

Expected:

- new tests fail until config and route policy are implemented

- [ ] **Step 2: Implement read-only remote config**

Add explicit config for remote management. Keep default disabled and local-only unless auth is configured.

- [ ] **Step 3: Verify**

Run:

```bash
cargo test --test remote_management -v
cargo test --test dashboard_http -v
cargo test
```

Expected:

- all tests pass
- no write-capable remote route exists

- [ ] **Step 4: Commit**

Run:

```bash
git add src/support/config.rs src/interfaces/dashboard/http.rs tests/dashboard_http.rs tests/remote_management.rs examples/agent-llm-mm.example.toml docs/testing-guide-2026-03-24.md
git commit -m "feat: add read-only remote management API"
```

## Phase 5: Auth, Multi-Tenancy, and Product Service Foundation

### Task 5.1: Specify Productization Foundation

**Recommended owner:** planner/security reviewer pair, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-productization-foundation.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/project-status.md`

- [ ] **Step 1: Write productization spec**

The spec must define:

- authentication model
- authorization roles
- tenant isolation model
- namespace-to-tenant mapping
- database isolation or row-level isolation choice
- operation-log tenant isolation
- secret storage policy
- admin action audit policy
- migration and rollback strategy

- [ ] **Step 2: Define minimum roles**

Initial roles:

| Role | Capability |
| --- | --- |
| viewer | read health, summary, operations |
| operator | manage local daemon state after auth is implemented |
| maintainer | run maintenance operations |
| admin | manage tenants and credentials |

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-productization-foundation.md docs/roadmap.md docs/project-status.md
git commit -m "docs: specify productization foundation"
```

### Task 5.2: Implement Auth and Tenant Model as Inactive Foundations

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Create: `src/domain/auth.rs`
- Create: `src/domain/tenant.rs`
- Create: `src/ports/auth_store.rs`
- Modify: `src/ports/mod.rs`
- Modify: `src/adapters/sqlite/schema.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Create: `tests/auth_tenant.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Write failing domain and persistence tests**

Tests must prove:

- tenant id validates allowed characters
- namespace-to-tenant mapping is explicit
- roles parse and serialize deterministically
- auth tables bootstrap
- operation-log queries cannot cross tenant boundary once tenant id is present

Run:

```bash
cargo test --test auth_tenant -v
```

Expected:

- fails because auth and tenant foundation does not exist

- [ ] **Step 2: Implement inactive foundation**

Add domain types and persistence support, but do not enable remote write behavior.

- [ ] **Step 3: Verify**

Run:

```bash
cargo test --test auth_tenant -v
cargo test --test sqlite_store -v
cargo test
```

Expected:

- all tests pass
- existing local MVP behavior remains unchanged

- [ ] **Step 4: Commit**

Run:

```bash
git add src/domain/auth.rs src/domain/tenant.rs src/ports/auth_store.rs src/ports/mod.rs src/adapters/sqlite/schema.rs src/adapters/sqlite/store.rs tests/auth_tenant.rs docs/testing-guide-2026-03-24.md
git commit -m "feat: add inactive auth tenant foundation"
```

## Phase 6: Production-Grade Self-Governing System Readiness

### Task 6.1: Write Production Self-Governance Readiness Criteria

**Recommended owner:** planner/reviewer pair, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-production-self-governance-readiness.md`
- Modify: `docs/release-readiness.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/progress-tracker.md`

- [ ] **Step 1: Define readiness levels**

Use these levels:

| Level | Meaning |
| --- | --- |
| MVP | local demo, manual operation, bounded hooks |
| internal alpha | daemon opt-in, durable operation log, observe-only all-entry candidates |
| controlled beta | authenticated remote read-only admin, tenant-aware stores, human-review write workflow |
| production candidate | audited write-capable admin, rollback, metrics, incident runbooks, policy conformance tests |
| production | SLOs, monitoring, backup/restore, security review, operational ownership |

- [ ] **Step 2: Define production blockers**

Production blockers must include:

- no security review
- no backup/restore
- no auth/multi-tenant isolation verification
- no durable operation log
- no rollback strategy for self-revision writes
- no incident response runbook
- no policy conformance test suite

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-production-self-governance-readiness.md docs/release-readiness.md docs/roadmap.md docs/progress-tracker.md
git commit -m "docs: define production self-governance readiness"
```

## Global Future Verification Gate

Run before claiming any future-stage milestone is complete:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

If a milestone touches daemon, remote management, auth, multi-tenancy, or operation logs, also run the milestone-specific tests named in that phase.

## Recommended Future Execution Order

1. Phase 0: write the future spec index and link it from status docs.
2. Phase 1: build durable operation log foundation.
3. Phase 2: add disabled-by-default local daemon skeleton.
4. Phase 3: implement observe-only all-entry reflection candidate recording.
5. Phase 4: add read-only remote management behind explicit config.
6. Phase 5: add inactive auth and tenant foundation before write-capable product behavior.
7. Phase 6: define and enforce production self-governance readiness levels.

Write-capable remote administration and autonomous production self-governance must not begin until Phases 1 through 6 are implemented, reviewed, and verified.

## Copyable Handoff for Next Session

```text
Work in /Users/yooyui/code/agent-llm-mm. Follow AGENTS.md. Use docs/superpowers/plans/2026-04-27-future-autonomy-and-productization-roadmap.md for future-stage planning. This is not the current MVP hardening track.

Before implementation, confirm docs/superpowers/plans/2026-04-27-post-mvp-hardening-and-boundary-plan.md has completed the release gate, current 4-hook contract, diagnostics, reflection contract, and evidence policy hardening tasks. If not, return to that plan first.

For future-stage work, start with Phase 0. Keep run_reflection as the durable identity/commitment write path unless a later architecture decision explicitly changes it. Start daemon, remote management, auth/multi-tenancy, and production self-governance as specs or disabled/read-only foundations before enabling writes.
```

## Self-Review Notes

- The previously excluded capabilities are now represented as explicit future tracks.
- The plan keeps current MVP claims separate from future implementation goals.
- Each future capability has a staged path with docs/spec work before code.
- Write-capable remote administration and production self-governance remain gated behind operation log, daemon policy, auth, tenant isolation, and readiness criteria.
