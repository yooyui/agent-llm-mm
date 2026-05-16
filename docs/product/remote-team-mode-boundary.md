# Remote and Team Mode Boundary

## Scope

This document defines the boundary for future remote and team-mode work. It is
not an implementation claim. The current product track remains a validated local
MVP entering productization, with Local Alpha focused on local setup, local
SQLite data safety, local read-only dashboard inspection, support bundles, and
governed `run_reflection` writes.

Remote/team mode starts only after local reliability, observability, and release
evidence are stable. Until then, remote write admin, team service operation, and
multi-tenancy are explicit non-goals.

## Remote Surface Order

The first remote-capable surface must be a remote read-only dashboard. It may
expose bounded health, summary, event, operation-log, and diagnostic views only
after the remote bind, auth, redaction, rate limit, and audit requirements have
been designed and reviewed.

The first remote surface must not expose:

- semantic memory writes
- identity writes
- commitment writes
- reflection writes
- provider credential updates
- daemon start / stop controls
- backup, restore, migration, or destructive maintenance actions
- support bundle upload or export over a remote channel

Any future remote dashboard must remain observability-first. If an operator
needs to change local state, that belongs to a later write/admin gate.

## Write/Admin Gate

No remote write or write-capable admin operation is allowed before all of these
exist and pass review:

- authentication for every non-local caller
- authorization for every route and operation
- durable audit records for attempted, accepted, rejected, and failed admin
  actions
- rollback or compensating-action design for each write class
- tests for auth, authorization, audit, rollback behavior, redaction, and
  failure semantics
- operator-facing docs that describe what can be changed and how to recover
- security review that lists blockers before remote/admin code is enabled

For self-revision behavior, future write/admin routes must not bypass
`run_reflection` for identity, commitment, claim, or reflection changes unless a
separate architecture decision, migration plan, and rollback strategy explicitly
replace that path.

## Namespace and Database Isolation

Team mode requires isolation before it can be described as a supported product
capability. At minimum, the design must decide and test:

- how a tenant maps to one or more namespaces
- whether each tenant receives a separate SQLite database or a tenant-aware
  shared schema
- how operation-log reads are constrained to the caller's tenant or namespace
- how reflection audit entries are scoped and queried
- how dashboard projections avoid cross-tenant data leakage
- how support bundles exclude data from other tenants or namespaces
- how provider credentials are separated, rotated, and redacted per tenant or
  deployment boundary
- how backup, restore, export, and migration avoid mixing formal, demo, test,
  or tenant data

The conservative default for early controlled trials is database isolation per
deployment or tenant, with explicit namespace mapping inside that database.
Shared-row multi-tenancy should remain blocked until tenant-aware schema,
queries, indexes, tests, support bundle filtering, and audit filtering exist.

## Transport Decision

The MCP `stdio` core remains local. It is the primary integration boundary for
local AI clients and should not be converted into a network service as part of
remote/team mode.

HTTP is allowed only for admin or API surfaces that need request/response
semantics, browser access, or controlled remote observation. Adding HTTP does
not make a route safe by itself; each HTTP route still needs auth,
authorization, audit, redaction, rate limiting, and tests appropriate to its
capability.

Transport split:

| Surface | Transport | Boundary |
| --- | --- | --- |
| Local MCP tools | `stdio` | Local product core; governed writes stay here |
| Local dashboard | localhost HTTP | Read-only Local Alpha inspection |
| Future remote dashboard | HTTP | Read-only first; requires auth and audit gate |
| Future admin/API routes | HTTP | Blocked until write/admin gate passes |
| Future team service | HTTP plus storage isolation | Blocked until isolation and security gates pass |

## Local Alpha Non-Goals

Local Alpha does not include:

- remote write admin
- remote team service operation
- multi-tenancy
- public reverse-proxy deployment of the dashboard
- browser-based mutation routes
- network-exposed MCP transport
- production self-governance
- daemon-triggered writes
- remote support bundle upload
- shared provider credential management

Docs, demos, release notes, and product wording must keep these as future-stage
items until the relevant gates have fresh evidence.

## Minimum Future Review Checklist

Before remote/team implementation starts, the implementing branch should have:

- threat model reviewed for local and remote surfaces
- remote read-only route inventory approved
- explicit list of forbidden first-stage routes
- auth and authorization model approved
- namespace/database isolation decision approved
- provider credential storage and redaction policy approved
- audit event schema approved
- rollback or compensating-action strategy approved for each future write class
- tests named for auth, authorization, audit, isolation, redaction, and no
  cross-tenant reads
- release wording checked so no remote write, team mode, multi-tenancy, or
  production self-governance claim is made early
