# Local Product Alpha Development Task List

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the validated MVP into a Local Product Alpha that a real local user can install, configure, run, inspect, backup, and troubleshoot.

**Architecture:** Keep the current MCP `stdio` service and SQLite store as the product core. Add product docs, config profiles, bootstrap/smoke scripts, data-safety runbooks, durable observability wiring, and gated daemon work in small independently verifiable slices.

**Tech Stack:** Rust, Cargo, RMCP `stdio`, SQLite, TOML, bash/zsh on macOS, PowerShell parity notes for Windows, local dashboard HTTP surface, Markdown docs, integration tests.

---

## Relationship to Roadmap

Source roadmap: `docs/superpowers/plans/2026-05-09-productization-roadmap.md`.

This file is the execution-oriented task list for the first productization slice:

- Phase 0: Product Definition and Gate Reset
- Phase 1: Local Product Alpha
- Phase 2 preparation: durable observability and observe-only daemon gates

The current MVP is accepted as the starting point, not the final product. Public wording should remain: "validated local MVP entering productization" until Local Product Alpha exits its gate.

## Execution Status

Last updated: 2026-05-14.

Completed locally and validated:

- [x] Task A1: Local Alpha PRD.
- [x] Task A2: Local Alpha Release Gate.
- [x] Task B1: Config Profile Examples.
- [x] Task B2: Config Profile Validation Tests.
- [x] Task C1: Product Bootstrap Command.
- [x] Task C2: Product Smoke Script.
- [x] Task D1: SQLite Backup and Restore Runbook.
- [x] Task D2: Support Bundle Design.
- [x] Task E2: Correlation ID Contract.
- [x] Task E3: Durable Operation-Log Dashboard History Query.
- [x] Task F1: Daemon Observe-Only Gate Doc.

Completed as read-only evidence:

- [x] Task E1: Runtime Operation Log Wiring Review. Current finding: operation log domain model and SQLite persistence exist. Successful MCP tool calls now append operation-log entries with generated correlation ids, dashboard events/projections preserve correlation ids, and dashboard now exposes a local read-only durable operation-log history query surface.

Next execution queue:

- [ ] Implement the support bundle script after operation-log history and log locations are stable.
- [ ] Build observe-only daemon diagnostics behind the no-write gate.

Current Local Alpha state remains: validated local MVP entering productization. Product Smoke, Data Safety, Support Bundle Design, Daemon Observe-Only Gate, Correlation ID Contract, and Durable Operation-Log Dashboard History now have working local paths or reviewed design gates, but Local Alpha is not complete until support bundle automation, observe-only daemon diagnostics, and the full Local Alpha gate have fresh evidence.

## File Map

Planned documentation and examples:

- Create: `docs/product/prd-local-alpha.md`
- Create: `docs/product/release-gate-local-alpha.md`
- Create: `docs/product/data-safety-local-alpha.md`
- Create: `docs/product/support-bundle-local-alpha.md`
- Create: `docs/product/daemon-observe-only-gate.md`
- Create: `docs/product/correlation-id-contract.md`
- Create: `examples/agent-llm-mm.dev.example.toml`
- Create: `examples/agent-llm-mm.prod-local.example.toml`
- Modify: `README.md`
- Modify: `docs/document-map.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/progress-tracker.md`
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`
- Modify: `docs/testing-guide-2026-03-24.md`

Likely script and test files:

- Modify: `scripts/agent-llm-mm.sh`
- Modify: `scripts/agent-llm-mm.ps1`
- Create: `scripts/product-smoke-local.sh`
- Create: `scripts/backup-sqlite.sh`
- Create: `scripts/restore-sqlite.sh`
- Create: `tests/product_smoke.rs` if Rust-level smoke helpers are needed
- Modify: `tests/bootstrap.rs`
- Modify: `tests/provider_config.rs`
- Modify: `tests/operation_log.rs`
- Modify: `tests/daemon_config.rs`
- Modify: `tests/dashboard_http.rs`
- Modify: `tests/dashboard_projection.rs`
- Modify: `tests/mcp_stdio.rs`

Implementation files to inspect before code changes:

- `src/support/config.rs`
- `src/support/doctor.rs`
- `src/application/daemon.rs`
- `src/domain/operation_log.rs`
- `src/ports/operation_log_store.rs`
- `src/adapters/sqlite/schema.rs`
- `src/adapters/sqlite/store.rs`
- `src/interfaces/dashboard/`
- `src/interfaces/mcp/server.rs`

## Milestone A: Product Definition

### Task A1: Local Alpha PRD

**Owner:** planner.

**Purpose:** Define what Local Product Alpha is allowed to claim.

**Files:**

- Create: `docs/product/prd-local-alpha.md`
- Modify: `README.md`
- Modify: `docs/document-map.md`

**Checklist:**

- [ ] Create `docs/product/prd-local-alpha.md`.
- [ ] Define target users: local AI-client users, agent developers, maintainers.
- [ ] Define product promise: persistent local memory, controlled self-revision, local observability.
- [ ] Define explicit non-goals: remote write admin, multi-tenancy, production self-governance, all-entry auto-reflection.
- [ ] Define Local Alpha exit gate in user-visible terms.
- [ ] Link the PRD from `README.md` and `docs/document-map.md`.

**Verification:**

```bash
rg -n 'Local Product Alpha|non-goals|remote write|multi-tenancy|self-governance' docs/product/prd-local-alpha.md README.md docs/document-map.md
git diff --check
```

**Commit:**

```bash
git add docs/product/prd-local-alpha.md README.md docs/document-map.md
git commit -m "docs: define local product alpha prd"
```

### Task A2: Local Alpha Release Gate

**Owner:** planner or reviewer.

**Purpose:** Separate product release readiness from the older MVP release gate.

**Files:**

- Create: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/release-gate.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `docs/document-map.md`

**Checklist:**

- [ ] Create a minimum Local Alpha gate with `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `./scripts/agent-llm-mm.sh doctor`.
- [ ] Add product smoke gate for install/config/smoke once Task C2 exists.
- [ ] Add self-revision evidence gate using the existing demo wrapper.
- [ ] Add dashboard local-only gate.
- [ ] Add daemon gate: daemon disabled by default; observe-only mode required before any daemon write path.
- [ ] Link from `docs/release-gate.md` as "product alpha gate".

**Verification:**

```bash
rg -n 'Local Alpha|product alpha|daemon.*disabled|observe-only|self-revision demo' docs/product/release-gate-local-alpha.md docs/release-gate.md docs/testing-guide-2026-03-24.md docs/document-map.md
git diff --check
```

**Commit:**

```bash
git add docs/product/release-gate-local-alpha.md docs/release-gate.md docs/testing-guide-2026-03-24.md docs/document-map.md
git commit -m "docs: add local alpha release gate"
```

## Milestone B: Config Profiles and Product Setup

### Task B1: Config Profile Examples

**Owner:** worker.

**Purpose:** Stop relying on one sample config for every use case.

**Files:**

- Create: `examples/agent-llm-mm.dev.example.toml`
- Create: `examples/agent-llm-mm.prod-local.example.toml`
- Modify: `examples/agent-llm-mm.example.toml`
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`

**Checklist:**

- [ ] Create a dev profile with local SQLite path guidance and dashboard disabled by default.
- [ ] Create a prod-local profile with explicit database path placeholder, dashboard local-only, daemon disabled, and provider placeholder.
- [ ] Keep demo config separate in `examples/agent-llm-mm.demo.example.toml`.
- [ ] Document that formal data, test data, and demo data must use separate `database_url` values.
- [ ] Ensure all example configs keep daemon disabled by default.

**Verification:**

```bash
rg -n 'database_url|dashboard|daemon|prod-local|dev|demo' examples/*.toml docs/development-macos.md docs/development-windows.md
./scripts/agent-llm-mm.sh doctor examples/agent-llm-mm.prod-local.example.toml
```

Expected:

- `doctor` may fail only if the provider placeholder intentionally lacks a real API key. If so, the doc must say to run `doctor` after replacing placeholders.
- Config parsing tests should be added in Task B2 if placeholders prevent direct `doctor` runs.

**Commit:**

```bash
git add examples/agent-llm-mm.dev.example.toml examples/agent-llm-mm.prod-local.example.toml examples/agent-llm-mm.example.toml docs/development-macos.md docs/development-windows.md
git commit -m "docs: add local product config profiles"
```

### Task B2: Config Profile Validation Tests

**Owner:** worker.

**Purpose:** Make profile drift visible in CI/local tests.

**Files:**

- Modify: `tests/provider_config.rs`
- Modify: `tests/bootstrap.rs`
- Modify: `src/support/config.rs` only if parsing lacks needed validation.

**Checklist:**

- [ ] Add a test that parses the dev example config without real secrets.
- [ ] Add a test that parses the prod-local example config structure.
- [ ] Assert daemon defaults or example values remain disabled.
- [ ] Assert dashboard host stays local by default in product examples.
- [ ] Assert serialized `doctor` output does not include API keys.

**Verification:**

```bash
cargo test --test provider_config --test bootstrap -v
```

**Commit:**

```bash
git add tests/provider_config.rs tests/bootstrap.rs src/support/config.rs
git commit -m "test: cover product config profiles"
```

## Milestone C: Bootstrap and Smoke Test

### Task C1: Product Bootstrap Command

**Owner:** worker.

**Purpose:** Give local users one obvious starting path.

**Files:**

- Modify: `scripts/agent-llm-mm.sh`
- Modify: `scripts/agent-llm-mm.ps1`
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`

**Checklist:**

- [x] Add a `doctor`-first product setup flow to docs.
- [x] Decide whether scripts need a `doctor-config` or `serve-config` alias; if not, document the current `[serve|doctor] [config_path]` contract clearly.
- [x] Keep the shell script pinned to `--bin agent_llm_mm`.
- [x] Keep unsupported modes failing with exit code 2.
- [x] Preserve macOS and Windows command separation.

**Verification:**

```bash
./scripts/agent-llm-mm.sh doctor
cargo test --test bootstrap shell_entry_pins_main_binary_when_auxiliary_bins_exist -v
```

**Commit:**

```bash
git add scripts/agent-llm-mm.sh scripts/agent-llm-mm.ps1 docs/development-macos.md docs/development-windows.md tests/bootstrap.rs
git commit -m "docs: clarify product bootstrap flow"
```

### Task C2: Product Smoke Script

**Owner:** worker.

**Purpose:** Create one command that proves a local alpha install is basically usable.

**Files:**

- Create: `scripts/product-smoke-local.sh`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/testing-guide-2026-03-24.md`

**Checklist:**

- [x] Script runs `./scripts/agent-llm-mm.sh doctor`.
- [x] Script runs the existing self-revision demo wrapper into a staging directory, then promotes validated artifacts to `target/reports/self-revision-demo/latest`.
- [x] Script verifies the 8 required demo artifacts are non-empty.
- [x] Script accepts an optional config path for `doctor`.
- [x] Script exits non-zero on missing artifacts, lock conflicts, or failed commands.

**Verification:**

```bash
bash scripts/product-smoke-local.sh
test -s target/reports/self-revision-demo/latest/report.md
```

**Commit:**

```bash
git add scripts/product-smoke-local.sh docs/product/release-gate-local-alpha.md docs/testing-guide-2026-03-24.md
git commit -m "chore: add local product smoke script"
```

## Milestone D: Data Safety Pack

### Task D1: SQLite Backup and Restore Runbook

**Owner:** worker.

**Purpose:** Make formal local data recoverable before product claims expand.

**Files:**

- Create: `docs/product/data-safety-local-alpha.md`
- Create: `scripts/backup-sqlite.sh`
- Create: `scripts/restore-sqlite.sh`
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`

**Checklist:**

- [x] Document formal/test/demo database separation.
- [x] Document a safe backup command for SQLite files.
- [x] Document restore procedure with "restore to a new path first" as default.
- [x] Add scripts that refuse empty database paths.
- [x] Add scripts that write backups outside the live database directory by default.
- [x] Include checksum generation for backups if local tools are available.

**Verification:**

```bash
bash -n scripts/backup-sqlite.sh
bash -n scripts/restore-sqlite.sh
rg -n 'restore to a new path|database_url|backup|checksum' docs/product/data-safety-local-alpha.md
```

**Commit:**

```bash
git add docs/product/data-safety-local-alpha.md scripts/backup-sqlite.sh scripts/restore-sqlite.sh docs/development-macos.md docs/development-windows.md
git commit -m "docs: add local data safety pack"
```

### Task D2: Support Bundle Design

**Owner:** planner or reviewer.

**Purpose:** Define what users can share when debugging without leaking secrets.

**Files:**

- Create: `docs/product/support-bundle-local-alpha.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/testing-guide-2026-03-24.md`

**Checklist:**

- [x] Define support bundle contents: doctor output, config shape with secrets redacted, recent operation summaries, logs if available, release metadata.
- [x] Define excluded content: API keys, raw provider payloads with secrets, full user database by default.
- [x] Define redaction terms and test expectations.
- [x] Defer implementation script until durable operation log and log locations are stable.

**Verification:**

```bash
rg -n 'API key|redact|support bundle|excluded|doctor' docs/product/support-bundle-local-alpha.md docs/product/release-gate-local-alpha.md
git diff --check
```

**Commit:**

```bash
git add docs/product/support-bundle-local-alpha.md docs/product/release-gate-local-alpha.md docs/testing-guide-2026-03-24.md
git commit -m "docs: design local support bundle"
```

## Milestone E: Durable Observability

### Task E1: Runtime Operation Log Wiring Review

**Owner:** reviewer.

**Purpose:** Determine exactly what is already implemented and what remains before dashboard can rely on durable history.

**Files:**

- Inspect: `src/domain/operation_log.rs`
- Inspect: `src/ports/operation_log_store.rs`
- Inspect: `src/adapters/sqlite/store.rs`
- Inspect: `src/interfaces/dashboard/projection.rs`
- Inspect: `tests/operation_log.rs`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/progress-tracker.md`

**Checklist:**

- [ ] Classify operation log status: domain only, SQLite persistence, runtime event wiring, dashboard query surface.
- [ ] Identify missing runtime write points for MCP tool start/completion/error.
- [ ] Identify missing correlation id propagation points.
- [ ] Identify whether dashboard currently reads durable history or bounded in-memory events.
- [ ] Update docs with exact status and next tasks.

**Verification:**

```bash
rg -n 'OperationLog|operation_log|correlation|dashboard' src tests docs/product/release-gate-local-alpha.md docs/progress-tracker.md
cargo test --test operation_log -v
```

**Commit:**

```bash
git add docs/product/release-gate-local-alpha.md docs/progress-tracker.md
git commit -m "docs: classify durable observability gaps"
```

### Task E2: Correlation ID Contract

**Owner:** planner then worker.

**Purpose:** Make product debugging traceable across tool call, provider call, operation log, trigger ledger, and reflection audit.

**Files:**

- Create or modify: `docs/product/correlation-id-contract.md`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/interfaces/dashboard/event.rs`
- Modify: `src/interfaces/dashboard/mod.rs`
- Modify: `src/interfaces/dashboard/projection.rs`
- Modify: `tests/dashboard_projection.rs`
- Modify: `tests/mcp_stdio.rs`

**Checklist:**

- [x] Document correlation id format and source.
- [x] Define behavior when callers omit correlation id: the MCP runtime generates a per-call id.
- [x] Propagate generated correlation id through successful MCP operation-log entries.
- [x] Preserve correlation id in dashboard self-revision diagnostics where practical.
- [x] Add tests proving two different MCP calls get distinct correlation ids.
- [x] Preserve correlation id in dashboard detail projections.

Boundary:

- Failure dashboard events carry generated correlation ids, but failure-path durable operation-log writes are not part of this task.
- Correlation ids are observability metadata only; `run_reflection` remains the only durable identity / commitment write path.

**Verification:**

```bash
cargo test --test dashboard_projection --test mcp_stdio --test operation_log -v
```

**Commit:**

```bash
git add docs/product/correlation-id-contract.md src/interfaces/dashboard/event.rs src/interfaces/dashboard/mod.rs src/interfaces/dashboard/projection.rs src/interfaces/mcp/server.rs tests/dashboard_projection.rs tests/mcp_stdio.rs
git commit -m "feat: add correlation id contract"
```

### Task E3: Durable Operation-Log Dashboard History Query

**Owner:** worker.

**Purpose:** Let Local Alpha users inspect durable MCP tool operation history from the local read-only dashboard API instead of relying only on bounded in-memory dashboard events.

**Files:**

- Modify: `src/interfaces/dashboard/http.rs`
- Modify: `src/interfaces/dashboard/mod.rs`
- Modify: `src/interfaces/dashboard/projection.rs` if projection shape changes
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/ports/operation_log_store.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Modify: `tests/dashboard_http.rs`
- Modify: `tests/mcp_stdio.rs`
- Modify: `docs/project-status.md`
- Modify: `docs/progress-tracker.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/testing-guide-2026-03-24.md`

**Checklist:**

- [x] Expose local read-only `GET /api/operation-log` history API.
- [x] Expose local read-only `GET /api/operation-log/{id}` detail API.
- [x] Support bounded filters for `limit`, `namespace`, `kind`, and `correlation_id`.
- [x] Apply a default and maximum history list limit of 100 entries.
- [x] Wire the dashboard service to the same SQLite store used by the MCP runtime.
- [x] Keep existing `/api/events` bounded in-memory live event behavior unchanged.
- [x] Keep write methods rejected on dashboard routes.
- [x] Add tests for direct dashboard HTTP history reads.
- [x] Add MCP stdio test proving dashboard history sees durable entries from real MCP calls.

**Boundary:**

- This is a local read-only JSON API, not a remote management surface.
- It does not add dashboard write actions.
- It does not make support bundle automation complete.
- Failure-path durable operation-log entries remain a later task.

**Verification:**

```bash
cargo test --test dashboard_http --test mcp_stdio --test operation_log -v
```

**Commit:**

```bash
git add src/interfaces/dashboard/http.rs src/interfaces/dashboard/mod.rs src/interfaces/mcp/server.rs src/ports/operation_log_store.rs src/adapters/sqlite/store.rs tests/dashboard_http.rs tests/mcp_stdio.rs docs/project-status.md docs/progress-tracker.md docs/product/release-gate-local-alpha.md docs/testing-guide-2026-03-24.md docs/superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md
git commit -m "feat: expose dashboard operation log history"
```

## Milestone F: Observe-Only Daemon Gate

### Task F1: Daemon Observe-Only Gate Doc

**Owner:** planner or reviewer.

**Purpose:** Prevent daemon work from jumping directly into write-capable autonomy.

**Files:**

- Create: `docs/product/daemon-observe-only-gate.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/progress-tracker.md`

**Checklist:**

- [x] Define observe-only daemon behavior.
- [x] Define forbidden behavior: no `run_reflection`, no identity/commitment writes, no remote listener.
- [x] Define required diagnostics.
- [x] Define exit gate before daemon can trigger governed self-revision.
- [x] Link to existing `docs/superpowers/specs/2026-04-27-local-daemon-trigger-policy.md`.

**Verification:**

```bash
rg -n 'observe-only|run_reflection|forbidden|daemon|remote listener' docs/product/daemon-observe-only-gate.md docs/product/release-gate-local-alpha.md docs/roadmap.md docs/progress-tracker.md
git diff --check
```

**Commit:**

```bash
git add docs/product/daemon-observe-only-gate.md docs/product/release-gate-local-alpha.md docs/roadmap.md docs/progress-tracker.md
git commit -m "docs: gate observe-only daemon work"
```

## Recommended Execution Order

1. [x] Task A1: Local Alpha PRD
2. [x] Task A2: Local Alpha Release Gate
3. [x] Task B1: Config Profile Examples
4. [x] Task B2: Config Profile Validation Tests
5. [x] Task C1: Product Bootstrap Command
6. [x] Task C2: Product Smoke Script
7. [x] Task D1: SQLite Backup and Restore Runbook
8. [x] Task D2: Support Bundle Design
9. [x] Task E1: Runtime Operation Log Wiring Review
10. [x] Task F1: Daemon Observe-Only Gate Doc
11. [x] Task E2: Correlation ID Contract
12. [x] Task E3: Durable Operation-Log Dashboard History Query

This order keeps the first three commits documentation-heavy, then moves into scripts and tests, then only touches runtime behavior after product gates are clear.

## Review Gates

Before implementing runtime behavior:

- [x] PRD reviewed against productization roadmap.
- [x] Local Alpha release gate reviewed against current MVP release gate.
- [x] Config examples validated without leaking secrets.
- [x] Product smoke script runs locally.
- [x] Data backup/restore docs reviewed for destructive-command risk.
- [x] Daemon observe-only gate reviewed before daemon code changes.

## Final Verification for the Whole Slice

Run:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
bash scripts/product-smoke-local.sh
```

Expected:

- format, diff, clippy, tests, and doctor pass
- product smoke script exits 0
- self-revision demo artifacts exist under `target/reports/self-revision-demo/latest`
- daemon remains disabled by default
- docs still state that formal product status requires Local Product Alpha / Beta gates, not just MVP tests
