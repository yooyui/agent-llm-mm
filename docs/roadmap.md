# MCP Memory Ledger 路线图

状态：`active summary`
更新日期：`2026-07-10`

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

- Rust MCP `stdio` 服务与 4 个工具；
- SQLite 持久化与 ingest / reflection 事务；
- `run_reflection` 受治理的 durable write path；
- mock、OpenAI-compatible、OpenRouter 配置路径；
- 本地 dashboard、doctor、support bundle、backup / restore 和 release preflight；
- 测试已分为 `fast` / `core` / `full` 三级；发布证据、打包和 provider certification 工具由非默认 `release-tools` feature 承载。

当前不能视为完整产品能力：

- snapshot 还不是可信的 namespace-scoped read model；
- 没有正式的 memory search / get / history MCP 接口；
- decision gate 只检查调用方请求动作，模型结果未复检；
- cross-episode 支持没有真实 provenance join；
- `doctor` 会 create / migrate / seed 数据库，不是纯只读诊断；
- legacy migration 缺 schema version、migration ledger 和显式事务恢复门；
- dashboard 无认证且配置可绑定非 loopback；
- daemon 仍是 observe-only idle lifecycle；
- memory layer / evidence relation 等投影尚未形成统一 runtime read path；
- 没有 GitHub Actions、真实二进制交付或 fresh-machine / Windows 完整证据。

## Now — M0 Truth and Safety Reset

目标：在继续扩功能前，修正产品可信性和数据安全边界。

工作流：

1. 统一 active plan、状态与证据入口，清理仓库卫生问题。
2. 建立 `MemoryScope`，让 snapshot 和 auto-reflection 只读取允许的 namespace / trigger window，并使用 recent-first 稳定排序。
3. 让 decision 使用服务端可信 policy，同时复检 requested / selected action。
4. 用 claim → evidence → episode 的真实关系替换全局数量推断。
5. 拆分 `init`、`migrate`、`doctor --read-only` 与显式 bootstrap。
6. 建立 schema version、事务 migration、备份、故障注入和恢复验证。
7. 无认证阶段强制 dashboard loopback。
8. 修正 CLI tracing，建立 format / clippy / tests / status-sync CI。
9. 独立评估 `rmcp` 升级破坏面，不顺带引入 remote/tasks/OAuth。

退出门：

- namespace leakage = 0；
- recent-first snapshot 顺序稳定；
- 模型返回受禁 action 时仍被阻断；
- `doctor --read-only` 前后 DB checksum / schema / row count 不变；
- migration 故障后可恢复且行数一致；
- 非 loopback dashboard 配置被拒绝；
- clean clone CI 通过。

M0 未通过前，不开始新 provider、daemon 写能力、remote/team 或正式发布扩张。

## Next — M1 Trustworthy Recall

目标：完成用户真正需要的本地记忆闭环。

计划能力：

- scoped events / claims / episodes / reflections / evidence relation read model；
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
