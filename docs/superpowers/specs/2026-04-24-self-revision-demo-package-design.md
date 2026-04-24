# Self-Revision Demo Package Design

**日期：** 2026-04-24  
**状态：** Draft for review  
**语言：** 简体中文

---

## 1. 文档目的

这份文档定义一套可重复、可审计、可直观看结果的 demo package，用来证明当前仓库已经具备下面两类效果：

- memory 会持久化，并影响后续 decision
- 在满足显式触发条件时，automatic self-revision MVP 会运行，并把修订写回 durable state

这个 demo package 不是在仓库里新增一套产品 UI，也不是把当前项目包装成完整自治代理系统。它的目标是用一条真实可重放的运行链路，向技术同事和外部协作者证明：

`events -> snapshot -> auto self-revision -> updated commitments -> later decision shift`

---

## 2. 一句话方案

第一版默认新增一套 **macOS 优先、零外网依赖、单 scenario 重放** 的演示包：

- 用本地 deterministic `openai-compatible` stub provider 替代真实外部模型
- 用现有 4 个 MCP tool 跑完整链路，不新增演示专用 MCP tool
- 用 SQLite durable state + MCP 输出双重取证
- 同一次运行同时产出：
  - 一键终端演示
  - 手工复演文档
  - Markdown 报告
  - 原始 JSON / SQL 摘要

---

## 3. 成功标准

只有同时满足下面 6 条，才算这个 demo package 成立：

1. 从空的临时 SQLite 库开始，不依赖已有状态。
2. `doctor` 能明确展示当前 runtime hook coverage 和 durable write path。
3. baseline snapshot 能稳定展示 memory 已真实落盘，而不是只存在于进程内。
4. 能展示一条“有冲突内容但不满足 explicit trigger 条件，因此不会 auto-reflect”的负例。
5. 能展示一条“满足 explicit conflict trigger 条件，因此发生 handled auto self-revision”的正例。
6. 能展示“同一个系统在 snapshot before / after 下，后续 decision 表现发生变化”。

---

## 4. 非目标

这次设计明确不做下面这些事情：

- 不新增 Web UI 或长期运行的 HTTP 产品界面
- 不把项目表述成完整自治 agent
- 不依赖真实外部模型或外网 API key
- 不扩展新的 MCP tool
- 不把 automatic self-revision 说成“所有入口默认自动运行”
- 不引入 richer evidence ranking、后台 daemon 或新的自治调度层

---

## 5. Demo 总体结构

这套 demo package 由 5 类内容组成。

### 5.1 一键 runner

主入口放在：

- `scripts/run-self-revision-demo.sh`

作用：

- 创建临时 SQLite 库
- 启动本地 deterministic stub provider
- 生成演示配置
- 驱动 MCP stdio 链路
- 查询 SQLite 结果
- 生成终端摘要和 Markdown 报告

第一版以 macOS shell 入口为主。Windows parity 不在这次最小闭环内。

### 5.2 本地 stub provider

第一版默认新增一个本地 helper，而不是依赖测试文件内部的 helper：

- `src/bin/demo_openai_compatible_stub.rs`

作用：

- 提供本地 `chat/completions` 风格 endpoint
- 对 self-revision proposal 返回固定、可审计的 JSON
- 对 decision 请求根据 snapshot commitments 返回不同 decision

不新增任何第三方 Node 依赖；实现语言保持 Rust，以减少运行前提和跨文件复制测试 helper 的成本。

### 5.3 演示配置模板

第一版默认新增：

- `examples/agent-llm-mm.demo.example.toml`

作用：

- 指向本地 stub provider
- 使用演示专用 `database_url`
- 明确这是 demo 配置，不和正式接入配置混用

### 5.4 用户可读文档

第一版默认新增两份文档：

- `docs/self-revision-demo-guide-2026-04-24.md`
- `docs/reports/self-revision-demo-2026-04-24.md`

前者是操作手册，后者是 canonical 报告样例。

### 5.5 运行时产物目录

运行时生成内容写到：

- `target/reports/self-revision-demo/<timestamp>/`

运行时产物默认至少包含：

- `doctor.json`
- `snapshot-before.json`
- `snapshot-after.json`
- `decision-before.json`
- `decision-after.json`
- `timeline.json`
- `sqlite-summary.json`
- `report.md`

仓库内长期保存的是 guide 和 canonical report，临时运行稿保存在 `target/`。

---

## 6. Canonical Scenario

这次只定义一条 canonical scenario，不做多故事线扩展。

### 6.1 Step 0: 环境初始化与边界确认

执行：

- 创建新的临时 SQLite 文件
- 启动本地 stub provider
- 生成演示专用配置
- 运行 `doctor`

要证明的内容：

- 当前运行的是本地 `stdio` MCP 服务
- 当前 automatic path 只有文档里声明的 4 条：
  - `ingest_interaction:failure`
  - `ingest_interaction:conflict`
  - `decide_with_snapshot:conflict`
  - `build_self_snapshot:periodic`
- 当前 durable write path 仍然只有 `run_reflection`

主要证据：

- `doctor.json`

### 6.2 Step 1: 建立 baseline memory

执行：

- 调用 `ingest_interaction` 写入最小 baseline event / claim / episode
- 调用 `build_self_snapshot`

baseline 内容固定为：

- 一个 self-scoped claim：`self.role is architect`
- 一个 episode reference，用于证明 episode 聚合可见
- 至少 1 个 evidence event

要证明的内容：

- memory 已真实落盘
- snapshot 能回读出 identity / commitments / claims / evidence / episodes
- baseline commitment `forbid:write_identity_core_directly` 存在

主要证据：

- MCP `build_self_snapshot` 的 `result.structuredContent.snapshot`
- SQLite 表：
  - `events`
  - `claims`
  - `identity_claims`
  - `commitments`
  - `evidence_links`
  - `episode_events`

### 6.3 Step 2: 先展示真实 gate 存在

执行：

- 使用 `snapshot_before`
- 调用 `decide_with_snapshot`
- `action = "write_identity_core_directly"`

要证明的内容：

- 当前决策链路不是空壳
- baseline commitment gate 会真实阻断 forbidden action
- blocked 决策不会再去请求模型

预期结果：

- `blocked = true`
- `decision = null`

主要证据：

- MCP `decide_with_snapshot` 返回值

### 6.4 Step 3: 负例，冲突内容但没有 explicit conflict hints

执行：

- 调用 `ingest_interaction`
- `event.summary = "self attempted a conflicting commitment overwrite"`
- 不传 `trigger_hints`

要证明的内容：

- 当前 automatic self-revision 是受限 MVP，不是“有冲突文本就一定自动修订”
- event 写入成功，不会因为没触发 auto-reflection 而失败
- 这时不应出现 conflict ledger handled 记录

预期结果：

- ingest 成功，返回 `event_id`
- `reflection_trigger_ledger` 无新增 conflict handled 记录
- `reflections` 无新增由该步引发的记录

### 6.5 Step 4: 正例，显式 conflict hints 触发 handled auto self-revision

执行：

- 再调一次 `ingest_interaction`
- `event.summary = "self attempted a commitment overwrite that requires confirmation"`
- `trigger_hints = ["conflict", "commitment"]`

stub provider 在这一步返回固定 proposal：

- `should_reflect = true`
- `rationale = "Conflict evidence suggests tighter commitment hygiene."`
- `machine_patch.commitment_patch.commitments = ["prefer:confirm_conflicting_commitment_updates_before_overwrite"]`

要证明的内容：

- MCP 表面返回仍然只是 ingest success
- automatic self-revision 实际发生了
- 反思结果已被转译回 `run_reflection` durable write path

预期结果：

- ingest 成功，返回 `event_id`
- `reflection_trigger_ledger` 新增一条：
  - `trigger_type = "conflict"`
  - `status = "handled"`
- `reflections` 新增一条审计记录
- 审计记录包含：
  - `summary`
  - `requested_commitment_updates`

主要证据：

- SQLite 表：
  - `reflection_trigger_ledger`
  - `reflections`
  - `commitments`

### 6.6 Step 5: 构建 `snapshot_after` 并做 diff

执行：

- 再次调用 `build_self_snapshot`
- 对 `snapshot_before` 与 `snapshot_after` 做结构化 diff

要证明的内容：

- commitment 变化已经进入 durable state
- 变化不只是审计表里记录了 patch，而是后续 snapshot 真能看到
- baseline commitment `forbid:write_identity_core_directly` 仍被保留

预期 diff：

- `snapshot_after.commitments` 比 `snapshot_before.commitments` 多出：
  - `prefer:confirm_conflicting_commitment_updates_before_overwrite`
- 同时仍保留：
  - `forbid:write_identity_core_directly`

### 6.7 Step 6: 决策变化的组合式证明

这一段必须同时展示两类证据。

#### A. 硬证据：gate 继续阻断 forbidden action

执行：

- 使用 `snapshot_after`
- 再次调用 `decide_with_snapshot`
- `action = "write_identity_core_directly"`

作用：

- 证明 baseline guardrail 仍然存在
- 说明 new commitment 并没有破坏已有安全边界

预期结果：

- `blocked = true`
- `decision = null`

#### B. 软证据：允许通过 gate 的 action，在 before / after 下得到不同 decision

执行：

- 使用同一个允许通过 gate 的 `action`
- `action = "review_conflicting_commitment_update"`
- 分别用 `snapshot_before` 和 `snapshot_after` 调 `decide_with_snapshot`

stub provider 对 decision 请求使用下面的 deterministic 规则：

- 若 snapshot commitments 不包含 `prefer:confirm_conflicting_commitment_updates_before_overwrite`
  - 返回 `decision.action = "apply_commitment_update_now"`
- 若 snapshot commitments 包含该 commitment
  - 返回 `decision.action = "confirm_conflicting_commitment_updates_before_overwrite"`

要证明的内容：

- 同一个 `decide_with_snapshot` 契约在不同 snapshot 下会得到不同结果
- 变化来源是 durable memory state 的差异，而不是脚本手工写死一段展示文案

预期结果：

- `decision_before.blocked = false`
- `decision_after.blocked = false`
- `decision_before.decision.action != decision_after.decision.action`

---

## 7. 取证模型

这个 demo 不能只看 MCP 表面输出，必须采用双层取证。

### 7.1 MCP 可直接证明的内容

可直接从 `structuredContent` 证明：

- `ingest_interaction` 成功并返回 `event_id`
- `build_self_snapshot` 返回 snapshot
- `decide_with_snapshot` 返回 `blocked` 和 `decision`

### 7.2 必须靠 SQLite 证明的内容

automatic self-revision 的 handled / rejected / suppressed 状态当前不会直接出现在 MCP structured output 里，因此必须查库确认：

- `reflection_trigger_ledger`
- `reflections`
- `commitments`

### 7.3 关键表与字段

#### Baseline memory

- `events(event_id, recorded_at, owner, kind, summary)`
- `claims(claim_id, owner, namespace, subject, predicate, object, mode, status)`
- `identity_claims(position, claim)`
- `commitments(description, owner)`
- `evidence_links(claim_id, event_id)`
- `episode_events(episode_reference, event_id)`

#### Auto self-revision audit

- `reflection_trigger_ledger(trigger_type, trigger_key, status, namespace, reflection_id)`
- `reflections(reflection_id, recorded_at, summary, supporting_evidence_event_ids, requested_commitment_updates)`

### 7.4 报告里的证据口径

报告必须明确标记每条结论来自：

- MCP output
- SQLite durable state
- 或两者联合

不能把 SQLite 观察到的结果说成“tool 直接返回了这些诊断”。

---

## 8. Stub Provider 设计

### 8.1 设计原则

- 本地运行
- deterministic
- 零外网依赖
- 同时支持 self-revision proposal 与 decision 请求

### 8.2 Self-revision 分支

当请求语义对应 conflict self-revision proposal 时，返回固定 JSON proposal。

第一版不追求通用智能，只追求：

- 能稳定命中 `ingest_interaction -> conflict`
- 能把 commitment patch 写回现有 durable path

### 8.3 Decision 分支

对 decision 请求，根据 snapshot commitments 是否包含：

- `prefer:confirm_conflicting_commitment_updates_before_overwrite`

来选择返回不同 action string。

### 8.4 不做的事

- 不模拟 richer ranking
- 不模拟多轮 reasoning
- 不模拟 streaming
- 不对所有 prompt 模式做通用兼容

---

## 9. 报告结构

报告主格式采用 `Markdown + Mermaid`。

### 9.1 固定章节

1. Demo 目标
2. 环境与边界
3. Scenario 时间线
4. Baseline snapshot
5. 负例：无 explicit hints，不触发 auto-reflection
6. 正例：explicit conflict hints，触发 handled auto self-revision
7. Snapshot diff
8. Decision proof A：gate 阻断
9. Decision proof B：allowed action 的 decision before / after 变化
10. 结论与限制

### 9.2 固定可视化元素

- Mermaid timeline 或 flowchart
- before / after 对比表
- 关键 JSON 摘要代码块
- commitment diff 列表

### 9.3 HTML 策略

仓库内长期事实单一来源是 Markdown。

如果需要更直观的本地查看体验，可以把 Markdown 渲染成临时 HTML，但 HTML 只作为运行时产物输出到 `target/`，不作为主文档格式提交。

---

## 10. 文件规划

第一版的最小文件集合如下。

### 10.1 新增文件

- `src/bin/demo_openai_compatible_stub.rs`
- `scripts/run-self-revision-demo.sh`
- `examples/agent-llm-mm.demo.example.toml`
- `docs/self-revision-demo-guide-2026-04-24.md`
- `docs/reports/self-revision-demo-2026-04-24.md`

### 10.2 按实现范围更新的现有文档

实现完成后，如果入口、验证方式或能力边界对用户可见，至少更新：

- `README.md`
- `docs/project-status.md`
- `docs/roadmap.md`
- `docs/development-macos.md`
- `docs/testing-guide-2026-03-24.md`

如果实现范围最终只影响 demo / verification 路径，也至少要更新：

- `README.md`
- `docs/testing-guide-2026-03-24.md`
- `docs/development-macos.md`

---

## 11. 验证要求

实现完成后，至少需要验证下面这些点：

1. runner 从空数据库开始可以完整跑通。
2. 不需要外部 API key。
3. `doctor` 输出中的 runtime hooks 与文档一致。
4. 无 explicit conflict hints 的负例不会误触发 auto-reflection。
5. 有 explicit conflict hints 的正例会产生 handled ledger + reflection audit。
6. `snapshot_after` 中存在新增 commitment，且 baseline commitment 未丢失。
7. decision proof A 显示 gate 阻断。
8. decision proof B 显示 allowed action 的 decision before / after 变化。

---

## 12. 风险与默认决策

### 12.1 风险：MCP 层不会直接返回 auto-reflection 诊断

默认决策：

- 报告直接写明“这部分取证来自 SQLite durable state”
- 不尝试伪造一个 MCP 层的即时诊断字段

### 12.2 风险：`decide_with_snapshot` 的业务契约很小

默认决策：

- 用“硬证据 + 软证据”双展示方式弥补表现力不足
- 不把它表述成完整决策引擎

### 12.3 风险：测试 helper 直接搬到生产脚本会引入耦合

默认决策：

- 复用测试里的行为模式
- 但用单独的 demo helper / binary 落地，不直接 `include!` 或复制测试支撑层到运行脚本

### 12.4 风险：报告容易退化成静态摆拍

默认决策：

- 所有关键结论都必须能在运行时产物里回溯到 JSON 或 SQL 摘要
- canonical report 必须由 runner 结果生成或刷新，不手写伪造结果

---

## 13. 结论

这次 demo package 的核心不是“再做一个功能”，而是把仓库当前已经存在但不够直观的能力，整理成一套对内对外都能成立的证据链：

- 先证明 baseline memory 和 gate 真实存在
- 再证明 explicit conflict trigger 会带来 handled auto self-revision
- 然后证明 updated commitments 已进入 durable state
- 最后证明这些变化会反映到后续 decision

这条证据链完全基于当前项目的真实能力边界，不依赖夸大表述，也不依赖外部模型的不稳定行为。
