use crate::domain::types::{EventKind, Owner};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub owner: Owner,
    pub kind: EventKind,
    pub summary: String,
}

impl Event {
    pub fn new(owner: Owner, kind: EventKind, summary: impl Into<String>) -> Self {
        Self {
            owner,
            kind,
            summary: summary.into(),
        }
    }
}
