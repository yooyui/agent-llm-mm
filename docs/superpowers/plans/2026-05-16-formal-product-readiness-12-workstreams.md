# Formal Product Readiness 12 Workstreams Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the current validated local MVP entering productization into a staged formal-product track with clear Local Alpha, Beta, remote/team, and GA readiness work.

**Architecture:** Keep the local MCP `stdio` service, SQLite store, governed `run_reflection` write path, local read-only dashboard, support bundle, operation log, and observe-only daemon diagnostics as the product core. Add missing product capabilities in gates: Local Alpha evidence first, then install/configuration, runtime coverage, daemon lifecycle, observability, decision protocol, evidence semantics, data lifecycle, provider matrix, security/remote/team mode, release operations, and deeper memory layering. No task may claim GA, production self-governance, remote write admin, or multi-tenancy until its gate is implemented and freshly verified.

**Tech Stack:** Rust, Tokio, RMCP `stdio`, SQLite, TOML, shell / PowerShell scripts, local dashboard HTTP surface, operation log store, deterministic self-revision demo artifacts, Markdown runbooks, integration tests, future auth / remote-admin design docs.

---

## Current Baseline

Current product wording remains:

- `validated local MVP entering productization`
- `Local Product Alpha in progress`
- `run_reflection` is still the only durable identity / commitment / reflection write path
- observe-only daemon diagnostics are local read-only diagnostics, not daemon write capability

Fresh baseline from the current `dev-work` merge:

- `cargo fmt --check` passed
- `git diff --check HEAD^ HEAD` passed
- `cargo clippy --all-targets --all-features -- -D warnings` passed
- `cargo test` passed with 194 tests
- `./scripts/agent-llm-mm.sh doctor` returned `status = ok`
- `doctor.self_revision_write_path = "run_reflection"`
- `doctor.daemon_observe_only.writes_allowed = false`

This plan starts from that state. It does not replace:

- `docs/superpowers/plans/2026-05-09-productization-roadmap.md`
- `docs/superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md`
- `docs/product/release-gate-local-alpha.md`
- `docs/progress-tracker.md`

It turns the 12 remaining product-readiness gaps into execution workstreams.

## Phase Order

| Phase | Workstreams | Product Claim After Completion |
| --- | --- | --- |
| Phase A | 1, 2 | Local Alpha can be claimed only if all Local Alpha gates have fresh evidence |
| Phase B | 3, 4, 5 | Local Alpha with durable observability and observe-only daemon readiness |
| Phase C | 6, 7, 8, 9 | Controlled Beta candidate foundations |
| Phase D | 10, 11 | Remote/team and formal release readiness foundations |
| Phase E | 12 | Deeper memory product direction after core product gates stabilize |

## Workstream 1: Local Alpha Full Release Gate Evidence

**Purpose:** Produce fresh, complete Local Alpha evidence instead of relying on MVP gate results or stale artifacts.

**Files:**

- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/progress-tracker.md`
- Modify: `docs/project-status.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Generate / inspect: `target/reports/self-revision-demo/latest/`
- Generate / inspect: `target/support-bundles/local-alpha-gate/`

**Implementation steps:**

- [ ] **Step 1: Verify clean baseline before gate run**

  Run:

  ```bash
  git status --short --branch
  cargo fmt --check
  git diff --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  ./scripts/agent-llm-mm.sh doctor
  ```

  Expected:

  - branch is intentional for the release-gate run
  - no unexpected dirty files before generated artifacts
  - every command exits with code `0`
  - `doctor.status = "ok"`
  - `doctor.self_revision_write_path = "run_reflection"`

- [ ] **Step 2: Run product smoke from a clean tree**

  Run:

  ```bash
  rm -rf target/reports/self-revision-demo/latest
  ./scripts/product-smoke-local.sh
  ```

  Expected:

  - script exits with code `0`
  - `target/reports/self-revision-demo/latest/doctor.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/snapshot-before.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/snapshot-after.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/decision-before.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/decision-after.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/timeline.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/sqlite-summary.json` exists and is non-empty
  - `target/reports/self-revision-demo/latest/report.md` exists and is non-empty

- [ ] **Step 3: Generate a Local Alpha support bundle**

  Run:

  ```bash
  rm -rf target/support-bundles/local-alpha-gate
  ./scripts/generate-support-bundle.sh target/support-bundles/local-alpha-gate
  find target/support-bundles/local-alpha-gate -maxdepth 1 -type f -print | sort
  rg -n 'api_key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///' target/support-bundles/local-alpha-gate || true
  find target/support-bundles/local-alpha-gate \( -name '*.sqlite' -o -name '*.toml' \) -print
  ```

  Expected:

  - bundle generation exits with code `0`
  - bundle contains only the allowed JSON files
  - secret search returns no unredacted secrets
  - no SQLite or raw TOML files are included
  - `manifest.json` indicates no upload was performed

- [ ] **Step 4: Record fresh gate evidence**

  Update:

  - `docs/product/release-gate-local-alpha.md`: add a dated evidence note with commands and pass/fail status
  - `docs/progress-tracker.md`: move Local Alpha gate evidence from open to fresh evidence if every gate passed
  - `docs/project-status.md`: update the latest validation date and test count if unchanged or changed
  - `docs/testing-guide-2026-03-24.md`: ensure the recommended Local Alpha order matches the real command sequence

- [ ] **Step 5: Commit the evidence docs**

  Run:

  ```bash
  git diff -- docs/product/release-gate-local-alpha.md docs/progress-tracker.md docs/project-status.md docs/testing-guide-2026-03-24.md
  git status --short
  git add docs/product/release-gate-local-alpha.md docs/progress-tracker.md docs/project-status.md docs/testing-guide-2026-03-24.md
  git commit -m "docs: record local alpha gate evidence"
  ```

## Workstream 2: Install and First-Run Configuration Experience

**Purpose:** Make Local Alpha usable by a real local user on a fresh machine, not just by the current development checkout.

**Files:**

- Modify: `scripts/agent-llm-mm.sh`
- Modify: `scripts/agent-llm-mm.ps1`
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`
- Modify: `docs/product/prd-local-alpha.md`
- Modify: `examples/agent-llm-mm.example.toml`
- Modify: `examples/agent-llm-mm.demo.example.toml`
- Modify: `examples/codex-mcp-config.toml`
- Test: `tests/bootstrap.rs`
- Test: `tests/provider_config.rs`

**Implementation steps:**

- [ ] **Step 1: Define the install/bootstrap contract**

  Add a concise contract to `docs/product/prd-local-alpha.md`:

  ```markdown
  ### First-Run Contract

  A Local Alpha user can clone or unpack the repository, copy one example config, run `doctor`, and start `serve` without editing source files. The supported first-run path is doctor-first: users must be able to validate config, provider shape, database path, dashboard status, daemon status, and runtime hooks before starting the MCP service.
  ```

- [ ] **Step 2: Add or confirm a bootstrap command**

  If the existing scripts already satisfy the contract, document them without adding new modes. If a mode is needed, add `bootstrap-local` with these required behaviors:

  - refuse unsupported platforms with a clear message
  - create no secrets
  - copy no config over an existing file
  - print next commands for `doctor` and `serve`

- [ ] **Step 3: Add tests for unsupported modes and config path handling**

  Extend `tests/bootstrap.rs` and `tests/provider_config.rs` to verify:

  - wrapper scripts still pin `--bin agent_llm_mm`
  - unsupported modes return exit code `2`
  - config examples parse without real secrets
  - `doctor` serialization does not expose API keys
  - daemon is disabled by default in all examples

  Run:

  ```bash
  cargo test --test bootstrap --test provider_config -v
  ```

- [ ] **Step 4: Document macOS and Windows separately**

  Update:

  - `docs/development-macos.md`: macOS install, config copy, `doctor`, `serve`, product smoke
  - `docs/development-windows.md`: Windows install, config copy, `doctor`, `serve`, product smoke parity status

  Do not mix PowerShell commands into the macOS flow or shell commands into the Windows flow.

- [ ] **Step 5: Verify first-run docs**

  Run on macOS:

  ```bash
  ./scripts/agent-llm-mm.sh doctor
  ./scripts/product-smoke-local.sh
  ```

  Run for Windows script syntax if PowerShell is available:

  ```bash
  pwsh -NoProfile -Command "& ./scripts/agent-llm-mm.ps1 doctor"
  ```

  If PowerShell is not available locally, document that Windows parity requires a Windows runner or manual Windows verification.

## Workstream 3: Automatic Self-Revision Runtime Coverage

**Purpose:** Make the existing four automatic self-revision hooks easier to verify and troubleshoot before expanding trigger coverage.

**Files:**

- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `src/support/doctor.rs`
- Modify: `docs/project-status.md`
- Modify: `docs/local-mcp-integration-2026-03-26.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Test: `tests/mcp_stdio.rs`
- Test: `tests/failure_modes.rs`
- Test: `tests/bootstrap.rs`

**Implementation steps:**

- [ ] **Step 1: Preserve the current four-hook contract**

  Keep the hook list exactly visible in `doctor` unless a separate spec changes it:

  - `ingest_interaction:failure`
  - `ingest_interaction:conflict`
  - `decide_with_snapshot:conflict`
  - `build_self_snapshot:periodic`

- [ ] **Step 2: Add focused regression tests for opt-in boundaries**

  Extend `tests/mcp_stdio.rs` to prove:

  - `ingest_interaction:conflict` does not trigger from arbitrary text without conflict-compatible `trigger_hints`
  - `decide_with_snapshot:conflict` does not run when the decision is blocked by the commitment gate
  - `build_self_snapshot:periodic` requires explicit `auto_reflect_namespace`
  - direct `run_reflection` does not recursively trigger auto-reflection

  Run:

  ```bash
  cargo test --test mcp_stdio auto_reflect -- --nocapture
  cargo test --test failure_modes -v
  ```

- [ ] **Step 3: Improve diagnostics without widening behavior**

  If current diagnostics are insufficient, add bounded fields that explain:

  - trigger type
  - namespace
  - handled / rejected / suppressed status
  - cooldown state
  - evidence window size

  Do not add a new MCP tool and do not add a new durable write path.

- [ ] **Step 4: Update integration docs**

  Update `docs/local-mcp-integration-2026-03-26.md` and `docs/testing-guide-2026-03-24.md` with:

  - exact trigger preconditions
  - expected best-effort failure behavior
  - commands for targeted regression tests
  - examples of when auto-reflection should not run

## Workstream 4: Observe-Only Daemon Lifecycle Readiness

**Purpose:** Prepare daemon lifecycle behavior while keeping daemon writes blocked.

**Files:**

- Modify: `src/application/daemon.rs`
- Modify: `src/support/config.rs`
- Modify: `src/support/doctor.rs`
- Modify: `docs/product/daemon-observe-only-gate.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Test: `tests/daemon_config.rs`
- Test: `tests/operation_log.rs`

**Implementation steps:**

- [ ] **Step 1: Keep daemon disabled by default**

  Preserve these default expectations:

  - `[daemon].enabled = false`
  - no background loop starts during `doctor`
  - no remote listener is configured
  - `writes_allowed = false`
  - `write_gate_approved = false`

- [ ] **Step 2: Add lifecycle design before code changes**

  Extend `docs/product/daemon-observe-only-gate.md` with:

  - start condition
  - stop condition
  - safe shutdown behavior
  - orphaned-work recovery behavior
  - observed-only candidate scan interval
  - proof that candidate scans do not call `run_reflection`

- [ ] **Step 3: Test observe-only diagnostics**

  Run:

  ```bash
  cargo test --test daemon_config -v
  cargo test --test operation_log -v
  ./scripts/agent-llm-mm.sh doctor
  ```

  Expected:

  - diagnostics read only local `daemon_config` and `operation_log`
  - semantic memory tables are not written by diagnostics after normal doctor bootstrap
  - failed / suppressed candidate counts remain bounded

- [ ] **Step 4: Add lifecycle code only after the design review**

  If lifecycle implementation is approved, implement observe-only start/stop behind `[daemon].enabled = true`. The implementation must:

  - run without identity / commitment / claim / event / reflection writes
  - record only allowed operation metadata if operation-log wiring is in scope
  - shut down cleanly on cancellation
  - expose enough status for `doctor` or local diagnostics

## Workstream 5: Product-Grade Observability and Support Bundle Diagnostics

**Purpose:** Make runtime behavior diagnosable without leaking secrets or raw provider payloads.

**Files:**

- Modify: `src/domain/operation_log.rs`
- Modify: `src/ports/operation_log_store.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Modify: `src/interfaces/dashboard/http.rs`
- Modify: `src/interfaces/dashboard/projection.rs`
- Modify: `src/support/support_bundle.rs`
- Modify: `docs/product/support-bundle-local-alpha.md`
- Modify: `docs/product/correlation-id-contract.md`
- Test: `tests/operation_log.rs`
- Test: `tests/dashboard_http.rs`
- Test: `tests/mcp_stdio.rs`
- Test: `tests/support_bundle.rs`

**Implementation steps:**

- [ ] **Step 1: Map observability coverage**

  Update `docs/product/correlation-id-contract.md` with a table covering:

  - MCP call
  - model call
  - operation-log entry
  - dashboard event
  - trigger ledger entry
  - reflection audit entry

  Mark each as implemented, partial, or future. Do not mark model / ledger / reflection correlation as implemented unless tests prove it.

- [ ] **Step 2: Add missing metadata tests before implementation**

  Add tests that prove:

  - known handler-level tool failures append safe operation-log metadata
  - framework-level parse/router failures remain documented if not logged
  - dashboard history never exposes raw provider payloads
  - support bundle operation summaries remain bounded and redacted

  Run:

  ```bash
  cargo test --test operation_log --test dashboard_http --test mcp_stdio --test support_bundle -v
  ```

- [ ] **Step 3: Extend support bundle only after redaction rules exist**

  Log excerpts may be added only if:

  - log locations are deterministic
  - redaction terms are test-covered
  - output is bounded
  - raw request / response payloads remain excluded by default

- [ ] **Step 4: Re-run secret scan**

  Run:

  ```bash
  rm -rf target/support-bundles/observability-check
  ./scripts/generate-support-bundle.sh target/support-bundles/observability-check
  rg -n 'api_key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///' target/support-bundles/observability-check || true
  find target/support-bundles/observability-check \( -name '*.sqlite' -o -name '*.toml' \) -print
  ```

## Workstream 6: Structured Decision Protocol

**Purpose:** Move `decide_with_snapshot` from a minimal action string toward a versioned, explainable decision payload without breaking existing callers.

**Files:**

- Modify: `src/domain/decision.rs`
- Modify: `src/application/decide_with_snapshot.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/adapters/model/openai_compatible.rs`
- Modify: `docs/project-status.md`
- Modify: `docs/provider-contract.md`
- Test: `tests/decision_flow.rs`
- Test: `tests/mcp_stdio.rs`
- Test: `tests/openai_compatible_model.rs`

**Implementation steps:**

- [ ] **Step 1: Write a decision protocol spec**

  Create or update a spec section that defines:

  - protocol version
  - action
  - rationale
  - blocked state
  - commitment-gate reason
  - provider diagnostics class
  - backward-compatible minimal action field

- [ ] **Step 2: Add compatibility tests**

  Tests must prove:

  - existing callers still receive the current field shape or compatible aliases
  - blocked decisions still return `blocked = true` and no unsafe provider action
  - provider malformed JSON remains a safe error
  - decision payload does not include secrets

  Run:

  ```bash
  cargo test --test decision_flow --test mcp_stdio --test openai_compatible_model -v
  ```

- [ ] **Step 3: Implement the smallest protocol extension**

  Add fields only after tests define their behavior. Do not make the decision engine claim planning, tool execution, or autonomous control.

## Workstream 7: Evidence and Reflection Semantics V2

**Purpose:** Improve evidence lookup and reflection deep-update semantics while keeping server-side governance strict.

**Files:**

- Modify: `src/domain/evidence_query.rs`
- Modify: `src/application/run_reflection.rs`
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Modify: `docs/superpowers/specs/2026-04-27-evidence-query-v2.md`
- Modify: `docs/superpowers/specs/2026-04-27-reflection-deeper-update-contract.md`
- Test: `tests/evidence_query_dto.rs`
- Test: `tests/sqlite_store.rs`
- Test: `tests/failure_modes.rs`
- Test: `tests/application_use_cases.rs`

**Implementation steps:**

- [ ] **Step 1: Lock current no-widening behavior**

  Add tests proving:

  - `proposed_evidence_query` only narrows within the current trigger window
  - empty query intersections return rejection / invalid params as currently documented
  - explicit evidence ids must pass server-side validation
  - namespace filters do not gather sibling namespace evidence

- [ ] **Step 2: Add one evidence-v2 slice**

  Pick one bounded extension, such as independent evidence kind semantics. Do not add weighting, ranking, relation graph, and cross-window search in the same slice.

- [ ] **Step 3: Update reflection contract**

  Document:

  - accepted evidence fields
  - rejected widening behavior
  - audit payload shape
  - fallback behavior when no evidence matches

- [ ] **Step 4: Run focused tests**

  Run:

  ```bash
  cargo test --test evidence_query_dto --test sqlite_store --test failure_modes --test application_use_cases -v
  ```

## Workstream 8: Data Lifecycle, Backup, Restore, and Migration

**Purpose:** Protect user data before Beta by making retention, backup, restore, export, and migration behavior explicit and testable.

**Files:**

- Modify: `docs/product/prd-local-alpha.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/development-macos.md`
- Modify: `docs/development-windows.md`
- Create: `docs/product/data-lifecycle.md`
- Test: `tests/sqlite_store.rs`
- Test: future migration tests under `tests/`

**Implementation steps:**

- [ ] **Step 1: Create the data lifecycle doc**

  Create `docs/product/data-lifecycle.md` with:

  - formal database profile separation
  - backup command
  - restore-to-new-path default
  - export boundaries
  - retention expectations for events, claims, reflections, operation log, and demo artifacts
  - migration verification checklist

- [ ] **Step 2: Add backup / restore verification commands**

  Document commands for SQLite backup:

  ```bash
  sqlite3 /path/to/agent-llm-mm.sqlite ".backup '/path/to/backup.sqlite'"
  shasum -a 256 /path/to/backup.sqlite
  ```

  Document restore verification:

  ```bash
  AGENT_LLM_MM_DATABASE_URL="sqlite:///path/to/restored-copy.sqlite" ./scripts/agent-llm-mm.sh doctor
  ```

- [ ] **Step 3: Add migration tests when schema changes**

  Every future schema migration must include:

  - legacy database fixture or setup SQL
  - bootstrap verification
  - data preservation check
  - rollback / failure note if rollback is not supported

- [ ] **Step 4: Link data lifecycle docs**

  Link `docs/product/data-lifecycle.md` from:

  - `docs/document-map.md`
  - `docs/product/release-gate-local-alpha.md`
  - `docs/development-macos.md`
  - `docs/development-windows.md`

## Workstream 9: Provider Matrix and Real-Provider Readiness

**Purpose:** Make provider support predictable beyond `mock` and the first `openai-compatible` path.

**Files:**

- Modify: `docs/provider-contract.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `examples/agent-llm-mm.example.toml`
- Modify: `src/support/config.rs`
- Modify: `src/adapters/model/`
- Test: `tests/provider_config.rs`
- Test: `tests/openai_compatible_model.rs`
- Test: `tests/mcp_stdio.rs`

**Implementation steps:**

- [ ] **Step 1: Create provider matrix section**

  Add a matrix covering:

  - `mock`
  - `openai-compatible`
  - Azure OpenAI candidate
  - OpenRouter candidate
  - local model gateway candidate

  Columns:

  - config keys
  - timeout behavior
  - non-success status behavior
  - malformed response behavior
  - redaction coverage
  - retry policy
  - decision parsing
  - self-revision parsing

- [ ] **Step 2: Add tests before each provider adapter**

  For any new provider, first add config and model tests for:

  - missing key rejection
  - `doctor` redaction
  - timeout classification
  - non-success status
  - malformed JSON
  - fenced JSON proposal parsing if applicable

- [ ] **Step 3: Keep app layer provider-agnostic**

  Provider-specific protocol details must stay under adapters / config. Application and domain code should continue using ports.

- [ ] **Step 4: Run provider verification**

  Run:

  ```bash
  cargo test --test provider_config --test openai_compatible_model --test mcp_stdio -v
  ./scripts/agent-llm-mm.sh doctor
  ```

## Workstream 10: Remote, Team Mode, Auth, and Security Gates

**Purpose:** Design remote/team features without accidentally shipping write-capable remote admin too early.

**Files:**

- Create: `docs/product/remote-team-mode-boundary.md`
- Create: `docs/security/threat-model-local-and-remote.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `docs/superpowers/plans/2026-05-09-productization-roadmap.md`

**Implementation steps:**

- [ ] **Step 1: Write remote/team boundary doc**

  Create `docs/product/remote-team-mode-boundary.md` with:

  - remote read-only dashboard as the first remote surface
  - no remote writes before auth, authorization, audit, rollback, and tests
  - namespace / database isolation expectations
  - transport decision: MCP `stdio` core remains local; HTTP only for admin/API surfaces that need it
  - explicit non-goals for Local Alpha

- [ ] **Step 2: Write threat model**

  Create `docs/security/threat-model-local-and-remote.md` covering:

  - assets: SQLite data, provider credentials, reflection audit, operation log, support bundles
  - local attacker capabilities
  - remote attacker capabilities
  - trust boundaries
  - write-admin abuse paths
  - mitigations required before remote/team mode

- [ ] **Step 3: Add release gate blocks**

  Update release docs to block these claims until gates pass:

  - remote write admin
  - team mode
  - multi-tenancy
  - production support
  - production self-governance

- [ ] **Step 4: Review before implementation**

  Run a code-review style doc review before any remote/admin code starts. The review must list blockers, not just summarize docs.

## Workstream 11: Formal Release Engineering

**Purpose:** Make releases repeatable and understandable before Beta or GA claims.

**Files:**

- Create: `docs/product/release-engineering.md`
- Modify: `CHANGELOG.md` if present, otherwise create release notes under `docs/releases/`
- Modify: `docs/release-readiness.md`
- Modify: `docs/release-gate.md`
- Modify: `docs/product/release-gate-local-alpha.md`
- Modify: `CONTRIBUTING.md`

**Implementation steps:**

- [ ] **Step 1: Create release engineering doc**

  Create `docs/product/release-engineering.md` with:

  - release artifact shape
  - version naming
  - changelog rules
  - release evidence directory rules
  - compatibility matrix requirements
  - soak test command requirements
  - deprecation policy

- [ ] **Step 2: Define artifact packaging**

  Decide whether the first release artifact is:

  - source-only tag
  - built binary archive
  - platform-specific package

  Document the chosen first artifact and the reason. Do not add packaging automation until the artifact contract is written.

- [ ] **Step 3: Add long-run smoke / soak command**

  Define a repeatable command that can run without external writes. It should cover:

  - `doctor`
  - product smoke
  - dashboard health when enabled
  - support bundle generation
  - no secret leakage scan

- [ ] **Step 4: Update contribution workflow**

  Update `CONTRIBUTING.md` with:

  - required verification before release commits
  - doc update expectation
  - product claim wording rules
  - no remote write/admin claims before the remote/team gate

## Workstream 12: Multi-Layer Memory Product Direction

**Purpose:** Preserve the larger memory vision without blocking the product gates on speculative architecture.

**Files:**

- Create: `docs/product/memory-layering-roadmap.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/progress-tracker.md`
- Modify: `docs/project-status.md`
- Modify: `docs/superpowers/specs/2026-04-27-future-autonomy-productization-index.md`

**Implementation steps:**

- [ ] **Step 1: Write memory layering roadmap**

  Create `docs/product/memory-layering-roadmap.md` with staged definitions for:

  - working memory
  - episodic memory
  - semantic memory
  - procedural memory
  - slow variables
  - self-model layering

- [ ] **Step 2: Define prerequisites**

  State that deeper layering starts only after:

  - evidence semantics are stable
  - reflection policy is stable
  - schema migration policy is tested
  - data lifecycle and backup gates are in place
  - product wording still separates MVP, Local Alpha, Beta, remote/team, and GA

- [ ] **Step 3: Split first implementable slice**

  The first memory-layering implementation slice should be narrow. A good first slice is richer `episodes` semantics with:

  - goal
  - outcome
  - lesson
  - linked evidence ids
  - snapshot projection test

- [ ] **Step 4: Keep speculative layers out of Local Alpha claims**

  Update docs so Local Alpha remains local MCP memory plus governed self-revision, not a full multi-layer cognitive architecture.

## Cross-Workstream Verification

Run this command set before claiming a workstream is complete:

```bash
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

For Local Alpha evidence work, also run:

```bash
./scripts/product-smoke-local.sh
rm -rf target/support-bundles/manual-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
```

For documentation-only work, run at minimum:

```bash
rg -n 'GA|production-ready|self-governing|remote write|multi-tenancy|Local Product Alpha' README.md docs
git diff --check
```

Expected documentation result:

- no new text claims GA readiness
- no new text claims production self-governance
- no new text claims remote write admin is implemented
- no new text claims multi-tenancy is implemented
- Local Alpha claims are tied to fresh gate evidence

## Suggested Execution Order

1. Workstream 1: Local Alpha Full Release Gate Evidence
2. Workstream 2: Install and First-Run Configuration Experience
3. Workstream 5: Product-Grade Observability and Support Bundle Diagnostics
4. Workstream 3: Automatic Self-Revision Runtime Coverage
5. Workstream 4: Observe-Only Daemon Lifecycle Readiness
6. Workstream 8: Data Lifecycle, Backup, Restore, and Migration
7. Workstream 9: Provider Matrix and Real-Provider Readiness
8. Workstream 6: Structured Decision Protocol
9. Workstream 7: Evidence and Reflection Semantics V2
10. Workstream 10: Remote, Team Mode, Auth, and Security Gates
11. Workstream 11: Formal Release Engineering
12. Workstream 12: Multi-Layer Memory Product Direction

This order intentionally keeps user-operable Local Alpha gates ahead of broader autonomy and remote/team work.

## Subagent Split

Use independent subagents only when the write sets are separated:

| Subagent | Workstream | Write Scope |
| --- | --- | --- |
| Gate verifier | 1 | release-gate docs, progress tracker, generated evidence notes |
| Install/config worker | 2 | scripts, platform docs, config examples, bootstrap tests |
| Runtime coverage worker | 3 | MCP server tests, auto-reflection diagnostics docs |
| Daemon worker | 4 | daemon config, observe-only diagnostics, daemon gate docs |
| Observability worker | 5 | operation log, dashboard API, support bundle, redaction tests |
| Decision protocol worker | 6 | decision domain/application/provider parsing/tests |
| Evidence worker | 7 | evidence query, reflection contract, SQLite query tests |
| Data lifecycle worker | 8 | lifecycle docs and migration test fixtures |
| Provider worker | 9 | provider contract, config tests, adapter additions |
| Security planner | 10 | remote/team boundary and threat model docs |
| Release engineer | 11 | release engineering docs, changelog/release workflow |
| Memory planner | 12 | memory layering roadmap and future spec index |

Each implementation task should get a spec-compliance review first, then a code-quality review. Do not merge multiple workers into the same files at the same time unless one worker has finished and the coordinator has integrated the result.

## Definition of Done

A workstream is done only when:

- its files are updated
- its verification commands have been run fresh
- docs use conservative product wording
- no unrelated refactor was included
- `docs/progress-tracker.md` reflects the new state
- the result is committed locally

The overall formal-product track is not done until at least Phase 3 exits successfully according to `docs/superpowers/plans/2026-05-09-productization-roadmap.md`.
