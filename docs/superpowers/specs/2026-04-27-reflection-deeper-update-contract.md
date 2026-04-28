# Reflection Deeper-Update Contract

Status: MVP / technical demo contract

This document specifies the current `run_reflection` deeper-update boundary as covered by the verification pointers below. It does not introduce a richer identity schema, a versioned policy engine, or autonomous slow-variable formation.

## Current Supported Updates

`run_reflection` is the only durable write path for reflection-driven updates to identity and commitments.

The current supported reflection shapes are:

- MCP `run_reflection` conflict reflection that targets an existing claim, marks it as disputed, and may also apply evidence-backed identity and commitment updates.
- MCP `run_reflection` failure reflection that targets an existing claim, supersedes it with a replacement claim, and may also apply evidence-backed identity and commitment updates.
- Application-layer record-only reflection with optional evidence-backed identity and commitment updates. This shape is not currently exposed as the shipped MCP `run_reflection` tool surface.
- Replacement evidence supplied either by explicit `replacement_evidence_event_ids`, by the narrow `replacement_evidence_query`, or by both. When both are present, resolved evidence is deduplicated.

The current deeper-update fields are:

- `identity_update.canonical_claims`
- `commitment_updates`

These fields are intentionally minimal. They represent the current identity core and commitment list, not a richer slow-variable model.

## Required Supporting Evidence

Supporting evidence is always required when a reflection includes any of these deeper writes:

- `identity_update`
- `commitment_updates`

For identity or commitment updates, at least one evidence event id must resolve successfully before persistence. Missing evidence for those updates is rejected as invalid parameters.

Replacement claims follow claim validation rules. Inferred replacement claims require supporting evidence; observed replacement claims are not hard-rejected solely because no evidence id was supplied.

Evidence may come from:

- explicit `replacement_evidence_event_ids`
- bounded `replacement_evidence_query`
- both sources together, with duplicate ids removed

Every resolved evidence event id must exist before the reflection transaction writes claims, identity, commitments, or audit records.

The current `replacement_evidence_query` is a narrow evidence lookup, not a ranking engine. It is limited to the current supported query shape and should not be documented as richer evidence weighting, relation traversal, or autonomous discovery.

## Disallowed Direct Identity Writes

Direct identity writes outside `run_reflection` are not part of the current contract.

The ingest MCP schema does not expose an identity-core write field. Reflection may update `identity_core` only when the update is explicit, evidence-backed, and routed through `run_reflection`.

An identity update with an empty `canonical_claims` list is rejected. The contract requires at least one canonical claim for an identity update.

Automatic self-revision does not get a separate identity-write path. When automatic self-revision produces a governed machine patch, the durable write still goes through `run_reflection`.

## Commitment Replacement Behavior

`commitment_updates` replaces the current commitment set only through `run_reflection`.

The current implementation preserves the baseline hard guard:

- `forbid:write_identity_core_directly`

If a requested commitment replacement omits that baseline commitment, the persistence path adds it back before writing the new commitment list.

This is a narrow baseline-preservation rule. It is not a complete commitment lifecycle system. The current contract does not implement commitment versioning, expiration, priority, or richer policy evaluation.

## Audit Record Expectations

Every accepted reflection appends a reflection audit record.

For deeper updates, the audit record is expected to preserve:

- `supporting_evidence_event_ids`
- `requested_identity_update`
- `requested_commitment_updates`

For replacement-claim reflections, the audit record also links the target claim and replacement claim ids through the existing reflection fields.

If a reflection is part of a handled automatic self-revision path, the trigger ledger entry is appended through the same reflection transaction after the reflection id is known. That does not create a second durable write path.

## Intentionally Not Implemented

The current contract intentionally does not implement:

- richer identity schema
- versioned slow-variable policy engine
- autonomous identity formation
- commitment lifecycle states beyond the current replacement list
- richer evidence weighting, relation traversal, or ranking
- background daemon behavior
- a new MCP tool for automatic self-revision writes
- any durable identity or commitment write path outside `run_reflection`

These are future roadmap topics. They should not be inferred from the current MVP deeper-update support.

## Verification Pointers

Current behavior is covered by focused application and MCP tests:

- `reflection_can_record_identity_and_commitment_updates_without_claim_transition`
- `reflection_without_replacement_claim_disputes_old_claim_and_updates_identity`
- `reflection_can_update_identity_and_commitments_with_audited_supporting_evidence`
- `reflection_preserves_baseline_commitment_when_updates_replace_commitments`
- `reflection_identity_or_commitment_updates_require_evidence_over_stdio`
- `reflection_identity_and_commitment_updates_are_applied_and_audited_over_stdio`

Recommended local checks:

```bash
cargo test --test application_use_cases reflection_can_record_identity_and_commitment_updates_without_claim_transition -v
cargo test --test application_use_cases reflection_without_replacement_claim_disputes_old_claim_and_updates_identity -v
cargo test --test application_use_cases reflection_can_update_identity_and_commitments_with_audited_supporting_evidence -v
cargo test --test application_use_cases reflection_preserves_baseline_commitment_when_updates_replace_commitments -v
cargo test --test mcp_stdio reflection_identity_or_commitment_updates_require_evidence_over_stdio -v
cargo test --test mcp_stdio reflection_identity_and_commitment_updates_are_applied_and_audited_over_stdio -v
```
