# 当前工作说明（2026-03-25，按 2026-03-27 实现复核更新）

> 说明：本文档保留为 `2026-03-27` 视角下的实现快照。其关于“真实模型 provider 尚未接入”“测试总数为 50”等结论属于当时判断，不代表当前主线最新状态。当前稳定入口请以 [README.md](/D:/Code/agent_llm_mm/README.md)、[project-status.md](/D:/Code/agent_llm_mm/docs/project-status.md) 和 [roadmap.md](/D:/Code/agent_llm_mm/docs/roadmap.md) 为准。

## 概览

- 当前分支：`master`
- 当前提交基线：`16599bfe94a41eaf9eb6efa46d8934a86e8ea7b7`
- 当前状态：工作树干净，已按当前仓库状态完成源码复核与全量测试复核
- 运行形态：Rust 单 crate，MCP `stdio` 服务，架构为 `Functional Core + Imperative Shell`

本说明不再只描述单一开发轮次，而是汇总截至 `2026-03-27` 的当前实现状态，用于回答三个问题：

1. 当前仓库到底已经实现了什么
2. 还停留在哪些最小化或占位式语义
3. 后续应该优先往哪里继续推进

## 当前进度判断

如果把整体工作拆成三个层次，当前进度可以这样判断：

- 工程 MVP 闭环：已完成
  - `events -> claims -> self_snapshot -> decision -> reflection` 的最小链路已经打通
- 本机 MCP 接入：已完成
  - `stdio`、SQLite、`doctor` / `serve`、自动化 E2E 都已经可用
- 完整产品语义：部分完成
  - 目前仍是“可验证的最小闭环”，不是原始设计里更丰富的完整自我机制

## 已实现

### 1. `namespace` 已在 domain / SQLite / MCP 三层闭环

当前仓库已经不是“只有 `Owner`、没有 `Namespace`”的状态，而是具备最小可用的 namespace 体系：

- domain 层已有 `Namespace`
- 支持 `self`
- 支持 `world`
- 支持 `user/<id>`
- 支持 `project/<id>`
- `ClaimDraft` 已校验 `owner <-> namespace` 兼容关系
- SQLite `claims` 表有数据库级 `CHECK` 约束
- MCP DTO 已支持显式传入 `namespace`

因此，“owner/namespace 的最小路由语义”已经实现，不再属于待办项。

### 2. `run_reflection` 已支持显式 evidence 驱动的 inferred replacement

当前 `run_reflection` 已具备下面这些语义：

- `ReflectionInput` 支持 `replacement_evidence_event_ids`
- 当 replacement claim 为 `Mode::Inferred` 时，不再一律 fail-closed
- 只要显式 evidence event id 列表满足校验条件，就允许 inferred replacement
- replacement claim 成功写入后，会同步写入 `evidence_links`
- 传入不存在的 event id 时，会在应用层返回 `invalid_params`

这意味着 reflection 已经从“只有拒绝分支”推进到“显式 evidence 驱动的最小正向分支”。

### 3. SQLite 规则已收敛到单一来源

当前 SQLite 适配层中，`owner <-> namespace` 的关键 SQL 规则已集中到 `schema` 模块：

- `claims` 表建表 SQL 通过共享 builder 生成
- legacy `claims` 迁移时的 namespace 回填表达式也来自共享定义
- `store` 层不再内嵌重复 SQL 片段

这降低了 schema / migration / store 三处漂移的风险，并且有测试锁住。

### 4. MCP `stdio` 接入链已经可用

当前仓库已经具备完整的本机接入骨架：

- `scripts/agent-llm-mm.ps1` 支持 `doctor` / `serve`
- `doctor` 会验证 SQLite bootstrap 和 runtime 初始化
- `serve` 会启动 MCP `stdio` 服务
- 当前 MCP 暴露 4 个工具：
  - `ingest_interaction`
  - `build_self_snapshot`
  - `decide_with_snapshot`
  - `run_reflection`

### 5. baseline commitment gate 已真实生效

当前 `decide_with_snapshot` 并不是完全空壳：

- snapshot 会带上 commitments
- baseline commitment 已可从 fresh runtime 读出
- forbidden action 会在模型调用前被 gate 阻断
- 对应行为已由真实 `stdio` E2E 覆盖

## 部分实现

### 1. `decide_with_snapshot` 仍然依赖 mock model

当前决策链分成两段：

- 前半段：真实 gate
- 后半段：mock 决策

因此它更适合作为流程调试和集成验证能力，而不是可以正式对外承诺的决策引擎。

### 2. `self_snapshot` 预算模型仍是简化版

当前的 `SnapshotBudget` 只有一个统一上限，实际效果主要体现在：

- evidence 去重
- evidence 截断
- 至少保留一条 evidence

它还不是对 `identity / commitments / claims / episodes` 分别施加预算的完整模型。

### 3. `episodes` 仍是轻量引用层

当前实现里，episode 更多是：

- 一个 `episode_reference`
- 关联若干 `event_id`

它还不是原始设计中更完整的 autobiography/episode 层，目前没有：

- `goal`
- `action_summary`
- `outcome`
- `lesson`
- `self_effect`

### 4. 默认数据库路径已可用，但作用域语义未定型

当前默认配置已经从“内存库”推进为“文件型 SQLite”：

- 重启后状态可持续
- 可以用环境变量覆盖数据库路径
- 本机接入已具备落盘基础

但还没有把默认路径正式定义为：

- 按用户共享
- 按项目隔离
- 按 workspace 隔离

## 未实现

下面这些能力仍然没有实现，文档中不应把它们写成“已经具备”：

- 自动 evidence lookup
- evidence weight / relation 等 richer evidence 语义
- reflection 对 `identity_core` 的形成或修订
- reflection 对 `commitments` 的重写、失效或升级
- richer `identity_core` schema
- richer `claim` / `episode` / `reflection` schema
- working memory / procedural memory 的独立建模
- 真实 LLM provider

## 本地验证结果

以下验证已在当前仓库状态上 fresh 运行通过：

- `cargo test`

测试结果摘要：

- `application_use_cases`: 11 passed
- `bootstrap`: 9 passed
- `decision_flow`: 2 passed
- `domain_invariants`: 4 passed
- `domain_snapshot`: 6 passed
- `failure_modes`: 3 passed
- `mcp_stdio`: 7 passed
- `sqlite_store`: 8 passed

合计：50 个测试通过。

当前最关键的回归点包括：

- `sqlite_owner_namespace_sql_rules_have_single_source`
- `reflection_accepts_inferred_replacement_with_explicit_evidence`
- `reflection_rejects_missing_replacement_evidence_event_ids`
- `inferred_replacement_reflection_with_evidence_is_accepted_over_stdio`
- `missing_replacement_evidence_event_ids_are_invalid_params_over_stdio`
- `fresh_stdio_runtime_blocks_forbidden_action_with_seeded_commitment`

## 当前结论

如果评价标准是：

- namespace 最小闭环是否已落地
- evidence-aware reflection 是否已有显式输入驱动的正向路径
- SQLite / MCP `stdio` / 脚本入口是否已具备本机可接入的最小稳定性

那么当前答案是：已经达到，并且有自动化验证支撑。

如果评价标准提升为“是否已经实现完整自我机制产品语义”，答案仍然是否定的。当前仓库更准确的定位是：

“一个已完成工程闭环和本机接入闭环的 self-agent memory MVP”，而不是“原始设计目标的完整实现”。

## 后期规划

### 近期优先项

1. 明确默认 SQLite 路径的作用域语义，并补自动化测试
2. 把测试文档继续升级为更接近 release gate 的操作说明
3. 明确哪些工具可以作为正式接入能力，哪些继续保留为实验能力

### 中期演进项

1. 把 reflection 从“显式 evidence 输入”推进到“显式输入 + 可选查询”
2. 把 `EventStore::has_event` 扩展为更明确的 evidence-oriented 查询接口
3. 逐步丰富 `identity_core`、`episodes` 与相关 schema

### 后期规划

1. 接入真实模型 provider，替换 mock 决策路径
2. 补 working memory / procedural memory 的独立层次
3. 评估更丰富的 transport、配置与数据隔离策略是否值得进入产品化路线
