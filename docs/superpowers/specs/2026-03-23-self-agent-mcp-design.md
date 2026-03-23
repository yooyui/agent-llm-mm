# Self Agent MCP Design

**日期：** 2026-03-23
**状态：** Draft approved in conversation, pending file review
**语言：** 简体中文

---

## 1. 目标

构建一个基于 Rust 的最小可行服务化 Agent，以 MCP Server 形式对外暴露能力，并将“长期记忆 + 自我机制”的核心约束落到可运行的工程结构中。

第一版重点不是构建一个通用聊天机器人，而是验证以下能力可以在统一运行时内闭环工作：

- 原始事件先入账，再做高层抽取
- 高层命题必须可追溯到原始证据
- 身份核心只能慢更新，不能被普通写入直接覆盖
- 承诺必须在决策前参与门控
- 反思只在冲突、失败或显式触发时运行

---

## 2. 本轮范围

### 2.1 In Scope

- Rust 实现的 MCP Server
- 函数式风格的纯核心实现
- 存储抽象层，默认 SQLite 适配器
- 模型端口抽象层，默认 Mock Model 适配器
- 单 Agent 实例运行时
- 场景级 MCP tools
- 围绕失败模式和系统不变量的测试

### 2.2 Out of Scope

- 多 Agent 协同与身份路由
- 真实 LLM provider 绑定
- 分布式部署
- Web UI
- 复杂权限系统
- 全量事件重放式 CQRS 或完整 event sourcing

---

## 3. 关键设计决策

### 3.1 服务接口

首版采用 MCP Server，而不是 REST API。

原因：

- 目标系统本身面向 Agent 工具调用
- 更贴近后续 Agent 运行时接入场景
- 首版不需要为了通用客户端兼容额外维护 HTTP 契约

### 3.2 编程模型

采用 `Functional Core + Imperative Shell`。

原因：

- 领域规则天然适合写成纯函数和显式状态变换
- 能把协议、存储、模型调用等副作用隔离在边缘
- 更容易验证失败模式和不变量

### 3.3 存储策略

采用“存储抽象 + 默认 SQLite”。

原因：

- SQLite 最适合 MVP 快速落地
- trait 抽象可以为 PostgreSQL 或其他后端保留扩展点
- 不需要在第一版提前引入部署复杂度

### 3.4 模型策略

定义模型端口，但首版默认使用 Mock Model。

原因：

- 保留后续接入真实模型的演进路径
- 避免把第一版复杂度拉到 provider 兼容和 prompt 调优上
- 测试可重复、可控

### 3.5 实例策略

首版只支持单 Agent 实例，但数据模型保留 `agent_id` 和 `namespace` 扩展点。

原因：

- 符合 YAGNI
- 避免过早做多租户或多身份编排
- 不阻断后续扩展到多实例

---

## 4. 总体架构

首版系统拆分为 4 层。

### 4.1 Domain

纯领域模型与纯规则函数，不直接依赖 MCP、SQLite、时钟或模型 provider。

包含对象：

- `events`
- `claims`
- `evidence_links`
- `identity_core`
- `commitments`
- `episodes`
- `reflections`
- `self_snapshot`

包含规则：

- 候选命题提取后的归类与状态迁移
- 证据门槛与追溯关系校验
- 冲突检测与状态变更
- 快照组装与预算裁剪
- 承诺门控
- 反思触发与修订决策

### 4.2 Application

负责用例编排，不包含底层存储细节和协议细节。

核心用例：

- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`

### 4.3 Ports

定义所有副作用边界，领域和应用层只依赖抽象接口。

建议端口：

- `EventStore`
- `ClaimStore`
- `IdentityStore`
- `CommitmentStore`
- `EpisodeStore`
- `ReflectionStore`
- `ModelPort`
- `Clock`
- `IdGenerator`

### 4.4 Adapters

负责具体集成。

适配器包括：

- SQLite 存储适配器
- Mock Model 适配器
- MCP 接口适配器
- 配置加载
- 日志与追踪

---

## 5. 目录结构

首版采用单 crate，可执行服务，目录建议如下：

```text
src/
  main.rs
  domain/
    event.rs
    claim.rs
    evidence.rs
    identity.rs
    commitment.rs
    episode.rs
    reflection.rs
    snapshot.rs
    rules/
      ingest.rs
      evidence.rs
      conflict.rs
      snapshot_builder.rs
      commitment_gate.rs
      reflection_policy.rs
  application/
    ingest_interaction.rs
    build_self_snapshot.rs
    decide_with_snapshot.rs
    run_reflection.rs
  ports/
    event_store.rs
    claim_store.rs
    identity_store.rs
    commitment_store.rs
    episode_store.rs
    reflection_store.rs
    model_port.rs
    clock.rs
    id_generator.rs
  adapters/
    sqlite/
    model/
  interfaces/
    mcp/
  support/
    config.rs
    tracing.rs
tests/
```

拆分原则：

- `domain` 只保留纯逻辑
- `application` 只负责编排
- `ports` 只定义抽象边界
- `adapters` 只承接副作用
- 不提前拆 workspace，避免过度设计

---

## 6. 数据模型

### 6.1 总体原则

- 每张表都带 `agent_id`
- 可变对象必须带 `status`
- 时间统一为 UTC
- 半结构化内容允许使用 JSON 文本字段
- 索引只作为加速层，不承载事实语义

### 6.2 `events`

作用：原始事实底账，保存输入、输出、观察、工具结果。

关键字段：

- `event_id`
- `agent_id`
- `ts`
- `session_id`
- `actor`
- `owner`
- `mode`
- `content`
- `source_ref`
- `hash`

约束：

- 只允许追加写入
- 正文不可被无痕改写

### 6.3 `claims`

作用：从事件中提炼出的原子命题。

关键字段：

- `claim_id`
- `agent_id`
- `namespace`
- `subject`
- `predicate`
- `object`
- `kind`
- `confidence`
- `stability`
- `status`
- `valid_from`
- `valid_to`

约束：

- 推断型命题需要更高证据门槛
- 命题冲突通过状态流转处理，而不是删除旧记录

### 6.4 `evidence_links`

作用：连接高层命题与原始事件。

关键字段：

- `claim_id`
- `event_id`
- `relation`
- `weight`

约束：

- 高层命题必须可追到至少一条原始证据

### 6.5 `identity_core`

作用：慢变量层，承载角色、价值排序、长期原则、边界信息。

关键字段：

- `core_id`
- `agent_id`
- `dimension`
- `statement`
- `confidence`
- `stability_score`
- `status`
- `effective_from`
- `effective_to`

约束：

- 普通 ingest 不能直接改写
- 只能通过 reflection 产生新版本并关闭旧版本

### 6.6 `commitments`

作用：把过去形成的规则和承诺转成未来行动的约束。

关键字段：

- `commitment_id`
- `agent_id`
- `text`
- `priority`
- `hardness`
- `scope`
- `activation_condition`
- `expiry_condition`
- `status`
- `source_ref`

约束：

- 必须参与行动前门控
- 不能只作为事后解释信息

### 6.7 `episodes`

作用：将离散事件聚合成自传式情景单元。

关键字段：

- `episode_id`
- `agent_id`
- `start_ts`
- `end_ts`
- `goal`
- `action_summary`
- `outcome`
- `lesson`
- `self_effect`

约束：

- 只保存“目标-行动-结果-教训-自我影响”的闭环

### 6.8 `reflections`

作用：记录自我修订、承诺修订和冲突仲裁过程。

关键字段：

- `reflection_id`
- `agent_id`
- `trigger`
- `input_refs`
- `old_refs`
- `new_refs`
- `decision`
- `rationale`
- `ts`

约束：

- 任何核心修订都必须留下 reflection 审计记录

---

## 7. 运行时闭环

首版固定 4 个应用用例，对外映射为 4 个 MCP tools。

### 7.1 `ingest_interaction`

职责：

- 接收一次交互或观察
- 写入原始事件
- 提取候选命题
- 执行归类、证据门槛和冲突检查
- 写入命题、证据链接与必要的情景聚合

顺序：

1. 参数校验与规范化
2. 生成 `events`
3. 追加写入 `events`
4. 提取候选 `claims`
5. 归类 `owner / mode / namespace / stability / status`
6. 运行证据门槛与冲突检测
7. 写入 `claims` 与 `evidence_links`
8. 必要时生成或更新 `episodes`

关键约束：

- `events` 永远先写
- 即使提取失败，底账也必须保留

### 7.2 `build_self_snapshot`

职责：

- 面向当前任务组装一份规模受控、可追溯的决策快照

顺序：

1. 读取有效 `identity_core`
2. 读取激活的 `commitments`
3. 按相关性检索 `claims`
4. 为高层摘要回拉至少一条原始证据
5. 补充相关 `episodes`
6. 应用预算限制，形成最终 `self_snapshot`

关键约束：

- 高层摘要召回时，必须伴随原始证据
- 快照预算必须受控，避免上下文污染

### 7.3 `decide_with_snapshot`

职责：

- 在承诺门控之后调用模型端口，生成结构化行动建议

顺序：

1. 运行 `commitment_gate`
2. 生成门控结果
3. 将 `self_snapshot + task + gate_result` 交给 `ModelPort`
4. 输出结构化决策结果
5. 需要时将结果追加为 `draft` 或 `acted` 事件

关键约束：

- 决策前必须执行承诺门控
- 模型瞬时输出不能直接写入长期自我层

### 7.4 `run_reflection`

职责：

- 在冲突、重大失败或显式触发时，执行自我修订和冲突仲裁

顺序：

1. 拉取相关 `events / claims / commitments / identity_core`
2. 判断冲突类型
3. 生成结构化修订决策
4. 写入 `reflections`
5. 更新相关对象状态
6. 必要时写入新的 `identity_core` 版本

关键约束：

- 只允许通过新增版本和状态变更修订核心层
- 不允许无痕覆盖历史

---

## 8. MCP Tool 设计

首版对外暴露以下 4 个场景级 tools：

- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`

响应结构建议统一为：

- `ok`
- `data`
- `warnings`
- `errors`
- `trace_id`

这样可以减少客户端处理复杂度，也方便后续调试与审计。

---

## 9. 错误模型

### 9.1 `ValidationError`

适用于：

- 字段缺失
- 类型错误
- 枚举值非法
- 时间格式非法
- JSON 结构非法

处理原则：

- 在接口层直接返回
- 不进入领域流程

### 9.2 `DomainError`

适用于：

- `owner` 与 `namespace` 不一致
- 推断型 claim 证据不足
- 试图直接改写 `identity_core`
- 承诺门控阻断
- reflection 输入不足

处理原则：

- 保持稳定错误码
- 供 MCP 客户端自动处理

### 9.3 `InfrastructureError`

适用于：

- SQLite 读写失败
- 序列化失败
- 模型端口不可用
- 时钟或 ID 生成适配器故障

处理原则：

- 在适配器层捕获并转换
- 不污染领域语义

---

## 10. 测试策略

### 10.1 纯函数单元测试

覆盖：

- 命题归类
- 状态迁移
- 证据门槛
- 冲突检测
- 快照组装与预算裁剪
- 承诺门控
- 反思策略

目标：

- 高密度、低成本验证核心规则

### 10.2 不变量测试

覆盖：

- `events` 只能追加
- `identity_core` 不能被普通 ingest 直接改写
- 高层命题至少有一条可追溯证据
- `decide_with_snapshot` 前必经 `commitment_gate`
- 高层摘要召回必须伴随原始证据
- `superseded` 对象不能再次作为 `active` 返回

目标：

- 把架构硬约束变成可执行测试

### 10.3 集成测试

基于临时 SQLite 数据库与 MCP Server 验证完整链路：

- ingest 后可见事件、命题、证据链接
- snapshot 能返回受控结果集
- decide 会被承诺门控影响
- reflection 会产生版本与状态变化，而不是覆盖旧数据

### 10.4 失败模式场景测试

首版至少落以下场景：

- 归属混淆
- 摘要漂移
- 自我漂移
- 承诺空转
- 近期劫持

目标：

- 让失败模式测试成为活文档

---

## 11. 非功能性要求

- 默认 UTF-8 编码
- 明确区分纯逻辑与副作用
- 数据结构尽量不可变
- 错误必须可枚举、可追踪
- 工具输出应具备审计友好性

---

## 12. 后续演进方向

如果 MVP 跑通，下一步可按以下顺序扩展：

1. 接入真实模型 provider
2. 支持多 Agent 实例
3. 引入更强的检索与排序策略
4. 增加 REST 或其他服务接口
5. 逐步向更强的事件溯源模型演进

---

## 13. 推荐结论

第一版最终采用如下方案：

- Rust 单 crate MCP Server
- `Functional Core + Imperative Shell`
- trait 驱动的存储与模型端口
- 默认 SQLite
- 默认 Mock Model
- 单 Agent 实例
- 暴露 4 个场景级 MCP tools
- 以失败模式和系统不变量为测试中心

这套方案在 KISS、YAGNI、DRY 与可扩展性之间取得了当前阶段最合理的平衡。
