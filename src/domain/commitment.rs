use crate::domain::types::Owner;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Commitment {
    pub owner: Owner,
    pub description: String,
}

impl Commitment {
    pub fn new(owner: Owner, description: impl Into<String>) -> Self {
        Self {
            owner,
            description: description.into(),
        }
    }
}
