# OpenAI Compatible Provider And Claude Code Integration Design

> Historical note: this design draft is kept for traceability. It reflects an earlier direction that relied on environment variables and `Claude Code` specific integration steps. The implemented mainline approach was later adjusted to config-file driven provider loading; current behavior is documented in [README.md](../../../README.md), [project-status.md](../../project-status.md), and [2026-03-31-openai-compatible-provider-config-file.md](../plans/2026-03-31-openai-compatible-provider-config-file.md).

**日期：** 2026-03-28
**状态：** Draft approved in conversation, pending file review
**语言：** 简体中文

---

## 1. 目标

在不改变当前 MCP `stdio` 服务形态的前提下，为 `agent_llm_mm` 增加可运行的 OpenAI 兼容模型 provider，并把该服务注册到本机 `Claude Code` 中，满足后续人工联调与回归测试。

本轮重点不是扩展新的 MCP tool，也不是重做领域层，而是把当前“模型仍是 mock”的闭环推进为“运行时可真实调第三方模型”的闭环，并给出清晰的测试说明。

目标分为三部分：

- 运行时目标：当前所有涉及模型调用的运行时路径切换为可配置的 OpenAI 兼容 provider
- 接入目标：把本仓库 MCP 服务注册到本机 `Claude Code`，使其可作为 MCP client 直接调用
- 验证目标：补齐红/绿测试用例与接入说明，区分配置错误、provider 错误、MCP 错误三类失败

---

## 2. 本轮范围

### 2.1 In Scope

- 为模型层增加 OpenAI 兼容 provider 适配器
- 保留 `ModelPort` 抽象，不改动应用层接口
- 通过配置在运行时选择 provider
- 扩展 `doctor`，让它能检查 provider 配置是否完整
- 更新 README 与本机接入文档
- 将本服务注册到本机 `Claude Code`
- 增加围绕 provider 配置、provider 响应解析、MCP 接入的红/绿测试

### 2.2 Clarification

当前代码基里，真实触发模型调用的运行时路径只有 `decide_with_snapshot`。

因此，“涉及到的都切换成 OpenAI 兼容 provider”在本轮里的具体含义是：

- 当前所有真实模型调用都从 `MockModel` 迁移到可配置 provider
- `ingest_interaction`、`build_self_snapshot`、`run_reflection` 继续保持本地逻辑，不人为引入新的 provider 耦合
- `mock` 仍可保留为测试替身，但不再是默认联调路径

### 2.3 Out of Scope

- 新增 HTTP transport
- 新增 Web UI 或管理后台
- 引入多 provider 路由、负载均衡、自动 fallback
- 接入完整对话式聊天能力
- 让所有 MCP tool 都直接调用远程模型
- 对领域模型做与 provider 无关的大规模重构

---

## 3. 关键设计决策

### 3.1 保持 MCP 形态不变

当前实现已经是 `stdio` MCP 服务，不需要再“改造成 MCP”。

因此本轮只做两件事：

- 把模型调用从 mock 变成真实 provider
- 把这个现有 MCP 服务挂到 `Claude Code`

这样 blast radius 最小，也符合用户“后续自己测试”的目标。

### 3.2 保持 `ModelPort` 不变

`application::decide_with_snapshot` 现在只依赖 `ModelPort`，这是一个非常干净的边界，应继续保留。

原因：

- 应用层不应该感知第三方 API 协议
- 测试替身和真实 provider 可以共用同一端口
- 后续若切别家 OpenAI 兼容服务，不需要再次穿透业务层

### 3.3 使用 OpenAI 兼容 `chat/completions`

首版采用最广泛兼容的 `chat/completions` 形式，而不是直接依赖某一家 SDK 或更激进的新接口。

原因：

- 第三方“OpenAI 兼容”服务通常优先支持 `chat/completions`
- 用通用 HTTP 客户端实现，比绑定单一官方 SDK 更稳妥
- 更利于对接不同 `base_url`

### 3.4 运行时默认切换到真实 provider，测试保留 mock

本轮不删除 `MockModel`，但要改变其角色：

- 运行时联调和 Claude Code 接入默认使用 `openai-compatible`
- 单元测试和部分离线验证仍可使用 `mock`

原因：

- 如果彻底删掉 mock，本地测试会被外部网络与密钥绑定，开发体验会明显变差
- 但如果继续默认走 mock，就达不到“接入到现在的 GPT 模型”的目标

### 3.5 用环境变量驱动配置

provider 配置继续沿用当前仓库的简单配置风格，通过环境变量驱动，而不是第一版引入专门配置文件。

建议新增：

- `AGENT_LLM_MM_MODEL_PROVIDER`
- `AGENT_LLM_MM_OPENAI_BASE_URL`
- `AGENT_LLM_MM_OPENAI_API_KEY`
- `AGENT_LLM_MM_OPENAI_MODEL`

必要时可补充：

- `AGENT_LLM_MM_OPENAI_TIMEOUT_MS`

### 3.6 优先使用 Claude CLI 官方命令注册 MCP

不直接手改 `Claude Code` 内部状态文件，优先使用：

```powershell
claude mcp add
```

原因：

- 减少格式漂移和内部结构变化带来的风险
- 更符合该客户端的预期管理方式
- 后续移除或调整时也可走同一入口

---

## 4. 总体架构

本轮架构仍保持现有四层，只在模型边界增加一个真实 provider 适配器。

### 4.1 Domain

不改动领域模型和规则。

保留：

- `SelfSnapshot`
- `commitment_gate`
- 现有 claim / event / reflection 语义

### 4.2 Application

`decide_with_snapshot` 继续执行现有流程：

1. 先用 `commitment_gate` 做规则阻断
2. 若阻断，则不调用 provider
3. 若放行，则构造 `ModelDecisionRequest`
4. 调用 `ModelPort`
5. 返回 `ModelDecision`

这一层不引入任何第三方 API 细节。

### 4.3 Adapters

模型适配器拆分为：

- `mock.rs`
- `openai_compatible.rs`

其中：

- `mock.rs` 仅用于离线测试与最小替身
- `openai_compatible.rs` 负责 HTTP 请求、响应解析与错误映射

### 4.4 Interfaces / Runtime

`interfaces::mcp::server::Runtime` 负责：

- 初始化 SQLite store
- 根据配置构造模型 provider
- 将 provider 暴露为 `ModelPort`

MCP tool 名称与参数保持不变：

- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`

---

## 5. 配置设计

### 5.1 AppConfig 扩展

当前 `AppConfig` 只有：

- `transport`
- `database_url`

本轮扩展为：

- `transport`
- `database_url`
- `model_provider`
- `model_config`

其中 `model_provider` 至少支持：

- `mock`
- `openai-compatible`

`model_config` 以枚举或结构体承载：

- `base_url`
- `api_key`
- `model`
- `timeout_ms`

### 5.2 默认行为

默认行为建议如下：

- 若未显式设置 `AGENT_LLM_MM_MODEL_PROVIDER`，默认使用 `openai-compatible`
- 若 provider 为 `openai-compatible`，缺少必要配置时 `doctor` 和 `serve` 都应快速失败
- 若测试需要离线运行，可显式设置 `AGENT_LLM_MM_MODEL_PROVIDER=mock`

这样可以避免用户以为已经接入真实模型，实际上仍在走 mock。

### 5.3 Doctor 输出

`DoctorReport` 应扩展出可观测但不泄露敏感信息的字段，例如：

- `provider`
- `model`
- `base_url`
- `status`

不得输出：

- 明文 API Key

---

## 6. OpenAI 兼容 provider 契约

### 6.1 请求协议

首版通过通用 HTTP POST 调用：

```text
POST {base_url}/chat/completions
```

请求中至少包含：

- `model`
- `messages`
- 合理的 `temperature`

`messages` 由 `ModelDecisionRequest` 映射得到：

- system：约束模型输出格式
- user：提供 `task`、`action`、`snapshot`

### 6.2 输出契约

首版要求 provider 返回一个可解析的“动作字符串”，并映射到：

```rust
ModelDecision {
    action: String
}
```

为了兼容更多 OpenAI 兼容服务，首版不强依赖复杂 JSON schema；更稳妥的策略是：

- system prompt 要求模型仅返回一个动作名
- 解析时读取首个 assistant 文本内容
- 去除首尾空白
- 结果为空时视为 provider 非法响应

这比强依赖 JSON mode 更容易跨厂商落地。

### 6.3 错误映射

provider 错误统一映射为应用层可诊断错误，不把底层 HTTP 细节直接泄露到 MCP 协议面。

建议区分：

- 配置错误：缺少 `base_url / api_key / model`
- 网络错误：请求失败、超时、连接断开
- 协议错误：非 2xx、响应结构异常
- 语义错误：模型返回空动作

---

## 7. Claude Code 接入设计

### 7.1 接入方式

接入方式不是改造服务，而是把当前服务注册为 `Claude Code` 的一个 MCP server。

建议使用 user scope：

```powershell
claude mcp add --scope user ...
```

### 7.2 注册形态

注册到 `Claude Code` 的命令应指向当前仓库脚本入口：

- `pwsh.exe`
- `-NoLogo`
- `-NoProfile`
- `-File`
- `D:\Code\agent_llm_mm\scripts\agent-llm-mm.ps1`
- `serve`

同时注入环境变量：

- `AGENT_LLM_MM_DATABASE_URL`
- `AGENT_LLM_MM_MODEL_PROVIDER=openai-compatible`
- `AGENT_LLM_MM_OPENAI_BASE_URL`
- `AGENT_LLM_MM_OPENAI_API_KEY`
- `AGENT_LLM_MM_OPENAI_MODEL`

### 7.3 验证思路

注册成功后，需要至少验证三层：

1. `claude mcp get <name>` 能读到该 server 配置
2. `doctor` 在同样环境变量下返回 `status = ok`
3. 通过 Claude Code 或本地 MCP stdio 测试客户端调用 `decide_with_snapshot` 时，能得到真实 provider 返回结果

---

## 8. 测试设计

### 8.1 测试目标

本轮测试不追求覆盖真实第三方服务的稳定性，而是验证：

- 配置正确时能走通
- 配置错误时能快速失败
- 规则阻断时不会误打 provider
- MCP 链路在 provider 切换后仍然稳定

### 8.2 红测

红测用于证明系统会在错误条件下正确失败。

建议至少覆盖：

1. `openai-compatible` 缺少 `base_url`
   - 预期：`doctor` 失败
2. `openai-compatible` 缺少 `api_key`
   - 预期：`doctor` 失败
3. `openai-compatible` 缺少 `model`
   - 预期：`doctor` 失败
4. commitment gate 阻断时
   - 预期：`decide_with_snapshot` 返回 `blocked = true`
   - 预期：provider 调用次数为 0
5. provider 返回空文本或不可解析内容
   - 预期：`decide_with_snapshot` 返回错误
6. provider 返回非 2xx
   - 预期：错误被映射为可诊断应用错误，而不是静默吞掉

### 8.3 绿测

绿测用于证明系统在正确条件下能正常工作。

建议至少覆盖：

1. 本地 stub OpenAI 兼容服务返回有效动作
   - 预期：`decide_with_snapshot` 返回 `blocked = false`
   - 预期：`decision.action` 等于 stub 返回动作
2. `doctor` 在完整配置下返回 `status = ok`
3. MCP `stdio` 服务在 `openai-compatible` 配置下仍暴露原有 4 个工具
4. MCP `stdio` 客户端调用 `decide_with_snapshot` 能拿到 provider 决策
5. `claude mcp add` 注册后，`claude mcp get` 能查到配置

### 8.4 测试策略

测试分层如下：

- 单元测试：配置解析、provider 响应解析、错误映射
- 应用层测试：gate 与 provider 调用关系
- MCP E2E：`stdio` 初始化、tool list、tool call
- 手工联调：Claude Code 内实际注册与 smoke test

真实第三方网络请求不应成为默认自动化测试前提。自动化测试应优先使用本地 stub server，确保可重复。

---

## 9. 文档设计

本轮文档至少更新以下内容：

- `README.md`
  - 当前 provider 状态从“mock only”改为“默认 OpenAI compatible，可回退 mock 用于测试”
- 本机接入文档
  - 补充 Claude Code 注册命令
  - 补充 provider 环境变量说明
- 测试说明
  - 增加红/绿用例、命令、预期结果

文档需要明确强调：

- 当前服务仍是 MCP `stdio`
- 不是 HTTP API 服务
- 真实 provider 只覆盖当前已有模型调用路径
- `mock` 是测试后备，不是正式接入目标

---

## 10. 风险与缓解

### 10.1 OpenAI 兼容性并不完全一致

风险：

- 不同第三方兼容服务在字段、错误码、返回格式上可能存在偏差

缓解：

- 首版只依赖最基础的 `chat/completions`
- 输出契约保持极简，只取 assistant 文本

### 10.2 Claude Code 本机配置漂移

风险：

- 直接手改内部配置文件容易因为结构变化失效

缓解：

- 优先使用 `claude mcp add` 管理 MCP server

### 10.3 自动化测试被真实网络拖垮

风险：

- 把 CI 或本地快速回归绑定到真实第三方接口会导致高脆弱性

缓解：

- 自动化测试默认用 stub server
- 真实 provider 联调只作为手工 smoke test

### 10.4 用户误以为所有工具都已模型化

风险：

- 看到“接入 GPT”后，容易误解所有 MCP tool 都在调用模型

缓解：

- 在 README 和测试说明里明确写出：当前只有 `decide_with_snapshot` 真实依赖模型调用

---

## 11. 验收标准

本轮完成的验收标准如下：

1. 代码中存在可运行的 OpenAI 兼容 provider 适配器
2. 运行时默认不再走 mock 联调路径
3. `doctor` 能检查 provider 配置并输出可诊断结果
4. 本机 `Claude Code` 已成功注册该 MCP server
5. 有一套明确的红/绿测试说明
6. 至少有一条自动化绿测证明 provider 路径可通过 stub 服务走通
7. 现有 MCP tool 名称与参数不发生破坏性变化

---

## 12. 实施边界

为了控制 blast radius，本轮实现刻意不做以下事情：

- 不重写 `application::decide_with_snapshot`
- 不修改领域规则
- 不新增额外 MCP tool
- 不引入无关抽象层
- 不在第一版加入多厂商 fallback 与策略路由

实现应遵循：

- KISS：只补 provider 与接入闭环
- YAGNI：不提前做多 provider 编排
- SOLID：provider 变化停留在 adapter / config / runtime 层
- DRY：单测和 MCP E2E 共享一致的 provider 配置语义
