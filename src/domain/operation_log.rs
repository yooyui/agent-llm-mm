use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationLogKind {
    Startup,
    Tool,
    Trigger,
    Reflection,
    Decision,
    Snapshot,
    Doctor,
    Error,
}

impl OperationLogKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Tool => "tool",
            Self::Trigger => "trigger",
            Self::Reflection => "reflection",
            Self::Decision => "decision",
            Self::Snapshot => "snapshot",
            Self::Doctor => "doctor",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationLogStatus {
    Started,
    Ok,
    Handled,
    Suppressed,
    Rejected,
    Failed,
}

impl OperationLogStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Ok => "ok",
            Self::Handled => "handled",
            Self::Suppressed => "suppressed",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    System,
    User,
    Model,
    Hook,
}

impl ActorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Model => "model",
            Self::Hook => "hook",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationLogEntry {
    pub operation_id: String,
    pub occurred_at: DateTime<Utc>,
    pub namespace: Option<String>,
    pub actor_kind: ActorKind,
    pub actor_id: String,
    pub entrypoint: String,
    pub operation_kind: OperationLogKind,
    pub status: OperationLogStatus,
    pub correlation_id: Option<String>,
    pub request_summary_json: Option<String>,
    pub response_summary_json: Option<String>,
    pub diagnostic_summary_json: Option<String>,
    pub redaction_version: i64,
}

const REDACTION_PATTERNS: &[&str] = &["api_key", "secret", "token", "bearer", "password"];

pub fn redact_secrets(json_text: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(json_text) else {
        return json_text.to_string();
    };
    redact_value(&mut value);
    serde_json::to_string(&value).unwrap_or_else(|_| json_text.to_string())
}

fn redact_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let lower = key.to_lowercase();
                if REDACTION_PATTERNS.iter().any(|p| lower.contains(p)) {
                    if val.is_string() {
                        *val = serde_json::Value::String("[REDACTED]".to_string());
                    }
                } else {
                    redact_value(val);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                redact_value(item);
            }
        }
        _ => {}
    }
}
