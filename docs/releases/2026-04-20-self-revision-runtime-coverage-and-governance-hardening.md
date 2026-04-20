# Self-Revision Runtime Coverage And Governance Hardening

## Release Scope

This local update continues the automatic self-revision MVP hardening work without changing the repository's MVP / technical demo positioning.

The main outcomes are:

- the pre-existing suppression baseline gap is closed
- `proposed_evidence_query` now participates in bounded evidence governance instead of remaining a pure carried contract
- MCP-wired automatic runtime coverage expands from 3 explicit hooks to 4 explicit hooks, while staying opt-in, best-effort, and non-daemonized

## Highlights

### Improved evidence governance

- `proposed_evidence_query` can narrow evidence only inside the current trigger window
- when explicit evidence ids are present, they remain authoritative only if they still satisfy the query filter inside the current trigger window
- query limit is now applied inside the current trigger window on the empty-id path, instead of being driven by unrelated newer global matches
- no widening path or ranking engine was introduced

### Improved runtime coverage

Current `doctor` runtime coverage is now:

- `ingest_interaction:failure`
- `ingest_interaction:conflict`
- `decide_with_snapshot:conflict`
- `build_self_snapshot:periodic`

The new ingest conflict hook remains bounded:

- it requires explicit `trigger_hints` containing `conflict` or `identity`
- it is still best-effort
- it does not create a new durable write path

### Improved boundary protection

- repeated suppression / handled-baseline semantics remain covered by regression tests
- `decide_with_snapshot` still does not auto-reflect without explicit `auto_reflect_namespace`
- `build_self_snapshot` still does not auto-reflect without explicit `auto_reflect_namespace`
- `run_reflection` remains the only durable write path / persistence funnel

## Verification

Fresh verification for this local update:

- `cargo test` passed in full
- `./scripts/agent-llm-mm.sh doctor` returned `status = ok`
- total tests passing: `127`

## Important Notes

- This is still a local MCP `stdio` memory demo, not a full autonomous agent system.
- `proposed_evidence_query` is now governed more strictly, but it is still not a richer evidence weighting / relation / ranking engine.
- Automatic self-revision runtime coverage is larger than before, but it is still limited to explicit hooks and not all MCP entry points.
