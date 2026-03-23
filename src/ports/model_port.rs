use async_trait::async_trait;

use crate::{domain::snapshot::SelfSnapshot, error::AppError};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModelInput {
    pub task: String,
    pub snapshot: SelfSnapshot,
    pub gate_blocked: bool,
}

impl ModelInput {
    pub fn new(task: String, snapshot: SelfSnapshot, gate_blocked: bool) -> Self {
        Self {
            task,
            snapshot,
            gate_blocked,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModelDecision {
    pub recommendation: String,
}

impl ModelDecision {
    pub fn new(recommendation: String) -> Self {
        Self { recommendation }
    }
}

#[async_trait]
pub trait ModelPort {
    async fn decide(&self, input: ModelInput) -> Result<ModelDecision, AppError>;
}
