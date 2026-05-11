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

先选择一个 profile，再复制为本机私有配置文件：

```powershell
Copy-Item .\examples\agent-llm-mm.dev.example.toml .\agent-llm-mm.local.toml
```

可选 profile：

- `examples/agent-llm-mm.dev.example.toml`: 本地开发和手工测试，默认 `provider = "mock"`，dashboard disabled。
- `examples/agent-llm-mm.prod-local.example.toml`: 正式本地数据，dashboard 只监听 `127.0.0.1`，daemon disabled；复制后必须替换 `database_url` 和 provider 占位值。
- `examples/agent-llm-mm.demo.example.toml`: self-revision demo runner 专用，通常不要手工复制为日常配置。

`examples/agent-llm-mm.example.toml` 只是通用入口说明，不再承载所有用途。然后编辑 `agent-llm-mm.local.toml`：

- 固定自己的 `database_url`
- 选择 `provider`
- dev/mock profile 不需要 API key；只有选择 `openai-compatible` 或 prod-local profile 时，才在已忽略的 `agent-llm-mm.local.toml` 里填写 `base_url`、`api_key` 和 `model`

常见 Windows SQLite URL 示例；dev、demo、prod-local 必须使用不同文件：

```toml
database_url = "sqlite:///D:/agent-llm-mm/dev.sqlite"
```

## 4. 本机预检

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

示例 profile 是结构模板，包含占位 `database_url`。先复制到 `agent-llm-mm.local.toml`，替换为本机可写 SQLite 路径后，再检查本机私有配置；如果选择 prod-local profile，还必须同时替换 `base_url`、`api_key` 和 `model`：

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 doctor .\agent-llm-mm.local.toml
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
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

发布前请按 [Release Gate](release-gate.md) 跑完整 gate；本节只是 Windows 日常验证入口。Release gate 中的 `./scripts/agent-llm-mm.sh doctor` 在 Windows 上对应 `pwsh -File .\scripts\agent-llm-mm.ps1 doctor`。

## 7.1 本地接入排障

| Symptom | Likely Cause | Verification | Fix |
| --- | --- | --- | --- |
| `doctor` cannot write SQLite | database path not writable or sandbox restriction | 先检查 `agent-llm-mm.local.toml` 的 `database_url` 与 PowerShell 启动环境里的 `AGENT_LLM_MM_DATABASE_URL`；如果 `doctor` 已返回 JSON，再核对其中的 `database_url` | 设置 `AGENT_LLM_MM_DATABASE_URL` 到可写 SQLite 路径，或在本地 TOML 固定可写路径后重试 |
| MCP client starts the wrong binary | auxiliary `src/bin` target ambiguity | 检查 MCP 客户端配置里的参数是否带 `--bin agent_llm_mm`（如 `args = ["run", "--quiet", "--bin", "agent_llm_mm", "--", "serve"]`） | 统一使用 PowerShell 脚本入口，或在客户端里固定 `cargo run --quiet --bin agent_llm_mm -- serve` |
| dashboard not visible | `[dashboard].enabled` 为 false，或端口不可用 | 查看 TOML 的 `[dashboard]` 区块和 `enabled`，以及 `pwsh -File .\scripts\agent-llm-mm.ps1 doctor` 输出 | 设置 `[dashboard].enabled = true`，并改用可用的本地端口（如 `127.0.0.1:8787`） |
| model calls fail | provider 配置不完整 | 执行 `pwsh -File .\scripts\agent-llm-mm.ps1 doctor`，确认 `provider`、`base_url`、`model` 已配置 | 在本地 TOML 更新 provider 信息；密钥仅放本地文件，不要提交 |

## 8. 额外说明

- `agent-llm-mm.local.toml` 已被 `.gitignore` 忽略，不应提交。
- 正式数据、手工测试数据和 demo 数据必须分开使用不同数据库文件；prod-local 只用于要保留、检查或备份的本地正式数据。
- 如果多个本机客户端共用同一 SQLite 文件，需要预期 SQLite 单写者模型带来的锁等待和状态互相影响。
- 所有示例 profile 都保持 `[daemon].enabled = false`；后续 daemon 观察模式必须走单独 gate。
