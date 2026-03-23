use crate::domain::types::Owner;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Commitment {
    owner: Owner,
    description: String,
}

impl Commitment {
    pub fn new(owner: Owner, description: impl Into<String>) -> Self {
        Self {
            owner,
            description: description.into(),
        }
    }

    pub fn owner(&self) -> Owner {
        self.owner
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}
