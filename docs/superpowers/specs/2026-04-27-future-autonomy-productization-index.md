# Future Autonomy and Productization Spec Index

## Purpose

This index tracks future capabilities beyond the current local MCP memory MVP. It does not claim these capabilities are implemented.

## Capability Tracks

| Track | Future Goal | Current Status | First Required Spec |
| --- | --- | --- | --- |
| Durable operation log | Persistent audit trail for all agent operations | Phase 1 MVP implemented (local SQLite) | durable-operation-log-design |
| Autonomous agent | Complete autonomous agent behavior | Not implemented | autonomy-governance-spec |
| Production self-governance | Production-grade self-governing system | Not implemented | production-governance-spec |
| Remote management | Remote management backend | Not implemented | remote-admin-boundary-spec |
| Product service | Auth, multi-tenancy, productization | Not implemented | productization-foundation-spec |
| Background daemon | All-entry auto-reflection daemon | Not implemented | daemon-trigger-policy-spec |

## Entry Criteria

- Current MVP release gate is stable.
- Current 4 runtime hooks have documented and tested boundaries.
- Self-revision diagnostics are inspectable.
- Evidence policy has bounded server-side validation.

## Exit Criteria

Each track has a separate implementation plan with tests, rollback strategy, and explicit non-goals before code work starts.
