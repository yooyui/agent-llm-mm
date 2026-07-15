use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    domain::{
        claim::ClaimReference,
        event::EventReference,
        types::{EventKind, MemoryScope, Mode, Namespace, Owner},
    },
    error::AppError,
    ports::{
        ClaimReadRecord, ClaimRecordQuery, ClaimStatus, EventReadRecord, EventRecordQuery,
        MAX_EVENT_RECORD_QUERY_LIMIT, MemoryReadStore,
    },
};

pub const DEFAULT_SEARCH_MEMORY_LIMIT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRecordType {
    Event,
    Claim,
}

impl MemoryRecordType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Claim => "claim",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMemoryInput {
    pub namespace: Namespace,
    pub record_type: MemoryRecordType,
    pub event_reference: Option<EventReference>,
    pub kind: Option<EventKind>,
    pub recorded_after: Option<DateTime<Utc>>,
    pub recorded_before: Option<DateTime<Utc>>,
    pub claim_reference: Option<ClaimReference>,
    pub claim_status: Option<ClaimStatus>,
    pub mode: Option<Mode>,
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

        match self.record_type {
            MemoryRecordType::Event
                if self.claim_reference.is_some()
                    || self.claim_status.is_some()
                    || self.mode.is_some() =>
            {
                Err(AppError::InvalidParams(
                    "claim_reference, claim_status, and mode require record_type Claim".to_string(),
                ))
            }
            MemoryRecordType::Claim
                if self.event_reference.is_some()
                    || self.kind.is_some()
                    || self.recorded_after.is_some()
                    || self.recorded_before.is_some() =>
            {
                Err(AppError::InvalidParams(
                    "event_reference, kind, and recorded time filters require record_type Event; claims do not have a stored recorded_at timestamp"
                        .to_string(),
                ))
            }
            _ => Ok(()),
        }
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
#[serde(tag = "record_type", rename_all = "snake_case")]
pub enum SearchMemoryRecord {
    Event {
        id: String,
        recorded_at: DateTime<Utc>,
        owner: Owner,
        namespace: String,
        kind: EventKind,
        summary: String,
        provenance: EventProvenance,
    },
    Claim {
        id: String,
        owner: Owner,
        namespace: String,
        subject: String,
        predicate: String,
        object: String,
        mode: Mode,
        status: ClaimStatus,
        provenance: ClaimProvenance,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventProvenance {
    pub evidence_event_reference: String,
    pub claim_ids: Vec<String>,
    pub episode_references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimProvenance {
    pub evidence_event_references: Vec<String>,
    pub episode_references: Vec<String>,
    pub source_reflection_id: Option<String>,
    pub supersedes_claim_reference: Option<String>,
    pub superseded_by_reflection_id: Option<String>,
    pub replacement_claim_reference: Option<String>,
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
    let records = match input.record_type {
        MemoryRecordType::Event => deps
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
            .collect(),
        MemoryRecordType::Claim => deps
            .query_claim_records(ClaimRecordQuery {
                scope,
                claim_reference: input.claim_reference,
                status: input.claim_status,
                mode: input.mode,
                limit: input.limit,
            })
            .await?
            .into_iter()
            .map(SearchMemoryRecord::from)
            .collect(),
    };

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
        Self::Event {
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

impl From<ClaimReadRecord> for SearchMemoryRecord {
    fn from(value: ClaimReadRecord) -> Self {
        Self::Claim {
            id: ClaimReference::from_claim_id(&value.claim.claim_id).canonical(),
            owner: value.claim.claim.owner(),
            namespace: value.claim.claim.namespace().as_str().to_string(),
            subject: value.claim.claim.subject().to_string(),
            predicate: value.claim.claim.predicate().to_string(),
            object: value.claim.claim.object().to_string(),
            mode: value.claim.claim.mode(),
            status: value.claim.status,
            provenance: ClaimProvenance {
                evidence_event_references: value
                    .evidence_event_references
                    .into_iter()
                    .map(|reference| reference.canonical())
                    .collect(),
                episode_references: value.episode_references,
                source_reflection_id: value.revision.source_reflection_id,
                supersedes_claim_reference: value
                    .revision
                    .supersedes_claim_reference
                    .map(|reference| reference.canonical()),
                superseded_by_reflection_id: value.revision.superseded_by_reflection_id,
                replacement_claim_reference: value
                    .revision
                    .replacement_claim_reference
                    .map(|reference| reference.canonical()),
            },
        }
    }
}
