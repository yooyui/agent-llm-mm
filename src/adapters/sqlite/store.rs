use std::{fs, path::PathBuf, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    Row, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePool},
};

use crate::{
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        event::Event,
        identity_core::IdentityCore,
        self_revision::TriggerType,
        types::{EventKind, Mode, Namespace, Owner},
    },
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, CommitmentStore, EpisodeStore, EventStore, EvidenceQuery,
        IdentityStore, IngestTransaction, IngestTransactionRunner, ReflectionStore,
        ReflectionTransaction, ReflectionTransactionRunner, StoredClaim, StoredEvent,
        StoredReflection, StoredTriggerLedgerEntry, TriggerLedgerStatus, TriggerLedgerStore,
    },
};

use super::schema::{
    OWNER_NAMESPACE_SCOPE_CONSTRAINT_NAME, claims_table_sql, init_sql,
    legacy_namespace_backfill_expression,
};

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn bootstrap(database_url: &str) -> Result<Self, AppError> {
        ensure_sqlite_parent_directory(database_url)?;

        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(|error| AppError::Message(error.to_string()))?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = map_sqlite(SqlitePool::connect_with(options).await)?;
        let mut connection = map_sqlite(pool.acquire().await)?;
        let init_sql = init_sql();

        for statement in init_sql.split(';').filter(|part| !part.trim().is_empty()) {
            map_sqlite(sqlx::query(statement).execute(connection.as_mut()).await)?;
        }
        ensure_claims_namespace_column(connection.as_mut()).await?;
        ensure_reflection_audit_columns(connection.as_mut()).await?;
        seed_baseline_commitments(connection.as_mut()).await?;

        Ok(Self { pool })
    }
}

fn ensure_sqlite_parent_directory(database_url: &str) -> Result<(), AppError> {
    let Some(path) = sqlite_file_path(database_url) else {
        return Ok(());
    };

    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    fs::create_dir_all(parent).map_err(|error| {
        AppError::Message(format!(
            "failed to create sqlite parent directory {}: {error}",
            parent.display()
        ))
    })?;

    Ok(())
}

fn sqlite_file_path(database_url: &str) -> Option<PathBuf> {
    let path = database_url.strip_prefix("sqlite://")?;
    let path = path.split_once('?').map_or(path, |(path, _)| path);
    if path.is_empty() || path == ":memory:" {
        return None;
    }

    #[cfg(windows)]
    let path = normalize_windows_sqlite_path(path);

    #[cfg(not(windows))]
    let path = path.to_string();

    Some(PathBuf::from(path))
}

#[cfg(windows)]
fn normalize_windows_sqlite_path(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':' {
        return path[1..].to_string();
    }

    path.to_string()
}

#[async_trait]
impl EventStore for SqliteStore {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError> {
        insert_event(&self.pool, &event).await
    }

    async fn list_event_references(&self) -> Result<Vec<String>, AppError> {
        let rows = map_sqlite(
            sqlx::query("SELECT event_id FROM events ORDER BY rowid")
                .fetch_all(&self.pool)
                .await,
        )?;

        Ok(rows
            .into_iter()
            .map(|row| format!("event:{}", row.get::<String, _>("event_id")))
            .collect())
    }

    async fn query_evidence_event_ids(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError> {
        let mut sql =
            String::from("SELECT event_id, recorded_at, owner, kind, summary FROM events");
        let mut predicates = Vec::new();

        if query.owner.is_some() {
            predicates.push("owner = ?");
        }

        if query.kind.is_some() {
            predicates.push("kind = ?");
        }

        if !predicates.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&predicates.join(" AND "));
        }

        sql.push_str(" ORDER BY recorded_at DESC, rowid DESC LIMIT ?");

        let mut rows = {
            let mut query_builder = sqlx::query(&sql);

            if let Some(owner) = query.owner {
                query_builder = query_builder.bind(owner_as_str(owner));
            }

            if let Some(kind) = query.kind {
                query_builder = query_builder.bind(event_kind_as_str(kind));
            }

            let limit = query.limit.unwrap_or(10);
            let limit = i64::try_from(limit).map_err(|_| {
                AppError::InvalidParams(
                    "evidence query limit exceeds the supported maximum".to_string(),
                )
            })?;
            query_builder = query_builder.bind(limit);

            map_sqlite(query_builder.fetch_all(&self.pool).await)?
        };

        Ok(rows
            .drain(..)
            .map(|row| row.get::<String, _>("event_id"))
            .collect())
    }

    async fn has_event(&self, event_id: &str) -> Result<bool, AppError> {
        let count = map_sqlite(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM events WHERE event_id = ?")
                .bind(event_id)
                .fetch_one(&self.pool)
                .await,
        )?;

        Ok(count > 0)
    }
}

#[async_trait]
impl ClaimStore for SqliteStore {
    async fn upsert_claim(&self, claim: StoredClaim) -> Result<(), AppError> {
        upsert_claim_row(&self.pool, &claim).await
    }

    async fn link_evidence(&self, claim_id: String, event_id: String) -> Result<(), AppError> {
        insert_evidence_link(&self.pool, &claim_id, &event_id).await
    }

    async fn list_active_claims(&self) -> Result<Vec<StoredClaim>, AppError> {
        let rows = map_sqlite(
            sqlx::query(
                r#"
                SELECT claim_id, owner, subject, predicate, object, mode, status
                , namespace
                FROM claims
                WHERE status = ?
                ORDER BY rowid
                "#,
            )
            .bind(ClaimStatus::Active.as_str())
            .fetch_all(&self.pool)
            .await,
        )?;

        rows.into_iter()
            .map(|row| stored_claim_from_row(&row))
            .collect()
    }

    async fn update_claim_status(
        &self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        update_claim_status_row(&self.pool, claim_id, status).await
    }
}

#[async_trait]
impl EpisodeStore for SqliteStore {
    async fn record_event_in_episode(
        &self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError> {
        insert_episode_event(&self.pool, &episode_reference, &event_id).await
    }

    async fn list_episode_references(&self) -> Result<Vec<String>, AppError> {
        let rows = map_sqlite(
            sqlx::query(
                r#"
                SELECT episode_reference
                FROM episode_events
                GROUP BY episode_reference
                ORDER BY MIN(rowid)
                "#,
            )
            .fetch_all(&self.pool)
            .await,
        )?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<String, _>("episode_reference"))
            .collect())
    }
}

#[async_trait]
impl ReflectionStore for SqliteStore {
    async fn append_reflection(&self, reflection: StoredReflection) -> Result<(), AppError> {
        insert_reflection(&self.pool, &reflection).await
    }
}

#[async_trait]
impl TriggerLedgerStore for SqliteStore {
    async fn record_trigger_attempt(
        &self,
        entry: StoredTriggerLedgerEntry,
    ) -> Result<(), AppError> {
        insert_trigger_ledger_entry(&self.pool, &entry).await
    }

    async fn latest_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError> {
        // Task 2 defines "latest" as the last recorded attempt for the
        // canonical trigger key, so append order wins over business timestamps.
        let row = map_sqlite(
            sqlx::query(
                r#"
                SELECT
                    ledger_id,
                    trigger_type,
                    namespace,
                    trigger_key,
                    status,
                    evidence_window,
                    handled_at,
                    cooldown_until,
                    episode_watermark,
                    reflection_id
                FROM reflection_trigger_ledger
                WHERE trigger_key = ?
                ORDER BY rowid DESC
                LIMIT 1
                "#,
            )
            .bind(trigger_key)
            .fetch_optional(&self.pool)
            .await,
        )?;

        row.as_ref()
            .map(stored_trigger_ledger_entry_from_row)
            .transpose()
    }
}

#[async_trait]
impl IdentityStore for SqliteStore {
    async fn load_identity(&self) -> Result<IdentityCore, AppError> {
        load_identity_rows(&self.pool).await
    }

    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError> {
        let mut tx = map_sqlite(self.pool.begin().await)?;
        replace_identity_rows(tx.as_mut(), &identity).await?;
        map_sqlite(tx.commit().await)?;
        Ok(())
    }
}

#[async_trait]
impl CommitmentStore for SqliteStore {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, AppError> {
        load_commitment_rows(&self.pool).await
    }
}

#[async_trait]
impl IngestTransactionRunner for SqliteStore {
    async fn begin_ingest_transaction(
        &self,
    ) -> Result<Box<dyn IngestTransaction + Send + '_>, AppError> {
        let transaction = map_sqlite(self.pool.begin().await)?;

        Ok(Box::new(SqliteIngestTransaction {
            transaction: Some(transaction),
            poisoned: false,
        }))
    }
}

#[async_trait]
impl ReflectionTransactionRunner for SqliteStore {
    async fn begin_reflection_transaction(
        &self,
    ) -> Result<Box<dyn ReflectionTransaction + Send + '_>, AppError> {
        let transaction = map_sqlite(self.pool.begin().await)?;

        Ok(Box::new(SqliteReflectionTransaction {
            transaction: Some(transaction),
            poisoned: false,
        }))
    }
}

struct SqliteIngestTransaction<'a> {
    transaction: Option<sqlx::Transaction<'a, Sqlite>>,
    poisoned: bool,
}

#[async_trait]
impl IngestTransaction for SqliteIngestTransaction<'_> {
    async fn append_event(&mut self, event: StoredEvent) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            insert_event(transaction.as_mut(), &event).await
        };
        self.note_result(result)
    }

    async fn record_event_in_episode(
        &mut self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            insert_episode_event(transaction.as_mut(), &episode_reference, &event_id).await
        };
        self.note_result(result)
    }

    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            upsert_claim_row(transaction.as_mut(), &claim).await
        };
        self.note_result(result)
    }

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            insert_evidence_link(transaction.as_mut(), &claim_id, &event_id).await
        };
        self.note_result(result)
    }

    async fn commit(mut self: Box<Self>) -> Result<(), AppError> {
        if self.poisoned {
            return Err(AppError::Message(
                "transaction is poisoned and cannot be committed".to_string(),
            ));
        }

        let transaction = self
            .transaction
            .take()
            .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
        map_sqlite(transaction.commit().await)?;
        Ok(())
    }
}

impl SqliteIngestTransaction<'_> {
    fn ensure_writable(&self) -> Result<(), AppError> {
        if self.poisoned {
            return Err(AppError::Message(
                "transaction is poisoned and cannot accept more writes".to_string(),
            ));
        }

        Ok(())
    }

    fn note_result<T>(&mut self, result: Result<T, AppError>) -> Result<T, AppError> {
        if result.is_err() {
            self.poisoned = true;
        }

        result
    }
}

struct SqliteReflectionTransaction<'a> {
    transaction: Option<sqlx::Transaction<'a, Sqlite>>,
    poisoned: bool,
}

#[async_trait]
impl ReflectionTransaction for SqliteReflectionTransaction<'_> {
    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            upsert_claim_row(transaction.as_mut(), &claim).await
        };
        self.note_result(result)
    }

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            insert_evidence_link(transaction.as_mut(), &claim_id, &event_id).await
        };
        self.note_result(result)
    }

    async fn append_reflection(&mut self, reflection: StoredReflection) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            insert_reflection(transaction.as_mut(), &reflection).await
        };
        self.note_result(result)
    }

    async fn append_trigger_ledger(
        &mut self,
        entry: StoredTriggerLedgerEntry,
    ) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            insert_trigger_ledger_entry(transaction.as_mut(), &entry).await
        };
        self.note_result(result)
    }

    async fn load_identity(&mut self) -> Result<IdentityCore, AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            load_identity_rows(transaction.as_mut()).await
        };
        self.note_result(result)
    }

    async fn replace_identity(&mut self, identity: IdentityCore) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            replace_identity_rows(transaction.as_mut(), &identity).await
        };
        self.note_result(result)
    }

    async fn load_commitments(&mut self) -> Result<Vec<Commitment>, AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            load_commitment_rows(transaction.as_mut()).await
        };
        self.note_result(result)
    }

    async fn replace_commitments(&mut self, commitments: Vec<Commitment>) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            replace_commitment_rows(transaction.as_mut(), &commitments).await
        };
        self.note_result(result)
    }

    async fn update_claim_status(
        &mut self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        self.ensure_writable()?;

        let result = {
            let transaction = self
                .transaction
                .as_mut()
                .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
            update_claim_status_row(transaction.as_mut(), claim_id, status).await
        };
        self.note_result(result)
    }

    async fn commit(mut self: Box<Self>) -> Result<(), AppError> {
        if self.poisoned {
            return Err(AppError::Message(
                "transaction is poisoned and cannot be committed".to_string(),
            ));
        }

        let transaction = self
            .transaction
            .take()
            .ok_or_else(|| AppError::Message("transaction already closed".to_string()))?;
        map_sqlite(transaction.commit().await)?;
        Ok(())
    }
}

impl SqliteReflectionTransaction<'_> {
    fn ensure_writable(&self) -> Result<(), AppError> {
        if self.poisoned {
            return Err(AppError::Message(
                "transaction is poisoned and cannot accept more writes".to_string(),
            ));
        }

        Ok(())
    }

    fn note_result<T>(&mut self, result: Result<T, AppError>) -> Result<T, AppError> {
        if result.is_err() {
            self.poisoned = true;
        }

        result
    }
}

async fn insert_event<'e, E>(executor: E, event: &StoredEvent) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO events (event_id, recorded_at, owner, kind, summary)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.event_id)
        .bind(event.recorded_at.to_rfc3339())
        .bind(owner_as_str(event.event.owner()))
        .bind(event_kind_as_str(event.event.kind()))
        .bind(event.event.summary())
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn upsert_claim_row<'e, E>(executor: E, claim: &StoredClaim) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    claim.claim.validate_namespace_owner().map_err(|error| {
        AppError::Message(format!("invalid claim namespace mapping: {error:?}"))
    })?;

    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO claims (claim_id, owner, namespace, subject, predicate, object, mode, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(claim_id) DO UPDATE SET
                owner = excluded.owner,
                namespace = excluded.namespace,
                subject = excluded.subject,
                predicate = excluded.predicate,
                object = excluded.object,
                mode = excluded.mode,
                status = excluded.status
            "#,
        )
        .bind(&claim.claim_id)
        .bind(owner_as_str(claim.claim.owner()))
        .bind(claim.claim.namespace().as_str())
        .bind(claim.claim.subject())
        .bind(claim.claim.predicate())
        .bind(claim.claim.object())
        .bind(mode_as_str(claim.claim.mode()))
        .bind(claim.status.as_str())
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn insert_evidence_link<'e, E>(
    executor: E,
    claim_id: &str,
    event_id: &str,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO evidence_links (claim_id, event_id)
            VALUES (?, ?)
            "#,
        )
        .bind(claim_id)
        .bind(event_id)
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn insert_episode_event<'e, E>(
    executor: E,
    episode_reference: &str,
    event_id: &str,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO episode_events (episode_reference, event_id)
            VALUES (?, ?)
            "#,
        )
        .bind(episode_reference)
        .bind(event_id)
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn insert_reflection<'e, E>(
    executor: E,
    reflection: &StoredReflection,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let supporting_evidence_event_ids = serialize_json(&reflection.supporting_evidence_event_ids)?;
    let requested_identity_update = serialize_optional_json(&reflection.requested_identity_update)?;
    let requested_commitment_updates =
        serialize_optional_json(&reflection.requested_commitment_updates)?;

    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO reflections (
                reflection_id,
                recorded_at,
                summary,
                superseded_claim_id,
                replacement_claim_id,
                supporting_evidence_event_ids,
                requested_identity_update,
                requested_commitment_updates
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&reflection.reflection_id)
        .bind(reflection.recorded_at.to_rfc3339())
        .bind(reflection.reflection.summary())
        .bind(&reflection.superseded_claim_id)
        .bind(&reflection.replacement_claim_id)
        .bind(supporting_evidence_event_ids)
        .bind(requested_identity_update)
        .bind(requested_commitment_updates)
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn insert_trigger_ledger_entry<'e, E>(
    executor: E,
    entry: &StoredTriggerLedgerEntry,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO reflection_trigger_ledger (
                ledger_id, trigger_type, namespace, trigger_key, status,
                evidence_window, handled_at, cooldown_until, episode_watermark, reflection_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.ledger_id)
        .bind(trigger_type_as_str(entry.trigger_type))
        .bind(entry.namespace.as_str())
        .bind(&entry.trigger_key)
        .bind(entry.status.as_str())
        .bind(serialize_json(&entry.evidence_window)?)
        .bind(entry.handled_at.map(|value| value.to_rfc3339()))
        .bind(entry.cooldown_until.map(|value| value.to_rfc3339()))
        .bind(serialize_episode_watermark(entry.episode_watermark)?)
        .bind(&entry.reflection_id)
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn update_claim_status_row<'e, E>(
    executor: E,
    claim_id: &str,
    status: ClaimStatus,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let result = map_sqlite(
        sqlx::query(
            r#"
            UPDATE claims
            SET status = ?
            WHERE claim_id = ?
            "#,
        )
        .bind(status.as_str())
        .bind(claim_id)
        .execute(executor)
        .await,
    )?;

    if result.rows_affected() == 0 {
        return Err(AppError::Message(format!(
            "cannot update missing claim: {claim_id}"
        )));
    }

    Ok(())
}

async fn seed_baseline_commitments<'e, E>(executor: E) -> Result<(), AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    map_sqlite(
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO commitments (description, owner)
            VALUES (?, ?)
            "#,
        )
        .bind("forbid:write_identity_core_directly")
        .bind("self")
        .execute(executor)
        .await,
    )?;

    Ok(())
}

async fn ensure_claims_namespace_column(
    connection: &mut sqlx::SqliteConnection,
) -> Result<(), AppError> {
    let namespace_column_exists = map_sqlite(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM pragma_table_info('claims') WHERE name = 'namespace'",
        )
        .fetch_one(&mut *connection)
        .await,
    )? > 0;
    let namespace_is_not_null = if namespace_column_exists {
        map_sqlite(
            sqlx::query_scalar::<_, i64>(
                r#"SELECT "notnull" FROM pragma_table_info('claims') WHERE name = 'namespace'"#,
            )
            .fetch_one(&mut *connection)
            .await,
        )? == 1
    } else {
        false
    };
    let claims_has_scope_check = if namespace_column_exists {
        map_sqlite(
            sqlx::query_scalar::<_, String>(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'claims'",
            )
            .fetch_one(&mut *connection)
            .await,
        )?
        .contains(OWNER_NAMESPACE_SCOPE_CONSTRAINT_NAME)
    } else {
        false
    };

    if !namespace_column_exists || !namespace_is_not_null || !claims_has_scope_check {
        rebuild_claims_table_with_namespace(connection, namespace_column_exists).await?;
    }

    Ok(())
}

async fn ensure_reflection_audit_columns(
    connection: &mut sqlx::SqliteConnection,
) -> Result<(), AppError> {
    let columns = map_sqlite(
        sqlx::query("SELECT name FROM pragma_table_info('reflections')")
            .fetch_all(&mut *connection)
            .await,
    )?
    .into_iter()
    .map(|row| row.get::<String, _>("name"))
    .collect::<Vec<_>>();

    if !columns.contains(&"supporting_evidence_event_ids".to_string()) {
        map_sqlite(
            sqlx::query(
                "ALTER TABLE reflections ADD COLUMN supporting_evidence_event_ids TEXT NOT NULL DEFAULT '[]'",
            )
            .execute(&mut *connection)
            .await,
        )?;
    }

    if !columns.contains(&"requested_identity_update".to_string()) {
        map_sqlite(
            sqlx::query("ALTER TABLE reflections ADD COLUMN requested_identity_update TEXT")
                .execute(&mut *connection)
                .await,
        )?;
    }

    if !columns.contains(&"requested_commitment_updates".to_string()) {
        map_sqlite(
            sqlx::query("ALTER TABLE reflections ADD COLUMN requested_commitment_updates TEXT")
                .execute(&mut *connection)
                .await,
        )?;
    }

    Ok(())
}

async fn rebuild_claims_table_with_namespace(
    connection: &mut sqlx::SqliteConnection,
    legacy_table_has_namespace: bool,
) -> Result<(), AppError> {
    let create_claims_table_sql = claims_table_sql(false);
    let namespace_expression = legacy_namespace_backfill_expression(legacy_table_has_namespace);
    let copy_sql = format!(
        r#"
        INSERT INTO claims (claim_id, owner, namespace, subject, predicate, object, mode, status)
        SELECT claim_id, owner, {namespace_expression}, subject, predicate, object, mode, status
        FROM claims_legacy
        "#
    );

    map_sqlite(
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *connection)
            .await,
    )?;
    map_sqlite(
        sqlx::query("ALTER TABLE claims RENAME TO claims_legacy")
            .execute(&mut *connection)
            .await,
    )?;
    map_sqlite(
        sqlx::query(&create_claims_table_sql)
            .execute(&mut *connection)
            .await,
    )?;
    map_sqlite(sqlx::query(&copy_sql).execute(&mut *connection).await)?;
    map_sqlite(
        sqlx::query("DROP TABLE claims_legacy")
            .execute(&mut *connection)
            .await,
    )?;
    map_sqlite(
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *connection)
            .await,
    )?;

    Ok(())
}

async fn load_identity_rows<'e, E>(executor: E) -> Result<IdentityCore, AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let rows = map_sqlite(
        sqlx::query(
            r#"
            SELECT claim
            FROM identity_claims
            ORDER BY position
            "#,
        )
        .fetch_all(executor)
        .await,
    )?;

    if rows.is_empty() {
        return Err(AppError::Message("missing identity".to_string()));
    }

    Ok(IdentityCore::new(
        rows.into_iter().map(|row| row.get("claim")).collect(),
    ))
}

async fn replace_identity_rows(
    connection: &mut sqlx::SqliteConnection,
    identity: &IdentityCore,
) -> Result<(), AppError> {
    map_sqlite(
        sqlx::query("DELETE FROM identity_claims")
            .execute(&mut *connection)
            .await,
    )?;

    for (position, claim) in identity.canonical_claims().iter().enumerate() {
        map_sqlite(
            sqlx::query("INSERT INTO identity_claims (position, claim) VALUES (?, ?)")
                .bind(position as i64)
                .bind(claim)
                .execute(&mut *connection)
                .await,
        )?;
    }

    Ok(())
}

async fn load_commitment_rows<'e, E>(executor: E) -> Result<Vec<Commitment>, AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let rows = map_sqlite(
        sqlx::query(
            r#"
            SELECT owner, description
            FROM commitments
            ORDER BY rowid
            "#,
        )
        .fetch_all(executor)
        .await,
    )?;

    rows.into_iter()
        .map(|row| {
            Ok(Commitment::new(
                parse_owner(&row.get::<String, _>("owner"))?,
                row.get::<String, _>("description"),
            ))
        })
        .collect()
}

async fn replace_commitment_rows(
    connection: &mut sqlx::SqliteConnection,
    commitments: &[Commitment],
) -> Result<(), AppError> {
    map_sqlite(
        sqlx::query("DELETE FROM commitments")
            .execute(&mut *connection)
            .await,
    )?;

    for commitment in commitments {
        map_sqlite(
            sqlx::query(
                r#"
                INSERT INTO commitments (description, owner)
                VALUES (?, ?)
                "#,
            )
            .bind(commitment.description())
            .bind(owner_as_str(commitment.owner()))
            .execute(&mut *connection)
            .await,
        )?;
    }

    Ok(())
}

fn serialize_json<T>(value: &T) -> Result<String, AppError>
where
    T: serde::Serialize,
{
    serde_json::to_string(value).map_err(|error| AppError::Message(error.to_string()))
}

fn serialize_optional_json<T>(value: &Option<T>) -> Result<Option<String>, AppError>
where
    T: serde::Serialize,
{
    value.as_ref().map(serialize_json).transpose()
}

fn deserialize_json<T>(value: &str) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(value).map_err(|error| AppError::Message(error.to_string()))
}

fn serialize_episode_watermark(value: Option<u64>) -> Result<Option<i64>, AppError> {
    value
        .map(|value| {
            i64::try_from(value).map_err(|_| {
                AppError::InvalidParams(format!(
                    "episode watermark {value} exceeds sqlite INTEGER range"
                ))
            })
        })
        .transpose()
}

fn map_sqlite<T>(result: Result<T, sqlx::Error>) -> Result<T, AppError> {
    result.map_err(|error| AppError::Message(error.to_string()))
}

fn stored_claim_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<StoredClaim, AppError> {
    let claim = ClaimDraft::new(
        parse_owner(&row.get::<String, _>("owner"))?,
        row.get::<String, _>("subject"),
        row.get::<String, _>("predicate"),
        row.get::<String, _>("object"),
        parse_mode(&row.get::<String, _>("mode"))?,
    )
    .with_namespace(parse_namespace(&row.get::<String, _>("namespace"))?);
    claim.validate_namespace_owner().map_err(|error| {
        AppError::Message(format!("invalid stored claim namespace mapping: {error:?}"))
    })?;

    Ok(StoredClaim::new(
        row.get("claim_id"),
        claim,
        parse_claim_status(&row.get::<String, _>("status"))?,
    ))
}

#[allow(dead_code)]
fn stored_event_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<StoredEvent, AppError> {
    Ok(StoredEvent::new(
        row.get("event_id"),
        parse_timestamp(&row.get::<String, _>("recorded_at"))?,
        Event::new(
            parse_owner(&row.get::<String, _>("owner"))?,
            parse_event_kind(&row.get::<String, _>("kind"))?,
            row.get::<String, _>("summary"),
        ),
    ))
}

fn stored_trigger_ledger_entry_from_row(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<StoredTriggerLedgerEntry, AppError> {
    let evidence_window = deserialize_json(&row.get::<String, _>("evidence_window"))?;
    let handled_at = parse_optional_timestamp(row.get::<Option<String>, _>("handled_at"))?;
    let cooldown_until = parse_optional_timestamp(row.get::<Option<String>, _>("cooldown_until"))?;
    let episode_watermark = row
        .get::<Option<i64>, _>("episode_watermark")
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                AppError::Message(format!("invalid negative episode watermark: {value}"))
            })
        })
        .transpose()?;

    Ok(StoredTriggerLedgerEntry {
        ledger_id: row.get("ledger_id"),
        trigger_type: parse_trigger_type(&row.get::<String, _>("trigger_type"))?,
        namespace: parse_namespace(&row.get::<String, _>("namespace"))?,
        trigger_key: row.get("trigger_key"),
        status: parse_trigger_ledger_status(&row.get::<String, _>("status"))?,
        evidence_window,
        handled_at,
        cooldown_until,
        episode_watermark,
        reflection_id: row.get("reflection_id"),
    })
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| AppError::Message(error.to_string()))
}

fn parse_optional_timestamp(value: Option<String>) -> Result<Option<DateTime<Utc>>, AppError> {
    value.as_deref().map(parse_timestamp).transpose()
}

fn owner_as_str(owner: Owner) -> &'static str {
    match owner {
        Owner::Self_ => "self",
        Owner::User => "user",
        Owner::World => "world",
        Owner::Unknown => "unknown",
    }
}

fn parse_owner(value: &str) -> Result<Owner, AppError> {
    match value {
        "self" => Ok(Owner::Self_),
        "user" => Ok(Owner::User),
        "world" => Ok(Owner::World),
        "unknown" => Ok(Owner::Unknown),
        _ => Err(AppError::Message(format!("unknown owner: {value}"))),
    }
}

fn mode_as_str(mode: Mode) -> &'static str {
    match mode {
        Mode::Observed => "observed",
        Mode::Said => "said",
        Mode::Acted => "acted",
        Mode::Inferred => "inferred",
        Mode::Draft => "draft",
    }
}

fn parse_mode(value: &str) -> Result<Mode, AppError> {
    match value {
        "observed" => Ok(Mode::Observed),
        "said" => Ok(Mode::Said),
        "acted" => Ok(Mode::Acted),
        "inferred" => Ok(Mode::Inferred),
        "draft" => Ok(Mode::Draft),
        _ => Err(AppError::Message(format!("unknown mode: {value}"))),
    }
}

fn parse_namespace(value: &str) -> Result<Namespace, AppError> {
    Namespace::parse(value).map_err(|error| {
        AppError::Message(format!("invalid stored namespace `{value}`: {error:?}"))
    })
}

fn trigger_type_as_str(trigger_type: TriggerType) -> &'static str {
    match trigger_type {
        TriggerType::Conflict => "conflict",
        TriggerType::Failure => "failure",
        TriggerType::Periodic => "periodic",
    }
}

fn parse_trigger_type(value: &str) -> Result<TriggerType, AppError> {
    match value {
        "conflict" => Ok(TriggerType::Conflict),
        "failure" => Ok(TriggerType::Failure),
        "periodic" => Ok(TriggerType::Periodic),
        _ => Err(AppError::Message(format!("unknown trigger type: {value}"))),
    }
}

fn event_kind_as_str(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Observation => "observation",
        EventKind::Conversation => "conversation",
        EventKind::Action => "action",
        EventKind::Reflection => "reflection",
    }
}

fn parse_event_kind(value: &str) -> Result<EventKind, AppError> {
    match value {
        "observation" => Ok(EventKind::Observation),
        "conversation" => Ok(EventKind::Conversation),
        "action" => Ok(EventKind::Action),
        "reflection" => Ok(EventKind::Reflection),
        _ => Err(AppError::Message(format!("unknown event kind: {value}"))),
    }
}

fn parse_claim_status(value: &str) -> Result<ClaimStatus, AppError> {
    match value {
        "active" => Ok(ClaimStatus::Active),
        "disputed" => Ok(ClaimStatus::Disputed),
        "superseded" => Ok(ClaimStatus::Superseded),
        _ => Err(AppError::Message(format!("unknown claim status: {value}"))),
    }
}

fn parse_trigger_ledger_status(value: &str) -> Result<TriggerLedgerStatus, AppError> {
    match value {
        "pending" => Ok(TriggerLedgerStatus::Pending),
        "handled" => Ok(TriggerLedgerStatus::Handled),
        "rejected" => Ok(TriggerLedgerStatus::Rejected),
        "suppressed" => Ok(TriggerLedgerStatus::Suppressed),
        _ => Err(AppError::Message(format!(
            "unknown trigger ledger status: {value}"
        ))),
    }
}
