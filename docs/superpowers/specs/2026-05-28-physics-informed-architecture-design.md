# Physics-Informed Architecture Design

> Status: planning spec only. This document does not implement runtime behavior,
> daemon writes, remote/team mode, provider adapters, release packaging, or a
> complete multi-layer memory architecture.

## Goal

Use objective physical-law analogies to organize the next architecture work for
`agent-llm-mm` while preserving the current product boundary: a local Rust MCP
`stdio` memory MVP / technical demo entering productization.

The planning target is a clearer structure for future work, not an immediate
large refactor. Near-term implementation should stay incremental, evidence-led,
and gated by spec review and focused verification.

## Current Evidence Baseline

This document was originally drafted from the `dev-work` checkout on
2026-05-28 and refreshed against the current `dev-work` implementation on
2026-05-29.

Checked commands:

```bash
git status --short --branch
./scripts/status-sync-check.sh
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
cargo run --quiet --bin local_alpha_evidence_summary -- --evidence-root .
cargo test
```

Observed baseline:

- `dev-work` carries the reviewed local implementation slices; branch ahead
  count is not used as release evidence.
- Current branch `cargo test` total declaration: 383 tests.
- `status-sync-check` reported that documented test totals and plan/status
  reality gates are in sync.
- `doctor` reported `status = ok` when run with an isolated writable SQLite
  path. If the default user database is read-only in the current environment,
  set `AGENT_LLM_MM_DATABASE_URL` to a writable local SQLite URL before using
  `doctor` as verification evidence.
- `doctor.self_revision_write_path = "run_reflection"`.
- `doctor.auto_reflection_runtime_hooks` reported four hooks:
  `ingest_interaction:failure`, `ingest_interaction:conflict`,
  `decide_with_snapshot:conflict`, and `build_self_snapshot:periodic`.
- `doctor.daemon_observe_only.writes_allowed = false` and
  `doctor.daemon_observe_only.remote_listener_enabled = false`.
- `doctor.remote_team_capability_inventory` reported remote/team capabilities
  blocked.
- `doctor.remote_team_security_gates.remote_writes_allowed = false`, with
  auth, authorization, audit, rate limit, tenant isolation, and rollback all
  unsatisfied.
- `local_alpha_evidence_summary` reported `overall_status = "in_progress"`.
  Product smoke, first-run bootstrap, support bundle, and local read-only
  boundary evidence are open or not verified; Windows parity is not verified.
  The summary now separates local `first_run_simulation` evidence from the
  real fresh-machine `first_run_bootstrap` gate.

## Status Classification

The project should continue to use explicit status language instead of broad
product claims.

| Capability area | Status | Current evidence | Missing evidence or implementation |
| --- | --- | --- | --- |
| Local MCP `stdio` memory MVP | `implemented` | Four MCP tools, SQLite persistence, `doctor status = ok`, documented main chain `events -> claims -> self_snapshot -> decision -> reflection` | Richer semantics and product hardening remain separate work |
| Governed `run_reflection` durable write path | `implemented` | `doctor.self_revision_write_path = "run_reflection"` and docs keep it as the identity / commitment / reflection write path | No alternate durable write path is approved |
| Automatic self-revision MVP | `implemented` | Trigger ledger, model proposal path, governed evidence window, four runtime hooks, demo package evidence in project docs | Not all-entry auto-reflection, not continuous autonomy, not a complete self-governing agent |
| Local read-only dashboard | `implemented` | Local dashboard HTTP surface, read-only history API, durable operation-log metadata in docs/tests | Not remote dashboard, not write UI, not authenticated team surface |
| Local support bundle | `partial` | Local generator, redaction boundaries, integrity manifest, focused tests | Not production support channel, not upload flow, not automatic log collector |
| Local Alpha evidence summary | `implemented` | Read-only rollup command exists and ran locally | The current evidence root still reports Local Alpha `in_progress` |
| Local Alpha full release gate | `partial` | Refresh/soak scripts and gate docs exist | Fresh product smoke/support bundle/first-run evidence, real fresh-machine evidence, Windows parity, and human release decision are still missing |
| Fresh-machine first-run | `simulation-only` | `first-run-bootstrap-smoke-local.sh` documents and tests isolated local simulation | Real fresh-machine clone/unpack/install evidence is missing |
| Windows parity | `planning-gate` | Windows docs and PowerShell/static contract coverage exist | Windows runner or Windows machine runtime evidence is missing |
| Provider matrix | `partial` | `mock`, `openai-compatible`, and `openrouter` are supported; OpenRouter is verified through local OpenAI-compatible `/chat/completions` stubs; planned-only providers are listed and rejected | Azure OpenAI and local provider adapters are not implemented; OpenRouter live-provider certification is not implemented |
| Evidence semantics v2 first slice | `implemented` | Read-only relation projection exists with selected / available-not-selected rows, window rank, rejection reason, bounded binary selection weight, rejected count, and no-widening governance; `doctor.system_layer_report.evidence_relation_contract` exposes the same v2 read-only contract at doctor level | Full ranking engine, widening, and richer evidence scoring are not implemented |
| Episode projection first slice | `implemented` | Read-only objective/outcome/linked-evidence projection exists | Full autobiographical episode schema and migration gate are not implemented |
| Multi-layer memory | `partial` | Read-only layered projection labels working / episodic / semantic / self-model as partial and procedural as not implemented | Distinct runtime layers, procedural memory, slow variables, durable self-model writes, lifecycle policy, and migrations are missing |
| Observe-only daemon diagnostics | `partial` | `doctor.daemon_observe_only` and local handle lifecycle tests exist; write/remote gates are closed | No connected background daemon service, no daemon loop, no daemon writes |
| Remote/team inventory | `implemented` as a blocker contract | `doctor` exposes blocked capability inventory | Remote/team product behavior is not implemented |
| Security/auth gate contracts | `implemented` as blocker contracts | `doctor` exposes unsatisfied auth/authz/audit/rate-limit/tenant/rollback gates | Actual auth, authorization, audit persistence, rate limits, tenant isolation, and rollback are not implemented |
| Release engineering | `partial` | Local soak runner and release docs exist | Installer, binary package, service manager, auto-updater, compatibility matrix automation, real release decision workflow, and Beta/GA evidence are missing |

## Non-Claims

The project must not be described as any of the following until matching gates
pass with fresh evidence:

- production-ready
- GA-ready
- full Local Alpha release complete
- complete self-governing agent
- all-entry automatic self-reflection system
- write-capable daemon service
- remote/team memory product
- authenticated remote admin surface
- multi-tenant service
- complete multi-layer cognitive architecture
- provider gateway supporting planned-only providers

## Physics Laws To Engineering Structure

The analogy is useful only when it constrains engineering decisions. Each law
below maps to a concrete boundary or review rule.

| Physical principle | Engineering interpretation | Structural rule for this project |
| --- | --- | --- |
| Causality | Effects must have traceable causes. | Every semantic write, gate status change, or release claim must link to evidence, command output, or an explicit human decision. |
| Conservation | State cannot appear from nowhere. | No capability can move from planned or simulated to implemented without code, tests, docs, and fresh evidence. |
| Arrow of time | Irreversibility and ordering matter. | Release gates must record dated evidence; stale MVP evidence cannot certify later Local Alpha or Beta claims. |
| Locality | Interactions happen through local neighborhoods. | Keep the MCP `stdio` core local; expose new behavior through bounded interfaces instead of broad global rewrites. |
| Feedback control | Stable systems need sensors, controllers, and actuators. | Separate observation, policy decision, and write actuation. Observe-only daemon diagnostics must not become write authority. |
| Entropy increase | Complexity grows unless bounded. | Add naming, status labels, drift checks, and small implementation slices to prevent docs, plans, tests, and reality gates from diverging. |
| Energy budget | Work is constrained by available energy. | Prioritize high-information, low-blast-radius slices before large architecture changes: reports, doctor diagnostics, status checks, and read-only projections first. |
| Boundary conditions | System behavior depends on initial and external constraints. | Keep Local Alpha local-only; block remote/team/security-sensitive work until auth, authorization, audit, rate limit, tenant isolation, rollback, and threat-model gates are satisfied. |

## Proposed Architecture Layers

The next structure should make state flow explicit. The layer names are planning
handles, not a mandate to rewrite directories immediately.

| Layer | Responsibility | Current anchors | Near-term direction |
| --- | --- | --- | --- |
| Substrate | Durable local process and storage: config, SQLite, migrations, script entrypoints, platform wrappers. | `scripts/agent-llm-mm.sh`, `scripts/agent-llm-mm.ps1`, config loading, SQLite store, backup/restore scripts | Add a read-only substrate report that summarizes config, DB path, platform entrypoint, and data lifecycle gate status. |
| Signal | Raw observations and bounded evidence handles: events, operation log, trigger candidates, evidence query windows. | `ingest_interaction`, `operation_log`, evidence relation projection, trigger ledger | Keep no-widening evidence policy explicit; add richer query slices only with focused tests. |
| Memory | Derived memory projections: claims, snapshot, episodes, layered memory projections, future semantic/procedural layers. | `build_self_snapshot`, read-only episode and layered memory projections, memory layering roadmap | Keep projections read-only until migration, evidence-link, and lifecycle tests exist for durable new layers. |
| Policy | Rules that decide what may happen: commitment gate, trigger governance, product wording guard, security gates. | `decide_with_snapshot`, trigger governance, product readiness checker, product wording guard, remote/team security gates | Make policy outputs machine-readable and auditable before adding new actuators. |
| Control Loop | Local feedback cycle that observes, decides, suppresses, cools down, and proposes. | `auto_reflect_if_needed`, trigger ledger, cooldown/suppression diagnostics, observe-only daemon diagnostics | Expand only inside the four existing hooks until diagnostics and failure semantics are stable. |
| Actuator | Any path that changes durable semantic state. | `run_reflection` | Preserve `run_reflection` as the durable identity / commitment / reflection write path. Future daemon or remote writes must call it or pass a separate architecture decision with migration and rollback. |
| Interface | Human and client surfaces: MCP `stdio`, local dashboard, support bundle, docs, future remote read-only dashboard. | MCP tools, local dashboard, support bundle, doctor, release scripts | Keep local surfaces read-only where documented; remote read-only dashboard is a future gated surface, not Local Alpha scope. |
| Release Boundary | Evidence, release decision, compatibility, platform parity, and product wording. | release gate docs, local evidence summary, release soak runner, status sync check | Treat release state as a first-class artifact; do not infer external evidence from local simulation. |

## Target Dependency Direction

The desired dependency direction is:

```text
Substrate -> Signal -> Memory -> Policy -> Control Loop -> Actuator
                 \          \          \             \
                  \          \          \             -> Interface
                   \          \          -> Release Boundary
                    -> Interface
```

Rules:

- `Actuator` must not depend on dashboard or release tooling.
- `Interface` may call policy or read projections, but write-capable interfaces
  must pass through `run_reflection` or a later approved write-path ADR.
- `Control Loop` may observe signal and policy state, but observe-only daemon
  work must not call the actuator.
- `Release Boundary` reads evidence and wording; it must not generate missing
  Windows, fresh-machine, remote/team, or human-decision evidence.
- `Memory` projections may read signal and substrate state, but future durable
  layers need migration and lifecycle gates before becoming write targets.

## Priority Plan

### Phase 0: Preserve Baseline And Review Gates

Purpose: keep current reality visible before structural changes.

Near-term tasks:

1. Keep `./scripts/status-sync-check.sh` in every planning/doc update loop.
2. Keep `doctor` as the quick runtime boundary check; when the default user DB
   may be read-only, run it with an explicit writable SQLite URL.
3. Add or maintain wording guards that block GA, production-ready, remote/team,
   remote write admin, and complete self-governance claims without matching
   gates.
4. Keep `run_reflection` documented as the only durable write path.

Verification:

```bash
git diff --check
./scripts/status-sync-check.sh
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
```

Exit condition: planning docs, status docs, and doctor output agree on current
capabilities and blockers.

### Phase 1: Read-Only Architecture Reports

Purpose: make the proposed layer model observable before changing runtime
responsibilities.

Recommended first implementation slice:

1. Add a read-only `system_layer_report` or extend `doctor` with a stable layer
   summary.
2. Report substrate, signal, memory, policy, control-loop, actuator, interface,
   and release-boundary status.
3. Mark each layer as `implemented`, `partial`, `simulation-only`,
   `planning-gate`, or `not-implemented`.
4. Include blockers for Local Alpha, Windows parity, fresh-machine evidence,
   remote/team, daemon writes, security gates, and memory layering.
5. Include dependency-rule evidence with `enforced`,
   `declared-test-contract`, or `open` status, an explicit
   `grants_capability = false` marker, evidence `source`, verification command
   for declared test contracts, and observed/expected values for daemon,
   release-boundary, interface, actuator, and memory write boundaries.

Verification:

```bash
cargo test --test product_completion_read_models -v
cargo test --test provider_config -v
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
./scripts/status-sync-check.sh
```

Exit condition: a reader can inspect one report and understand which layer owns
which responsibility without reading every roadmap, and can inspect each
dependency rule to see which local runtime state or declared test contract keeps
the boundary enforced or declared.

### Phase 2: Local Alpha Evidence Completion

Purpose: complete the user-operable local product gate without expanding into
remote/team or autonomy work.

Tasks:

1. Refresh product smoke evidence.
2. Refresh support bundle evidence.
3. Refresh first-run bootstrap evidence, while preserving the distinction
   between simulation and real fresh-machine evidence.
4. Run real fresh-machine or clean-checkout validation and record it separately
   from local simulation artifacts.
5. Run Windows runner or Windows machine validation and record platform parity
   separately from static PowerShell contract checks.
6. Generate a source-only release decision artifact for human review.

Verification:

```bash
./scripts/local-alpha-release-gate-refresh.sh
cargo run --quiet --bin local_alpha_evidence_summary -- --evidence-root .
./scripts/product-readiness-check.sh <candidate-name>
./scripts/release-decision-local.sh <candidate-name> <evidence-root>
./scripts/status-sync-check.sh
```

Exit condition: every Local Alpha gate has fresh reviewable evidence, and a
human release decision explicitly records approved, rejected, or deferred state.

Blocked until external evidence exists:

- real fresh-machine install/bootstrap evidence
- Windows runtime parity evidence
- human release decision

### Phase 3: Signal And Evidence Semantics

Purpose: improve the quality of what the system observes before increasing what
it can do.

Tasks:

1. Expand evidence queries one slice at a time while preserving no-widening
   governance for self-revision trigger windows.
2. Add relation, rank, or weight fields only when their semantics are tested and
   reflected in docs.
3. Keep provider payload and raw diagnostics out of operation logs and support
   bundles.
4. Add failure-mode tests for ambiguous, empty, stale, or conflicting evidence.

Verification:

```bash
cargo test --test evidence_query_dto -v
cargo test --test failure_modes -v
cargo test --test product_completion_read_models -v
./scripts/status-sync-check.sh
```

Exit condition: evidence can be inspected, narrowed, and rejected without
fabricating causality or broadening the trigger window silently.

### Phase 4: Memory Layering, Still Local And Gated

Purpose: move from read-only projections toward durable richer memory only after
schema and lifecycle risks are controlled.

Tasks:

1. Start with richer episodic semantics: goal, outcome, lesson, linked evidence
   IDs, and snapshot projection tests.
2. Add migration tests before any durable table change.
3. Keep semantic and procedural candidates read-only until approval, versioning,
   rollback, and evidence-link rules exist.
4. Preserve `run_reflection` for identity and commitment writes.
5. Keep slow variables and durable self-model writes behind separate gates.

Verification:

```bash
cargo test --test product_completion_read_models -v
cargo test --test sqlite_backup_restore -v
cargo test
./scripts/status-sync-check.sh
```

Exit condition: each new memory layer is evidence-linked, migration-covered,
and does not imply complete multi-layer cognition.

### Phase 5: Observe-Only Daemon Stabilization

Purpose: make future daemon work debuggable while keeping write authority
closed.

Tasks:

1. Keep daemon disabled by default.
2. Preserve `writes_allowed = false` and `remote_listener_enabled = false` in
   Local Alpha.
3. Improve diagnostics for candidate reads, cooldown, suppression, and clean
   shutdown.
4. Wire `serve` to start observe-only lifecycle behavior only when
   `[daemon].enabled = true`, then stop it after stdio service exit.
5. Prove no semantic writes occur in observe-only mode.
6. Require a separate write-capable daemon gate before any daemon can call
   `run_reflection`.

Verification:

```bash
cargo test --test daemon_config -v
cargo test --test mcp_stdio serve_starts_observe_only_daemon_when_enabled_without_semantic_writes -v
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
./scripts/status-sync-check.sh
```

Exit condition: observe-only daemon behavior is useful for diagnostics and
cannot be mistaken for background autonomy.

Blocked until later review:

- write-capable daemon loop
- daemon-triggered writes
- all-entry automatic self-reflection
- continuous self-governance claims

### Phase 6: Provider Expansion

Purpose: broaden model compatibility without turning the project into a
provider gateway.

Tasks:

1. Pick one provider at a time.
2. Add config parser support, doctor diagnostics, model adapter, redaction,
   timeout behavior, non-success behavior, malformed-response behavior, and MCP
   `stdio` path tests in the same change.
3. Keep planned-only providers rejected until the implementation lands.
4. Confirm provider work does not add a side-channel durable write path.

Verification:

```bash
cargo test --test provider_config -v
cargo test --test openai_compatible_model -v
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file -v
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openrouter_provider_from_config_file -v
cargo test --test mcp_stdio ingest_interaction_auto_reflection_uses_openrouter_provider_from_config_file -v
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
```

Exit condition: the new provider is observable, redacted, bounded, and tested
through the real MCP `stdio` path.

### Phase 7: Remote/Team/Security Foundation

Purpose: prepare remote and team mode only after local product evidence is
stable.

Tasks:

1. Keep the first remote surface read-only.
2. Require auth, authorization, audit, rate limit, tenant isolation, and rollback
   before any remote write/admin route.
3. Decide namespace-to-tenant and database isolation model before team trials.
4. Prove support bundle, dashboard, operation-log, reflection-audit, backup, and
   export paths cannot cross tenant boundaries.
5. Preserve MCP `stdio` as the local core; do not convert it into a network
   service as part of remote/team mode.

Verification:

```bash
cargo test --test product_completion_read_models -v
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
./scripts/product-readiness-check.sh <candidate-name>
./scripts/status-sync-check.sh
```

Exit condition: remote/team work has reviewed security gates before product
behavior is enabled.

Blocked until all security gates exist:

- remote write admin
- team shared memory
- support bundle upload
- multi-tenancy
- public dashboard exposure

### Phase 8: Release Engineering And Beta/GA Readiness

Purpose: make release state reproducible before stronger product claims.

Tasks:

1. Keep source-only candidate evidence until packaging and compatibility gates
   exist.
2. Add installer/binary/service-manager work only after Local Alpha evidence is
   fresh.
3. Add compatibility matrix automation after Windows/fresh-machine evidence
   flow is real.
4. Keep Beta and GA wording blocked until release, security, migration,
   lifecycle, support, and recovery gates pass.

Verification:

```bash
bash -n scripts/release-soak-local.sh
cargo test --test local_alpha_release_evidence release_soak -v
./scripts/release-soak-local.sh <candidate-name>
./scripts/product-readiness-check.sh <candidate-name>
./scripts/status-sync-check.sh
```

Exit condition: release artifacts explain what was verified, what remains open,
and which claims are still blocked.

## Recommended Implementation Order

Use this order for future work unless a newer user decision supersedes it:

1. Read-only system layer report in `doctor` or a dedicated support command.
2. Local Alpha evidence refresh and real evidence collection.
3. Evidence semantics hardening.
4. Richer episodic semantics with migration tests.
5. Observe-only daemon diagnostic stabilization.
6. One provider adapter expansion, if needed.
7. Remote/team security design review before any remote route work.
8. Release packaging and compatibility matrix automation.

This order follows the energy-budget rule: prefer work that clarifies the
system and reduces claim risk before work that increases write power,
distribution scope, or operational surface area.

## Spec Review Checklist

Before implementation starts, review this document for:

- status labels match current evidence
- no Local Alpha completion claim without fresh gates
- no real fresh-machine claim based on simulation
- no Windows parity claim based on macOS or static script checks
- no remote/team implementation claim
- no auth/authz/audit/rate-limit/tenant/rollback implementation claim
- no daemon write or background autonomy claim
- no complete self-governing agent claim
- no all-entry automatic self-reflection claim
- no complete multi-layer memory claim
- `run_reflection` remains the durable identity / commitment / reflection write
  path

## Success Criteria For This Planning Round

This planning round is complete when:

- this Markdown spec is committed or ready for review as a planning artifact
- `git diff --check` passes
- `./scripts/status-sync-check.sh` or
  `cargo run --quiet --bin status_sync_check` passes
- the document keeps MVP / technical demo wording conservative
- implementation work remains deferred to separate reviewed slices
