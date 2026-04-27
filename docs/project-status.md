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
- proposal 首阶段已支持 `proposed_evidence_event_ids`、`proposed_evidence_query` 与 `confidence`；这些字段当前用于收口证据候选与置信度，其中 `proposed_evidence_query` 在 explicit ids 为空时可作为 bounded narrowing hint，对当前 trigger window 做交集收口，并在有交集时按当前窗口内的候选顺序应用 `limit`；若没有交集则回退到 full trigger window。explicit ids 非空时，这些 ids 也必须满足 query 在当前 trigger window 内的过滤约束，但不代表 richer widening / ranking engine 已落地
- 已新增 trigger ledger 持久化，能记录 handled / rejected / suppressed 结果、episode watermark 和 cooldown，并通过 structured diagnostics 暴露 trigger / rejection / suppression / cooldown 信息
- 已新增 `auto_reflect_if_needed` 协调器，负责 trigger 判定、proposal 请求、治理校验和写入前收口
- 当前 MCP-wired automatic path 已谨慎扩到 4 条：`ingest_interaction -> failure`、`ingest_interaction -> conflict`、`decide_with_snapshot -> conflict`、`build_self_snapshot -> periodic`
- `ingest_interaction` / `decide_with_snapshot` / `build_self_snapshot` 上的 best-effort auto-reflection 失败都不会把已经成功的 MCP 主路径改写成额外的 MCP 错误
- 通过治理的 automatic self-revision 最终仍复用 `run_reflection` 作为 identity / commitments 的 durable write path
- 直接 `run_reflection` MCP tool 不会递归触发 auto-reflection

这代表“自动 self-revision MVP”已经存在，但它仍然是受限、保守、局部接线的 demo 能力。

### 8. self-revision demo package

- 已新增 deterministic `openai-compatible` stub provider binary
- 已新增 demo runner binary，复用真实 MCP `stdio` 服务和现有 4 个 MCP tool 跑 canonical scenario
- 已新增 macOS shell wrapper：`./scripts/run-self-revision-demo.sh`
- 运行后会生成 `doctor.json`、snapshot before / after、decision before / after、timeline、SQLite summary 和 Markdown report
- 该 demo 只证明当前 MVP 的可重复证据链，不新增 MCP tool、daemon、Web UI 或新的 durable write path

### 9. Production dashboard service

- 已支持通过 `[dashboard]` 配置随 `serve` 启动只读 HTTP 面板
- 面板展示运行时 operation 事件，并保持 MCP `stdio` 输出不被污染
- 当前 UI 为 `Memory-chan Live Desk`，使用内嵌生成图物料与 CSS 装饰复刻清新活力二次元观测面板
- 生成图物料位于 `src/interfaces/dashboard/static/`，版权/归属说明已记录在 `NOTICE`
- 当前事件记录为 bounded in-memory recorder，不是 durable operation-log database

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
- 当前协调器和 ledger 也已接通这 3 类 trigger 的最小 runtime coverage
- 当前 MCP-wired automatic path 只有这 4 条，而且需要按各自边界显式触发：
  - `ingest_interaction -> failure`
  - `ingest_interaction -> conflict`
  - `decide_with_snapshot -> conflict`
  - `build_self_snapshot -> periodic`
- `ingest_interaction -> conflict` 仍要求显式 `trigger_hints` 包含 `conflict` 或 `identity`
- `decide_with_snapshot` 与 `build_self_snapshot` 仍要求调用方显式传 `auto_reflect_namespace`
- `decide_with_snapshot` 的 conflict auto-reflection 还要求显式 conflict-compatible `trigger_hints`，并且只会在非 blocked 决策后 best-effort 运行，不会改变原有 decision payload 形状
- 当前没有新增单独的 auto-reflection MCP tool，也没有后台 daemon、定时调度器或“所有入口统一自动反思”的运行形态

因此，当前仓库可以准确描述成“已实现 trigger-ledger-backed automatic self-revision MVP”，但不能描述成“完整自治 self-governing agent”。

## 未实现

- richer 自动 evidence lookup（当前 `replacement_evidence_query` / `proposed_evidence_query` 仍仅有 `owner / kind / limit` 的窄化 evidence-oriented 查询基础）
- richer evidence weighting / relation / ranking
- evidence weight / relation
- `identity_core` 的 richer schema 与版本化形成机制
- `commitments` 的 richer schema、升级 / 失效策略与更细粒度生命周期
- 更多 provider 类型
- richer `claim / episode / identity` schema
- working memory / procedural memory 的独立建模
- 持续后台自治运行、独立 daemon 与更完整的多层 memory 自治系统

## 当前验证状态

截至 `2026-04-27`，已 fresh 运行：

- `cargo fmt --check`
- `git diff --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `./scripts/agent-llm-mm.sh doctor` 或 `cargo run --quiet --bin agent_llm_mm -- doctor`
- `cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v`
- `./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest`

结果：

- `application_use_cases`: 20
- `bootstrap`: 15
- `dashboard_config`: 4
- `dashboard_http`: 2
- `dashboard_projection`: 2
- `dashboard_recorder`: 2
- `decision_flow`: 2
- `domain_invariants`: 4
- `domain_snapshot`: 6
- `demo_openai_compatible_stub`: 1
- `failure_modes`: 27
- `mcp_stdio`: 27
- `openai_compatible_model`: 7
- `provider_config`: 5
- `self_revision_demo_runner`: 2
- `sqlite_store`: 17
- 合计：143 个测试通过
- `doctor` 返回 JSON，且 `status = ok`
- self-revision demo package 生成 8 个本地 artifact，并证明 before / after decision shift

## 对外描述建议

如果要给协作者一句话描述当前项目，建议用下面这个口径：

“这是一个 Rust 编写的本机 MCP `stdio` memory demo，已经打通事件写入、快照构建、最小决策门控、反思修订，以及由 trigger ledger、证据门槛和慢更新约束保护的 automatic self-revision MVP；但它仍然是本地 demo，不是完整自治代理系统。”
