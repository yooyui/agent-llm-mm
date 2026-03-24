# 功能实现比对（2026-03-24）

## 比对对象

本文件用于对比以下两份材料：

- 原始设计日志：[llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md)
- 当前实现说明：[current-work-2026-03-24.md](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md)

比对基线：

- 当前工作分支：`codex/self-agent-mcp`
- 当前实现提交：`d23af005b50297cf557b9eede2f080fd1672b1a0`

## 总结结论

当前分支已经实现了原始设计日志中“最小可运行回路”的核心骨架，即：

`events -> claims -> episodes -> self_snapshot -> decision -> reflection`

但当前实现仍然是最小闭环，不是原始日志里描述的“完整自我机制”。如果把原始日志看作完整目标，那么当前状态更准确的定位是：

- 已完成：最小运行链路、关键不变量、MCP stdio 暴露、SQLite 持久化、门控决策、基础反思路径
- 部分完成：snapshot 控制、episode 建模、reflection 策略、持久化作用域
- 尚未完成：namespace 体系、丰富 schema、identity/commitment 真正反思修订、多层 memory 体系、程序性记忆

因此，[current-work-2026-03-24.md](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md) 作为“当前分支实现状态说明”是成立的；但如果把它理解为“原始设计目标完成说明”，则仍然偏乐观。

## 一、已实现能力

### 1. 最小运行回路已落地

原始日志给出的最小运行链路见：

- [raw-log#L1155](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1155) 到 [raw-log#L1161](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1161)

当前实现中的对应位置：

- 事件写入与 claim 派生：[ingest_interaction.rs:66](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/ingest_interaction.rs#L66)
- snapshot 组装：[build_self_snapshot.rs:20](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/build_self_snapshot.rs#L20)
- 决策前门控：[decide_with_snapshot.rs:20](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/decide_with_snapshot.rs#L20)
- 反思写回：[run_reflection.rs:40](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/run_reflection.rs#L40)

判断：已实现。

### 2. `identity_core` 不能被普通 ingest 直接改写

原始日志要求：

- [raw-log#L1140](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1140)

当前实现：

- [identity_core.rs:18](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/identity_core.rs#L18) 到 [identity_core.rs:22](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/identity_core.rs#L22)

判断：已实现，而且边界清晰。

### 3. 推断型命题具备证据门槛

原始日志要求：

- [raw-log#L1139](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1139)

当前实现：

- [claim.rs:41](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/claim.rs#L41) 到 [claim.rs:46](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/claim.rs#L46)

补充说明：

- ingest 路径会对 draft claim 做证据校验：[ingest_interaction.rs:51](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/ingest_interaction.rs#L51)
- reflection 路径已收紧为 fail-closed：[run_reflection.rs:56](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/run_reflection.rs#L56)

判断：已实现最小版，但不是最终理想语义。

### 4. commitment 在行动前参与门控

原始日志要求：

- [raw-log#L1142](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1142)

当前实现：

- snapshot 携带 commitment：[build_self_snapshot.rs:27](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/build_self_snapshot.rs#L27)
- gate 判断：[commitment_gate.rs:8](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/rules/commitment_gate.rs#L8)
- 具体冲突规则：[conflict.rs:1](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/rules/conflict.rs#L1)
- 决策前阻断：[decide_with_snapshot.rs:27](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/decide_with_snapshot.rs#L27)

判断：已实现，但规则仍是单条 baseline 规则，不是完整 commitment 系统。

### 5. snapshot 构建必须带 evidence

原始日志要求：

- [raw-log#L1147](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1147)

当前实现：

- snapshot 验证与 evidence 去重保底：[snapshot_builder.rs:6](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/rules/snapshot_builder.rs#L6)

判断：已实现最小版，当前是“必须带 evidence reference”，还不是“回拉结构化原始证据”。

## 二、部分实现能力

### 1. `self_snapshot` 已有，但预算模型还是简化版

原始日志期望：

- [raw-log#L1141](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1141)

当前实现：

- [snapshot.rs:3](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/snapshot.rs#L3)
- [snapshot_builder.rs:8](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/rules/snapshot_builder.rs#L8)

差异：

- 当前只有统一的 `SnapshotBudget`
- 实际仅用于 evidence 截断
- 没有对 identity、commitments、claims、episodes 分别施加独立预算

判断：部分实现。

### 2. `episodes` 已存在，但还不是原始日志里的“自传闭环”

原始日志期望：

- [raw-log#L1114](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1114)
- [raw-log#L1144](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1144)

当前实现：

- 领域对象：[episode.rs:3](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/episode.rs#L3)
- 持久化层实际只有 `episode_reference -> event_id` 关联：[schema.rs:28](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/adapters/sqlite/schema.rs#L28)

差异：

- 目前没有 `goal/action_summary/outcome/lesson/self_effect`
- 也没有完整 episode 聚合流程
- 当前 snapshot 里只有 episode 引用字符串

判断：只有占位式实现。

### 3. reflection 机制已存在，但只覆盖 claim 冲突与 supersede

原始日志期望：

- [raw-log#L1145](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1145)
- [raw-log#L1161](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1161)

当前实现：

- 触发与决策枚举：[reflection_policy.rs:1](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/rules/reflection_policy.rs#L1)
- 反思应用逻辑：[run_reflection.rs:40](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/run_reflection.rs#L40)

差异：

- 支持 `Conflict`、`Failure`、`Manual`
- 不支持原始日志中的周期性反思
- 不会修订 `identity_core`
- 不会修订 `commitments`
- 反思记录也只有最简 `summary`

判断：部分实现，且明显偏最小化。

### 4. 默认 SQLite 持久化已落地，但作用域策略未定型

当前实现说明在 [current-work-2026-03-24.md#L25](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md#L25) 到 [current-work-2026-03-24.md#L33](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md#L33) 已说明默认持久化行为。

当前代码：

- [config.rs:25](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/support/config.rs#L25)

差异：

- 当前默认路径是固定 temp 文件
- 具备跨重启连续性
- 但尚未定义为“按用户共享”还是“按项目隔离”

判断：已可用，但不是最终产品语义。

## 三、未实现能力

### 1. `namespace` 体系未落地

原始日志明确要求：

- [raw-log#L1128](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1128)
- [raw-log#L1138](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1138)

当前实现：

- 只有粗粒度 `Owner`：[types.rs:1](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/types.rs#L1)

未实现点：

- `self`
- `user/<id>`
- `project/<id>`
- `world`

这类可路由 namespace 当前都不存在。

### 2. 丰富 schema 基本未落地

原始日志为各层对象定义了较丰富字段：

- [raw-log#L1109](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1109) 到 [raw-log#L1115](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1115)

当前 SQLite schema：

- [schema.rs:1](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/adapters/sqlite/schema.rs#L1) 到 [schema.rs:51](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/adapters/sqlite/schema.rs#L51)

缺失的代表性字段包括：

- `namespace`
- `kind`
- `confidence`
- `stability`
- `valid_from` / `valid_to`
- `hardness`
- `priority`
- `scope`
- `activation_condition`
- `expiry_condition`
- `goal`
- `outcome`
- `lesson`
- `self_effect`
- `trigger`
- `input_refs`
- `decision`
- `rationale`

### 3. `identity_core` 仍是字符串列表，不是慢变量层

原始日志期望：

- [raw-log#L1112](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1112)

当前实现：

- [identity_core.rs:3](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/domain/identity_core.rs#L3)

未实现点：

- `dimension`
- `confidence`
- `stability_score`
- `status`
- `effective_from`
- `effective_to`

### 4. reflection 还不能真正修订 `identity_core` 或 `commitments`

原始日志要求：

- [raw-log#L1140](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1140)
- [raw-log#L1161](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L1161)

当前实现：

- `run_reflection` 只更新 claim 状态和 replacement claim：[run_reflection.rs:56](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/src/application/run_reflection.rs#L56)

未实现点：

- identity_core 形成或修订规则
- commitments 重写或失效规则
- 周期阈值触发反思

### 5. 四层 memory 模型只覆盖了部分层

原始日志的四层划分见：

- [raw-log#L468](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L468)
- [raw-log#L487](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L487)
- [raw-log#L506](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L506)
- [raw-log#L526](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md#L526)

当前覆盖情况：

- `episodic / semantic`：有最小子集
- `working memory`：未单独建模
- `procedural memory`：基本未落地

## 四、对当前工作说明文档的判断

[current-work-2026-03-24.md](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md) 在以下方面是准确的：

- 已清楚记录最后一轮修复项
- 已清楚记录当前实现边界
- 已承认后续仍需补 reflection evidence-aware、默认 DB 作用域、baseline policy 上移等事项

尤其这些位置和本次比对高度一致：

- 反思路径证据约束：[current-work-2026-03-24.md#L14](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md#L14)
- 默认 stdio SQLite：[current-work-2026-03-24.md#L25](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md#L25)
- commitment gate：[current-work-2026-03-24.md#L35](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md#L35)
- 后续建议：[current-work-2026-03-24.md#L145](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/current-work-2026-03-24.md#L145)

但这份文档没有系统列出与原始日志之间的结构性差距，尤其没有显式强调：

- `namespace` 缺失
- 丰富 schema 缺失
- `episodes` 仍非自传闭环
- reflection 不能修订 `identity_core` / `commitments`
- working/procedural memory 未落地

因此，这份说明文档适合作为“当前分支状态说明”，不适合作为“原始设计目标完成说明”。

## 五、最终结论

如果对照原始日志看，当前实现已经到达：

- 一个可信的最小可运行闭环
- 一个有测试、有持久化、有真实 stdio E2E 的工程骨架
- 一个已把关键错误路径封住的最小 self-agent memory 原型

但距离原始日志中的完整目标，仍至少还差下面几块：

1. `namespace` 与归属隔离
2. 更丰富的 memory schema
3. 完整 episode / autobiography 层
4. evidence-aware reflection
5. reflection 驱动的 identity / commitment 修订
6. 更完整的多层 memory 体系

所以，当前分支的准确定位应当是：

“原始设计日志的 MVP 工程化落地”，而不是“原始设计日志的完整实现”。
