# MCP Memory Ledger

Local-first MCP memory for AI agents, backed by SQLite and evidence-gated self-revision.

Languages: English | [Simplified Chinese](docs/README.zh-CN.md) | [Japanese](docs/README.ja.md)

MCP Memory Ledger is a Rust MCP `stdio` memory service for local AI clients. It records interactions, evidence, claims, self snapshots, and reflection audits in SQLite so an agent can use durable memory inside explicit, inspectable boundaries instead of relying only on a single prompt context.

The project grew out of a discussion about information loss, durable memory, and how selected parts of the past can safely constrain future agent behavior. The [origin and mainline principles](docs/origin-and-principles.md) are now the entry point for deciding what belongs in this repository.

The current project is best understood as a technical MVP for local agent memory, MCP integration, SQLite persistence, and governed self-revision. It is not a production autonomous-agent platform. Remote team mode, multi-tenancy, packaged installers, daemon write capabilities, and production security boundaries remain gated roadmap work.

## Features

- **Local MCP memory service**: exposes `ingest_interaction`, `search_memory`, `get_memory`, `build_self_snapshot`, `decide_with_snapshot`, and `run_reflection` over MCP `stdio`.
- **Scoped event and claim recall**: `search_memory` requires an explicit namespace. It defaults to bounded recent-first event records, while additive `record_type = Claim` queries return scoped claims filtered by canonical/raw claim reference, status, and mode. Both deterministic read paths are provider-free and fail closed across namespaces.
- **Scoped stable-ID lookup**: `get_memory(namespace, id, record_type?)` returns one complete Event or Claim record. Omitting `record_type` preserves Event behavior; Claim lookup requires `record_type = Claim`. Both types accept canonical or raw IDs, and missing/cross-namespace IDs return `record: null` without widening.
- **SQLite persistence**: stores events, claims, evidence, reflection audits, trigger ledger entries, and operation logs.
- **Evidence-gated self-revision**: claim, identity, and commitment updates must be backed by explicit evidence and governance rules. `run_reflection` remains the only durable write path for identity, commitment, and reflection changes.
- **Bounded scoped snapshots**: the M0.2 path accepts an explicit namespace, optional evidence manifest, and inclusive time window; it applies owner/namespace filtering and stable recent-first ordering in SQLite, and feeds automatic reflection only from the frozen trigger scope and evidence window.
- **Bounded local operations**: includes operation-log lookup, backup / restore helpers, redacted diagnostics, explicit `init` / `migrate`, and a no-write `doctor --read-only` default. `serve` refuses missing or stale databases instead of changing them implicitly.
- **Local runtime safety gate**: an enabled unauthenticated dashboard accepts only localhost/loopback hosts, while CLI tracing is initialized on stderr so MCP/JSON stdout stays protocol-only.
- **Reproducible source gate**: Rust `1.95.0` is pinned, and Linux/macOS CI runs formatting, all-feature Clippy, the full test tier, and status synchronization.
- **Provider integration**: supports `mock`, `openai-compatible`, and OpenRouter configuration paths. Provider secrets should stay in private local config or environment variables.

## Use Cases

- Add MCP memory to a local AI client.
- Study how an agent can update long-term memory through explicit evidence.
- Validate a minimal loop for self snapshots, reflection, and commitment gates.
- Use a Rust + SQLite + MCP `stdio` project as an engineering reference.

## Quick Start

macOS:

```zsh
./scripts/agent-llm-mm.sh bootstrap-local
./scripts/agent-llm-mm.sh init
./scripts/agent-llm-mm.sh doctor --read-only
./scripts/agent-llm-mm.sh serve
```

Windows:

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 bootstrap-local
pwsh -File .\scripts\agent-llm-mm.ps1 init
pwsh -File .\scripts\agent-llm-mm.ps1 doctor --read-only
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

`bootstrap-local` creates a local config template from the development example. It does not overwrite existing files, create secrets, or start the service.

Platform and integration guides:

- [macOS development guide](docs/development-macos.md)
- [Windows development guide](docs/development-windows.md)
- [Local MCP integration guide](docs/local-mcp-integration-2026-03-26.md)

## Demo

Run the reproducible self-revision demo:

```zsh
./scripts/run-self-revision-demo.sh
```

The demo starts a deterministic local `openai-compatible` stub provider, runs the canonical scenario through the real MCP `stdio` service, and writes its report to `target/reports/self-revision-demo/...`.

## Local Diagnostics

Generate a redacted support bundle:

```zsh
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
```

The support bundle contains redacted JSON summaries only. It does not copy the full SQLite database, raw TOML, provider payloads, or raw `.log` files. To export a focused log excerpt or a specific MCP tool call, pass `--log-file` or `--correlation-id` explicitly.

## Current Boundaries

Implemented:

- MCP `stdio` main flow
- SQLite persistence with owner / namespace constraints
- Audited claim replacement through `run_reflection`
- Minimal identity and commitment revision
- Trigger-ledger-backed automatic self-revision MVP
- Read-only dashboard, `doctor`, local support bundle, and local gate summary scripts
- Explicit SQLite schema version / migration ledger, transactional legacy migration, pre-write backup and restore rehearsal
- Loopback-only enabled dashboard configuration, stderr-only tracing, and Linux/macOS source CI on pinned Rust `1.95.0`
- M1.1.1 scoped event recall through the additive `search_memory` MCP tool; cross-namespace exact-ID matches return empty and semantic memory tables remain unchanged by reads
- M1.1.2 scoped claim recall through additive `search_memory(record_type = Claim)`; omitted `claim_status` defaults to `Active`, claim results retain canonical evidence/episode and direct reflection revision links, and event/time filters are rejected because claims have no stored `recorded_at`
- M1.2.1 scoped event lookup through additive `get_memory`; a missing or cross-namespace stable ID returns `record: null` without widening
- M1.2.2 scoped claim lookup through explicit `get_memory(record_type = Claim)`; exact lookup accepts canonical/raw Claim IDs and returns Active, Disputed, or Superseded claims with the same provenance shape as Claim search

Partially implemented:

- M0.2 is complete only for the explicit scoped-snapshot boundary described above. Legacy calls that omit `namespace` remain unscoped for MCP compatibility, and the repository still has no complete user-facing recall contract.
- `build_self_snapshot` now accepts an additive `namespace` input. The server derives the matching owner and pushes exact owner/namespace filtering through the application/query-port/SQLite path for claims, event references, and episode references. `MemoryScope` accepts only a matching owner+namespace pair or the fully empty legacy compatibility state; partial and mismatched deserialized scopes are rejected. Calls that omit `namespace` retain the legacy unscoped behavior for MCP compatibility, so callers that require isolation must pass it explicitly.
- `build_self_snapshot` also accepts an additive explicit `evidence_manifest`, which requires an explicit `namespace` so it cannot enter the legacy unscoped compatibility path and is capped at 256 entries before query construction. DTO, application, and store boundaries fail closed; snapshot inputs are validated before optional auto-reflection runs. Bare event IDs and `event:<id>` references normalize to one canonical representation with order-preserving linear-time deduplication; SQLite intersects the manifest with the server-owned owner/namespace scope, and an empty intersection stays empty without widening. Tool operation logs use the snapshot namespace, while separate auto-reflection diagnostics retain their own namespace.
- Optional `recorded_after` / `recorded_before` now add an inclusive snapshot time window. Either bound requires an explicit `namespace`; reversed bounds are rejected before optional auto-reflection. SQLite intersects owner/namespace, manifest, and time in the evidence query, normalizes project-canonical timestamps and common legacy RFC3339 `Z` / offset variants into a fixed-width UTC key while preserving nine fractional-second digits, and returns recent-first evidence with a stable row-id tie-break. Episodes are selected and ordered by their latest qualifying event tuple, while claims remain scope-only because they do not currently carry a recorded timestamp. An explicit window with no matches stays empty without widening; legacy unbounded snapshot calls remain available but now use recent-first SQLite ordering.
- Scoped snapshot v2 now also feeds automatic reflection from an explicit server-derived `MemoryScope`, the current trigger evidence manifest, and the inclusive bounds derived from that manifest. Candidate detection and its dependent episode read use the same owner/namespace/window; no qualifying intersection leaves auto-reflection untriggered rather than falling back to historical data. Explicit MCP `build_self_snapshot` calls retain their legacy-compatible path.
- In the active reflection runtime, MCP `replacement_evidence_event_ids`, application explicit/query evidence merging, and model-proposed auto-reflection evidence accept either raw IDs or `event:<id>`. They parse through `EventReference`, reject blank/empty/repeated-prefix forms, and deduplicate by underlying raw ID while preserving first-seen order. Store lookups, evidence links, reflection audit `supporting_evidence_event_ids`, and auto-reflection diagnostic `*_event_ids` deliberately remain raw for compatibility; canonical `event:<id>` is reserved for reference-shaped fields.
- The read-only evidence-relation and episode-summary projections now apply the same parsing and ordered raw-ID deduplication to their trigger/selected/episode/linked `*_event_ids`, before subset checks and count/rank derivation. Their JSON readback deliberately remains raw IDs and still performs no persistence or runtime read-path work.
- The deterministic offline self-revision demo now exposes its generated baseline event in `timeline.json` as a canonical `event_reference`; malformed MCP event IDs fail the runner instead of entering artifacts. Snapshot evidence is already reference-shaped, while the stub's empty `proposed_evidence_event_ids` and SQLite `supporting_evidence_event_ids` remain explicit raw-ID compatibility fields. Repository-wide normalization remains partial: support bundle and other excluded surfaces are not changed by these slices.
- `decide_with_snapshot` still uses an action-string contract and is not a full decision engine. M0.3 replaces caller-provided commitments with the current server-side commitment store before invoking the model, and applies the same commitment gate to both the requested and provider-selected actions. A selected action rejected by policy is returned as blocked with no `decision` payload. A non-blocked action keeps the compatible `model_decision` / `{ "action": "..." }` shape but now reports `decision_authority = experimental_non_authoritative` and `policy_scope = server_commitment_gate_only`; `gate.blocked = false` is not a full policy-passed verdict.
- Automatic identity revision now computes cross-episode support only from distinct episodes reached through the selected active claims' persisted `evidence_links` and `episode_events`. Unrelated global episodes no longer raise the support count. This is a bounded governance join over the existing schema, not a complete provenance model.
- Governance validation failures write only a rejected trigger audit. Failures while appending the handled trigger ledger or committing the reflection transaction roll back pending identity, commitment, claim/evidence, reflection, and handled-ledger changes; a separate rejected audit is then recorded outside the failed transaction. This is locally verified failure atomicity, not crash-recovery or distributed transaction support.
- Identity, claims, evidence, and episodes in the decision snapshot are still caller-provided; there is no server-created snapshot handle or complete policy/provenance binding yet.
- Episodes are currently lightweight projections, not a complete autobiographical memory model.
- The runtime read interface now covers complete event and claim search/lookup records with status, mode, evidence/episode provenance, and direct source/supersession reflection links. Episode/reflection records, complete revision history, correction tools, and richer memory-layer projections are not yet unified behind the same contract.
- Provider live evidence proves configuration and connectivity only. It does not prove model quality, SLA, or production readiness.
- Local alpha gates still depend on external evidence such as a real fresh-machine run, Windows parity, and a human release decision.
- The repository remains on `rmcp 0.5.0`. An isolated `2.2.0` compatibility probe is documented as no-go for an in-place M0.5 bump because one handler error contract regressed; the future upgrade must remain capability-neutral.

Not implemented:

- Full memory layering
- Complete cross-record memory lookup, episode/reflection read models, provenance/reflection history, and audited correction tools
- Richer evidence ranking / weighting
- Production-grade remote, team, or multi-tenant mode
- Daemon write capabilities and autonomous background operation
- Installers, service managers, auto-updaters, and release certification

See [project status](docs/project-status.md), the [roadmap](docs/roadmap.md), and the [active 2026-07-10 project plan](docs/plans/2026-07-10-product-replan.md) for the current implementation boundary and execution order.

## Documentation

### Understand the project

1. [Project origin and mainline principles](docs/origin-and-principles.md)
2. [Positioning](docs/positioning.md)
3. [Current implementation status](docs/project-status.md)
4. [Now / Next / Later roadmap](docs/roadmap.md)
5. [Active project plan](docs/plans/2026-07-10-product-replan.md)

The active plan is the only current execution queue. Historical checklists are
preserved for traceability, but they do not define current work.

### Build and verify

- [macOS development guide](docs/development-macos.md)
- [Windows development guide](docs/development-windows.md)
- [Local MCP integration guide](docs/local-mcp-integration-2026-03-26.md)
- [Testing guide](docs/testing-guide-2026-03-24.md)

### Reference and history

- [Complete document map](docs/document-map.md)
- [Historical archive](docs/archive.md)

## Verification

Tests are split into `fast`, `core`, and `full` tiers. Release evidence, packaging,
and provider-certification tooling is opt-in through the `release-tools` feature.

Common local checks:

```zsh
./scripts/test-tier.sh fast
./scripts/test-tier.sh core
./scripts/status-sync-check.sh
./scripts/agent-llm-mm.sh doctor
git diff --check
```

Use `./scripts/test-tier.sh full` for release-tool changes and final full-feature
verification. For provider, dashboard, release-evidence, or support-bundle changes,
follow the relevant layered checks in the [testing guide](docs/testing-guide-2026-03-24.md).

## Naming

The public project name is MCP Memory Ledger. The current Rust crate, binary, scripts, config examples, and some historical docs still use `agent_llm_mm` / `agent-llm-mm` as compatibility identifiers.

## Acknowledgements

This repository has been developed, reviewed, and documented with active support from OpenAI Codex as a collaborative development tool. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright 2026 yooyui
