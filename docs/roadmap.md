# 路线图

本路线图面向 GitHub 协作与后续开发沟通，强调“当前已承诺什么、下一步先做什么、哪些还只是中长期方向”。

## 近期

### 1. 收口默认 SQLite 作用域语义

目标：

- 明确默认数据库路径到底代表“按用户共享”还是“按项目隔离”
- 把该语义写进文档与回归测试

原因：

- 这是当前最容易影响真实使用方式的边界
- 也是协作者最容易误解的地方

### 2. 收口 release gate 文档

目标：

- 把现有测试说明进一步整理成更接近发布门槛的操作手册
- 明确最小验证集和建议验证集

原因：

- 当前已有测试基础
- 但公开协作时，还需要更稳定的“提交前 / 发布前”规则

### 3. 明确能力承诺边界

目标：

- 哪些能力可以对外说“已经可用”
- 哪些能力只能说“实验 / demo / internal validation”

重点边界：

- `decide_with_snapshot` 已可走 `openai-compatible` provider，但仍不是完整决策引擎
- richer memory semantics 尚未落地

## 中期

### 1. 扩展更多 provider 类型

目标：

- 在现有 provider 枚举与 provider-specific config 结构上继续增加更多 provider
- 保持应用层与领域层不感知第三方协议细节

候选方向：

- Azure OpenAI
- OpenRouter
- 本地模型网关

### 2. 把 reflection 从显式输入推进到“显式输入 + 可选查询”

目标：

- 保留显式 `replacement_evidence_event_ids`
- 同时探索更合适的 evidence 查询接口

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
