# Self-Agent MCP 测试指南（2026-03-24，按 2026-06-08 fresh 验证更新）

## 1. 目标

这份文档说明当前仓库应如何测试，覆盖：

- 代码格式与静态检查
- 自动化测试套件
- `namespace` / SQLite 迁移 / MCP `stdio` 的定向验证
- `doctor` 预检
- 手工 smoke test 的推荐方式

如果目标是判断“是否可以按当前 demo / MVP 口径发布”，请同时阅读 [Release Gate](release-gate.md)。本页偏向测试与回归，`release-gate.md` 负责收口发布前的最低 gate、self-revision 证据 gate、dashboard gate 和 sandbox 失败解释口径。

如果目标是判断“是否可以进入 Local Product Alpha / product alpha 口径”，请使用 [Local Alpha Release Gate](product/release-gate-local-alpha.md) 作为入口。本测试指南只保留参考入口，不复制产品 gate 的长清单；Local Alpha gate 会额外检查 product smoke evidence、dashboard local-only 边界、daemon disabled / observe-only 边界，以及 remote write / multi-tenancy 等产品文案限制。

当前工作目录（按实际环境替换）：

`~/code/agent-llm-mm`

命令执行环境要求：

- 安装 Rust toolchain
- `cargo` 可用
- `bash` 或 `zsh`（用于 `scripts/agent-llm-mm.sh`）

---

## 2. 当前测试基线

截至 `2026-06-08`，`cargo test -- --list --format terse` 当前枚举测试摘要如下：

- `lib unit tests`: 9 passed
- `application_use_cases`: 25 passed
- `bootstrap`: 24 passed
- `daemon_config`: 12 passed
- `dashboard_config`: 4 passed
- `dashboard_http`: 7 passed
- `dashboard_projection`: 2 passed
- `dashboard_recorder`: 2 passed
- `decision_flow`: 2 passed
- `demo_openai_compatible_stub`: 1 passed
- `domain_invariants`: 4 passed
- `domain_snapshot`: 6 passed
- `evidence_query_dto`: 4 passed
- `failure_modes`: 36 passed
- `first_run_bootstrap_smoke`: 4 passed
- `local_alpha_release_evidence`: 20 passed
- `mcp_stdio`: 44 passed
- `non_mvp_product_tracks`: 7 passed
- `openai_compatible_model`: 11 passed
- `operation_log`: 9 passed
- `product_completion_read_models`: 15 passed
- `product_readiness`: 15 passed
- `provider_config`: 17 passed
- `release_decision`: 5 passed
- `self_revision_demo_runner`: 2 passed
- `sqlite_backup_restore`: 6 passed
- `sqlite_store`: 23 passed
- `status_sync`: 11 passed
- `support_bundle`: 35 passed

合计：362 个测试通过。

---

## 3. 测试前准备

### 3.1 环境要求

- 安装 Rust toolchain
- 可用的 `cargo`
- `bash` 或 `zsh`：用于 `scripts/agent-llm-mm.sh`

### 3.2 建议进入工作目录

```zsh
cd ~/code/agent-llm-mm
```

### 3.3 数据库隔离建议

未显式设置 `database_url` 时，服务会把默认 SQLite 文件放到当前平台的用户数据目录，并按“本机用户共享默认库”语义复用。为了避免和已有运行实例互相污染，手工测试时建议显式设置：

```zsh
cp examples/agent-llm-mm.example.toml agent-llm-mm.local.toml
```

然后修改 `agent-llm-mm.local.toml` 里的：

- `database_url`
- `provider`
- provider-specific 配置

如果只是跑现有自动化测试，不需要手工设置；测试本身已经为大多数场景隔离了数据库。若运行环境不能写入默认用户数据目录（例如受限沙箱或只读 home 目录），`doctor` 相关测试和命令会因为默认 SQLite 路径不可写而失败；此时应通过 `AGENT_LLM_MM_DATABASE_URL` 或本地 TOML 指向一个可写的 SQLite 文件。正式接入、手工测试和实验验证仍建议各自使用不同数据库文件。

---

## 4. 推荐测试顺序

建议按下面顺序执行：

1. `cargo fmt --check`
2. `git diff --check`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `cargo test`
5. `AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor`
6. `AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor-cargo.sqlite cargo run --quiet --bin agent_llm_mm -- doctor`
7. 如果改动涉及 automatic self-revision MVP，再补跑本指南里的 runtime coverage / diagnostics / evidence policy 定向验证
8. 如果改动涉及 demo package，先用 timestamped / scratch output 跑 `./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/manual-$(date +%Y%m%d-%H%M%S)`；如果要按 Local Alpha 发布口径复核 `latest` 证据链，使用下一条 product smoke
9. 如果改动涉及 Local Alpha product smoke gate、启动包装脚本或本地产品化证据链，在 repo root 补跑 `./scripts/product-smoke-local.sh [config_path]`；如果当前目录不是 repo root，使用 `/path/to/agent-llm-mm/scripts/product-smoke-local.sh`，并在需要配置文件时传入绝对 config path
10. 如果改动涉及 bootstrap wrapper，确认脚本契约仍是 `[serve|doctor|bootstrap-local] [config_path]`，unsupported mode 返回 exit code `2`，`bootstrap-local` 不覆盖已有配置、不生成 secret、不运行 `doctor` 或 `serve`，相对目标路径按仓库根目录解析，输出的下一步命令能处理含空格路径，并补跑 `cargo test --test bootstrap -v`
11. 如果改动涉及 first-run bootstrap smoke、本地首启证据或 `bootstrap-local -> doctor` 产品化路径，补跑 `bash -n scripts/first-run-bootstrap-smoke-local.sh` 和 `cargo test --test first_run_bootstrap_smoke -v`
12. 如果改动涉及 Local Alpha evidence summary、发布证据汇总或 gate status 输出，补跑 `bash -n scripts/local-alpha-evidence-summary.sh`、`cargo test --test local_alpha_release_evidence -v`，并用 `cargo run --quiet --bin local_alpha_evidence_summary -- --evidence-root .` spot-check JSON 输出；该 summary 只是本地只读 gate 状态汇总，不是自动认证
13. 如果改动涉及 Local Alpha release-gate refresh 或本机 gate 证据刷新流程，补跑 `bash -n scripts/local-alpha-release-gate-refresh.sh`、`cargo test --test local_alpha_release_evidence -v`，并按需执行 `./scripts/local-alpha-release-gate-refresh.sh [config_path]`；该 refresh 只产生本机可复现证据，不生成真实 fresh-machine、Windows runner、remote/team 或发布决策证据
14. 如果改动涉及 release engineering、release evidence directory、soak evidence 或候选发布说明，补跑 `bash -n scripts/release-soak-local.sh`、`cargo test --test local_alpha_release_evidence release_soak -v`，并按需执行 `./scripts/release-soak-local.sh <candidate-name> [config_path]`；该 soak 只生成本地 release evidence，不生成真实 fresh-machine、Windows runner、remote/team、上传、tag、安装包或发布认证证据
15. 如果改动涉及 SQLite 备份、恢复、schema migration 前置检查或 data lifecycle gate，补跑 `bash -n scripts/backup-sqlite.sh scripts/restore-sqlite.sh` 和 `cargo test --test sqlite_backup_restore -v`
16. 如果改动涉及 product readiness、release decision artifact、产品措辞 gate、remote/team inventory/security gates、evidence relation、episode projection、layered memory projection 或 `doctor.system_layer_report`，补跑 `cargo test --test product_readiness -v`、`cargo test --test release_decision -v`、`cargo test --test product_completion_read_models -v`、`cargo test --test provider_config -v` 和 `./scripts/product-readiness-check.sh <candidate-name>` 的本地预检；这些检查只能核验本地门禁、doctor 只读架构层报告、runtime / declared-test-contract dependency-rule evidence、physics-informed non-claim / wording guard 和只读投影，不生成真实 fresh-machine、Windows runner、remote/team 产品模式、GA 或发布认证证据
17. 如果改动涉及 release evidence index、provider certification preflight、packaging preflight 或 richer memory semantics projection，补跑 `cargo test --test non_mvp_product_tracks -v`、`bash -n scripts/release-evidence-index.sh scripts/provider-certification-check.sh scripts/packaging-preflight-check.sh`，并按需执行对应脚本；这些 preflight 只读取本地 evidence/config shape，不调用 provider endpoint、不生成 installer、不上传文件、不认证 live provider、Local Alpha、Beta、GA 或 production-ready；provider live evidence 需要非空 JSON、匹配 provider 且 `status = "passed"`，packaging archive evidence 需要预期 archive 全部存在且非空

如果当前机器没有 `pwsh`，PowerShell runtime 行为测试会跳过；这种情况下只代表 Rust 测试覆盖了 PowerShell 脚本文本契约和 no-clobber 静态断言，Windows runner 或 Windows 实机验证仍需单独记录。

如果只想快速回归某个变更，再执行对应的定向测试。若需要一份面向发布前核验的固定检查单，demo / MVP 发布直接使用 [Release Gate](release-gate.md)；Local Alpha / product alpha 发布使用 [Local Alpha Release Gate](product/release-gate-local-alpha.md)。

---

## 5. 全量验证

### 5.1 格式检查

```zsh
cargo fmt --check
```

通过标准：

- 命令退出码为 `0`
- 没有 diff 输出

### 5.2 补丁空白检查

```zsh
git diff --check
```

通过标准：

- 命令退出码为 `0`
- 没有 whitespace error、冲突标记或补丁格式问题

### 5.3 静态检查

```zsh
cargo clippy --all-targets --all-features -- -D warnings
```

通过标准：

- 命令退出码为 `0`
- 没有 warning

### 5.4 全量测试

```zsh
cargo test
```

重点覆盖：

- domain invariants
- application use cases
- SQLite adapter
- MCP `stdio` E2E
- failure modes
- 启动与配置基线

通过标准：

- 所有测试通过
- 没有失败、panic 或 `UnexpectedEof`

### 5.5 本机预检

```zsh
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
```

```zsh
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor-cargo.sqlite cargo run --quiet --bin agent_llm_mm -- doctor
```

预期输出为 JSON，至少包含：

- `transport`
- `database_url`
- `provider`
- `status`

通过标准：

- `status` 为 `ok`
- 未出现 bootstrap 或 runtime 初始化错误

---

## 6. 定向测试

### 6.1 `namespace` / SQLite 约束

```zsh
cargo test --test sqlite_store
```

重点覆盖：

- `events.namespace` / `claims.namespace` 持久化
- legacy schema 迁移
- `owner <-> namespace` 数据库级 `CHECK` 约束
- adapter 写入/读取兜底
- owner/namespace SQL 规则是否保持单一来源
- evidence query 是否按 namespace 先过滤、再排序和限流

特别关注这些测试名：

- `sqlite_store_bootstraps_all_tables`
- `sqlite_query_evidence_event_ids_filters_by_namespace_before_limit`
- `sqlite_query_evidence_event_ids_filters_by_kind_only`
- `sqlite_query_evidence_event_ids_rejects_zero_limit`
- `sqlite_bootstrap_backfills_namespace_for_legacy_event_rows`
- `sqlite_bootstrap_backfills_namespace_for_legacy_claim_rows`
- `sqlite_store_rejects_owner_namespace_mismatch_on_write`
- `sqlite_database_rejects_corrupt_namespace_owner_pair_before_read`
- `sqlite_owner_namespace_sql_rules_have_single_source`

适用场景：

- 修改了 `src/adapters/sqlite/schema.rs`
- 修改了 `src/adapters/sqlite/store.rs`
- 修改了 `Namespace` / `ClaimDraft` 相关规则

### 6.2 SQLite backup / restore 脚本门禁

```zsh
bash -n scripts/backup-sqlite.sh scripts/restore-sqlite.sh
cargo test --test sqlite_backup_restore -v
```

这个门禁执行 bash 脚本；没有 `bash` 的平台会按现有脚本测试模式跳过 Rust 测试里的脚本调用。Windows 上需要 Git Bash、WSL 或等价 bash 环境来实际验证脚本行为。

重点覆盖：

- `backup-sqlite.sh` 生成本地备份后，`restore-sqlite.sh` 可以恢复到新的 SQLite 路径并保留数据内容
- restore 拒绝覆盖已有目标文件
- backup 拒绝把备份目录放到 live database 目录树内
- backup / restore 拒绝 in-memory SQLite 文件语义
- SQLite file URL 的 invalid percent encoding 会被拒绝
- restore 目标路径里的 `..` 组件会被拒绝，避免恢复写入绕过调用方指定的新路径边界

该门禁仍是本地脚本回归，不代表远程备份、云同步、定时 daemon、生产灾备、admin/auth 或团队模式能力。

### 6.3 MCP `stdio` 端到端

```zsh
cargo test --test mcp_stdio
```

重点覆盖：

- 工具是否正确暴露
- `ingest_interaction -> build_self_snapshot` 是否共享 runtime 状态
- `run_reflection` 是否影响 active snapshot
- 配置文件指定 provider 后是否真的走到对应 provider
- `run_reflection` 的显式 evidence 输入是否允许 inferred replacement
- `run_reflection` 的 query-based evidence 输入是否被正确校验
- `run_reflection` 的 `identity_update` / `commitment_updates` 是否真正落盘并反映到后续 snapshot
- `run_reflection` 的 `identity_update` / `commitment_updates` 缺少 resolved evidence 时是否返回 `invalid_params`
- baseline commitment 是否阻断 forbidden action
- 非法 namespace 是否返回 `-32602 invalid_params`

特别关注这些测试名：

- `server_exposes_expected_tools_over_stdio`
- `stdio_tools_share_runtime_state_across_calls`
- `decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file`
- `conflicting_reflection_over_stdio_removes_claim_from_active_snapshot`
- `inferred_replacement_reflection_with_evidence_is_accepted_over_stdio`
- `reflected_claim_replacement_query_is_accepted_over_stdio`
- `reflection_identity_and_commitment_updates_are_applied_and_audited_over_stdio`
- `reflection_identity_or_commitment_updates_require_evidence_over_stdio`
- `replacement_evidence_query_limit_overflow_is_invalid_params_over_stdio`
- `fresh_stdio_runtime_blocks_forbidden_action_with_seeded_commitment`
- `invalid_namespace_is_reported_as_invalid_params_over_stdio`

适用场景：

- 修改了 `src/interfaces/mcp/dto.rs`
- 修改了 `src/interfaces/mcp/server.rs`
- 修改了应用层输入校验或错误映射

### 6.4 Provider 合规预检

新增 provider 前先阅读 [Provider Readiness Checklist](provider-contract.md)。下面这组命令只是当前共享 provider 路径的最小验证；如果 checklist 里仍有 `partial` 或 `gap` 且新 provider 依赖该行为，新增 provider 的同一变更必须补齐对应专用回归或记录明确例外。

```zsh
cargo test --test provider_config -v
cargo test --test openai_compatible_model -v
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file -v
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openrouter_provider_from_config_file -v
cargo test --test mcp_stdio ingest_interaction_auto_reflection_uses_openrouter_provider_from_config_file -v
cargo test --test support_bundle support_bundle_reports_openrouter_config_shape_without_provider_secrets -v
```

重点覆盖：

- config validation behavior
- `doctor` redaction behavior
- timeout handling
- non-success HTTP status behavior
- malformed JSON behavior
- decision action parsing
- self-revision proposal parsing
- evidence policy parsing
- config-selected provider 是否真实走到 MCP `stdio` provider path

当前覆盖映射：

- `tests/provider_config.rs`
  - `default_config_uses_mock_provider_when_no_config_file_is_present`
  - `load_from_path_reads_openai_compatible_provider_from_toml_file`
  - `load_from_path_reads_openrouter_provider_from_toml_file`
  - `load_prefers_config_path_from_environment`
  - `load_prefers_database_url_env_over_default_config_file`
  - `openrouter_example_config_parses_without_live_looking_secret`
  - `doctor_fails_when_openai_provider_config_is_missing_api_key`
  - `doctor_reports_openrouter_provider_without_exposing_api_key`
  - `doctor_fails_when_openrouter_provider_config_is_missing_model`
  - `doctor_report_does_not_contain_api_key_in_serialized_output`
- `tests/openai_compatible_model.rs`
  - `openai_compatible_model_parses_first_assistant_message_into_action`
  - `openai_compatible_model_rejects_empty_action`
  - `openai_compatible_model_surfaces_non_success_status`
  - `openai_compatible_model_parses_self_revision_proposal_from_assistant_message`
  - `openai_compatible_model_defaults_missing_machine_patch_in_self_revision_proposal`
  - `openai_compatible_model_accepts_fenced_json_self_revision_proposal`
  - `openai_compatible_model_parses_self_revision_evidence_policy`
  - `openai_compatible_model_fails_gracefully_on_malformed_json_response`
  - `openai_compatible_model_surfaces_timeout_as_error`
- `tests/evidence_query_dto.rs`
  - `evidence_query_dto_parses_recency_window_fields`
  - `evidence_query_dto_rejects_invalid_recency_timestamp`
  - `evidence_query_dto_rejects_zero_limit`
- `tests/operation_log.rs`
  - `operation_log_redacts_summary_json_before_persisting`
  - `operation_log_queries_by_correlation_id`
- `tests/mcp_stdio.rs`
  - `decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file`
  - `decide_with_snapshot_over_stdio_uses_openrouter_provider_from_config_file`
  - `ingest_interaction_auto_reflection_uses_openrouter_provider_from_config_file`

未来新增 provider 前的阻断缺口：

- self-revision proposal 的 malformed JSON 仍只通过 proposal 解析路径间接覆盖；新增 provider 前需要按 provider contract 补齐更明确的 self-revision proposal 错误断言

### 6.5 领域不变量

```zsh
cargo test --test domain_invariants --test domain_snapshot
```

重点覆盖：

- inferred claim 的证据门槛
- `identity_core` 不能被普通 ingest 直接改写
- namespace 默认派生和 owner 匹配
- snapshot evidence 预算与 gate 行为

### 6.6 应用层编排

```zsh
cargo test --test application_use_cases --test failure_modes
```

重点覆盖：

- ingest 事务顺序
- reflection 状态流转
- inferred replacement 在有显式 evidence 时可通过
- query-based evidence 会被去重、限流并做上限校验
- replacement claim 的 evidence link 会写入
- deep reflection 会更新 `identity_core` / `commitments` 并写入审计字段
- snapshot 组装
- failure mode 回归

特别关注这些测试名：

- `reflection_rejects_inferred_replacement_without_external_evidence`
- `reflection_accepts_inferred_replacement_with_explicit_evidence`
- `reflection_can_update_identity_and_commitments_with_audited_supporting_evidence`
- `reflection_preserves_baseline_commitment_when_updates_replace_commitments`
- `reflection_rejects_identity_update_without_supporting_evidence`
- `reflection_rejects_identity_update_when_evidence_query_resolves_empty`
- `reflection_without_replacement_claim_disputes_old_claim_and_updates_identity`
- `reflection_rejects_missing_replacement_evidence_event_ids`
- `reflection_rejects_empty_identity_update_even_with_supporting_evidence`

### 6.7 automatic self-revision runtime coverage

```zsh
cargo test --test mcp_stdio ingest_interaction_can_trigger_conflict_auto_reflection_when_explicit_conflict_hints_present -v
cargo test --test mcp_stdio ingest_interaction_does_not_auto_reflect_conflict_without_explicit_conflict_hints -v
cargo test --test mcp_stdio ingest_interaction_does_not_auto_reflect_conflict_with_non_conflict_trigger_hints -v
cargo test --test mcp_stdio ingest_interaction_returns_success_even_when_conflict_auto_reflection_fails -v
cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v
cargo test --test mcp_stdio blocked_decide_with_snapshot_does_not_auto_reflect_conflict_hints -v
cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v
cargo test --test mcp_stdio build_self_snapshot_returns_snapshot_when_best_effort_periodic_auto_reflection_fails -v
cargo test --test mcp_stdio ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection -v
```

重点覆盖：

- 当前 MCP-wired automatic path 是否仍然准确限定为：
  - `ingest_interaction -> failure`
  - `ingest_interaction -> conflict`
  - `decide_with_snapshot -> conflict`
  - `build_self_snapshot -> periodic`
- `ingest_interaction -> conflict` 是否仍要求显式 `trigger_hints` 包含 `conflict` 或 `identity`
- `decide_with_snapshot` 的 conflict auto-reflection 是否仍要求显式 conflict-compatible `trigger_hints`，且只在非 blocked 决策后运行
- `build_self_snapshot` 的 periodic auto-reflection 是否仍要求显式 `auto_reflect_namespace`
- best-effort auto-reflection 失败是否不会把主 MCP 成功路径改写成 MCP 错误
- `run_reflection` 是否仍是唯一 durable write path / persistence funnel

运行时 contract matrix 的权威定义见 `docs/project-status.md` §8 的「runtime hook contract matrix」表。本指南不再复制该表，只核对其语义不变。

实现细节核对：

- `ingest_interaction:failure` 当前对应 `failure` 或 `rollback` trigger hints，加上 failure evidence threshold。
- `build_self_snapshot:periodic` 属于 snapshot tool flow，但 best-effort reflection attempt 发生在 `build_self_snapshot::execute` 之前；它不是后台 scheduler。

### 6.8 automatic self-revision diagnostics

```zsh
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_recursion_guard_skip -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_not_triggered_case -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_rejected_proposal -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v
cargo test --test failure_modes auto_reflection_applies_model_proposed_evidence_subset_but_preserves_full_trigger_window_in_handled_ledger -v
cargo test --test failure_modes auto_reflection_repeated_suppression_does_not_extend_existing_cooldown -v
cargo test --test bootstrap doctor_reports_self_revision_runtime_coverage -v
```

重点覆盖：

- structured diagnostics 是否返回可直接检查的 summary contract：
  - `trigger_type`: `failure` / `conflict` / `periodic`
  - `namespace`
  - `trigger_key`
  - `outcome`: `handled` / `rejected` / `suppressed` / `not_triggered` / `skipped`
  - `suppression_reason`
  - `rejection_reason`
  - `cooldown_boundary`
  - `cooldown_state`: `none` / `set` / `active`
  - `evidence_window_size`
  - `selected_evidence_event_ids`
  - `durable_write_path = run_reflection`
- suppressed cooldown 是否保持已有窗口而不是在重复 suppression 时被无界延长
- `doctor` 输出是否保守暴露 runtime hook coverage 与 `self_revision_write_path`
- `doctor` 输出的 runtime hook list 是否仍然精确等于上面的 4 条 contract matrix，且 `self_revision_write_path = run_reflection`
- `doctor` 输出 runtime hooks 不应被解读成新增 MCP tool、后台 daemon 或“所有请求自动反思”

判读要点：

- `rejected` 表示触发器已经命中并进入 proposal 路径，但模型未给出可接受 proposal；此时应检查 `rejection_reason`，而不是 `suppression_reason`。治理校验失败会记录 rejected ledger 并以错误返回，不作为成功返回的 diagnostics summary。
- `suppressed` 表示这次触发被已有 ledger 状态压住，例如 `cooldown_active`、`evidence_window_unchanged` 或 `episode_watermark_unchanged`；此时应检查 `suppression_reason` 与 `cooldown_boundary`。
- `not_triggered` 和 `skipped` 仍然是前台、按调用发生的诊断结果：它们说明“本次调用未进入 durable write”，不表示系统存在后台自治流程。
- `selected_evidence_event_ids` 只表示已进入 handled durable write 的实际证据子集；`rejected`、`suppressed`、`not_triggered`、`skipped` 没有 durable write selection，应通过 `evidence_window_size` 读取本次触发窗口规模。
- `durable_write_path = run_reflection` 只是说明一旦进入 durable write，唯一允许的落盘路径仍是 `run_reflection`；它不代表新增 MCP tool、后台 daemon、额外 hook 或独立 self-revision worker。

### 6.9 self-revision evidence policy

```zsh
cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_outside_trigger_window -v
cargo test --test failure_modes auto_reflection_applies_model_proposed_evidence_subset_but_preserves_full_trigger_window_in_handled_ledger -v
cargo test --test failure_modes auto_reflection_intersects_proposed_evidence_query_with_current_trigger_window_when_ids_are_empty -v
cargo test --test failure_modes auto_reflection_applies_query_limit_within_current_trigger_window_when_ids_are_empty -v
cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_ids_that_do_not_match_query_policy -v
cargo test --test failure_modes auto_reflection_rejects_empty_proposed_evidence_query_instead_of_widening -v
cargo test --test failure_modes auto_reflection_scopes_trigger_window_to_input_namespace -v
cargo test --test failure_modes auto_reflection_rejects_namespace_filter_with_no_trigger_window_intersection -v
cargo test --test failure_modes auto_reflection_rejects_noop_proposal_when_query_has_no_trigger_window_intersection -v
cargo test --test openai_compatible_model openai_compatible_model_parses_self_revision_evidence_policy -v
```

重点覆盖：

- proposal 首阶段 evidence contract 是否包含 `proposed_evidence_event_ids`、`proposed_evidence_query` 与 `confidence`
- model 提议的 evidence id 是否仍必须落在当前 trigger window 内
- project / user scoped conflict 和 periodic trigger window 是否在 proposal narrowing 前排除 sibling namespace 的事件
- 当 model 同时提供 explicit ids 和 `proposed_evidence_query` 时，这些 ids 是否仍必须满足 query 在当前 trigger window 内的过滤约束
- handled ledger 是否保留完整 evidence window，而不是只保留 model 选择的子集
- `proposed_evidence_query` 在 explicit ids 为空时是否只会对当前 trigger window 做交集收口，并在有交集时只按当前窗口内候选应用 `limit`
- `proposed_evidence_query` 的 `recorded_after` / `recorded_before` 是否按 inclusive recency window 参与 trigger-window 内过滤
- `proposed_evidence_query` 在 explicit ids 为空且 query 无交集时是否拒绝处理，而不是绕过 query 改用 full trigger window
- record-only / no-op proposal 是否同样不能绕过 no-match query rejection
- `proposed_evidence_query` 当前是否仍不会在 id 为空时自动 widening / ranking

### 6.10 automatic self-revision MVP 定向验证

这是当前 self-revision MVP 的最低定向回归集。只要你改了下面任一部分，就至少补跑这 7 条：

- `src/application/auto_reflect_if_needed.rs`
- `src/interfaces/mcp/server.rs`
- `src/interfaces/mcp/dto.rs`
- `src/adapters/sqlite/store.rs`
- `src/ports/trigger_ledger_store.rs`
- `src/support/config.rs` 里与启动/数据库加载语义相关的代码

命令：

```zsh
cargo test --test application_use_cases auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger -v
cargo test --test sqlite_store sqlite_trigger_ledger_records_namespace_periodic_watermark_and_cooldown -v
cargo test --test mcp_stdio ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection -v
cargo test --test mcp_stdio ingest_interaction_can_trigger_conflict_auto_reflection_when_explicit_conflict_hints_present -v
cargo test --test mcp_stdio ingest_interaction_does_not_auto_reflect_conflict_without_explicit_conflict_hints -v
cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v
cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v
```

如果改动包含 `src/support/config.rs`，再追加：

```zsh
cargo test --test provider_config -v
```

覆盖点：

- 应用层会在重复 failure 窗口里只自动修订一次，并把 handled ledger 正确落盘
- SQLite adapter 会持久化 trigger ledger 的 `namespace`、`episode_watermark` 和 `cooldown_until`
- stdio runtime 的 4 条当前 MCP-wired automatic path 都会被最低回归集直接覆盖：
  - `ingest_interaction -> failure`
  - `ingest_interaction -> conflict`
  - `decide_with_snapshot -> conflict`
  - `build_self_snapshot -> periodic`
- direct `run_reflection` 不会递归回自动链路

额外注意：

- `decide_with_snapshot` / `build_self_snapshot` 仍要求显式 `auto_reflect_namespace`，`decide_with_snapshot` 还要求显式 conflict-compatible `trigger_hints`，并且只在非 blocked 决策后才会 best-effort 触发
- 不要把这组测试解读成“所有 MCP 入口都会自动反思”
- 当前 auto-reflection 仍通过已有 `run_reflection` 写入 identity / commitments，不存在新的 durable write 通道

### 6.11 self-revision demo package

如果改动涉及下面任一部分，需要补跑 demo package 定向验证：

- `src/bin/demo_openai_compatible_stub.rs`
- `src/bin/run_self_revision_demo.rs`
- `scripts/run-self-revision-demo.sh`
- `examples/agent-llm-mm.demo.example.toml`
- automatic self-revision runtime hook / provider / MCP `stdio` 相关代码

推荐命令：

```zsh
cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/manual-$(date +%Y%m%d-%H%M%S)
```

通过后，指定 output dir 下至少应有；按 Local Alpha 发布口径复核 `latest` 时，改用 `./scripts/product-smoke-local.sh`，不要直接让 demo wrapper 写入 `latest`：

- `doctor.json`
- `snapshot-before.json`
- `snapshot-after.json`
- `decision-before.json`
- `decision-after.json`
- `timeline.json`
- `sqlite-summary.json`
- `report.md`

重点确认：

- negative conflict 场景不会增加 handled conflict ledger
- positive conflict 场景会增加 handled conflict ledger
- after snapshot 会出现 revised commitment
- before / after decision action 会发生变化
- `doctor.json` 仍声明 durable write path 是 `run_reflection`

---

### 6.12 Local Alpha product smoke script

如果改动涉及 Local Alpha 发布 gate、本地启动包装、`doctor` 配置传递，或 self-revision demo 证据链的产品化入口，需要补跑 product smoke：

```zsh
./scripts/product-smoke-local.sh
```

如果要同时验证某个本地配置文件的 bootstrap / doctor 路径，可以传入可选 config path：

```zsh
./scripts/product-smoke-local.sh agent-llm-mm.local.toml
```

这里的 `agent-llm-mm.local.toml` 是本地用户配置占位路径，文件必须已存在；这个 smoke 只验证 `doctor` 的 config path 解析与传递，不会把 config path 传给 deterministic demo wrapper。

以上 repo-relative 示例要求当前目录是 repo root。如果从其他当前目录运行，使用脚本绝对路径：

```zsh
cd /tmp && /path/to/agent-llm-mm/scripts/product-smoke-local.sh
```

如果从其他当前目录运行且要传入配置文件，config path 也使用绝对路径：

```zsh
cd /tmp && /path/to/agent-llm-mm/scripts/product-smoke-local.sh /path/to/agent-llm-mm/agent-llm-mm.local.toml
```

通过标准：

- 脚本可从 repo root 或其他当前目录调用，并能定位 repo root
- 可选 config path 必须存在；脚本会解析成绝对路径后只传给 `./scripts/agent-llm-mm.sh doctor`
- `doctor` 退出码为 `0`
- 脚本会先把 demo wrapper 输出到 staging 目录，8 个 artifact 均通过后才替换 `target/reports/self-revision-demo/latest`
- `target/reports/self-revision-demo/latest` 下 8 个 required demo artifacts 均存在且非空

限制说明：`scripts/run-self-revision-demo.sh` 当前只接受第 1 个参数作为 output dir，不接受 config path。因此 product smoke 的 config path 只覆盖 `doctor`，self-revision demo 仍使用现有 deterministic demo 契约。这条 product smoke 是 Local Alpha gate 的产品化入口，不替代 demo / MVP 的 [Release Gate](release-gate.md)，也不表示 GA / production-ready。

---

### 6.13 Local first-run bootstrap smoke

如果改动涉及 `bootstrap-local`、本地配置引导、`doctor` 配置路径、默认数据库环境变量隔离，或 Local Alpha first-run 证据，需要补跑：

```zsh
bash -n scripts/first-run-bootstrap-smoke-local.sh
cargo test --test first_run_bootstrap_smoke -v
./scripts/first-run-bootstrap-smoke-local.sh target/first-run-bootstrap-smoke/manual-check
```

通过标准：

- 输出目录不存在或为空时脚本成功；非空目录会在写任何 artifact 前拒绝
- 脚本生成 `agent-llm-mm.local.toml`、`doctor.json`、`summary.json` 和同目录下的 `first-run.sqlite`
- `doctor.json.status = "ok"`、`provider = "mock"`、`self_revision_write_path = "run_reflection"`
- `doctor.json.daemon_enabled = false`，且 `daemon_observe_only.writes_allowed = false`
- `summary.json.fresh_machine_simulation = true`，`real_fresh_machine_evidence = false`
- 脚本会清理 `AGENT_LLM_MM_CONFIG` / `AGENT_LLM_MM_DATABASE_URL` 干扰，不写真实 HOME，不启动 `serve`，不调用 `product-smoke-local.sh` 或 demo wrapper，不调用远端命令

这条 smoke 只证明当前 checkout 内的本地首启模拟链路，不替代 product smoke、真实 fresh-machine install、Windows runner parity、installer、packager、remote bootstrapper 或 GA 证据。

---

### 6.14 Local release soak runner

如果改动涉及 release engineering、候选发布证据目录、soak evidence、release note 或产品化发布口径，需要补跑本地 release soak：

```zsh
bash -n scripts/release-soak-local.sh
cargo test --test local_alpha_release_evidence release_soak -v
./scripts/release-soak-local.sh local-alpha-YYYYMMDD.1-rc.1
```

如果要验证某个本地配置文件的 `doctor` / product smoke / support bundle 分支，可传入可选 config path：

```zsh
./scripts/release-soak-local.sh local-alpha-YYYYMMDD.1-rc.1 agent-llm-mm.local.toml
```

通过标准：

- candidate name 只能包含字母、数字、点、下划线或短横线，且不能包含 `..`
- evidence directory 写入 `target/reports/releases/<candidate-name>/`，目录必须不存在或为空
- 目录内包含 `git-head.txt`、`git-status-before.txt`、`git-status-after.txt`、`command-summary.tsv`、`commands/`、`secret-scan.log`、`artifact-scan.log`、`support-bundle-files.txt`、`support-bundle-sha256.txt`、`product-smoke-latest-files.txt`、`product-smoke-latest-sha256.txt`、`local-alpha-evidence-summary.json`、`local-alpha-evidence-summary.md`、`compatibility-matrix.json`、`release-boundaries.json` 和 `release-soak-summary.md`
- 运行顺序覆盖 `doctor`、`cargo test --test dashboard_http -v`、product smoke、first-run simulation、support bundle generation、secret scan、raw artifact scan 和 Local Alpha evidence summary
- support bundle secret scan 不应发现未脱敏 secret-like marker；raw artifact scan 不应发现 `.sqlite`、`.toml` 或 `.log`

这条 soak 只生成本地候选证据；它不生成真实 fresh-machine evidence、Windows runner evidence、remote/team evidence、上传、source tag、binary package、installer、service manager、auto-updater、release decision 或 GA / production-ready 证明。

---

## 7. 手工 Smoke Test

`MCP` `stdio` 是 JSON-RPC 交互协议，手工敲消息成本较高。当前项目更推荐直接运行自动化 E2E 测试，而不是纯手工交互。

如果你仍然想做一次最小人工验证，推荐下面的方式。

### 7.1 使用独立数据库启动服务

```zsh
cd ~/code/agent-llm-mm
cp examples/agent-llm-mm.example.toml agent-llm-mm.local.toml
./scripts/agent-llm-mm.sh serve
```

这会启动 MCP `stdio` 服务。由于它等待 JSON-RPC 消息，终端表面上会“挂住”，这是正常现象。

### 7.2 更实用的人工验证方式

另开一个终端，直接跑现有 E2E：

```zsh
cd ~/code/agent-llm-mm
cargo test --test mcp_stdio -- --nocapture
```

原因：

- 这条测试已经覆盖真实二进制
- 使用真实 `stdio`
- 覆盖 `initialize / tools/list / tools/call` 全链路
- 比手工拼 JSON-RPC 更稳定

### 7.3 手工验证 openai-compatible provider

如果你要专门确认 provider 路径已经不是 `mock`，推荐跑：

```powershell
cargo test --test openai_compatible_model -- --nocapture
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file -- --nocapture
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openrouter_provider_from_config_file -- --nocapture
cargo test --test mcp_stdio ingest_interaction_auto_reflection_uses_openrouter_provider_from_config_file -- --nocapture
```

新增 provider 的验收应先按 [Provider Readiness Checklist](provider-contract.md) 补齐 `partial` / `gap` 对应的专用回归或记录明确例外，再执行上述手工验证。

### 7.4 手工验证 evidence-aware reflection

如果你要专门手测 reflection 的显式证据行为，推荐先跑自动化：

```powershell
cargo test --test application_use_cases reflection_accepts_inferred_replacement_with_explicit_evidence
cargo test --test mcp_stdio inferred_replacement_reflection_with_evidence_is_accepted_over_stdio
```

如果必须走手工 `stdio` 路径，`run_reflection` 的关键入参如下：

```json
{
  "reflection": {
    "summary": "Two external observations support promoting the inferred replacement."
  },
  "supersede_claim_id": "<event_id>:claim:0",
  "replacement_claim": {
    "owner": "Self_",
    "subject": "self.role",
    "predicate": "is",
    "object": "principal_architect",
    "mode": "Inferred"
  },
  "replacement_evidence_event_ids": [
    "evt-reflection-1",
    "evt-reflection-2"
  ]
}
```

预期：

- `replacement_evidence_event_ids` 中的每个 ID 都必须对应一条已持久化的 `events` 记录
- 返回 `replacement_claim_id`
- 不是 `invalid_params`
- 后续 snapshot 中 active claim 应变为 replacement 对应的新命题

如果你要验证最小 deep reflection 更新，可在上述基础上再加：

```json
{
  "identity_update": {
    "canonical_claims": [
      "identity:self=staff_architect",
      "identity:style=evidence_first"
    ]
  },
  "commitment_updates": [
    {
      "owner": "Self_",
      "description": "prefer:evidence_backed_identity_updates"
    },
    {
      "owner": "Self_",
      "description": "forbid:write_identity_core_directly"
    }
  ]
}
```

额外预期：

- 后续 `build_self_snapshot` 返回的新 `identity` 与 `commitments` 已更新
- `reflections` 表会保留 supporting evidence 与请求更新内容的 JSON 审计字段

### 7.5 手工验证 automatic self-revision runtime hooks

如果你要专门观察 automatic self-revision MVP，而不是只看最终 snapshot，优先跑自动化：

```zsh
cargo test --test application_use_cases auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger -v
cargo test --test mcp_stdio ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection -v
cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v
cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v
```

当前你应期待的是：

- 第二次重复 failure 触发会因为 ledger cooldown 被 suppress
- 已成功的 `ingest_interaction` 不会因为 post-ingest auto-reflection 失败而变成 MCP error
- 已成功的 `decide_with_snapshot` / `build_self_snapshot` 也不应因为 best-effort auto-reflection 失败而变成 MCP error
- direct `run_reflection` 只执行显式请求，不会再触发一轮自动修订
- 这组验证只覆盖当前 4 条已接线 hook；不代表所有 MCP entry point 都会自动反思

### 7.6 手工验证 self-revision demo package

如果你想看一套可读 report，而不是逐条跑 MCP `stdio` 测试：

```zsh
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/manual-$(date +%Y%m%d-%H%M%S)
```

然后打开：

```text
target/reports/self-revision-demo/manual-<timestamp>/report.md
```

这条路径使用本地 deterministic provider，不需要真实 API key，也不会访问外网。
Local Alpha 发布证据的 `latest` 目录由 `./scripts/product-smoke-local.sh` 负责更新，不要在手工 demo 中直接覆盖它。

### 7.7 手工验证 Local Alpha product smoke

如果你想从 Local Alpha gate 的产品化入口复核本机 `doctor` 和 self-revision demo 证据链：

```zsh
./scripts/product-smoke-local.sh
```

带本地配置文件时：

```zsh
./scripts/product-smoke-local.sh agent-llm-mm.local.toml
```

以上 repo-relative 示例要求当前目录是 repo root；从其他当前目录运行时，使用 `/path/to/agent-llm-mm/scripts/product-smoke-local.sh`，如果要传入配置文件，也传入绝对 config path。

注意：config path 只传给 `doctor`；deterministic self-revision demo wrapper 仍只接收 output dir。

### 7.8 手工验证 dashboard 面板

如果你要手工查看只读 dashboard：

```zsh
cp examples/agent-llm-mm.example.toml agent-llm-mm.local.toml
./scripts/agent-llm-mm.sh serve
```

然后访问：

```text
http://127.0.0.1:8787/
```

如果改动涉及 dashboard，至少补跑：

```zsh
cargo test --test dashboard_config --test dashboard_recorder --test dashboard_projection --test dashboard_http
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
```

dashboard HTTP 测试会监听本机端口，受限沙箱中可能需要在允许本地监听的环境运行。该面板只读，不会调用 `run_reflection` 或修改 SQLite；`dashboard_rejects_write_methods_on_read_only_routes` 覆盖 POST / PUT / PATCH / DELETE 返回 `405 Method Not Allowed`，`dashboard_serves_html_summary_events_detail_and_health` 覆盖 HTML、JSON API、health 和 SSE 只读 GET surface。

如果改动涉及 dashboard 视觉或静态物料，还需要确认：

- `GET /` 包含 `Memory-chan Live Desk`
- 静态 HTML visual contract 覆盖动态 ID 省略、指标卡/operation chain 自适应网格、侧栏贴纸 `contain` 显示、移动端顶部状态条重排、hero 文案遮罩和移动端无横向溢出
- `GET /assets/memory-chan-hero.png` 返回 `content-type: image/png`
- `GET /assets/memory-chan-sidebar.png` 返回 `content-type: image/png`
- 生成图物料的仓库归属说明已经同步到 `NOTICE`

---

## 8. 迁移验证

如果你改了 SQLite schema 或 migration，至少跑下面两条：

```powershell
cargo test --test sqlite_store sqlite_store_bootstraps_all_tables
cargo test --test sqlite_store sqlite_bootstrap_backfills_namespace_for_legacy_claim_rows
```

这两条分别验证：

- 新建数据库的 schema 是否正确
- 旧数据库升级后是否完成 namespace 回填和强约束恢复

如果你改了 `claims` 表约束，再加跑：

```powershell
cargo test --test sqlite_store sqlite_database_rejects_corrupt_namespace_owner_pair_before_read
```

---

## 9. 常见问题排查

### 9.1 `cargo fmt --check` 失败

现象：

- 输出 diff

处理：

```powershell
cargo fmt
cargo fmt --check
```

### 9.2 `mcp_stdio` 失败，出现 `UnexpectedEof`

优先怀疑：

- 服务端启动后 panic
- `tools/call` 的 DTO 解析失败
- SQLite bootstrap 或 migration 出错
- provider 配置文件无法解析

排查顺序：

1. 先跑 `cargo test --test mcp_stdio -- --nocapture`
2. 再跑 `cargo test --test sqlite_store`
3. 如果是最近改了 DTO，优先检查 `src/interfaces/mcp/dto.rs`
4. 如果是 provider 路径，优先检查 `agent-llm-mm.local.toml`

### 9.3 SQLite 相关测试失败

优先怀疑：

- schema 与 adapter SQL 不一致
- legacy migration 没把旧表升级到最新约束
- `owner <-> namespace` 规则和数据库 `CHECK` 不一致

优先检查：

- `src/adapters/sqlite/schema.rs`
- `src/adapters/sqlite/store.rs`
- `src/domain/types.rs`
- `src/domain/claim.rs`
- `src/support/config.rs`

### 9.4 数据库路径加载语义和预期不一致

优先检查你是走哪条配置路径：

- `AppConfig::load()`：默认启动路径，会在读取配置文件后继续接受 `AGENT_LLM_MM_DATABASE_URL` 覆盖
- `AppConfig::load_from_path()`：显式文件加载路径，会保留文件里显式给出的 `database_url`

这意味着：

- 如果你通过脚本或默认启动路径运行服务，同时又设置了 `AGENT_LLM_MM_DATABASE_URL`，最终数据库位置可能不是 TOML 文件里写的那个
- 如果你在测试里直接调用 `load_from_path()`，显式文件里的 `database_url` 不会再被环境变量覆盖
- 但如果该文件省略 `database_url`，`load_from_path()` 仍可能通过 `AppConfig::default()` 继承环境变量派生出的默认路径

### 9.5 `invalid_params` 变成 `internal_error`

说明错误映射回退了。

优先检查：

- `src/error.rs`
- `src/interfaces/mcp/server.rs`

预期行为：

- 调用方参数错误返回 `-32602`
- 基础设施或服务端异常才返回 `-32603`

---

## 10. 修改后最低测试门槛

如果你只是改了一处小逻辑，最低建议如下：

### 改 domain / claim / namespace 规则

```powershell
cargo test --test domain_invariants --test domain_snapshot --test application_use_cases
```

### 改 SQLite schema / migration / store

```powershell
cargo test --test sqlite_store
```

### 改 reflection 输入 / DTO / 证据门槛

```powershell
cargo test --test application_use_cases --test failure_modes --test mcp_stdio
```

### 改 automatic self-revision / trigger ledger / runtime hook wiring

```zsh
cargo test --test application_use_cases auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger -v
cargo test --test sqlite_store sqlite_trigger_ledger_records_namespace_periodic_watermark_and_cooldown -v
cargo test --test mcp_stdio ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection -v
cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v
cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v
```

### 改 self-revision demo package

```zsh
cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/manual-$(date +%Y%m%d-%H%M%S)
```

发布前使用对应 release gate 的 freshness 口径：这些 artifact 必须来自同一次成功运行，不能只因为已有旧文件就视为通过。Local Alpha 发布前不要直接让 demo wrapper 写入 `latest`，应使用 `./scripts/product-smoke-local.sh` 的 staging / promote 路径。

### 改 Local Alpha product smoke gate / wrapper

```zsh
bash -n scripts/product-smoke-local.sh
./scripts/product-smoke-local.sh
./scripts/product-smoke-local.sh agent-llm-mm.local.toml
(cd /tmp && /path/to/agent-llm-mm/scripts/product-smoke-local.sh)
(cd /tmp && /path/to/agent-llm-mm/scripts/product-smoke-local.sh /path/to/agent-llm-mm/agent-llm-mm.local.toml)
test -s target/reports/self-revision-demo/latest/report.md
git diff --check
```

其中 `./scripts/product-smoke-local.sh` 是 repo root 示例；`agent-llm-mm.local.toml` 是本地用户配置占位路径，文件必须已存在，且只覆盖 `doctor` config path；从其他当前目录运行时，改用 `/path/to/agent-llm-mm/scripts/product-smoke-local.sh` 这类绝对脚本路径，并用绝对 config path 验证配置分支。

这组命令只覆盖 Local Alpha product smoke 入口；demo / MVP 发布前仍以 `release-gate.md` 为准，Local Alpha 发布前仍以 `docs/product/release-gate-local-alpha.md` 为准。

### 改 Local Alpha support bundle

```zsh
cargo test --test support_bundle -v
bash -n scripts/generate-support-bundle.sh
rm -rf target/support-bundles/manual-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
rm -rf target/support-bundles/manual-correlation-check
./scripts/generate-support-bundle.sh target/support-bundles/manual-correlation-check --correlation-id mcp-tool-call-018fbc89-9ac1-4f5d-8b2a-1f6f5f27b205
find target/support-bundles/manual-check -maxdepth 1 -type f -print | sort
rg -n 'api_key|api-key|x-api-key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///|token=' target/support-bundles/manual-check || true
find target/support-bundles/manual-check \( -name '*.sqlite' -o -name '*.toml' -o -name '*.log' \) -print
tmp_log="$(mktemp target/support-bundles/manual-log.XXXXXX.log)"
printf 'INFO request prompt=hidden Authorization: Bearer sk-manual token=abc sqlite:///Users/example/private.sqlite\n' > "${tmp_log}"
rm -rf target/support-bundles/manual-check-with-log
./scripts/generate-support-bundle.sh target/support-bundles/manual-check-with-log --log-file "${tmp_log}"
test -s target/support-bundles/manual-check-with-log/local-log-excerpts.json
rg -n 'api_key|api-key|x-api-key|Authorization|Bearer|sk-|provider_token|openai_api_key|password|secret|sqlite:///|token=' target/support-bundles/manual-check-with-log || true
find target/support-bundles/manual-check-with-log \( -name '*.sqlite' -o -name '*.toml' -o -name '*.log' \) -print
rg -n 'API key|redact|support bundle|excluded|doctor|generate-support-bundle' docs/product/support-bundle-local-alpha.md docs/product/release-gate-local-alpha.md
git diff --check
```

这组命令验证首版本地 support bundle 生成器、脚本入口、脱敏边界、read-only operation-log 查询、显式 `--correlation-id mcp-tool-call-<uuid-v4>` operation summary 过滤、显式 `--log-file` 本地日志摘要/摘录，以及文档口径。输出目录必须不存在或为空；测试会覆盖非空目录被拒绝，避免旧的本地文件混入可分享支持包。敏感词扫描应无实际泄露；`find` 命令不应打印 `.sqlite`、`.toml` 或原始 `.log` 文件。operation summary 过滤只接受生成型 `mcp-tool-call-<uuid-v4>` correlation id，并仍只输出 metadata，不输出 request / response / diagnostic payload summary；user/project namespace 只输出 shape，secret-like operation metadata 会被替换。日志摘录只允许显式传入单个本地文件，secret-like config/log 文件名会折叠成 `<local-path>/<redacted-name>`；不允许扫描默认日志目录、home、系统日志、browser profile、SSH/cookie/session、shell history 或 `target/` 输出。超大日志只读取有界尾部窗口，并以 `line_number_scope = "tail"` 标记行号语义。该生成器不会创建或迁移缺失 SQLite 数据库，也不会通过 runtime bootstrap seed 默认 identity / commitments；它仍是本地诊断辅助，不代表远程上传、生产支持通道或 Local Alpha 完成。

### 改 release engineering / local soak evidence

```zsh
bash -n scripts/release-soak-local.sh
cargo test --test local_alpha_release_evidence release_soak -v
rm -rf target/reports/releases/manual-local-soak
./scripts/release-soak-local.sh manual-local-soak
test -s target/reports/releases/manual-local-soak/release-soak-summary.md
test -s target/reports/releases/manual-local-soak/command-summary.tsv
test -s target/reports/releases/manual-local-soak/local-alpha-evidence-summary.json
test -s target/reports/releases/manual-local-soak/support-bundle-sha256.txt
test -s target/reports/releases/manual-local-soak/product-smoke-latest-sha256.txt
test ! -s target/reports/releases/manual-local-soak/secret-scan.log
test ! -s target/reports/releases/manual-local-soak/artifact-scan.log
git diff --check
```

这组命令验证本地 release soak runner、candidate-specific evidence directory、doctor / dashboard HTTP / product smoke / first-run simulation / support bundle / evidence summary 串联，以及 support bundle secret/raw-artifact scan 和 support bundle / product smoke SHA-256 manifest。它不创建 release tag、安装包、Windows runner evidence、真实 fresh-machine evidence、remote/team evidence、上传或发布认证证据；`local-alpha-evidence-summary.json` 若仍为 `in_progress` / `not_verified`，必须保留 open gate。

### 改 daemon observe-only gate

```zsh
rg -n 'observe-only|run_reflection|forbidden|daemon|remote listener' docs/product/daemon-observe-only-gate.md docs/product/release-gate-local-alpha.md docs/roadmap.md docs/progress-tracker.md
cargo test --test daemon_config -v
git diff --check
```

这组命令验证 daemon 观察模式的边界文档、doctor diagnostics、`serve` 中 `[daemon].enabled = true` 时的 observe-only handle wiring、本地 handle start / stop / drop-abort 生命周期。Local Alpha 仍保持 daemon disabled by default；observe-only 阶段不能调用 `run_reflection`，也不能声明后台自治或 write-capable daemon。

### 改 daemon observe-only diagnostics / doctor 输出

```zsh
cargo test --test daemon_config -v
cargo test --test operation_log -v
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
rg -n 'daemon_observe_only|observe-only|writes_allowed|remote_listener_enabled|operation_log' README.md docs/product/daemon-observe-only-gate.md docs/product/release-gate-local-alpha.md docs/project-status.md docs/progress-tracker.md
git diff --check
```

这组命令验证 `doctor.daemon_observe_only` 的本机只读诊断字段、daemon 默认关闭、observe-only 写入 gate、operation-log status 查询，以及文档口径。`doctor` 的 runtime bootstrap 仍会执行既有 SQLite 初始化和 baseline guard 初始化，但 `doctor` 本身不启动 daemon handle；observe-only diagnostics 本身只能读取本地 `operation_log` 的 failed / suppressed `tool` 与 `trigger` 候选，不能调用 `run_reflection`、不能新增 identity / commitments / claims / events / reflections 语义写入，也不能声明 daemon 已具备后台自治。

### 改 correlation id / operation log observability

```zsh
cargo test --test dashboard_projection --test dashboard_http --test mcp_stdio --test operation_log -v
cargo test --test mcp_stdio mcp_tool_failure_does_not_persist_provider_error_payload_in_operation_log -v
cargo test --test mcp_stdio dashboard_failed_tool_event_does_not_expose_provider_error_payload -v
```

这组命令验证 MCP tool call 级 correlation id、dashboard 详情投影、`/api/operation-log` 本机只读 durable history 查询和 handler-level MCP tool operation-log 元数据。失败路径记录不改变 MCP error code / error message 语义，且 durable diagnostic 与 dashboard failure event 只保留安全分类元数据，不落 raw request / provider payload；该链路只是 observability metadata，不代表新增 identity / commitments / reflection 的旁路写入能力。`rmcp` framework-level 解析/路由失败（例如非 object `arguments`）不进入项目 handler，因此不声明为 durable operation-log 覆盖范围。

```zsh
cargo test --test mcp_stdio non_object_mcp_tool_arguments_do_not_reach_handler_operation_log -v
rg -n 'correlation_id|mcp-tool-call|run_reflection|operation-log' docs/product/correlation-id-contract.md docs/product/release-gate-local-alpha.md
git diff --check
```

### 改 `src/support/config.rs`

```zsh
cargo test --test provider_config -v
```

### 改 MCP DTO / server / 错误映射

```powershell
cargo test --test mcp_stdio
```

### 普通提交前快速回归

```zsh
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
```

demo / MVP 发布前核验不使用这段简表作为最终依据；请按 [Release Gate](release-gate.md) 执行完整 MVP gate。Local Alpha / product alpha 发布前核验使用 [Local Alpha Release Gate](product/release-gate-local-alpha.md)。

---

## 11. 当前结论

截至 `2026-05-14`，推荐把下面五条当作普通提交前基线；demo / MVP 发布前仍以 [Release Gate](release-gate.md) 为准；Local Alpha / product alpha 发布前以 [Local Alpha Release Gate](product/release-gate-local-alpha.md) 为准：

```zsh
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
AGENT_LLM_MM_DATABASE_URL=sqlite:///private/tmp/agent-llm-mm-doctor.sqlite ./scripts/agent-llm-mm.sh doctor
```

如果这五条都通过，说明当前工作树至少满足：

- 编码规范通过
- 编译与静态检查通过
- `namespace`、SQLite migration、MCP `stdio`、reflection 闭环和 automatic self-revision MVP 基线都可继续追加定向验证
- 本机运行时 bootstrap 正常
