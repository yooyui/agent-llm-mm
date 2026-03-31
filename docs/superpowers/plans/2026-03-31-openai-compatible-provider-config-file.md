# OpenAI Compatible Provider With Config File Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `agent_llm_mm` 增加基于配置文件加载的 `openai-compatible` provider，并保留后续扩展其他 provider 的边界，同时补齐发布前隐私清理与验证。

**Architecture:** 保持 `ModelPort` 与应用层不变，在 `support/config` 中增加 TOML 配置加载与 provider 枚举，在 `adapters/model` 中增加 `openai_compatible` 适配器，并由 `interfaces/mcp/server::Runtime` 通过 provider 工厂构造实际模型。真实配置默认来自仓库根目录 `agent-llm-mm.local.toml`，可由 `AGENT_LLM_MM_CONFIG` 覆盖，示例配置放在 `examples/`，真实密钥不进入仓库。

**Tech Stack:** Rust 2024, Tokio, RMCP stdio, SQLite, TOML/Serde, Reqwest, PowerShell 7

---

### Task 1: 扩展配置模型与配置文件加载

**Files:**
- Modify: `src/support/config.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Test: `tests/bootstrap.rs`
- Create: `tests/provider_config.rs`

- [ ] **Step 1: 写失败测试，固定配置文件加载契约**
- [ ] **Step 2: 运行配置相关测试，确认红测成立**
- [ ] **Step 3: 实现 TOML 配置加载、默认路径与 `AGENT_LLM_MM_CONFIG` 覆盖**
- [ ] **Step 4: 运行配置相关测试，确认绿测通过**

### Task 2: 引入 provider 枚举与 openai-compatible 适配器

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/adapters/model/mod.rs`
- Create: `src/adapters/model/openai_compatible.rs`
- Test: `tests/openai_compatible_model.rs`

- [ ] **Step 1: 写失败测试，固定 provider 请求/响应与错误映射契约**
- [ ] **Step 2: 运行 provider 单测，确认红测成立**
- [ ] **Step 3: 实现 `openai_compatible` 适配器，保留 `mock` 作为显式 provider**
- [ ] **Step 4: 运行 provider 单测，确认绿测通过**

### Task 3: 改造 runtime，按配置构造 provider

**Files:**
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/support/doctor.rs`
- Test: `tests/bootstrap.rs`
- Test: `tests/mcp_stdio.rs`

- [ ] **Step 1: 写失败测试，固定 runtime/doctor 的 provider 选择行为**
- [ ] **Step 2: 运行相关测试，确认红测成立**
- [ ] **Step 3: 实现 provider 工厂与 doctor 安全输出**
- [ ] **Step 4: 运行相关测试，确认绿测通过**

### Task 4: 补配置样例、脚本与文档

**Files:**
- Create: `examples/agent-llm-mm.example.toml`
- Modify: `scripts/agent-llm-mm.ps1`
- Modify: `README.md`
- Modify: `docs/local-mcp-integration-2026-03-26.md`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: 写文档和样例期望，明确配置文件路径与 provider 选择方式**
- [ ] **Step 2: 实现脚本透传与文档更新**
- [ ] **Step 3: 自检文档命令、路径与示例一致性**

### Task 5: 发布前隐私清理与最终验证

**Files:**
- Modify: `.gitignore`
- Review: `README.md`
- Review: `examples/agent-llm-mm.example.toml`
- Review: `docs/local-mcp-integration-2026-03-26.md`
- Review: `docs/testing-guide-2026-03-24.md`
- Review: `tests/*.rs`

- [ ] **Step 1: 确保 `agent-llm-mm.local.toml` 被忽略且示例配置不含真实密钥**
- [ ] **Step 2: 扫描 docs/examples/tests/scripts，确认无真实 API key、token、私有 endpoint**
- [ ] **Step 3: 运行最终验证：`cargo test`、`pwsh -File .\\scripts\\agent-llm-mm.ps1 doctor`（mock 与配置文件路径）**
- [ ] **Step 4: 总结剩余风险，只保留真实未完成项**
