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
  - 当前实现边界、已实现 / 部分实现 / 未实现，重点包含当前 4 条 MCP-wired automatic path、structured diagnostics、dashboard 只读观测面板，以及 `run_reflection` durable write path 的保守范围
- [self-revision-demo-guide-2026-04-24.md](self-revision-demo-guide-2026-04-24.md)
  - 零外网依赖的一键 self-revision demo package 运行指南与 artifact 说明
- [progress-tracker.md](progress-tracker.md)
  - 把“目标态 / 当前状态 / 当前证据 / 剩余缺口 / 建议下一步”压成一张持续更新的进度追踪对照表
- [release-readiness.md](release-readiness.md)
  - 当前 demo 是否适合发布到 GitHub
- [release-gate.md](release-gate.md)
  - 发布前最小 gate、self-revision 证据 gate、dashboard gate，以及 sandbox 失败的解释口径
- [product/prd-local-alpha.md](product/prd-local-alpha.md)
  - Local Product Alpha PRD，定义目标用户、产品承诺、范围内 / non-goals、用户工作流、Alpha exit gate、验收命令和文档入口；当前口径是 MVP gate 已过并进入产品化路线，但 Local Product Alpha / GA 仍需按后续 gate 完成
- [product/release-gate-local-alpha.md](product/release-gate-local-alpha.md)
  - Local Alpha / product alpha release gate，区分 MVP gate 与产品 gate，收口最低命令、product smoke evidence、self-revision demo 8 个 artifact、dashboard local-only、daemon disabled / observe-only 和产品文案边界
- [product/support-bundle-local-alpha.md](product/support-bundle-local-alpha.md)
  - Local Alpha 支持包 gate，定义首版本地生成器、可分享内容、排除内容、脱敏术语、验证命令和剩余限制；当前不代表生产支持通道或 Local Alpha 已完成
- [product/data-lifecycle.md](product/data-lifecycle.md)
  - Local Alpha 数据生命周期文档，定义 formal / test / demo `database_url` 隔离、SQLite backup / restore-to-new-path、export 边界、保留预期和 schema migration 验证清单
- [product/release-engineering.md](product/release-engineering.md)
  - 正式产品化发布工程规则，定义 source-only artifact、版本命名、changelog、release evidence directory、compatibility matrix、soak test 和 deprecation policy
- [product/daemon-observe-only-gate.md](product/daemon-observe-only-gate.md)
  - daemon observe-only gate，定义 Local Alpha 阶段只读诊断边界、forbidden behavior、required diagnostics 和进入写能力前的退出 gate
- [product/remote-team-mode-boundary.md](product/remote-team-mode-boundary.md)
  - 远程 / 团队模式边界文档，明确 remote read-only first、remote write/admin gate、namespace/database isolation、transport split 和 Local Alpha non-goals
- [product/correlation-id-contract.md](product/correlation-id-contract.md)
  - correlation ID contract，定义 MCP tool call 级 `mcp-tool-call-<uuid-v4>`、dashboard / operation-log 传播和不越过 `run_reflection` 的观测边界
- [product/memory-layering-roadmap.md](product/memory-layering-roadmap.md)
  - 多层 memory 产品方向路线图，定义 working / episodic / semantic / procedural memory、slow variables 和 self-model layering 的未来阶段，不代表当前已实现
- [security/threat-model-local-and-remote.md](security/threat-model-local-and-remote.md)
  - 本地与远程 surface 的威胁模型，覆盖 SQLite data、provider credentials、reflection audit、operation log、support bundles、trust boundaries 和 remote/team 前置 mitigations
- [provider-contract.md](provider-contract.md)
  - 新增 provider 前的就绪清单，覆盖配置校验、`doctor` 脱敏、错误处理、解析契约和现有测试映射
- [roadmap.md](roadmap.md)
  - 近期 / 中期 / 后期规划，明确哪些是 MVP 延伸，哪些不在近期承诺内
- [2026-05-09-productization-roadmap.md](superpowers/plans/2026-05-09-productization-roadmap.md)
  - MVP release gate 之后的正式产品化路线图，按 local alpha、durable observability、controlled beta、remote/team mode 和 GA readiness 分阶段推进
- [2026-05-09-local-product-alpha-development-tasks.md](superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md)
  - 正式产品化第一轮 Local Product Alpha 的开发任务列表，包含 PRD、release gate、配置 profile、smoke、数据安全、观测和 daemon gate
- [2026-05-16-formal-product-readiness-12-workstreams.md](superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md)
  - 把当前 12 项待完善产品化工作拆成可执行 workstream，覆盖 Local Alpha gate、安装配置、runtime coverage、daemon、可观测性、decision protocol、evidence 语义、数据生命周期、provider、安全远程、发布工程和多层 memory 方向
- [../NOTICE](../NOTICE)
  - 项目版权、独立项目声明，以及 dashboard 生成图物料的归属说明

## 4. 建议先读

- [README.md](../README.md)
  - 先看项目一句话说明和多语言入口
- [CONTRIBUTING.md](../CONTRIBUTING.md)
  - 公开协作入口、验证要求与文档更新预期
- [project-status.md](project-status.md)
  - 当前实现边界、已实现 / 部分实现 / 未实现
- [progress-tracker.md](progress-tracker.md)
  - 适合直接用来跟踪后续开发任务推进
- [release-readiness.md](release-readiness.md)
  - 当前 demo 是否适合发布到 GitHub
- [release-gate.md](release-gate.md)
  - 发布前应执行的命令、artifact 验证，以及 sandbox-only 失败的记录方式
- [product/prd-local-alpha.md](product/prd-local-alpha.md)
  - 进入 Local Product Alpha 工作前先读，确认 scope、non-goals、remote write / multi-tenancy / self-governance 边界和 Alpha exit gate
- [product/release-gate-local-alpha.md](product/release-gate-local-alpha.md)
  - 判断 Local Alpha 是否可对外表述前阅读；它是产品 alpha gate，不替代 `release-gate.md` 的 MVP gate
- [product/support-bundle-local-alpha.md](product/support-bundle-local-alpha.md)
  - 生成或设计本地排障材料前阅读，避免泄露 API keys、raw provider payloads、provider URL secrets、raw TOML 或完整 SQLite 数据库
- [product/data-lifecycle.md](product/data-lifecycle.md)
  - 做备份、恢复、迁移、正式/测试/demo 数据隔离或支持包导出边界前阅读
- [product/release-engineering.md](product/release-engineering.md)
  - 做 release note、source-only tag、compatibility matrix、soak evidence 或 deprecation 规则前阅读
- [product/daemon-observe-only-gate.md](product/daemon-observe-only-gate.md)
  - 任何 daemon 代码或文档推进前阅读，确认 observe-only 阶段不调用 `run_reflection`、不写 identity / commitments、不开远程监听
- [product/remote-team-mode-boundary.md](product/remote-team-mode-boundary.md)
  - 任何远程 dashboard、admin/API、team mode 或多租户设计前阅读，确认 remote write/admin 仍被 gate 阻断
- [security/threat-model-local-and-remote.md](security/threat-model-local-and-remote.md)
  - 任何 remote/team 或安全边界设计前阅读，用于列出 assets、attacker capabilities、trust boundaries 和 mitigations
- [product/correlation-id-contract.md](product/correlation-id-contract.md)
  - 任何 MCP handler、dashboard projection、operation-log 或 support bundle 变更前阅读，确认 correlation id 只是 observability metadata
- [product/memory-layering-roadmap.md](product/memory-layering-roadmap.md)
  - 任何 richer episode、semantic memory、procedural memory、slow variables 或 self-model 设计前阅读，确认它们仍是后续方向
- [roadmap.md](roadmap.md)
  - 近期 / 中期 / 后期规划
- [2026-05-09-productization-roadmap.md](superpowers/plans/2026-05-09-productization-roadmap.md)
  - 正式产品化阶段规划
- [2026-05-09-local-product-alpha-development-tasks.md](superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md)
  - Local Product Alpha 开发执行清单
- [2026-05-16-formal-product-readiness-12-workstreams.md](superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md)
  - 当前 12 项待完善工作的执行规划，适合继续拆给 subagent 或作为后续本地提交批次的任务总表

## 5. 发布物料

- [github-publish-prep-2026-03-31.md](github-publish-prep-2026-03-31.md)
  - GitHub description、topics、首页文案和发布阻塞项
- [2026-03-31-initial-public-release.md](releases/2026-03-31-initial-public-release.md)
  - 首次公开发布的 release note 草稿
- [2026-04-20-self-revision-runtime-coverage-and-governance-hardening.md](releases/2026-04-20-self-revision-runtime-coverage-and-governance-hardening.md)
  - 本地 runtime coverage 与 evidence governance 收口更新记录
- [self-revision-demo-2026-04-24.md](reports/self-revision-demo-2026-04-24.md)
  - self-revision demo package 的 canonical report 口径

## 6. 接入与验证文档

- [local-mcp-integration-2026-03-26.md](local-mcp-integration-2026-03-26.md)
  - 如何把本项目接入本机 AI 客户端，以及当前 runtime hooks、`doctor` 输出和 self-revision MVP 运行边界
- [testing-guide-2026-03-24.md](testing-guide-2026-03-24.md)
  - 当前测试基线、推荐验证顺序、self-revision runtime coverage / diagnostics / evidence policy 定向回归和常见问题排查
- [provider-contract.md](provider-contract.md)
  - 新增 provider 的就绪清单，以及 `tests/provider_config.rs`、`tests/openai_compatible_model.rs`、`tests/mcp_stdio.rs` 的覆盖映射
- [release-gate.md](release-gate.md)
  - 发布 gate 的最小命令集、self-revision demo artifact 要求，以及 dashboard 边界检查
- [product/release-engineering.md](product/release-engineering.md)
  - 正式产品化发布工程规则；第一阶段使用 source-only artifact，不添加包装自动化声明
- [product/prd-local-alpha.md](product/prd-local-alpha.md)
  - Local Product Alpha PRD；后续产品化任务应先确认这里的范围、non-goals、exit gate 与验收命令
- [product/release-gate-local-alpha.md](product/release-gate-local-alpha.md)
  - Local Alpha release gate；产品化发布前从这里确认 product smoke、self-revision demo、dashboard local-only、daemon observe-only 和 remote write / multi-tenancy 文案边界
- [product/data-lifecycle.md](product/data-lifecycle.md)
  - Local Alpha 数据生命周期；正式/测试/demo SQLite 隔离、备份、恢复、导出和 migration 验证入口
- [product/support-bundle-local-alpha.md](product/support-bundle-local-alpha.md)
  - Local Alpha support bundle gate；当前已有 `./scripts/generate-support-bundle.sh <output_dir> [config_path]` 首版本地生成器，但仍不是远程上传或生产支持能力
- [product/remote-team-mode-boundary.md](product/remote-team-mode-boundary.md)
  - 远程 / 团队模式边界；remote read-only first，remote write/admin、team mode 和 multi-tenancy 仍需后续 gate
- [security/threat-model-local-and-remote.md](security/threat-model-local-and-remote.md)
  - 本地与远程 surface 威胁模型；remote/team 工作前的安全审查入口
- [product/daemon-observe-only-gate.md](product/daemon-observe-only-gate.md)
  - Local Alpha daemon observe-only gate；防止 daemon work 提前进入写能力或后台自治声明
- [product/correlation-id-contract.md](product/correlation-id-contract.md)
  - Local Alpha correlation id contract；用于把 MCP tool call、dashboard event 和 operation-log metadata 串起来
- [product/memory-layering-roadmap.md](product/memory-layering-roadmap.md)
  - 多层 memory 产品方向；当前不是 Local Alpha 已实现能力
- [self-revision-demo-guide-2026-04-24.md](self-revision-demo-guide-2026-04-24.md)
  - 如何运行 `./scripts/run-self-revision-demo.sh` 并复核 8 个 demo artifact
- [examples/codex-mcp-config.toml](../examples/codex-mcp-config.toml)
  - Codex 本机 MCP 配置样例
- [examples/agent-llm-mm.demo.example.toml](../examples/agent-llm-mm.demo.example.toml)
  - self-revision demo runner 使用的本地 deterministic provider 配置样例

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
- [2026-04-24-self-revision-demo-package.md](superpowers/plans/2026-04-24-self-revision-demo-package.md)
  - self-revision demo package 的分步实现计划
- [2026-03-28-openai-compatible-provider-claude-code.md](superpowers/plans/2026-03-28-openai-compatible-provider-claude-code.md)
  - 较早的 provider 实现计划草稿，已被后续配置文件方案替代
- [2026-03-28-openai-compatible-provider-claude-code-design.md](superpowers/specs/2026-03-28-openai-compatible-provider-claude-code-design.md)
  - 较早的 provider 设计稿，保留用于追溯，不代表当前最终实现
- [2026-04-19-self-agent-memory-self-revision-mvp-design.md](superpowers/specs/2026-04-19-self-agent-memory-self-revision-mvp-design.md)
  - 基于原始逐轮日志与再次确认问答整合出的 self-revision MVP 设计初稿
- [2026-05-09-productization-roadmap.md](superpowers/plans/2026-05-09-productization-roadmap.md)
  - MVP 之后的正式产品化路线图，当前下一阶段开发应优先参考
- [2026-05-09-local-product-alpha-development-tasks.md](superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md)
  - 正式产品化第一轮任务列表，适合拆给 subagent 或按 milestone 执行
- [2026-05-16-formal-product-readiness-12-workstreams.md](superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md)
  - 12 项后续完善工作的正式产品化执行规划，连接 Local Alpha gate、Beta、remote/team 和 GA readiness 前置工作
- [product/data-lifecycle.md](product/data-lifecycle.md)
  - Workstream 8 的数据生命周期落地文档
- [product/release-engineering.md](product/release-engineering.md)
  - Workstream 11 的 release engineering 落地文档
- [product/remote-team-mode-boundary.md](product/remote-team-mode-boundary.md)
  - Workstream 10 的远程 / 团队模式边界文档
- [product/memory-layering-roadmap.md](product/memory-layering-roadmap.md)
  - Workstream 12 的多层 memory 产品方向文档

## 8. 阅读建议

- 如果你是第一次看这个仓库：先读 README，再读 `project-status.md`、`release-readiness.md` 与 `release-gate.md`
- 如果你想确认 automatic self-revision 到底实现到哪里：直接读 `project-status.md`，再读 `roadmap.md`
- 如果你想快速看见 automatic self-revision 的证据链：读 `self-revision-demo-guide-2026-04-24.md`，再运行 `./scripts/run-self-revision-demo.sh`
- 如果你想确认 runtime hooks、diagnostics 和 durable write path：读 `project-status.md`，再读 `local-mcp-integration-2026-03-26.md`
- 如果你要接入新 provider：先读 `provider-contract.md`，再按 `testing-guide-2026-03-24.md` 的 provider 验证顺序执行
- 如果你想接入或开发：先进入对应平台文档，再读 `local-mcp-integration-2026-03-26.md`、`testing-guide-2026-03-24.md` 与 `release-gate.md`
- 如果你要把项目推进成正式产品：先读 `superpowers/plans/2026-05-09-productization-roadmap.md`，再读 `product/prd-local-alpha.md` 确认 Local Product Alpha 的 scope、non-goals 和 exit gate，然后读 `product/release-gate-local-alpha.md` 确认产品 alpha gate；第一轮按 `superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md` 执行，后续 12 项完善工作按 `superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md` 拆分推进
- 如果你想追溯设计来源：最后读原始讨论资料和历史快照
