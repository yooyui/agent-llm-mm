# Release Engineering

This document defines the release engineering rules for the productization
track. It complements the MVP release gate in [`../release-gate.md`](../release-gate.md)
and the Local Alpha gate in [`release-gate-local-alpha.md`](release-gate-local-alpha.md).

The current repository remains a validated local MVP entering productization.
This document does not certify Beta, GA, production self-governance, remote
write admin, remote team service, or multi-tenancy.

## Release Artifact Shape

The first productization-stage release artifact must use the most conservative
shape that is already maintainable:

- a signed or clearly named source-only Git tag, plus repository source at that
  tag
- release notes that point to the exact evidence directory for that candidate
- no binary packaging, installer, service manager, auto-updater, remote
  bootstrapper, or packaging automation claim unless a later workstream adds and
  gates that capability

The source-only artifact is intentional. It keeps the first release auditable
from source and avoids implying a packaged product surface before install,
upgrade, rollback, and platform support are implemented and verified.

## Version Naming

Use conservative pre-release names until the relevant product gate has fresh
evidence.

Recommended shape:

- `local-mvp-YYYYMMDD.N` for MVP-oriented source snapshots
- `local-alpha-YYYYMMDD.N` only after the Local Alpha gate has fresh evidence
- append `-rc.N` for release candidates that still need review

Do not use names that imply general availability, production support, remote
administration, multi-tenant service, or hosted operation. In particular, avoid
`v1.0`, `stable`, `production`, or `ga` naming until a separate gate explicitly
allows that claim.

## Changelog Rules

Every release candidate needs a changelog entry or release note that is short,
evidence-backed, and boundary-aware.

Required sections:

- `Added`: new user-visible capability or documentation surface
- `Changed`: behavior, command, config, or workflow changes
- `Fixed`: user-visible defects or verification regressions
- `Evidence`: commands run, evidence directory path, and date
- `Boundaries`: explicit non-claims such as no remote write admin, no
  production self-governance, no multi-tenancy, and no GA readiness

Do not describe an experimental path as complete product behavior. If a feature
is observe-only, local-only, disabled by default, or demo-only, say so in the
changelog.

## Release Evidence Directory Rules

Each release candidate must have one evidence directory generated from the same
candidate commit or tag.

Recommended path shape:

```text
target/reports/releases/<candidate-name>/
```

Required contents:

- command transcript or summary for the applicable release gate
- `git rev-parse HEAD` output or equivalent commit identity
- `git status --short --branch` output captured before the release decision
- product smoke output when Local Alpha claims are being considered
- self-revision demo artifacts when the release notes mention that evidence
- compatibility matrix result for supported local platforms
- soak test command result when the candidate changes runtime, persistence,
  dashboard, daemon, provider, or MCP behavior

Do not reuse a stale `latest` directory as release evidence. If `latest` is
used by a script, copy or summarize its contents into the candidate-specific
release evidence directory after the successful run.

## Compatibility Matrix Requirements

A release candidate must state which local platform and runtime combinations
were actually checked.

Minimum matrix fields:

- platform: macOS, Windows, or other
- shell / wrapper: `scripts/agent-llm-mm.sh` or `scripts/agent-llm-mm.ps1`
- Rust toolchain version
- SQLite persistence path shape, not a private absolute path
- provider mode checked: deterministic demo, mock, or configured
  `openai-compatible`
- dashboard status: disabled, local-only enabled, or not checked
- daemon status: disabled, observe-only checked, or not checked
- result: passed, failed, blocked, or not applicable

Do not infer platform support from another platform. If Windows was not checked,
record it as not checked instead of claiming parity.

## Soak Test Command Requirements

When a candidate changes runtime behavior, persistence, dashboard behavior,
daemon behavior, provider behavior, or MCP tool handling, release evidence must
include a bounded local soak command.

The current local runner is:

```bash
./scripts/release-soak-local.sh <candidate-name> [config_path]
```

It writes candidate-specific evidence under
`target/reports/releases/<candidate-name>/` and runs:

- `./scripts/agent-llm-mm.sh doctor [config_path]`
- `cargo test --test dashboard_http -v`
- `scripts/product-smoke-local.sh [config_path]`
- `scripts/first-run-bootstrap-smoke-local.sh target/first-run-bootstrap-smoke/local-alpha-gate`
- `scripts/generate-support-bundle.sh target/support-bundles/local-alpha-gate [config_path]`
- support-bundle secret and raw-artifact scans
- support-bundle and product-smoke SHA-256 manifests
- `scripts/local-alpha-evidence-summary.sh` into the release evidence directory

The evidence records:

- exact commands run
- exit codes
- start and end time or elapsed duration
- git HEAD and before / after working tree status
- config path shape, without embedding secrets
- support bundle file list, product smoke latest file list, their SHA-256 manifests, and Local Alpha gate summary
- any sandbox-only blocker separately from code failures

This runner creates local soak evidence only. It does not create a source tag,
binary package, installer, service manager, auto-updater, Windows runner
evidence, real fresh-machine evidence, remote/team evidence, upload, release
decision, or Local Alpha certification.

## Deprecation Policy

Deprecations must be explicit, documented, and conservative.

Required deprecation entry:

- deprecated behavior, command, config key, or document claim
- replacement path
- first release candidate where the deprecation is announced
- earliest release candidate where removal may happen
- migration or verification command, if applicable
- rollback note when data or configuration is affected

Do not remove a documented command, config key, data shape, or public claim in
the same release candidate where it is first deprecated unless the existing
behavior is unsafe and the release note explains the reason.

Deprecation must not create a hidden remote write path, remote admin surface, or
new durable identity / commitment write path. `run_reflection` remains the only
durable identity / commitments / reflection write path until a later gate
explicitly changes that boundary.

## Release Decision Boundary

Before publishing any release note, verify that public wording still matches
the implemented evidence:

- source-only release artifacts are allowed for the first phase
- Local Alpha wording requires fresh Local Alpha gate evidence
- Beta, GA, production-ready, remote write admin, remote team service,
  multi-tenancy, and complete self-governance claims are blocked unless a later
  gate explicitly approves them
- release engineering evidence records what was checked, what was not checked,
  and what remains out of scope
