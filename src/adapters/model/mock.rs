use async_trait::async_trait;

use crate::{
    domain::self_revision::{SelfRevisionProposal, SelfRevisionRequest},
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

        Ok(ModelDecision::new(action.to_string()))
    }

    async fn propose_self_revision(
        &self,
        request: SelfRevisionRequest,
    ) -> Result<SelfRevisionProposal, AppError> {
        Ok(SelfRevisionProposal::no_revision(format!(
            "mock model did not detect a valid {:?} revision",
            request.trigger_type
        )))
    }
}
