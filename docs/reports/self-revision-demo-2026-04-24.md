# Self-Revision Demo Report (2026-04-24)

## 摘要

这份 canonical report 记录 `2026-04-24` self-revision demo package 的稳定结论口径。最新本地运行仍应以 `target/reports/self-revision-demo/<timestamp>/report.md` 为准。

## Latest Canonical Run

- transport: `stdio`
- provider: `openai-compatible`
- durable write path: `run_reflection`
- status: `ok`

Runtime hooks:

- `ingest_interaction:failure`
- `ingest_interaction:conflict`
- `decide_with_snapshot:conflict`
- `build_self_snapshot:periodic`

Timeline:

- gate before update blocked: `true`
- handled conflict rows before explicit hints: `0`
- handled conflict rows after explicit hints: `1`

Current commitments:

- `prefer:confirm_conflicting_commitment_updates_before_overwrite`
- `forbid:write_identity_core_directly`

## 证明链路

| 检查点 | 预期 |
| --- | --- |
| `doctor.json` | `status = ok`，`self_revision_write_path = run_reflection` |
| `snapshot-before.json` | 只包含 baseline commitment，不包含 revised commitment |
| negative event | 不带 conflict hint，不新增 handled conflict ledger |
| positive event | 带 conflict hint，新增 handled conflict ledger |
| `snapshot-after.json` | 包含 `prefer:confirm_conflicting_commitment_updates_before_overwrite` |
| `decision-before.json` | 返回 baseline action |
| `decision-after.json` | 返回 revised action |
| `sqlite-summary.json` | 可复核 events / commitments / reflections / trigger ledger 的落库状态 |

## 关键边界

- demo runner 通过真实 MCP `stdio` 服务调用现有工具。
- self-revision proposal 来自本地 deterministic `openai-compatible` stub provider。
- durable 更新仍经由 `run_reflection`。
- 这不是后台自治进程，也不是新的 MCP tool。

## 复现命令

```zsh
./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest
```

通过后，`target/reports/self-revision-demo/latest/report.md` 会给出本次运行的实际目录内摘要。
