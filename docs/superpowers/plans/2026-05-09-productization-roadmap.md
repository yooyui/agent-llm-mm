# Productization Roadmap After MVP

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this roadmap task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `agent_llm_mm` from a validated local MCP `stdio` MVP into a formal product track without overstating current production readiness.

**Architecture:** Keep the current local-first runtime as the product core, then add product packaging, data isolation, durable observability, daemon orchestration, guarded admin surfaces, security controls, and release operations in gated phases. `run_reflection` remains the only durable identity/commitment write path until a later architecture decision explicitly replaces it.

**Tech Stack:** Rust, Tokio, RMCP `stdio`, SQLite, TOML configuration, local dashboard HTTP surface, operation log store, future local daemon, provider adapters, Markdown release/runbook docs, integration tests, deterministic demo artifacts.

---

## Executive Decision

The MVP phase is OK for the current scope: the local release gate has passed, `cargo test` is green at 170 tests, `doctor` reports `status = ok`, and the self-revision demo package can generate the required evidence artifacts.

That does not mean the repository is already a formal product. It means the validated MVP is now strong enough to become the base for a productization track.

Productization should start as a **local-first formal product**:

- First product promise: auditable self-agent memory for local AI clients.
- First distribution shape: installable local service plus documented client integration.
- First admin surface: local read-only dashboard, then guarded local write/admin tools only after auth and audit are ready.
- First autonomy shape: explicit MCP hooks plus optional local daemon in observe-only mode before any broader automatic behavior.

## Product Positioning

### Target User

Primary early users:

- developers and AI-agent operators who run Codex-like local clients
- teams that want persistent agent memory, auditability, and controlled self-revision
- maintainers who need predictable local setup before trusting autonomous behavior

Secondary future users:

- small teams that want a shared memory service
- product teams that need remote admin, policy control, and multi-user isolation

### Product Promise

The product should promise:

- persistent local memory with clear namespace boundaries
- reproducible self-snapshot and decision evidence
- governed self-revision with audit trail
- provider-agnostic model access through explicit configuration
- safe local observability and release gates

It should not promise full autonomous self-governance until daemon, policy, audit, auth, rollback, and long-run verification are complete.

## Productization Approaches

| Approach | What It Optimizes | Risk | Recommendation |
| --- | --- | --- | --- |
| Local-first productization | Turns the working MVP into an installable, supportable local product | Slower to reach remote/team features | Recommended |
| Remote service first | Makes the product easier to demonstrate to teams | Requires auth, tenancy, audit, transport, and deployment before the core is proven | Defer |
| Autonomy first | Chases the original self-governing agent vision fastest | High risk of unsafe writes, unclear rollback, and hard-to-debug behavior | Defer until local product is stable |

Decision: start with local-first productization, then expand to single-tenant remote/admin, then evaluate multi-tenant or broader autonomy.

## Product Phases

### Phase 0: Product Definition and Gate Reset

**Goal:** Replace MVP framing with a formal product track while preserving accurate current-state claims.

**Deliverables:**

- [ ] Product requirements document for the first formal release
- [ ] Product naming and packaging decision
- [ ] Updated roadmap and progress tracker
- [ ] Product release gate separate from MVP release gate
- [ ] Clear public wording: "validated local MVP entering productization", not "production-ready autonomous agent"

**Exit gate:**

- Product release gate exists as a separate document.
- README and overview docs link to the productization roadmap.
- No public doc claims remote admin, multi-tenancy, background autonomy, or production self-governance is already implemented.

### Phase 1: Local Product Alpha

**Goal:** Make the current local service installable, configurable, inspectable, and supportable by a real user on one machine.

**Deliverables:**

- [ ] One-command local install or bootstrap path for macOS
- [ ] Windows bootstrap parity check
- [ ] First-run config wizard or guided config template
- [ ] Stable config profile layout for `dev`, `demo`, `prod-local`
- [ ] Explicit database separation for formal data, test data, and demo data
- [ ] Local backup / restore / export runbook for SQLite
- [ ] Dashboard startup profile documented as local-only and read-only by default
- [ ] Provider readiness flow for at least one real provider beyond mock

**Exit gate:**

- Fresh machine setup succeeds using only documented commands.
- `doctor` explains config, provider, dashboard, daemon, database, and runtime-hook status without exposing secrets.
- `cargo test`, product smoke tests, and self-revision demo artifacts pass from the installed path.

### Phase 2: Durable Observability and Local Daemon Alpha

**Goal:** Add enough operational evidence to run the product continuously on one local machine without guessing what happened.

**Deliverables:**

- [ ] Durable operation log wired into runtime start/completion/error events
- [ ] Dashboard reads durable history in addition to live bounded events
- [ ] Correlation id across MCP calls, model calls, operation log, trigger ledger, and reflection audit
- [ ] Local daemon implemented behind `[daemon].enabled = true`
- [ ] Daemon starts in observe-only mode
- [ ] Daemon trigger policy documented and test-covered
- [ ] Safe shutdown and orphaned-work recovery behavior

**Exit gate:**

- Operation log survives restart and supports namespace/time/correlation queries.
- Daemon defaults to disabled.
- Observe-only daemon can run without changing identity or commitments.
- Any daemon-triggered write path still goes through governed `run_reflection`.

### Phase 3: Product Beta

**Goal:** Make the product reliable enough for a controlled team trial while still avoiding broad production claims.

**Deliverables:**

- [ ] Release artifact packaging and versioned changelog
- [ ] Product-level smoke test script
- [ ] Error taxonomy and support bundle export
- [ ] Provider matrix with config, timeout, redaction, malformed-response, and retry behavior
- [ ] Policy config for automatic self-revision thresholds
- [ ] Data retention policy for events, claims, reflections, operation log, and demo artifacts
- [ ] Upgrade / migration tests from prior local databases

**Exit gate:**

- A beta user can install, configure, run, inspect, backup, and upgrade the product using docs.
- Support bundle excludes secrets and contains enough diagnostics for triage.
- Long-run local soak test is documented and has a repeatable command.

### Phase 4: Remote Admin and Team Mode

**Goal:** Add remote/team capabilities only after audit, auth, and local reliability are stable.

**Deliverables:**

- [ ] Remote admin boundary spec
- [ ] Authentication and authorization design
- [ ] Read-only remote dashboard first
- [ ] Write/admin operations require auth, audit log, and explicit policy
- [ ] Namespace/database isolation model for team usage
- [ ] Transport decision: keep MCP `stdio` core, add HTTP only for admin/API surfaces that need it
- [ ] Backup, restore, and migration procedure for team data

**Exit gate:**

- Remote read-only admin can be tested without write capability.
- Every write-capable admin action has auth, authorization, audit, rollback, and tests.
- Multi-user or multi-tenant claims are blocked until isolation tests exist.

### Phase 5: GA Readiness

**Goal:** Decide whether the product is ready for public "formal product" claims.

**Deliverables:**

- [ ] GA release gate
- [ ] Security review checklist
- [ ] Threat model for local and remote surfaces
- [ ] Disaster recovery runbook
- [ ] Compatibility matrix for OS, Rust toolchain, SQLite, and supported providers
- [ ] Public docs rewritten around product use, not demo explanation
- [ ] SemVer and deprecation policy

**Exit gate:**

- Product can be installed and upgraded by a user who did not build it.
- Product can be operated for a defined period with repeatable diagnostics.
- Security, data isolation, backup, and recovery have been tested.
- Public docs accurately separate local product, remote beta, and future autonomy.

## Workstreams

| Workstream | Owner Type | Near-Term Output | Product Risk Reduced |
| --- | --- | --- | --- |
| Product requirements | planner | PRD and release gate | Prevents vague "formal product" scope |
| Packaging and setup | worker | install/bootstrap scripts | Reduces setup friction |
| Config and secrets | worker + reviewer | config profiles, redaction tests | Prevents secret leakage and environment drift |
| Data lifecycle | worker | backup/restore/migration docs and tests | Prevents data loss |
| Observability | worker | durable operation log integration | Makes runtime behavior debuggable |
| Daemon policy | planner + worker | observe-only daemon plan and implementation | Controls autonomy risk |
| Dashboard/admin | frontend/backend worker | local read-only product dashboard | Gives users inspection surface |
| Provider ecosystem | worker | provider readiness matrix | Makes real model use supportable |
| Security | reviewer | threat model and auth plan | Blocks unsafe remote/product claims |
| Release operations | reviewer + worker | CI/release runbook | Makes releases repeatable |

## First Implementation Slice

The first productization slice should be narrow and shippable:

### Task 1: Product PRD and Release Gate

- [ ] Create `docs/product/prd-local-alpha.md`
- [ ] Create `docs/product/release-gate-local-alpha.md`
- [ ] Define local alpha scope, non-goals, install target, support target, and exit gate
- [ ] Link both docs from README, document map, and roadmap

### Task 2: Installable Local Alpha

- [ ] Audit `scripts/agent-llm-mm.sh` for product install assumptions
- [ ] Add a documented local install/bootstrap command
- [ ] Add config profile examples for `dev`, `demo`, and `prod-local`
- [ ] Add smoke test command that runs `doctor`, MCP stdio sanity, and dashboard health when enabled

### Task 3: Data Safety Pack

- [ ] Document database profile separation
- [ ] Add backup and restore commands for SQLite
- [ ] Add migration verification checklist
- [ ] Add support bundle export design, with secret redaction rules

### Task 4: Durable Observability

- [ ] Decide whether operation log is the dashboard's durable history source in local alpha
- [ ] Add correlation id conventions to docs and tests
- [ ] Extend dashboard projection to make durable events inspectable without write routes
- [ ] Add regression tests for no secret leakage in operation summaries

### Task 5: Observe-Only Daemon

- [ ] Implement daemon start/stop lifecycle only after Task 4 is stable
- [ ] Keep daemon disabled by default
- [ ] Start with observe-only mode
- [ ] Add a separate gate before daemon is allowed to call governed self-revision

## Product Safety Rules

- Do not expose write-capable remote admin until auth, authorization, audit, and rollback exist.
- Do not widen automatic self-revision to all entrypoints until operation log, daemon policy, trigger ledger, and rejection/suppression diagnostics are stable.
- Do not change durable identity/commitment writes away from `run_reflection` without an architecture decision record and migration plan.
- Do not claim multi-tenancy until namespace, database, dashboard, operation-log, and config isolation are all tested.
- Do not mix demo data with formal user data; product docs must always recommend separate `database_url` values.

## Definition of "Formal Product"

This repository can be called a formal product only when at least Phase 3 exits successfully.

Before that:

- after Phase 0: productization planned
- after Phase 1: local product alpha
- after Phase 2: local product alpha with durable observability and observe-only daemon
- after Phase 3: controlled product beta
- after Phase 5: GA candidate

## Immediate Next Step

Create a local-alpha PRD and a local-alpha release gate. That keeps the next implementation round concrete and prevents the productization effort from expanding into remote admin, multi-tenancy, or full autonomy before the local product is stable.
