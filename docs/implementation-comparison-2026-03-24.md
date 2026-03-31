# 功能实现比对（2026-03-24，按 2026-03-27 实现复核更新）

> 说明：本文档是“原始设计日志”与 `2026-03-27` 时点实现之间的历史比对，不是当前仓库最新状态说明。其关于“真实模型 provider 仍未接入”等表述属于当时判断。当前稳定状态请以 [project-status.md](/D:/Code/agent_llm_mm/docs/project-status.md) 与 [README.md](/D:/Code/agent_llm_mm/README.md) 为准。

## 比对对象

本文件用于对比以下两份材料：

- 原始设计日志：[llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md)
- 当前实现说明：[current-work-2026-03-25.md](/D:/Code/agent_llm_mm/docs/current-work-2026-03-25.md)

比对基线：

- 当前分支：`master`
- 当前实现提交：`16599bfe94a41eaf9eb6efa46d8934a86e8ea7b7`

## 总结结论

当前仓库已经实现了原始设计日志中“最小可运行回路”的工程骨架：

`events -> claims -> self_snapshot -> decision -> reflection`

而且相比更早阶段，当前实现已经额外补上了两块关键能力：

- `namespace` 最小闭环
- 显式 evidence 驱动的 inferred replacement reflection

但如果把原始设计日志视为“完整自我机制”的目标，那么当前状态仍然只能定位为：

- 已完成：MVP 工程闭环、本机 MCP 接入、关键不变量、SQLite 持久化、最小 evidence-aware reflection
- 部分完成：snapshot 预算、episodes、reflection 产品语义、默认存储作用域
- 尚未完成：rich schema、identity/commitment 深层修订、多层 memory 完整形态、真实模型能力

## 一、已实现

### 1. 最小运行回路已落地

当前代码已经具备从事件到反思的最小链路：

- `ingest_interaction` 负责写入事件并派生命题
- `build_self_snapshot` 从持久化数据构建快照
- `decide_with_snapshot` 在 gate 通过后调用模型接口
- `run_reflection` 负责争议化或 supersede 现有 claim

判断：已实现。

### 2. `namespace` 最小体系已落地

这项能力在更早的比对里曾被判定为未实现，但按当前仓库复核后，这个结论已经不成立。

当前实现已经具备：

- `self`
- `world`
- `user/<id>`
- `project/<id>`

并且这些语义已经贯穿：

- domain 校验
- DTO 输入解析
- SQLite `CHECK` 约束
- legacy schema 回填

判断：已实现最小版。

### 3. `identity_core` 不能被普通 ingest 直接改写

当前 `identity_core` 仍然有非常强的保护边界：

- 普通 ingest 不能直接更新 identity
- baseline commitment 也会阻断直接写 identity 的动作

判断：已实现。

### 4. 推断型命题具备证据门槛

当前实现里：

- ingest 路径对 inferred claim 仍要求证据门槛
- reflection 路径已经支持显式 evidence event id 列表
- evidence event id 不存在时，会在应用层返回参数错误

因此“推断型内容必须有外部支撑”这条最小不变量已经成立。

判断：已实现最小版。

### 5. commitment 在行动前参与门控

当前快照会携带 commitment，`decide_with_snapshot` 在调用模型前会先过 gate：

- 冲突动作会被直接阻断
- 被阻断时不会继续调用模型
- 这条路径已经有真实 `stdio` E2E 覆盖

判断：已实现，但规则仍非常基础。

### 6. 快照构建必须带 evidence

当前 `self_snapshot` 构建要求：

- evidence 不能为空
- evidence 会去重
- evidence 会按预算截断

判断：已实现最小版。

## 二、部分实现

### 1. `self_snapshot` 已有，但预算模型还是简化版

当前只有统一 `SnapshotBudget`，主要控制 evidence 的数量，不是对多层内容分别控预算。

判断：部分实现。

### 2. `episodes` 已存在，但仍偏占位

当前 episode 语义主要体现在：

- `episode_reference`
- `episode_reference -> event_id` 关联

尚未形成原始日志中的 richer episode/autobiography 层。

判断：部分实现。

### 3. reflection 已有，但只覆盖 claim 级别修订

当前 reflection 能做的事情主要是：

- `Conflict` 时把旧 claim 标为 `Disputed`
- `Failure` 路径下 supersede 旧 claim，并可带 replacement claim
- 记录最小审计信息

它还不能：

- 修订 `identity_core`
- 修订 `commitments`
- 做周期性反思
- 做 richer reasoning

判断：部分实现。

### 4. 默认 SQLite 持久化已落地，但作用域策略未定型

当前默认路径已经是文件型 SQLite，能保证跨重启连续性，但默认语义尚未正式收口为：

- 按用户共享
- 按项目隔离
- 按 workspace 隔离

判断：部分实现。

## 三、未实现

### 1. richer schema 仍未落地

原始日志中大量 richer 字段还没有进入当前 schema，包括但不限于：

- `confidence`
- `stability`
- `valid_from`
- `valid_to`
- `priority`
- `goal`
- `outcome`
- `lesson`
- `self_effect`
- `rationale`

判断：未实现。

### 2. `identity_core` 仍不是慢变量层

当前 `identity_core` 仍更接近：

- 字符串列表
- 基线身份声明

还不是带维度、稳定性和生效区间的 slow-variable 层。

判断：未实现。

### 3. reflection 还不能真正修订 `identity_core` 或 `commitments`

当前 reflection 只会更新 claim 状态以及 replacement claim，不会形成更深层的自我修订规则。

判断：未实现。

### 4. working memory / procedural memory 仍未独立建模

当前覆盖更多集中在 episodic / semantic 的最小子集，尚未形成更完整的多层 memory 体系。

判断：未实现。

### 5. 真实模型 provider 仍未接入

`decide_with_snapshot` 仍依赖 mock model，因此不能把它视为生产级决策能力。

判断：未实现。

## 四、当前进度解读

如果只看工程闭环，当前进度已经明显超过“概念验证”阶段：

- 本机 `stdio` MCP 服务已可启动
- SQLite 已可落盘
- `doctor` / `serve` 已可用
- 工具暴露面稳定
- 自动化 E2E 已覆盖关键链路

但如果对照原始设计日志的完整目标，当前仍处于“骨架完整、语义未满”的阶段。更准确的说法是：

“原始设计日志的 MVP 工程化落地已经完成，但 richer memory semantics 和产品语义仍在后续规划中。”

## 五、最终结论

当前分支的准确定位应当是：

“一个具备 namespace、持久化、MCP `stdio` 接入、evidence-aware reflection 最小正向路径和自动化验证的 self-agent memory MVP”

而不是：

“原始设计日志的完整实现”

因此，说明文档应该明确标注：

- 哪些能力已经实现
- 哪些能力只是最小化版本
- 哪些能力仍是后期规划

只有这样，文档才不会对当前实现边界做出过度承诺。
