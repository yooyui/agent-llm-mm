# Local Daemon Trigger Policy

## Scope

This spec defines the trigger policy for a future local background daemon. The daemon is **not implemented** — this document defines the contract it must satisfy before implementation begins.

## Core Constraints

1. Daemon is **local-only** in the first stage — no network listener, no remote trigger
2. Daemon is **disabled by default** — requires explicit `[daemon] enabled = true` in config
3. Daemon only **reads** from durable operation log and existing stores
4. Daemon writes identity/commitment updates **only** by calling the existing governed `run_reflection` path
5. Daemon has cooldown, concurrency limit, and shutdown semantics
6. Daemon **never** changes MCP tool responses after the fact

## Trigger Classes

| Trigger Class | Source | Initial Status |
| --- | --- | --- |
| Repeated failure | Durable operation log (consecutive failed operations) | Future |
| Unresolved conflict | Claims store and trigger ledger (stale conflict entries) | Future |
| Scheduled review | Local timer (configurable interval) | Future |
| Stale commitment review | Commitments store and audit log (commitments without recent evidence) | Future |

## Cooldown and Concurrency

- Minimum cooldown between daemon-triggered reflections: configurable, default 5 minutes
- Maximum concurrent daemon tasks: configurable, default 1
- If `run_reflection` is already in progress (from MCP hook or prior daemon trigger), daemon waits

## Shutdown Semantics

- Daemon stops cleanly on SIGTERM/SIGINT
- In-flight `run_reflection` calls are allowed to complete (bounded by existing timeout)
- No orphaned state: if daemon crashes mid-reflection, the trigger ledger records the attempt as failed

## Config Shape

```toml
[daemon]
enabled = false
poll_interval_ms = 60000
max_concurrent_tasks = 1
cooldown_ms = 300000
```

## Doctor Integration

Doctor reports daemon config without starting the daemon:
- `daemon_enabled`: bool
- `daemon_poll_interval_ms`: u64
- `daemon_max_concurrent_tasks`: u32

## Non-Goals

- Remote trigger ingestion
- Cross-process coordination
- Autonomous widening of reflection scope beyond existing trigger classes
- Replacing MCP hook-driven auto-reflection (daemon supplements, not replaces)
