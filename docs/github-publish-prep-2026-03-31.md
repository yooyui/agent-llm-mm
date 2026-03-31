# GitHub Publish Prep

## Repository Name

Suggested repository name:

- `agent_llm_mm`

## Suggested Description

Primary option:

`Rust-based local MCP stdio memory demo for AI clients, with SQLite persistence and config-file driven provider loading.`

Shorter option:

`Local MCP stdio memory demo in Rust with SQLite persistence and OpenAI-compatible provider support.`

## Suggested Topics

Recommended topics:

- `rust`
- `mcp`
- `model-context-protocol`
- `ai-agent`
- `memory`
- `sqlite`
- `openai-compatible`
- `local-first`
- `stdio`
- `llm`

## Suggested Homepage Copy

### README first paragraph

`agent_llm_mm` is a Rust-based local MCP `stdio` server for AI clients. It validates a minimal self-agent memory loop around interaction ingestion, self-snapshot construction, gated decisions, and reflection, backed by SQLite persistence and config-file driven provider loading.

### Short tagline

`A local-first MCP memory demo for AI clients.`

### Public positioning

Use this positioning consistently:

- technical demo
- research-oriented MVP
- local MCP integration prototype

Avoid these claims:

- production-ready memory platform
- complete self-agent system
- full decision engine

## Suggested Release Metadata

Suggested first release tag:

- `v0.1.0`

Suggested first release title:

- `v0.1.0 - Initial public release`

## GitHub Release Body Source

Recommended source file:

- [2026-03-31-initial-public-release.md](releases/2026-03-31-initial-public-release.md)

## Remaining Blocking Item

Current blocker for push/release automation:

- no Git remote is configured locally
- `https://github.com/yooyui/agent_llm_mm.git` does not currently exist
- GitHub CLI is not installed on this machine

## Recommended Next Steps

1. Create the GitHub repository under `yooyui`
2. Share the exact repository URL, or let local `origin` be configured
3. Push `master`
4. Create tag `v0.1.0`
5. Paste the prepared release note into the GitHub release form
