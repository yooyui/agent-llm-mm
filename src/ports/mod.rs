use async_trait::async_trait;

use crate::error::AppError;

pub mod claim_store;
pub mod clock;
pub mod commitment_store;
pub mod episode_store;
pub mod event_store;
pub mod id_generator;
pub mod identity_store;
pub mod model_port;
pub mod reflection_store;

pub use claim_store::{ClaimStatus, ClaimStore, StoredClaim};
pub use clock::Clock;
pub use commitment_store::CommitmentStore;
pub use episode_store::EpisodeStore;
pub use event_store::{EventStore, StoredEvent};
pub use id_generator::IdGenerator;
pub use identity_store::IdentityStore;
pub use model_port::{ModelDecision, ModelInput, ModelPort};
pub use reflection_store::{ReflectionStore, StoredReflection};

#[async_trait]
pub trait IngestTransaction {
    async fn append_event(&mut self, event: StoredEvent) -> Result<(), AppError>;
    async fn record_event_in_episode(
        &mut self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError>;
    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError>;
    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError>;
    async fn commit(self: Box<Self>) -> Result<(), AppError>;
}

#[async_trait]
pub trait IngestTransactionRunner {
    async fn begin_ingest_transaction(&self) -> Result<Box<dyn IngestTransaction + '_>, AppError>;
}

#[async_trait]
pub trait ReflectionTransaction {
    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError>;
    async fn append_reflection(&mut self, reflection: StoredReflection) -> Result<(), AppError>;
    async fn update_claim_status(
        &mut self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError>;
    async fn commit(self: Box<Self>) -> Result<(), AppError>;
}

#[async_trait]
pub trait ReflectionTransactionRunner {
    async fn begin_reflection_transaction(
        &self,
    ) -> Result<Box<dyn ReflectionTransaction + '_>, AppError>;
}
