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
- 最新 fresh 验证：`2026-03-31`
  - `cargo test` 全量通过，共 58 个测试
  - `pwsh -File .\scripts\agent-llm-mm.ps1 doctor` 返回 `status = ok`

## 先看这些

- [当前实现状态](docs/project-status.md)
- [发布准备评估](docs/release-readiness.md)
- [未来路线图](docs/roadmap.md)
- [文档总览](docs/document-map.md)

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
- `namespace` 最小闭环
  - `self`
  - `world`
  - `user/<id>`
  - `project/<id>`
- `openai-compatible` provider
  - 通过配置文件选择 provider
  - 支持本地 `chat/completions` 风格兼容接口
- 本机接入链路
  - `scripts/agent-llm-mm.ps1` 支持 `doctor` / `serve`
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
- `episodes`
  - 当前主要是 `episode_reference -> event_id` 的轻量聚合
- 默认数据库作用域
  - 已是文件型 SQLite
  - 但“按用户共享 / 按项目隔离 / 按 workspace 隔离”的正式语义尚未定型

### 未实现

- 自动 evidence lookup
- evidence weight / relation
- reflection 对 `identity_core` 与 `commitments` 的深层修订
- 更多 provider 类型（如 Azure/OpenRouter/本地模型）
- 更完整的多层 memory 体系

## 快速开始

进入项目目录：

```powershell
Set-Location 'D:\Code\agent_llm_mm'
```

先从示例配置复制一份本地配置文件：

```powershell
Copy-Item .\examples\agent-llm-mm.example.toml .\agent-llm-mm.local.toml
```

然后编辑 `agent-llm-mm.local.toml`：

- 固定自己的 `database_url`
- 选择 `provider`
- 填入自己的 API key

`agent-llm-mm.local.toml` 已被 `.gitignore` 忽略，不应提交到仓库。

如果你只想离线验证，也可以把配置改成：

```toml
[model]
provider = "mock"
```

先做本机预检：

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

如果预检通过，再启动 MCP 服务：

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

服务启动后终端会保持占用并等待 `stdio` JSON-RPC 输入，这是正常现象。

## 验证命令

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

## 文档导航

### 面向 GitHub 协作

- [文档总览](docs/document-map.md)
- [当前实现状态](docs/project-status.md)
- [未来路线图](docs/roadmap.md)
- [发布准备评估](docs/release-readiness.md)
- [协作说明](CONTRIBUTING.md)

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
- 如果多个本机客户端共用同一 SQLite 文件，需要预期 SQLite 单写者模型带来的竞争与锁等待。
- 本地配置文件不应提交到仓库，尤其不要把真实 API key 写进 `examples/`、`docs/` 或测试文件。
- `decide_with_snapshot` 已可走真实 provider，但当前仍是最小动作协议，不建议把它表述成完整决策引擎。

## Acknowledgements

This repository was developed through iterative implementation and discussion with OpenAI Codex. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.

## License

This repository is licensed under the Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Copyright `2026` `yooyui`
