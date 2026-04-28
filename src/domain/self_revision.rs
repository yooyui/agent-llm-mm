use crate::{
    domain::{snapshot::SelfSnapshot, types::Namespace},
    ports::EvidenceQuery,
};
use chrono::{DateTime, Utc};

pub const SELF_REVISION_DURABLE_WRITE_PATH: &str = "run_reflection";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    #[serde(alias = "Conflict")]
    Conflict,
    #[serde(alias = "Failure")]
    Failure,
    #[serde(alias = "Periodic")]
    Periodic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoReflectOutcome {
    #[serde(alias = "Handled")]
    Handled,
    #[serde(alias = "Rejected")]
    Rejected,
    #[serde(alias = "Suppressed")]
    Suppressed,
    #[serde(alias = "NotTriggered")]
    NotTriggered,
    #[serde(alias = "Skipped")]
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AutoReflectDiagnosticSummary {
    pub trigger_type: TriggerType,
    pub outcome: AutoReflectOutcome,
    pub suppression_reason: Option<String>,
    pub rejection_reason: Option<String>,
    pub cooldown_boundary: Option<DateTime<Utc>>,
    pub evidence_window_size: usize,
    pub selected_evidence_event_ids: Vec<String>,
    pub durable_write_path: String,
}

impl AutoReflectDiagnosticSummary {
    pub fn new(
        trigger_type: TriggerType,
        outcome: AutoReflectOutcome,
        suppression_reason: Option<String>,
        rejection_reason: Option<String>,
        cooldown_boundary: Option<DateTime<Utc>>,
        evidence_window_size: usize,
        selected_evidence_event_ids: Vec<String>,
    ) -> Self {
        Self {
            trigger_type,
            outcome,
            suppression_reason,
            rejection_reason,
            cooldown_boundary,
            evidence_window_size,
            selected_evidence_event_ids,
            durable_write_path: SELF_REVISION_DURABLE_WRITE_PATH.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionRequest {
    pub trigger_type: TriggerType,
    pub namespace: Namespace,
    pub snapshot: SelfSnapshot,
    pub evidence_event_ids: Vec<String>,
    pub trigger_hints: Vec<String>,
}

impl SelfRevisionRequest {
    pub fn new(
        trigger_type: TriggerType,
        namespace: Namespace,
        snapshot: SelfSnapshot,
        evidence_event_ids: Vec<String>,
        trigger_hints: Vec<String>,
    ) -> Self {
        Self {
            trigger_type,
            namespace,
            snapshot,
            evidence_event_ids,
            trigger_hints,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SelfRevisionPatch {
    pub identity_patch: Option<SelfRevisionIdentityPatch>,
    pub commitment_patch: Option<SelfRevisionCommitmentPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionIdentityPatch {
    pub canonical_claims: Vec<String>,
}

impl SelfRevisionIdentityPatch {
    pub fn new(canonical_claims: Vec<String>) -> Self {
        Self { canonical_claims }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionCommitmentPatch {
    pub commitments: Vec<String>,
}

impl SelfRevisionCommitmentPatch {
    pub fn new(commitments: Vec<String>) -> Self {
        Self { commitments }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfRevisionProposal {
    pub should_reflect: bool,
    pub rationale: String,
    #[serde(default)]
    pub machine_patch: SelfRevisionPatch,
    #[serde(default)]
    pub proposed_evidence_event_ids: Vec<String>,
    #[serde(default)]
    pub proposed_evidence_query: Option<EvidenceQuery>,
    #[serde(default)]
    pub confidence: Option<String>,
}

impl SelfRevisionProposal {
    pub fn no_revision(rationale: impl Into<String>) -> Self {
        Self {
            should_reflect: false,
            rationale: rationale.into(),
            machine_patch: SelfRevisionPatch::default(),
            proposed_evidence_event_ids: Vec::new(),
            proposed_evidence_query: None,
            confidence: None,
        }
    }
}
