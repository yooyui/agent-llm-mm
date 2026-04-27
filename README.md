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
- 当前状态：适合以“公开技术 demo / MVP”身份发布到 GitHub，不应包装成完整产品
- 最新 fresh 验证：`2026-04-25`
  - `cargo test` 全量通过，共 143 个测试
  - `doctor` 预检返回 `status = ok`
  - self-revision demo package 可一键生成本地证据链

## 先看这些

- [当前实现状态](docs/project-status.md)
- [进度追踪对照表](docs/progress-tracker.md)
- [发布准备评估](docs/release-readiness.md)
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
  - 已具备窄化的 `replacement_evidence_query` 基础能力（结构化首版），用于在反思时提供可选的证据检索起点
  - 已支持带审计记录的最小 `identity_core` / `commitments` 深层修订
- trigger-ledger-backed automatic self-revision MVP
  - 已有 `self_revision` 领域契约、`ModelPort::propose_self_revision` 端口，以及 `mock` / `openai-compatible` proposal adapter
  - proposal 首阶段已带 `proposed_evidence_event_ids`、`proposed_evidence_query`、`confidence` 契约，用于收口证据候选与置信度；其中 `proposed_evidence_query` 在 explicit ids 为空时可作为 bounded narrowing hint，对当前 trigger window 做交集收口，并在有交集时按当前窗口内的候选顺序应用 `limit`；若没有交集则回退到 full trigger window。explicit ids 非空时，这些 ids 也必须满足 query 在当前 trigger window 内的过滤约束，但仍不是 widening/ranking engine
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
- production dashboard service
  - 可通过 `[dashboard]` 配置启停
  - 随 `serve` 启动本机 HTTP 只读观测面板
  - 以 `Memory-chan Live Cockpit` 清新活力二次元风格展示 MCP tool 调用、runtime operation 和 auto-reflection 事件
  - 面板内嵌生成图物料：`src/interfaces/dashboard/static/memory_chan_hero.png` 与 `src/interfaces/dashboard/static/memory_chan_sidebar.png`
  - 保留 decision / snapshot 投影字段用于后续扩展
  - 不改变 MCP tool 列表，不污染 MCP `stdout`
- `namespace` 最小闭环
  - `self`
  - `world`
  - `user/<id>`
  - `project/<id>`
- `openai-compatible` provider
  - 通过配置文件选择 provider
  - 支持本地 `chat/completions` 风格兼容接口
- 本机接入链路
  - 平台入口见对应平台文档
  - MCP `stdio` 当前暴露 4 个工具：
    - `ingest_interaction`
    - `build_self_snapshot`
    - `decide_with_snapshot`
    - `run_reflection`

### 部分实现

- `decide_with_snapshot`
  - commitment gate 已真实生效
  - 已可切到 `openai-compatible` provider
  - 当前只支持返回“动作字符串”的最小协议
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
- `self_snapshot`
  - 当前只有统一 `SnapshotBudget`
  - 主要对 evidence 数量做截断
- `run_reflection`
  - 已可最小更新 `identity_core` 与 `commitments`
  - 但当前仍是“显式 canonical claims / 显式 commitment 列表”的首版契约，不是 richer schema / versioned policy
- `episodes`
  - 当前主要是 `episode_reference -> event_id` 的轻量聚合
- 默认数据库作用域
  - 已是文件型 SQLite
  - 默认语义已收口为“本机用户共享的持久化默认库”
  - 如需项目隔离、实验隔离或测试隔离，应显式设置 `database_url`
- 配置覆盖语义
  - `AppConfig::load_from_path()` 会保留显式文件里的 `database_url`
  - 若显式文件省略 `database_url`，`load_from_path()` 仍可能通过 `AppConfig::default()` 继承环境变量派生出的默认路径
  - `AppConfig::load()` / 默认启动路径仍允许用 `AGENT_LLM_MM_DATABASE_URL` 覆盖数据库位置

### 未实现

- richer 自动 evidence lookup（当前 `replacement_evidence_query` / `proposed_evidence_query` 仍只支持 `owner / kind / limit` 的结构化首版）
- richer evidence weighting / relation / ranking
- evidence weight / relation
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

## 文档导航

### 面向 GitHub 协作

- [文档总览](docs/document-map.md)
- [当前实现状态](docs/project-status.md)
- [进度追踪对照表](docs/progress-tracker.md)
- [未来路线图](docs/roadmap.md)
- [发布准备评估](docs/release-readiness.md)
- [协作说明](CONTRIBUTING.md)
- [macOS 开发与接入指南](docs/development-macos.md)
- [Windows 开发与接入指南](docs/development-windows.md)

### 接入与验证

- [本机 MCP 接入说明](docs/local-mcp-integration-2026-03-26.md)
- [测试指南](docs/testing-guide-2026-03-24.md)
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

Copyright `2026` `yooyui`
