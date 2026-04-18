# Windows 开发与接入指南

这份文档面向在 Windows 上开发、验证和接入 `agent_llm_mm` 的协作者。

## 1. 环境前提

- 已安装 Rust toolchain
- `cargo` 可用
- 已安装 PowerShell 7
- 当前仓库内提供 Windows 入口脚本：
  - `pwsh -File .\scripts\agent-llm-mm.ps1 doctor`
  - `pwsh -File .\scripts\agent-llm-mm.ps1 serve`

## 2. 进入项目目录

```powershell
Set-Location 'D:\Code\agent_llm_mm'
```

请按你的本机实际路径替换上面的示例目录。

## 3. 准备本地配置

先复制一份本地配置文件：

```powershell
Copy-Item .\examples\agent-llm-mm.example.toml .\agent-llm-mm.local.toml
```

然后编辑 `agent-llm-mm.local.toml`：

- 固定自己的 `database_url`
- 选择 `provider`
- 填入自己的 API key

常见 Windows SQLite URL 示例：

```toml
database_url = "sqlite:///D:/back/agent-llm-mm-codex.sqlite"
```

## 4. 本机预检

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

预期输出为 JSON，至少包含：

- `transport`
- `database_url`
- `provider`
- `status`

## 5. 启动 MCP 服务

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

服务启动后会占用当前终端并等待 `stdio` JSON-RPC 输入，这是正常现象。

## 6. Codex 配置

推荐使用 PowerShell 入口脚本：

```toml
[mcp_servers.agent-llm-mm]
command = "pwsh"
args = ["-File", "D:/Code/agent_llm_mm/scripts/agent-llm-mm.ps1", "serve"]
env = { AGENT_LLM_MM_CONFIG = "D:/Code/agent_llm_mm/agent-llm-mm.local.toml" }
transport = "stdio"
```

## 7. 推荐验证顺序

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

## 8. 额外说明

- `agent-llm-mm.local.toml` 已被 `.gitignore` 忽略，不应提交。
- 正式数据、手工测试数据和实验数据建议分开使用不同数据库文件。
- 如果多个本机客户端共用同一 SQLite 文件，需要预期 SQLite 单写者模型带来的锁等待和状态互相影响。
