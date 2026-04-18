# 项目说明

## 简述

`agent_llm_mm` 是一个 Rust 编写的本机 MCP `stdio` 服务，用于验证 AI 客户端里的长期记忆 / 自我快照 / 反思修订最小闭环。当前版本以 SQLite 为持久化基础，更适合作为技术 demo、集成验证仓库和研究型原型，而不是完整产品。

## 当前范围

- 本机 `stdio` MCP 服务
- SQLite 持久化
- 配置文件驱动的 provider 加载
- `openai-compatible` provider
- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`
- `doctor` / `serve` 启动入口

## 当前边界

- `decide_with_snapshot` 已可走 `openai-compatible` provider，但返回契约仍是最小动作字符串
- 没有远程 HTTP transport
- 没有 richer evidence lookup / weight / relation
- 还没有更多 provider 类型
- 没有更完整的多层 memory 体系

## 适合的使用方式

- 本机 AI 客户端接入实验
- self-agent memory 相关概念验证
- Rust + MCP + SQLite 的最小工程骨架参考

## 文档约束

- 每次处理完一个任务后，如果该任务影响了行为、能力边界、接入方式、配置、验证命令或协作规则，必须同步更新对应文档。
- 不应把文档更新留到最后统一处理；代码与文档应尽量在同一轮任务内一起收口。

## 当前验证状态

截至 `2026-03-31`：

- `cargo test` 全量通过，共 58 个测试
- `doctor` 返回 `status = ok`

## 致谢

本仓库在开发、讨论和文档整理过程中明确使用了 OpenAI Codex 作为协作式开发工具。感谢 OpenAI 提供相关工具与研究生态，使这种以讨论驱动、迭代收口的开发方式成为可能。
