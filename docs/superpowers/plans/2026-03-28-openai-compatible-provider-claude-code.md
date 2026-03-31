# OpenAI Compatible Provider And Claude Code Integration Implementation Plan

> Historical note: this plan is retained as an earlier implementation plan draft. It assumed environment-variable driven provider loading and `Claude Code` specific registration. The current mainline implementation instead uses config-file driven provider loading; see [2026-03-31-openai-compatible-provider-config-file.md](2026-03-31-openai-compatible-provider-config-file.md), [project-status.md](../../project-status.md), and [README.md](../../../README.md).

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `agent_llm_mm` 增加可运行的 OpenAI 兼容 provider、把当前 MCP `stdio` 服务注册到本机 `Claude Code`、并补齐红/绿测试与说明文档。

**Architecture:** 保持 `ModelPort` 与 `decide_with_snapshot` 业务层不变，在 adapter/config/runtime 层增加 `openai-compatible` 实现与 provider 选择逻辑。MCP tool 面不扩展，只复用现有 `serve`/`doctor` 入口，并通过 `claude mcp add` 把服务注册到本机 `Claude Code`。

**Tech Stack:** Rust 2024, Tokio, RMCP stdio, SQLite, PowerShell 7, Claude Code CLI, OpenAI-compatible `chat/completions`, 本地 stub HTTP 服务测试

---

### Task 1: 扩展配置模型并锁定失败边界

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/support/config.rs`
- Modify: `src/support/doctor.rs`
- Modify: `tests/bootstrap.rs`
- Create: `tests/provider_config.rs`

- [ ] **Step 1: 写失败测试，固定 provider 配置契约**

```rust
#[test]
fn default_config_uses_openai_compatible_provider() {
    let config = AppConfig::default();
    assert_eq!(config.model_provider, ModelProviderKind::OpenAiCompatible);
}

#[tokio::test]
async fn doctor_fails_when_openai_provider_is_missing_api_key() {
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url: sqlite_url_for_test(),
        model_provider: ModelProviderKind::OpenAiCompatible,
        model_config: ModelConfig::openai_compatible(
            "https://example.invalid".to_string(),
            None,
            Some("gpt-4o-mini".to_string()),
            30_000,
        ),
    };

    let error = run_doctor(config).await.expect_err("doctor should fail");
    assert!(error.to_string().contains("AGENT_LLM_MM_OPENAI_API_KEY"));
}
```

- [ ] **Step 2: 运行测试，确认当前实现确实不支持这些配置**

Run:

```powershell
cargo test provider_config -- --nocapture
cargo test bootstrap -- --nocapture
```

Expected:

- 新增测试编译失败，或因 `AppConfig`/`DoctorReport` 字段缺失而失败
- 证明配置模型尚未扩展

- [ ] **Step 3: 实现最小配置扩展**

```rust
pub const MODEL_PROVIDER_ENV_VAR: &str = "AGENT_LLM_MM_MODEL_PROVIDER";
pub const OPENAI_BASE_URL_ENV_VAR: &str = "AGENT_LLM_MM_OPENAI_BASE_URL";
pub const OPENAI_API_KEY_ENV_VAR: &str = "AGENT_LLM_MM_OPENAI_API_KEY";
pub const OPENAI_MODEL_ENV_VAR: &str = "AGENT_LLM_MM_OPENAI_MODEL";
pub const OPENAI_TIMEOUT_MS_ENV_VAR: &str = "AGENT_LLM_MM_OPENAI_TIMEOUT_MS";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ModelProviderKind {
    Mock,
    OpenAiCompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiCompatibleConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelConfig {
    Mock,
    OpenAiCompatible(OpenAiCompatibleConfig),
}
```

Implementation notes:

- 默认 `model_provider` 设为 `OpenAiCompatible`
- `doctor` 在 provider=`OpenAiCompatible` 时做显式配置校验
- `DoctorReport` 新增 `provider`、`model`、`base_url`，但不输出 API key
- `Cargo.toml` 同步加入 HTTP 客户端依赖，为后续任务预留

- [ ] **Step 4: 运行测试，确认配置与 doctor 失败边界通过**

Run:

```powershell
cargo test provider_config -- --nocapture
cargo test bootstrap -- --nocapture
```

Expected:

- 新增配置测试通过
- `bootstrap` 里与 `doctor` 相关的测试通过
- `default_config_uses_stdio_transport` 等旧测试仍通过

- [ ] **Step 5: 提交这一组改动**

```powershell
git add Cargo.toml src/support/config.rs src/support/doctor.rs tests/bootstrap.rs tests/provider_config.rs
git commit -m "feat: add provider config and doctor validation"
```

### Task 2: 增加 OpenAI 兼容 provider 适配器

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/adapters/model/mod.rs`
- Create: `src/adapters/model/openai_compatible.rs`
- Create: `tests/openai_compatible_model.rs`

- [ ] **Step 1: 写失败测试，固定 provider 请求/响应契约**

```rust
#[tokio::test]
async fn openai_compatible_model_parses_first_assistant_message_into_action() {
    let stub = spawn_stub_server(json!({
        "id": "chatcmpl-test",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "summarize_memory_state"
            }
        }]
    })).await;

    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })?;

    let decision = model.decide(sample_request()).await?;
    assert_eq!(decision.action, "summarize_memory_state");
}

#[tokio::test]
async fn openai_compatible_model_rejects_empty_action() {
    let stub = spawn_stub_server(json!({
        "choices": [{
            "message": {
                "role": "assistant",
                "content": "   "
            }
        }]
    })).await;

    let model = model_for_stub(&stub)?;
    let error = model.decide(sample_request()).await.expect_err("empty content should fail");
    assert!(error.to_string().contains("empty model action"));
}
```

- [ ] **Step 2: 运行测试，确认当前适配器尚不存在**

Run:

```powershell
cargo test openai_compatible_model -- --nocapture
```

Expected:

- 编译失败，因为 `OpenAiCompatibleModel` 文件与类型尚未创建

- [ ] **Step 3: 实现最小 provider 适配器**

```rust
pub struct OpenAiCompatibleModel {
    client: reqwest::Client,
    config: OpenAiCompatibleConfig,
}

#[async_trait]
impl ModelPort for OpenAiCompatibleModel {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url.trim_end_matches('/')))
            .bearer_auth(&self.config.api_key)
            .json(&build_payload(&self.config.model, request))
            .send()
            .await
            .map_err(map_network_error)?;

        let response = response.error_for_status().map_err(map_http_error)?;
        let body: ChatCompletionResponse = response.json().await.map_err(map_protocol_error)?;
        let action = extract_action(body)?;
        Ok(ModelDecision::new(action))
    }
}
```

Implementation notes:

- 优先用 `reqwest` + `rustls-tls`
- 请求只走最基础的 `chat/completions`
- `messages` 里把 `task`、`action`、`snapshot` 序列化给模型
- 解析只取第一条 assistant 文本，trim 后判空
- 错误统一转为 `AppError::Message`

- [ ] **Step 4: 运行 provider 测试，确认契约通过**

Run:

```powershell
cargo test openai_compatible_model -- --nocapture
```

Expected:

- stub server 驱动的绿测通过
- 空响应、非 2xx 等红测通过

- [ ] **Step 5: 提交这一组改动**

```powershell
git add Cargo.toml src/adapters/model/mod.rs src/adapters/model/openai_compatible.rs tests/openai_compatible_model.rs
git commit -m "feat: add openai compatible model adapter"
```

### Task 3: 把运行时从硬编码 MockModel 切到可配置 provider

**Files:**
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `tests/decision_flow.rs`
- Modify: `tests/application_use_cases.rs`

- [ ] **Step 1: 写失败测试，固定 gate 与 provider 选择行为**

```rust
#[tokio::test]
async fn runtime_uses_openai_provider_when_config_requests_it() {
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url: sqlite_url_for_test(),
        model_provider: ModelProviderKind::OpenAiCompatible,
        model_config: ModelConfig::OpenAiCompatible(stub_openai_config()),
    };

    let runtime = build_runtime_for_test(config).await?;
    let decision = runtime.decide(sample_request()).await?;
    assert_eq!(decision.action, "summarize_memory_state");
}
```

And keep existing gate test shape:

```rust
assert_eq!(deps.model_call_count(), 0);
```

- [ ] **Step 2: 运行测试，确认当前 Runtime 仍然硬编码 `MockModel`**

Run:

```powershell
cargo test decision_flow -- --nocapture
cargo test application_use_cases decide_with_snapshot -- --nocapture
```

Expected:

- 新增 runtime/provider 相关测试失败
- 现有 gate 测试仍通过，证明改动点集中在 runtime 绑定

- [ ] **Step 3: 实现最小运行时切换**

```rust
enum RuntimeModel {
    Mock(MockModel),
    OpenAiCompatible(OpenAiCompatibleModel),
}

#[async_trait]
impl ModelPort for RuntimeModel {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        match self {
            Self::Mock(model) => model.decide(request).await,
            Self::OpenAiCompatible(model) => model.decide(request).await,
        }
    }
}
```

Implementation notes:

- `Runtime::bootstrap` 根据 `AppConfig` 构造 `RuntimeModel`
- `validate_stdio_runtime` 复用同一 bootstrap 路径，不允许 `doctor` 与 `serve` 分叉配置语义
- 不修改 `decide_with_snapshot` 的业务逻辑

- [ ] **Step 4: 运行运行时与应用层测试**

Run:

```powershell
cargo test decision_flow -- --nocapture
cargo test application_use_cases decide_with_snapshot -- --nocapture
```

Expected:

- gate 阻断时仍不会调用 provider
- 允许路径下会调用配置好的 provider 或替身

- [ ] **Step 5: 提交这一组改动**

```powershell
git add src/interfaces/mcp/server.rs tests/decision_flow.rs tests/application_use_cases.rs
git commit -m "feat: wire runtime model provider selection"
```

### Task 4: 保持 MCP stdio 闭环，并新增 provider 路径的 E2E 测试

**Files:**
- Modify: `tests/mcp_stdio.rs`
- Modify: `tests/bootstrap.rs`
- Create: `tests/support/openai_stub.rs`

- [ ] **Step 1: 写失败测试，固定 `stdio + openai-compatible` 绿测**

```rust
#[tokio::test]
async fn decide_with_snapshot_over_stdio_uses_openai_compatible_provider() {
    let stub = openai_stub::spawn_ok("summarize_memory_state").await;
    let mut client = spawn_stdio_client_with_env(&[
        ("AGENT_LLM_MM_MODEL_PROVIDER", "openai-compatible"),
        ("AGENT_LLM_MM_OPENAI_BASE_URL", &stub.base_url()),
        ("AGENT_LLM_MM_OPENAI_API_KEY", "test-key"),
        ("AGENT_LLM_MM_OPENAI_MODEL", "gpt-4o-mini"),
    ]).await?;

    let snapshot = sample_snapshot_with_no_blocking_commitments();
    let response = client.call_tool("decide_with_snapshot", json!({
        "task": "summarize current memory",
        "action": "read_identity_core",
        "snapshot": snapshot,
    })).await?;

    assert_eq!(
        response["result"]["structuredContent"]["decision"]["action"],
        "summarize_memory_state"
    );
}
```

- [ ] **Step 2: 运行测试，确认当前 stdio 客户端还无法驱动 provider 配置**

Run:

```powershell
cargo test mcp_stdio -- --nocapture
```

Expected:

- 新增测试失败，原因通常是没有 provider 环境变量通路或没有 openai stub 适配器

- [ ] **Step 3: 实现最小 E2E 支撑**

Implementation notes:

- 给 `tests/mcp_stdio.rs` 的 `spawn_stdio_client` 增加可选环境变量注入
- 把本地 stub server 抽到 `tests/support/openai_stub.rs`
- 保留现有 tool list、snapshot、reflection 测试不动
- 只为 provider 路径补新增绿测和对应红测

- [ ] **Step 4: 运行 MCP 与启动链路测试**

Run:

```powershell
cargo test mcp_stdio -- --nocapture
cargo test bootstrap -- --nocapture
```

Expected:

- 原有 4 个 tool 仍然存在
- `openai-compatible` provider 路径的 stdio E2E 通过
- 启动链路依然通过 `doctor` / `serve`

- [ ] **Step 5: 提交这一组改动**

```powershell
git add tests/mcp_stdio.rs tests/bootstrap.rs tests/support/openai_stub.rs
git commit -m "test: cover openai compatible provider over mcp stdio"
```

### Task 5: 更新文档并注册 Claude Code MCP server

**Files:**
- Modify: `README.md`
- Modify: `docs/local-mcp-integration-2026-03-26.md`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: 写文档差异清单**

Expected doc changes:

- `README.md`：去掉“运行时仍是 mock”的表述，改为“默认 OpenAI-compatible，mock 仅用于测试”
- `docs/local-mcp-integration-2026-03-26.md`：补充 provider 环境变量与 `claude mcp add` 命令
- `docs/testing-guide-2026-03-24.md`：新增红/绿测试用例、命令、预期结果

- [ ] **Step 2: 更新文档**

Key command to document:

```powershell
claude mcp add --scope user `
  -e AGENT_LLM_MM_DATABASE_URL=sqlite:///D:/back/agent-llm-mm-claude.sqlite `
  -e AGENT_LLM_MM_MODEL_PROVIDER=openai-compatible `
  -e AGENT_LLM_MM_OPENAI_BASE_URL=https://<your-openai-compatible-host> `
  -e AGENT_LLM_MM_OPENAI_API_KEY=<redacted> `
  -e AGENT_LLM_MM_OPENAI_MODEL=gpt-4o-mini `
  agent-llm-mm `
  -- "C:\Program Files\PowerShell\7\pwsh.exe" -NoLogo -NoProfile -File "D:\Code\agent_llm_mm\scripts\agent-llm-mm.ps1" serve
```

Doc notes:

- 明确当前服务仍是 MCP `stdio`
- 红测写清配置缺失、provider 非法响应、gate 阻断
- 绿测写清 doctor 通过、stdio tool list 通过、Claude Code 已注册

- [ ] **Step 3: 先跑只读验证命令**

Run:

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
claude mcp list
```

Expected:

- `doctor` 在 provider 配置齐全时返回 `status = ok`
- `claude mcp list` 可正常工作，为后续注册做准备

- [ ] **Step 4: 注册 Claude Code MCP server 并验证**

Run:

```powershell
claude mcp add --scope user `
  -e AGENT_LLM_MM_DATABASE_URL=sqlite:///D:/back/agent-llm-mm-claude.sqlite `
  -e AGENT_LLM_MM_MODEL_PROVIDER=openai-compatible `
  -e AGENT_LLM_MM_OPENAI_BASE_URL=https://<your-openai-compatible-host> `
  -e AGENT_LLM_MM_OPENAI_API_KEY=<real-key> `
  -e AGENT_LLM_MM_OPENAI_MODEL=<real-model> `
  agent-llm-mm `
  -- "C:\Program Files\PowerShell\7\pwsh.exe" -NoLogo -NoProfile -File "D:\Code\agent_llm_mm\scripts\agent-llm-mm.ps1" serve

claude mcp get agent-llm-mm
claude mcp list
```

Expected:

- `agent-llm-mm` 出现在 Claude Code MCP 列表里
- `claude mcp get` 能看到 stdio 命令和环境变量键

- [ ] **Step 5: 提交文档改动**

```powershell
git add README.md docs/local-mcp-integration-2026-03-26.md docs/testing-guide-2026-03-24.md
git commit -m "docs: add claude code integration and red green tests"
```

### Task 6: 运行最终验证并整理交付说明

**Files:**
- Modify: `README.md`
- Modify: `docs/testing-guide-2026-03-24.md`

- [ ] **Step 1: 运行格式化与测试回归**

Run:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Expected:

- 格式检查通过
- clippy 无警告
- 全量测试通过

- [ ] **Step 2: 运行接入 smoke test**

Run:

```powershell
$env:AGENT_LLM_MM_DATABASE_URL = 'sqlite:///D:/back/agent-llm-mm-claude.sqlite'
$env:AGENT_LLM_MM_MODEL_PROVIDER = 'openai-compatible'
$env:AGENT_LLM_MM_OPENAI_BASE_URL = 'https://<your-openai-compatible-host>'
$env:AGENT_LLM_MM_OPENAI_API_KEY = '<real-key>'
$env:AGENT_LLM_MM_OPENAI_MODEL = '<real-model>'
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

Expected:

- 返回 JSON
- `status = "ok"`
- `provider = "openai-compatible"`

- [ ] **Step 3: 记录红/绿测试结果**

Red cases to report:

- 缺少 `AGENT_LLM_MM_OPENAI_API_KEY`
- 缺少 `AGENT_LLM_MM_OPENAI_BASE_URL`
- provider 返回空动作
- provider 返回非 2xx

Green cases to report:

- stub provider 下 `cargo test` 通过
- `stdio` 下 `decide_with_snapshot` 能拿到真实 provider 决策
- `claude mcp get agent-llm-mm` 可见注册结果

- [ ] **Step 4: 最终提交**

```powershell
git add Cargo.toml src tests README.md docs
git commit -m "feat: integrate openai compatible provider for claude code mcp"
```
