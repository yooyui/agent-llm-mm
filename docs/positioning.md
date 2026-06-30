# MCP Memory Ledger Positioning

## Canonical Name

Use **MCP Memory Ledger** as the project name.

Keep `agent_llm_mm` only as the current implementation package name for the Rust crate, binary, scripts, configuration examples, and compatibility references until a separate rename migration is planned.

## One-Sentence Description

MCP Memory Ledger is a local-first MCP memory layer for AI agents with SQLite persistence, explicit evidence events, and governed self-revision.

## Longer Description

MCP Memory Ledger gives local AI clients a persistent MCP `stdio` memory server. It records interactions, builds self snapshots, and supports evidence-gated self-revision through bounded triggers and auditable diagnostics. The current repository remains a technical demo / MVP and should not be described as a production-grade autonomous agent platform.

## Target Users

- Developers building local AI clients with MCP
- Agent researchers studying long-term memory and self-revision
- Engineers studying agent memory, evidence, and governance patterns
- Maintainers who need a conservative Rust + SQLite MCP memory reference

## What To Say

- Local-first MCP memory layer
- Evidence-gated self-revision MVP
- Rust MCP `stdio` server with SQLite persistence
- OpenAI-compatible / OpenRouter provider integration
- Technical demo / verified MVP entering productization planning

## What Not To Say

- Production-grade autonomous agent platform
- Complete self-governing AI system
- Full remote/team/multi-tenant product
- Provider quality certification or model gateway
- Drop-in replacement for broad memory platforms or stateful agent platforms

## Naming Policy

- Project name: MCP Memory Ledger
- Repository slug: `mcp-memory-ledger`
- Current implementation package: `agent_llm_mm`
- Current compatibility aliases: `agent_llm_mm`, `agent-llm-mm`
- Do not use `agent_llm_mm` as the public project headline in new docs

## GitHub Metadata

Repository slug:

```text
mcp-memory-ledger
```

Suggested GitHub description:

```text
Local-first MCP memory layer for AI agents with SQLite persistence and evidence-gated self-revision.
```

Suggested topics:

```text
mcp
model-context-protocol
agent-memory
ai-agents
llm-memory
self-revision
local-first
sqlite
rust
openai-compatible
```

## GEO Notes

Generative search systems need short, repeated, consistent definitions. Put the canonical one-sentence description near the top of README files, FAQ pages, release notes, and project overview pages. Avoid keyword stuffing; prefer direct definitions, concrete boundaries, and concise explanations of what this project does and does not claim.
