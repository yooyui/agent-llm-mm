use crate::domain::types::{Mode, Owner};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DomainError {
    InsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimDraft {
    pub owner: Owner,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub mode: Mode,
}

impl ClaimDraft {
    pub fn new(
        owner: Owner,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
        mode: Mode,
    ) -> Self {
        Self {
            owner,
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            mode,
        }
    }

    pub fn new_inferred(
        owner: Owner,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self::new(owner, subject, predicate, object, Mode::Inferred)
    }

    pub fn validate(&self, evidence_count: usize) -> Result<(), DomainError> {
        if self.mode == Mode::Inferred && evidence_count == 0 {
            return Err(DomainError::InsufficientEvidence);
        }

        Ok(())
    }
}
