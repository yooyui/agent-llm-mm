use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{domain::operation_log::OperationLogEntry, error::AppError};

#[derive(Debug, Clone, Default)]
pub struct OperationLogQuery {
    pub operation_id: Option<String>,
    pub namespace: Option<String>,
    pub operation_kind: Option<String>,
    pub status: Option<String>,
    pub correlation_id: Option<String>,
    pub limit: Option<usize>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait OperationLogStore {
    async fn append_operation(&self, entry: OperationLogEntry) -> Result<(), AppError>;
    async fn query_operations(
        &self,
        query: OperationLogQuery,
    ) -> Result<Vec<OperationLogEntry>, AppError>;
}
