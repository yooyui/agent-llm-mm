# Daemon Observe-Only Gate

## Scope

This gate protects future daemon work from jumping directly into write-capable
autonomy. Local Alpha keeps the daemon disabled by default. `doctor` may report
daemon configuration, but it must not start a daemon.

Observe-only mode is a diagnostic phase. It may read local state and report what
it would consider later, but it must not change identity, commitments,
reflections, claims, events, or tool responses.

## Required Observe-Only Behavior

An observe-only daemon may:

- read configured local stores and durable operation-log summaries
- inspect trigger candidates for repeated failures, conflicts, or periodic
  freshness checks
- compute diagnostics about suppression, cooldown, concurrency, and errors
- expose local-only diagnostic summaries for manual review
- exit cleanly without starting remote listeners or background write workers

It must be safe to run with formal local data because it does not write semantic
memory, identity, commitments, or reflection records.

## Forbidden Behavior

During observe-only Local Alpha work, the daemon must not:

- call `run_reflection`
- write identity updates
- write commitment updates
- write reflection audit records
- create a side-channel durable write path
- start a remote listener
- ingest remote triggers
- expose remote management controls
- change MCP tool responses
- claim continuous autonomy, self-governance, or all-entry automatic
  self-revision

The future daemon trigger policy in
[`../superpowers/specs/2026-04-27-local-daemon-trigger-policy.md`](../superpowers/specs/2026-04-27-local-daemon-trigger-policy.md)
describes a later stage where daemon-triggered writes, if allowed, still go
through governed `run_reflection`. That later policy does not authorize
`run_reflection` calls in the observe-only gate.

## Required Diagnostics

Observe-only diagnostics must be explicit enough to debug without implying write
authority:

- daemon enabled/disabled state and effective config
- polling interval and max concurrency settings
- local-only data sources inspected
- trigger candidates observed
- trigger candidates skipped or suppressed
- cooldown status
- in-flight task count
- errors encountered while reading local state
- whether a write-capable daemon gate has been approved; for Local Alpha this is
  `false`

Diagnostics may be logged or displayed as local observation data, but they must
not be treated as identity or commitment updates.

## Exit Gate Before Write-Capable Daemon Work

Before any daemon can trigger governed self-revision, a separate gate must prove:

- daemon remains disabled by default
- observe-only mode has tests for no identity, commitment, or reflection writes
- durable operation-log read paths are stable
- correlation IDs connect daemon diagnostics to operation summaries
- cooldown and concurrency behavior are tested
- failures are visible without changing MCP tool responses
- any future write still calls the existing governed `run_reflection` path
- there is no remote listener, remote trigger ingestion, or remote write admin
  surface

Failing this gate means daemon work remains observe-only or not implemented.

## Local Alpha Release Position

Passing this document's review does not make a daemon implemented. It only means
the product has a safe boundary for future daemon work. Local Alpha may say
"daemon disabled by default" and "observe-only daemon work is gated"; it must not
claim background autonomy or production self-governance.
