# P1 P2 P3 Product Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the remaining P1, P2, and P3 product-readiness work as real code-backed slices, while keeping simulated, planning-only, Windows, remote, and human-decision gaps blocked until fresh evidence exists.

**Architecture:** Add a product-readiness gate checker first so no later slice can mark a simulated or planning-only item complete. Then land small, testable runtime slices: status/plan synchronization, release decision artifacts, decision protocol v2 metadata, richer episode/evidence semantics, provider matrix contract hardening, remote/team read-only gates, and the first multi-layer memory projection. External-environment requirements such as Windows runner parity, true fresh-machine install, real remote auth deployment, and human release decisions remain explicit blockers unless the current branch contains fresh evidence.

**Tech Stack:** Rust, Tokio, SQLite, TOML, JSON/Markdown evidence artifacts, shell wrappers, existing `doctor`, `status-sync-check`, `local-alpha-evidence-summary`, and integration tests.

---

## Completion Rules

- Code, tests, docs, and fresh evidence must line up before any item moves to `implemented`.
- Local simulation remains `simulation-only` unless a real environment summary says otherwise.
- `run_reflection` remains the only durable identity / commitment / reflection write path.
- Remote/team and daemon write behavior stay blocked until separate auth, audit, rollback, and security gates pass.
- The first pass should prefer narrow, shippable product foundations over broad speculative architecture.

## P1 Execution Slices

- [x] **P1.1 Product readiness gate checker**
  - Add a binary and script that read local evidence summaries and release-soak directories.
  - Output JSON/Markdown with `ready = false` while real fresh-machine, Windows parity, release decision, installer/package, remote/team, or GA evidence is missing.
  - Tests must prove simulation evidence is a blocker, not a pass.

- [x] **P1.2 Release decision artifact**
  - Add a local source-only release decision template/generator.
  - Require candidate name, evidence directory, open gates, human decision field, rollback note, and non-claims.
  - Tests must reject approving a candidate when `local-alpha-evidence-summary.json` is `in_progress`.

- [x] **P1.3 Plan/status synchronization v2**
  - Extend `status-sync-check` beyond cargo-test totals.
  - Detect stale completed workstream checkboxes when the reality gate still says `partial`, `simulation-only`, `planning-gate`, or `not-implemented`.
  - Tests must cover plan/status contradiction reporting without changing files.

- [x] **P1.4 Support bundle and daemon lifecycle hardening**
  - Keep support bundle local-only and redacted.
  - Add manifest integrity fields for generated evidence where useful.
  - Keep daemon lifecycle observe-only; add diagnostics for why writes are blocked.

## P2 Execution Slices

- [x] **P2.1 Decision protocol v2**
  - Extend decision result metadata without breaking `blocked` and `decision` legacy fields.
  - Add `protocol_version = 2`, `decision_id`, `requested_action`, `selected_action`, `confidence`, `policy_checks`, and `non_claims`.
  - Tests must prove old callers can still read `blocked` and `decision.action`.

- [x] **P2.2 Evidence semantics v2 first slice**
  - Add an explicit evidence relation/read model over existing events and evidence links.
  - Keep no-widening governance in auto-reflection.
  - Tests must prove relation/ranking metadata is bounded and does not pull evidence outside the trigger window.

- [x] **P2.3 Richer episode semantics first slice**
  - Add structured episode summary projection backed by existing `episode_events`.
  - Include objective/outcome/linked evidence ids only as local metadata.
  - Tests must prove episode projection does not modify identity or commitments.

- [x] **P2.4 Provider readiness hardening**
  - Keep `azure-openai`, `openrouter`, and `local` as not configurable until real adapters exist.
  - Add adapter-specific readiness rows and tests for missing implementation.
  - Only mark a provider supported in the same change that adds config, parser, doctor, adapter, error handling, and MCP stdio tests.

## P3 Execution Slices

- [x] **P3.1 Remote/team read-only inventory**
  - Add a machine-readable remote/team capability inventory that reports every remote/team feature as blocked unless auth/audit/isolation gates exist.
  - Tests must prove no write-capable remote route or support-bundle upload is exposed.

- [x] **P3.2 Security and auth gate contracts**
  - Add local gate checks for auth, authorization, audit, rate limit, tenant isolation, and rollback prerequisites.
  - Keep this as contract plus tests until implementation starts.

- [x] **P3.3 Multi-layer memory first runtime slice**
  - Add a read-only layered memory projection over existing snapshot and episode data.
  - Label working/episodic/semantic/procedural/self-model layers as `implemented`, `partial`, or `not_implemented`.
  - Tests must prove this is a projection, not new durable self-model writes.

- [x] **P3.4 Product wording guard**
  - Add checks that block Beta, GA, production-ready, remote write admin, remote team service, and complete self-governance claims unless corresponding gates are satisfied.
  - Wire the check into status sync or product readiness gate output.

## Verification Baseline

Run after every completed slice:

```bash
cargo fmt --check
git diff --check
bash -n scripts/status-sync-check.sh
cargo test --test status_sync -v
cargo test
./scripts/agent-llm-mm.sh doctor
```

Run additionally when release evidence changes:

```bash
bash -n scripts/local-alpha-evidence-summary.sh scripts/local-alpha-release-gate-refresh.sh scripts/release-soak-local.sh
cargo test --test local_alpha_release_evidence -v
./scripts/local-alpha-evidence-summary.sh --evidence-root .
```
