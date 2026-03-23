use agent_llm_mm::{
    domain::{
        claim::ClaimDraft,
        event::Event,
        types::{EventKind, Mode, Owner},
    },
    ports::{ClaimStatus, IngestTransactionRunner, StoredClaim, StoredEvent},
};
use chrono::{DateTime, Utc};
use sqlx::{Row, sqlite::SqlitePool};

#[tokio::test]
async fn sqlite_store_bootstraps_all_tables() {
    let context = test_support::new_sqlite_store().await;
    let tables = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&context.pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.get::<String, _>("name"))
    .collect::<Vec<_>>();

    assert!(tables.contains(&"events".to_string()));
    assert!(tables.contains(&"claims".to_string()));
    assert!(tables.contains(&"evidence_links".to_string()));
    assert!(tables.contains(&"episode_events".to_string()));
    assert!(tables.contains(&"reflections".to_string()));
}

#[tokio::test]
async fn sqlite_round_trips_claim_with_evidence() {
    let context = test_support::new_sqlite_store().await;
    let ids = test_support::seed_event_and_claim(&context.store)
        .await
        .unwrap();
    let claim_row = sqlx::query(
        r#"
        SELECT claim_id, subject, object, status
        FROM claims
        WHERE claim_id = ?
        "#,
    )
    .bind(&ids.claim_id)
    .fetch_one(&context.pool)
    .await
    .unwrap();
    let evidence_rows = sqlx::query(
        r#"
        SELECT event_id
        FROM evidence_links
        WHERE claim_id = ?
        ORDER BY rowid
        "#,
    )
    .bind(&ids.claim_id)
    .fetch_all(&context.pool)
    .await
    .unwrap();

    assert_eq!(claim_row.get::<String, _>("claim_id"), ids.claim_id);
    assert_eq!(claim_row.get::<String, _>("subject"), "self.role");
    assert_eq!(claim_row.get::<String, _>("object"), "architect");
    assert_eq!(claim_row.get::<String, _>("status"), "active");
    assert_eq!(evidence_rows.len(), 1);
    assert_eq!(evidence_rows[0].get::<String, _>("event_id"), ids.event_id);
}

mod test_support {
    use super::*;
    use agent_llm_mm::adapters::sqlite::SqliteStore;

    pub struct TestContext {
        pub store: SqliteStore,
        pub pool: SqlitePool,
    }

    pub struct SeedIds {
        pub claim_id: String,
        pub event_id: String,
    }

    pub async fn new_sqlite_store() -> TestContext {
        let path =
            std::env::temp_dir().join(format!("agent-llm-mm-{}.sqlite", uuid::Uuid::new_v4()));
        let database_url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
        let store = SqliteStore::bootstrap(&database_url).await.unwrap();
        let pool = SqlitePool::connect(&database_url).await.unwrap();

        TestContext { store, pool }
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
