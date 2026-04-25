use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Startup,
    Tool,
    Trigger,
    Reflection,
    Decision,
    Snapshot,
    Doctor,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Started,
    Ok,
    Handled,
    Suppressed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationEvent {
    pub id: String,
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub operation: String,
    pub namespace: Option<String>,
    pub summary: String,
    pub correlation_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventQuery {
    pub limit: Option<usize>,
    pub kind: Option<OperationKind>,
    pub status: Option<OperationStatus>,
    pub namespace: Option<String>,
}
