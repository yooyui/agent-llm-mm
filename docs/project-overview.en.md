# Project Overview

## Summary

`agent_llm_mm` is a Rust-based local MCP `stdio` server that validates a minimal loop for long-term memory, self-snapshot construction, and reflection. The current version uses SQLite for persistence and is best described as a technical demo, integration prototype, or research-oriented MVP rather than a complete product.

## Current Scope

- Local MCP `stdio` server
- SQLite persistence
- Config-file driven provider loading
- `openai-compatible` provider
- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`
- `doctor` / `serve` entry points

## Current Boundaries

- `decide_with_snapshot` can now use an `openai-compatible` provider, but its output contract is still a minimal action string
- There is no remote HTTP transport
- There is no richer evidence lookup / weight / relation yet
- There are no additional provider integrations yet
- The broader multi-layer memory model is still incomplete

## Best Fit

- Local AI client integration experiments
- Self-agent memory demos and technical validation
- A minimal Rust + MCP + SQLite reference implementation

## Documentation Discipline

- After finishing each task, update the corresponding documentation whenever that task changes behavior, capability boundaries, integration flow, configuration, verification commands, or collaboration rules.
- Do not defer documentation updates until the end of a larger batch of work; code and docs should be closed out together whenever possible.

## Verification Status

As of `2026-03-31`:

- `cargo test` passes in full with 58 tests
- `doctor` returns `status = ok`

## Acknowledgement

This repository was developed, discussed, and documented with active support from OpenAI Codex as a collaborative development tool. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.
