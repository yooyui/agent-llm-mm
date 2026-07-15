use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    domain::{
        event::EventReference,
        types::{EventKind, MemoryScope, Namespace, Owner},
    },
    error::AppError,
    ports::{EventReadRecord, EventRecordQuery, MAX_EVENT_RECORD_QUERY_LIMIT, MemoryReadStore},
};

pub const DEFAULT_SEARCH_MEMORY_LIMIT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMemoryInput {
    pub namespace: Namespace,
    pub event_reference: Option<EventReference>,
    pub kind: Option<EventKind>,
    pub recorded_after: Option<DateTime<Utc>>,
    pub recorded_before: Option<DateTime<Utc>>,
    pub limit: usize,
}

impl SearchMemoryInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.limit == 0 {
            return Err(AppError::InvalidParams(
                "search_memory limit must be at least 1".to_string(),
            ));
        }
        if self.limit > MAX_EVENT_RECORD_QUERY_LIMIT {
            return Err(AppError::InvalidParams(format!(
                "search_memory limit must be at most {MAX_EVENT_RECORD_QUERY_LIMIT}"
            )));
        }
        if self
            .recorded_after
            .zip(self.recorded_before)
            .is_some_and(|(after, before)| after > before)
        {
            return Err(AppError::InvalidParams(
                "recorded_after must be less than or equal to recorded_before".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchMemoryResult {
    pub owner: Owner,
    pub namespace: String,
    pub limit: usize,
    pub records: Vec<SearchMemoryRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchMemoryRecord {
    pub record_type: &'static str,
    pub id: String,
    pub recorded_at: DateTime<Utc>,
    pub owner: Owner,
    pub namespace: String,
    pub kind: EventKind,
    pub summary: String,
    pub provenance: EventProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventProvenance {
    pub evidence_event_reference: String,
    pub claim_ids: Vec<String>,
    pub episode_references: Vec<String>,
}

pub async fn execute<D>(deps: &D, input: SearchMemoryInput) -> Result<SearchMemoryResult, AppError>
where
    D: MemoryReadStore + Sync,
{
    input.validate()?;
    let scope = MemoryScope::for_namespace(input.namespace.clone());
    let owner = scope
        .owner()
        .expect("namespace-derived memory scope must have an owner");
    let records = deps
        .query_event_records(EventRecordQuery {
            scope,
            event_reference: input.event_reference,
            kind: input.kind,
            recorded_after: input.recorded_after,
            recorded_before: input.recorded_before,
            limit: input.limit,
        })
        .await?
        .into_iter()
        .map(SearchMemoryRecord::from)
        .collect();

    Ok(SearchMemoryResult {
        owner,
        namespace: input.namespace.as_str().to_string(),
        limit: input.limit,
        records,
    })
}

impl From<EventReadRecord> for SearchMemoryRecord {
    fn from(value: EventReadRecord) -> Self {
        let reference = value.event.event_reference();
        Self {
            record_type: "event",
            id: reference.clone(),
            recorded_at: value.event.recorded_at,
            owner: value.event.event.owner(),
            namespace: value.event.event.namespace().as_str().to_string(),
            kind: value.event.event.kind(),
            summary: value.event.event.summary().to_string(),
            provenance: EventProvenance {
                evidence_event_reference: reference,
                claim_ids: value.claim_ids,
                episode_references: value.episode_references,
            },
        }
    }
}
