# macOS 开发与接入指南

这份文档面向当前在 macOS 上开发、验证和接入 `agent_llm_mm` 的协作者。

## 1. 环境前提

- 已安装 Rust toolchain
- `cargo` 可用
- 使用 `zsh` 或 `bash`
- 当前仓库内提供 macOS 原生入口脚本：
  - `./scripts/agent-llm-mm.sh doctor`
  - `./scripts/agent-llm-mm.sh serve`
  - `./scripts/run-self-revision-demo.sh`

## 2. 进入项目目录

```zsh
cd ~/code/agent-llm-mm
```

请按你的本机实际路径替换上面的示例目录。

## 3. 准备本地配置

先复制一份本地配置文件：

```zsh
cp examples/agent-llm-mm.example.toml agent-llm-mm.local.toml
```

然后编辑 `agent-llm-mm.local.toml`：

- 固定自己的 `database_url`
- 选择 `provider`
- 填入自己的 API key

建议的 macOS SQLite URL 示例：

```toml
database_url = "sqlite:///Users/<you>/Library/Application%20Support/agent-llm-mm-codex.sqlite"
```

## 4. 本机预检

优先使用仓库内脚本：

```zsh
./scripts/agent-llm-mm.sh doctor
```

如果你想绕过脚本，也可以：

```zsh
cargo run --quiet --bin agent_llm_mm -- doctor
```

预期输出为 JSON，至少包含：

- `transport`
- `database_url`
- `provider`
- `status`

## 5. 启动 MCP 服务

```zsh
./scripts/agent-llm-mm.sh serve
```

或：

```zsh
cargo run --quiet --bin agent_llm_mm -- serve
```

服务启动后会占用当前终端并等待 `stdio` JSON-RPC 输入，这是正常现象。

## 6. Codex 配置

推荐直接使用 macOS 原生入口脚本：

```toml
[mcp_servers.agent-llm-mm]
command = "/absolute/path/agent-llm-mm/scripts/agent-llm-mm.sh"
args = ["serve"]
env = { AGENT_LLM_MM_CONFIG = "/absolute/path/agent-llm-mm/agent-llm-mm.local.toml" }
transport = "stdio"
```

如果你不想经过脚本，也可以：

```toml
[mcp_servers.agent-llm-mm-cargo]
command = "cargo"
args = ["run", "--quiet", "--", "serve"]
env = { AGENT_LLM_MM_CONFIG = "/absolute/path/agent-llm-mm/agent-llm-mm.local.toml" }
transport = "stdio"
```

## 7. 推荐验证顺序

```zsh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

## 8. Self-Revision Demo Package

如果要在本机快速验证 automatic self-revision MVP 的完整证据链：

```zsh
./scripts/run-self-revision-demo.sh
```

固定输出目录：

```zsh
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

该脚本会构建本地二进制、启动 deterministic `openai-compatible` stub provider，并通过真实 MCP `stdio` 服务生成：

- `doctor.json`
- `snapshot-before.json`
- `snapshot-after.json`
- `decision-before.json`
- `decision-after.json`
- `timeline.json`
- `sqlite-summary.json`
- `report.md`

这条 demo 不需要真实 API key，也不会访问外网。

## 9. 额外说明

- `agent-llm-mm.local.toml` 已被 `.gitignore` 忽略，不应提交。
- 未显式设置 `database_url` 时，默认库会落到当前平台的用户数据目录，并按“本机用户共享”语义复用。
- 正式数据、手工测试数据和实验数据建议分开使用不同数据库文件。
- 如果多个本机客户端共用同一 SQLite 文件，需要预期 SQLite 单写者模型带来的锁等待和状态互相影响。
