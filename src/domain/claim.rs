use crate::domain::{DomainError, types::{Mode, Owner}};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimDraft {
    owner: Owner,
    subject: String,
    predicate: String,
    object: String,
    mode: Mode,
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

    pub fn owner(&self) -> Owner {
        self.owner
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn predicate(&self) -> &str {
        &self.predicate
    }

    pub fn object(&self) -> &str {
        &self.object
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}
