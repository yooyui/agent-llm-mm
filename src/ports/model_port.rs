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
}

impl ModelDecision {
    pub fn new(action: String) -> Self {
        Self { action }
    }
}

#[async_trait]
pub trait ModelPort {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError>;
}
