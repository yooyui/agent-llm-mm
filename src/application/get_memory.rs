use serde::Serialize;

use crate::{
    domain::{
        claim::ClaimReference,
        event::EventReference,
        types::{Namespace, Owner},
    },
    error::AppError,
    ports::MemoryReadStore,
};

use super::search_memory::{MemoryRecordType, SearchMemoryInput, SearchMemoryRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryRecordReference {
    Event(EventReference),
    Claim(ClaimReference),
}

impl MemoryRecordReference {
    pub fn record_type(&self) -> MemoryRecordType {
        match self {
            Self::Event(_) => MemoryRecordType::Event,
            Self::Claim(_) => MemoryRecordType::Claim,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetMemoryInput {
    pub namespace: Namespace,
    pub id: MemoryRecordReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetMemoryResult {
    pub owner: Owner,
    pub namespace: String,
    pub record: Option<SearchMemoryRecord>,
}

pub async fn execute<D>(deps: &D, input: GetMemoryInput) -> Result<GetMemoryResult, AppError>
where
    D: MemoryReadStore + Sync,
{
    let record_type = input.id.record_type();
    let (event_reference, claim_reference) = match input.id {
        MemoryRecordReference::Event(reference) => (Some(reference), None),
        MemoryRecordReference::Claim(reference) => (None, Some(reference)),
    };
    let result = super::search_memory::execute(
        deps,
        SearchMemoryInput {
            namespace: input.namespace,
            record_type,
            event_reference,
            kind: None,
            recorded_after: None,
            recorded_before: None,
            claim_reference,
            claim_status: None,
            mode: None,
            limit: 1,
        },
    )
    .await?;

    Ok(GetMemoryResult {
        owner: result.owner,
        namespace: result.namespace,
        record: result.records.into_iter().next(),
    })
}
