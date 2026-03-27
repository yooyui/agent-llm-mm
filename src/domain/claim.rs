use crate::domain::{
    DomainError,
    types::{Mode, Namespace, Owner},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimDraft {
    owner: Owner,
    namespace: Namespace,
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
        Self::new_with_namespace(
            owner,
            Namespace::for_owner(owner),
            subject,
            predicate,
            object,
            mode,
        )
    }

    pub fn new_with_namespace(
        owner: Owner,
        namespace: Namespace,
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
        mode: Mode,
    ) -> Self {
        Self {
            owner,
            namespace,
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

    pub fn with_namespace(mut self, namespace: Namespace) -> Self {
        self.namespace = namespace;
        self
    }

    pub fn validate(&self, evidence_count: usize) -> Result<(), DomainError> {
        if self.mode == Mode::Inferred && evidence_count == 0 {
            return Err(DomainError::InsufficientEvidence);
        }

        self.validate_namespace_owner()?;

        Ok(())
    }

    pub fn validate_namespace_owner(&self) -> Result<(), DomainError> {
        if !self.namespace.matches_owner(self.owner) {
            return Err(DomainError::NamespaceOwnerMismatch);
        }

        Ok(())
    }

    pub fn owner(&self) -> Owner {
        self.owner
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
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
