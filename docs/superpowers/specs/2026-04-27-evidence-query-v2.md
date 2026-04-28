# Evidence Query v2 Contract

Status: MVP / technical demo design contract, not implemented runtime behavior

This document specifies the next narrow evidence-query shape for the local MCP `stdio` technical demo / MVP. It does not introduce model-based evidence ranking, relation traversal, autonomous search, or a new durable write path.

`run_reflection` remains the only durable write path for identity and commitment updates.

## Current Baseline

The current v1 query shape is intentionally small:

- `owner`
- `kind`
- `limit`

It is used by:

- `replacement_evidence_query` in explicit `run_reflection`
- `proposed_evidence_query` in governed automatic self-revision proposals

The current SQLite `events` table stores `event_id`, `recorded_at`, `owner`, `kind`, and `summary`. It does not yet store an event namespace. Claims and trigger-ledger entries already have namespace concepts, but evidence-event queries must not infer event namespace from claim namespace, summaries, or owner alone.

The current store methods are:

- bounded `query_evidence_event_ids`, with a default limit of `10` when the caller omits `limit`
- unbounded `query_evidence_event_ids_unbounded`, used only where the caller must intersect results with an already governed candidate window

Current ordering is deterministic: `recorded_at DESC, rowid DESC`.

Current query results are plain event ids. Richer evidence semantics, relation strength, weighting, and ranking do not exist today.

## V2 Scope

Evidence Query v2 may add only these fields and semantics:

- explicit namespace filter
- evidence kind filter, mapped to the existing event kind taxonomy unless a later schema migration creates a separate evidence-kind column
- bounded recency window
- deterministic limit behavior
- clear no-match behavior

The first implementation slice should be namespace-aware narrowing. For automatic self-revision proposals, that narrowing happens inside the existing governed trigger window. For explicit `run_reflection`, there is no trigger window, so namespace-aware lookup must be a direct store filter and an empty result must stay `invalid_params`.

The goal is a clearer lookup and governance contract shared by explicit reflection evidence lookup and automatic self-revision proposal validation. It is not a reasoning engine.

## Deferred From V2

The following are deliberately out of scope for v2:

- model-based ranking
- relation graph traversal
- weight scoring
- cross-namespace widening
- autonomous evidence search outside the governed trigger window
- background daemon behavior
- a new MCP tool for evidence search
- a durable self-revision write path outside `run_reflection`

Those topics belong to later evidence and productization work after the MVP boundary is stable.

## Proposed Query Shape

The v2 application shape should be an extension of the current `EvidenceQuery`, not a parallel search subsystem:

```text
EvidenceQueryV2 {
  namespace: Option<Namespace>,
  owner: Option<Owner>,
  kind: Option<EventKind>,
  recorded_after: Option<DateTime<Utc>>,
  recorded_before: Option<DateTime<Utc>>,
  limit: Option<usize>
}
```

Implementation may use the current `EvidenceQuery` name if the change is source-compatible enough, but the behavior must remain explicit and test-covered.

Field semantics:

- `namespace`: narrows to evidence events explicitly stored in that namespace. It must not be derived from `owner` alone.
- `owner`: narrows by event owner, as today.
- `kind`: narrows by event kind, as today. If a future schema separates evidence kind from event kind, that future change needs its own spec.
- `recorded_after` / `recorded_before`: bound a recency window. The window is inclusive and is applied before `limit`.
- `limit`: is applied after all filters and after deterministic ordering.

## Ordering And Limit Behavior

The deterministic ordering rule is:

1. newest `recorded_at` first
2. stable database insertion order as the tie breaker, matching the current SQLite `rowid DESC` behavior

`limit` must be applied after namespace, owner, kind, and recency filters. A limit must never be interpreted as permission to widen the search beyond the filtered candidate set.

Overflowing or otherwise unsupported limits must remain `invalid_params`.

Duplicate event ids must be deduplicated before the result is persisted or used as governed reflection evidence. Deduplication must preserve the first occurrence after deterministic ordering or after explicit-id order, depending on caller context.

## No-Match Behavior

No-match behavior depends on caller context.

For explicit `run_reflection` replacement evidence queries:

- an empty query result with no explicit evidence ids remains `invalid_params`
- explicit evidence ids remain the authoritative input only after server-side existence validation
- a query must not silently widen to unrelated evidence

For automatic self-revision proposal queries:

- `proposed_evidence_query` narrows within the current governed trigger window
- if the narrowed query has no matches and the proposal did not provide explicit ids, v2 must treat that as "no narrowing result" and must not reset the query to a broader search result
- the current v1 fallback to the full trigger-window evidence set is legacy behavior to retire for the namespace-aware v2 slice, not a compliant v2 fallback
- no no-match behavior may query outside the governed trigger window
- if explicit ids are present, they must be inside the current trigger window
- if explicit ids and a query are both present, explicit ids must also satisfy the server-side query validation

This preserves the safety property: a model proposal can narrow evidence but cannot widen evidence beyond the trigger window or silently bypass its own query.

## Self-Revision Proposal Interaction

`SelfRevisionProposal.proposed_evidence_query` is a governance hint, not an autonomous search command.

Rules:

- the server owns candidate-window construction
- the model may propose `proposed_evidence_event_ids`
- the model may propose `proposed_evidence_query`
- explicit ids are accepted only if every id satisfies server-side validation
- a proposed query is intersected with the current trigger window
- an empty query result never authorizes cross-namespace widening or a broader replacement result
- rejected proposed ids must leave no durable reflection write

Automatic self-revision still writes durable identity and commitment changes only through `run_reflection`.

## Namespace Migration Boundary

Because current events do not persist namespace, explicit `run_reflection` namespace filters require real event namespace storage before they can be supported. Acceptable approaches include:

- adding namespace to the event domain object and SQLite `events` table with an explicit migration

Automatic self-revision proposal validation may also derive a candidate namespace from already validated runtime trigger-window context, but only for narrowing the governed in-memory candidate set. That shortcut does not support explicit `run_reflection`, because explicit `run_reflection` queries the store directly and has no trigger window.

Unacceptable approaches:

- parsing namespace from event summaries
- treating `owner` as a namespace substitute
- using claim namespace through `evidence_links` as the event namespace
- widening to all events when namespace metadata is missing

Legacy events must get a conservative documented namespace behavior before namespace filtering is enabled. If legacy behavior cannot be made precise in one slice, namespace-aware filtering should reject unsupported legacy rows rather than silently widening.

## Verification Requirements

Current v1 governance behavior is already covered by:

- `auto_reflection_intersects_proposed_evidence_query_with_current_trigger_window_when_ids_are_empty`
- `auto_reflection_ignores_proposed_evidence_query_for_widening_when_ids_are_empty`
- `auto_reflection_applies_query_limit_within_current_trigger_window_when_ids_are_empty`
- `auto_reflection_rejects_model_proposed_evidence_ids_that_do_not_match_query_policy`
- `auto_reflection_rejects_model_proposed_evidence_outside_trigger_window`
- `reflection_rejects_identity_update_when_evidence_query_resolves_empty`

Related docs:

- `docs/superpowers/specs/2026-04-27-reflection-deeper-update-contract.md`
- `docs/local-mcp-integration-2026-03-26.md`
- `docs/testing-guide-2026-03-24.md`

The first v2 implementation slice should add tests for:

- namespace filter returns only events in the requested namespace
- namespace filter preserves newest-first deterministic order
- `limit` is applied inside the namespace-filtered candidate set
- explicit `run_reflection` namespace or recency no-match behavior remains `invalid_params`
- self-revision proposals cannot use namespace filters to widen beyond the trigger window
- no-match namespace filters do not silently bypass the proposed filter

Recommended command set for that slice:

```bash
cargo test --test sqlite_store -v
cargo test --test failure_modes -v
cargo test --test mcp_stdio -v
cargo test
```

Sandbox-only listener or database-path failures should be recorded separately from code failures, following `docs/release-gate.md`.
