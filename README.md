# MCP Memory Ledger

Local-first MCP memory for AI agents, backed by SQLite and evidence-gated self-revision.

Languages: English | [Simplified Chinese](docs/README.zh-CN.md) | [Japanese](docs/README.ja.md)

MCP Memory Ledger is a Rust MCP `stdio` memory service for local AI clients. It records interactions, evidence, claims, self snapshots, and reflection audits in SQLite so an agent can use durable memory inside explicit, inspectable boundaries instead of relying only on a single prompt context.

The project grew out of a discussion about information loss, durable memory, and how selected parts of the past can safely constrain future agent behavior. The [origin and mainline principles](docs/origin-and-principles.md) are now the entry point for deciding what belongs in this repository.

The current project is best understood as a technical MVP for local agent memory, MCP integration, SQLite persistence, and governed self-revision. It is not a production autonomous-agent platform. Remote team mode, multi-tenancy, packaged installers, daemon write capabilities, and production security boundaries remain gated roadmap work.

## Features

- **Local MCP memory service**: exposes `ingest_interaction`, `build_self_snapshot`, `decide_with_snapshot`, and `run_reflection` over MCP `stdio`.
- **SQLite persistence**: stores events, claims, evidence, reflection audits, trigger ledger entries, and operation logs.
- **Evidence-gated self-revision**: claim, identity, and commitment updates must be backed by explicit evidence and governance rules. `run_reflection` remains the only durable write path for identity, commitment, and reflection changes.
- **Bounded local operations**: includes operation-log lookup, backup / restore helpers, and redacted diagnostics. The current `doctor` runtime check can create, migrate, and seed the configured SQLite database, so it must not be treated as a no-write inspection until the planned command split lands.
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
./scripts/agent-llm-mm.sh doctor
./scripts/agent-llm-mm.sh serve
```

Windows:

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 bootstrap-local
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
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

Partially implemented:

- `build_self_snapshot` is not yet a trustworthy namespace-scoped read model: it aggregates store-wide claims, event references, and episode references, then applies a budget.
- `decide_with_snapshot` still uses an action-string contract and is not a full decision engine.
- The decision gate checks caller-provided commitments and the requested action; it does not yet bind to a server-created snapshot or re-check the provider-selected action.
- Episodes are currently lightweight projections, not a complete autobiographical memory model.
- Evidence relation, episode summary, and memory-layer projections are not yet unified behind a runtime memory read interface.
- Provider live evidence proves configuration and connectivity only. It does not prove model quality, SLA, or production readiness.
- Local alpha gates still depend on external evidence such as a real fresh-machine run, Windows parity, and a human release decision.

Not implemented:

- Full memory layering
- User-facing scoped memory search, record lookup, provenance history, and audited correction tools
- Richer evidence ranking / weighting
- Versioned transactional schema migrations and a no-write doctor mode
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
