# Self-Agent Memory Self-Revision MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在现有 `events -> claims -> self_snapshot -> decision -> reflection` 闭环之上，增加一个“自动触发 + 模型提案 + 服务端治理 + 审计落库”的 self-revision MVP，同时继续把 `run_reflection` 保持为唯一长期层写路径。

**Architecture:** 新增一个应用层协调器 `auto_reflect_if_needed`，在 MCP 入口前统一做 `conflict / failure / periodic` 检测、trigger ledger 判重、模型提案和服务端门槛校验，并把通过的 proposal 转译成现有 `run_reflection` 输入。新增持久化 `reflection_trigger_ledger` 表、模型 proposal 契约、trigger hints DTO 与防递归保护；长期层更新仍通过 `run_reflection` 完成，以保留 supersede/dispute/expire 语义和审计痕迹。

**Tech Stack:** Rust 2024, Tokio, RMCP stdio, SQLite, SQLx, Serde, Reqwest, existing mock/openai-compatible model adapters

---

### Task 1: 固定 self-revision 领域契约与模型 proposal 端口

**Files:**
- Create: `src/domain/self_revision.rs`
- Modify: `src/domain/mod.rs`
- Modify: `src/ports/model_port.rs`
- Modify: `src/adapters/model/mock.rs`
- Modify: `src/adapters/model/openai_compatible.rs`
- Test: `tests/openai_compatible_model.rs`
- Test: `tests/application_use_cases.rs`

- [x] **Step 1: 写失败测试，固定 self-revision proposal 的最小契约**

```rust
#[tokio::test]
async fn mock_model_returns_no_revision_when_snapshot_has_no_signals() {
    let model = MockModel;
    let proposal = model
        .propose_self_revision(SelfRevisionRequest::new(
            TriggerType::Failure,
            Namespace::self_(),
            SelfSnapshot {
                identity: vec![],
                commitments: vec![],
                claims: vec![],
                evidence: vec![],
                episodes: vec![],
            },
            vec![],
            vec![],
        ))
        .await
        .unwrap();

    assert!(!proposal.should_reflect);
    assert!(proposal.machine_patch.identity_patch.is_none());
    assert!(proposal.machine_patch.commitment_patch.is_none());
}
```

- [x] **Step 2: 运行红测，确认现有 `ModelPort` 还不支持 self-revision proposal**

Run: `cargo test --test application_use_cases mock_model_returns_no_revision_when_snapshot_has_no_signals -v`

Expected: FAIL with compile errors for missing `SelfRevisionRequest` / `propose_self_revision`

- [x] **Step 3: 实现领域契约与模型端口扩展**

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerType {
    Conflict,
    Failure,
    Periodic,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionRequest {
    pub trigger_type: TriggerType,
    pub namespace: Namespace,
    pub snapshot: SelfSnapshot,
    pub evidence_event_ids: Vec<String>,
    pub trigger_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionProposal {
    pub should_reflect: bool,
    pub rationale: String,
    pub machine_patch: SelfRevisionPatch,
}

#[async_trait]
pub trait ModelPort {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError>;
    async fn propose_self_revision(
        &self,
        request: SelfRevisionRequest,
    ) -> Result<SelfRevisionProposal, AppError>;
}
```

- [x] **Step 4: 为 mock / openai-compatible 适配器补最小 proposal 行为**

```rust
#[async_trait]
impl ModelPort for MockModel {
    async fn propose_self_revision(
        &self,
        request: SelfRevisionRequest,
    ) -> Result<SelfRevisionProposal, AppError> {
        Ok(SelfRevisionProposal::no_revision(
            format!("mock model did not detect a valid {:?} revision", request.trigger_type),
        ))
    }
}
```

- [x] **Step 5: 运行绿测，确认 proposal 契约与适配器都稳定**

Run: `cargo test --test application_use_cases --test openai_compatible_model -v`

Expected: PASS with new self-revision proposal tests included

### Task 2: 增加 trigger ledger 持久化与 SQLite 迁移

**Files:**
- Create: `src/ports/trigger_ledger_store.rs`
- Modify: `src/ports/mod.rs`
- Modify: `src/adapters/sqlite/schema.rs`
- Modify: `src/adapters/sqlite/store.rs`
- Test: `tests/sqlite_store.rs`

- [x] **Step 1: 写失败测试，固定 ledger 的判重与 periodic 水位行为**

```rust
#[tokio::test]
async fn sqlite_trigger_ledger_records_namespace_periodic_watermark_and_cooldown() {
    let context = test_support::new_sqlite_store().await;

    context
        .store
        .record_trigger_attempt(StoredTriggerLedgerEntry::handled(
            "ledger-1",
            TriggerType::Periodic,
            "project/agent-llm-mm",
            "periodic:project/agent-llm-mm",
            20,
        ))
        .await
        .unwrap();

    let entry = context
        .store
        .latest_trigger_entry("periodic:project/agent-llm-mm")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(entry.namespace.as_str(), "project/agent-llm-mm");
    assert_eq!(entry.episode_watermark, Some(20));
    assert_eq!(entry.status, TriggerLedgerStatus::Handled);
}
```

- [x] **Step 2: 运行红测，确认当前 schema/store 不含 ledger**

Run: `cargo test --test sqlite_store sqlite_trigger_ledger_records_namespace_periodic_watermark_and_cooldown -v`

Expected: FAIL with missing schema / trait / method errors

- [x] **Step 3: 新增持久化表与存储端口**

```sql
CREATE TABLE IF NOT EXISTS reflection_trigger_ledger (
    ledger_id TEXT PRIMARY KEY,
    trigger_type TEXT NOT NULL,
    namespace TEXT NOT NULL,
    trigger_key TEXT NOT NULL,
    status TEXT NOT NULL,
    evidence_window TEXT NOT NULL DEFAULT '[]',
    handled_at TEXT,
    cooldown_until TEXT,
    episode_watermark INTEGER,
    reflection_id TEXT
);
```

```rust
#[async_trait]
pub trait TriggerLedgerStore {
    async fn record_trigger_attempt(&self, entry: StoredTriggerLedgerEntry) -> Result<(), AppError>;
    async fn latest_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError>;
}
```

- [x] **Step 4: 实现 SQLite 读写与 legacy bootstrap 兼容**

```rust
async fn insert_trigger_ledger_entry<'e, E>(
    executor: E,
    entry: &StoredTriggerLedgerEntry,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query(
        r#"
        INSERT INTO reflection_trigger_ledger (
            ledger_id, trigger_type, namespace, trigger_key, status,
            evidence_window, handled_at, cooldown_until, episode_watermark, reflection_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&entry.ledger_id)
    .bind(entry.trigger_type.as_str())
    .bind(entry.namespace.as_str())
    .bind(&entry.trigger_key)
    .bind(entry.status.as_str())
    .bind(serialize_json(&entry.evidence_window)?)
    .bind(entry.handled_at.map(|value| value.to_rfc3339()))
    .bind(entry.cooldown_until.map(|value| value.to_rfc3339()))
    .bind(entry.episode_watermark.map(|value| value as i64))
    .bind(&entry.reflection_id)
    .execute(executor)
    .await?;
    Ok(())
}
```

- [x] **Step 5: 运行绿测，确认 ledger 迁移、落盘和读取通过**

Run: `cargo test --test sqlite_store -v`

Expected: PASS including new ledger store coverage

### Task 3: 实现自动反思协调器与服务端治理

**Files:**
- Create: `src/application/auto_reflect_if_needed.rs`
- Modify: `src/application/mod.rs`
- Modify: `src/application/build_self_snapshot.rs`
- Modify: `src/application/run_reflection.rs`
- Modify: `src/ports/mod.rs`
- Modify: `tests/application_use_cases.rs`
- Modify: `tests/failure_modes.rs`

- [x] **Step 1: 写失败测试，固定 conflict / failure / periodic 的最小闭环**

```rust
#[tokio::test]
async fn auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger() {
    let deps = test_support::auto_reflection_deps();
    deps.seed_failure_window(vec![
        "rollback after violating a hard commitment",
        "second rollback after violating the same hard commitment",
    ]);

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(result.triggered);
    assert_eq!(result.trigger_type, Some(TriggerType::Failure));
    assert_eq!(deps.reflections().len(), 1);
    assert_eq!(deps.latest_trigger_status(), Some(TriggerLedgerStatus::Handled));
}
```

- [x] **Step 2: 运行红测，确认协调器、trigger 评估和 proposal 校验尚不存在**

Run: `cargo test --test application_use_cases auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger -v`

Expected: FAIL with missing `auto_reflect_if_needed` module and supporting types

- [x] **Step 3: 实现自动触发评估、证据收集和 ledger 去重**

```rust
pub async fn execute<D>(deps: &D, input: AutoReflectInput) -> Result<AutoReflectResult, AppError>
where
    D: TriggerLedgerStore
        + EventStore
        + ClaimStore
        + CommitmentStore
        + IdentityStore
        + EpisodeStore
        + ReflectionTransactionRunner
        + ModelPort
        + Clock
        + IdGenerator
        + Sync,
{
    if input.recursion_guard == RecursionGuard::SkipAutoReflection {
        return Ok(AutoReflectResult::skipped("recursion guard enabled"));
    }

    let candidate = detect_trigger_candidate(deps, &input).await?;
    if !candidate.should_consider {
        return Ok(AutoReflectResult::not_triggered());
    }

    if trigger_is_suppressed(deps, &candidate).await? {
        record_suppressed_trigger(deps, &candidate).await?;
        return Ok(AutoReflectResult::suppressed(candidate.trigger_type));
    }

    let snapshot = build_revision_snapshot(deps, &candidate).await?;
    let proposal = deps
        .propose_self_revision(SelfRevisionRequest::from_candidate(
            &candidate,
            snapshot,
        ))
        .await?;
    let validated = validate_self_revision(deps, &candidate, &proposal).await?;
    apply_validated_self_revision(deps, candidate, proposal, validated).await
}
```

- [x] **Step 4: 把“受治理的全 patch”转译成现有 `run_reflection` 输入**

```rust
let reflection_input = ReflectionInput::new(
    Reflection::new(proposal.rationale.clone()),
    supersede_claim_id,
    replacement_claim,
    resolved_evidence_event_ids,
)
.with_replacement_evidence_query(optional_query)
.with_identity_update(validated_identity_claims)
.with_commitment_updates(validated_commitments);
```

- [x] **Step 5: 加入 identity 慢更新三重制动器**

```rust
fn validate_identity_patch(
    proposal: &SelfRevisionProposal,
    context: &IdentityRevisionContext,
) -> Result<Vec<String>, AppError> {
    ensure_min_supporting_claims(context, 3)?;
    ensure_cross_episode_support(context, 2)?;
    ensure_no_high_conflict(context)?;
    ensure_identity_cooldown_elapsed(context)?;
    ensure_identity_patch_limit(context, 2)?;
    Ok(materialize_identity_claims(proposal, context))
}
```

- [x] **Step 6: 运行绿测，确认自动反思路径通过且失败模式被锁住**

Run: `cargo test --test application_use_cases --test failure_modes -v`

Expected: PASS with new auto-reflection and rejection-path regressions

### Task 4: 串入 MCP DTO / Runtime，并加防递归保护

**Files:**
- Modify: `src/interfaces/mcp/dto.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/support/doctor.rs`
- Modify: `tests/mcp_stdio.rs`
- Modify: `tests/bootstrap.rs`

- [x] **Step 1: 写失败测试，固定 MCP 入口上的自动触发与防递归行为**

```rust
#[tokio::test]
async fn ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection() {
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_database().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "rollback after violating a hard commitment"
                },
                "claim_drafts": [],
                "episode_reference": "episode:auto-reflect-1",
                "trigger_hints": ["failure", "rollback"]
            }),
        )
        .await
        .unwrap();

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reflections"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(reflection_count, 1);
}
```

- [x] **Step 2: 运行红测，确认 DTO 与 MCP runtime 尚未支持 trigger hints / auto reflection**

Run: `cargo test --test mcp_stdio ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection -v`

Expected: FAIL with unknown `trigger_hints` field or missing auto-reflection behavior

- [x] **Step 3: 给 DTO 增加可选 trigger hints 与内部 recursion guard**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IngestInteractionParams {
    pub event: EventDto,
    pub claim_drafts: Vec<ClaimDraftDto>,
    pub episode_reference: Option<String>,
    #[serde(default)]
    pub trigger_hints: Vec<String>,
}
```

- [x] **Step 4: 在 MCP runtime 中统一串入自动反思检查**

```rust
let auto_reflect_result = auto_reflect_if_needed::execute(
    &self.runtime,
    AutoReflectInput::from_ingest(&params).with_recursion_guard(RecursionGuard::Allow),
)
.await
.map_err(app_error_to_mcp)?;
```

- [x] **Step 5: 运行绿测，确认 stdio 工具链和 bootstrap 仍可工作**

Run: `cargo test --test mcp_stdio --test bootstrap -v`

Expected: PASS and stdio tool list remains stable

### Task 5: 更新文档、测试指南和能力边界说明

**Files:**
- Modify: `README.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `docs/local-mcp-integration-2026-03-26.md`
- Modify: `docs/document-map.md`

- [x] **Step 1: 先写文档期望，明确新增的是“自动 self-revision MVP”，不是完整自治系统**

```md
- 已实现：trigger-ledger backed automatic self-revision MVP
- 部分实现：model-generated revision proposal under server governance
- 未实现：richer evidence weighting, full multi-layer memory, autonomous daemon behavior
```

- [x] **Step 2: 更新 README / project-status / roadmap 的能力边界**

```md
当前版本可在 MCP 请求进入时自动检测 `conflict / failure / periodic` 触发，
并在 trigger ledger、证据门槛与慢更新约束保护下生成最小 self-revision proposal；
但它仍然是本地 `stdio` memory demo，不是完整自治代理系统。
```

- [x] **Step 3: 更新 testing-guide 与 local integration 文档**

Run:

```bash
cargo test --test application_use_cases auto_reflection_runs_once_for_repeated_failure_and_records_handled_ledger -v
cargo test --test sqlite_store sqlite_trigger_ledger_records_namespace_periodic_watermark_and_cooldown -v
cargo test --test mcp_stdio ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection -v
```

Expected: 文档中的命令、字段和行为描述与实现一致

### Task 6: 最终验证与收尾

**Files:**
- Review: `src/application/auto_reflect_if_needed.rs`
- Review: `src/interfaces/mcp/server.rs`
- Review: `src/adapters/sqlite/store.rs`
- Review: `tests/application_use_cases.rs`
- Review: `tests/sqlite_store.rs`
- Review: `tests/mcp_stdio.rs`
- Review: `README.md`
- Review: `docs/project-status.md`

- [x] **Step 1: 运行全量测试**

Run: `cargo test`

Expected: PASS with all existing and新增 tests green

- [x] **Step 2: 运行预检，确认 runtime 仍然可启动**

Run: `./scripts/agent-llm-mm.sh doctor`

Expected: JSON output with `status = ok`

- [x] **Step 3: 复核需求覆盖**

```text
- automatic conflict/failure/periodic trigger
- persistent trigger ledger with de-duplication
- model rationale + machine patch proposal
- governed full patch translation into run_reflection
- identity cooldown + evidence threshold + patch size guard
- docs updated to MVP/demo positioning
```

- [x] **Step 4: 仅保留真实剩余风险**

```text
- summary 文本规则与模型补位的误触发边界仍是 MVP 级别
- periodic 仍是 namespace + interval + threshold 的固定策略，不是可编程 policy
- user/<id> 对 self identity 的影响仍只应作为间接证据，不应扩展成直接人格写入
```
