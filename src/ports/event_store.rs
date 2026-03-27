use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{domain::event::Event, error::AppError};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredEvent {
    pub event_id: String,
    pub recorded_at: DateTime<Utc>,
    pub event: Event,
}

impl StoredEvent {
    pub fn new(event_id: String, recorded_at: DateTime<Utc>, event: Event) -> Self {
        Self {
            event_id,
            recorded_at,
            event,
        }
    }

    pub fn event_reference(&self) -> String {
        format!("event:{}", self.event_id)
    }
}

#[async_trait]
pub trait EventStore {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError>;
    async fn list_event_references(&self) -> Result<Vec<String>, AppError>;
    async fn has_event(&self, event_id: &str) -> Result<bool, AppError>;
}
