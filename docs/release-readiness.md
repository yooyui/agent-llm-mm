# 发布准备评估

## 结论

当前 demo 可以发布到 GitHub，但建议定位为：

- 技术 demo
- research-oriented MVP
- local MCP integration prototype

发布前执行口径见 [Release Gate](release-gate.md)。该 gate 只覆盖本机 Rust MCP `stdio` technical demo / MVP 的最低发布核验，不代表 production autonomy、remote administration、multi-tenant deployment 或 background daemon readiness。

不建议定位为：

- 生产级产品
- 完整 self-agent memory system
- 已接入真实模型的成熟决策引擎

## 公开仓库附加说明

如果本仓库以公开仓库形式发布，当前这套文档口径是成立的：

- 它明确承认这是一个 MVP / demo
- 它没有把最小 provider 集成包装成完整产品能力
- 它保留了原始设计讨论和实现复核资料
- 它说明了本仓库是在 OpenAI Codex 的协作式开发与讨论流程中逐步形成的

这类表述对公开仓库是有帮助的，因为它能让读者同时看到：

- 当前成果
- 当前边界
- 开发方法

## 为什么现在可以发布

### 1. 工程闭环已经存在

当前仓库已经打通：

- 事件写入
- 命题持久化
- 自我快照构建
- 最小决策门控
- 反思修订

### 2. 本机接入路径已经存在

当前已有：

- `doctor`
- `serve`
- 本机 MCP `stdio` 服务入口
- Codex MCP 配置样例

### 3. 自动化验证已经存在

截至 `2026-04-28` 的 fresh 验证结果：

- `cargo test` 全量通过，共 151 个测试
- `doctor` 返回 `status = ok`

### 4. 当前边界已经能被文档清楚说明

这是能否上 GitHub 的关键点。当前仓库虽然仍是 MVP，但它的边界并不是模糊的：

- `openai-compatible` provider 已接入
- provider 配置走本地 TOML 文件
- richer memory semantics 还在后续阶段
- 默认数据库作用域已明确为“本机用户共享默认库，隔离靠显式 `database_url`”

只要这些边界在 README 和说明文档里写清楚，这个仓库就适合发布为协作型 demo。

## 当前不适合过度承诺的点

### 1. `decide_with_snapshot`

- gate 是真的
- 已可走 `openai-compatible` provider
- 返回契约仍是最小动作字符串

因此不应把它写成“完整 AI 决策引擎”。

### 2. memory 语义仍然是 MVP

当前已经有骨架，但还没有完整实现：

- minimal deeper reflection 已有，但 richer schema / versioned reflection policy 仍未完成
- richer identity model
- richer episode model
- procedural memory

### 3. 数据隔离策略已有最小可发布结论

SQLite 落盘已经可用，默认路径语义也已收口为“本机用户共享默认库”。剩余注意点不在于语义不清，而在于正式数据、测试数据和实验数据仍应通过显式 `database_url` 主动隔离。

## 建议的发布口径

如果你要在 GitHub 上写一句简短介绍，建议使用这种口径：

“A Rust-based local MCP `stdio` memory demo for AI clients. It validates a minimal loop around interaction ingestion, self-snapshot construction, gated decisions, and reflection, backed by SQLite persistence.”

## 发布前建议检查项

### 最低必做

- 按 [Release Gate](release-gate.md) 跑完整发布 gate（minimum gate、self-revision 证据 gate、dashboard gate），并单独记录 sandbox-only failure 与代码失败的区别
- 确认 README、状态文档、路线图、三语说明都已更新
- 确认接入命令与验证命令可以直接复制使用
- 确认对“已实现 / 部分实现 / 未实现”的边界没有过度承诺

### 如果准备公开仓库

- 已补齐 `Apache-2.0` `LICENSE` 与 `NOTICE`
- 确认 `CONTRIBUTING.md` 与 README 链接可直接使用
- 确认公开说明里不会误导读者认为它已经是完整产品

### 如果只是团队私有协作

- 可以先不补开源协作文档
- 重点保证 README、文档索引和验证说明足够清楚
