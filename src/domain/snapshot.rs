use crate::domain::claim::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotBudget {
    limit: usize,
}

impl SnapshotBudget {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    pub fn max(&self, minimum: usize) -> usize {
        self.limit.max(minimum)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotRequest {
    pub identity: Vec<String>,
    pub commitments: Vec<String>,
    pub claims: Vec<String>,
    pub evidence: Vec<String>,
    pub episodes: Vec<String>,
    pub budget: SnapshotBudget,
}

impl SnapshotRequest {
    pub fn fixture_minimal() -> Self {
        Self {
            identity: vec!["identity:self=architect".to_string()],
            commitments: vec!["forbid:write_identity_core_directly".to_string()],
            claims: vec!["claim:self.role=architect".to_string()],
            evidence: vec!["event:evt-minimal".to_string()],
            episodes: vec!["episode:minimal".to_string()],
            budget: SnapshotBudget::new(1),
        }
    }

    pub fn validate(self) -> Result<Self, DomainError> {
        if self.evidence.is_empty() {
            return Err(DomainError::SnapshotNeedsEvidence);
        }

        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelfSnapshot {
    pub identity: Vec<String>,
    pub commitments: Vec<String>,
    pub claims: Vec<String>,
    pub evidence: Vec<String>,
    pub episodes: Vec<String>,
}
