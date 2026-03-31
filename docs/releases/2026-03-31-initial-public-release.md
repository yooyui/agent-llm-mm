# Initial Public Release

## Suggested Tag

`v0.1.0`

## Suggested Title

`v0.1.0 - Initial public release`

## Release Summary

This is the first public release of `agent_llm_mm`, a Rust-based local MCP `stdio` memory demo for AI clients.

The project now provides a usable local integration baseline with:

- SQLite-backed persistence
- MCP `stdio` tooling for memory ingestion, snapshot building, reflection, and decision flow
- config-file driven provider loading
- an implemented `openai-compatible` provider path
- automated tests covering runtime behavior and provider integration

This release should be understood as a technical demo / research-oriented MVP, not a complete production system.

## Highlights

### New

- Added config-file driven provider loading
- Added an `openai-compatible` model adapter
- Added provider-specific bootstrap and validation in `doctor`
- Added an example local config file for safe local setup
- Added public-repo documentation in Chinese, English, and Japanese
- Added `Apache-2.0` licensing with `NOTICE`

### Improved

- Runtime can now select between `mock` and `openai-compatible` providers
- MCP `stdio` integration now has automated coverage for the config-file driven provider path
- Release-facing docs now distinguish between current capabilities, roadmap, and historical design materials

### Testing

Fresh verification for this release:

- `cargo test` passed in full
- Total tests passing: `58`
- `pwsh -File .\scripts\agent-llm-mm.ps1 doctor -ConfigPath <config>` returned `status = ok`

## Important Notes

- `decide_with_snapshot` is no longer limited to the mock-only path, but it still uses a minimal action-string contract.
- This repository does not yet implement richer memory semantics such as deeper identity revision, richer evidence modeling, or multi-layer memory completion.
- Additional provider integrations are not implemented yet.
- Local secrets must stay in `agent-llm-mm.local.toml`, which is intentionally ignored by git.

## Upgrade / Setup Notes

1. Copy `examples/agent-llm-mm.example.toml` to `agent-llm-mm.local.toml`
2. Fill in your local database path and provider settings
3. Keep the local config file out of version control
4. Run:

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

## Suggested Short Release Description

Initial public release of a Rust-based local MCP `stdio` memory demo with SQLite persistence, config-file driven provider loading, and `openai-compatible` model integration.
