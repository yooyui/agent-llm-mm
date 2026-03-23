use async_trait::async_trait;

use crate::error::AppError;

#[async_trait]
pub trait EpisodeStore {
    async fn record_event_in_episode(
        &self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError>;

    async fn list_episode_references(&self) -> Result<Vec<String>, AppError>;
}
