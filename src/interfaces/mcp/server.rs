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
use uuid::Uuid;

use agent_llm_mm::{
    adapters::{model::mock::MockModel, sqlite::SqliteStore},
    application::{build_self_snapshot, decide_with_snapshot, ingest_interaction, run_reflection},
    domain::identity_core::IdentityCore,
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, IdGenerator,
        IdentityStore, IngestTransaction, IngestTransactionRunner, ModelDecision,
        ModelDecisionRequest, ModelPort, ReflectionStore, ReflectionTransaction,
        ReflectionTransactionRunner, StoredClaim, StoredEvent, StoredReflection,
    },
    support::config::AppConfig,
};

use super::dto::{
    BuildSelfSnapshotParams, DecideWithSnapshotParams, IngestInteractionParams, RunReflectionParams,
};

pub async fn run_stdio_server() -> Result<()> {
    let server = Server::from_config(AppConfig::default()).await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[derive(Clone)]
pub struct Server {
    runtime: Runtime,
    tool_router: ToolRouter<Self>,
}

impl Server {
    async fn from_config(config: AppConfig) -> Result<Self, AppError> {
        let runtime = Runtime::bootstrap(&config.database_url).await?;
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
        let result = ingest_interaction::execute(&self.runtime, params.into())
            .await
            .map_err(app_error_to_mcp)?;
        structured(result)
    }

    #[tool(description = "Build a self snapshot from the persisted memory store.")]
    async fn build_self_snapshot(
        &self,
        Parameters(params): Parameters<BuildSelfSnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
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
        let result = decide_with_snapshot::execute(&self.runtime, params.into())
            .await
            .map_err(app_error_to_mcp)?;
        structured(result)
    }

    #[tool(description = "Record a reflection that supersedes an existing claim.")]
    async fn run_reflection(
        &self,
        Parameters(params): Parameters<RunReflectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = run_reflection::execute(&self.runtime, params.into())
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
    model: MockModel,
}

impl Runtime {
    async fn bootstrap(database_url: &str) -> Result<Self, AppError> {
        let store = SqliteStore::bootstrap(database_url).await?;
        let runtime = Self {
            store,
            model: MockModel,
        };
        runtime.ensure_default_identity().await?;
        Ok(runtime)
    }

    async fn ensure_default_identity(&self) -> Result<(), AppError> {
        if self.store.load_identity().await.is_err() {
            self.store
                .save_identity(IdentityCore::new(vec![
                    "identity:self=agent_llm_mm".to_string(),
                ]))
                .await?;
        }
        Ok(())
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
    ) -> Result<Vec<agent_llm_mm::domain::commitment::Commitment>, AppError> {
        self.store.list_commitments().await
    }
}

#[async_trait]
impl ModelPort for Runtime {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        self.model.decide(request).await
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
    McpError::internal_error(error.to_string(), None)
}

fn structured<T>(value: T) -> Result<CallToolResult, McpError>
where
    T: Serialize,
{
    let json = serde_json::to_value(value)
        .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    Ok(CallToolResult::structured(json))
}
