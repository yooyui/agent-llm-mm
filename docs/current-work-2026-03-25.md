# 当前工作说明（2026-03-25）

## 概览

- 当前工作分支：`codex/self-agent-mcp`
- 当前提交基线：`ff2f5eb661fdf516b8ce28d24fc28d9e0682f543`
- 当前状态：工作树有未提交改动，但已完成本地 fresh 验证
- 运行形态：Rust 单 crate，MCP stdio 服务，架构为 `Functional Core + Imperative Shell`

本轮工作的目标，是在上一轮 `namespace` 垂直切片的基础上继续收口两类问题：

1. 把 SQLite 中 `owner <-> namespace` 的 SQL 规则抽成单一来源，减少 schema / migration / store 三处漂移风险
2. 把 `run_reflection` 从“fail-closed 拒绝 inferred replacement”推进到“显式 evidence 输入驱动的 evidence-aware reflection”，并补齐坏输入的参数级校验

## 本轮已完成内容

### 1. SQLite `owner/namespace` 规则已抽成单一 SQL 来源

本轮把 SQLite 适配层里重复的规则定义收敛到了 `schema` 模块：

- `owner_namespace_scope` 约束名集中定义
- `claims` 表建表 SQL 通过共享 builder 生成
- legacy `claims` 迁移时的默认 namespace 回填表达式集中定义
- `store` 层不再内嵌 owner/namespace 规则字面量

这样带来的直接收益是：

- fresh bootstrap 与 legacy rebuild 使用同一套 SQL 规则
- 后续修改 owner/namespace 兼容性时，只需要改一个地方
- 避免 schema 与 migration 语义漂移

同时保留了上一轮已经落地的数据库级硬约束：

- fresh 数据库的 `claims` 表继续具备 `CHECK` 约束
- legacy 数据库重建后也恢复相同的 `CHECK` 约束
- 非法 `owner/namespace` 组合会被数据库直接拒绝

### 2. `run_reflection` 已支持显式 evidence 驱动的 inferred replacement

本轮为 `run_reflection` 新增了显式证据输入能力：

- `ReflectionInput` 新增 `replacement_evidence_event_ids`
- 当 replacement claim 为 `Mode::Inferred` 时，不再一律 fail-closed
- 只要显式 evidence 列表满足校验条件，就允许 inferred replacement 通过
- replacement claim 写入成功后，会同步写入对应的 `evidence_links`

当前采用的是最小闭环设计：

- 不做复杂 evidence weight
- 不做自动相似度检索
- 不扩展到更复杂的 reflection policy
- 只使用显式传入的 event id 列表完成证据门槛与可追溯性闭环

这样既保留了 KISS / YAGNI，也把上一轮“只有拒绝分支，没有允许分支”的状态推进到了真正可验证的 evidence-aware 路径。

### 3. 缺失 evidence event id 现在会在应用层被拒绝

review 过程中发现一个关键缺口：

- `replacement_evidence_event_ids` 虽然控制了 inferred replacement 的证据数量
- 但如果传入的 event id 根本不存在，错误会在 SQLite 外键阶段才爆出
- 这种错误原先会表现为 `-32603 internal_error`

本轮对此做了修复：

- `EventStore` 新增 `has_event` 能力
- `run_reflection` 在进入事务写入前，先检查每个 evidence event id 是否存在
- 缺失 event id 时直接返回 `AppError::InvalidParams`
- stdio 路径会将其映射为 `-32602 invalid_params`

这意味着当前行为已经从“数据库层被动兜底”升级为“应用层主动校验 + 协议层正确报错”。

### 4. 测试文档已补齐

本轮还新增了测试文档：

- [testing-guide-2026-03-24.md](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/testing-guide-2026-03-24.md)

文档覆盖了：

- 全量验证命令
- `sqlite_store` / `mcp_stdio` / `application_use_cases` 的定向测试方式
- `evidence-aware reflection` 的自动化与手工验证方式
- `replacement_evidence_event_ids` 必须引用已持久化 event 的前置条件
- 常见失败与排查路径

## 本轮涉及的核心文件

- `src/adapters/sqlite/schema.rs`
- `src/adapters/sqlite/store.rs`
- `src/application/run_reflection.rs`
- `src/interfaces/mcp/dto.rs`
- `src/interfaces/mcp/server.rs`
- `src/ports/event_store.rs`
- `src/ports/mod.rs`
- `tests/application_use_cases.rs`
- `tests/failure_modes.rs`
- `tests/mcp_stdio.rs`
- `tests/sqlite_store.rs`
- `docs/testing-guide-2026-03-24.md`

## 本地验证结果

以下验证已在当前工作树上 fresh 运行通过：

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

测试结果摘要：

- `application_use_cases`: 11 passed
- `bootstrap`: 4 passed
- `decision_flow`: 2 passed
- `domain_invariants`: 4 passed
- `domain_snapshot`: 6 passed
- `failure_modes`: 3 passed
- `mcp_stdio`: 7 passed
- `sqlite_store`: 8 passed

本轮新增或强化的关键测试包括：

- `sqlite_owner_namespace_sql_rules_have_single_source`
- `reflection_accepts_inferred_replacement_with_explicit_evidence`
- `reflection_rejects_missing_replacement_evidence_event_ids`
- `inferred_replacement_reflection_with_evidence_is_accepted_over_stdio`
- `missing_replacement_evidence_event_ids_are_invalid_params_over_stdio`

## 子代理执行与审查结论

### 本轮使用的子代理

- SQLite 规则抽取实现子代理：`019d1f4a-8fa6-7b53-bdd6-6c18e3a03495`
- 代码审查子代理：`019d1f58-f1b0-7532-a2c0-564c5b4ad2c2`

### SQLite 规则抽取结论

子代理将 owner/namespace 的 SQL 规则成功集中到 `schema` 层，并通过共享接口供 `store` 使用。其工作产出与主线程的 reflection 改动已完成集成，未发现明显冲突。

### Review 结论

review 最初指出了 1 个需要修复的 P1：

- `replacement_evidence_event_ids` 只做数量校验，没有在应用层验证 event 是否存在
- 坏输入会拖到 SQLite 外键阶段，再被映射成 `internal_error`

该问题已在本轮修复，并新增两条回归测试锁住：

- application：`reflection_rejects_missing_replacement_evidence_event_ids`
- stdio：`missing_replacement_evidence_event_ids_are_invalid_params_over_stdio`

修复后，当前这一轮没有保留阻塞性 findings。

## 当前工作的总体判断

如果评价标准是：

- `owner/namespace` SQL 规则是否已经去重并形成单一来源
- inferred replacement 是否已经具备显式 evidence 驱动的正向分支
- 坏 evidence id 输入是否会得到参数级错误而不是基础设施错误

那么本轮答案是：已经完成，并且有本地验证支撑。

如果评价标准提升为“reflection 是否已经是完整产品语义”，答案仍然是否定的。当前实现虽然已经从 fail-closed 推进到 evidence-aware，但仍然属于最小版本：

- 只支持显式 evidence event id 列表
- 不支持自动 evidence lookup
- 不支持 richer reflection reasoning
- 不涉及 identity_core / commitments 的更深层修订策略

## 后续建议

### 建议 1：把 reflection 的 evidence lookup 从“显式输入”升级为“显式输入 + 可选查询”

当前最小设计已经可用，但仍依赖调用方先知道 event id。下一步可考虑：

- 保留显式输入分支
- 增加可选的 store 查询能力
- 在查询结果可证明存在时，允许 inferred replacement 通过

### 建议 2：把 `EventStore::has_event` 扩展为更丰富的 evidence 查询接口

如果后续要支持：

- 至少两条独立证据
- 证据去重
- evidence weight / relation

那么单纯的 `has_event` 不够，需要更明确的 evidence-oriented port。

### 建议 3：把 testing guide 继续升级为 release gate 文档

当前测试文档已经适合日常开发使用。后续如果这条分支要长期维护，建议继续补：

- 提交前清单
- 典型坏输入样例
- 默认数据库作用域验证步骤

## 结论

当前这条分支已经达到：

- `namespace` 规则在 domain / SQLite / MCP 三层上的闭环
- SQLite owner/namespace 规则的单一来源化
- evidence-aware reflection 的最小正向闭环
- 缺失 evidence id 的参数级报错收口
- 配套测试文档可直接供后续开发使用

它仍不是“完整自我机制”的最终形态，但已经把本轮两个明确目标都推进到了可验证、可继续集成的状态。
