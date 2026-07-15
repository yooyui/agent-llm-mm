use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    domain::{
        event::EventReference,
        types::{EventKind, MemoryScope},
    },
    error::AppError,
};

use super::StoredEvent;

pub const MAX_EVENT_RECORD_QUERY_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecordQuery {
    pub scope: MemoryScope,
    pub event_reference: Option<EventReference>,
    pub kind: Option<EventKind>,
    pub recorded_after: Option<DateTime<Utc>>,
    pub recorded_before: Option<DateTime<Utc>>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventReadRecord {
    pub event: StoredEvent,
    pub claim_ids: Vec<String>,
    pub episode_references: Vec<String>,
}

impl EventReadRecord {
    pub fn new(
        event: StoredEvent,
        claim_ids: Vec<String>,
        episode_references: Vec<String>,
    ) -> Self {
        Self {
            event,
            claim_ids,
            episode_references,
        }
    }
}

#[async_trait]
pub trait MemoryReadStore {
    async fn query_event_records(
        &self,
        query: EventRecordQuery,
    ) -> Result<Vec<EventReadRecord>, AppError>;
}
