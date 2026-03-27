# agent_llm_mm

一个面向本机 AI 工具接入的 Rust MCP `stdio` 服务，当前重点是把“长期记忆 + 自我快照 + 反思修订”的最小闭环跑通。

## 当前状态

- 运行形态：本机 `stdio` MCP 服务
- 存储：SQLite
- 测试状态：仓库内含完整自动化测试
- 适用场景：嵌入 Codex 等支持本地 MCP 子进程的 AI 客户端

## 当前能力

- `ingest_interaction`
  - 记录交互事件并提取命题
- `build_self_snapshot`
  - 从持久化记忆构建受预算控制的自我快照
- `run_reflection`
  - 以审计友好的方式 supersede 已有 claim，并支持显式 evidence 输入
- `decide_with_snapshot`
  - 目前仍使用 mock model，更适合调试和演示，不应视为生产级决策引擎

## 非目标

- 远程 HTTP 服务
- 多租户或多 Agent 编排
- 可视化管理后台
- 内置真实 LLM provider

## 环境要求

- Windows 11
- PowerShell 7
- Rust toolchain
- `cargo`

## 快速开始

进入项目目录：

```powershell
Set-Location 'D:\Code\agent_llm_mm\.worktrees\codex-self-agent-mcp'
```

建议先固定数据库路径，不要使用系统临时目录：

```powershell
$env:AGENT_LLM_MM_DATABASE_URL = 'sqlite:///D:/back/agent-llm-mm-codex.sqlite'
```

先做本地预检：

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

如果预检通过，再启动 MCP 服务：

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

服务启动后终端会保持占用并等待 `stdio` JSON-RPC 输入，这是正常现象。

## Codex 本机 MCP 配置

示例配置见：

- [examples/codex-mcp-config.toml](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/examples/codex-mcp-config.toml)
- [docs/local-mcp-integration-2026-03-26.md](/D:/Code/agent_llm_mm/.worktrees/codex-self-agent-mcp/docs/local-mcp-integration-2026-03-26.md)

## 验证命令

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

## 接入注意事项

- 这是 `stdio` MCP 服务，不会输出 Web 地址或启动 HTTP 端口。
- 生产使用时请显式设置 `AGENT_LLM_MM_DATABASE_URL`，避免落到系统临时目录。
- 建议为正式数据和实验数据使用不同 SQLite 文件。
- 如果多个本机客户端共用同一 SQLite 文件，需预期 SQLite 单写者模型带来的竞争和锁等待。
- `decide_with_snapshot` 还未接真实模型，不建议把它当成正式决策能力对外承诺。
