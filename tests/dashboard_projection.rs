use agent_llm_mm::interfaces::dashboard::{
    DashboardRuntimeInfo, OperationEvent, OperationKind, OperationStatus, build_summary,
    project_event_detail,
};
use chrono::Utc;
use serde_json::json;

fn event(sequence: u64, kind: OperationKind, status: OperationStatus) -> OperationEvent {
    OperationEvent {
        id: format!("op_{sequence}"),
        sequence,
        timestamp: Utc::now(),
        kind,
        status,
        operation: "run_reflection".to_string(),
        namespace: Some("self".to_string()),
        summary: "reflection handled".to_string(),
        correlation_id: Some("corr-1".to_string()),
        payload: json!({ "reflection_id": "reflection-1" }),
    }
}

#[test]
fn summary_counts_events_by_kind_and_status() {
    let events = vec![
        event(1, OperationKind::Tool, OperationStatus::Ok),
        event(2, OperationKind::Reflection, OperationStatus::Handled),
        event(3, OperationKind::Reflection, OperationStatus::Rejected),
    ];
    let runtime = DashboardRuntimeInfo {
        service_name: "agent-llm-mm".to_string(),
        transport: "stdio".to_string(),
        provider: "mock".to_string(),
        dashboard_enabled: true,
        read_only: true,
    };

    let summary = build_summary(&events, &runtime);

    assert_eq!(summary.total_events, 3);
    assert_eq!(summary.reflection_events, 2);
    assert_eq!(summary.failed_events, 0);
    assert_eq!(summary.runtime.service_name, "agent-llm-mm");
}

#[test]
fn detail_projection_preserves_payload_and_read_only_boundary() {
    let detail = project_event_detail(&event(
        7,
        OperationKind::Reflection,
        OperationStatus::Handled,
    ));

    assert_eq!(detail.id, "op_7");
    assert_eq!(detail.operation, "run_reflection");
    assert!(detail.read_only);
    assert_eq!(detail.payload["reflection_id"], "reflection-1");
}
