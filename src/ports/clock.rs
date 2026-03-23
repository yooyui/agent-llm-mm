use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AppError;

#[async_trait]
pub trait Clock {
    async fn now(&self) -> Result<DateTime<Utc>, AppError>;
}
