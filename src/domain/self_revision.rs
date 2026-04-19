use crate::domain::{snapshot::SelfSnapshot, types::Namespace};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerType {
    Conflict,
    Failure,
    Periodic,
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
}

impl SelfRevisionProposal {
    pub fn no_revision(rationale: impl Into<String>) -> Self {
        Self {
            should_reflect: false,
            rationale: rationale.into(),
            machine_patch: SelfRevisionPatch::default(),
        }
    }
}
