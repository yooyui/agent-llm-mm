# 路线图

本路线图面向 GitHub 协作与后续开发沟通，强调“当前已承诺什么、下一步先做什么、哪些还只是中长期方向”。

当前已收口的一项基础语义是：默认 SQLite 路径表示“本机用户共享的持久化默认库”；如果需要正式数据、测试数据或实验数据隔离，应显式设置不同的 `database_url`。

## 近期

### 1. 收口 release gate 文档

目标：

- 把现有测试说明进一步整理成更接近发布门槛的操作手册
- 明确最小验证集和建议验证集

原因：

- 当前已有测试基础
- 但公开协作时，还需要更稳定的“提交前 / 发布前”规则

### 2. 明确能力承诺边界

目标：

- 哪些能力可以对外说“已经可用”
- 哪些能力只能说“实验 / demo / internal validation”

重点边界：

- `decide_with_snapshot` 已可走 `openai-compatible` provider，但仍不是完整决策引擎
- richer memory semantics 尚未落地

### 3. 收口 reflection 的 deeper-update 契约

目标：

- 在已支持显式 `replacement_evidence_event_ids`、结构化 `replacement_evidence_query` 和最小 `identity_core` / `commitments` 更新的基础上，继续明确输入约束、保底规则与审计边界
- 为后续 richer schema、版本化策略和更广泛证据能力奠基

原因：

- 反思路径的最小 deep-update 闭环已经落地，下一步不应再引入新的接口漂移，而应优先把规则与边界收口清楚
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
- 在现有窄化查询基础上扩展 richer query 语义、更广泛 evidence 能力与更稳定的 deep-update policy

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

### 2. 评估更完整的产品化封装

候选方向：

- 更清晰的隔离策略
- 更丰富的 transport
- 更稳定的配置与部署方式

## 当前不纳入近期承诺的事项

- 远程 HTTP 服务
- 多租户或多 Agent 编排
- 可视化管理后台
- 把仓库包装成“生产级完整自我机制产品”
