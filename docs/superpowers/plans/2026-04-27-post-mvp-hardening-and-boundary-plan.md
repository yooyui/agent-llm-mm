# Post-MVP Hardening and Boundary Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the currently usable local MCP memory MVP into a cleaner, auditable, boundary-safe demo that is easier to release, verify, and extend without prematurely claiming production-grade autonomy.

**Architecture:** Keep `run_reflection` as the only durable write path for identity and commitment updates. Keep automatic self-revision as opt-in, governed runtime hooks over the existing MCP `stdio` service. Treat dashboard, release gates, evidence policy, and schema work as separate increments so each can be verified and reviewed independently.

**Tech Stack:** Rust, Tokio, RMCP, SQLite via `sqlx`, TOML configuration, local shell/PowerShell wrappers, Markdown documentation, deterministic demo artifacts under `target/reports/`.

---

## Current Boundary Statement

This plan starts from the current confirmed state:

- The project is usable as a local Rust MCP `stdio` memory technical demo / MVP.
- The main implemented chain is `events -> claims -> self_snapshot -> decision -> reflection`.
- The automatic self-revision MVP is present, but currently limited to 4 runtime hooks:
  - `ingest_interaction:failure`
  - `ingest_interaction:conflict`
  - `decide_with_snapshot:conflict`
  - `build_self_snapshot:periodic`
- `run_reflection` remains the only durable write path for identity and commitment updates.
- The dashboard is local-only and read-only; it is not a remote admin console.

This plan explicitly does not implement the following productization/autonomy goals in the short-term MVP-hardening track:

- 完整自治 agent
- 生产级 self-governing 系统
- 远程管理后台
- 带认证、多租户、持久化 operation log 的产品化服务
- “所有入口都会自动反思”的后台 daemon

Those goals are not discarded. They are tracked as a later-stage roadmap in `docs/superpowers/plans/2026-04-27-future-autonomy-and-productization-roadmap.md`, and should only begin after the current MVP boundaries, release gates, runtime hook contract, diagnostics, and evidence policy are stable.

## Execution Rules

- Start implementation in an isolated worktree if the active workspace is dirty.
- Do not squash commits unless explicitly requested.
- Use TDD for code changes: failing test first, minimal implementation, passing test, docs update.
- Do not add a new durable self-revision write path.
- Do not add background daemon behavior in the short-term tasks.
- Do not expand dashboard into write-capable UI.
- Run spec review before implementation and code quality review before merge.
- If local subagents are used:
  - use `gpt-5.4` for planning, debugging, reviews, and multi-file implementation;
  - use `gpt-5.3-codex-spark` only for bounded searches, narrow docs checks, and small isolated edits.

## File Map

- `README.md`: public entry summary, current status, capability boundary, verification baseline.
- `docs/project-status.md`: canonical current-state vs partial vs missing-gap statement.
- `docs/progress-tracker.md`: progress table for done / partial / unimplemented work.
- `docs/roadmap.md`: near-term and mid-term development order.
- `docs/release-readiness.md`: release gate and public-positioning checklist.
- `docs/testing-guide-2026-03-24.md`: command-level verification guide.
- `docs/development-macos.md`: macOS startup and local validation flow.
- `docs/development-windows.md`: Windows startup and local validation flow.
- `docs/local-mcp-integration-2026-03-26.md`: MCP client registration flow.
- `src/interfaces/mcp/server.rs`: MCP runtime hooks and write-path constants.
- `src/application/auto_reflect_if_needed.rs`: trigger handling, proposal governance, diagnostics.
- `src/domain/self_revision.rs`: self-revision trigger and proposal contract.
- `src/adapters/sqlite/store.rs`: trigger ledger and evidence query persistence behavior.
- `tests/bootstrap.rs`: doctor, runtime coverage, entrypoint, config smoke coverage.
- `tests/mcp_stdio.rs`: end-to-end MCP behavior and auto-reflection hook coverage.
- `tests/failure_modes.rs`: rejection, suppression, cooldown, evidence-policy edge cases.
- `tests/application_use_cases.rs`: application-layer reflection and decision behavior.
- `tests/sqlite_store.rs`: persistence and query semantics.
- `tests/dashboard_*.rs`: local dashboard read-only behavior and visual contract tests.

## Phase A: Immediate Project Hygiene

### Task A1: Refresh Verification Counts and Status Docs

**Recommended owner:** docs-focused worker, `gpt-5.3-codex-spark`.

**Files:**
- Modify: `README.md`
- Modify: `docs/project-status.md`
- Modify: `docs/progress-tracker.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify if stale references remain: `docs/release-readiness.md`

- [ ] **Step 1: Confirm the fresh test count**

Run:

```bash
cargo test -- --list | rg ': test$' | wc -l
```

Expected:

```text
145
```

- [ ] **Step 2: Find stale verification numbers**

Run:

```bash
rg -n '143|80|dashboard_http`: 2|dashboard_http' README.md docs
```

Expected finding set includes the stale `143` baseline and the stale `dashboard_http`: `2` entries.

- [ ] **Step 3: Update docs to the current baseline**

Required replacements:

- `cargo test` baseline: `143` -> `145`
- `dashboard_http`: `2` -> `4`
- `docs/release-readiness.md` old `80` test note -> current `145` test baseline

Keep the project description conservative: local technical demo / MVP, not production system.

- [ ] **Step 4: Verify documentation-only changes**

Run:

```bash
git diff --check
rg -n '143|80|dashboard_http`: 2' README.md docs
```

Expected:

- `git diff --check` exits `0`
- `rg` returns no stale verification-count matches

- [ ] **Step 5: Commit**

Run:

```bash
git add README.md docs/project-status.md docs/progress-tracker.md docs/testing-guide-2026-03-24.md docs/release-readiness.md
git commit -m "docs: refresh verification baseline"
```

### Task A2: Resolve the Root-Level `failure` Artifact Decision

**Recommended owner:** narrow explorer, `gpt-5.3-codex-spark`.

**Files:**
- Inspect: `failure`
- Modify only if a durable rule is needed: `.gitignore`
- Modify only if documenting the decision is useful: `docs/project-status.md`

- [ ] **Step 1: Reconfirm artifact facts**

Run:

```bash
git status --short --branch
wc -c failure
git check-ignore -v failure || true
rg -n --hidden --glob '!target/**' --glob '!.git/**' 'failure'
```

Expected current facts:

- `failure` is untracked
- `failure` is `0` bytes
- `failure` is not ignored by `.gitignore`
- no source file depends on the root-level artifact

- [ ] **Step 2: Choose the narrowest outcome**

Use this decision table:

| Evidence | Action |
| --- | --- |
| `failure` is still empty and unreferenced | Ask for explicit approval before deleting it, or leave it untracked and mention it in status |
| `failure` is regenerated by a repeatable command | Add the exact generated path to `.gitignore` and document the generating command |
| `failure` contains useful diagnostic content | Move the content into an appropriate doc or test fixture, then remove the stray root file after approval |

- [ ] **Step 3: Verify no accidental code change**

Run:

```bash
git diff --stat
git status --short --branch
```

Expected:

- no source-code diff from this task
- `failure` is either intentionally still untracked or explicitly handled by an approved cleanup

## Phase B: Release Gate and Operator Clarity

### Task B1: Write a Release Gate Runbook

**Recommended owner:** docs worker, `gpt-5.4` if restructuring multiple docs; otherwise `gpt-5.3-codex-spark`.

**Files:**
- Create: `docs/release-gate.md`
- Modify: `README.md`
- Modify: `docs/document-map.md`
- Modify: `docs/release-readiness.md`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Create the release gate doc**

Create `docs/release-gate.md` with these sections:

```markdown
# Release Gate

## Scope

This gate is for the local MCP `stdio` technical demo / MVP. Passing it does not certify production autonomy, remote administration, multi-tenant deployment, or background daemon behavior.

## Minimum Gate

- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `./scripts/agent-llm-mm.sh doctor`

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

## Failure Interpretation

Sandbox-only failures must be called out separately from code failures. If a command writes outside the workspace or opens a local listener, rerun it in an environment that permits that operation and record both results.
```

- [ ] **Step 2: Link it from entry docs**

Add a link to `docs/release-gate.md` in:

- `README.md`
- `docs/document-map.md`
- `docs/release-readiness.md`
- `docs/testing-guide-2026-03-24.md`

- [ ] **Step 3: Verify commands in the gate**

Run:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

Expected:

- all commands exit `0`
- if sandbox restrictions block a command, rerun with appropriate local permissions and record that as an environment constraint, not as a product feature

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/release-gate.md README.md docs/document-map.md docs/release-readiness.md docs/testing-guide-2026-03-24.md
git commit -m "docs: add release gate runbook"
```

### Task B2: Clarify Local MCP Integration Troubleshooting

**Recommended owner:** docs worker, `gpt-5.3-codex-spark`.

**Files:**
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`
- Modify: `docs/local-mcp-integration-2026-03-26.md`
- Modify: `examples/codex-mcp-config.toml` only if the example is stale

- [ ] **Step 1: Add a troubleshooting matrix**

Each platform doc should cover:

| Symptom | Likely Cause | Verification | Fix |
| --- | --- | --- | --- |
| `doctor` cannot write SQLite | database path not writable or sandbox restriction | run `./scripts/agent-llm-mm.sh doctor` and inspect `database_url` | set `AGENT_LLM_MM_DATABASE_URL` or run in permitted local environment |
| MCP client starts the wrong binary | auxiliary `src/bin` target ambiguity | inspect wrapper for `--bin agent_llm_mm` | use `scripts/agent-llm-mm.sh` or `scripts/agent-llm-mm.ps1` |
| dashboard not visible | `[dashboard].enabled` is false or port unavailable | inspect TOML config and doctor output | enable dashboard and choose an available localhost port |
| model calls fail | provider config incomplete | run `doctor` and confirm provider/base_url/model | fix local TOML without committing secrets |

- [ ] **Step 2: Verify no secret leaks**

Run:

```bash
rg -n 'sk-[A-Za-z0-9]|api_key = "[^"]+"' README.md docs examples
```

Expected:

- no real API key in tracked docs or examples
- placeholder values are clearly fake

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/development-macos.md docs/development-windows.md docs/local-mcp-integration-2026-03-26.md examples/codex-mcp-config.toml
git commit -m "docs: clarify local integration troubleshooting"
```

## Phase C: Stabilize Current Automatic Self-Revision Hooks

### Task C1: Add a Runtime Hook Contract Matrix

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `docs/project-status.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `tests/bootstrap.rs`
- Modify if constants drift: `src/interfaces/mcp/server.rs`

- [ ] **Step 1: Write the expected hook contract in docs**

Add a matrix with these exact rows:

| Hook | Trigger Input | Runs When | Does Not Do |
| --- | --- | --- | --- |
| `ingest_interaction:failure` | repeated or explicit failure signal | after successful ingest path | does not turn successful ingest into MCP error if best-effort reflection fails |
| `ingest_interaction:conflict` | explicit `trigger_hints` containing `conflict` or `identity` | after successful ingest path | does not infer conflict from arbitrary text alone |
| `decide_with_snapshot:conflict` | explicit `auto_reflect_namespace` and conflict-compatible `trigger_hints` | after non-blocked decision | does not run when commitment gate blocks the decision |
| `build_self_snapshot:periodic` | explicit `auto_reflect_namespace` | during snapshot build with periodic policy | does not create a background scheduler |

- [ ] **Step 2: Ensure doctor locks the same contract**

Run:

```bash
cargo test --test bootstrap doctor_reports_self_revision_runtime_coverage -v
```

Expected:

- test passes
- doctor still reports exactly the 4 hooks above
- `self_revision_write_path` is `run_reflection`

- [ ] **Step 3: Add or adjust tests only if matrix and tests diverge**

If docs and tests diverge, update `tests/bootstrap.rs` to assert the documented hook list exactly. Do not add extra hooks in this task.

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/project-status.md docs/testing-guide-2026-03-24.md tests/bootstrap.rs src/interfaces/mcp/server.rs
git commit -m "docs: lock self-revision runtime hook contract"
```

### Task C2: Tighten Hook Failure Semantics Tests

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `tests/mcp_stdio.rs`
- Modify: `tests/failure_modes.rs`
- Modify if a real gap is found: `src/application/auto_reflect_if_needed.rs`
- Modify if a real gap is found: `src/interfaces/mcp/server.rs`

- [ ] **Step 1: Audit current hook behavior tests**

Run:

```bash
rg -n 'auto_reflect|auto_reflection|conflict|periodic|failure|blocked' tests/mcp_stdio.rs tests/failure_modes.rs
```

Expected:

- tests exist for successful hook path
- tests exist for suppressed/rejected path
- tests exist for best-effort failure not breaking main MCP path

- [ ] **Step 2: Add missing regression tests before implementation**

Only add tests for uncovered behavior. Candidate tests:

- `decide_with_snapshot_conflict_hook_requires_explicit_namespace`
- `decide_with_snapshot_conflict_hook_requires_conflict_hint`
- `build_self_snapshot_periodic_hook_requires_explicit_namespace`
- `ingest_interaction_conflict_hook_does_not_trigger_from_plain_text_without_hint`

- [ ] **Step 3: Run the new tests and confirm they fail only if behavior is missing**

Run the exact test names added in Step 2:

```bash
cargo test --test mcp_stdio <new_test_name> -v
```

Expected:

- if the behavior already exists, the test passes and no implementation change is needed
- if the behavior is missing, the test fails with a specific assertion that identifies the missing gate

- [ ] **Step 4: Implement the minimal gate or diagnostic fix**

Allowed implementation scope:

- adjust hook opt-in condition
- preserve best-effort failure semantics
- improve diagnostic result shape

Forbidden implementation scope:

- adding a fifth hook
- adding daemon behavior
- adding a new MCP tool for auto-reflection
- bypassing `run_reflection`

- [ ] **Step 5: Run targeted and full verification**

Run:

```bash
cargo test --test mcp_stdio -v
cargo test --test failure_modes -v
cargo test
./scripts/agent-llm-mm.sh doctor
```

Expected:

- all tests pass
- doctor still reports the same 4 hooks

- [ ] **Step 6: Commit**

Run:

```bash
git add tests/mcp_stdio.rs tests/failure_modes.rs src/application/auto_reflect_if_needed.rs src/interfaces/mcp/server.rs docs/testing-guide-2026-03-24.md
git commit -m "test: tighten self-revision hook semantics"
```

## Phase D: Improve Self-Revision Observability Without Productizing

### Task D1: Add a Diagnostic Summary Contract

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `src/domain/self_revision.rs`
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `tests/failure_modes.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Define the diagnostic fields that must remain inspectable**

The diagnostic surface should keep these concepts explicit:

- trigger type: `failure`, `conflict`, `periodic`
- outcome: `handled`, `rejected`, `suppressed`, `not_triggered`, `skipped`
- suppression reason
- rejection reason
- cooldown boundary
- evidence window size
- selected evidence ids
- durable write path: `run_reflection`

- [ ] **Step 2: Write failing tests for missing diagnostic fields**

Add focused tests in `tests/failure_modes.rs` if any field above is not currently asserted. Use existing constructors and fixtures in the file; do not introduce a second fake runtime.

Run:

```bash
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_rejected_proposal -v
```

Expected:

- existing assertions pass, or new assertions fail on a specific missing field

- [ ] **Step 3: Implement minimal diagnostic enrichment**

Only change the diagnostic payload or helper methods needed by the tests. Do not change trigger decisions in this task.

- [ ] **Step 4: Document how to inspect diagnostics**

Update `docs/testing-guide-2026-03-24.md` with:

- targeted tests for each diagnostic case
- how to distinguish `rejected` from `suppressed`
- why these diagnostics do not imply background autonomy

- [ ] **Step 5: Verify**

Run:

```bash
cargo test --test failure_modes -v
cargo test --test mcp_stdio -v
cargo test
```

Expected:

- all tests pass
- no new MCP tool appears

- [ ] **Step 6: Commit**

Run:

```bash
git add src/domain/self_revision.rs src/application/auto_reflect_if_needed.rs tests/failure_modes.rs docs/testing-guide-2026-03-24.md
git commit -m "feat: clarify self-revision diagnostics"
```

## Phase E: Reflection Contract Hardening

### Task E1: Specify Deeper-Update Input Rules

**Recommended owner:** planner/docs worker, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-reflection-deeper-update-contract.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`

- [ ] **Step 1: Write the contract spec**

Create a spec with these sections:

- Current supported updates
- Required supporting evidence
- Disallowed direct identity writes
- Commitment replacement behavior
- Audit record expectations
- What is intentionally not implemented: richer schema, versioned policy engine, autonomous slow-variable formation

- [ ] **Step 2: Tie spec to status docs**

Link the spec from `docs/project-status.md` and `docs/roadmap.md` under the reflection deeper-update sections.

- [ ] **Step 3: Verify no behavior claim exceeds tests**

Run:

```bash
rg -n 'complete autonomous|production-grade|daemon|all entry|所有入口|完整自治|生产级' docs/superpowers/specs/2026-04-27-reflection-deeper-update-contract.md docs/project-status.md docs/roadmap.md
```

Expected:

- matches only appear in explicit non-goal or boundary sections

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-reflection-deeper-update-contract.md docs/project-status.md docs/roadmap.md
git commit -m "docs: specify reflection deeper-update contract"
```

### Task E2: Add Contract Tests for Reflection Updates

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `tests/application_use_cases.rs`
- Modify: `tests/mcp_stdio.rs`
- Modify if gaps are found: `src/application/run_reflection.rs`
- Modify if gaps are found: `src/adapters/sqlite/store.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Add tests for contract boundaries**

Candidate tests:

- reflection rejects identity update without supporting evidence
- reflection records requested identity update in audit payload
- reflection preserves baseline hard commitment unless replacement is explicit and supported
- MCP `run_reflection` reports `invalid_params` for missing evidence in deeper updates

- [ ] **Step 2: Run each new test and confirm current behavior**

Run each new test directly:

```bash
cargo test --test application_use_cases <new_test_name> -v
cargo test --test mcp_stdio <new_test_name> -v
```

Expected:

- tests pass if behavior already exists
- any failure identifies a narrow contract gap

- [ ] **Step 3: Implement only the narrow contract gap**

Allowed:

- stricter evidence validation
- clearer invalid-params mapping
- audit payload preservation

Forbidden:

- richer identity schema
- versioned policy engine
- background automatic identity formation

- [ ] **Step 4: Verify**

Run:

```bash
cargo test --test application_use_cases -v
cargo test --test mcp_stdio -v
cargo test --test sqlite_store -v
cargo test
```

Expected:

- all tests pass
- no new durable write path exists

- [ ] **Step 5: Commit**

Run:

```bash
git add tests/application_use_cases.rs tests/mcp_stdio.rs src/application/run_reflection.rs src/adapters/sqlite/store.rs docs/testing-guide-2026-03-24.md
git commit -m "test: harden reflection update contract"
```

## Phase F: Evidence Query v2, Still Narrow

### Task F1: Design Evidence-Oriented Query v2

**Recommended owner:** planner, `gpt-5.4`.

**Files:**
- Create: `docs/superpowers/specs/2026-04-27-evidence-query-v2.md`
- Modify: `docs/roadmap.md`

- [ ] **Step 1: Define the v2 scope**

Allowed v2 additions:

- explicit namespace filter
- evidence kind filter
- bounded recency window
- deterministic limit behavior
- clear no-match behavior

Deferred from v2:

- model-based ranking
- relation graph
- weight scoring
- cross-namespace widening
- autonomous evidence search outside the trigger window

- [ ] **Step 2: Document interaction with self-revision proposals**

The spec must state:

- `proposed_evidence_query` narrows within the current trigger window.
- explicit evidence ids remain authoritative only if they satisfy server-side validation.
- empty query results must not widen beyond the governed trigger window.

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/superpowers/specs/2026-04-27-evidence-query-v2.md docs/roadmap.md
git commit -m "docs: design evidence query v2"
```

### Task F2: Implement One Evidence Query v2 Slice

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `src/domain/evidence_link.rs` if query shape changes
- Modify: `src/ports/event_store.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `tests/sqlite_store.rs`
- Modify: `tests/failure_modes.rs`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Pick exactly one v2 slice**

Recommended first slice:

- namespace-aware narrowing inside the existing trigger window

Do not implement ranking, weighting, or relation graph in this task.

- [ ] **Step 2: Write failing SQLite tests**

Add tests in `tests/sqlite_store.rs`:

- query returns only events inside the requested namespace
- query preserves recent-first deterministic order
- query enforces limit inside the filtered candidate set

- [ ] **Step 3: Write failing governance tests**

Add tests in `tests/failure_modes.rs`:

- self-revision proposal cannot use namespace filter to widen beyond trigger window
- no-match namespace filter falls back according to the documented current-window rule

- [ ] **Step 4: Implement minimal query support**

Change only the domain query shape, port method, SQLite filtering, and governance intersection needed by the tests.

- [ ] **Step 5: Verify**

Run:

```bash
cargo test --test sqlite_store -v
cargo test --test failure_modes -v
cargo test --test mcp_stdio -v
cargo test
```

Expected:

- all tests pass
- existing MVP behavior remains bounded and deterministic

- [ ] **Step 6: Commit**

Run:

```bash
git add src/domain/evidence_link.rs src/ports/event_store.rs src/adapters/sqlite/store.rs src/application/auto_reflect_if_needed.rs tests/sqlite_store.rs tests/failure_modes.rs docs/testing-guide-2026-03-24.md
git commit -m "feat: add namespace-aware evidence narrowing"
```

## Phase G: Dashboard Local-Only Hardening

### Task G1: Preserve Read-Only Dashboard Boundary

**Recommended owner:** implementation worker, `gpt-5.4`.

**Files:**
- Modify: `src/interfaces/dashboard/http.rs`
- Modify: `src/interfaces/dashboard/projection.rs`
- Modify: `tests/dashboard_http.rs`
- Modify: `docs/project-status.md`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Add explicit tests for read-only route surface**

In `tests/dashboard_http.rs`, assert:

- `GET` routes work for dashboard HTML, summary, events detail, and health
- unsupported write methods return a non-success status
- no route invokes `run_reflection`
- MCP stdout safety test still passes

- [ ] **Step 2: Run tests**

Run:

```bash
cargo test --test dashboard_http -v
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
```

Expected:

- dashboard remains local read-only observability
- MCP `stdout` remains clean

- [ ] **Step 3: Update docs**

Docs must keep these statements:

- local-only
- read-only
- bounded in-memory recorder
- not authentication/multi-tenant/admin product
- not durable operation-log database

- [ ] **Step 4: Commit**

Run:

```bash
git add src/interfaces/dashboard/http.rs src/interfaces/dashboard/projection.rs tests/dashboard_http.rs docs/project-status.md docs/testing-guide-2026-03-24.md
git commit -m "test: preserve dashboard read-only boundary"
```

## Phase H: Provider Contract Readiness

### Task H1: Add Provider Contract Checklist Before New Providers

**Recommended owner:** docs worker, `gpt-5.3-codex-spark`.

**Files:**
- Create: `docs/provider-contract.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/document-map.md`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: Write provider contract checklist**

The checklist must include:

- config validation behavior
- doctor redaction behavior
- timeout handling
- non-success HTTP status behavior
- malformed JSON behavior
- decision action parsing
- self-revision proposal parsing
- evidence policy parsing

- [ ] **Step 2: Link existing tests**

Reference existing coverage in:

- `tests/provider_config.rs`
- `tests/openai_compatible_model.rs`
- `tests/mcp_stdio.rs`

- [ ] **Step 3: Commit**

Run:

```bash
git add docs/provider-contract.md docs/roadmap.md docs/document-map.md docs/testing-guide-2026-03-24.md
git commit -m "docs: add provider contract checklist"
```

## Review Gates

### Spec Review Gate

Run before implementing any phase after Phase A:

```bash
rg -n '完整自治|生产级|remote admin|远程管理|daemon|所有入口|multi-tenant|多租户|durable operation' README.md docs
```

Expected:

- matches appear only as explicit boundaries, non-goals, or deferred future work
- no doc implies the current system has production autonomy

### Code Quality Review Gate

Run before merging implementation work:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

Expected:

- all commands exit `0`
- if sandbox blocks SQLite or local listener behavior, rerun in an environment that permits those operations and record the distinction

### Demo Evidence Gate

Run when a task touches self-revision, provider, MCP stdio, or demo package:

```bash
cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

Expected:

- demo artifacts are regenerated
- `doctor.json` reports `status = ok`
- `doctor.json` reports `self_revision_write_path = run_reflection`
- before/after decision artifact still proves the canonical decision shift

## Recommended Execution Order

1. Phase A: fix verification drift and resolve the `failure` artifact decision.
2. Phase B: add the release gate and local integration troubleshooting.
3. Phase C: lock the current 4 runtime hooks before adding any capability.
4. Phase D: improve diagnostics without changing runtime scope.
5. Phase E: harden reflection deeper-update contract.
6. Phase F: implement one bounded evidence query v2 slice.
7. Phase G: preserve dashboard local-only read-only boundary.
8. Phase H: prepare provider contract before adding more providers.

Do not start daemon/productization work until Phases A through F are complete and reviewed. When that work starts, switch to `docs/superpowers/plans/2026-04-27-future-autonomy-and-productization-roadmap.md` instead of expanding this short-term hardening plan inline.

## Copyable Handoff for Next Session

```text
Work in /Users/yooyui/code/agent-llm-mm. Follow AGENTS.md. Keep the current execution scope framed as local Rust MCP stdio memory MVP hardening. Do not add production autonomy, remote admin behavior, multi-tenant/auth productization, durable operation-log database, or an all-entry auto-reflection daemon in this track; those are planned later in docs/superpowers/plans/2026-04-27-future-autonomy-and-productization-roadmap.md.

Use docs/superpowers/plans/2026-04-27-post-mvp-hardening-and-boundary-plan.md as the execution plan. Start with Phase A. If the workspace is dirty, create an isolated worktree first. Use TDD for code changes. Run spec review before implementation and code quality review before merge. Preserve run_reflection as the only durable self-revision write path.

First commands:
git status --short --branch
cargo test -- --list | rg ': test$' | wc -l
rg -n '143|80|dashboard_http`: 2|dashboard_http' README.md docs
```

## Self-Review Notes

- The plan covers immediate doc drift, release gate clarity, runtime hook stabilization, diagnostics, reflection contract, bounded evidence-query expansion, dashboard boundary, and provider-readiness work.
- The plan explicitly defers complete autonomy, production self-governance, remote admin, multi-tenant/auth productization, durable operation logs, and all-entry background daemon behavior to the separate future roadmap.
- Each implementation phase has concrete files, commands, expected results, and commit boundaries.
- Code-changing phases preserve the repo rule that `run_reflection` remains the durable write path.
