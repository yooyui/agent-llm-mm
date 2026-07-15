use serde::Serialize;

use crate::{
    domain::{
        event::EventReference,
        types::{Namespace, Owner},
    },
    error::AppError,
    ports::MemoryReadStore,
};

use super::search_memory::{MemoryRecordType, SearchMemoryInput, SearchMemoryRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetMemoryInput {
    pub namespace: Namespace,
    pub id: EventReference,
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
    let result = super::search_memory::execute(
        deps,
        SearchMemoryInput {
            namespace: input.namespace,
            record_type: MemoryRecordType::Event,
            event_reference: Some(input.id),
            kind: None,
            recorded_after: None,
            recorded_before: None,
            claim_reference: None,
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
