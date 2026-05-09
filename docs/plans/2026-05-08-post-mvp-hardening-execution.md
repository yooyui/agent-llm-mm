# Post-MVP Hardening 执行记录

> 执行日期：2026-05-08 | 分支：dev-work | 基线：153 → 155 tests passing

## 决策记录

| # | 决策 | 理由 |
|---|------|------|
| D1 | 删除空 `failure` 文件 | 0字节遗留物，无实际内容，git status 噪音 |
| D2 | Phase A+B 先行，C+F 并行 Agent | A/B 是文档收口无代码风险；C/F 独立无交叉文件 |
| D3 | 在 dev-work 分支直接工作 | 已有隔离 worktree 但均为旧会话，当前分支干净 |
| D4 | bounded recency (Phase F) 作为本次核心代码交付 | 是 evidence-query-v2 spec 明确要求的下一片 |
| D5 | Phase G 已有覆盖，无需新增 | `dashboard_rejects_write_methods_on_read_only_routes` 已存在 |

## 执行结果

### Phase A: 项目卫生 ✅
- [x] 删除空 `failure` 文件
- [x] 确认测试计数与文档一致 (153)

### Phase B: Release Gate 补强 ✅
- [x] 补充 release-gate.md 中的 Troubleshooting 表格
- [x] 补充 Doctor Output Fields 段落

### Phase C: Runtime Hook 契约锁定 ✅
- [x] 新增 `runtime_hook_contract_is_exactly_four_hooks` 测试
- [x] 断言 `AUTO_REFLECTION_RUNTIME_HOOKS` 恰好为 4 条指定 hook

### Phase F: Evidence Query v2 - Bounded Recency ✅
- [x] `EvidenceQuery` 添加 `recorded_after: Option<DateTime<Utc>>` 和 `recorded_before: Option<DateTime<Utc>>`
- [x] SQLite store `query_evidence_event_ids_with_limit` 实现 inclusive `recorded_at >= ?` / `recorded_at <= ?` 过滤
- [x] 新增 `sqlite_query_evidence_event_ids_filters_by_recency_window` 测试（覆盖 after-only / before-only / window 与 inclusive 边界场景）
- [x] MCP DTO 与 automatic self-revision proposal narrowing 均保留 `recorded_after` / `recorded_before` 过滤语义

### Phase G: Dashboard 边界 ✅
- [x] 确认 `dashboard_rejects_write_methods_on_read_only_routes` 测试已存在

## 验证结果

```
cargo fmt --check       ✅
cargo clippy            ✅ (0 warnings)
cargo test              ✅ (155 tests, 0 failures)
```

## 变更文件清单

| 文件 | 变更类型 |
|------|---------|
| `docs/release-gate.md` | 补充 troubleshooting + doctor fields |
| `src/ports/event_store.rs` | EvidenceQuery 添加 recency 字段 |
| `src/adapters/sqlite/store.rs` | SQL 过滤实现 |
| `src/application/auto_reflect_if_needed.rs` | 调用点补齐新字段 |
| `src/interfaces/mcp/dto.rs` | 调用点补齐新字段 |
| `tests/bootstrap.rs` | hook 契约测试 |
| `tests/sqlite_store.rs` | recency window 测试 |
| `tests/application_use_cases.rs` | 调用点补齐 |
| `tests/failure_modes.rs` | 调用点补齐 |
| `tests/openai_compatible_model.rs` | 调用点补齐 |
| `docs/plans/2026-05-08-post-mvp-hardening-execution.md` | 本文件 |

## 后续建议

- Phase D (Diagnostics): 改进 structured diagnostics 输出格式
- Phase E (Reflection Contract): 补充 deeper-update 边界测试
- Phase H (Provider Contract): 补充 provider readiness checklist 验证
