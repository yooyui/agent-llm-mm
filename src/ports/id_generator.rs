use async_trait::async_trait;

use crate::error::AppError;

#[async_trait]
pub trait IdGenerator {
    async fn next_id(&self) -> Result<String, AppError>;
}
