use crate::domain::{
    DomainError,
    types::{EventKind, Namespace, Owner},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    owner: Owner,
    namespace: Namespace,
    kind: EventKind,
    summary: String,
}

impl Event {
    pub fn new(owner: Owner, kind: EventKind, summary: impl Into<String>) -> Self {
        Self {
            owner,
            namespace: Namespace::for_owner(owner),
            kind,
            summary: summary.into(),
        }
    }

    pub fn new_with_namespace(
        owner: Owner,
        namespace: Namespace,
        kind: EventKind,
        summary: impl Into<String>,
    ) -> Result<Self, DomainError> {
        if !namespace.matches_owner(owner) {
            return Err(DomainError::NamespaceOwnerMismatch);
        }

        Ok(Self {
            owner,
            namespace,
            kind,
            summary: summary.into(),
        })
    }

    pub fn owner(&self) -> Owner {
        self.owner
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub fn kind(&self) -> EventKind {
        self.kind
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}
