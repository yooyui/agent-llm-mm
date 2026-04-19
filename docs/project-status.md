# 当前实现状态

## 项目定位

当前仓库更准确的定位是：

“一个面向本机 AI 客户端的 trigger-ledger-backed self-agent memory MVP / technical demo”

它已经完成了工程闭环、本机接入闭环，以及受治理的 automatic self-revision MVP；但还没有完成原始设计里更完整的“自我机制”产品语义，也不能对外包装成完整自治系统。

## 已实现

### 1. 最小运行链路

当前最小链路已经打通：

`events -> claims -> self_snapshot -> decision -> reflection`

对应到 MCP 工具层，当前可用的 4 个工具是：

- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`

### 2. SQLite 持久化

- 已使用文件型 SQLite
- 支持跨重启保持数据
- `AppConfig::load()` 的默认启动路径可通过 `AGENT_LLM_MM_DATABASE_URL` 显式覆盖数据库位置
- `AppConfig::load_from_path()` 会保留显式文件里的 `database_url`
- 若显式文件省略 `database_url`，`load_from_path()` 仍会通过 `AppConfig::default()` 继承环境变量派生出来的默认数据库路径

### 3. `namespace` 最小闭环

当前已支持：

- `self`
- `world`
- `user/<id>`
- `project/<id>`

并且这些语义已贯穿：

- domain 校验
- SQLite 约束
- MCP DTO 输入

### 4. 反思最小正向路径

`run_reflection` 已支持：

- 审计友好的 supersede / dispute 行为
- 显式 `replacement_evidence_event_ids`
- 一套窄化的结构化 `replacement_evidence_query` 首批能力（首版证据检索基础）
- 一条最小可用的 `identity_core` / `commitments` 深层修订路径，并把 supporting evidence 与请求的更新内容写入 reflection 审计记录
- 缺失 evidence event id 时返回 `invalid_params`

### 5. 本机接入入口

- `scripts/agent-llm-mm.sh doctor`
- `scripts/agent-llm-mm.sh serve`
- 可作为本机 MCP 子进程被 Codex 类客户端接入

### 6. `openai-compatible` provider

- 已实现 `openai-compatible` 模型适配器
- 已支持通过本地 TOML 配置文件选择 provider
- runtime 已能按配置在 `mock` 与 `openai-compatible` 间切换
- `doctor` 会输出 provider / base_url / model，但不会泄露 API key

### 7. automatic self-revision MVP

- 已新增 `self_revision` 领域契约，包含 trigger type、proposal rationale 和 machine patch 最小结构
- `ModelPort` 已支持 `propose_self_revision`
- `mock` 与 `openai-compatible` adapter 已实现最小 proposal 行为
- 已新增 trigger ledger 持久化，能记录 handled / rejected / suppressed 结果、episode watermark 和 cooldown
- 已新增 `auto_reflect_if_needed` 协调器，负责 trigger 判定、proposal 请求、治理校验和写入前收口
- 当前唯一 MCP-wired automatic path 是 `ingest_interaction -> failure trigger`；若 post-ingest auto-reflection 失败，不会把已经成功的 ingest 回滚成 MCP 错误
- 通过治理的 automatic self-revision 最终仍复用 `run_reflection` 作为 identity / commitments 的 durable write path
- 直接 `run_reflection` MCP tool 不会递归触发 auto-reflection

这代表“自动 self-revision MVP”已经存在，但它仍然是受限、保守、局部接线的 demo 能力。

## 部分实现

### 1. `decide_with_snapshot`

- commitment gate 是真实能力
- 下游模型调用已可走 `openai-compatible`
- 当前返回契约仍是“动作字符串”

因此它更适合作为最小决策闭环和集成验证能力，而不是完整决策引擎。

### 2. provider 生态

- 当前 provider 边界已经抽出来
- 但仓库内目前只实现了 `mock` 与 `openai-compatible`

### 3. `self_snapshot`

- 当前有统一 `SnapshotBudget`
- 主要控制 evidence 数量

它还不是对 `identity / commitments / claims / episodes` 分层预算的完整模型。

### 4. `episodes`

- 当前主要是 `episode_reference -> event_id` 级别的轻量聚合

它还不是带 `goal / outcome / lesson / self_effect` 的完整自传式建模。

### 5. `identity_core` / `commitments` 深层修订

- 当前已经能通过 `run_reflection` 最小更新 `identity_core`
- 当前已经能通过 `run_reflection` 最小更新 `commitments`
- 反思审计会记录 supporting evidence 与请求的更新载荷

但它仍然只是首版收口，不是 richer schema、版本化 slow-variable 层或完整策略系统。

### 6. 默认数据库作用域

- 已可稳定落盘
- 默认语义已收口为“本机用户共享的持久化默认库”
- 若需要按项目、按环境或按实验隔离，应显式配置不同的 `database_url`

### 7. self-revision 触发面与运行形态

- 当前领域层 trigger type 已覆盖 `failure / conflict / periodic`
- 当前协调器和 ledger 也能表达这些 trigger 类型
- 但当前唯一接到 MCP entry point 的 automatic path 只有 `ingest_interaction` 成功后触发的 `failure` 路径，并通过 ingest DTO 提供 `trigger_hints`
- `conflict` 与 `periodic` 目前只存在于 domain / coordinator / ledger 契约里，还没有接到 MCP entry point
- 当前没有新增单独的 auto-reflection MCP tool，也没有后台 daemon、定时调度器或“所有入口统一自动反思”的运行形态

因此，当前仓库可以准确描述成“已实现 trigger-ledger-backed automatic self-revision MVP”，但不能描述成“完整自治 self-governing agent”。

## 未实现

- richer 自动 evidence lookup（当前仅有 `owner / kind / limit` 的窄化 evidence-oriented 查询基础）
- richer evidence weighting / relation / ranking
- evidence weight / relation
- `identity_core` 的 richer schema 与版本化形成机制
- `commitments` 的 richer schema、升级 / 失效策略与更细粒度生命周期
- 更多 provider 类型
- richer `claim / episode / identity` schema
- working memory / procedural memory 的独立建模
- 持续后台自治运行、独立 daemon 与更完整的多层 memory 自治系统

## 当前验证状态

截至 `2026-04-19`，已 fresh 运行：

- `cargo test -q`
- `./scripts/agent-llm-mm.sh doctor` 或 `cargo run --quiet -- doctor`

结果：

- `application_use_cases`: 20
- `bootstrap`: 12
- `decision_flow`: 2
- `domain_invariants`: 4
- `domain_snapshot`: 6
- `failure_modes`: 11
- `mcp_stdio`: 16
- `openai_compatible_model`: 6
- `provider_config`: 5
- `sqlite_store`: 15
- 合计：97 个测试通过
- `doctor` 返回 JSON，且 `status = ok`

## 对外描述建议

如果要给协作者一句话描述当前项目，建议用下面这个口径：

“这是一个 Rust 编写的本机 MCP `stdio` memory demo，已经打通事件写入、快照构建、最小决策门控、反思修订，以及由 trigger ledger、证据门槛和慢更新约束保护的 automatic self-revision MVP；但它仍然是本地 demo，不是完整自治代理系统。”
