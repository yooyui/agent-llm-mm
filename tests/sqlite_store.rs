use agent_llm_mm::{
    domain::{
        claim::ClaimDraft,
        event::Event,
        identity_core::IdentityCore,
        reflection::Reflection,
        self_revision::TriggerType,
        types::{EventKind, Mode, Namespace, Owner},
    },
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, CommitmentStore, EventStore, EvidenceQuery, IdentityStore,
        IngestTransactionRunner, ReflectionTransactionRunner, StoredClaim, StoredEvent,
        StoredReflection, StoredTriggerLedgerEntry, TriggerLedgerStatus, TriggerLedgerStore,
    },
};
use chrono::{DateTime, Utc};
use sqlx::{Row, sqlite::SqlitePool};
use std::path::PathBuf;

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
    assert!(tables.contains(&"reflection_trigger_ledger".to_string()));
    assert!(tables.contains(&"identity_claims".to_string()));
    assert!(tables.contains(&"commitments".to_string()));

    let claims_sql = sqlx::query_scalar::<_, String>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'claims'",
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();
    assert!(
        claims_sql.contains("CHECK"),
        "claims table should include a database-level namespace compatibility check"
    );
}

#[tokio::test]
async fn sqlite_query_evidence_event_ids_is_recent_first_and_filtered() {
    let context = test_support::new_sqlite_store().await;
    let now = test_support::fixed_now();

    context
        .store
        .append_event(StoredEvent::new(
            "evt-world-old".to_string(),
            now,
            Event::new(Owner::World, EventKind::Observation, "older world obs"),
        ))
        .await
        .unwrap();
    context
        .store
        .append_event(StoredEvent::new(
            "evt-user-note".to_string(),
            now + chrono::Duration::seconds(60),
            Event::new(Owner::User, EventKind::Conversation, "user conversation"),
        ))
        .await
        .unwrap();
    context
        .store
        .append_event(StoredEvent::new(
            "evt-world-new".to_string(),
            now + chrono::Duration::seconds(120),
            Event::new(Owner::World, EventKind::Observation, "newer world obs"),
        ))
        .await
        .unwrap();

    let results = context
        .store
        .query_evidence_event_ids(EvidenceQuery {
            owner: Some(Owner::World),
            kind: Some(EventKind::Observation),
            limit: Some(2),
        })
        .await
        .unwrap();

    assert_eq!(
        results,
        vec!["evt-world-new".to_string(), "evt-world-old".to_string()]
    );
}

#[cfg(target_pointer_width = "64")]
#[tokio::test]
async fn sqlite_query_evidence_event_ids_rejects_limit_above_i64_max() {
    let context = test_support::new_sqlite_store().await;
    let excessive_limit = (i64::MAX as u64 + 1) as usize;

    let result = context
        .store
        .query_evidence_event_ids(EvidenceQuery {
            owner: None,
            kind: None,
            limit: Some(excessive_limit),
        })
        .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
}

#[test]
fn sqlite_owner_namespace_sql_rules_have_single_source() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_source =
        std::fs::read_to_string(manifest_dir.join("src/adapters/sqlite/schema.rs")).unwrap();
    let store_source =
        std::fs::read_to_string(manifest_dir.join("src/adapters/sqlite/store.rs")).unwrap();

    assert!(
        schema_source.contains("claims_table_sql("),
        "schema.rs should define the shared claims table SQL builder"
    );
    assert!(
        schema_source.contains("legacy_namespace_backfill_expression("),
        "schema.rs should define the shared legacy namespace backfill expression"
    );
    assert!(
        store_source.contains("claims_table_sql("),
        "store.rs should use the shared claims table SQL builder"
    );
    assert!(
        store_source.contains("legacy_namespace_backfill_expression("),
        "store.rs should use the shared legacy namespace backfill expression"
    );

    for forbidden_fragment in [
        "CONSTRAINT owner_namespace_scope CHECK (",
        "OR (owner = 'user' AND namespace LIKE 'user/%')",
        "OR (owner = 'world' AND (namespace = 'world' OR namespace LIKE 'project/%'))",
        "OR (owner = 'unknown' AND (namespace = 'world' OR namespace LIKE 'project/%'))",
        "CASE owner WHEN 'self' THEN 'self' WHEN 'user' THEN 'user/default' ELSE 'world' END",
        "COALESCE(NULLIF(namespace, ''), CASE owner WHEN 'self' THEN 'self' WHEN 'user' THEN 'user/default' ELSE 'world' END)",
    ] {
        assert!(
            !store_source.contains(forbidden_fragment),
            "store.rs should not inline owner/namespace SQL fragment: {forbidden_fragment}"
        );
    }
}

#[tokio::test]
async fn sqlite_round_trips_claim_with_evidence() {
    let context = test_support::new_sqlite_store().await;
    let ids = test_support::seed_event_and_claim(&context.store)
        .await
        .unwrap();
    let claim_row = sqlx::query(
        r#"
        SELECT claim_id, namespace, subject, object, status
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
    assert_eq!(claim_row.get::<String, _>("namespace"), "self");
    assert_eq!(claim_row.get::<String, _>("subject"), "self.role");
    assert_eq!(claim_row.get::<String, _>("object"), "architect");
    assert_eq!(claim_row.get::<String, _>("status"), "active");
    assert_eq!(evidence_rows.len(), 1);
    assert_eq!(evidence_rows[0].get::<String, _>("event_id"), ids.event_id);
}

#[tokio::test]
async fn sqlite_bootstrap_backfills_namespace_for_legacy_claim_rows() {
    let context = test_support::new_legacy_claim_store().await;

    let namespace = sqlx::query_scalar::<_, String>(
        "SELECT namespace FROM claims WHERE claim_id = 'legacy-claim'",
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();
    let namespace_not_null = sqlx::query_scalar::<_, i64>(
        r#"SELECT "notnull" FROM pragma_table_info('claims') WHERE name = 'namespace'"#,
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();
    let legacy_invalid_insert = sqlx::query(
        r#"
        INSERT INTO claims (claim_id, owner, namespace, subject, predicate, object, mode, status)
        VALUES ('legacy-invalid-check', 'self', 'user/default', 'self.role', 'is', 'architect', 'observed', 'active')
        "#,
    )
    .execute(&context.pool)
    .await;

    assert_eq!(namespace, "user/default");
    assert_eq!(namespace_not_null, 1);
    assert!(
        legacy_invalid_insert.is_err(),
        "legacy migrations should restore the same namespace check constraint as fresh databases"
    );
}

#[tokio::test]
async fn sqlite_store_rejects_owner_namespace_mismatch_on_write() {
    let context = test_support::new_sqlite_store().await;

    let result = context
        .store
        .upsert_claim(StoredClaim::new(
            "claim-invalid-write".to_string(),
            ClaimDraft::new_with_namespace(
                Owner::Self_,
                Namespace::for_user("default"),
                "self.role",
                "is",
                "architect",
                Mode::Observed,
            ),
            ClaimStatus::Active,
        ))
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn sqlite_database_rejects_corrupt_namespace_owner_pair_before_read() {
    let context = test_support::new_sqlite_store().await;

    let insert_result = sqlx::query(
        r#"
        INSERT INTO claims (claim_id, owner, namespace, subject, predicate, object, mode, status)
        VALUES ('claim-invalid-read', 'self', 'user/default', 'self.role', 'is', 'architect', 'observed', 'active')
        "#,
    )
    .execute(&context.pool)
    .await;

    assert!(
        insert_result.is_err(),
        "database check constraints should reject corrupt owner/namespace pairs before reads"
    );
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
async fn sqlite_reflection_transactions_replace_identity_and_commitments_atomically() {
    let context = test_support::new_sqlite_store().await;
    test_support::seed_claim(&context.store, "claim-old")
        .await
        .unwrap();
    context
        .store
        .save_identity(IdentityCore::new(vec![
            "identity:self=architect".to_string(),
            "identity:style=rigorous".to_string(),
        ]))
        .await
        .unwrap();

    let mut ok_tx = context.store.begin_reflection_transaction().await.unwrap();
    let loaded_identity = ok_tx.load_identity().await.unwrap();
    let loaded_commitments = ok_tx.load_commitments().await.unwrap();
    assert_eq!(
        loaded_identity.canonical_claims(),
        &[
            "identity:self=architect".to_string(),
            "identity:style=rigorous".to_string(),
        ]
    );
    assert_eq!(
        loaded_commitments,
        vec![agent_llm_mm::domain::commitment::Commitment::new(
            Owner::Self_,
            "forbid:write_identity_core_directly",
        )]
    );

    ok_tx
        .replace_identity(IdentityCore::new(vec![
            "identity:self=staff_architect".to_string(),
            "identity:style=evidence-first".to_string(),
        ]))
        .await
        .unwrap();
    ok_tx
        .replace_commitments(vec![
            agent_llm_mm::domain::commitment::Commitment::new(
                Owner::Self_,
                "prefer:evidence_backed_identity_updates",
            ),
            agent_llm_mm::domain::commitment::Commitment::new(
                Owner::Self_,
                "forbid:write_identity_core_directly",
            ),
        ])
        .await
        .unwrap();
    ok_tx
        .append_reflection(
            StoredReflection::new(
                "refl-audit".to_string(),
                test_support::fixed_now(),
                Reflection::new("replace claim and update deeper self state"),
                Some("claim-old".to_string()),
                Some("claim-new".to_string()),
            )
            .with_supporting_evidence_event_ids(vec![
                "evt-reflection-1".to_string(),
                "evt-reflection-3".to_string(),
            ])
            .with_requested_identity_update(Some(
                agent_llm_mm::domain::reflection::ReflectionIdentityUpdate::new(vec![
                    "identity:self=staff_architect".to_string(),
                    "identity:style=evidence-first".to_string(),
                ]),
            ))
            .with_requested_commitment_updates(Some(vec![
                agent_llm_mm::domain::commitment::Commitment::new(
                    Owner::Self_,
                    "prefer:evidence_backed_identity_updates",
                ),
                agent_llm_mm::domain::commitment::Commitment::new(
                    Owner::Self_,
                    "forbid:write_identity_core_directly",
                ),
            ])),
        )
        .await
        .unwrap();
    ok_tx
        .update_claim_status("claim-old", ClaimStatus::Superseded)
        .await
        .unwrap();
    ok_tx.commit().await.unwrap();

    let persisted_identity = context.store.load_identity().await.unwrap();
    let persisted_commitments = context.store.list_commitments().await.unwrap();
    let reflection_row = sqlx::query(
        r#"
        SELECT
            supporting_evidence_event_ids,
            requested_identity_update,
            requested_commitment_updates
        FROM reflections
        WHERE reflection_id = 'refl-audit'
        "#,
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();

    assert_eq!(
        persisted_identity.canonical_claims(),
        &[
            "identity:self=staff_architect".to_string(),
            "identity:style=evidence-first".to_string(),
        ]
    );
    assert_eq!(
        persisted_commitments,
        vec![
            agent_llm_mm::domain::commitment::Commitment::new(
                Owner::Self_,
                "prefer:evidence_backed_identity_updates",
            ),
            agent_llm_mm::domain::commitment::Commitment::new(
                Owner::Self_,
                "forbid:write_identity_core_directly",
            ),
        ]
    );
    assert_eq!(
        serde_json::from_str::<Vec<String>>(
            &reflection_row.get::<String, _>("supporting_evidence_event_ids"),
        )
        .unwrap(),
        vec![
            "evt-reflection-1".to_string(),
            "evt-reflection-3".to_string(),
        ]
    );
    assert_eq!(
        serde_json::from_str::<agent_llm_mm::domain::reflection::ReflectionIdentityUpdate>(
            &reflection_row.get::<String, _>("requested_identity_update"),
        )
        .unwrap()
        .canonical_claims,
        vec![
            "identity:self=staff_architect".to_string(),
            "identity:style=evidence-first".to_string(),
        ]
    );
    assert_eq!(
        serde_json::from_str::<Vec<agent_llm_mm::domain::commitment::Commitment>>(
            &reflection_row.get::<String, _>("requested_commitment_updates"),
        )
        .unwrap(),
        vec![
            agent_llm_mm::domain::commitment::Commitment::new(
                Owner::Self_,
                "prefer:evidence_backed_identity_updates",
            ),
            agent_llm_mm::domain::commitment::Commitment::new(
                Owner::Self_,
                "forbid:write_identity_core_directly",
            ),
        ]
    );

    let mut failing_tx = context.store.begin_reflection_transaction().await.unwrap();
    failing_tx
        .replace_identity(IdentityCore::new(vec![
            "identity:self=rolled_back".to_string(),
        ]))
        .await
        .unwrap();
    failing_tx
        .replace_commitments(vec![agent_llm_mm::domain::commitment::Commitment::new(
            Owner::Self_,
            "prefer:should_roll_back",
        )])
        .await
        .unwrap();
    failing_tx
        .append_reflection(
            StoredReflection::new(
                "refl-audit-rollback".to_string(),
                test_support::fixed_now(),
                Reflection::new("this deeper update should roll back"),
                Some("claim-missing".to_string()),
                None,
            )
            .with_supporting_evidence_event_ids(vec!["evt-reflection-9".to_string()]),
        )
        .await
        .unwrap();

    let update_result = failing_tx
        .update_claim_status("claim-missing", ClaimStatus::Superseded)
        .await;
    assert!(update_result.is_err());
    let commit_result = failing_tx.commit().await;
    assert!(commit_result.is_err());

    let rolled_back_identity = context.store.load_identity().await.unwrap();
    let rolled_back_commitments = context.store.list_commitments().await.unwrap();
    let rolled_back_reflection_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reflections WHERE reflection_id = 'refl-audit-rollback'",
    )
    .fetch_one(&context.pool)
    .await
    .unwrap();

    assert_eq!(rolled_back_identity, persisted_identity);
    assert_eq!(rolled_back_commitments, persisted_commitments);
    assert_eq!(rolled_back_reflection_count, 0);
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

#[tokio::test]
async fn sqlite_bootstrap_migrates_legacy_reflections_table_with_audit_columns() {
    let path = std::env::temp_dir().join(format!(
        "agent-llm-mm-legacy-reflections-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    let database_url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    std::fs::File::create(&path).unwrap();
    let pool = SqlitePool::connect(&database_url).await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE reflections (
            reflection_id TEXT PRIMARY KEY,
            recorded_at TEXT NOT NULL,
            summary TEXT NOT NULL,
            superseded_claim_id TEXT,
            replacement_claim_id TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO reflections (
            reflection_id,
            recorded_at,
            summary,
            superseded_claim_id,
            replacement_claim_id
        )
        VALUES ('legacy-refl', '2026-03-23T10:00:00Z', 'legacy reflection row', 'claim-old', NULL)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    drop(pool);

    let _store = agent_llm_mm::adapters::sqlite::SqliteStore::bootstrap(&database_url)
        .await
        .unwrap();
    let migrated_pool = SqlitePool::connect(&database_url).await.unwrap();

    let columns = sqlx::query("SELECT name FROM pragma_table_info('reflections') ORDER BY cid")
        .fetch_all(&migrated_pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<Vec<_>>();
    let trigger_ledger_table_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'reflection_trigger_ledger'",
    )
    .fetch_one(&migrated_pool)
    .await
    .unwrap();
    let legacy_summary = sqlx::query_scalar::<_, String>(
        "SELECT summary FROM reflections WHERE reflection_id = 'legacy-refl'",
    )
    .fetch_one(&migrated_pool)
    .await
    .unwrap();

    assert!(columns.contains(&"supporting_evidence_event_ids".to_string()));
    assert!(columns.contains(&"requested_identity_update".to_string()));
    assert!(columns.contains(&"requested_commitment_updates".to_string()));
    assert_eq!(trigger_ledger_table_count, 1);
    assert_eq!(legacy_summary, "legacy reflection row");
}

#[tokio::test]
async fn sqlite_trigger_ledger_records_namespace_periodic_watermark_and_cooldown() {
    let context = test_support::new_sqlite_store().await;
    let trigger_key = "periodic:project/agent-llm-mm";
    let first_handled_at = test_support::fixed_now();
    let second_handled_at = first_handled_at + chrono::Duration::minutes(15);
    let second_cooldown_until = second_handled_at + chrono::Duration::hours(6);

    context
        .store
        .record_trigger_attempt(StoredTriggerLedgerEntry {
            ledger_id: "ledger-1".to_string(),
            trigger_type: TriggerType::Periodic,
            namespace: Namespace::for_project("agent-llm-mm"),
            trigger_key: trigger_key.to_string(),
            status: TriggerLedgerStatus::Handled,
            evidence_window: vec!["event:1".to_string()],
            handled_at: Some(first_handled_at),
            cooldown_until: Some(first_handled_at + chrono::Duration::hours(1)),
            episode_watermark: Some(20),
            reflection_id: Some("refl-1".to_string()),
        })
        .await
        .unwrap();

    context
        .store
        .record_trigger_attempt(StoredTriggerLedgerEntry {
            ledger_id: "ledger-2".to_string(),
            trigger_type: TriggerType::Periodic,
            namespace: Namespace::for_project("agent-llm-mm"),
            trigger_key: trigger_key.to_string(),
            status: TriggerLedgerStatus::Suppressed,
            evidence_window: vec!["event:2".to_string(), "event:3".to_string()],
            handled_at: Some(second_handled_at),
            cooldown_until: Some(second_cooldown_until),
            episode_watermark: Some(42),
            reflection_id: None,
        })
        .await
        .unwrap();

    let entry = context
        .store
        .latest_trigger_entry(trigger_key)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(entry.namespace.as_str(), "project/agent-llm-mm");
    assert_eq!(entry.status, TriggerLedgerStatus::Suppressed);
    assert_eq!(entry.episode_watermark, Some(42));
    assert_eq!(entry.cooldown_until, Some(second_cooldown_until));
    assert_eq!(entry.handled_at, Some(second_handled_at));
    assert_eq!(
        entry.evidence_window,
        vec!["event:2".to_string(), "event:3".to_string()]
    );
}

#[cfg(target_pointer_width = "64")]
#[tokio::test]
async fn sqlite_trigger_ledger_rejects_episode_watermark_above_i64_max() {
    let context = test_support::new_sqlite_store().await;

    let result = context
        .store
        .record_trigger_attempt(StoredTriggerLedgerEntry {
            ledger_id: "ledger-overflow".to_string(),
            trigger_type: TriggerType::Periodic,
            namespace: Namespace::for_project("agent-llm-mm"),
            trigger_key: "periodic:project/agent-llm-mm".to_string(),
            status: TriggerLedgerStatus::Handled,
            evidence_window: Vec::new(),
            handled_at: Some(test_support::fixed_now()),
            cooldown_until: None,
            episode_watermark: Some(i64::MAX as u64 + 1),
            reflection_id: None,
        })
        .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
}

#[tokio::test]
async fn sqlite_trigger_ledger_latest_entry_uses_append_order_not_handled_at() {
    let context = test_support::new_sqlite_store().await;
    let trigger_key = "periodic:project/agent-llm-mm";
    let later_business_time = test_support::fixed_now() + chrono::Duration::hours(4);
    let earlier_business_time = test_support::fixed_now() - chrono::Duration::hours(2);

    context
        .store
        .record_trigger_attempt(StoredTriggerLedgerEntry {
            ledger_id: "ledger-business-late".to_string(),
            trigger_type: TriggerType::Periodic,
            namespace: Namespace::for_project("agent-llm-mm"),
            trigger_key: trigger_key.to_string(),
            status: TriggerLedgerStatus::Handled,
            evidence_window: vec!["event:late".to_string()],
            handled_at: Some(later_business_time),
            cooldown_until: Some(later_business_time + chrono::Duration::hours(1)),
            episode_watermark: Some(100),
            reflection_id: Some("refl-late".to_string()),
        })
        .await
        .unwrap();

    context
        .store
        .record_trigger_attempt(StoredTriggerLedgerEntry {
            ledger_id: "ledger-appended-last".to_string(),
            trigger_type: TriggerType::Periodic,
            namespace: Namespace::for_project("agent-llm-mm"),
            trigger_key: trigger_key.to_string(),
            status: TriggerLedgerStatus::Suppressed,
            evidence_window: vec!["event:appended-last".to_string()],
            handled_at: Some(earlier_business_time),
            cooldown_until: None,
            episode_watermark: Some(101),
            reflection_id: None,
        })
        .await
        .unwrap();

    let entry = context
        .store
        .latest_trigger_entry(trigger_key)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(entry.ledger_id, "ledger-appended-last");
    assert_eq!(entry.handled_at, Some(earlier_business_time));
    assert_eq!(entry.status, TriggerLedgerStatus::Suppressed);
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

    pub async fn new_legacy_claim_store() -> TestContext {
        let path = std::env::temp_dir().join(format!(
            "agent-llm-mm-legacy-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let database_url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
        std::fs::File::create(&path).unwrap();
        let pool = SqlitePool::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE claims (
                claim_id TEXT PRIMARY KEY,
                owner TEXT NOT NULL,
                subject TEXT NOT NULL,
                predicate TEXT NOT NULL,
                object TEXT NOT NULL,
                mode TEXT NOT NULL,
                status TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO claims (claim_id, owner, subject, predicate, object, mode, status)
            VALUES ('legacy-claim', 'user', 'user.preference', 'likes', 'concise', 'observed', 'active')
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        drop(pool);

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
