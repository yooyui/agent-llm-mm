# 本机 MCP 接入说明（2026-03-26，按 2026-04-29 fresh 验证更新）

## 1. 目标

把 `agent_llm_mm` 以本机 `stdio` MCP 服务的方式接入 Codex 等 AI 客户端，并保证：

- 启动路径稳定
- SQLite 落盘路径可控
- 有独立的预检命令
- 能清楚区分“可正式嵌入的能力”和“仍属 mock/实验的能力”
- 对 automatic self-revision MVP 的入口范围和失败语义有准确预期

## 2. 推荐接入形态

macOS 下优先入口是 `scripts/agent-llm-mm.sh`。它提供 `doctor` / `serve` 两种封装，并贴合当前 zsh / bash 环境。

如果你想完全绕过脚本，也可直接运行 `cargo run --quiet --bin agent_llm_mm -- <serve|doctor>`。

原因：

- 可以从任意当前目录启动
- 可以固化项目根目录
- 可以统一 `serve` / `doctor` 两种模式
- 后续切换为预编译二进制时，客户端配置无需大改

入口脚本：

- [scripts/agent-llm-mm.sh](../scripts/agent-llm-mm.sh)

## 3. 本机预检

正式接入前，先执行：

```zsh
cd ~/code/agent-llm-mm
cp examples/agent-llm-mm.example.toml agent-llm-mm.local.toml
./scripts/agent-llm-mm.sh doctor
```

预期输出为 JSON，至少包含：

- `transport`
- `database_url`
- `auto_reflection_runtime_hooks`
- `self_revision_write_path`
- `status`

当前 `status = "ok"` 代表：

- 配置已解析
- SQLite 已可成功 bootstrap
- provider 已按配置完成校验
- 默认 runtime 初始化已通过
- `doctor` 还能保守显示当前 MCP runtime hook coverage 和 durable write path

当前 `doctor` 输出里的 self-revision 相关字段，应按下面口径理解：

- `auto_reflection_runtime_hooks`
  - 只表示当前 MCP-wired automatic path 的文档化 runtime coverage
  - 当前准确值应为：
    - `ingest_interaction:failure`
    - `ingest_interaction:conflict`
    - `decide_with_snapshot:conflict`
    - `build_self_snapshot:periodic`
- `self_revision_write_path`
  - 当前准确值应为 `run_reflection`
  - 这表示 automatic self-revision 最终仍收口到既有 durable write path，而不是新增 MCP tool 或旁路持久化接口

这些字段不表示：

- 存在新的 auto-reflection MCP tool
- 存在后台 daemon / 定时自治进程
- 所有 MCP 请求都会自动反思

截至 `2026-04-29`，fresh 验证还包括：

- `cargo test` 全量通过，153 个测试通过
- 其中 `application_use_cases` 22、`failure_modes` 30、`mcp_stdio` 27、`sqlite_store` 19、`dashboard_http` 5
- self-revision demo package wrapper 可生成本地 artifact report

## 4. 启动服务

```zsh
cd ~/code/agent-llm-mm
./scripts/agent-llm-mm.sh serve
```

注意：

- 该命令会启动 MCP `stdio` 服务
- 终端看起来像“挂住”，这是正确行为
- 不应在服务运行期间往标准输入随意写普通文本

## 5. Codex 配置示例

你当前机器上的 Codex 配置格式已经在使用：

- `[mcp_servers.<name>]`
- `command`
- `args`
- `env`

可直接参考：

- [examples/codex-mcp-config.toml](../examples/codex-mcp-config.toml)

推荐做法：

- `command` 指向 `scripts/agent-llm-mm.sh`
- `args` 只传 `serve`
- `env` 显式传入 `AGENT_LLM_MM_CONFIG`

```toml
command = "/absolute/path/agent-llm-mm/scripts/agent-llm-mm.sh"
args = ["serve"]
```

如你不想经过脚本，也可直接用 `cargo run`：

```toml
command = "cargo"
args = ["run", "--quiet", "--bin", "agent_llm_mm", "--", "serve"]
```

## 6. 接入排障（与平台文档共用）

同类问题请先按对应平台文档执行 `doctor` 排查，核心症状与处理一致：

| Symptom | Likely Cause | Verification | Fix |
| --- | --- | --- | --- |
| `doctor` cannot write SQLite | database path not writable or sandbox restriction | 先检查本地 TOML 的 `database_url` 与启动环境里的 `AGENT_LLM_MM_DATABASE_URL`；如果 `doctor` 已返回 JSON，再核对其中的 `database_url` | 设定 `AGENT_LLM_MM_DATABASE_URL` 指向可写路径，或在本地 TOML 固定可写 SQLite 路径 |
| MCP client starts the wrong binary | auxiliary `src/bin` target ambiguity | 检查客户端是否显式传递 `--bin agent_llm_mm` | 优先使用脚本入口（`agent-llm-mm.sh` / `agent-llm-mm.ps1`），或固定 `--bin agent_llm_mm` |
| dashboard not visible | `[dashboard].enabled = false` 或端口占用 | 查看配置和 `doctor` 输出 | 将 `enabled` 设为 `true`，并选择可用 localhost 端口 |
| model calls fail | provider 配置不完整 | `doctor` 中确认 `provider`、`base_url`、`model` | 补齐本地 TOML 的 provider 配置（仅本地文件，勿提交 API key） |

平台细节请参考：

- [docs/development-macos.md](./development-macos.md)（平台版排障矩阵）
- [docs/development-windows.md](./development-windows.md)（平台版排障矩阵）

## 7. 当前能力状态

### 已实现

- `ingest_interaction`
- `build_self_snapshot`
- `run_reflection`
- `doctor` / `serve`
- SQLite 持久化
- `namespace` 最小闭环
- `openai-compatible` provider
- 配置文件驱动的 provider 选择
- trigger-ledger-backed automatic self-revision MVP
  - 当前 MCP-wired automatic path 只有 4 条：
    - `ingest_interaction -> failure`
    - `ingest_interaction -> conflict`
    - `decide_with_snapshot -> conflict`
    - `build_self_snapshot -> periodic`
  - `ingest_interaction` 仍可通过 ingest DTO 提供 `trigger_hints`
  - `ingest_interaction -> conflict` 仍要求显式 `trigger_hints` 包含 `conflict` 或 `identity`
- `decide_with_snapshot` / `build_self_snapshot` 当前仍要求显式传 `auto_reflect_namespace`
- `decide_with_snapshot` 还要求显式传 conflict-compatible `trigger_hints`，否则不会因为“库里已有 evidence”而自动进入 conflict self-revision
  - proposal 首阶段已可携带 `proposed_evidence_event_ids`、`proposed_evidence_query`、`confidence`；其中 query 在 explicit ids 为空时可对当前 trigger window 做 bounded narrowing，并在有交集时只按当前窗口内候选应用 `limit`，若没有交集则拒绝处理而不是绕过 query；在 explicit ids 非空时也会约束这些 ids 必须满足当前窗口内的 query 过滤条件，但这仍不是 richer widening / ranking engine
  - best-effort auto-reflection 现在会返回 structured trigger / rejection / suppression / cooldown diagnostics，供日志与测试复用
  - proposal 会经过服务端治理，再转译到既有 `run_reflection`
  - 没有新增单独 MCP tool；identity / commitments 的 durable write path 仍是 `run_reflection`
  - direct `run_reflection` 不会递归进入 auto-reflection

### 部分实现

- `decide_with_snapshot`
- automatic self-revision 的 runtime coverage

原因：

- commitment gate 是真实能力
- 下游模型已可走 `openai-compatible`
- 返回契约仍是最小动作字符串
- 更适合作为流程验证能力，而不是最终生产决策能力
- 当前领域层与协调器能表达 `failure / conflict / periodic` trigger type
- 当前 MCP runtime coverage 已接到这 4 条 hook，但仍是受限、显式、best-effort 的 demo 形态
- 这不代表“所有 MCP entry point 自动反思”，也不代表出现后台自治调度

### 未实现

- richer 自动 evidence lookup
- richer evidence weighting / relation / ranking
- richer reflection 语义（当前已有最小 `identity_core` / `commitments` 深层修订，但仍不是 richer schema / versioned policy）
- 更多 provider 类型
- 持续后台自治运行、独立 daemon、完整自治代理行为

## 8. 正式接入时需要注意的点

### 8.1 数据库路径

未显式设置 `database_url` 时，默认库会落到当前平台的用户数据目录，并按“本机用户共享默认库”语义复用。正式接入时仍建议在 `agent-llm-mm.local.toml` 里固定为你可备份、可区分环境的路径，例如：

```toml
sqlite:///Users/<you>/Library/Application%20Support/agent-llm-mm-codex.sqlite
```

### 8.1.1 配置文件

推荐用法：

- 从 `examples/agent-llm-mm.example.toml` 复制一份本地配置
- 写入自己的 `database_url`
- 选择 `provider`
- 填入自己的 API key
- 不要把 `agent-llm-mm.local.toml` 提交到仓库

配置加载语义需要特别注意：

- 服务默认启动路径走 `AppConfig::load()`，会先读 `AGENT_LLM_MM_CONFIG` 或默认 `agent-llm-mm.local.toml`，然后再允许 `AGENT_LLM_MM_DATABASE_URL` 覆盖数据库路径
- 如果你的集成或测试直接走 `AppConfig::load_from_path()`，显式文件里的 `database_url` 会被保留
- 但如果该文件省略 `database_url`，`load_from_path()` 仍可能通过 `AppConfig::default()` 继承 `AGENT_LLM_MM_DATABASE_URL` 派生出的默认路径

实践建议：

- 正式接入时不要同时依赖 TOML 里的 `database_url` 和外部注入的 `AGENT_LLM_MM_DATABASE_URL`，除非你就是想显式覆盖
- 如果要诊断“为什么接到了另一份 SQLite”，先检查客户端启动环境里是否偷偷带上了 `AGENT_LLM_MM_DATABASE_URL`

### 8.2 数据隔离

建议至少区分：

- 正式接入库
- 手工测试库
- 开发实验库

避免把反思、修订和测试事件混入正式记忆。

### 8.3 并发访问

SQLite 非常适合本机 MVP，但它仍然是单写者模型。若多个 AI 客户端并发共享同一数据库文件，需要预期：

- 锁等待
- 写入竞争
- 调试时状态互相影响

更稳妥的做法是每个环境单独一份数据库文件。

### 8.4 日志与 stdout

这是 `stdio` MCP 服务，因此：

- MCP 协议通信依赖标准输入输出
- 不应在 `serve` 模式额外向 `stdout` 打印杂讯
- 诊断信息应放到 `doctor` 模式或日志侧

### 8.5 能力边界

当前这条分支已经具备可嵌入的最小记忆闭环，但还不是完整产品：

- 默认 MCP transport 仍是 `stdio`；只有显式设置 `[dashboard].enabled = true` 时，才会额外启动本机只读 HTTP dashboard
- 无远程 Web 管理后台、写入型 dashboard 或 MCP HTTP transport
- 无更丰富的 evidence 自动检索
- 已有最小 `identity_update` / `commitment_updates` 反思修订，但仍无 richer schema、版本化策略与更细粒度生命周期
- 无更多 provider 类型
- automatic self-revision 当前 MCP-wired automatic path 仅限 `ingest_interaction -> failure`、`ingest_interaction -> conflict`、`decide_with_snapshot -> conflict`、`build_self_snapshot -> periodic`
- `decide_with_snapshot` / `build_self_snapshot` 仍要求显式 `auto_reflect_namespace`；`decide_with_snapshot` 还要求显式 conflict-compatible `trigger_hints`，并且只在非 blocked 决策后 best-effort 运行
- automatic self-revision 仍受 trigger ledger、证据门槛和慢更新约束保护，不是完整自治 daemon

## 9. 推荐验证顺序

```zsh
cd ~/code/agent-llm-mm
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

如果你刚改了 self-revision / runtime 接线，再追加：

```zsh
cargo test --test application_use_cases auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger -v
cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v
cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v
cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_outside_trigger_window -v
cargo test --test bootstrap doctor_reports_self_revision_runtime_coverage -v
```

如果都通过，再把它挂到本机 MCP 客户端配置里。
