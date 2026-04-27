use serde::Serialize;
use serde_json::Value;

use super::{OperationEvent, OperationKind, OperationStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DashboardRuntimeInfo {
    pub service_name: String,
    pub transport: String,
    pub provider: String,
    pub dashboard_enabled: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DashboardSummary {
    pub runtime: DashboardRuntimeInfo,
    pub total_events: usize,
    pub tool_events: usize,
    pub reflection_events: usize,
    pub decision_events: usize,
    pub snapshot_events: usize,
    pub failed_events: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OperationDetail {
    pub id: String,
    pub operation: String,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub namespace: Option<String>,
    pub summary: String,
    pub payload: Value,
    pub read_only: bool,
}

pub fn build_summary(
    events: &[OperationEvent],
    runtime: &DashboardRuntimeInfo,
) -> DashboardSummary {
    DashboardSummary {
        runtime: runtime.clone(),
        total_events: events.len(),
        tool_events: count_kind(events, OperationKind::Tool),
        reflection_events: count_kind(events, OperationKind::Reflection),
        decision_events: count_kind(events, OperationKind::Decision),
        snapshot_events: count_kind(events, OperationKind::Snapshot),
        failed_events: events
            .iter()
            .filter(|event| event.status == OperationStatus::Failed)
            .count(),
    }
}

pub fn project_event_detail(event: &OperationEvent) -> OperationDetail {
    OperationDetail {
        id: event.id.clone(),
        operation: event.operation.clone(),
        kind: event.kind,
        status: event.status,
        namespace: event.namespace.clone(),
        summary: event.summary.clone(),
        payload: event.payload.clone(),
        read_only: true,
    }
}

fn count_kind(events: &[OperationEvent], kind: OperationKind) -> usize {
    events.iter().filter(|event| event.kind == kind).count()
}
