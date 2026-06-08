# Non-MVP Product Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add four conservative, code-backed non-MVP productization slices: release evidence indexing, provider live-certification preflight, richer memory semantics projection, and packaging preflight.

**Architecture:** Each slice is a read-only projection or preflight over existing local evidence, config, and product-readiness gates. The work improves product maturity without granting remote/team writes, daemon writes, provider live certification, packaging certification, Beta, GA, or production-ready claims.

**Tech Stack:** Rust, serde JSON/Markdown summaries, existing local evidence helpers, shell wrappers, integration tests.

---

## Boundary Rules

- `run_reflection` remains the only durable identity / commitment / reflection write path.
- New summaries are read-only and must not start `serve`, run remote commands, upload files, call provider APIs, create release tags, or generate installers.
- Provider certification preflight must redact API keys, URL userinfo, URL path content, and query secrets.
- Provider live evidence files must be non-empty JSON with matching `provider`, `status = "passed"`, expected `evidence_kind`, `mode = "live"`, non-empty `generated_at`, `local_only = false`, live provenance fields, and successful command evidence before they can be reported as present.
- Packaging preflight must keep installer, service manager, auto-updater, and binary packaging blocked until real complete artifacts exist; zero-byte or partial binary archives are not enough.
- Documentation must keep the repository positioned as local-first productization in progress, not Beta/GA/production-ready.

## Existing RED Contract

- [x] **Step 1: Add contract tests**

```bash
cargo test --test non_mvp_product_tracks -v
```

Expected current RED result: compile failure because `release_evidence_index`, `provider_certification`, `memory_semantics_projection`, and `packaging_preflight` do not exist.

## Task 1: Release Evidence Index

**Files:**
- Create: `src/support/release_evidence_index.rs`
- Create: `src/bin/release_evidence_index.rs`
- Create: `scripts/release-evidence-index.sh`
- Modify after integration: `src/support/mod.rs`
- Test: `tests/non_mvp_product_tracks.rs`

- [x] **Step 1: Implement read-only release evidence index**
  - Add `ReleaseEvidenceIndexOptions`, `ReleaseEvidenceIndex`, `ReleaseEvidenceIndexEntry`, and `build_release_evidence_index`.
  - Reuse `summarize_local_alpha_evidence` and `summarize_product_readiness`.
  - Convert satisfied local alpha gates to `present`, open product gates to `missing`, not-verified gates to `not_verified`, and blocked future capability gates to `blocked`.
  - Include `kind = "release_evidence_index"`, `local_only = true`, `ready_for_human_review`, `missing_required_count`, `blocked_gate_count`, `non_claims`, and Markdown.

- [x] **Step 2: Add CLI/script wrapper**
  - CLI accepts `--release-candidate`, `--evidence-root`, `--output-json`, and `--output-md`.
  - Shell wrapper rejects unsafe candidate names before calling cargo.

- [x] **Step 3: Verify focused test**

```bash
cargo test --test non_mvp_product_tracks release_evidence_index -v
```

## Task 2: Provider Certification Preflight

**Files:**
- Create: `src/support/provider_certification.rs`
- Create: `src/bin/provider_certification_check.rs`
- Create: `scripts/provider-certification-check.sh`
- Modify after integration: `src/support/mod.rs`
- Test: `tests/non_mvp_product_tracks.rs`

- [x] **Step 1: Implement read-only provider certification summary**
  - Add `ProviderCertificationOptions`, `ProviderCertificationSummary`, and `summarize_provider_certification`.
  - Validate config using existing `AppConfig::validate`.
  - Mark `config_preflight_status = "passed"` for valid selected providers.
  - Keep `live_certified = false` and `live_certification_status = "blocked"` unless future live evidence support is explicitly added.
  - Emit missing live evidence items: `live_decision_path`, `live_self_revision_path`, `provider_error_handling`, and `redaction_review`.
  - Redact provider URL to shape only and never serialize API keys or URL secrets.

- [x] **Step 2: Add CLI/script wrapper**
  - CLI reads optional config path or defaults to `AppConfig::load`.
  - Shell wrapper must not call `curl`, `ssh`, `scp`, `rsync`, or provider endpoints.

- [x] **Step 3: Verify focused test**

```bash
cargo test --test non_mvp_product_tracks provider_certification -v
```

## Task 3: Richer Memory Semantics Projection

**Files:**
- Create: `src/domain/memory_semantics_projection.rs`
- Modify after integration: `src/domain/mod.rs`
- Test: `tests/non_mvp_product_tracks.rs`

- [x] **Step 1: Implement read-only semantic projection**
  - Add `MemorySemanticsProjectionInput`, `MemorySemanticsProjection`, `MemorySemanticCapability`, and `build_memory_semantics_projection`.
  - Report `evidence_relations`, `episode_summaries`, `semantic_claims`, `procedural_memory`, and `durable_self_model_writes`.
  - Use statuses `partial`, `not_implemented`, or `blocked`.
  - Always set `read_only = true`, `writes_performed = false`, `durable_self_model_write_path = "run_reflection"`, and `writes_allowed = false`.

- [x] **Step 2: Verify focused test**

```bash
cargo test --test non_mvp_product_tracks memory_semantics -v
```

## Task 4: Packaging Preflight

**Files:**
- Create: `src/support/packaging_preflight.rs`
- Create: `src/bin/packaging_preflight_check.rs`
- Create: `scripts/packaging-preflight-check.sh`
- Modify after integration: `src/support/mod.rs`
- Test: `tests/non_mvp_product_tracks.rs`

- [x] **Step 1: Implement read-only packaging preflight**
  - Add `PackagingPreflightOptions`, `PackagingPreflightSummary`, `PackagingPreflightBlocker`, and `summarize_packaging_preflight`.
  - Validate release candidate names with `validate_release_candidate`.
  - Check for source-only release soak artifacts under `target/reports/releases/<candidate>/`.
  - Keep `packaging_ready = false` unless real binary archive, installer, service manager, and auto-updater evidence exist.
  - Include blockers for `binary_archive`, `installer`, `service_manager`, and `auto_updater`.

- [x] **Step 2: Add CLI/script wrapper**
  - CLI accepts `--release-candidate`, `--evidence-root`, `--output-json`, and `--output-md`.
  - Shell wrapper rejects unsafe candidate names and must not create tags, installers, uploads, or remote calls.

- [x] **Step 3: Verify focused test**

```bash
cargo test --test non_mvp_product_tracks packaging_preflight -v
```

## Task 5: Integration And Documentation

**Files:**
- Modify: `src/support/mod.rs`
- Modify: `src/domain/mod.rs`
- Modify: `README.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `docs/development-macos.md`
- Modify if needed: `docs/product/follow-up-reality-gates.md`

- [x] **Step 1: Wire modules and run contract test**

```bash
cargo test --test non_mvp_product_tracks -v
```

- [x] **Step 2: Update docs conservatively**
  - Describe the four new read-only/preflight capabilities.
  - Keep blocked non-claims for Local Alpha completion, Windows parity, remote/team, daemon writes, live provider certification, binary packaging, Beta, GA, and production-ready.

- [x] **Step 3: Final verification**

```bash
cargo fmt --check
git diff --check
bash -n scripts/release-evidence-index.sh
bash -n scripts/provider-certification-check.sh
bash -n scripts/packaging-preflight-check.sh
cargo test --test non_mvp_product_tracks -v
cargo test --test product_readiness -v
cargo test --test provider_config -v
cargo test --test product_completion_read_models -v
cargo test --test local_alpha_release_evidence release_soak -v
cargo test
./scripts/agent-llm-mm.sh doctor
```

Final verification was rerun on 2026-06-08 after the provider URL path-redaction
consistency fix, including `cargo test --test provider_config -v`,
`cargo test --test support_bundle -v`, `cargo test --test non_mvp_product_tracks
-v`, `./scripts/status-sync-check.sh`, full `cargo test`, doctor smoke with an
isolated SQLite URL, and the three new preflight script smokes. The reviewer
agent also completed a read-only review with no findings.
