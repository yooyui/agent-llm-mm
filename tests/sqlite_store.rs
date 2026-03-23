use agent_llm_mm::{
    domain::{
        claim::ClaimDraft,
        event::Event,
        types::{EventKind, Mode, Owner},
    },
    ports::{ClaimStatus, IngestTransactionRunner, StoredClaim, StoredEvent},
};
use chrono::{DateTime, Utc};

#[tokio::test]
async fn sqlite_store_bootstraps_all_tables() {
    let store = test_support::new_sqlite_store().await;
    let tables = store.list_tables().await.unwrap();

    assert!(tables.contains(&"events".to_string()));
    assert!(tables.contains(&"claims".to_string()));
    assert!(tables.contains(&"evidence_links".to_string()));
    assert!(tables.contains(&"episode_events".to_string()));
    assert!(tables.contains(&"reflections".to_string()));
}

#[tokio::test]
async fn sqlite_round_trips_claim_with_evidence() {
    let store = test_support::new_sqlite_store().await;
    let ids = test_support::seed_event_and_claim(&store).await.unwrap();

    let loaded = store.load_claim_with_evidence(&ids.claim_id).await.unwrap();

    assert_eq!(loaded.claim.claim_id, ids.claim_id);
    assert_eq!(loaded.claim.claim.subject(), "self.role");
    assert_eq!(loaded.claim.claim.object(), "architect");
    assert_eq!(loaded.claim.status, ClaimStatus::Active);
    assert_eq!(loaded.evidence.len(), 1);
    assert_eq!(loaded.evidence[0].event_id, ids.event_id);
}

mod test_support {
    use super::*;
    use agent_llm_mm::adapters::sqlite::SqliteStore;

    pub struct SeedIds {
        pub claim_id: String,
        pub event_id: String,
    }

    pub async fn new_sqlite_store() -> SqliteStore {
        let path =
            std::env::temp_dir().join(format!("agent-llm-mm-{}.sqlite", uuid::Uuid::new_v4()));
        let database_url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));

        SqliteStore::bootstrap(&database_url).await.unwrap()
    }

    pub async fn seed_event_and_claim(
        store: &SqliteStore,
    ) -> Result<SeedIds, agent_llm_mm::error::AppError> {
        let event_id = "evt-1".to_string();
        let claim_id = "claim-1".to_string();
        let recorded_at = fixed_now();
        let event = StoredEvent::new(
            event_id.clone(),
            recorded_at,
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "The user asked for stronger memory.",
            ),
        );
        let claim = StoredClaim::new(
            claim_id.clone(),
            ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
            ClaimStatus::Active,
        );
        let mut tx = store.begin_ingest_transaction().await?;
        tx.append_event(event).await?;
        tx.upsert_claim(claim).await?;
        tx.link_evidence(claim_id.clone(), event_id.clone()).await?;
        tx.commit().await?;

        Ok(SeedIds { claim_id, event_id })
    }

    fn fixed_now() -> DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }
}
