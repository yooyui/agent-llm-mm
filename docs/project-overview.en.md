# MCP Memory Ledger Project Overview

## Summary

MCP Memory Ledger is a Rust-based local MCP `stdio` memory demo that validates a minimal loop for long-term memory, self-snapshot construction, and reflection. The current version uses SQLite for persistence and is best described as a technical demo, integration prototype, or research-oriented MVP rather than a complete product.

Compatibility note: the current Rust crate, binary, scripts, configuration examples, and some historical docs still use the technical identifier `agent_llm_mm` / `agent-llm-mm`. Use MCP Memory Ledger as the public project name.

## Current Scope

- Local MCP `stdio` server
- SQLite persistence
- Config-file driven provider loading
- `openai-compatible` / OpenRouter provider
- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`
- `doctor` / `serve` entry points

## Current Boundaries

- `decide_with_snapshot` can now use an `openai-compatible` or OpenRouter provider, but its output contract is still a minimal action string
- There is no remote HTTP transport
- There is no richer evidence lookup / weight / relation yet
- Azure and local-model providers are not implemented yet; OpenRouter uses the OpenAI-compatible transport, and the explicit live runner only generates provider preflight evidence, not provider quality, SLA, or gateway certification
- The release evidence index, provider certification preflight, packaging preflight, and richer memory semantics projection are local read-only / preflight capabilities; they do not create missing product evidence, certify provider quality, build installers, or complete the broader multi-layer memory model

## Best Fit

- Local AI client integration experiments
- Self-agent memory demos and technical validation
- A minimal Rust + MCP + SQLite reference implementation

## Documentation Discipline

- After finishing each task, update the corresponding documentation whenever that task changes behavior, capability boundaries, integration flow, configuration, verification commands, or collaboration rules.
- Do not defer documentation updates until the end of a larger batch of work; code and docs should be closed out together whenever possible.

## Verification Status

As of `2026-07-10`:

- tests are split into `fast`, `core`, and `full` tiers; default core checks do not compile release tooling
- the `release-tools` feature retains release-evidence, packaging, and provider-certification verification
- `doctor` returns `status = ok`

## Acknowledgement

This repository was developed, discussed, and documented with active support from OpenAI Codex as a collaborative development tool. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.
