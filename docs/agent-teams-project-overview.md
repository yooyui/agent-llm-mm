# agent-llm-mm 项目全貌梳理报告

> 最后更新：2026-05-08 | 版本 v1.0 | 分支：dev-work

---

## 一、Agent Teams 总览

### 1.1 项目定位

agent-llm-mm 是一个基于 Rust 的本地 MCP stdio 内存管理系统。核心链路为：**events → claims → self_snapshot → decision → reflection**。采用六边形架构 (Ports & Adapters) + 领域驱动设计 (DDD)。

### 1.2 团队地图

```
┌─────────────────────────────────────────────────────────────────┐
│                    Interface Teams (接口层)                       │
│  ┌─ MCP stdio Team ─────────────┐  ┌─ Dashboard Team ──────────┐ │
│  │ 4 tools: ingest / snapshot / │  │ axum HTTP + SSE +         │ │
│  │ decide / reflect             │  │ Memory-chan Live Desk UI  │ │
│  └──────────────────────────────┘  └───────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                  Application Teams (应用层)                       │
│  ┌─ Ingest Team ───┐ ┌─ Snapshot Team ──┐ ┌─ Decision Team ──┐ │
│  └──────────────────┘ └──────────────────┘ └──────────────────┘ │
│  ┌─ Reflection Team ─────────────────┐ ┌─ Auto-Reflect Team ─┐ │
│  └───────────────────────────────────┘ └──────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                    Domain Teams (领域层)                          │
│  Event │ Claim │ Commitment │ Episode │ Snapshot │ Reflection   │
│  Self-Revision │ Identity Core │ Rules (gate/conflict/policy)   │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                  Ports & Adapter Teams (端口与适配层)              │
│  ┌─ SQLite Adapter ────┐  ┌─ Model Adapter ───┐  ┌─ Config ──┐ │
│  │ 8 tables + migration │  │ Mock + OpenAI     │  │ TOML + env │ │
│  └──────────────────────┘  └───────────────────┘  └───────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 团队一览表

| 团队 | 源码路径 | 职责 | 状态 |
|------|---------|------|------|
| Event Team | `src/domain/event.rs`, `types.rs` | 领域事件建模，Owner/Mode/EventKind/Namespace 类型系统 | 成熟 |
| Claim Team | `src/domain/claim.rs`, `evidence_link.rs` | 命题领域建模，证据链接 | 成熟 |
| Commitment Team | `src/domain/commitment.rs` | 承诺/约束建模 | 成熟 |
| Episode Team | `src/domain/episode.rs` | 剧集聚合建模 | 骨架 |
| Snapshot Team | `src/domain/snapshot.rs` | 自我快照建模 | 部分完成 |
| Reflection Team | `src/domain/reflection.rs` | 反思聚合根建模 | 成熟 |
| Self-Revision Team | `src/domain/self_revision.rs` | 自我修订领域契约 | 成熟 |
| Identity Core Team | `src/domain/identity_core.rs` | 身份核心建模 | 骨架 |
| Rules Team | `src/domain/rules/` | 领域规则引擎 | 成熟 |
| Ingest Team | `src/application/ingest_interaction.rs` | 事件摄取用例编排 | 成熟 |
| Snapshot App Team | `src/application/build_self_snapshot.rs` | 构建自我快照用例 | 成熟 |
| Decision Team | `src/application/decide_with_snapshot.rs` | 决策用例 (gate + model) | 部分完成 |
| Reflection App Team | `src/application/run_reflection.rs` | 持久化反思写入 | 成熟 |
| Auto-Reflect Team | `src/application/auto_reflect_if_needed.rs` | 自动反思协调器 (1024行) | 成熟 |
| SQLite Adapter Team | `src/adapters/sqlite/` | 8张表完整持久化实现 | 成熟 |
| Model Adapter Team | `src/adapters/model/` | Mock + OpenAI兼容模型适配 | 部分完成 |
| MCP stdio Team | `src/interfaces/mcp/` | 4个MCP tool服务端 | 成熟 |
| Dashboard Team | `src/interfaces/dashboard/` | 只读HTTP面板 + SSE | 成熟 |
| Config Team | `src/support/config.rs` | TOML配置加载/校验 | 成熟 |
| Doctor Team | `src/support/doctor.rs` | 运行时就绪检查 | 成熟 |
| CLI Team | `src/support/cli.rs` | 命令行入口解析 | 成熟 |

---

## 二、核心业务团队 (Domain Teams)

### 2.1 Event Team

**文件**: `src/domain/event.rs` (57行), `src/domain/types.rs` (96行)

**关键类型**:
- `Owner` — `Self_ | User | World | Unknown`
- `Mode` — `Observed | Said | Acted | Inferred | Draft`
- `EventKind` — `Observation | Conversation | Action | Reflection`
- `Namespace` — 4种子类型：`self`, `world`, `user/<id>`, `project/<id>`
- `Event` — 领域事件聚合根，含 owner, namespace, kind, summary

**质量评估**: 类型安全，`Namespace::parse()` 对非法格式返回 `DomainError`，`Event::new_with_namespace()` 自动校验 namespace-owner 不匹配条件（owner=Self_ 时 namespace 不能是 user/*或 project/*）。

### 2.2 Claim Team

**文件**: `src/domain/claim.rs` (107行), `src/domain/evidence_link.rs` (22行)

**关键类型**:
- `ClaimDraft` — 声明草稿，含 owner, namespace, subject, predicate, object, mode
- `EvidenceLink` — 证据链接 (source_event_id → target_claim_id)
- `ClaimStatus` — `Active | Disputed | Superseded`

**质量评估**: `ClaimDraft::validate_namespace_owner()` 在存储前做命名空间一致性校验。Inferred模式的声明必须有至少一条支持证据。

### 2.3 Commitment Team

**文件**: `src/domain/commitment.rs` (24行)

**关键类型**: `Commitment` — 含 owner, description

**已交付**: 承诺实体及 `conflicts_with_commitment()` 规则函数。数据库种子承诺：`forbid:write_identity_core_directly`。

### 2.4 Episode Team

**文件**: `src/domain/episode.rs` (24行)

**当前状态**: 骨架级别。仅支持 `episode_reference → event_id` 关联，缺少完整的自传式建模（goal/outcome/lesson/self_effect）。`Episode` 结构体已定义但实际使用简化为引用映射。

### 2.5 Snapshot Team

**文件**: `src/domain/snapshot.rs` (56行)

**关键类型**: `SelfSnapshot`, `SnapshotBudget`, `SnapshotRequest`

**当前状态**: 部分完成。`SnapshotBudget` 按 evidence 数量控制预算。快照包含 identity, commitments, claims, evidence, episodes 五个维度。

### 2.6 Identity Core Team

**文件**: `src/domain/identity_core.rs` (23行)

**关键类型**: `IdentityCore` — 最小 canonical_claims 修订路径

**已知问题**: `allow_direct_ingest_update()` 始终返回 false (所有分支)，且未被任何代码调用 — **死代码**。

---

## 三、应用服务团队 (Application Teams)

### 3.1 Ingest Interaction Team

**文件**: `src/application/ingest_interaction.rs` (92行)

**职责**: 事件摄取用例编排

**已交付**: 完整的事务编排 — event + claims + episode 三方写入，返回 `IngestResult { event_id }`。依赖 `IngestTransactionRunner + IdGenerator + Clock`。

### 3.2 Build Self Snapshot Team

**文件**: `src/application/build_self_snapshot.rs` (61行)

**职责**: 构建自我快照用例

**已交付**: 按 `SnapshotBudget` 限制证据查询与聚合。依赖 `IdentityStore + CommitmentStore + ClaimStore + EventStore + EpisodeStore`。

### 3.3 Decide With Snapshot Team

**文件**: `src/application/decide_with_snapshot.rs` (47行)

**职责**: 决策用例 — 承诺门控 + 模型调用

**状态**: 部分完成。返回仍是最小动作字符串，不支持结构化决策输出。

### 3.4 Run Reflection Team

**文件**: `src/application/run_reflection.rs` (318行)

**职责**: 持久化反思写入（唯一 durable write path）

**已交付**: supersede/dispute 行为、identity/commitments deeper update、支持证据替换、commitment gate 校验。反射分类策略（`ReflectionTrigger` 枚举，4种触发类型）。

### 3.5 Auto-Reflect If Needed Team

**文件**: `src/application/auto_reflect_if_needed.rs` (1024行)

**职责**: 自动反思协调器 — 整个系统最复杂的用例

**已交付全链路**:
1. `detect_trigger_candidate()` — 触发检测（failure需≥2条hints且有足够证据，conflict需hints匹配，periodic需有新episode）
2. `evaluate_trigger_suppression()` — 抑制评估（cooldown检查、证据窗口不变检查、episode水印检查）
3. `build_revision_snapshot()` — 构建修订快照
4. `validate_self_revision()` — 验证：identity patch需≥3条支持声明、≥2条跨episode支持、无高冲突、冷却期已过、patch大小1-2条
5. `resolve_governed_evidence_window()` — 治理层证据窗口决议
6. `apply_validated_self_revision()` — 应用自修订

**4个运行时Hook**:
- `ingest_interaction:failure`
- `ingest_interaction:conflict`
- `decide_with_snapshot:conflict`
- `build_self_snapshot:periodic`

---

## 四、端口与基础设施团队 (Ports & Infrastructure Teams)

### 4.1 Ports 定义层

**文件**: `src/ports/` (12文件, 471行)

| Trait | 文件 | 方法数 |
|-------|------|--------|
| `EventStore` | `event_store.rs` | 4 (append/list/query_evidence/has) |
| `ClaimStore` | `claim_store.rs` | 4 (upsert/link/list/update_status) |
| `CommitmentStore` | `commitment_store.rs` | 1 (list) |
| `EpisodeStore` | `episode_store.rs` | 2 (record/list) |
| `ReflectionStore` | `reflection_store.rs` | 1 (append) |
| `TriggerLedgerStore` | `trigger_ledger_store.rs` | 3 (record/latest/latest_handled) |
| `IdentityStore` | `identity_store.rs` | 2 (load/save) |
| `ModelPort` | `model_port.rs` | 2 (decide/propose_self_revision) |
| `IdGenerator` | `id_generator.rs` | 1 (next_id) |
| `Clock` | `clock.rs` | 1 (now) |
| `IngestTransaction` | `ports/mod.rs` | 5 (append/record/upsert/link/commit) |
| `ReflectionTransaction` | `ports/mod.rs` | 9 (upsert/link/append_reflection/append_trigger/load_replace_identity/load_replace_commitments/update_claim_status/commit) |

### 4.2 SQLite 适配器团队

**文件**: `src/adapters/sqlite/schema.rs` (153行), `store.rs` (1476行)

**8张表**:
1. `events` (含 `owner_namespace_scope` CHECK 约束)
2. `claims` (含命名空间范围约束)
3. `evidence_links` (claim ↔ event 多对多)
4. `episode_events` (episode_reference ↔ event_id)
5. `reflections` (含审计列：supporting_evidence_event_ids, requested_identity_update, requested_commitment_updates)
6. `reflection_trigger_ledger` (自修订触发跟踪)
7. `identity_claims` (有序身份声明)
8. `commitments` (承诺表)

**迁移逻辑**: `ensure_events_namespace_column()` / `ensure_claims_namespace_column()` / `ensure_reflection_audit_columns()` 提供向后兼容的列升级。

**所有SQL查询均使用参数化绑定** (`bind()`) — 无SQL注入风险。

**已知问题**:
- `stored_event_from_row()` 有 `#[allow(dead_code)]` 标记 — 死代码
- `not-a-sqlite-url` 文件存在于仓库根目录（非提交文件），是 SQLite 数据库二进制数据，疑似测试 fixture 误放

### 4.3 模型适配器团队

**文件**: `src/adapters/model/mock.rs` (33行), `openai_compatible.rs` (231行)

**已交付**:
- `MockModel` — 确定性决策（根据声明是否为空返回不同动作），`propose_self_revision` 始终返回 `no_revision`
- `OpenAiCompatibleModel` — 通过 reqwest HTTP POST 调用 `/chat/completions`，支持 `decide` 和 `propose_self_revision`，包含 JSON 提取、code fence 剥离、错误处理

**状态**: 部分完成（仅 2 类 provider）

---

## 五、接口交付团队 (Interface Teams)

### 5.1 MCP stdio 服务团队

**文件**: `src/interfaces/mcp/server.rs` (687行), `dto.rs` (437行)

**4个MCP工具**:

| 工具 | 功能 | 自动反射触发 |
|------|------|------------|
| `ingest_interaction` | 持久化事件+声明 | 是 (failure/conflict) |
| `build_self_snapshot` | 构建自我快照 | 是 (periodic) |
| `decide_with_snapshot` | 承诺门控+模型决策 | 是 (conflict) |
| `run_reflection` | 记录反射+更新 | 否 (手动路径) |

**Runtime 结构体**:
```rust
struct Runtime {
    store: SqliteStore,
    model: RuntimeModel,   // Mock | OpenAiCompatible
    dashboard: DashboardObserver,
}
```
实现所有 12 个 Ports trait，均委托给 `self.store` 或 `self.model`。

**DTO层** (`dto.rs`): MCP 参数 DTO → 领域类型转换。`AutoReflectInput` 构建器方法：`from_ingest()`, `from_build_snapshot()`, `from_decide()`。

### 5.2 Dashboard 面板团队

**文件**: `src/interfaces/dashboard/` (6文件, 2217行)

**组件**:

| 组件 | 文件 | 功能 |
|------|------|------|
| HTTP Server | `http.rs` (179行) | axum HTTP + SSE /api/events/stream |
| Assets | `assets.rs` (1622行) | 内嵌2张PNG + 完整SPA HTML (Memory-chan Live Desk) |
| Observer | `mod.rs` (130行) | DashboardObserver (Disabled/Enabled)，所有操作事件录入 |
| Recorder | `recorder.rs` (70行) | OperationRecorder: VecDeque + broadcast通道环形缓冲区 |
| Events | `event.rs` (145行) | OperationEvent结构体，事件构建函数 |
| Projection | `projection.rs` (71行) | DashboardSummary, OperationDetail |

**API路由**:
- `GET /` — Memory-chan Live Desk HTML
- `GET /api/summary` — 运行时摘要
- `GET /api/events` — 事件列表（支持 kind/status/namespace 过滤）
- `GET /api/events/{id}` — 事件详情
- `GET /api/events/stream` — SSE 实时事件流
- `GET /api/health` — 健康检查（标记 read_only: true）

**安全性**: 默认绑定 `127.0.0.1`，仅本地访问。health 端点明确标注只读。写操作严格走 MCP stdio。

---

## 六、支撑设施团队 (Support Teams)

### 6.1 配置团队

**文件**: `src/support/config.rs` (308行)

**加载链**: `AGENT_LLM_MM_CONFIG` 环境变量 → TOML文件 → 当前目录 `agent-llm-mm.local.toml` → `Default`
**环境变量覆盖**: `AGENT_LLM_MM_DATABASE_URL` 始终覆盖配置文件

**关键配置**:
- `TransportKind` — 仅 `Stdio`
- `ModelProviderKind` — `Mock | OpenAiCompatible`
- `DashboardConfig` — host, port (默认8787), base_path, event_capacity, sse_enabled, open_browser, required
- 默认数据库路径: macOS `~/Library/Application Support/agent-llm-mm/agent-llm-mm.sqlite`

**已知问题**: Cargo.toml 无 `[features]` 段，dashboard 和 openai 依赖无法按需编译

### 6.2 Doctor 团队

**文件**: `src/support/doctor.rs` (53行)

运行时就绪检查，输出 `DoctorReport`（含 transport/db/provider/dashboard/auto-reflection hooks 信息）。API key 已脱敏（显示为 `{provider}:configured`）。

---

## 七、测试团队 (Testing Brigade)

### 7.1 测试全景

16个测试文件，约 10,923 行。`cargo test` 全量通过（153个测试）。

### 7.2 测试文件清单

| 测试文件 | 行数 | 类型 | 覆盖内容 |
|----------|------|------|----------|
| `application_use_cases.rs` | 1,845 | 集成 | 完整用例编排测试 |
| `failure_modes.rs` | 2,558 | 集成 | 故障模式全覆盖 |
| `mcp_stdio.rs` | 2,853 | 集成 | MCP stdio 全链路 |
| `sqlite_store.rs` | 1,255 | 集成 | SQLite 所有 trait 实现 |
| `bootstrap.rs` | 384 | 集成/单元 | 配置、CLI、doctor |
| `openai_compatible_model.rs` | 405 | 集成 | 模型适配器 |
| `dashboard_http.rs` | 242 | 集成 | HTTP 端点 |
| `provider_config.rs` | 217 | 单元 | TOML 配置加载 |
| `self_revision_demo_runner.rs` | 145 | 集成 | 端到端演示 |
| `decision_flow.rs` | 133 | 单元 | decide_with_snapshot |
| `dashboard_recorder.rs` | 94 | 单元 | 操作记录器 |
| `dashboard_projection.rs` | 58 | 单元 | 事件投影 |
| `dashboard_config.rs` | 74 | 单元 | Dashboard 配置 |
| `demo_openai_compatible_stub.rs` | 89 | 集成 | 演示桩服务器 |
| `domain_invariants.rs` | 61 | 单元 | 领域不变量 |
| `domain_snapshot.rs` | 68 | 单元 | 快照构建逻辑 |

### 7.3 已知测试缺口

| 优先级 | 缺口 | 来源 |
|--------|------|------|
| 高 | 无 malformed JSON 回归测试 | provider-contract.md 标记 `partial` |
| 高 | 无 doctor redaction 正面断言 | provider-contract.md 标记 `partial` |
| 高 | 无 timeout-failure 回归测试 | provider-contract.md 标记 `partial` |
| 中 | 无独立领域单元测试 | 所有 domain 测试通过集成路径间接覆盖 |
| 中 | 无 `AppConfig::load()` 的 database_url fallback 测试 | README 提及但无测试 |
| 中 | 无 dashboard SSE endpoint 负向测试 | 仅有正常路径 |
| 低 | 无并发工具调用顺序测试 | |
| 低 | 无性能/基准测试 | |
| 低 | 无 `evidence_links` 表外键负向测试 | |

---

## 八、当前状态矩阵

### 8.1 已完成 (Done)

- [x] 核心 MCP stdio 服务器及 4 个工具 (ingest, build_snapshot, decide, reflect)
- [x] SQLite 持久化层 (8张表 + namespace 列迁移 + reflection 审计列)
- [x] Dashboard 只读 HTTP 面板 + SSE 事件流 + Memory-chan Live Desk UI
- [x] 自动反射 (auto-reflection) 全链路：trigger → suppress → propose → validate → apply
- [x] OpenAI 兼容模型适配器 (decide + propose_self_revision)
- [x] 自修订 (self-revision) 演示包 (run_self_revision_demo + demo_openai_compatible_stub)
- [x] Post-MVP Hardening Plan Phase A-H
  - 验证基准刷新
  - 发布门 runbook
  - 本地 MCP 集成排障
  - 运行时 Hook 合约矩阵
  - 诊断摘要合约
  - 深度更新合约 Spec + 测试
  - Evidence Query v2 设计 + namespace-aware narrowing
  - Dashboard 只读边界保留
  - Provider 合约检查清单

### 8.2 部分完成 (Partial)

- [~] Evidence Query v2 — `recorded_after/recorded_before` inclusive 时间窗口字段已在 SQLite 查询、MCP DTO 和 automatic self-revision proposal narrowing 中实现；独立 evidence-kind 分类体系未建立
- [~] `decide_with_snapshot` — 返回协议仍是最小动作字符串
- [~] Provider 生态 — 仅 mock + openai-compatible 两种
- [~] Episode 建模 — 骨架级别，缺 goal/outcome/lesson/self_effect

### 8.3 待修复 (To Fix)

- [ ] **空 `failure` 文件** (Task A2) — 仓库根目录的 0 字节文件，未被 .gitignore 覆盖
- [ ] **死代码 `allow_direct_ingest_update`** — `identity_core.rs:18` 始终返回 false 且未调用
- [ ] **死代码 `stored_event_from_row`** — `store.rs:1319` 有 `#[allow(dead_code)]` 标记
- [ ] **`not-a-sqlite-url` 误提交文件** — SQLite 数据库二进制数据在仓库根目录
- [ ] **Cargo.toml 缺少 `[features]`** — dashboard 和 openai 依赖无法按需编译
- [ ] **无 Rustdoc API 文档**

### 8.4 远期规划 (Future Roadmap — 全未开始)

参考 `docs/superpowers/plans/2026-04-27-future-autonomy-and-productization-roadmap.md`：

| Phase | 内容 | 状态 |
|-------|------|------|
| Phase 0 | 创建自主/产品化 Spec 索引 | 未开始 |
| Phase 1 | 持久化 Operation Log | 未开始 |
| Phase 2 | 默认禁用的本地 Daemon 骨架 | 未开始 |
| Phase 3 | 全入口观察模式反射候选记录 | 未开始 |
| Phase 4 | 只读远程管理 API | 未开始 |
| Phase 5 | 认证与多租户基础 | 未开始 |
| Phase 6 | 生产级自治理就绪标准 | 未开始 |

---

## 九、近期行动建议

### 9.1 关闭测试缺口 (当前 Sprint)

1. **malformed JSON 回归测试** — 模拟 OpenAI 端返回非JSON响应，验证错误处理
2. **doctor redaction 正面测试** — 验证 API key 不被包含在 `DoctorReport` 输出中
3. **timeout-failure 回归测试** — 模型调用超时时确认返回正确错误码而非 panic
4. **独立领域单元测试** — 为 `Namespace::parse()`, `ClaimDraft::validate_namespace_owner()` 等添加直接测试

### 9.2 技术债清理 (下一个 Sprint)

1. **删除/实现 `allow_direct_ingest_update`** — 要么完成实现，要么移除
2. **移除 `#[allow(dead_code)]` 的 `stored_event_from_row`** — 或为它添加测试
3. **清理 `failure` 和 `not-a-sqlite-url` 文件** — 移到 `.gitignore` 或删除
4. **添加 `[features]`** — 将 dashboard 和 openai-compatible 做成可选的编译特性

### 9.3 Evidence Query v2 收尾

1. 设计独立的 evidence-kind 分类体系并迁移 schema

---

## 附录 A：术语表

| 术语 | 定义 |
|------|------|
| Event | 交互事件（observation/conversation/action/reflection），原子单元 |
| Claim | 声明/命题 — 含 subject/predicate/object，由 agent 对事件做出断言 |
| Commitment | 承诺/约束 — agent 对自身行为的自我限制 |
| Episode | 剧集 — 一组相关事件的聚合单元 |
| Snapshot | 快照 — 当前身份+承诺+声明+证据+剧集的即时摘要 |
| Reflection | 反思 — 对已有声明的重新评估，可产生新声明取代旧声明 |
| Self-Revision | 自修订 — agent 根据证据自动修正身份/承诺的过程 |
| Namespace | 命名空间 — 隔离不同用户/项目/agent 的数据边界 |
| Evidence | 证据 — 支持声明的事件/声明引用链 |
| Trigger | 触发器 — 自动启动自修订流程的条件（failure/conflict/periodic） |
| Trigger Ledger | 触发分类账 — 跟踪自修订触发和抑制状态的持久化记录 |
| Durable Write Path | 持久写入路径 — 唯一可信的自修订写入入口（`run_reflection`） |

## 附录 B：文件数量统计

```
src/main.rs                                    18
src/lib.rs                                     52
src/error.rs                                   15
src/support/config.rs                         308
src/support/cli.rs                             42
src/support/doctor.rs                          53
src/support/tracing.rs                          8
src/domain/* (13 files)                       655
src/ports/* (12 files)                        471
src/application/* (5 files)                 1,546
src/adapters/sqlite/store.rs                1,476
src/adapters/sqlite/schema.rs                 153
src/adapters/model/openai_compatible.rs        231
src/adapters/model/mock.rs                      33
src/interfaces/dashboard/* (6 files)         2,217
src/interfaces/mcp/server.rs                  687
src/interfaces/mcp/dto.rs                     437
tests/* (16 files)                         10,923
─────────────────────────────────────────────────
总行数 (不含 docs/)                       ~19,774
```

## 附录 C：Agent Teams 源码全映射

| Agent Team | 源文件绝对路径 |
|------------|---------------|
| Event Team | `/Users/yooyui/code/agent-llm-mm/src/domain/event.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/domain/types.rs` |
| Claim Team | `/Users/yooyui/code/agent-llm-mm/src/domain/claim.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/domain/evidence_link.rs` |
| Commitment Team | `/Users/yooyui/code/agent-llm-mm/src/domain/commitment.rs` |
| Episode Team | `/Users/yooyui/code/agent-llm-mm/src/domain/episode.rs` |
| Snapshot Team | `/Users/yooyui/code/agent-llm-mm/src/domain/snapshot.rs` |
| Reflection Team | `/Users/yooyui/code/agent-llm-mm/src/domain/reflection.rs` |
| Self-Revision Team | `/Users/yooyui/code/agent-llm-mm/src/domain/self_revision.rs` |
| Identity Core Team | `/Users/yooyui/code/agent-llm-mm/src/domain/identity_core.rs` |
| Rules Team | `/Users/yooyui/code/agent-llm-mm/src/domain/rules/` |
| Ingest Team | `/Users/yooyui/code/agent-llm-mm/src/application/ingest_interaction.rs` |
| Snapshot App Team | `/Users/yooyui/code/agent-llm-mm/src/application/build_self_snapshot.rs` |
| Decision Team | `/Users/yooyui/code/agent-llm-mm/src/application/decide_with_snapshot.rs` |
| Reflection App Team | `/Users/yooyui/code/agent-llm-mm/src/application/run_reflection.rs` |
| Auto-Reflect Team | `/Users/yooyui/code/agent-llm-mm/src/application/auto_reflect_if_needed.rs` |
| SQLite Adapter Team | `/Users/yooyui/code/agent-llm-mm/src/adapters/sqlite/schema.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/adapters/sqlite/store.rs` |
| Model Adapter Team | `/Users/yooyui/code/agent-llm-mm/src/adapters/model/mock.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/adapters/model/openai_compatible.rs` |
| MCP stdio Team | `/Users/yooyui/code/agent-llm-mm/src/interfaces/mcp/server.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/interfaces/mcp/dto.rs` |
| Dashboard Team | `/Users/yooyui/code/agent-llm-mm/src/interfaces/dashboard/http.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/interfaces/dashboard/assets.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/interfaces/dashboard/mod.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/interfaces/dashboard/event.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/interfaces/dashboard/recorder.rs` |
| | `/Users/yooyui/code/agent-llm-mm/src/interfaces/dashboard/projection.rs` |
| Config Team | `/Users/yooyui/code/agent-llm-mm/src/support/config.rs` |
| Doctor Team | `/Users/yooyui/code/agent-llm-mm/src/support/doctor.rs` |
| CLI Team | `/Users/yooyui/code/agent-llm-mm/src/support/cli.rs` |
