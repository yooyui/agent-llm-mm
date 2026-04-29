# Provider Readiness Checklist

## 0. Scope

- This repository remains a local Rust MCP `stdio` technical demo / MVP, not a production provider gateway.
- This checklist defines the readiness boundary before adding a new provider. It does not add a provider implementation by itself.
- A new provider should match the minimum observable behavior already expected from `mock` and `openai-compatible`: deterministic config loading, bounded network behavior, explicit parse errors, and preserved MCP `stdio` behavior.
- Items are marked `existing`, `partial`, or `gap` against the current test suite. For a new provider, every applicable item must either be `existing` or gain a provider-specific regression in the same change. `partial` and `gap` items are blockers for broadening provider support until the missing regressions are added or an explicit documented exception is accepted for that provider.

## 1. Provider Checklist

| Item | Required behavior | Current verification status | Current signals |
| --- | --- | --- | --- |
| Config validation behavior | Config loading must preserve provider selection, provider-specific fields, config-file precedence, and required-field validation. Missing required provider fields must fail before normal runtime use. | existing | `tests/provider_config.rs` covers default `mock`, explicit `openai-compatible` config loading, `AGENT_LLM_MM_CONFIG`, `AGENT_LLM_MM_DATABASE_URL`, and missing `api_key` failure through `doctor`. |
| `doctor` redaction behavior | `doctor` may expose provider, base URL, model, status, and runtime readiness, but must not expose API keys or equivalent secrets. | partial | `docs/project-status.md` documents the boundary and `tests/provider_config.rs` covers missing key failure. There is not yet a direct positive assertion that a configured secret is absent from the serialized report. |
| Timeout handling | Provider network calls must use bounded timeout configuration and surface timeout failures as provider errors, not hangs or silent fallback. | partial | `timeout_ms` is parsed in `tests/provider_config.rs` and used by `OpenAiCompatibleModel`; there is not yet a dedicated timeout-failure regression. |
| Non-success HTTP status behavior | Non-2xx provider responses must return observable provider errors. | existing | `tests/openai_compatible_model.rs::openai_compatible_model_surfaces_non_success_status`. |
| Malformed JSON behavior | Malformed or schema-incompatible model responses must fail without panic and without fabricating a valid decision or self-revision proposal. | partial | `tests/openai_compatible_model.rs` covers empty decision action, structured proposal parsing, missing `machine_patch` defaults, and fenced JSON proposals. Add explicit malformed JSON cases before broadening providers. |
| Decision action parsing | The first assistant message content is the action; blank content is rejected. | existing | `openai_compatible_model_parses_first_assistant_message_into_action` and `openai_compatible_model_rejects_empty_action`. |
| Self-revision proposal parsing | `should_reflect`, `rationale`, `machine_patch`, and defaulted patch fields must parse consistently, including fenced JSON. | existing | `openai_compatible_model_parses_self_revision_proposal_from_assistant_message`, `openai_compatible_model_defaults_missing_machine_patch_in_self_revision_proposal`, and `openai_compatible_model_accepts_fenced_json_self_revision_proposal`. |
| Evidence policy parsing | `proposed_evidence_event_ids`, `proposed_evidence_query`, and `confidence` must parse into the structured self-revision proposal contract. | existing | `openai_compatible_model_parses_self_revision_evidence_policy`. |
| MCP `stdio` provider path | Config-selected provider behavior must flow through the real MCP `stdio` path without corrupting protocol output. | existing | `tests/mcp_stdio.rs::decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file`. |

## 2. Current Coverage Map

`tests/provider_config.rs`:

- `default_config_uses_mock_provider_when_no_config_file_is_present`
- `load_from_path_reads_openai_compatible_provider_from_toml_file`
- `load_prefers_config_path_from_environment`
- `load_prefers_database_url_env_over_default_config_file`
- `doctor_fails_when_openai_provider_config_is_missing_api_key`

`tests/openai_compatible_model.rs`:

- `openai_compatible_model_parses_first_assistant_message_into_action`
- `openai_compatible_model_rejects_empty_action`
- `openai_compatible_model_surfaces_non_success_status`
- `openai_compatible_model_parses_self_revision_proposal_from_assistant_message`
- `openai_compatible_model_defaults_missing_machine_patch_in_self_revision_proposal`
- `openai_compatible_model_accepts_fenced_json_self_revision_proposal`
- `openai_compatible_model_parses_self_revision_evidence_policy`

`tests/mcp_stdio.rs`:

- `decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file`

## 3. Required Validation Commands

Run these before treating a new provider as ready for the MVP track:

```zsh
cargo test --test provider_config -v
cargo test --test openai_compatible_model -v
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file -v
```

If the new provider adds provider-specific parsing or transport behavior, add provider-specific regressions alongside these commands rather than weakening the shared contract.

## 4. Blocking Gaps Before More Providers

- Add a positive `doctor` redaction regression that proves configured secrets are absent from the serialized report and user-facing diagnostics.
- Add a timeout-failure regression for provider calls.
- Add explicit malformed JSON regressions for decision and self-revision proposal responses.
- Keep `run_reflection` as the only durable self-revision write path; provider work must not introduce a side-channel durable write path.

## 5. Non-Goals

- This checklist does not add Azure OpenAI, OpenRouter, local gateway, or any other provider.
- This checklist does not turn the project into a remote provider service or production credential manager.
- This checklist does not change the MCP tool contract, dashboard boundary, or automatic self-revision runtime hooks.
