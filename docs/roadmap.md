# MCP Memory Ledger 路线图

状态：`active summary`
更新日期：`2026-07-15`

本文件只回答三个问题：现在做什么、接下来做什么、哪些方向暂不排期。具体任务、依赖、证据门与停止条件统一见 [2026-07-10 全新项目规划](plans/2026-07-10-product-replan.md)。

## 产品北极星

把现有 technical MVP 收束成一个可信、可检索、可审计、可恢复的本地 MCP Memory Ledger。

所有路线选择先服从[项目起点与主线原则](origin-and-principles.md)：原始事件保真、高层记忆可追溯、scope 隔离、历史可纠错、反思受治理。不能加强这些原则的能力默认不进入当前主线。

第一产品闭环是：

```text
记录 -> scoped 检索 -> 查看证据 -> 纠错/撤销 -> 备份恢复
```

不是：

```text
自动决策 -> 后台自治 -> 远程团队平台
```

当前仓库仍是 local-first technical MVP，不是 Local Alpha、Beta、GA 或生产级自治平台。

## 当前事实

已实现：

- Rust MCP `stdio` 服务与 6 个工具；`search_memory` 已覆盖显式 namespace 的 Event 与 Claim recall 首片，`get_memory` 当前仍只覆盖 Event lookup 首片；
- SQLite 持久化与 ingest / reflection 事务；
- `run_reflection` 受治理的 durable write path；
- mock、OpenAI-compatible、OpenRouter 配置路径；
- 本地 dashboard、doctor、support bundle、backup / restore 和 release preflight；
- 测试已分为 `fast` / `core` / `full` 三级；发布证据、打包和 provider certification 工具由非默认 `release-tools` feature 承载。

当前不能视为完整产品能力：

- 显式 namespace / manifest / time-window 的 M0.2 scoped snapshot 已收口，但省略 namespace 的 legacy 兼容调用仍未隔离，且尚无完整 recall read model；
- 已有正式的 Event `search_memory` / `get_memory` 首片，以及 additive Claim `search_memory` 首片；Claim lookup、episode/reflection read model、完整 history 与 correction 接口仍未实现；
- decision 已改用服务端 commitments，并同时 gate requested / provider-selected action；允许的 action-string 结果显式标记为 `experimental_non_authoritative`，且 policy scope 只覆盖 server commitment gate。但其他 snapshot 字段仍由 caller 提供，尚无完整 trusted snapshot handle 或 provenance binding；
- cross-episode identity support 已使用 claim → evidence → episode 的真实 distinct join，但仍没有完整 provenance graph；
- M0.4 已完成：`init` / `migrate` / 默认只读 `doctor` / 显式 `doctor --allow-bootstrap` 已拆分，`serve` 不再隐式改库；
- SQLite 使用 schema version 与 migration ledger；legacy migration 先建立 backup anchor 和 restore rehearsal，再在事务中执行 FK / 表行数 readback；
- dashboard 无认证，但启用时配置已强制 localhost / loopback；remote/public exposure 仍未实现；
- daemon 仍是 observe-only idle lifecycle；
- memory layer / evidence relation 等投影尚未形成统一 runtime read path；
- 已有 Linux/macOS GitHub Actions 与固定 Rust toolchain；仍没有真实二进制交付或 fresh-machine / Windows 完整证据。

## Complete — M0 Truth and Safety Reset

目标：在继续扩功能前，修正产品可信性和数据安全边界。

工作流：

1. 统一 active plan、状态与证据入口，清理仓库卫生问题。
2. **已完成（M0.2，2026-07-14）**：建立 `MemoryScope`，让显式 scoped snapshot 和 auto-reflection 只读取允许的 namespace / trigger window，并使用 recent-first 稳定排序；完成 active reflection、只读 evidence/episode projection 与 offline demo artifact 的限定 event-ID 收口。省略 namespace 的 legacy unscoped 兼容、repository-wide ID 统一和 support bundle inventory 不计入该退出门。
3. **已完成首个切片（M0.3，2026-07-14）**：decision 使用服务端 commitment store 覆盖 caller commitments，同时复检 requested / provider-selected action；被拒绝的 selected action 不返回 authoritative decision payload。
4. **已完成第二个切片（M0.3，2026-07-14）**：用 claim → evidence → episode 的 distinct store join 替换全局 episode 数量推断，无关 episode 不计入 identity support。
5. **已完成第三个切片（M0.3，2026-07-14）**：验证治理拒绝、handled-ledger 写入失败与 transaction commit 失败均不留下部分 identity / commitment / reflection 更新，失败后只保留 rejected audit。
6. **已完成第四个切片并收口（M0.3，2026-07-14）**：v2 envelope additive 返回 `decision_authority = experimental_non_authoritative` 与 `policy_scope = server_commitment_gate_only`，保留 legacy 字段和 provider action-string contract，不把有界字面量 gate 误报为完整 policy passed。
7. **已完成（M0.4，2026-07-14）**：拆分显式数据库生命周期；只读 doctor 对 missing / old / read-only 路径不 create / migrate / seed；migration ledger、事务、备份、恢复演练、FK / 行数读回和 release-soak 隔离库均有回归证据。
8. **已完成（M0.5，2026-07-14）**：无认证阶段强制 dashboard loopback；CLI tracing 固定写入 stderr。
9. **已完成（M0.5，2026-07-14）**：Linux/macOS CI 执行 format / all-feature Clippy / full tests / status sync。
10. **已完成（M0.5，2026-07-14）**：固定 Rust `1.95.0` 与 `rust-version = 1.95`。
11. **已完成 spike（M0.5，2026-07-14）**：`rmcp 0.5.0` → `2.2.0` 直接升级 no-go；后续迁移必须独立且不得引入 remote/tasks/OAuth。

退出门：

- namespace leakage = 0；
- recent-first snapshot 顺序稳定；
- 模型返回受禁 action 时仍被阻断；
- `doctor --read-only` 前后 DB checksum / schema / row count 不变；
- migration 故障后可恢复且行数一致；
- 非 loopback dashboard 配置被拒绝；
- clean clone CI workflow 覆盖 Linux/macOS，且同一组固定门禁在本机通过。

M0 未通过前，不开始新 provider、daemon 写能力、remote/team 或正式发布扩张。

## Active — M1 Trustworthy Recall

目标：完成用户真正需要的本地记忆闭环。

已完成三片（2026-07-15）：`M1.1.1 Scoped Event Recall Read Model`、`M1.1.2 Scoped Claim Provenance Read` 与 `M1.2.1 Scoped Event Lookup`。Event 路径通过真实 MCP `stdio` 暴露显式 namespace、bounded recent-first 查询和 stable-ID 单条 lookup；additive Claim 查询支持 canonical/raw claim reference、status、mode 和有界 limit，默认只返回 Active claims，并保留 canonical evidence/episode references 与直接 reflection revision links。两类查询都在 SQL limit 前按 owner + namespace 收窄、保持只读且不依赖 provider；跨 scope exact reference 返回空/null 而不扩大。完整 M1、Claim lookup、episode/reflection read model、完整 history、supersession/correction 和真实客户端退出门仍开放。

计划能力：

- 继续完成 scoped episodes / reflections / evidence relation read model，并把已完成的 Event / Claim 首片收敛到稳定跨类型合同；
- `search_memory`；
- `get_memory`；
- `get_reflection_history`；
- 受审计的 `supersede_memory`；
- 保留 ID、scope、时间、status、mode 和 provenance 的返回结构；
- provider 离线时仍可用的 deterministic read path。

退出门：真实 MCP 客户端完成“写入 → 重连 → 检索 → 查看证据 → supersede → 回看历史”，且两个干扰 namespace 没有任何数据混入。

## Next — M2 Local Product Alpha

目标：从 source-only MVP 进入可验证的本地产品 alpha。

计划能力：

- 版本化 macOS binary archive 与 checksum；
- 无需 Rust toolchain 的 first-run；
- fresh-machine 核心用户闭环；
- Windows runtime parity 的真实证据或明确 unsupported 口径；
- backup / restore-to-new-path / read-only verify / manual switch；
- 一个 real provider 的连通与解析证据；
- 人工 release decision 与 rollback note。

退出门：fresh-machine 首次闭环不超过 10 分钟、namespace leakage = 0、backup / restore 100% 通过、artifact 可追溯且公开文档没有越级声明。

## Later — M3 Retrieval Quality and Lifecycle

目标：在可信读取与真实交付稳定后，提高记忆质量和长期维护能力。

- 冻结 retrieval evaluation set；
- 比较 structured query、FTS5、recency 和 relation weight；
- retention、tombstone、compaction 与 export；
- identity / commitment version、diff、effective time、rollback target；
- 将有效 projection 接入正式 read path，其余保留在 labs 或删除；
- 在可靠 scope、trigger、idempotency 和 rollback 之后再扩 automatic self-revision。

初始质量门：recall@5 ≥ 0.80、provenance coverage = 100%、namespace leakage = 0。

## Not Scheduled — M4 Remote and Autonomy

当前不排期：

- Streamable HTTP remote server；
- OAuth / authz / rate limit / tenant isolation；
- remote dashboard / admin；
- write-capable daemon、queue、retry、dead letter；
- MCP experimental tasks；
- multi-agent orchestration；
- full procedural memory / self-model autonomy。

只有 M2 稳定、真实用户需求成立、两周 dogfood 无数据丢失或 scope 泄漏，并完成独立 threat model 与 rollback runbook 后，才允许重新评估。

## 执行顺序

```text
M0 Truth and Safety
  -> M1 Trustworthy Recall
      -> M2 Local Product Alpha
          -> M3 Retrieval Quality and Lifecycle
              -> M4 Remote and Autonomy (only if separately approved)
```

每次只领取一个最小切片。测试或 readback 不通过时停在当前阶段，不以新增 preflight、projection 或文档包装替代产品能力。

## 文档入口

- 项目起点：[origin-and-principles.md](origin-and-principles.md)
- 公共定位：[positioning.md](positioning.md)
- 当前事实：[project-status.md](project-status.md)
- 唯一 active plan：[2026-07-10-product-replan.md](plans/2026-07-10-product-replan.md)
- 证据状态：[follow-up-reality-gates.md](product/follow-up-reality-gates.md)
- 计划索引：[plans/README.md](plans/README.md)
- 历史归档：[archive.md](archive.md)

2026-03 至 2026-06 的历史 specs、execution plans、阶段快照和发布记录已移至 `codex/archive/pre-mainline-reset-2026-07-10`，不再作为当前下一步来源。
