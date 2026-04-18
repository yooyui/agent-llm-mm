# 本机 MCP 接入说明（2026-03-26，按 2026-03-27 实现复核更新）

## 1. 目标

把 `agent_llm_mm` 以本机 `stdio` MCP 服务的方式接入 Codex 等 AI 客户端，并保证：

- 启动路径稳定
- SQLite 落盘路径可控
- 有独立的预检命令
- 能清楚区分“可正式嵌入的能力”和“仍属 mock/实验的能力”

## 2. 推荐接入形态

macOS 下优先入口是 `scripts/agent-llm-mm.sh`。它提供 `doctor` / `serve` 两种封装，并贴合当前 zsh / bash 环境。

如果你想完全绕过脚本，也可直接运行 `cargo run --quiet -- <serve|doctor>`。

原因：

- 可以从任意当前目录启动
- 可以固化项目根目录
- 可以统一 `serve` / `doctor` 两种模式
- 后续切换为预编译二进制时，客户端配置无需大改

入口脚本：

- [scripts/agent-llm-mm.sh](../scripts/agent-llm-mm.sh)

## 3. 本机预检

正式接入前，先执行：

```zsh
cd ~/code/agent-llm-mm
cp examples/agent-llm-mm.example.toml agent-llm-mm.local.toml
./scripts/agent-llm-mm.sh doctor
```

预期输出为 JSON，至少包含：

- `transport`
- `database_url`
- `status`

当前 `status = "ok"` 代表：

- 配置已解析
- SQLite 已可成功 bootstrap
- provider 已按配置完成校验
- 默认 runtime 初始化已通过

## 4. 启动服务

```zsh
cd ~/code/agent-llm-mm
./scripts/agent-llm-mm.sh serve
```

注意：

- 该命令会启动 MCP `stdio` 服务
- 终端看起来像“挂住”，这是正确行为
- 不应在服务运行期间往标准输入随意写普通文本

## 5. Codex 配置示例

你当前机器上的 Codex 配置格式已经在使用：

- `[mcp_servers.<name>]`
- `command`
- `args`
- `env`

可直接参考：

- [examples/codex-mcp-config.toml](../examples/codex-mcp-config.toml)

推荐做法：

- `command` 指向 `scripts/agent-llm-mm.sh`
- `args` 只传 `serve`
- `env` 显式传入 `AGENT_LLM_MM_CONFIG`

```toml
command = "/absolute/path/agent-llm-mm/scripts/agent-llm-mm.sh"
args = ["serve"]
```

如你不想经过脚本，也可直接用 `cargo run`：

```toml
command = "cargo"
args = ["run", "--quiet", "--", "serve"]
```

## 6. 当前能力状态

### 已实现

- `ingest_interaction`
- `build_self_snapshot`
- `run_reflection`
- `doctor` / `serve`
- SQLite 持久化
- `namespace` 最小闭环
- `openai-compatible` provider
- 配置文件驱动的 provider 选择

### 部分实现

- `decide_with_snapshot`

原因：

- commitment gate 是真实能力
- 下游模型已可走 `openai-compatible`
- 返回契约仍是最小动作字符串
- 更适合作为流程验证能力，而不是最终生产决策能力

### 未实现

- richer 自动 evidence lookup
- richer reflection 语义（当前已有最小 `identity_core` / `commitments` 深层修订，但仍不是 richer schema / versioned policy）
- 更多 provider 类型

## 7. 正式接入时需要注意的点

### 7.1 数据库路径

未显式设置 `database_url` 时，默认库会落到当前平台的用户数据目录，并按“本机用户共享默认库”语义复用。正式接入时仍建议在 `agent-llm-mm.local.toml` 里固定为你可备份、可区分环境的路径，例如：

```toml
sqlite:///Users/<you>/Library/Application%20Support/agent-llm-mm-codex.sqlite
```

### 7.1.1 配置文件

推荐用法：

- 从 `examples/agent-llm-mm.example.toml` 复制一份本地配置
- 写入自己的 `database_url`
- 选择 `provider`
- 填入自己的 API key
- 不要把 `agent-llm-mm.local.toml` 提交到仓库

### 7.2 数据隔离

建议至少区分：

- 正式接入库
- 手工测试库
- 开发实验库

避免把反思、修订和测试事件混入正式记忆。

### 7.3 并发访问

SQLite 非常适合本机 MVP，但它仍然是单写者模型。若多个 AI 客户端并发共享同一数据库文件，需要预期：

- 锁等待
- 写入竞争
- 调试时状态互相影响

更稳妥的做法是每个环境单独一份数据库文件。

### 7.4 日志与 stdout

这是 `stdio` MCP 服务，因此：

- MCP 协议通信依赖标准输入输出
- 不应在 `serve` 模式额外向 `stdout` 打印杂讯
- 诊断信息应放到 `doctor` 模式或日志侧

### 7.5 能力边界

当前这条分支已经具备可嵌入的最小记忆闭环，但还不是完整产品：

- 无 Web UI
- 无 HTTP transport
- 无更丰富的 evidence 自动检索
- 已有最小 `identity_update` / `commitment_updates` 反思修订，但仍无 richer schema、版本化策略与更细粒度生命周期
- 无更多 provider 类型

## 8. 推荐验证顺序

```zsh
cd ~/code/agent-llm-mm
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
./scripts/agent-llm-mm.sh doctor
```

如果都通过，再把它挂到本机 MCP 客户端配置里。
