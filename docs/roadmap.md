# 路线图

本路线图面向 GitHub 协作与后续开发沟通，强调“当前已承诺什么、下一步先做什么、哪些还只是中长期方向”。

当前已收口的一项基础语义是：默认 SQLite 路径表示“本机用户共享的持久化默认库”；如果需要正式数据、测试数据或实验数据隔离，应显式设置不同的 `database_url`。

另一项当前已落地但必须谨慎表述的能力是：仓库已经具备 trigger-ledger-backed automatic self-revision MVP。不过，这个能力当前仍是本地 `stdio` demo 里的受限自动修订链路，不是完整自治系统。

## 近期

### 1. 收口 release gate 文档

目标：

- 把现有测试说明进一步整理成更接近发布门槛的操作手册
- 明确最小验证集和建议验证集

原因：

- 当前已有测试基础
- 但公开协作时，还需要更稳定的“提交前 / 发布前”规则

### 2. 扩大 automatic self-revision 的 MCP runtime coverage

目标：

- 先把当前已落地的 3 条 MCP runtime hook 文档、验证与诊断口径收口稳定：
  - `ingest_interaction -> failure`
  - `decide_with_snapshot -> conflict`
  - `build_self_snapshot -> periodic`
- 在不新增旁路持久化接口的前提下，优先验证这些 hook 的 opt-in 条件、best-effort 失败语义和排查路径，而不是继续扩大到“所有入口自动反思”

重点边界：

- `decide_with_snapshot` 已可走 `openai-compatible` provider，但仍不是完整决策引擎
- 当前 MCP-wired automatic path 只有上述 3 条，不代表所有 MCP entry point 都会自动反思
- `decide_with_snapshot` 与 `build_self_snapshot` 当前仍要求显式 `auto_reflect_namespace`
- `run_reflection` 仍是唯一 durable write path；没有新增旁路持久化接口
- direct `run_reflection` 不递归 auto-reflection；没有后台 daemon 或“所有入口自动反思”
- richer memory semantics 尚未落地

### 3. 提高 self-revision 的可观测性与治理精度

目标：

- 保持 `run_reflection` 为长期写入路径，同时继续收紧 trigger 候选、cooldown 与 slow-update 规则
- 明确 best-effort auto-reflection 的 structured diagnostics、日志排查方式和运行时边界
- 让 ledger / trigger / rejection / suppression 的诊断信息更容易在验证和接入文档里被复用

原因：

- 当前 MVP 已证明“ledger + governed proposal + `run_reflection` durable write path”这条路径可行
- 下一步的价值不在于新建更多接口，而在于减少误触发、补强排查路径，并为后续扩大触发面提供更稳的观测基础

### 4. 收口 reflection 的 deeper-update 契约

目标：

- 在已支持显式 `replacement_evidence_event_ids`、结构化 `replacement_evidence_query` 和最小 `identity_core` / `commitments` 更新的基础上，继续明确输入约束、保底规则与审计边界
- 在 self-revision proposal 已支持 `proposed_evidence_event_ids`、`proposed_evidence_query` 与 `confidence` 首阶段契约的基础上，继续明确这些字段的治理边界
- 为后续 richer schema、版本化策略和更广泛证据能力奠基

原因：

- 反思路径的最小 deep-update 闭环已经落地，下一步不应再引入新的接口漂移，而应优先把规则与边界收口清楚
- `proposed_evidence_query` 当前仍只是首阶段 evidence contract，不应被扩写成自动 widening / ranking engine
- 这将减少后续 evidence 语义与 schema 改造的反复成本

## 中期

### 1. 扩展更多 provider 类型

目标：

- 在现有 provider 枚举与 provider-specific config 结构上继续增加更多 provider
- 保持应用层与领域层不感知第三方协议细节

候选方向：

- Azure OpenAI
- OpenRouter
- 本地模型网关

### 2. 把 reflection 从“显式输入 + 可选查询 + 最小 deep update”推进到 richer 语义

目标：

- 保留显式 `replacement_evidence_event_ids`
- 保留现有最小 `identity_core` / `commitments` 更新能力
- 在现有窄化查询基础上扩展 richer query 语义、更广泛 evidence 能力、更清晰的权重/关联关系与更稳定的 deep-update policy

### 3. 丰富 evidence 语义

目标：

- 不再只判断 event 是否存在
- 逐步引入更清晰的 evidence-oriented 查询与关联能力

### 4. 丰富 `episodes` / `identity_core` / claim schema

目标：

- 把当前骨架式数据结构逐步推进到更有表达力的形态
- 让“状态快照”更接近原始设计里的语义目标

## 后期

### 1. 扩展到更完整的 memory 分层

目标：

- working memory
- episodic memory
- semantic memory
- procedural memory
- 以及更完整的 slow variable / policy / self-model layering

### 2. 评估更完整的产品化封装

候选方向：

- 更清晰的隔离策略
- 更丰富的 transport
- 更稳定的配置与部署方式

## 当前不纳入近期承诺的事项

- 远程 HTTP 服务
- 多租户或多 Agent 编排
- 可视化管理后台
- 持续后台自治运行或完整 daemon 化自我治理
- 把仓库包装成“生产级完整自我机制产品”
