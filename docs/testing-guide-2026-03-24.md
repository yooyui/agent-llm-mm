# Self-Agent MCP 测试指南（2026-03-24，按 2026-04-28 fresh 验证更新）

## 1. 目标

这份文档说明当前仓库应如何测试，覆盖：

- 代码格式与静态检查
- 自动化测试套件
- `namespace` / SQLite 迁移 / MCP `stdio` 的定向验证
- `doctor` 预检
- 手工 smoke test 的推荐方式

如果目标是判断“是否可以按当前 demo / MVP 口径发布”，请同时阅读 [Release Gate](release-gate.md)。本页偏向测试与回归，`release-gate.md` 负责收口发布前的最低 gate、self-revision 证据 gate、dashboard gate 和 sandbox 失败解释口径。

当前工作目录（按实际环境替换）：

`~/code/agent-llm-mm`

命令执行环境要求：

- 安装 Rust toolchain
- `cargo` 可用
- `bash` 或 `zsh`（用于 `scripts/agent-llm-mm.sh`）

---

## 2. 当前测试基线

截至 `2026-04-28`，`cargo test` 全量通过，摘要如下：

- `application_use_cases`: 22 passed
- `bootstrap`: 15 passed
- `dashboard_config`: 4 passed
- `dashboard_http`: 5 passed
- `dashboard_projection`: 2 passed
- `dashboard_recorder`: 2 passed
- `decision_flow`: 2 passed
- `domain_invariants`: 4 passed
- `domain_snapshot`: 6 passed
- `demo_openai_compatible_stub`: 1 passed
- `failure_modes`: 29 passed
- `mcp_stdio`: 27 passed
- `openai_compatible_model`: 7 passed
- `provider_config`: 5 passed
- `self_revision_demo_runner`: 2 passed
- `sqlite_store`: 19 passed

合计：152 个测试通过。

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
5. `./scripts/agent-llm-mm.sh doctor`
6. `cargo run --quiet --bin agent_llm_mm -- doctor`
7. 如果改动涉及 automatic self-revision MVP，再补跑本指南里的 runtime coverage / diagnostics / evidence policy 定向验证
8. 如果改动涉及 demo package，或要按发布口径复核证据链，再补跑 `./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest`

如果只想快速回归某个变更，再执行对应的定向测试。若需要一份面向发布前核验的固定检查单，直接使用 [Release Gate](release-gate.md)。

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
./scripts/agent-llm-mm.sh doctor
```

```zsh
cargo run --quiet --bin agent_llm_mm -- doctor
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
- `sqlite_bootstrap_backfills_namespace_for_legacy_event_rows`
- `sqlite_bootstrap_backfills_namespace_for_legacy_claim_rows`
- `sqlite_store_rejects_owner_namespace_mismatch_on_write`
- `sqlite_database_rejects_corrupt_namespace_owner_pair_before_read`
- `sqlite_owner_namespace_sql_rules_have_single_source`

适用场景：

- 修改了 `src/adapters/sqlite/schema.rs`
- 修改了 `src/adapters/sqlite/store.rs`
- 修改了 `Namespace` / `ClaimDraft` 相关规则

### 6.2 MCP `stdio` 端到端

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

### 6.3 Provider 合规预检

新增 provider 前先阅读 [Provider Readiness Checklist](provider-contract.md)。下面这组命令只是当前共享 provider 路径的最小验证；如果 checklist 里仍有 `partial` 或 `gap` 且新 provider 依赖该行为，新增 provider 的同一变更必须补齐对应专用回归或记录明确例外。

```zsh
cargo test --test provider_config -v
cargo test --test openai_compatible_model -v
cargo test --test mcp_stdio decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file -v
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
  - `load_prefers_config_path_from_environment`
  - `load_prefers_database_url_env_over_default_config_file`
  - `doctor_fails_when_openai_provider_config_is_missing_api_key`
- `tests/openai_compatible_model.rs`
  - `openai_compatible_model_parses_first_assistant_message_into_action`
  - `openai_compatible_model_rejects_empty_action`
  - `openai_compatible_model_surfaces_non_success_status`
  - `openai_compatible_model_parses_self_revision_proposal_from_assistant_message`
  - `openai_compatible_model_defaults_missing_machine_patch_in_self_revision_proposal`
  - `openai_compatible_model_accepts_fenced_json_self_revision_proposal`
  - `openai_compatible_model_parses_self_revision_evidence_policy`
- `tests/mcp_stdio.rs`
  - `decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file`

新增 provider 前的阻断缺口：

- `doctor` redaction 目前缺少正向断言；新增 provider 前需要证明 configured secret 不会出现在 serialized report 或用户可见诊断中
- timeout failure 目前缺少专用回归；新增 provider 前需要覆盖超时不会挂起或静默 fallback
- malformed JSON 目前只有部分解析失败路径覆盖；新增 provider 前需要补齐 decision 与 self-revision proposal 的明确错误断言

### 6.4 领域不变量

```zsh
cargo test --test domain_invariants --test domain_snapshot
```

重点覆盖：

- inferred claim 的证据门槛
- `identity_core` 不能被普通 ingest 直接改写
- namespace 默认派生和 owner 匹配
- snapshot evidence 预算与 gate 行为

### 6.5 应用层编排

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

### 6.6 automatic self-revision runtime coverage

```zsh
cargo test --test mcp_stdio ingest_interaction_can_trigger_conflict_auto_reflection_when_explicit_conflict_hints_present -v
cargo test --test mcp_stdio ingest_interaction_does_not_auto_reflect_conflict_without_explicit_conflict_hints -v
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

运行时 contract matrix 应保持为：

| Hook | Trigger Input | Runs When | Does Not Do |
| --- | --- | --- | --- |
| `ingest_interaction:failure` | repeated or explicit failure signal | after successful ingest path | does not turn successful ingest into MCP error if best-effort reflection fails |
| `ingest_interaction:conflict` | explicit `trigger_hints` containing `conflict` or `identity` | after successful ingest path | does not infer conflict from arbitrary text alone |
| `decide_with_snapshot:conflict` | explicit `auto_reflect_namespace` and conflict-compatible `trigger_hints` | after non-blocked decision | does not run when commitment gate blocks the decision |
| `build_self_snapshot:periodic` | explicit `auto_reflect_namespace` | during snapshot build with periodic policy | does not create a background scheduler |

实现细节核对：

- `ingest_interaction:failure` 当前对应 `failure` 或 `rollback` trigger hints，加上 failure evidence threshold。
- `build_self_snapshot:periodic` 属于 snapshot tool flow，但 best-effort reflection attempt 发生在 `build_self_snapshot::execute` 之前；它不是后台 scheduler。

### 6.7 automatic self-revision diagnostics

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
  - `outcome`: `handled` / `rejected` / `suppressed` / `not_triggered` / `skipped`
  - `suppression_reason`
  - `rejection_reason`
  - `cooldown_boundary`
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

### 6.8 self-revision evidence policy

```zsh
cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_outside_trigger_window -v
cargo test --test failure_modes auto_reflection_applies_model_proposed_evidence_subset_but_preserves_full_trigger_window_in_handled_ledger -v
cargo test --test failure_modes auto_reflection_intersects_proposed_evidence_query_with_current_trigger_window_when_ids_are_empty -v
cargo test --test failure_modes auto_reflection_applies_query_limit_within_current_trigger_window_when_ids_are_empty -v
cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_ids_that_do_not_match_query_policy -v
cargo test --test failure_modes auto_reflection_rejects_empty_proposed_evidence_query_instead_of_widening -v
cargo test --test failure_modes auto_reflection_rejects_namespace_filter_with_no_trigger_window_intersection -v
cargo test --test failure_modes auto_reflection_rejects_noop_proposal_when_query_has_no_trigger_window_intersection -v
cargo test --test openai_compatible_model openai_compatible_model_parses_self_revision_evidence_policy -v
```

重点覆盖：

- proposal 首阶段 evidence contract 是否包含 `proposed_evidence_event_ids`、`proposed_evidence_query` 与 `confidence`
- model 提议的 evidence id 是否仍必须落在当前 trigger window 内
- 当 model 同时提供 explicit ids 和 `proposed_evidence_query` 时，这些 ids 是否仍必须满足 query 在当前 trigger window 内的过滤约束
- handled ledger 是否保留完整 evidence window，而不是只保留 model 选择的子集
- `proposed_evidence_query` 在 explicit ids 为空时是否只会对当前 trigger window 做交集收口，并在有交集时只按当前窗口内候选应用 `limit`
- `proposed_evidence_query` 在 explicit ids 为空且 query 无交集时是否拒绝处理，而不是绕过 query 改用 full trigger window
- record-only / no-op proposal 是否同样不能绕过 no-match query rejection
- `proposed_evidence_query` 当前是否仍不会在 id 为空时自动 widening / ranking

### 6.9 automatic self-revision MVP 定向验证

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

### 6.10 self-revision demo package

如果改动涉及下面任一部分，需要补跑 demo package 定向验证：

- `src/bin/demo_openai_compatible_stub.rs`
- `src/bin/run_self_revision_demo.rs`
- `scripts/run-self-revision-demo.sh`
- `examples/agent-llm-mm.demo.example.toml`
- automatic self-revision runtime hook / provider / MCP `stdio` 相关代码

推荐命令：

```zsh
cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

通过后，`target/reports/self-revision-demo/latest` 下至少应有；按发布口径复核时，这些 artifact 必须来自同一次成功运行：

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
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

然后打开：

```text
target/reports/self-revision-demo/latest/report.md
```

这条路径使用本地 deterministic provider，不需要真实 API key，也不会访问外网。

### 7.7 手工验证 dashboard 面板

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
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

发布前使用 `release-gate.md` 的 freshness 口径：这些 artifact 必须来自同一次成功运行，不能只因为 `latest` 目录已有旧文件就视为通过。

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
./scripts/agent-llm-mm.sh doctor
```

发布前核验不使用这段简表作为最终依据；请按 [Release Gate](release-gate.md) 执行完整 gate。

---

## 11. 当前结论

截至 `2026-04-28`，推荐把下面五条当作普通提交前基线；发布前仍以 [Release Gate](release-gate.md) 为准：

```zsh
cargo fmt --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

如果这五条都通过，说明当前工作树至少满足：

- 编码规范通过
- 编译与静态检查通过
- `namespace`、SQLite migration、MCP `stdio`、reflection 闭环和 automatic self-revision MVP 基线都可继续追加定向验证
- 本机运行时 bootstrap 正常
