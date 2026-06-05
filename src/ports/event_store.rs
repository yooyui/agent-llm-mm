use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    domain::{
        event::Event,
        types::{EventKind, Namespace, Owner},
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
    pub namespace: Option<Namespace>,
    pub owner: Option<Owner>,
    pub kind: Option<EventKind>,
    pub limit: Option<usize>,
    pub recorded_after: Option<DateTime<Utc>>,
    pub recorded_before: Option<DateTime<Utc>>,
    /// 有界收窄过滤：按 event_id 前缀精确匹配，仍 intersect-only / no-widening，不引入排序或打分。
    pub event_id_prefix: Option<String>,
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
