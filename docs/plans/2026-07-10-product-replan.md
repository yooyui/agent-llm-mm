# MCP Memory Ledger 全新项目规划

状态：`active / planning complete / implementation not started`
规划日期：`2026-07-10`
代码基线：`dev-work@6fcbb5f`
写入边界：本文件只定义后续执行，不代表任何代码能力已经改变或任何发布 gate 已通过。

## 1. 总体决策

这次规划不是推倒重写，而是重新排序。

MCP Memory Ledger 已经拥有可运行的 Rust + SQLite + MCP `stdio` 核心、受治理的 reflection 写路径和大量测试资产。当前问题不是“功能太少”，而是产品价值、运行时事实和发布工程之间失衡：写入与治理工具已经很多，但可信的按 scope 读取、检索、解释、纠错和恢复仍不完整。

新的主线是：

> 先把项目做成一个可信、可检索、可审计、可恢复的本地 Agent Memory Ledger，再讨论自治、远程和团队能力。

因此做出以下决策：

1. 公共名称继续使用 **MCP Memory Ledger**；`agent_llm_mm` / `agent-llm-mm` 只保留为实现兼容标识。
2. 第一目标用户是单机开发者和本地 AI 客户端用户，不是团队管理员或自主 Agent 平台运营者。
3. 第一产品闭环是“记录 → 检索 → 查看证据 → 纠错/撤销 → 备份恢复”，不是“自动决策 → 后台自治”。
4. MCP `stdio` 保持主传输。远程 HTTP、认证和团队模式不进入近期承诺。
5. `decide_with_snapshot`、automatic self-revision、daemon 和多层记忆投影在可信读取闭环完成前统一按实验能力管理。
6. 发布证据工具只服务于产品能力，不再作为独立主线继续扩张。
7. 所有里程碑按“全局建模、单点验证、最小执行、证据通过后扩张”推进。

## 2. 事实基线

### 2.1 已实现

- 单一 Rust crate，正式 CLI 为 `serve` 与 `doctor`。
- MCP `stdio` 运行时暴露 4 个工具：`ingest_interaction`、`build_self_snapshot`、`decide_with_snapshot`、`run_reflection`。
- SQLite 持久化 events、claims、evidence links、episode events、reflections、trigger ledger、identity、commitments 和 operation log。
- ingest 与 reflection 具备事务边界；`run_reflection` 是当前 identity / commitments 的唯一 durable write path。
- mock、OpenAI-compatible 和 OpenRouter provider 路径存在。
- 本地 dashboard、support bundle、backup / restore、release preflight 和多类状态报告存在。
- 当前测试列表枚举 400 项；本轮完整 `cargo test` 已在 `2026-07-10` 串行通过。

### 2.2 直接阻断 Local Alpha 的事实

| 编号 | 当前事实 | 风险 | 规划结论 |
| --- | --- | --- | --- |
| F-01 | Snapshot 输入只有 budget，运行时会读取全库 active claims、events 和 episodes；event 引用按旧序截断 | 跨 namespace / 陈旧证据进入 snapshot 与自动反思 | M0 先建立 `MemoryScope` 和存储层过滤 |
| F-02 | `decide_with_snapshot` 接受调用方完整 snapshot，只 gate requested action，模型 selected action 不复检 | 返回“gate passed”但策略语义不可信 | M0 双重 gate，或先降级为 lab-only |
| F-03 | 跨 episode 支持数由全局 episode 数与 claim 数取最小值，没有 claim → evidence → episode 的真实 join | 不相关 episode 可抬高 identity 更新支持度 | M0 补读取关系后重做治理校验 |
| F-04 | `doctor` 会 create / migrate / seed SQLite，并补默认 identity | 诊断命令会改变被检查对象，release soak 可能碰正式库 | M0 拆分只读诊断与显式 init / migrate |
| F-05 | Legacy 表重建没有 schema version、migration ledger 或显式事务保护 | 中途失败可能留下半迁移数据库 | M0 建立版本化迁移与恢复门禁 |
| F-06 | MCP 没有按 namespace 查询 event、claim、episode、reflection 和 evidence relation 的正式接口 | “记忆已写入，但用户无法可靠取回和解释” | M1 建设 Read Model v2 |
| F-07 | Dashboard 无认证且配置允许非 loopback bind | 本地只读口径与可配置暴露面不一致 | M0 在无认证阶段强制 loopback |
| F-08 | `.github/workflows` 缺失，CLI 启动路径未初始化 tracing，真实二进制包尚未建立 | 本地测试资产没有持续门禁，排障与交付不完整 | M0 补 CI / tracing；M2 补真实包 |
| F-09 | evidence / episode / memory layer projection 主要停留在定义和测试调用 | 测试存在被误读为运行时产品能力 | 未接入前标记 partial / experimental |
| F-10 | 文档有多套并行计划、旧测试总数与命名漂移 | 下一步来源不唯一，状态容易高估 | 本计划成为唯一 active plan |

### 2.3 外部协议校准

- MCP 官方仍把 `stdio` 与 Streamable HTTP 都定义为标准传输，并建议客户端在可行时支持 `stdio`。本项目当前不需要为了“跟上协议”而提前进入远程服务。[MCP Transports](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)
- MCP Resources 适合暴露由应用控制的上下文数据。M1 可以在稳定查询契约之后评估只读 `memory://` resources，但不应绕过同一 scope 和 provenance 规则。[MCP Server Features](https://modelcontextprotocol.io/specification/2025-06-18/server/index)
- 当前仓库固定 `rmcp = 0.5`，官方 Rust SDK 文档已经进入更新的 API 线并提供 resources、notifications、tasks 与 OAuth 等能力。M0 只做兼容性 spike，不直接跳版本或顺手扩功能。[Official MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- MCP Tasks 仍带实验性质，不是当前本地记忆闭环的前置依赖。[MCP Tasks](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks)

## 3. 主用户与核心任务

### 3.1 主用户

第一阶段只服务一个明确角色：

> 在本机使用 Codex 类或其他 MCP 客户端，希望 Agent 跨会话保留项目事实、偏好、承诺和更正记录的开发者。

### 3.2 Jobs To Be Done

1. 当客户端会话结束或重启后，我仍能取回之前明确记录的项目事实。
2. 当一条记忆被返回时，我能看到它来自哪个 event、属于哪个 namespace、何时记录、是否已被替换。
3. 当记忆错误或过期时，我能更正、supersede 或撤销，而不是直接丢失历史。
4. 当我切换项目或用户 scope 时，其他 scope 的数据不会混入结果。
5. 当数据库升级或机器迁移时，我能先备份、验证恢复，再切换正式路径。
6. 当 provider 不可用时，确定性的本地记录和检索仍然可用。

### 3.3 产品承诺

Local Alpha 只承诺：

- local-first、single-user、MCP `stdio`；
- scoped persistent memory；
- deterministic recall with provenance；
- auditable correction / supersession；
- explicit init / migrate / backup / restore；
- loopback-only local diagnostics；
- 保守、可验证的产品措辞。

### 3.4 明确非目标

- 远程写管理、团队模式和多租户；
- 完整自治 Agent 或无人值守后台自我修订；
- 完整决策引擎、policy arbitration 或 action executor；
- 所有入口自动反思；
- provider 质量、SLA 或模型能力认证；
- service manager、auto-updater、云同步和生产灾备；
- 把只读 projection、preflight 或文档状态当作已交付产品能力。

## 4. 目标架构

### 4.1 模块职责

| 层 | 目标职责 | 不允许的越界 |
| --- | --- | --- |
| Ledger Core | Event、Claim、Evidence、Episode、Reflection、Identity、Commitment 的不变量 | 不依赖 MCP、HTTP 或 provider |
| Read Model | 按 `MemoryScope` 查询、排序、过滤、provenance join、历史读取 | 不在应用层全库读取后再截断 |
| Governance | conflict、supersession、manual reflection、bounded auto-reflection | 不绕过 evidence validation 或 durable write path |
| Persistence | SQLite schema、versioned migration、index、transaction、retention | 不在 doctor 中隐式迁移正式库 |
| MCP Interface | 兼容旧工具，新增稳定的 memory read contract | 不信任调用方自带 policy / snapshot |
| Local Ops | init、doctor read-only、migrate、backup、restore、support bundle | 不自动发布、不复制原始凭据或数据库 |
| Labs | decision、daemon、memory layering、experimental tasks | 不进入 Local Alpha 默认能力或完成口径 |

### 4.2 核心对象

M0 先冻结以下概念，不立即重做所有表：

- `MemoryScope`：owner + namespace + optional project/root identity。
- `MemoryRecord`：可检索的 event / claim 统一读取视图，保留原始 ID 和类型。
- `Provenance`：claim → evidence event → episode / reflection 的可读取关系。
- `SnapshotHandle`：服务端创建、带 scope / version / evidence manifest 的可信 snapshot 引用。
- `RevisionVersion`：identity / commitment 的版本、effective time、diff 和 rollback target。
- `MigrationVersion`：明确的 schema version、执行记录和恢复状态。

### 4.3 兼容策略

- 保留现有 4 个 MCP 工具，先加 additive 字段和新工具，不静默改变旧 caller 的成功/失败语义。
- `decide_with_snapshot` 在可信 snapshot handle 完成前明确标为 experimental；如果无法兼容修正，则先新增 v2 工具而不是直接破坏旧 schema。
- 数据迁移必须先支持旧数据库 dry-run、备份和 rollback rehearsal。
- `agent_llm_mm` 技术标识的完整重命名不与产品功能改造捆绑。

## 5. Now / Next / Later

| 阶段 | 目标 | 预计节奏 | 扩张条件 |
| --- | --- | --- | --- |
| **Now — M0 Truth and Safety Reset** | 修复可信性边界、迁移和诊断语义；建立 CI | 1–2 周参考节奏 | M0 所有证据门通过 |
| **Next — M1 Trustworthy Recall** | 做出真正可用的 scoped read / search / provenance / correction 闭环 | 2–4 周参考节奏 | 真实本地客户端完成核心用户旅程 |
| **Next — M2 Local Product Alpha** | 真实二进制包、fresh-machine、Windows 边界、备份恢复与人工 release decision | 2–3 周参考节奏 | Alpha exit gate 全部有 fresh evidence |
| **Later — M3 Retrieval Quality and Lifecycle** | ranking、retention、versioned revision、runtime projections | 3–5 周参考节奏 | M1/M2 稳定，先冻结评测集 |
| **Not scheduled — M4 Remote and Autonomy** | remote/team、write-capable daemon、full autonomy | 不排期 | 有真实需求且安全前置门全部满足 |

预计节奏只用于排序，不是发布日期承诺。

## 6. M0 — Truth and Safety Reset

M0 是唯一允许立即领取的里程碑。没有通过 M0，不能开始新 provider、远程、daemon 写能力或正式发布工作。

### M0.1 统一事实入口和仓库卫生

- [ ] 把 status-sync 的 active plan 指针从旧 P1/P2/P3 文件迁到本计划。
- [ ] 清理被跟踪的根目录 SQLite fixture；先确认测试不依赖，再以正常提交删除并加入忽略/fixture 规则。
- [ ] 轮换本机私有配置中的非占位 provider credential；不把值写入日志、文档或提交。
- [ ] 增加仓库 secret / binary fixture hygiene check。
- [ ] 保持旧计划为 historical，不删除追溯证据。

证据门：工作树不含意外数据库或凭据；文档只有一个 active plan；status sync 读取本计划。

### M0.2 Scoped Snapshot v2

- [ ] 为 snapshot 输入增加服务端 `MemoryScope`。
- [ ] 在 store/query port 层按 namespace、owner、时间和显式 evidence manifest 查询。
- [ ] 统一裸 event ID 与 `event:<id>` 表示。
- [ ] recent-first 排序必须在 SQL 层完成，稳定 tie-break 使用 `(recorded_at, rowid/id)`。
- [ ] auto-reflection snapshot 只能使用当前 trigger window 和允许的 scope 关系。

证据门：至少覆盖 self / world / 两个 project / 两个 user namespace；跨 scope 注入为 0；相同输入产生相同顺序。

### M0.3 Governance Correctness

- [ ] `decide_with_snapshot` 使用服务端可信 commitments / policy。
- [ ] requested action 与 selected action 都经过同一 policy gate。
- [ ] 如果 selected action 不可结构化验证，则返回 non-authoritative / experimental 结果，不能标记 policy passed。
- [ ] 用 claim → evidence → episode 的真实 join 计算跨 episode 支持。
- [ ] 治理失败只产生 rejected audit，不留下部分 identity / commitment 更新。

证据门：伪造 caller snapshot 不能移除 baseline commitment；provider 返回受禁 action 必须被阻断；无关 episode 不计入支持数。

### M0.4 Explicit Database Lifecycle

- [ ] 拆分 `init`、`migrate`、`doctor --read-only` 和显式 `doctor --allow-bootstrap`。
- [ ] `doctor --read-only` 对不存在数据库、旧 schema 和不可写路径只报告，不 create / migrate / seed。
- [ ] 建立 schema version 与 migration ledger。
- [ ] legacy rebuild 在事务和备份锚点下执行；失败后原库可恢复。
- [ ] 每次迁移执行 `foreign_key_check`、表/行数 readback 和恢复演练。
- [ ] release soak 强制使用隔离数据库，不接受未确认的正式库路径。

证据门：doctor 前后数据库 checksum / schema / row count 不变；故障注入后恢复成功；旧库 roundtrip 无数据丢失。

### M0.5 Runtime Boundary and CI

- [ ] 无认证阶段拒绝 dashboard 非 loopback host。
- [ ] 修正 CLI tracing 初始化，使 stderr 日志与 MCP stdout 保持隔离。
- [ ] 修复当前 toolchain 在 `tests/provider_live_certification.rs` 报出的 `clippy::collapsible_if`，恢复零 warning 基线。
- [ ] 增加 GitHub Actions：format、clippy、tests、status sync；先覆盖 macOS / Linux，Windows 进入 M2 完整 parity gate。
- [ ] 固定 Rust toolchain / MSRV 决策。
- [ ] 对 `rmcp 0.5` 到当前官方 SDK 做独立兼容 spike，只提交迁移清单、破坏面和测试矩阵；不得顺带引入 remote/tasks/OAuth。

证据门：CI 在 clean clone 通过；stdout 只有 MCP 消息；dashboard loopback 规则有正反测试；SDK spike 有 go / no-go 结论。

## 7. M1 — Trustworthy Recall

### M1.1 Read Model v2

- 按 scope 读取完整 event、claim、episode、reflection 和 evidence relation。
- 查询结果保留 ID、namespace、owner、recorded_at、status、mode、provenance 和 supersession 状态。
- SQLite 增加经过 explain/benchmark 证明需要的索引；不先假设 FTS 或向量数据库。

### M1.2 MCP Memory Contract

第一批建议接口：

- `search_memory`：按 scope、文本/结构过滤、时间范围、类型和 limit 检索。
- `get_memory`：按稳定 ID 返回完整记录与 provenance。
- `get_reflection_history`：返回 claim / identity / commitment 的修订链。
- `supersede_memory`：显式 evidence 下的受审计纠错；默认不 hard delete。

现有 `ingest_interaction` 保持兼容。MCP Resources 只在上述查询契约稳定后评估，且必须复用同一 read service。

### M1.3 用户闭环

标准验收故事：

1. 在 `project/a` 写入 10 条记忆，在 `project/b` 写入 5 条干扰记忆。
2. 断开并重新连接 MCP 客户端。
3. 在 `project/a` 检索，结果只包含当前 scope。
4. 查看任一命中的 event / claim / episode provenance。
5. supersede 一条错误 claim，默认结果只返回新版本，历史接口仍可回看旧版本。
6. provider 离线时重复上述 deterministic read path，结果仍可用。

M1 退出门：该故事由真实本地客户端完成，并保存 MCP transcript、SQLite readback 和无跨 scope 泄漏证据。

## 8. M2 — Local Product Alpha

- 建立版本化 macOS binary archive 和 checksum；Local Alpha 不要求 installer、service manager 或 auto-updater。
- quick start 不要求用户本机安装 Rust toolchain。
- `init → doctor --read-only → serve → remember/search/inspect/correct` 在 fresh macOS 环境可按文档完成。
- Windows runner 执行真实 runtime parity；未通过前明确写“不支持/未验证”，不做模糊 parity 声明。
- backup → restore-to-new-path → read-only verify → manual switch 全链路通过。
- 至少一个 real provider 路径只证明配置、连通与解析；deterministic memory read 不依赖 provider。
- support bundle 继续保持本地、脱敏、无原始 SQLite / TOML / provider payload。
- 人工 release decision 记录 reviewer、open gates、rollback note 和最终结论。

M2 退出指标：

- fresh-machine 首次完成核心用户闭环不超过 10 分钟；
- 不需要源码编辑或 `cargo run`；
- namespace leakage = 0；
- backup / restore roundtrip = 100%；
- 公开文档不存在 Local Alpha / Beta / GA 越级声明；
- release artifact 可解包、checksum 匹配、版本可追溯。

## 9. M3 — Retrieval Quality and Lifecycle

只有 M1/M2 通过后才进入：

- 冻结至少 50 个 scoped recall 场景，再比较 structured query、FTS5、recency、relation weight 等方案。
- 初始质量目标为 recall@5 ≥ 0.80、provenance coverage = 100%、namespace leakage = 0；若基线差异较大，先记录基线再批准目标调整。
- 增加 tombstone、retention、compaction、export 和恢复验证，不直接 hard delete 审计历史。
- identity / commitment 引入版本、diff、effective time 和 rollback target。
- evidence relation、episode summary 和 memory layer projection 要么接入正式 read path，要么删除或明确留在 labs。
- automatic self-revision 只在可靠 trigger、scope、idempotency、rollback 和人工可解释性通过后扩张。

## 10. M4 — 暂不排期

以下方向不进入当前执行队列：

- Streamable HTTP remote server；
- OAuth / authz / rate limit / tenant isolation；
- remote read-only dashboard 与 remote write admin；
- write-capable daemon、queue、retry、dead letter 和持续自治；
- MCP experimental tasks；
- multi-agent orchestration；
- full procedural memory / self-model autonomy。

进入条件：M2 稳定、至少 3 个真实用户完成两周 dogfood、无数据丢失或 scope 泄漏、需求证据明确，并在实现前完成独立 threat model 与 rollback runbook。

## 11. 验证策略

### 11.1 验证阶梯

每个里程碑按以下顺序扩张：

1. 只读现状确认；
2. 一个失败测试证明风险；
3. 一个最小实现切片；
4. 一个 SQLite / namespace / client 单点闭环；
5. 小样本集成验证；
6. fresh-machine / cross-platform；
7. 人工 release decision。

前一级证据不通过，不进入后一级。

### 11.2 风险与最小证明动作

| 风险假设 | 最小证明动作 | 通过条件 | 停止 / 回滚 |
| --- | --- | --- | --- |
| snapshot 会跨 scope | 两个 namespace 各写一条冲突记忆并构建 scoped snapshot | 输出仅含目标 scope | 任一泄漏立即停止 auto-reflection 扩张 |
| selected action 绕过 gate | stub 返回受禁 selected action | 结果 blocked，模型 action 不被标记 passed | 保持 decision experimental |
| doctor 会改库 | 对已知 DB 记录 checksum / schema / rows，再运行 read-only doctor | 三类 readback 完全不变 | 禁止用于 release soak 正式库 |
| migration 半完成 | 在 copy / rename / FK check 注入失败 | 原库可打开、行数一致、可再次迁移 | 恢复备份，阻断 schema 发布 |
| dashboard 暴露非本机 | 配置 `0.0.0.0` 启动 | 无 auth 时拒绝启动 | 保持 dashboard disabled |
| recall 只命中 ID 不可解释 | search 命中一个 claim 后读取 provenance | event / episode / reflection 链可读取 | 不宣布 trustworthy recall |
| package 不可复现 | clean machine 解包并运行核心故事 | 无 Rust toolchain、checksum 正确 | 保持 source-only MVP 口径 |

### 11.3 每阶段必须报告

- 全局建模了什么；
- 实际只执行了哪个最小切片；
- 收集了什么证据；
- 哪些能力故意没有扩张；
- 剩余风险和下一最小动作。

## 12. 成功指标

| 维度 | 指标 |
| --- | --- |
| 激活 | fresh-machine 到首次 remember + recall ≤ 10 分钟 |
| 隔离 | namespace leakage = 0 |
| 可解释 | recall provenance coverage = 100% |
| 可恢复 | backup / restore roundtrip = 100% |
| 迁移 | 每个 schema migration 有 version、backup anchor、failure test、FK check |
| 可用性 | provider 离线时 deterministic read path 仍可用 |
| 质量 | M3 评测集 recall@5 初始目标 ≥ 0.80 |
| 工程 | clean clone CI 通过 format、clippy、tests、status sync |
| 产品真实度 | implemented / partial / experimental / blocked 与代码一致 |
| 文档治理 | active 总计划始终只有一份 |

## 13. 参考 30 天执行节奏

### 第 1 周：真相与回归测试

- 完成 M0.1；
- 为 scope 泄漏、selected action 复检、doctor 写入和 migration failure 建立红灯测试；
- 冻结兼容策略和数据库备份锚点。

### 第 2 周：Foundation v2

- 实现 `MemoryScope` 和 scoped store queries；
- 拆分 doctor / init / migrate；
- 收口 dashboard loopback、tracing 与基础 CI。

### 第 3 周：Read Model v2

- 先实现 `search_memory` 与 `get_memory`；
- 建立 claim → evidence → episode / reflection provenance；
- 用两个 namespace 小样本验证。

### 第 4 周：真实客户端闭环

- 完成 supersession history；
- 在一个真实 MCP 客户端跑完整用户故事；
- 根据证据决定进入 M2，或继续修复 M1。

## 14. 停止条件

出现以下任一情况必须停在当前里程碑，不得用新增功能掩盖：

- scope 泄漏、数据丢失或迁移不可恢复；
- 需要破坏现有 MCP schema，但没有兼容层和迁移说明；
- doctor / smoke / release 命令可能接触未确认的正式数据库；
- 测试数量或状态文档与当前分支不一致；
- 只读 projection 被当成 runtime capability；
- 需要远程监听、auth、第三方发布或凭据变更，但未获得单独授权；
- external evidence 缺失却准备声明 Local Alpha、Beta、GA 或 production-ready。

## 15. 文档迁移

| 文档 | 新角色 |
| --- | --- |
| `docs/positioning.md` | 继续作为命名与公共定位唯一来源 |
| `docs/project-status.md` | 只写当前实现与已验证事实 |
| `docs/roadmap.md` | 只写 Now / Next / Later 摘要 |
| 本文件 | 唯一 active execution plan |
| `docs/product/follow-up-reality-gates.md` | 证据状态与阻断项 |
| `docs/progress-tracker.md` | 2026-05-09 历史快照，不再是任务总表 |
| 2026-05/06 多份 productization plans | historical / superseded，保留追溯 |

## 16. 当前进度与下一步

已完成：

- 2026-07-10 只读审计了代码架构、运行时能力、文档口径、测试清单和发布链；
- 建立新的产品主线、里程碑、证据门和停止条件；
- 串行通过 `cargo fmt --check`、`git diff --check`、status-sync 定向检查和完整 `cargo test`；测试清单为 400 项；
- `cargo clippy --all-targets --all-features -- -D warnings` 尚未通过：当前 toolchain 在 `tests/provider_live_certification.rs:925` 报 `clippy::collapsible_if`；本次规划边界内未修改产品代码；
- 未修改业务代码、未发布、未推送、未运行远程或 live-provider 操作。

尚未开始：M0 的任何代码实现。

下一最小动作：领取 **M0.1 统一事实入口和仓库卫生**，先更新 status-sync 的 active plan 契约并为 P0 四类风险建立失败测试；不要同时开始 M1 接口开发。
