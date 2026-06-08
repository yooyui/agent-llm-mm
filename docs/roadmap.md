# 路线图

本路线图面向 GitHub 协作与后续开发沟通，强调“当前已承诺什么、下一步先做什么、哪些还只是中长期方向”。

当前已收口的一项基础语义是：默认 SQLite 路径表示“本机用户共享的持久化默认库”；如果需要正式数据、测试数据或实验数据隔离，应显式设置不同的 `database_url`。

另一项当前已落地但必须谨慎表述的能力是：仓库已经具备 trigger-ledger-backed automatic self-revision MVP。不过，这个能力当前仍是本地 `stdio` demo 里的受限自动修订链路，不是完整自治系统。

2026-04-24 已补齐 self-revision demo package：它能零外网依赖地跑出 doctor、snapshot before / after、decision before / after、timeline、SQLite summary 和 report，用来证明当前 MVP 的可重复证据链。它不改变上述保守边界。

2026-04-27 已补齐可配置的本机只读 dashboard：它随 `serve` 在 `[dashboard].enabled = true` 时启动，展示 `Memory-chan Live Desk` 运行面板、runtime operation 事件和静态生成图物料。它仍是本机观测面板，不是远程管理后台、写入界面或 durable operation-log database。

2026-05-09 的 historical MVP release gate snapshot 曾确认当时可作为产品化起点：`cargo test` 全量通过 170 个测试，`doctor` 返回 `status = ok`，self-revision demo package 可生成 release gate 要求的证据链。当前验证总数以 `status-sync-check` 监控文档为准。正式产品化进入下一阶段规划，详见 [Productization Roadmap After MVP](superpowers/plans/2026-05-09-productization-roadmap.md)。这不改变当前事实：仓库还不是 GA / 生产级完整自治产品。

2026-05-16 前序 formal product readiness slice 已刷新 Local Alpha minimum gate、product smoke gate 和 support bundle gate 证据：当时 `cargo test` 全量通过 194 个测试，`doctor` 返回 `status = ok`，`product-smoke-local.sh` 通过 staging / promote 流程刷新 `target/reports/self-revision-demo/latest`，`generate-support-bundle.sh` 生成本地脱敏诊断 JSON 且不包含 `.sqlite` 或 `.toml` 文件。后续 support bundle 增加了显式 `--log-file <path>` 本地日志摘要/安全摘录边界，以及显式 `--correlation-id mcp-tool-call-<uuid-v4>` operation summary 过滤；它仍不自动扫描日志位置、不复制原始 `.log` 文件，也不导出 raw operation payload。这仍不代表 fresh-machine install、remote/team、multi-tenancy、Beta 或 GA 已完成。

2026-05-16 后续补齐了 Local Alpha first-run bootstrap helper：`bootstrap-local` 可安全复制 dev 示例配置到本机私有 config，拒绝覆盖已有文件，不生成 secret，不运行 `doctor` / `serve`，不启动 daemon；相对目标路径按仓库根目录解析。随后新增的 `first-run-bootstrap-smoke-local.sh` 可在隔离输出目录里模拟 `bootstrap-local -> doctor`，生成 `doctor.json` / `summary.json` 和 isolated SQLite 证据，并清理 config/database 环境变量干扰。该能力收窄 fresh-machine setup 的本地模拟证据，但仍需要真实 fresh-machine / Windows runner 证据后才能声明 Local Alpha install/bootstrap gate 完成。

2026-05-16 又补齐了 Local Alpha SQLite backup / restore 本地回归门禁：`cargo test --test sqlite_backup_restore -v` 覆盖 backup -> restore-to-new-path roundtrip、restore 拒绝覆盖、backup 拒绝 live DB 子目录、in-memory / invalid URL 拒绝和 `..` restore target 拒绝。该门禁把 data lifecycle 从纯文档规则推进为可回归验证的本地脚本边界，但仍不是远程备份、云同步、定时备份或生产灾备。

2026-06-08 补齐了四条非 MVP 产品化 read-only / preflight 切片：`release-evidence-index.sh` 会把候选 evidence root 下的 Local Alpha evidence summary 与 product readiness gates 合并成 present / missing / not_verified / blocked 索引；`provider-certification-check.sh` 会校验 provider config shape、脱敏 URL path 内容，并列出 live decision、live self-revision、error handling 和 redaction review 证据缺口，只有非空、provider 匹配、`status = "passed"`、expected `evidence_kind`、`mode = "live"`、非空 `generated_at`、`local_only = false`、`endpoint_reached = true`、`redaction_reviewed = true`、`request_outcome = "passed"` 且带显式 `exit_code = 0` 的成功 command evidence 的 JSON 才能算 present；`packaging-preflight-check.sh` 会区分 source-only release soak artifacts 与真实 binary archive / installer / service manager / auto-updater evidence，并要求 archive 可解析为对应 `.tar.gz` / `.zip` 且与 checksum manifest 的 name / size / SHA-256 匹配，纯文本占位、截断 archive、零字节、部分 archive、缺失 manifest 或 manifest mismatch 不会满足 binary archive gate；`memory_semantics_projection` 会只读暴露 evidence relations、episode summaries、semantic claims、procedural memory 和 durable self-model write 状态。这些能力只用于本地审查和缺口收口，不生成缺失证据、不调用 provider endpoint、不创建安装包、不上传文件、不授予 durable memory write，也不改变 Local Alpha / Beta / GA / production-ready 边界。

## 下一阶段：正式产品化

目标：

- 把当前已验证的本机 MCP `stdio` MVP 推进为 local-first formal product
- 先完成 Local Product Alpha，再进入 durable observability、observe-only daemon、controlled beta、remote/team mode 和 GA readiness
- 继续保持 `run_reflection` 作为 identity / commitments 的唯一 durable write path，直到后续 ADR 明确替换
- 在 auth、authorization、audit、rollback 和隔离测试完成前，不暴露可写远程管理能力
- Local Product Alpha 的支持包已有首版本地生成器：只允许分享脱敏 `doctor` shape、配置 shape、受限 operation summaries、release metadata、product smoke summary、manifest、显式 `--correlation-id mcp-tool-call-<uuid-v4>` 过滤后的 operation-log metadata，以及显式 `--log-file <path>` 触发的 bounded / redacted `local-log-excerpts.json`；它仍不自动发现日志位置、不复制原始日志文件，也不导出 raw request / response / diagnostic payload
- Local Product Alpha 的 first-run 配置引导已有 `bootstrap-local` 脚本入口和本地 `bootstrap-local -> doctor` smoke simulation；下一步应补真实 fresh-machine install / Windows runner evidence，而不是把 helper 表述为 installer 或 production bootstrapper
- Local Product Alpha 的 data lifecycle 已有本地 backup / restore 脚本和回归门禁；restore 仍默认写到新路径，正式 `database_url` 切换必须由人工在 `doctor` 验证后决定
- daemon 先经过 observe-only gate：Local Alpha 阶段不调用 `run_reflection`、不写 identity / commitments、不启动 remote listener；当前 `doctor.daemon_observe_only` 只提供本机只读 preflight 诊断和 operation-log failed / suppressed 候选计数，`serve` 也只会在 `[daemon].enabled = true` 时启动 observe-only lifecycle handle
- correlation ID 先用于 observability：成功/失败 MCP tool call、dashboard event 和 operation-log metadata 可按 `mcp-tool-call-<uuid-v4>` 串联，但不新增语义写路径
- 架构结构优化先以只读诊断落地：`doctor.system_layer_report` 展示 substrate / signal / memory / policy / control_loop / actuator / interface / release_boundary 的当前状态和 blocker，不代表大规模重构、daemon 写能力、remote/team、完整 memory layering 或 Local Alpha 完成
- 候选证据、provider certification、packaging 和 richer memory semantics 先以本地只读索引 / preflight 落地：这些入口报告缺口，不生成外部证据、不认证 live provider、不生成 installer、不新增 durable self-model 写路径

当前权威规划：

- [Productization Roadmap After MVP](superpowers/plans/2026-05-09-productization-roadmap.md)
- [Formal Product Readiness 12 Workstreams](superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md)
- [Local Alpha Data Lifecycle](product/data-lifecycle.md)
- [Release Engineering](product/release-engineering.md)
- [Remote and Team Mode Boundary](product/remote-team-mode-boundary.md)
- [Threat Model: Local and Remote Surfaces](security/threat-model-local-and-remote.md)
- [Memory Layering Roadmap](product/memory-layering-roadmap.md)
- [Local Alpha Support Bundle Design](product/support-bundle-local-alpha.md)
- [Daemon Observe-Only Gate](product/daemon-observe-only-gate.md)
- [Correlation ID Contract](product/correlation-id-contract.md)

## 近期

### 1. 收口 release gate 文档

目标：

- 把现有测试说明进一步整理成更接近发布门槛的操作手册
- 明确最小验证集和建议验证集
- 把 self-revision demo package 固定为自动修订能力的发布前可读证据

原因：

- 当前已有测试基础
- 当前已有可重复 demo artifact，可以作为协作者理解 automatic self-revision MVP 的最短路径
- 但公开协作时，还需要更稳定的“提交前 / 发布前”规则

### 2. 收口 automatic self-revision 的 MCP runtime coverage

目标：

- 先把当前已落地的 4 条 MCP runtime hook 文档、验证与诊断口径收口稳定：
  - `ingest_interaction -> failure`
  - `ingest_interaction -> conflict`
  - `decide_with_snapshot -> conflict`
  - `build_self_snapshot -> periodic`
- 在不新增旁路持久化接口的前提下，优先验证这些 hook 的 opt-in 条件、best-effort 失败语义和排查路径，而不是继续扩大到“所有入口自动反思”
- demo package 当前用于证明其中的显式 conflict path；它不是扩大 hook 覆盖面的实现本身

重点边界：

- `decide_with_snapshot` 已可走 `openai-compatible` 或 OpenRouter provider；OpenRouter 当前只表示本地 stub 验证的 OpenAI-compatible `/chat/completions` transport，不是 live-provider certification；该能力仍不是完整决策引擎
- Provider certification preflight 继续保持 live certification blocked，除非四类 evidence 文件都带 matching provider、`status = passed`、expected `evidence_kind`、`mode = live`、非空 `generated_at`、`local_only = false`、`endpoint_reached = true`、`redaction_reviewed = true`、`request_outcome = passed` 和带显式 `exit_code = 0` 的成功 command evidence；显式 stub/simulated runner 只生成本地非 live evidence，不能解锁 live certification
- Packaging preflight 现在要求四个平台 archive 可解析为对应 `.tar.gz` / `.zip`，并与 `packaging-archive-manifest.json` 的 size / SHA-256 匹配；archive evidence runner 只校验并记录已存在 archive，不构建二进制、installer、service manager、auto-updater、上传或 tag
- 当前 MCP-wired automatic path 只有上述 4 条，不代表所有 MCP entry point 都会自动反思
- `decide_with_snapshot` 与 `build_self_snapshot` 当前仍要求显式 `auto_reflect_namespace`
- `run_reflection` 仍是唯一 durable write path；没有新增旁路持久化接口
- direct `run_reflection` 不递归 auto-reflection；没有后台 daemon 或“所有入口自动反思”
- richer memory semantics 当前只有只读 projection 首片，尚未落地完整 schema、ranking engine、procedural memory 或 durable self-model writes

### 3. 提高 self-revision 的可观测性与治理精度

目标：

- 保持 `run_reflection` 为长期写入路径，同时继续收紧 trigger 候选、cooldown 与 slow-update 规则
- 明确 best-effort auto-reflection 的 structured diagnostics、日志排查方式和运行时边界
- 让 ledger / trigger / rejection / suppression 的诊断信息更容易在验证和接入文档里被复用

原因：

- 当前 MVP 已证明“ledger + governed proposal + `run_reflection` durable write path”这条路径可行
- 下一步的价值不在于新建更多接口，而在于减少误触发、补强排查路径，并为后续扩大触发面提供更稳的观测基础

### 4. 收口 reflection 的 deeper-update 契约

当前规格文档：`docs/superpowers/specs/2026-04-27-reflection-deeper-update-contract.md`

目标：

- 在已支持显式 `replacement_evidence_event_ids`、结构化 `replacement_evidence_query` 和最小 `identity_core` / `commitments` 更新的基础上，继续明确输入约束、保底规则与审计边界
- 在 self-revision proposal 已支持 `proposed_evidence_event_ids`、`proposed_evidence_query` 与 `confidence` 首阶段契约的基础上，继续明确这些字段的治理边界
- 为后续 richer schema、版本化策略和更广泛证据能力奠基

原因：

- 反思路径的最小 deep-update 闭环已经落地，下一步不应再引入新的接口漂移，而应优先把规则与边界收口清楚
- `proposed_evidence_query` 当前仍只是首阶段 evidence contract，已支持 namespace / owner / kind / inclusive recency window / limit 的窄化过滤，不应被扩写成自动 widening / ranking engine
- 这将减少后续 evidence 语义与 schema 改造的反复成本

### 5. 收口 evidence-oriented query v2（窄化版）

当前规格文档：`docs/superpowers/specs/2026-04-27-evidence-query-v2.md`

目标：

- 在现有 `namespace / owner / kind / inclusive recency window / limit` 查询基础上，继续收口 deterministic limit / no-match 语义
- 补齐 evidence kind filter 边界，并保持 explicit evidence ids 只有通过服务端 validation 后才是 authoritative
- explicit namespace filter 的首片已经补齐 event namespace schema / migration；不能用 owner、summary 或 claim namespace 代替 event namespace
- explicit reflection evidence lookup 继续走直接 store filter，空查询结果仍是 `invalid_params`
- self-revision proposal 只允许在当前 trigger window 内 bounded narrowing，空查询结果不能绕过 query 去做更宽搜索
- project / user scoped conflict 与 periodic trigger window 会先排除 sibling namespace 事件，但这仍不是 cross-namespace search / ranking 能力
- 后续仍需补独立 evidence kind 语义

原因：

- 当前已持久化 `events.namespace`，但这只是 namespace-aware narrowing 首片，不代表 richer evidence discovery 已落地
- v2 的价值是让 evidence 查找更可审计，而不是引入 ranking、relation graph、weight scoring、cross-namespace widening 或 autonomous evidence search
- 这一步继续保持 `run_reflection` 是 identity / commitment durable update 的唯一写路径

## 中期

### 1. 扩展更多 provider 类型

目标：

- 在现有 provider 枚举与 provider-specific config 结构上继续增加更多 provider；当前已完成 OpenRouter 的本地 OpenAI-compatible transport 切片
- 保持应用层与领域层不感知第三方协议细节
- 新增 provider 前先通过 [Provider Readiness Checklist](provider-contract.md)，尤其是 config validation、`doctor` redaction、timeout、非成功状态、malformed JSON、decision/self-revision/evidence policy 解析和 `stdio` provider path

候选方向：

- Azure OpenAI
- OpenRouter live-provider certification
- 本地模型网关

### 2. 把 reflection 从“显式输入 + 可选查询 + 最小 deep update”推进到 richer 语义

目标：

- 保留显式 `replacement_evidence_event_ids`
- 保留现有最小 `identity_core` / `commitments` 更新能力
- 在现有窄化查询基础上扩展 richer query 语义、更广泛 evidence 能力、更清晰的权重/关联关系与更稳定的 deep-update policy
- 近期只承诺一个 bounded evidence query v2 slice；更丰富的 weighting / relation / ranking / cross-window evidence semantics 仍留在中期

### 3. 丰富 evidence 语义

目标：

- 不再只判断 event 是否存在
- 逐步引入更清晰的 evidence-oriented 查询与关联能力

### 4. 丰富 `episodes` / `identity_core` / claim schema

目标：

- 把当前骨架式数据结构逐步推进到更有表达力的形态
- 让“状态快照”更接近原始设计里的语义目标

## 后期

### 1. 扩展到更完整的 memory 分层

目标：

- working memory
- episodic memory
- semantic memory
- procedural memory
- 以及更完整的 slow variable / policy / self-model layering

### 2. 推进正式产品化封装

候选方向：

- 更清晰的隔离策略
- 更丰富的 transport
- 更稳定的配置与部署方式

当前该方向已进入下一阶段产品化规划，实施顺序以 [Productization Roadmap After MVP](superpowers/plans/2026-05-09-productization-roadmap.md) 为准。

多层 memory 方向已收敛到 [Memory Layering Roadmap](product/memory-layering-roadmap.md)。该文档只定义未来阶段和 first slice，不改变当前 Local Alpha 的产品承诺。

## 当前不纳入近期承诺的事项

- 未经产品化设计和安全 gate 的远程 HTTP 服务
- 多租户或多 Agent 编排
- 远程、可写或多租户可视化管理后台
- 持续后台自治运行或完整 daemon 化自我治理
- 把仓库包装成”生产级完整自我机制产品”

完整自治、生产级 self-governing、远程管理、认证/多租户、以及”所有入口自动反思”的 daemon 均进入产品化路线图，但必须按阶段 gate 推进，不能从当前 MVP 直接宣称已完成。详见 [Productization Roadmap After MVP](superpowers/plans/2026-05-09-productization-roadmap.md) 和 [Future Autonomy Productization Spec Index](superpowers/specs/2026-04-27-future-autonomy-productization-index.md)。
