use chrono::{DateTime, Utc};

use crate::{
    domain::{claim::ClaimDraft, event::Event},
    error::AppError,
    ports::{ClaimStatus, Clock, IdGenerator, IngestTransactionRunner, StoredClaim, StoredEvent},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IngestInput {
    event: Event,
    claim_drafts: Vec<ClaimDraft>,
    episode_reference: Option<String>,
}

impl IngestInput {
    pub fn new(
        event: Event,
        claim_drafts: Vec<ClaimDraft>,
        episode_reference: Option<String>,
    ) -> Self {
        Self {
            event,
            claim_drafts,
            episode_reference,
        }
    }

    fn into_parts(self) -> (Event, Vec<ClaimDraft>, Option<String>) {
        (self.event, self.claim_drafts, self.episode_reference)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IngestResult {
    pub event_id: String,
}

impl IngestResult {
    fn from_event(event: &StoredEvent) -> Self {
        Self {
            event_id: event.event_id.clone(),
        }
    }
}

fn build_event(event_id: String, recorded_at: DateTime<Utc>, event: Event) -> StoredEvent {
    StoredEvent::new(event_id, recorded_at, event)
}

fn derive_claims(event_id: &str, drafts: Vec<ClaimDraft>) -> Result<Vec<StoredClaim>, AppError> {
    drafts
        .into_iter()
        .enumerate()
        .map(|(index, draft)| {
            draft.validate(1)?;
            Ok(StoredClaim::new(
                format!("{event_id}:claim:{index}"),
                draft,
                ClaimStatus::Active,
            ))
        })
        .collect()
}

pub async fn execute<D>(deps: &D, input: IngestInput) -> Result<IngestResult, AppError>
where
    D: IngestTransactionRunner + IdGenerator + Clock + Sync,
{
    let (event, claim_drafts, episode_reference) = input.into_parts();
    let event = build_event(deps.next_id().await?, deps.now().await?, event);
    let mut transaction = deps.begin_ingest_transaction().await?;

    transaction.append_event(event.clone()).await?;

    if let Some(episode_reference) = episode_reference {
        transaction
            .record_event_in_episode(episode_reference, event.event_id.clone())
            .await?;
    }

    for claim in derive_claims(&event.event_id, claim_drafts)? {
        transaction.upsert_claim(claim.clone()).await?;
        transaction
            .link_evidence(claim.claim_id.clone(), event.event_id.clone())
            .await?;
    }

    transaction.commit().await?;

    Ok(IngestResult::from_event(&event))
}
