# agent_llm_mm

## 一句话说明

一个面向 AI 客户端的本地 Rust MCP `stdio` memory demo，支持 SQLite 持久化、配置文件驱动的 provider 加载、`openai-compatible` 模型接入，以及 trigger-ledger-backed automatic self-revision MVP。

## 主多语言文档

- [项目说明（中文）](docs/project-overview.zh-CN.md)
- [Project Overview (English)](docs/project-overview.en.md)
- [プロジェクト概要（日本語）](docs/project-overview.ja.md)

## 项目介绍

- 类型：本机 `stdio` MCP 服务
- 存储：SQLite
- 适用场景：可启动本地 MCP 子进程的 AI 客户端集成、研究型 demo、工程验证
- 当前状态：MVP release gate 已通过，适合以“已验证本地 MVP，进入正式产品化路线”对外说明；正式产品能力仍按产品化 gate 分阶段推进
- 最新 fresh 验证：`2026-05-24`
  - `cargo test` 全量通过，共 291 个测试
  - `doctor` 预检返回 `status = ok`
  - `status-sync-check` 已加入本地只读文档漂移检查，用于对齐当前测试总数声明，并阻断已勾选计划项与 reality gate 状态不一致的完成声明
  - Local Alpha product smoke 通过 staging / promote 流程刷新本地证据链
  - Local Alpha support bundle 生成本地脱敏诊断 JSON，未包含 `.sqlite` 或 `.toml` 文件
  - Local first-run bootstrap smoke 已加入脚本入口，用于模拟 `bootstrap-local -> doctor` 的本地首启证据
  - Local Alpha evidence summary 已加入本地只读 gate 状态汇总入口；它不运行 product smoke、不启动服务、不上传文件、不认证 Local Alpha 完成
  - Product readiness checker 已加入候选级本地门禁汇总，会把真实 fresh-machine、Windows parity、release decision、remote/team、安全/auth 和产品措辞 gate 缺口保持为 blocked
  - Local release soak runner 已加入本地 release evidence 入口；它生成 candidate-specific evidence directory，不生成 Windows runner、真实 fresh-machine、remote/team 或发布认证证据
  - SQLite backup / restore 本地脚本门禁已覆盖备份恢复 roundtrip、拒绝覆盖、拒绝 live DB 子目录备份和拒绝 `..` restore target

## 先看这些

- [当前实现状态](docs/project-status.md)
- [进度追踪对照表](docs/progress-tracker.md)
- [发布准备评估](docs/release-readiness.md)
- [Release Gate Runbook](docs/release-gate.md)
- [Local Product Alpha PRD](docs/product/prd-local-alpha.md)
- [Local Alpha 数据生命周期](docs/product/data-lifecycle.md)
- [产品化二次跟进现实 Gate](docs/product/follow-up-reality-gates.md)
- [Structured Decision Protocol](docs/product/structured-decision-protocol.md)
- [Provider Readiness Checklist](docs/provider-contract.md)
- [Release Engineering](docs/product/release-engineering.md)
- [正式产品化路线图](docs/superpowers/plans/2026-05-09-productization-roadmap.md)
- [正式产品化 12 项后续工作规划](docs/superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md)
- [远程 / 团队模式边界](docs/product/remote-team-mode-boundary.md)
- [多层 Memory 路线图](docs/product/memory-layering-roadmap.md)
- [未来路线图](docs/roadmap.md)
- [文档总览](docs/document-map.md)

## 平台入口

- 当前在 macOS 上开发或接入：读 [macOS 开发与接入指南](docs/development-macos.md)
- 当前在 Windows 上开发或接入：读 [Windows 开发与接入指南](docs/development-windows.md)

## 公开仓库说明

本仓库以公开协作仓库的方式整理，目标是让协作者能够清楚理解三件事：

- 这个项目当前已经实现到什么程度
- 哪些能力仍然只是 MVP / demo 边界
- 原始设计讨论、实现复核和后续路线图分别在哪里看

同时，这个仓库的开发、讨论和文档整理过程明确使用了 OpenAI Codex 作为协作式开发工具。它参与了需求讨论、实现推进、文档收口和发布前整理。对 OpenAI 提供的工具与研究方向，项目在此表示感谢。

## 当前能力

### 已实现

- `ingest_interaction`
  - 记录交互事件并持久化派生命题
  - 支持显式 `namespace` 与 `episode_reference`
- `build_self_snapshot`
  - 从持久化的 `identity / commitments / active claims / evidence / episodes` 组装快照
- `run_reflection`
  - 以审计友好的方式 supersede 既有 claim
  - 支持显式 `replacement_evidence_event_ids`
  - 已具备窄化的 `replacement_evidence_query` 基础能力，支持 namespace / owner / kind / limit 过滤；空查询结果仍返回 `invalid_params`
  - 已支持带审计记录的最小 `identity_core` / `commitments` 深层修订
- trigger-ledger-backed automatic self-revision MVP
  - 已有 `self_revision` 领域契约、`ModelPort::propose_self_revision` 端口，以及 `mock` / `openai-compatible` proposal adapter
  - proposal 首阶段已带 `proposed_evidence_event_ids`、`proposed_evidence_query`、`confidence` 契约，用于收口证据候选与置信度；其中 `proposed_evidence_query` 在 explicit ids 为空时可作为 bounded narrowing hint，对当前 trigger window 做交集收口，并在有交集时按当前窗口内的候选顺序应用 `limit`；project / user scoped conflict 与 periodic trigger window 会先排除 sibling namespace 事件；若没有交集，不再改用完整 trigger window。explicit ids 非空时，这些 ids 也必须满足 query 在当前 trigger window 内的过滤约束，但仍不是 widening/ranking engine
  - 已有 trigger ledger 持久化、cooldown 去重，以及带 structured trigger / rejection / suppression / cooldown 信息的 handled/rejected/suppressed 诊断
  - 当前 MCP-wired automatic path 只有 4 条：
    - `ingest_interaction -> failure`
    - `ingest_interaction -> conflict`
    - `decide_with_snapshot -> conflict`
    - `build_self_snapshot -> periodic`
  - 这些 automatic path 仍是 best-effort runtime hook，不代表“所有请求都会自动反思”
  - 通过治理后的 proposal 会被转译回现有 `run_reflection` 持久化路径；没有新增独立 MCP tool
  - 直接调用 `run_reflection` 不会递归触发 auto-reflection
- self-revision demo package
  - 提供零外网依赖的一键 demo：`./scripts/run-self-revision-demo.sh`
  - demo runner 会启动本地 deterministic `openai-compatible` stub provider，并通过真实 MCP `stdio` 服务跑 canonical scenario
  - 输出 `doctor.json`、snapshot before / after、decision before / after、timeline、SQLite summary 和 Markdown report
  - 该 demo 只证明当前 MVP 边界内的可重复链路，不新增 MCP tool、daemon 或产品化运行形态
- local read-only dashboard service
  - 可通过 `[dashboard]` 配置启停
  - 随 `serve` 启动本机 HTTP 只读观测面板
  - 以 `Memory-chan Live Desk` 清新活力二次元风格展示 MCP tool 调用、runtime operation 和 auto-reflection 事件
  - 面板内嵌生成图物料：`src/interfaces/dashboard/static/memory_chan_hero.png` 与 `src/interfaces/dashboard/static/memory_chan_sidebar.png`
  - 保留 decision / snapshot 投影字段用于后续扩展
  - MCP tool 调用会生成 `mcp-tool-call-<uuid-v4>` correlation id，并在 dashboard 成功/失败事件、auto-reflection 诊断事件和 detail projection 中保留
  - 已知 MCP tool 调用在 object-shaped arguments 进入项目 handler 后，成功与 handler-reached 失败都会追加 tool-level `operation_log` 元数据，便于按 correlation id 排查；framework-level 解析/路由失败不在该 handler-level 记录范围内。这只是 observability metadata，不是新的 identity / commitment / reflection 写路径，也不改变 MCP error 语义
  - 本机只读 dashboard API 已提供 `GET /api/operation-log` 与 `GET /api/operation-log/{id}` durable history 查询，可按 `limit`、`namespace`、`kind`、`correlation_id` 做受限排障；history 列表默认最多返回 100 条，单次查询最大 100 条
  - 不改变 MCP tool 列表，不污染 MCP `stdout`
- local support bundle generator
  - 提供首版本机排障材料生成入口：`./scripts/generate-support-bundle.sh <output_dir> [config_path] [--log-file <path>] [--correlation-id <id>]`
  - `<output_dir>` 必须不存在或为空，避免旧的本地文件混入可分享支持包目录
  - 输出 redacted `doctor` shape、config shape、bounded operation summaries、release metadata、product smoke summary 和 manifest
  - 可在显式传入生成型 `--correlation-id mcp-tool-call-<uuid-v4>` 时，仅导出该 correlation id 对应的 operation-log metadata，并把 filter 形状写入 `operation-summaries.json`
  - 可在显式传入 `--log-file <path>` 时输出 bounded / redacted `local-log-excerpts.json`，但不会自动扫描日志目录或复制原始 `.log` 文件
  - 默认不复制完整 SQLite 数据库、不包含 raw TOML、不包含 provider payload、不上传数据
  - `manifest.json` 会输出只读、未 runtime bootstrap、未包含 SQLite/TOML/raw log/provider payload 的显式 safety checks
  - 该能力只是 Local Alpha 诊断辅助，不代表生产支持通道、远程上传能力或 Local Alpha 已完成
- local alpha evidence summary
  - 提供本地只读 gate 汇总入口：`./scripts/local-alpha-evidence-summary.sh`
  - 读取已有 product smoke latest、first-run bootstrap summary、Windows parity summary 和 support bundle 目录，输出 JSON 与可选 Markdown
  - 每个 gate 输出 `name`、`status`、`evidence_path` 或 `reason`；fresh-machine 或 Windows 证据缺失时会保持 `in_progress` / `not_verified` 等保守状态
  - 不启动 `serve`，不运行 product smoke，不上传文件，不触发 daemon 写，不新增 durable write path；`run_reflection` 仍是唯一 durable identity / commitment / reflection 写路径
  - 该能力只是可审查的状态汇总，不是自动认证，也不代表 Local Alpha 已完成
- product readiness checker
  - 提供候选级本地只读 gate 入口：`./scripts/product-readiness-check.sh <release-candidate> [evidence_root]`
  - 读取 Local Alpha evidence summary、release decision artifact、remote/team capability inventory、security/auth gates 和 product wording guard
  - `real_fresh_machine`、`windows_parity`、`release_decision`、`remote_team`、`security_auth` 或 blocked wording 缺失时保持 `ready = false`
  - 不运行 smoke、不启动服务、不上传文件、不认证 Local Alpha、Beta、remote/team 或 GA
- local release decision artifact
  - 提供 source-only release decision 模板/生成器：`./scripts/release-decision-local.sh <candidate-name> <evidence-root>`
  - 记录 candidate、evidence directory、open gates、human decision、reviewer、rollback note 和 non-claims
  - evidence summary 仍为 `in_progress` 时不能生成 approved 决策
- local alpha release-gate refresh
  - 提供本地可重复 gate 刷新入口：`./scripts/local-alpha-release-gate-refresh.sh [config_path]`
  - 串联 product smoke、first-run bootstrap simulation、support bundle generation 和 evidence summary 输出
  - 只刷新本机可产生的证据，不生成 Windows runner、真实 fresh-machine、remote/team、上传或发布决策证据
  - 刷新后若 summary 仍为 `in_progress` / `not_verified`，应保留对应 open gate，而不是改写成 Local Alpha 完成声明
- local release soak runner
  - 提供本地 release evidence 入口：`./scripts/release-soak-local.sh <candidate-name> [config_path]`
  - 写入 `target/reports/releases/<candidate-name>/`，记录 git HEAD / status、命令日志、doctor、dashboard HTTP 回归、product smoke、first-run simulation、support bundle、secret / artifact scan、support bundle / product smoke SHA-256 manifest 和 Local Alpha evidence summary
  - 只生成本地候选证据，不创建 source tag、binary package、installer、service manager、auto-updater、Windows runner、真实 fresh-machine、remote/team、上传或发布认证证据
- local first-run bootstrap helper / smoke
  - `./scripts/agent-llm-mm.sh bootstrap-local [config_path]` 和 PowerShell 等价入口可从 dev 示例生成本机配置模板
  - 默认目标是 `agent-llm-mm.local.toml`，也可显式传入目标路径
  - 如果显式目标是相对路径，入口脚本会按仓库根目录解析；跨目录调用时建议传绝对路径
  - 目标已存在时拒绝覆盖，父目录不存在时拒绝创建，避免误写用户配置
  - 只复制 `examples/agent-llm-mm.dev.example.toml`，不生成 secret，不运行 `doctor`，不启动 `serve` 或 daemon
  - `./scripts/first-run-bootstrap-smoke-local.sh [output_dir]` 会在隔离输出目录里模拟 `bootstrap-local -> doctor`，生成 `doctor.json` 与 `summary.json`，并把 SQLite 指向该输出目录
  - 该 smoke 会清理 config/database 环境变量干扰，不写真实 HOME，不启动 `serve`，不调用 product smoke，不触发 daemon 写能力
  - 该能力只是 first-run 配置引导和本地 fresh-machine simulation evidence，不是 installer、packager、远程 bootstrapper、真实 fresh-machine / Windows runner 证据或 Local Alpha 完整 gate 证明
- local SQLite backup / restore script gate
  - `./scripts/backup-sqlite.sh` 和 `./scripts/restore-sqlite.sh` 已纳入 `cargo test --test sqlite_backup_restore -v` 本地回归门禁
  - 测试覆盖 backup -> restore-to-new-path roundtrip、restore 拒绝覆盖已有目标、backup 拒绝 live database 目录树内备份、in-memory SQLite 拒绝、invalid percent encoding 拒绝和 `..` restore target 拒绝
  - restore 仍只写新路径，正式库是否切换仍由人工在 `doctor` 验证后决定
  - 该能力不是远程备份、云同步、定时 daemon、生产灾备、admin/auth 或团队模式能力
- observe-only daemon diagnostics
  - `doctor` 输出新增 `daemon_observe_only` 本机只读诊断字段，用于展示 observe-only 模式、数据源、候选计数、cooldown 状态、并发占用和读取错误
  - 当 `[daemon].enabled = true` 时，diagnostics 只读取本地 `operation_log` 中 `tool` / `trigger` 的 `failed` 与 `suppressed` 候选，并保持 `write_gate_approved = false`、`writes_allowed = false`、`remote_listener_enabled = false`
  - `DaemonHandle` 已有本地 start / stop 生命周期回归，证明 disabled 模式会快速退出、observe-only 模式可干净关闭，且写 gate 与 remote listener 仍为关闭
  - 这只是 daemon 写能力前的观察 gate，不调用 `run_reflection`，不新增 identity / commitments / reflection durable write path，也不代表后台自治或 Local Alpha 已完成
- decision / evidence / episode read-model slices
  - `decide_with_snapshot` response envelope 已升级到 `protocol_version = 2`，新增 `decision_id`、`requested_action`、`selected_action`、`confidence`、`policy_checks` 和 `non_claims`
  - 旧的 `blocked` / `decision` 字段保持兼容；`decision` 内仍是最小 `action` 字符串
  - 新增只读 evidence relation read model，显式暴露 trigger window 内的 selected subset、window rank 和 no-widening policy
  - 新增只读 episode summary projection，把 objective / outcome / linked evidence ids 表示为本地 metadata，不写 identity 或 commitments
- remote/team and memory gates
  - `doctor` 现在输出 `remote_team_capability_inventory` 和 `remote_team_security_gates` 机器可读字段，所有 remote/team 能力默认 blocked，support bundle upload 为 false
  - security/auth gates 覆盖 auth、authorization、audit、rate limit、tenant isolation 和 rollback，全部通过前 remote writes 保持 blocked
  - 新增只读 layered memory projection，标记 working / episodic / semantic / procedural / self_model 层为 `partial` 或 `not_implemented`，不新增 durable self-model 写路径
- `namespace` 最小闭环
  - `self`
  - `world`
  - `user/<id>`
  - `project/<id>`
  - event / claim / trigger ledger 均保留明确 namespace，legacy event rows 会按 owner 规则回填
- `openai-compatible` provider
  - 通过配置文件选择 provider
  - 支持本地 `chat/completions` 风格兼容接口
- 本机接入链路
  - 平台入口见对应平台文档
  - first-run 配置可先用 `bootstrap-local` 生成本机 dev 模板，再运行 `doctor` 和 `serve`
  - MCP `stdio` 当前暴露 4 个工具：
    - `ingest_interaction`
    - `build_self_snapshot`
    - `decide_with_snapshot`
    - `run_reflection`

### 部分实现

- `decide_with_snapshot`
  - commitment gate 已真实生效
  - 已可切到 `openai-compatible` provider
  - 返回 envelope 已有 `protocol_version = 2`、`decision_id`、`requested_action`、`selected_action`、`confidence`、`policy_checks`、`non_claims` 和 commitment-gate metadata，同时保留旧的 `blocked` / `decision` 字段
  - 当前仍是围绕动作字符串的兼容协议，不是完整决策引擎
- self-revision 触发面与治理深度
  - 当前 trigger type 已有 `failure / conflict / periodic` 契约，协调器与 ledger 也支持这些类型
  - 当前 MCP runtime coverage 已谨慎接到 4 条路径：`ingest_interaction -> failure`、`ingest_interaction -> conflict`、`decide_with_snapshot -> conflict`、`build_self_snapshot -> periodic`
  - `ingest_interaction -> conflict` 仍要求显式 `trigger_hints` 包含 `conflict` 或 `identity`
  - `decide_with_snapshot` 与 `build_self_snapshot` 仍要求显式 `auto_reflect_namespace`，`decide_with_snapshot` 还需要显式 conflict-compatible `trigger_hints`，并且只在非 blocked 决策后做 best-effort conflict auto-reflection
  - 当前没有“所有 MCP entry point 自动反思”的统一运行形态，也没有后台 daemon / 定时自治进程
  - 当前 evidence 选择、trigger 判定和 deep-update 校验仍是保守的 MVP 规则，不是完整自治治理系统
- provider 扩展性
  - 已保留 provider 枚举与 provider-specific config 结构
  - 但目前只内建 `mock` 与 `openai-compatible`
  - `doctor.provider_matrix` 会把 `azure-openai`、`openrouter`、`local` 标为 `planned-only` 且不可配置，并输出缺失的 config parser、doctor diagnostics、model adapter、error handling、redaction 和 MCP stdio tests；这不是可运行 adapter
- `self_snapshot`
  - 当前只有统一 `SnapshotBudget`
  - 主要对 evidence 数量做截断
- `run_reflection`
  - 已可最小更新 `identity_core` 与 `commitments`
  - 但当前仍是“显式 canonical claims / 显式 commitment 列表”的首版契约，不是 richer schema / versioned policy
- `episodes`
  - 当前主要是 `episode_reference -> event_id` 的轻量聚合
  - 只读 episode summary projection 已能表达 objective / outcome / linked evidence ids；这不是完整自传式 episode schema
- 默认数据库作用域
  - 已是文件型 SQLite
  - 默认语义已收口为“本机用户共享的持久化默认库”
  - 如需项目隔离、实验隔离或测试隔离，应显式设置 `database_url`
- 配置覆盖语义
  - `AppConfig::load_from_path()` 会保留显式文件里的 `database_url`
  - 若显式文件省略 `database_url`，`load_from_path()` 仍可能通过 `AppConfig::default()` 继承环境变量派生出的默认路径
  - `AppConfig::load()` / 默认启动路径仍允许用 `AGENT_LLM_MM_DATABASE_URL` 覆盖数据库位置

### 未实现

- richer 自动 evidence lookup（当前 `replacement_evidence_query` / `proposed_evidence_query` 仍只是 namespace / owner / kind / inclusive recency window / limit 的窄化查询；只读 relation projection 已有首片，但独立 evidence kind、weighting 和完整 ranking engine 仍未实现）
- richer evidence weighting / full ranking engine
- `identity_core` / `commitments` 的 richer schema、版本化修订与更细策略
- 更多 provider 类型（如 Azure/OpenRouter/本地模型）
- 更完整的多层 memory 体系
- 持续后台自治运行、独立 daemon 调度或完整 self-governing agent 行为

## 快速开始

平台相关的环境准备、配置、预检、启动和验证命令已经拆到独立文档：

- macOS：见 [docs/development-macos.md](docs/development-macos.md)
- Windows：见 [docs/development-windows.md](docs/development-windows.md)

如果想快速验证当前 automatic self-revision MVP 的真实效果，可以直接运行：

```zsh
./scripts/run-self-revision-demo.sh
```

该命令会启动本地 demo stub provider、跑完整 canonical scenario，并把 artifact 输出到 `target/reports/self-revision-demo/...`。

如果要生成本地排障支持包，可以运行：

```zsh
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
# 可选：只导出某次 MCP tool call 的 operation-log metadata
./scripts/generate-support-bundle.sh target/support-bundles/manual-correlation-check --correlation-id mcp-tool-call-018fbc89-9ac1-4f5d-8b2a-1f6f5f27b205
```

输出目录必须不存在或为空；生成器会拒绝非空目录，避免旧的本地文件被误当作支持包内容分享。该支持包只包含脱敏 JSON 摘要，不会复制完整 SQLite 数据库、raw TOML、provider payload 或原始 `.log` 文件。需要日志排障时必须显式传入 `--log-file <path>`，生成器只输出受限的 `local-log-excerpts.json`。需要聚焦某次工具调用时可显式传入生成型 `--correlation-id mcp-tool-call-<uuid-v4>`，生成器只过滤 operation-log metadata，不输出 raw request / response / diagnostic payload。

如果要汇总 Local Alpha gate 状态，可以运行：

```zsh
./scripts/local-alpha-evidence-summary.sh \
  --evidence-root . \
  --output-json target/reports/local-alpha/evidence-summary.json \
  --output-md target/reports/local-alpha/evidence-summary.md
```

该命令只读取本地已有证据并输出 JSON / Markdown 汇总；它不会生成缺失证据，也不会自动认证 Local Alpha。

如果要刷新本机可产生的 Local Alpha gate 证据，可以运行：

```zsh
./scripts/local-alpha-release-gate-refresh.sh [config_path]
```

该命令会依次刷新 product smoke、first-run simulation、support bundle 和 evidence summary；它仍不会生成真实 fresh-machine、Windows runner、remote/team 或发布决策证据。

如果要为 release candidate 生成本机 soak evidence，可以运行：

```zsh
./scripts/release-soak-local.sh local-alpha-YYYYMMDD.1-rc.1 [config_path]
```

该命令会写入 `target/reports/releases/<candidate-name>/`，并记录 doctor、dashboard HTTP 回归、product smoke、first-run simulation、support bundle、secret/artifact scan、support bundle / product smoke SHA-256 manifest 和 evidence summary。它仍不会生成真实 fresh-machine、Windows runner、remote/team、上传、tag、二进制包或发布认证证据。

## 文档导航

### 面向 GitHub 协作

- [文档总览](docs/document-map.md)
- [当前实现状态](docs/project-status.md)
- [进度追踪对照表](docs/progress-tracker.md)
- [未来路线图](docs/roadmap.md)
- [发布准备评估](docs/release-readiness.md)
- [Release Gate Runbook](docs/release-gate.md)
- [Local Product Alpha PRD](docs/product/prd-local-alpha.md)
- [Local Alpha 数据生命周期](docs/product/data-lifecycle.md)
- [产品化二次跟进现实 Gate](docs/product/follow-up-reality-gates.md)
- [Release Engineering](docs/product/release-engineering.md)
- [远程 / 团队模式边界](docs/product/remote-team-mode-boundary.md)
- [多层 Memory 路线图](docs/product/memory-layering-roadmap.md)
- [正式产品化 12 项后续工作规划](docs/superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md)
- [协作说明](CONTRIBUTING.md)
- [macOS 开发与接入指南](docs/development-macos.md)
- [Windows 开发与接入指南](docs/development-windows.md)

### 接入与验证

- [本机 MCP 接入说明](docs/local-mcp-integration-2026-03-26.md)
- [测试指南](docs/testing-guide-2026-03-24.md)
- [Release Gate Runbook](docs/release-gate.md)
- [Release Engineering](docs/product/release-engineering.md)
- [Local Alpha 数据生命周期](docs/product/data-lifecycle.md)
- [远程 / 团队模式边界](docs/product/remote-team-mode-boundary.md)
- [本地与远程威胁模型](docs/security/threat-model-local-and-remote.md)
- [Self-Revision Demo Guide](docs/self-revision-demo-guide-2026-04-24.md)
- [Self-Revision Demo Report](docs/reports/self-revision-demo-2026-04-24.md)
- [Codex MCP 配置样例](examples/codex-mcp-config.toml)
- [Provider 配置样例](examples/agent-llm-mm.example.toml)
- [Self-revision demo 配置样例](examples/agent-llm-mm.demo.example.toml)

### 原始资料与历史快照

- [原始讨论整理稿](docs/llm-agent-memory-self-dialogue-2026-03-23.zh-CN.md)
- [原始讨论逐轮日志](docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md)
- [当前工作说明（2026-03-25）](docs/current-work-2026-03-25.md)
- [功能实现比对（2026-03-24）](docs/implementation-comparison-2026-03-24.md)
- [阶段计划（2026-03-27）](docs/2026-03-27-plan.md)

## 接入注意事项

- 默认形态仍是 `stdio` MCP 服务；只有显式设置 `[dashboard].enabled = true` 时，才会额外启动只读 HTTP dashboard。
- dashboard 的生成图物料和项目归属声明见 [NOTICE](NOTICE)；这些物料只用于本项目的本机观测面板。
- 正式使用时请通过 `agent-llm-mm.local.toml` 或 `AGENT_LLM_MM_CONFIG` 提供 provider 配置。
- 建议为正式数据、手工测试数据和实验数据使用不同 SQLite 文件。
- 未显式配置 `database_url` 时，默认库会落到当前平台的用户数据目录，并按“本机用户共享”语义复用。
- 如果多个本机客户端共用同一 SQLite 文件，需要预期 SQLite 单写者模型带来的竞争与锁等待。
- 本地配置文件不应提交到仓库，尤其不要把真实 API key 写进 `examples/`、`docs/` 或测试文件。
- `decide_with_snapshot` 已可走真实 provider，但当前仍是最小动作协议，不建议把它表述成完整决策引擎。
- 当前 automatic self-revision 是带 ledger、证据门槛和慢更新约束的 MVP；它仍然是本地 `stdio` memory demo，不是完整自治代理系统。

## Acknowledgements

This repository was developed through iterative implementation and discussion with OpenAI Codex. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.

## License

This repository is licensed under the Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright 2026 yooyui
