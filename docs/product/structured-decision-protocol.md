# Structured Decision Protocol

## Scope

This document defines the current `decide_with_snapshot` response envelope for the local Rust MCP `stdio` technical demo / MVP. It is a compatibility contract for callers, not a complete decision engine design.

The model provider contract remains conservative: `ModelDecision` still only carries the model-selected `action` string. The structured protocol metadata is added by the application layer around that decision.

## Version 2 Response Envelope

`DecideWithSnapshotResult` preserves the existing caller fields:

- `blocked`: boolean gate outcome.
- `decision`: model decision object when a model was called, or `null` when the commitment gate blocked the action.

Version 2 keeps the earlier metadata and adds local explainability fields:

- `protocol_version`: currently `2`.
- `decision_id`: deterministic local id for the requested action.
- `requested_action`: action requested by the caller.
- `selected_action`: model-selected action when the model runs, or `null` when blocked.
- `confidence`: bounded local metadata for the current provider path; this is not calibrated statistical confidence.
- `status`: `blocked` when the commitment gate stops the action, or `model_decision` when the model decision path returns normally.
- `reason`: machine-readable reason when blocked; currently `commitment_gate_blocked_action`. It is `null` for normal model decisions.
- `gate`: commitment-gate metadata with `name`, `blocked`, and `reason`.
- `policy_checks`: current policy-check list. The first slice contains the commitment gate.
- `non_claims`: explicit boundaries for this local protocol.

Blocked responses must not call the model and must keep `decision: null`. Non-blocked responses must keep the original `decision` payload shape, currently `{ "action": "..." }`.

## Non-Goals

- This does not add multi-step decision planning, confidence scoring, or policy arbitration.
- This does not require providers to emit structured JSON decisions.
- This does not change `run_reflection`; it remains the only durable self-revision write path.
- This does not change provider readiness or provider matrix status files.

## Verification

The compatibility slice is covered by:

```zsh
cargo test --test decision_flow -v
```
