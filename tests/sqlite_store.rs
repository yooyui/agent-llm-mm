use agent_llm_mm::{
    domain::{
        claim::ClaimDraft,
        event::Event,
        identity_core::IdentityCore,
        reflection::Reflection,
        types::{EventKind, Mode, Owner},
    },
    ports::{
        ClaimStatus, ClaimStore, CommitmentStore, IdentityStore, IngestTransactionRunner,
        ReflectionTransactionRunner, StoredClaim, StoredEvent, StoredReflection,
    },
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
    assert!(tables.contains(&"identity_claims".to_string()));
    assert!(tables.contains(&"commitments".to_string()));
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

#[tokio::test]
async fn sqlite_reflection_transactions_commit_and_roll_back_as_expected() {
    let context = test_support::new_sqlite_store().await;
    test_support::seed_claim(&context.store, "claim-old")
        .await
        .unwrap();

    let mut ok_tx = context.store.begin_reflection_transaction().await.unwrap();
    ok_tx
        .upsert_claim(StoredClaim::new(
            "claim-new".to_string(),
            ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "senior_architect",
                Mode::Observed,
            ),
            ClaimStatus::Active,
        ))
        .await
        .unwrap();
    ok_tx
        .append_reflection(StoredReflection::new(
            "refl-1".to_string(),
            test_support::fixed_now(),
            Reflection::new("replace old claim"),
            Some("claim-old".to_string()),
            Some("claim-new".to_string()),
        ))
        .await
        .unwrap();
    ok_tx
        .update_claim_status("claim-old", ClaimStatus::Superseded)
        .await
        .unwrap();
    ok_tx.commit().await.unwrap();

    let committed_reflection_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reflections WHERE reflection_id = 'refl-1'",
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();
    let old_status =
        sqlx::query_scalar::<_, String>("SELECT status FROM claims WHERE claim_id = 'claim-old'")
            .fetch_one(&context.pool)
            .await
            .unwrap();
    assert_eq!(committed_reflection_count, 1);
    assert_eq!(old_status, "superseded");

    let mut failing_tx = context.store.begin_reflection_transaction().await.unwrap();
    failing_tx
        .upsert_claim(StoredClaim::new(
            "claim-rolled-back".to_string(),
            ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "staff_architect",
                Mode::Observed,
            ),
            ClaimStatus::Active,
        ))
        .await
        .unwrap();
    failing_tx
        .append_reflection(StoredReflection::new(
            "refl-missing".to_string(),
            test_support::fixed_now(),
            Reflection::new("this should fail"),
            Some("claim-missing".to_string()),
            Some("claim-rolled-back".to_string()),
        ))
        .await
        .unwrap();

    let update_result = failing_tx
        .update_claim_status("claim-missing", ClaimStatus::Superseded)
        .await;
    assert!(update_result.is_err());
    let commit_result = failing_tx.commit().await;
    assert!(commit_result.is_err());

    let rolled_back_reflection_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reflections WHERE reflection_id = 'refl-missing'",
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();
    let rolled_back_claim_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM claims WHERE claim_id = 'claim-rolled-back'",
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();
    assert_eq!(rolled_back_reflection_count, 0);
    assert_eq!(rolled_back_claim_count, 0);
}

#[tokio::test]
async fn sqlite_store_persists_identity_and_reads_commitments_for_snapshot_ports() {
    let context = test_support::new_sqlite_store().await;

    context
        .store
        .save_identity(IdentityCore::new(vec![
            "identity:self=architect".to_string(),
            "identity:style=rigorous".to_string(),
        ]))
        .await
        .unwrap();

    let identity = context.store.load_identity().await.unwrap();
    let commitments = context.store.list_commitments().await.unwrap();

    assert_eq!(
        identity.canonical_claims(),
        &[
            "identity:self=architect".to_string(),
            "identity:style=rigorous".to_string()
        ]
    );
    assert_eq!(commitments.len(), 1);
    assert_eq!(
        commitments[0].description(),
        "forbid:write_identity_core_directly"
    );
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

    pub async fn seed_claim(
        store: &SqliteStore,
        claim_id: &str,
    ) -> Result<(), agent_llm_mm::error::AppError> {
        store
            .upsert_claim(StoredClaim::new(
                claim_id.to_string(),
                ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
                ClaimStatus::Active,
            ))
            .await
    }
    pub fn fixed_now() -> DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }
}
