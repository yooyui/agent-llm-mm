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
    domain::self_revision::{SelfRevisionProposal, SelfRevisionRequest},
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, EvidenceQuery,
        IdGenerator, IdentityStore, IngestTransaction, IngestTransactionRunner, ModelDecision,
        ModelDecisionRequest, ModelPort, ReflectionStore, ReflectionTransaction,
        ReflectionTransactionRunner, StoredClaim, StoredEvent, StoredReflection,
        StoredTriggerLedgerEntry, TriggerLedgerStore,
    },
    support::config::{AppConfig, ModelConfig},
};

use super::dto::{
    BuildSelfSnapshotParams, DecideWithSnapshotParams, IngestInteractionParams, RunReflectionParams,
};

pub const AUTO_REFLECTION_RUNTIME_HOOKS: [&str; 3] = [
    "ingest_interaction:failure",
    "decide_with_snapshot:conflict",
    "build_self_snapshot:periodic",
];
pub const SELF_REVISION_WRITE_PATH: &str = "run_reflection";

pub async fn run_stdio_server() -> Result<()> {
    let config = AppConfig::load().map_err(anyhow::Error::msg)?;
    run_stdio_server_with_config(config).await
}

pub async fn run_stdio_server_with_config(config: AppConfig) -> Result<()> {
    let server = Server::from_config(config).await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

pub async fn validate_stdio_runtime(config: &AppConfig) -> Result<(), AppError> {
    Runtime::bootstrap(config).await.map(|_| ())
}

#[derive(Clone)]
pub struct Server {
    runtime: Runtime,
    tool_router: ToolRouter<Self>,
}

impl Server {
    async fn from_config(config: AppConfig) -> Result<Self, AppError> {
        let runtime = Runtime::bootstrap(&config).await?;
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
        let auto_reflect_input =
            AutoReflectInput::from_ingest(&params).map_err(app_error_to_mcp)?;
        let auto_reflect_trigger_type = auto_reflect_input.trigger_type;
        let auto_reflect_trigger_key = auto_reflect_input.trigger_key();
        let input =
            IngestInput::try_from(params).map_err(|error| app_error_to_mcp(error.into()))?;
        let result = ingest_interaction::execute(&self.runtime, input)
            .await
            .map_err(app_error_to_mcp)?;
        match auto_reflect_if_needed::execute(
            &self.runtime,
            auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
        )
        .await
        {
            Ok(diagnostics) => log_auto_reflection_success(
                AUTO_REFLECTION_RUNTIME_HOOKS[0],
                &diagnostics,
                Some(result.event_id.as_str()),
            ),
            Err(error) => {
                warn!(
                    runtime_hook = AUTO_REFLECTION_RUNTIME_HOOKS[0],
                    event_id = %result.event_id,
                    trigger_type = ?auto_reflect_trigger_type,
                    trigger_key = %auto_reflect_trigger_key,
                    error = %error,
                    "best-effort auto-reflection failed after successful ingest"
                );
            }
        }
        structured(result)
    }

    #[tool(description = "Build a self snapshot from the persisted memory store.")]
    async fn build_self_snapshot(
        &self,
        Parameters(params): Parameters<BuildSelfSnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let auto_reflect_input =
            AutoReflectInput::from_build_snapshot(&params).map_err(app_error_to_mcp)?;
        if let Some(auto_reflect_input) = auto_reflect_input {
            let auto_reflect_trigger_type = auto_reflect_input.trigger_type;
            let auto_reflect_trigger_key = auto_reflect_input.trigger_key();
            match auto_reflect_if_needed::execute(
                &self.runtime,
                auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
            )
            .await
            {
                Ok(diagnostics) => log_auto_reflection_success(
                    AUTO_REFLECTION_RUNTIME_HOOKS[2],
                    &diagnostics,
                    None,
                ),
                Err(error) => {
                    warn!(
                        runtime_hook = AUTO_REFLECTION_RUNTIME_HOOKS[2],
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
            .map_err(app_error_to_mcp)?;
        structured(result)
    }

    #[tool(description = "Decide on an action using a provided self snapshot.")]
    async fn decide_with_snapshot(
        &self,
        Parameters(params): Parameters<DecideWithSnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let auto_reflect_input =
            AutoReflectInput::from_decide(&params).map_err(app_error_to_mcp)?;
        let result = decide_with_snapshot::execute(&self.runtime, params.into())
            .await
            .map_err(app_error_to_mcp)?;
        if !result.blocked {
            if let Some(auto_reflect_input) = auto_reflect_input {
                let auto_reflect_trigger_type = auto_reflect_input.trigger_type;
                let auto_reflect_trigger_key = auto_reflect_input.trigger_key();
                match auto_reflect_if_needed::execute(
                    &self.runtime,
                    auto_reflect_input.with_recursion_guard(RecursionGuard::Allow),
                )
                .await
                {
                    Ok(diagnostics) => log_auto_reflection_success(
                        AUTO_REFLECTION_RUNTIME_HOOKS[1],
                        &diagnostics,
                        None,
                    ),
                    Err(error) => {
                        warn!(
                            runtime_hook = AUTO_REFLECTION_RUNTIME_HOOKS[1],
                            trigger_type = ?auto_reflect_trigger_type,
                            trigger_key = %auto_reflect_trigger_key,
                            error = %error,
                            "best-effort conflict auto-reflection failed after successful decide_with_snapshot"
                        );
                    }
                }
            }
        }
        structured(result)
    }

    #[tool(description = "Record a reflection that supersedes an existing claim.")]
    async fn run_reflection(
        &self,
        Parameters(params): Parameters<RunReflectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let input = ReflectionInput::try_from(params).map_err(app_error_to_mcp)?;
        let result = run_reflection::execute(&self.runtime, input)
            .await
            .map_err(app_error_to_mcp)?;
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
}

#[derive(Clone)]
enum RuntimeModel {
    Mock(MockModel),
    OpenAiCompatible(OpenAiCompatibleModel),
}

impl Runtime {
    async fn bootstrap(config: &AppConfig) -> Result<Self, AppError> {
        config.validate_model_config().map_err(AppError::Message)?;

        let store = SqliteStore::bootstrap(&config.database_url).await?;
        let runtime = Self {
            store,
            model: build_runtime_model(config)?,
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

fn log_auto_reflection_success(
    runtime_hook: &'static str,
    result: &auto_reflect_if_needed::AutoReflectResult,
    event_id: Option<&str>,
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
}

fn structured<T>(value: T) -> Result<CallToolResult, McpError>
where
    T: Serialize,
{
    let json = serde_json::to_value(value)
        .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    Ok(CallToolResult::structured(json))
}
