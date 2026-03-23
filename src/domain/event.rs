use crate::domain::types::{EventKind, Owner};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    owner: Owner,
    kind: EventKind,
    summary: String,
}

impl Event {
    pub fn new(owner: Owner, kind: EventKind, summary: impl Into<String>) -> Self {
        Self {
            owner,
            kind,
            summary: summary.into(),
        }
    }

    pub fn owner(&self) -> Owner {
        self.owner
    }

    pub fn kind(&self) -> EventKind {
        self.kind
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}
