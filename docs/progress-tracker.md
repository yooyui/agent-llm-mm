# 进度追踪对照表

## 目的

本表用于把“原始规划 / 当前实现 / 剩余缺口 / 下一步任务”放到同一处，方便后续开发时持续更新，而不是每次都在多份历史文档之间来回比对。

当前追踪基线：`2026-04-24`

建议把它当成“任务推进总表”使用：

- 每次完成一个任务后，至少更新对应行的“当前状态 / 当前证据 / 剩余缺口 / 建议下一步”
- 如果某项范围被 roadmap 调整，再同步更新“目标态”
- 若有 fresh 验证结果，优先写入具体命令、测试集或 `doctor` 输出，而不是只写“已验证”

## 状态标签

| 标签 | 含义 |
| --- | --- |
| `已完成` | 当前阶段承诺已落地，并且已有文档或测试支撑 |
| `部分完成` | 已有可用 MVP 或局部实现，但离目标态仍有明确差距 |
| `未实现` | 目标已被讨论或列入路线图，但仓库里尚未形成可用实现 |
| `暂不纳入近期承诺` | 方向存在，但 roadmap 明确不作为近期交付目标 |

## 总表

| 领域 | 目标态 | 当前状态 | 状态标签 | 当前证据 | 剩余缺口 | 建议下一步 |
| --- | --- | --- | --- | --- | --- | --- |
| 最小主链路 | 稳定打通 `events -> claims -> self_snapshot -> decision -> reflection` | 已打通完整最小闭环，并作为当前仓库主能力存在 | `已完成` | `project-status.md` 明确列出主链路；`cargo test` 全量通过 `153` 个测试 | 主要缺口不在主链路存在性，而在 richer 语义与自动化范围 | 维持现有测试基线，避免后续扩展时破坏最小闭环 |
| 本机 MCP 接入 | 作为本机 AI 客户端可稳定拉起的 `stdio` MCP 服务 | 已有 `doctor` / `serve`、SQLite 落盘、4 个 MCP tools，可供本机客户端接入 | `已完成` | `README.md`、`development-macos.md`、`development-windows.md`；`doctor` 返回 `status = ok` | 仍缺更正式的 release gate 操作手册和更清晰的接入排障口径 | 优先补强 release gate 与接入验证文档 |
| automatic self-revision MVP | 在现有 `run_reflection` 之上形成“自动触发 + 模型提案 + 服务端治理 + 审计落库”的最小自我修订回路 | `self_revision` 契约、`auto_reflect_if_needed` 协调器、trigger ledger、governed evidence window 和 `run_reflection` durable write path 均已存在，并已有一键 demo package 证明 before / after decision shift | `已完成` | `src/domain/self_revision.rs`、`src/application/auto_reflect_if_needed.rs`、`src/bin/run_self_revision_demo.rs`、`scripts/run-self-revision-demo.sh`、`docs/self-revision-demo-guide-2026-04-24.md`；相关测试集通过 | 当前实现仍是保守 MVP，不等于完整自治治理系统；demo 只覆盖 canonical scenario，不是所有入口自动反思 | 继续做 runtime coverage 和治理边界收口，而不是另起旁路写接口 |
| production dashboard service | 通过配置开关提供随 `serve` 启动的本机只读运行观测面板 | 已实现 `[dashboard]` 配置、bounded in-memory operation recorder、HTTP JSON API、SSE、静态生成图物料和 `Memory-chan Live Desk` 二次元风格页面 | `已完成` | `README.md`、`project-status.md`、`development-macos.md`、`testing-guide-2026-03-24.md`、`NOTICE`；`tests/dashboard_*` 和 MCP stdout safety 测试 | 不是远程管理后台、写入界面或 durable operation-log database；暂无认证、多租户和持久化操作日志 | 维持只读边界；后续只在明确需要时再评估认证、部署和持久化日志 |
| automatic self-revision runtime coverage | 形成更完整、清楚、可排查的自动触发覆盖 | 当前只接了 4 条 hook：`ingest_interaction:failure`、`ingest_interaction:conflict`、`decide_with_snapshot:conflict`、`build_self_snapshot:periodic`；demo package 复核显式 conflict path | `部分完成` | `doctor` 输出 4 条 hooks；`tests/bootstrap.rs`、`tests/mcp_stdio.rs`、`tests/self_revision_demo_runner.rs` 覆盖这些路径 | 仍不是“所有入口统一自动反思”；多个入口依旧需要显式 `trigger_hints` 或 `auto_reflect_namespace`；没有 daemon | 近期继续收口这 4 条 hook 的文档、诊断、失败语义和排障路径 |
| `decide_with_snapshot` 决策能力 | 从最小 gate + 动作字符串，推进到更完整、可解释的决策能力 | commitment gate 已真实生效，也可走 `openai-compatible` provider，但输出仍是最小动作字符串协议 | `部分完成` | `project-status.md` 标记为部分实现；相关 `decision_flow`、`mcp_stdio` 测试存在 | 还不是完整决策引擎，也不适合对外包装成生产级 agent decision engine | 暂时保持保守表述，优先稳定边界，不急于扩写为完整决策框架 |
| reflection deeper update | 在审计友好的前提下，稳定支持 `identity_core` / `commitments` 的深层修订 | 当前已支持显式 evidence ids、结构化窄化查询、最小 identity / commitments 更新 | `部分完成` | `README.md`、`project-status.md`；`run_reflection` 相关测试 | 仍不是 richer schema / versioned policy；输入契约和 slow-variable 策略仍偏首版 | 近期继续收口 deeper-update 契约和服务端校验边界 |
| evidence 语义与查询治理 | 从存在性校验推进到 richer lookup / weight / relation / ranking | 当前只落地了 `namespace / owner / kind / limit` 级别的窄化查询，`proposed_evidence_query` 只在 trigger window 内做 bounded narrowing；project / user scoped conflict 与 periodic trigger window 会先排除 sibling namespace 事件 | `部分完成` | `README.md`、`roadmap.md`、`auto_reflect_if_needed.rs` 中的 governed evidence window 逻辑 | 缺 richer lookup、weighting、relation、ranking；还没有更成熟的证据评分与关联模型 | 中期优先扩展 evidence-oriented query，再考虑 richer ranking/weighting |
| `identity_core` / `commitments` / `claims` / `episodes` schema | 从骨架式字符串与轻量聚合推进到更有表达力的 schema | `identity_core` 与 `commitments` 已可最小修订；`episodes` 仍主要是 `episode_reference -> event_id` 聚合 | `部分完成` | `project-status.md` 对 `episodes` 和 deeper update 有明确说明 | 缺 richer schema、版本化形成机制、更细生命周期和更完整自传式建模 | 中期先丰富 `episodes`、`identity_core` 和 claim schema，再谈更深层 memory semantics |
| provider 生态 | 在现有 provider 边界上继续增加更多 provider 类型 | 当前仅有 `mock` 与 `openai-compatible` 两类 provider | `部分完成` | `README.md`、`project-status.md`、`roadmap.md` | 缺 Azure OpenAI、OpenRouter、本地模型网关等更多适配器 | 中期按统一 provider contract 继续扩展，不让应用层感知第三方协议细节 |
| 多层 memory 体系 | 形成 working / episodic / semantic / procedural 等更完整分层 | 当前仍以最小 self-agent memory 闭环为主，未形成独立多层 memory 回路 | `未实现` | `README.md`、`project-status.md`、`roadmap.md` 明确列为后续方向 | 缺独立层次、层间筛选/回写/淘汰策略，以及 slow-variable / self-model layering | 后期再进入该方向，前提是 schema、evidence 与 reflection policy 已稳定 |
| 持续后台自治与完整 self-governing agent | 形成持续运行、统一触发、非手工驱动的更完整自治行为 | 当前没有后台 daemon、定时自治进程或“所有入口统一自动反思”的运行形态 | `未实现` | `project-status.md`、`roadmap.md` 都明确否定当前已实现该能力 | 缺 daemon 化调度、统一自动触发入口、持续运行治理策略 | 明确保留为后期方向，不在近期承诺里提前产品化 |
| 产品化封装 | 从本机 demo 评估到更清晰的部署/隔离/transport 边界 | 当前仍定位为本机 `stdio` technical demo / MVP，不应包装成完整产品 | `暂不纳入近期承诺` | `README.md`、`roadmap.md`、`release-readiness.md` | 缺更稳定的 transport、隔离策略、部署方式和产品级运维能力 | 继续保持 demo / MVP 口径，避免路线图表达漂移 |

## 近期建议任务序列

| 顺位 | 建议任务 | 目的 |
| --- | --- | --- |
| `1` | 收口 release gate 文档 | 把现有验证基线和 self-revision demo package 变成更清楚的提交前 / 发布前操作手册 |
| `2` | 稳定 4 条 runtime hooks 的文档与回归测试 | 先把当前已落地 coverage 的边界、诊断和失败语义说清楚 |
| `3` | 继续收紧 self-revision 治理与可观测性 | 降低误触发，补强 suppression / rejection / cooldown 排查路径 |
| `4` | 收口 reflection deeper-update 契约 | 让 identity / commitments 的修订边界更稳定，减少后续 schema 漂移 |
| `5` | 扩展 evidence-oriented 查询能力 | 为后续 richer evidence policy 和 schema 升级打基础 |

## 维护建议

- 如果某个任务只是补测试或补文档，也应同步更新对应行的“当前证据”
- 如果某项能力从 `部分完成` 进入 `已完成`，最好附上最新验证命令或测试集名称
- 如果 roadmap 改了优先级，不要只改 `roadmap.md`，也要同步改这里的“建议下一步”
