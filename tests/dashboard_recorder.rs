use agent_llm_mm::interfaces::dashboard::{
    EventQuery, OperationEvent, OperationKind, OperationRecorder, OperationStatus,
};
use chrono::Utc;
use serde_json::json;

fn event(
    sequence: u64,
    kind: OperationKind,
    status: OperationStatus,
    namespace: Option<&str>,
) -> OperationEvent {
    OperationEvent {
        id: format!("op_{sequence}"),
        sequence,
        timestamp: Utc::now(),
        kind,
        status,
        operation: "ingest_interaction".to_string(),
        namespace: namespace.map(str::to_string),
        summary: format!("event {sequence}"),
        correlation_id: None,
        payload: json!({ "sequence": sequence }),
    }
}

#[test]
fn recorder_keeps_sequence_order_and_drops_oldest_after_capacity() {
    let recorder = OperationRecorder::new(2);

    recorder.append(event(
        1,
        OperationKind::Tool,
        OperationStatus::Started,
        Some("self"),
    ));
    recorder.append(event(
        2,
        OperationKind::Tool,
        OperationStatus::Ok,
        Some("self"),
    ));
    recorder.append(event(
        3,
        OperationKind::Reflection,
        OperationStatus::Handled,
        Some("self"),
    ));

    let events = recorder.recent(EventQuery::default());

    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
}

#[test]
fn recorder_filters_by_kind_status_and_namespace() {
    let recorder = OperationRecorder::new(5);
    recorder.append(event(
        1,
        OperationKind::Tool,
        OperationStatus::Ok,
        Some("self"),
    ));
    recorder.append(event(
        2,
        OperationKind::Reflection,
        OperationStatus::Handled,
        Some("self"),
    ));
    recorder.append(event(
        3,
        OperationKind::Reflection,
        OperationStatus::Rejected,
        Some("project/demo"),
    ));

    let query = EventQuery {
        limit: Some(10),
        kind: Some(OperationKind::Reflection),
        status: Some(OperationStatus::Handled),
        namespace: Some("self".to_string()),
    };

    let events = recorder.recent(query);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sequence, 2);
}
