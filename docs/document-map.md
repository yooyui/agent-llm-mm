# 文档总览

本文件用于把当前仓库里的文档分成两层：

- 稳定入口文档：适合放到 GitHub 给协作者快速理解当前项目
- 原始资料与历史快照：保留研究过程、实现复核和阶段性判断

## 1. 建议先读

- [README.md](/D:/Code/agent_llm_mm/README.md)
  - 仓库首页入口、快速开始、文档导航
- [CONTRIBUTING.md](/D:/Code/agent_llm_mm/CONTRIBUTING.md)
  - 公开协作入口、验证要求与文档更新预期
- [project-status.md](/D:/Code/agent_llm_mm/docs/project-status.md)
  - 当前实现边界、已实现 / 部分实现 / 未实现
- [release-readiness.md](/D:/Code/agent_llm_mm/docs/release-readiness.md)
  - 当前 demo 是否适合发布到 GitHub
- [roadmap.md](/D:/Code/agent_llm_mm/docs/roadmap.md)
  - 近期 / 中期 / 后期规划

## 2. 三语项目说明

- [project-overview.zh-CN.md](/D:/Code/agent_llm_mm/docs/project-overview.zh-CN.md)
- [project-overview.en.md](/D:/Code/agent_llm_mm/docs/project-overview.en.md)
- [project-overview.ja.md](/D:/Code/agent_llm_mm/docs/project-overview.ja.md)

这些文件适合直接转发给中文、英文和日文协作者，用来快速解释项目是什么、现在到哪一步、边界在哪里。

## 2.1 发布物料

- [github-publish-prep-2026-03-31.md](/D:/Code/agent_llm_mm/docs/github-publish-prep-2026-03-31.md)
  - GitHub description、topics、首页文案和发布阻塞项
- [2026-03-31-initial-public-release.md](/D:/Code/agent_llm_mm/docs/releases/2026-03-31-initial-public-release.md)
  - 首次公开发布的 release note 草稿

## 3. 接入与验证文档

- [local-mcp-integration-2026-03-26.md](/D:/Code/agent_llm_mm/docs/local-mcp-integration-2026-03-26.md)
  - 如何把本项目接入本机 AI 客户端
- [testing-guide-2026-03-24.md](/D:/Code/agent_llm_mm/docs/testing-guide-2026-03-24.md)
  - 当前测试基线、推荐验证顺序和常见问题排查
- [examples/codex-mcp-config.toml](/D:/Code/agent_llm_mm/examples/codex-mcp-config.toml)
  - Codex 本机 MCP 配置样例

## 4. 原始资料与历史快照

### 原始讨论资料

- [llm-agent-memory-self-dialogue-2026-03-23.zh-CN.md](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-2026-03-23.zh-CN.md)
  - 原始讨论的整理稿 / 提炼稿
- [llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md](/D:/Code/agent_llm_mm/docs/llm-agent-memory-self-dialogue-raw-log-2026-03-23.zh-CN.md)
  - 逐轮原始日志，保留上下文和表达顺序

### 历史实现复核

- [current-work-2026-03-24.md](/D:/Code/agent_llm_mm/docs/current-work-2026-03-24.md)
  - 较早阶段的实现状态快照
- [current-work-2026-03-25.md](/D:/Code/agent_llm_mm/docs/current-work-2026-03-25.md)
  - 按 2026-03-27 复核后的实现状态说明
- [implementation-comparison-2026-03-24.md](/D:/Code/agent_llm_mm/docs/implementation-comparison-2026-03-24.md)
  - 原始设计日志与当前实现的比对

### 阶段规划

- [2026-03-27-plan.md](/D:/Code/agent_llm_mm/docs/2026-03-27-plan.md)
  - 某一轮阶段计划，不等于当前稳定路线图
- [2026-03-28-openai-compatible-provider-claude-code.md](/D:/Code/agent_llm_mm/docs/superpowers/plans/2026-03-28-openai-compatible-provider-claude-code.md)
  - 较早的 provider 实现计划草稿，已被后续配置文件方案替代
- [2026-03-28-openai-compatible-provider-claude-code-design.md](/D:/Code/agent_llm_mm/docs/superpowers/specs/2026-03-28-openai-compatible-provider-claude-code-design.md)
  - 较早的 provider 设计稿，保留用于追溯，不代表当前最终实现

## 5. 阅读建议

- 如果你是第一次看这个仓库：先读 README，再读 `project-status.md` 与 `release-readiness.md`
- 如果你想接入本地客户端：再读 `local-mcp-integration-2026-03-26.md`
- 如果你要继续开发：再读 `testing-guide-2026-03-24.md` 与 `roadmap.md`
- 如果你想追溯设计来源：最后读原始讨论资料和历史快照
