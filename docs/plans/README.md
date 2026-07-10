# 项目计划索引

本目录只保留一个当前执行入口。历史计划可以用于追溯，但不得覆盖当前代码事实、项目状态或 active plan。

## 当前计划

- [2026-07-10-product-replan.md](2026-07-10-product-replan.md)
  - 状态：`active / planning complete / implementation not started`
  - 目标：把现有 technical MVP 收束为可信、可检索、可审计、可恢复的本地 MCP Memory Ledger
  - 当前里程碑：`M0 Truth and Safety Reset`

## 权威层级

| 文档 | 唯一职责 |
| --- | --- |
| [`docs/positioning.md`](../positioning.md) | 公共名称、目标用户和对外口径 |
| [`docs/project-status.md`](../project-status.md) | 当前分支真实实现状态 |
| [`docs/roadmap.md`](../roadmap.md) | Now / Next / Later 产品路线 |
| [当前 active plan](2026-07-10-product-replan.md) | 任务顺序、依赖、证据门和停止条件 |
| [`docs/product/follow-up-reality-gates.md`](../product/follow-up-reality-gates.md) | 已实现、部分实现、未验证和阻断证据 |

出现冲突时，先以当前代码和测试证据为准，再按上表顺序修正文档。测试文件存在不等于测试已经在当前分支通过；只有当前运行记录可以作为 fresh evidence。

## 已被取代的计划

以下文件保留为历史设计与执行记录，不再作为下一步任务来源：

- `docs/superpowers/plans/2026-05-09-productization-roadmap.md`
- `docs/superpowers/plans/2026-05-09-local-product-alpha-development-tasks.md`
- `docs/superpowers/plans/2026-05-16-formal-product-readiness-12-workstreams.md`
- `docs/superpowers/plans/2026-05-24-p1-p2-p3-product-completion-plan.md`
- `docs/superpowers/plans/2026-06-08-non-mvp-product-completion.md`
- `docs/progress-tracker.md`

这些历史文件中的已完成项、未完成框和阶段名称只描述当时语境。后续任务必须从当前 active plan 领取，并把结果写回当前状态与 reality gate。

## 更新规则

1. 开始实现前，只把当前 active milestone 的一个最小切片标为进行中。
2. 代码、测试与对应文档在同一任务更新。
3. 先记录验证命令和结果，再改变完成状态。
4. 遇到证据不符时停止扩张，不自动进入下一个里程碑。
5. 新的大方向先记录决策，不另建第二套并行总路线。
