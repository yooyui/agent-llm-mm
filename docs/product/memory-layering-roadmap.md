# Memory Layering Roadmap

## Purpose

This roadmap defines the future multi-layer memory direction for
`agent_llm_mm`. It is a product capability roadmap, not an implementation status
claim.

Local Alpha remains a local MCP memory service plus governed self-revision. It
is not a complete multi-layer cognitive architecture, not a production
self-governing system, and not a remote/team memory product.

## Product Wording Boundary

Use conservative stage wording until the matching gates pass:

| Stage | Allowed Wording | Blocked Wording |
| --- | --- | --- |
| MVP | validated local MVP entering productization | formal product, production-ready, complete autonomy |
| Local Alpha | local MCP memory plus governed self-revision | complete multi-layer cognitive architecture |
| Beta | controlled beta after lifecycle, migration, and support gates | GA, production self-governance |
| Remote/team | remote or team capability only after auth, audit, isolation, and backup gates | multi-tenant production service without isolation tests |
| GA | GA only after security, migration, lifecycle, support, and recovery gates pass | GA by roadmap intent alone |

## Prerequisites

The multi-layer memory roadmap must not begin runtime implementation until these
foundation contracts are stable and tested:

- evidence semantics stable: evidence ids, evidence queries, projections, and
  redaction rules have stable meanings.
- reflection policy stable: `run_reflection` governance, rejection,
  suppression, cooldown, and diagnostics are stable enough to support richer
  memory writes.
- schema migration policy tested: local SQLite migrations, rollback
  expectations, and compatibility checks are tested against prior local
  databases.
- data lifecycle/backup gates in place: backup, restore, export, retention, and
  deletion rules exist before new memory layers increase data value and risk.
- product wording remains staged: docs keep MVP, Local Alpha, Beta,
  remote/team, and GA claims separate.

## Layer Definitions

| Layer | Future Role | Local Alpha Status | First Gate |
| --- | --- | --- | --- |
| Working memory | Short-lived task context, active goals, temporary constraints, and current evidence handles | Not implemented as a distinct layer | Define expiry, visibility, and no-durable-commit rules |
| Episodic memory | Structured records of tasks, outcomes, lessons, and linked evidence | Current events/reflections are partial inputs only | Add richer episode semantics behind tests |
| Semantic memory | Stable distilled concepts, project facts, domain rules, and cross-episode summaries | Not implemented as a distinct layer | Prove evidence-backed extraction and contradiction handling |
| Procedural memory | Reusable workflows, policies, checklists, and operational playbooks | Current docs and scripts are human-facing, not a runtime layer | Define approval, versioning, and rollback rules |
| Slow variables | Long-horizon preferences, calibrated thresholds, trust levels, and policy weights | Not implemented | Define governance, review cadence, and bounded update paths |
| Self-model layering | Explicit model of agent capabilities, limits, commitments, and known failure modes | Current self-revision diagnostics are partial evidence only | Define read-only projection before any durable self-model writes |

## Phased Direction

### Phase 0: Contract Stabilization

Goal: keep Local Alpha honest while preparing the data contracts.

- Stabilize evidence semantics and projection names.
- Keep `run_reflection` as the durable identity and commitment write path.
- Keep automatic behavior local-first, opt-in where applicable, and governed.
- Finish schema migration, backup, restore, and retention gates before adding
  new durable memory tables.

Exit: current memory, evidence, reflection, and lifecycle contracts can be
tested without relying on demo-only assumptions.

### Phase 1: Richer Episodic Semantics

Goal: turn task history into structured episodes without introducing a full
semantic or self-model layer.

The first implementable slice is `richer episodes semantics`.

Minimum episode fields:

- `goal`: what the task attempted to accomplish.
- `outcome`: what actually happened, including success, partial success,
  refusal, or failure.
- `lesson`: a bounded, reusable takeaway that does not exceed the evidence.
- `linked_evidence_ids`: evidence ids that support the episode.
- `snapshot_projection_test`: a deterministic test proving the episode appears
  in the expected snapshot projection without corrupting existing projections.

Acceptance expectations:

- An episode can be written and read without changing identity or commitments.
- Episode summaries preserve linked evidence ids rather than copying raw
  evidence bodies into every projection.
- The snapshot projection test proves compatibility with the existing local MCP
  memory flow.
- The feature remains local-only and does not imply semantic memory,
  procedural memory, slow variables, or self-model writes are implemented.

Exit: richer episode records are test-covered, migration-covered, and projected
read-only before any later layer consumes them.

### Phase 2: Semantic Memory Candidate

Goal: introduce evidence-backed, stable distilled facts only after richer
episodes are reliable.

- Extract semantic candidates from multiple episodes.
- Require source episode and evidence links.
- Track contradictions and supersession instead of overwriting facts silently.
- Keep semantic memory read-only in projections until governance rules exist.

Exit: semantic candidates can be generated, inspected, rejected, and migrated
without replacing episodic records.

### Phase 3: Procedural Memory Candidate

Goal: make reusable workflows explicit without silently changing runtime
behavior.

- Model procedures as versioned, inspectable records.
- Link procedures to evidence and human-facing docs where applicable.
- Require explicit activation before a procedure affects runtime behavior.
- Track rollback and deprecation metadata.

Exit: procedures are searchable and auditable, but runtime use remains gated.

### Phase 4: Slow Variables and Policy Calibration

Goal: represent long-horizon preferences and policy weights with strict
governance.

- Define allowed slow-variable names, types, bounds, and review cadence.
- Require evidence links and policy approval for updates.
- Keep updates reversible and visible in audit projections.
- Prevent one task from causing broad unreviewed preference drift.

Exit: slow variables are durable only when bounded, reviewed, and recoverable.

### Phase 5: Self-Model Layering

Goal: expose a layered self-model only after evidence, reflection, episodes,
semantic memory, procedural memory, and slow variables have stable gates.

- Start with read-only self-model projections.
- Separate capabilities, limitations, commitments, and known failure modes.
- Preserve `run_reflection` as the durable commitment path unless a later
  architecture decision replaces it with migration and rollback support.
- Block claims of production self-governance until long-run, security,
  lifecycle, and recovery gates pass.

Exit: the self-model is inspectable, evidence-linked, and governed before any
broader autonomy claim is made.

## Non-Goals

- This roadmap does not implement multi-layer memory.
- This roadmap does not replace `run_reflection`.
- This roadmap does not add remote/team memory service behavior.
- This roadmap does not claim Beta, GA, production self-governance, or complete
  autonomous agent behavior.
- This roadmap does not allow Local Alpha wording to imply a complete
  multi-layer cognitive architecture.

## Verification Hooks for Future Work

Future implementation specs should include at least:

- migration tests from existing local SQLite databases.
- snapshot projection tests for each new layer.
- evidence-link integrity tests.
- redaction and support-bundle tests.
- backup/restore compatibility checks.
- product wording checks that block overstated Local Alpha, Beta, remote/team,
  or GA claims.
