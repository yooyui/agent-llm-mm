use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{domain::reflection::Reflection, error::AppError};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredReflection {
    pub reflection_id: String,
    pub recorded_at: DateTime<Utc>,
    pub reflection: Reflection,
    pub superseded_claim_id: Option<String>,
    pub replacement_claim_id: Option<String>,
}

impl StoredReflection {
    pub fn new(
        reflection_id: String,
        recorded_at: DateTime<Utc>,
        reflection: Reflection,
        superseded_claim_id: Option<String>,
        replacement_claim_id: Option<String>,
    ) -> Self {
        Self {
            reflection_id,
            recorded_at,
            reflection,
            superseded_claim_id,
            replacement_claim_id,
        }
    }
}

#[async_trait]
pub trait ReflectionStore {
    async fn append_reflection(&self, reflection: StoredReflection) -> Result<(), AppError>;
}
