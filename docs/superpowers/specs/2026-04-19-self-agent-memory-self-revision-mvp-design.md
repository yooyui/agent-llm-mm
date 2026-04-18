# Self-Agent Memory Self-Revision MVP Design

**日期：** 2026-04-19  
**状态：** Draft for review  
**语言：** 简体中文

---

## 1. 文档目的

这份文档不是重新从零发明一个系统，而是把两部分内容整合成一份可读的实现初稿：

- `2026-03-23` 原始讨论逐轮日志中的核心约束
- 本轮围绕“最小可自我修订的 self-agent memory MVP”进行的再次确认

目标不是一次性把 richer memory semantics 做完，而是在当前仓库已经具备的最小闭环之上：

`events -> claims -> self_snapshot -> decision -> reflection`

再往前推进一层，得到一个“会自动触发、会提出修订、但仍受服务端治理”的自我修订 MVP。

---

## 2. 一句话方案

在现有 `run_reflection` 之上增加一个受治理的自动反思协调层：

- 服务端自动检测 `conflict / failure / periodic` 三类触发
- 由模型负责“是否反思”判断和“修订提案”生成
- 由服务端负责证据校验、慢更新约束、版本化展开、去重和审计
- 所有实际写入仍复用 `run_reflection`，不允许模型直接越过审计路径改库

这个 MVP 更准确的定位是：

“一个具备自动触发、模型提案、服务端治理和可追溯审计的 self-revision memory demo。”

---

## 3. 这轮整合后的默认结论

下面这些结论视为本轮探讨后的默认方案，不再拆成前面对话里的互斥选项。

### 3.1 触发方式

- 自动触发，不依赖调用方显式手工调用 `run_reflection`
- 触发类型包含：
  - `conflict`
  - `failure`
  - `periodic`

### 3.2 触发入口

- 所有对外 MCP 入口都可以在进入主逻辑前执行自动触发检查
- 但内部自动反思路径必须带防递归保护，避免 `run_reflection -> 自动触发检查 -> 再次反思` 的递归循环

### 3.3 触发识别方式

采用分层识别，而不是单一策略：

1. 显式 trigger hints 优先
2. 结构化规则判断补位
3. `summary` 文本规则优先
4. 规则未命中时，再让模型做补位分类

这意味着第一版不是“纯规则”，也不是“完全自由语义推断”，而是一个可审计的混合触发模型。

### 3.4 修订范围

- 自动修订 `identity_core`
- 自动修订 `commitments`

但两者的门槛不同：

- `identity_core` 走严格慢更新门槛
- `commitments` 走较低但仍需要 resolved evidence 的门槛

### 3.5 提案方式

模型负责两层输出：

- `human-readable rationale`
- `machine patch`

但 `machine patch` 不是直接执行权限，只是提案表达能力。

### 3.6 Patch 权限解释

第一版采用“受治理的全 patch 权限”：

- 模型可以提出较完整的增删改 patch
- 服务端不能直接照 patch 覆盖存储
- 所有删除或替换都必须被翻译成版本化状态迁移与 reflection 审计记录

### 3.7 周期性触发策略

`periodic` 采用整合版规则：

- 按 `namespace` 分桶
- 同时设置最小时间间隔
- 通过持久化 `trigger ledger` 去重和记录水位

### 3.8 Namespace 影响范围

- `self`
  - 可完整进入 periodic 自动反思
  - 可修订 `identity_core + commitments`
- `project/<id>`
  - 可进入 periodic 自动反思
  - 主要作用于 project-scoped commitments
  - 可作为 self 修订的辅助证据来源
- `user/<id>`
  - 可进入 periodic 整理
  - 可影响 user-scoped 记忆
  - 在证据充分时可影响 commitments
  - 不能单独直接改写 `identity_core`
  - 只允许作为 self identity 修订的间接证据来源
- `world`
  - 第一版不进入 periodic 自动反思

---

## 4. 仍然必须遵守的原始约束

本轮方案虽然更自动，但不改变原始日志中的核心硬约束。

### 4.1 `identity_core` 只能通过 reflection 改

不能由普通提取器或普通事件写入直接覆盖。

### 4.2 反思必须是触发式，而不是每轮运行

自动触发不等于无条件频繁触发。触发检查可以处处存在，但真正运行反思必须受到去重、冷却和阈值控制。

### 4.3 冲突时旧内容不能直接消失

不允许通过覆盖写入抹掉旧版本。旧内容只能进入：

- `disputed`
- `superseded`
- `expired`

并且要保留版本关系和修订痕迹。

### 4.4 `identity_core` 必须慢更新

第一版的主制动器采用三层组合：

- 证据阈值
- 冷却期
- 单次变更上限

---

## 5. MVP 要解决的真实问题

当前仓库已经能做“被调用的 reflection 修订”，但还缺一层：

- 系统不会自己判断“现在该不该反思”
- 系统不会自己提出“该如何修订 identity / commitments”
- 系统也没有防止多入口重复触发的去重账本

所以这次 MVP 的重点不是重做 `reflection`，而是补一个上层协调器，把已有能力连成自我修订回路。

---

## 6. 总体架构增量

### 6.1 新增一个自动反思协调用例

建议新增应用层用例，例如：

- `auto_reflect_if_needed`
- 或 `run_self_revision_cycle`

职责：

- 汇总显式 hints、结构化信号和文本/模型分类结果
- 生成触发候选
- 查 `trigger ledger` 判重和检查冷却条件
- 收集证据窗口与相关 snapshot
- 调模型生成修订提案
- 做服务端治理校验
- 转译为 `run_reflection` 输入
- 提交写入并登记 ledger

### 6.2 保持 `run_reflection` 为唯一写路径

`run_reflection` 继续承担这些职责：

- 版本化 claim 状态迁移
- `identity_core` 更新
- `commitments` 更新
- reflection 审计落库

自动反思协调器只是上层编排者，不直接替代它。

### 6.3 MCP 层接入方式

第一版不要求新建完全独立的 MCP 主入口才能成立。

建议方式：

- 现有 MCP 工具进入主逻辑前先执行 `auto_reflect_if_needed`
- 对 `ingest_interaction` 和 `decide_with_snapshot` 提供可选 `trigger_hints`
- 对 `build_self_snapshot` 这类偏读接口，只执行基于持久化状态的挂起触发检查
- 对 `run_reflection` 本身增加防递归保护，不在内部再次自动触发新的反思

---

## 7. 触发模型

### 7.1 `conflict`

第一版的 `conflict` 来源建议包括：

- 显式传入 `conflict` hint
- 新旧 claims 的结构化冲突
- 对既有 commitments 或 identity 的明显反例
- 文本规则命中“冲突/纠正/推翻/反例/不再成立”等模式
- 规则未命中时，由模型做补位分类

### 7.2 `failure`

第一版把失败分成强触发和弱触发两档：

- 强触发：
  - `hard commitment violation`
  - 明确回滚
  - 同一窗口内重复失败达到阈值
- 弱触发：
  - 一次普通任务失败
  - 一次普通偏差事件

弱触发先入 ledger，不立刻改 self；当同类失败累计到阈值后，再升级为强触发。

### 7.3 `periodic`

`periodic` 使用 namespace 分桶 + 最小时间间隔：

- 每个 `namespace` 单独统计自上次 periodic reflection 以来新增的已关闭 `episodes`
- 当某个桶新增 `episodes >= 阈值`，且距离该桶上次 periodic reflection 超过最小时间间隔，才允许触发

建议首版默认值：

- `episode_threshold = 20`
- `min_interval = 24h`

这些值在 MVP 阶段可先作为配置常量，而不是复杂策略系统。

---

## 8. Trigger Ledger 设计

第一版应新增持久化 `trigger ledger`，而不是依赖内存冷却。

### 8.1 目的

- 防止同一批证据在多个 MCP 入口上重复触发
- 为 periodic 提供水位和时间间隔控制
- 让自动反思仍然可追溯

### 8.2 最小记录内容

建议至少记录：

- `ledger_id`
- `trigger_type`
- `namespace`
- `trigger_key`
- `evidence_window`
- `status`
- `last_seen_at`
- `handled_at`
- `reflection_id`
- `cooldown_until`
- `episode_watermark`

### 8.3 状态建议

- `pending`
- `handled`
- `rejected`
- `suppressed`

这样既能记录触发过，也能记录“因为冷却期、证据不足或重复而未实际执行”的情况。

---

## 9. 模型输出契约

模型不直接写库，只输出“反思判定 + 修订提案”。

### 9.1 第一层：人类可读解释

建议包含：

- `should_reflect`
- `trigger_type`
- `rationale`
- `summary_of_evidence`
- `risk_notes`

### 9.2 第二层：机器可执行 patch

建议包含：

- `identity_patch`
- `commitment_patch`
- `proposed_evidence_event_ids`
- `proposed_evidence_query`
- `confidence`

### 9.3 Patch 的服务端解释原则

- `identity_patch` 允许表达新增、降级、替换候选
- `commitment_patch` 允许表达新增、替换、失效候选
- 所有 `remove` 类操作都必须被服务端翻译成版本化状态迁移，而不是物理删除

---

## 10. 服务端治理规则

### 10.1 `identity_core` 的严格门槛

第一版建议最少卡住下面这些约束：

- 至少 3 条一致性 claims
- 跨至少 2 个 sessions 或 episodes
- 没有高置信冲突未解决
- 命中 trigger ledger 的冷却约束
- 单次 identity 变更量不得超过上限

可将“单次变更上限”先收口成：

- 单次 reflection 最多变更 `1-2` 条 identity 项

### 10.2 `commitments` 的较低门槛

`commitments` 可以比 `identity_core` 更容易修订，但仍需：

- 至少一个 resolved evidence
- 不违反 baseline commitments
- 不绕过 commitment gate 的已有保底规则

### 10.3 删除和替换的统一规则

不允许模型或服务端直接删除长期层对象：

- `identity_core` 的移除必须转为 supersede/dispute 关系
- `commitments` 的移除必须转为 supersede/expire 关系

### 10.4 证据不足时的处理

- 不能满足门槛的 proposal 不应写入长期层
- 但应在 ledger 或 reflection 审计中留下“被拒绝的自动修订尝试”记录

这样可以避免系统出现“总在想改自己，但没有留下痕迹”的黑箱行为。

---

## 11. 建议的数据模型增量

### 11.1 新增表：`reflection_trigger_ledger`

作用：

- 自动触发判重
- 周期性水位控制
- 自动反思状态跟踪

### 11.2 扩展 `reflections` 审计字段

当前 `reflections` 已有基础审计能力。为了支持本 MVP，建议再补最小自动反思审计字段，例如：

- `trigger_type`
- `trigger_namespace`
- `trigger_source`
- `proposal_rationale`
- `proposal_patch_json`
- `proposal_confidence`

这里不要求第一版一定做成丰富 schema；可以先用 JSON 文本字段承载补充审计内容。

---

## 12. MVP 运行回路

第一版建议形成下面这条闭环：

1. MCP 请求进入
2. `auto_reflect_if_needed` 读取输入 hints 与当前持久化状态
3. 生成触发候选并查询 `trigger ledger`
4. 若触发被 suppress，则直接继续原请求
5. 若触发成立，则构建局部 `self_snapshot` 与证据窗口
6. 调模型输出：
   - 是否反思
   - 反思理由
   - machine patch
7. 服务端校验：
   - 证据是否存在
   - namespace 是否允许
   - identity 是否满足慢更新门槛
   - commitments 是否满足较低门槛
   - patch 是否需要转译为 supersede/dispute/expire
8. 把通过的 proposal 转成 `run_reflection` 输入
9. 调用 `run_reflection` 提交
10. 更新 `trigger ledger`
11. 再进入原始 MCP 请求主逻辑

这个顺序的重点是：

- 自动反思先作为“治理动作”运行
- 原始业务请求继续保留
- 整个系统仍然是本地 MCP `stdio` demo，而不是后台自治系统

---

## 13. 第一版明确不做的内容

为了把 MVP 做实，下面这些内容明确不纳入本轮：

- 完整 multi-layer memory 重构
- richer evidence relation / weight
- 基于自由文本的大范围开放式人格归纳
- `world` namespace 的 periodic 自动反思
- 复杂版本化策略 DSL
- 模型直接写库
- 无 ledger 的自由自动反思

---

## 14. 对现有仓库的最小改动思路

这份设计稿偏向增量演进，而不是推倒重来。

建议实现顺序是：

1. 新增 trigger ledger 存储抽象与 SQLite 适配
2. 新增自动反思协调用例
3. 新增模型输出 DTO 与服务端校验逻辑
4. 把提案转译成现有 `run_reflection` 输入
5. 在 MCP 入口串入触发检查与防递归保护
6. 补 conflict/failure/periodic 的回归测试
7. 补文档和测试基线

---

## 15. 这份初稿的定位

这是一份“探讨记录整合后的实现方案初稿”，不是最终定稿。

它的作用是：

- 让本轮讨论从离散问答收口成一个统一设计面
- 把关键术语、边界和默认选择一次写清
- 为后续实现计划和子任务拆分提供稳定起点

如果这份稿子的大方向正确，下一步再基于它拆详细实现计划会更稳。
