# 当前工作说明（2026-03-24）

> 说明：本文档保留为 `2026-03-24` 阶段快照，其中关于 reflection、默认数据库路径和测试统计的结论已被后续实现推进。当前仓库状态请以 [current-work-2026-03-25.md](current-work-2026-03-25.md) 为准。

## 概览

- 当前工作分支：`codex/self-agent-mcp`
- 当前提交：`d23af005b50297cf557b9eede2f080fd1672b1a0`
- 当前状态：实现已完成，工作树干净，已通过本地验证
- 运行形态：Rust 单 crate，MCP stdio 服务，架构为 `Functional Core + Imperative Shell`

本轮工作的目标，是把 self-agent memory MCP 服务从“功能可运行”推进到“关键失败模式被锁住、真实 stdio 启动链可验证、SQLite 持久化路径可用于实际运行”的状态。

## 本轮已完成内容

### 1. 反思路径的证据约束修复

针对 `run_reflection` 路径中 replacement claim 的证据约束进行了收紧：

- 当前实现会拒绝没有真实外部证据支撑的 inferred replacement claim
- 新增回归测试，验证失败时不会产生 replacement claim
- 新增回归测试，验证失败时不会生成 reflection 记录
- 同时保留正常的 observed replacement supersede 流程

当前采用的是 fail-closed 策略。因为 `run_reflection` use case 目前没有 evidence lookup / evidence count 输入能力，所以无法表达“有证据时允许 inferred replacement”的更细粒度语义；本轮修复选择先封住绕过点，而不是继续放行一个无法证明安全的 inferred replacement 流程。

### 2. 默认 stdio runtime 改为文件型 SQLite

默认配置不再使用内存库，而是改为文件型 SQLite，并支持环境变量覆盖：

- `AppConfig::default()` 默认返回文件型 SQLite URL
- 支持通过 `AGENT_LLM_MM_DATABASE_URL` 指定数据库路径
- stdio 集成测试使用独立临时库 + 环境变量注入，保证测试隔离和确定性

这样做的直接收益是：服务重启后状态不会丢失，真实 stdio 运行路径具备可用的持久化基础。

### 3. fresh stdio runtime 的 commitment gate 真实生效

本轮修复还补强了 commitment gate 的实际启动效果：

- SQLite bootstrap 时会 seed baseline commitment：`forbid:write_identity_core_directly`
- 新增真实 stdio E2E 测试
- 测试通过真实二进制、真实 stdio、真实 JSON-RPC 调用链验证：
  - fresh runtime 可构建 snapshot
  - snapshot 中包含 baseline commitment
  - `decide_with_snapshot` 会阻断 forbidden action
  - 被阻断时不会调用 model

### 4. SQLite 读写验证收口

SQLite adapter 的测试同步收口：

- 不再依赖测试里手工插入 baseline commitment
- 改为验证 store bootstrap 后即可读取 baseline commitment
- 保持反思事务提交与回滚语义测试继续通过

## 本轮涉及的核心文件

- `src/application/run_reflection.rs`
- `src/support/config.rs`
- `src/adapters/sqlite/store.rs`
- `tests/application_use_cases.rs`
- `tests/bootstrap.rs`
- `tests/mcp_stdio.rs`
- `tests/sqlite_store.rs`

## 本地验证结果

以下验证已在当前提交上 fresh 运行通过：

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

测试结果摘要：

- `application_use_cases`: 8 passed
- `bootstrap`: 4 passed
- `decision_flow`: 2 passed
- `domain_invariants`: 2 passed
- `domain_snapshot`: 6 passed
- `failure_modes`: 3 passed
- `mcp_stdio`: 4 passed
- `sqlite_store`: 4 passed

除测试套件外，还额外做了一次手工 restart persistence 验证：

- 使用同一个 SQLite 文件分别启动两个独立 stdio 进程
- 第一个进程写入 event / claim / episode
- 第二个进程重启后成功读取到已持久化的 claim、commitment 和 episode

这说明当前默认持久化路径至少在“跨进程、同库、重启后可恢复状态”这个基本目标上是成立的。

## 子代理执行与审查结论

### 已关闭的子代理

以下本轮使用过的子代理已关闭或回收：

- `019d1acd-3133-7253-8811-6796ab91872f`
- `019d1e73-dafe-7a41-abbf-e571d04c78b7`
- `019d1e74-a321-75d0-b4b0-efcf8ebd8040`
- `019d1e76-969c-7db0-8dcd-9ee7a2dddc2f`
- `019d1e48-c27c-7303-9408-318b0fe02969`

### Spec review 结论

Spec review 结论为通过：

- inferred replacement claim 已被拒绝，且失败无副作用
- 默认数据库不再是内存库，并支持 env override
- fresh stdio gate 有真实 E2E 证明
- 未见明显越界实现

### Code quality review 结论

Code quality review 给出了一组需要记录的提醒，其中有两类值得保留：

#### A. 可接受的安全收紧

review 指出：当前 `run_reflection` 会拒绝所有 inferred replacement claim，而不仅是“无证据的 inferred replacement claim”。

这个判断本身是成立的，但当前实现中没有 evidence 输入或查询能力，因此这不是“修错了”，而是有意采取的安全收紧。换言之：

- 当前行为比原先更严格
- 当前行为不是对未来理想语义的完整实现
- 但它优先保证不变量成立，避免继续接受无法证明安全的 inferred replacement

#### B. 默认数据库作用域的设计取舍

review 还指出：默认数据库路径现在是固定的文件路径，因此未显式设置 `AGENT_LLM_MM_DATABASE_URL` 的多个进程会共享同一份数据库。

这个提醒也是成立的。本轮实现实际上做的是下面这个取舍：

- 目标优先级一：默认路径必须可持久化，重启不丢状态
- 目标优先级二：测试与特殊调用方通过 env override 实现隔离
- 暂未实现：更精细的默认隔离策略，例如按项目、按 workspace、按用户 profile 或按会话命名数据库路径

因此，这里更适合被视为后续设计点，而不是当前提交的阻塞缺陷。

## 当前工作的总体判断

如果评价标准是“是否已经修复最后一轮 branch review 指出的 1 个 P0 + 2 个 P1 阻塞项”，答案是：已经修复，并具备足够的本地验证支撑。

如果评价标准提升为“默认运行策略是否已经是最终形态”，答案是：还不是。目前已经有一个安全、可运行、可持久化、可测试的最小闭环，但仍保留了若干明确的后续演进点。

## 后续建议

### 建议 1：为 reflection 引入显式 evidence 输入或查询能力

这是最重要的后续增强项。

目标是把当前的 fail-closed 策略升级为真正的 evidence-aware 策略：

- `run_reflection` 接收 evidence references，或能查询可用 evidence count
- 允许“有证据的 inferred replacement”
- 保持“无证据的 inferred replacement”继续被拒绝
- 为允许分支增加正向测试，而不是只覆盖拒绝分支

### 建议 2：把默认数据库路径策略从“固定 temp 文件”升级为“显式作用域”

建议为默认路径增加更清晰的作用域语义，例如：

- 按项目路径派生数据库名
- 按 workspace 派生数据库名
- 或通过显式 CLI / config 设定默认数据库位置

这样可以同时保留持久化能力，并降低不同项目或不同会话之间的共享状态串扰风险。

### 建议 3：把 baseline commitment seeding 从 store 层上移到 runtime/bootstrap policy 层

当前把 seeding 放在 `SqliteStore::bootstrap()` 里是最小实现，但从分层语义看，更干净的做法是：

- `SqliteStore` 只负责持久化
- `Runtime::bootstrap()` 或更高层的启动策略负责“是否写入 baseline commitments”

这样可以减少存储适配器对上层业务策略的耦合。

### 建议 4：补一条“默认路径行为”的自动化测试

当前测试已经证明：

- 显式 env override 时行为正确
- fresh stdio E2E 可工作
- 手工跨进程 restart persistence 成立

但还缺一条专门覆盖默认路径策略的自动化测试，例如：

- 清理相关 env
- 启动两个独立进程
- 验证默认路径是否满足预期共享或隔离语义

这样后续无论你决定保留共享默认路径，还是切到按项目隔离，都能被 CI 明确保护住。

## 建议的下一步顺序

建议按下面顺序继续：

1. 明确产品语义：默认数据库到底应该按用户共享，还是按项目隔离
2. 如果选择按项目隔离，先改默认路径策略，再补回归测试
3. 为 reflection 设计 evidence-aware 接口，再把 inferred replacement 从“全拒绝”升级为“按证据判定”
4. 如需长期维护该服务，再考虑把 baseline policy 从 store 层上移

## 结论

当前这条分支已经达到“可保留、可继续集成、可进入下一轮设计收口”的状态。

它不是最终形态，但已经把最危险的错误路径封住了，也把真实 stdio 启动链上的持久化与 gate 行为拉到了可验证状态。后续工作重点，不再是补救性修 bug，而是把默认运行语义和 reflection 证据模型正式做清楚。
