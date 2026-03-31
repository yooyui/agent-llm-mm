# Self-Agent MCP 测试指南（2026-03-24，按 2026-03-27 实现复核更新）

## 1. 目标

这份文档说明当前仓库应如何测试，覆盖：

- 代码格式与静态检查
- 自动化测试套件
- `namespace` / SQLite 迁移 / MCP `stdio` 的定向验证
- `doctor` 预检
- 手工 smoke test 的推荐方式

当前工作目录：

`D:\Code\agent_llm_mm`

命令执行环境要求：

- 使用 `pwsh.exe`
- 默认 UTF-8
- Windows 11

---

## 2. 当前测试基线

截至 `2026-03-31`，`cargo test` 全量通过，摘要如下：

- `application_use_cases`: 11 passed
- `bootstrap`: 9 passed
- `decision_flow`: 2 passed
- `domain_invariants`: 4 passed
- `domain_snapshot`: 6 passed
- `failure_modes`: 3 passed
- `mcp_stdio`: 8 passed
- `openai_compatible_model`: 3 passed
- `provider_config`: 4 passed
- `sqlite_store`: 8 passed

合计：58 个测试通过。

---

## 3. 测试前准备

### 3.1 环境要求

- 安装 Rust toolchain
- 可用的 `cargo`
- 使用 PowerShell 7

### 3.2 建议进入工作目录

```powershell
Set-Location 'D:\Code\agent_llm_mm'
```

### 3.3 数据库隔离建议

服务默认会把 SQLite 文件放到系统临时目录下的 `agent-llm-mm.sqlite`。为了避免和已有运行实例互相污染，手工测试时建议显式设置：

```powershell
Copy-Item .\examples\agent-llm-mm.example.toml .\agent-llm-mm.local.toml
```

然后修改 `agent-llm-mm.local.toml` 里的：

- `database_url`
- `provider`
- provider-specific 配置

如果只是跑现有自动化测试，不需要手工设置；测试本身已经为大多数场景隔离了数据库。

---

## 4. 推荐测试顺序

建议按下面顺序执行：

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`
4. `pwsh -File .\scripts\agent-llm-mm.ps1 doctor`

如果只想快速回归某个变更，再执行对应的定向测试。

---

## 5. 全量验证

### 5.1 格式检查

```powershell
cargo fmt --check
```

通过标准：

- 命令退出码为 `0`
- 没有 diff 输出

### 5.2 静态检查

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

通过标准：

- 命令退出码为 `0`
- 没有 warning

### 5.3 全量测试

```powershell
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

### 5.4 本机预检

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
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

```powershell
cargo test --test sqlite_store
```

重点覆盖：

- `claims.namespace` 持久化
- legacy schema 迁移
- `owner <-> namespace` 数据库级 `CHECK` 约束
- adapter 写入/读取兜底
- owner/namespace SQL 规则是否保持单一来源

特别关注这些测试名：

- `sqlite_store_bootstraps_all_tables`
- `sqlite_bootstrap_backfills_namespace_for_legacy_claim_rows`
- `sqlite_store_rejects_owner_namespace_mismatch_on_write`
- `sqlite_database_rejects_corrupt_namespace_owner_pair_before_read`
- `sqlite_owner_namespace_sql_rules_have_single_source`

适用场景：

- 修改了 `src/adapters/sqlite/schema.rs`
- 修改了 `src/adapters/sqlite/store.rs`
- 修改了 `Namespace` / `ClaimDraft` 相关规则

### 6.2 MCP `stdio` 端到端

```powershell
cargo test --test mcp_stdio
```

重点覆盖：

- 工具是否正确暴露
- `ingest_interaction -> build_self_snapshot` 是否共享 runtime 状态
- `run_reflection` 是否影响 active snapshot
- 配置文件指定 provider 后是否真的走到对应 provider
- `run_reflection` 的显式 evidence 输入是否允许 inferred replacement
- baseline commitment 是否阻断 forbidden action
- 非法 namespace 是否返回 `-32602 invalid_params`

特别关注这些测试名：

- `server_exposes_expected_tools_over_stdio`
- `stdio_tools_share_runtime_state_across_calls`
- `decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file`
- `conflicting_reflection_over_stdio_removes_claim_from_active_snapshot`
- `inferred_replacement_reflection_with_evidence_is_accepted_over_stdio`
- `fresh_stdio_runtime_blocks_forbidden_action_with_seeded_commitment`
- `invalid_namespace_is_reported_as_invalid_params_over_stdio`

适用场景：

- 修改了 `src/interfaces/mcp/dto.rs`
- 修改了 `src/interfaces/mcp/server.rs`
- 修改了应用层输入校验或错误映射

### 6.3 领域不变量

```powershell
cargo test --test domain_invariants --test domain_snapshot
```

重点覆盖：

- inferred claim 的证据门槛
- `identity_core` 不能被普通 ingest 直接改写
- namespace 默认派生和 owner 匹配
- snapshot evidence 预算与 gate 行为

### 6.4 应用层编排

```powershell
cargo test --test application_use_cases --test failure_modes
```

重点覆盖：

- ingest 事务顺序
- reflection 状态流转
- inferred replacement 在有显式 evidence 时可通过
- replacement claim 的 evidence link 会写入
- snapshot 组装
- failure mode 回归

特别关注这些测试名：

- `reflection_rejects_inferred_replacement_without_external_evidence`
- `reflection_accepts_inferred_replacement_with_explicit_evidence`
- `reflection_rejects_missing_replacement_evidence_event_ids`

---

## 7. 手工 Smoke Test

`MCP` `stdio` 是 JSON-RPC 交互协议，手工敲消息成本较高。当前项目更推荐直接运行自动化 E2E 测试，而不是纯手工交互。

如果你仍然想做一次最小人工验证，推荐下面的方式。

### 7.1 使用独立数据库启动服务

```powershell
Set-Location 'D:\Code\agent_llm_mm'
Copy-Item .\examples\agent-llm-mm.example.toml .\agent-llm-mm.local.toml
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

这会启动 MCP `stdio` 服务。由于它等待 JSON-RPC 消息，终端表面上会“挂住”，这是正常现象。

### 7.2 更实用的人工验证方式

另开一个终端，直接跑现有 E2E：

```powershell
Set-Location 'D:\Code\agent_llm_mm'
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

### 9.4 `invalid_params` 变成 `internal_error`

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

### 改 MCP DTO / server / 错误映射

```powershell
cargo test --test mcp_stdio
```

### 准备提交前

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

---

## 11. 当前结论

截至 `2026-03-27`，推荐把下面四条当作提交前基线：

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

如果这四条都通过，说明当前工作树至少满足：

- 编码规范通过
- 编译与静态检查通过
- `namespace`、SQLite migration、MCP `stdio` 和 reflection 闭环都通过自动化验证
- 本机运行时 bootstrap 正常
