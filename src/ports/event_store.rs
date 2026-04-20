use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    domain::{
        event::Event,
        types::{EventKind, Owner},
    },
    error::AppError,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredEvent {
    pub event_id: String,
    pub recorded_at: DateTime<Utc>,
    pub event: Event,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceQuery {
    pub owner: Option<Owner>,
    pub kind: Option<EventKind>,
    pub limit: Option<usize>,
}

impl StoredEvent {
    pub fn new(event_id: String, recorded_at: DateTime<Utc>, event: Event) -> Self {
        Self {
            event_id,
            recorded_at,
            event,
        }
    }

    pub fn event_reference(&self) -> String {
        format!("event:{}", self.event_id)
    }
}

#[async_trait]
pub trait EventStore {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError>;
    async fn list_event_references(&self) -> Result<Vec<String>, AppError>;
    async fn query_evidence_event_ids(&self, query: EvidenceQuery)
    -> Result<Vec<String>, AppError>;
    async fn query_evidence_event_ids_unbounded(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError>;
    async fn has_event(&self, event_id: &str) -> Result<bool, AppError>;
}
