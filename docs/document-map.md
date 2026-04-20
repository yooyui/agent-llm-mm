# 文档总览

本文件用于把当前仓库里的文档分成两层：

- 稳定入口文档：适合放到 GitHub 给协作者快速理解当前项目
- 原始资料与历史快照：保留研究过程、实现复核和阶段性判断

## 1. 一句话说明

`agent_llm_mm` 是一个面向 AI 客户端的本地 Rust MCP `stdio` memory demo，支持 SQLite 持久化、配置文件驱动的 provider 加载、`openai-compatible` 模型接入，以及 trigger-ledger-backed automatic self-revision MVP。

## 2. 多语言项目说明

- [project-overview.zh-CN.md](project-overview.zh-CN.md)
- [project-overview.en.md](project-overview.en.md)
- [project-overview.ja.md](project-overview.ja.md)

这些文件适合先快速理解项目，再决定是否深入到状态、路线图和实现细节。

## 3. 项目介绍与状态入口

- [README.md](../README.md)
  - 仓库首页入口、项目介绍、快速开始、文档导航，以及当前 self-revision MVP 的保守边界
- [development-macos.md](development-macos.md)
  - macOS 环境准备、配置、预检、启动和验证
- [development-windows.md](development-windows.md)
  - Windows 环境准备、配置、预检、启动和验证
- [project-status.md](project-status.md)
  - 当前实现边界、已实现 / 部分实现 / 未实现，重点包含当前 4 条 MCP-wired automatic path、structured diagnostics，以及 `run_reflection` durable write path 的保守范围
- [release-readiness.md](release-readiness.md)
  - 当前 demo 是否适合发布到 GitHub
- [roadmap.md](roadmap.md)
  - 近期 / 中期 / 后期规划，明确哪些是 MVP 延伸，哪些不在近期承诺内

## 4. 建议先读

- [README.md](../README.md)
  - 先看项目一句话说明和多语言入口
- [CONTRIBUTING.md](../CONTRIBUTING.md)
  - 公开协作入口、验证要求与文档更新预期
- [project-status.md](project-status.md)
  - 当前实现边界、已实现 / 部分实现 / 未实现
- [release-readiness.md](release-readiness.md)
  - 当前 demo 是否适合发布到 GitHub
- [roadmap.md](roadmap.md)
  - 近期 / 中期 / 后期规划

## 5. 发布物料

- [github-publish-prep-2026-03-31.md](github-publish-prep-2026-03-31.md)
  - GitHub description、topics、首页文案和发布阻塞项
- [2026-03-31-initial-public-release.md](releases/2026-03-31-initial-public-release.md)
  - 首次公开发布的 release note 草稿
- [2026-04-20-self-revision-runtime-coverage-and-governance-hardening.md](releases/2026-04-20-self-revision-runtime-coverage-and-governance-hardening.md)
  - 本地 runtime coverage 与 evidence governance 收口更新记录

## 6. 接入与验证文档

- [local-mcp-integration-2026-03-26.md](local-mcp-integration-2026-03-26.md)
  - 如何把本项目接入本机 AI 客户端，以及当前 runtime hooks、`doctor` 输出和 self-revision MVP 运行边界
- [testing-guide-2026-03-24.md](testing-guide-2026-03-24.md)
  - 当前测试基线、推荐验证顺序、self-revision runtime coverage / diagnostics / evidence policy 定向回归和常见问题排查
- [examples/codex-mcp-config.toml](../examples/codex-mcp-config.toml)
  - Codex 本机 MCP 配置样例

## 7. 原始资料与历史快照

### 原始讨论资料

- [llm-agent-memory-self-dialogue-2026-03-23.zh-CN.md](llm-agent-memory-self-dialogue-2026-03-23.zh-CN.md)
  - 原始讨论的整理稿 / 提炼稿
- [llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md](llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md)
  - 逐轮原始日志，保留上下文和表达顺序

### 历史实现复核

- [current-work-2026-03-24.md](current-work-2026-03-24.md)
  - 较早阶段的实现状态快照
- [current-work-2026-03-25.md](current-work-2026-03-25.md)
  - 按 2026-03-27 复核后的实现状态说明
- [implementation-comparison-2026-03-24.md](implementation-comparison-2026-03-24.md)
  - 原始设计日志与当前实现的比对

### 阶段规划

- [2026-03-27-plan.md](2026-03-27-plan.md)
  - 某一轮阶段计划，不等于当前稳定路线图
- [2026-04-19-self-agent-memory-self-revision-mvp.md](superpowers/plans/2026-04-19-self-agent-memory-self-revision-mvp.md)
  - 基于 2026-04-19 self-revision 设计初稿拆出的实现计划，默认面向 subagent 执行
- [2026-03-28-openai-compatible-provider-claude-code.md](superpowers/plans/2026-03-28-openai-compatible-provider-claude-code.md)
  - 较早的 provider 实现计划草稿，已被后续配置文件方案替代
- [2026-03-28-openai-compatible-provider-claude-code-design.md](superpowers/specs/2026-03-28-openai-compatible-provider-claude-code-design.md)
  - 较早的 provider 设计稿，保留用于追溯，不代表当前最终实现
- [2026-04-19-self-agent-memory-self-revision-mvp-design.md](superpowers/specs/2026-04-19-self-agent-memory-self-revision-mvp-design.md)
  - 基于原始逐轮日志与再次确认问答整合出的 self-revision MVP 设计初稿

## 8. 阅读建议

- 如果你是第一次看这个仓库：先读 README，再读 `project-status.md` 与 `release-readiness.md`
- 如果你想确认 automatic self-revision 到底实现到哪里：直接读 `project-status.md`，再读 `roadmap.md`
- 如果你想确认 runtime hooks、diagnostics 和 durable write path：读 `project-status.md`，再读 `local-mcp-integration-2026-03-26.md`
- 如果你想接入或开发：先进入对应平台文档，再读 `local-mcp-integration-2026-03-26.md` 与 `testing-guide-2026-03-24.md`
- 如果你想追溯设计来源：最后读原始讨论资料和历史快照
