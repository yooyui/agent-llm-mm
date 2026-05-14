use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Serialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    adapters::{
        model::{mock::MockModel, openai_compatible::OpenAiCompatibleModel},
        sqlite::SqliteStore,
    },
    application::{
        auto_reflect_if_needed::{self, AutoReflectInput, RecursionGuard},
        build_self_snapshot, decide_with_snapshot, ingest_interaction,
        ingest_interaction::IngestInput,
        run_reflection,
        run_reflection::ReflectionInput,
    },
    domain::identity_core::IdentityCore,
    domain::operation_log::{ActorKind, OperationLogEntry, OperationLogKind, OperationLogStatus},
    domain::self_revision::{
        SELF_REVISION_DURABLE_WRITE_PATH, SelfRevisionProposal, SelfRevisionRequest, TriggerType,
    },
    error::AppError,
    interfaces::dashboard::{
        DashboardHandle, DashboardObserver, DashboardRuntimeInfo, OperationRecorder,
        OperationStatus, start_dashboard_service_with_operation_log,
    },
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, EvidenceQuery,
        IdGenerator, IdentityStore, IngestTransaction, IngestTransactionRunner, ModelDecision,
        ModelDecisionRequest, ModelPort, OperationLogStore, ReflectionStore, ReflectionTransaction,
        ReflectionTransactionRunner, StoredClaim, StoredEvent, StoredReflection,
        StoredTriggerLedgerEntry, TriggerLedgerStatus, TriggerLedgerStore,
    },
    support::config::{AppConfig, ModelConfig, ModelProviderKind, TransportKind},
};

use super::dto::{
    BuildSelfSnapshotParams, DecideWithSnapshotParams, IngestInteractionParams, RunReflectionParams,
};

pub const AUTO_REFLECTION_RUNTIME_HOOKS: [&str; 4] = [
    "ingest_interaction:failure",
    "ingest_interaction:conflict",
    "decide_with_snapshot:conflict",
    "build_self_snapshot:periodic",
];
pub const SELF_REVISION_WRITE_PATH: &str = SELF_REVISION_DURABLE_WRITE_PATH;

pub async fn run_stdio_server() -> Result<()> {
    let config = AppConfig::load().map_err(anyhow::Error::msg)?;
    run_stdio_server_with_config(config).await
}

pub async fn run_stdio_server_with_config(config: AppConfig) -> Result<()> {
    config.validate().map_err(anyhow::Error::msg)?;
    let store = SqliteStore::bootstrap(&config.database_url).await?;
    let (dashboard_observer, _dashboard_handle) =
        start_configured_dashboard(&config, Some(store.clone())).await?;
    let server = Server::from_parts(config, store, dashboard_observer).await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

pub async fn validate_stdio_runtime(config: &AppConfig) -> Result<(), AppError> {
    Runtime::bootstrap(config, DashboardObserver::disabled())
        .await
        .map(|_| ())
}

async fn start_configured_dashboard(
    config: &AppConfig,
    operation_log: Option<SqliteStore>,
) -> Result<(DashboardObserver, Option<DashboardHandle>)> {
    if !config.dashboard.enabled {
        return Ok((DashboardObserver::disabled(), None));
    }

    let recorder = OperationRecorder::new(config.dashboard.event_capacity);
    let runtime = DashboardRuntimeInfo {
        service_name: "agent-llm-mm".to_string(),
        transport: transport_label(config.transport).to_string(),
        provider: provider_label(config.model_provider).to_string(),
        dashboard_enabled: true,
        read_only: true,
    };

    match start_dashboard_service_with_operation_log(
        config.dashboard.clone(),
        recorder.clone(),
        runtime,
        operation_log,
    )
    .await
    {
        Ok(handle) => {
            let observer = DashboardObserver::enabled(recorder);
            observer.record_dashboard_started(&handle.base_url());
            info!(dashboard_url = %handle.base_url(), "dashboard service started");
            Ok((observer, Some(handle)))
        }
        Err(error) if config.dashboard.required => Err(error),
        Err(error) => {
            warn!(error = %error, "dashboard service failed to start; continuing without dashboard");
            Ok((DashboardObserver::disabled(), None))
        }
    }
}

fn transport_label(transport: TransportKind) -> &'static str {
    match transport {
        TransportKind::Stdio => "stdio",
    }
}

fn provider_label(provider: ModelProviderKind) -> &'static str {
    match provider {
        ModelProviderKind::Mock => "mock",
        ModelProviderKind::OpenAiCompatible => "openai-compatible",
    }
}

#[derive(Clone)]
pub struct Server {
    runtime: Runtime,
    tool_router: ToolRouter<Self>,
}

impl Server {
    async fn from_parts(
        config: AppConfig,
        store: SqliteStore,
        dashboard: DashboardObserver,
    ) -> Result<Self, AppError> {
        let runtime = Runtime::from_store(&config, store, dashboard).await?;
        Ok(Self {
            runtime,
            tool_router: Self::tool_router(),
        })
    }
}

#[tool_router]
impl Server {
    #[tool(description = "Persist an interaction event and any derived claims.")]
    async fn ingest_interaction(
        &self,
        Parameters(params): Parameters<IngestInteractionParams>,
    ) -> Result<CallToolResult, McpError> {
        let correlation_id = generated_mcp_correlation_id();
        let auto_reflect_input = AutoReflectInput::from_ingest(&params).map_err(|error| {
            record_app_error_to_mcp(
                &self.runtime.dashboard,
                "ingest_interaction",
                None,
                Some(correlation_id.clone()),
                error,
            )
        })?;
        let dashboard_namespace = Some(auto_reflect_input.namespace.as_str().to_string());
        let runtime_hook = runtime_hook_for("ingest_interaction", auto_reflect_input.trigger_type);
        let auto_reflect_trigger_type = auto_reflect_input.trigger_type;
        let auto_reflect_trigger_key = auto_reflect_input.trigger_key();
        let input = IngestInput::try_from(params).map_err(|error| {
            record_app_error_to_mcp(
                &self.runtime.dashboard,
                "ingest_interaction",
                dashboard_namespace.clone(),
                Some(correlation_id.clone()),
                AppError::from(error),
            )
        })?;
        let result = ingest_interaction::execute(&self.runtime, input)
            .await
            .map_err(|error| {
                record_app_error_to_mcp(
                    &self.runtime.dashboard,
                    "ingest_interaction",
                    dashboard_namespace.clone(),
                    Some(correlation_id.clone()),
                    error,
                )
            })?;
        match auto_reflect_if_needed::execute(
            &self.runtime,
            auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
        )
        .await
        {
            Ok(diagnostics) => log_auto_reflection_success(
                runtime_hook,
                &diagnostics,
                Some(result.event_id.as_str()),
                &self.runtime.dashboard,
                dashboard_namespace.clone(),
                Some(correlation_id.clone()),
            ),
            Err(error) => {
                warn!(
                    runtime_hook,
                    event_id = %result.event_id,
                    trigger_type = ?auto_reflect_trigger_type,
                    trigger_key = %auto_reflect_trigger_key,
                    error = %error,
                    "best-effort auto-reflection failed after successful ingest"
                );
            }
        }
        self.runtime.dashboard.record_tool_ok(
            "ingest_interaction",
            dashboard_namespace.clone(),
            Some(correlation_id.clone()),
            format!("ingest stored event {}", result.event_id),
            &result,
        );
        self.runtime
            .record_tool_operation(
                ToolOperationRecord::ok(
                    "ingest_interaction",
                    dashboard_namespace,
                    Some(correlation_id),
                )
                .with_response_summary(serde_json::json!({ "event_id": result.event_id })),
            )
            .await;
        structured(result)
    }

    #[tool(description = "Build a self snapshot from the persisted memory store.")]
    async fn build_self_snapshot(
        &self,
        Parameters(params): Parameters<BuildSelfSnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let correlation_id = generated_mcp_correlation_id();
        let auto_reflect_input =
            AutoReflectInput::from_build_snapshot(&params).map_err(|error| {
                record_app_error_to_mcp(
                    &self.runtime.dashboard,
                    "build_self_snapshot",
                    params.auto_reflect_namespace.clone(),
                    Some(correlation_id.clone()),
                    error,
                )
            })?;
        let dashboard_namespace = params.auto_reflect_namespace.clone();
        if let Some(auto_reflect_input) = auto_reflect_input {
            let auto_reflect_namespace = Some(auto_reflect_input.namespace.as_str().to_string());
            let auto_reflect_trigger_type = auto_reflect_input.trigger_type;
            let auto_reflect_trigger_key = auto_reflect_input.trigger_key();
            match auto_reflect_if_needed::execute(
                &self.runtime,
                auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
            )
            .await
            {
                Ok(diagnostics) => log_auto_reflection_success(
                    runtime_hook_for("build_self_snapshot", auto_reflect_trigger_type),
                    &diagnostics,
                    None,
                    &self.runtime.dashboard,
                    auto_reflect_namespace,
                    Some(correlation_id.clone()),
                ),
                Err(error) => {
                    warn!(
                        runtime_hook =
                            runtime_hook_for("build_self_snapshot", auto_reflect_trigger_type),
                        trigger_type = ?auto_reflect_trigger_type,
                        trigger_key = %auto_reflect_trigger_key,
                        error = %error,
                        "best-effort periodic auto-reflection failed"
                    );
                }
            }
        }
        let result = build_self_snapshot::execute(&self.runtime, params.into())
            .await
            .map_err(|error| {
                record_app_error_to_mcp(
                    &self.runtime.dashboard,
                    "build_self_snapshot",
                    dashboard_namespace.clone(),
                    Some(correlation_id.clone()),
                    error,
                )
            })?;
        self.runtime.dashboard.record_tool_ok(
            "build_self_snapshot",
            dashboard_namespace.clone(),
            Some(correlation_id.clone()),
            format!(
                "snapshot built with {} evidence links",
                result.snapshot.evidence.len()
            ),
            &result,
        );
        self.runtime
            .record_tool_operation(
                ToolOperationRecord::ok(
                    "build_self_snapshot",
                    dashboard_namespace,
                    Some(correlation_id),
                )
                .with_response_summary(serde_json::json!({
                    "snapshot_evidence_count": result.snapshot.evidence.len()
                })),
            )
            .await;
        structured(result)
    }

    #[tool(description = "Decide on an action using a provided self snapshot.")]
    async fn decide_with_snapshot(
        &self,
        Parameters(params): Parameters<DecideWithSnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let correlation_id = generated_mcp_correlation_id();
        let auto_reflect_input = AutoReflectInput::from_decide(&params).map_err(|error| {
            record_app_error_to_mcp(
                &self.runtime.dashboard,
                "decide_with_snapshot",
                params.auto_reflect_namespace.clone(),
                Some(correlation_id.clone()),
                error,
            )
        })?;
        let dashboard_namespace = params.auto_reflect_namespace.clone();
        let result = decide_with_snapshot::execute(&self.runtime, params.into())
            .await
            .map_err(|error| {
                record_app_error_to_mcp(
                    &self.runtime.dashboard,
                    "decide_with_snapshot",
                    dashboard_namespace.clone(),
                    Some(correlation_id.clone()),
                    error,
                )
            })?;
        if !result.blocked
            && let Some(auto_reflect_input) = auto_reflect_input
        {
            let auto_reflect_namespace = Some(auto_reflect_input.namespace.as_str().to_string());
            let auto_reflect_trigger_type = auto_reflect_input.trigger_type;
            let auto_reflect_trigger_key = auto_reflect_input.trigger_key();
            match auto_reflect_if_needed::execute(
                &self.runtime,
                auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
            )
            .await
            {
                Ok(diagnostics) => log_auto_reflection_success(
                    runtime_hook_for("decide_with_snapshot", auto_reflect_trigger_type),
                    &diagnostics,
                    None,
                    &self.runtime.dashboard,
                    auto_reflect_namespace,
                    Some(correlation_id.clone()),
                ),
                Err(error) => {
                    warn!(
                        runtime_hook =
                            runtime_hook_for("decide_with_snapshot", auto_reflect_trigger_type),
                        trigger_type = ?auto_reflect_trigger_type,
                        trigger_key = %auto_reflect_trigger_key,
                        error = %error,
                        "best-effort conflict auto-reflection failed after successful decide_with_snapshot"
                    );
                }
            }
        }
        let summary = if result.blocked {
            "decision blocked by commitment gate".to_string()
        } else {
            "decision returned model action".to_string()
        };
        self.runtime.dashboard.record_tool_ok(
            "decide_with_snapshot",
            dashboard_namespace.clone(),
            Some(correlation_id.clone()),
            summary,
            &result,
        );
        self.runtime
            .record_tool_operation(
                ToolOperationRecord::ok(
                    "decide_with_snapshot",
                    dashboard_namespace,
                    Some(correlation_id),
                )
                .with_response_summary(serde_json::json!({ "blocked": result.blocked })),
            )
            .await;
        structured(result)
    }

    #[tool(description = "Record a reflection that supersedes an existing claim.")]
    async fn run_reflection(
        &self,
        Parameters(params): Parameters<RunReflectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let correlation_id = generated_mcp_correlation_id();
        let input = ReflectionInput::try_from(params).map_err(|error| {
            record_app_error_to_mcp(
                &self.runtime.dashboard,
                "run_reflection",
                None,
                Some(correlation_id.clone()),
                error,
            )
        })?;
        let result = run_reflection::execute(&self.runtime, input)
            .await
            .map_err(|error| {
                record_app_error_to_mcp(
                    &self.runtime.dashboard,
                    "run_reflection",
                    None,
                    Some(correlation_id.clone()),
                    error,
                )
            })?;
        self.runtime.dashboard.record_tool_ok(
            "run_reflection",
            None,
            Some(correlation_id.clone()),
            format!("reflection recorded {}", result.reflection_id),
            &result,
        );
        self.runtime
            .record_tool_operation(
                ToolOperationRecord::ok("run_reflection", None, Some(correlation_id))
                    .with_response_summary(
                        serde_json::json!({ "reflection_id": result.reflection_id }),
                    ),
            )
            .await;
        structured(result)
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some("Self-agent memory tools exposed over MCP stdio.".to_string()),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
struct Runtime {
    store: SqliteStore,
    model: RuntimeModel,
    dashboard: DashboardObserver,
}

#[derive(Clone)]
enum RuntimeModel {
    Mock(MockModel),
    OpenAiCompatible(OpenAiCompatibleModel),
}

impl Runtime {
    async fn bootstrap(config: &AppConfig, dashboard: DashboardObserver) -> Result<Self, AppError> {
        config.validate().map_err(AppError::Message)?;

        let store = SqliteStore::bootstrap(&config.database_url).await?;
        Self::from_store(config, store, dashboard).await
    }

    async fn from_store(
        config: &AppConfig,
        store: SqliteStore,
        dashboard: DashboardObserver,
    ) -> Result<Self, AppError> {
        config.validate().map_err(AppError::Message)?;

        let runtime = Self {
            store,
            model: build_runtime_model(config)?,
            dashboard,
        };
        runtime.ensure_default_identity().await?;
        Ok(runtime)
    }

    async fn ensure_default_identity(&self) -> Result<(), AppError> {
        match self.store.load_identity().await {
            Ok(_) => Ok(()),
            Err(AppError::Message(message)) if message == "missing identity" => {
                self.store
                    .save_identity(IdentityCore::new(vec![
                        "identity:self=agent_llm_mm".to_string(),
                    ]))
                    .await
            }
            Err(error) => Err(error),
        }
    }

    async fn record_tool_operation(&self, record: ToolOperationRecord) {
        let entry = OperationLogEntry {
            operation_id: Uuid::new_v4().to_string(),
            occurred_at: Utc::now(),
            namespace: record.namespace,
            actor_kind: ActorKind::System,
            actor_id: "mcp-stdio".to_string(),
            entrypoint: record.entrypoint.to_string(),
            operation_kind: OperationLogKind::Tool,
            status: record.status,
            correlation_id: record.correlation_id,
            request_summary_json: record.request_summary.map(|value| value.to_string()),
            response_summary_json: record.response_summary.map(|value| value.to_string()),
            diagnostic_summary_json: record.diagnostic_summary.map(|value| value.to_string()),
            redaction_version: 1,
        };

        if let Err(error) = self.store.append_operation(entry).await {
            warn!(
                entrypoint = record.entrypoint,
                error = %error,
                "failed to append MCP tool operation log entry"
            );
        }
    }
}

struct ToolOperationRecord {
    entrypoint: &'static str,
    namespace: Option<String>,
    status: OperationLogStatus,
    correlation_id: Option<String>,
    request_summary: Option<serde_json::Value>,
    response_summary: Option<serde_json::Value>,
    diagnostic_summary: Option<serde_json::Value>,
}

impl ToolOperationRecord {
    fn ok(
        entrypoint: &'static str,
        namespace: Option<String>,
        correlation_id: Option<String>,
    ) -> Self {
        Self {
            entrypoint,
            namespace,
            status: OperationLogStatus::Ok,
            correlation_id,
            request_summary: None,
            response_summary: None,
            diagnostic_summary: None,
        }
    }

    fn with_response_summary(mut self, summary: serde_json::Value) -> Self {
        self.response_summary = Some(summary);
        self
    }
}

fn generated_mcp_correlation_id() -> String {
    format!("mcp-tool-call-{}", Uuid::new_v4())
}

fn build_runtime_model(config: &AppConfig) -> Result<RuntimeModel, AppError> {
    match &config.model_config {
        ModelConfig::Mock => Ok(RuntimeModel::Mock(MockModel)),
        ModelConfig::OpenAiCompatible(model_config) => Ok(RuntimeModel::OpenAiCompatible(
            OpenAiCompatibleModel::new(model_config.clone())?,
        )),
    }
}

#[async_trait]
impl Clock for Runtime {
    async fn now(&self) -> Result<DateTime<Utc>, AppError> {
        Ok(Utc::now())
    }
}

#[async_trait]
impl IdGenerator for Runtime {
    async fn next_id(&self) -> Result<String, AppError> {
        Ok(Uuid::new_v4().to_string())
    }
}

#[async_trait]
impl EventStore for Runtime {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError> {
        self.store.append_event(event).await
    }

    async fn list_event_references(&self) -> Result<Vec<String>, AppError> {
        self.store.list_event_references().await
    }

    async fn query_evidence_event_ids(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError> {
        self.store.query_evidence_event_ids(query).await
    }

    async fn query_evidence_event_ids_unbounded(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError> {
        self.store.query_evidence_event_ids_unbounded(query).await
    }

    async fn has_event(&self, event_id: &str) -> Result<bool, AppError> {
        self.store.has_event(event_id).await
    }
}

#[async_trait]
impl ClaimStore for Runtime {
    async fn upsert_claim(&self, claim: StoredClaim) -> Result<(), AppError> {
        self.store.upsert_claim(claim).await
    }

    async fn link_evidence(&self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.store.link_evidence(claim_id, event_id).await
    }

    async fn list_active_claims(&self) -> Result<Vec<StoredClaim>, AppError> {
        self.store.list_active_claims().await
    }

    async fn update_claim_status(
        &self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        self.store.update_claim_status(claim_id, status).await
    }
}

#[async_trait]
impl EpisodeStore for Runtime {
    async fn record_event_in_episode(
        &self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError> {
        self.store
            .record_event_in_episode(episode_reference, event_id)
            .await
    }

    async fn list_episode_references(&self) -> Result<Vec<String>, AppError> {
        self.store.list_episode_references().await
    }
}

#[async_trait]
impl ReflectionStore for Runtime {
    async fn append_reflection(&self, reflection: StoredReflection) -> Result<(), AppError> {
        self.store.append_reflection(reflection).await
    }
}

#[async_trait]
impl TriggerLedgerStore for Runtime {
    async fn record_trigger_attempt(
        &self,
        entry: StoredTriggerLedgerEntry,
    ) -> Result<(), AppError> {
        self.store.record_trigger_attempt(entry).await
    }

    async fn latest_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError> {
        self.store.latest_trigger_entry(trigger_key).await
    }

    async fn latest_handled_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError> {
        self.store.latest_handled_trigger_entry(trigger_key).await
    }
}

#[async_trait]
impl IdentityStore for Runtime {
    async fn load_identity(&self) -> Result<IdentityCore, AppError> {
        self.store.load_identity().await
    }

    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError> {
        self.store.save_identity(identity).await
    }
}

#[async_trait]
impl CommitmentStore for Runtime {
    async fn list_commitments(
        &self,
    ) -> Result<Vec<crate::domain::commitment::Commitment>, AppError> {
        self.store.list_commitments().await
    }
}

#[async_trait]
impl ModelPort for Runtime {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        match &self.model {
            RuntimeModel::Mock(model) => model.decide(request).await,
            RuntimeModel::OpenAiCompatible(model) => model.decide(request).await,
        }
    }

    async fn propose_self_revision(
        &self,
        request: SelfRevisionRequest,
    ) -> Result<SelfRevisionProposal, AppError> {
        match &self.model {
            RuntimeModel::Mock(model) => model.propose_self_revision(request).await,
            RuntimeModel::OpenAiCompatible(model) => model.propose_self_revision(request).await,
        }
    }
}

#[async_trait]
impl IngestTransactionRunner for Runtime {
    async fn begin_ingest_transaction(
        &self,
    ) -> Result<Box<dyn IngestTransaction + Send + '_>, AppError> {
        self.store.begin_ingest_transaction().await
    }
}

#[async_trait]
impl ReflectionTransactionRunner for Runtime {
    async fn begin_reflection_transaction(
        &self,
    ) -> Result<Box<dyn ReflectionTransaction + Send + '_>, AppError> {
        self.store.begin_reflection_transaction().await
    }
}

fn app_error_to_mcp(error: AppError) -> McpError {
    match error {
        AppError::InvalidParams(message) => McpError::invalid_params(message, None),
        AppError::Message(message) => McpError::internal_error(message, None),
    }
}

fn record_app_error_to_mcp(
    dashboard: &DashboardObserver,
    operation: &str,
    namespace: Option<String>,
    correlation_id: Option<String>,
    error: AppError,
) -> McpError {
    let summary = format!("{operation} failed");
    let message = error.to_string();
    dashboard.record_tool_failed(operation, namespace, correlation_id, summary, message);
    app_error_to_mcp(error)
}

fn log_auto_reflection_success(
    runtime_hook: &'static str,
    result: &auto_reflect_if_needed::AutoReflectResult,
    event_id: Option<&str>,
    dashboard: &DashboardObserver,
    namespace: Option<String>,
    correlation_id: Option<String>,
) {
    info!(
        runtime_hook,
        event_id = ?event_id,
        triggered = result.triggered,
        trigger_type = ?result.trigger_type,
        trigger_key = ?result.trigger_key,
        ledger_status = ?result.ledger_status,
        reflection_id = ?result.reflection_id,
        suppression_reason = ?result.suppression_reason,
        reason = ?result.reason,
        cooldown_until = ?result.cooldown_until,
        evidence_event_ids = ?result.evidence_event_ids,
        "best-effort auto-reflection completed"
    );
    dashboard.record_auto_reflection(
        runtime_hook,
        namespace,
        correlation_id,
        auto_reflection_status(result),
        auto_reflection_summary(result),
        result,
    );
}

fn runtime_hook_for(source: &'static str, trigger_type: TriggerType) -> &'static str {
    match (source, trigger_type) {
        ("ingest_interaction", TriggerType::Failure) => AUTO_REFLECTION_RUNTIME_HOOKS[0],
        ("ingest_interaction", TriggerType::Conflict) => AUTO_REFLECTION_RUNTIME_HOOKS[1],
        ("decide_with_snapshot", TriggerType::Conflict) => AUTO_REFLECTION_RUNTIME_HOOKS[2],
        ("build_self_snapshot", TriggerType::Periodic) => AUTO_REFLECTION_RUNTIME_HOOKS[3],
        _ => "auto_reflection:unknown",
    }
}

fn auto_reflection_status(result: &auto_reflect_if_needed::AutoReflectResult) -> OperationStatus {
    match result.ledger_status {
        Some(TriggerLedgerStatus::Handled) => OperationStatus::Handled,
        Some(TriggerLedgerStatus::Rejected) => OperationStatus::Rejected,
        Some(TriggerLedgerStatus::Suppressed) => OperationStatus::Suppressed,
        Some(TriggerLedgerStatus::Pending) => OperationStatus::Started,
        None if result.triggered => OperationStatus::Handled,
        None => OperationStatus::Ok,
    }
}

fn auto_reflection_summary(result: &auto_reflect_if_needed::AutoReflectResult) -> String {
    if let Some(reflection_id) = result.reflection_id.as_deref() {
        return format!("auto-reflection linked reflection {reflection_id}");
    }
    if let Some(reason) = result.suppression_reason.as_deref() {
        return format!("auto-reflection suppressed: {reason}");
    }
    if let Some(reason) = result.reason.as_deref() {
        return format!("auto-reflection checked: {reason}");
    }
    "auto-reflection checked runtime evidence".to_string()
}

fn structured<T>(value: T) -> Result<CallToolResult, McpError>
where
    T: Serialize,
{
    let json = serde_json::to_value(value)
        .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    Ok(CallToolResult::structured(json))
}
