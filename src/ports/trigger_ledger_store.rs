use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    domain::{self_revision::TriggerType, types::Namespace},
    error::AppError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerLedgerStatus {
    Pending,
    Handled,
    Rejected,
    Suppressed,
}

impl TriggerLedgerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Handled => "handled",
            Self::Rejected => "rejected",
            Self::Suppressed => "suppressed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredTriggerLedgerEntry {
    pub ledger_id: String,
    pub trigger_type: TriggerType,
    pub namespace: Namespace,
    /// Canonical dedupe/lookup key for this trigger attempt.
    ///
    /// Callers are responsible for encoding any trigger-type or namespace
    /// partitioning they need into this key.
    pub trigger_key: String,
    pub status: TriggerLedgerStatus,
    pub evidence_window: Vec<String>,
    pub handled_at: Option<DateTime<Utc>>,
    pub cooldown_until: Option<DateTime<Utc>>,
    pub episode_watermark: Option<u64>,
    pub reflection_id: Option<String>,
}

impl StoredTriggerLedgerEntry {
    pub fn new(
        ledger_id: impl Into<String>,
        trigger_type: TriggerType,
        namespace: Namespace,
        trigger_key: impl Into<String>,
        status: TriggerLedgerStatus,
    ) -> Self {
        Self {
            ledger_id: ledger_id.into(),
            trigger_type,
            namespace,
            trigger_key: trigger_key.into(),
            status,
            evidence_window: Vec::new(),
            handled_at: None,
            cooldown_until: None,
            episode_watermark: None,
            reflection_id: None,
        }
    }
}

#[async_trait]
pub trait TriggerLedgerStore {
    async fn record_trigger_attempt(&self, entry: StoredTriggerLedgerEntry)
    -> Result<(), AppError>;

    /// Returns the most recently recorded attempt for `trigger_key`.
    ///
    /// "Latest" means last appended ledger row for this canonical key, not the
    /// maximum `handled_at` business timestamp.
    async fn latest_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError>;

    /// Returns the most recently recorded handled attempt for `trigger_key`.
    async fn latest_handled_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError>;
}
