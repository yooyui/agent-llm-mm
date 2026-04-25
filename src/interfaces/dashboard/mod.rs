use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

pub mod assets;
pub mod event;
pub mod http;
pub mod projection;
pub mod recorder;

pub use event::{EventQuery, OperationEvent, OperationKind, OperationStatus};
pub use http::{DashboardHandle, start_dashboard_service};
pub use projection::{
    DashboardRuntimeInfo, DashboardSummary, OperationDetail, build_summary, project_event_detail,
};
pub use recorder::OperationRecorder;

#[derive(Debug, Clone)]
pub enum DashboardObserver {
    Disabled,
    Enabled {
        recorder: OperationRecorder,
        sequence: Arc<AtomicU64>,
    },
}

impl DashboardObserver {
    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn enabled(recorder: OperationRecorder) -> Self {
        Self::Enabled {
            recorder,
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn recorder(&self) -> Option<OperationRecorder> {
        match self {
            Self::Disabled => None,
            Self::Enabled { recorder, .. } => Some(recorder.clone()),
        }
    }

    pub fn next_sequence(&self) -> u64 {
        match self {
            Self::Disabled => 0,
            Self::Enabled { sequence, .. } => sequence.fetch_add(1, Ordering::Relaxed) + 1,
        }
    }

    pub fn record_event(&self, event: OperationEvent) {
        if let Self::Enabled { recorder, .. } = self {
            recorder.append(event);
        }
    }

    pub fn record_dashboard_started(&self, base_url: &str) {
        let sequence = self.next_sequence();
        if sequence == 0 {
            return;
        }
        self.record_event(event::dashboard_started(base_url, sequence));
    }

    pub fn record_tool_ok<T: Serialize>(
        &self,
        operation: &str,
        namespace: Option<String>,
        summary: String,
        payload: &T,
    ) {
        let sequence = self.next_sequence();
        if sequence == 0 {
            return;
        }
        self.record_event(event::tool_completed(
            Uuid::new_v4().to_string(),
            operation,
            namespace,
            sequence,
            summary,
            serde_json::to_value(payload).unwrap_or_else(|_| json!({ "serialization": "failed" })),
        ));
    }

    pub fn record_tool_failed(
        &self,
        operation: &str,
        namespace: Option<String>,
        summary: String,
        error: String,
    ) {
        let sequence = self.next_sequence();
        if sequence == 0 {
            return;
        }
        self.record_event(event::tool_failed(
            operation, namespace, sequence, summary, error,
        ));
    }

    pub fn record_auto_reflection<T: Serialize>(
        &self,
        operation: &str,
        namespace: Option<String>,
        status: OperationStatus,
        summary: String,
        payload: &T,
    ) {
        let sequence = self.next_sequence();
        if sequence == 0 {
            return;
        }
        self.record_event(event::auto_reflection_event(
            operation,
            namespace,
            sequence,
            status,
            summary,
            serde_json::to_value(payload).unwrap_or_else(|_| json!({ "serialization": "failed" })),
        ));
    }
}
