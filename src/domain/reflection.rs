use crate::domain::commitment::Commitment;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Reflection {
    summary: String,
}

impl Reflection {
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
        }
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReflectionIdentityUpdate {
    pub canonical_claims: Vec<String>,
}

impl ReflectionIdentityUpdate {
    pub fn new(canonical_claims: Vec<String>) -> Self {
        Self { canonical_claims }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct ReflectionUpdates {
    pub identity: Option<ReflectionIdentityUpdate>,
    pub commitments: Option<Vec<Commitment>>,
}
