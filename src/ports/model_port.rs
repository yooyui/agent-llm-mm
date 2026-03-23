use async_trait::async_trait;

use crate::{domain::snapshot::SelfSnapshot, error::AppError};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModelDecisionRequest {
    pub task: String,
    pub action: String,
    pub snapshot: SelfSnapshot,
}

impl ModelDecisionRequest {
    pub fn new(task: String, action: String, snapshot: SelfSnapshot) -> Self {
        Self {
            task,
            action,
            snapshot,
        }
    }
}

pub type ModelInput = ModelDecisionRequest;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModelDecision {
    pub action: String,
    pub rationale: String,
}

impl ModelDecision {
    pub fn new(action: String, rationale: String) -> Self {
        Self { action, rationale }
    }
}

#[async_trait]
pub trait ModelPort {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError>;
}
