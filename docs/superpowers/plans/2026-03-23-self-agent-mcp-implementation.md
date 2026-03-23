# Self Agent MCP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个基于 Rust 的单实例 MCP Server，落地最小可行自我机制运行时，具备事件入账、命题提取、快照组装、承诺门控和反思修订能力。

**Architecture:** 采用 `Functional Core + Imperative Shell`。纯领域规则位于 `domain`，应用编排位于 `application`，副作用边界通过 `ports` trait 抽象，SQLite、Mock Model 与 MCP server 位于 `adapters` 和 `interfaces`。MCP 首版使用 `rmcp` + stdio transport，保持实现最小且便于本地联调。

**Tech Stack:** Rust, Tokio, RMCP (`rmcp`), SQLite (`sqlx` + sqlite), Serde, Chrono, UUID, Tracing, Tempfile

---

## Planned File Map

- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/error.rs`
- Create: `src/support/mod.rs`
- Create: `src/support/config.rs`
- Create: `src/support/tracing.rs`
- Create: `src/domain/mod.rs`
- Create: `src/domain/types.rs`
- Create: `src/domain/event.rs`
- Create: `src/domain/claim.rs`
- Create: `src/domain/evidence_link.rs`
- Create: `src/domain/identity_core.rs`
- Create: `src/domain/commitment.rs`
- Create: `src/domain/episode.rs`
- Create: `src/domain/reflection.rs`
- Create: `src/domain/snapshot.rs`
- Create: `src/domain/rules/mod.rs`
- Create: `src/domain/rules/ingest.rs`
- Create: `src/domain/rules/conflict.rs`
- Create: `src/domain/rules/snapshot_builder.rs`
- Create: `src/domain/rules/commitment_gate.rs`
- Create: `src/domain/rules/reflection_policy.rs`
- Create: `src/ports/mod.rs`
- Create: `src/ports/event_store.rs`
- Create: `src/ports/claim_store.rs`
- Create: `src/ports/identity_store.rs`
- Create: `src/ports/commitment_store.rs`
- Create: `src/ports/episode_store.rs`
- Create: `src/ports/reflection_store.rs`
- Create: `src/ports/model_port.rs`
- Create: `src/ports/clock.rs`
- Create: `src/ports/id_generator.rs`
- Create: `src/application/mod.rs`
- Create: `src/application/ingest_interaction.rs`
- Create: `src/application/build_self_snapshot.rs`
- Create: `src/application/decide_with_snapshot.rs`
- Create: `src/application/run_reflection.rs`
- Create: `src/adapters/mod.rs`
- Create: `src/adapters/sqlite/mod.rs`
- Create: `src/adapters/sqlite/schema.rs`
- Create: `src/adapters/sqlite/store.rs`
- Create: `src/adapters/model/mod.rs`
- Create: `src/adapters/model/mock.rs`
- Create: `src/interfaces/mod.rs`
- Create: `src/interfaces/mcp/mod.rs`
- Create: `src/interfaces/mcp/dto.rs`
- Create: `src/interfaces/mcp/server.rs`
- Create: `tests/bootstrap.rs`
- Create: `tests/domain_invariants.rs`
- Create: `tests/domain_snapshot.rs`
- Create: `tests/application_use_cases.rs`
- Create: `tests/sqlite_store.rs`
- Create: `tests/decision_flow.rs`
- Create: `tests/mcp_stdio.rs`
- Create: `tests/failure_modes.rs`

**Responsibilities:**

- `src/domain/*`：纯数据结构与纯规则函数，不直接依赖数据库、MCP 或模型。
- `src/application/*`：用例编排，按顺序串联“读-算-写”动作。
- `src/ports/*`：trait 定义，隔离所有副作用。
- `src/adapters/sqlite/*`：SQLite schema 与具体存储实现。
- `src/adapters/model/*`：模型端口默认 mock 实现。
- `src/interfaces/mcp/*`：MCP tools 输入输出和 server 装配。
- `tests/*`：围绕 TDD、不变量和失败模式的验证。

### Task 1: Bootstrap The Rust Crate

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/error.rs`
- Create: `src/support/mod.rs`
- Create: `src/support/config.rs`
- Create: `src/support/tracing.rs`
- Test: `tests/bootstrap.rs`

- [ ] **Step 1: Write the failing bootstrap test**

```rust
use agent_llm_mm::support::config::{AppConfig, TransportKind};

#[test]
fn default_config_uses_stdio_transport() {
    let config = AppConfig::default();
    assert_eq!(config.transport, TransportKind::Stdio);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test bootstrap`
Expected: FAIL with crate/module resolution errors because Rust crate and support modules do not exist yet.

- [ ] **Step 3: Initialize the crate and minimal runtime shell**

```toml
[package]
name = "agent_llm_mm"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
rmcp = "0.5"
schemars = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
uuid = { version = "1", features = ["serde", "v4"] }

[dev-dependencies]
tempfile = "3"
```

```rust
// src/support/config.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Stdio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub transport: TransportKind,
    pub database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transport: TransportKind::Stdio,
            database_url: "sqlite::memory:".to_string(),
        }
    }
}
```

- [ ] **Step 4: Run bootstrap checks**

Run: `cargo test --test bootstrap`
Expected: PASS

Run: `cargo check`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml .gitignore src/lib.rs src/main.rs src/error.rs src/support/mod.rs src/support/config.rs src/support/tracing.rs tests/bootstrap.rs
git commit -m "chore: bootstrap rust mcp crate"
```

### Task 2: Define Core Domain Models And Invariants

**Files:**
- Create: `src/domain/mod.rs`
- Create: `src/domain/types.rs`
- Create: `src/domain/event.rs`
- Create: `src/domain/claim.rs`
- Create: `src/domain/evidence_link.rs`
- Create: `src/domain/identity_core.rs`
- Create: `src/domain/commitment.rs`
- Create: `src/domain/episode.rs`
- Create: `src/domain/reflection.rs`
- Test: `tests/domain_invariants.rs`

- [ ] **Step 1: Write the failing domain invariant tests**

```rust
use agent_llm_mm::domain::{
    claim::ClaimDraft,
    types::{Mode, Owner},
};

#[test]
fn inferred_claim_requires_external_evidence() {
    let draft = ClaimDraft::new_inferred(Owner::Self_, "self.role", "is", "architect");
    assert!(draft.validate(0).is_err());
}

#[test]
fn identity_core_updates_are_not_allowed_from_ingest_mode() {
    let result = agent_llm_mm::domain::identity_core::allow_direct_ingest_update(Mode::Said);
    assert!(!result);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test domain_invariants`
Expected: FAIL with missing domain modules and missing validation functions.

- [ ] **Step 3: Implement immutable domain objects and invariant helpers**

```rust
// src/domain/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Owner {
    Self_,
    User,
    World,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    Observed,
    Said,
    Acted,
    Inferred,
    Draft,
}
```

```rust
// src/domain/claim.rs
impl ClaimDraft {
    pub fn validate(&self, evidence_count: usize) -> Result<(), DomainError> {
        if self.mode == Mode::Inferred && evidence_count == 0 {
            return Err(DomainError::InsufficientEvidence);
        }
        Ok(())
    }
}
```

```rust
// src/domain/identity_core.rs
pub fn allow_direct_ingest_update(mode: Mode) -> bool {
    matches!(mode, Mode::Draft) && false
}
```

- [ ] **Step 4: Run domain invariant tests**

Run: `cargo test --test domain_invariants`
Expected: PASS

Run: `cargo test domain:: --lib`
Expected: PASS for current domain unit tests

- [ ] **Step 5: Commit**

```bash
git add src/domain/mod.rs src/domain/types.rs src/domain/event.rs src/domain/claim.rs src/domain/evidence_link.rs src/domain/identity_core.rs src/domain/commitment.rs src/domain/episode.rs src/domain/reflection.rs tests/domain_invariants.rs
git commit -m "feat: add core domain models and invariants"
```

### Task 3: Implement Snapshot Assembly And Commitment Gate

**Files:**
- Create: `src/domain/snapshot.rs`
- Create: `src/domain/rules/mod.rs`
- Create: `src/domain/rules/snapshot_builder.rs`
- Create: `src/domain/rules/commitment_gate.rs`
- Create: `src/domain/rules/conflict.rs`
- Test: `tests/domain_snapshot.rs`

- [ ] **Step 1: Write the failing snapshot and gate tests**

```rust
use agent_llm_mm::domain::{
    rules::{commitment_gate::gate_decision, snapshot_builder::build_snapshot},
    snapshot::SnapshotRequest,
};

#[test]
fn snapshot_summary_includes_supporting_event_reference() {
    let snapshot = build_snapshot(SnapshotRequest::fixture_minimal()).unwrap();
    assert!(!snapshot.evidence.is_empty());
}

#[test]
fn hard_commitment_blocks_conflicting_action() {
    let result = gate_decision("write_identity_core_directly", &SnapshotRequest::fixture_minimal().commitments);
    assert!(result.blocked);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test domain_snapshot`
Expected: FAIL with missing snapshot types and rule functions.

- [ ] **Step 3: Implement pure snapshot and gate functions**

```rust
// src/domain/snapshot.rs
pub struct SelfSnapshot {
    pub identity: Vec<String>,
    pub commitments: Vec<String>,
    pub claims: Vec<String>,
    pub evidence: Vec<String>,
    pub episodes: Vec<String>,
}
```

```rust
// src/domain/rules/snapshot_builder.rs
pub fn build_snapshot(input: SnapshotRequest) -> Result<SelfSnapshot, DomainError> {
    let evidence = input
        .evidence
        .iter()
        .take(input.budget.max(1))
        .cloned()
        .collect::<Vec<_>>();

    if evidence.is_empty() {
        return Err(DomainError::SnapshotNeedsEvidence);
    }

    Ok(SelfSnapshot {
        identity: input.identity,
        commitments: input.commitments,
        claims: input.claims,
        evidence,
        episodes: input.episodes,
    })
}
```

```rust
// src/domain/rules/commitment_gate.rs
pub fn gate_decision(action: &str, commitments: &[String]) -> GateResult {
    let blocked = commitments.iter().any(|rule| rule == "forbid:write_identity_core_directly")
        && action == "write_identity_core_directly";
    GateResult { blocked }
}
```

- [ ] **Step 4: Run snapshot and gate tests**

Run: `cargo test --test domain_snapshot`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/domain/snapshot.rs src/domain/rules/mod.rs src/domain/rules/snapshot_builder.rs src/domain/rules/commitment_gate.rs src/domain/rules/conflict.rs tests/domain_snapshot.rs
git commit -m "feat: add snapshot assembly and commitment gate"
```

### Task 4: Add Port Traits And Application Use Cases

**Files:**
- Create: `src/ports/mod.rs`
- Create: `src/ports/event_store.rs`
- Create: `src/ports/claim_store.rs`
- Create: `src/ports/identity_store.rs`
- Create: `src/ports/commitment_store.rs`
- Create: `src/ports/episode_store.rs`
- Create: `src/ports/reflection_store.rs`
- Create: `src/ports/model_port.rs`
- Create: `src/ports/clock.rs`
- Create: `src/ports/id_generator.rs`
- Create: `src/application/mod.rs`
- Create: `src/application/ingest_interaction.rs`
- Create: `src/application/build_self_snapshot.rs`
- Create: `src/application/decide_with_snapshot.rs`
- Create: `src/application/run_reflection.rs`
- Test: `tests/application_use_cases.rs`

- [ ] **Step 1: Write the failing application tests with in-memory fakes**

```rust
#[tokio::test]
async fn ingest_writes_events_before_claims() {
    let deps = test_support::in_memory_deps();
    let result = agent_llm_mm::application::ingest_interaction::execute(&deps, test_support::ingest_input()).await;
    assert!(result.is_ok());
    assert_eq!(deps.log(), vec!["append_event", "upsert_claim", "link_evidence"]);
}

#[tokio::test]
async fn reflection_supersedes_old_claim_instead_of_deleting_it() {
    let deps = test_support::in_memory_deps();
    let result = agent_llm_mm::application::run_reflection::execute(&deps, test_support::reflection_input()).await;
    assert!(result.is_ok());
    assert!(deps.was_status_updated_to("superseded"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test application_use_cases`
Expected: FAIL with missing ports, use case modules, and async orchestration functions.

- [ ] **Step 3: Implement trait-based ports and orchestration functions**

```rust
// src/ports/event_store.rs
#[async_trait::async_trait]
pub trait EventStore {
    async fn append_event(&self, event: Event) -> Result<(), AppError>;
}
```

```rust
// src/application/ingest_interaction.rs
pub async fn execute<D>(deps: &D, input: IngestInput) -> Result<IngestResult, AppError>
where
    D: EventStore + ClaimStore + EpisodeStore + IdGenerator + Clock + Sync,
{
    let event = input.into_event(deps.next_id().await?, deps.now().await?);
    deps.append_event(event.clone()).await?;
    for claim in derive_claims(&event)? {
        deps.upsert_claim(claim.clone()).await?;
        deps.link_evidence(claim.claim_id.clone(), event.event_id.clone()).await?;
    }
    Ok(IngestResult::from_event(event))
}
```

- [ ] **Step 4: Run the application tests**

Run: `cargo test --test application_use_cases`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ports/mod.rs src/ports/event_store.rs src/ports/claim_store.rs src/ports/identity_store.rs src/ports/commitment_store.rs src/ports/episode_store.rs src/ports/reflection_store.rs src/ports/model_port.rs src/ports/clock.rs src/ports/id_generator.rs src/application/mod.rs src/application/ingest_interaction.rs src/application/build_self_snapshot.rs src/application/decide_with_snapshot.rs src/application/run_reflection.rs tests/application_use_cases.rs
git commit -m "feat: add application use cases and ports"
```

### Task 5: Implement SQLite Schema And Store Adapter

**Files:**
- Create: `src/adapters/mod.rs`
- Create: `src/adapters/sqlite/mod.rs`
- Create: `src/adapters/sqlite/schema.rs`
- Create: `src/adapters/sqlite/store.rs`
- Test: `tests/sqlite_store.rs`

- [ ] **Step 1: Write the failing SQLite adapter tests**

```rust
#[tokio::test]
async fn sqlite_store_bootstraps_all_tables() {
    let store = test_support::new_sqlite_store().await;
    let tables = store.list_tables().await.unwrap();
    assert!(tables.contains(&"events".to_string()));
    assert!(tables.contains(&"reflections".to_string()));
}

#[tokio::test]
async fn sqlite_round_trips_claim_with_evidence() {
    let store = test_support::new_sqlite_store().await;
    let ids = test_support::seed_event_and_claim(&store).await.unwrap();
    let loaded = store.load_claim_with_evidence(&ids.claim_id).await.unwrap();
    assert_eq!(loaded.evidence.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test sqlite_store`
Expected: FAIL with missing SQLite store and schema bootstrap code.

- [ ] **Step 3: Implement schema bootstrap and store methods**

```rust
// src/adapters/sqlite/schema.rs
pub const INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    event_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    ts TEXT NOT NULL,
    session_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    owner TEXT NOT NULL,
    mode TEXT NOT NULL,
    content TEXT NOT NULL,
    source_ref TEXT,
    hash TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS claims (
    claim_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    subject TEXT NOT NULL,
    predicate TEXT NOT NULL,
    object TEXT NOT NULL,
    kind TEXT NOT NULL,
    confidence REAL NOT NULL,
    stability TEXT NOT NULL,
    status TEXT NOT NULL,
    valid_from TEXT,
    valid_to TEXT
);
"#;
```

```rust
// src/adapters/sqlite/store.rs
pub struct SqliteStore {
    pool: sqlx::SqlitePool,
}

impl SqliteStore {
    pub async fn bootstrap(database_url: &str) -> Result<Self, AppError> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;
        for statement in INIT_SQL.split(';').filter(|part| !part.trim().is_empty()) {
            sqlx::query(statement).execute(&pool).await?;
        }
        Ok(Self { pool })
    }
}
```

- [ ] **Step 4: Run the SQLite adapter tests**

Run: `cargo test --test sqlite_store`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/adapters/mod.rs src/adapters/sqlite/mod.rs src/adapters/sqlite/schema.rs src/adapters/sqlite/store.rs tests/sqlite_store.rs
git commit -m "feat: add sqlite storage adapter"
```

### Task 6: Add Mock Model And Decision Flow

**Files:**
- Create: `src/adapters/model/mod.rs`
- Create: `src/adapters/model/mock.rs`
- Test: `tests/decision_flow.rs`
- Modify: `src/application/decide_with_snapshot.rs`
- Modify: `src/ports/model_port.rs`

- [ ] **Step 1: Write the failing decision tests**

```rust
#[tokio::test]
async fn decision_returns_blocked_without_calling_model_when_gate_fails() {
    let deps = test_support::deps_with_blocking_commitment();
    let result = agent_llm_mm::application::decide_with_snapshot::execute(&deps, test_support::decision_input()).await.unwrap();
    assert!(result.blocked);
    assert_eq!(deps.model_call_count(), 0);
}

#[tokio::test]
async fn mock_model_receives_snapshot_context_when_gate_passes() {
    let deps = test_support::deps_with_mock_model();
    let result = agent_llm_mm::application::decide_with_snapshot::execute(&deps, test_support::decision_input()).await.unwrap();
    assert_eq!(result.action, "summarize_memory_state");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test decision_flow`
Expected: FAIL with missing model adapter behavior and incomplete decision use case.

- [ ] **Step 3: Implement model port and mock adapter**

```rust
// src/ports/model_port.rs
#[async_trait::async_trait]
pub trait ModelPort {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError>;
}
```

```rust
// src/adapters/model/mock.rs
pub struct MockModel;

#[async_trait::async_trait]
impl ModelPort for MockModel {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        Ok(ModelDecision {
            action: if request.snapshot.claims.is_empty() {
                "request_more_context".to_string()
            } else {
                "summarize_memory_state".to_string()
            },
            rationale: "mocked".to_string(),
        })
    }
}
```

- [ ] **Step 4: Run the decision flow tests**

Run: `cargo test --test decision_flow`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/adapters/model/mod.rs src/adapters/model/mock.rs src/application/decide_with_snapshot.rs src/ports/model_port.rs tests/decision_flow.rs
git commit -m "feat: add mock model decision flow"
```

### Task 7: Expose MCP Tools Over Stdio

**Files:**
- Create: `src/interfaces/mod.rs`
- Create: `src/interfaces/mcp/mod.rs`
- Create: `src/interfaces/mcp/dto.rs`
- Create: `src/interfaces/mcp/server.rs`
- Modify: `src/main.rs`
- Test: `tests/mcp_stdio.rs`

- [ ] **Step 1: Write the failing MCP stdio integration test**

```rust
#[tokio::test]
async fn server_exposes_expected_tools_over_stdio() {
    let client = test_support::spawn_stdio_client().await.unwrap();
    let tools = client.list_all_tools().await.unwrap();
    let names = tools.into_iter().map(|tool| tool.name.to_string()).collect::<Vec<_>>();
    assert!(names.contains(&"ingest_interaction".to_string()));
    assert!(names.contains(&"build_self_snapshot".to_string()));
    assert!(names.contains(&"decide_with_snapshot".to_string()));
    assert!(names.contains(&"run_reflection".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test mcp_stdio`
Expected: FAIL with missing RMCP server implementation and no stdio bootstrap.

- [ ] **Step 3: Implement RMCP server and tool DTO mapping**

```rust
// src/interfaces/mcp/server.rs
#[derive(Clone)]
pub struct SelfAgentServer {
    app: Arc<AppRuntime>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<SelfAgentServer>,
}

#[rmcp::tool_router]
impl SelfAgentServer {
    pub fn new(app: Arc<AppRuntime>) -> Self {
        Self {
            app,
            tool_router: Self::tool_router(),
        }
    }

    #[rmcp::tool(description = "Append interaction events and derive claims")]
    async fn ingest_interaction(&self, input: IngestInteractionInput) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let result = self.app.ingest(input).await.map_err(map_error)?;
        Ok(ok_json_result(result))
    }
}
```

```rust
// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::default();
    let runtime = Arc::new(build_runtime(&config).await?);
    let server = SelfAgentServer::new(runtime);
    server.serve((tokio::io::stdin(), tokio::io::stdout())).await?.waiting().await?;
    Ok(())
}
```

- [ ] **Step 4: Run MCP integration test**

Run: `cargo test --test mcp_stdio -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/interfaces/mod.rs src/interfaces/mcp/mod.rs src/interfaces/mcp/dto.rs src/interfaces/mcp/server.rs src/main.rs tests/mcp_stdio.rs
git commit -m "feat: expose self-agent tools over mcp stdio"
```

### Task 8: Lock Failure Modes And Final Verification

**Files:**
- Create: `tests/failure_modes.rs`
- Modify: `tests/application_use_cases.rs`
- Modify: `tests/mcp_stdio.rs`
- Modify: `src/domain/rules/reflection_policy.rs`
- Modify: `src/application/run_reflection.rs`

- [ ] **Step 1: Write the failing failure-mode regression tests**

```rust
#[tokio::test]
async fn inferred_claim_without_evidence_is_rejected() {
    let deps = test_support::deps_for_failure_modes();
    let result = agent_llm_mm::application::ingest_interaction::execute(&deps, test_support::inferred_without_evidence()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn reflection_marks_conflict_as_disputed_instead_of_deleting_history() {
    let deps = test_support::deps_for_failure_modes();
    agent_llm_mm::application::run_reflection::execute(&deps, test_support::conflicting_reflection()).await.unwrap();
    assert!(deps.history_contains_status("disputed"));
}

#[tokio::test]
async fn snapshot_budget_prevents_recent_event_hijack() {
    let deps = test_support::deps_for_failure_modes();
    let snapshot = agent_llm_mm::application::build_self_snapshot::execute(&deps, test_support::budgeted_snapshot()).await.unwrap();
    assert!(snapshot.evidence.len() <= 3);
}
```

- [ ] **Step 2: Run the regression tests to verify they fail**

Run: `cargo test --test failure_modes`
Expected: FAIL because reflection policy and snapshot budgeting are not strict enough yet.

- [ ] **Step 3: Tighten reflection policy and finish end-to-end invariants**

```rust
// src/domain/rules/reflection_policy.rs
pub fn classify_reflection(trigger: ReflectionTrigger) -> ReflectionDecision {
    match trigger {
        ReflectionTrigger::Conflict => ReflectionDecision::MarkDisputed,
        ReflectionTrigger::Failure => ReflectionDecision::SupersedeWithReplacement,
        ReflectionTrigger::Manual => ReflectionDecision::RecordOnly,
    }
}
```

```rust
// src/application/run_reflection.rs
if matches!(decision, ReflectionDecision::MarkDisputed) {
    deps.mark_claim_status(input.claim_id.clone(), Status::Disputed).await?;
}
```

- [ ] **Step 4: Run full verification**

Run: `cargo fmt --check`
Expected: PASS

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/failure_modes.rs tests/application_use_cases.rs tests/mcp_stdio.rs src/domain/rules/reflection_policy.rs src/application/run_reflection.rs
git commit -m "test: lock failure mode regressions"
```

## Notes For Execution

- 保持 `domain` 纯净，不要把 `sqlx`, `rmcp`, `tokio::process` 等基础设施类型漏进领域层。
- `identity_core` 的更新必须通过 `run_reflection` 路径完成。
- `events` 必须先写入，再派生 `claims` 与 `episodes`。
- `build_self_snapshot` 必须回拉至少一条原始证据，避免高层摘要漂移。
- `decide_with_snapshot` 必须先执行 `commitment_gate`，阻断时不可调用模型端口。
- 首版坚持 stdio transport，不要在本计划内扩展 streamable HTTP。
- 如果实现中发现某个文件开始承担多个职责，优先在当前任务内拆开，而不是累积技术债。
