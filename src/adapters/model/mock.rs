use async_trait::async_trait;

use crate::{
    error::AppError,
    ports::{ModelDecision, ModelDecisionRequest, ModelPort},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct MockModel;

#[async_trait]
impl ModelPort for MockModel {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        let action = if request.snapshot.claims.is_empty() {
            "request_more_context"
        } else {
            "summarize_memory_state"
        };

        Ok(ModelDecision::new(
            action.to_string(),
            "mocked".to_string(),
        ))
    }
}
