# MCP Memory Ledger

面向本地 AI Agent 的证据门控记忆与自我修订层。

MCP Memory Ledger 是一个本地优先的 Rust MCP `stdio` 记忆服务。它把交互、证据、claim、自我快照和自我修订记录到 SQLite 中，让 AI 客户端可以在可审计的边界内使用长期记忆，而不是只依赖一次性的 prompt 上下文。

当前项目适合作为本地 Agent 记忆、MCP 集成、SQLite 持久化和受治理自我修订的技术 MVP。它不是生产级自治 Agent 平台，远程团队模式、多租户、安装包发布、daemon 写能力和生产安全边界仍按文档中的门禁分阶段推进。

## 核心能力

- **本地 MCP 记忆服务**：通过 `stdio` 暴露 `ingest_interaction`、`build_self_snapshot`、`decide_with_snapshot`、`run_reflection` 4 个 MCP 工具。
- **SQLite 持久化**：保存 event、claim、evidence、reflection audit、trigger ledger 和 operation log。
- **证据门控自我修订**：claim、identity、commitment 的修订必须经过明确证据和治理规则；`run_reflection` 仍是 identity / commitment / reflection 的唯一持久化写路径。
- **可观测诊断**：提供只读 dashboard、doctor 预检、operation log 查询和本地支持包生成器。
- **Provider 接入**：内置 `mock`、`openai-compatible` 和 OpenRouter 配置路径；provider 密钥只应放在本机私有配置或环境变量中。
- **本地发布门禁**：包含本地 Alpha、provider 预检、打包预检、发布证据索引等只读检查脚本，用于证明边界而不是自动发布。

## 适合场景

- 给本地 AI 客户端接入 MCP memory
- 研究 Agent 如何基于证据更新长期记忆
- 验证 self snapshot、reflection、commitment gate 的最小闭环
- 作为 Rust + SQLite + MCP `stdio` 项目的工程参考
- 生成本地诊断材料，定位 provider、配置、dashboard 或 operation log 问题

## 快速开始

macOS 日常入口：

```zsh
./scripts/agent-llm-mm.sh bootstrap-local
./scripts/agent-llm-mm.sh doctor
./scripts/agent-llm-mm.sh serve
```

Windows 日常入口：

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 bootstrap-local
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

`bootstrap-local` 会从 dev 示例生成本机配置模板，不覆盖已有文件，不生成 secret，也不会启动服务。更多平台细节见：

- [macOS 开发与接入指南](docs/development-macos.md)
- [Windows 开发与接入指南](docs/development-windows.md)
- [本机 MCP 接入说明](docs/local-mcp-integration-2026-03-26.md)

## 演示

运行可重复的 self-revision demo：

```zsh
./scripts/run-self-revision-demo.sh
```

该 demo 会启动本地 deterministic `openai-compatible` stub provider，通过真实 MCP `stdio` 服务跑 canonical scenario，并把报告写入 `target/reports/self-revision-demo/...`。

## 本地诊断

生成脱敏支持包：

```zsh
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
```

支持包只包含脱敏 JSON 摘要，不复制完整 SQLite 数据库、raw TOML、provider payload 或原始 `.log` 文件。需要导出日志片段或聚焦某次 MCP tool call 时，请显式传入 `--log-file` 或 `--correlation-id`。

汇总本地 Alpha 门禁状态：

```zsh
./scripts/local-alpha-evidence-summary.sh \
  --evidence-root . \
  --output-json target/reports/local-alpha/evidence-summary.json \
  --output-md target/reports/local-alpha/evidence-summary.md
```

该命令只读取已有本地证据并输出汇总，不生成缺失证据，也不自动认证本地 Alpha。

## 当前边界

已实现：

- MCP `stdio` 主链路
- SQLite 持久化和 owner / namespace 约束
- `run_reflection` 审计式 claim 替换
- 最小 identity / commitment 修订
- 基于 trigger ledger 的自动自我修订 MVP
- 只读 dashboard、doctor、本地支持包和本地门禁汇总脚本

部分实现：

- `decide_with_snapshot` 仍围绕动作字符串协议，不是完整决策引擎
- episode 目前主要是轻量 projection，不是完整自传式记忆模型
- provider live evidence 只证明配置和连通性，不证明模型质量、SLA 或生产可用性
- 本地 Alpha 门禁仍依赖真实 fresh-machine、Windows parity 和人工 release decision 等外部证据

未实现：

- 完整 memory layering
- richer evidence ranking / weighting
- 生产级 remote / team / multi-tenant 能力
- daemon 写能力和后台自治运行
- 安装包、service manager、auto-updater 和发布认证流程

完整实现状态见 [当前实现状态](docs/project-status.md) 和 [路线图](docs/roadmap.md)。

## 文档

- [项目定位与搜索优化口径](docs/positioning.md)
- [常见问题](docs/faq.md)
- [中文项目说明](docs/project-overview.zh-CN.md)
- [英文项目说明](docs/project-overview.en.md)
- [日文项目说明](docs/project-overview.ja.md)
- [测试指南](docs/testing-guide-2026-03-24.md)
- [发布准备评估](docs/release-readiness.md)
- [发布门禁运行手册](docs/release-gate.md)
- [本地 Alpha 产品需求说明](docs/product/prd-local-alpha.md)
- [Provider 就绪检查清单](docs/provider-contract.md)
- [文档总览](docs/document-map.md)

## 验证

截至 `2026-06-08`，`cargo test` 全量通过，共 400 个测试。

常用本地检查：

```zsh
cargo test
./scripts/agent-llm-mm.sh doctor
git diff --check
```

涉及 provider、dashboard、发布证据或支持包的改动，请按 [测试指南](docs/testing-guide-2026-03-24.md) 跑对应分层验证。

## 命名兼容

对外项目名是 MCP Memory Ledger。当前 Rust crate、binary、脚本、配置样例和部分历史文档仍使用 `agent_llm_mm` / `agent-llm-mm` 作为技术标识；这属于兼容保留，不代表项目名仍是旧名称。

## 许可证

本仓库采用 Apache License 2.0，详见 [LICENSE](LICENSE) 与 [NOTICE](NOTICE)。

Copyright 2026 yooyui
