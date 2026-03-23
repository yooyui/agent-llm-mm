use async_trait::async_trait;

use crate::{domain::commitment::Commitment, error::AppError};

#[async_trait]
pub trait CommitmentStore {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, AppError>;
}
