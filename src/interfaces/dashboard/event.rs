use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

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

pub fn dashboard_started(base_url: &str, sequence: u64) -> OperationEvent {
    OperationEvent {
        id: Uuid::new_v4().to_string(),
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Startup,
        status: OperationStatus::Ok,
        operation: "dashboard_startup".to_string(),
        namespace: None,
        summary: format!("dashboard serving {base_url}"),
        correlation_id: None,
        payload: json!({ "base_url": base_url, "read_only": true }),
    }
}

pub fn tool_started(operation: &str, namespace: Option<String>, sequence: u64) -> OperationEvent {
    OperationEvent {
        id: Uuid::new_v4().to_string(),
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Tool,
        status: OperationStatus::Started,
        operation: operation.to_string(),
        namespace,
        summary: format!("{operation} started"),
        correlation_id: None,
        payload: json!({ "operation": operation }),
    }
}

pub fn tool_completed(
    operation_id: String,
    operation: &str,
    namespace: Option<String>,
    sequence: u64,
    summary: String,
    payload: Value,
) -> OperationEvent {
    OperationEvent {
        id: operation_id,
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Tool,
        status: OperationStatus::Ok,
        operation: operation.to_string(),
        namespace,
        summary,
        correlation_id: None,
        payload,
    }
}

pub fn tool_failed(
    operation: &str,
    namespace: Option<String>,
    sequence: u64,
    summary: String,
    error: String,
) -> OperationEvent {
    OperationEvent {
        id: Uuid::new_v4().to_string(),
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Error,
        status: OperationStatus::Failed,
        operation: operation.to_string(),
        namespace,
        summary,
        correlation_id: None,
        payload: json!({ "error": error }),
    }
}

pub fn auto_reflection_event(
    operation: &str,
    namespace: Option<String>,
    sequence: u64,
    status: OperationStatus,
    summary: String,
    payload: Value,
) -> OperationEvent {
    OperationEvent {
        id: Uuid::new_v4().to_string(),
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Reflection,
        status,
        operation: operation.to_string(),
        namespace,
        summary,
        correlation_id: None,
        payload,
    }
}
