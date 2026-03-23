use std::str::FromStr;

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
        types::{EventKind, Mode, Owner},
    },
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, CommitmentStore, EpisodeStore, EventStore, IdentityStore,
        IngestTransaction, IngestTransactionRunner, ReflectionStore, ReflectionTransaction,
        ReflectionTransactionRunner, StoredClaim, StoredEvent, StoredReflection,
    },
};

use super::schema::INIT_SQL;

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn bootstrap(database_url: &str) -> Result<Self, AppError> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(|error| AppError::Message(error.to_string()))?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = map_sqlite(SqlitePool::connect_with(options).await)?;

        for statement in INIT_SQL.split(';').filter(|part| !part.trim().is_empty()) {
            map_sqlite(sqlx::query(statement).execute(&pool).await)?;
        }

        Ok(Self { pool })
    }
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
impl IdentityStore for SqliteStore {
    async fn load_identity(&self) -> Result<IdentityCore, AppError> {
        let rows = map_sqlite(
            sqlx::query(
                r#"
                SELECT claim
                FROM identity_claims
                ORDER BY position
                "#,
            )
            .fetch_all(&self.pool)
            .await,
        )?;

        if rows.is_empty() {
            return Err(AppError::Message("missing identity".to_string()));
        }

        Ok(IdentityCore::new(
            rows.into_iter().map(|row| row.get("claim")).collect(),
        ))
    }

    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError> {
        let mut tx = map_sqlite(self.pool.begin().await)?;
        map_sqlite(
            sqlx::query("DELETE FROM identity_claims")
                .execute(tx.as_mut())
                .await,
        )?;

        for (position, claim) in identity.canonical_claims().iter().enumerate() {
            map_sqlite(
                sqlx::query("INSERT INTO identity_claims (position, claim) VALUES (?, ?)")
                    .bind(position as i64)
                    .bind(claim)
                    .execute(tx.as_mut())
                    .await,
            )?;
        }

        map_sqlite(tx.commit().await)?;
        Ok(())
    }
}

#[async_trait]
impl CommitmentStore for SqliteStore {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, AppError> {
        let rows = map_sqlite(
            sqlx::query(
                r#"
                SELECT owner, description
                FROM commitments
                ORDER BY rowid
                "#,
            )
            .fetch_all(&self.pool)
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
    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO claims (claim_id, owner, subject, predicate, object, mode, status)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(claim_id) DO UPDATE SET
                owner = excluded.owner,
                subject = excluded.subject,
                predicate = excluded.predicate,
                object = excluded.object,
                mode = excluded.mode,
                status = excluded.status
            "#,
        )
        .bind(&claim.claim_id)
        .bind(owner_as_str(claim.claim.owner()))
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
    map_sqlite(
        sqlx::query(
            r#"
            INSERT INTO reflections (
                reflection_id,
                recorded_at,
                summary,
                superseded_claim_id,
                replacement_claim_id
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&reflection.reflection_id)
        .bind(reflection.recorded_at.to_rfc3339())
        .bind(reflection.reflection.summary())
        .bind(&reflection.superseded_claim_id)
        .bind(&reflection.replacement_claim_id)
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

fn map_sqlite<T>(result: Result<T, sqlx::Error>) -> Result<T, AppError> {
    result.map_err(|error| AppError::Message(error.to_string()))
}

fn stored_claim_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<StoredClaim, AppError> {
    Ok(StoredClaim::new(
        row.get("claim_id"),
        ClaimDraft::new(
            parse_owner(&row.get::<String, _>("owner"))?,
            row.get::<String, _>("subject"),
            row.get::<String, _>("predicate"),
            row.get::<String, _>("object"),
            parse_mode(&row.get::<String, _>("mode"))?,
        ),
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

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| AppError::Message(error.to_string()))
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
        "superseded" => Ok(ClaimStatus::Superseded),
        _ => Err(AppError::Message(format!("unknown claim status: {value}"))),
    }
}
