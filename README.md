# agent_llm_mm

## 一句话说明

一个面向 AI 客户端的本地 Rust MCP `stdio` memory demo，支持 SQLite 持久化、配置文件驱动的 provider 加载，以及 `openai-compatible` 模型接入。

## 主多语言文档

- [项目说明（中文）](docs/project-overview.zh-CN.md)
- [Project Overview (English)](docs/project-overview.en.md)
- [プロジェクト概要（日本語）](docs/project-overview.ja.md)

## 项目介绍

- 类型：本机 `stdio` MCP 服务
- 存储：SQLite
- 适用场景：可启动本地 MCP 子进程的 AI 客户端集成、研究型 demo、工程验证
- 当前状态：适合以“公开技术 demo / MVP”身份发布到 GitHub，不应包装成完整产品
- 最新 fresh 验证：`2026-04-18`
  - `cargo test` 全量通过，共 80 个测试
  - `doctor` 预检返回 `status = ok`

## 先看这些

- [当前实现状态](docs/project-status.md)
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

### 未实现

- richer 自动 evidence lookup（当前仍只支持 `owner / kind / limit` 的结构化首版）
- evidence weight / relation
- `identity_core` / `commitments` 的 richer schema、版本化修订与更细策略
- 更多 provider 类型（如 Azure/OpenRouter/本地模型）
- 更完整的多层 memory 体系

## 快速开始

平台相关的环境准备、配置、预检、启动和验证命令已经拆到独立文档：

- macOS：见 [docs/development-macos.md](docs/development-macos.md)
- Windows：见 [docs/development-windows.md](docs/development-windows.md)

## 文档导航

### 面向 GitHub 协作

- [文档总览](docs/document-map.md)
- [当前实现状态](docs/project-status.md)
- [未来路线图](docs/roadmap.md)
- [发布准备评估](docs/release-readiness.md)
- [协作说明](CONTRIBUTING.md)
- [macOS 开发与接入指南](docs/development-macos.md)
- [Windows 开发与接入指南](docs/development-windows.md)

### 接入与验证

- [本机 MCP 接入说明](docs/local-mcp-integration-2026-03-26.md)
- [测试指南](docs/testing-guide-2026-03-24.md)
- [Codex MCP 配置样例](examples/codex-mcp-config.toml)
- [Provider 配置样例](examples/agent-llm-mm.example.toml)

### 原始资料与历史快照

- [原始讨论整理稿](docs/llm-agent-memory-self-dialogue-2026-03-23.zh-CN.md)
- [原始讨论逐轮日志](docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md)
- [当前工作说明（2026-03-25）](docs/current-work-2026-03-25.md)
- [功能实现比对（2026-03-24）](docs/implementation-comparison-2026-03-24.md)
- [阶段计划（2026-03-27）](docs/2026-03-27-plan.md)

## 接入注意事项

- 这是 `stdio` MCP 服务，不会输出 Web 地址或启动 HTTP 端口。
- 正式使用时请通过 `agent-llm-mm.local.toml` 或 `AGENT_LLM_MM_CONFIG` 提供 provider 配置。
- 建议为正式数据、手工测试数据和实验数据使用不同 SQLite 文件。
- 未显式配置 `database_url` 时，默认库会落到当前平台的用户数据目录，并按“本机用户共享”语义复用。
- 如果多个本机客户端共用同一 SQLite 文件，需要预期 SQLite 单写者模型带来的竞争与锁等待。
- 本地配置文件不应提交到仓库，尤其不要把真实 API key 写进 `examples/`、`docs/` 或测试文件。
- `decide_with_snapshot` 已可走真实 provider，但当前仍是最小动作协议，不建议把它表述成完整决策引擎。

## Acknowledgements

This repository was developed through iterative implementation and discussion with OpenAI Codex. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.

## License

This repository is licensed under the Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright `2026` `yooyui`
