# Local MCP Minimal Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the project runnable and diagnosable as a local MCP stdio server that can be embedded into Codex-style local AI clients with minimal friction.

**Architecture:** Keep `stdio` as the only transport and add a thin CLI layer for `serve` and `doctor` modes. Reuse the existing runtime bootstrap path so the same SQLite and identity initialization logic backs both diagnostics and the actual MCP server.

**Tech Stack:** Rust 2024, Tokio, rmcp, SQLite/sqlx, PowerShell 7

---

### Task 1: Add CLI parsing and doctor behavior with TDD

**Files:**
- Create: `src/support/cli.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Modify: `src/support/mod.rs`
- Modify: `src/interfaces/mcp/mod.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Test: `tests/bootstrap.rs`

- [ ] **Step 1: Write failing tests for CLI parsing and doctor bootstrap**

Add tests that prove:
- default CLI command is `serve`
- `doctor` is parsed explicitly
- doctor mode bootstraps the configured SQLite path and returns a report without blocking

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cargo test --test bootstrap`
Expected: failures for missing CLI and doctor behavior

- [ ] **Step 3: Implement the minimal CLI and doctor runtime path**

Add:
- a small `AppCommand` parser with `serve` and `doctor`
- a doctor report type
- a reusable runtime validation path that uses the same store bootstrap logic as MCP server startup

- [ ] **Step 4: Re-run the targeted tests**

Run: `cargo test --test bootstrap`
Expected: PASS

- [ ] **Step 5: Refactor only if needed**

Keep `stdio` startup behavior unchanged for the default path.

### Task 2: Add a stable PowerShell entrypoint for local MCP clients

**Files:**
- Create: `scripts/agent-llm-mm.ps1`

- [ ] **Step 1: Add a script that resolves the project root and invokes the binary in `serve` or `doctor` mode**

Script requirements:
- default to `serve`
- support `doctor`
- allow forwarding `AGENT_LLM_MM_DATABASE_URL`
- work from any current directory

- [ ] **Step 2: Smoke test the script locally**

Run:
- `pwsh -File .\scripts\agent-llm-mm.ps1 doctor`
- `pwsh -File .\scripts\agent-llm-mm.ps1 serve`

Expected:
- doctor exits successfully
- serve starts and waits on stdio

### Task 3: Add docs and Codex MCP configuration examples

**Files:**
- Create: `README.md`
- Create: `docs/local-mcp-integration-2026-03-26.md`
- Create: `examples/codex-mcp-config.toml`

- [ ] **Step 1: Document project status and supported local integration path**

Cover:
- current capabilities
- current non-goals
- local testing flow
- `decide_with_snapshot` still using mock model

- [ ] **Step 2: Add a Codex config example**

Use the existing local config style:
- `[mcp_servers.<name>]`
- `command`
- `args`
- `env`

- [ ] **Step 3: Document the recommended verification sequence**

Include:
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- script-based `doctor`

### Task 4: Verify the full minimal closure

**Files:**
- Verify only

- [ ] **Step 1: Run format check**

Run: `cargo fmt --check`
Expected: exit code `0`

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: exit code `0`

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: all tests PASS

- [ ] **Step 4: Run doctor smoke test**

Run: `pwsh -File .\scripts\agent-llm-mm.ps1 doctor`
Expected: successful diagnostic output

- [ ] **Step 5: Run serve smoke test**

Run: `pwsh -File .\scripts\agent-llm-mm.ps1 serve`
Expected: process stays alive waiting for stdio until interrupted
