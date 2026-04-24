# Self-Revision Runtime Coverage And Governance Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在已落地的 automatic self-revision MVP 基础上，扩展 MCP runtime coverage、补强可观测性与治理诊断，并继续收口 deeper-update 契约，同时继续把 `run_reflection` 保持为唯一长期层写路径。

**Architecture:** 继续复用现有 `auto_reflect_if_needed -> run_reflection` 路径，不新增旁路持久化接口。MCP runtime 只在明确入口上做 best-effort auto-reflection，并通过更清晰的 DTO、结果诊断和 `doctor`/测试基线把运行边界说清楚。对 deeper-update 只补强 proposal/evidence/policy 契约与服务端校验，不引入 richer autonomous daemon 行为。

**Tech Stack:** Rust 2024, Tokio, RMCP stdio, SQLite, SQLx, Serde, Tracing, Reqwest, existing mock/openai-compatible model adapters

---

### Task 1: 扩大 `decide_with_snapshot` 的 conflict runtime coverage

**Files:**
- Modify: `src/interfaces/mcp/dto.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Test: `tests/mcp_stdio.rs`

- [x] **Step 1: 写失败测试，固定 `decide_with_snapshot` 的 conflict 自动触发边界**

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"conflict-backed commitment tightening","machine_patch":{"commitment_patch":{"commitments":["prefer:reflect_before_overwriting_commitments"]}}}"#
                }
            }]
        }),
    )
    .await;
    let config = test_support::openai_config_for_stub(&stub);
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let snapshot = test_support::seed_snapshot_via_stdio(&mut client).await;
    let result = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "resolve a conflicting commitment update",
                "action": "overwrite_commitment",
                "snapshot": snapshot,
                "auto_reflect_namespace": "self",
                "trigger_hints": ["conflict", "commitment"]
            }),
        )
        .await
        .unwrap();

    let blocked = result["result"]["structuredContent"]["blocked"]
        .as_bool()
        .unwrap();
    assert!(!blocked);

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reflection_count, 1);
}
```

- [x] **Step 2: 运行红测，确认当前 DTO / runtime 还没接 `decide_with_snapshot` 的 auto-reflection**

Run: `cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v`
Expected: FAIL with unknown `auto_reflect_namespace` / `trigger_hints` field or missing runtime behavior

- [x] **Step 3: 给 `DecideWithSnapshotParams` 增加显式 runtime coverage 字段**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DecideWithSnapshotParams {
    pub task: String,
    pub action: String,
    pub snapshot: SelfSnapshotDto,
    #[serde(default)]
    pub trigger_hints: Vec<String>,
    #[serde(default)]
    pub auto_reflect_namespace: Option<String>,
}

impl DecideWithSnapshotParams {
    fn auto_reflect_namespace(&self) -> Result<Option<Namespace>, AppError> {
        self.auto_reflect_namespace
            .as_ref()
            .map(|value| Namespace::parse(value.to_string()).map_err(AppError::from))
            .transpose()
    }
}

impl AutoReflectInput {
    pub fn from_decide(params: &DecideWithSnapshotParams) -> Result<Option<Self>, AppError> {
        params.auto_reflect_namespace()?.map(|namespace| {
            Ok(Self::for_conflict(namespace, params.trigger_hints.clone()))
        }).transpose()
    }
}
```

- [x] **Step 4: 在 stdio runtime 的 `decide_with_snapshot` 路径里做 best-effort conflict 自动触发**

```rust
async fn decide_with_snapshot(
    &self,
    Parameters(params): Parameters<DecideWithSnapshotParams>,
) -> Result<CallToolResult, McpError> {
    let auto_reflect_input =
        AutoReflectInput::from_decide(&params).map_err(app_error_to_mcp)?;
    let result = decide_with_snapshot::execute(&self.runtime, params.into())
        .await
        .map_err(app_error_to_mcp)?;

    if let Some(auto_reflect_input) = auto_reflect_input {
        if let Err(error) = auto_reflect_if_needed::execute(
            &self.runtime,
            auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
        )
        .await
        {
            warn!(error = %error, "best-effort conflict auto-reflection failed after decide_with_snapshot");
        }
    }

    structured(result)
}
```

- [x] **Step 5: 运行绿测，确认 `decide_with_snapshot` 的 runtime coverage 不破坏原结果形状**

Run: `cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v`
Expected: PASS

- [x] **Step 6: 提交**

```bash
git add src/interfaces/mcp/dto.rs src/interfaces/mcp/server.rs tests/mcp_stdio.rs
git commit -m "feat: wire conflict auto-reflection into decide_with_snapshot"
```

### Task 2: 扩大 `build_self_snapshot` 的 periodic runtime coverage

**Files:**
- Modify: `src/interfaces/mcp/dto.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Test: `tests/mcp_stdio.rs`

- [x] **Step 1: 写失败测试，固定 `build_self_snapshot` 的 periodic 自动触发边界**

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"periodic review tightened commitments","machine_patch":{"commitment_patch":{"commitments":["prefer:periodic_reflection_review"]}}}"#
                }
            }]
        }),
    )
    .await;
    let config = test_support::openai_config_for_stub(&stub);
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    test_support::seed_periodic_window_via_stdio(&mut client, 20).await;

    let first = client
        .call_tool(
            "build_self_snapshot",
            json!({
                "budget": 4,
                "auto_reflect_namespace": "project/agent-llm-mm"
            }),
        )
        .await
        .unwrap();
    let second = client
        .call_tool(
            "build_self_snapshot",
            json!({
                "budget": 4,
                "auto_reflect_namespace": "project/agent-llm-mm"
            }),
        )
        .await
        .unwrap();

    assert!(first["result"]["structuredContent"]["snapshot"].is_object());
    assert!(second["result"]["structuredContent"]["snapshot"].is_object());

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reflection_count, 1);
}
```

- [x] **Step 2: 运行红测，确认当前 `build_self_snapshot` 还没有 periodic runtime 接线**

Run: `cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v`
Expected: FAIL with unknown `auto_reflect_namespace` field or missing auto-reflection behavior

- [x] **Step 3: 给 `BuildSelfSnapshotParams` 增加显式 periodic namespace 字段**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BuildSelfSnapshotParams {
    pub budget: usize,
    #[serde(default)]
    pub auto_reflect_namespace: Option<String>,
}

impl BuildSelfSnapshotParams {
    fn auto_reflect_namespace(&self) -> Result<Option<Namespace>, AppError> {
        self.auto_reflect_namespace
            .as_ref()
            .map(|value| Namespace::parse(value.to_string()).map_err(AppError::from))
            .transpose()
    }
}

impl AutoReflectInput {
    pub fn from_build_snapshot(params: &BuildSelfSnapshotParams) -> Result<Option<Self>, AppError> {
        params.auto_reflect_namespace()?.map(|namespace| {
            Ok(Self::for_periodic(namespace, vec!["periodic".to_string()]))
        }).transpose()
    }
}
```

- [x] **Step 4: 在 stdio runtime 的 `build_self_snapshot` 路径里做 best-effort periodic 自动触发**

```rust
async fn build_self_snapshot(
    &self,
    Parameters(params): Parameters<BuildSelfSnapshotParams>,
) -> Result<CallToolResult, McpError> {
    let auto_reflect_input =
        AutoReflectInput::from_build_snapshot(&params).map_err(app_error_to_mcp)?;

    if let Some(auto_reflect_input) = auto_reflect_input {
        if let Err(error) = auto_reflect_if_needed::execute(
            &self.runtime,
            auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
        )
        .await
        {
            warn!(error = %error, "best-effort periodic auto-reflection failed before build_self_snapshot");
        }
    }

    let result = build_self_snapshot::execute(&self.runtime, params.into())
        .await
        .map_err(app_error_to_mcp)?;
    structured(result)
}
```

- [x] **Step 5: 运行绿测，确认 periodic 路径只在显式 namespace 下触发一次**

Run: `cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v`
Expected: PASS

- [x] **Step 6: 提交**

```bash
git add src/interfaces/mcp/dto.rs src/interfaces/mcp/server.rs tests/mcp_stdio.rs
git commit -m "feat: wire periodic auto-reflection into build_self_snapshot"
```

### Task 3: 为 best-effort auto-reflection 增加可观测性与静态能力报告

**Files:**
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `src/support/doctor.rs`
- Modify: `tests/application_use_cases.rs`
- Modify: `tests/failure_modes.rs`
- Modify: `tests/bootstrap.rs`

- [x] **Step 1: 写失败测试，固定 auto-reflection 的诊断字段和静态能力报告**

```rust
#[tokio::test]
async fn auto_reflection_returns_structured_diagnostics_for_suppressed_trigger() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_periodic_cooldown("project/agent-llm-mm:periodic");

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();

    assert_eq!(result.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(result.trigger_key.as_deref(), Some("project/agent-llm-mm:periodic"));
    assert!(result.cooldown_until.is_some());
    assert_eq!(result.suppression_reason.as_deref(), Some("cooldown_active"));
}

#[tokio::test]
async fn doctor_reports_self_revision_runtime_coverage() {
    let config = AppConfig::default();
    let report = run_doctor(config).await.unwrap();

    assert_eq!(
        report.auto_reflection_runtime_hooks,
        vec![
            "ingest_interaction:failure".to_string(),
            "decide_with_snapshot:conflict".to_string(),
            "build_self_snapshot:periodic".to_string(),
        ]
    );
    assert_eq!(report.self_revision_write_path, "run_reflection");
}
```

- [x] **Step 2: 运行红测，确认当前诊断结果和 `doctor` 报告还不够细**

Run: `cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v && cargo test --test bootstrap doctor_reports_self_revision_runtime_coverage -v`
Expected: FAIL with missing fields like `trigger_key` / `suppression_reason` / `auto_reflection_runtime_hooks`

- [x] **Step 3: 扩展 `AutoReflectResult`，把 suppression/rejection/handled 诊断结构化**

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AutoReflectResult {
    pub triggered: bool,
    pub trigger_type: Option<TriggerType>,
    pub reflection_id: Option<String>,
    pub ledger_status: Option<TriggerLedgerStatus>,
    pub reason: Option<String>,
    pub trigger_key: Option<String>,
    pub evidence_event_ids: Vec<String>,
    pub cooldown_until: Option<chrono::DateTime<Utc>>,
    pub suppression_reason: Option<String>,
}
```

- [x] **Step 4: 在 server 的 best-effort runtime 路径里写出结构化诊断日志**

```rust
match auto_reflect_if_needed::execute(
    &self.runtime,
    auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
)
.await
{
    Ok(result) => {
        tracing::info!(
            trigger_type = ?result.trigger_type,
            trigger_key = ?result.trigger_key,
            ledger_status = ?result.ledger_status,
            reflection_id = ?result.reflection_id,
            suppression_reason = ?result.suppression_reason,
            "best-effort auto-reflection finished"
        );
    }
    Err(error) => {
        tracing::warn!(error = %error, "best-effort auto-reflection failed");
    }
}
```

- [x] **Step 5: 扩展 `DoctorReport`，把 runtime coverage 说清楚**

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub transport: TransportKind,
    pub database_url: String,
    pub provider: ModelProviderKind,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub auto_reflection_runtime_hooks: Vec<String>,
    pub self_revision_write_path: &'static str,
    pub status: &'static str,
}
```

- [x] **Step 6: 运行绿测，确认诊断字段和静态能力报告稳定**

Run: `cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v && cargo test --test bootstrap doctor_reports_self_revision_runtime_coverage -v`
Expected: PASS

- [x] **Step 7: 提交**

```bash
git add src/application/auto_reflect_if_needed.rs src/interfaces/mcp/server.rs src/support/doctor.rs tests/application_use_cases.rs tests/failure_modes.rs tests/bootstrap.rs
git commit -m "feat: add self-revision runtime diagnostics"
```

### Task 4: 收口 self-revision 的 evidence / policy 契约

**Files:**
- Modify: `src/domain/self_revision.rs`
- Modify: `src/adapters/model/openai_compatible.rs`
- Modify: `src/application/auto_reflect_if_needed.rs`
- Modify: `tests/openai_compatible_model.rs`
- Modify: `tests/application_use_cases.rs`
- Modify: `tests/failure_modes.rs`

- [x] **Step 1: 写失败测试，固定 proposal 里的 evidence / policy 元数据和服务端校验**

```rust
#[tokio::test]
async fn openai_compatible_model_parses_self_revision_evidence_policy() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"weighted evidence supports identity update","machine_patch":{"identity_patch":{"canonical_claims":["identity:self=mentor"]}},"proposed_evidence_event_ids":["evt-1","evt-2"],"confidence":"medium"}"#
                }
            }]
        }),
    )
    .await;
    let model = test_support::openai_model_for_stub(&stub);

    let proposal = model
        .propose_self_revision(test_support::sample_self_revision_request())
        .await
        .unwrap();

    assert_eq!(proposal.proposed_evidence_event_ids, vec!["evt-1".to_string(), "evt-2".to_string()]);
    assert_eq!(proposal.confidence.as_deref(), Some("medium"));
}

#[tokio::test]
async fn auto_reflection_rejects_model_proposed_evidence_outside_trigger_window() {
    let deps = test_support::auto_reflection_deps();
    deps.set_self_revision_proposal(test_support::proposal_with_out_of_window_evidence());

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(deps.latest_trigger_status(), Some(TriggerLedgerStatus::Rejected));
}
```

- [x] **Step 2: 运行红测，确认当前 proposal 契约还不表达 evidence / policy 元数据**

Run: `cargo test --test openai_compatible_model openai_compatible_model_parses_self_revision_evidence_policy -v && cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_outside_trigger_window -v`
Expected: FAIL with missing `proposed_evidence_event_ids` / `confidence` / validation behavior

- [x] **Step 3: 扩展 `SelfRevisionProposal` 的第一阶段 evidence / policy 字段**

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionProposal {
    pub should_reflect: bool,
    pub rationale: String,
    #[serde(default)]
    pub machine_patch: SelfRevisionPatch,
    #[serde(default)]
    pub proposed_evidence_event_ids: Vec<String>,
    #[serde(default)]
    pub proposed_evidence_query: Option<EvidenceQuery>,
    #[serde(default)]
    pub confidence: Option<String>,
}
```

- [x] **Step 4: 在协调器里只接受“trigger window ∩ proposal evidence candidates”的交集**

```rust
fn resolve_governed_evidence_window(
    candidate_evidence: &[String],
    proposal: &SelfRevisionProposal,
) -> Result<Vec<String>, AppError> {
    if proposal.proposed_evidence_event_ids.is_empty() {
        return Ok(candidate_evidence.to_vec());
    }

    let filtered = proposal
        .proposed_evidence_event_ids
        .iter()
        .filter(|event_id| candidate_evidence.contains(*event_id))
        .cloned()
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        return Err(AppError::InvalidParams(
            "model proposed evidence outside the current trigger window".to_string(),
        ));
    }

    Ok(filtered)
}
```

- [x] **Step 5: 运行绿测，确认 deeper-update 的第一阶段契约稳定**

Run: `cargo test --test openai_compatible_model openai_compatible_model_parses_self_revision_evidence_policy -v && cargo test --test failure_modes auto_reflection_rejects_model_proposed_evidence_outside_trigger_window -v`
Expected: PASS

- [x] **Step 6: 提交**

```bash
git add src/domain/self_revision.rs src/adapters/model/openai_compatible.rs src/application/auto_reflect_if_needed.rs tests/openai_compatible_model.rs tests/application_use_cases.rs tests/failure_modes.rs
git commit -m "feat: tighten self-revision evidence policy contract"
```

### Task 5: 更新 README / status / testing / integration 文档

**Files:**
- Modify: `README.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `docs/local-mcp-integration-2026-03-26.md`
- Modify: `docs/document-map.md`

- [x] **Step 1: 写失败文档检查清单，明确要同步的新边界**

```text
- 当前 MCP-wired automatic paths: ingest->failure, decide->conflict, build_self_snapshot->periodic
- best-effort auto-reflection 会输出可观测的 trigger / suppression / cooldown 诊断
- self-revision proposal 已有第一阶段 evidence / confidence 契约
- 仍然没有新增 MCP tool
- 仍然没有后台 daemon 或完整自治系统
```

- [x] **Step 2: 更新 README / project-status / roadmap 的能力边界与后续路线**

```md
当前版本已经把 automatic self-revision 从 ingest-only failure 路径，
扩展到了明确的 MCP runtime coverage（failure / conflict / periodic），
并补入 trigger / rejection / suppression 的诊断信息；
但它仍然是本地 `stdio` memory demo，不是完整自治代理系统。
```

- [x] **Step 3: 更新 testing-guide 与 local integration 文档的定向验证命令**

```zsh
cargo test --test mcp_stdio decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow -v
cargo test --test mcp_stdio build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace -v
cargo test --test failure_modes auto_reflection_returns_structured_diagnostics_for_suppressed_trigger -v
cargo test --test openai_compatible_model openai_compatible_model_parses_self_revision_evidence_policy -v
```

- [x] **Step 4: 提交**

```bash
git add README.md docs/project-status.md docs/roadmap.md docs/testing-guide-2026-03-24.md docs/local-mcp-integration-2026-03-26.md docs/document-map.md
git commit -m "docs: update self-revision runtime coverage and diagnostics"
```

### Task 6: 最终验证与收尾

**Files:**
- Review: `src/application/auto_reflect_if_needed.rs`
- Review: `src/interfaces/mcp/server.rs`
- Review: `src/interfaces/mcp/dto.rs`
- Review: `src/domain/self_revision.rs`
- Review: `tests/application_use_cases.rs`
- Review: `tests/failure_modes.rs`
- Review: `tests/mcp_stdio.rs`
- Review: `tests/openai_compatible_model.rs`
- Review: `README.md`
- Review: `docs/project-status.md`

- [x] **Step 1: 运行全量测试**

Run: `cargo test`
Expected: PASS with all existing and new tests green

- [x] **Step 2: 运行预检，确认 runtime 仍可启动**

Run: `./scripts/agent-llm-mm.sh doctor`
Expected: JSON output with `status = ok`

- [x] **Step 3: 复核需求覆盖**

```text
- conflict MCP runtime coverage
- periodic MCP runtime coverage
- best-effort auto-reflection diagnostics
- trigger / rejection / suppression observability
- proposal evidence / confidence contract
- run_reflection remains the only durable write path
- docs updated to MVP/demo positioning
```

- [x] **Step 4: 仅保留真实剩余风险**

```text
- evidence weighting / relation / ranking 仍是第一阶段契约，不是完整 policy engine
- conflict / periodic runtime coverage 仍是显式入口接线，不是“所有请求自动反思”
- user/<id> 对 self identity 的影响仍应保持为间接证据，不应扩展成直接人格写入
- 自动修订仍是本地 stdio MVP；没有后台 daemon、长期调度器或自治 worker
```
